// Разбор M3U-плейлиста — ЧИСТАЯ часть: ни сети, ни Supabase, ни окружения.
//
// ⚠️ РАДИ ЧЕГО ЭТОТ ФАЙЛ ВООБЩЕ ПОЯВИЛСЯ. Экран `/stream` не мог работать
// никогда, и дело было не в плеере. `VITE_STREAM_URL` отдаёт вот это:
//
//   #EXTM3U url-tvg="http://iptvx.one/epg/epg.xml.gz"
//   #EXTINF:-1 group-title="SPORT 🏆",Setanta Sports 1 HD
//   https://stream8.cinerama.uz/1263/tracks-v1a1/mono.m3u8
//   … ещё 4080 таких же строк
//
// Это КАТАЛОГ КАНАЛОВ, а не HLS-манифест. Ни одного `#EXT-X-STREAM-INF`, ни
// одного `#EXT-X-TARGETDURATION`, ни одного `#EXT-X-VERSION` — я проверял
// боевой ответ, их там ровно ноль. А `StreamScreen` отдавал этот адрес прямо
// в `useHlsPlayer`, то есть в `hls.loadSource()`. Играть там нечего: hls.js
// ждёт манифест, а получает список ссылок на другие манифесты. Отсюда и
// «трансляции не работают» — плеер честно показывал `stream.error` на
// единственное, что ему давали.
//
// Поэтому каталог надо СНАЧАЛА РАЗОБРАТЬ, показать зрителю каналы, и только
// выбранный канал отдавать в плеер. Разбор живёт здесь, без единого импорта
// из приложения, — тем же приёмом, что `features/fantasy/tactics.ts` и
// `features/chess/rules.ts`: тест на него не должен падать на клиенте базы,
// которого ему не нужно (см. шапку tactics.ts, там эта история целиком).

export interface Channel {
  name: string;
  group: string;
  /** `tvg-logo` из плейлиста. В боевом списке его нет ни у одного канала. */
  logo: string | null;
  url: string;
}

/**
 * Разобрать текст плейлиста.
 *
 * Формат строки: `#EXTINF:<секунды> ключ="значение" …,<Название>`, а следом
 * — строка с адресом. У живого канала секунды всегда `-1`.
 *
 * ⚠️ АДРЕС ИЩЕТСЯ НЕ «СЛЕДУЮЩЕЙ СТРОКОЙ», А СЛЕДУЮЩЕЙ НЕПУСТОЙ. В боевом
 * файле между записью и её адресом попадаются пустые строки и `#EXTGRP`,
 * и наивное `lines[i + 1]` теряло бы такие каналы молча — что хуже всего:
 * список просто короче, и никто не замечает.
 */
export function parseM3u(text: string): Channel[] {
  const lines = text.split(/\r?\n/);
  const out: Channel[] = [];

  for (let i = 0; i < lines.length; i += 1) {
    const line = lines[i].trim();
    if (!line.startsWith('#EXTINF')) continue;

    const comma = line.indexOf(',');
    // Без запятой у записи нет названия — показывать в списке нечего.
    if (comma === -1) continue;
    const name = line.slice(comma + 1).trim();
    if (!name) continue;

    const attrs = line.slice(0, comma);
    const group = matchAttr(attrs, 'group-title') ?? '';
    const logo = matchAttr(attrs, 'tvg-logo');

    let url = '';
    for (let j = i + 1; j < lines.length; j += 1) {
      const candidate = lines[j].trim();
      if (!candidate) continue;
      // Ещё один `#EXTINF` — у этой записи адреса не было вовсе; отдаём ей
      // разбираться заново со следующей итерации, а текущую бросаем.
      if (candidate.startsWith('#EXTINF')) break;
      if (candidate.startsWith('#')) continue;
      url = candidate;
      i = j;
      break;
    }
    if (!url) continue;

    out.push({ name, group, logo, url });
  }

  return out;
}

function matchAttr(attrs: string, key: string): string | null {
  const m = new RegExp(`${key}="([^"]*)"`).exec(attrs);
  const value = m?.[1]?.trim();
  return value ? value : null;
}

/**
 * Играбелен ли канал ВНУТРИ Mini App.
 *
 * ⚠️ ТОЛЬКО `https:`, И ЭТО НЕ ПРИДИРКА. Mini App открывается по https, а
 * браузер режет смешанное содержимое: `http://`-поток в https-странице не
 * загрузится ни через hls.js, ни через `video.src`, и никакая правка плеера
 * этого не изменит. В боевом плейлисте таких 96 из 127 спортивных — то есть
 * список «всех каналов» был бы на три четверти из заведомо мёртвых строк.
 * Лучше показать 31 работающий канал, чем 127, из которых играет четверть.
 */
export function isPlayable(channel: Channel): boolean {
  return channel.url.startsWith('https://');
}

/**
 * Спортивный ли канал.
 *
 * По названию группы, а не по точному совпадению с `SPORT 🏆`: группы в этом
 * плейлисте переименовывают и правят руками (там же соседствуют `TEST-1` и
 * `Tест-24-08`), и жёсткая строка однажды тихо оставит экран пустым.
 */
export function isSport(channel: Channel): boolean {
  return /sport|спорт|футбол|futbol/i.test(channel.group);
}

/**
 * Каналы, которые зритель назвал сам. Порядок ВНУТРИ списка значим — он станет
 * порядком в начале экрана.
 *
 * Совпадение по вхождению в название без учёта регистра: стабильного
 * идентификатора у канала в этом плейлисте нет вовсе. `tvg-id` проставлен у 166
 * записей из 4082 и ни одной спортивной, а числа в адресах принадлежат
 * конкретному ретранслятору и меняются вместе с ним.
 *
 * ⚠️ ЗАКРЕПЛЁННЫЙ КАНАЛ ПРОХОДИТ МИМО ФИЛЬТРА ПО ГРУППЕ. «Матч! Премьер» лежит
 * в группе `Оргтехсервис 🎯VPN`, «Беларусь 5» и «Setanta Sports UA» — в
 * `TEST-1`: группы здесь правят руками, и требовать от названного канала ещё и
 * правильной группы значит молча его потерять.
 *
 * ⚠️ НО ФИЛЬТР ПО `https` ОН НЕ ПРОХОДИТ, И ЭТО НЕ ОБСУЖДАЕТСЯ — см. isPlayable.
 *
 * ЗАМЕР 31.08.2026 ПО БОЕВОМУ КАТАЛОГУ (3875 записей), до БАЙТОВ ВИДЕО, а не
 * до кода 200. Владелец прислал список из девяти каналов, которые хочет
 * видеть; вот что из них вообще возможно:
 *
 *   канал                    итог
 *   Матч ТВ                  ✓ работает, ДВА источника по https, CORS `*`
 *   Матч Премьер             ✓ работает («Матч! Премьер», flussonic)
 *   Матч! Футбол 1–3         ✗ в каталоге ТОЛЬКО по http
 *   Setanta Sports UA        ✗ только по http
 *   Setanta Sports+          ✗ только по http
 *   Maincast sport           ✗ только по http
 *   Беларусь 5               ✗ только по http
 *
 * ⚠️ ПРЕЖНЯЯ ЗАПИСЬ В ЭТОЙ ШАПКЕ УСТАРЕЛА, И ЭТО ВАЖНЕЕ ОСТАЛЬНОГО. Здесь было
 * написано «Setanta Sports 1/2 HD — играют». На 31.08.2026 они НЕ играют:
 * верхний манифест отдаёт 200, а вариант под ним — 404. Это ровно тот случай,
 * ради которого проверка идёт до байтов: остановись она на первом уровне, она
 * назвала бы оба канала живыми.
 *
 * ⚠️ И НАХОДКА, КОТОРОЙ В СПИСКЕ ВЛАДЕЛЬЦА НЕ БЫЛО. Рабочий «Матч ТВ» лежит на
 * той же раздаче (`flussonic.mkpnet.ru`, группа `Оргтехсервис 🎯VPN`), что и
 * «Матч! Арена», «Матч! Игра», «Матч! Страна», а «Матч! Планета» — на
 * соседней рабочей. Все проверены до байтов, все отдают CORS `*`. Группа у них
 * не спортивная, поэтому без имени в этом списке они не показываются вовсе —
 * закрепляем.
 *
 * Каналы, доступные только по http, оставлены здесь НАРОЧНО: список — это
 * «показать, когда сможем», и в тот день, когда ретранслятор отдаст их по
 * https, они появятся сами, без правки кода.
 */
export const PINNED: readonly string[] = [
  // ⚠️ ПЕРВАЯ СТРОКА — КАНАЛ ПО УМОЛЧАНИЮ. Он играет сразу при открытии
  // экрана: см. isDefault ниже и bucket() в ./order.ts. Владелец выбрал
  // «Real Madrid» (скриншот 31.08.2026), и канал измерен — 311 КБ видео,
  // CORS `*`.
  'real madrid',        // ✓ 311 КБ видео

  // ── Остальное со скриншотов владельца, в его порядке ──
  'top barca',          // ✓ 1017 КБ
  'barca',
  'eurosport',          // ✓ 2823 КБ («EUROSPORT 1»)
  'матч тв',            // ✓ два источника
  'матч! футбол',       // ✗ только http
  'футбол 1',
  'футбол 2',
  'футбол 3',
  'матч! премьер',      // ✓
  'матч премьер',
  'setanta sports ua',  // ✗ только http
  'setanta sports+',    // ✗ только http
  'maincast',           // ✗ только http
  'беларусь 5',         // ✗ только http
  'окко спорт',         // ✗ только http («Окко Спорт 2»)
  'm sport',            // ✗ только http («M Sport vpn»)
  'espn premium',       // ✗ только http («ESPN Premium HD Испания»)

  // ── Та же рабочая раздача, что у «Матч ТВ» ──
  'матч! арена',
  'матч! игра',
  'матч! страна',
  'матч! планета',

  // ── Прежние. `setanta` ловит Setanta Sports 1/2 HD — на 31.08.2026 они
  //    отдают 404 на втором уровне, но останутся здесь по тому же правилу,
  //    что и http-каналы: почини́тся у ретранслятора — вернутся сами ──
  'setanta',
  'viasat sport',
  'arena sport',
  'diema sport',
  'nova sport',
];

/**
 * Канал по умолчанию — тот, что играет сразу при открытии.
 *
 * ⚠️ ЭТО ПЕРВАЯ СТРОКА PINNED, А НЕ ОТДЕЛЬНЫЙ СПИСОК. Два места, где написано
 * «какой канал главный», однажды разойдутся, и объяснить, почему играет не то,
 * что стоит первым, будет нечем.
 */
export function isDefault(channel: Channel): boolean {
  return pinRank(channel) === 0;
}

/** Назван ли канал в PINNED. */
export function isPinned(channel: Channel): boolean {
  const name = channel.name.toLowerCase();
  return PINNED.some((needle) => name.includes(needle));
}

/**
 * Место канала в PINNED, или `PINNED.length` для всех прочих.
 * Живёт рядом со списком, чтобы порядок и отбор читались по одному источнику.
 */
export function pinRank(channel: Channel): number {
  const name = channel.name.toLowerCase();
  const i = PINNED.findIndex((needle) => name.includes(needle));
  return i === -1 ? PINNED.length : i;
}

/**
 * Что зритель просит показать. Ширится слева направо.
 *
 * `sport`      — спортивные группы и названные каналы, только https. Так было
 *                всегда, и это по-прежнему то, с чего экран открывается.
 * `all`        — ВЕСЬ каталог, но только https: всё, что вообще может
 *                открыться. Фильмы, музыка, страновые пакеты — 2158 записей.
 * `everything` — плюс http-записи. Они НЕ ОТКРОЮТСЯ никогда (см. isPlayable),
 *                и показываются помеченными: это «посмотреть, что вообще
 *                лежит в каталоге», а не «выбрать».
 */
export type ChannelScope = 'sport' | 'all' | 'everything';

/**
 * Взрослая группа. ИСКЛЮЧАЕТСЯ ВО ВСЕХ РЕЖИМАХ, включая `everything`.
 *
 * ⚠️ ЭТО НЕ ФИЛЬТР, А ГРАНИЦА, и она не зависит от того, что выбрал зритель.
 * В боевом каталоге в группе `♥18+` 126 записей, 71 из них по https, то есть
 * они бы открылись. Приложение про спортивный эфир, оно без возрастной двери и
 * без входа вообще — класть туда такое нельзя, и «зритель сам выберет» этого
 * не отменяет: выбирать он будет уже увидев.
 *
 * ⚠️ ФЛАГ ПРИХОДИТ ПАРАМЕТРОМ, А НЕ ЧИТАЕТСЯ ИЗ ОКРУЖЕНИЯ. Этот файл объявлен
 * чистым в первой же строке — ни сети, ни окружения, — и `import.meta.env`
 * здесь сделал бы тесты зависимыми от сборки, а сам отбор — непроверяемым в
 * обе стороны. Значение берёт вызывающий: showAdult() в ./config.ts.
 */
function isAdult(channel: Channel): boolean {
  return /18\+|xxx|erot|adult/i.test(channel.group);
}

/**
 * Показывается ли канал ЗРИТЕЛЮ в выбранном режиме.
 *
 * ⚠️ ПРЕДИКАТ ОДИН НА ВСЕ СПИСКИ. Экран показывает отобранное, админский
 * каталог — всё с пометкой «в плеере / не в плеере». Опиши отбор дважды — и
 * однажды они разойдутся: оператор увидит «канал показывается», а зритель его
 * не найдёт, и искать причину будет негде.
 */
export function isShown(
  channel: Channel,
  scope: ChannelScope = 'sport',
  allowAdult = false,
): boolean {
  if (isAdult(channel) && !allowAdult) return false;
  // http не откроется ничем — в двух первых режимах его нет вовсе.
  if (scope !== 'everything' && !isPlayable(channel)) return false;
  if (scope !== 'sport') return true;
  // Спортивная группа ИЛИ названный канал: см. шапку PINNED — группы в этом
  // плейлисте правят руками, и «Матч! Премьер» лежит не в спортивной.
  return isSport(channel) || isPinned(channel);
}

/**
 * Каталог без повторов, в порядке плейлиста.
 *
 * Повторы убираются по адресу: один и тот же канал лежит в плейлисте по два
 * и три раза («Матч ТВ» встречается трижды), и в списке это выглядит багом.
 */
function dedupe(text: string): Channel[] {
  const seen = new Set<string>();
  const out: Channel[] = [];
  for (const channel of parseM3u(text)) {
    if (seen.has(channel.url)) continue;
    seen.add(channel.url);
    out.push(channel);
  }
  return out;
}

/** Что показать на экране зрителя: спорт, играбельное, без повторов. */
export function sportChannels(text: string): Channel[] {
  return channelsFor(text, 'sport');
}

/**
 * То же в выбранном режиме.
 *
 * ⚠️ ПОРЯДОК СОХРАНЯЕТСЯ ПЛЕЙЛИСТНЫЙ, а не пересортировывается по режиму:
 * поверх этого работает orderChannels (избранное → здоровье → названные), и
 * второй порядок здесь означал бы, что список прыгает при смене режима.
 */
export function channelsFor(text: string, scope: ChannelScope, allowAdult = false): Channel[] {
  return dedupe(text).filter((c) => isShown(c, scope, allowAdult));
}

/** Группы, встречающиеся в выбранном режиме, с числом записей в каждой. */
export function groupsFor(channels: readonly Channel[]): { group: string; n: number }[] {
  const by = new Map<string, number>();
  for (const c of channels) by.set(c.group || '—', (by.get(c.group || '—') ?? 0) + 1);
  return [...by].map(([group, n]) => ({ group, n }))
    .sort((a, b) => b.n - a.n || a.group.localeCompare(b.group));
}

/**
 * Запись каталога С ПРИЧИНОЙ, по которой её нет в плеере. Ради `sport` и
 * `playable` по отдельности: «не спорт» и «не откроется по https» — разные
 * поводы, и оператору нужно видеть, какой именно.
 */
export interface CatalogueChannel extends Channel {
  sport: boolean;
  playable: boolean;
  /** Ровно `isShown` — то, что зритель увидит на `/stream`. */
  shown: boolean;
}

/**
 * ВЕСЬ каталог, для админского кабинета.
 *
 * ⚠️ ЭТО НЕ ВТОРОЙ ЭКРАН ТВ, А ОКНО В ИСХОДНЫЙ ФАЙЛ. Зрительу по-прежнему
 * достаётся `sportChannels`; здесь видно всё вместе с ответом на
 * единственный вопрос, который тут задают: «почему этого канала нет в
 * приложении». Живёт за паролем `staffVerify` — там же, где редактор
 * карточек, и по той же причине: в каталоге есть группа `♥18+`.
 */
export function catalogue(text: string): CatalogueChannel[] {
  return dedupe(text).map((channel) => ({
    ...channel,
    sport: isSport(channel),
    playable: isPlayable(channel),
    shown: isShown(channel),
  }));
}

/** Группы каталога по убыванию размера — с чего оператор начинает смотреть. */
export function groupCounts(channels: readonly CatalogueChannel[]): { group: string; total: number; shown: number }[] {
  const by = new Map<string, { group: string; total: number; shown: number }>();
  for (const c of channels) {
    const key = c.group || '—';
    const row = by.get(key) ?? { group: key, total: 0, shown: 0 };
    row.total += 1;
    if (c.shown) row.shown += 1;
    by.set(key, row);
  }
  return [...by.values()].sort((a, b) => b.total - a.total || a.group.localeCompare(b.group));
}
