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
    graphics::{
        renderer::RenderPlugin,
        window::WindowHandler,
    },
};

stylance::import_crate_style!(style, "src/app/components/window.module.scss");

/// A window (i.e. a HTML canvas) to which a scene is rendered.
/// This creates a container (div) that can be sized using CSS. The canvas will
/// atomatically be resized to fill this container.
///
/// # TODO
///
/// - Make sure the window is destroyed when the component is disposed.
/// - Add event handler property
#[component]
pub fn Window(handler: impl WindowHandler, render_plugin: impl RenderPlugin) -> impl IntoView {
    let Context { renderer, .. } = Context::get();

    let container_node_ref = create_node_ref::<Div>();
    let canvas_node_ref = create_node_ref::<Canvas>();
    let window_handle = StoredValue::new(None);

    canvas_node_ref.on_load(move |canvas| {
        spawn_local(async move {
            let window = renderer
                .create_window(
                    canvas.deref().clone(),
                    Box::new(handler),
                    Box::new(render_plugin),
                )
                .await;
            window_handle.set_value(Some(window));
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
            ></canvas>
        </div>
    }
}
