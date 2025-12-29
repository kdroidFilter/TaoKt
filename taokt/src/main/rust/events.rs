use std::path::PathBuf;

use crate::{
    ElementState, Key, KeyCode, ModifiersState, MouseButton, MouseScrollDelta, PhysicalPositionF64,
    PhysicalPositionI32, TaoError, Theme,
};

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum TaoUserEvent {
    Timer,
    Message { value: String },
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum TaoStartCause {
    Init,
    Poll,
    WaitCancelled,
    ResumeTimeReached,
    Other { value: String },
}

impl From<tao::event::StartCause> for TaoStartCause {
    fn from(value: tao::event::StartCause) -> Self {
        match value {
            tao::event::StartCause::Init => TaoStartCause::Init,
            tao::event::StartCause::Poll => TaoStartCause::Poll,
            tao::event::StartCause::WaitCancelled { .. } => TaoStartCause::WaitCancelled,
            tao::event::StartCause::ResumeTimeReached { .. } => TaoStartCause::ResumeTimeReached,
            other => TaoStartCause::Other {
                value: format!("{other:?}"),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct RawKeyEvent {
    pub physical_key: KeyCode,
    pub state: ElementState,
}

impl From<tao::event::RawKeyEvent> for RawKeyEvent {
    fn from(value: tao::event::RawKeyEvent) -> Self {
        Self {
            physical_key: value.physical_key.into(),
            state: value.state.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct KeyEvent {
    pub physical_key: KeyCode,
    pub logical_key: Key,
    pub state: ElementState,
}

impl From<tao::event::KeyEvent> for KeyEvent {
    fn from(value: tao::event::KeyEvent) -> Self {
        Self {
            physical_key: value.physical_key.into(),
            logical_key: value.logical_key.into(),
            state: value.state.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum TaoWindowEvent {
    CloseRequested,
    Destroyed,
    DroppedFile { path: String },
    KeyboardInput { event: KeyEvent },
    ModifiersChanged { modifiers: ModifiersState },
    CursorMoved { position: PhysicalPositionF64 },
    CursorEntered,
    MouseInput {
        state: ElementState,
        button: MouseButton,
    },
    Moved { position: PhysicalPositionI32 },
    ThemeChanged { theme: Theme },
    Other { value: String },
}

fn path_to_string(path: PathBuf) -> String {
    path.to_string_lossy().to_string()
}

impl From<tao::event::WindowEvent<'_>> for TaoWindowEvent {
    fn from(value: tao::event::WindowEvent<'_>) -> Self {
        use tao::event::WindowEvent as NativeWindowEvent;
        match value {
            NativeWindowEvent::CloseRequested => TaoWindowEvent::CloseRequested,
            NativeWindowEvent::Destroyed => TaoWindowEvent::Destroyed,
            NativeWindowEvent::DroppedFile(path) => TaoWindowEvent::DroppedFile {
                path: path_to_string(path),
            },
            NativeWindowEvent::KeyboardInput { event, .. } => TaoWindowEvent::KeyboardInput {
                event: event.into(),
            },
            NativeWindowEvent::ModifiersChanged(modifiers) => TaoWindowEvent::ModifiersChanged {
                modifiers: modifiers.into(),
            },
            NativeWindowEvent::CursorMoved { position, .. } => TaoWindowEvent::CursorMoved {
                position: position.into(),
            },
            NativeWindowEvent::CursorEntered { .. } => TaoWindowEvent::CursorEntered,
            NativeWindowEvent::MouseInput { state, button, .. } => TaoWindowEvent::MouseInput {
                state: state.into(),
                button: button.into(),
            },
            NativeWindowEvent::Moved(position) => TaoWindowEvent::Moved {
                position: position.into(),
            },
            NativeWindowEvent::ThemeChanged(theme) => TaoWindowEvent::ThemeChanged { theme: theme.into() },
            other => TaoWindowEvent::Other {
                value: format!("{other:?}"),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum TaoDeviceEvent {
    MouseMotion { delta_x: f64, delta_y: f64 },
    MouseWheel { delta: MouseScrollDelta },
    Button { button: u32, state: ElementState },
    Key { event: RawKeyEvent },
    Other { value: String },
}

impl From<tao::event::DeviceEvent> for TaoDeviceEvent {
    fn from(value: tao::event::DeviceEvent) -> Self {
        use tao::event::DeviceEvent as NativeDeviceEvent;
        match value {
            NativeDeviceEvent::MouseMotion { delta, .. } => TaoDeviceEvent::MouseMotion {
                delta_x: delta.0,
                delta_y: delta.1,
            },
            NativeDeviceEvent::MouseWheel { delta, .. } => TaoDeviceEvent::MouseWheel {
                delta: delta.into(),
            },
            NativeDeviceEvent::Button { button, state, .. } => TaoDeviceEvent::Button {
                button,
                state: state.into(),
            },
            NativeDeviceEvent::Key(event) => TaoDeviceEvent::Key { event: event.into() },
            other => TaoDeviceEvent::Other {
                value: format!("{other:?}"),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum TaoEvent {
    NewEvents { cause: TaoStartCause },
    WindowEvent { window_id: u64, event: TaoWindowEvent },
    DeviceEvent { event: TaoDeviceEvent },
    UserEvent { event: TaoUserEvent },
    MainEventsCleared,
    RedrawRequested { window_id: u64 },
    RedrawEventsCleared,
    Reopen { has_visible_windows: bool },
    LoopDestroyed,
    Other { value: String },
}

pub(crate) fn convert_event<F>(
    event: tao::event::Event<'_, TaoUserEvent>,
    map_window_id: F,
) -> TaoEvent
where
    F: Fn(tao::window::WindowId) -> u64,
{
    use tao::event::Event as NativeEvent;
    match event {
        NativeEvent::NewEvents(cause) => TaoEvent::NewEvents {
            cause: cause.into(),
        },
        NativeEvent::WindowEvent {
            window_id, event, ..
        } => TaoEvent::WindowEvent {
            window_id: map_window_id(window_id),
            event: event.into(),
        },
        NativeEvent::DeviceEvent { event, .. } => TaoEvent::DeviceEvent { event: event.into() },
        NativeEvent::UserEvent(event) => TaoEvent::UserEvent { event },
        NativeEvent::MainEventsCleared => TaoEvent::MainEventsCleared,
        NativeEvent::RedrawRequested(window_id) => TaoEvent::RedrawRequested {
            window_id: map_window_id(window_id),
        },
        NativeEvent::RedrawEventsCleared => TaoEvent::RedrawEventsCleared,
        NativeEvent::Reopen {
            has_visible_windows,
            ..
        } => TaoEvent::Reopen { has_visible_windows },
        NativeEvent::LoopDestroyed => TaoEvent::LoopDestroyed,
        other => TaoEvent::Other {
            value: format!("{other:?}"),
        },
    }
}

impl From<uniffi::UnexpectedUniFFICallbackError> for TaoError {
    fn from(value: uniffi::UnexpectedUniFFICallbackError) -> Self {
        TaoError::message(value.reason)
    }
}
