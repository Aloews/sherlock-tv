import { describe, it, expect, afterEach, vi } from 'vitest';
import { showCatalogue } from './config';

// Проверка ровно одного: строка 'true' и НИЧЕГО больше открывает каталог.
// Ошибиться здесь дорого — за флагом стоит группа `♥18+`.
describe('showCatalogue', () => {
  afterEach(() => { vi.unstubAllEnvs(); });

  it('is off when the variable is unset', () => {
    vi.stubEnv('VITE_SHOW_CATALOGUE', '');
    expect(showCatalogue()).toBe(false);
  });

  it('is on only for the exact string "true"', () => {
    vi.stubEnv('VITE_SHOW_CATALOGUE', 'true');
    expect(showCatalogue()).toBe(true);
  });

  // ⚠️ Ни одно из этих значений НЕ должно открывать каталог. Проверка
  // `Boolean(env)` пропустила бы каждое, а '1' и 'yes' — то, что человек
  // напишет в .env, не задумываясь.
  it.each(['1', 'yes', 'TRUE', 'True', 'on', 'false', '0'])(
    'stays off for %j',
    (value) => {
      vi.stubEnv('VITE_SHOW_CATALOGUE', value);
      expect(showCatalogue()).toBe(false);
    },
  );
});
