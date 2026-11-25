---
name: release-gate
description: MatchLive acceptance gates for catalog correctness, playback and secure delivery.
---
1. Map the change to observable user outcomes and regression tests.
2. Inspect the full diff and catalog/relay/player data flow.
3. Run tests, i18n checks, production checks, typecheck and build.
4. Verify M3U parsing, channel metadata and selected media URL.
5. Verify HTTPS/mixed-content rules, CORS, gzip, redirects and cache behavior.
6. Exercise actual playback in a supported browser where relevant.
7. Test loading, empty, buffering, retry and network failure states.
8. Require exact-head CI and inspect failed/skipped steps.
9. Record commit, run IDs, commands, browser evidence and residual risks.
A successful catalog fetch or green build alone is not playback acceptance.
