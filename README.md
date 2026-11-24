# Sherlock — Historical World Simulation

Sherlock is a persistent systemic simulation of human civilization. The world—not football—is the product: ecology, geography, population, knowledge, production, trade, finance, institutions, power, culture and historical memory interact in one causal system. Football is an optional specialized sector.

## Stack
- Rust workspace: deterministic domain kernel.
- Axum API and PostgreSQL persistence.
- React + TypeScript + Vite client.
- GitHub Actions for Rust, web and database checks.

## Local development
Requirements: Rust stable, Node 22+, npm, Docker.

```bash
docker compose up -d postgres
cargo test --workspace
cargo run -p sherlock-api
cd apps/web && npm install && npm run dev
```

API: `http://127.0.0.1:8080`. Configure `DATABASE_URL` and `VITE_API_URL` via environment variables.

See [product concept](docs/PRODUCT_CONCEPT.md), [delivery map](docs/MAP.md) and [agent contract](AGENTS.md).

A successful build is not product acceptance. Acceptance requires deterministic replay, invariants, PostgreSQL round-trip, API integration and real client journeys.
