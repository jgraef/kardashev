use std::fmt::Debug;

use nalgebra::{
    Point2,
    Vector2,
    Vector3,
};

#[derive(Clone, Debug)]
pub enum MouseEvent {
    ButtonUp {
        button: MouseButton,
        position: Point2<f32>,
    },
    ButtonDown {
        button: MouseButton,
        position: Point2<f32>,
    },
    Move {
        position: Point2<f32>,
        delta: Vector2<f32>,
    },
    Enter,
    Leave,
    Wheel {
        delta: Vector3<f32>,
        mode: WheelDeltaMode,
    },
}

impl MouseEvent {
    pub(crate) fn from_websys_mouse_up(event: &web_sys::MouseEvent) -> Option<Self> {
        Some(Self::ButtonUp {
            button: MouseButton::from_websys(event.button())?,
            position: mouse_position_from_websys(event),
        })
    }

    pub(crate) fn from_websys_mouse_down(event: &web_sys::MouseEvent) -> Option<Self> {
        Some(Self::ButtonDown {
            button: MouseButton::from_websys(event.button())?,
            position: mouse_position_from_websys(event),
        })
    }

    pub(crate) fn from_websys_mouse_move(event: &web_sys::MouseEvent) -> Option<Self> {
        Some(Self::Move {
            position: mouse_position_from_websys(event),
            delta: mouse_delta_from_websys(event),
        })
    }

    pub(crate) fn from_websys_mouse_enter(_event: &web_sys::MouseEvent) -> Option<Self> {
        Some(Self::Enter)
    }

    pub(crate) fn from_websys_mouse_leave(_event: &web_sys::MouseEvent) -> Option<Self> {
        Some(Self::Leave)
    }

    pub(crate) fn from_websys_wheel(event: &web_sys::WheelEvent) -> Option<Self> {
        Some(Self::Wheel {
            delta: Vector3::new(
                event.delta_x() as f32,
                event.delta_y() as f32,
                event.delta_z() as f32,
            ),
            mode: WheelDeltaMode::from_websys(event.delta_mode())?,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Back,
    Forward,
}

impl MouseButton {
    const BUTTONS: [MouseButton; 5] = [
        Self::Left,
        Self::Middle,
        Self::Right,
        Self::Back,
        Self::Forward,
    ];

    fn from_websys(button: i16) -> Option<Self> {
        match button {
            0 => Some(Self::Left),
            1 => Some(Self::Middle),
            2 => Some(Self::Right),
            3 => Some(Self::Back),
            4 => Some(Self::Forward),
            _ => None,
        }
    }

    const fn bitmask(&self) -> u16 {
        match self {
            MouseButton::Left => 0x0001,
            MouseButton::Middle => 0x0002,
            MouseButton::Right => 0x0004,
            MouseButton::Back => 0x0008,
            MouseButton::Forward => 0x0010,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum WheelDeltaMode {
    Pixel,
    Line,
    Page,
}

impl WheelDeltaMode {
    fn from_websys(mode: u32) -> Option<Self> {
        match mode {
            0x00 => Some(Self::Pixel),
            0x01 => Some(Self::Line),
            0x02 => Some(Self::Page),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct MouseInputState {
    pub buttons: MouseButtonState,
    pub position: Option<Point2<f32>>,
    pub absolute_scroll: Vector3<f32>,
}

impl MouseInputState {
    pub fn push(&mut self, event: &MouseEvent) {
        match event {
            MouseEvent::ButtonUp { button, .. } => {
                self.buttons.set_up(*button);
            }
            MouseEvent::ButtonDown { button, .. } => {
                self.buttons.set_down(*button);
            }
            MouseEvent::Move { position, .. } => {
                self.position = Some(*position);
            }
            MouseEvent::Enter => {}
            MouseEvent::Leave => {
                self.position = None;
            }
            MouseEvent::Wheel { delta, .. } => {
                self.absolute_scroll += delta;
            }
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct MouseButtonState {
    state: u16,
}

impl MouseButtonState {
    pub fn set_up(&mut self, button: MouseButton) {
        self.state &= !button.bitmask();
    }

    pub fn set_down(&mut self, button: MouseButton) {
        self.state |= button.bitmask();
    }

    pub fn is_down(&self, button: MouseButton) -> bool {
        self.state & button.bitmask() != 0
    }

    pub fn down_iter(&self) -> MouseButtonStateIter {
        MouseButtonStateIter {
            state: *self,
            buttons: MouseButton::BUTTONS.iter(),
            down: true,
        }
    }

    pub fn up_iter(&self) -> MouseButtonStateIter {
        MouseButtonStateIter {
            state: *self,
            buttons: MouseButton::BUTTONS.iter(),
            down: false,
        }
    }
}

impl Debug for MouseButtonState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.down_iter()).finish()
    }
}

#[derive(Clone, Debug)]
pub struct MouseButtonStateIter {
    state: MouseButtonState,
    buttons: std::slice::Iter<'static, MouseButton>,
    down: bool,
}

impl Iterator for MouseButtonStateIter {
    type Item = MouseButton;

    fn next(&mut self) -> Option<Self::Item> {
        let button = *self.buttons.next()?;
        (self.state.is_down(button) == self.down).then_some(button)
    }
}

fn mouse_position_from_websys(event: &web_sys::MouseEvent) -> Point2<f32> {
    Point2::new(event.offset_x() as f32, event.offset_y() as f32)
}

fn mouse_delta_from_websys(event: &web_sys::MouseEvent) -> Vector2<f32> {
    Vector2::new(event.movement_x() as f32, event.movement_y() as f32)
}
