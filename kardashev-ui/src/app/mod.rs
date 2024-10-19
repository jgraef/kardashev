mod components;
mod config;
mod map;

use core::str;
use std::task::Poll;

use components::window::provide_graphics;
use hecs::Entity;
use kardashev_client::ApiClient;
use kardashev_protocol::asset_id;
use kardashev_style::style;
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
    Vector3,
};
use palette::WithAlpha;

use self::map::Map;
use crate::{
    app::{
        components::dock::Dock,
        config::{
            provide_config,
            Config,
            Urls,
        },
        map::MapPlugin,
    },
    assets::{
        load::Load,
        system::AssetsPlugin,
    },
    ecs::{
        server::World,
        system::{
            System,
            SystemContext,
        },
        Label,
    },
    error::Error,
    graphics::{
        camera::{
            CameraProjection,
            ClearColor,
        },
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
    universe::star::render::Star,
};

#[style(path = "src/app/app.scss")]
struct Style;

/// Main app component
#[component]
pub fn App() -> impl IntoView {
    let urls = Urls::default();
    tracing::info!(?urls, "endpoints");

    provide_meta_context();
    provide_config();
    provide_graphics();
    provide_world();

    /*let (log_level, _, _) = use_local_storage::<Option<tracing::Level>, OptionCodec<FromToStringCodec>>("log-level");
    create_effect(move |_| {
        let log_level = log_level.get().unwrap_or(Level::DEBUG);
        tracing::info!("setting log level to {log_level:?}");
        tracing_wasm::set_as_global_default_with_config(WASMLayerConfigBuilder::new().set_max_level(log_level).build());
    });*/

    view! {
        <Router>
            <div class=Style::app>
                <Dock />
                <main class=Style::main>
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

fn provide_world() {
    let Config { urls, .. } = expect_context();
    let urls = urls.unwrap_or_default();
    let asset_url = urls.asset_url;
    let api_url = urls.api_url;
    let api_client = ApiClient::new(api_url);
    provide_context(api_client.clone());

    tracing::debug!("creating world");
    let world = World::builder()
        .with_resource(api_client)
        .with_plugin(AssetsPlugin::from_url(asset_url))
        .with_plugin(InputPlugin::default())
        .with_plugin(RenderPlugin)
        .with_plugin(MapPlugin)
        .with_startup_system({
            struct StartupSystem;

            impl System for StartupSystem {
                type Error = Error;

                fn label(&self) -> &'static str {
                    "startup"
                }

                fn poll_system(
                    &mut self,
                    _task_context: &mut std::task::Context<'_>,
                    system_context: &mut SystemContext<'_>,
                ) -> Poll<Result<(), Self::Error>> {
                    //let api_client = context.resources.get::<ApiClient>().unwrap();
                    // todo: we don't want to wait here for a reply, but should spawn a oneshot
                    // system when the request finishes
                    //let stars = api_client.get_stars().await?;

                    let _star_entity = system_context.world.spawn((
                        Transform {
                            model_matrix: Similarity3::identity(),
                        },
                        Mesh::from(shape::Sphere::default().mesh().build()),
                        Load::<Material>::new(asset_id!("4eef57a3-9df8-4fa1-939f-109c3b02f9f0")),
                        Label::new_static("star"),
                    ));

                    let _star2 = system_context.world.spawn((
                        Transform {
                            model_matrix: Similarity3::new(
                                Vector3::new(-3.0, 0.0, 0.0),
                                Vector3::zeros(),
                                1.0,
                            ),
                        },
                        Star {
                            color: palette::named::PINK.into_format().with_alpha(1.0),
                        },
                        Label::new_static("better star"),
                    ));

                    let camera_entity = system_context.world.spawn((
                        Transform::look_at(Point3::new(0., -2., 5.), Point3::origin()),
                        CameraProjection::new(1., 45., 0.1, 100.),
                        ClearColor::new(palette::named::BLACK.into_format().with_alpha(1.0)),
                        Label::new_static("camera"),
                    ));

                    system_context
                        .resources
                        .insert(MainCamera { camera_entity });

                    Poll::Ready(Ok(()))
                }
            }

            StartupSystem
        })
        .build();

    provide_context(world);
}

#[derive(Clone, Debug)]
pub struct MainCamera {
    pub camera_entity: Entity,
}
