# TaoKt - Kotlin Bindings for Tao Windowing Library

TaoKt provides Kotlin Multiplatform bindings for [Tao](https://github.com/tauri-apps/tao), the cross-platform windowing library from the Tauri project.

## Overview

TaoKt enables Kotlin applications to create native windows and handle events without relying on AWT/Swing. It supports both Kotlin/JVM and Kotlin/Native targets.

### Features

- **Cross-platform windowing**: Create native windows on macOS, Linux, and Windows
- **Event handling**: Full event loop with keyboard, mouse, and window events
- **DPI awareness**: Proper handling of high-DPI displays
- **Modern graphics**: Support for Metal, Vulkan, DirectX 12, and OpenGL backends
- **Kotlin Multiplatform**: Works on both JVM and Native targets

### Supported Platforms

| Platform | Graphics Backend | Status |
|----------|-----------------|--------|
| macOS | Metal | ✅ Stable |
| macOS | OpenGL | ✅ Stable |
| Linux (X11) | Vulkan | ✅ Stable |
| Linux (X11) | OpenGL | ✅ Stable |
| Linux (Wayland) | Vulkan | 🚧 Experimental |
| Windows | DirectX 12 | ✅ Stable |
| Windows | Vulkan | ✅ Stable |
| Windows | OpenGL | ✅ Stable |

---

## TaoKt Modules

This repo contains Kotlin/JVM bindings for `tauri-apps/tao` using Gobley (UniFFI + JNA), plus Kotlin ports of all upstream `tao` examples (`third_party/tao/examples`).

### Modules

- `:taokt-bindings` (dir `taokt/`): Rust `cdylib` + generated UniFFI Kotlin bindings.
- `:taokt-examples` (dir `taokt-examples/`): Kotlin ports of the `tao` examples.
- `:taokt-examples-native` (dir `taokt-examples-native/`): Kotlin/Native ports of the `tao` examples.

### Build bindings

```shell
./gradlew :taokt-bindings:build
```

### Run examples

```shell
./gradlew :taokt-examples:run --args="<example>"
```

### Run examples (Kotlin/Native)

Build/run the host executable (example for Linux x64):

```shell
./gradlew :taokt-examples-native:runDebugExecutableLinuxX64 --args="window"
```

List available examples:

```shell
./gradlew :taokt-examples:run
```

Notes:

- Icon examples use `third_party/tao/examples/icon.png` and `third_party/tao/examples/icon.ico`.
- On Linux/Windows, the bindings enable `EventLoopBuilderExt*::with_any_thread(true)` so the event loop can be created from the JVM thread.
- Kotlin/Native builds only the host target by default; set `-Ptaokt.enableAllNativeTargets=true` to keep non-host native tasks enabled.
- On Linux (Kotlin/Native), you need GTK3 dev libraries (for example on Debian/Ubuntu: `sudo apt-get install libgtk-3-dev`).
- Kotlin/Native on Windows supports x64 only (`mingwX64`).

---

## Quick Start

A minimal TaoKt application that opens a native window:

```kotlin
import io.github.kdroidfilter.taokt.tao.*
import io.github.kdroidfilter.taokt.tao.run as taoRun

fun main() {
    taoRun(
        object : TaoEventHandler {
            private var window: Window? = null

            override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                when (event) {
                    is TaoEvent.NewEvents -> if (event.cause == TaoStartCause.Init) {
                        window = app.createWindow(
                            WindowBuilder().apply {
                                setTitle("Hello TaoKt")
                                setInnerSize(LogicalSize(300.0, 300.0))
                            },
                        )
                    }

                    is TaoEvent.WindowEvent -> when (event.event) {
                        TaoWindowEvent.CloseRequested -> window?.close()
                        TaoWindowEvent.Destroyed -> return ControlFlow.Exit
                        else -> {}
                    }

                    TaoEvent.MainEventsCleared -> window?.requestRedraw()
                    else -> {}
                }
                return ControlFlow.Wait
            }
        },
    )
}
```

You can also run the built-in `window` example with `./gradlew :taokt-examples:run --args="window"`.

---

## API Overview

### Core Types

| Type | Description |
|------|-------------|
| `App` | Application context with event loop proxy |
| `Window` | Native window handle |
| `WindowBuilder` | Builder pattern for window configuration |
| `TaoEvent` | Event types (window, keyboard, mouse, etc.) |
| `ControlFlow` | Event loop control (Wait, Poll, Exit) |

### Graphics Types

| Type | Description |
|------|-------------|
| `RawWindowHandle` | Platform-specific window handles for graphics |
| `GraphicsBackend` | Supported backends (Metal, Vulkan, DirectX12, OpenGL) |

### Event Types

| Type | Description |
|------|-------------|
| `TaoWindowEvent` | Window events (resize, close, focus, etc.) |
| `TaoDeviceEvent` | Raw input events |
| `KeyEvent` | Keyboard input with key codes and modifiers |
| `MouseButton` | Mouse button identifiers |

---

## Architecture

```
┌─────────────────────────────────────┐
│  Kotlin application using TaoKt     │
└─────────────────────────────────────┘
              ↓ UniFFI / JNA
┌─────────────────────────────────────┐
│  TaoKt Rust Bindings                │
│  - Window management                │
│  - Event handling                   │
│  - Graphics abstractions            │
└─────────────────────────────────────┘
              ↓
┌─────────────────────────────────────┐
│  Tao (Rust)                         │
│  - Native windowing                 │
│  - OS integration                   │
└─────────────────────────────────────┘
              ↓
┌─────────────────────────────────────┐
│  Platform APIs                      │
│  - AppKit (macOS)                   │
│  - Win32 (Windows)                  │
│  - GTK/X11/Wayland (Linux)          │
└─────────────────────────────────────┘
```

---

## License

TaoKt is distributed under the Apache License 2.0. See `third_party/tao/LICENSE` for upstream Tao licensing details.
