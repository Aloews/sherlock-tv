#!/usr/bin/env node
// Проверка проверок: способна ли каждая из них ВООБЩЕ упасть.
//
// ⚠️ ЗАЧЕМ. В этом приложении 134 юнит-теста, и они были зелёными ровно тогда,
// когда владелец пятый раз писал «ТВ не работает». Зелёная проверка, не
// способная покраснеть, хуже отсутствия проверки: отсутствие видно, а ложная
// зелень внушает уверенность.
//
// Этот скрипт НАРОЧНО ЛОМАЕТ по одной вещи за раз и требует, чтобы
// соответствующая проверка это заметила. Не заметила — она пустая, и скрипт
// валит прогон.
//
// ⚠️ ЛОМАЕТ ОН НАСТОЯЩИЕ ФАЙЛЫ и восстанавливает их в `finally`. Если процесс
// убить посреди прогона, останется испорченный файл — поэтому перед работой
// проверяется, что дерево чистое, а после каждой поломки идёт немедленное
// восстановление из памяти, а не из git.
//
//   node scripts/check-tests.mjs
//
// Выход: 0 — все проверки способны падать; 1 — есть пустая.

import { readFileSync, writeFileSync, existsSync, unlinkSync } from 'node:fs';
import { execSync } from 'node:child_process';

const CASES = [];
function testcase(name, why, fn) { CASES.push({ name, why, fn }); }

/** Прогнать команду. `true` — вышла нулём. */
function passes(cmd, timeoutMs = 300_000) {
  try {
    execSync(cmd, { stdio: 'pipe', timeout: timeoutMs });
    return true;
  } catch {
    return false;
  }
}

/** Испортить файл, прогнать проверку, вернуть файл как был. */
function withBroken(path, mutate, run) {
  const original = readFileSync(path, 'utf-8');
  try {
    writeFileSync(path, mutate(original), 'utf-8');
    return run();
  } finally {
    writeFileSync(path, original, 'utf-8');
  }
}

// ---------------------------------------------------------------------------
testcase(
  'check-i18n замечает пропавший ключ',
  'девять локалей — правило, которое приехало сюда вместе с экраном; ' +
  'проверка, не видящая дыру, выпускает экран с сырым ключом вместо текста',
  () => withBroken('src/i18n/locales/es.json',
    (s) => {
      const d = JSON.parse(s);
      // Убираем ровно один ключ — самый тихий из возможных сбоев.
      delete d.stream.search;
      return JSON.stringify(d, null, 2) + '\n';
    },
    () => !passes('node scripts/check-i18n.mjs')),
);

testcase(
  'tsc замечает несуществующее поле',
  'строгий режим — единственное, что ловит опечатку в имени поля до прода',
  () => withBroken('src/features/stream/playlist.ts',
    (s) => s.replace('return channel.url.startsWith', 'return channel.nosuchfield.startsWith'),
    () => !passes('npx tsc --noEmit')),
);

testcase(
  'vitest замечает сломанное правило отбора',
  'если перевернуть проверку https на обратную, тесты ОБЯЗАНЫ покраснеть — ' +
  'иначе они не проверяют то, ради чего написаны: 96 из 127 каналов по http ' +
  'в https-странице не загрузятся ничем',
  () => withBroken('src/features/stream/playlist.ts',
    (s) => s.replace("return channel.url.startsWith('https://');",
                     "return !channel.url.startsWith('https://');"),
    () => !passes('npx vitest run src/features/stream/playlist.test.ts')),
);

testcase(
  'vitest замечает сломанный порядок каналов',
  'порядок по здоровью — та самая правка, из-за которой мёртвые каналы стояли ' +
  'первыми; тест на неё обязан ловить откат',
  () => withBroken('src/features/stream/order.ts',
    (s) => s.replace("if (h === 'played') return 0;", "if (h === 'played') return 2;"),
    () => !passes('npx vitest run src/features/stream/order.test.ts')),
);

testcase(
  'vitest замечает открытый каталог',
  'за флагом VITE_SHOW_CATALOGUE стоит группа ♥18+ на 126 записей; проверка, ' +
  'которая не краснеет от «открыть всем», не охраняет ничего',
  () => withBroken('src/features/stream/config.ts',
    (s) => s.replace("return import.meta.env.VITE_SHOW_CATALOGUE === 'true';",
                     'return true;'),
    () => !passes('npx vitest run src/features/stream/config.test.ts')),
);

testcase(
  'vitest замечает угадываемый код комнаты',
  'код комнаты — единственный ключ от неё: проверить, кто стучится, релею ' +
  'нечем. Проверка, которая не краснеет от кода «room», не охраняет ничего',
  () => withBroken('src/features/together/roomCode.ts',
    (s) => s.replace('const LENGTH = 10;', 'const LENGTH = 4;'),
    () => !passes('npx vitest run src/features/together/roomCode.test.ts')),
);

testcase(
  'vitest замечает 200 без пропуска',
  'ответ 200 над сломанным следующим шагом называет живым то, что не ' +
  'работает: такой «пропуск» отвергнет уже LiveKit, на шаг позже и не там',
  () => withBroken('src/features/together/tokenApi.ts',
    (s) => s.replace("  if (typeof grant.url !== 'string' || typeof grant.token !== 'string'\n      || grant.url.length === 0 || grant.token.length === 0) {\n    return { status: 'malformed' };\n  }\n", ''),
    () => !passes('npx vitest run src/features/together/tokenApi.test.ts')),
);

testcase(
  'check-prod замечает мёртвый адрес',
  'проверка прода — единственная, способная упасть по той причине, по которой ' +
  'ломается приложение; если она зелёная на несуществующем хосте, она пустая',
  () => !passes('VITE_STREAM_URL=https://no-such-host.invalid/playlist.m3u8 '
                + 'node scripts/check-prod.mjs', 120_000),
);

// ---------------------------------------------------------------------------
// Дерево обязано быть чистым: иначе восстановление затрёт чужие правки.
const dirty = execSync('git status --porcelain', { encoding: 'utf-8' }).trim();
if (dirty) {
  console.error('Рабочее дерево не чистое — скрипт ломает файлы и возвращает их обратно,');
  console.error('и на грязном дереве это опасно. Закоммитьте или спрячьте изменения.\n');
  console.error(dirty.split('\n').slice(0, 10).join('\n'));
  process.exit(2);
}

console.log('\nПроверка проверок: способна ли каждая упасть, если её сломать\n');

const vacuous = [];
for (const c of CASES) {
  process.stdout.write(`  … ${c.name}`);
  let caught = false;
  try {
    caught = c.fn();
  } catch (e) {
    console.log(`\r  ! ${c.name} — сам прогон упал: ${String(e).slice(0, 60)}`);
    vacuous.push(c);
    continue;
  }
  console.log(`\r  ${caught ? '✓' : '✗'} ${c.name}${caught ? '' : '  — ПУСТАЯ'}`);
  if (!caught) vacuous.push(c);
}

// Восстановление могло не сработать — убеждаемся, что дерево снова чистое.
const after = execSync('git status --porcelain', { encoding: 'utf-8' }).trim();
console.log('');
if (after) {
  console.error('⚠ ФАЙЛЫ НЕ ВОССТАНОВЛЕНЫ. Проверьте и откатите вручную:\n' + after);
  process.exit(2);
}

if (vacuous.length === 0) {
  console.log('✓ все проверки краснеют, когда их ломают — им можно верить');
  process.exit(0);
}

console.error(`✗ ПУСТЫХ ПРОВЕРОК: ${vacuous.length}. Зелень от них ничего не значит.\n`);
for (const c of vacuous) console.error(`  ${c.name}\n    ${c.why}`);
process.exit(1);
