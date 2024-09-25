mod components;
mod map;

use std::sync::{
    Arc,
    RwLock,
};

use hecs::World;
use kardashev_client::Client;
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
    renderer::Renderer,
};

stylance::import_crate_style!(style, "src/app/app.module.scss");

fn get_api_url() -> Url {
    fn get_url() -> Option<Url> {
        gloo_utils::document().base_uri().ok()??.parse().ok()
    }
    let url: Url = get_url().expect("could not determine API URL");
    //let url: Url = "http://localhost:3333/".parse().unwrap();
    tracing::debug!(url = %url);
    url
}

#[derive(Clone)]
pub struct Context {
    pub client: Client,
    pub renderer: Renderer,
    pub world: Arc<RwLock<World>>,
}

impl Context {
    fn provide() -> Self {
        let client = Client::new(get_api_url());

        tracing::debug!("creating renderer");
        let renderer = Renderer::new(Default::default());

        tracing::debug!("creating world");
        let world = World::new();

        let context = Self {
            client,
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
