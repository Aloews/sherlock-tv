use crate::world::{
    EntityId, EntityKind, EventEnvelope, EventPayload, Provenance, Tick, WorldState,
};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct EventDraft {
    pub actor: String,
    pub cause: String,
    pub affected_entities: Vec<EntityId>,
    pub provenance: Provenance,
    pub payload: EventPayload,
}

pub trait SimulationSystem: Send + Sync {
    fn id(&self) -> &'static str;
    fn advance(
        &self,
        world: &mut WorldState,
        tick: Tick,
        random: &mut dyn RngCore,
    ) -> Result<Vec<EventDraft>, SystemError>;
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("{message}")]
pub struct SystemError {
    pub message: String,
}

impl SystemError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SimulationError {
    #[error("world tick overflow")]
    TickOverflow,
    #[error("event sequence overflow")]
    EventSequenceOverflow,
    #[error("system {system_id} failed: {source}")]
    System {
        system_id: String,
        source: SystemError,
    },
}

pub struct SimulationEngine {
    systems: Vec<Box<dyn SimulationSystem>>,
}

impl SimulationEngine {
    pub fn new(mut systems: Vec<Box<dyn SimulationSystem>>) -> Self {
        systems.sort_by(|left, right| left.id().cmp(right.id()));
        Self { systems }
    }

    pub fn civilization() -> Self {
        Self::new(vec![Box::new(CivilizationSystem)])
    }

    pub fn advance_tick(
        &self,
        world: &mut WorldState,
    ) -> Result<Vec<EventEnvelope>, SimulationError> {
        let mut candidate = world.clone();
        let next_tick = candidate
            .tick
            .checked_next()
            .ok_or(SimulationError::TickOverflow)?;
        let mut drafts = Vec::new();
        for system in &self.systems {
            let mut random = ChaCha8Rng::seed_from_u64(seed_for_system(
                candidate.seed,
                next_tick,
                candidate.ruleset_version,
                system.id(),
            ));
            let system_drafts = system
                .advance(&mut candidate, next_tick, &mut random)
                .map_err(|source| SimulationError::System {
                    system_id: system.id().to_owned(),
                    source,
                })?;
            drafts.extend(system_drafts);
        }
        drafts.push(EventDraft {
            actor: "simulation".into(),
            cause: "scheduled_tick".into(),
            affected_entities: Vec::new(),
            provenance: Provenance::GeneratedSimulation,
            payload: EventPayload::TickAdvanced,
        });
        candidate.tick = next_tick;
        let mut events = Vec::with_capacity(drafts.len());
        for draft in drafts {
            let sequence = candidate.next_event_sequence;
            candidate.next_event_sequence = sequence
                .checked_add(1)
                .ok_or(SimulationError::EventSequenceOverflow)?;
            events.push(EventEnvelope {
                event_id: sequence,
                world_id: candidate.id,
                tick: next_tick,
                sequence,
                actor: draft.actor,
                cause: draft.cause,
                affected_entities: draft.affected_entities,
                ruleset_version: candidate.ruleset_version,
                provenance: draft.provenance,
                payload: draft.payload,
            });
        }
        candidate.event_log.extend(events.iter().cloned());
        *world = candidate;
        Ok(events)
    }
}

struct CivilizationSystem;

impl SimulationSystem for CivilizationSystem {
    fn id(&self) -> &'static str {
        "civilization-economy-v1"
    }

    fn advance(
        &self,
        world: &mut WorldState,
        tick: Tick,
        _random: &mut dyn RngCore,
    ) -> Result<Vec<EventDraft>, SystemError> {
        let settlement_ids: Vec<EntityId> = world
            .entities
            .values()
            .filter(|entity| entity.kind == EntityKind::Settlement)
            .map(|entity| entity.id)
            .collect();
        let mut drafts = Vec::new();

        for id in settlement_ids {
            let entity = world
                .entities
                .get_mut(&id)
                .ok_or_else(|| SystemError::new("settlement disappeared during tick"))?;
            let read = |key: &str| -> Result<u64, SystemError> {
                entity
                    .attributes
                    .get(key)
                    .and_then(serde_json::Value::as_u64)
                    .ok_or_else(|| {
                        SystemError::new(format!("invalid or missing settlement attribute: {key}"))
                    })
            };
            let population = read("population")?;
            let food = read("food")?;
            let production = read("food_production")?;
            let food_per_person = read("food_per_person")?;
            let timber = read("timber")?;
            let timber_regeneration = read("timber_regeneration")?;
            let treasury = read("treasury")?;
            let tax_rate = read("tax_rate_basis_points")?;
            let consumption = population
                .checked_mul(food_per_person)
                .ok_or_else(|| SystemError::new("food consumption overflow"))?;
            let available = food
                .checked_add(production)
                .ok_or_else(|| SystemError::new("food stock overflow"))?;
            let new_food = available.saturating_sub(consumption);
            let food_delta = i64::try_from(new_food).unwrap_or(i64::MAX)
                - i64::try_from(food).unwrap_or(i64::MAX);
            entity.attributes.insert("food".into(), new_food.into());
            let new_timber = timber
                .checked_add(timber_regeneration)
                .ok_or_else(|| SystemError::new("timber stock overflow"))?;
            entity.attributes.insert("timber".into(), new_timber.into());
            if food_delta != 0 {
                drafts.push(EventDraft {
                    actor: "economy".into(),
                    cause: "seasonal_food_balance".into(),
                    affected_entities: vec![id],
                    provenance: Provenance::GeneratedSimulation,
                    payload: EventPayload::ResourceStockChanged {
                        site_id: id,
                        resource: "food".into(),
                        delta: food_delta,
                    },
                });
            }
            drafts.push(EventDraft {
                actor: "economy".into(),
                cause: "renewable_resource_regrowth".into(),
                affected_entities: vec![id],
                provenance: Provenance::GeneratedSimulation,
                payload: EventPayload::ResourceStockChanged {
                    site_id: id,
                    resource: "timber".into(),
                    delta: i64::try_from(timber_regeneration).unwrap_or(i64::MAX),
                },
            });

            let mut next_population = population;
            let mut population_delta = 0_i64;
            if new_food < 100 && population > 10 {
                next_population -= 1;
                population_delta = -1;
            } else if new_food > 700 && population < 100_000 && tick.0 % 5 == 0 {
                next_population += 1;
                population_delta = 1;
            }
            if population_delta != 0 {
                entity
                    .attributes
                    .insert("population".into(), next_population.into());
                drafts.push(EventDraft {
                    actor: "demography".into(),
                    cause: if population_delta > 0 {
                        "food_surplus"
                    } else {
                        "food_shortage"
                    }
                    .into(),
                    affected_entities: vec![id],
                    provenance: Provenance::GeneratedSimulation,
                    payload: EventPayload::PopulationChanged {
                        settlement_id: id,
                        delta: population_delta,
                        reason: if population_delta > 0 {
                            "sustained_food_surplus".into()
                        } else {
                            "food_insecurity".into()
                        },
                    },
                });
            }

            let tax_base = next_population
                .checked_mul(2)
                .ok_or_else(|| SystemError::new("tax base overflow"))?;
            let tax_income = tax_base
                .checked_mul(tax_rate)
                .ok_or_else(|| SystemError::new("tax income overflow"))?
                / 10_000;
            let new_treasury = treasury
                .checked_add(tax_income)
                .ok_or_else(|| SystemError::new("treasury overflow"))?;
            entity
                .attributes
                .insert("treasury".into(), new_treasury.into());
            drafts.push(EventDraft {
                actor: "public_finance".into(),
                cause: "population_tax_assessment".into(),
                affected_entities: vec![id],
                provenance: Provenance::GeneratedSimulation,
                payload: EventPayload::Custom {
                    domain: "finance".into(),
                    event_type: "tax_revenue".into(),
                    payload: serde_json::json!({"amount": tax_income, "treasury": new_treasury}),
                },
            });
        }
        Ok(drafts)
    }
}

fn seed_for_system(seed: u64, tick: Tick, ruleset_version: u32, system_id: &str) -> u64 {
    let stable_system_hash = system_id
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
        });
    let mut value = seed
        ^ tick.0.rotate_left(17)
        ^ u64::from(ruleset_version).rotate_left(41)
        ^ stable_system_hash;
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{EntityKind, EventPayload, Provenance, WorldState};

    struct RandomEventSystem;
    impl SimulationSystem for RandomEventSystem {
        fn id(&self) -> &'static str {
            "random-event"
        }
        fn advance(
            &self,
            _world: &mut WorldState,
            _tick: Tick,
            random: &mut dyn RngCore,
        ) -> Result<Vec<EventDraft>, SystemError> {
            let amount = u64::from(random.next_u32());
            Ok(vec![EventDraft {
                actor: "test-system".into(),
                cause: "test-draw".into(),
                affected_entities: vec![EntityId(1)],
                provenance: Provenance::GeneratedSimulation,
                payload: EventPayload::Custom {
                    domain: "test".into(),
                    event_type: "random_draw".into(),
                    payload: serde_json::json!({ "amount": amount }),
                },
            }])
        }
    }

    struct FailingSystem;
    impl SimulationSystem for FailingSystem {
        fn id(&self) -> &'static str {
            "failure"
        }
        fn advance(
            &self,
            world: &mut WorldState,
            _tick: Tick,
            _random: &mut dyn RngCore,
        ) -> Result<Vec<EventDraft>, SystemError> {
            world.register_entity(EntityKind::Person, "temporary".into());
            Err(SystemError::new("deliberate test failure"))
        }
    }

    #[test]
    fn tick_advances_and_emits_ordered_tick_event() {
        let engine = SimulationEngine::new(vec![]);
        let mut world = WorldState::new(7, 1);
        let events = engine
            .advance_tick(&mut world)
            .expect("tick should succeed");
        assert_eq!(world.tick, Tick(1));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].sequence, 0);
        assert_eq!(events[0].payload, EventPayload::TickAdvanced);
    }

    #[test]
    fn starter_world_contains_two_populated_settlements() {
        let world = WorldState::starter(42);
        assert_eq!(world.entities.len(), 2);
        assert_eq!(
            world
                .entities
                .values()
                .filter(|e| e.kind == EntityKind::Settlement)
                .count(),
            2
        );
        assert_eq!(world.entities[&EntityId(1)].attributes["population"], 120);
    }

    #[test]
    fn economy_changes_stocks_and_is_deterministic() {
        let engine = SimulationEngine::civilization();
        let mut left = WorldState::starter(42);
        let mut right = WorldState::starter(42);
        let events = engine.advance_tick(&mut left).expect("first tick");
        engine.advance_tick(&mut right).expect("same tick");
        assert_eq!(left, right);
        assert!(
            events
                .iter()
                .any(|event| matches!(event.payload, EventPayload::ResourceStockChanged { .. }))
        );
        assert_eq!(left.entities[&EntityId(1)].attributes["food"], 910);
        assert_eq!(left.entities[&EntityId(1)].attributes["timber"], 518);
    }

    #[test]
    fn food_surplus_increases_population_on_fifth_tick() {
        let engine = SimulationEngine::civilization();
        let mut world = WorldState::starter(42);
        for _ in 0..5 {
            engine.advance_tick(&mut world).expect("tick");
        }
        assert_eq!(world.entities[&EntityId(1)].attributes["population"], 121);
    }

    #[test]
    fn failed_system_does_not_partially_mutate_world() {
        let engine = SimulationEngine::new(vec![Box::new(FailingSystem)]);
        let mut world = WorldState::new(7, 1);
        let before = world.clone();
        assert!(engine.advance_tick(&mut world).is_err());
        assert_eq!(world, before);
    }

    #[test]
    fn same_seed_and_tick_produce_same_random_payload_and_events() {
        let engine = SimulationEngine::new(vec![Box::new(RandomEventSystem)]);
        let mut left = WorldState::new(1234, 1);
        let mut right = WorldState::new(1234, 1);
        let left_events = engine.advance_tick(&mut left).expect("left tick");
        let right_events = engine.advance_tick(&mut right).expect("right tick");
        assert_eq!(left_events, right_events);
        assert_eq!(left, right);
    }

    #[test]
    fn systems_execute_in_stable_identifier_order() {
        struct NamedSystem(&'static str);
        impl SimulationSystem for NamedSystem {
            fn id(&self) -> &'static str {
                self.0
            }
            fn advance(
                &self,
                _world: &mut WorldState,
                _tick: Tick,
                _random: &mut dyn RngCore,
            ) -> Result<Vec<EventDraft>, SystemError> {
                Ok(vec![EventDraft {
                    actor: self.0.into(),
                    cause: "order-test".into(),
                    affected_entities: vec![],
                    provenance: Provenance::GeneratedSimulation,
                    payload: EventPayload::Custom {
                        domain: "test".into(),
                        event_type: "order".into(),
                        payload: serde_json::Value::Null,
                    },
                }])
            }
        }
        let engine = SimulationEngine::new(vec![
            Box::new(NamedSystem("zeta")),
            Box::new(NamedSystem("alpha")),
        ]);
        let mut world = WorldState::new(1, 1);
        let events = engine.advance_tick(&mut world).expect("tick");
        assert_eq!(events[0].actor, "alpha");
        assert_eq!(events[1].actor, "zeta");
    }

    #[test]
    fn world_snapshot_round_trip_preserves_domain_state_and_excludes_event_history() {
        let engine = SimulationEngine::civilization();
        let mut world = WorldState::starter(314);
        engine.advance_tick(&mut world).expect("tick");
        let encoded = serde_json::to_string(&world).expect("serialize world");
        let mut decoded: WorldState = serde_json::from_str(&encoded).expect("deserialize world");
        assert!(!world.event_log.is_empty());
        assert!(decoded.event_log.is_empty());
        assert_eq!(decoded.next_event_sequence, world.next_event_sequence);
        let mut expected_snapshot = world.clone();
        expected_snapshot.event_log.clear();
        assert_eq!(decoded, expected_snapshot);
        assert!(decoded.geography.validate().is_ok());
        decoded.event_log = world.event_log.clone();
        assert_eq!(decoded, world);
    }

    #[test]
    fn tick_overflow_is_rejected_without_mutation() {
        let engine = SimulationEngine::new(vec![]);
        let mut world = WorldState::new(1, 1);
        world.tick = Tick(u64::MAX);
        let before = world.clone();
        assert_eq!(
            engine.advance_tick(&mut world),
            Err(SimulationError::TickOverflow)
        );
        assert_eq!(world, before);
    }
}
