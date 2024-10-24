pub mod keyboard;
pub mod mouse;

use self::{
    keyboard::{
        KeyboardEvent,
        KeyboardInput,
    },
    mouse::MouseEvent,
};
use crate::{
    ecs::plugin::{
        Plugin,
        RegisterPluginContext,
    },
    input::{
        keyboard::KeyboardInputState,
        mouse::MouseInputState,
    },
};

#[derive(Clone, Debug)]
pub enum InputEvent {
    Mouse(MouseEvent),
    Keyboard(KeyboardEvent),
}

#[derive(Clone, Debug, Default)]
pub struct InputState {
    pub keyboard: KeyboardInputState,
    pub mouse: MouseInputState,
}

impl InputState {
    pub fn push(&mut self, event: &InputEvent) {
        match event {
            InputEvent::Keyboard(event) => self.keyboard.push(event),
            InputEvent::Mouse(event) => self.mouse.push(event),
        }
    }
}

#[derive(Debug)]
pub struct InputPlugin {
    pub keyboard_input: KeyboardInput,
}

impl Default for InputPlugin {
    fn default() -> Self {
        Self {
            keyboard_input: KeyboardInput::install(),
        }
    }
}

impl Plugin for InputPlugin {
    fn register(self, context: RegisterPluginContext) {
        context.resources.insert(self.keyboard_input);
    }
}
