pub mod keyboard;
pub mod mouse;

use std::collections::HashSet;

use nalgebra::{
    Point2,
    Vector3,
};

use self::{
    keyboard::{
        KeyCode,
        KeyboardEvent,
        KeyboardInput,
    },
    mouse::{
        MouseButton,
        MouseEvent,
    },
};
use crate::world::plugin::{
    Plugin,
    RegisterPluginContext,
};

#[derive(Clone, Debug)]
pub enum InputEvent {
    Mouse(MouseEvent),
    Keyboard(KeyboardEvent),
}

fn mouse_position_from_websys(event: &web_sys::MouseEvent) -> Point2<f32> {
    Point2::new(event.offset_x() as f32, event.offset_y() as f32)
}

#[derive(Clone, Debug, Default)]
pub struct InputState {
    pub keys_pressed: HashSet<KeyCode>,
    pub mouse_buttons_pressed: HashSet<MouseButton>,
    pub mouse_position: Option<Point2<f32>>,
    pub absolute_scroll: Vector3<f32>,
}

impl InputState {
    pub fn push(&mut self, event: &InputEvent) {
        match event {
            InputEvent::Mouse(MouseEvent::ButtonUp { button, .. }) => {
                self.mouse_buttons_pressed.remove(button);
            }
            InputEvent::Mouse(MouseEvent::ButtonDown { button, .. }) => {
                self.mouse_buttons_pressed.insert(*button);
            }
            InputEvent::Mouse(MouseEvent::Move { position }) => {
                self.mouse_position = Some(*position);
            }
            InputEvent::Mouse(MouseEvent::Enter) => {}
            InputEvent::Mouse(MouseEvent::Leave) => {
                self.mouse_position = None;
            }
            InputEvent::Mouse(MouseEvent::Wheel { delta, .. }) => {
                self.absolute_scroll += delta;
            }
            InputEvent::Keyboard(KeyboardEvent::KeyUp { code, .. }) => {
                self.keys_pressed.remove(code);
            }
            InputEvent::Keyboard(KeyboardEvent::KeyDown { code, .. }) => {
                self.keys_pressed.insert(*code);
            }
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
