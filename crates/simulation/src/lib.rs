use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use sherlock_domain::{EventEnvelope, EventPayload, Provenance, Tick, WorldState};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct EventDraft {
    pub actor: String,
    pub cause: String,
    pub affected_entities: Vec<sherlock_domain::EntityId>,
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
        Self { message: message.into() }
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

    pub fn advance_tick(
        &self,
        world: &mut WorldState,
    ) -> Result<Vec<EventEnvelope>, SimulationError> {
        let mut candidate = world.clone();
        let next_tick = candidate.tick.checked_next().ok_or(SimulationError::TickOverflow)?;
        let world_seed = candidate.seed;
        let ruleset_version = candidate.ruleset_version;
        let mut drafts = Vec::new();

        for system in &self.systems {
            let mut random = ChaCha8Rng::seed_from_u64(seed_for_system(
                world_seed,
                next_tick,
                ruleset_version,
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
                event_id: uuid::Uuid::new_v5(
                    &candidate.id.0,
                    format!("{}:{sequence}", next_tick.0).as_bytes(),
                ),
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

        *world = candidate;
        Ok(events)
    }
}

fn seed_for_system(seed: u64, tick: Tick, ruleset_version: u32, system_id: &str) -> u64 {
    let stable_system_hash = system_id.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
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
    use sherlock_domain::{EntityId, EventPayload, Provenance, WorldState};

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
                affected_entities: vec![EntityId(uuid::Uuid::nil())],
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
            world.tick = Tick(999);
            Err(SystemError::new("deliberate test failure"))
        }
    }

    #[test]
    fn tick_advances_and_emits_ordered_tick_event() {
        let engine = SimulationEngine::new(vec![]);
        let mut world = WorldState::new(7, 1);

        let events = engine.advance_tick(&mut world).expect("tick should succeed");

        assert_eq!(world.tick, Tick(1));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].sequence, 0);
        assert_eq!(events[0].payload, EventPayload::TickAdvanced);
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
    fn same_seed_and_tick_produce_same_random_payload() {
        let engine = SimulationEngine::new(vec![Box::new(RandomEventSystem)]);
        let mut left = WorldState::new(1234, 1);
        let mut right = WorldState::new(1234, 1);

        let left_events = engine.advance_tick(&mut left).expect("left tick");
        let right_events = engine.advance_tick(&mut right).expect("right tick");

        assert_eq!(left_events[0].payload, right_events[0].payload);
    }

    #[test]
    fn replay_from_the_same_snapshot_produces_identical_events_and_state() {
        let engine = SimulationEngine::new(vec![Box::new(RandomEventSystem)]);
        let mut original = WorldState::new(9876, 1);
        let mut replay = original.clone();

        let original_events = engine.advance_tick(&mut original).expect("original tick");
        let replay_events = engine.advance_tick(&mut replay).expect("replay tick");

        assert_eq!(original_events, replay_events);
        assert_eq!(original, replay);
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
}
