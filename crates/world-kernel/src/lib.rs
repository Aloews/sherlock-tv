use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Good {
    Food,
    Timber,
    Stone,
    Iron,
}

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
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    GeneratedSimulation,
    HistoricalSource { source_id: String },
    DerivedEstimate { method: String },
    ModelAssumption { assumption_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventPayload {
    WorldCreated,
    WorldAdvanced,
    GoodsProduced {
        settlement_id: String,
        good: Good,
        amount: u64,
    },
    FoodConsumed {
        settlement_id: String,
        amount: u64,
    },
    ResourceExtracted {
        site_id: String,
        settlement_id: String,
        good: Good,
        amount: u64,
    },
    TradeExecuted {
        seller_id: String,
        buyer_id: String,
        good: Good,
        quantity: u64,
        total_price: i64,
    },
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
    #[error("inventory arithmetic overflow in {0}")]
    InventoryOverflow(String),
    #[error("treasury arithmetic overflow in {0}")]
    TreasuryOverflow(String),
    #[error("food shortage in {settlement_id}: required {required}, available {available}")]
    FoodShortage {
        settlement_id: String,
        required: u64,
        available: u64,
    },
    #[error("input shortage in {settlement_id} for {good:?}")]
    InputShortage { settlement_id: String, good: Good },
    #[error("unknown settlement: {0}")]
    UnknownSettlement(String),
    #[error("unknown resource site: {0}")]
    UnknownResourceSite(String),
    #[error(
        "insufficient resource stock at {site_id}: requested {requested}, available {available}"
    )]
    InsufficientResource {
        site_id: String,
        requested: u64,
        available: u64,
    },
    #[error("insufficient goods in {settlement_id}: requested {requested}, available {available}")]
    InsufficientGoods {
        settlement_id: String,
        requested: u64,
        available: u64,
    },
    #[error(
        "insufficient treasury in {settlement_id}: required {required}, available {available}"
    )]
    InsufficientFunds {
        settlement_id: String,
        required: i64,
        available: i64,
    },
    #[error("trade quantity and unit price must be positive")]
    InvalidTrade,
    #[error("seller and buyer must be different settlements")]
    SameSettlementTrade,
}

impl World {
    fn next_sequence(&self) -> u64 {
        self.events.len() as u64
    }

    pub fn advance_tick(&mut self) -> Result<(), SimulationError> {
        let mut next = self.clone();
        let tick = next
            .tick
            .checked_add(1)
            .ok_or(SimulationError::TickOverflow)?;
        let mut changes = Vec::new();

        for settlement in next.settlements.values_mut() {
            for recipe in &settlement.production {
                for (good, amount) in &recipe.inputs {
                    if settlement.inventory.get(good).copied().unwrap_or(0) < *amount {
                        return Err(SimulationError::InputShortage {
                            settlement_id: settlement.id.clone(),
                            good: *good,
                        });
                    }
                }
                for (good, amount) in &recipe.inputs {
                    *settlement.inventory.entry(*good).or_default() -= *amount;
                }
                let stock = settlement.inventory.entry(recipe.output).or_default();
                *stock = stock
                    .checked_add(recipe.output_amount)
                    .ok_or_else(|| SimulationError::InventoryOverflow(settlement.id.clone()))?;
                changes.push((
                    settlement.id.clone(),
                    recipe.output,
                    recipe.output_amount,
                    recipe.id.clone(),
                    false,
                ));
            }

            let required = settlement
                .population
                .checked_mul(settlement.food_per_person_per_tick)
                .ok_or_else(|| SimulationError::InventoryOverflow(settlement.id.clone()))?;
            let available = settlement.inventory.get(&Good::Food).copied().unwrap_or(0);
            if available < required {
                return Err(SimulationError::FoodShortage {
                    settlement_id: settlement.id.clone(),
                    required,
                    available,
                });
            }
            *settlement.inventory.entry(Good::Food).or_default() -= required;
            if required > 0 {
                changes.push((
                    settlement.id.clone(),
                    Good::Food,
                    required,
                    "population_consumption".into(),
                    true,
                ));
            }
        }

        next.tick = tick;
        let mut sequence = next.next_sequence();
        for (settlement_id, good, amount, cause, consumption) in changes {
            let payload = if consumption {
                EventPayload::FoodConsumed {
                    settlement_id: settlement_id.clone(),
                    amount,
                }
            } else {
                EventPayload::GoodsProduced {
                    settlement_id: settlement_id.clone(),
                    good,
                    amount,
                }
            };
            next.events.push(WorldEvent {
                world_id: next.id,
                tick,
                sequence,
                actor: "world_simulation".into(),
                cause,
                affected_entities: vec![settlement_id],
                provenance: Provenance::GeneratedSimulation,
                ruleset_version: next.ruleset_version,
                payload,
            });
            sequence += 1;
        }
        next.events.push(WorldEvent {
            world_id: next.id,
            tick,
            sequence,
            actor: "world_simulation".into(),
            cause: "scheduled_tick".into(),
            affected_entities: vec![],
            provenance: Provenance::GeneratedSimulation,
            ruleset_version: next.ruleset_version,
            payload: EventPayload::WorldAdvanced,
        });
        *self = next;
        Ok(())
    }

    pub fn extract_resource(&mut self, command: ExtractionCommand) -> Result<(), SimulationError> {
        if command.quantity == 0 {
            return Err(SimulationError::InsufficientResource {
                site_id: command.site_id,
                requested: 0,
                available: 0,
            });
        }
        let mut next = self.clone();
        let site = next
            .resource_sites
            .get(&command.site_id)
            .ok_or_else(|| SimulationError::UnknownResourceSite(command.site_id.clone()))?;
        let available = site.stocks.get(&command.good).copied().unwrap_or(0);
        if available < command.quantity {
            return Err(SimulationError::InsufficientResource {
                site_id: command.site_id,
                requested: command.quantity,
                available,
            });
        }
        let settlement = next
            .settlements
            .get(&command.settlement_id)
            .ok_or_else(|| SimulationError::UnknownSettlement(command.settlement_id.clone()))?;
        let new_stock = settlement
            .inventory
            .get(&command.good)
            .copied()
            .unwrap_or(0)
            .checked_add(command.quantity)
            .ok_or_else(|| SimulationError::InventoryOverflow(command.settlement_id.clone()))?;

        *next
            .resource_sites
            .get_mut(&command.site_id)
            .unwrap()
            .stocks
            .entry(command.good)
            .or_default() -= command.quantity;
        next.settlements
            .get_mut(&command.settlement_id)
            .unwrap()
            .inventory
            .insert(command.good, new_stock);
        let sequence = next.next_sequence();
        next.events.push(WorldEvent {
            world_id: next.id,
            tick: next.tick,
            sequence,
            actor: command.actor,
            cause: "resource_extraction".into(),
            affected_entities: vec![command.site_id.clone(), command.settlement_id.clone()],
            provenance: Provenance::GeneratedSimulation,
            ruleset_version: next.ruleset_version,
            payload: EventPayload::ResourceExtracted {
                site_id: command.site_id,
                settlement_id: command.settlement_id,
                good: command.good,
                amount: command.quantity,
            },
        });
        *self = next;
        Ok(())
    }

    pub fn execute_trade(&mut self, command: TradeCommand) -> Result<(), SimulationError> {
        if command.quantity == 0 || command.unit_price <= 0 {
            return Err(SimulationError::InvalidTrade);
        }
        if command.seller_id == command.buyer_id {
            return Err(SimulationError::SameSettlementTrade);
        }
        let mut next = self.clone();
        let seller = next
            .settlements
            .get(&command.seller_id)
            .ok_or_else(|| SimulationError::UnknownSettlement(command.seller_id.clone()))?;
        let buyer = next
            .settlements
            .get(&command.buyer_id)
            .ok_or_else(|| SimulationError::UnknownSettlement(command.buyer_id.clone()))?;
        let goods_available = seller.inventory.get(&command.good).copied().unwrap_or(0);
        if goods_available < command.quantity {
            return Err(SimulationError::InsufficientGoods {
                settlement_id: command.seller_id,
                requested: command.quantity,
                available: goods_available,
            });
        }
        let total_price = i64::try_from(command.quantity)
            .ok()
            .and_then(|quantity| quantity.checked_mul(command.unit_price))
            .ok_or(SimulationError::TreasuryOverflow(command.buyer_id.clone()))?;
        if buyer.treasury < total_price {
            return Err(SimulationError::InsufficientFunds {
                settlement_id: command.buyer_id,
                required: total_price,
                available: buyer.treasury,
            });
        }
        let seller_treasury = seller
            .treasury
            .checked_add(total_price)
            .ok_or_else(|| SimulationError::TreasuryOverflow(command.seller_id.clone()))?;
        let buyer_goods = buyer
            .inventory
            .get(&command.good)
            .copied()
            .unwrap_or(0)
            .checked_add(command.quantity)
            .ok_or_else(|| SimulationError::InventoryOverflow(command.buyer_id.clone()))?;

        let seller = next.settlements.get_mut(&command.seller_id).unwrap();
        *seller.inventory.entry(command.good).or_default() -= command.quantity;
        seller.treasury = seller_treasury;
        let buyer = next.settlements.get_mut(&command.buyer_id).unwrap();
        buyer.inventory.insert(command.good, buyer_goods);
        buyer.treasury -= total_price;

        let sequence = next.next_sequence();
        next.events.push(WorldEvent {
            world_id: next.id,
            tick: next.tick,
            sequence,
            actor: command.actor,
            cause: "bilateral_trade".into(),
            affected_entities: vec![command.seller_id.clone(), command.buyer_id.clone()],
            provenance: Provenance::GeneratedSimulation,
            ruleset_version: next.ruleset_version,
            payload: EventPayload::TradeExecuted {
                seller_id: command.seller_id,
                buyer_id: command.buyer_id,
                good: command.good,
                quantity: command.quantity,
                total_price,
            },
        });
        *self = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> World {
        let river = Settlement {
            id: "river-town".into(),
            name: "River Town".into(),
            population: 100,
            treasury: 500,
            inventory: [(Good::Food, 200), (Good::Timber, 20)]
                .into_iter()
                .collect(),
            food_per_person_per_tick: 1,
            production: vec![ProductionRecipe {
                id: "fishery".into(),
                output: Good::Food,
                output_amount: 30,
                inputs: BTreeMap::new(),
            }],
        };
        let hill = Settlement {
            id: "hill-village".into(),
            name: "Hill Village".into(),
            population: 40,
            treasury: 100,
            inventory: [(Good::Food, 100), (Good::Stone, 5)].into_iter().collect(),
            food_per_person_per_tick: 1,
            production: vec![],
        };
        let site = ResourceSite {
            id: "stone-quarry".into(),
            name: "Stone Quarry".into(),
            stocks: [(Good::Stone, 1000)].into_iter().collect(),
        };
        World {
            id: Uuid::nil(),
            seed: 42,
            ruleset_version: 1,
            tick: 0,
            settlements: [(river.id.clone(), river), (hill.id.clone(), hill)]
                .into_iter()
                .collect(),
            resource_sites: [(site.id.clone(), site)].into_iter().collect(),
            events: vec![],
        }
    }

    #[test]
    fn tick_produces_consumes_and_records_events() {
        let mut w = world();
        w.advance_tick().unwrap();
        assert_eq!(w.settlements["river-town"].inventory[&Good::Food], 130);
        assert_eq!(w.settlements["hill-village"].inventory[&Good::Food], 60);
        assert_eq!(w.tick, 1);
        assert!(w
            .events
            .iter()
            .any(|e| matches!(e.payload, EventPayload::FoodConsumed { amount: 100, .. })));
    }

    #[test]
    fn failed_tick_is_atomic() {
        let mut w = world();
        w.settlements
            .get_mut("river-town")
            .unwrap()
            .inventory
            .insert(Good::Food, 0);
        let before = w.clone();
        assert!(w.advance_tick().is_err());
        assert_eq!(w, before);
    }

    #[test]
    fn extraction_conserves_stock_and_is_audited() {
        let mut w = world();
        w.extract_resource(ExtractionCommand {
            site_id: "stone-quarry".into(),
            settlement_id: "hill-village".into(),
            good: Good::Stone,
            quantity: 25,
            actor: "player".into(),
        })
        .unwrap();
        assert_eq!(w.resource_sites["stone-quarry"].stocks[&Good::Stone], 975);
        assert_eq!(w.settlements["hill-village"].inventory[&Good::Stone], 30);
        assert!(matches!(
            w.events[0].payload,
            EventPayload::ResourceExtracted { amount: 25, .. }
        ));
    }

    #[test]
    fn trade_transfers_goods_and_treasury_atomically() {
        let mut w = world();
        w.execute_trade(TradeCommand {
            seller_id: "hill-village".into(),
            buyer_id: "river-town".into(),
            good: Good::Stone,
            quantity: 3,
            unit_price: 10,
            actor: "player".into(),
        })
        .unwrap();
        assert_eq!(w.settlements["hill-village"].inventory[&Good::Stone], 2);
        assert_eq!(w.settlements["river-town"].inventory[&Good::Stone], 3);
        assert_eq!(w.settlements["hill-village"].treasury, 130);
        assert_eq!(w.settlements["river-town"].treasury, 470);
    }

    #[test]
    fn rejected_trade_does_not_mutate_world() {
        let mut w = world();
        let before = w.clone();
        let result = w.execute_trade(TradeCommand {
            seller_id: "hill-village".into(),
            buyer_id: "river-town".into(),
            good: Good::Iron,
            quantity: 100,
            unit_price: 10,
            actor: "player".into(),
        });
        assert!(result.is_err());
        assert_eq!(w, before);
    }

    #[test]
    fn replay_and_snapshot_round_trip_are_deterministic() {
        let mut a = world();
        let mut b = world();
        a.settlements.get_mut("river-town").unwrap().population = 0;
        b.settlements.get_mut("river-town").unwrap().population = 0;
        for _ in 0..4 {
            a.advance_tick().unwrap();
            b.advance_tick().unwrap();
        }
        assert_eq!(a, b);
        let encoded = serde_json::to_vec(&a).unwrap();
        let decoded: World = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(a, decoded);
    }
}
