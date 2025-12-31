use std::sync::{Arc, Mutex};

use crate::{
    CursorIcon, Icon, LogicalSize, Monitor, PhysicalPositionF64, PhysicalPositionI32, PhysicalSizeU32, ProgressBarState,
    TaoError, Theme, VideoMode, WindowSizeConstraints,
};

#[derive(Clone)]
struct SendableWindowBuilder(tao::window::WindowBuilder);

// Safety: the builder only stores configuration data and raw handles; it is only
// used to build windows on the event-loop thread.
unsafe impl Send for SendableWindowBuilder {}

#[derive(Clone, uniffi::Enum)]
pub enum Fullscreen {
    Borderless { monitor: Option<Arc<Monitor>> },
    Exclusive { video_mode: Arc<VideoMode> },
}

impl Fullscreen {
    pub(crate) fn to_tao(&self) -> tao::window::Fullscreen {
        match self {
            Fullscreen::Borderless { monitor } => tao::window::Fullscreen::Borderless(
                monitor.as_ref().map(|m| m.inner.clone()),
            ),
            Fullscreen::Exclusive { video_mode } => {
                let inner = video_mode.inner.lock().unwrap();
                tao::window::Fullscreen::Exclusive(inner.clone())
            }
        }
    }
}

fn fullscreen_from_tao(fullscreen: tao::window::Fullscreen) -> Fullscreen {
    match fullscreen {
        tao::window::Fullscreen::Borderless(monitor) => Fullscreen::Borderless {
            monitor: monitor.map(|m| Arc::new(Monitor { inner: m })),
        },
        tao::window::Fullscreen::Exclusive(video_mode) => Fullscreen::Exclusive {
            video_mode: Arc::new(VideoMode {
                inner: Mutex::new(video_mode),
            }),
        },
        _ => Fullscreen::Borderless { monitor: None },
    }
}

#[derive(uniffi::Object)]
pub struct WindowBuilder {
    inner: Mutex<SendableWindowBuilder>,
}

#[uniffi::export]
impl WindowBuilder {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(SendableWindowBuilder(tao::window::WindowBuilder::new())),
        }
    }

    pub fn set_title(&self, title: String) {
        let mut inner = self.inner.lock().unwrap();
        inner.0 = inner.0.clone().with_title(title);
    }

    pub fn set_inner_size(&self, size: LogicalSize) {
        let mut inner = self.inner.lock().unwrap();
        let size: tao::dpi::Size = tao::dpi::LogicalSize::new(size.width, size.height).into();
        inner.0 = inner.0.clone().with_inner_size(size);
    }

    pub fn set_min_inner_size(&self, size: LogicalSize) {
        let mut inner = self.inner.lock().unwrap();
        let size: tao::dpi::Size = tao::dpi::LogicalSize::new(size.width, size.height).into();
        inner.0 = inner.0.clone().with_min_inner_size(size);
    }

    pub fn set_decorations(&self, decorations: bool) {
        let mut inner = self.inner.lock().unwrap();
        inner.0 = inner.0.clone().with_decorations(decorations);
    }

    pub fn set_resizable(&self, resizable: bool) {
        let mut inner = self.inner.lock().unwrap();
        inner.0 = inner.0.clone().with_resizable(resizable);
    }

    pub fn set_transparent(&self, transparent: bool) {
        let mut inner = self.inner.lock().unwrap();
        inner.0 = inner.0.clone().with_transparent(transparent);
    }

    pub fn set_fullscreen(&self, fullscreen: Option<Fullscreen>) {
        let mut inner = self.inner.lock().unwrap();
        inner.0 = inner
            .0
            .clone()
            .with_fullscreen(fullscreen.as_ref().map(|f| f.to_tao()));
    }

    pub fn set_window_icon(&self, icon: Option<Arc<Icon>>) {
        let mut inner = self.inner.lock().unwrap();
        inner.0 = inner
            .0
            .clone()
            .with_window_icon(icon.as_ref().map(|i| i.inner.clone()));
    }

    pub fn set_theme(&self, theme: Option<Theme>) {
        let mut inner = self.inner.lock().unwrap();
        inner.0 = inner
            .0
            .clone()
            .with_theme(theme.map(|t| tao::window::Theme::from(t)));
    }

    pub fn set_parent_window(&self, parent: Arc<Window>) -> Result<(), TaoError> {
        let parent_window = parent.inner.lock().unwrap();

        #[cfg(target_os = "macos")]
        {
            use tao::platform::macos::{WindowBuilderExtMacOS, WindowExtMacOS};
            let mut inner = self.inner.lock().unwrap();
            let ns_window = parent_window.ns_window();
            inner.0 = inner.0.clone().with_parent_window(ns_window);
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        {
            use tao::platform::windows::{WindowBuilderExtWindows, WindowExtWindows};
            let mut inner = self.inner.lock().unwrap();
            let hwnd = parent_window.hwnd();
            inner.0 = inner.0.clone().with_parent_window(hwnd);
            return Ok(());
        }

        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            use tao::platform::unix::{WindowBuilderExtUnix, WindowExtUnix};
            let mut inner = self.inner.lock().unwrap();
            let gtk_window = parent_window.gtk_window();
            inner.0 = inner.0.clone().with_transient_for(gtk_window);
            return Ok(());
        }

        #[allow(unreachable_code)]
        Err(TaoError::Unsupported)
    }
}

impl WindowBuilder {
    pub(crate) fn clone_inner(&self) -> tao::window::WindowBuilder {
        self.inner.lock().unwrap().0.clone()
    }
}

#[derive(uniffi::Object)]
pub struct Window {
    pub(crate) id: u64,
    pub(crate) inner: Mutex<tao::window::Window>,
}

#[uniffi::export]
impl Window {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn ns_view_handle(&self) -> Result<u64, TaoError> {
        #[cfg(target_os = "macos")]
        {
            use tao::platform::macos::WindowExtMacOS;
            let window = self.inner.lock().unwrap();
            return Ok(window.ns_view() as u64);
        }
        #[cfg(not(target_os = "macos"))]
        {
            return Err(TaoError::Unsupported);
        }
    }

    pub fn ns_window_handle(&self) -> Result<u64, TaoError> {
        #[cfg(target_os = "macos")]
        {
            use tao::platform::macos::WindowExtMacOS;
            let window = self.inner.lock().unwrap();
            return Ok(window.ns_window() as u64);
        }
        #[cfg(not(target_os = "macos"))]
        {
            return Err(TaoError::Unsupported);
        }
    }

    /// Returns the HWND handle (Windows only).
    pub fn hwnd_handle(&self) -> Result<u64, TaoError> {
        #[cfg(target_os = "windows")]
        {
            use tao::platform::windows::WindowExtWindows;
            let window = self.inner.lock().unwrap();
            return Ok(window.hwnd() as u64);
        }
        #[cfg(not(target_os = "windows"))]
        {
            return Err(TaoError::Unsupported);
        }
    }

    /// Returns the HINSTANCE handle (Windows only).
    pub fn hinstance_handle(&self) -> Result<u64, TaoError> {
        #[cfg(target_os = "windows")]
        {
            use tao::platform::windows::WindowExtWindows;
            let window = self.inner.lock().unwrap();
            return Ok(window.hinstance() as u64);
        }
        #[cfg(not(target_os = "windows"))]
        {
            return Err(TaoError::Unsupported);
        }
    }

    /// Returns the X11 window ID (Linux X11 only).
    pub fn xlib_window_handle(&self) -> Result<u64, TaoError> {
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            use tao::platform::unix::WindowExtUnix;
            let window = self.inner.lock().unwrap();
            if let Some(xlib_window) = window.xlib_window() {
                return Ok(xlib_window as u64);
            }
            return Err(TaoError::Unsupported);
        }
        #[cfg(not(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )))]
        {
            return Err(TaoError::Unsupported);
        }
    }

    /// Returns the X11 display pointer (Linux X11 only).
    pub fn xlib_display_handle(&self) -> Result<u64, TaoError> {
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            use tao::platform::unix::WindowExtUnix;
            let window = self.inner.lock().unwrap();
            if let Some(xlib_display) = window.xlib_display() {
                return Ok(xlib_display as u64);
            }
            return Err(TaoError::Unsupported);
        }
        #[cfg(not(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )))]
        {
            return Err(TaoError::Unsupported);
        }
    }

    /// Returns the X11 screen ID (Linux X11 only).
    pub fn xlib_screen_id(&self) -> Result<i32, TaoError> {
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            use tao::platform::unix::WindowExtUnix;
            let window = self.inner.lock().unwrap();
            if let Some(screen_id) = window.xlib_screen_id() {
                return Ok(screen_id);
            }
            return Err(TaoError::Unsupported);
        }
        #[cfg(not(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )))]
        {
            return Err(TaoError::Unsupported);
        }
    }

    /// Returns the Wayland surface pointer (Linux Wayland only).
    pub fn wayland_surface_handle(&self) -> Result<u64, TaoError> {
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            use tao::platform::unix::WindowExtUnix;
            let window = self.inner.lock().unwrap();
            if let Some(wayland_surface) = window.wayland_surface() {
                return Ok(wayland_surface as u64);
            }
            return Err(TaoError::Unsupported);
        }
        #[cfg(not(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )))]
        {
            return Err(TaoError::Unsupported);
        }
    }

    /// Returns the Wayland display pointer (Linux Wayland only).
    pub fn wayland_display_handle(&self) -> Result<u64, TaoError> {
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            use tao::platform::unix::WindowExtUnix;
            let window = self.inner.lock().unwrap();
            if let Some(wayland_display) = window.wayland_display() {
                return Ok(wayland_display as u64);
            }
            return Err(TaoError::Unsupported);
        }
        #[cfg(not(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )))]
        {
            return Err(TaoError::Unsupported);
        }
    }

    /// Returns the raw window handle for graphics operations.
    pub fn raw_window_handle(&self) -> Result<crate::RawWindowHandle, TaoError> {
        use crate::graphics::WindowGraphicsExt;
        WindowGraphicsExt::raw_window_handle(self)
    }

    /// Returns the raw window handle for a specific graphics backend.
    pub fn raw_window_handle_for_backend(
        &self,
        backend: crate::GraphicsBackend,
    ) -> Result<crate::RawWindowHandle, TaoError> {
        use crate::graphics::WindowGraphicsExt;
        WindowGraphicsExt::raw_window_handle_for_backend(self, backend)
    }

    pub fn request_redraw(&self) {
        let window = self.inner.lock().unwrap();
        window.request_redraw();
    }

    pub fn set_title(&self, title: String) {
        let window = self.inner.lock().unwrap();
        window.set_title(&title);
    }

    pub fn scale_factor(&self) -> f64 {
        let window = self.inner.lock().unwrap();
        window.scale_factor()
    }

    pub fn set_cursor_icon(&self, icon: CursorIcon) {
        let window = self.inner.lock().unwrap();
        window.set_cursor_icon(icon.into());
    }

    pub fn set_cursor_grab(&self, grab: bool) -> Result<(), TaoError> {
        let window = self.inner.lock().unwrap();
        window.set_cursor_grab(grab)?;
        Ok(())
    }

    pub fn set_cursor_visible(&self, visible: bool) {
        let window = self.inner.lock().unwrap();
        window.set_cursor_visible(visible);
    }

    pub fn set_decorations(&self, decorations: bool) {
        let window = self.inner.lock().unwrap();
        window.set_decorations(decorations);
    }

    pub fn set_resizable(&self, resizable: bool) {
        let window = self.inner.lock().unwrap();
        window.set_resizable(resizable);
    }

    pub fn set_minimized(&self, minimized: bool) {
        let window = self.inner.lock().unwrap();
        window.set_minimized(minimized);
    }

    pub fn is_minimized(&self) -> bool {
        let window = self.inner.lock().unwrap();
        window.is_minimized()
    }

    pub fn set_focus(&self) {
        let window = self.inner.lock().unwrap();
        window.set_focus();
    }

    pub fn set_visible(&self, visible: bool) {
        let window = self.inner.lock().unwrap();
        window.set_visible(visible);
    }

    pub fn set_always_on_top(&self, always_on_top: bool) {
        let window = self.inner.lock().unwrap();
        window.set_always_on_top(always_on_top);
    }

    pub fn set_always_on_bottom(&self, always_on_bottom: bool) {
        let window = self.inner.lock().unwrap();
        window.set_always_on_bottom(always_on_bottom);
    }

    pub fn set_content_protection(&self, enabled: bool) {
        let window = self.inner.lock().unwrap();
        window.set_content_protection(enabled);
    }

    pub fn is_minimizable(&self) -> bool {
        let window = self.inner.lock().unwrap();
        window.is_minimizable()
    }

    pub fn set_minimizable(&self, minimizable: bool) {
        let window = self.inner.lock().unwrap();
        window.set_minimizable(minimizable);
    }

    pub fn is_maximizable(&self) -> bool {
        let window = self.inner.lock().unwrap();
        window.is_maximizable()
    }

    pub fn set_maximizable(&self, maximizable: bool) {
        let window = self.inner.lock().unwrap();
        window.set_maximizable(maximizable);
    }

    pub fn is_closable(&self) -> bool {
        let window = self.inner.lock().unwrap();
        window.is_closable()
    }

    pub fn set_closable(&self, closable: bool) {
        let window = self.inner.lock().unwrap();
        window.set_closable(closable);
    }

    pub fn is_maximized(&self) -> bool {
        let window = self.inner.lock().unwrap();
        window.is_maximized()
    }

    pub fn set_maximized(&self, maximized: bool) {
        let window = self.inner.lock().unwrap();
        window.set_maximized(maximized);
    }

    pub fn inner_size(&self) -> PhysicalSizeU32 {
        let window = self.inner.lock().unwrap();
        window.inner_size().into()
    }

    pub fn outer_size(&self) -> PhysicalSizeU32 {
        let window = self.inner.lock().unwrap();
        window.outer_size().into()
    }

    pub fn outer_position(&self) -> Result<PhysicalPositionI32, TaoError> {
        let window = self.inner.lock().unwrap();
        Ok(window.outer_position()?.into())
    }

    pub fn inner_position(&self) -> Result<PhysicalPositionI32, TaoError> {
        let window = self.inner.lock().unwrap();
        Ok(window.inner_position()?.into())
    }

    pub fn set_outer_position(&self, position: PhysicalPositionI32) {
        let window = self.inner.lock().unwrap();
        let position: tao::dpi::Position =
            tao::dpi::PhysicalPosition::new(position.x, position.y).into();
        window.set_outer_position(position);
    }

    pub fn set_inner_size(&self, size: PhysicalSizeU32) {
        let window = self.inner.lock().unwrap();
        let size: tao::dpi::Size = tao::dpi::PhysicalSize::new(size.width, size.height).into();
        window.set_inner_size(size);
    }

    pub fn set_min_inner_size(&self, size: Option<PhysicalSizeU32>) {
        let window = self.inner.lock().unwrap();
        let size = size.map(|s| tao::dpi::Size::from(tao::dpi::PhysicalSize::new(s.width, s.height)));
        window.set_min_inner_size(size);
    }

    pub fn set_max_inner_size(&self, size: Option<PhysicalSizeU32>) {
        let window = self.inner.lock().unwrap();
        let size = size.map(|s| tao::dpi::Size::from(tao::dpi::PhysicalSize::new(s.width, s.height)));
        window.set_max_inner_size(size);
    }

    pub fn set_cursor_position(&self, position: PhysicalPositionI32) -> Result<(), TaoError> {
        let window = self.inner.lock().unwrap();
        let position: tao::dpi::Position =
            tao::dpi::PhysicalPosition::new(position.x, position.y).into();
        window.set_cursor_position(position)?;
        Ok(())
    }

    pub fn set_ime_position(&self, position: PhysicalPositionF64) {
        let window = self.inner.lock().unwrap();
        let position: tao::dpi::Position =
            tao::dpi::PhysicalPosition::new(position.x, position.y).into();
        window.set_ime_position(position);
    }

    pub fn drag_window(&self) -> Result<(), TaoError> {
        let window = self.inner.lock().unwrap();
        window.drag_window()?;
        Ok(())
    }

    pub fn fullscreen(&self) -> Option<Fullscreen> {
        let window = self.inner.lock().unwrap();
        window.fullscreen().map(fullscreen_from_tao)
    }

    pub fn set_fullscreen(&self, fullscreen: Option<Fullscreen>) {
        let window = self.inner.lock().unwrap();
        window.set_fullscreen(fullscreen.as_ref().map(|f| f.to_tao()));
    }

    pub fn set_window_icon(&self, icon: Option<Arc<Icon>>) {
        let window = self.inner.lock().unwrap();
        window.set_window_icon(icon.as_ref().map(|i| i.inner.clone()));
    }

    pub fn current_monitor(&self) -> Option<Arc<Monitor>> {
        let window = self.inner.lock().unwrap();
        window.current_monitor().map(|m| Arc::new(Monitor { inner: m }))
    }

    pub fn primary_monitor(&self) -> Option<Arc<Monitor>> {
        let window = self.inner.lock().unwrap();
        window.primary_monitor().map(|m| Arc::new(Monitor { inner: m }))
    }

    pub fn available_monitors(&self) -> Vec<Arc<Monitor>> {
        let window = self.inner.lock().unwrap();
        window
            .available_monitors()
            .map(|m| Arc::new(Monitor { inner: m }))
            .collect()
    }

    pub fn set_progress_bar(&self, state: ProgressBarState) {
        let window = self.inner.lock().unwrap();
        window.set_progress_bar(state.into());
    }

    pub fn set_inner_size_constraints(&self, constraints: WindowSizeConstraints) {
        let window = self.inner.lock().unwrap();
        window.set_inner_size_constraints(constraints.into());
    }

    pub fn theme(&self) -> Theme {
        let window = self.inner.lock().unwrap();
        window.theme().into()
    }

    pub fn set_theme(&self, theme: Option<Theme>) {
        let window = self.inner.lock().unwrap();
        window.set_theme(theme.map(|t| t.into()));
    }

    pub fn set_overlay_icon(&self, icon: Option<Arc<Icon>>) {
        #[cfg(windows)]
        {
            use tao::platform::windows::WindowExtWindows;
            let window = self.inner.lock().unwrap();
            window.set_overlay_icon(icon.as_ref().map(|i| &i.inner));
        }

        #[cfg(not(windows))]
        let _ = icon;
    }

    pub fn set_badge_count(&self, count: Option<i64>) {
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            use tao::platform::unix::WindowExtUnix;
            let window = self.inner.lock().unwrap();
            window.set_badge_count(count, None);
        }

        #[cfg(target_os = "ios")]
        {
            use tao::platform::ios::WindowExtIOS;
            let window = self.inner.lock().unwrap();
            window.set_badge_count(count.unwrap_or(0) as usize);
        }
    }

    pub fn set_badge_label(&self, label: Option<String>) {
        #[cfg(target_os = "macos")]
        {
            use tao::platform::macos::WindowExtMacOS;
            let window = self.inner.lock().unwrap();
            window.set_badge_label(label);
        }

        #[cfg(not(target_os = "macos"))]
        let _ = label;
    }

    pub fn debug_string(&self) -> String {
        format!("Window(id={})", self.id)
    }
}
