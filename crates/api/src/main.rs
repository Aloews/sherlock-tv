use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;
use sherlock_game_core::WorldState;
use std::{env, net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, sync::RwLock};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
struct AppState {
    world: Arc<RwLock<WorldState>>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

#[derive(Serialize)]
struct WorldSummary {
    world_id: u64,
    tick: u64,
    ruleset_version: u32,
    entity_count: usize,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "sherlock-api",
    })
}

async fn world_summary(State(state): State<AppState>) -> Json<WorldSummary> {
    let world = state.world.read().await;
    Json(WorldSummary {
        world_id: world.id.0,
        tick: world.tick.0,
        ruleset_version: world.ruleset_version,
        entity_count: world.entities.len(),
    })
}

fn app(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health))
        .route("/api/v1/world", get(world_summary))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG").unwrap_or_else(|_| "sherlock_api=info,tower_http=info".into()),
        )
        .init();

    let state = AppState {
        world: Arc::new(RwLock::new(WorldState::new(1, 1))),
    };
    let address: SocketAddr = env::var("SHERLOCK_BIND")
        .unwrap_or_else(|_| "0.0.0.0:8080".into())
        .parse()?;
    let listener = TcpListener::bind(address).await?;

    tracing::info!(%address, "Sherlock API listening");
    axum::serve(listener, app(state)).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_endpoint_identifies_service() {
        let response = health().await.0;
        assert_eq!(response.status, "ok");
        assert_eq!(response.service, "sherlock-api");
    }

    #[tokio::test]
    async fn world_summary_reads_real_core_state() {
        let state = AppState {
            world: Arc::new(RwLock::new(WorldState::new(77, 3))),
        };
        let response = world_summary(State(state)).await.0;

        assert_eq!(response.world_id, 77);
        assert_eq!(response.tick, 0);
        assert_eq!(response.ruleset_version, 3);
        assert_eq!(response.entity_count, 0);
    }
}
