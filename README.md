This is a Kotlin Multiplatform project targeting Desktop (JVM).

## TaoKt (tao → Kotlin/JVM)

This repo also contains a Kotlin/JVM binding for `tauri-apps/tao` using Gobley (UniFFI + JNA), plus Kotlin ports of all upstream `tao` examples (`third_party/tao/examples`).

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
