# Architecture

## Workspace

- `crates/world-kernel`: domain types, invariants, deterministic transitions and causal event records. No database or web dependencies.
- `apps/server`: Axum HTTP boundary, Maud server-rendered UI, PostgreSQL transactions and migrations.
- PostgreSQL: versioned world snapshots plus append-only events keyed by (world_id, sequence).

The application is Rust. No Node, TypeScript, React, Vite or client-side JavaScript runtime is part of the new application.

## Transaction boundary

Every state-changing command:
1. Begins a database transaction.
2. Locks the current world snapshot.
3. Deserializes and validates the world.
4. Executes an atomic domain transition.
5. Updates the snapshot and appends every emitted event in the same transaction.
6. Commits only if all writes succeed.

The domain kernel clones candidate state and only replaces the original after a complete successful transition.

## Event envelope

Each event stores world ID, tick, sequence, actor, cause, affected entities, ruleset version, payload and provenance. Provenance distinguishes generated simulation, historical source, scholarly interpretation, model assumption and derived estimate.

## Current limitations

This is an initial material-world slice, not the complete civilization simulation. Current scenario data is a small explicit model assumption. Replay-from-genesis, deterministic RNG, multi-rate scheduling, authentication, authorization, geography, population cohorts, finance and institutional systems remain unimplemented.
