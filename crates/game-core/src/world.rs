use crate::geography::{GridPoint, WorldMap};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorldId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EntityId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Tick(pub u64);

impl Tick {
    pub fn checked_next(self) -> Option<Self> {
        self.0.checked_add(1).map(Self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    Person,
    Household,
    Settlement,
    Firm,
    Bank,
    State,
    Institution,
    ResourceSite,
    Infrastructure,
    CulturalGroup,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityRecord {
    pub id: EntityId,
    pub kind: EntityKind,
    pub name: String,
    pub created_at: Tick,
    pub location: Option<GridPoint>,
    pub attributes: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Provenance {
    GeneratedSimulation,
    HistoricalSource { source_id: String },
    ScholarlyInterpretation { source_id: String },
    ModelAssumption { assumption_id: String },
    DerivedEstimate { method: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventPayload {
    WorldCreated,
    TickAdvanced,
    EntityRegistered {
        entity_id: EntityId,
        kind: EntityKind,
    },
    PopulationChanged {
        settlement_id: EntityId,
        delta: i64,
        reason: String,
    },
    ResourceStockChanged {
        site_id: EntityId,
        resource: String,
        delta: i64,
    },
    ProductionCompleted {
        organization_id: EntityId,
        good: String,
        quantity: u64,
    },
    TradeSettled {
        buyer_id: EntityId,
        seller_id: EntityId,
        good: String,
        quantity: u64,
    },
    CreditChanged {
        lender_id: EntityId,
        borrower_id: EntityId,
        delta_minor_units: i64,
    },
    InstitutionChanged {
        institution_id: EntityId,
        change: String,
    },
    PowerChanged {
        from_entity: EntityId,
        to_entity: EntityId,
        authority: String,
    },
    CultureChanged {
        group_id: EntityId,
        change: String,
    },
    Custom {
        domain: String,
        event_type: String,
        payload: serde_json::Value,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: u64,
    pub world_id: WorldId,
    pub tick: Tick,
    pub sequence: u64,
    pub actor: String,
    pub cause: String,
    pub affected_entities: Vec<EntityId>,
    pub ruleset_version: u32,
    pub provenance: Provenance,
    pub payload: EventPayload,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldState {
    pub id: WorldId,
    pub seed: u64,
    pub ruleset_version: u32,
    pub tick: Tick,
    pub next_entity_id: u64,
    pub next_event_sequence: u64,
    pub event_log: Vec<EventEnvelope>,
    pub geography: WorldMap,
    pub entities: BTreeMap<EntityId, EntityRecord>,
}

struct SettlementSetup {
    population: u64,
    food_stock: u64,
    food_production: u64,
    food_per_person: u64,
    timber_stock: u64,
    timber_regeneration: u64,
}

impl WorldState {
    pub fn new(seed: u64, ruleset_version: u32) -> Self {
        Self {
            id: WorldId(seed),
            seed,
            ruleset_version,
            tick: Tick(0),
            next_entity_id: 1,
            next_event_sequence: 0,
            event_log: Vec::new(),
            geography: WorldMap::starter(seed),
            entities: BTreeMap::new(),
        }
    }

    pub fn starter(seed: u64) -> Self {
        let mut world = Self::new(seed, 1);
        let river = world
            .register_entity(EntityKind::Settlement, "Речной край".into())
            .expect("initial entity id");
        let hill = world
            .register_entity(EntityKind::Settlement, "Каменный холм".into())
            .expect("initial entity id");
        world.settlement_attributes(
            river,
            SettlementSetup {
                population: 120,
                food_stock: 900,
                food_production: 250,
                food_per_person: 2,
                timber_stock: 500,
                timber_regeneration: 18,
            },
        );
        world.set_entity_location(river, GridPoint { x: 2, y: 2 });
        world.set_entity_location(hill, GridPoint { x: 5, y: 4 });
        world.settlement_attributes(
            hill,
            SettlementSetup {
                population: 80,
                food_stock: 600,
                food_production: 170,
                food_per_person: 2,
                timber_stock: 220,
                timber_regeneration: 12,
            },
        );
        world.event_log.push(EventEnvelope {
            event_id: 0,
            world_id: world.id,
            tick: Tick(0),
            sequence: 0,
            actor: "world_factory".into(),
            cause: "starter_scenario_created".into(),
            affected_entities: vec![river, hill],
            ruleset_version: world.ruleset_version,
            provenance: Provenance::ModelAssumption {
                assumption_id: "starter-settlements-v1".into(),
            },
            payload: EventPayload::WorldCreated,
        });
        world.next_event_sequence = 1;
        world
    }

    fn settlement_attributes(&mut self, id: EntityId, setup: SettlementSetup) {
        if let Some(entity) = self.entities.get_mut(&id) {
            entity
                .attributes
                .insert("population".into(), setup.population.into());
            entity
                .attributes
                .insert("food".into(), setup.food_stock.into());
            entity
                .attributes
                .insert("food_production".into(), setup.food_production.into());
            entity
                .attributes
                .insert("food_per_person".into(), setup.food_per_person.into());
            entity
                .attributes
                .insert("timber".into(), setup.timber_stock.into());
            entity.attributes.insert(
                "timber_regeneration".into(),
                setup.timber_regeneration.into(),
            );
            entity.attributes.insert("treasury".into(), 1000_u64.into());
            entity
                .attributes
                .insert("tax_rate_basis_points".into(), 500_u64.into());
        }
    }

    pub fn set_entity_location(&mut self, id: EntityId, position: GridPoint) -> bool {
        if self.geography.cell_at(position).is_none() {
            return false;
        }
        let Some(entity) = self.entities.get_mut(&id) else {
            return false;
        };
        entity.location = Some(position);
        true
    }

    pub fn register_entity(&mut self, kind: EntityKind, name: String) -> Option<EntityId> {
        let id = EntityId(self.next_entity_id);
        self.next_entity_id = self.next_entity_id.checked_add(1)?;
        self.entities.insert(
            id,
            EntityRecord {
                id,
                kind,
                name,
                created_at: self.tick,
                location: None,
                attributes: BTreeMap::new(),
            },
        );
        Some(id)
    }
}
