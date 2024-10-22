use std::f32::consts::PI;

use kardashev_style::style;
use leptos::{
    component,
    expect_context,
    on_cleanup,
    store_value,
    view,
    IntoView,
};
use nalgebra::{
    Point3,
    Translation3,
    Vector3,
};
use palette::WithAlpha;
use tokio::sync::mpsc;

use crate::{
    app::components::window::{
        Window,
        WindowEvent,
    },
    ecs::{
        plugin::{
            Plugin,
            RegisterPluginContext,
        },
        server::WorldServer,
        system::{
            System,
            SystemContext,
        },
        Label,
    },
    error::Error,
    graphics::{
        blinn_phong::{
            BlinnPhongRenderPipeline,
            CreateBlinnPhongRenderPipeline,
        },
        camera::{
            CameraProjection,
            ClearColor,
            RenderTarget,
        },
        hdr::CreateToneMapPass,
        render_3d::{
            CreateRender3dPass,
            CreateRender3dPipeline,
            CreateRender3dPipelineContext,
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
};

#[style(path = "src/app/world_view.scss")]
struct Style;

#[component]
pub fn WorldView() -> impl IntoView {
    let camera_entity = store_value(None);
    let (tx_mouse, rx_mouse) = mpsc::channel(128);

    let on_load = move |surface: &Surface| {
        tracing::debug!("spawning camera for window");

        let render_target = RenderTarget::new(
            surface,
            CreateToneMapPass {
                inner: CreateRender3dPass {
                    create_pipeline: CreateWorldViewPipeline,
                },
                format: wgpu::TextureFormat::Rgba16Float,
            },
        );
        let surface_size = surface.size();
        let aspect = (surface_size.width as f32) / (surface_size.height as f32);

        let world = expect_context::<WorldServer>();
        let _ = world.run(move |system_context| {
            let entity = system_context.world.spawn((
                Label::new_static("map camera"),
                Transform::look_at(Point3::new(0., -2., 5.), Point3::origin()),
                CameraProjection::new(aspect, PI / 3.0, 0.1, 100.),
                ClearColor::new(palette::named::BLACK.into_format().with_alpha(1.0)),
                WorldViewCameraController {
                    mouse_input: rx_mouse,
                    keyboard_input: system_context
                        .resources
                        .get::<KeyboardInput>()
                        .expect("no keyboard input")
                        .clone(),
                    state: Default::default(),
                    z_mouse: 10.0,
                },
                render_target,
            ));

            camera_entity.set_value(Some(entity));
        });
    };

    let on_event = move |event| {
        match event {
            WindowEvent::Mouse(mouse_event) => {
                let _ = tx_mouse.try_send(mouse_event);
            }
            WindowEvent::Resize { surface_size } => {
                if let Some(camera_entity) = camera_entity.get_value() {
                    let world = expect_context::<WorldServer>();
                    let aspect = (surface_size.width as f32) / (surface_size.height as f32);
                    let _ = world.run(move |system_context| {
                        let mut camera = system_context
                            .world
                            .get::<&mut CameraProjection>(camera_entity)
                            .unwrap();
                        camera.set_aspect(aspect);
                    });
                }
            }
            WindowEvent::Visibility { .. } => {}
        }
    };

    on_cleanup(move || {
        camera_entity.update_value(|camera_entity| {
            if let Some(camera_entity) = *camera_entity {
                let world = expect_context::<WorldServer>();
                let _ = world.run(move |system_context| {
                    let _ = system_context.world.despawn(camera_entity);
                });
            }
            *camera_entity = None;
        });
    });

    view! {
        <div class=Style::window>
            <Window on_load on_event />
        </div>
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct CreateWorldViewPipeline;

impl CreateRender3dPipeline for CreateWorldViewPipeline {
    type Pipeline = WorldViewPipeline;

    fn create_pipeline(self, context: &CreateRender3dPipelineContext) -> WorldViewPipeline {
        WorldViewPipeline {
            pbr: CreateBlinnPhongRenderPipeline.create_pipeline(context),
            //stars: RenderStarPipeline::create_pipeline(pipeline_context),
        }
    }
}

#[derive(Debug)]
struct WorldViewPipeline {
    pbr: BlinnPhongRenderPipeline,
    //stars: RenderStarPipeline,
}

impl Render3dPipeline for WorldViewPipeline {
    fn render(&mut self, pipeline_context: &mut Render3dPipelineContext) {
        self.pbr.render(pipeline_context);
        //self.stars.render(pipeline_context);
    }
}

#[derive(Debug)]
struct WorldViewCameraController {
    mouse_input: mpsc::Receiver<MouseEvent>,
    keyboard_input: KeyboardInput,
    state: InputState,
    z_mouse: f32,
}

#[derive(Clone, Copy, Debug)]
struct WorldViewCameraControllerSystem;

impl System for WorldViewCameraControllerSystem {
    type Error = Error;

    fn label(&self) -> &'static str {
        "map-camera-controller"
    }

    fn poll_system(&mut self, system_context: &mut SystemContext<'_>) -> Result<(), Self::Error> {
        let query = system_context.world.query_mut::<(
            &mut WorldViewCameraController,
            &mut Transform,
            &CameraProjection,
        )>();

        for (_entity, (controller, camera_transform, camera_projection)) in query {
            match controller.mouse_input.try_recv() {
                Ok(event) => {
                    controller.state.mouse.push(&event);

                    match event {
                        MouseEvent::Move { delta, .. } => {
                            if controller.state.mouse.buttons.is_down(MouseButton::Left) {
                                let world_delta =
                                    camera_projection.projection_matrix.unproject_point(
                                        &Point3::new(delta.x, -delta.y, controller.z_mouse),
                                    );
                                camera_transform.model_matrix *= Translation3::from(Vector3::new(
                                    world_delta.x,
                                    world_delta.y,
                                    0.0,
                                ));
                            }

                            if controller.state.mouse.buttons.is_down(MouseButton::Right) {
                                // todo
                            }
                        }
                        MouseEvent::Wheel { delta, .. } => {
                            tracing::debug!(y = delta.y, "wheel");
                            camera_transform.model_matrix *=
                                Translation3::from(Vector3::new(0.0, 0.0, delta.y / 1000.0));
                        }
                        _ => {}
                    }
                }
                Err(_) => {}
            }
        }

        Ok(())
    }
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn register(self, context: RegisterPluginContext) {
        context.schedule.add_system(WorldViewCameraControllerSystem);
    }
}
