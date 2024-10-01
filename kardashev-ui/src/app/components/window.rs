use std::{
    fmt::Display,
    str::FromStr,
};

use bitflags::bitflags;
use leptos::{
    component,
    create_effect,
    create_node_ref,
    expect_context,
    html::{
        Canvas,
        Div,
    },
    provide_context,
    store_value,
    view,
    IntoView,
    Signal,
    SignalGet,
    SignalGetUntracked,
};
use leptos_use::{
    signal_debounced,
    use_element_size_with_options,
    UseElementSizeOptions,
};
use nalgebra::{
    Point2,
    Vector3,
};
use web_sys::{
    KeyboardEvent,
    MouseEvent,
    ResizeObserverBoxOptions,
    WheelEvent,
};

use crate::{
    error::Error,
    graphics::{
        Graphics,
        Surface,
        SurfaceSize,
        WindowHandle,
    },
    utils::spawn_local_and_handle_error,
};

stylance::import_crate_style!(style, "src/app/components/window.module.scss");

pub fn provide_graphics() {
    tracing::debug!("creating renderer");
    let graphics = Graphics::new(Default::default());
    provide_context(graphics);
}

/// A window (i.e. a HTML canvas) to which a scene is rendered.
/// This creates a container (div) that can be sized using CSS. The canvas will
/// atomatically be resized to fill this container.
///
/// # TODO
///
/// - Add event handler property
#[component]
pub fn Window<OnLoad, OnResize, OnInput>(
    on_load: OnLoad,
    on_resize: OnResize,
    on_input: OnInput,
) -> impl IntoView
where
    OnLoad: FnOnce(&Surface) + 'static,
    OnResize: FnMut(SurfaceSize) + 'static,
    OnInput: FnMut(InputEvent) + 'static,
{
    let container_node_ref = create_node_ref::<Div>();
    let canvas_node_ref = create_node_ref::<Canvas>();

    let container_size = use_element_size_with_options(
        container_node_ref,
        UseElementSizeOptions::default().box_(ResizeObserverBoxOptions::ContentBox),
    );
    let container_size = signal_debounced(
        Signal::derive(move || {
            SurfaceSize {
                width: (container_size.width.get() as u32).max(1),
                height: (container_size.height.get() as u32).max(1),
            }
        }),
        500.,
    );

    let window_handle = WindowHandle::new();
    let surface_handle = store_value(None);

    canvas_node_ref.on_load(move |_canvas| {
        tracing::debug!("window loaded");

        spawn_local_and_handle_error(async move {
            let graphics = expect_context::<Graphics>();
            let surface = graphics
                .create_surface(window_handle, container_size.get_untracked())
                .await?;

            on_load(&surface);

            surface_handle.set_value(Some(surface));

            Ok::<(), Error>(())
        });
    });

    let on_resize = store_value(on_resize);
    create_effect(move |_| {
        let size = container_size.get();
        tracing::debug!(?size, "container resized");

        surface_handle.update_value(|surface| {
            if let Some(surface) = surface {
                surface.resize(size);
            }
        });

        on_resize.update_value(|on_resize| {
            on_resize(size);
        });
    });

    let on_input = store_value(on_input);
    let on_input = move |event: Option<InputEvent>| {
        if let Some(event) = event {
            on_input.update_value(|on_input| on_input(event));
        }
    };

    view! {
        <div
            node_ref=container_node_ref
            class=style::window
        >
            <canvas
                node_ref=canvas_node_ref
                width=move || container_size.get().width
                height=move || container_size.get().height
                data-raw-handle=window_handle
                on:mouseup=move |event| on_input(InputEvent::from_websys_mouse_up(event))
                on:mousedown=move |event| on_input(InputEvent::from_websys_mouse_down(event))
                on:mousemove=move |event| on_input(InputEvent::from_websys_mouse_move(event))
                on:mouseenter=move |event| on_input(InputEvent::from_websys_mouse_enter(event))
                on:mouseleave=move |event| on_input(InputEvent::from_websys_mouse_leave(event))
                on:wheel=move |event| on_input(InputEvent::from_websys_wheel(event))
                on:keyup=move |event| on_input(InputEvent::from_websys_key_up(event))
                on:keydown=move |event| on_input(InputEvent::from_websys_key_down(event))
            ></canvas>
        </div>
    }
}

#[derive(Clone, Debug)]
pub enum InputEvent {
    MouseUp {
        button: MouseButton,
        position: Point2<f32>,
    },
    MouseDown {
        button: MouseButton,
        position: Point2<f32>,
    },
    MouseMove {
        position: Point2<f32>,
    },
    MouseEnter,
    MouseLeave,
    Wheel {
        delta: Vector3<f32>,
        mode: WheelDeltaMode,
    },
    KeyUp {
        code: KeyCode,
        repeat: bool,
        modifiers: KeyModifiers,
    },
    KeyDown {
        code: KeyCode,
        repeat: bool,
        modifiers: KeyModifiers,
    },
}

impl InputEvent {
    fn from_websys_mouse_up(event: MouseEvent) -> Option<Self> {
        Some(Self::MouseUp {
            button: MouseButton::from_websys(event.button())?,
            position: Point2::new(event.client_x() as f32, event.client_y() as f32),
        })
    }

    fn from_websys_mouse_down(event: MouseEvent) -> Option<Self> {
        Some(Self::MouseDown {
            button: MouseButton::from_websys(event.button())?,
            position: Point2::new(event.client_x() as f32, event.client_y() as f32),
        })
    }

    fn from_websys_mouse_move(event: MouseEvent) -> Option<Self> {
        Some(Self::MouseMove {
            position: Point2::new(event.client_x() as f32, event.client_y() as f32),
        })
    }

    fn from_websys_mouse_enter(_event: MouseEvent) -> Option<Self> {
        Some(Self::MouseEnter)
    }

    fn from_websys_mouse_leave(_event: MouseEvent) -> Option<Self> {
        Some(Self::MouseLeave)
    }

    fn from_websys_wheel(event: WheelEvent) -> Option<Self> {
        Some(Self::Wheel {
            delta: Vector3::new(
                event.delta_x() as f32,
                event.delta_y() as f32,
                event.delta_z() as f32,
            ),
            mode: WheelDeltaMode::from_websys(event.delta_mode())?,
        })
    }

    fn from_websys_key_up(event: KeyboardEvent) -> Option<Self> {
        Some(Self::KeyUp {
            code: KeyCode::from_websys(&event.code())?,
            repeat: event.repeat(),
            modifiers: KeyModifiers::from_websys(&event),
        })
    }

    fn from_websys_key_down(event: KeyboardEvent) -> Option<Self> {
        Some(Self::KeyDown {
            code: KeyCode::from_websys(&event.code())?,
            repeat: event.repeat(),
            modifiers: KeyModifiers::from_websys(&event),
        })
    }
}

#[derive(Clone, Copy, Debug)]
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

macro_rules! key_codes {
    {$($key:ident,)*} => {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum KeyCode {
            $($key,)*
        }

        impl KeyCode {
            fn from_websys(code: &str) -> Option<Self> {
                match code {
                    $(
                        stringify!($key) => Some(Self::$key),
                    )*
                    _ => None,
                }
            }

            fn to_websys(&self) -> &'static str {
                match self {
                    $(
                        Self::$key => stringify!($key),
                    )*
                }
            }
        }
    };
}

key_codes! {
    Again,
    AltLeft,
    AltRight,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    Backquote,
    Backslash,
    Backspace,
    BracketLeft,
    BracketRight,
    BrowserBack,
    BrowserFavorites,
    BrowserForward,
    BrowserHome,
    BrowserRefresh,
    BrowserSearch,
    BrowserStop,
    CapsLock,
    Comma,
    ContextMenu,
    ControlLeft,
    ControlRight,
    Convert,
    Copy,
    Cut,
    Delete,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Eject,
    End,
    Enter,
    Equal,
    Escape,
    F1,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F2,
    F20,
    F21,
    F22,
    F23,
    F24,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    Find,
    Help,
    Home,
    Insert,
    IntlBackslash,
    IntlRo,
    IntlYen,
    KanaMode,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
    Lang1,
    Lang2,
    LaunchApp1,
    LaunchApp2,
    LaunchMail,
    MediaPlayPause,
    MediaSelect,
    MediaStop,
    MediaTrackNext,
    MediaTrackPrevious,
    MetaLeft,
    MetaRight,
    Minus,
    NonConvert,
    NumLock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadComma,
    NumpadDecimal,
    NumpadDivide,
    NumpadEnter,
    NumpadEqual,
    NumpadMultiply,
    NumpadSubtract,
    Open,
    PageDown,
    PageUp,
    Paste,
    Pause,
    Period,
    PrintScreen,
    Props,
    Quote,
    ScrollLock,
    Select,
    Semicolon,
    ShiftLeft,
    ShiftRight,
    Slash,
    Space,
    Tab,
    Undo,
    VolumeDown,
    VolumeMute,
    VolumeUp,
    WakeUp,
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid keycode: {0}")]
pub struct KeyCodeParseError(String);

impl FromStr for KeyCode {
    type Err = KeyCodeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        KeyCode::from_websys(s).ok_or_else(|| KeyCodeParseError(s.to_owned()))
    }
}

impl Display for KeyCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_websys())
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Default)]
    pub struct KeyModifiers: u8 {
        const ALT   = 0b00000010;
        const CTRL  = 0b00000001;
        const META  = 0b00000100;
        const SHIFT = 0b00001000;
    }
}

impl KeyModifiers {
    pub fn from_websys(event: &KeyboardEvent) -> Self {
        let mut mods = KeyModifiers::default();
        if event.alt_key() {
            mods |= KeyModifiers::ALT;
        }
        if event.ctrl_key() {
            mods |= KeyModifiers::CTRL;
        }
        if event.meta_key() {
            mods |= KeyModifiers::META;
        }
        if event.shift_key() {
            mods |= KeyModifiers::SHIFT;
        }
        mods
    }
}
