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
    resource::{
        Resources,
        Tick,
    },
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

impl Label {
    pub fn new(label: impl ToString) -> Self {
        Self {
            label: label.to_string().into(),
        }
    }

    pub fn new_static(label: &'static str) -> Self {
        Self {
            label: label.into(),
        }
    }
}
