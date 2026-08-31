import type { Config } from 'tailwindcss';

// Те же токены, что в sherlock-scholes-, и через те же CSS-переменные:
// `<alpha-value>` — это то, что оставляет работающими модификаторы прозрачности
// (bg-brand-surface/50, text-brand-muted/70), а их в перенесённых компонентах
// достаточно, чтобы замена сломала вид молча.
//
// Переключателя оформления здесь нет: в игре он менял две визуальные темы, а у
// плеера тема одна. Переменные остались, потому что на них написана вся
// перенесённая вёрстка, а не потому что их кто-то переключает.
const brand = (name: string) => `rgb(var(--brand-${name}) / <alpha-value>)`;

export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        brand: {
          accent:     brand('accent'),
          accentSoft: brand('accent-soft'),
          accentDeep: brand('accent-deep'),
          highlight:  brand('highlight'),
          bg:         brand('bg'),
          surface:    brand('surface'),
          border:     brand('border'),
          muted:      brand('muted'),
        },
      },
      fontFamily: { sans: ['Inter', 'system-ui', 'sans-serif'] },
    },
  },
  plugins: [],
} satisfies Config;
