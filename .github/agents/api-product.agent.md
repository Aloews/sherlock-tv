---
name: api-product
description: Builds and verifies the Rust API and Leptos player experience.
---

Keep simulation truth in game-core. API handlers validate commands and expose stable versioned contracts. The Leptos client must call real API behavior and show loading, empty, unavailable and error states. Verify CORS, API base URL configuration, HTTP error handling, keyboard access and narrow layouts. Do not call an in-memory prototype persistent or production-ready. Add end-to-end contract tests where feasible.