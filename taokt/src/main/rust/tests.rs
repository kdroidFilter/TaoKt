//! Unit tests for taokt bindings.

#[cfg(test)]
mod types_tests {
    use crate::types::*;

    #[test]
    fn test_logical_size_creation() {
        let size = LogicalSize {
            width: 800.0,
            height: 600.0,
        };
        assert_eq!(size.width, 800.0);
        assert_eq!(size.height, 600.0);
    }

    #[test]
    fn test_logical_size_to_tao() {
        let size = LogicalSize {
            width: 1920.0,
            height: 1080.0,
        };
        let tao_size: tao::dpi::LogicalSize<f64> = size.into();
        assert_eq!(tao_size.width, 1920.0);
        assert_eq!(tao_size.height, 1080.0);
    }

    #[test]
    fn test_physical_size_u32_creation() {
        let size = PhysicalSizeU32 {
            width: 1920,
            height: 1080,
        };
        assert_eq!(size.width, 1920);
        assert_eq!(size.height, 1080);
    }

    #[test]
    fn test_physical_position_i32_creation() {
        let pos = PhysicalPositionI32 { x: 100, y: 200 };
        assert_eq!(pos.x, 100);
        assert_eq!(pos.y, 200);
    }

    #[test]
    fn test_physical_position_f64_creation() {
        let pos = PhysicalPositionF64 { x: 100.5, y: 200.5 };
        assert_eq!(pos.x, 100.5);
        assert_eq!(pos.y, 200.5);
    }

    #[test]
    fn test_element_state_conversion() {
        let pressed = ElementState::Pressed;
        let released = ElementState::Released;

        let tao_pressed: tao::event::ElementState = pressed.into();
        let tao_released: tao::event::ElementState = released.into();

        assert!(matches!(tao_pressed, tao::event::ElementState::Pressed));
        assert!(matches!(tao_released, tao::event::ElementState::Released));
    }

    #[test]
    fn test_element_state_from_tao() {
        let tao_pressed = tao::event::ElementState::Pressed;
        let tao_released = tao::event::ElementState::Released;

        let pressed: ElementState = tao_pressed.into();
        let released: ElementState = tao_released.into();

        assert_eq!(pressed, ElementState::Pressed);
        assert_eq!(released, ElementState::Released);
    }

    #[test]
    fn test_modifiers_state_default() {
        let mods = ModifiersState {
            shift: false,
            control: false,
            alt: false,
            super_key: false,
        };
        assert!(!mods.shift);
        assert!(!mods.control);
        assert!(!mods.alt);
        assert!(!mods.super_key);
    }

    #[test]
    fn test_modifiers_state_with_modifiers() {
        let mods = ModifiersState {
            shift: true,
            control: true,
            alt: false,
            super_key: true,
        };
        assert!(mods.shift);
        assert!(mods.control);
        assert!(!mods.alt);
        assert!(mods.super_key);
    }

    #[test]
    fn test_control_flow_variants() {
        let keep = ControlFlow::Keep;
        let wait = ControlFlow::Wait;
        let poll = ControlFlow::Poll;
        let exit = ControlFlow::Exit;

        assert!(matches!(keep, ControlFlow::Keep));
        assert!(matches!(wait, ControlFlow::Wait));
        assert!(matches!(poll, ControlFlow::Poll));
        assert!(matches!(exit, ControlFlow::Exit));
    }

    #[test]
    fn test_theme_variants() {
        let light = Theme::Light;
        let dark = Theme::Dark;

        assert!(matches!(light, Theme::Light));
        assert!(matches!(dark, Theme::Dark));
    }

    #[test]
    fn test_device_event_filter_conversion() {
        let always = DeviceEventFilter::Always;
        let unfocused = DeviceEventFilter::Unfocused;
        let never = DeviceEventFilter::Never;

        let tao_always: tao::event_loop::DeviceEventFilter = always.into();
        let tao_unfocused: tao::event_loop::DeviceEventFilter = unfocused.into();
        let tao_never: tao::event_loop::DeviceEventFilter = never.into();

        assert!(matches!(
            tao_always,
            tao::event_loop::DeviceEventFilter::Always
        ));
        assert!(matches!(
            tao_unfocused,
            tao::event_loop::DeviceEventFilter::Unfocused
        ));
        assert!(matches!(
            tao_never,
            tao::event_loop::DeviceEventFilter::Never
        ));
    }
}

#[cfg(test)]
mod error_tests {
    use crate::types::*;

    #[test]
    fn test_error_message_creation() {
        let error = TaoError::message("Test error message");
        match error {
            TaoError::Message { details } => {
                assert_eq!(details, "Test error message");
            }
            _ => panic!("Expected Message variant"),
        }
    }

    #[test]
    fn test_error_unsupported() {
        let error = TaoError::Unsupported;
        assert!(matches!(error, TaoError::Unsupported));
    }

    #[test]
    fn test_error_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let tao_error: TaoError = io_error.into();
        match tao_error {
            TaoError::Message { details } => {
                assert!(details.contains("not found") || details.contains("Not found"));
            }
            _ => panic!("Expected Message variant"),
        }
    }

    #[test]
    fn test_error_display() {
        let error = TaoError::message("Display test");
        let display = format!("{}", error);
        assert_eq!(display, "Display test");

        let unsupported = TaoError::Unsupported;
        let display_unsupported = format!("{}", unsupported);
        assert!(display_unsupported.contains("Unsupported"));
    }
}

#[cfg(test)]
mod graphics_tests {
    use crate::graphics::*;

    #[test]
    fn test_graphics_backend_default_for_platform() {
        let backend = GraphicsBackend::default_for_platform();
        // Should return a valid backend for the current platform
        assert!(backend.is_supported());
    }

    #[test]
    fn test_graphics_backend_is_supported() {
        // Metal should only be supported on macOS/iOS
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        assert!(GraphicsBackend::Metal.is_supported());

        #[cfg(not(any(target_os = "macos", target_os = "ios")))]
        assert!(!GraphicsBackend::Metal.is_supported());

        // DirectX12 should only be supported on Windows
        #[cfg(target_os = "windows")]
        assert!(GraphicsBackend::DirectX12.is_supported());

        #[cfg(not(target_os = "windows"))]
        assert!(!GraphicsBackend::DirectX12.is_supported());

        // OpenGL should be supported everywhere
        assert!(GraphicsBackend::OpenGL.is_supported());
    }

    #[test]
    fn test_raw_window_handle_empty() {
        let handle = RawWindowHandle::empty();
        assert!(!handle.is_valid());
        assert_eq!(handle.width, 0);
        assert_eq!(handle.height, 0);
        assert_eq!(handle.scale_factor, 1.0);
    }

    #[test]
    fn test_raw_window_handle_validation() {
        let mut handle = RawWindowHandle::empty();

        // Empty handle should be invalid
        assert!(!handle.is_valid());

        // With ns_view on Metal, should be valid
        #[cfg(target_os = "macos")]
        {
            handle.backend = GraphicsBackend::Metal;
            handle.ns_view = Some(0x12345678);
            assert!(handle.is_valid());
        }

        // With hwnd on DirectX12/Vulkan, should be valid
        #[cfg(target_os = "windows")]
        {
            handle.backend = GraphicsBackend::DirectX12;
            handle.hwnd = Some(0x12345678);
            assert!(handle.is_valid());
        }
    }
}

#[cfg(test)]
mod cursor_tests {
    use crate::types::*;

    #[test]
    fn test_cursor_icon_variants() {
        let icons = [
            CursorIcon::Default,
            CursorIcon::Crosshair,
            CursorIcon::Hand,
            CursorIcon::Arrow,
            CursorIcon::Move,
            CursorIcon::Text,
            CursorIcon::Wait,
            CursorIcon::Help,
            CursorIcon::Progress,
            CursorIcon::NotAllowed,
        ];

        for icon in icons {
            // Just ensure we can create and match all variants
            let _ = match icon {
                CursorIcon::Default => "default",
                CursorIcon::Crosshair => "crosshair",
                CursorIcon::Hand => "hand",
                CursorIcon::Arrow => "arrow",
                CursorIcon::Move => "move",
                CursorIcon::Text => "text",
                CursorIcon::Wait => "wait",
                CursorIcon::Help => "help",
                CursorIcon::Progress => "progress",
                CursorIcon::NotAllowed => "not-allowed",
                _ => "other",
            };
        }
    }
}

#[cfg(test)]
mod mouse_button_tests {
    use crate::types::*;

    #[test]
    fn test_mouse_button_variants() {
        let left = MouseButton::Left;
        let right = MouseButton::Right;
        let middle = MouseButton::Middle;
        let other = MouseButton::Other { button: 4 };

        assert!(matches!(left, MouseButton::Left));
        assert!(matches!(right, MouseButton::Right));
        assert!(matches!(middle, MouseButton::Middle));
        assert!(matches!(other, MouseButton::Other { button: 4 }));
    }

    #[test]
    fn test_mouse_scroll_delta_variants() {
        let line_delta = MouseScrollDelta::LineDelta { x: 1.0, y: 2.0 };
        let pixel_delta = MouseScrollDelta::PixelDelta {
            position: PhysicalPositionF64 { x: 10.0, y: 20.0 },
        };

        match line_delta {
            MouseScrollDelta::LineDelta { x, y } => {
                assert_eq!(x, 1.0);
                assert_eq!(y, 2.0);
            }
            _ => panic!("Expected LineDelta"),
        }

        match pixel_delta {
            MouseScrollDelta::PixelDelta { position } => {
                assert_eq!(position.x, 10.0);
                assert_eq!(position.y, 20.0);
            }
            _ => panic!("Expected PixelDelta"),
        }
    }
}
