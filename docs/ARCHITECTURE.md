# Architecture

The workspace follows the separation proven in Aloews/Sherlock, adapted to a civilization simulation. It copies architectural boundaries, not football product behavior or legacy implementation.

## Crates

- `game-core`: pure Rust world domain, entity model, event envelope, simulation system contract, deterministic scheduler and transaction boundary. No HTTP, browser, database or UI dependencies.
- `dynasty-core`: family relationships, kinship, succession and inheritance rules. It is a domain subsystem, not a standalone football career mode.
- `api`: Axum transport, application state, versioned routes, request validation and serialization. Domain rules remain in core crates.
- `web`: Leptos client compiled to WebAssembly. It owns presentation and user interaction, not simulation truth.
- Future persistence adapters will own PostgreSQL snapshots and append-only event storage. Historical-source adapters will preserve source identity, timestamp, interpretation and uncertainty.

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

## Persistence boundary

The next persistence milestone will add PostgreSQL snapshots and append-only events in a single transaction. No production database or legacy credentials are connected by this foundation.


## Starter geography units

The starter map is an 8 × 6 grid. Coordinates are zero-based integer cell indices; adjacency is orthogonal (north, south, east, west). Elevation is measured in meters. Soil fertility is a dimensionless index from 0 to 100. Food and timber stocks, production and consumption are abstract simulation units, not kilograms or tonnes. Treasury is an integer monetary balance in abstract minor units; tax rates are basis points. Starter values are model assumptions, not historical observations. The initial scenario event records the starter-scenario assumption provenance.
