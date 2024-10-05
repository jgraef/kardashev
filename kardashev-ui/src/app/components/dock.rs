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

crate::style!("src/app/components/dock.scss");

#[component]
pub fn Item<H: ToHref + 'static>(
    href: H,
    #[prop(into)] icon: String,
    #[prop(into)] label: Oco<'static, str>,
) -> impl IntoView {
    view! {
        <li class=Style::ITEM>
            <A href={href} active_class="active" class=Style::LINK>
                <BootstrapIcon icon=icon alt=label />
            </A>
        </li>
    }
}

#[component]
pub fn Dock() -> impl IntoView {
    view! {
        <nav class=Style::DOCK>
            <ul class=Style::GROUP_TOP>
                <Item href="/dashboard" icon="speedometer" label="Dashboard" />
                <Item href="/map" icon="radar" label="Map" />
            </ul>
            <ul class=Style::GROUP_BOTTOM>
                <Item href="/settings" icon="gear" label="Settings" />
            </ul>
        </nav>
    }
}
