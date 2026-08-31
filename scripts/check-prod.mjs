#!/usr/bin/env node
// Проверка ПРОДА. Ни одного мока, ни одного стенда.
//
// ⚠️ ЗАЧЕМ ЭТОТ ФАЙЛ СУЩЕСТВУЕТ — прочитайте, прежде чем что-то тут менять.
//
// В этом приложении 134 юнит-теста, и все они гоняют код против стенда:
// плейлист подделан, ответы каналов подделаны, сегменты подделаны. Так и
// надо — но такой тест НЕ МОЖЕТ упасть по той причине, по которой ломается
// приложение. В игре, откуда этот экран приехал, они были зелёными ровно
// тогда, когда владелец присылал скриншоты со сломанным ТВ.
//
// Хуже: «проверка» вручную останавливалась на первом манифесте. Канал отдал
// 200 — значит работает. А ниже первого уровня лежало вот это (замер
// 25.08.2026, боевой каталог):
//
//   Матч! Премьер        master -> variant -> сегмент 1.3 МБ    ЖИВОЙ
//   Setanta Sports 1 HD  master 200 -> variant 404              МЁРТВЫЙ
//   Setanta Sports 2 HD  master 200 -> variant не отвечает      МЁРТВЫЙ
//
// Setanta стояли вторым и третьим в списке. Верхний манифест у них отвечает
// 200, и любая проверка, которая на нём останавливается, называет их живыми.
//
// ПОЭТОМУ ЗДЕСЬ ДВА ПРАВИЛА, И ОНИ НЕ ОБСУЖДАЮТСЯ:
//
//   1. Проверка идёт до КОНЦА цепочки — до байтов видео, а не до кода 200.
//   2. У каждой проверки есть ОТРИЦАТЕЛЬНЫЙ КОНТРОЛЬ: та же проверка,
//      направленная на заведомо сломанное, ОБЯЗАНА упасть. Если не упала —
//      проверка ничего не проверяет, и скрипт называет её ПУСТОЙ. Зелёная
//      пустая проверка хуже красной: она врёт с уверенностью.
//
//   node scripts/check-prod.mjs
//
// Выход: 0 — всё живо и все проверки не пусты; 1 — есть падение или пустая.

import { readFileSync, existsSync } from 'node:fs';

// Адрес выкаченного приложения. Проверка бандла ходит сюда; если приложение
// ещё никуда не выкачено, задайте PROD_APP_URL или пропустите этот раздел.
const APP = process.env.PROD_APP_URL ?? '';
const TIMEOUT_MS = 25_000;

// Ключи берём из окружения, а при его отсутствии — из .env. Без этого
// проверка дайджеста молча превращалась в «не измерено», то есть в ту самую
// пустую зелень, против которой весь этот файл и написан.
function env(name) {
  if (process.env[name]) return process.env[name];
  if (!existsSync('.env')) return null;
  for (const line of readFileSync('.env', 'utf-8').split('\n')) {
    const m = /^\s*([A-Z0-9_]+)\s*=\s*(.*)$/.exec(line);
    if (m && m[1] === name) return m[2].trim().replace(/^["']|["']$/g, '');
  }
  return null;
}

async function get(url, headers = {}) {
  const ac = new AbortController();
  const t = setTimeout(() => ac.abort(), TIMEOUT_MS);
  try {
    return await fetch(url, { headers, signal: ac.signal, redirect: 'follow' });
  } finally {
    clearTimeout(t);
  }
}

/** Первая строка манифеста, которая не комментарий и не пустая. */
function firstChild(text) {
  for (const line of text.split('\n')) {
    const l = line.trim();
    if (l && !l.startsWith('#')) return l;
  }
  return null;
}

function resolve(parentUrl, child) {
  if (child.startsWith('http')) return child;
  const u = new URL(parentUrl);
  if (child.startsWith('/')) return `${u.origin}${child}`;
  return `${u.origin}${u.pathname.replace(/\/[^/]*$/, '')}/${child}`;
}

/**
 * Пройти HLS-цепочку до настоящих байтов видео.
 *
 * ⚠️ ИМЕННО ЭТО И БЫЛО ПРОПУЩЕНО. Останавливаться на первом манифесте нельзя:
 * у Setanta он отвечает 200, а вариант под ним — 404.
 *
 * `depth` — сколько уровней манифестов пройти. Три хватает: master -> variant
 * -> media, дальше идут сегменты.
 */
async function playableBytes(url, depth = 3) {
  let current = url;
  for (let level = 0; level < depth; level += 1) {
    let res;
    try {
      res = await get(current, { Origin: APP });
    } catch (e) {
      return { ok: false, why: `уровень ${level}: ${String(e).slice(0, 40)}` };
    }
    if (!res.ok) return { ok: false, why: `уровень ${level}: HTTP ${res.status}` };

    const body = await res.arrayBuffer();
    const head = new TextDecoder('utf-8', { fatal: false })
      .decode(body.slice(0, 8)).trimStart();

    // Не манифест — значит это уже медиа. Считаем байты: пустой «сегмент»
    // на 300 байт видео не несёт, и принимать его за успех нельзя.
    if (!head.startsWith('#EXTM3U')) {
      return body.byteLength > 50_000
        ? { ok: true, why: `${Math.round(body.byteLength / 1024)} КБ видео` }
        : { ok: false, why: `сегмент всего ${body.byteLength} байт` };
    }

    const text = new TextDecoder().decode(body);
    const child = firstChild(text);
    if (!child) return { ok: false, why: `уровень ${level}: манифест без ссылок` };
    current = resolve(current, child);
  }
  return { ok: false, why: `не дошли до медиа за ${depth} уровня` };
}

// ---------------------------------------------------------------------------
const results = [];
function record(name, ok, detail, control) {
  results.push({ name, ok, detail, control });
}

// ------------------------------------------------------------- каталог -----
async function checkTv() {
  const relay = env('VITE_STREAM_URL')
    ?? 'https://stream-service-production-1616.up.railway.app/playlist.m3u8';

  let text;
  try {
    const r = await get(relay);
    if (!r.ok) { record('ТВ: каталог', false, `HTTP ${r.status}`, 'н/д'); return; }
    text = await r.text();
  } catch (e) {
    record('ТВ: каталог', false, String(e).slice(0, 50), 'н/д');
    return;
  }

  // Тот же отбор, что у приложения: спортивные группы + закреплённые, только https.
  const PIN = ['матч! премьер', 'матч премьер', 'беларусь 5', 'матч! футбол',
    'футбол 1', 'футбол 2', 'футбол 3', 'setanta sports ua', 'setanta',
    'viasat sport', 'eurosport', 'real madrid', 'barca', 'arena sport',
    'diema sport', 'nova sport'];
  const rank = (n) => {
    const low = n.toLowerCase();
    const i = PIN.findIndex((p) => low.includes(p));
    return i === -1 ? PIN.length : i;
  };

  const lines = text.split('\n');
  const seen = new Set();
  const chans = [];
  for (let i = 0; i < lines.length; i += 1) {
    const l = lines[i].trim();
    if (!l.startsWith('#EXTINF')) continue;
    const comma = l.indexOf(',');
    if (comma === -1) continue;
    const name = l.slice(comma + 1).trim();
    const group = /group-title="([^"]*)"/.exec(l)?.[1] ?? '';
    let url = '';
    for (let j = i + 1; j < lines.length; j += 1) {
      const c = lines[j].trim();
      if (!c) continue;
      if (c.startsWith('#EXTINF')) break;
      if (c.startsWith('#')) continue;
      url = c; i = j; break;
    }
    if (!url || !url.startsWith('https://') || seen.has(url)) continue;
    if (!/sport|спорт|футбол|futbol/i.test(group) && rank(name) === PIN.length) continue;
    seen.add(url);
    chans.push({ name, url });
  }
  chans.sort((a, b) => rank(a.name) - rank(b.name));

  record('ТВ: каталог', chans.length > 0,
         `${chans.length} каналов к показу`, 'н/д');
  if (chans.length === 0) return;

  // Первые пять — те, что зритель увидит сверху и на которые нажмёт.
  const top = chans.slice(0, 5);
  const checked = [];
  for (const c of top) {
    const r = await playableBytes(c.url);
    checked.push({ ...c, ...r });
    console.log(`   ${r.ok ? '✓' : '✗'} ${c.name.slice(0, 28).padEnd(28)} ${r.why}`);
  }
  const alive = checked.filter((c) => c.ok);

  // ⚠️ ОТРИЦАТЕЛЬНЫЙ КОНТРОЛЬ. Та же playableBytes на заведомо мёртвом адресе
  // ОБЯЗАНА сказать «не ок». Если скажет «ок» — проверка выше ничего не стоит.
  const neg = await playableBytes('https://stream-service-production-1616.up.railway.app/no-such.m3u8');
  const controlWorks = !neg.ok;

  record('ТВ: верхний канал играет', checked[0]?.ok === true,
         checked[0] ? `${checked[0].name}: ${checked[0].why}` : 'нет каналов',
         controlWorks ? 'контроль упал как должен' : '⚠ КОНТРОЛЬ НЕ УПАЛ — проверка пустая');
  record('ТВ: живых среди первых пяти', alive.length > 0,
         `${alive.length} из ${checked.length}`,
         controlWorks ? 'контроль упал как должен' : '⚠ КОНТРОЛЬ НЕ УПАЛ — проверка пустая');
}


// -------------------------------------------------------------- бандл ------
// Выкачено ли то, что мы думаем. Без адреса раздел пропускается: приложение
// может быть ещё не развёрнуто, и это не повод валить прогон.
async function checkBundle() {
  if (!APP) return;
  try {
    const html = await (await get(APP)).text();
    const main = /\/assets\/index-[^"]+\.js/.exec(html)?.[0];
    if (!main) { record('Прод: бандл', false, 'не найден index-*.js', 'н/д'); return; }
    const js = await (await get(APP + main)).text();

    // ⚠️ ОТРИЦАТЕЛЬНЫЙ КОНТРОЛЬ: строки, которой в бандле быть НЕ МОЖЕТ, поиск
    // обязан не найти. Иначе он «находит» что угодно.
    const has = (s) => js.includes(s);
    const controlWorks = !has('заведомо-отсутствующая-строка-контроля');

    record('Прод: кэш каталога выкачен', has('ml_tv_channels'), main,
           controlWorks ? 'контроль не нашёл несуществующее' : '⚠ КОНТРОЛЬ НАШЁЛ ЧУШЬ');

    // ⚠️ И ГЛАВНОЕ: КАТАЛОГА В ПУБЛИЧНОЙ СБОРКЕ БЫТЬ НЕ ДОЛЖНО. За флагом
    // VITE_SHOW_CATALOGUE стоит группа ♥18+ на 126 записей; если панель
    // случайно собрали с флагом, её строки окажутся в бандле, и узнать об
    // этом надо здесь, а не от зрителя.
    record('Прод: каталог НЕ выкачен', !has('Отсечённые'), main,
           controlWorks ? 'контроль не нашёл несуществующее' : '⚠ КОНТРОЛЬ НАШЁЛ ЧУШЬ');
  } catch (e) {
    record('Прод: бандл', false, String(e).slice(0, 50), 'н/д');
  }
}

// ------------------------------------------------------------- печать -------
console.log(`\nПроверка прода${APP ? `: ${APP}` : ' (только каталог и каналы)'}\n`);
await checkTv();
await checkBundle();

const w = Math.max(...results.map((r) => r.name.length));
console.log('');
for (const r of results) {
  console.log(`${r.ok ? '✓' : '✗'} ${r.name.padEnd(w)}  ${r.detail}`);
  if (r.control.startsWith('⚠')) console.log(`  ${' '.repeat(w)}  ${r.control}`);
}

const failed = results.filter((r) => !r.ok);
const vacuous = results.filter((r) => r.control.startsWith('⚠'));
console.log('');
if (vacuous.length) console.log(`⚠  ПУСТЫХ ПРОВЕРОК: ${vacuous.length} — они не могут упасть, верить им нельзя`);
if (failed.length) console.log(`✗  падений: ${failed.length}`);
if (!failed.length && !vacuous.length) console.log('✓  всё живо, все проверки способны падать');

process.exit(failed.length || vacuous.length ? 1 : 0);
