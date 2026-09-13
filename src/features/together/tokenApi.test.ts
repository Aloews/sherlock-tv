import { describe, it, expect, afterEach, vi } from 'vitest';
import { fetchGrant } from './tokenApi';

/**
 * ПРИЧИНА ОТКАЗА НАЗЫВАЕТСЯ, А НЕ СВОДИТСЯ К «НЕ ПОЛУЧИЛОСЬ».
 *
 * Эти случаи чинятся по-разному: `not_configured` — переменной сборки,
 * `forbidden` — секретом релея, `unavailable` — переменными релея, `network`
 * не чинится вовсе. Один текст «ошибка» на всех отправляет человека чинить не
 * то — и именно так это уже выглядело в приложениях с такой же архитектурой.
 */
const ENDPOINT = 'https://relay.example/livekit-token';

function answer(status: number, body?: unknown) {
  return vi.fn(async () => new Response(
    body === undefined ? '' : JSON.stringify(body),
    { status, headers: { 'content-type': 'application/json' } },
  ));
}

afterEach(() => {
  vi.unstubAllEnvs();
  vi.unstubAllGlobals();
});

describe('fetchGrant', () => {
  it('без VITE_LIVEKIT_TOKEN_URL не ходит в сеть вовсе', async () => {
    vi.stubEnv('VITE_LIVEKIT_TOKEN_URL', '');
    const fetchSpy = answer(200, {});
    vi.stubGlobal('fetch', fetchSpy);
    expect(await fetchGrant('abc-123', 'Вася')).toEqual({ status: 'not_configured' });
    expect(fetchSpy).not.toHaveBeenCalled();
  });

  it('отдаёт пропуск и адрес сервиса', async () => {
    vi.stubEnv('VITE_LIVEKIT_TOKEN_URL', ENDPOINT);
    vi.stubGlobal('fetch', answer(200, {
      url: 'wss://example.livekit.cloud', token: 'eyJ.a.b',
      room: 'abc-123', identity: 'tv-1',
    }));
    expect(await fetchGrant('abc-123', 'Вася')).toEqual({
      status: 'ok', url: 'wss://example.livekit.cloud', token: 'eyJ.a.b',
      room: 'abc-123', identity: 'tv-1',
    });
  });

  it('код комнаты, имя и общий секрет уходят в запрос', async () => {
    vi.stubEnv('VITE_LIVEKIT_TOKEN_URL', ENDPOINT);
    vi.stubEnv('VITE_LIVEKIT_ACCESS_TOKEN', 'shh');
    const fetchSpy = answer(200, { url: 'wss://x', token: 't' });
    vi.stubGlobal('fetch', fetchSpy);
    await fetchGrant('abc-123', 'Вася');
    const asked = new URL((fetchSpy.mock.calls[0] as unknown as [string])[0]);
    expect(asked.searchParams.get('room')).toBe('abc-123');
    expect(asked.searchParams.get('name')).toBe('Вася');
    expect(asked.searchParams.get('token')).toBe('shh');
  });

  it('без секрета в сборке он в запрос не подставляется', async () => {
    vi.stubEnv('VITE_LIVEKIT_TOKEN_URL', ENDPOINT);
    vi.stubEnv('VITE_LIVEKIT_ACCESS_TOKEN', '');
    const fetchSpy = answer(200, { url: 'wss://x', token: 't' });
    vi.stubGlobal('fetch', fetchSpy);
    await fetchGrant('abc-123', '');
    const asked = new URL((fetchSpy.mock.calls[0] as unknown as [string])[0]);
    expect(asked.searchParams.has('token')).toBe(false);
    expect(asked.searchParams.has('name')).toBe(false);
  });

  it.each([
    [400, 'bad_room'],
    [403, 'forbidden'],
    [503, 'unavailable'],
    [500, 'network'],
  ])('код %i читается как %s', async (status, expected) => {
    vi.stubEnv('VITE_LIVEKIT_TOKEN_URL', ENDPOINT);
    vi.stubGlobal('fetch', answer(status, { error: 'x' }));
    expect((await fetchGrant('abc-123', '')).status).toBe(expected);
  });

  it('отсутствие ответа — это network, а не тихий отказ', async () => {
    vi.stubEnv('VITE_LIVEKIT_TOKEN_URL', ENDPOINT);
    vi.stubGlobal('fetch', vi.fn(async () => { throw new TypeError('failed'); }));
    expect((await fetchGrant('abc-123', '')).status).toBe('network');
  });

  // ⚠️ 200 БЕЗ ПРОПУСКА — ЭТО ОТКАЗ. Промежуточный ответ 200 над сломанным
  // следующим шагом называет живым то, что не работает: такой «пропуск»
  // отвергнет уже LiveKit, на шаг позже и не в том месте.
  it.each([
    [{}],
    [{ url: 'wss://x' }],
    [{ token: 'eyJ' }],
    [{ url: '', token: 'eyJ' }],
    [{ url: 'wss://x', token: '' }],
  ])('200 с негодным телом %j — это malformed', async (body) => {
    vi.stubEnv('VITE_LIVEKIT_TOKEN_URL', ENDPOINT);
    vi.stubGlobal('fetch', answer(200, body));
    expect((await fetchGrant('abc-123', '')).status).toBe('malformed');
  });

  it('200 не-JSON — тоже malformed, а не падение', async () => {
    vi.stubEnv('VITE_LIVEKIT_TOKEN_URL', ENDPOINT);
    vi.stubGlobal('fetch', vi.fn(async () => new Response('не json', { status: 200 })));
    expect((await fetchGrant('abc-123', '')).status).toBe('malformed');
  });
});
