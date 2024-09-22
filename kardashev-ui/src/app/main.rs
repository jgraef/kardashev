use hecs::World;
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

use crate::scene::{
    camera::Camera,
    renderer::SceneView,
    transform::Transform,
    window::leptos::Window,
    Scene,
};

#[component]
pub fn MainPage() -> impl IntoView {
    let mut world = World::new();

    let camera = world.spawn((
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
    ));

    /*let _teapot = world.spawn((
        Transform {
            transform: Similarity3::identity(),
        },
        TEAPOT_MESH.clone(),
    ));*/

    let scene = Scene::new(world);

    let scene_view = SceneView { scene, camera };

    view! {
        <Window scene_view />
    }
}
