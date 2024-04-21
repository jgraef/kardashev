use leptos::{
    create_node_ref,
    html::Canvas,
    view,
    IntoView,
};

use crate::{
    app::{
        expect_context,
        Context,
    },
    scene::renderer::WindowEvent,
};

pub fn Window() -> impl IntoView {
    let canvas_node_ref = create_node_ref::<Canvas>();

    canvas_node_ref.on_load(|canvas| {
        let Context { scene_renderer, .. } = expect_context();
        scene_renderer.create_window(canvas, |event| {
            match event {
                WindowEvent::Created { handle } => {
                    // todo: we need to keep this around. when it's dropped the
                    // window is destroyed
                }
            }
        });
    });

    view! {
        <div class="window">
            <canvas node_ref=canvas_node_ref></canvas>
        </div>
    }
}
