use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sherlock_game_core::{EntityKind, SimulationEngine, WorldState};
use std::{env, net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, sync::RwLock};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[derive(Clone)]
struct AppState {
    world: Arc<RwLock<Option<WorldState>>>,
}

#[derive(Serialize)]
struct HealthResponse { status: &'static str, service: &'static str }

#[derive(Deserialize)]
struct CreateWorldRequest { seed: Option<u64> }

#[derive(Serialize)]
struct SettlementView {
    id: u64,
    name: String,
    population: u64,
    food: u64,
    timber: u64,
    treasury: u64,
}

#[derive(Serialize)]
struct WorldView {
    world_id: u64,
    seed: u64,
    tick: u64,
    ruleset_version: u32,
    entity_count: usize,
    event_count: u64,
    settlements: Vec<SettlementView>,
}

#[derive(Debug, Serialize)]
struct TickResponse {
    world: WorldView,
    emitted_events: usize,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok", service: "sherlock-civilization-api" })
}

fn world_view(world: &WorldState) -> WorldView {
    let settlements = world.entities.values()
        .filter(|entity| entity.kind == EntityKind::Settlement)
        .map(|entity| {
            let number = |key: &str| entity.attributes.get(key).and_then(serde_json::Value::as_u64).unwrap_or(0);
            SettlementView {
                id: entity.id.0,
                name: entity.name.clone(),
                population: number("population"),
                food: number("food"),
                timber: number("timber"),
                treasury: number("treasury"),
            }
        }).collect();
    WorldView {
        world_id: world.id.0,
        seed: world.seed,
        tick: world.tick.0,
        ruleset_version: world.ruleset_version,
        entity_count: world.entities.len(),
        event_count: world.next_event_sequence,
        settlements,
    }
}

async fn get_world(State(state): State<AppState>) -> Result<Json<WorldView>, StatusCode> {
    let guard = state.world.read().await;
    guard.as_ref().map(|world| Json(world_view(world))).ok_or(StatusCode::NOT_FOUND)
}

async fn create_world(
    State(state): State<AppState>,
    Json(request): Json<CreateWorldRequest>,
) -> Json<WorldView> {
    let mut guard = state.world.write().await;
    let world = WorldState::starter(request.seed.unwrap_or(42));
    let view = world_view(&world);
    *guard = Some(world);
    Json(view)
}

async fn advance_tick(State(state): State<AppState>) -> Result<Json<TickResponse>, StatusCode> {
    let mut guard = state.world.write().await;
    let world = guard.as_mut().ok_or(StatusCode::NOT_FOUND)?;
    let events = SimulationEngine::civilization()
        .advance_tick(world)
        .map_err(|_| StatusCode::CONFLICT)?;
    Ok(Json(TickResponse { world: world_view(world), emitted_events: events.len() }))
}

fn app(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health))
        .route("/api/v1/world", get(get_world).post(create_world))
        .route("/api/v1/world/tick", post(advance_tick))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(env::var("RUST_LOG").unwrap_or_else(|_| "sherlock_api=info,tower_http=info".into()))
        .init();
    let state = AppState { world: Arc::new(RwLock::new(None)) };
    let address: SocketAddr = env::var("SHERLOCK_BIND")
        .unwrap_or_else(|_| "0.0.0.0:8080".into()).parse()?;
    let listener = TcpListener::bind(address).await?;
    tracing::info!(%address, "Sherlock Civilization API listening");
    axum::serve(listener, app(state)).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> AppState { AppState { world: Arc::new(RwLock::new(None)) } }

    #[tokio::test]
    async fn health_endpoint_identifies_civilization_service() {
        let response = health().await.0;
        assert_eq!(response.status, "ok");
        assert_eq!(response.service, "sherlock-civilization-api");
    }

    #[tokio::test]
    async fn world_is_created_with_two_settlements_and_requested_seed() {
        let state = state();
        let view = create_world(State(state.clone()), Json(CreateWorldRequest { seed: Some(77) })).await.0;
        assert_eq!(view.seed, 77);
        assert_eq!(view.tick, 0);
        assert_eq!(view.settlements.len(), 2);
        assert_eq!(view.settlements.iter().map(|s| s.population).sum::<u64>(), 200);
        assert_eq!(get_world(State(state)).await.expect("world exists").0.seed, 77);
    }

    #[tokio::test]
    async fn tick_advances_economy_and_returns_events() {
        let state = state();
        create_world(State(state.clone()), Json(CreateWorldRequest { seed: Some(42) })).await;
        let response = advance_tick(State(state)).await.expect("tick").0;
        assert_eq!(response.world.tick, 1);
        assert!(response.emitted_events >= 5);
        assert_eq!(response.world.settlements[0].food, 910);
        assert_eq!(response.world.settlements[0].timber, 518);
    }

    #[tokio::test]
    async fn missing_world_returns_not_found() {
        assert_eq!(get_world(State(state())).await.unwrap_err(), StatusCode::NOT_FOUND);
        assert_eq!(advance_tick(State(state())).await.unwrap_err(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn tick_overflow_returns_conflict_without_mutating_world() {
        let state = state();
        create_world(State(state.clone()), Json(CreateWorldRequest { seed: Some(42) })).await;
        state.world.write().await.as_mut().expect("world").tick = sherlock_game_core::Tick(u64::MAX);
        assert_eq!(advance_tick(State(state.clone())).await.unwrap_err(), StatusCode::CONFLICT);
        assert_eq!(state.world.read().await.as_ref().expect("world").tick.0, u64::MAX);
    }
}
