use crate::geography::{GridPoint, WorldMap};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const CURRENT_RULESET_VERSION: u32 = 2;

fn choose_resource_position(
    world: &WorldState,
    preferred: &[crate::geography::TerrainKind],
    used: &[GridPoint],
) -> GridPoint {
    world
        .geography
        .cells
        .iter()
        .find(|cell| preferred.contains(&cell.terrain) && !used.contains(&cell.position))
        .or_else(|| {
            world
                .geography
                .cells
                .iter()
                .find(|cell| !used.contains(&cell.position))
        })
        .map(|cell| cell.position)
        .unwrap_or(GridPoint { x: 0, y: 0 })
}

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
    #[serde(skip)]
    pub event_log: Vec<EventEnvelope>,
    pub geography: WorldMap,
    pub entities: BTreeMap<EntityId, EntityRecord>,
    #[serde(default)]
    pub resource_sites: BTreeMap<EntityId, ResourceSite>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    ForestBiomass,
    Stone,
    IronOre,
    FreshWater,
}

impl ResourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ForestBiomass => "forest_biomass",
            Self::Stone => "stone",
            Self::IronOre => "iron_ore",
            Self::FreshWater => "fresh_water",
        }
    }

    pub fn unit(self) -> ResourceUnit {
        match self {
            Self::ForestBiomass | Self::Stone => ResourceUnit::CubicMeters,
            Self::IronOre => ResourceUnit::Tonnes,
            Self::FreshWater => ResourceUnit::CubicMeters,
        }
    }

    pub fn is_renewable(self) -> bool {
        matches!(self, Self::ForestBiomass | Self::FreshWater)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceUnit {
    CubicMeters,
    Tonnes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceSite {
    pub id: EntityId,
    pub name: String,
    pub kind: ResourceKind,
    pub location: GridPoint,
    pub stock: u64,
    pub capacity: u64,
    pub regeneration_per_tick: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceSiteError {
    InvalidLocation,
    StockExceedsCapacity,
    NonRenewableRegeneration,
    EntityIdExhausted,
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
            resource_sites: BTreeMap::new(),
        }
    }

    pub fn starter(seed: u64) -> Self {
        let mut world = Self::new(seed, CURRENT_RULESET_VERSION);
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
                timber_regeneration: 0,
            },
        );
        assert!(world.set_entity_location(river, GridPoint { x: 2, y: 2 }));
        assert!(world.set_entity_location(hill, GridPoint { x: 5, y: 4 }));
        world.settlement_attributes(
            hill,
            SettlementSetup {
                population: 80,
                food_stock: 600,
                food_production: 170,
                food_per_person: 2,
                timber_stock: 220,
                timber_regeneration: 0,
            },
        );

        let mut used_positions = vec![GridPoint { x: 2, y: 2 }, GridPoint { x: 5, y: 4 }];
        let forest_position = choose_resource_position(
            &world,
            &[crate::geography::TerrainKind::Forest],
            &used_positions,
        );
        used_positions.push(forest_position);
        let stone_position = choose_resource_position(
            &world,
            &[
                crate::geography::TerrainKind::Mountain,
                crate::geography::TerrainKind::Hills,
            ],
            &used_positions,
        );
        used_positions.push(stone_position);
        let iron_position = choose_resource_position(
            &world,
            &[crate::geography::TerrainKind::Mountain],
            &used_positions,
        );
        used_positions.push(iron_position);
        let water_position = choose_resource_position(
            &world,
            &[crate::geography::TerrainKind::River],
            &used_positions,
        );

        let forest = world
            .add_resource_site(
                "Лесной массив".into(),
                ResourceKind::ForestBiomass,
                forest_position,
                850,
                1_200,
                8,
            )
            .expect("valid starter forest");
        let quarry = world
            .add_resource_site(
                "Каменный карьер".into(),
                ResourceKind::Stone,
                stone_position,
                600,
                1_000,
                0,
            )
            .expect("valid starter quarry");
        let ore = world
            .add_resource_site(
                "Железное месторождение".into(),
                ResourceKind::IronOre,
                iron_position,
                350,
                700,
                0,
            )
            .expect("valid starter ore deposit");
        let water = world
            .add_resource_site(
                "Речная система".into(),
                ResourceKind::FreshWater,
                water_position,
                8_000,
                12_000,
                50,
            )
            .expect("valid starter water system");

        world.event_log.push(EventEnvelope {
            event_id: 0,
            world_id: world.id,
            tick: Tick(0),
            sequence: 0,
            actor: "world_factory".into(),
            cause: "starter_scenario_created".into(),
            affected_entities: vec![river, hill, forest, quarry, ore, water],
            ruleset_version: world.ruleset_version,
            provenance: Provenance::ModelAssumption {
                assumption_id: "starter-world-v2".into(),
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

    pub fn add_resource_site(
        &mut self,
        name: String,
        kind: ResourceKind,
        location: GridPoint,
        stock: u64,
        capacity: u64,
        regeneration_per_tick: u64,
    ) -> Result<EntityId, ResourceSiteError> {
        if self.geography.cell_at(location).is_none() {
            return Err(ResourceSiteError::InvalidLocation);
        }
        if stock > capacity {
            return Err(ResourceSiteError::StockExceedsCapacity);
        }
        if !kind.is_renewable() && regeneration_per_tick != 0 {
            return Err(ResourceSiteError::NonRenewableRegeneration);
        }
        let id = self
            .register_entity(EntityKind::ResourceSite, name.clone())
            .ok_or(ResourceSiteError::EntityIdExhausted)?;
        if !self.set_entity_location(id, location) {
            return Err(ResourceSiteError::InvalidLocation);
        }
        self.resource_sites.insert(
            id,
            ResourceSite {
                id,
                name,
                kind,
                location,
                stock,
                capacity,
                regeneration_per_tick,
            },
        );
        Ok(id)
    }

    pub fn validate(&self) -> Result<(), String> {
        self.geography
            .validate()
            .map_err(|error| format!("invalid world geography: {error}"))?;
        for (id, entity) in &self.entities {
            if *id != entity.id {
                return Err(format!("entity key does not match entity id: {}", id.0));
            }
            if entity
                .location
                .is_some_and(|location| self.geography.cell_at(location).is_none())
            {
                return Err(format!("entity {} is outside world geography", id.0));
            }
            if entity.kind == EntityKind::ResourceSite && !self.resource_sites.contains_key(id) {
                return Err(format!(
                    "resource-site entity {} has no typed resource state",
                    id.0
                ));
            }
        }
        if self
            .entities
            .keys()
            .next_back()
            .is_some_and(|id| self.next_entity_id <= id.0)
        {
            return Err("next entity id does not exceed registered entity ids".into());
        }
        for (id, site) in &self.resource_sites {
            if *id != site.id {
                return Err(format!(
                    "resource-site key does not match site id: {}",
                    id.0
                ));
            }
            let entity = self
                .entities
                .get(id)
                .ok_or_else(|| format!("resource site {} has no entity record", id.0))?;
            if entity.kind != EntityKind::ResourceSite || entity.location != Some(site.location) {
                return Err(format!(
                    "resource site {} has inconsistent entity metadata",
                    id.0
                ));
            }
            if self.geography.cell_at(site.location).is_none() {
                return Err(format!("resource site {} is outside world geography", id.0));
            }
            if site.stock > site.capacity {
                return Err(format!("resource site {} exceeds its capacity", id.0));
            }
            if !site.kind.is_renewable() && site.regeneration_per_tick != 0 {
                return Err(format!("non-renewable resource site {} regenerates", id.0));
            }
        }
        Ok(())
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
