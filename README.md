# Sherlock

Rust-first simulation of a living civilization. The implementation follows the proven workspace separation used by [Aloews/Sherlock](https://github.com/Aloews/Sherlock), adapted to a world simulation rather than copying its football product.

## Workspace

- `crates/api` — Axum HTTP API and application composition.
- `crates/game-core` — world domain, simulation systems, deterministic tick engine and event model.
- `crates/dynasty-core` — families, kinship, succession and intergenerational continuity.
- `crates/web` — Leptos WebAssembly client.
- `docs` — product, architecture and implementation contracts.
- `.github` — CI and bounded engineering backlog.

The world simulation is the product. Football is an optional domain that may later use the same population, labor, capital, organizations, finance and culture systems. No legacy Sherlock football application code or production data is copied into this repository.

## Development

Requires stable Rust and the `wasm32-unknown-unknown` target.

```sh
rustup target add wasm32-unknown-unknown
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo check -p sherlock-web --target wasm32-unknown-unknown
cargo run -p sherlock-api
```

The API listens on `0.0.0.0:8080` by default. Override with `SHERLOCK_BIND`.

## Current scope

This commit establishes the workspace boundaries and a small executable vertical foundation. It is not a finished game, persistent world service or production deployment. See [docs/MAP.md](docs/MAP.md) for acceptance gates.
