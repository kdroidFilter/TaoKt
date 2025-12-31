//! Graphics backend abstraction for multi-platform rendering.
//!
//! This module provides platform-agnostic window handle types that can be used
//! by rendering backends (Metal, Vulkan, DirectX12, OpenGL).

use crate::TaoError;
use std::sync::Arc;

/// Supported graphics backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GraphicsBackend {
    /// Metal (macOS, iOS)
    Metal,
    /// Vulkan (Linux, Windows, Android)
    Vulkan,
    /// DirectX 12 (Windows)
    DirectX12,
    /// OpenGL (cross-platform fallback)
    OpenGL,
}

impl GraphicsBackend {
    /// Returns the default backend for the current platform.
    pub fn default_for_platform() -> Self {
        #[cfg(target_os = "macos")]
        return Self::Metal;

        #[cfg(target_os = "ios")]
        return Self::Metal;

        #[cfg(target_os = "windows")]
        return Self::DirectX12;

        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        return Self::Vulkan;

        #[cfg(target_os = "android")]
        return Self::Vulkan;

        #[allow(unreachable_code)]
        Self::OpenGL
    }

    /// Returns true if this backend is supported on the current platform.
    pub fn is_supported(&self) -> bool {
        match self {
            Self::Metal => cfg!(any(target_os = "macos", target_os = "ios")),
            Self::Vulkan => cfg!(any(
                target_os = "linux",
                target_os = "windows",
                target_os = "android",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            )),
            Self::DirectX12 => cfg!(target_os = "windows"),
            Self::OpenGL => true, // OpenGL is available everywhere as fallback
        }
    }
}

/// Raw window handle containing platform-specific window information.
///
/// This struct provides all the necessary handles for creating graphics surfaces
/// on any supported platform.
#[derive(Debug, Clone, uniffi::Record)]
pub struct RawWindowHandle {
    /// The recommended graphics backend for this window.
    pub backend: GraphicsBackend,

    // macOS / iOS handles
    /// NSView pointer (macOS) - used for Metal layer attachment.
    pub ns_view: Option<u64>,
    /// NSWindow pointer (macOS) - used for window-level operations.
    pub ns_window: Option<u64>,
    /// UIView pointer (iOS) - used for Metal layer attachment.
    pub ui_view: Option<u64>,
    /// UIWindow pointer (iOS).
    pub ui_window: Option<u64>,

    // Windows handles
    /// HWND handle (Windows) - the window handle.
    pub hwnd: Option<u64>,
    /// HINSTANCE handle (Windows) - the application instance.
    pub hinstance: Option<u64>,

    // Linux X11 handles
    /// X11 Window ID (Linux X11).
    pub xlib_window: Option<u64>,
    /// X11 Display pointer (Linux X11).
    pub xlib_display: Option<u64>,
    /// X11 Visual ID (Linux X11) - for OpenGL context creation.
    pub xlib_visual_id: Option<u64>,
    /// X11 Screen number (Linux X11).
    pub xlib_screen: Option<i32>,

    // Linux Wayland handles
    /// Wayland surface pointer (Linux Wayland).
    pub wayland_surface: Option<u64>,
    /// Wayland display pointer (Linux Wayland).
    pub wayland_display: Option<u64>,

    // Android handles
    /// ANativeWindow pointer (Android).
    pub android_native_window: Option<u64>,

    // Common properties
    /// Window width in physical pixels.
    pub width: u32,
    /// Window height in physical pixels.
    pub height: u32,
    /// Scale factor (DPI scaling).
    pub scale_factor: f64,
}

impl RawWindowHandle {
    /// Creates an empty handle with default values.
    pub fn empty() -> Self {
        Self {
            backend: GraphicsBackend::default_for_platform(),
            ns_view: None,
            ns_window: None,
            ui_view: None,
            ui_window: None,
            hwnd: None,
            hinstance: None,
            xlib_window: None,
            xlib_display: None,
            xlib_visual_id: None,
            xlib_screen: None,
            wayland_surface: None,
            wayland_display: None,
            android_native_window: None,
            width: 0,
            height: 0,
            scale_factor: 1.0,
        }
    }

    /// Returns true if this handle has valid platform-specific data.
    pub fn is_valid(&self) -> bool {
        match self.backend {
            GraphicsBackend::Metal => self.ns_view.is_some() || self.ui_view.is_some(),
            GraphicsBackend::DirectX12 => self.hwnd.is_some(),
            GraphicsBackend::Vulkan => {
                self.hwnd.is_some()
                    || self.xlib_window.is_some()
                    || self.wayland_surface.is_some()
                    || self.android_native_window.is_some()
            }
            GraphicsBackend::OpenGL => {
                self.ns_view.is_some()
                    || self.hwnd.is_some()
                    || self.xlib_window.is_some()
                    || self.wayland_surface.is_some()
            }
        }
    }
}

/// Extension trait for Window to get graphics handles.
pub trait WindowGraphicsExt {
    /// Gets the raw window handle for graphics operations.
    fn raw_window_handle(&self) -> Result<RawWindowHandle, TaoError>;

    /// Gets the raw window handle for a specific backend.
    fn raw_window_handle_for_backend(
        &self,
        backend: GraphicsBackend,
    ) -> Result<RawWindowHandle, TaoError>;
}

impl WindowGraphicsExt for crate::Window {
    fn raw_window_handle(&self) -> Result<RawWindowHandle, TaoError> {
        self.raw_window_handle_for_backend(GraphicsBackend::default_for_platform())
    }

    fn raw_window_handle_for_backend(
        &self,
        backend: GraphicsBackend,
    ) -> Result<RawWindowHandle, TaoError> {
        let window = self.inner.lock().unwrap();
        let size = window.inner_size();
        let scale = window.scale_factor();

        let mut handle = RawWindowHandle {
            backend,
            ns_view: None,
            ns_window: None,
            ui_view: None,
            ui_window: None,
            hwnd: None,
            hinstance: None,
            xlib_window: None,
            xlib_display: None,
            xlib_visual_id: None,
            xlib_screen: None,
            wayland_surface: None,
            wayland_display: None,
            android_native_window: None,
            width: size.width,
            height: size.height,
            scale_factor: scale,
        };

        #[cfg(target_os = "macos")]
        {
            use tao::platform::macos::WindowExtMacOS;
            handle.ns_view = Some(window.ns_view() as u64);
            handle.ns_window = Some(window.ns_window() as u64);
        }

        #[cfg(target_os = "ios")]
        {
            use tao::platform::ios::WindowExtIOS;
            handle.ui_view = Some(window.ui_view() as u64);
            handle.ui_window = Some(window.ui_window() as u64);
        }

        #[cfg(target_os = "windows")]
        {
            use tao::platform::windows::WindowExtWindows;
            handle.hwnd = Some(window.hwnd() as u64);
            handle.hinstance = Some(window.hinstance() as u64);
        }

        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            use tao::platform::unix::WindowExtUnix;

            // Try X11 first
            if let Some(xlib_window) = window.xlib_window() {
                handle.xlib_window = Some(xlib_window as u64);
            }
            if let Some(xlib_display) = window.xlib_display() {
                handle.xlib_display = Some(xlib_display as u64);
            }
            if let Some(xlib_screen) = window.xlib_screen_id() {
                handle.xlib_screen = Some(xlib_screen);
            }

            // Try Wayland
            if let Some(wayland_surface) = window.wayland_surface() {
                handle.wayland_surface = Some(wayland_surface as u64);
            }
            if let Some(wayland_display) = window.wayland_display() {
                handle.wayland_display = Some(wayland_display as u64);
            }
        }

        #[cfg(target_os = "android")]
        {
            use tao::platform::android::WindowExtAndroid;
            // Note: ANativeWindow access may require additional setup
        }

        if !handle.is_valid() {
            return Err(TaoError::Unsupported);
        }

        Ok(handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphics_backend_default() {
        let backend = GraphicsBackend::default_for_platform();
        assert!(backend.is_supported());
    }

    #[test]
    fn test_raw_window_handle_empty() {
        let handle = RawWindowHandle::empty();
        assert!(!handle.is_valid());
    }
}
