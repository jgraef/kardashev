mod components;
mod config;
mod world_view;

use core::str;

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
use nalgebra::Point3;
use palette::{
    Srgb,
    WithAlpha,
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
    collide::ColliderPlugin,
    ecs::{
        server::WorldServer,
        system::SystemContext,
        Label,
    },
    error::Error,
    graphics::{
        light::{
            AmbientLight,
            PointLight,
        },
        material::{
            Material,
            Tint,
        },
        mesh::{
            shape::sphere::Sphere,
            Mesh,
            MeshBuilder,
            Meshable,
        },
        pipeline::forward::blinn_phong::BlinnPhongMaterial,
        transform::{
            Parent,
            Transform,
        },
        RenderPlugin,
    },
    input::InputPlugin,
    utils::futures::spawn_local_and_handle_error,
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
        .with_plugin(AssetsPlugin::from_url(asset_url))
        .with_plugin(InputPlugin::default())
        .with_plugin(ColliderPlugin::default())
        .with_plugin(MapPlugin)
        .with_plugin(RenderPlugin)
        .build();

    provide_context(world.clone());

    spawn_local_and_handle_error(load_world(world, api_client));
}

fn limit_vec<T>(mut vec: Vec<T>, max: usize) -> Vec<T> {
    if vec.len() > max {
        vec.resize_with(max, || unreachable!());
    }
    vec
}

const SOLAR_RADIUS_TO_PARSEC: f32 = 2.25461E-8;

async fn load_world(world: WorldServer, api_client: ApiClient) -> Result<(), Error> {
    let stars = api_client.get_stars().await?;
    let stars = limit_vec(stars, 100);
    tracing::debug!("loaded {} stars from API", stars.len());

    let sphere = Mesh::from(Sphere::default().mesh().build())
        .with_asset_id(asset_id!("d264e0db-9e26-4cca-8469-3fcb1d674bf5"));

    world.run(move |system_context| {
        let root = system_context.world.spawn((Transform::identity(),));

        for star in stars {
            let mut builder = hecs::EntityBuilder::new();
            if let Some(name) = star.name {
                builder.add(Label::new(name));
            }
            builder.add(Parent { entity: root });
            builder.add(
                Transform::from_position(star.position)
                    .with_scaling(star.radius * SOLAR_RADIUS_TO_PARSEC * 1000000.0),
            );
            builder.add(sphere.clone());
            builder.add(Load::<Material<BlinnPhongMaterial>>::new(asset_id!(
                "4eef57a3-9df8-4fa1-939f-109c3b02f9f0"
            )));
            builder.add(Tint {
                tint: star.color.with_alpha(1.0),
            });
            //builder.add(PointLight::new(star.color));

            system_context.world.spawn(builder.build());
        }
    });

    Ok(())
}

#[allow(dead_code)]
fn create_test_world(system_context: &mut SystemContext) {
    let shape = Sphere::default().mesh().build();
    //let shape = shape::Cuboid::default().mesh().build();
    //let shape2 = shape::Sphere::default()
    //    .mesh()
    //    .with_mesh_type(shape::SphereMeshType::Ico { subdivisions: 40 })
    //    .build();
    let sphere = Mesh::from(shape).with_asset_id(asset_id!("d264e0db-9e26-4cca-8469-3fcb1d674bf5"));
    //let sphere2 =
    //    Mesh::from(shape2).with_asset_id(asset_id!("
    // ca1524bb-501d-4430-8ded-0672f44e7aa3"));

    const SUN_LIGHT_COLOR: Srgb<f32> = Srgb::new(1.0, 0.92902, 0.89906);

    let _sun = system_context.world.spawn((
        Transform::from_position(Point3::origin()),
        sphere.clone(),
        Load::<Material<BlinnPhongMaterial>>::new(asset_id!(
            "4eef57a3-9df8-4fa1-939f-109c3b02f9f0"
        )),
        //Load::<Material<PbrMaterial>>::new(asset_id!("4eef57a3-9df8-4fa1-939f-109c3b02f9f0")),
        Label::new_static("star"),
        PointLight::new(SUN_LIGHT_COLOR),
    ));

    let _earth = system_context.world.spawn((
        Transform::from_position(Point3::new(-5.0, 0.0, 0.0)),
        sphere,
        Load::<Material<BlinnPhongMaterial>>::new(asset_id!(
            "d5b74211-70fb-4b4c-9199-c5aa89b90b01" //"cbef3406-54ae-4832-bebf-27c3ac9e130c"
        )),
        //Load::<Material<PbrMaterial>>::new(asset_id!("d5b74211-70fb-4b4c-9199-c5aa89b90b01")),
        Label::new_static("earth"),
    ));

    /*let _earth2 = system_context.world.spawn((
        Transform::from_position(Point3::new(5.0, 0.0, 0.0)),
        sphere2,
        Load::<Material<BlinnPhongMaterial>>::new(asset_id!(
            "d5b74211-70fb-4b4c-9199-c5aa89b90b01" //"cbef3406-54ae-4832-bebf-27c3ac9e130c"
        )),
        //Load::<Material<PbrMaterial>>::new(asset_id!("d5b74211-70fb-4b4c-9199-c5aa89b90b01")),
        Label::new_static("earth"),
    ));*/

    system_context.resources.insert(AmbientLight {
        color: palette::named::WHITE.into_format() * 0.1,
    });
}
