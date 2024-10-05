mod components;
mod map;

use components::window::provide_graphics;
use kardashev_client::ApiClient;
use kardashev_protocol::asset_id;
use leptos::{
    component,
    expect_context,
    provide_context,
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
use nalgebra::Similarity3;
use url::Url;

use self::map::Map;
use crate::{
    app::components::dock::Dock,
    assets::{
        AssetsPlugin,
        Load,
    },
    error::Error,
    graphics::{
        material::Material,
        mesh::{
            shape,
            Mesh,
            MeshBuilder,
            Meshable,
        },
        transform::Transform,
        RenderPlugin,
    },
    input::InputPlugin,
    world::{
        OneshotSystem,
        RunSystemContext,
        World,
    },
};

stylance::import_crate_style!(style, "src/app/app.module.scss");

#[derive(Clone, Debug)]
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

/// Main app component
#[component]
pub fn App() -> impl IntoView {
    let urls = Urls::default();
    tracing::info!(?urls, "endpoints");

    provide_meta_context();
    provide_client(urls.api_url);
    provide_graphics();
    provide_world(urls.asset_url);

    /*let (log_level, _, _) = use_local_storage::<Option<tracing::Level>, OptionCodec<FromToStringCodec>>("log-level");
    create_effect(move |_| {
        let log_level = log_level.get().unwrap_or(Level::DEBUG);
        tracing::info!("setting log level to {log_level:?}");
        tracing_wasm::set_as_global_default_with_config(WASMLayerConfigBuilder::new().set_max_level(log_level).build());
    });*/

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

fn provide_client(api_url: Url) {
    let api_client = ApiClient::new(api_url);
    provide_context(api_client);
}

fn provide_world(asset_url: Url) {
    let api_client = expect_context::<ApiClient>();

    tracing::debug!("creating world");
    let world = World::builder()
        .with_resource(api_client)
        .with_plugin(AssetsPlugin::from_url(asset_url))
        .with_plugin(InputPlugin::default())
        .with_plugin(RenderPlugin)
        .with_startup_system({
            struct StartupSystem;

            impl OneshotSystem for StartupSystem {
                fn label(&self) -> &'static str {
                    "startup"
                }

                async fn run<'c: 'd, 'd>(
                    self,
                    context: &'d mut RunSystemContext<'c>,
                ) -> Result<(), Error> {
                    //let api_client = context.resources.get::<ApiClient>().unwrap();
                    // todo: we don't want to wait here for a reply, but should spawn a oneshot
                    // system when the request finishes
                    //let stars = api_client.get_stars().await?;

                    context.world.spawn((
                        Transform {
                            model_matrix: Similarity3::identity(),
                        },
                        Mesh::from(shape::Sphere::default().mesh().build()),
                        Load::<Material>::new(asset_id!("796e953f-3985-4cc2-994b-ff74aaba2fe9")),
                    ));

                    Ok(())
                }
            }

            StartupSystem
        })
        .build();

    provide_context(world);
}
