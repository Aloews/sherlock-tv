# MatchLive — Project Map

Last reviewed: 2026-11-14. Initial map seeded from README; verify source and current production behaviour before relying on historical measurements.

## Product scope
Standalone sports-channel catalogue and playback UI based on M3U catalogue parsing. The README states that the catalogue/playlist relay is hosted by `stream-service` in `Aloews/sherlock-ai-bot`.

## Main navigation
- `src/features/stream/`: catalogue parsing, filtering, ordering, cache and playback
- `src/App.tsx`: application screen
- `src/shared/ui/Logo.tsx`: canonical app name/logo
- `iptv_tester.py`: scheduled channel checks
- `docs/ADR-0004-live-tv-playlist-relay.md`: rights and relay decision record

## Critical product constraints
- The source URL is a catalogue, not a playable HLS stream; resolve the selected channel manifest before playback.
- Only HTTPS streams are playable from the HTTPS app; mixed content is blocked by browsers.
- The application displays links/catalogue data and must not retransmit third-party video without rights.
- LiveKit carries room voice/camera/chat metadata; it does not relay the TV stream.
- Keep public viewer UI free of staff-only catalogue data and credentials.

## Verification
Run the repository's current typecheck, production build, tests and i18n check; confirm command names from package.json before execution. Verify stream relay CORS, compression and catalogue freshness against live HTTP responses.

## AI agent operating contract
- Do not weaken playback, rights, privacy or content-exposure boundaries.
- Use isolated branches and draft PRs; no automatic production deployment.
- Inspect current source and deployment before changes.


## AI engineering toolchain — required project standard

This section is shared across the active project maps. It records installation and runtime requirements; a repository entry is **not** proof that a tool is installed or active.

### 1. Caveman — concise agent output

- Upstream: https://github.com/JuliusBrussee/caveman
- Claude Code: `claude plugin marketplace add JuliusBrussee/caveman && claude plugin install caveman@caveman`
- Codex CLI: `npx -y github:JuliusBrussee/caveman -- --only codex` (or install its skill with `npx skills add JuliusBrussee/caveman -a codex -g`).
- Use the project/user scope matching the actual coding-agent host. Restart/reload the host and verify the skill is discoverable before counting it as installed.
- Do not apply compression to source code, identifiers, error messages, test assertions, API contracts, security rules, or acceptance criteria. Preserve negations and exact technical facts.

### 2. codebase-memory-mcp — persistent code graph

- Upstream: https://github.com/DeusData/codebase-memory-mcp
- Supported host install (macOS/Linux): `curl -fsSL https://raw.githubusercontent.com/DeusData/codebase-memory-mcp/main/install.sh | bash`
- The installer detects supported coding-agent clients and registers MCP/instructions where available. Restart the coding agent, verify the MCP server is visible, then explicitly index this repository.
- Keep the graph/index on a persistent developer or agent host. A disposable GitHub Actions runner or ephemeral Railway cron container is not a persistent memory installation.
- Before production rollout, pin and verify the release/checksum; do not silently install unpinned `main` into a production runtime.

### 3. Spotify Portal AI Plugins — AiKA delegation

- Upstream: https://github.com/spotify/portal-ai-plugins
- Claude Code:
  `claude plugin marketplace add spotify/portal-ai-plugins`
  `claude plugin install portal@portal`
  `claude plugin install shunt@portal`
- Codex: `codex plugin marketplace add spotify/portal-ai-plugins`, then install Spotify Portal from `/plugins`. Cursor is installed through its team marketplace.
- Start a new supported agent session and run `/portal:setup`; authenticate the Portal CLI. Run its read-only doctor/readiness checks.
- The shunt plugin is Claude Code-only. It requires Portal CLI authentication and AiKA availability; verify `bulk-reader` and `code-writer` modes before relying on delegation. Do not create modes or change Portal settings without explicit authorization.
- Never put Portal credentials or local MCP configuration in this repository.

### 4. Runtime compatibility and installation evidence

- These are coding-agent integrations, not generic Python/Node runtime packages. A worker that only sends HTTP requests to the Starimg API cannot load Claude/Codex plugins or call a local MCP server unless it is explicitly given that agent runtime and MCP client.
- GitHub Actions runners are ephemeral: use them for bounded jobs, CI checks and a temporary fallback agent, not as the persistent host for a local code graph or authenticated Spotify Portal session.
- The Railway `stream-service` worker is a Python/Starimg HTTP worker; its current code path does not expose a Claude Code/Codex plugin host or MCP client. Do not claim these integrations are installed there.
- Record each tool independently as `not installed`, `installed`, `connected`, `indexed`, and `verified`. Only advance a state after observing its exact runtime check. Never store tokens or secret values in this map.
- Current blocker: the last Starimg Fable task request returned HTTP 402 before model execution. Do not retry billable generation until the provider returns a successful, authenticated test request.

### 5. GitHub Actions fallback

- When the Railway worker is unavailable, use a repository-owned GitHub Actions agent workflow only if that workflow exists, its trigger/permissions are verified, and the model/provider can be invoked successfully.
- For Sherlock Scholes, `.github/workflows/local-autonomous-agent.yml` listens for an owner-authored issue comment beginning with `/ai ` and runs the local Qwen2.5-Coder 3B model on a GitHub-hosted runner. It is a separate fallback from Starimg/Fable, has a restricted patch allowlist, and must produce a draft PR with CI evidence. It is not Fable 5.2.
- Do not assume another repository has this fallback: inspect its workflow and trigger before requesting work there.
- No automatic merge or production deployment. Review the exact PR head, run required checks, and verify deployment separately.

### Installation status (2026-11-14)

| Integration | Repository documentation | Active runtime installation |
|---|---|---|
| Caveman | Project skills at `.agents/skills/caveman/` and `.claude/skills/caveman/` | Included in this PR; host discovery/activation not yet verified |
| codebase-memory-mcp | Recorded here | Not verified; no persistent supported host indexed |
| Spotify Portal / shunt | Recorded here | Not verified; Portal authentication and AiKA readiness not established |

