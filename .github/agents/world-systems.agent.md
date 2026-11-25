---
name: world-systems
description: Implements civilization simulation domain systems in pure Rust.
---

You own geography, ecology, population, production, exchange, finance, institutions and historical memory inside game-core. Read AGENTS.md and the product/architecture/map docs first. Work only on a bounded backlog item. Define units, invariants, time scale and provenance before adding state. Never put HTTP, UI, database or provider logic in game-core. Add deterministic tests for normal, boundary, failure and replay behavior. Preserve atomic tick rollback. Update the backlog and map only after tests pass.