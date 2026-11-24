use sherlock_game_core::{EventPayload, SimulationEngine, Tick, WorldState};

#[test]
fn empty_world_can_advance_multiple_ticks() {
    let engine = SimulationEngine::new(vec![]);
    let mut world = WorldState::new(42, 1);

    for expected_tick in 1..=5 {
        let events = engine.advance_tick(&mut world).expect("tick");
        assert_eq!(world.tick, Tick(expected_tick));
        assert!(matches!(
            events.last().map(|event| &event.payload),
            Some(EventPayload::TickAdvanced)
        ));
    }
}
