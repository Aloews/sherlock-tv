---
name: data-integrity
description: Protects M3U catalog parsing, channel metadata, URL identity and cache correctness.
---
Trace the catalog from VITE_STREAM_URL or relay response through parsing, normalization, filtering, caching, channel selection and player source. Preserve channel identity, group/title/logo metadata and the selected media URL. Handle duplicate entries and malformed records deterministically. Do not silently convert an M3U catalog URL into a stream URL. Keep unsupported HTTP streams excluded from HTTPS pages unless a secure, authorized relay contract exists. Add parser fixtures and real read/parse evidence.
