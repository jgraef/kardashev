use leptos::{
    component,
    view,
    IntoView,
    MaybeSignal,
    Oco,
    SignalGet,
};

stylance::import_crate_style!(style, "src/app/components/icon.module.scss");

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
        <img src="/assets/kardashev.svg" class=style::kardashev_icon />
    }
}
