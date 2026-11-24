# Sherlock

A Rust-first, persistent simulation of civilization. The world is the product: ecology, people, knowledge, production, exchange, finance, institutions, power, culture and historical memory evolve as connected systems.

Football and other real-world industries are optional domains built on the same world model, never the core of the simulation.

## Status

The repository has been reset. This is the first clean-slate engineering foundation, not a finished game.

## Workspace

- crates/domain — stable domain contracts, world identity, entities and causal events.
- crates/simulation — deterministic, atomic simulation ticks and ordered system execution.

## Local checks

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
