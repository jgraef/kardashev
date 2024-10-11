use std::{
    future::Future,
    pin::pin,
    task::Poll,
};

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
use nalgebra::Translation3;
use tokio::sync::watch;

use crate::{
    app::{
        components::window::{
            Window,
            WindowEvent,
        },
        MainCamera,
    },
    error::Error,
    graphics::{
        camera::ChangeCameraAspectRatio,
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
        plugin::{
            Plugin,
            RegisterPluginContext,
        },
        server::World,
        system::{
            System,
            SystemContext,
        },
    },
};

#[style(path = "src/app/map.scss")]
struct Style;

#[component]
pub fn Map() -> impl IntoView {
    let camera_entity = store_value(None);
    let (tx_input_state, _rx_input_state) = watch::channel(InputState::default());

    let on_load = move |surface: &Surface| {
        tracing::debug!("spawning camera for window");

        let world = expect_context::<World>();
        let render_target = RenderTarget::from_surface(surface);

        world.add_system(AttachRenderTarget {
            render_target: Some(render_target),
            camera_entity,
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
                if let Some(camera_entity) = camera_entity.get_value() {
                    let world = expect_context::<World>();
                    let aspect = (surface_size.width as f32) / (surface_size.height as f32);
                    world.add_system(ChangeCameraAspectRatio {
                        camera_entity,
                        aspect,
                    });
                }
            }
            WindowEvent::Visibility { .. } => {}
        }
    };

    on_cleanup(move || {
        camera_entity.update_value(|camera_entity| {
            if let Some(camera_entity) = *camera_entity {
                let world = expect_context::<World>();
                world.add_system(DetachRenderTarget { camera_entity });
            }
            *camera_entity = None;
        });
    });

    view! {
        <Window on_load on_event />
    }
}

#[derive(Debug)]
struct AttachRenderTarget {
    render_target: Option<RenderTarget>,
    camera_entity: StoredValue<Option<Entity>>,
}

impl System for AttachRenderTarget {
    type Error = Error;

    fn label(&self) -> &'static str {
        "attach-render-target"
    }

    fn poll_system(
        &mut self,
        _task_context: &mut std::task::Context<'_>,
        system_context: &mut SystemContext<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let render_target = self
            .render_target
            .take()
            .expect("system was not supposed to be polled again");

        if let Some(MainCamera { camera_entity }) = system_context.resources.get::<MainCamera>() {
            let _ = system_context
                .world
                .insert_one(*camera_entity, render_target);
            self.camera_entity.set_value(Some(*camera_entity));
        }

        Poll::Ready(Ok(()))
    }
}

struct DetachRenderTarget {
    camera_entity: Entity,
}

impl System for DetachRenderTarget {
    type Error = Error;

    fn label(&self) -> &'static str {
        "detach-render-target"
    }

    fn poll_system(
        &mut self,
        _task_context: &mut std::task::Context<'_>,
        system_context: &mut SystemContext<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let _ = system_context
            .world
            .remove_one::<RenderTarget>(self.camera_entity);
        Poll::Ready(Ok(()))
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
    type Error = Error;

    fn label(&self) -> &'static str {
        "map-camera-controller"
    }

    fn poll_system(
        &mut self,
        task_context: &mut std::task::Context<'_>,
        system_context: &mut SystemContext<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        // if at least one future is pending
        #[allow(unused_assignments)]
        let mut pending = false;

        // todo: listen for component-added events
        // for now we'll pretend the events thingy exists, and is always pending
        pending = true;

        let query = system_context
            .world
            .query_mut::<(&mut MapCameraController, &mut Transform)>();

        for (entity, (controller, transform)) in query {
            {
                // `changed` is cancel-safe, so we can do this.
                // but this needs to be in a block, otherwise rustc will complain that
                // `controller.input_state` is still borrowed by the future.
                match pin!(controller.input_state.changed()).poll(task_context) {
                    Poll::Pending => {
                        pending = true;
                        continue;
                    }
                    Poll::Ready(Ok(())) => {
                        // the input state changed
                    }
                    Poll::Ready(Err(_)) => {
                        // sender dropped, remove the component
                        tracing::debug!(?entity, "input state sender dropped. removing map camera controller from entity");
                        system_context
                            .command_buffer
                            .remove_one::<MapCameraController>(entity);
                        continue;
                    }
                }
            }

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

        // at least one future is still pending. if there are no controllers, it will
        // usually still be pending by listening for component-added events (todo)
        if pending {
            Poll::Pending
        }
        else {
            // otherwise there is no more work to be done by this system
            Poll::Ready(Ok(()))
        }
    }
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn register(self, context: RegisterPluginContext) {
        context
            .schedule
            .add_system(MapCameraControllerSystem { step_size: 0.5 });
    }
}
