use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Form, Json, Router,
};
use maud::{html, Markup, DOCTYPE};
use serde::Deserialize;
use sherlock_world_kernel::{
    EventPayload, ExtractionCommand, Good, ProductionRecipe, Provenance, ResourceSite,
    Settlement, SimulationError, TradeCommand, World, WorldEvent,
};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::{collections::BTreeMap, net::SocketAddr, sync::Arc};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

type AppState = Arc<PgPool>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()))
        .init();
    let database_url = std::env::var("DATABASE_URL")?;
    let bind_addr: SocketAddr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8080".into()).parse()?;
    let pool = PgPoolOptions::new().max_connections(10).connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    let app = Router::new()
        .route("/", get(home))
        .route("/worlds", post(create_world))
        .route("/worlds/{id}", get(show_world))
        .route("/worlds/{id}/tick", post(advance_world))
        .route("/worlds/{id}/extract", post(extract_resource))
        .route("/worlds/{id}/trade", post(execute_trade))
        .route("/health", get(|| async { "ok" }))
        .with_state(Arc::new(pool))
        .layer(TraceLayer::new_for_http());
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    tracing::info!(%bind_addr, "Sherlock server listening");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn home() -> Html<String> {
    Html(layout(html! {
        section class="hero" {
            p class="eyebrow" { "HISTORICAL WORLD SIMULATION" }
            h1 { "SHERLOCK" }
            h2 { "История не задана. Она возникает." }
            p { "Цивилизация как единая система: земля, ресурсы, население, производство, обмен и институты." }
            form method="post" action="/worlds" {
                button type="submit" { "Создать новый мир" }
            }
        }
        section class="principles" {
            article { h3 { "Причины, а не сценарий" } p { "События возникают из взаимодействия систем и оставляют проверяемый след." } }
            article { h3 { "История и модель разделены" } p { "Источники, интерпретации, допущения и результаты симуляции имеют разное происхождение." } }
            article { h3 { "Единый мир" } p { "Экономика, политика, культура и технологии — взаимосвязанные части одной модели." } }
        }
    }))
}

async fn create_world(State(db): State<AppState>) -> Result<Redirect, AppError> {
    let id = Uuid::new_v4();
    let river = Settlement {
        id: "river-town".into(), name: "Речной город".into(), population: 100,
        treasury: 1_000, food_per_person_per_tick: 1,
        inventory: [(Good::Food, 500), (Good::Timber, 40)].into_iter().collect(),
        production: vec![ProductionRecipe {
            id: "fishery".into(), inputs: BTreeMap::new(),
            output: Good::Food, output_amount: 25,
        }],
    };
    let hill = Settlement {
        id: "hill-village".into(), name: "Горное поселение".into(), population: 40,
        treasury: 100, food_per_person_per_tick: 1,
        inventory: [(Good::Food, 250), (Good::Stone, 10), (Good::Timber, 15)].into_iter().collect(),
        production: vec![],
    };
    let quarry = ResourceSite {
        id: "stone-quarry".into(), name: "Каменный карьер".into(),
        stocks: [(Good::Stone, 1_000)].into_iter().collect(),
    };
    let mut world = World {
        id, seed: 42, ruleset_version: 1, tick: 0,
        settlements: [(river.id.clone(), river), (hill.id.clone(), hill)].into_iter().collect(),
        resource_sites: [(quarry.id.clone(), quarry)].into_iter().collect(),
        events: vec![],
    };
    world.events.push(WorldEvent {
        world_id: id, tick: 0, sequence: 0, actor: "scenario_initializer".into(),
        cause: "world_creation".into(),
        affected_entities: vec!["river-town".into(), "hill-village".into(), "stone-quarry".into()],
        provenance: Provenance::ModelAssumption { assumption_id: "starter-scenario-v1".into() },
        ruleset_version: 1, payload: EventPayload::WorldCreated,
    });
    let mut tx = db.begin().await?;
    sqlx::query("INSERT INTO worlds (id,seed,ruleset_version,tick,snapshot) VALUES ($1,$2,$3,0,$4)")
        .bind(id).bind(42_i64).bind(1_i32).bind(serde_json::to_value(&world)?)
        .execute(&mut *tx).await?;
    persist_events(&mut tx, &world.events).await?;
    tx.commit().await?;
    Ok(Redirect::to(&format!("/worlds/{id}")))
}

async fn show_world(
    State(db): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Html<String>, AppError> {
    let world = load_world(&db, id).await?;
    Ok(Html(layout(world_page(&world))))
}

async fn advance_world(
    State(db): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Redirect, AppError> {
    mutate_world(&db, id, |world| world.advance_tick()).await?;
    Ok(Redirect::to(&format!("/worlds/{id}")))
}

#[derive(Deserialize)]
struct ExtractionForm { site_id: String, settlement_id: String, good: Good, quantity: u64 }

async fn extract_resource(
    State(db): State<AppState>,
    Path(id): Path<Uuid>,
    Form(form): Form<ExtractionForm>,
) -> Result<Redirect, AppError> {
    mutate_world(&db, id, move |world| world.extract_resource(ExtractionCommand {
        site_id: form.site_id, settlement_id: form.settlement_id,
        good: form.good, quantity: form.quantity, actor: "player".into(),
    })).await?;
    Ok(Redirect::to(&format!("/worlds/{id}")))
}

#[derive(Deserialize)]
struct TradeForm {
    seller_id: String, buyer_id: String, good: Good, quantity: u64, unit_price: i64,
}

async fn execute_trade(
    State(db): State<AppState>,
    Path(id): Path<Uuid>,
    Form(form): Form<TradeForm>,
) -> Result<Redirect, AppError> {
    mutate_world(&db, id, move |world| world.execute_trade(TradeCommand {
        seller_id: form.seller_id, buyer_id: form.buyer_id, good: form.good,
        quantity: form.quantity, unit_price: form.unit_price, actor: "player".into(),
    })).await?;
    Ok(Redirect::to(&format!("/worlds/{id}")))
}

async fn load_world(db: &PgPool, id: Uuid) -> Result<World, AppError> {
    let row = sqlx::query("SELECT snapshot FROM worlds WHERE id = $1")
        .bind(id).fetch_optional(db).await?.ok_or(AppError::NotFound)?;
    Ok(serde_json::from_value(row.try_get("snapshot")?)?)
}

async fn mutate_world<F>(db: &PgPool, id: Uuid, command: F) -> Result<(), AppError>
where
    F: FnOnce(&mut World) -> Result<(), SimulationError>,
{
    let mut tx = db.begin().await?;
    let row = sqlx::query("SELECT snapshot FROM worlds WHERE id = $1 FOR UPDATE")
        .bind(id).fetch_optional(&mut *tx).await?.ok_or(AppError::NotFound)?;
    let mut world: World = serde_json::from_value(row.try_get("snapshot")?)?;
    let previous_event_count = world.events.len();
    command(&mut world).map_err(|error| AppError::Simulation(error.to_string()))?;
    sqlx::query("UPDATE worlds SET tick=$2, snapshot=$3, updated_at=now() WHERE id=$1")
        .bind(id).bind(i64::try_from(world.tick).map_err(|_| AppError::BadRequest)?)
        .bind(serde_json::to_value(&world)?).execute(&mut *tx).await?;
    persist_events(&mut tx, &world.events[previous_event_count..]).await?;
    tx.commit().await?;
    Ok(())
}

async fn persist_events(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    events: &[WorldEvent],
) -> Result<(), AppError> {
    for event in events {
        sqlx::query("INSERT INTO world_events (world_id,sequence,tick,event) VALUES ($1,$2,$3,$4)")
            .bind(event.world_id)
            .bind(i64::try_from(event.sequence).map_err(|_| AppError::BadRequest)?)
            .bind(i64::try_from(event.tick).map_err(|_| AppError::BadRequest)?)
            .bind(serde_json::to_value(event)?)
            .execute(&mut **tx).await?;
    }
    Ok(())
}

fn world_page(world: &World) -> Markup {
    html! {
        section class="world-head" {
            a href="/" class="back" { "← Sherlock" }
            p class="eyebrow" { "СИМУЛЯЦИЯ ЦИВИЛИЗАЦИИ" }
            h1 { "Мир " (world.id.to_string().chars().take(8).collect::<String>()) }
            div class="world-meta" { span { "ТАКТ " (world.tick) } span { "SEED " (world.seed) } span { "ПРАВИЛА " (world.ruleset_version) } }
            form method="post" action=(format!("/worlds/{}/tick", world.id)) {
                button type="submit" { "Продвинуть мир на один такт" }
            }
        }
        section class="settlements" {
            @for settlement in world.settlements.values() {
                article class="settlement" {
                    p class="eyebrow" { "ПОСЕЛЕНИЕ" }
                    h2 { (&settlement.name) }
                    p class="population" { (settlement.population) " жителей" }
                    p { "Казна: " (settlement.treasury) }
                    h3 { "Запасы" }
                    div class="goods" {
                        @for (good, amount) in &settlement.inventory {
                            span { (good_label(*good)) " · " (amount) }
                        }
                    }
                    h3 { "Производство" }
                    @if settlement.production.is_empty() { p class="muted" { "Производственных цепочек пока нет." } }
                    @for recipe in &settlement.production {
                        p class="muted" { (recipe.id) " → " (good_label(recipe.output)) " " (recipe.output_amount) }
                    }
                }
            }
        }
        section class="operations" {
            article {
                h2 { "Добыча ресурсов" }
                form method="post" action=(format!("/worlds/{}/extract", world.id)) {
                    label { "Место добычи" select name="site_id" {
                        @for site in world.resource_sites.values() { option value=(site.id) { (&site.name) } }
                    }}
                    label { "Поселение" select name="settlement_id" {
                        @for settlement in world.settlements.values() { option value=(settlement.id) { (&settlement.name) } }
                    }}
                    label { "Ресурс" select name="good" {
                        option value="stone" { "Камень" }
                        option value="iron" { "Железо" }
                        option value="timber" { "Дерево" }
                        option value="food" { "Еда" }
                    }}
                    label { "Количество" input type="number" name="quantity" min="1" value="10" required }
                    button type="submit" { "Добыть" }
                }
            }
            article {
                h2 { "Торговля между поселениями" }
                form method="post" action=(format!("/worlds/{}/trade", world.id)) {
                    label { "Продавец" select name="seller_id" {
                        @for settlement in world.settlements.values() { option value=(settlement.id) { (&settlement.name) } }
                    }}
                    label { "Покупатель" select name="buyer_id" {
                        @for settlement in world.settlements.values() { option value=(settlement.id) { (&settlement.name) } }
                    }}
                    label { "Товар" select name="good" {
                        option value="stone" { "Камень" }
                        option value="iron" { "Железо" }
                        option value="timber" { "Дерево" }
                        option value="food" { "Еда" }
                    }}
                    label { "Количество" input type="number" name="quantity" min="1" value="1" required }
                    label { "Цена за единицу" input type="number" name="unit_price" min="1" value="10" required }
                    button type="submit" { "Заключить сделку" }
                }
            }
        }
        section class="resources" {
            h2 { "Места и ресурсы" }
            @for site in world.resource_sites.values() {
                article { h3 { (&site.name) } @for (good, amount) in &site.stocks { p { (good_label(*good)) " · " (amount) } } }
            }
        }
        section class="chronicle" {
            h2 { "Летопись событий" }
            @for event in world.events.iter().rev().take(40) {
                article class="event" {
                    span class="event-tick" { "ТАКТ " (event.tick) " / #" (event.sequence) }
                    strong { (event.cause) }
                    span { (event_summary(&event.payload)) }
                    small { "актор: " (&event.actor) " · происхождение: " (provenance_label(&event.provenance)) }
                }
            }
        }
    }
}

fn good_label(good: Good) -> &'static str {
    match good { Good::Food => "Еда", Good::Timber => "Дерево", Good::Stone => "Камень", Good::Iron => "Железо" }
}

fn event_summary(payload: &EventPayload) -> String {
    match payload {
        EventPayload::WorldCreated => "Создан мир".into(),
        EventPayload::TickAdvanced => "Завершён такт симуляции".into(),
        EventPayload::GoodsProduced { settlement_id, good, amount, .. } => format!("{settlement_id}: произведено {amount} — {}", good_label(*good)),
        EventPayload::FoodConsumed { settlement_id, amount } => format!("{settlement_id}: потреблено еды {amount}"),
        EventPayload::ResourceExtracted { site_id, settlement_id, good, amount } => format!("{settlement_id} добыл {amount} — {} из {site_id}", good_label(*good)),
        EventPayload::TradeExecuted { seller_id, buyer_id, good, quantity, total_price } => format!("{seller_id} → {buyer_id}: {quantity} — {} за {total_price}", good_label(*good)),
    }
}

fn provenance_label(provenance: &Provenance) -> &'static str {
    match provenance {
        Provenance::GeneratedSimulation => "результат симуляции",
        Provenance::HistoricalSource { .. } => "исторический источник",
        Provenance::ScholarlyInterpretation { .. } => "научная интерпретация",
        Provenance::ModelAssumption { .. } => "допущение модели",
        Provenance::DerivedEstimate { .. } => "расчётная оценка",
    }
}

fn layout(content: Markup) -> String {
    html! {
        (DOCTYPE)
        html lang="ru" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Sherlock — Historical World Simulation" }
                style {
                    (r#"
                    :root{color-scheme:dark;font-family:Inter,ui-sans-serif,system-ui,sans-serif;background:#0b1017;color:#e8edf4}
                    *{box-sizing:border-box}body{margin:0;background:radial-gradient(ellipse at 75% -20%,#26364b 0,transparent 45%),#0b1017}
                    main{max-width:1120px;margin:auto;padding:38px 28px 80px}h1,h2,h3,p{margin-top:0}
                    h1{font-size:clamp(42px,8vw,82px);letter-spacing:-.07em;margin:8px 0 18px}
                    h2{font-size:25px;letter-spacing:-.03em}.eyebrow{font-size:10px;letter-spacing:.2em;color:#c4a878;font-weight:700}
                    .hero{padding:9vh 0 8vh;max-width:760px}.hero h2{font-size:clamp(24px,4vw,40px);font-weight:500}
                    .hero p{color:#a7b3c2;line-height:1.8;max-width:620px}
                    button{background:#c9a875;color:#151a20;border:0;padding:13px 20px;font-weight:750;cursor:pointer;border-radius:3px}
                    button:hover{background:#e2c18e}a{color:#d7b783;text-decoration:none}.principles,.settlements,.operations{display:grid;grid-template-columns:repeat(auto-fit,minmax(240px,1fr));gap:14px}
                    .principles article,.settlement,.operations article,.resources article{background:#ffffff08;border:1px solid #ffffff18;padding:22px;border-radius:5px}
                    .principles p,.muted{color:#98a5b5;line-height:1.6}.world-head{padding:26px 0 34px}.world-head h1{font-size:48px}
                    .world-meta{display:flex;gap:22px;color:#99a7b8;font-size:11px;letter-spacing:.12em;margin:20px 0}
                    .settlements{margin:18px 0 28px}.population{font-size:24px;color:#e0c18f}.goods{display:flex;flex-wrap:wrap;gap:7px}
                    .goods span{background:#263344;padding:6px 9px;border-radius:3px;font-size:12px}
                    .operations{margin:18px 0 30px}.operations form{display:grid;gap:12px}
                    label{display:grid;gap:6px;font-size:12px;color:#aab6c5}
                    input,select{background:#111b27;color:#e8edf4;border:1px solid #394657;padding:10px;border-radius:3px;width:100%}
                    .resources{margin:30px 0}.resources article{display:inline-block;margin:0 10px 10px 0;min-width:180px}
                    .chronicle{margin-top:36px}.event{display:grid;grid-template-columns:110px 1fr 2fr;gap:14px;padding:13px 0;border-bottom:1px solid #ffffff15;align-items:center}
                    .event small{grid-column:2/-1;color:#8492a4}.event-tick{font-size:10px;color:#c9aa79}
                    .back{display:inline-block;margin-bottom:26px}footer{margin-top:70px;color:#647286;font-size:10px;letter-spacing:.16em}
                    @media(max-width:650px){main{padding:24px 16px}.event{grid-template-columns:1fr;gap:5px}.event small{grid-column:auto}.world-meta{flex-wrap:wrap}}
                    "#)
                }
            }
            body { main { (content) footer { "SHERLOCK · ДЕТЕРМИНИРОВАННЫЕ ПРАВИЛА · СОБЫТИЯ С СОХРАНЁННЫМ ПРОИСХОЖДЕНИЕМ" } } }
        }
    }.into_string()
}

enum AppError { NotFound, BadRequest, Simulation(String), Database(sqlx::Error), Serialization(serde_json::Error) }
impl From<sqlx::Error> for AppError { fn from(value: sqlx::Error) -> Self { Self::Database(value) } }
impl From<serde_json::Error> for AppError { fn from(value: serde_json::Error) -> Self { Self::Serialization(value) } }
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "Мир не найден".into()),
            Self::BadRequest => (StatusCode::BAD_REQUEST, "Числовое значение вне допустимого диапазона".into()),
            Self::Simulation(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
            Self::Database(error) => {
                tracing::error!(%error, "database operation failed");
                (StatusCode::INTERNAL_SERVER_ERROR, "Ошибка хранилища".into())
            }
            Self::Serialization(error) => {
                tracing::error!(%error, "snapshot serialization failed");
                (StatusCode::INTERNAL_SERVER_ERROR, "Ошибка сериализации состояния".into())
            }
        };
        (status, Html(layout(html! { section class="hero" { h1 { "Ошибка" } p { (message) } a href="/" { "На главную" } }))),).into_response()
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn postgres_migrations_create_event_store() {
        let Ok(url) = std::env::var("DATABASE_URL") else { return; };
        let pool = sqlx::PgPool::connect(&url).await.expect("connect to PostgreSQL service");
        sqlx::migrate!("./migrations").run(&pool).await.expect("apply schema");
        let row = sqlx::query("SELECT to_regclass('public.worlds') IS NOT NULL AS worlds, to_regclass('public.world_events') IS NOT NULL AS events")
            .fetch_one(&pool).await.expect("inspect migrated schema");
        use sqlx::Row;
        assert!(row.get::<bool, _>("worlds"));
        assert!(row.get::<bool, _>("events"));
    }
}
