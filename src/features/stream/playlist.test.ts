import { describe, it, expect } from 'vitest';
import {
  parseM3u, isPlayable, isSport, isShown, isPinned, pinRank,
  sportChannels, channelsFor, catalogue, groupCounts,
  type Channel,
} from './playlist';

// Строки взяты из БОЕВОГО ответа VITE_STREAM_URL, а не придуманы: именно его
// форма и сломала экран, поэтому проверять надо её, а не удобный образец.
const REAL = `#EXTM3U url-tvg="http://iptvx.one/epg/epg.xml.gz"
#EXTINF:-1,🛠Ревизия №-2 🚦24.08.2026|17-20|👍iptv.org.ua💰
http://cdn10.live-tv.od.ua:8081/leonovtv/test-abr/playlist.m3u8


#EXTINF:-1 group-title="KINO ZAL",Анаконда 2025 США|екшн|пригоди|комедія
https://zetvideo.net/content/stream/films/anaconda_2025/hls/1080/index.m3u8
#EXTINF:-1 group-title="SPORT 🏆",Setanta Sports 1 HD
https://stream8.cinerama.uz/1263/tracks-v1a1/mono.m3u8
#EXTINF:-1 group-title="SPORT 🏆",Матч ТВ
http://37.230.164.98:8080/matchtv/index.m3u8
#EXTINF:-1 group-title="SPORT 🏆",Red Bull TV
https://rbmn-live.akamaized.net/hls/live/590964/BoRB-AT/master_1660.m3u8
`;

describe('parseM3u', () => {
  it('reads every entry, films and channels alike', () => {
    const all = parseM3u(REAL);
    expect(all).toHaveLength(5);
  });

  it('takes the name from after the comma, attributes from before it', () => {
    const [first] = parseM3u('#EXTINF:-1 group-title="SPORT 🏆",Red Bull TV\nhttps://a/b.m3u8');
    expect(first).toEqual<Channel>({
      name: 'Red Bull TV',
      group: 'SPORT 🏆',
      logo: null,
      url: 'https://a/b.m3u8',
    });
  });

  it('reads tvg-logo when the playlist carries one', () => {
    const [c] = parseM3u(
      '#EXTINF:-1 tvg-logo="https://cdn/logo.png" group-title="SPORT",Eurosport\nhttps://a/b.m3u8',
    );
    expect(c.logo).toBe('https://cdn/logo.png');
  });

  // Это ровно та ошибка, ради которой адрес ищется циклом, а не lines[i + 1]:
  // в боевом файле после записи попадаются пустые строки.
  it('finds the url past blank lines, not only on the next line', () => {
    const [c] = parseM3u('#EXTINF:-1 group-title="SPORT",X\n\n\nhttps://a/b.m3u8');
    expect(c.url).toBe('https://a/b.m3u8');
  });

  it('skips comment lines between the entry and its url', () => {
    const [c] = parseM3u('#EXTINF:-1 group-title="SPORT",X\n#EXTGRP:SPORT\nhttps://a/b.m3u8');
    expect(c.url).toBe('https://a/b.m3u8');
  });

  // Запись без адреса не должна «съесть» адрес следующей — иначе канал
  // получит чужой поток, а это хуже, чем его отсутствие.
  it('drops an entry with no url instead of stealing the next one', () => {
    const out = parseM3u(
      '#EXTINF:-1 group-title="SPORT",Broken\n#EXTINF:-1 group-title="SPORT",Good\nhttps://a/b.m3u8',
    );
    expect(out).toHaveLength(1);
    expect(out[0]).toMatchObject({ name: 'Good', url: 'https://a/b.m3u8' });
  });

  it('ignores an entry with no comma and no name', () => {
    expect(parseM3u('#EXTINF:-1 group-title="SPORT"\nhttps://a/b.m3u8')).toEqual([]);
    expect(parseM3u('#EXTINF:-1 group-title="SPORT",\nhttps://a/b.m3u8')).toEqual([]);
  });

  it('survives an empty body and a header-only playlist', () => {
    expect(parseM3u('')).toEqual([]);
    expect(parseM3u('#EXTM3U url-tvg="http://x/epg.xml.gz"')).toEqual([]);
  });

  it('handles CRLF line endings', () => {
    const [c] = parseM3u('#EXTINF:-1 group-title="SPORT",X\r\nhttps://a/b.m3u8\r\n');
    expect(c.url).toBe('https://a/b.m3u8');
  });

  it('leaves the group empty rather than guessing when there is no group-title', () => {
    const [c] = parseM3u('#EXTINF:-1,Ревизия\nhttps://a/b.m3u8');
    expect(c.group).toBe('');
  });
});

describe('isPlayable', () => {
  const at = (url: string): Channel => ({ name: 'X', group: 'SPORT', logo: null, url });

  it('accepts https', () => {
    expect(isPlayable(at('https://a/b.m3u8'))).toBe(true);
  });

  // 96 из 127 спортивных каналов боевого плейлиста именно такие: в https-
  // странице Mini App они не загрузятся никогда.
  it('rejects http — mixed content never loads inside the Mini App', () => {
    expect(isPlayable(at('http://37.230.164.98:8080/matchtv/index.m3u8'))).toBe(false);
  });

  it('rejects a protocol-relative url, which the player cannot resolve either', () => {
    expect(isPlayable(at('//a/b.m3u8'))).toBe(false);
  });

  it('does not accept https as a substring somewhere later in the url', () => {
    expect(isPlayable(at('http://a/redirect?to=https://b/c.m3u8'))).toBe(false);
  });
});

describe('isSport', () => {
  const inGroup = (group: string): Channel => ({ name: 'X', group, logo: null, url: 'https://a' });

  it('matches the playlist group as it is spelled today', () => {
    expect(isSport(inGroup('SPORT 🏆'))).toBe(true);
  });

  // Ради этого проверка по подстроке, а не равенство: группы в этом плейлисте
  // переименовывают руками.
  it('survives the group being renamed or translated', () => {
    expect(isSport(inGroup('Спорт'))).toBe(true);
    expect(isSport(inGroup('Sport HD'))).toBe(true);
    expect(isSport(inGroup('FUTBOL'))).toBe(true);
    expect(isSport(inGroup('Футбол'))).toBe(true);
  });

  it('rejects the groups that must never reach a football game', () => {
    expect(isSport(inGroup('KINO ZAL'))).toBe(false);
    expect(isSport(inGroup('♥18+'))).toBe(false);
    expect(isSport(inGroup('Фильмы-VPN'))).toBe(false);
    expect(isSport(inGroup(''))).toBe(false);
  });
});

describe('sportChannels', () => {
  it('keeps only sport channels that can actually play', () => {
    const out = sportChannels(REAL);
    expect(out.map((c) => c.name)).toEqual(['Setanta Sports 1 HD', 'Red Bull TV']);
  });

  it('drops films even when they are served over https', () => {
    expect(sportChannels(REAL).some((c) => c.group === 'KINO ZAL')).toBe(false);
  });

  // «Матч ТВ» лежит в боевом плейлисте трижды; в списке это выглядит багом.
  it('collapses repeats of the same url', () => {
    const twice = [
      '#EXTINF:-1 group-title="SPORT",Матч ТВ',
      'https://a/match.m3u8',
      '#EXTINF:-1 group-title="SPORT 🏆",Матч HD',
      'https://a/match.m3u8',
    ].join('\n');
    expect(sportChannels(twice)).toHaveLength(1);
  });

  it('keeps two different urls that share a name', () => {
    const two = [
      '#EXTINF:-1 group-title="SPORT",KHL',
      'https://a/1028.m3u8',
      '#EXTINF:-1 group-title="SPORT",KHL',
      'https://a/1422.m3u8',
    ].join('\n');
    expect(sportChannels(two)).toHaveLength(2);
  });

  it('returns nothing rather than throwing on a playlist that failed to load', () => {
    expect(sportChannels('')).toEqual([]);
    expect(sportChannels('<html>404 Not Found</html>')).toEqual([]);
  });
});

describe('pinned channels bypass the group filter', () => {
  // ⚠️ ЭТО И ЕСТЬ ПРИЧИНА, ПО КОТОРОЙ ОТБОР ЗНАЕТ ПРО PINNED. «Матч! Премьер»
  // лежит в группе `Оргтехсервис 🎯VPN`, а не в `SPORT 🏆`, и требование
  // «правильной» группы молча выбрасывало бы названный зрителем канал.
  it('keeps a pinned channel that sits in a non-sport group', () => {
    const text = [
      '#EXTINF:-1 group-title="Оргтехсервис 🎯VPN",Матч! Премьер',
      'https://flussonic.example/tv-x/video.m3u8',
    ].join('\n');
    expect(sportChannels(text).map((c) => c.name)).toEqual(['Матч! Премьер']);
  });

  it('keeps a pinned channel out of TEST-1 too', () => {
    const text = [
      '#EXTINF:-1 group-title="TEST-1",Беларусь 5',
      'https://example.test/by5.m3u8',
    ].join('\n');
    expect(sportChannels(text)).toHaveLength(1);
  });

  // Пропуск мимо группы НЕ значит пропуск мимо https: смешанное содержимое не
  // загрузится ничем, и показывать такую строку — показывать заведомо мёртвое.
  it('still drops a pinned channel served over http', () => {
    const text = [
      '#EXTINF:-1 group-title="TEST-1",Беларусь 5',
      'http://example.test/by5.m3u8',
    ].join('\n');
    expect(sportChannels(text)).toEqual([]);
  });

  it('does not let an unpinned non-sport channel through', () => {
    const text = [
      '#EXTINF:-1 group-title="KINO ZAL",Анаконда 2025',
      'https://example.test/film.m3u8',
    ].join('\n');
    expect(sportChannels(text)).toEqual([]);
  });
});

// ── Каталог для админки ──────────────────────────────────────
//
// Смысл этих проверок один: зрителю и оператору отбор считает ОДИН предикат.
// Разойдутся — оператор увидит «показывается», зритель канала не найдёт, и
// объяснить расхождение будет нечем.
describe('catalogue', () => {
  it('keeps everything the player never sees', () => {
    const all = catalogue(REAL);
    expect(all.map((c) => c.name)).toContain('Анаконда 2025 США|екшн|пригоди|комедія');
    // Каталог ШИРЕ списка зрителя — иначе окно в исходный файл бессмысленно.
    expect(all.length).toBeGreaterThan(sportChannels(REAL).length);
  });

  it('marks WHY a channel is out — group and https are different reasons', () => {
    const by = new Map(catalogue(REAL).map((c) => [c.name, c]));

    const film = by.get('Анаконда 2025 США|екшн|пригоди|комедія');
    expect(film).toMatchObject({ sport: false, playable: true, shown: false });

    // «Матч ТВ» — спортивный и всё равно не в плеере: он по http.
    const matchTv = by.get('Матч ТВ');
    expect(matchTv).toMatchObject({ sport: true, playable: false, shown: false });

    const setanta = by.get('Setanta Sports 1 HD');
    expect(setanta).toMatchObject({ sport: true, playable: true, shown: true });
  });

  // ⚠️ ЭТИ ДВЕ ПРОВЕРКИ НЕ ЛОВЯТ ПОЛОМКУ САМОГО ОТБОРА, и это не упущение:
  // сломай `isShown` — обе стороны сломаются одинаково и останутся согласны
  // (проверено: при `isShown = () => true` падают шесть тестов выше, а эти
  // две нет). Они ловят РАСХОЖДЕНИЕ — тот день, когда кто-нибудь опишет
  // отбор второй раз рядом. Именно от него и защищаемся.
  it('agrees with the player list channel for channel', () => {
    const shownInCatalogue = catalogue(REAL).filter((c) => c.shown).map((c) => c.url);
    expect(shownInCatalogue).toEqual(sportChannels(REAL).map((c) => c.url));
  });

  it('dedupes by url, same as the player list', () => {
    const twice = [
      '#EXTINF:-1 group-title="KINO ZAL",Анаконда',
      'https://example.test/film.m3u8',
      '#EXTINF:-1 group-title="KINO ZAL",Анаконда (повтор)',
      'https://example.test/film.m3u8',
    ].join('\n');
    expect(catalogue(twice)).toHaveLength(1);
  });

  it('isShown is exactly what sportChannels applies', () => {
    for (const c of parseM3u(REAL)) {
      const inPlayer = sportChannels(REAL).some((s) => s.url === c.url);
      expect(isShown(c)).toBe(inPlayer);
    }
  });
});

describe('groupCounts', () => {
  it('orders groups by size and counts what the player gets', () => {
    const rows = groupCounts(catalogue(REAL));
    expect(rows[0]).toMatchObject({ group: 'SPORT 🏆', total: 3, shown: 2 });
    // Запись без group-title не теряется — у неё своя строка.
    expect(rows.some((r) => r.group === '—')).toBe(true);
  });
});

// ── Список владельца ─────────────────────────────────────────
//
// Имена взяты со скриншота 31.08.2026: девять каналов, которые владелец
// назвал сам. Проверка на то, что КАЖДЫЙ из них ловится списком PINNED, —
// иначе строка в нём есть, а канал в приложение не попадает, и заметить это
// нечем: экран выглядит исправным, просто короче.
describe('PINNED ловит каждый канал из списка владельца', () => {
  const WANTED = [
    'Матч ТВ', 'Матч! Футбол 1', 'Матч! Футбол 2', 'Матч! Футбол 3',
    'Матч Премьер', 'Setanta Sports UA', 'Setanta Sports+',
    'Maincast sport', 'Беларусь 5',
  ];

  it.each(WANTED)('%s закреплён', (name) => {
    expect(isPinned({ name, group: 'что угодно', logo: null, url: 'https://x/y.m3u8' }))
      .toBe(true);
  });

  // ⚠️ И РАБОЧАЯ РОДНЯ «Матч ТВ» С ТОЙ ЖЕ РАЗДАЧИ. Группа у них неспортивная
  // (`Оргтехсервис 🎯VPN`), так что мимо PINNED они не проходят вовсе.
  it.each(['Матч! Арена', 'Матч! Игра', 'Матч! Страна', 'Матч! Планета'])(
    '%s закреплён — иначе неспортивная группа его отсечёт',
    (name) => {
      expect(isShown({ name, group: 'Оргтехсервис 🎯VPN', logo: null,
                       url: 'https://flussonic.mkpnet.ru/tv-x/video.m3u8' })).toBe(true);
    },
  );

  // ⚠️ ЗАКРЕПЛЕНИЕ НЕ ОТМЕНЯЕТ https. «Матч! Футбол 1» в каталоге есть только
  // по http, и показывать его нельзя — браузер режет смешанное содержимое.
  // Строка в PINNED для него это «покажем, когда сможем», а не «покажем».
  it('does not smuggle an http channel in just because it is pinned', () => {
    expect(isShown({ name: 'Матч! Футбол 1', group: 'TEST-1', logo: null,
                     url: 'http://37.230.164.98:8080/matchfootball1/index.m3u8' })).toBe(false);
  });

  // Порядок внутри PINNED — это порядок на экране. «Матч ТВ» владелец
  // поставил первым, и он же единственный из списка с двумя рабочими
  // источниками; проверяем, что он и правда впереди остальных названных.
  it('puts Матч ТВ ahead of the rest of the owner list', () => {
    const rank = (name: string) =>
      pinRank({ name, group: '', logo: null, url: 'https://x/y.m3u8' });
    for (const other of ['Матч! Футбол 1', 'Матч Премьер', 'Setanta Sports UA', 'Беларусь 5']) {
      expect(rank('Матч ТВ')).toBeLessThan(rank(other));
    }
  });
});

// ── Режимы отбора ────────────────────────────────────────────
//
// Владелец попросил открыть весь каталог и выбирать самому. Отбор по
// спортивным группам был ДОГАДКОЙ приложения о том, что ему нужно; фильтр по
// https — не догадка, а измеренный факт (браузер режет смешанное содержимое).
// Проверки ниже держат эту границу.
describe('channelsFor', () => {
  const CATALOGUE = [
    '#EXTINF:-1 group-title="SPORT 🏆",Setanta Sports 1 HD',
    'https://sport.test/1.m3u8',
    '#EXTINF:-1 group-title="SPORT 🏆",Матч ТВ по http',
    'http://sport.test/2.m3u8',
    '#EXTINF:-1 group-title="KINO ZAL",Анаконда 2025',
    'https://kino.test/film.m3u8',
    '#EXTINF:-1 group-title="МУЗИКА 🎶",Music Box',
    'https://music.test/1.m3u8',
    '#EXTINF:-1 group-title="Германия 🇩🇪",Das Erste',
    'http://de.test/1.m3u8',
  ].join('\n');

  it('sport: только спорт и только https — как было', () => {
    expect(channelsFor(CATALOGUE, 'sport').map((c) => c.name))
      .toEqual(['Setanta Sports 1 HD']);
  });

  it('all: весь каталог, но только то, что может открыться', () => {
    expect(channelsFor(CATALOGUE, 'all').map((c) => c.name))
      .toEqual(['Setanta Sports 1 HD', 'Анаконда 2025', 'Music Box']);
  });

  it('everything: плюс http, которые не откроются никогда', () => {
    expect(channelsFor(CATALOGUE, 'everything').map((c) => c.name))
      .toEqual(['Setanta Sports 1 HD', 'Матч ТВ по http', 'Анаконда 2025',
                'Music Box', 'Das Erste']);
  });

  it('режимы вложены: sport ⊆ all ⊆ everything', () => {
    const s = new Set(channelsFor(CATALOGUE, 'sport').map((c) => c.url));
    const a = new Set(channelsFor(CATALOGUE, 'all').map((c) => c.url));
    const e = new Set(channelsFor(CATALOGUE, 'everything').map((c) => c.url));
    for (const u of s) expect(a.has(u), u).toBe(true);
    for (const u of a) expect(e.has(u), u).toBe(true);
  });
});

// ⚠️ САМАЯ ВАЖНАЯ ПРОВЕРКА В ЭТОМ ФАЙЛЕ. В боевом каталоге в группе `♥18+`
// 126 записей, 71 из них по https — то есть они бы открылись. Граница не
// зависит от выбранного режима, и «показать всё» её не снимает.
describe('взрослая группа', () => {
  const ADULT = [
    '#EXTINF:-1 group-title="♥18+",Что-то взрослое',
    'https://adult.test/1.m3u8',
    '#EXTINF:-1 group-title="XXX HD",И ещё',
    'https://adult.test/2.m3u8',
    '#EXTINF:-1 group-title="SPORT 🏆",Setanta Sports 1 HD',
    'https://sport.test/1.m3u8',
  ].join('\n');

  // ⚠️ В РЕЖИМЕ `sport` ЭТА ПРОВЕРКА СЛАБЕЕ ОСТАЛЬНЫХ ДВУХ, и это надо знать:
  // взрослый канал там отсекается и без границы — просто потому, что его
  // группа не спортивная. Снятие границы валит два случая из трёх, а не три
  // (проверено). Работают здесь `all` и `everything`; строка про `sport`
  // оставлена, чтобы режим не выпал из перебора, когда появится четвёртый.
  it.each(['sport', 'all', 'everything'] as const)(
    'не проходит в режиме %s',
    (scope) => {
      expect(channelsFor(ADULT, scope).map((c) => c.name)).toEqual(['Setanta Sports 1 HD']);
    },
  );

  // Отрицательная сторона: флаг ДОЛЖЕН открывать её — иначе проверки выше
  // проходили бы и на «выбросить всё подряд», ничего не доказывая.
  it('открывается только явным флагом', () => {
    expect(channelsFor(ADULT, 'all', true).map((c) => c.name))
      .toEqual(['Что-то взрослое', 'И ещё', 'Setanta Sports 1 HD']);
  });
});
