use leptos::{
    component,
    spawn_local,
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
    graphics::{
        camera::{
            Camera,
            ClearColor,
        },
        transform::Transform,
    },
};

stylance::import_crate_style!(style, "src/app/map.module.scss");

#[component]
pub fn Map() -> impl IntoView {
    view! {
        <h1>Map</h1>
        <Window on_load=|render_target| {
            spawn_local(async move {
                let Context { world, .. } = Context::get();
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
                        projection: Projective3::identity(),
                    },
                    ClearColor {
                        clear_color: palette::named::BLACK.into_format().with_alpha(1.0),
                    },
                    render_target,
                )).await;
            });
        } />
    }
}
