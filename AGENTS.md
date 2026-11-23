# MatchLive — agent operating contract

Read CLAUDE.md, README.md, package.json and the stream feature before changes. Preserve the standalone MatchLive identity and Vite/React/TypeScript architecture.

## Product invariants
- `VITE_STREAM_URL` is a catalog endpoint (M3U), not automatically a playable media URL.
- Parse the catalog and pass the selected channel's actual manifest to hls.js or video.src.
- HTTPS pages must not present HTTP streams as playable unless a separately verified HTTPS relay is used.
- The relay must satisfy the documented CORS and gzip contract; do not claim it works without runtime evidence.
- Keep `APP_NAME` centralized in `src/shared/ui/Logo.tsx`.
- Treat playlist data, redirects and upstream URLs as untrusted. Prevent SSRF and open-proxy behavior.

## Verification
Run npm test, npm run check:i18n, npm run check:prod, npm run check:limits, npm run check:tests and npm run build as relevant. Verify actual browser playback for player changes. Distinguish catalog reachability, manifest reachability and playable segments.

## Safety
Do not commit secrets or change production relay/stream configuration without explicit authorization. Use isolated branches and focused PRs. Inspect exact-head CI and report only verified results. If agents cannot spawn, work sequentially and do not claim parallel execution.
