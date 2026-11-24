use leptos::prelude::*;

#[component]
fn WorldOverview() -> impl IntoView {
    view! {
        <section class="world-overview">
            <p class="eyebrow">"SHERLOCK / WORLD SIMULATION"</p>
            <h1>"Живой мир. Собственная история."</h1>
            <p class="lead">
                "Цивилизация развивается как единая система: природа, люди, знания, производство, обмен, финансы, институты и культура."
            </p>
            <div class="world-status">
                <span class="status-indicator"></span>
                <div>
                    <strong>"Новый мир ещё не создан"</strong>
                    <p>"Здесь появятся карта мира, ход времени, события и инструменты наблюдения."</p>
                </div>
            </div>
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

#[component]
pub fn App() -> impl IntoView {
    view! {
        <main class="app-shell">
            <header class="topbar">
                <a class="brand" href="#/">"SHERLOCK"</a>
                <span class="product-label">"CIVILIZATION SIMULATION"</span>
            </header>
            <WorldOverview/>
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
