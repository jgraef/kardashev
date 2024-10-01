use std::{
    fmt::Display,
    str::FromStr,
};

use bitflags::bitflags;
use leptos_use::{
    use_event_listener,
    use_window,
};
use tokio::sync::broadcast;

#[derive(Debug)]
pub struct KeyboardInput {
    rx: broadcast::Receiver<KeyboardEvent>,
}

impl Clone for KeyboardInput {
    fn clone(&self) -> Self {
        Self {
            rx: self.rx.resubscribe(),
        }
    }
}

impl KeyboardInput {
    pub fn install() -> Self {
        let (tx_up, rx) = broadcast::channel(128);
        let tx_down = tx_up.clone();

        let _ = use_event_listener(use_window(), leptos::ev::keyup, move |event| {
            if let Some(event) = KeyboardEvent::from_websys_key_up(&event) {
                tracing::debug!(?event);
                let _ = tx_up.send(event);
            }
        });

        let _ = use_event_listener(use_window(), leptos::ev::keydown, move |event| {
            if let Some(event) = KeyboardEvent::from_websys_key_down(&event) {
                tracing::debug!(?event);
                let _ = tx_down.send(event);
            }
        });

        KeyboardInput { rx }
    }

    pub async fn next(&mut self) -> KeyboardEvent {
        self.rx.recv().await.unwrap()
    }

    pub fn try_next(&mut self) -> Option<KeyboardEvent> {
        self.rx.try_recv().ok()
    }
}

#[derive(Clone, Debug)]
pub enum KeyboardEvent {
    KeyUp {
        code: KeyCode,
        modifiers: KeyModifiers,
    },
    KeyDown {
        code: KeyCode,
        repeat: bool,
        modifiers: KeyModifiers,
    },
}

impl KeyboardEvent {
    fn from_websys_key_up(event: &web_sys::KeyboardEvent) -> Option<Self> {
        Some(Self::KeyUp {
            code: KeyCode::from_websys(&event.code())?,
            modifiers: KeyModifiers::from_websys(event),
        })
    }

    fn from_websys_key_down(event: &web_sys::KeyboardEvent) -> Option<Self> {
        Some(Self::KeyDown {
            code: KeyCode::from_websys(&event.code())?,
            repeat: event.repeat(),
            modifiers: KeyModifiers::from_websys(event),
        })
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
    pub fn from_websys(event: &web_sys::KeyboardEvent) -> Self {
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
