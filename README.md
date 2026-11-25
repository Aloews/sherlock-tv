# Sherlock Civilization

Rust-first simulation of a living civilization. The world simulation is the product: geography, ecology, population, production, trade, finance, institutions, dynasties, culture and historical memory evolve together.

The repository is a clean rebuild. No legacy Sherlock football application code or production data is copied here. Football may later be an optional domain using the shared civilization systems.

## Workspace

- `crates/game-core` — deterministic world state, geography, simulation systems and causal event model.
- `crates/api` — Axum HTTP API, PostgreSQL snapshots and append-only event storage.
- `crates/web` — Leptos WebAssembly client.
- `crates/dynasty-core` — families, kinship, succession and intergenerational continuity.
- `docs` — product, architecture and implementation contracts.
- `.github/agents` — specialized engineering agent profiles.
- `.github/agent-backlog.json` — bounded implementation and readiness tasks.

## Run locally

Requires stable Rust, Docker Compose, the `wasm32-unknown-unknown` target and Trunk.

1. Start PostgreSQL:

```sh
docker compose up -d postgres
```

2. Export the local development connection string:

```sh
export DATABASE_URL=postgres://sherlock:sherlock@localhost:5432/sherlock
```

3. Start the API. Database migrations are applied automatically at startup:

```sh
cargo run -p sherlock-api
```

The API listens on `0.0.0.0:8080` by default. Override with `SHERLOCK_BIND`.

4. In another terminal, build and serve the browser client:

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve --manifest-path crates/web/Cargo.toml
```

Open the local Trunk URL. Set the API address in the client to the reachable API origin. For a deployed browser client, set `SHERLOCK_API_URL` at build time or enter the API URL in the UI.

## API

- `GET /healthz`
- `POST /api/v1/world` — create and persist a seeded world.
- `GET /api/v1/world/{world_id}` — load a persisted world.
- `POST /api/v1/world/{world_id}/tick` — atomically advance one tick, persist the new snapshot and append events.

The starter world is a prototype scenario with abstract food, timber and treasury units. It is not a historical reconstruction. PostgreSQL stores one JSONB state snapshot per tick and a separate append-only event stream. The current schema does not yet provide replay from genesis, authentication, multi-user authorization or production hardening.

## Quality gates

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
DATABASE_URL=postgres://sherlock:sherlock@localhost:5432/sherlock cargo test --workspace --all-targets
cargo check -p sherlock-web --target wasm32-unknown-unknown
cargo build --release -p sherlock-api
```

GitHub Actions runs the same checks with a disposable PostgreSQL 16 service. See [docs/MAP.md](docs/MAP.md) for milestone acceptance criteria and remaining work.
