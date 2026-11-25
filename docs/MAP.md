# Sherlock implementation map

## Foundation

- [x] Reset the target repository tree while preserving Git history.
- [x] Adopt the four-crate Rust workspace layout from Aloews/Sherlock.
- [x] Separate world simulation, dynastic domain, HTTP API and WASM client.
- [x] Define product, architecture and evidence contracts.
- [x] Add deterministic tick execution and rollback tests.
- [x] Add API health/world-summary boundary and Leptos application shell.
- [ ] Verify exact branch commit in GitHub Actions, including WASM target.
- [ ] Commit a generated Cargo.lock after dependency resolution.

## World simulation vertical slice

- [~] Scenario creation and versioned world seed — starter scenario exists; seed controls world identity and deterministic simulation streams; initial scenario event is recorded.
- [~] Population, food, renewable timber and treasury transitions — initial settlement-level economy exists; cohort/household detail remains.
- [~] Typed grid geography, deterministic 8×6 starter map, terrain, climate bands, elevation, soil fertility, river corridor, water access, regions and adjacency; hydrology and spatial resource sites remain.
- [ ] Population cohorts, households, skills, health and migration.
- [ ] Labor, production functions, inventories, logistics and trade across settlements.
- [ ] Money, double-entry accounting, firms, credit and debt.
- [ ] States, institutions, laws, taxation, public goods and legitimacy beyond the starter tax calculation.
- [ ] Families, ownership, inheritance, succession and dynastic networks.
- [ ] Culture, information, beliefs and historical memory.
- [~] PostgreSQL world records, per-tick JSONB snapshots and append-only event rows; deterministic replay verification from genesis is implemented, while replay checkpoints and retention/compaction policy remain.
- [~] Rust API commands for create, inspect, step and deterministic replay verification; pause and fast-forward remain.
- [~] Leptos interface for create/load/step/replay verification, map, settlement and event-history inspection; persisted browser sessions and production API configuration remain.

## Current implementation boundary

The current playable-slice branch contains a PostgreSQL-backed starter world with two settlements, deterministic tick execution, food/timber/population/treasury transitions, versioned causal tick events and recent event history, create/inspect/step/replay-verification HTTP routes and a Leptos client that invokes create/inspect/step/replay-verification. World state and event history are stored in PostgreSQL and survive API restarts. Replay verification rebuilds the world from its seed, replays committed ticks, and compares both the reconstructed domain snapshot and append-only event log. API tests use a real PostgreSQL service. Multi-user authorization/isolation, replay checkpoints, pause/fast-forward, production deployment and production hardening remain unimplemented. Treat this as an interactive prototype, not a complete or production-ready game.

## Acceptance gates

A milestone is accepted only when invariants and property tests pass, deterministic replay reproduces the same state and event stream, persistence round-trips preserve state, provenance is inspectable, and the interface exercises real domain behavior. A successful compile or health endpoint is not product acceptance.
