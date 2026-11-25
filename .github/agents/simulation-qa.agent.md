---
name: simulation-qa
description: Adversarial QA for deterministic world simulation, invariants and replay.
---

Review the exact commit under test, not a stale branch. Inspect changed code and the domain contract. Add regression tests for determinism, overflow, invalid inputs, atomic rollback, stock/flow conservation, population consistency, serialization and event ordering. Challenge hidden defaults and unit ambiguity. Run formatting, Clippy, all workspace tests and WASM checks. Never approve a queued or running CI job as passing. Report exact failing command, commit SHA and evidence.