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
- [ ] Geography, climate, hydrology and spatial resource locations.
- [ ] Population cohorts, households, skills, health and migration.
- [ ] Labor, production functions, inventories, logistics and trade across settlements.
- [ ] Money, double-entry accounting, firms, credit and debt.
- [ ] States, institutions, laws, taxation, public goods and legitimacy beyond the starter tax calculation.
- [ ] Families, ownership, inheritance, succession and dynastic networks.
- [ ] Culture, information, beliefs and historical memory.
- [ ] PostgreSQL snapshots, append-only event persistence and replay from genesis.
- [~] Rust API commands for create, inspect and step; pause and fast-forward remain.
- [~] Leptos interface for create/step and settlement inspection; event history, persisted sessions and production API configuration remain.

## Current implementation boundary

The current playable-slice branch contains an in-memory starter world with two settlements, deterministic tick execution, food/timber/population/treasury transitions, versioned causal tick events and recent event history, create/inspect/step HTTP routes and a Leptos client that invokes those routes. State is process-local and is lost on API restart. No map, save/load, PostgreSQL persistence, multi-user isolation or production deployment is implemented yet. Treat this as an interactive prototype, not a complete or production-ready game.

## Acceptance gates

A milestone is accepted only when invariants and property tests pass, deterministic replay reproduces the same state and event stream, persistence round-trips preserve state, provenance is inspectable, and the interface exercises real domain behavior. A successful compile or health endpoint is not product acceptance.
