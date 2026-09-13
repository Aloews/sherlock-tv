import { accessToken, tokenUrl } from './config';

/**
 * Пропуск в комнату: что вернул релей.
 *
 * ⚠️ ПРИЧИНА ОТКАЗА НАЗЫВАЕТСЯ, А НЕ СВОДИТСЯ К «НЕ ПОЛУЧИЛОСЬ». Эти четыре
 * случая чинятся по-разному и до сих пор выглядели одинаково в играх с той же
 * архитектурой: `not_configured` правит владелец переменной сборки,
 * `forbidden` — секретом релея, `unavailable` — переменными релея, `network`
 * не правится вовсе и означает «попробуйте позже». Один текст «ошибка» на
 * всех четверых отправляет человека чинить не то.
 */
export type GrantResult =
  | { status: 'ok'; url: string; token: string; room: string; identity: string }
  | { status: 'not_configured' }   // VITE_LIVEKIT_TOKEN_URL нет в этой сборке
  | { status: 'bad_room' }         // релей не принял код комнаты
  | { status: 'forbidden' }        // не сошёлся общий секрет выдачи
  | { status: 'unavailable' }      // у релея нет ключей LiveKit
  | { status: 'network' }          // ответа не было вовсе
  | { status: 'malformed' };       // ответ 200 без пригодного пропуска

export async function fetchGrant(room: string, name: string): Promise<GrantResult> {
  const base = tokenUrl();
  if (!base) return { status: 'not_configured' };

  const url = new URL(base);
  url.searchParams.set('room', room);
  if (name) url.searchParams.set('name', name);
  const secret = accessToken();
  if (secret) url.searchParams.set('token', secret);

  let response: Response;
  try {
    response = await fetch(url.toString());
  } catch {
    return { status: 'network' };
  }

  if (response.status === 400) return { status: 'bad_room' };
  if (response.status === 403) return { status: 'forbidden' };
  if (response.status === 503) return { status: 'unavailable' };
  if (!response.ok) return { status: 'network' };

  let body: unknown;
  try {
    body = await response.json();
  } catch {
    return { status: 'malformed' };
  }
  const grant = body as Partial<Extract<GrantResult, { status: 'ok' }>>;
  // ⚠️ 200 БЕЗ ПРОПУСКА — ЭТО ОТКАЗ, А НЕ УСПЕХ. Ответ с кодом 200 над
  // сломанным следующим шагом называет живым то, что не работает; отдать
  // такой «пропуск» в LiveKit значит получить отказ уже там, на шаг позже и
  // не в том месте.
  if (typeof grant.url !== 'string' || typeof grant.token !== 'string'
      || grant.url.length === 0 || grant.token.length === 0) {
    return { status: 'malformed' };
  }
  return {
    status: 'ok',
    url: grant.url,
    token: grant.token,
    room: typeof grant.room === 'string' ? grant.room : room,
    identity: typeof grant.identity === 'string' ? grant.identity : '',
  };
}
