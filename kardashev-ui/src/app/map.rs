use std::sync::{
    Arc,
    RwLock,
};

use hecs::{
    Entity,
    World,
};
use leptos::{
    component,
    view,
    IntoView,
};
use nalgebra::{
    Point3,
    Projective3,
    Similarity3,
    Vector3,
};
use palette::WithAlpha;
use winit::dpi::PhysicalSize;

use crate::{
    app::{
        components::window::Window,
        Context,
    },
    graphics::{
        camera::Camera,
        renderer::Render3dPlugin,
        transform::Transform,
        window::WindowHandler,
    },
};

stylance::import_crate_style!(style, "src/app/map.module.scss");

#[component]
pub fn Map() -> impl IntoView {
    let Context { world, .. } = Context::get();

    view! {
        <h1>Map</h1>
        <Window handler=WorldRenderer::new(world) render_plugin=Render3dPlugin />
    }
}

struct WorldRenderer {
    world: Arc<RwLock<World>>,
    camera: Entity,
}

impl WorldRenderer {
    pub fn new(world: Arc<RwLock<World>>) -> Self {
        let camera = {
            let mut world = world.write().unwrap();
            world.spawn((
                Transform {
                    transform: Similarity3::face_towards(
                        &Point3::new(-10.0, 0.0, 0.0),
                        &Point3::origin(),
                        &Vector3::new(0.0, 1.0, 0.0),
                        1.0,
                    ),
                },
                Camera {
                    clear_color: Some(palette::named::BLACK.into_format().with_alpha(1.0)),
                    projection: Projective3::identity(),
                },
            ))
        };

        Self { world, camera }
    }
}

impl WindowHandler for WorldRenderer {
    fn on_resize(&mut self, _new_size: PhysicalSize<u32>) {
        // todo: update camera aspect ratio
    }
}

#[derive(Clone)]
pub struct Scene {
    pub world: Arc<RwLock<World>>,
    pub camera: Entity,
}
