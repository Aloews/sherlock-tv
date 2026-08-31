import { useTranslation } from 'react-i18next';
import { APP_LANGS, setLang, type AppLang } from '@/i18n';

// Подписи — на самих языках: список языков читает тот, кто текущего НЕ
// понимает, и переводить их на текущий значит писать не для него.
const LABEL: Record<AppLang, string> = {
  ru: 'Русский', en: 'English', es: 'Español', pt: 'Português', fr: 'Français',
  zh: '中文', ja: '日本語', ko: '한국어', ar: 'العربية',
};

/**
 * Выбор языка. В игре его давал общий каркас; здесь каркаса нет, а девять
 * локалей приехали вместе с экраном — без переключателя восемь из них были бы
 * недостижимы.
 *
 * Обычный `<select>`, а не своё меню: он открывается системным списком на
 * телефоне, доступен с клавиатуры и не требует ни строчки на закрытие по клику
 * снаружи.
 */
export function LangPicker() {
  const { i18n } = useTranslation();
  const current = (APP_LANGS as readonly string[]).includes(i18n.language)
    ? (i18n.language as AppLang)
    : 'ru';

  return (
    <select
      value={current}
      onChange={(e) => setLang(e.target.value as AppLang)}
      aria-label={LABEL[current]}
      className="h-9 shrink-0 rounded-full bg-brand-surface border border-brand-border
                 text-white text-xs px-2.5 focus:outline-none focus:border-brand-accent"
    >
      {APP_LANGS.map((code) => (
        <option key={code} value={code}>{LABEL[code]}</option>
      ))}
    </select>
  );
}
