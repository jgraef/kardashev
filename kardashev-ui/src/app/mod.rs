mod map;

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
    components::dock::Dock,
    scene::renderer::SceneRenderer,
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
    pub scene_renderer: SceneRenderer,
}

fn provide_context() -> Context {
    let client = Client::new(get_api_url());

    //tracing::debug!("creating scene renderer");
    let scene_renderer = SceneRenderer::new(Default::default());

    let context = Context {
        client,
        scene_renderer,
    };

    leptos::provide_context(context.clone());

    context
}

pub fn expect_context() -> Context {
    leptos::expect_context()
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_context();

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
