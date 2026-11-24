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
    pub inputs: BTreeMap<Good, u64>,
    pub output: Good,
    pub output_amount: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settlement {
    pub id: String,
    pub name: String,
    pub population: u64,
    pub treasury: i64,
    pub food_per_person_per_tick: u64,
    pub inventory: BTreeMap<Good, u64>,
    pub production: Vec<ProductionRecipe>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceSite {
    pub id: String,
    pub name: String,
    pub stocks: BTreeMap<Good, u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct World {
    pub id: Uuid,
    pub seed: u64,
    pub ruleset_version: u32,
    pub tick: u64,
    pub settlements: BTreeMap<String, Settlement>,
    pub resource_sites: BTreeMap<String, ResourceSite>,
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
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Provenance {
    GeneratedSimulation,
    HistoricalSource { source_id: String },
    ScholarlyInterpretation { source_id: String },
    ModelAssumption { assumption_id: String },
    DerivedEstimate { method: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventPayload {
    WorldCreated,
    TickAdvanced,
    GoodsProduced { settlement_id: String, good: Good, amount: u64, recipe_id: String },
    FoodConsumed { settlement_id: String, amount: u64 },
    ResourceExtracted { site_id: String, settlement_id: String, good: Good, amount: u64 },
    TradeExecuted { seller_id: String, buyer_id: String, good: Good, quantity: u64, total_price: i64 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractionCommand {
    pub site_id: String,
    pub settlement_id: String,
    pub good: Good,
    pub quantity: u64,
    pub actor: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradeCommand {
    pub seller_id: String,
    pub buyer_id: String,
    pub good: Good,
    pub quantity: u64,
    pub unit_price: i64,
    pub actor: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SimulationError {
    #[error("tick counter overflow")]
    TickOverflow,
    #[error("event sequence overflow")]
    EventSequenceOverflow,
    #[error("inventory arithmetic overflow for settlement {0}")]
    InventoryOverflow(String),
    #[error("treasury arithmetic overflow for settlement {0}")]
    TreasuryOverflow(String),
    #[error("food shortage in {settlement_id}: required {required}, available {available}")]
    FoodShortage { settlement_id: String, required: u64, available: u64 },
    #[error("missing production input {good:?} in settlement {settlement_id}")]
    InputShortage { settlement_id: String, good: Good },
    #[error("unknown settlement {0}")]
    UnknownSettlement(String),
    #[error("unknown resource site {0}")]
    UnknownResourceSite(String),
    #[error("resource shortage at {site_id}: requested {requested}, available {available}")]
    ResourceShortage { site_id: String, requested: u64, available: u64 },
    #[error("goods shortage in {settlement_id}: requested {requested}, available {available}")]
    GoodsShortage { settlement_id: String, requested: u64, available: u64 },
    #[error("insufficient funds in {settlement_id}: required {required}, available {available}")]
    InsufficientFunds { settlement_id: String, required: i64, available: i64 },
    #[error("trade quantity and unit price must be positive")]
    InvalidTrade,
    #[error("a settlement cannot trade with itself")]
    SelfTrade,
}

impl World {
    fn next_sequence(&self) -> Result<u64, SimulationError> {
        match self.events.last() {
            Some(event) => event.sequence.checked_add(1).ok_or(SimulationError::EventSequenceOverflow),
            None => Ok(0),
        }
    }

    fn record(
        &mut self,
        actor: impl Into<String>,
        cause: impl Into<String>,
        affected_entities: Vec<String>,
        payload: EventPayload,
    ) -> Result<(), SimulationError> {
        let sequence = self.next_sequence()?;
        self.events.push(WorldEvent {
            world_id: self.id,
            tick: self.tick,
            sequence,
            actor: actor.into(),
            cause: cause.into(),
            affected_entities,
            provenance: Provenance::GeneratedSimulation,
            ruleset_version: self.ruleset_version,
            payload,
        });
        Ok(())
    }

    pub fn advance_tick(&mut self) -> Result<(), SimulationError> {
        let mut next = self.clone();
        let new_tick = next.tick.checked_add(1).ok_or(SimulationError::TickOverflow)?;
        let settlement_ids: Vec<String> = next.settlements.keys().cloned().collect();
        let mut pending = Vec::new();

        for id in settlement_ids {
            let settlement = next.settlements.get_mut(&id).expect("id collected from map");
            for recipe in &settlement.production {
                for (good, amount) in &recipe.inputs {
                    let available = settlement.inventory.get(good).copied().unwrap_or(0);
                    if available < *amount {
                        return Err(SimulationError::InputShortage { settlement_id: id.clone(), good: *good });
                    }
                }
                for (good, amount) in &recipe.inputs {
                    *settlement.inventory.entry(*good).or_default() -= *amount;
                }
                let output = settlement.inventory.entry(recipe.output).or_default();
                *output = output.checked_add(recipe.output_amount)
                    .ok_or_else(|| SimulationError::InventoryOverflow(id.clone()))?;
                pending.push((
                    format!("production:{}", recipe.id),
                    vec![id.clone()],
                    EventPayload::GoodsProduced {
                        settlement_id: id.clone(), good: recipe.output,
                        amount: recipe.output_amount, recipe_id: recipe.id.clone(),
                    },
                ));
            }
            let required = settlement.population.checked_mul(settlement.food_per_person_per_tick)
                .ok_or_else(|| SimulationError::InventoryOverflow(id.clone()))?;
            let available = settlement.inventory.get(&Good::Food).copied().unwrap_or(0);
            if available < required {
                return Err(SimulationError::FoodShortage { settlement_id: id.clone(), required, available });
            }
            *settlement.inventory.entry(Good::Food).or_default() -= required;
            if required > 0 {
                pending.push((
                    "daily_consumption".into(),
                    vec![id.clone()],
                    EventPayload::FoodConsumed { settlement_id: id, amount: required },
                ));
            }
        }
        next.tick = new_tick;
        for (cause, affected, payload) in pending {
            next.record("simulation", cause, affected, payload)?;
        }
        next.record("simulation", "scheduled_tick", vec![], EventPayload::TickAdvanced)?;
        *self = next;
        Ok(())
    }

    pub fn extract_resource(&mut self, command: ExtractionCommand) -> Result<(), SimulationError> {
        if command.quantity == 0 {
            return Err(SimulationError::ResourceShortage {
                site_id: command.site_id, requested: 0, available: 0,
            });
        }
        let mut next = self.clone();
        let site = next.resource_sites.get(&command.site_id)
            .ok_or_else(|| SimulationError::UnknownResourceSite(command.site_id.clone()))?;
        let available = site.stocks.get(&command.good).copied().unwrap_or(0);
        if available < command.quantity {
            return Err(SimulationError::ResourceShortage {
                site_id: command.site_id, requested: command.quantity, available,
            });
        }
        let settlement = next.settlements.get(&command.settlement_id)
            .ok_or_else(|| SimulationError::UnknownSettlement(command.settlement_id.clone()))?;
        let new_inventory = settlement.inventory.get(&command.good).copied().unwrap_or(0)
            .checked_add(command.quantity)
            .ok_or_else(|| SimulationError::InventoryOverflow(command.settlement_id.clone()))?;
        *next.resource_sites.get_mut(&command.site_id).expect("site checked")
            .stocks.entry(command.good).or_default() -= command.quantity;
        next.settlements.get_mut(&command.settlement_id).expect("settlement checked")
            .inventory.insert(command.good, new_inventory);
        next.record(
            command.actor, "resource_extraction",
            vec![command.site_id.clone(), command.settlement_id.clone()],
            EventPayload::ResourceExtracted {
                site_id: command.site_id, settlement_id: command.settlement_id,
                good: command.good, amount: command.quantity,
            },
        )?;
        *self = next;
        Ok(())
    }

    pub fn execute_trade(&mut self, command: TradeCommand) -> Result<(), SimulationError> {
        if command.quantity == 0 || command.unit_price <= 0 {
            return Err(SimulationError::InvalidTrade);
        }
        if command.seller_id == command.buyer_id {
            return Err(SimulationError::SelfTrade);
        }
        let mut next = self.clone();
        let seller = next.settlements.get(&command.seller_id)
            .ok_or_else(|| SimulationError::UnknownSettlement(command.seller_id.clone()))?;
        let buyer = next.settlements.get(&command.buyer_id)
            .ok_or_else(|| SimulationError::UnknownSettlement(command.buyer_id.clone()))?;
        let available = seller.inventory.get(&command.good).copied().unwrap_or(0);
        if available < command.quantity {
            return Err(SimulationError::GoodsShortage {
                settlement_id: command.seller_id, requested: command.quantity, available,
            });
        }
        let total = i64::try_from(command.quantity).ok()
            .and_then(|quantity| quantity.checked_mul(command.unit_price))
            .ok_or_else(|| SimulationError::TreasuryOverflow(command.buyer_id.clone()))?;
        if buyer.treasury < total {
            return Err(SimulationError::InsufficientFunds {
                settlement_id: command.buyer_id, required: total, available: buyer.treasury,
            });
        }
        let seller_cash = seller.treasury.checked_add(total)
            .ok_or_else(|| SimulationError::TreasuryOverflow(command.seller_id.clone()))?;
        let buyer_goods = buyer.inventory.get(&command.good).copied().unwrap_or(0)
            .checked_add(command.quantity)
            .ok_or_else(|| SimulationError::InventoryOverflow(command.buyer_id.clone()))?;

        let seller = next.settlements.get_mut(&command.seller_id).expect("seller checked");
        *seller.inventory.entry(command.good).or_default() -= command.quantity;
        seller.treasury = seller_cash;
        let buyer = next.settlements.get_mut(&command.buyer_id).expect("buyer checked");
        buyer.inventory.insert(command.good, buyer_goods);
        buyer.treasury -= total;
        next.record(
            command.actor, "bilateral_trade",
            vec![command.seller_id.clone(), command.buyer_id.clone()],
            EventPayload::TradeExecuted {
                seller_id: command.seller_id, buyer_id: command.buyer_id,
                good: command.good, quantity: command.quantity, total_price: total,
            },
        )?;
        *self = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> World {
        let river = Settlement {
            id: "river-town".into(), name: "River Town".into(), population: 100,
            treasury: 1_000, food_per_person_per_tick: 1,
            inventory: [(Good::Food, 500), (Good::Timber, 40)].into_iter().collect(),
            production: vec![ProductionRecipe {
                id: "fishery".into(), inputs: BTreeMap::new(),
                output: Good::Food, output_amount: 25,
            }],
        };
        let hill = Settlement {
            id: "hill-village".into(), name: "Hill Village".into(), population: 40,
            treasury: 100, food_per_person_per_tick: 1,
            inventory: [(Good::Food, 250), (Good::Stone, 10)].into_iter().collect(),
            production: vec![],
        };
        let quarry = ResourceSite {
            id: "stone-quarry".into(), name: "Stone Quarry".into(),
            stocks: [(Good::Stone, 1_000)].into_iter().collect(),
        };
        World {
            id: Uuid::nil(), seed: 42, ruleset_version: 1, tick: 0,
            settlements: [(river.id.clone(), river), (hill.id.clone(), hill)].into_iter().collect(),
            resource_sites: [(quarry.id.clone(), quarry)].into_iter().collect(),
            events: vec![],
        }
    }

    #[test]
    fn tick_runs_production_consumption_and_audit() {
        let mut w = world();
        w.advance_tick().unwrap();
        assert_eq!(w.tick, 1);
        assert_eq!(w.settlements["river-town"].inventory[&Good::Food], 425);
        assert_eq!(w.settlements["hill-village"].inventory[&Good::Food], 210);
        assert_eq!(w.events.len(), 4);
        assert!(w.events.windows(2).all(|pair| pair[0].sequence < pair[1].sequence));
    }

    #[test]
    fn failed_tick_is_atomic() {
        let mut w = world();
        w.settlements.get_mut("river-town").unwrap().inventory.insert(Good::Food, 0);
        let before = w.clone();
        assert!(w.advance_tick().is_err());
        assert_eq!(w, before);
    }

    #[test]
    fn extraction_conserves_resource_and_records_cause() {
        let mut w = world();
        w.extract_resource(ExtractionCommand {
            site_id: "stone-quarry".into(), settlement_id: "hill-village".into(),
            good: Good::Stone, quantity: 25, actor: "player".into(),
        }).unwrap();
        assert_eq!(w.resource_sites["stone-quarry"].stocks[&Good::Stone], 975);
        assert_eq!(w.settlements["hill-village"].inventory[&Good::Stone], 35);
        assert!(matches!(w.events[0].payload, EventPayload::ResourceExtracted { amount: 25, .. }));
    }

    #[test]
    fn trade_conserves_goods_and_money() {
        let mut w = world();
        w.execute_trade(TradeCommand {
            seller_id: "hill-village".into(), buyer_id: "river-town".into(),
            good: Good::Stone, quantity: 3, unit_price: 10, actor: "player".into(),
        }).unwrap();
        assert_eq!(w.settlements["hill-village"].inventory[&Good::Stone], 7);
        assert_eq!(w.settlements["river-town"].inventory[&Good::Stone], 3);
        assert_eq!(w.settlements["hill-village"].treasury, 130);
        assert_eq!(w.settlements["river-town"].treasury, 970);
        assert_eq!(w.settlements.values().map(|s| s.treasury).sum::<i64>(), 1_100);
    }

    #[test]
    fn failed_trade_does_not_mutate_any_state() {
        let mut w = world();
        let before = w.clone();
        let result = w.execute_trade(TradeCommand {
            seller_id: "hill-village".into(), buyer_id: "river-town".into(),
            good: Good::Iron, quantity: 100, unit_price: 10, actor: "player".into(),
        });
        assert!(result.is_err());
        assert_eq!(w, before);
    }

    #[test]
    fn snapshot_serialization_round_trip_is_exact() {
        let original = world();
        let bytes = serde_json::to_vec(&original).unwrap();
        let restored: World = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(original, restored);
    }
}
