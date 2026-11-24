use axum::{extract::{Path, State}, http::{header, HeaderValue, Method, StatusCode}, routing::{get, post}, Json, Router};
use sherlock_world_kernel::{Good, ProductionRecipe, Settlement, World};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use tower_http::cors::CorsLayer;
use std::{collections::BTreeMap, sync::Arc};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://sherlock:sherlock@127.0.0.1:5432/sherlock".into());
    let pool = PgPoolOptions::new().max_connections(10).connect(&url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/api/worlds", post(create_world))
        .route("/api/worlds/{id}", get(read_world))
        .route("/api/worlds/{id}/advance", post(advance_world))
        .layer(CorsLayer::new().allow_origin(["http://127.0.0.1:5173".parse::<HeaderValue>()?, "http://localhost:5173".parse::<HeaderValue>()?]).allow_methods([Method::GET, Method::POST]).allow_headers([header::CONTENT_TYPE]))
        .with_state(Arc::new(pool));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn create_world(State(db): State<Arc<PgPool>>) -> Result<(StatusCode, Json<World>), ApiError> {
    let id = Uuid::new_v4();
    let settlement = Settlement {
        id: "river-town".into(), name: "River Town".into(), population: 100, treasury: 1000,
        inventory: [(Good::Food, 500), (Good::Timber, 50)].into_iter().collect::<BTreeMap<_,_>>(),
        food_per_person_per_tick: 1,
        production: vec![ProductionRecipe { id: "fishery".into(), output: Good::Food, output_amount: 25, inputs: BTreeMap::new() }],
    };
    let world = World { id, seed: 42, ruleset_version: 1, tick: 0,
        settlements: [(settlement.id.clone(), settlement)].into_iter().collect(),
        events: vec![sherlock_world_kernel::WorldEvent {
            world_id: id, tick: 0, sequence: 0, actor: "scenario_initializer".into(),
            cause: "scenario_start".into(), affected_entities: vec!["river-town".into()],
            provenance: Provenance::GeneratedSimulation, ruleset_version: 1,
            payload: EventPayload::WorldCreated,
        }],
    };
    let mut tx = db.begin().await?;
    sqlx::query("INSERT INTO worlds (id,tick,snapshot) VALUES ($1,0,$2)")
        .bind(id).bind(serde_json::to_value(&world)?).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO world_events (world_id,tick,sequence,event) VALUES ($1,0,0,$2)")
        .bind(id).bind(serde_json::to_value(&world.events[0])?).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(world)))
}

async fn read_world(State(db): State<Arc<PgPool>>, Path(id): Path<Uuid>) -> Result<Json<World>, ApiError> {
    let row = sqlx::query("SELECT snapshot FROM worlds WHERE id=$1").bind(id).fetch_optional(&*db).await?.ok_or(ApiError::NotFound)?;
    Ok(Json(serde_json::from_value(row.try_get("snapshot")?)?))
}

async fn advance_world(State(db): State<Arc<PgPool>>, Path(id): Path<Uuid>) -> Result<Json<World>, ApiError> {
    let mut tx = db.begin().await?;
    let row = sqlx::query("SELECT snapshot FROM worlds WHERE id=$1 FOR UPDATE").bind(id).fetch_optional(&mut *tx).await?.ok_or(ApiError::NotFound)?;
    let mut world: World = serde_json::from_value(row.try_get("snapshot")?)?;
    let old_len = world.events.len();
    world.advance_tick().map_err(|e| ApiError::Simulation(e.to_string()))?;
    sqlx::query("UPDATE worlds SET tick=$2,snapshot=$3,updated_at=now() WHERE id=$1")
        .bind(id).bind(i64::try_from(world.tick).map_err(|_| ApiError::BadRequest)?)
        .bind(serde_json::to_value(&world)?).execute(&mut *tx).await?;
    for event in world.events.iter().skip(old_len) {
        let sequence = i64::try_from(event.sequence).map_err(|_| ApiError::BadRequest)?;
        sqlx::query("INSERT INTO world_events (world_id,tick,sequence,event) VALUES ($1,$2,$3,$4)")
            .bind(id).bind(i64::try_from(event.tick).map_err(|_| ApiError::BadRequest)?)
            .bind(sequence)
            .bind(serde_json::to_value(event)?).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(Json(world))
}

enum ApiError { NotFound, BadRequest, Simulation(String), Db(sqlx::Error), Json(serde_json::Error) }
impl From<sqlx::Error> for ApiError { fn from(e: sqlx::Error) -> Self { Self::Db(e) } }
impl From<serde_json::Error> for ApiError { fn from(e: serde_json::Error) -> Self { Self::Json(e) } }
impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "world not found".into()),
            Self::BadRequest => (StatusCode::BAD_REQUEST, "numeric value out of range".into()),
            Self::Simulation(e) => (StatusCode::UNPROCESSABLE_ENTITY, e),
            Self::Db(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::Json(e) => (StatusCode::BAD_REQUEST, e.to_string()),
        };
        (status, Json(serde_json::json!({"error": message}))).into_response()
    }
}


#[cfg(test)]
mod integration_tests {
    #[tokio::test]
    async fn postgres_migrations_create_world_tables_when_database_is_configured() {
        let Ok(url) = std::env::var("DATABASE_URL") else { return; };
        let pool = sqlx::PgPool::connect(&url).await.expect("connect to CI PostgreSQL");
        sqlx::migrate!("./migrations").run(&pool).await.expect("apply migrations");
        let row = sqlx::query("SELECT to_regclass('public.worlds') IS NOT NULL AS worlds, to_regclass('public.world_events') IS NOT NULL AS events")
            .fetch_one(&pool).await.expect("query migrated schema");
        use sqlx::Row;
        assert!(row.get::<bool, _>("worlds"));
        assert!(row.get::<bool, _>("events"));
    }
}
