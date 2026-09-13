import { useState, useCallback, useMemo, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { IconFlag, IconStar, IconStarFilled, IconSearch } from '@tabler/icons-react';
import { hapticImpact, initTelegram } from '@/shared/lib/telegram';
import { OptionRow } from '@/shared/ui/OptionRow';
import { Chip } from '@/shared/ui/Chip';
import { StreamPlayer } from '@/features/stream/StreamPlayer';
import { useChannels } from '@/features/stream/useChannels';
import { orderChannels, nextAlive, filterChannels } from '@/features/stream/order';
import { groupsFor, isPlayable, type ChannelScope } from '@/features/stream/playlist';
import {
  readHealth, markHealth, readFavourites, toggleFavourite,
} from '@/features/stream/channelCache';
import { buildReportMailto } from '@/features/stream/reportMailto';
import { ChannelsPanel } from '@/features/stream/ChannelsPanel';
import { showCatalogue, showAdult } from '@/features/stream/config';
import { TogetherPanel } from '@/features/together/TogetherPanel';
import { LangPicker } from '@/shared/ui/LangPicker';
import { Logo, APP_NAME } from '@/shared/ui/Logo';

// Адрес КАТАЛОГА каналов, а не потока. Публичен по природе: браузеру всё равно
// надо знать, откуда его брать. Разбор — features/stream/playlist.ts.
const PLAYLIST_URL = import.meta.env.VITE_STREAM_URL as string | undefined;

// Контакт для жалобы правообладателя. Не задан — ссылки нет вовсе, а не
// мёртвый mailto:undefined.
const ABUSE_CONTACT = import.meta.env.VITE_STREAM_ABUSE_CONTACT as string | undefined;

// Сколько строк рисуем разом. В режиме «все каналы» их 2158.
const PAGE = 60;

/**
 * Прямой эфир — САМОСТОЯТЕЛЬНОЕ приложение.
 *
 * ⚠️ ЭТО БЫЛ ЭКРАН `/stream` В ИГРЕ, И ПЕРЕЕХАЛ ОН ЦЕЛИКОМ. Логика подбора,
 * порядка, кэша и плеера перенесена дословно, вместе с комментариями: в них
 * записаны замеры и разборы поломок, за которые уже заплачено (каталог на
 * 4081 запись вместо потока, 96 из 127 спортивных каналов по http, отсев
 * мёртвых по факту отказа). Переписывать их «по-новому» значило бы потерять
 * ровно то, ради чего они писались.
 *
 * Что изменилось против экрана в игре — и только это:
 *   • нет кнопки «назад»: возвращаться некуда, приложение само себе корень;
 *   • появился выбор языка: в игре его давал общий каркас;
 *   • каталог всех каналов — за флагом сборки, см. ниже.
 */
export function App() {
  const { t } = useTranslation();
  // ⚠️ РЕЖИМ ЖИВЁТ В СОСТОЯНИИ, А НЕ В СБОРКЕ. Владелец попросил показать
  // весь каталог и выбирать самому: отбор по спортивным группам — это ДОГАДКА
  // приложения о том, что ему нужно, и отменять её должен он, а не пересборка.
  // Открываемся всё равно на спорте: приложение про эфир, и первый экран
  // обязан быть про него.
  const [scope, setScope] = useState<ChannelScope>('sport');
  const [group, setGroup] = useState<string>('');
  const channels = useChannels(PLAYLIST_URL, scope, showAdult());
  const [selected, setSelected] = useState<string | null>(null);
  // Каналы, отказавшие в ЭТОМ сеансе. Список не зашит в код нарочно: три из
  // семи сломанных отдают манифест без CORS, то есть играют на iOS и не играют
  // на Android — статический перечень врал бы половине. См. ./features/stream/order.ts.
  const [dead, setDead] = useState<ReadonlySet<string>>(() => new Set());

  // Исходы прошлых заходов читаются ОДИН раз: пересортировать список под
  // ногами у зрителя, пока он читает, — худшее, что можно сделать.
  const [health] = useState(() => readHealth());
  const [favourites, setFavourites] = useState(() => readFavourites());
  const [query, setQuery] = useState('');
  const [favOnly, setFavOnly] = useState(false);

  useEffect(() => { initTelegram(); }, []);

  // Заголовок вкладки — на языке экрана. В index.html он зашит по-русски как
  // запасной (его видно до загрузки скрипта), но оставлять его над английским
  // или арабским экраном значит показывать чужой язык в самом заметном месте.
  // Вкладка — именем приложения, а не описанием: в ряду из двадцати вкладок
  // человек ищет бренд, а «Прямой эфир» там неотличим от чего угодно.
  useEffect(() => { document.title = APP_NAME; }, []);

  const raw = channels.status === 'ok' ? channels.data : [];
  // Группы считаются по ПОЛНОМУ набору режима, а не по отфильтрованному:
  // иначе выбранная группа исчезала бы из своего же селектора.
  const groups = useMemo(() => groupsFor(raw), [raw]);
  // Порядок считается ОДИН раз по всему списку, отбор — поверх него. Обратный
  // порядок пересортировывал бы результат поиска, и избранное с рабочим
  // оказывались бы не там, где зритель их только что видел.
  const ordered = useMemo(
    () => orderChannels(raw, health, favourites), [raw, health, favourites]);
  const byGroup = useMemo(
    () => (group ? ordered.filter((c) => (c.group || '—') === group) : ordered),
    [ordered, group]);
  const list = useMemo(
    () => filterChannels(byGroup, query, favOnly, favourites),
    [byGroup, query, favOnly, favourites]);

  // ⚠️ РИСУЕМ СОТНЯМИ. В режиме «все каналы» это 2158 строк — столько же узлов
  // DOM, и экран замирает на секунды. Зритель ищет конкретный канал, а не
  // листает каталог целиком.
  const [limit, setLimit] = useState(PAGE);
  useEffect(() => { setLimit(PAGE); }, [scope, group, query, favOnly]);
  // Смена режима и группы сбрасывает выбор: канал из прошлой выборки мог в
  // новую не попасть, и «играет то, чего в списке нет» читается как поломка.
  useEffect(() => { setSelected(null); }, [scope]);
  // Группа под названием только когда она РАЗЛИЧАЕТ: сегодня плейлист отдаёт
  // все каналы из одной `SPORT 🏆`, и подпись превращается в тридцать одну
  // одинаковую строку шума. Появится вторая группа — подпись вернётся сама.
  const showGroup = new Set(list.map((c) => c.group)).size > 1;
  // Первый ЖИВОЙ канал играет сам: экран, который открывается чёрным
  // прямоугольником и ждёт выбора, читается как сломанный.
  // ⚠️ ИГРАЕТ КАНАЛ ИЗ ПОЛНОГО СПИСКА, А НЕ ИЗ ОТФИЛЬТРОВАННОГО. Иначе набор
  // текста в поиске обрывал бы то, что уже играет.
  const playing = selected && !dead.has(selected)
    ? selected
    : nextAlive(ordered, null, dead);

  const handleFailed = useCallback((failedUrl: string) => {
    markHealth(failedUrl, 'failed');
    setDead((prev) => {
      if (prev.has(failedUrl)) return prev;
      const next = new Set(prev);
      next.add(failedUrl);
      return next;
    });
    setSelected(nextAlive(ordered, failedUrl, new Set([...dead, failedUrl])));
  }, [ordered, dead]);

  const handlePlaying = useCallback((url: string) => markHealth(url, 'played'), []);

  // Живых не осталось — это НЕ «каналов нет», и сказать надо разное.
  const allDead = ordered.length > 0 && playing === null;

  return (
    <div className="min-h-screen bg-brand-bg ds-screen flex flex-col">
      <div className="flex items-center gap-2.5 px-4 py-4">
        <Logo size={26} className="text-brand-accent shrink-0" />
        <div className="flex-1 min-w-0">
          <h1 className="ds-display text-white text-xl font-black leading-none">{APP_NAME}</h1>
          {/* Имя — собственное и не переводится; подпись под ним переведена и
              объясняет, что это. Без неё бренд ничего не говорит тому, кто
              видит приложение впервые. */}
          <p className="text-brand-muted text-[10.5px] leading-none mt-1 truncate">
            {t('stream.title')}
          </p>
        </div>
        <LangPicker />
      </div>

      <div className="flex-1 overflow-y-auto px-4 pb-8 flex flex-col items-center gap-3">
        {!PLAYLIST_URL ? (
          <p className="text-brand-muted text-[12px] text-center pt-8">{t('stream.not_configured')}</p>
        ) : (
          <>
            {playing && (
              <StreamPlayer
                key={playing}
                url={playing}
                onFailed={handleFailed}
                onPlaying={handlePlaying}
              />
            )}

            {/* Смотреть вместе — СРАЗУ ПОД ПЛЕЕРОМ, а не в конце списка
                каналов. Разговор идёт о том, что на экране, и уходить за ним
                вниз через шестьдесят строк — значит потерять картинку из
                виду. Панели нет вовсе, если совместный просмотр не настроен
                (VITE_LIVEKIT_TOKEN_URL). */}
            <TogetherPanel
              currentUrl={playing}
              currentTitle={ordered.find((c) => c.url === playing)?.name ?? ''}
              onFollow={setSelected}
            />

            {allDead && (
              <p className="text-brand-muted text-[12px] text-center pt-8">{t('stream.all_failed')}</p>
            )}

            {channels.status === 'loading' && (
              <p className="text-brand-muted text-[12px] text-center pt-8">{t('stream.channels_loading')}</p>
            )}
            {channels.status === 'error' && (
              <p className="text-brand-muted text-[12px] text-center pt-8">{t('stream.channels_failed')}</p>
            )}
            {/* Пусто — это НЕ поломка: плейлист живой, но спортивных каналов,
                которые вообще могут проиграться, в нём сегодня нет. */}
            {channels.status === 'ok' && list.length === 0 && (
              <p className="text-brand-muted text-[12px] text-center pt-8">{t('stream.channels_empty')}</p>
            )}

            {ordered.length > 0 && (
              <div className="w-full max-w-sm flex flex-col gap-1.5 pt-1">
                {/* ⚠️ ТРИ РЕЖИМА, А НЕ ПЕРЕКЛЮЧАТЕЛЬ «СПОРТ / ВСЁ». Между «всем,
                    что может открыться» и «всем, что вообще лежит в каталоге»
                    разница принципиальная: вторых 1597, и ни один из них не
                    заиграет никогда — браузер режет http в https-странице.
                    Свалить их в одну кучу с рабочими значило бы предложить
                    выбирать из списка, где две трети — заведомо мёртвые. */}
                <div className="flex gap-1.5 flex-wrap">
                  {(['sport', 'all', 'everything'] as ChannelScope[]).map((id) => (
                    <Chip
                      key={id}
                      label={t(`stream.scope_${id}`)}
                      selected={scope === id}
                      onClick={() => { hapticImpact('light'); setScope(id); setGroup(''); }}
                    />
                  ))}
                </div>

                {/* Группы селектором, а не чипами: их 58, и рядом чипов они
                    заняли бы весь экран. Число рядом с именем — сколько в ней
                    записей в текущем режиме. */}
                {groups.length > 1 && (
                  <select
                    value={group}
                    onChange={(e) => setGroup(e.target.value)}
                    aria-label={t('stream.group_all')}
                    className="w-full h-11 rounded-full bg-brand-surface border border-brand-border
                               text-white text-xs px-3 focus:outline-none focus:border-brand-accent"
                  >
                    <option value="">{t('stream.group_all', { n: raw.length })}</option>
                    {groups.map((g) => (
                      <option key={g.group} value={g.group}>{g.group} · {g.n}</option>
                    ))}
                  </select>
                )}

                <div className="flex items-center gap-2">
                  <div className="relative flex-1">
                    <IconSearch
                      size={15} stroke={2}
                      className="absolute left-2.5 top-1/2 -translate-y-1/2 text-brand-muted pointer-events-none"
                    />
                    <input
                      value={query}
                      onChange={(e) => setQuery(e.target.value)}
                      placeholder={t('stream.search')}
                      aria-label={t('stream.search')}
                      className="w-full h-11 pl-8 pr-3 rounded-full bg-brand-surface border border-brand-border text-white text-xs placeholder:text-brand-muted focus:outline-none focus:border-brand-accent"
                    />
                  </div>
                  <Chip
                    label={t('stream.favourites')}
                    selected={favOnly}
                    onClick={() => { hapticImpact('light'); setFavOnly((v) => !v); }}
                  />
                </div>

                <p className="text-brand-muted text-[10.5px] uppercase tracking-wide flex items-center gap-1.5">
                  {t('stream.channels_title')}
                  <span className="tabular-nums opacity-70">{list.length}</span>
                </p>

                {/* Пусто после отбора — это НЕ «каналов нет»: список есть,
                    просто под запрос ничего не подошло. */}
                {list.length === 0 && (
                  <p className="text-brand-muted text-[12px] text-center py-6">
                    {t('stream.nothing_found')}
                  </p>
                )}

                {list.slice(0, limit).map((channel) => {
                  const failed = dead.has(channel.url);
                  // ⚠️ http НЕ «ОТКАЗАЛ», А НЕ ОТКРОЕТСЯ В ПРИНЦИПЕ, и сказать
                  // это надо разными словами: первое лечится следующей
                  // попыткой, второе не лечится ничем на нашей стороне.
                  const blocked = !isPlayable(channel);
                  const isFav = favourites.includes(channel.url);
                  return (
                    <div key={channel.url} className="flex items-center gap-2">
                      <div className="flex-1 min-w-0">
                        <OptionRow
                          title={channel.name}
                          // Отказавший канал не прячется, а честно подписывается:
                          // исчезнувшая строка читается как баг, а «канал
                          // недоступен» объясняет, почему по ней больше не нажать.
                          description={
                            blocked
                              ? t('stream.blocked_http')
                              : failed
                                ? t('stream.channel_failed')
                                : showGroup ? channel.group || undefined : undefined
                          }
                          selected={channel.url === playing}
                          disabled={failed || blocked}
                          onClick={() => { hapticImpact('light'); setSelected(channel.url); }}
                        />
                      </div>
                      {/* Звезда — СОСЕД OptionRow, а не часть его: «выбранное»
                          выражает только OptionRow, и дописывать внутрь строки
                          вторую трактовку нельзя. */}
                      <button
                        type="button"
                        onClick={() => { hapticImpact('light'); setFavourites(toggleFavourite(channel.url)); }}
                        aria-label={t(isFav ? 'stream.unfavourite' : 'stream.favourite')}
                        aria-pressed={isFav}
                        className="w-11 h-11 flex items-center justify-center shrink-0 text-brand-muted hover:text-white transition-colors"
                      >
                        {isFav
                          ? <IconStarFilled size={17} className="text-brand-accent" />
                          : <IconStar size={17} stroke={1.75} />}
                      </button>
                    </div>
                  );
                })}

                {list.length > limit && (
                  <button
                    type="button"
                    onClick={() => { hapticImpact('light'); setLimit((n) => n + PAGE); }}
                    className="w-full py-2.5 text-sm text-brand-accent"
                  >
                    {t('stream.show_more', { n: Math.min(PAGE, list.length - limit) })}
                  </button>
                )}
              </div>
            )}

            {/* ⚠️ ПОЛНЫЙ КАТАЛОГ — ЗА ФЛАГОМ СБОРКИ, А НЕ ЗА ПАРОЛЕМ.
                В игре он жил во вкладке кабинета, и дверью был `staffVerify` —
                проверка пароля НА СЕРВЕРЕ. Здесь сервера нет: любой «пароль»
                на клиенте лежит в бандле и открывается просмотром исходников.
                А в каталоге группа `♥18+` на 126 записей, так что притворяться
                дверью тут нельзя. Поэтому выбор честный: сборка либо СОДЕРЖИТ
                каталог, либо нет — оператор собирает себе отдельный экземпляр
                с VITE_SHOW_CATALOGUE=true, а публичная сборка его не содержит
                вовсе, и обойти нечего. */}
            {showCatalogue() && (
              <div className="w-full max-w-sm pt-4">
                <ChannelsPanel />
              </div>
            )}

            {ABUSE_CONTACT && (
              <a
                href={buildReportMailto(
                  ABUSE_CONTACT,
                  t('stream.report_subject'),
                  t('stream.report_body'),
                )}
                onClick={() => hapticImpact('light')}
                className="flex items-center gap-1.5 text-brand-muted text-[10.5px] underline underline-offset-2 pt-1"
              >
                <IconFlag size={13} stroke={1.75} />
                {t('stream.report_link')}
              </a>
            )}
          </>
        )}
      </div>
    </div>
  );
}
