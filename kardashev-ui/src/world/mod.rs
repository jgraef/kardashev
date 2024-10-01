mod plugin;
mod resource;
mod schedule;
mod server;
mod system;

use std::borrow::Cow;

pub use self::{
    plugin::{
        Plugin,
        RegisterPluginContext,
    },
    resource::Resources,
    server::{
        Builder,
        World,
    },
    system::{
        NullSystem,
        OneshotSystem,
        RunSystemContext,
        System,
    },
};

#[derive(Clone, Debug)]
pub struct Label {
    pub label: Cow<'static, str>,
}
