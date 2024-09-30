mod components;
mod map;

use components::window::provide_graphics;
use kardashev_client::{
    ApiClient,
    AssetClient,
};
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
use nalgebra::{
    Point3,
    Similarity3,
    Translation3,
    Vector3,
};
use url::Url;

use self::map::Map;
use crate::{
    app::components::dock::Dock,
    assets::{
        load_image,
        Assets,
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
        Graphics,
        RenderPlugin,
    },
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

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_client();
    provide_graphics();
    provide_world();

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

fn provide_client() {
    let urls = Urls::default();
    tracing::info!(?urls, "endpoints");
    let api_client = ApiClient::new(urls.api_url);
    let asset_client = AssetClient::new(urls.asset_url);
    provide_context(api_client);
    provide_context(asset_client);
}

fn provide_world() {
    let api_client = expect_context::<ApiClient>();
    let asset_client = expect_context::<AssetClient>();

    tracing::debug!("creating world");
    let world = World::builder()
        .with_resource(asset_client)
        .with_resource(api_client)
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

                    // todo: use asset server
                    let star_texture =
                        load_image("assets/12dc9bbe-ca8f-4860-a739-26357e4e3bd2.png")
                            .await
                            .unwrap();
                    let mesh = Mesh::from(shape::Sphere::default().mesh().build());
                    let material = Material::from_diffuse_image(star_texture.clone());

                    context.world.spawn((
                        Transform {
                            matrix: Similarity3::new(Vector3::zeros(), Vector3::zeros(), 1.0),
                        },
                        mesh,
                        material,
                    ));

                    Ok(())
                }
            }

            StartupSystem
        })
        .build();

    provide_context(world);
}
