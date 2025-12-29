package io.github.kdroidfilter.taokt.examples

enum class HostOs {
    Windows,
    Mac,
    Linux,
    Other,
}

expect object Platform {
    val hostOs: HostOs
    fun sleepMillis(ms: Long)
}

