import { useCallback, useEffect, useRef, useState } from 'react';
import type { Room } from 'livekit-client';
import { fetchGrant, type GrantResult } from './tokenApi';

/**
 * Совместный просмотр: разговор поверх эфира.
 *
 * ⚠️ SDK ГРУЗИТСЯ ДИНАМИЧЕСКИ И ТОЛЬКО ОТСЮДА. `livekit-client` весит около
 * полумегабайта; зритель, который просто смотрит канал, не должен платить за
 * него ни байтом. `import()` внутри `connect` оставляет его отдельным куском,
 * который не загрузится, пока никто не нажал «смотреть вместе».
 *
 * ⚠️ ЭЛЕМЕНТЫ <video> И <audio> СТРОИТ SDK, А НЕ REACT. Он знает и кодек, и
 * флаги автовоспроизведения, без которых телефон молча откажется играть
 * встроенное видео, и момент, когда дорожка умерла. Рисовать `<video>` в JSX
 * и подсовывать ему MediaStream значило бы перенести всё это сюда — и дать
 * перерисовке подменить элемент под живой дорожкой. Поэтому: append, никогда
 * не clone и никогда не `src`.
 *
 * ⚠️ ПОДПИСАННАЯ ДОРОЖКА — ЕЩЁ НЕ ЗВУЧАЩАЯ. LiveKit отдаёт звук и на этом
 * останавливается: `attach()` строит элемент, а положить его в документ —
 * наше дело. Пропустить этот шаг — это ровно то, что снаружи выглядит как
 * «подключились, но никого не слышно».
 */

export interface Peer {
  identity: string;
  name: string;
  speaking: boolean;
}

export interface Feed {
  /** Ключ ДОРОЖКИ, а не участника: у одного участника их может быть больше. */
  key: string;
  element: HTMLVideoElement;
  self: boolean;
}

/** Что смотрит комната. Приходит сообщением от того, кто переключил канал. */
export interface Watching {
  url: string;
  title: string;
  by: string;
}

export type TogetherState = 'idle' | 'connecting' | 'connected';
export type TogetherError = GrantResult['status'] | 'sdk_failed' | 'join_failed' | null;

interface Api {
  state: TogetherState;
  error: TogetherError;
  room: string | null;
  peers: Peer[];
  feeds: Feed[];
  micOn: boolean;
  camOn: boolean;
  watching: Watching | null;
  connect: (room: string, name: string) => Promise<void>;
  leave: () => void;
  toggleMic: () => void;
  toggleCam: () => void;
  announce: (url: string, title: string) => void;
}

/** Что публикует камера. Маленькое намеренно — см. комментарий в панели. */
const CAMERA_CAPTURE = { resolution: { width: 320, height: 240, frameRate: 15 } };
const CAMERA_PUBLISH = {
  simulcast: false,
  videoEncoding: { maxBitrate: 150_000, maxFramerate: 15 },
};

export function useTogether(sink: React.RefObject<HTMLElement>): Api {
  const [state, setState] = useState<TogetherState>('idle');
  const [error, setError] = useState<TogetherError>(null);
  const [roomCode, setRoomCode] = useState<string | null>(null);
  const [peers, setPeers] = useState<Peer[]>([]);
  const [feeds, setFeeds] = useState<Feed[]>([]);
  const [micOn, setMicOn] = useState(false);
  const [camOn, setCamOn] = useState(false);
  const [watching, setWatching] = useState<Watching | null>(null);
  const roomRef = useRef<Room | null>(null);
  const selfName = useRef('');

  // Уход со страницы обязан закрыть комнату. Иначе участник остаётся в ней
  // призраком: остальные видят имя, из которого никогда не раздастся звук.
  useEffect(() => () => { roomRef.current?.disconnect(); }, []);

  const connect = useCallback(async (code: string, name: string) => {
    if (roomRef.current) return;
    setState('connecting');
    setError(null);
    selfName.current = name;

    const grant = await fetchGrant(code, name);
    if (grant.status !== 'ok') {
      setState('idle');
      setError(grant.status);
      return;
    }

    let LK: typeof import('livekit-client');
    try {
      LK = await import('livekit-client');
    } catch {
      // ⚠️ ОТДЕЛЬНАЯ ПРИЧИНА, А НЕ «НЕ ПОДКЛЮЧИЛОСЬ». Пропуск выдан, и не
      // загрузился НАШ кусок кода: чинится пересборкой, а не настройками
      // LiveKit. Свалить это в общий отказ значит отправить искать поломку
      // не туда.
      setState('idle');
      setError('sdk_failed');
      return;
    }

    const room = new LK.Room({ adaptiveStream: false, dynacast: true });
    roomRef.current = room;

    const refreshPeers = () => {
      const others = [...room.remoteParticipants.values()].map((p) => ({
        identity: p.identity,
        name: p.name || p.identity,
        speaking: p.isSpeaking,
      }));
      setPeers(others);
    };

    const tracks = new Map<string, Feed>();
    const publishFeeds = () => setFeeds([...tracks.values()]);

    room.on(LK.RoomEvent.ParticipantConnected, refreshPeers);
    room.on(LK.RoomEvent.ParticipantDisconnected, refreshPeers);
    room.on(LK.RoomEvent.ActiveSpeakersChanged, refreshPeers);

    room.on(LK.RoomEvent.TrackSubscribed, (track) => {
      if (track.kind === LK.Track.Kind.Audio) {
        sink.current?.appendChild(track.attach());
        return;
      }
      if (track.kind !== LK.Track.Kind.Video) return;
      if (track.source !== LK.Track.Source.Camera) return;
      const element = track.attach() as HTMLVideoElement;
      // Телефон откажется играть встроенное видео без этих трёх — и откажется
      // МОЛЧА: элемент стоит в разметке, показывая первый кадр или ничего.
      element.autoplay = true;
      element.playsInline = true;
      element.muted = true;
      tracks.set(track.sid ?? String(tracks.size), { key: track.sid ?? String(tracks.size), element, self: false });
      publishFeeds();
    });

    room.on(LK.RoomEvent.TrackUnsubscribed, (track) => {
      track.detach().forEach((el) => el.remove());
      if (track.sid) tracks.delete(track.sid);
      publishFeeds();
    });

    // Что смотрит комната. Сообщение — просто подсказка: канал у зрителя
    // НЕ переключается сам, ему предлагают перейти. Выдернуть картинку
    // из-под человека, который смотрит, — это не «вместе», это помеха.
    room.on(LK.RoomEvent.DataReceived, (payload, participant) => {
      try {
        const msg = JSON.parse(new TextDecoder().decode(payload)) as Record<string, unknown>;
        if (msg.t !== 'watching' || typeof msg.url !== 'string') return;
        setWatching({
          url: msg.url,
          title: typeof msg.title === 'string' ? msg.title : '',
          by: participant?.name || participant?.identity || '',
        });
      } catch {
        // Чужое сообщение не нашего вида. Молчим: ронять разговор из-за
        // неразобранного байта нельзя.
      }
    });

    room.on(LK.RoomEvent.Disconnected, () => {
      roomRef.current = null;
      setState('idle');
      setPeers([]);
      setFeeds([]);
      setWatching(null);
      setMicOn(false);
      setCamOn(false);
    });

    try {
      await room.connect(grant.url, grant.token);
      await room.localParticipant.setMicrophoneEnabled(true);
    } catch {
      roomRef.current = null;
      await room.disconnect().catch(() => {});
      setState('idle');
      setError('join_failed');
      return;
    }

    setMicOn(true);
    setRoomCode(grant.room);
    setState('connected');
    refreshPeers();
  }, [sink]);

  const leave = useCallback(() => {
    void roomRef.current?.disconnect();
    roomRef.current = null;
    setState('idle');
    setRoomCode(null);
  }, []);

  const toggleMic = useCallback(() => {
    const room = roomRef.current;
    if (!room) return;
    const next = !room.localParticipant.isMicrophoneEnabled;
    void room.localParticipant.setMicrophoneEnabled(next).then(() => setMicOn(next));
  }, []);

  const toggleCam = useCallback(() => {
    const room = roomRef.current;
    if (!room) return;
    const next = !room.localParticipant.isCameraEnabled;
    void room.localParticipant
      .setCameraEnabled(next, CAMERA_CAPTURE, CAMERA_PUBLISH)
      .then(() => setCamOn(next));
  }, []);

  const announce = useCallback((url: string, title: string) => {
    const room = roomRef.current;
    if (!room) return;
    const payload = new TextEncoder().encode(JSON.stringify({ t: 'watching', url, title }));
    // reliable: сообщение одно на переключение канала, и потерять его значит
    // оставить комнату смотреть разное, ничего об этом не зная.
    void room.localParticipant.publishData(payload, { reliable: true });
    setWatching({ url, title, by: selfName.current });
  }, []);

  return {
    state, error, room: roomCode, peers, feeds,
    micOn, camOn, watching,
    connect, leave, toggleMic, toggleCam, announce,
  };
}
