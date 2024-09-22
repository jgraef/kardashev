use leptos::{
    component,
    create_node_ref,
    html::{
        Canvas,
        Div,
    },
    view,
    IntoView,
};
use leptos_use::{
    use_element_size_with_options,
    UseElementSizeOptions,
};
use web_sys::ResizeObserverBoxOptions;

use crate::scene::renderer::SceneView;

/// A window (i.e. a HTML canvas) to which a scene is rendered.
/// This creates a container (div) that can be sized using CSS. The canvas will
/// atomatically be resized to fill this container.
///
/// # TODO
///
/// - Make sure the window is destroyed when the component is disposed.
/// - Add event handler property
#[component]
pub fn Window(#[prop(optional)] scene_view: Option<SceneView>) -> impl IntoView {
    let _ = scene_view;
    //let Context { scene_renderer, .. } = expect_context();

    let container_node_ref = create_node_ref::<Div>();
    let canvas_node_ref = create_node_ref::<Canvas>();
    //let window_handle = StoredValue::new(None);

    canvas_node_ref.on_load(move |_canvas| {
        /*spawn_local(async move {
            let (window, mut events) = scene_renderer
                .create_window(canvas.deref().clone(), scene_view)
                .await;

            window_handle.set_value(Some(window));

            while let Some(event) = events.next().await {
                match event {
                    // todo
                }
            }
        })*/
    });

    let container_size = use_element_size_with_options(
        container_node_ref,
        UseElementSizeOptions::default().box_(ResizeObserverBoxOptions::ContentBox),
    );

    view! {
        <div
            node_ref=container_node_ref
            class="window"
        >
            <canvas
                node_ref=canvas_node_ref
                width=container_size.width
                height=container_size.height
            ></canvas>
        </div>
    }
}
