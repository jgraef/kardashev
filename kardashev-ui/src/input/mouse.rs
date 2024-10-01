use nalgebra::{
    Point2,
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

fn mouse_position_from_websys(event: &web_sys::MouseEvent) -> Point2<f32> {
    Point2::new(event.offset_x() as f32, event.offset_y() as f32)
}
