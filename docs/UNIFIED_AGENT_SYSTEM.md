# Unified engineering agents — MatchLive

The agents in `.claude/` supplement `AGENTS.md`, `CLAUDE.md` and `README.md`.

## Roles
- `engineering-lead`: scope, acceptance, coordination and evidence.
- `stream-catalog`: M3U parsing, metadata, filtering and media URL selection.
- `playback-qa`: HLS/video playback, browser compatibility and player states.
- `provider-integration`: catalog relay and upstream service contracts.
- `backend-engineer`: server/relay behavior and safe URL handling.
- `frontend-product-qa`: channel selection, player UX and accessibility.
- `data-integrity`: catalog identity, deduplication and cache correctness.
- `security-reviewer`: SSRF, open proxy, CORS, redirects and secrets.
- `qa-adversarial`: independent attempts to falsify playback and release claims.

Skills: `reality-first`, `release-gate`, `agent-handoff`.
Commands: `/verify`, `/finish-task`, `/security-gate`.
Dispatch: `/dispatch` (launches named specialist subagents when the Claude Code runtime exposes the Task tool; otherwise requires sequential work).

Run `python3 scripts/validate_agent_system.py`. The CI workflow validates file layout and metadata only; it does not prove Claude Code loaded agents or that real channels play.
