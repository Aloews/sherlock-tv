# Engineering agent contract

Agents working in this repository operate as a coordinated engineering team.

## Roles
- Systems architect: preserve domain boundaries, deterministic time, explicit units and causal event contracts.
- World-systems engineer: implement geography, ecology, population, production, trade, finance and institutions in game-core.
- API engineer: expose versioned, validated commands and queries; never place domain rules in handlers.
- Product engineer: connect real API behavior to the Leptos client; implement loading, empty, unavailable and error states.
- Simulation QA: challenge invariants, determinism, rollback, replay and serialization; add regression tests before accepting a task.
- Release/readiness reviewer: inspect the exact PR head, run all CI gates, verify browser and API integration, and reject claims unsupported by evidence.

## Required workflow
1. Read README.md, docs/PRODUCT_CONCEPT.md, docs/ARCHITECTURE.md, docs/MAP.md and .github/agent-backlog.json before implementation.
2. Select one bounded backlog task and record it in .github/agent-state.json.
3. Inspect current code and tests before changing files.
4. Implement the smallest coherent vertical increment. Do not mark documentation-only work as product completion.
5. Add tests for normal, boundary, invalid and rollback cases. Preserve deterministic behavior for a fixed seed and ruleset.
6. Run cargo fmt --all -- --check, cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace --all-targets, cargo check -p sherlock-web --target wasm32-unknown-unknown and cargo build --release -p sherlock-api.
7. Inspect the exact commit's GitHub Actions result and changed files. A queued or running check is not a pass.
8. Update docs/MAP.md and agent state only with verified results. Never claim deployment, persistence, browser integration or readiness without direct evidence.

## Stop conditions
- Any red CI check, failing invariant, nondeterministic replay, unbalanced ledger, negative physical inventory or unverified external data.
- A change that copies legacy football application behavior into the civilization kernel.
- A feature that uses fabricated values without marking them as simulation assumptions.

## Definition of ready
A playable milestone must create a world, inspect its state, execute a real tick, show resulting state in the browser, preserve event causality, and pass all required quality gates. PostgreSQL snapshots and the separate append-only event stream are authoritative; read consistency and transactional tick writes must be tested. Any remaining in-memory-only state must be explicitly identified.
