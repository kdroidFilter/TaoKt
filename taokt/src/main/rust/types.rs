use std::time::Duration;

use tao::{
    dpi::{LogicalSize as TaoLogicalSize, PhysicalPosition as TaoPhysicalPosition, PhysicalSize as TaoPhysicalSize},
    event::ElementState as TaoElementState,
    event_loop::DeviceEventFilter as TaoDeviceEventFilter,
};

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum TaoError {
    #[error("{details}")]
    Message { details: String },

    #[error("Unsupported on this platform")]
    Unsupported,
}

impl TaoError {
    pub(crate) fn message(message: impl Into<String>) -> Self {
        Self::Message {
            details: message.into(),
        }
    }
}

impl From<std::io::Error> for TaoError {
    fn from(value: std::io::Error) -> Self {
        TaoError::message(value.to_string())
    }
}

impl From<tao::error::ExternalError> for TaoError {
    fn from(value: tao::error::ExternalError) -> Self {
        TaoError::message(value.to_string())
    }
}

impl From<tao::error::NotSupportedError> for TaoError {
    fn from(_value: tao::error::NotSupportedError) -> Self {
        TaoError::Unsupported
    }
}

impl From<tao::window::BadIcon> for TaoError {
    fn from(value: tao::window::BadIcon) -> Self {
        TaoError::message(value.to_string())
    }
}

impl From<tao::error::OsError> for TaoError {
    fn from(value: tao::error::OsError) -> Self {
        TaoError::message(value.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum ElementState {
    Pressed,
    Released,
}

impl From<TaoElementState> for ElementState {
    fn from(value: TaoElementState) -> Self {
        match value {
            TaoElementState::Pressed => Self::Pressed,
            TaoElementState::Released => Self::Released,
            _ => Self::Released,
        }
    }
}

impl From<ElementState> for TaoElementState {
    fn from(value: ElementState) -> Self {
        match value {
            ElementState::Pressed => TaoElementState::Pressed,
            ElementState::Released => TaoElementState::Released,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum DeviceEventFilter {
    Always,
    Unfocused,
    Never,
}

impl From<DeviceEventFilter> for TaoDeviceEventFilter {
    fn from(value: DeviceEventFilter) -> Self {
        match value {
            DeviceEventFilter::Always => TaoDeviceEventFilter::Always,
            DeviceEventFilter::Unfocused => TaoDeviceEventFilter::Unfocused,
            DeviceEventFilter::Never => TaoDeviceEventFilter::Never,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct LogicalSize {
    pub width: f64,
    pub height: f64,
}

impl From<LogicalSize> for TaoLogicalSize<f64> {
    fn from(value: LogicalSize) -> Self {
        TaoLogicalSize::new(value.width, value.height)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct PhysicalSizeU32 {
    pub width: u32,
    pub height: u32,
}

impl From<TaoPhysicalSize<u32>> for PhysicalSizeU32 {
    fn from(value: TaoPhysicalSize<u32>) -> Self {
        Self {
            width: value.width,
            height: value.height,
        }
    }
}

impl From<PhysicalSizeU32> for TaoPhysicalSize<u32> {
    fn from(value: PhysicalSizeU32) -> Self {
        TaoPhysicalSize::new(value.width, value.height)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct PhysicalPositionI32 {
    pub x: i32,
    pub y: i32,
}

impl From<TaoPhysicalPosition<i32>> for PhysicalPositionI32 {
    fn from(value: TaoPhysicalPosition<i32>) -> Self {
        Self { x: value.x, y: value.y }
    }
}

impl From<PhysicalPositionI32> for TaoPhysicalPosition<i32> {
    fn from(value: PhysicalPositionI32) -> Self {
        TaoPhysicalPosition::new(value.x, value.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct PhysicalPositionF64 {
    pub x: f64,
    pub y: f64,
}

impl From<TaoPhysicalPosition<f64>> for PhysicalPositionF64 {
    fn from(value: TaoPhysicalPosition<f64>) -> Self {
        Self { x: value.x, y: value.y }
    }
}

impl From<PhysicalPositionF64> for TaoPhysicalPosition<f64> {
    fn from(value: PhysicalPositionF64) -> Self {
        TaoPhysicalPosition::new(value.x, value.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum ControlFlow {
    /// Keep the current control flow unchanged.
    Keep,
    Wait,
    Poll,
    Exit,
    /// Wait until `now + duration_ms`.
    WaitUntil { duration_ms: u64 },
}

impl ControlFlow {
    pub(crate) fn to_tao(self) -> Option<tao::event_loop::ControlFlow> {
        match self {
            ControlFlow::Keep => None,
            ControlFlow::Wait => Some(tao::event_loop::ControlFlow::Wait),
            ControlFlow::Poll => Some(tao::event_loop::ControlFlow::Poll),
            ControlFlow::Exit => Some(tao::event_loop::ControlFlow::Exit),
            ControlFlow::WaitUntil { duration_ms } => Some(tao::event_loop::ControlFlow::WaitUntil(
                std::time::Instant::now() + Duration::from_millis(duration_ms),
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum Key {
    Escape,
    ArrowLeft,
    ArrowRight,
    Character { value: String },
    Other { value: String },
}

impl From<tao::keyboard::Key<'_>> for Key {
    fn from(value: tao::keyboard::Key<'_>) -> Self {
        use tao::keyboard::Key as TaoKey;
        match value {
            TaoKey::Escape => Key::Escape,
            TaoKey::ArrowLeft => Key::ArrowLeft,
            TaoKey::ArrowRight => Key::ArrowRight,
            TaoKey::Character(s) => Key::Character {
                value: s.to_string(),
            },
            other => Key::Other {
                value: format!("{other:?}"),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum KeyCode {
    Space,
    KeyA,
    KeyD,
    KeyL,
    KeyM,
    KeyV,
    Other { value: String },
}

impl From<tao::keyboard::KeyCode> for KeyCode {
    fn from(value: tao::keyboard::KeyCode) -> Self {
        use tao::keyboard::KeyCode as TaoKeyCode;
        match value {
            TaoKeyCode::Space => KeyCode::Space,
            TaoKeyCode::KeyA => KeyCode::KeyA,
            TaoKeyCode::KeyD => KeyCode::KeyD,
            TaoKeyCode::KeyL => KeyCode::KeyL,
            TaoKeyCode::KeyM => KeyCode::KeyM,
            TaoKeyCode::KeyV => KeyCode::KeyV,
            other => KeyCode::Other {
                value: format!("{other:?}"),
            },
        }
    }
}

impl From<KeyCode> for tao::keyboard::KeyCode {
    fn from(value: KeyCode) -> Self {
        use tao::keyboard::KeyCode as TaoKeyCode;
        match value {
            KeyCode::Space => TaoKeyCode::Space,
            KeyCode::KeyA => TaoKeyCode::KeyA,
            KeyCode::KeyD => TaoKeyCode::KeyD,
            KeyCode::KeyL => TaoKeyCode::KeyL,
            KeyCode::KeyM => TaoKeyCode::KeyM,
            KeyCode::KeyV => TaoKeyCode::KeyV,
            KeyCode::Other { .. } => TaoKeyCode::Unidentified(tao::keyboard::NativeKeyCode::Unidentified),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct ModifiersState {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub super_key: bool,
}

impl From<tao::keyboard::ModifiersState> for ModifiersState {
    fn from(value: tao::keyboard::ModifiersState) -> Self {
        Self {
            shift: value.shift_key(),
            control: value.control_key(),
            alt: value.alt_key(),
            super_key: value.super_key(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other { value: u16 },
}

impl From<tao::event::MouseButton> for MouseButton {
    fn from(value: tao::event::MouseButton) -> Self {
        use tao::event::MouseButton as TaoMouseButton;
        match value {
            TaoMouseButton::Left => MouseButton::Left,
            TaoMouseButton::Right => MouseButton::Right,
            TaoMouseButton::Middle => MouseButton::Middle,
            TaoMouseButton::Other(v) => MouseButton::Other { value: v },
            _ => MouseButton::Other { value: 0 },
        }
    }
}

impl From<MouseButton> for tao::event::MouseButton {
    fn from(value: MouseButton) -> Self {
        match value {
            MouseButton::Left => tao::event::MouseButton::Left,
            MouseButton::Right => tao::event::MouseButton::Right,
            MouseButton::Middle => tao::event::MouseButton::Middle,
            MouseButton::Other { value } => tao::event::MouseButton::Other(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum MouseScrollDelta {
    LineDelta { x: f32, y: f32 },
    PixelDelta { x: f64, y: f64 },
    Other { value: String },
}

impl From<tao::event::MouseScrollDelta> for MouseScrollDelta {
    fn from(value: tao::event::MouseScrollDelta) -> Self {
        match value {
            tao::event::MouseScrollDelta::LineDelta(x, y) => MouseScrollDelta::LineDelta { x, y },
            tao::event::MouseScrollDelta::PixelDelta(p) => MouseScrollDelta::PixelDelta { x: p.x, y: p.y },
            other => MouseScrollDelta::Other {
                value: format!("{other:?}"),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum Theme {
    Light,
    Dark,
}

impl From<tao::window::Theme> for Theme {
    fn from(value: tao::window::Theme) -> Self {
        match value {
            tao::window::Theme::Light => Theme::Light,
            tao::window::Theme::Dark => Theme::Dark,
            _ => Theme::Light,
        }
    }
}

impl From<Theme> for tao::window::Theme {
    fn from(value: Theme) -> Self {
        match value {
            Theme::Light => tao::window::Theme::Light,
            Theme::Dark => tao::window::Theme::Dark,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum CursorIcon {
    Default,
    Crosshair,
    Hand,
    Arrow,
    Move,
    Text,
    Wait,
    Help,
    Progress,
    NotAllowed,
    ContextMenu,
    Cell,
    VerticalText,
    Alias,
    Copy,
    NoDrop,
    Grab,
    Grabbing,
    AllScroll,
    ZoomIn,
    ZoomOut,
    EResize,
    NResize,
    NeResize,
    NwResize,
    SResize,
    SeResize,
    SwResize,
    WResize,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ColResize,
    RowResize,
}

impl From<CursorIcon> for tao::window::CursorIcon {
    fn from(value: CursorIcon) -> Self {
        use tao::window::CursorIcon as TaoCursorIcon;
        match value {
            CursorIcon::Default => TaoCursorIcon::Default,
            CursorIcon::Crosshair => TaoCursorIcon::Crosshair,
            CursorIcon::Hand => TaoCursorIcon::Hand,
            CursorIcon::Arrow => TaoCursorIcon::Arrow,
            CursorIcon::Move => TaoCursorIcon::Move,
            CursorIcon::Text => TaoCursorIcon::Text,
            CursorIcon::Wait => TaoCursorIcon::Wait,
            CursorIcon::Help => TaoCursorIcon::Help,
            CursorIcon::Progress => TaoCursorIcon::Progress,
            CursorIcon::NotAllowed => TaoCursorIcon::NotAllowed,
            CursorIcon::ContextMenu => TaoCursorIcon::ContextMenu,
            CursorIcon::Cell => TaoCursorIcon::Cell,
            CursorIcon::VerticalText => TaoCursorIcon::VerticalText,
            CursorIcon::Alias => TaoCursorIcon::Alias,
            CursorIcon::Copy => TaoCursorIcon::Copy,
            CursorIcon::NoDrop => TaoCursorIcon::NoDrop,
            CursorIcon::Grab => TaoCursorIcon::Grab,
            CursorIcon::Grabbing => TaoCursorIcon::Grabbing,
            CursorIcon::AllScroll => TaoCursorIcon::AllScroll,
            CursorIcon::ZoomIn => TaoCursorIcon::ZoomIn,
            CursorIcon::ZoomOut => TaoCursorIcon::ZoomOut,
            CursorIcon::EResize => TaoCursorIcon::EResize,
            CursorIcon::NResize => TaoCursorIcon::NResize,
            CursorIcon::NeResize => TaoCursorIcon::NeResize,
            CursorIcon::NwResize => TaoCursorIcon::NwResize,
            CursorIcon::SResize => TaoCursorIcon::SResize,
            CursorIcon::SeResize => TaoCursorIcon::SeResize,
            CursorIcon::SwResize => TaoCursorIcon::SwResize,
            CursorIcon::WResize => TaoCursorIcon::WResize,
            CursorIcon::EwResize => TaoCursorIcon::EwResize,
            CursorIcon::NsResize => TaoCursorIcon::NsResize,
            CursorIcon::NeswResize => TaoCursorIcon::NeswResize,
            CursorIcon::NwseResize => TaoCursorIcon::NwseResize,
            CursorIcon::ColResize => TaoCursorIcon::ColResize,
            CursorIcon::RowResize => TaoCursorIcon::RowResize,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum ProgressState {
    None,
    Normal,
    Indeterminate,
    Paused,
    Error,
}

impl From<ProgressState> for tao::window::ProgressState {
    fn from(value: ProgressState) -> Self {
        match value {
            ProgressState::None => tao::window::ProgressState::None,
            ProgressState::Normal => tao::window::ProgressState::Normal,
            ProgressState::Indeterminate => tao::window::ProgressState::Indeterminate,
            ProgressState::Paused => tao::window::ProgressState::Paused,
            ProgressState::Error => tao::window::ProgressState::Error,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct ProgressBarState {
    pub progress: Option<u64>,
    pub state: Option<ProgressState>,
    pub desktop_filename: Option<String>,
}

impl From<ProgressBarState> for tao::window::ProgressBarState {
    fn from(value: ProgressBarState) -> Self {
        tao::window::ProgressBarState {
            progress: value.progress,
            state: value.state.map(|s| s.into()),
            desktop_filename: value.desktop_filename,
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct WindowSizeConstraints {
    pub min_width: Option<f64>,
    pub max_width: Option<f64>,
    pub min_height: Option<f64>,
    pub max_height: Option<f64>,
}

impl From<WindowSizeConstraints> for tao::window::WindowSizeConstraints {
    fn from(value: WindowSizeConstraints) -> Self {
        use tao::dpi::LogicalUnit;
        let mut c = tao::window::WindowSizeConstraints::default();
        c.min_width = value
            .min_width
            .map(|w| LogicalUnit::new(w).into());
        c.max_width = value
            .max_width
            .map(|w| LogicalUnit::new(w).into());
        c.min_height = value
            .min_height
            .map(|h| LogicalUnit::new(h).into());
        c.max_height = value
            .max_height
            .map(|h| LogicalUnit::new(h).into());
        c
    }
}
