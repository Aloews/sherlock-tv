# Sherlock — Historical World Simulation

## Product
A persistent systemic simulation of human civilization across geography, ecology, population, technology, production, trade, finance, institutions, political power, culture, collective beliefs and historical memory. It is not primarily a football manager, stock-trading game, dynasty clicker or scripted historical campaign.

The world is primary. Football and real football data are optional sectors participating through organizations, labor, capital, audiences, infrastructure, media and culture. They must not define the core architecture.

## Principles
1. One connected world with shared entities, resources and institutions.
2. Emergent outcomes from rules, constraints, decisions, information and shocks—not scripted conclusions.
3. Causal continuity: each material change records time, actor, cause, affected entities and provenance.
4. Generational time: people, families, firms, banks, states and institutions have lifecycles, succession, obligations and memory.
5. Bounded knowledge: agents act on what they know or believe, not omniscient state.
6. Historical grounding with source, date, uncertainty and transformation metadata.
7. No fake reality: generated events are never represented as verified historical facts.
8. Inspectability: trace why prices, crises, migration, institutions and power changed.
9. Deterministic replay where practical, recording inputs, ruleset versions and seeds.
10. Modular specialization through shared contracts.

## World systems
- Planet/ecology: geography, climate, water, soil, minerals, energy, biodiversity and hazards.
- People/settlements: individuals, households, skills, health, education, migration, birth/death, cities and social groups.
- Knowledge/technology: discovery, research, diffusion, education, infrastructure and production methods.
- Production/exchange: land, labor, capital, firms, supply chains, goods, logistics, trade routes and prices.
- Money/finance: account units, credit, debt, collateral, interest, banks, payments, investment, liquidity, defaults, bubbles and crises.
- Organizations/institutions: firms, guilds, banks, universities, cultural bodies, courts, states, regulators and rules.
- Power/politics: authority, legitimacy, coalitions, taxation, public goods, conflict, diplomacy and institutional change.
- Dynasties/intergenerational capital: family networks, inheritance, ownership, reputation, education, influence and succession.
- Culture/belief: language, identity, religion, myths, narratives, status, trust, norms, media and expectations.
- Historical memory: archives, records, education, institutional learning, forgotten knowledge and reinterpretation.
- Specialized sectors: football and other industries using shared population, labor, capital, infrastructure, finance and culture.

## Causal hypothesis
Resources/environment ↔ settlements/population ↔ skills/knowledge ↔ production/exchange ↔ surplus/wealth distribution ↔ credit/investment ↔ organizations/institutions ↔ political power ↔ expectations/trust ↔ growth, fragility, crisis or adaptation ↔ institutional/generational memory. This is a design hypothesis, not a universal deterministic law; divergent outcomes and feedback in both directions must be possible.

## Player experience and time
The player observes, investigates and intervenes. Campaign role (individual, household, organization, dynasty, state or analyst) is scenario configuration, not a hard-coded engine assumption. Actions consume resources, require information/access, create obligations and may have delayed consequences.

Support pause, step and fast-forward with configurable time scales. Demography, climate, institutions and technology must not be forced into the same tick rate as markets, logistics or political decisions.

## Evidence model
Historical imports carry source/provenance, valid-time intervals, observed/estimated status, uncertainty and transformation history. Facts, derived estimates, assumptions and generated outcomes remain distinct. Conflicting sources remain representable.

## Architecture
Rust-first pure domain engine; bounded contexts connected by typed commands/events; PostgreSQL snapshots, event log, entity histories, ownership and provenance; API/client as adapters; versioned schemas, rulesets and migrations.

## Phases
0. Contracts and a small persisted/replayable material-world slice.
1. Geography, environment, population, households, labor, production, inventories, logistics and exchange.
2. Money, prices, credit, banks, firms, ownership, taxation, insolvency and investment.
3. States, laws, enforcement, public goods, coalitions, legitimacy, conflict and diplomacy.
4. Individuals, families, succession, inheritance, culture, beliefs, archives and institutional learning.
5. Technology, media, environmental feedback and specialized sectors including football.
6. Scenarios, investigation, interventions, causal-chain UI, evidence browser and accessibility.

## Acceptance
Require invariants/property tests, deterministic replay, PostgreSQL round-trip, inspectable causality/provenance, no fact/simulation confusion, real API/database integration, real client journeys and measured performance. Compilation alone is not acceptance.
