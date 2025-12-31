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

* [/composeApp](./composeApp/src) is for code that will be shared across your Compose Multiplatform applications.
  It contains several subfolders:
    - [commonMain](./composeApp/src/commonMain/kotlin) is for code that’s common for all targets.
    - Other folders are for Kotlin code that will be compiled for only the platform indicated in the folder name.
      For example, if you want to use Apple’s CoreCrypto for the iOS part of your Kotlin app,
      the [iosMain](./composeApp/src/iosMain/kotlin) folder would be the right place for such calls.
      Similarly, if you want to edit the Desktop (JVM) specific part, the [jvmMain](./composeApp/src/jvmMain/kotlin)
      folder is the appropriate location.

### Build and Run Desktop (JVM) Application

To build and run the development version of the desktop app, use the run configuration from the run widget
in your IDE’s toolbar or run it directly from the terminal:

- on macOS/Linux
  ```shell
  ./gradlew :composeApp:run
  ```
- on Windows
  ```shell
  .\gradlew.bat :composeApp:run
  ```

---

Learn more about [Kotlin Multiplatform](https://www.jetbrains.com/help/kotlin-multiplatform-dev/get-started.html)…

---

## Integration with Skiko

TaoKt integrates seamlessly with Skiko for hardware-accelerated 2D graphics rendering.

### Example: Creating a Skiko Window

```kotlin
import io.github.kdroidfilter.taokt.tao.*
import org.jetbrains.skiko.*
import org.jetbrains.skia.*

fun main() {
    val layer = SkiaLayer()
    layer.renderDelegate = object : SkikoRenderDelegate {
        override fun onRender(canvas: Canvas, width: Int, height: Int, nanoTime: Long) {
            canvas.clear(Color.WHITE)
            // Draw with Skia APIs
            val paint = Paint().apply {
                color = Color.BLUE
                isAntiAlias = true
            }
            canvas.drawCircle(width / 2f, height / 2f, 100f, paint)
        }
    }

    run(object : TaoEventHandler {
        private var window: Window? = null

        override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
            when (event) {
                is TaoEvent.NewEvents -> {
                    if (event.cause == TaoStartCause.Init) {
                        window = app.createWindow(WindowBuilder().apply {
                            setTitle("Skiko + Tao")
                            setInnerSize(LogicalSize(800.0, 600.0))
                        })
                        layer.attachTo(window!!)
                    }
                }
                is TaoEvent.RedrawRequested -> {
                    layer.needRender()
                }
                is TaoEvent.MainEventsCleared -> {
                    window?.requestRedraw()
                }
                is TaoEvent.WindowEvent -> {
                    when (event.event) {
                        TaoWindowEvent.CloseRequested -> {
                            layer.detach()
                            return ControlFlow.Exit
                        }
                        else -> {}
                    }
                }
                else -> {}
            }
            return ControlFlow.Wait
        }
    })
}
```

### Running the Skiko Sample

```bash
# From the skiko directory
./gradlew :skiko:runTaoClock -Pskiko.tao.enabled=true
```

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
│  Kotlin Application                 │
│  (Skiko + TaoKt)                    │
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

This project is part of Skiko and follows the same license terms.
