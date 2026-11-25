# Architecture

The workspace follows the separation proven in Aloews/Sherlock, adapted to a civilization simulation. It copies architectural boundaries, not football product behavior or legacy implementation.

## Crates

- `game-core`: pure Rust world domain, entity model, event envelope, simulation system contract, deterministic scheduler and transaction boundary. No HTTP, browser, database or UI dependencies.
- `dynasty-core`: family relationships, kinship, succession and inheritance rules. It is a domain subsystem, not a standalone football career mode.
- `api`: Axum transport, SQLx/PostgreSQL persistence adapter, versioned routes, request validation and serialization. Domain rules remain in core crates.
- `web`: Leptos client compiled to WebAssembly. It owns presentation and user interaction, not simulation truth.
- Historical-source adapters will preserve source identity, timestamp, interpretation and uncertainty.

## Dependency direction

`web -> api contracts`; `api -> game-core + dynasty-core`; `dynasty-core -> game-core domain contracts` only when integration is introduced. Core crates never depend on API, web, persistence or external providers.

## Tick transaction

1. Clone or isolate the current world state.
2. Derive a stable random stream per world, tick, ruleset and system ID.
3. Execute systems in stable ID order.
4. Validate the candidate state and event sequence.
5. Commit only if all systems succeed.

A failed system must not partially mutate the caller's world.

## Determinism and events

Random streams are independent per system so adding a new system does not perturb all existing systems. Events carry world ID, tick, sequence, actor, cause, affected entities, ruleset version and provenance. Event IDs are stable within a world/tick sequence. Rules and event schemas require explicit versioning.

## API and client

The API exposes versioned contracts; infrastructure health is separate from application readiness. The client must represent loading, empty, unavailable, partial and error states distinctly. Missing values are never silently converted to zero.



## Starter geography units

The starter map is an 8 × 6 grid. Coordinates are zero-based integer cell indices; adjacency is orthogonal (north, south, east, west). Elevation is measured in meters. Soil fertility is a dimensionless index from 0 to 100. Food and timber stocks, production and consumption are abstract simulation units, not kilograms or tonnes. Treasury is an integer monetary balance in abstract minor units; tax rates are basis points. Starter values are model assumptions, not historical observations. The initial scenario event records the starter-scenario assumption provenance.


## Persistence boundary

The API stores one authoritative JSONB world snapshot per world and an append-only event row for each committed event. Event history is deliberately excluded from JSONB snapshots and rehydrated from the event table, avoiding quadratic snapshot growth. Each successful tick updates the current world snapshot, appends a tick snapshot and inserts all events in one PostgreSQL transaction while holding a row lock for that world. A failed simulation tick leaves the stored snapshot and event log unchanged. Read paths load the snapshot and event rows in one repeatable-read transaction so concurrent ticks cannot produce a mixed view. The replay endpoint rebuilds the initial scenario for the snapshot's ruleset version, replays committed ticks and compares both domain state and the append-only event log. Ruleset v1 retains the original settlement-only starter; ruleset v2 adds ecological resource sites and deterministic regeneration. The current schema does not yet implement replay checkpoints, event compaction, schema-upgrade migrations for historical snapshots, or multi-user authorization.


## Ecological resources

Ruleset v2 creates spatial resource sites from terrain: fish in river cells, timber in forests, stone in hills, iron in mountains and wild food on sufficiently fertile plains. Each site has a typed resource kind, map coordinate, current stock, capacity and regeneration units per tick. Renewable stock is capped at capacity; non-renewable deposits do not regenerate. Resource changes are emitted as causal events. The quantities are abstract simulation units, not measured historical production. Resource extraction, consumption, transport and ecological carrying capacity are not implemented yet.
