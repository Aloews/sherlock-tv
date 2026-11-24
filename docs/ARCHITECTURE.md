# Architecture

## Design boundaries

- domain owns stable identifiers, domain entities, time, provenance and event contracts. It has no database, HTTP or UI dependencies.
- simulation owns deterministic execution, system ordering, tick transactions and simulation rules. It operates on domain state and returns causal events.
- Future adapters (persistence, historical-data ingestion, API, UI and scenario tooling) depend inward on these crates. Domain and simulation never depend on adapters.

## Simulation transaction

A tick is atomic:
1. Clone or otherwise isolate the current state.
2. Derive a deterministic random stream from world seed, tick and ruleset.
3. Execute systems in stable identifier order.
4. Validate resulting state and event sequence.
5. Commit the new state only if every system succeeds.

A failed system must not partially mutate the caller's world.

## Event envelope

Every material event carries world ID, tick, monotonically increasing sequence, actor, cause, affected entity IDs, ruleset version, provenance and a typed payload. Persistence will store snapshots and append-only event history in one database transaction.

## Determinism

Randomness is seeded per world and tick. System execution order is stable. Changes to rules or event schemas require explicit versioning. Replay tests compare resulting state and event streams, not just successful return codes.

## Planned adapters

PostgreSQL event store and snapshots; historical-source registry; scenario loader; Rust HTTP server; server-rendered interface; observability and operational tooling. These are deliberately not mixed into the domain kernel.
