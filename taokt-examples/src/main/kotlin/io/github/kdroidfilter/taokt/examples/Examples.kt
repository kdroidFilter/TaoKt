package io.github.kdroidfilter.taokt.examples

import io.github.kdroidfilter.taokt.tao.*
import io.github.kdroidfilter.taokt.tao.run as taoRun
import io.github.kdroidfilter.taokt.tao.runReturnLoop as taoRunReturnLoop
import io.github.kdroidfilter.taokt.tao.runReturnLoopWithConfig as taoRunReturnLoopWithConfig
import io.github.kdroidfilter.taokt.tao.runWithConfig as taoRunWithConfig
import kotlin.concurrent.thread
import java.util.concurrent.LinkedBlockingQueue

object Examples {
    private enum class ControlFlowMode { WAIT, WAIT_UNTIL, POLL }

    val ALL: Map<String, (List<String>) -> Unit> = linkedMapOf(
        "control_flow" to { controlFlow() },
        "cursor" to { cursor() },
        "cursor_grab" to { cursorGrab() },
        "custom_events" to { customEvents() },
        "decorations" to { decorations() },
        "drag_window" to { dragWindow() },
        "fullscreen" to { fullscreen() },
        "handling_close" to { handlingClose() },
        "min_max_size" to { minMaxSize() },
        "minimize" to { minimize() },
        "monitor_list" to { monitorList() },
        "mouse_wheel" to { mouseWheel() },
        "multithreaded" to { multithreaded() },
        "multiwindow" to { multiwindow() },
        "overlay" to { overlay() },
        "parentwindow" to { parentWindow() },
        "progress_bar" to { progressBar() },
        "reopen_event" to { reopenEvent() },
        "request_redraw" to { requestRedraw() },
        "request_redraw_threaded" to { requestRedrawThreaded() },
        "resizable" to { resizable() },
        "set_ime_position" to { setImePosition() },
        "theme" to { theme() },
        "timer" to { timer() },
        "transparent" to { transparent() },
        "video_modes" to { videoModes() },
        "window" to { window() },
        "window_debug" to { windowDebug() },
        "window_icon" to { windowIcon() },
        "window_run_return" to { windowRunReturn() },
    )

    fun printHelp() {
        println("Usage: ./gradlew :taokt-examples:run --args=\"<example>\"")
        println("Available examples:")
        ALL.keys.forEach { println("  - $it") }
    }

    private fun osName(): String = System.getProperty("os.name").lowercase()
    private fun isWindows(): Boolean = osName().contains("win")
    private fun isMac(): Boolean = osName().contains("mac")
    private fun isLinuxLike(): Boolean =
        osName().contains("linux") || osName().contains("freebsd") || osName().contains("openbsd") ||
            osName().contains("netbsd") || osName().contains("dragonfly")

    private fun promptInt(prompt: String): Int? {
        print(prompt)
        val line = readLine()?.trim().orEmpty()
        if (line.isBlank()) return null
        return line.toIntOrNull()
    }

    private fun <T> List<T>.getOrNullIndex(index: Int): T? =
        if (index < 0 || index >= size) null else this[index]

    private class MultithreadedWorker(
        private val window: Window,
        private val baseWidth: UInt,
        private val baseHeight: UInt,
    ) {
        private val queue = LinkedBlockingQueue<TaoWindowEvent?>()
        @Volatile private var running = true
        private lateinit var thread: Thread

        private var modifiers = ModifiersState(false, false, false, false)
        private var videoModes: List<VideoMode> = emptyList()
        private var videoModeId: Int = 0

        fun start() {
            videoModes = window.currentMonitor()?.videoModes().orEmpty()
            thread = thread(start = true, isDaemon = true) {
                while (true) {
                    val ev = queue.take() ?: break
                    handle(ev)
                }
                window.close()
            }
        }

        fun send(event: TaoWindowEvent) {
            if (running) queue.put(event)
        }

        fun stop() {
            running = false
            queue.offer(null)
        }

        private fun <T> List<T>.getOrNullIndex(index: Int): T? =
            if (index < 0 || index >= size) null else this[index]

        private fun handle(event: TaoWindowEvent) {
            when (event) {
                is TaoWindowEvent.Moved -> {
                    val prev = videoModes.getOrNullIndex(videoModeId)
                    videoModes = window.currentMonitor()?.videoModes().orEmpty()
                    videoModeId = videoModeId.coerceAtMost(videoModes.lastIndex.coerceAtLeast(0))
                    val next = videoModes.getOrNullIndex(videoModeId)
                    if (next?.displayString() != prev?.displayString() && next != null) {
                        println("Window moved to another monitor, picked video mode: ${next.displayString()}")
                    }
                }

                is TaoWindowEvent.ModifiersChanged -> modifiers = event.modifiers
                is TaoWindowEvent.KeyboardInput -> {
                    if (event.event.state != ElementState.RELEASED) return

                    val state = !modifiers.shift
                    val key = event.event.logicalKey
                    window.setTitle(key.toString())

                    when (key) {
                        is Key.Character -> when (key.value.lowercase()) {
                            "a" -> window.setAlwaysOnTop(state)
                            "c" -> window.setCursorIcon(if (state) CursorIcon.PROGRESS else CursorIcon.DEFAULT)
                            "d" -> window.setDecorations(!state)
                            "f" -> {
                                val fullscreen = if (state) {
                                    if (modifiers.alt) {
                                        videoModes.getOrNullIndex(videoModeId)?.let { Fullscreen.Exclusive(it) }
                                    } else {
                                        Fullscreen.Borderless(null)
                                    }
                                } else {
                                    null
                                }
                                window.setFullscreen(fullscreen)
                            }

                            "g" -> window.setCursorGrab(state)
                            "h" -> window.setCursorVisible(!state)
                            "i" -> {
                                println("Info:")
                                println("-> outerPosition : ${runCatching { window.outerPosition() }.getOrNull()}")
                                println("-> innerPosition : ${runCatching { window.innerPosition() }.getOrNull()}")
                                println("-> outerSize     : ${window.outerSize()}")
                                println("-> innerSize     : ${window.innerSize()}")
                                println("-> fullscreen    : ${window.fullscreen()}")
                            }

                            "l" -> window.setMinInnerSize(
                                if (state) PhysicalSizeU32(baseWidth, baseHeight) else null,
                            )

                            "m" -> window.setMaximized(state)
                            "p" -> {
                                val pos = runCatching { window.outerPosition() }.getOrNull() ?: return
                                val sign = if (state) 1 else -1
                                window.setOuterPosition(PhysicalPositionI32(pos.x + 10 * sign, pos.y + 10 * sign))
                            }

                            "q" -> window.requestRedraw()
                            "r" -> window.setResizable(state)
                            "s" -> {
                                val size = if (state) {
                                    PhysicalSizeU32(baseWidth + 100u, baseHeight + 100u)
                                } else {
                                    PhysicalSizeU32(baseWidth, baseHeight)
                                }
                                window.setInnerSize(size)
                            }

                            "w" -> window.setCursorPosition(
                                PhysicalPositionI32((baseWidth / 2u).toInt(), (baseHeight / 2u).toInt()),
                            )

                            "z" -> {
                                window.setVisible(false)
                                Thread.sleep(1_000)
                                window.setVisible(true)
                            }
                        }

                        Key.ArrowLeft, Key.ArrowRight -> {
                            if (videoModes.isEmpty()) return
                            videoModeId = when (key) {
                                Key.ArrowLeft -> (videoModeId - 1).coerceAtLeast(0)
                                Key.ArrowRight -> (videoModeId + 1).coerceAtMost(videoModes.lastIndex)
                                else -> videoModeId
                            }
                            println("Picking video mode: ${videoModes[videoModeId].displayString()}")
                        }

                        else -> {}
                    }
                }

                else -> {}
            }
        }
    }

    private fun window() {
        taoRun(
            object : TaoEventHandler {
                private var window: Window? = null

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    println(event)
                    when (event) {
                        is TaoEvent.NewEvents -> {
                            if (event.cause == TaoStartCause.Init) {
                                val builder = WindowBuilder().apply {
                                    setTitle("A fantastic window!")
                                    setInnerSize(LogicalSize(300.0, 300.0))
                                    setMinInnerSize(LogicalSize(200.0, 200.0))
                                }
                                window = app.createWindow(builder)
                            }
                        }

                        is TaoEvent.WindowEvent -> when (event.event) {
                            TaoWindowEvent.CloseRequested -> {
                                window?.close()
                                window = null
                            }

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

    private fun controlFlow() {
        println("Press '1' to switch to Wait mode.")
        println("Press '2' to switch to WaitUntil mode.")
        println("Press '3' to switch to Poll mode.")
        println("Press 'R' to toggle requestRedraw() calls.")
        println("Press 'Esc' to close the window.")

        taoRun(
            object : TaoEventHandler {
                private var window: Window? = null
                private var mode: ControlFlowMode = ControlFlowMode.WAIT
                private var requestRedraw: Boolean = false
                private var waitCancelled: Boolean = false
                private var closeRequested: Boolean = false

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    when (event) {
                        is TaoEvent.NewEvents -> {
                            if (event.cause == TaoStartCause.Init && window == null) {
                                window = app.createWindow(
                                    WindowBuilder().apply {
                                        setTitle(
                                            "Press 1, 2, 3 to change control flow mode. Press R to toggle redraw requests.",
                                        )
                                    },
                                )
                            }
                            waitCancelled = (event.cause == TaoStartCause.WaitCancelled && mode == ControlFlowMode.WAIT_UNTIL)
                        }

                        is TaoEvent.WindowEvent -> when (val we = event.event) {
                            TaoWindowEvent.CloseRequested -> closeRequested = true
                            is TaoWindowEvent.KeyboardInput -> {
                                val keyEvent = we.event
                                if (keyEvent.state == ElementState.PRESSED) {
                                    when (val k = keyEvent.logicalKey) {
                                        is Key.Character -> when (k.value.lowercase()) {
                                            "1" -> mode = ControlFlowMode.WAIT
                                            "2" -> mode = ControlFlowMode.WAIT_UNTIL
                                            "3" -> mode = ControlFlowMode.POLL
                                            "r" -> requestRedraw = !requestRedraw
                                        }

                                        Key.Escape -> closeRequested = true
                                        else -> {}
                                    }
                                }
                            }

                            else -> {}
                        }

                        TaoEvent.MainEventsCleared -> {
                            val w = window
                            if (w != null && requestRedraw && !waitCancelled && !closeRequested) {
                                w.requestRedraw()
                            }
                            if (closeRequested) return ControlFlow.Exit
                        }

                        TaoEvent.RedrawEventsCleared -> {
                            return when (mode) {
                                ControlFlowMode.WAIT -> ControlFlow.Wait
                                ControlFlowMode.WAIT_UNTIL -> if (waitCancelled) {
                                    ControlFlow.Keep
                                } else {
                                    ControlFlow.WaitUntil(durationMs = 100u)
                                }

                                ControlFlowMode.POLL -> {
                                    Thread.sleep(100)
                                    ControlFlow.Poll
                                }
                            }
                        }

                        else -> {}
                    }
                    return ControlFlow.Wait
                }
            },
        )
    }

    private fun cursor() {
        taoRun(
            object : TaoEventHandler {
                private var window: Window? = null
                private var cursorIdx = 0
                private val cursors = CursorIcon.entries

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    when (event) {
                        is TaoEvent.NewEvents -> if (event.cause == TaoStartCause.Init && window == null) {
                            window = app.createWindow(WindowBuilder())
                            window?.setTitle("A fantastic window!")
                        }

                        is TaoEvent.WindowEvent -> when (val we = event.event) {
                            TaoWindowEvent.CloseRequested -> return ControlFlow.Exit
                            is TaoWindowEvent.KeyboardInput -> {
                                if (we.event.state == ElementState.PRESSED) {
                                    val icon = cursors[cursorIdx]
                                    println("Setting cursor to \"$icon\"")
                                    window?.setCursorIcon(icon)
                                    cursorIdx = (cursorIdx + 1) % cursors.size
                                }
                            }

                            else -> {}
                        }

                        else -> {}
                    }
                    return ControlFlow.Wait
                }
            },
        )
    }

    private fun cursorGrab() {
        taoRun(
            object : TaoEventHandler {
                private var window: Window? = null
                private var modifiers = ModifiersState(false, false, false, false)

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    when (event) {
                        is TaoEvent.NewEvents -> if (event.cause == TaoStartCause.Init && window == null) {
                            window = app.createWindow(
                                WindowBuilder().apply { setTitle("Super Cursor Grab'n'Hide Simulator 9000") },
                            )
                        }

                        is TaoEvent.WindowEvent -> when (val we = event.event) {
                            TaoWindowEvent.CloseRequested -> return ControlFlow.Exit
                            is TaoWindowEvent.ModifiersChanged -> modifiers = we.modifiers
                            is TaoWindowEvent.KeyboardInput -> {
                                val keyEvent = we.event
                                if (keyEvent.state == ElementState.RELEASED) {
                                    when (val k = keyEvent.logicalKey) {
                                        Key.Escape -> return ControlFlow.Exit
                                        is Key.Character -> when (k.value.lowercase()) {
                                            "g" -> window?.setCursorGrab(!modifiers.shift)
                                            "h" -> window?.setCursorVisible(modifiers.shift)
                                        }

                                        else -> {}
                                    }
                                }
                            }

                            else -> {}
                        }

                        is TaoEvent.DeviceEvent -> when (val de = event.event) {
                            is TaoDeviceEvent.MouseMotion -> println("mouse moved: (${de.deltaX}, ${de.deltaY})")
                            is TaoDeviceEvent.Button -> when (de.state) {
                                ElementState.PRESSED -> println("mouse button ${de.button} pressed")
                                ElementState.RELEASED -> println("mouse button ${de.button} released")
                            }

                            else -> {}
                        }

                        else -> {}
                    }
                    return ControlFlow.Wait
                }
            },
        )
    }

    private fun customEvents() {
        taoRun(
            object : TaoEventHandler {
                private var started = false

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    when (event) {
                        is TaoEvent.NewEvents -> if (event.cause == TaoStartCause.Init && !started) {
                            started = true
                            app.createWindow(WindowBuilder().apply { setTitle("A fantastic window!") })

                            val proxy = app.createProxy()
                            thread(isDaemon = true) {
                                while (true) {
                                    Thread.sleep(1_000)
                                    try {
                                        proxy.sendEvent(TaoUserEvent.Timer)
                                    } catch (_: TaoException) {
                                        return@thread
                                    }
                                }
                            }
                        }

                        is TaoEvent.UserEvent -> println("user event: ${event.event}")
                        is TaoEvent.WindowEvent -> if (event.event == TaoWindowEvent.CloseRequested) return ControlFlow.Exit
                        else -> {}
                    }
                    return ControlFlow.Wait
                }
            },
        )
    }

    private fun decorations() {
        taoRun(
            object : TaoEventHandler {
                private var window: Window? = null
                private var decorations = true

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    when (event) {
                        is TaoEvent.NewEvents -> if (event.cause == TaoStartCause.Init && window == null) {
                            window = app.createWindow(
                                WindowBuilder().apply {
                                    setTitle("Hit space to toggle decorations.")
                                    setInnerSize(LogicalSize(400.0, 200.0))
                                    setDecorations(decorations)
                                },
                            )
                        }

                        is TaoEvent.WindowEvent -> when (val we = event.event) {
                            TaoWindowEvent.CloseRequested -> return ControlFlow.Exit
                            is TaoWindowEvent.KeyboardInput -> {
                                val keyEvent = we.event
                                if (keyEvent.physicalKey == KeyCode.Space && keyEvent.state == ElementState.RELEASED) {
                                    decorations = !decorations
                                    println("Decorations: $decorations")
                                    window?.setDecorations(decorations)
                                }
                            }

                            else -> {}
                        }

                        else -> {}
                    }
                    return ControlFlow.Wait
                }
            },
        )
    }

    private fun dragWindow() {
        taoRun(
            object : TaoEventHandler {
                private var window1: Window? = null
                private var window2: Window? = null
                private var switched = false
                private var enteredId: ULong = 0u

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    when (event) {
                        is TaoEvent.NewEvents -> if (event.cause == TaoStartCause.Init && window1 == null) {
                            println("Switch which window is to be dragged by pressing \"x\".")
                            window1 = app.createWindowDefault()
                            window2 = app.createWindowDefault()
                            enteredId = window2!!.id()
                            nameWindows()
                        }

                        is TaoEvent.WindowEvent -> when (val we = event.event) {
                            TaoWindowEvent.CloseRequested -> return ControlFlow.Exit
                            TaoWindowEvent.CursorEntered -> {
                                enteredId = event.windowId
                                nameWindows()
                            }

                            is TaoWindowEvent.MouseInput -> {
                                if (we.state == ElementState.PRESSED && we.button == MouseButton.Left) {
                                    val w1 = window1 ?: return ControlFlow.Wait
                                    val w2 = window2 ?: return ControlFlow.Wait
                                    val dragTarget =
                                        if ((event.windowId == w1.id() && switched) || (event.windowId == w2.id() && !switched)) w2 else w1
                                    dragTarget.dragWindow()
                                }
                            }

                            is TaoWindowEvent.KeyboardInput -> {
                                val keyEvent = we.event
                                if (keyEvent.state == ElementState.RELEASED && keyEvent.logicalKey == Key.Character("x")) {
                                    switched = !switched
                                    nameWindows()
                                    println("Switched!")
                                }
                            }

                            else -> {}
                        }

                        else -> {}
                    }
                    return ControlFlow.Wait
                }

                private fun nameWindows() {
                    val w1 = window1 ?: return
                    val w2 = window2 ?: return
                    val (dragTarget, other) =
                        if ((enteredId == w1.id() && switched) || (enteredId == w2.id() && !switched)) {
                            w2 to w1
                        } else {
                            w1 to w2
                        }
                    dragTarget.setTitle("drag target")
                    other.setTitle("tao window")
                }
            },
        )
    }

    private fun fullscreen() {
        val mode = promptInt(
            "Please choose the fullscreen mode: (1) exclusive, (2) borderless, (3) borderless on current monitor: ",
        ) ?: return

        taoRun(
            object : TaoEventHandler {
                private var window: Window? = null
                private var fullscreen: Fullscreen? = null
                private var decorations = true

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    when (event) {
                        is TaoEvent.NewEvents -> if (event.cause == TaoStartCause.Init && window == null) {
                            fullscreen = when (mode) {
                                1 -> promptForMonitor(app)?.let { monitor ->
                                    val vm = promptForVideoMode(monitor)
                                    vm?.let { Fullscreen.Exclusive(it) }
                                }

                                2 -> Fullscreen.Borderless(promptForMonitor(app))
                                3 -> Fullscreen.Borderless(null)
                                else -> null
                            }

                            window = app.createWindow(
                                WindowBuilder().apply {
                                    setTitle("Hello world!")
                                    setFullscreen(fullscreen)
                                },
                            )
                        }

                        is TaoEvent.WindowEvent -> when (val we = event.event) {
                            TaoWindowEvent.CloseRequested -> return ControlFlow.Exit
                            is TaoWindowEvent.KeyboardInput -> {
                                val keyEvent = we.event
                                if (keyEvent.state == ElementState.PRESSED) {
                                    val w = window ?: return ControlFlow.Wait
                                    when (val k = keyEvent.logicalKey) {
                                        Key.Escape -> return ControlFlow.Exit
                                        is Key.Character -> when (k.value.lowercase()) {
                                            "f" -> {
                                                if (w.fullscreen() != null) {
                                                    w.setFullscreen(null)
                                                } else {
                                                    w.setFullscreen(fullscreen)
                                                }
                                            }

                                            "s" -> println("window.fullscreen() = ${w.fullscreen()}")
                                            "m" -> w.setMaximized(!w.isMaximized())
                                            "d" -> {
                                                decorations = !decorations
                                                w.setDecorations(decorations)
                                            }
                                        }

                                        else -> {}
                                    }
                                }
                            }

                            else -> {}
                        }

                        else -> {}
                    }
                    return ControlFlow.Wait
                }

                private fun promptForMonitor(app: App): Monitor? {
                    val monitors = app.availableMonitors()
                    monitors.forEachIndexed { idx, monitor ->
                        println("Monitor #$idx: ${monitor.name()}")
                    }
                    val num = promptInt("Please write the number of the monitor to use: ") ?: return null
                    val monitor = monitors.getOrNullIndex(num)
                    println("Using ${monitor?.name()}")
                    return monitor
                }

                private fun promptForVideoMode(monitor: Monitor): VideoMode? {
                    val videoModes = monitor.videoModes()
                    videoModes.forEachIndexed { idx, vm ->
                        println("Video mode #$idx: ${vm.displayString()}")
                    }
                    val num = promptInt("Please write the number of the video mode to use: ") ?: return null
                    val vm = videoModes.getOrNullIndex(num)
                    println("Using ${vm?.displayString()}")
                    return vm
                }
            },
        )
    }

    private fun handlingClose() {
        taoRun(
            object : TaoEventHandler {
                private var closeRequested = false

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    when (event) {
                        is TaoEvent.NewEvents -> if (event.cause == TaoStartCause.Init) {
                            app.createWindow(WindowBuilder().apply { setTitle("Your faithful window") })
                        }

                        is TaoEvent.WindowEvent -> when (val we = event.event) {
                            TaoWindowEvent.CloseRequested -> {
                                println("Are you ready to bid your window farewell? [Y/N]")
                                closeRequested = true
                            }

                            is TaoWindowEvent.KeyboardInput -> {
                                val keyEvent = we.event
                                if (keyEvent.state == ElementState.RELEASED) {
                                    val k = keyEvent.logicalKey
                                    if (k is Key.Character) {
                                        when (k.value.lowercase()) {
                                            "y" -> if (closeRequested) return ControlFlow.Exit
                                            "n" -> if (closeRequested) {
                                                println("Your window will continue to stay by your side.")
                                                closeRequested = false
                                            }
                                        }
                                    }
                                }
                            }

                            else -> {}
                        }

                        else -> {}
                    }
                    return ControlFlow.Wait
                }
            },
        )
    }

    private fun minMaxSize() {
        val minWidth = 400.0
        val maxWidth = 800.0
        val minHeight = 200.0
        val maxHeight = 400.0

        println("constraint keys:")
        println("  (E) Toggle the min width")
        println("  (F) Toggle the max width")
        println("  (P) Toggle the min height")
        println("  (V) Toggle the max height")

        taoRun(
            object : TaoEventHandler {
                private var window: Window? = null
                private var constraints = WindowSizeConstraints(null, null, null, null)

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    when (event) {
                        is TaoEvent.NewEvents -> if (event.cause == TaoStartCause.Init && window == null) {
                            window = app.createWindowDefault()
                        }

                        is TaoEvent.WindowEvent -> when (val we = event.event) {
                            TaoWindowEvent.CloseRequested -> return ControlFlow.Exit
                            is TaoWindowEvent.KeyboardInput -> {
                                val keyEvent = we.event
                                if (keyEvent.state == ElementState.RELEASED) {
                                    when (val k = keyEvent.logicalKey) {
                                        is Key.Character -> when (k.value) {
                                            "e" -> constraints = constraints.copy(minWidth = constraints.minWidth?.let { null } ?: minWidth)
                                            "f" -> constraints = constraints.copy(maxWidth = constraints.maxWidth?.let { null } ?: maxWidth)
                                            "p" -> constraints = constraints.copy(minHeight = constraints.minHeight?.let { null } ?: minHeight)
                                            "v" -> constraints = constraints.copy(maxHeight = constraints.maxHeight?.let { null } ?: maxHeight)
                                        }

                                        else -> {}
                                    }
                                    window?.setInnerSizeConstraints(constraints)
                                }
                            }

                            else -> {}
                        }

                        else -> {}
                    }
                    return ControlFlow.Wait
                }
            },
        )
    }

    private fun minimize() {
        taoRun(
            object : TaoEventHandler {
                private var window: Window? = null

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    when (event) {
                        is TaoEvent.NewEvents -> if (event.cause == TaoStartCause.Init && window == null) {
                            window = app.createWindow(WindowBuilder().apply { setTitle("A fantastic window!") })
                        }

                        is TaoEvent.WindowEvent -> when (val we = event.event) {
                            TaoWindowEvent.CloseRequested -> return ControlFlow.Exit
                            is TaoWindowEvent.KeyboardInput -> {
                                val w = window ?: return ControlFlow.Wait
                                if (event.windowId == w.id() && we.event.logicalKey == Key.Character("m")) {
                                    w.setMinimized(true)
                                }
                            }

                            else -> {}
                        }

                        else -> {}
                    }
                    return ControlFlow.Wait
                }
            },
        )
    }

    private fun monitorList() {
        taoRun(
            object : TaoEventHandler {
                private var done = false

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    if (done) return ControlFlow.Exit
                    if (event is TaoEvent.NewEvents && event.cause == TaoStartCause.Init) {
                        done = true
                        val window = app.createWindowDefault()
                        println("availableMonitors():")
                        window.availableMonitors().forEach { println("  - ${it.debugString()}") }
                        println("primaryMonitor(): ${window.primaryMonitor()?.debugString()}")
                        window.close()
                        return ControlFlow.Exit
                    }
                    return ControlFlow.Wait
                }
            },
        )
    }

    private fun mouseWheel() {
        taoRun(
            object : TaoEventHandler {
                private var window: Window? = null

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    when (event) {
                        is TaoEvent.NewEvents -> if (event.cause == TaoStartCause.Init && window == null) {
                            window = app.createWindow(WindowBuilder().apply { setTitle("Mouse Wheel events") })
                        }

                        is TaoEvent.WindowEvent -> if (event.event == TaoWindowEvent.CloseRequested) return ControlFlow.Exit

                        is TaoEvent.DeviceEvent -> {
                            val w = window ?: return ControlFlow.Wait
                            when (val de = event.event) {
                                is TaoDeviceEvent.MouseWheel -> {
                                    when (val delta = de.delta) {
                                        is MouseScrollDelta.LineDelta -> {
                                            println("mouse wheel Line Delta: (${delta.x},${delta.y})")
                                            val pixelsPerLine = 120.0
                                            try {
                                                val pos = w.outerPosition()
                                                w.setOuterPosition(
                                                    PhysicalPositionI32(
                                                        pos.x - (delta.x * pixelsPerLine).toInt(),
                                                        pos.y - (delta.y * pixelsPerLine).toInt(),
                                                    ),
                                                )
                                            } catch (_: TaoException) {
                                            }
                                        }

                                        is MouseScrollDelta.PixelDelta -> {
                                            println("mouse wheel Pixel Delta: (${delta.x},${delta.y})")
                                            try {
                                                val pos = w.outerPosition()
                                                w.setOuterPosition(
                                                    PhysicalPositionI32(
                                                        pos.x - delta.x.toInt(),
                                                        pos.y - delta.y.toInt(),
                                                    ),
                                                )
                                            } catch (_: TaoException) {
                                            }
                                        }

                                        else -> {}
                                    }
                                }

                                else -> {}
                            }
                        }

                        else -> {}
                    }
                    return ControlFlow.Wait
                }
            },
        )
    }

    private fun multiwindow() {
        taoRun(
            object : TaoEventHandler {
                private val windows = linkedMapOf<ULong, Window>()

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    when (event) {
                        is TaoEvent.NewEvents -> if (event.cause == TaoStartCause.Init && windows.isEmpty()) {
                            repeat(3) {
                                val w = app.createWindowDefault()
                                windows[w.id()] = w
                            }
                        }

                        is TaoEvent.WindowEvent -> when (val we = event.event) {
                            TaoWindowEvent.CloseRequested -> {
                                println("Window ${event.windowId} has received the signal to close")
                                windows.remove(event.windowId)?.close()
                                if (windows.isEmpty()) return ControlFlow.Exit
                            }

                            is TaoWindowEvent.KeyboardInput -> {
                                if (we.event.state == ElementState.PRESSED) {
                                    val w = app.createWindowDefault()
                                    windows[w.id()] = w
                                }
                            }

                            else -> {}
                        }

                        else -> {}
                    }
                    return if (windows.isEmpty()) ControlFlow.Exit else ControlFlow.Wait
                }
            },
        )
    }

    private fun multithreaded() {
        val windowCount = 3
        val baseWidth = 600u
        val baseHeight = 400u

        taoRun(
            object : TaoEventHandler {
                private val workers = linkedMapOf<ULong, MultithreadedWorker>()

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    if (event is TaoEvent.NewEvents && event.cause == TaoStartCause.Init && workers.isEmpty()) {
                        repeat(windowCount) {
                            val w = app.createWindow(
                                WindowBuilder().apply {
                                    setInnerSize(LogicalSize(baseWidth.toDouble(), baseHeight.toDouble()))
                                },
                            )
                            val worker = MultithreadedWorker(w, baseWidth, baseHeight)
                            workers[w.id()] = worker
                            worker.start()
                        }
                    }

                    when (event) {
                        is TaoEvent.WindowEvent -> {
                            val worker = workers[event.windowId]
                            when (val we = event.event) {
                                TaoWindowEvent.CloseRequested,
                                TaoWindowEvent.Destroyed,
                                is TaoWindowEvent.KeyboardInput -> {
                                    val shouldClose = when (we) {
                                        TaoWindowEvent.CloseRequested,
                                        TaoWindowEvent.Destroyed,
                                        -> true

                                        is TaoWindowEvent.KeyboardInput -> {
                                            we.event.state == ElementState.RELEASED && we.event.logicalKey == Key.Escape
                                        }

                                        else -> false
                                    }

                                    if (shouldClose) {
                                        workers.remove(event.windowId)?.stop()
                                    } else if (worker != null) {
                                        worker.send(we)
                                    }
                                }

                                else -> if (worker != null) worker.send(we)
                            }
                        }

                        else -> {}
                    }

                    return if (workers.isEmpty()) ControlFlow.Exit else ControlFlow.Wait
                }
            },
        )
    }

    private fun overlay() {
        taoRun(
            object : TaoEventHandler {
                private var window: Window? = null
                private var modifiers = ModifiersState(false, false, false, false)

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    when (event) {
                        is TaoEvent.NewEvents -> if (event.cause == TaoStartCause.Init && window == null) {
                            window = app.createWindowDefault()
                            println("Key mappings:")
                            if (isWindows()) println("  [any key]: Show the Overlay Icon") else println("  [1-5]: Show a Badge count")
                            println("  Ctrl+1: Clear")
                        }

                        is TaoEvent.WindowEvent -> when (val we = event.event) {
                            TaoWindowEvent.CloseRequested -> return ControlFlow.Exit
                            is TaoWindowEvent.ModifiersChanged -> modifiers = we.modifiers
                            is TaoWindowEvent.KeyboardInput -> {
                                val keyEvent = we.event
                                if (keyEvent.state == ElementState.RELEASED && keyEvent.logicalKey is Key.Character) {
                                    val keyStr = (keyEvent.logicalKey as Key.Character).value
                                    val count = when (keyStr) {
                                        "1" -> 1
                                        "2" -> 2
                                        "3" -> 3
                                        "4" -> 4
                                        "5" -> 5
                                        else -> 20
                                    }

                                    val w = window ?: return ControlFlow.Wait
                                    val isEmpty = !modifiers.shift && !modifiers.control && !modifiers.alt && !modifiers.superKey
                                    if (isEmpty) {
                                        when {
                                            isWindows() -> {
                                                val iconPath = "third_party/tao/examples/icon.ico"
                                                val icon = runCatching { Icon.fromFile(iconPath) }.getOrNull()
                                                w.setOverlayIcon(icon)
                                            }

                                            isLinuxLike() -> w.setBadgeCount(count.toLong())
                                            isMac() -> w.setBadgeLabel(count.toString())
                                        }
                                    } else if (modifiers.control && keyStr == "1") {
                                        when {
                                            isWindows() -> w.setOverlayIcon(null)
                                            isLinuxLike() -> w.setBadgeCount(null)
                                            isMac() -> w.setBadgeLabel(null)
                                        }
                                    }
                                }
                            }

                            else -> {}
                        }

                        else -> {}
                    }
                    return ControlFlow.Wait
                }
            },
        )
    }

    private fun parentWindow() {
        if (!(isWindows() || isMac() || isLinuxLike())) {
            println("This platform doesn't have the parent window support.")
            return
        }

        taoRun(
            object : TaoEventHandler {
                private val windows = linkedMapOf<ULong, Window>()

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    when (event) {
                        is TaoEvent.NewEvents -> if (event.cause == TaoStartCause.Init && windows.isEmpty()) {
                            println("TAO application started!")

                            val mainWindow = app.createWindowDefault()
                            val childBuilder = WindowBuilder().apply {
                                setInnerSize(LogicalSize(200.0, 200.0))
                                setParentWindow(mainWindow)
                            }
                            val childWindow = app.createWindow(childBuilder)
                            windows[mainWindow.id()] = mainWindow
                            windows[childWindow.id()] = childWindow
                        }

                        is TaoEvent.WindowEvent -> if (event.event == TaoWindowEvent.CloseRequested) {
                            println("Window ${event.windowId} has received the signal to close")
                            windows.remove(event.windowId)?.close()
                            if (windows.isEmpty()) return ControlFlow.Exit
                        }

                        else -> {}
                    }
                    return ControlFlow.Wait
                }
            },
        )
    }

    private fun progressBar() {
        println("Key mappings:")
        println("  [1-5]: Set progress to [0%, 25%, 50%, 75%, 100%]")
        println("  Ctrl+1: Set state to None")
        println("  Ctrl+2: Set state to Normal")
        println("  Ctrl+3: Set state to Indeterminate")
        println("  Ctrl+4: Set state to Paused")
        println("  Ctrl+5: Set state to Error")

        taoRun(
            object : TaoEventHandler {
                private var window: Window? = null
                private var modifiers = ModifiersState(false, false, false, false)

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    when (event) {
                        is TaoEvent.NewEvents -> if (event.cause == TaoStartCause.Init && window == null) {
                            window = app.createWindowDefault()
                        }

                        is TaoEvent.WindowEvent -> when (val we = event.event) {
                            TaoWindowEvent.CloseRequested -> return ControlFlow.Exit
                            is TaoWindowEvent.ModifiersChanged -> modifiers = we.modifiers
                            is TaoWindowEvent.KeyboardInput -> {
                                val keyEvent = we.event
                                if (keyEvent.state == ElementState.RELEASED && keyEvent.logicalKey is Key.Character) {
                                    val keyStr = (keyEvent.logicalKey as Key.Character).value
                                    val w = window ?: return ControlFlow.Wait

                                    val isEmpty =
                                        !modifiers.shift && !modifiers.control && !modifiers.alt && !modifiers.superKey

                                    if (isEmpty) {
                                        val progress = when (keyStr) {
                                            "1" -> 0uL
                                            "2" -> 25uL
                                            "3" -> 50uL
                                            "4" -> 75uL
                                            "5" -> 100uL
                                            else -> null
                                        }
                                        if (progress != null) {
                                            w.setProgressBar(
                                                ProgressBarState(
                                                    progress = progress,
                                                    state = ProgressState.NORMAL,
                                                    desktopFilename = null,
                                                ),
                                            )
                                        }
                                    } else if (modifiers.control) {
                                        val state = when (keyStr) {
                                            "1" -> ProgressState.NONE
                                            "2" -> ProgressState.NORMAL
                                            "3" -> ProgressState.INDETERMINATE
                                            "4" -> ProgressState.PAUSED
                                            "5" -> ProgressState.ERROR
                                            else -> null
                                        }
                                        if (state != null) {
                                            w.setProgressBar(
                                                ProgressBarState(
                                                    progress = null,
                                                    state = state,
                                                    desktopFilename = null,
                                                ),
                                            )
                                        }
                                    }
                                }
                            }

                            else -> {}
                        }

                        else -> {}
                    }
                    return ControlFlow.Wait
                }
            },
        )
    }

    private fun reopenEvent() {
        taoRun(
            object : TaoEventHandler {
                private var window: Window? = null

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    when (event) {
                        is TaoEvent.NewEvents -> if (event.cause == TaoStartCause.Init && window == null) {
                            window = app.createWindowDefault()
                        }

                        is TaoEvent.WindowEvent -> if (event.event == TaoWindowEvent.CloseRequested) {
                            window?.close()
                            window = null
                        }

                        is TaoEvent.Reopen -> {
                            println("on reopen, has visible windows: ${event.hasVisibleWindows}")
                            if (!event.hasVisibleWindows && window == null) {
                                window = app.createWindowDefault()
                            }
                        }

                        TaoEvent.MainEventsCleared -> window?.requestRedraw()
                        else -> {}
                    }
                    return ControlFlow.Wait
                }
            },
        )
    }

    private fun requestRedraw() {
        taoRun(
            object : TaoEventHandler {
                private var window: Window? = null

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    println(event)
                    when (event) {
                        is TaoEvent.NewEvents -> if (event.cause == TaoStartCause.Init && window == null) {
                            window = app.createWindow(WindowBuilder().apply { setTitle("A fantastic window!") })
                        }

                        is TaoEvent.WindowEvent -> when (val we = event.event) {
                            TaoWindowEvent.CloseRequested -> return ControlFlow.Exit
                            is TaoWindowEvent.MouseInput -> if (we.state == ElementState.RELEASED) window?.requestRedraw()
                            else -> {}
                        }

                        is TaoEvent.RedrawRequested -> println("\nredrawing!\n")
                        else -> {}
                    }
                    return ControlFlow.Wait
                }
            },
        )
    }

    private fun requestRedrawThreaded() {
        taoRun(
            object : TaoEventHandler {
                private var window: Window? = null
                @Volatile private var running = true

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    println(event)
                    when (event) {
                        is TaoEvent.NewEvents -> if (event.cause == TaoStartCause.Init && window == null) {
                            val w = app.createWindow(WindowBuilder().apply { setTitle("A fantastic window!") })
                            window = w
                            thread(isDaemon = true) {
                                while (running) {
                                    Thread.sleep(1_000)
                                    w.requestRedraw()
                                }
                            }
                        }

                        is TaoEvent.WindowEvent -> if (event.event == TaoWindowEvent.CloseRequested) {
                            running = false
                            window?.close()
                            return ControlFlow.Exit
                        }

                        is TaoEvent.RedrawRequested -> println("\nredrawing!\n")
                        else -> {}
                    }
                    return ControlFlow.Wait
                }
            },
        )
    }

    private fun resizable() {
        taoRun(
            object : TaoEventHandler {
                private var window: Window? = null
                private var resizable = false

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    when (event) {
                        is TaoEvent.NewEvents -> if (event.cause == TaoStartCause.Init && window == null) {
                            window = app.createWindow(
                                WindowBuilder().apply {
                                    setTitle("Hit space to toggle resizability.")
                                    setInnerSize(LogicalSize(400.0, 200.0))
                                    setResizable(resizable)
                                },
                            )
                        }

                        is TaoEvent.WindowEvent -> when (val we = event.event) {
                            TaoWindowEvent.CloseRequested -> return ControlFlow.Exit
                            is TaoWindowEvent.KeyboardInput -> {
                                val keyEvent = we.event
                                if (keyEvent.physicalKey == KeyCode.Space && keyEvent.state == ElementState.RELEASED) {
                                    resizable = !resizable
                                    println("Resizable: $resizable")
                                    window?.setResizable(resizable)
                                }
                            }

                            else -> {}
                        }

                        else -> {}
                    }
                    return ControlFlow.Wait
                }
            },
        )
    }

    private fun setImePosition() {
        println("Ime position will system default")
        println("Click to set ime position to cursor's")

        taoRun(
            object : TaoEventHandler {
                private var window: Window? = null
                private var cursorPos = PhysicalPositionF64(0.0, 0.0)

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    when (event) {
                        is TaoEvent.NewEvents -> if (event.cause == TaoStartCause.Init && window == null) {
                            window = app.createWindowDefault()
                            window?.setTitle("A fantastic window!")
                        }

                        is TaoEvent.WindowEvent -> when (val we = event.event) {
                            is TaoWindowEvent.CursorMoved -> cursorPos = we.position
                            is TaoWindowEvent.MouseInput -> if (we.state == ElementState.RELEASED) {
                                println("Setting ime position to ${cursorPos.x}, ${cursorPos.y}")
                                window?.setImePosition(cursorPos)
                            }

                            TaoWindowEvent.CloseRequested -> return ControlFlow.Exit
                            else -> {}
                        }

                        else -> {}
                    }
                    return ControlFlow.Wait
                }
            },
        )
    }

    private fun theme() {
        taoRun(
            object : TaoEventHandler {
                private var window: Window? = null

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    when (event) {
                        is TaoEvent.NewEvents -> if (event.cause == TaoStartCause.Init && window == null) {
                            val w = app.createWindow(WindowBuilder().apply { setTitle("A fantastic window!") })
                            window = w
                            println("Initial theme: ${w.theme()}")
                            println("Press D for Dark Mode")
                            println("Press L for Light Mode")
                            println("Press A for Auto Mode")
                        }

                        is TaoEvent.WindowEvent -> when (val we = event.event) {
                            TaoWindowEvent.CloseRequested -> return ControlFlow.Exit
                            is TaoWindowEvent.KeyboardInput -> {
                                val keyEvent = we.event
                                when (keyEvent.physicalKey) {
                                    KeyCode.KeyD -> window?.setTheme(Theme.DARK)
                                    KeyCode.KeyL -> window?.setTheme(Theme.LIGHT)
                                    KeyCode.KeyA -> window?.setTheme(null)
                                    else -> {}
                                }
                            }

                            is TaoWindowEvent.ThemeChanged -> println("Theme is changed: ${we.theme}")
                            else -> {}
                        }

                        else -> {}
                    }
                    return ControlFlow.Wait
                }
            },
        )
    }

    private fun timer() {
        taoRun(
            object : TaoEventHandler {
                private var started = false

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    println(event)
                    return when (event) {
                        is TaoEvent.NewEvents -> {
                            if (event.cause == TaoStartCause.Init && !started) {
                                started = true
                                app.createWindow(WindowBuilder().apply { setTitle("A fantastic window!") })
                                ControlFlow.WaitUntil(durationMs = 1_000u)
                            } else if (event.cause == TaoStartCause.ResumeTimeReached) {
                                println("\nTimer\n")
                                ControlFlow.WaitUntil(durationMs = 1_000u)
                            } else {
                                ControlFlow.Wait
                            }
                        }

                        is TaoEvent.WindowEvent ->
                            if (event.event == TaoWindowEvent.CloseRequested) ControlFlow.Exit else ControlFlow.Wait

                        else -> ControlFlow.Wait
                    }
                }
            },
        )
    }

    private fun transparent() {
        taoRun(
            object : TaoEventHandler {
                private var window: Window? = null

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    when (event) {
                        is TaoEvent.NewEvents -> if (event.cause == TaoStartCause.Init && window == null) {
                            window = app.createWindow(
                                WindowBuilder().apply {
                                    setDecorations(false)
                                    setTransparent(true)
                                    setTitle("A fantastic window!")
                                },
                            )
                        }

                        is TaoEvent.WindowEvent -> if (event.event == TaoWindowEvent.CloseRequested) return ControlFlow.Exit
                        else -> {}
                    }
                    return ControlFlow.Wait
                }
            },
        )
    }

    private fun videoModes() {
        taoRun(
            object : TaoEventHandler {
                private var done = false

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    if (done) return ControlFlow.Exit
                    if (event is TaoEvent.NewEvents && event.cause == TaoStartCause.Init) {
                        done = true
                        val monitor = app.primaryMonitor()
                        if (monitor == null) {
                            println("No primary monitor detected.")
                            return ControlFlow.Exit
                        }
                        println("Listing available video modes:")
                        monitor.videoModes().forEach { println(it.displayString()) }
                        return ControlFlow.Exit
                    }
                    return ControlFlow.Wait
                }
            },
        )
    }

    private fun windowDebug() {
        println("debugging keys:")
        println("  (E) Enter exclusive fullscreen")
        println("  (F) Toggle borderless fullscreen")
        println("  (P) Toggle borderless fullscreen on system's preferred monitor")
        println("  (V) Toggle visibility")
        println("  (T) Toggle always on top")
        println("  (B) Toggle always on bottom")
        println("  (C) Toggle content protection")
        println("  (R) Toggle resizable")
        println("  (M) Toggle minimized")
        println("  (X) Toggle maximized")
        println("  (Q) Quit event loop")
        println("  (Shift + M) Toggle minimizable")
        println("  (Shift + X) Toggle maximizable")
        println("  (Shift + Q) Toggle closable")

        taoRunWithConfig(
            RunConfig(deviceEventFilter = DeviceEventFilter.NEVER),
            object : TaoEventHandler {
                private var window: Window? = null
                private var alwaysOnBottom = false
                private var alwaysOnTop = false
                private var visible = true
                private var contentProtection = false
                private var resizable = false

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    when (event) {
                        is TaoEvent.NewEvents -> if (event.cause == TaoStartCause.Init && window == null) {
                            window = app.createWindow(
                                WindowBuilder().apply {
                                    setTitle("A fantastic window!")
                                    setInnerSize(LogicalSize(100.0, 100.0))
                                },
                            )
                        }

                        is TaoEvent.DeviceEvent -> when (val de = event.event) {
                            is TaoDeviceEvent.Key -> {
                                val w = window ?: return ControlFlow.Wait
                                if (de.event.state != ElementState.RELEASED) return ControlFlow.Wait
                                when (de.event.physicalKey) {
                                    KeyCode.KeyM -> if (w.isMinimized()) {
                                        w.setMinimized(false)
                                        w.setFocus()
                                    }

                                    KeyCode.KeyV -> if (!visible) {
                                        visible = true
                                        w.setVisible(true)
                                    }

                                    else -> {}
                                }
                            }

                            else -> {}
                        }

                        is TaoEvent.WindowEvent -> when (val we = event.event) {
                            TaoWindowEvent.CloseRequested -> return ControlFlow.Exit
                            is TaoWindowEvent.KeyboardInput -> {
                                val w = window ?: return ControlFlow.Wait
                                val keyEvent = we.event
                                if (keyEvent.state != ElementState.RELEASED) return ControlFlow.Wait
                                val logical = keyEvent.logicalKey
                                val keyStr = (logical as? Key.Character)?.value ?: return ControlFlow.Wait

                                when (keyStr) {
                                    "e" -> {
                                        val monitor = w.currentMonitor()
                                        val modes = monitor?.videoModes().orEmpty()
                                        val best = modes.maxByOrNull { it.size().width.toLong() * it.size().height.toLong() }
                                        if (best != null) w.setFullscreen(Fullscreen.Exclusive(best)) else println("no video modes available")
                                    }

                                    "f" -> if (w.fullscreen() != null) {
                                        w.setFullscreen(null)
                                    } else {
                                        w.setFullscreen(Fullscreen.Borderless(w.currentMonitor()))
                                    }

                                    "p" -> if (w.fullscreen() != null) {
                                        w.setFullscreen(null)
                                    } else {
                                        w.setFullscreen(Fullscreen.Borderless(null))
                                    }

                                    "r" -> {
                                        resizable = !resizable
                                        w.setResizable(resizable)
                                        println("Resizable: $resizable")
                                    }

                                    "m" -> w.setMinimized(!w.isMinimized())
                                    "q" -> return ControlFlow.Exit
                                    "v" -> {
                                        visible = !visible
                                        w.setVisible(visible)
                                    }

                                    "x" -> w.setMaximized(!w.isMaximized())
                                    "t" -> {
                                        alwaysOnTop = !alwaysOnTop
                                        w.setAlwaysOnTop(alwaysOnTop)
                                    }

                                    "b" -> {
                                        alwaysOnBottom = !alwaysOnBottom
                                        w.setAlwaysOnBottom(alwaysOnBottom)
                                    }

                                    "c" -> {
                                        contentProtection = !contentProtection
                                        w.setContentProtection(contentProtection)
                                    }

                                    "M" -> w.setMinimizable(!w.isMinimizable())
                                    "X" -> w.setMaximizable(!w.isMaximizable())
                                    "Q" -> w.setClosable(!w.isClosable())
                                }
                            }

                            else -> {}
                        }

                        else -> {}
                    }
                    return ControlFlow.Wait
                }
            },
        )
    }

    private fun windowIcon() {
        taoRun(
            object : TaoEventHandler {
                private var window: Window? = null

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    when (event) {
                        is TaoEvent.NewEvents -> if (event.cause == TaoStartCause.Init && window == null) {
                            val iconPath = "third_party/tao/examples/icon.png"
                            val icon = Icon.fromFile(iconPath)
                            window = app.createWindow(
                                WindowBuilder().apply {
                                    setTitle("An iconic window!")
                                    setWindowIcon(icon)
                                },
                            )
                        }

                        is TaoEvent.WindowEvent -> when (val we = event.event) {
                            TaoWindowEvent.CloseRequested -> return ControlFlow.Exit
                            is TaoWindowEvent.DroppedFile -> {
                                val icon = Icon.fromFile(we.path)
                                window?.setWindowIcon(icon)
                            }

                            else -> {}
                        }

                        else -> {}
                    }
                    return ControlFlow.Wait
                }
            },
        )
    }

    private fun windowRunReturn() {
        taoRunReturnLoop(
            object : TaoRunReturnHandler {
                private var window: Window? = null
                @Volatile private var quit = false

                override fun handleEvent(event: TaoEvent, app: App): ControlFlow {
                    when (event) {
                        is TaoEvent.NewEvents -> if (event.cause == TaoStartCause.Init && window == null) {
                            window = app.createWindow(WindowBuilder().apply { setTitle("A fantastic window!") })
                        }

                        is TaoEvent.WindowEvent -> {
                            println(event.event)
                            if (event.event == TaoWindowEvent.CloseRequested) {
                                quit = true
                            }
                        }

                        TaoEvent.MainEventsCleared -> return ControlFlow.Exit
                        else -> {}
                    }
                    return ControlFlow.Wait
                }

                override fun render() {
                    println("rendering")
                    Thread.sleep(16)
                }

                override fun shouldQuit(): Boolean = quit
            },
        )
    }
}
