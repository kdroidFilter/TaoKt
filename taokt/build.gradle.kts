import gobley.gradle.GobleyHost
import gobley.gradle.cargo.dsl.jvm
import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.jetbrains.kotlin.gradle.plugin.mpp.KotlinNativeTarget
import org.jetbrains.kotlin.konan.target.HostManager
import org.jetbrains.kotlin.konan.target.KonanTarget

plugins {
    alias(libs.plugins.gobleyCargo)
    alias(libs.plugins.gobleyUniffi)
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.atomicfu)
}

group = "io.github.kdroidfilter.taokt"
version = "0.1.0-SNAPSHOT"

val enableAllNativeTargets =
    providers.gradleProperty("taokt.enableAllNativeTargets").map(String::toBoolean).orElse(false)

cargo {
    packageDirectory = layout.projectDirectory
    builds.jvm {
        // Build Rust library only for the current host platform by default.
        embedRustLibrary = (rustTarget == GobleyHost.current.rustTarget)
    }
}

uniffi {
    generateFromLibrary {
        namespace = "taokt"
        packageName = "io.github.kdroidfilter.taokt.tao"
    }
}

kotlin {
    jvm()
    linuxX64()
    linuxArm64()
    macosX64()
    macosArm64()
    mingwX64()

    jvmToolchain(17)

    jvm {
        compilerOptions {
            jvmTarget = JvmTarget.JVM_17
        }
    }

    targets.withType<KotlinNativeTarget>().configureEach {
        val isHostTarget = (konanTarget == HostManager.host)
        if (!enableAllNativeTargets.get() && !isHostTarget) {
            compilations.configureEach {
                compileTaskProvider.configure { enabled = false }
                cinterops.configureEach {
                    tasks.named(interopProcessingTaskName).configure { enabled = false }
                }
            }
        }
    }
}

if (!enableAllNativeTargets.get()) {
    val hostKonanTarget = HostManager.host

    fun KotlinNativeTarget.taskNameTokens(): Set<String> {
        val kotlinToken = name.replaceFirstChar(Char::uppercaseChar)
        val rustToken = when (konanTarget) {
            KonanTarget.LINUX_X64 -> "LinuxX64"
            KonanTarget.LINUX_ARM64 -> "LinuxArm64"
            KonanTarget.MACOS_X64 -> "MacOSX64"
            KonanTarget.MACOS_ARM64 -> "MacOSArm64"
            KonanTarget.MINGW_X64 -> "MinGWX64"
            else -> null
        }
        return buildSet {
            add(kotlinToken)
            if (rustToken != null) add(rustToken)
        }
    }

    val disabledTargetNameTokens = kotlin.targets.withType<KotlinNativeTarget>()
        .filter { it.konanTarget != hostKonanTarget }
        .flatMap { it.taskNameTokens() }
        .toSet()

    tasks.matching { task ->
        disabledTargetNameTokens.any(task.name::contains) && (
            task.name.startsWith("cargoBuild")
                || task.name.startsWith("cargoCheck")
                || task.name.startsWith("rustUpTargetAdd")
                || task.name.startsWith("cinterop")
        )
    }.configureEach {
        enabled = false
    }
}
