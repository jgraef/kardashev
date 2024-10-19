pub mod plugin;
pub mod resource;
pub mod schedule;
pub mod server;
pub mod system;
pub mod tick;
pub mod world;

use std::borrow::Cow;

use self::{
    plugin::{
        Plugin,
        RegisterPluginContext,
    },
    resource::Resources,
};
use crate::ecs::system::DynSystemError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("system error: {system}")]
    System {
        system: &'static str,
        #[source]
        error: DynSystemError,
    },
}

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
