use leptos::{
    component,
    view,
    IntoView,
};

#[component]
pub fn MainPage() -> impl IntoView {
    view! {
        <canvas id="canvas_main" width="100%" height="100%"></canvas>
    }
}
