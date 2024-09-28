use std::ops::Deref;

use leptos::{
    component,
    create_node_ref,
    html::{
        Canvas,
        Div,
    },
    spawn_local,
    view,
    IntoView,
    StoredValue,
};
use leptos_use::{
    signal_debounced,
    use_element_size_with_options,
    UseElementSizeOptions,
};
use web_sys::ResizeObserverBoxOptions;

use crate::{
    app::Context,
    error::Error,
    graphics::{
        rendering_system::RenderTarget,
        SurfaceSize,
        WindowHandle,
    },
    utils::spawn_local_and_handle_error,
    world::World,
};

stylance::import_crate_style!(style, "src/app/components/window.module.scss");

/// A window (i.e. a HTML canvas) to which a scene is rendered.
/// This creates a container (div) that can be sized using CSS. The canvas will
/// atomatically be resized to fill this container.
///
/// # TODO
///
/// - Add event handler property
#[component]
pub fn Window(on_load: impl FnOnce(RenderTarget) + 'static) -> impl IntoView {
    let Context { graphics, .. } = Context::get();

    let container_node_ref = create_node_ref::<Div>();
    let canvas_node_ref = create_node_ref::<Canvas>();

    let window_handle = WindowHandle::new();

    canvas_node_ref.on_load(move |canvas| {
        let surface_size = SurfaceSize::from_html_canvas(&canvas);
        spawn_local_and_handle_error(async move {
            let surface = graphics.create_surface(window_handle, surface_size).await?;
            let render_target = RenderTarget::from_surface(&surface);
            on_load(render_target);
            Ok::<(), Error>(())
        })
    });

    let container_size = use_element_size_with_options(
        container_node_ref,
        UseElementSizeOptions::default().box_(ResizeObserverBoxOptions::ContentBox),
    );
    let width = signal_debounced(container_size.width, 1000.);
    let height = signal_debounced(container_size.height, 1000.);

    view! {
        <div
            node_ref=container_node_ref
            class=style::window
        >
            <canvas
                node_ref=canvas_node_ref
                width=width
                height=height
                data-raw-handle=window_handle
            ></canvas>
        </div>
    }
}
