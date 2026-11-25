pub mod ecology;
pub mod geography;
pub mod simulation;
pub mod world;

pub use ecology::{EcologySystem, ResourceKind, ResourceSite, ResourceSiteError};
pub use geography::{
    ClimateBand, GridPoint, MapCell, MapError, Region, RegionId, TerrainKind, WorldMap,
};
pub use simulation::{
    EventDraft, SimulationEngine, SimulationError, SimulationSystem, SystemError,
};
pub use world::{
    EntityId, EntityKind, EntityRecord, EventEnvelope, EventPayload, Provenance, Tick, WorldId,
    WorldState,
};
