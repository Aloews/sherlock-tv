import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import ru from './locales/ru.json';
import en from './locales/en.json';
import es from './locales/es.json';
import pt from './locales/pt.json';
import fr from './locales/fr.json';
import zh from './locales/zh.json';
import ja from './locales/ja.json';
import ko from './locales/ko.json';
import ar from './locales/ar.json';

// ⚠️ ЗДЕСЬ ВСЕ ДЕВЯТЬ ЯЗЫКОВ СТАТИЧНО, И ЭТО НЕ КОПИЯ ИГРЫ, А РАСЧЁТ.
// В sherlock-scholes- локали грузятся динамически, потому что там их папка
// весит 352 КБ при бандле 933 КБ — треть первого кадра уходила на восемь
// языков, которые зритель не читает. Здесь переехал ОДИН блок из восемнадцати
// строк: все девять локалей вместе — около 5 КБ. Динамический импорт ради
// пяти килобайт добавил бы девять сетевых запросов и асинхронный старт, то
// есть заплатил бы задержкой за экономию, которой нет.
//
// Порог, при котором это решение надо пересмотреть, простой: как только
// локали перевалят за пару десятков килобайт, вернуть ленивую загрузку из
// игры — она там написана и проверена.

export const APP_LANGS = ['ru', 'en', 'es', 'pt', 'fr', 'zh', 'ja', 'ko', 'ar'] as const;
export type AppLang = (typeof APP_LANGS)[number];

const LANG_KEY = 'ml_lang';

const RESOURCES: Record<AppLang, object> = { ru, en, es, pt, fr, zh, ja, ko, ar };

function isAppLang(lang: string | null | undefined): lang is AppLang {
  return !!lang && (APP_LANGS as readonly string[]).includes(lang);
}

function detectLang(): AppLang {
  try {
    const saved = localStorage.getItem(LANG_KEY);
    if (isAppLang(saved)) return saved;
  } catch {
    // приватный режим — читать нечего, идём дальше
  }
  const tgLang = window.Telegram?.WebApp?.initDataUnsafe?.user?.language_code;
  if (isAppLang(tgLang)) return tgLang;
  // Язык браузера: приложение живёт и вне Telegram, и там это единственная
  // подсказка. `ru-RU` → `ru`.
  const nav = typeof navigator !== 'undefined' ? navigator.language?.split('-')[0] : null;
  if (isAppLang(nav)) return nav;
  return 'ru';
}

const lang = detectLang();

void i18n.use(initReactI18next).init({
  resources: Object.fromEntries(
    (Object.entries(RESOURCES) as [AppLang, object][])
      .map(([code, translation]) => [code, { translation }]),
  ),
  lng: lang,
  // en, потом ru: недостающий ключ должен показать хоть какой-то текст, а не
  // собственное имя.
  fallbackLng: ['en', 'ru'],
  interpolation: { escapeValue: false },
});

// ⚠️ `ar` — СПРАВА НАЛЕВО. В игре это делал общий каркас; здесь каркаса нет,
// и без этой строки арабский экран остался бы свёрстан слева направо.
document.documentElement.lang = lang;
document.documentElement.dir = lang === 'ar' ? 'rtl' : 'ltr';

export function setLang(next: AppLang): void {
  try { localStorage.setItem(LANG_KEY, next); } catch { /* приватный режим */ }
  void i18n.changeLanguage(next);
  document.documentElement.lang = next;
  document.documentElement.dir = next === 'ar' ? 'rtl' : 'ltr';
}

export default i18n;
