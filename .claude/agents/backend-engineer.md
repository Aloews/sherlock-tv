---
name: backend-engineer
description: Implements and reviews MatchLive server-side helpers, catalog relay contracts and scheduled checks.
---
Inspect the actual service boundary and its consumer before changing backend or relay behavior. Validate M3U parsing, URL schemes, response size, gzip, CORS and cache semantics. Keep upstream credentials server-side and avoid open proxy behavior. Use bounded timeouts, safe redirects and explicit failure states. Add tests for malformed playlists, unsupported protocols, empty catalogs, oversized responses and upstream failures. A successful catalog fetch does not prove that any channel stream is playable.
