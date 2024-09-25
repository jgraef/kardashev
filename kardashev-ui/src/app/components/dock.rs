use leptos::{
    component,
    view,
    IntoView,
    Oco,
};
use leptos_router::{
    ToHref,
    A,
};

use super::icon::BootstrapIcon;

stylance::import_crate_style!(style, "src/app/components/dock.module.scss");

#[component]
pub fn Item<H: ToHref + 'static>(
    href: H,
    #[prop(into)] icon: String,
    #[prop(into)] label: Oco<'static, str>,
) -> impl IntoView {
    view! {
        <li class=style::item>
            <A href={href} active_class="active" class=style::link>
                <BootstrapIcon icon=icon alt=label />
            </A>
        </li>
    }
}

#[component]
pub fn Dock() -> impl IntoView {
    view! {
        <nav class=style::dock>
            <ul class=style::group_top>
                <Item href="/dashboard" icon="speedometer" label="Dashboard" />
                <Item href="/map" icon="radar" label="Map" />
            </ul>
            <ul class=style::group_bottom>
                <Item href="/settings" icon="gear" label="Settings" />
            </ul>
        </nav>
    }
}
