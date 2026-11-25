use leptos::prelude::*;
use serde::Deserialize;

const DEFAULT_API_BASE: &str = match option_env!("SHERLOCK_API_URL") {
    Some(value) => value,
    None => "http://localhost:8080",
};

#[derive(Clone, Debug, Deserialize)]
struct WorldView {
    world_id: u64,
    seed: u64,
    tick: u64,
    ruleset_version: u32,
    entity_count: usize,
    event_count: usize,
    settlements: Vec<SettlementView>,
    map: MapView,
    recent_events: Vec<EventView>,
}

#[derive(Clone, Debug, Deserialize)]
struct MapView {
    width: u16,
    height: u16,
    cells: Vec<MapCellView>,
}

#[derive(Clone, Debug, Deserialize)]
struct MapCellView {
    x: u16,
    y: u16,
    terrain: String,
    climate: String,
    elevation_meters: u16,
    soil_fertility: u8,
    water_access: bool,
}

#[derive(Clone, Debug, Deserialize)]
struct EventView {
    tick: u64,
    sequence: u64,
    actor: String,
    cause: String,
    event_type: String,
}

#[derive(Clone, Debug, Deserialize)]
struct SettlementView {
    id: u64,
    name: String,
    population: u64,
    food: u64,
    timber: u64,
    treasury: u64,
}

#[component]
fn WorldDashboard() -> impl IntoView {
    let (world, set_world) = signal(None::<WorldView>);
    let (status, set_status) = signal("Подключите API и создайте новый мир.".to_string());
    let (api_base, set_api_base) = signal(DEFAULT_API_BASE.trim_end_matches('/').to_string());

    let create_world = move |_| {
        let base = api_base.get_untracked();
        set_status.set("Создаём мир…".into());
        leptos::task::spawn_local(async move {
            let result = async {
                let response = gloo_net::http::Request::post(&format!("{base}/api/v1/world"))
                    .header("Content-Type", "application/json")
                    .body(r#"{"seed":42}"#)
                    .map_err(|error| error.to_string())?
                    .send()
                    .await
                    .map_err(|error| error.to_string())?;
                if !response.ok() {
                    return Err(format!("API вернул HTTP {}", response.status()));
                }
                response
                    .json::<WorldView>()
                    .await
                    .map_err(|error| error.to_string())
            }
            .await;
            match result {
                Ok(view) => {
                    set_status.set("Мир создан. Симуляция готова к первому ходу.".into());
                    set_world.set(Some(view));
                }
                Err(error) => set_status.set(format!("Не удалось создать мир: {error}")),
            }
        });
    };

    let advance_tick = move |_| {
        let base = api_base.get_untracked();
        set_status.set("Выполняется ход…".into());
        leptos::task::spawn_local(async move {
            let result = async {
                let response = gloo_net::http::Request::post(&format!("{base}/api/v1/world/tick"))
                    .send()
                    .await
                    .map_err(|error| error.to_string())?;
                if !response.ok() {
                    return Err(format!("API вернул HTTP {}", response.status()));
                }
                response
                    .json::<TickResponse>()
                    .await
                    .map_err(|error| error.to_string())
            }
            .await;
            match result {
                Ok(response) => {
                    set_status.set(format!(
                        "Ход {} завершён. Записано событий: {}.",
                        response.world.tick, response.emitted_events
                    ));
                    set_world.set(Some(response.world));
                }
                Err(error) => set_status.set(format!("Не удалось выполнить ход: {error}")),
            }
        });
    };

    view! {
        <section class="world-overview">
            <p class="eyebrow">"SHERLOCK / WORLD SIMULATION"</p>
            <h1>"Живой мир. Собственная история."</h1>
            <p class="lead">"Цивилизация развивается как единая система: природа, люди, производство, обмен, финансы и институты. Каждый ход меняет состояние мира и оставляет проверяемый след событий."</p>

            <section class="control-panel">
                <label for="api-base">"Адрес игрового API"</label>
                <input id="api-base" type="url" prop:value=move || api_base.get()
                    on:input=move |event| set_api_base.set(event_target_value(&event)) />
                <div class="actions">
                    <button class="primary-button" on:click=create_world>"Создать мир"</button>
                    <button class="secondary-button" on:click=advance_tick disabled=move || world.get().is_none()>"Следующий ход"</button>
                </div>
                <p class="status-message" aria-live="polite">{move || status.get()}</p>
            </section>

            <Show when=move || world.get().is_some()>
                {move || world.get().map(|current| view! {
                    <section class="world-panel">
                        <div class="world-heading">
                            <div><p class="eyebrow">"ТЕКУЩЕЕ СОСТОЯНИЕ"</p><h2>{format!("Мир #{}", current.world_id)}</h2></div>
                            <div class="tick-counter"><span>"ХОД"</span><strong>{current.tick}</strong></div>
                        </div>
                        <div class="metrics-grid">
                            <div><span>"НАСЕЛЕНИЕ"</span><strong>{current.settlements.iter().map(|s| s.population).sum::<u64>()}</strong></div>
                            <div><span>"ПОСЕЛЕНИЯ"</span><strong>{current.settlements.len()}</strong></div>
                            <div><span>"СОБЫТИЯ"</span><strong>{current.event_count}</strong></div>
                            <div><span>"SEED / RULESET"</span><strong>{format!("{} / {}", current.seed, current.ruleset_version)}</strong></div>
                        </div>
                        <section class="map-panel">
                            <div class="section-heading">
                                <div><p class="eyebrow">"ГЕОГРАФИЯ"</p><h3>"Рельеф и природные условия"</h3></div>
                                <span>{format!("{} × {}", current.map.width, current.map.height)}</span>
                            </div>
                            <div class="world-map" style:grid-template-columns=move || format!("repeat({}, minmax(0, 1fr))", current.map.width)>
                                {current.map.cells.into_iter().map(|cell| {
                                    let class_name = format!("map-cell terrain-{}", cell.terrain);
                                    let title = format!(
                                        "Координаты: {}, {} · климат: {} · высота: {} м · плодородие: {} · вода: {}",
                                        cell.x,
                                        cell.y,
                                        cell.climate,
                                        cell.elevation_meters,
                                        cell.soil_fertility,
                                        if cell.water_access { "есть" } else { "нет" }
                                    );
                                    view! {
                                        <div class=class_name title=title>
                                            <span>{match cell.terrain.as_str() {
                                                "river" => "Р",
                                                "forest" => "Л",
                                                "hills" => "Х",
                                                "mountain" => "Г",
                                                _ => "П",
                                            }}</span>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                            <div class="map-legend">
                                <span>"П — равнины"</span><span>"Л — лес"</span><span>"Х — холмы"</span><span>"Г — горы"</span><span>"Р — река"</span>
                            </div>
                        </section>
                        <div class="settlement-grid">
                            {current.settlements.into_iter().map(|settlement| view! {
                                <article class="settlement-card">
                                    <p class="eyebrow">{format!("ПОСЕЛЕНИЕ #{}", settlement.id)}</p>
                                    <h3>{settlement.name}</h3>
                                    <div class="settlement-stat"><span>"Население"</span><strong>{settlement.population}</strong></div>
                                    <div class="settlement-stat"><span>"Запасы пищи"</span><strong>{settlement.food}</strong></div>
                                    <div class="settlement-stat"><span>"Древесина"</span><strong>{settlement.timber}</strong></div>
                                    <div class="settlement-stat"><span>"Казна"</span><strong>{settlement.treasury}</strong></div>
                                </article>
                            }).collect_view()}
                        </div>
                        <section class="event-history">
                            <div class="section-heading">
                                <div><p class="eyebrow">"ХРОНИКА"</p><h3>"Последние события"</h3></div>
                                <span>{format!("{} записей", current.event_count)}</span>
                            </div>
                            <div class="event-list">
                                {current.recent_events.into_iter().map(|event| view! {
                                    <article class="event-row">
                                        <span class="event-tick">{format!("ХОД {}", event.tick)}</span>
                                        <div><strong>{event.event_type}</strong><p>{format!("{} · {}", event.actor, event.cause)}</p></div>
                                        <span class="event-sequence">{format!("#{}", event.sequence)}</span>
                                    </article>
                                }).collect_view()}
                            </div>
                        </section>
                        <p class="entity-note">{format!("Всего сущностей в мире: {}", current.entity_count)}</p>
                    </section>
                })}
            </Show>

            <div class="system-grid">
                <article><span>"01"</span><h2>"Планета"</h2><p>"География, климат и экология"</p></article>
                <article><span>"02"</span><h2>"Население"</h2><p>"Люди, семьи и поселения"</p></article>
                <article><span>"03"</span><h2>"Экономика"</h2><p>"Труд, производство и обмен"</p></article>
                <article><span>"04"</span><h2>"Институты"</h2><p>"Организации, право и власть"</p></article>
                <article><span>"05"</span><h2>"Династии"</h2><p>"Родственные связи и наследование"</p></article>
                <article><span>"06"</span><h2>"Культура"</h2><p>"Знания, нормы и историческая память"</p></article>
            </div>
        </section>
    }
}

#[derive(Clone, Debug, Deserialize)]
struct TickResponse {
    world: WorldView,
    emitted_events: usize,
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <main class="app-shell">
            <header class="topbar">
                <a class="brand" href="#/">"SHERLOCK"</a>
                <span class="product-label">"CIVILIZATION SIMULATION"</span>
            </header>
            <WorldDashboard/>
            <footer>"SIMULATION FOUNDATION / RUST"</footer>
        </main>
    }
}

#[cfg(target_arch = "wasm32")]
pub fn mount_app() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn mount_app() {}
