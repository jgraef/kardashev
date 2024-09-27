mod components;
mod map;

use std::sync::{
    Arc,
    RwLock,
};

use hecs::World;
use kardashev_client::ApiClient;
use leptos::{
    component,
    view,
    IntoView,
};
use leptos_meta::provide_meta_context;
use leptos_router::{
    Redirect,
    Route,
    Router,
    Routes,
};
use url::Url;

use self::map::Map;
use crate::{
    app::components::dock::Dock,
    graphics::Graphics,
};

stylance::import_crate_style!(style, "src/app/app.module.scss");

struct Urls {
    api_url: Url,
    asset_url: Url,
}

impl Default for Urls {
    fn default() -> Self {
        fn get_base_url() -> Option<Url> {
            gloo_utils::document().base_uri().ok()??.parse().ok()
        }
        let base_url: Url = get_base_url().expect("could not determine base URL");
        let api_url = base_url.join("api").unwrap();
        let asset_url = base_url.join("assets").unwrap();
        tracing::debug!(%api_url, %asset_url);
        Urls { api_url, asset_url }
    }
}

#[derive(Clone)]
pub struct Context {
    pub api_client: ApiClient,
    pub renderer: Graphics,
    pub world: Arc<RwLock<World>>,
}

impl Context {
    fn provide() -> Self {
        let urls = Urls::default();

        let api_client = ApiClient::new(urls.api_url);

        tracing::debug!("creating renderer");
        let renderer = Graphics::new(Default::default());

        tracing::debug!("creating world");
        let world = World::new();

        let context = Self {
            api_client,
            renderer,
            world: Arc::new(RwLock::new(world)),
        };

        leptos::provide_context(context.clone());

        context
    }

    pub fn get() -> Self {
        leptos::expect_context()
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    Context::provide();

    view! {
        <Router>
            <div class=style::app>
                <Dock />
                <main class=style::main>
                    <Routes>
                        <Route path="/" view=|| view!{ <Redirect path="/dashboard"/> } />
                        <Route path="/dashboard" view=|| view!{ "TODO: Dashboard" } />
                        <Route path="/map" view=Map />
                    </Routes>
                </main>
            </div>
        </Router>
    }
}
