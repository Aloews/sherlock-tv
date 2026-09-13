import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
  IconCamera, IconCameraOff, IconMicrophone, IconMicrophoneOff,
  IconUsers, IconCopy, IconCheck,
} from '@tabler/icons-react';
import { hapticImpact } from '@/shared/lib/telegram';
import { togetherEnabled } from './config';
import { isValidRoomCode, makeRoomCode, normalizeRoomCode } from './roomCode';
import { useTogether, type Feed } from './useTogether';

interface Props {
  /** Что играет прямо сейчас — этим и делятся с комнатой. */
  currentUrl: string | null;
  currentTitle: string;
  /** Перейти на канал, который смотрит комната. */
  onFollow: (url: string) => void;
}

/**
 * СМОТРЕТЬ ВМЕСТЕ: разговор поверх эфира.
 *
 * ⚠️ ЭТО ДОБАВКА, А НЕ ЧАСТЬ ПЛЕЕРА. Не настроен `VITE_LIVEKIT_TOKEN_URL` —
 * панели нет вовсе, и приложение работает ровно как работало. Эфир — продукт,
 * разговор поверх него — слой; слой, из-за которого не играет канал, хуже
 * отсутствующего слоя.
 *
 * ⚠️ КАНАЛ НЕ ПЕРЕКЛЮЧАЕТСЯ САМ. Комната сообщает, что смотрит, и зритель
 * нажимает «перейти». Выдернуть картинку из-под человека, который смотрит, —
 * это не «вместе», а помеха; к тому же половина каналов у половины зрителей
 * не играет (см. отсев мёртвых в features/stream/order.ts), и принудительный
 * переход раз за разом уводил бы часть комнаты в чёрный экран.
 *
 * ⚠️ САМА ТРАНСЛЯЦИЯ НЕ ПЕРЕДАЁТСЯ ЧЕРЕЗ LIVEKIT, и это решение, а не
 * ограничение. Каждый смотрит свой поток от источника; по комнате идут только
 * голос, картинка с камеры и одна строка «смотрим вот это». Ретранслировать
 * чужой эфир участникам — это раздача чужого сигнала со своего сервера, чего
 * этот продукт не делает нигде: он показывает каталог, а не вещает.
 *
 * ⚠️ КАМЕРА 320×240 И ВЫКЛЮЧЕНА ПО УМОЛЧАНИЮ. Смотрят матч, а не друг друга;
 * картинка здесь — лицо рядом с экраном. Просить 720p и позволить сервису
 * сжать её до той же величины — это та же батарея телефона, потраченная
 * впустую.
 */
export function TogetherPanel({ currentUrl, currentTitle, onFollow }: Props) {
  const { t } = useTranslation();
  const sink = useRef<HTMLDivElement>(null);
  const tg = useTogether(sink);
  const [code, setCode] = useState('');
  const [name, setName] = useState('');
  const [copied, setCopied] = useState(false);

  // Панели нет, а не есть кнопка, отвечающая ошибкой. `import.meta.env`
  // подставляется на сборке, и Vite вырезает всё ниже вместе с ней.
  if (!togetherEnabled()) return null;

  const joined = tg.state === 'connected';

  const start = (value: string) => {
    const room = normalizeRoomCode(value);
    if (!isValidRoomCode(room)) return;
    hapticImpact('light');
    void tg.connect(room, name.trim());
  };

  const copy = () => {
    if (!tg.room) return;
    hapticImpact('light');
    void navigator.clipboard?.writeText(tg.room).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    }).catch(() => { /* буфера может не быть — код виден на экране и так */ });
  };

  return (
    <div className="w-full max-w-sm bg-brand-surface border border-brand-border rounded-xl p-3 space-y-2">
      <div className="flex items-center gap-2">
        <IconUsers size={16} stroke={1.75} className="text-brand-muted shrink-0" />
        <span className="flex-1 text-white text-[12.5px]">{t('together.title')}</span>
        {joined && (
          <span className="text-brand-muted text-[10.5px] tabular-nums">
            {t('together.people', { count: tg.peers.length + 1 })}
          </span>
        )}
      </div>

      {!joined && (
        <>
          <p className="text-brand-muted text-[10.5px] leading-snug">{t('together.about')}</p>
          <input
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder={t('together.your_name')}
            maxLength={32}
            className="w-full bg-brand-bg border border-brand-border rounded-lg px-3 py-2 text-[12.5px] text-white placeholder:text-brand-muted/70 outline-none focus:border-brand-accent/60"
          />
          <div className="flex gap-1.5">
            <input
              value={code}
              onChange={(e) => setCode(e.target.value)}
              placeholder={t('together.code_placeholder')}
              autoCapitalize="off"
              autoCorrect="off"
              spellCheck={false}
              maxLength={48}
              className="flex-1 min-w-0 bg-brand-bg border border-brand-border rounded-lg px-3 py-2 text-[12.5px] text-white placeholder:text-brand-muted/70 outline-none focus:border-brand-accent/60 font-mono"
            />
            <button
              type="button"
              disabled={!isValidRoomCode(normalizeRoomCode(code)) || tg.state === 'connecting'}
              onClick={() => start(code)}
              className="shrink-0 px-3 rounded-lg border border-brand-accent bg-brand-accent/15 text-white text-[12px] disabled:opacity-40"
            >
              {t('together.join')}
            </button>
          </div>
          {/* ⚠️ СВОЙ КОД НЕ ПРИДУМЫВАЕТСЯ, А ВЫДАЁТСЯ. Код — это ключ от
              комнаты: проверить, кто стучится, релею нечем, и «match1» подберут
              перебором. Поле выше — только для кода, полученного от друга. */}
          <button
            type="button"
            disabled={tg.state === 'connecting'}
            onClick={() => { const fresh = makeRoomCode(); setCode(fresh); start(fresh); }}
            className="w-full py-2 rounded-lg border border-brand-border text-brand-muted text-[12px] active:text-white disabled:opacity-40"
          >
            {t('together.create')}
          </button>
          {tg.state === 'connecting' && (
            <p className="text-brand-muted text-[10.5px]">{t('together.connecting')}</p>
          )}
          {/* Причина отказа называется: эти случаи чинятся по-разному, и
              общее «ошибка» отправляет человека чинить не то. */}
          {tg.error && (
            <p className="text-brand-muted text-[10.5px]">{t(`together.error_${tg.error}`)}</p>
          )}
        </>
      )}

      {joined && (
        <>
          <div className="flex items-center gap-1.5">
            <code className="flex-1 min-w-0 truncate bg-brand-bg border border-brand-border rounded-lg px-2 py-1.5 text-[12px] text-white font-mono">
              {tg.room}
            </code>
            <button
              type="button"
              onClick={copy}
              aria-label={t('together.copy')}
              className="shrink-0 p-2 rounded-lg border border-brand-border text-brand-muted active:text-white"
            >
              {copied ? <IconCheck size={14} stroke={2} /> : <IconCopy size={14} stroke={1.75} />}
            </button>
          </div>
          <p className="text-brand-muted/70 text-[10px] leading-snug">{t('together.code_is_the_key')}</p>

          <div className="flex gap-1.5">
            <Toggle on={tg.micOn} onClick={tg.toggleMic}
                    label={tg.micOn ? t('together.mic_on') : t('together.mic_off')}
                    icon={tg.micOn ? <IconMicrophone size={14} stroke={1.75} />
                                   : <IconMicrophoneOff size={14} stroke={1.75} />} />
            <Toggle on={tg.camOn} onClick={tg.toggleCam}
                    label={tg.camOn ? t('together.cam_on') : t('together.cam_off')}
                    icon={tg.camOn ? <IconCamera size={14} stroke={1.75} />
                                   : <IconCameraOff size={14} stroke={1.75} />} />
            <button
              type="button"
              onClick={() => { hapticImpact('light'); tg.leave(); }}
              className="ml-auto px-3 rounded-lg border border-brand-border text-brand-muted text-[12px] active:text-white"
            >
              {t('together.leave')}
            </button>
          </div>

          {tg.peers.length > 0 && (
            <p className="text-brand-muted text-[10.5px] truncate">
              {tg.peers.map((p) => (p.speaking ? `· ${p.name}` : p.name)).join(', ')}
            </p>
          )}

          {/* Поделиться тем, что играет. Кнопка, а не автоматическая рассылка:
              переключений каналов бывает десяток подряд, пока ищут рабочий. */}
          {currentUrl && (
            <button
              type="button"
              onClick={() => { hapticImpact('light'); tg.announce(currentUrl, currentTitle); }}
              className="w-full py-2 rounded-lg border border-brand-border text-brand-muted text-[12px] active:text-white"
            >
              {t('together.share_channel')}
            </button>
          )}

          {tg.watching && tg.watching.url !== currentUrl && (
            <button
              type="button"
              onClick={() => { hapticImpact('light'); onFollow(tg.watching!.url); }}
              className="w-full py-2 rounded-lg border border-brand-accent bg-brand-accent/15 text-white text-[12px] text-left px-3"
            >
              {t('together.follow', {
                who: tg.watching.by || t('together.someone'),
                channel: tg.watching.title || t('together.another_channel'),
              })}
            </button>
          )}

          <VideoStrip feeds={tg.feeds} />
        </>
      )}

      {/* Звук. Пустой контейнер, в который SDK кладёт свои <audio>: подписанная
          дорожка — ещё не звучащая, и положить элемент в документ наше дело. */}
      <div ref={sink} className="hidden" aria-hidden="true" />
    </div>
  );
}

function Toggle({ on, onClick, label, icon }: {
  on: boolean; onClick: () => void; label: string; icon: React.ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={() => { hapticImpact('light'); onClick(); }}
      className={`inline-flex items-center gap-1.5 px-2.5 py-2 rounded-lg border text-[12px] ${
        on ? 'border-brand-accent bg-brand-accent/15 text-white'
           : 'border-brand-border text-brand-muted'}`}
    >
      {icon}
      {label}
    </button>
  );
}

/**
 * Картинки с камер, когда они есть.
 *
 * ⚠️ НИЧЕГО НЕ РИСУЕТ, ПОКА НИКТО НЕ ВКЛЮЧИЛ КАМЕРУ, — а это обычный случай.
 * Пустая полоса из рамок отодвигала бы плеер вниз ради того, чего в
 * большинстве сеансов не будет.
 *
 * Элементы построены SDK и сюда только вставляются: перерисовка не должна
 * подменить элемент под живой дорожкой.
 */
function VideoStrip({ feeds }: { feeds: Feed[] }) {
  const host = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const node = host.current;
    if (!node) return;
    node.replaceChildren();
    for (const feed of feeds) {
      const tile = document.createElement('div');
      tile.className = 'w-20 h-16 rounded-lg overflow-hidden bg-brand-bg shrink-0';
      feed.element.className = 'w-full h-full object-cover';
      tile.appendChild(feed.element);
      node.appendChild(tile);
    }
  }, [feeds]);

  if (feeds.length === 0) return null;
  return <div ref={host} className="flex gap-1.5 overflow-x-auto" />;
}
