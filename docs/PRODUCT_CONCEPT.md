# Sherlock — Historical World Simulation

## Product definition

Sherlock is a persistent systemic simulation of human civilization across geography, ecology, population, technology, production, trade, finance, institutions, political power, culture, collective beliefs and historical memory.

The world is the primary product. Football and real-world football data are optional specialized domains. They must use shared population, labor, capital, infrastructure, finance, organizations and cultural systems; they do not define the core architecture.

## Non-negotiable principles

1. One world and connected systems; no isolated minigame economies.
2. Emergent outcomes from rules, constraints, decisions, information and shocks rather than scripted historical outcomes.
3. Every material change has time, cause, actor, affected entities and an auditable event.
4. Generational continuity for people, households, firms, banks, states and institutions.
5. Agents act on what they know or believe, not omniscient world state.
6. Historical fact, scholarly interpretation, model assumption, derived estimate and generated simulation output remain distinct.
7. No generated entity or event may be presented as verified history or current real-world data.
8. Players and developers can inspect causal chains.
9. Deterministic replay where practical, with recorded seeds, inputs and ruleset versions.
10. Specialized domains integrate through shared contracts.

## World layers

- Planet and ecology: geography, climate, water, soil, minerals, energy, biodiversity and hazards.
- People and settlements: individuals, households, skills, health, education, migration, birth/death, cities and social groups.
- Knowledge and technology: discovery, research, diffusion, education and infrastructure.
- Production and exchange: land, labor, capital, firms, supply chains, goods, services, logistics, routes and prices.
- Money and finance: units of account, credit, debt, collateral, interest, banks, payments, investment, liquidity, defaults and crises.
- Organizations and institutions: firms, guilds, banks, universities, religious/cultural bodies, courts, states and regulators.
- Power and politics: authority, legitimacy, coalitions, taxation, public goods, conflict, diplomacy and institutional change.
- Dynasties and intergenerational capital: family networks, inheritance, alliances, ownership, reputation, education and succession.
- Culture and shared belief: language, identity, religion, myths, status, trust, norms, media and expectations.
- Historical memory: archives, records, education, institutional learning and reinterpretation.
- Specialized sectors: football and other industries using shared world primitives.

## Causal hypothesis

Resources and environment → settlements and population → skills and knowledge → production and exchange → surplus and distribution → credit and investment → organizations and institutions → control and political power → expectations, narratives and trust → investment or extraction → growth, fragility, crisis or adaptation → institutional and generational memory.

This is a design hypothesis, not a universal deterministic law. Feedback in both directions and divergent historical outcomes must remain possible.

## Player role

The player observes, investigates and intervenes. The campaign role (individual, household, organization, dynasty, state or analyst) is scenario configuration, not a hard-coded assumption in the world engine. Actions consume resources, require access or information, create obligations and may have delayed consequences.

## Time

Support pause, step and fast-forward with configurable time scales. Demography, climate and institutional change must not be forced into the same tick frequency as markets, logistics or political decisions. Scheduling and event ordering must be explicit and versioned.

## Evidence model

Historical imports require source identity, provenance, valid-time intervals, observed/estimated status, uncertainty and transformation records where available. Conflicting sources remain representable. Never silently collapse disagreement into one asserted fact.

## Architecture

- Rust-only application and domain code.
- Pure, testable simulation kernel separated from adapters.
- PostgreSQL snapshots and append-only event history.
- Typed commands, events, shared IDs, units and time contracts.
- Versioned schemas and rulesets.
- Deterministic seeded execution and replay.
- No production secrets or destructive changes to unrelated production data.

## Delivery sequence

0. Contracts and vertical slice: world IDs, time, units, ownership, event envelope, provenance, deterministic RNG, snapshots, two settlements, resources, population, production, trade, treasury, multi-tick persistence and replay.
1. Material world: geography, resource stocks/flows, environment, population cohorts, households, labor, inventories, logistics and exchange.
2. Finance and organizations: money, prices, credit, collateral, debt service, banks, firms, ownership, taxation, insolvency and investment.
3. Power and institutions: states, laws, enforcement, public goods, coalitions, legitimacy, conflict and diplomacy.
4. Generations, culture and memory: families, succession, inheritance, networks, education, reputation, collective beliefs and archives.
5. Technology and specialized sectors, including football as one optional domain.
6. Scenarios, investigation, intervention UI, causal-chain visualization, source browser, save/restore and accessibility.

## Acceptance gates

A phase is accepted only when invariants and property tests pass; seeded replay reproduces state; persistence round-trips preserve state and events; provenance is inspectable; generated output is never mislabeled as history; real integration tests cover storage/API; UI exercises actual domain behavior; and performance is measured against declared world-size targets.
