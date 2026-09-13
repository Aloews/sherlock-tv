/**
 * Совместный просмотр — НЕОБЯЗАТЕЛЬНЫЙ слой, и его отсутствие это норма.
 *
 * ⚠️ НЕ НАСТРОЕНО — ПАНЕЛИ НЕТ ВОВСЕ, а не есть кнопка, которая отвечает
 * ошибкой. Приложение работало без совместного просмотра и обязано работать
 * дальше: эфир — это продукт, разговор поверх него — добавка.
 *
 * Адрес выдачи пропусков публичен по природе (браузеру всё равно надо знать,
 * куда идти), ключа LiveKit здесь нет и быть не может: он лежит в релее
 * `stream-service` репозитория Aloews/sherlock-ai-bot, и наружу оттуда
 * выходит только короткоживущий пропуск.
 */

/** Адрес `/livekit-token` у релея. Не задан — совместного просмотра нет. */
export function tokenUrl(): string | undefined {
  const raw = import.meta.env.VITE_LIVEKIT_TOKEN_URL as string | undefined;
  return raw && raw.length > 0 ? raw : undefined;
}

/**
 * Общий секрет выдачи, если релей ею закрыт.
 *
 * ⚠️ ЭТО НЕ ПРОВЕРКА ПРАВ, И НАЗЫВАТЬ ЕЁ ТАК НЕЛЬЗЯ. Значение лежит в бандле
 * и открывается просмотром исходников — ровно как `STREAM_ACCESS_TOKEN` у
 * плейлиста. Оно закрывает выдачу от посторонних скриптов, а не от человека,
 * открывшего вкладку разработчика.
 */
export function accessToken(): string | undefined {
  const raw = import.meta.env.VITE_LIVEKIT_ACCESS_TOKEN as string | undefined;
  return raw && raw.length > 0 ? raw : undefined;
}

export function togetherEnabled(): boolean {
  return tokenUrl() !== undefined;
}
