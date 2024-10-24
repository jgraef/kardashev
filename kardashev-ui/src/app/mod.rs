mod components;
mod config;
mod world_view;

use core::str;
use std::f32::consts::PI;

use components::window::provide_graphics;
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
use leptos_router::Router;
use nalgebra::{
    Similarity3, UnitQuaternion, Vector3
};

use crate::{
    app::{
        config::{
            provide_config,
            Config,
            Urls,
        },
        world_view::{
            MapPlugin,
            WorldView,
        },
    },
    assets::{
        load::Load,
        system::AssetsPlugin,
    },
    ecs::{
        server::WorldServer,
        system::SystemContext,
        Label,
    },
    graphics::{
        blinn_phong::BlinnPhongMaterial,
        light::AmbientLight,
        material::Material,
        mesh::{
            shape,
            Mesh,
            MeshBuilder,
            Meshable,
        },
        pbr::PbrMaterial,
        transform::Transform,
        RenderPlugin,
    },
    input::InputPlugin,
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
                //<Dock />
                <main class=Style::main>
                    /*<Routes>
                        <Route path="/" view=|| view!{ <Redirect path="/dashboard"/> } />
                        <Route path="/dashboard" view=|| view!{ "TODO: Dashboard" } />
                        <Route path="/map" view=Map />
                    </Routes>*/
                    <WorldView />
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
    let world = WorldServer::builder()
        .with_resource(api_client)
        .with_plugin(AssetsPlugin::from_url(asset_url))
        .with_plugin(InputPlugin::default())
        .with_plugin(RenderPlugin)
        .with_plugin(MapPlugin)
        .with_startup_system(create_world)
        .build();

    provide_context(world);
}

fn create_world(system_context: &mut SystemContext) {
    //let api_client = context.resources.get::<ApiClient>().unwrap();
    // todo: we don't want to wait here for a reply, but should spawn a oneshot
    // system when the request finishes
    //let stars = api_client.get_stars().await?;

    //let sphere = Mesh::from(shape::Sphere::default().mesh().build());
    let sphere = Mesh::from(shape::Cuboid::default().mesh().build());

    let _sun = system_context.world.spawn((
        Transform {
            model_matrix: Similarity3::identity(),
        }.with_rotation(UnitQuaternion::from_euler_angles(0.25 * PI, 0.25 * PI, 0.25 * PI)),
        sphere.clone(),
        Load::<Material<BlinnPhongMaterial>>::new(asset_id!(
            //"4eef57a3-9df8-4fa1-939f-109c3b02f9f0"
            "cbef3406-54ae-4832-bebf-27c3ac9e130c"
        )),
        Load::<Material<PbrMaterial>>::new(asset_id!("4eef57a3-9df8-4fa1-939f-109c3b02f9f0")),
        Label::new_static("star"),
    ));

    let _earth = system_context.world.spawn((
        Transform {
            model_matrix: Similarity3::new(Vector3::new(-5.0, 0.0, 0.0), Vector3::zeros(), 1.0),
        },
        sphere,
        Load::<Material<BlinnPhongMaterial>>::new(asset_id!(
            "d5b74211-70fb-4b4c-9199-c5aa89b90b01"
        )),
        Load::<Material<PbrMaterial>>::new(asset_id!("d5b74211-70fb-4b4c-9199-c5aa89b90b01")),
        Label::new_static("earth"),
    ));

    /*let _light = system_context.world.spawn((
        Transform {
            model_matrix: Similarity3::new(Vector3::new(0.0, -2.0, 5.0), Vector3::zeros(), 1.0),
        },
        PointLight {
            color: palette::named::YELLOW.into_format(),
        },
    ));*/

    system_context.resources.insert(AmbientLight {
        color: palette::named::WHITE.into_format() * 0.1,
    });
}
