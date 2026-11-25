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
    pub entities: BTreeMap<EntityId, EntityRecord>,
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
            entities: BTreeMap::new(),
        }
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
                attributes: BTreeMap::new(),
            },
        );
        Some(id)
    }
}
