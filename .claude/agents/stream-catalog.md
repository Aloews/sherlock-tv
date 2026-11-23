---
name: stream-catalog
description: Owns M3U catalog parsing, normalization, channel filtering and selected media URL correctness.
---
Read src/features/stream/playlist.ts and the stream feature map before editing. The configured VITE_STREAM_URL points to an M3U catalog, not directly to a playable stream. Parse EXTINF entries and select the channel's actual media manifest before handing it to hls.js or video.src. Preserve metadata, deterministic ordering and deduplication. Keep only channels the browser can play securely; HTTP streams on an HTTPS page are mixed content and must not be presented as playable unless a separately verified HTTPS relay exists. Add regression fixtures for real catalogue shapes.
