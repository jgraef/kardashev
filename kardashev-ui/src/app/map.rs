use std::task::Poll;

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
    Translation3,
    Vector3,
};
use tokio::sync::mpsc;

use crate::{
    app::{
        components::window::{
            Window,
            WindowEvent,
        },
        MainCamera,
    },
    ecs::{
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
    error::Error,
    graphics::{
        camera::{
            CameraProjection,
            ChangeCameraAspectRatio,
            RenderTarget,
        },
        render_3d::{
            CreateRender3dPipelineContext,
            Render3dMeshesWithMaterial,
            Render3dPass,
            Render3dPipeline,
            Render3dPipelineContext,
        },
        transform::Transform,
        Surface,
    },
    input::{
        keyboard::KeyboardInput,
        mouse::{
            MouseButton,
            MouseEvent,
        },
        InputState,
    },
    universe::star::render::RenderStarPipeline,
};

#[style(path = "src/app/map.scss")]
struct Style;

#[component]
pub fn Map() -> impl IntoView {
    let camera_entity = store_value(None);
    let (tx_mouse, rx_mouse) = mpsc::channel(128);

    let on_load = move |surface: &Surface| {
        tracing::debug!("spawning camera for window");

        let world = expect_context::<World>();
        let render_target = RenderTarget::from_surface::<Render3dPass<MapPipeline>>(surface);

        world.run_system(AttachCamera {
            render_target: Some(render_target),
            mouse_events: Some(rx_mouse),
            camera_entity,
        });
    };

    let on_event = move |event| {
        match event {
            WindowEvent::Mouse(mouse_event) => {
                let _ = tx_mouse.send(mouse_event);
            }
            WindowEvent::Resize { surface_size } => {
                if let Some(camera_entity) = camera_entity.get_value() {
                    let world = expect_context::<World>();
                    let aspect = (surface_size.width as f32) / (surface_size.height as f32);
                    world.run_system(ChangeCameraAspectRatio {
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
                world.run_system(DetachCamera { camera_entity });
            }
            *camera_entity = None;
        });
    });

    view! {
        <Window on_load on_event />
    }
}

#[derive(Debug)]
struct MapPipeline {
    meshes_with_material: Render3dMeshesWithMaterial,
    stars: RenderStarPipeline,
}

impl Render3dPipeline for MapPipeline {
    fn create_pipeline(pipeline_context: &CreateRender3dPipelineContext) -> Self {
        Self {
            meshes_with_material: Render3dMeshesWithMaterial::create_pipeline(pipeline_context),
            stars: RenderStarPipeline::create_pipeline(pipeline_context),
        }
    }

    fn render(&mut self, pipeline_context: &mut Render3dPipelineContext) {
        self.meshes_with_material.render(pipeline_context);
        self.stars.render(pipeline_context);
    }
}

#[derive(Debug)]
struct AttachCamera {
    camera_entity: StoredValue<Option<Entity>>,
    render_target: Option<RenderTarget>,
    mouse_events: Option<mpsc::Receiver<MouseEvent>>,
}

impl System for AttachCamera {
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

        let keyboard_input = system_context
            .resources
            .get::<KeyboardInput>()
            .expect("no keyboard input")
            .clone();
        let mouse_input = self
            .mouse_events
            .take()
            .expect("system was not supposed to be polled again");
        let controller = MapCameraController {
            mouse_input,
            keyboard_input,
            state: Default::default(),
        };

        if let Some(MainCamera { camera_entity }) = system_context.resources.get::<MainCamera>() {
            let _ = system_context
                .world
                .insert(*camera_entity, (render_target, controller));
            self.camera_entity.set_value(Some(*camera_entity));
        }

        Poll::Ready(Ok(()))
    }
}

struct DetachCamera {
    camera_entity: Entity,
}

impl System for DetachCamera {
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
            .remove::<(RenderTarget, MapCameraController)>(self.camera_entity);
        Poll::Ready(Ok(()))
    }
}

#[derive(Debug)]
struct MapCameraController {
    mouse_input: mpsc::Receiver<MouseEvent>,
    keyboard_input: KeyboardInput,
    state: InputState,
}

#[derive(Clone, Copy, Debug)]
struct MapCameraControllerSystem;

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
        let mut pending = false;

        // todo: listen for component-added events

        let query =
            system_context
                .world
                .query_mut::<(&mut MapCameraController, &mut Transform, &CameraProjection)>();

        for (_entity, (controller, camera_transform, camera_projection)) in query {
            match controller.mouse_input.poll_recv(task_context) {
                Poll::Ready(Some(event)) => {
                    tracing::debug!(?event);
                    controller.state.mouse.push(&event);

                    if let MouseEvent::Move { position: _, delta } = event {
                        if controller.state.mouse.buttons.is_down(MouseButton::Left) {
                            let transform = camera_projection.projection_matrix.as_projective()
                                * camera_transform.model_matrix;
                            //let world_position =
                            // transform.inverse_transform_point(&Point3::new(position.x,
                            // position.y, controller.z_mouse));
                            let world_delta = transform
                                .inverse_transform_vector(&Vector3::new(delta.x, delta.y, 0.0));
                            camera_transform
                                .model_matrix
                                .append_translation_mut(&Translation3::from(world_delta));
                        }

                        if controller.state.mouse.buttons.is_down(MouseButton::Right) {
                            // todo
                        }
                    }
                }
                Poll::Ready(None) => {}
                Poll::Pending => {
                    pending = true;
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
        context.schedule.add_system(MapCameraControllerSystem);
    }
}
