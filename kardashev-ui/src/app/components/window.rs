use kardashev_style::style;
use leptos::{
    component,
    create_effect,
    create_node_ref,
    expect_context,
    html::{
        Canvas,
        Div,
    },
    provide_context,
    store_value,
    view,
    IntoView,
    Signal,
    SignalGet,
    SignalGetUntracked,
};
use leptos_use::{
    signal_debounced,
    use_document_visibility,
    use_element_size_with_options,
    use_element_visibility,
    UseElementSizeOptions,
};
use web_sys::{
    ResizeObserverBoxOptions,
    VisibilityState,
};

use crate::{
    error::Error,
    graphics::{
        Graphics,
        Surface,
        SurfaceSize,
        WindowHandle,
    },
    input::mouse::MouseEvent,
    utils::spawn_local_and_handle_error,
};

#[style(path = "src/app/components/window.scss")]
struct Style;

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
pub fn Window<OnLoad, OnEvent>(on_load: OnLoad, on_event: OnEvent) -> impl IntoView
where
    OnLoad: FnOnce(&Surface) + 'static,
    OnEvent: FnMut(WindowEvent) + 'static,
{
    let container_node_ref = create_node_ref::<Div>();
    let canvas_node_ref = create_node_ref::<Canvas>();

    let container_size = use_element_size_with_options(
        container_node_ref,
        UseElementSizeOptions::default().box_(ResizeObserverBoxOptions::ContentBox),
    );
    let container_size = signal_debounced(
        Signal::derive(move || {
            SurfaceSize {
                width: (container_size.width.get() as u32).max(1),
                height: (container_size.height.get() as u32).max(1),
            }
        }),
        500.,
    );

    let window_handle = WindowHandle::new();
    let surface_handle = store_value(None);

    canvas_node_ref.on_load(move |_canvas| {
        tracing::debug!("window loaded");

        spawn_local_and_handle_error(async move {
            let graphics = expect_context::<Graphics>();
            let surface = graphics
                .create_surface(window_handle, container_size.get_untracked())
                .await?;

            on_load(&surface);

            surface_handle.set_value(Some(surface));

            Ok::<(), Error>(())
        });
    });

    let on_event = store_value(on_event);
    let on_event = move |event| {
        on_event.update_value(|on_event| {
            on_event(event);
        });
    };

    create_effect(move |_| {
        let surface_size = container_size.get();
        tracing::debug!(?surface_size, "container resized");

        surface_handle.update_value(|surface| {
            if let Some(surface) = surface {
                surface.resize(surface_size);
            }
        });

        on_event(WindowEvent::Resize { surface_size })
    });

    let on_mouse_input = move |event: Option<MouseEvent>| {
        if let Some(event) = event {
            on_event(WindowEvent::Mouse(event));
        }
    };

    let element_visibility = use_element_visibility(container_node_ref);
    let document_visibility = use_document_visibility();
    let is_visible = Signal::derive(move || {
        element_visibility.get() && document_visibility.get() == VisibilityState::Visible
    });
    create_effect(move |old_value| {
        let new_value = is_visible.get();

        if old_value.map_or(true, |v| v != new_value) {
            surface_handle.update_value(|surface| {
                if let Some(surface) = surface {
                    surface.set_visible(new_value);
                }
            });

            on_event(WindowEvent::Visibility { visible: new_value });
        }

        new_value
    });

    view! {
        <div
            node_ref=container_node_ref
            class=Style::window
        >
            <canvas
                node_ref=canvas_node_ref
                width=move || container_size.get().width
                height=move || container_size.get().height
                data-raw-handle=window_handle
                on:mouseup=move |event| on_mouse_input(MouseEvent::from_websys_mouse_up(&event))
                on:mousedown=move |event| on_mouse_input(MouseEvent::from_websys_mouse_down(&event))
                on:mousemove=move |event| on_mouse_input(MouseEvent::from_websys_mouse_move(&event))
                on:mouseenter=move |event| on_mouse_input(MouseEvent::from_websys_mouse_enter(&event))
                on:mouseleave=move |event| on_mouse_input(MouseEvent::from_websys_mouse_leave(&event))
                on:wheel=move |event| on_mouse_input(MouseEvent::from_websys_wheel(&event))
            ></canvas>
        </div>
    }
}

#[derive(Clone, Debug)]
pub enum WindowEvent {
    Mouse(MouseEvent),
    Resize { surface_size: SurfaceSize },
    Visibility { visible: bool },
}
