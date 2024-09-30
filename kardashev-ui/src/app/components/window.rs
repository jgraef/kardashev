use leptos::{
    component,
    create_effect,
    create_node_ref,
    create_signal,
    expect_context,
    html::{
        Canvas,
        Div,
    },
    provide_context,
    view,
    IntoView,
    Signal,
    SignalGet,
    SignalSet,
    SignalUpdate,
};
use leptos_use::{
    signal_debounced,
    use_element_size_with_options,
    UseElementSizeOptions,
};
use web_sys::ResizeObserverBoxOptions;

use crate::{
    error::Error,
    graphics::{
        Graphics,
        Surface,
        SurfaceSize,
        WindowHandle,
    },
    utils::spawn_local_and_handle_error,
};

stylance::import_crate_style!(style, "src/app/components/window.module.scss");

pub fn provide_graphics() {
    tracing::debug!("creating renderer");
    let graphics = Graphics::new(Default::default());
    provide_context(graphics);
}

/// A window (i.e. a HTML canvas) to which a scene is rendered.
/// This creates a container (div) that can be sized using CSS. The canvas will
/// atomatically be resized to fill this container.
///
/// # TODO
///
/// - Add event handler property
#[component]
pub fn Window(on_load: impl FnOnce(&Surface) + 'static) -> impl IntoView {
    let graphics = expect_context::<Graphics>();

    let container_node_ref = create_node_ref::<Div>();
    let canvas_node_ref = create_node_ref::<Canvas>();

    let container_size = use_element_size_with_options(
        container_node_ref,
        UseElementSizeOptions::default().box_(ResizeObserverBoxOptions::ContentBox),
    );
    let container_size = signal_debounced(
        Signal::derive(move || {
            SurfaceSize {
                width: container_size.width.get() as u32,
                height: container_size.height.get() as u32,
            }
        }),
        500.,
    );

    let window_handle = WindowHandle::new();
    let (_surface, set_surface) = create_signal(None);

    canvas_node_ref.on_load(move |canvas| {
        let surface_size = SurfaceSize::from_html_canvas(&canvas);
        spawn_local_and_handle_error(async move {
            let surface = graphics.create_surface(window_handle, surface_size).await?;
            on_load(&surface);
            set_surface.set(Some(surface));

            Ok::<(), Error>(())
        })
    });

    create_effect(move |_| {
        set_surface.update(|surface| {
            if let Some(surface) = surface {
                surface.resize(container_size.get());
            }
        });
    });

    view! {
        <div
            node_ref=container_node_ref
            class=style::window
        >
            <canvas
                node_ref=canvas_node_ref
                width=move || container_size.get().width
                height=move || container_size.get().height
                data-raw-handle=window_handle
            ></canvas>
        </div>
    }
}
