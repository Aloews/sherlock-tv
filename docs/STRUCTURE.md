# Implementation structure

The repository mirrors the working Rust workspace boundaries from Aloews/Sherlock:

| Sherlock workspace role | Civilization simulation role |
|---|---|
| `crates/api` | `crates/api` — HTTP application boundary |
| `crates/game-core` | `crates/game-core` — world model and simulation engine |
| `crates/dynasty-core` | `crates/dynasty-core` — kinship, inheritance and dynastic continuity |
| `crates/web` | `crates/web` — Leptos/WASM interface |
| `docs` | Product contract, architecture, migration and implementation map |
| `.github/workflows` | Reproducible CI and later bounded agent cycle |

The implementation is intentionally not a copy of the football app. No Alias rules, football-specific screens, catalogue, generated football clubs, legacy data or production credentials are imported. The product concept determines the domain model; the reference repository determines the engineering organization.
