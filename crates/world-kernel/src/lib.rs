use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Good { Food, Timber, Stone, Iron }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionRecipe {
    pub id: String,
    pub output: Good,
    pub output_amount: u64,
    pub inputs: BTreeMap<Good, u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settlement {
    pub id: String,
    pub name: String,
    pub population: u64,
    pub treasury: i64,
    pub inventory: BTreeMap<Good, u64>,
    pub food_per_person_per_tick: u64,
    pub production: Vec<ProductionRecipe>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct World {
    pub id: Uuid,
    pub seed: u64,
    pub ruleset_version: u32,
    pub tick: u64,
    pub settlements: BTreeMap<String, Settlement>,
    pub events: Vec<WorldEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldEvent {
    pub world_id: Uuid,
    pub tick: u64,
    pub sequence: u64,
    pub actor: String,
    pub cause: String,
    pub affected_entities: Vec<String>,
    pub provenance: Provenance,
    pub ruleset_version: u32,
    pub payload: EventPayload,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provenance { GeneratedSimulation, HistoricalSource { source_id: String }, DerivedEstimate { method: String }, ModelAssumption { assumption_id: String } }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventPayload {
    WorldCreated,
    WorldAdvanced,
    GoodsProduced { settlement_id: String, good: Good, amount: u64 },
    FoodConsumed { settlement_id: String, amount: u64 },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SimulationError {
    #[error("tick counter overflow")]
    TickOverflow,
    #[error("inventory arithmetic overflow in {0}")]
    InventoryOverflow(String),
    #[error("food shortage in {settlement_id}: required {required}, available {available}")]
    FoodShortage { settlement_id: String, required: u64, available: u64 },
    #[error("input shortage in {settlement_id} for {good:?}")]
    InputShortage { settlement_id: String, good: Good },
}

impl World {
    pub fn advance_tick(&mut self) -> Result<(), SimulationError> {
        let mut next = self.clone();
        let tick = next.tick.checked_add(1).ok_or(SimulationError::TickOverflow)?;
        let mut changes = Vec::new();

        for settlement in next.settlements.values_mut() {
            for recipe in &settlement.production {
                for (good, amount) in &recipe.inputs {
                    if settlement.inventory.get(good).copied().unwrap_or(0) < *amount {
                        return Err(SimulationError::InputShortage { settlement_id: settlement.id.clone(), good: *good });
                    }
                }
                for (good, amount) in &recipe.inputs {
                    *settlement.inventory.entry(*good).or_default() -= *amount;
                }
                let stock = settlement.inventory.entry(recipe.output).or_default();
                *stock = stock.checked_add(recipe.output_amount).ok_or_else(|| SimulationError::InventoryOverflow(settlement.id.clone()))?;
                changes.push((settlement.id.clone(), recipe.output, recipe.output_amount, recipe.id.clone(), false));
            }
            let required = settlement.population.checked_mul(settlement.food_per_person_per_tick)
                .ok_or_else(|| SimulationError::InventoryOverflow(settlement.id.clone()))?;
            let available = settlement.inventory.get(&Good::Food).copied().unwrap_or(0);
            if available < required {
                return Err(SimulationError::FoodShortage { settlement_id: settlement.id.clone(), required, available });
            }
            *settlement.inventory.entry(Good::Food).or_default() -= required;
            if required > 0 { changes.push((settlement.id.clone(), Good::Food, required, "population_consumption".into(), true)); }
        }

        next.tick = tick;
        let mut sequence = next.events.len() as u64;
        for (settlement_id, good, amount, cause, consumption) in changes {
            let payload = if consumption {
                EventPayload::FoodConsumed { settlement_id: settlement_id.clone(), amount }
            } else {
                EventPayload::GoodsProduced { settlement_id: settlement_id.clone(), good, amount }
            };
            next.events.push(WorldEvent {
                world_id: next.id, tick, sequence, actor: "world_simulation".into(), cause,
                affected_entities: vec![settlement_id], provenance: Provenance::GeneratedSimulation,
                ruleset_version: next.ruleset_version, payload,
            });
            sequence += 1;
        }
        next.events.push(WorldEvent {
            world_id: next.id, tick, sequence, actor: "world_simulation".into(), cause: "scheduled_tick".into(),
            affected_entities: vec![], provenance: Provenance::GeneratedSimulation,
            ruleset_version: next.ruleset_version, payload: EventPayload::WorldAdvanced,
        });
        *self = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn world() -> World {
        let town = Settlement {
            id: "river-town".into(), name: "River Town".into(), population: 100, treasury: 500,
            inventory: [(Good::Food, 200), (Good::Timber, 20)].into_iter().collect(),
            food_per_person_per_tick: 1,
            production: vec![ProductionRecipe { id: "fishery".into(), output: Good::Food, output_amount: 30, inputs: BTreeMap::new() }],
        };
        World { id: Uuid::nil(), seed: 42, ruleset_version: 1, tick: 0,
            settlements: [(town.id.clone(), town)].into_iter().collect(), events: vec![] }
    }
    #[test] fn tick_produces_consumes_and_records_events() {
        let mut w = world(); w.advance_tick().unwrap();
        assert_eq!(w.settlements["river-town"].inventory[&Good::Food], 130);
        assert_eq!(w.tick, 1);
        assert!(w.events.iter().any(|e| matches!(e.payload, EventPayload::FoodConsumed { amount: 100, .. })));
    }
    #[test] fn failed_tick_is_atomic() {
        let mut w = world(); w.settlements.get_mut("river-town").unwrap().inventory.insert(Good::Food, 0);
        let before = w.clone(); assert!(w.advance_tick().is_err()); assert_eq!(w, before);
    }
    #[test] fn replay_is_deterministic() {
        let mut a = world(); let mut b = world();
        for _ in 0..4 { a.advance_tick().unwrap(); b.advance_tick().unwrap(); }
        assert_eq!(a, b);
    }
}
