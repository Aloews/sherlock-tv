// Telegram WebApp adapter — ТОЛЬКО то, чем пользуется плеер.
//
// В игре этот модуль на 342 строки: initData, подпись, deep-link комнаты,
// тема, кнопки. Плееру из всего этого нужны две функции, и тащить остальное
// значило бы принести сюда поверхность, которую здесь некому вызывать и
// нечем проверить.
//
// ⚠️ ПРИЛОЖЕНИЕ НЕ ТРЕБУЕТ TELEGRAM. Оно открывается и в обычном браузере, и
// обе функции ниже там работают — иначе разработка шла бы вслепую.

interface TelegramWebApp {
  HapticFeedback?: {
    impactOccurred(style: 'light' | 'medium' | 'heavy'): void;
  };
  openLink?(url: string): void;
  ready?(): void;
  expand?(): void;
}

declare global {
  interface Window {
    Telegram?: {
      WebApp?: TelegramWebApp & {
        // Нужен только i18n — узнать язык клиента. Подпись здесь не
        // проверяется и проверять её нечем: сервера у приложения нет,
        // а язык интерфейса привилегий не даёт.
        initDataUnsafe?: { user?: { language_code?: string } };
      };
    };
  }
}

const tg = typeof window !== 'undefined' ? window.Telegram?.WebApp : undefined;

/** Сообщить клиенту, что приложение готово, и развернуть его на весь экран. */
export function initTelegram(): void {
  tg?.ready?.();
  tg?.expand?.();
}

const IMPACT_MS: Record<'light' | 'medium' | 'heavy', number> = {
  light: 10,
  medium: 25,
  heavy: 45,
};

function fallbackVibrate(pattern: number | number[]): void {
  try {
    navigator.vibrate?.(pattern);
  } catch {
    // не поддерживается или запрещено — молчим
  }
}

export function hapticImpact(style: 'light' | 'medium' | 'heavy' = 'medium'): void {
  if (tg?.HapticFeedback) tg.HapticFeedback.impactOccurred(style);
  else fallbackVibrate(IMPACT_MS[style]);
}

/**
 * Открыть ОБЫЧНУЮ веб-ссылку снаружи Mini App.
 *
 * Загружать её внутрь WebView нельзя: это заменило бы приложение страницей,
 * с которой нет пути назад.
 *
 * Снаружи Telegram — window.open: приложение живёт и в обычном браузере, а
 * ссылка, которая там молча ничего не делает, — это ссылка, которую никто не
 * может проверить.
 */
export function openLink(url: string): void {
  if (tg?.openLink) { tg.openLink(url); return; }
  window.open(url, '_blank', 'noopener,noreferrer');
}
