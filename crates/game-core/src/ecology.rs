use crate::{
    geography::GridPoint,
    simulation::{EventDraft, SimulationSystem, SystemError},
    world::{EntityId, EventPayload, Provenance, Tick, WorldState},
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    WildFood,
    Fish,
    Timber,
    Stone,
    Iron,
}

impl ResourceKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WildFood => "wild_food",
            Self::Fish => "fish",
            Self::Timber => "timber",
            Self::Stone => "stone",
            Self::Iron => "iron",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceSite {
    pub id: EntityId,
    pub kind: ResourceKind,
    pub location: GridPoint,
    pub stock_units: u64,
    pub capacity_units: u64,
    pub regeneration_units_per_tick: u64,
}

impl ResourceSite {
    pub fn validate(&self) -> Result<(), ResourceSiteError> {
        if self.stock_units > self.capacity_units {
            return Err(ResourceSiteError::StockExceedsCapacity {
                stock: self.stock_units,
                capacity: self.capacity_units,
            });
        }
        Ok(())
    }

    pub const fn renewable(&self) -> bool {
        self.regeneration_units_per_tick > 0
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ResourceSiteError {
    #[error("resource stock {stock} exceeds capacity {capacity}")]
    StockExceedsCapacity { stock: u64, capacity: u64 },
}

pub struct EcologySystem;

impl SimulationSystem for EcologySystem {
    fn id(&self) -> &'static str {
        "ecology-resource-regeneration-v1"
    }

    fn advance(
        &self,
        world: &mut WorldState,
        _tick: Tick,
        _random: &mut dyn RngCore,
    ) -> Result<Vec<EventDraft>, SystemError> {
        let site_ids: Vec<EntityId> = world.resource_sites.keys().copied().collect();
        let mut drafts = Vec::new();

        for id in site_ids {
            let site = world
                .resource_sites
                .get_mut(&id)
                .ok_or_else(|| SystemError::new("resource site disappeared during tick"))?;
            site.validate()
                .map_err(|error| SystemError::new(error.to_string()))?;

            let available_capacity = site.capacity_units - site.stock_units;
            let regenerated = site.regeneration_units_per_tick.min(available_capacity);
            if regenerated == 0 {
                continue;
            }

            let new_stock = site
                .stock_units
                .checked_add(regenerated)
                .ok_or_else(|| SystemError::new("resource regeneration overflow"))?;
            let delta = i64::try_from(regenerated)
                .map_err(|_| SystemError::new("resource regeneration exceeds event delta range"))?;
            site.stock_units = new_stock;
            drafts.push(EventDraft {
                actor: "ecology".into(),
                cause: "natural_resource_regeneration".into(),
                affected_entities: vec![id],
                provenance: Provenance::GeneratedSimulation,
                payload: EventPayload::ResourceStockChanged {
                    site_id: id,
                    resource: site.kind.as_str().into(),
                    delta,
                },
            });
        }

        Ok(drafts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        geography::WorldMap,
        simulation::SimulationEngine,
        world::{EntityKind, WorldState},
    };

    fn world_with_site(stock: u64, capacity: u64, regeneration: u64) -> (WorldState, EntityId) {
        let mut world = WorldState::new(41, 2);
        let id = world
            .register_resource_site(
                "Тестовый ресурс".into(),
                ResourceKind::Timber,
                GridPoint { x: 1, y: 1 },
                stock,
                capacity,
                regeneration,
            )
            .expect("valid test resource site");
        (world, id)
    }

    #[test]
    fn renewable_resource_regenerates_without_exceeding_capacity() {
        let (mut world, id) = world_with_site(98, 100, 7);
        let events = SimulationEngine::new(vec![Box::new(EcologySystem)])
            .advance_tick(&mut world)
            .expect("ecology tick");
        assert_eq!(world.resource_sites[&id].stock_units, 100);
        assert_eq!(events.len(), 1);
        let next_events = SimulationEngine::new(vec![Box::new(EcologySystem)])
            .advance_tick(&mut world)
            .expect("second ecology tick");
        assert!(next_events.is_empty());
        assert_eq!(world.resource_sites[&id].stock_units, 100);
        assert!(matches!(
            &events[0].payload,
            EventPayload::ResourceStockChanged {
                site_id,
                resource,
                delta: 2
            } if *site_id == id && resource == "timber"
        ));
    }

    #[test]
    fn nonrenewable_resource_does_not_regenerate() {
        let (mut world, id) = world_with_site(75, 100, 0);
        SimulationEngine::new(vec![Box::new(EcologySystem)])
            .advance_tick(&mut world)
            .expect("ecology tick");
        assert_eq!(world.resource_sites[&id].stock_units, 75);
        assert!(world.event_log.is_empty());
    }

    #[test]
    fn invalid_stock_fails_atomically() {
        let (mut world, id) = world_with_site(50, 100, 5);
        world.resource_sites.get_mut(&id).expect("site").stock_units = 101;
        let before = world.clone();
        let result = SimulationEngine::new(vec![Box::new(EcologySystem)]).advance_tick(&mut world);
        assert!(result.is_err());
        assert_eq!(world, before);
    }

    #[test]
    fn starter_resource_sites_are_deterministic_and_spatially_valid() {
        let first = WorldState::starter_v2(42);
        let second = WorldState::starter_v2(42);
        assert_eq!(first.resource_sites, second.resource_sites);
        assert!(!first.resource_sites.is_empty());
        assert!(first.resource_sites.values().all(|site| {
            site.validate().is_ok() && first.geography.cell_at(site.location).is_some()
        }));
        assert!(
            first
                .entities
                .values()
                .filter(|entity| entity.kind == EntityKind::ResourceSite)
                .count()
                == first.resource_sites.len()
        );
        assert!(WorldMap::starter(42).validate().is_ok());
    }
}
