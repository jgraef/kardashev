use hecs::Entity;
use kardashev_style::style;
use leptos::{
    component,
    expect_context,
    on_cleanup,
    store_value,
    view,
    IntoView,
    StoredValue,
};
use nalgebra::{
    Point3,
    Translation3,
};
use palette::WithAlpha;
use tokio::sync::watch;

use crate::{
    app::components::window::{
        Window,
        WindowEvent,
    },
    error::Error,
    graphics::{
        camera::{
            Camera,
            ChangeCameraAspectRatio,
            ClearColor,
        },
        pipeline::RenderTarget,
        transform::Transform,
        Surface,
    },
    input::{
        keyboard::KeyCode,
        InputEvent,
        InputState,
    },
    world::{
        Label,
        OneshotSystem,
        Plugin,
        RegisterPluginContext,
        RunSystemContext,
        System,
        World,
    },
};

#[style(path = "src/app/map.scss")]
struct Style;

#[component]
pub fn Map() -> impl IntoView {
    let camera_entity = store_value(None);
    let (tx_input_state, rx_input_state) = watch::channel(InputState::default());

    let on_load = move |surface: &Surface| {
        tracing::debug!("spawning camera for window");

        let world = expect_context::<World>();
        let render_target = RenderTarget::from_surface(surface);

        world.run_oneshot_system(SpawnMapCamera {
            render_target,
            camera_entity,
            input_state: rx_input_state,
        })
    };

    let on_event = move |event| {
        tracing::debug!(?event);

        match event {
            WindowEvent::Mouse(mouse_event) => {
                tx_input_state
                    .send_modify(|input_state| input_state.push(&InputEvent::Mouse(mouse_event)));
            }
            WindowEvent::Resize { surface_size } => {
                camera_entity.with_value(move |camera_entity| {
                    if let Some(camera_entity) = *camera_entity {
                        let world = expect_context::<World>();
                        let aspect = (surface_size.width as f32) / (surface_size.height as f32);
                        world.run_oneshot_system(ChangeCameraAspectRatio {
                            camera_entity,
                            aspect,
                        });
                    }
                });
            }
            WindowEvent::Visibility { .. } => {}
        }
    };

    on_cleanup(move || {
        camera_entity.update_value(|camera_entity| {
            if let Some(camera_entity) = camera_entity.take() {
                tracing::debug!(?camera_entity, "despawning camera");

                let world = expect_context::<World>();
                world.despawn(camera_entity);
            }
        });
    });

    view! {
        <Window on_load on_event />
    }
}

#[derive(Debug)]
struct SpawnMapCamera {
    render_target: RenderTarget,
    camera_entity: StoredValue<Option<Entity>>,
    input_state: watch::Receiver<InputState>,
}

impl OneshotSystem for SpawnMapCamera {
    fn label(&self) -> &'static str {
        "spawn-map-camera"
    }

    async fn run<'c: 'd, 'd>(self, context: &'d mut RunSystemContext<'c>) -> Result<(), Error> {
        let entity = context.world.spawn((
            Transform::look_at(Point3::new(0., -2., 5.), Point3::origin()),
            Camera::new(1., 45., 0.1, 100.),
            ClearColor {
                clear_color: palette::named::BLACK.into_format().with_alpha(1.0),
            },
            Label {
                label: "map".into(),
            },
            self.render_target,
        ));

        self.camera_entity.set_value(Some(entity));

        Ok(())
    }
}

#[derive(Debug)]
struct MapCameraController {
    input_state: watch::Receiver<InputState>,
}

#[derive(Clone, Copy, Debug)]
struct MapCameraControllerSystem {
    step_size: f32,
}

impl System for MapCameraControllerSystem {
    fn label(&self) -> &'static str {
        "map-camera-controller"
    }

    async fn run<'s: 'c, 'c: 'd, 'd>(
        &'s mut self,
        context: &'d mut RunSystemContext<'c>,
    ) -> Result<(), Error> {
        let query = context
            .world
            .query_mut::<(&mut MapCameraController, &mut Transform)>();

        for (_, (controller, transform)) in query {
            let input_state = controller.input_state.borrow_and_update();

            for key_code in &input_state.keys_pressed {
                match *key_code {
                    KeyCode::KeyW => {
                        transform.model_matrix *= Translation3::new(0.0, self.step_size, 0.0)
                    }
                    KeyCode::KeyA => {
                        transform.model_matrix *= Translation3::new(-self.step_size, 0.0, 0.0)
                    }
                    KeyCode::KeyS => {
                        transform.model_matrix *= Translation3::new(0.0, -self.step_size, 0.0)
                    }
                    KeyCode::KeyD => {
                        transform.model_matrix *= Translation3::new(-self.step_size, 0.0, 0.0)
                    }
                    KeyCode::KeyR => {
                        transform.model_matrix *= Translation3::new(0.0, 0.0, -self.step_size)
                    }
                    KeyCode::KeyF => {
                        transform.model_matrix *= Translation3::new(0.0, 0.0, self.step_size)
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn register(self, context: RegisterPluginContext) {
        context
            .scheduler
            .add_update_system(MapCameraControllerSystem { step_size: 0.5 });
    }
}
