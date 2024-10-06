use kardashev_style::style;
use leptos::{
    component,
    view,
    IntoView,
    MaybeSignal,
    Oco,
    SignalGet,
};

#[style(path = "src/app/components/icon.scss")]
struct Style;

#[component]
pub fn BootstrapIcon(
    #[prop(into)] icon: MaybeSignal<String>,
    #[prop(into, optional)] alt: Option<Oco<'static, str>>,
) -> impl IntoView {
    view! { <i class={move || format!("bi bi-{}", icon.get())} aria-label=alt></i> }
}

#[component]
pub fn KardashevIcon() -> impl IntoView {
    view! {
        <img src="/assets/kardashev.svg" class=Style::kardashev_icon />
    }
}
