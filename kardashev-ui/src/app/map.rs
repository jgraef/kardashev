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

use crate::{
    app::{
        components::window::Window,
        Context,
    },
    renderer::{
        camera::Camera,
        transform::Transform,
        Scene,
    },
};

stylance::import_crate_style!(style, "src/app/map.module.scss");

#[component]
pub fn Map() -> impl IntoView {
    let Context { world, .. } = Context::get();

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

    let scene = Scene { world, camera };

    view! {
        <h1>Map</h1>
        <Window scene />
    }
}
