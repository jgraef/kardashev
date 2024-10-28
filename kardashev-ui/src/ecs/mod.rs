pub mod plugin;
pub mod resource;
pub mod schedule;
pub mod server;
pub mod system;

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

#[derive(Clone, Debug)]
pub struct Dirty<T> {
    value: T,
    dirty: bool,
}

impl<T> Dirty<T> {
    pub fn new(value: T, dirty: bool) -> Self {
        Self { value, dirty }
    }

    pub fn new_clean(value: T) -> Self {
        Self::new(value, false)
    }

    pub fn new_dirty(value: T) -> Self {
        Self::new(value, true)
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}

impl<T> AsRef<T> for Dirty<T> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<T> AsMut<T> for Dirty<T> {
    fn as_mut(&mut self) -> &mut T {
        self.dirty = true;
        &mut self.value
    }
}
