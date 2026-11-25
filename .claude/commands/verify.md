# Verify MatchLive task

1. Read AGENTS.md, CLAUDE.md, README.md and stream feature sources.
2. Record branch, exact HEAD and changed paths.
3. Run npm test, npm run check:i18n, npm run check:prod, npm run check:limits, npm run check:tests and npm run build as applicable.
4. Verify catalog parsing and selected media URL, not just catalog reachability.
5. Test HTTP mixed-content exclusion and relay CORS/gzip/redirect behavior.
6. Exercise real browser playback and capture console/network evidence where relevant.
7. Test empty catalog, malformed entry, unsupported channel, buffering and failure/retry.
8. Inspect exact-head CI and report PASS only with evidence.
