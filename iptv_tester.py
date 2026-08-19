import argparse
import csv
import time
import requests
import m3u8
from datetime import datetime

def parse_m3u(playlist_url):
    """Скачивает и парсит M3U-плейлист, возвращает список каналов (name, url)."""
    resp = requests.get(playlist_url, timeout=30)
    resp.raise_for_status()
    lines = resp.text.splitlines()
    channels = []
    current_name = None
    for line in lines:
        if line.startswith('#EXTINF'):
            name_part = line.split(',', 1)[-1].strip()
            current_name = name_part
        elif line and not line.startswith('#'):
            if current_name:
                channels.append({'name': current_name, 'url': line.strip()})
                current_name = None
    return channels

def check_hls(url, timeout=15):
    start = time.time()
    try:
        playlist = m3u8.load(url, timeout=timeout)
        if not playlist.segments:
            return False, None
        seg_url = playlist.segments[0].absolute_uri
        resp = requests.get(seg_url, timeout=timeout, stream=True)
        for chunk in resp.iter_content(chunk_size=1024):
            break
        latency = time.time() - start
        return resp.status_code == 200, latency
    except Exception as e:
        return False, None

def check_ts(url, timeout=15):
    start = time.time()
    try:
        resp = requests.get(url, timeout=timeout, stream=True)
        for chunk in resp.iter_content(chunk_size=1024):
            break
        latency = time.time() - start
        return resp.status_code == 200, latency
    except Exception as e:
        return False, None

def check_stream(url, timeout=15):
    if '.m3u8' in url:
        return check_hls(url, timeout)
    else:
        return check_ts(url, timeout)

def main():
    parser = argparse.ArgumentParser(description='Тестирование IPTV-источников')
    parser.add_argument('--playlist', required=True, help='URL M3U-плейлиста')
    parser.add_argument('--channels', default='', help='Список каналов через запятую для фильтрации (пусто = все)')
    parser.add_argument('--interval', type=int, default=300, help='Интервал между проверками в секундах')
    parser.add_argument('--output', default='iptv_results.csv', help='Файл для сохранения результатов')
    parser.add_argument('--max-checks', type=int, default=0, help='Максимальное число проверок (0 = бесконечно)')
    args = parser.parse_args()

    print(f"Загружаем плейлист: {args.playlist}")
    all_channels = parse_m3u(args.playlist)
    print(f"Найдено каналов: {len(all_channels)}")

    if args.channels:
        filter_names = [name.strip().lower() for name in args.channels.split(',')]
        channels = [ch for ch in all_channels if any(f in ch['name'].lower() for f in filter_names)]
        print(f"Отобрано каналов по фильтру: {len(channels)}")
    else:
        channels = all_channels

    if not channels:
        print("Нет каналов для тестирования.")
        return

    with open(args.output, 'w', newline='', encoding='utf-8') as f:
        writer = csv.writer(f)
        writer.writerow(['timestamp', 'channel_name', 'url', 'is_available', 'latency_sec'])

    check_count = 0
    while True:
        timestamp = datetime.now().isoformat()
        print(f"\n[{timestamp}] Проверка {len(channels)} каналов...")
        for ch in channels:
            ok, latency = check_stream(ch['url'])
            status = 'OK' if ok else 'FAIL'
            if latency is not None:
                print(f"  - {ch['name']}: {status} (latency: {latency:.2f}s)")
            else:
                print(f"  - {ch['name']}: {status}")
            with open(args.output, 'a', newline='', encoding='utf-8') as f:
                writer = csv.writer(f)
                writer.writerow([timestamp, ch['name'], ch['url'], 1 if ok else 0, latency])

        check_count += 1
        if args.max_checks > 0 and check_count >= args.max_checks:
            break

        print(f"Ожидание {args.interval} секунд до следующей проверки...")
        time.sleep(args.interval)

if __name__ == '__main__':
    main()
