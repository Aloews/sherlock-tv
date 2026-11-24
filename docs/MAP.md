# Sherlock delivery map

## North star
A persistent inspectable civilization simulation in which ecology, population, knowledge, production, trade, finance, institutions, power, dynasties, culture and memory interact over generations. Football is optional.

## Current state
This branch replaces the previous MatchLive codebase with a clean project skeleton. The first milestone is foundational, not a complete game. No production readiness is implied.

## W0 — executable foundation
- [x] Product definition and architecture constraints.
- [x] Clean Rust workspace and domain kernel.
- [x] Basic deterministic tick with atomic failure behavior.
- [x] PostgreSQL schema and API snapshot/event transaction boundary.
- [x] Minimal API-backed web shell.
- [x] CI definition for Rust, web and PostgreSQL service.
- [ ] Verify exact-head CI and real PostgreSQL integration.
- [ ] Independent replay, conservation and failure-injection tests.

## W1 — material world
- [ ] Resource-bearing locations, extraction and regeneration.
- [ ] Multiple settlements, population cohorts and labor.
- [ ] Production recipes, inventories, logistics and trade.
- [ ] Treasury, double-entry ledger and price formation.
- [ ] Multi-tick scenarios with explainable causal chains.
- [ ] Snapshot reload and replay equivalence.

## W2 — economic institutions
- [ ] Firms, ownership, banks, capital, credit, collateral, debt service, taxes and defaults.
- [ ] Investment, wages, productivity, distribution and balance-sheet invariants.
- [ ] Bubbles, contagion and crises as emergent feedback.

## W3–W5
States/laws/courts/legitimacy/diplomacy; people/families/dynasties/inheritance/culture/memory; research/technology/ecology and optional specialized sectors.

## W6 — player experience
Scenarios, observation/investigation/intervention, causal explorer, evidence browser, save/restore, accessibility and responsive clients.

## Release gates
No acceptance on compile-only evidence. Require domain invariants, replay, PostgreSQL round-trip, API integration, client journey tests and inspected exact-SHA CI.
