package io.github.kdroidfilter.taokt.examples

import kotlin.native.OsFamily
import kotlin.native.Platform
import platform.posix.usleep
import kotlin.experimental.ExperimentalNativeApi

actual object Platform {
    @OptIn(ExperimentalNativeApi::class)
    actual val hostOs: HostOs =
        when (Platform.osFamily) {
            OsFamily.WINDOWS -> HostOs.Windows
            OsFamily.MACOSX -> HostOs.Mac
            OsFamily.LINUX -> HostOs.Linux
            else -> HostOs.Other
        }

    actual fun sleepMillis(ms: Long) {
        usleep((ms.coerceAtLeast(0) * 1000).toUInt())
    }
}

