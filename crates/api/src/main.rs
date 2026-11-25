use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sherlock_game_core::{
    EntityKind, EventEnvelope, EventPayload, ResourceKind, SimulationEngine, TerrainKind, WorldId,
    WorldState,
};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{env, net::SocketAddr};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[derive(Clone)]
struct AppState {
    pool: PgPool,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

#[derive(Deserialize)]
struct CreateWorldRequest {
    seed: Option<u64>,
}

#[derive(Debug, Serialize)]
struct SettlementView {
    id: u64,
    name: String,
    x: u16,
    y: u16,
    population: u64,
    food: u64,
    timber: u64,
    treasury: u64,
}

#[derive(Debug, Serialize)]
struct MapCellView {
    x: u16,
    y: u16,
    terrain: &'static str,
    climate: &'static str,
    elevation_meters: u16,
    soil_fertility: u8,
    water_access: bool,
    settlements: Vec<String>,
    resources: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ResourceSiteView {
    id: u64,
    kind: &'static str,
    x: u16,
    y: u16,
    stock_units: u64,
    capacity_units: u64,
    regeneration_units_per_tick: u64,
}

#[derive(Debug, Serialize)]
struct MapView {
    width: u16,
    height: u16,
    cells: Vec<MapCellView>,
}

#[derive(Debug, Serialize)]
struct EventView {
    tick: u64,
    sequence: u64,
    actor: String,
    cause: String,
    event_type: String,
}

#[derive(Debug, Serialize)]
struct WorldView {
    world_id: u64,
    seed: u64,
    tick: u64,
    ruleset_version: u32,
    entity_count: usize,
    event_count: usize,
    settlements: Vec<SettlementView>,
    resource_sites: Vec<ResourceSiteView>,
    map: MapView,
    recent_events: Vec<EventView>,
}

#[derive(Debug, Serialize)]
struct TickResponse {
    world: WorldView,
    emitted_events: usize,
}

#[derive(Debug, Serialize)]
struct ReplayView {
    world_id: u64,
    tick: u64,
    replayed_ticks: u64,
    event_count: usize,
    snapshot_matches: bool,
    event_log_matches: bool,
    verified: bool,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "sherlock-civilization-api",
    })
}

fn event_type(event: &EventEnvelope) -> String {
    match &event.payload {
        EventPayload::WorldCreated => "world_created".into(),
        EventPayload::TickAdvanced => "tick_advanced".into(),
        EventPayload::EntityRegistered { .. } => "entity_registered".into(),
        EventPayload::PopulationChanged { .. } => "population_changed".into(),
        EventPayload::ResourceStockChanged { resource, .. } => {
            format!("resource_stock_changed:{resource}")
        }
        EventPayload::ProductionCompleted { good, .. } => {
            format!("production_completed:{good}")
        }
        EventPayload::TradeSettled { good, .. } => format!("trade_settled:{good}"),
        EventPayload::CreditChanged { .. } => "credit_changed".into(),
        EventPayload::InstitutionChanged { .. } => "institution_changed".into(),
        EventPayload::PowerChanged { .. } => "power_changed".into(),
        EventPayload::CultureChanged { .. } => "culture_changed".into(),
        EventPayload::Custom {
            domain, event_type, ..
        } => format!("{domain}.{event_type}"),
    }
}

fn terrain_name(terrain: TerrainKind) -> &'static str {
    match terrain {
        TerrainKind::Plains => "plains",
        TerrainKind::Forest => "forest",
        TerrainKind::Hills => "hills",
        TerrainKind::Mountain => "mountain",
        TerrainKind::River => "river",
    }
}

fn climate_name(climate: sherlock_game_core::ClimateBand) -> &'static str {
    match climate {
        sherlock_game_core::ClimateBand::Boreal => "boreal",
        sherlock_game_core::ClimateBand::Temperate => "temperate",
        sherlock_game_core::ClimateBand::Subtropical => "subtropical",
    }
}

fn resource_name(kind: ResourceKind) -> &'static str {
    kind.as_str()
}

fn world_view(world: &WorldState) -> Result<WorldView, StatusCode> {
    let settlements = world
        .entities
        .values()
        .filter(|entity| entity.kind == EntityKind::Settlement)
        .map(|entity| {
            let number = |key: &str| {
                entity
                    .attributes
                    .get(key)
                    .and_then(serde_json::Value::as_u64)
                    .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
            };
            let position = entity.location.ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(SettlementView {
                id: entity.id.0,
                name: entity.name.clone(),
                x: position.x,
                y: position.y,
                population: number("population")?,
                food: number("food")?,
                timber: number("timber")?,
                treasury: number("treasury")?,
            })
        })
        .collect::<Result<Vec<_>, StatusCode>>()?;

    let map = MapView {
        width: world.geography.width,
        height: world.geography.height,
        cells: world
            .geography
            .cells
            .iter()
            .map(|cell| MapCellView {
                x: cell.position.x,
                y: cell.position.y,
                terrain: terrain_name(cell.terrain),
                climate: climate_name(cell.climate),
                elevation_meters: cell.elevation_meters,
                soil_fertility: cell.soil_fertility,
                water_access: cell.water_access,
                settlements: world
                    .entities
                    .values()
                    .filter(|entity| {
                        entity.kind == EntityKind::Settlement
                            && entity.location == Some(cell.position)
                    })
                    .map(|entity| entity.name.clone())
                    .collect(),
                resources: world
                    .resource_sites
                    .values()
                    .filter(|site| site.location == cell.position)
                    .map(|site| format!("{}: {}", resource_name(site.kind), site.stock_units))
                    .collect(),
            })
            .collect(),
    };

    let resource_sites = world
        .resource_sites
        .values()
        .map(|site| ResourceSiteView {
            id: site.id.0,
            kind: resource_name(site.kind),
            x: site.location.x,
            y: site.location.y,
            stock_units: site.stock_units,
            capacity_units: site.capacity_units,
            regeneration_units_per_tick: site.regeneration_units_per_tick,
        })
        .collect();

    let recent_events = world
        .event_log
        .iter()
        .rev()
        .take(12)
        .map(|event| EventView {
            tick: event.tick.0,
            sequence: event.sequence,
            actor: event.actor.clone(),
            cause: event.cause.clone(),
            event_type: event_type(event),
        })
        .collect();

    Ok(WorldView {
        world_id: world.id.0,
        seed: world.seed,
        tick: world.tick.0,
        ruleset_version: world.ruleset_version,
        entity_count: world.entities.len(),
        event_count: world.event_log.len(),
        settlements,
        resource_sites,
        map,
        recent_events,
    })
}

fn database_id(world_id: u64) -> Result<i64, StatusCode> {
    i64::try_from(world_id).map_err(|_| StatusCode::NOT_FOUND)
}

async fn load_events(
    connection: &mut sqlx::PgConnection,
    world_id: i64,
) -> Result<Vec<EventEnvelope>, StatusCode> {
    let values = sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT event FROM world_events WHERE world_id = $1 ORDER BY sequence",
    )
    .bind(world_id)
    .fetch_all(&mut *connection)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    values
        .into_iter()
        .map(serde_json::from_value)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn load_world(pool: &PgPool, world_id: i64) -> Result<WorldState, StatusCode> {
    let mut transaction = pool
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
        .execute(&mut *transaction)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let snapshot = sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT current_snapshot FROM worlds WHERE id = $1",
    )
    .bind(world_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
    let mut world: WorldState =
        serde_json::from_value(snapshot).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    world.event_log = load_events(&mut transaction, world_id).await?;
    transaction
        .commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(world)
}

async fn insert_events(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    world_id: i64,
    events: &[EventEnvelope],
) -> Result<(), StatusCode> {
    for event in events {
        let sequence = i64::try_from(event.sequence).map_err(|_| StatusCode::CONFLICT)?;
        let tick = i64::try_from(event.tick.0).map_err(|_| StatusCode::CONFLICT)?;
        let payload = serde_json::to_value(event).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        sqlx::query(
            "INSERT INTO world_events (world_id, sequence, tick, event_type, event) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(world_id)
        .bind(sequence)
        .bind(tick)
        .bind(event_type(event))
        .bind(payload)
        .execute(&mut **transaction)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    Ok(())
}

async fn create_world(
    State(state): State<AppState>,
    Json(request): Json<CreateWorldRequest>,
) -> Result<Json<WorldView>, StatusCode> {
    let seed = request.seed.unwrap_or(42);
    let database_seed = i64::try_from(seed).map_err(|_| StatusCode::BAD_REQUEST)?;
    let mut transaction = state
        .pool
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut world = WorldState::starter_v2(seed);
    let world_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO worlds (seed, ruleset_version, tick) VALUES ($1, $2, 0) RETURNING id",
    )
    .bind(database_seed)
    .bind(i32::try_from(world.ruleset_version).map_err(|_| StatusCode::BAD_REQUEST)?)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    world.id = WorldId(u64::try_from(world_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?);
    for event in &mut world.event_log {
        event.world_id = world.id;
    }
    let snapshot = serde_json::to_value(&world).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    sqlx::query("UPDATE worlds SET current_snapshot = $2 WHERE id = $1")
        .bind(world_id)
        .bind(&snapshot)
        .execute(&mut *transaction)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    sqlx::query(
        "INSERT INTO world_snapshots (world_id, tick, ruleset_version, state) VALUES ($1, 0, $2, $3)",
    )
    .bind(world_id)
    .bind(i32::try_from(world.ruleset_version).map_err(|_| StatusCode::BAD_REQUEST)?)
    .bind(snapshot)
    .execute(&mut *transaction)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    insert_events(&mut transaction, world_id, &world.event_log).await?;
    transaction
        .commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(world_view(&world)?))
}

async fn get_world(
    Path(world_id): Path<u64>,
    State(state): State<AppState>,
) -> Result<Json<WorldView>, StatusCode> {
    let world = load_world(&state.pool, database_id(world_id)?).await?;
    Ok(Json(world_view(&world)?))
}

async fn advance_tick(
    Path(world_id): Path<u64>,
    State(state): State<AppState>,
) -> Result<Json<TickResponse>, StatusCode> {
    let database_id = database_id(world_id)?;
    let mut transaction = state
        .pool
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let snapshot = sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT current_snapshot FROM worlds WHERE id = $1 FOR UPDATE",
    )
    .bind(database_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
    let mut world: WorldState =
        serde_json::from_value(snapshot).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    world.event_log = load_events(&mut transaction, database_id).await?;
    let previous_event_count = world.event_log.len();
    let events = SimulationEngine::civilization()
        .advance_tick(&mut world)
        .map_err(|_| StatusCode::CONFLICT)?;
    let snapshot = serde_json::to_value(&world).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tick = i64::try_from(world.tick.0).map_err(|_| StatusCode::CONFLICT)?;
    sqlx::query(
        "UPDATE worlds SET tick = $2, current_snapshot = $3, updated_at = NOW() WHERE id = $1",
    )
    .bind(database_id)
    .bind(tick)
    .bind(&snapshot)
    .execute(&mut *transaction)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    sqlx::query(
        "INSERT INTO world_snapshots (world_id, tick, ruleset_version, state) VALUES ($1, $2, $3, $4)",
    )
    .bind(database_id)
    .bind(tick)
    .bind(i32::try_from(world.ruleset_version).map_err(|_| StatusCode::BAD_REQUEST)?)
    .bind(snapshot)
    .execute(&mut *transaction)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    insert_events(
        &mut transaction,
        database_id,
        &world.event_log[previous_event_count..],
    )
    .await?;
    transaction
        .commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(TickResponse {
        world: world_view(&world)?,
        emitted_events: events.len(),
    }))
}

async fn verify_replay(
    Path(world_id): Path<u64>,
    State(state): State<AppState>,
) -> Result<Json<ReplayView>, StatusCode> {
    const MAX_REPLAY_TICKS: u64 = 100_000;
    let world = load_world(&state.pool, database_id(world_id)?).await?;
    if world.tick.0 > MAX_REPLAY_TICKS {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let mut replayed = WorldState::starter_for_ruleset(world.seed, world.ruleset_version)
        .ok_or(StatusCode::UNPROCESSABLE_ENTITY)?;
    replayed.id = world.id;
    for event in &mut replayed.event_log {
        event.world_id = world.id;
    }

    let engine = SimulationEngine::civilization();
    for _ in 0..world.tick.0 {
        engine
            .advance_tick(&mut replayed)
            .map_err(|_| StatusCode::CONFLICT)?;
    }

    let event_log_matches = replayed.event_log == world.event_log;
    let mut replayed_snapshot = replayed.clone();
    replayed_snapshot.event_log.clear();
    let mut stored_snapshot = world.clone();
    stored_snapshot.event_log.clear();
    let snapshot_matches = replayed_snapshot == stored_snapshot;

    Ok(Json(ReplayView {
        world_id: world.id.0,
        tick: world.tick.0,
        replayed_ticks: world.tick.0,
        event_count: world.event_log.len(),
        snapshot_matches,
        event_log_matches,
        verified: snapshot_matches && event_log_matches,
    }))
}

fn app(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health))
        .route("/api/v1/world", post(create_world))
        .route("/api/v1/world/{world_id}", get(get_world))
        .route("/api/v1/world/{world_id}/tick", post(advance_tick))
        .route("/api/v1/world/{world_id}/replay", get(verify_replay))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG").unwrap_or_else(|_| "sherlock_api=info,tower_http=info".into()),
        )
        .init();

    let database_url = env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    let state = AppState { pool };
    let address: SocketAddr = env::var("SHERLOCK_BIND")
        .unwrap_or_else(|_| "0.0.0.0:8080".into())
        .parse()?;
    let listener = TcpListener::bind(address).await?;
    tracing::info!(%address, "Sherlock Civilization API listening");
    axum::serve(listener, app(state)).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_state() -> AppState {
        let database_url =
            env::var("DATABASE_URL").expect("DATABASE_URL is set for integration tests");
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("connect test database");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("apply test migrations");
        AppState { pool }
    }

    #[tokio::test]
    async fn health_endpoint_identifies_civilization_service() {
        let response = health().await.0;
        assert_eq!(response.status, "ok");
        assert_eq!(response.service, "sherlock-civilization-api");
    }

    #[tokio::test]
    async fn world_is_created_persisted_and_loaded_with_geography() {
        let state = test_state().await;
        let created = create_world(
            State(state.clone()),
            Json(CreateWorldRequest { seed: Some(77) }),
        )
        .await
        .expect("world creation")
        .0;
        assert_eq!(created.seed, 77);
        assert_eq!(created.tick, 0);
        assert_eq!(created.settlements.len(), 2);
        assert_eq!((created.settlements[0].x, created.settlements[0].y), (2, 2));
        assert_eq!((created.settlements[1].x, created.settlements[1].y), (5, 4));
        assert_eq!(
            created
                .settlements
                .iter()
                .map(|s| s.population)
                .sum::<u64>(),
            200
        );
        assert_eq!(created.event_count, 1);
        assert!(!created.resource_sites.is_empty());
        assert!(
            created
                .resource_sites
                .iter()
                .all(|site| site.stock_units <= site.capacity_units)
        );
        assert_eq!(created.map.width, 8);
        assert_eq!(created.map.height, 6);
        assert_eq!(created.map.cells.len(), 48);
        let river_cell = created
            .map
            .cells
            .iter()
            .find(|cell| cell.x == 2 && cell.y == 2)
            .expect("settlement map cell");
        assert_eq!(river_cell.settlements, vec!["Речной край"]);
        let loaded = get_world(Path(created.world_id), State(state))
            .await
            .expect("load persisted world")
            .0;
        assert_eq!(loaded.world_id, created.world_id);
        assert_eq!(loaded.event_count, created.event_count);
    }

    #[tokio::test]
    async fn tick_advances_and_persists_snapshot_and_events_atomically() {
        let state = test_state().await;
        let created = create_world(
            State(state.clone()),
            Json(CreateWorldRequest { seed: Some(42) }),
        )
        .await
        .expect("world creation")
        .0;
        let response = advance_tick(Path(created.world_id), State(state.clone()))
            .await
            .expect("tick")
            .0;
        assert_eq!(response.world.tick, 1);
        assert!(response.emitted_events >= 5);
        assert_eq!(response.world.settlements[0].food, 910);
        assert_eq!(response.world.settlements[0].timber, 518);
        assert_eq!(response.world.event_count, 1 + response.emitted_events);
        assert!(
            response
                .world
                .resource_sites
                .iter()
                .any(|site| { site.regeneration_units_per_tick > 0 && site.stock_units > 0 })
        );
        let loaded = get_world(Path(created.world_id), State(state))
            .await
            .expect("load persisted world")
            .0;
        assert_eq!(loaded.tick, 1);
        assert_eq!(loaded.event_count, response.world.event_count);
    }

    #[tokio::test]
    async fn replay_reconstructs_snapshot_and_append_only_event_log() {
        let state = test_state().await;
        let created = create_world(
            State(state.clone()),
            Json(CreateWorldRequest { seed: Some(2026) }),
        )
        .await
        .expect("world creation")
        .0;

        for _ in 0..8 {
            let _response = advance_tick(Path(created.world_id), State(state.clone()))
                .await
                .expect("advance tick");
        }

        let replay = verify_replay(Path(created.world_id), State(state))
            .await
            .expect("deterministic replay")
            .0;
        assert_eq!(replay.tick, 8);
        assert_eq!(replay.replayed_ticks, 8);
        assert!(replay.snapshot_matches);
        assert!(replay.event_log_matches);
        assert!(replay.verified);
    }

    #[tokio::test]
    async fn missing_world_returns_not_found() {
        let state = test_state().await;
        assert_eq!(
            get_world(Path(u64::MAX), State(state.clone()))
                .await
                .unwrap_err(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            advance_tick(Path(u64::MAX), State(state))
                .await
                .unwrap_err(),
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn tick_overflow_returns_conflict_without_mutating_snapshot() {
        let state = test_state().await;
        let created = create_world(
            State(state.clone()),
            Json(CreateWorldRequest { seed: Some(43) }),
        )
        .await
        .expect("world creation")
        .0;
        let database_id = database_id(created.world_id).expect("database id");
        let mut world = load_world(&state.pool, database_id)
            .await
            .expect("load world");
        world.tick = sherlock_game_core::Tick(u64::MAX);
        let snapshot = serde_json::to_value(&world).expect("serialize overflow snapshot");
        sqlx::query("UPDATE worlds SET current_snapshot = $2 WHERE id = $1")
            .bind(database_id)
            .bind(snapshot)
            .execute(&state.pool)
            .await
            .expect("store overflow fixture");

        assert_eq!(
            advance_tick(Path(created.world_id), State(state.clone()))
                .await
                .unwrap_err(),
            StatusCode::CONFLICT
        );
        let after = load_world(&state.pool, database_id)
            .await
            .expect("load world after rejected tick");
        assert_eq!(after.tick, sherlock_game_core::Tick(u64::MAX));
        assert_eq!(after.event_log.len(), created.event_count);
    }
}
