#!/usr/bin/env node
// Замер лимитов, в которые ТВ УЖЕ упиралось.
//
// ЗАЧЕМ. Каждый пункт ниже — не гипотетический потолок из документации, а
// случай, который однажды что-то сломал и стоил расследования:
//
//   вес каталога       870 КБ без сжатия: ~17 с молчания на медленном 3G, и
//                      зритель решает, что ТВ не работает. Релей научили
//                      отдавать gzip (824 343 → 172 835 байт, 2.48 → 0.81 с),
//                      и эта строка следит, чтобы он не разучился
//   localStorage       кэш каналов кладётся туда, и каталог целиком туда не
//                      лёг бы: 870 КБ — почти весь лимит в 5 МБ
//
// ⚠️ ЭТО ЗАМЕР, А НЕ ПРОВЕРКА. Скрипт ничего не заваливает и ничего не
// запрещает: он печатает числа и говорит, сколько осталось. Решение «пушить
// или подождать» остаётся за человеком — порог, при котором стоит подождать,
// у каждого лимита свой и меняется вместе с задачей.
//
//   node scripts/check-limits.mjs
//
// Секреты берутся из окружения и .env, если он есть; чего нет — честно
// помечается как «не измерено», а не подставляется наугад.

import { readFileSync, existsSync, statSync, readdirSync } from 'node:fs';
import { join } from 'node:path';

const TIMEOUT_MS = 20_000;

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
    return await fetch(url, { headers, signal: ac.signal });
  } finally {
    clearTimeout(t);
  }
}

const rows = [];
/** @param status 'ok' | 'warn' | 'skip' */
function row(name, value, headroom, status = 'ok') {
  rows.push({ name, value, headroom, status });
}

// ---------------------------------------------------------------- GitHub ----
async function github() {
  const token = env('GITHUB_TOKEN') ?? env('GH_TOKEN');
  if (!token) {
    row('GitHub API', 'не измерено', 'нет GITHUB_TOKEN в окружении', 'skip');
    return;
  }
  try {
    const r = await get('https://api.github.com/rate_limit', {
      Authorization: `Bearer ${token}`, Accept: 'application/vnd.github+json',
    });
    const d = await r.json();
    // ⚠️ ОТСУТСТВИЕ СТРОКИ ХУЖЕ ПЛОХОЙ СТРОКИ. Если ответ пришёл не той формы
    // (сменился API, токен без прав), молчаливый `continue` убирал лимит из
    // таблицы совсем — и «GitHub не показан» читалось бы как «с ним всё
    // хорошо». Ровно та ошибка, которую этот скрипт и призван ловить.
    if (!d.resources?.core && !d.resources?.graphql) {
      // 401/403 — это «нечем мерить», а не «лимит исчерпан», и путать их
      // нельзя: первое чинится токеном, второе — ожиданием.
      const noAccess = r.status === 401 || r.status === 403;
      row('GitHub API', noAccess ? 'нет доступа' : 'ответ не разобран',
          noAccess ? `HTTP ${r.status} — токен не подошёл к api.github.com`
                   : `HTTP ${r.status}, нет resources`,
          noAccess ? 'skip' : 'warn');
      return;
    }
    for (const key of ['core', 'graphql']) {
      const c = d.resources?.[key];
      if (!c) continue;
      const mins = Math.max(0, Math.round((c.reset * 1000 - Date.now()) / 60000));
      // ⚠️ ЧЕРНОВИК СНИМАЕТСЯ ТОЛЬКО ЧЕРЕЗ GRAPHQL. У него свой счётчик, и
      // исчерпанный graphql при живом core значит «код готов, а смержить
      // нечем» — ровно то, обо что эта сессия и споткнулась.
      row(`GitHub ${key}`, `${c.remaining} из ${c.limit}`,
          c.remaining === 0 ? `сброс через ${mins} мин` : 'ок',
          c.remaining === 0 ? 'warn' : 'ok');
    }
  } catch (e) {
    row('GitHub API', 'ошибка', String(e).slice(0, 60), 'warn');
  }
}

// -------------------------------------------------------------- Supabase ----
async function supabase() {
  const url = env('VITE_SUPABASE_URL');
  const key = env('VITE_SUPABASE_ANON_KEY');
  if (!url || !key) {
    row('PostgREST db-max-rows', 'не измерено', 'нет VITE_SUPABASE_* ', 'skip');
    return;
  }
  try {
    // ⚠️ ПРОСИМ БОЛЬШЕ, ЧЕМ ВЛЕЗЕТ, И СМОТРИМ НА CONTENT-RANGE. Сервер режет
    // ответ молча и отвечает 200: `?limit=5000` однажды вернул ровно 1000
    // строк при `Content-Range: 0-999/2919`, и оба сборщика приняли это за
    // всю колоду. Признак усечения — ТОЛЬКО в заголовке.
    const r = await get(`${url}/rest/v1/cards?select=id&limit=5000`, {
      apikey: key, Authorization: `Bearer ${key}`, Prefer: 'count=exact',
    });
    const range = r.headers.get('content-range');
    const got = (await r.json()).length;
    const total = range?.split('/')[1];
    row('PostgREST db-max-rows', `отдал ${got} строк`,
        total && Number(total) > got
          ? `⚠ в таблице ${total} — ответ УСЕЧЁН, читать страницами с order=`
          : `в таблице ${total ?? '?'} — влезло целиком`,
        total && Number(total) > got ? 'warn' : 'ok');
  } catch (e) {
    row('PostgREST db-max-rows', 'ошибка', String(e).slice(0, 60), 'warn');
  }
}

// ------------------------------------------------------------- релей ТВ -----
async function relay() {
  const url = env('VITE_STREAM_URL');
  if (!url || !url.startsWith('http')) {
    row('Каталог ТВ', 'не измерено', 'нет VITE_STREAM_URL', 'skip');
    return;
  }
  try {
    const started = Date.now();
    const r = await get(url, { 'Accept-Encoding': 'gzip, deflate, br' });
    const body = await r.arrayBuffer();
    const ms = Date.now() - started;
    const kb = Math.round(body.byteLength / 1024);
    const enc = r.headers.get('content-encoding');
    const cache = r.headers.get('cache-control') ?? '—';
    // 400 кбит/с — медленный 3G; именно на нём экран молчит достаточно долго,
    // чтобы игрок ушёл.
    const slow3g = (body.byteLength * 8) / 1000 / 400;
    row('Каталог ТВ', `${kb} КБ за ${ms} мс`,
        enc ? `сжат (${enc}), cache ${cache}`
            : `⚠ БЕЗ СЖАТИЯ, cache ${cache} → ~${slow3g.toFixed(0)} с на 3G`,
        enc ? 'ok' : 'warn');
  } catch (e) {
    row('Каталог ТВ', 'ошибка', String(e).slice(0, 60), 'warn');
  }
}


// ------------------------------------------------------------- бандл -------
function bundle() {
  const dir = 'dist/assets';
  if (!existsSync(dir)) {
    row('Вес первого захода', 'не измерено', 'нет dist — сначала npm run build', 'skip');
    return;
  }
  // ⚠️ ПЕРВЫЙ ЗАХОД И ДОГРУЖАЕМОЕ — РАЗНЫЕ ЧИСЛА, И СУММА ИХ ВРЁТ. Раньше
  // строка складывала ВСЁ, что лежит в dist/assets. С появлением совместного
  // просмотра там появился кусок livekit-client на 586 КБ, который не
  // грузится, пока никто не нажал «смотреть вместе», — и общая сумма
  // объявила бы первый заход потяжелевшим на две трети, не будучи им.
  //
  // Разделяются по имени: entry-куски Vite зовёт `index-*`, ленивые — по
  // имени модуля (`livekit-client.esm-*`).
  let first = 0;
  let lazy = 0;
  const lazyNames = [];
  for (const f of readdirSync(dir)) {
    if (!f.endsWith('.js') && !f.endsWith('.css')) continue;
    const size = statSync(join(dir, f)).size;
    if (f.startsWith('index-')) {
      first += size;
    } else {
      lazy += size;
      lazyNames.push(f.replace(/-[A-Za-z0-9_-]{8}\.js$/, ''));
    }
  }
  const kb = Math.round(first / 1024);
  // hls.js — примерно треть этого веса, и на iOS он не нужен вовсе: там
  // воспроизведение нативное (см. useHlsPlayer). Динамический import() под
  // ветку «нет нативной поддержки» — очевидный следующий шаг, но он меняет
  // перенесённый код, а не адрес, поэтому сделан отдельно от переезда.
  row('Вес первого захода', `${kb} КБ`,
      kb > 900 ? '⚠ вырос — проверить, что попало в бандл' : 'в пределах прежнего',
      kb > 900 ? 'warn' : 'ok');

  // ⚠️ СТРОКА ПЕЧАТАЕТСЯ ДАЖЕ КОГДА ЛЕНИВЫХ КУСКОВ НЕТ. Их отсутствие значит
  // одно из двух: либо их правда нет, либо сборка шла БЕЗ VITE_STREAM_URL — а
  // тогда Vite вырезает мёртвой веткой весь плеер вместе с панелью
  // совместного просмотра, и «вес первого захода» занижен на треть. Молчащая
  // строка выдала бы второй случай за первый.
  row('Догружается по требованию', lazy ? `${Math.round(lazy / 1024)} КБ` : 'нет',
      lazy ? lazyNames.join(', ')
           : 'сборка без VITE_STREAM_URL вырезает плеер и панель — число выше занижено',
      lazy ? 'ok' : 'warn');

  const cache = 5 * 1024;
  row('localStorage', `лимит ~${cache} КБ`,
      'кладём РЕЗУЛЬТАТ разбора (единицы КБ); каталог на 870 КБ туда не лёг бы', 'ok');
}

// ------------------------------------------------------------- печать -------
const MARK = { ok: '  ', warn: '⚠ ', skip: '· ' };

await relay();
bundle();

const w1 = Math.max(...rows.map((r) => r.name.length));
const w2 = Math.max(...rows.map((r) => r.value.length));
console.log('\nЛимиты, в которые ТВ уже упиралось:\n');
for (const r of rows) {
  console.log(`${MARK[r.status]}${r.name.padEnd(w1)}  ${r.value.padEnd(w2)}  ${r.headroom}`);
}

const warned = rows.filter((r) => r.status === 'warn');
const skipped = rows.filter((r) => r.status === 'skip');
console.log('');
if (warned.length) console.log(`⚠  требует внимания: ${warned.length}`);
if (skipped.length) console.log(`·  не измерено: ${skipped.length} (нет доступа или сборки)`);
if (!warned.length && !skipped.length) console.log('Всё измерено, запаса хватает.');

// ⚠️ НЕ ЗАВАЛИВАЕМ ПРОГОН. Скрипт печатает числа; решение «выкатывать или
// подождать» остаётся за человеком, и порог у каждой строки свой.
process.exit(0);
