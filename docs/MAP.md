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

- [ ] Scenario creation and versioned world seed.
- [ ] Geography, climate, hydrology and resource stocks/flows.
- [ ] Population cohorts, households, skills, health and migration.
- [ ] Labor, production, inventories, logistics and trade across settlements.
- [ ] Money, double-entry accounting, firms, credit and debt.
- [ ] States, institutions, laws, taxation, public goods and legitimacy.
- [ ] Families, ownership, inheritance, succession and dynastic networks.
- [ ] Culture, information, beliefs and historical memory.
- [ ] PostgreSQL snapshots, append-only events and replay from genesis.
- [ ] Rust API commands for create, inspect, pause, step and fast-forward.
- [ ] Usable interface exercising real domain behavior.

## Acceptance gates

A milestone is accepted only when invariants and property tests pass, deterministic replay reproduces the same state and event stream, persistence round-trips preserve state, provenance is inspectable, and the interface exercises real domain behavior. A successful compile or health endpoint is not product acceptance.
