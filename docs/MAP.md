# Sherlock delivery map

## Product direction

The canonical product definition is `docs/PRODUCT_CONCEPT.md`. This is a new Rust-only application. Previous sherlock-tv application files are not carried forward. Their Git history remains available for rollback.

## Current foundation

- [x] Clear the working tree of the old application on the rebuild branch.
- [x] Remove old IPTV/streaming, React, TypeScript, Node and Vite application files from the rebuild tree.
- [x] Create a Rust workspace.
- [x] Create a pure Rust domain kernel with settlements, inventories, production, resource sites, extraction, bilateral trade, treasury and causal events.
- [x] Add atomic state transitions and initial invariant tests.
- [x] Add a Rust Axum + Maud server-rendered interface.
- [x] Add PostgreSQL snapshot/event schema and transactional persistence.
- [x] Add Rust-only CI.
- [ ] Pass exact-head formatting, Clippy and test CI.
- [ ] Verify real PostgreSQL create/load/tick/trade/extraction round trips.
- [ ] Implement replay from a genesis snapshot and recorded commands.
- [ ] Add deterministic RNG and explicit time/scheduler contracts.
- [ ] Add geography and environment as first-class domain entities.
- [ ] Add population cohorts, households, labor and migration.
- [ ] Add supply chains and multi-settlement logistics.
- [ ] Add double-entry accounting, finance, firms and credit.

## Release gate

Do not merge or deploy until the exact PR head passes all checks and the PostgreSQL integration test verifies a real persisted world lifecycle. A health endpoint or successful compile is not product acceptance.
