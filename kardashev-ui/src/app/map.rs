use hecs::Entity;
use leptos::{
    component,
    expect_context,
    on_cleanup,
    spawn_local,
    store_value,
    view,
    IntoView,
};
use nalgebra::{
    Similarity3,
    Translation3,
};
use palette::WithAlpha;

use crate::{
    app::components::window::Window,
    error::Error,
    graphics::{
        camera::{
            Camera,
            ClearColor,
        },
        rendering_system::RenderTarget,
        transform::Transform,
        Surface,
        SurfaceSize,
    },
    world::{
        Label,
        OneshotSystem,
        RunSystemContext,
        World,
    },
};

stylance::import_crate_style!(style, "src/app/map.module.scss");

#[component]
pub fn Map() -> impl IntoView {
    let camera_entity = store_value(None);

    let on_load = move |surface: &Surface| {
        let render_target = RenderTarget::from_surface(surface);

        spawn_local(async move {
            tracing::debug!("spawning camera for window");

            let world = expect_context::<World>();
            let entity = world
                .spawn((
                    Transform {
                        /*matrix: Similarity3::look_at_lh(
                            &Point3::new(-2.0, 0.0, 0.0),
                            &Point3::origin(),
                            &Vector3::new(0.0, 1.0, 0.0),
                            1.0,
                        ),*/
                        //matrix: Similarity3::identity()
                        matrix: Similarity3::identity() * Translation3::new(0., 0., 5.0),
                        //matrix: Similarity3::look_at_lh(&Point3::new(0.0, 0.0, 5.0),
                        // &Point3::origin(), &Vector3::new(0.0, 1.0, 0.0), 1.0),
                    },
                    Camera::new(1., 45., 0.1, 100.),
                    ClearColor {
                        clear_color: palette::named::DARKSLATEGRAY.into_format().with_alpha(1.0),
                    },
                    Label {
                        label: "map".into(),
                    },
                    render_target,
                ))
                .await;

            camera_entity.set_value(Some(entity));
        });
    };

    let on_resize = move |surface_size: SurfaceSize| {
        camera_entity.with_value(move |camera_entity| {
            if let Some(camera_entity) = *camera_entity {
                let world = expect_context::<World>();
                let aspect = (surface_size.width as f32) / (surface_size.height as f32);
                spawn_local(async move {
                    world
                        .run_oneshot_system(ChangeCameraAspectRatio {
                            camera_entity,
                            aspect,
                        })
                        .await;
                });
            }
        });
    };

    let on_input = move |_event| {
        // todo: send through channel with receiver in a component attached to
        // the camera
    };

    on_cleanup(move || {
        camera_entity.update_value(|camera_entity| {
            if let Some(camera_entity) = camera_entity.take() {
                tracing::debug!(?camera_entity, "despawning camera for window");

                let world = expect_context::<World>();
                spawn_local(async move {
                    world.despawn(camera_entity).await;
                });
            }
        });
    });

    view! {
        <h1>Map</h1>
        <Window on_load on_resize on_input />
    }
}

#[derive(Debug)]
pub struct ChangeCameraAspectRatio {
    pub camera_entity: Entity,
    pub aspect: f32,
}

impl OneshotSystem for ChangeCameraAspectRatio {
    fn label(&self) -> &'static str {
        "change-camera-aspect-ratio"
    }

    async fn run<'c: 'd, 'd>(self, context: &'d mut RunSystemContext<'c>) -> Result<(), Error> {
        let mut camera = context
            .world
            .get::<&mut Camera>(self.camera_entity)
            .unwrap();
        camera.set_aspect(self.aspect);
        Ok(())
    }
}
