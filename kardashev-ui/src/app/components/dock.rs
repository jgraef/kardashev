use kardashev_style::style;
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

#[style(path = "src/app/components/dock.scss")]
struct Style;

#[component]
pub fn Item<H: ToHref + 'static>(
    href: H,
    #[prop(into)] icon: String,
    #[prop(into)] label: Oco<'static, str>,
) -> impl IntoView {
    view! {
        <li class=Style::item>
            <A href={href} active_class="active" class=Style::link>
                <BootstrapIcon icon=icon alt=label />
            </A>
        </li>
    }
}

#[component]
pub fn Dock() -> impl IntoView {
    view! {
        <nav class=Style::dock>
            <ul class=Style::group_top>
                <Item href="/dashboard" icon="speedometer" label="Dashboard" />
                <Item href="/map" icon="radar" label="Map" />
            </ul>
            <ul class=Style::group_bottom>
                <Item href="/settings" icon="gear" label="Settings" />
            </ul>
        </nav>
    }
}
