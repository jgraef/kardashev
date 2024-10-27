use std::f32::consts::PI;

use kardashev_style::style;
use leptos::{
    component,
    create_rw_signal,
    event_target_value,
    expect_context,
    on_cleanup,
    store_value,
    view,
    IntoView,
    RwSignal,
    SignalGet,
    SignalGetUntracked,
    SignalSet,
    SignalUpdate,
    SignalWith,
};
use nalgebra::{
    Point3,
    Similarity3,
    Translation3,
    UnitQuaternion,
    Vector3,
};
use palette::WithAlpha;
use strum::VariantNames;
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
        system::SystemContext,
        Label,
    },
    graphics::{
        camera::{
            CameraProjection,
            ClearColor,
            DontRender,
            RenderTarget,
        },
        pipeline::{
            deferred::{
                debug::Channel,
                CreateDeferredPipeline,
                DeferredPipeline,
            },
            forward::blinn_phong::{
                BlinnPhongRenderPipeline,
                CreateBlinnPhongRenderPipeline,
            },
            hdr::{
                CreateHdrPipeline,
                HdrPipeline,
                ToneMap,
            },
            CreatePipeline,
            CreatePipelineContext,
            RenderPipeline,
            RenderPipelineContext,
            RenderWorldInput,
            TextureConfig,
            TextureOutput,
        },
        render_frame::{
            CreateRenderView,
            DynRenderView,
            RenderFrameInfo,
        },
        stats::ResourceUsages,
        transform::{
            Parent,
            Transform,
        },
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
    utils::human_size,
};

#[style(path = "src/app/world_view.scss")]
struct Style;

#[component]
pub fn WorldView() -> impl IntoView {
    let camera_entity = store_value(None);
    let (tx_mouse, rx_mouse) = mpsc::channel(128);

    let debug_info = create_rw_signal(DebugInfo::default());
    let debug_config = DebugConfig {
        pipeline: create_rw_signal(Pipeline::ForwardBlinnPhong),
        tone_map: create_rw_signal(ToneMap::Aces),
        gamma: create_rw_signal(1.0),
    };

    let on_load = move |surface: &Surface| {
        tracing::debug!("spawning camera for window");

        let surface_size = surface.size();
        let aspect = (surface_size.width as f32) / (surface_size.height as f32);

        let render_target = RenderTarget::from_surface(surface);
        let render_view = DynRenderView::new(
            CreateWorldViewPipeline { debug_config }.create_render_view_from_surface(surface),
        );

        let world = expect_context::<WorldServer>();
        let _ = world.run(move |system_context| {
            system_context.resources.insert(debug_info);

            let entity = system_context.world.spawn((
                Label::new_static("map camera"),
                Transform::look_at(Point3::new(0., 0., 5.), Point3::origin(), Vector3::y()),
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
                    debug_config,
                },
                render_target,
                render_view,
            ));

            let _light = system_context.world.spawn((
                Transform {
                    model_matrix: Similarity3::default(),
                },
                Parent { entity },
                //PointLight::new(palette::named::RED.into_format()),
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
                        camera.projection_matrix.set_aspect(aspect);
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
            <ul class=Style::debug_overlay>
                {move || {
                    debug_info.with(|debug_info| {
                        view!{
                            <li>"FPS: " {format!("{:.2}", debug_info.fps)}</li>
                            <li>"Frame time: " {format!("{:.2} ms", debug_info.frame_time)}</li>
                            <li>"Pipeline: " {format!("{:?}", debug_info.which)}</li>
                            <li>"Resources:"
                                <ul>
                                    <li>"Buffers: " {debug_info.resources.buffers} " - " {human_size(debug_info.resources.buffer_memory)}</li>
                                    <li>"Textures: " {debug_info.resources.textures} " - " {human_size(debug_info.resources.texture_memory)}</li>
                                </ul>
                            </li>
                        }
                    })
                }}
                <li>
                    "Tone map: "
                    <select
                        on:change=move |event| {
                            let value = event_target_value(&event);
                            let value = match value.as_str() {
                                "Aces" => ToneMap::Aces,
                                "Reinard" => ToneMap::Reinard,
                                "Exposure" => ToneMap::Exposure { exposure: 1.0 },
                                _ => return,
                            };
                            debug_config.tone_map.set(value);
                        }
                    >
                        {
                            ToneMap::VARIANTS.into_iter().map(move |name| {
                                let name = *name;
                                view!{
                                    <option
                                        value=name
                                        selected={move || {
                                            let current: &'static str = debug_config.tone_map.get().into();
                                            current == name
                                        }}
                                    >
                                        {name}
                                    </option>
                                }
                            }).collect::<Vec<_>>()
                        }
                    </select>
                    {move || {
                        match debug_config.tone_map.get() {
                            ToneMap::Exposure { exposure } => {
                                view!{
                                    <ul>
                                        <li>
                                            "Exposure: "
                                            <input
                                                type="text"
                                                value={exposure}
                                                on:change=move |event| {
                                                    let value = event_target_value(&event);
                                                    let Ok(value) = value.parse() else { return; };
                                                    debug_config.tone_map.set(ToneMap::Exposure { exposure: value });
                                                }
                                            />
                                        </li>
                                    </ul>
                                }.into_view()
                            }
                            _ => ().into_view(),
                        }
                    }}
                </li>
                <li>
                    "Gamma: "
                    <input
                        type="text"
                        value={move || debug_config.gamma.get()}
                        on:change=move |event| {
                            let value = event_target_value(&event);
                            let Ok(value) = value.parse() else { return; };
                            debug_config.gamma.set(value);
                        }
                    />
                </li>
            </ul>
        </div>
    }
}

#[derive(Clone, Copy, Debug)]
struct DebugConfig {
    pipeline: RwSignal<Pipeline>,
    tone_map: RwSignal<ToneMap>,
    gamma: RwSignal<f32>,
}

#[derive(Clone, Debug)]
struct CreateWorldViewPipeline {
    debug_config: DebugConfig,
}

impl CreatePipeline for CreateWorldViewPipeline {
    type Pipeline = WorldViewPipeline;
    type InputConfig = ();
    type OutputConfig = TextureConfig;

    fn create_pipeline(
        self,
        context: &CreatePipelineContext,
        input_config: &mut Self::InputConfig,
        output_config: &mut Self::OutputConfig,
    ) -> Self::Pipeline {
        WorldViewPipeline {
            debug_config: self.debug_config,
            hdr: CreateHdrPipeline::new(CreateSwitchedPipeline {
                debug_config: self.debug_config,
            })
            .with_tone_map(self.debug_config.tone_map.get_untracked())
            .with_gamma(self.debug_config.gamma.get_untracked())
            .create_pipeline(context, input_config, output_config),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
enum Pipeline {
    #[default]
    ForwardBlinnPhong,
    DeferredBlinnPhong {
        debug_channel: Option<Channel>,
    },
}

struct WorldViewPipeline {
    debug_config: DebugConfig,
    hdr: HdrPipeline<SwitchedPipeline>,
}

impl RenderPipeline for WorldViewPipeline {
    type Input<'a> = RenderWorldInput<'a>;
    type Output<'a> = TextureOutput<'a>;

    fn render(
        &mut self,
        context: &mut RenderPipelineContext,
        input: Self::Input<'_>,
        output: Self::Output<'_>,
    ) {
        self.hdr
            .set_tone_map(self.debug_config.tone_map.get_untracked());
        self.hdr.set_gamma(self.debug_config.gamma.get_untracked());
        self.hdr.render(context, input, output);
    }
}

#[derive(Debug)]
struct CreateSwitchedPipeline {
    debug_config: DebugConfig,
}

impl CreatePipeline for CreateSwitchedPipeline {
    type Pipeline = SwitchedPipeline;
    type InputConfig = ();
    type OutputConfig = TextureConfig;

    fn create_pipeline(
        self,
        context: &CreatePipelineContext,
        input_config: &mut Self::InputConfig,
        output_config: &mut Self::OutputConfig,
    ) -> Self::Pipeline {
        SwitchedPipeline {
            debug_config: self.debug_config,
            forward_blinn_phong: CreateBlinnPhongRenderPipeline.create_pipeline(
                context,
                input_config,
                output_config,
            ),
            deferred: CreateDeferredPipeline.create_pipeline(context, input_config, output_config),
        }
    }
}

#[derive(Debug)]
struct SwitchedPipeline {
    debug_config: DebugConfig,
    forward_blinn_phong: BlinnPhongRenderPipeline,
    deferred: DeferredPipeline,
}

impl RenderPipeline for SwitchedPipeline {
    type Input<'a> = RenderWorldInput<'a>;
    type Output<'a> = TextureOutput<'a>;

    fn render(
        &mut self,
        context: &mut RenderPipelineContext,
        input: Self::Input<'_>,
        output: Self::Output<'_>,
    ) {
        match self.debug_config.pipeline.get_untracked() {
            Pipeline::DeferredBlinnPhong { debug_channel } => {
                self.deferred.set_debug(debug_channel);
                self.deferred.render(context, input, output);
            }
            Pipeline::ForwardBlinnPhong => {
                self.forward_blinn_phong.render(context, input, output);
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
    debug_config: DebugConfig,
}

fn world_view_camera_controller_system(system_context: &mut SystemContext) {
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
                                camera_transform.model_matrix *= Translation3::from(Vector3::new(
                                    world_delta.x,
                                    world_delta.y,
                                    0.0,
                                ));
                            }

                            if controller.state.mouse.buttons.is_down(MouseButton::Right) {
                                let world_delta =
                                    camera_projection.projection_matrix.unproject_point(
                                        &Point3::new(delta.x, -delta.y, controller.z_mouse),
                                    );
                                let yaw = (world_delta.x / controller.z_mouse).asin();
                                let pitch = (world_delta.y / controller.z_mouse).asin();

                                camera_transform.model_matrix.isometry.rotation *=
                                    UnitQuaternion::from_axis_angle(&-Vector3::y_axis(), yaw)
                                        * UnitQuaternion::from_axis_angle(
                                            &Vector3::x_axis(),
                                            pitch,
                                        );
                            }
                        }
                        MouseEvent::Wheel { delta, .. } => {
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
                        _ => {}
                    }
                }
                None => break,
            }
        }
    }
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn register(self, context: RegisterPluginContext) {
        context
            .schedule
            .add_system(world_view_camera_controller_system);
        context.schedule.add_system(update_debug_info);
    }
}

#[derive(Debug, Default)]
struct DebugInfo {
    fps: f32,
    frame_time: f32,
    which: Pipeline,
    resources: ResourceUsages,
}

fn update_debug_info(system_context: &mut SystemContext) {
    if let Some(frame_info) = system_context.resources.get::<RenderFrameInfo>() {
        let debug_info = system_context
            .resources
            .get::<RwSignal<DebugInfo>>()
            .unwrap();
        debug_info.update(|debug_info| {
            debug_info.fps = frame_info.fps().unwrap_or_default();
            debug_info.frame_time = frame_info.frame_time().as_secs_f32() * 1000.0;
            debug_info.resources = ResourceUsages::get();
        });
    }
}
