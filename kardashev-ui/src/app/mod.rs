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
    Point3,
    UnitQuaternion,
};
use palette::Srgb;

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
        light::{
            AmbientLight,
            PointLight,
        },
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
    let shape = shape::Sphere::default().mesh().build();
    //let shape = shape::Cuboid::default().mesh().build();
    let sphere = Mesh::from(shape).with_asset_id(asset_id!("d264e0db-9e26-4cca-8469-3fcb1d674bf5"));

    const SUN_LIGHT_COLOR: Srgb<f32> = Srgb::new(1.0, 0.92902, 0.89906);

    let _sun = system_context.world.spawn((
        Transform::from_position(Point3::origin()),
        sphere.clone(),
        Load::<Material<BlinnPhongMaterial>>::new(asset_id!(
            "4eef57a3-9df8-4fa1-939f-109c3b02f9f0"
        )),
        Load::<Material<PbrMaterial>>::new(asset_id!("4eef57a3-9df8-4fa1-939f-109c3b02f9f0")),
        Label::new_static("star"),
        PointLight::new(SUN_LIGHT_COLOR),
    ));

    let _earth = system_context.world.spawn((
        Transform::from_position(Point3::new(-5.0, 0.0, 0.0))
            .with_rotation(UnitQuaternion::from_euler_angles(0.25 * PI, 0.25 * PI, 0.0)),
        sphere,
        Load::<Material<BlinnPhongMaterial>>::new(asset_id!(
            "d5b74211-70fb-4b4c-9199-c5aa89b90b01" //"cbef3406-54ae-4832-bebf-27c3ac9e130c"
        )),
        Load::<Material<PbrMaterial>>::new(asset_id!("d5b74211-70fb-4b4c-9199-c5aa89b90b01")),
        Label::new_static("earth"),
    ));

    system_context.resources.insert(AmbientLight {
        color: palette::named::WHITE.into_format() * 0.1,
    });
}
