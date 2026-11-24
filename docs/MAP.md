# Sherlock implementation map

## Clean-slate rules

- The default branch was reset to a one-file baseline. Old application code is not carried into the new product.
- Git history is preserved for rollback; no history rewrite or force-push was used.
- The new application and all domain logic are Rust. Do not reintroduce React, TypeScript, Node or Vite.
- Do not connect or deploy this foundation to legacy production services or databases.

## Foundation

- [x] Reset the repository tree.
- [x] Define the civilization-simulation product direction.
- [x] Establish domain and simulation crate boundaries.
- [x] Implement world identity, typed causal events and provenance contracts.
- [x] Implement atomic deterministic tick execution.
- [ ] Verify exact-head formatting, Clippy and tests.
- [ ] Add property tests for deterministic replay and failed-tick atomicity.

## First playable vertical slice

- [ ] Scenario creation and versioned world seed.
- [ ] Geography, climate and resource stocks/flows.
- [ ] Population cohorts, households, skills and migration.
- [ ] Labor, production, inventories, logistics and trade across settlements.
- [ ] Money, double-entry accounting, firms, credit and debt.
- [ ] States, institutions, laws, taxation, public goods and legitimacy.
- [ ] Families, ownership, inheritance, succession and dynastic networks.
- [ ] Culture, information, beliefs and historical memory.
- [ ] PostgreSQL snapshots, append-only events and replay from genesis.
- [ ] Rust server and usable interface for inspect, pause, step, fast-forward and intervene.

## Acceptance gates

A milestone is accepted only when invariants and property tests pass, deterministic replay reproduces the same state and event stream, persistence round-trips preserve state, provenance is inspectable, and the interface exercises real domain behavior. A successful compile or health endpoint is not product acceptance.
