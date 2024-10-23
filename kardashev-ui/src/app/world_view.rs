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
use tokio::sync::{
    mpsc,
    watch,
};

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
            DontRender,
            RenderTarget,
        },
        hdr::CreateToneMapPass,
        pbr::{
            CreatePbrRenderPipeline,
            PbrRenderPipeline,
        },
        render_3d::{
            CreateRender3dPass,
            CreateRender3dPipeline,
            CreateRender3dPipelineContext,
            Render3dPipeline,
            Render3dPipelineContext,
        },
        render_frame::{
            AttachedRenderPass,
            CreateRenderPass,
        },
        transform::Transform,
        Surface,
    },
    input::{
        keyboard::{
            KeyCode,
            KeyboardEvent,
            KeyboardInput,
        },
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
    let (tx_pipeline_switch, rx_pipeline_switch) = watch::channel(WhichPipeline::BlinnPhong);

    let on_load = move |surface: &Surface| {
        tracing::debug!("spawning camera for window");

        let surface_size = surface.size();
        let aspect = (surface_size.width as f32) / (surface_size.height as f32);

        let render_target = RenderTarget::from_surface(surface);
        let render_pass = AttachedRenderPass::new(
            CreateToneMapPass {
                inner: CreateRender3dPass {
                    create_pipeline: CreateWorldViewPipeline {
                        switch: rx_pipeline_switch,
                    },
                },
                format: wgpu::TextureFormat::Rgba16Float,
            }
            .create_render_pass_from_surface(&surface),
        );

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
                    switch_pipeline: tx_pipeline_switch,
                },
                render_target,
                render_pass,
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
            WindowEvent::Visibility { visible } => {
                if let Some(camera_entity) = camera_entity.get_value() {
                    let world = expect_context::<WorldServer>();
                    let _ = world.run(move |system_context| {
                        if visible {
                            let _ = system_context.world.remove_one::<DontRender>(camera_entity);
                        }
                        else {
                            system_context
                                .world
                                .insert_one(camera_entity, DontRender)
                                .unwrap();
                        }
                    });
                }
            }
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

#[derive(Clone, Debug)]
struct CreateWorldViewPipeline {
    switch: watch::Receiver<WhichPipeline>,
}

impl CreateRender3dPipeline for CreateWorldViewPipeline {
    type Pipeline = WorldViewPipeline;

    fn create_pipeline(self, context: &CreateRender3dPipelineContext) -> WorldViewPipeline {
        WorldViewPipeline {
            switch: self.switch,
            pbr: CreatePbrRenderPipeline.create_pipeline(context),
            blinn_phong: CreateBlinnPhongRenderPipeline.create_pipeline(context),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum WhichPipeline {
    Pbr,
    BlinnPhong,
}

impl WhichPipeline {
    pub fn toggle(&mut self) {
        *self = match *self {
            WhichPipeline::Pbr => WhichPipeline::BlinnPhong,
            WhichPipeline::BlinnPhong => WhichPipeline::Pbr,
        };
    }
}

#[derive(Debug)]
struct WorldViewPipeline {
    switch: watch::Receiver<WhichPipeline>,
    pbr: PbrRenderPipeline,
    blinn_phong: BlinnPhongRenderPipeline,
}

impl Render3dPipeline for WorldViewPipeline {
    fn render(&mut self, pipeline_context: &mut Render3dPipelineContext) {
        match *self.switch.borrow() {
            WhichPipeline::Pbr => {
                self.pbr.render(pipeline_context);
            }
            WhichPipeline::BlinnPhong => {
                self.blinn_phong.render(pipeline_context);
            }
        }
    }
}

#[derive(Debug)]
struct WorldViewCameraController {
    mouse_input: mpsc::Receiver<MouseEvent>,
    keyboard_input: KeyboardInput,
    state: InputState,
    z_mouse: f32,
    switch_pipeline: watch::Sender<WhichPipeline>,
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
            loop {
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
                                    camera_transform.model_matrix *= Translation3::from(
                                        Vector3::new(world_delta.x, world_delta.y, 0.0),
                                    );
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
                    Err(_) => break,
                }
            }

            loop {
                match controller.keyboard_input.try_next() {
                    Some(event) => {
                        match event {
                            KeyboardEvent::KeyDown {
                                code: KeyCode::F9,
                                repeat: false,
                                ..
                            } => {
                                controller
                                    .switch_pipeline
                                    .send_modify(|which| which.toggle());
                            }
                            _ => {}
                        }
                    }
                    None => break,
                }
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
