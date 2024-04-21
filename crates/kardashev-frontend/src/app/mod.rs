mod main;

use std::sync::Arc;

use kardashev_client::Client;
use leptos::{
    component,
    view,
    DynAttrs,
    IntoView,
    Oco,
};
use leptos_meta::{
    provide_meta_context,
    Html,
};
use leptos_router::{
    Route,
    Router,
    Routes,
};
use url::Url;

use self::main::MainPage;
use crate::scene::renderer::SceneRenderer;

#[component]
pub fn BootstrapIcon(#[prop(into)] icon: Oco<'static, str>) -> impl IntoView {
    view! { <i class={format!("bi bi-{icon}")}></i> }
}

fn get_api_url() -> Url {
    //let url: Url = gloo_utils::document().base_uri().ok()??.parse().ok()?;
    let url: Url = "http://localhost:3333/".parse().unwrap();
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
    let scene_renderer = SceneRenderer::new();

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
        <Html
            attr:data-bs-theme="dark"
        />
        <Router>
            <div class="d-flex flex-column" style="width: 100vw; height: 100vh;">
                <main class="main d-flex flex-column w-100 h-100 mw-100 mh-100">
                    <Routes>
                        <Route path="/" view=MainPage />
                    </Routes>
                </main>
            </div>
        </Router>
    }
}
