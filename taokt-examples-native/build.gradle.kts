import org.jetbrains.kotlin.gradle.plugin.mpp.KotlinNativeTarget
import org.jetbrains.kotlin.konan.target.Family
import org.jetbrains.kotlin.konan.target.HostManager

plugins {
    alias(libs.plugins.kotlinMultiplatform)
}

group = "io.github.kdroidfilter.taokt"
version = "0.1.0-SNAPSHOT"

val enableAllNativeTargets =
    providers.gradleProperty("taokt.enableAllNativeTargets").map(String::toBoolean).orElse(false)

kotlin {
    linuxX64()
    linuxArm64()
    macosX64()
    macosArm64()
    mingwX64()

    sourceSets {
        commonMain.dependencies {
            implementation(projects.taoktBindings)
            implementation(libs.kotlinx.coroutinesCore)
        }
    }

    targets.withType<KotlinNativeTarget>().configureEach {
        binaries.executable {
            entryPoint = "io.github.kdroidfilter.taokt.examples.main"

            if (konanTarget.family == Family.LINUX) {
                linkerOpts(
                    "-L/usr/lib/x86_64-linux-gnu",
                    "-L/lib/x86_64-linux-gnu",
                    "-L/usr/lib/aarch64-linux-gnu",
                    "-L/lib/aarch64-linux-gnu",
                    "-L/usr/lib64",
                    "-L/lib64",
                    "-L/usr/lib",
                    "-L/lib",
                )
            }
        }

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
        return setOf(name.replaceFirstChar(Char::uppercaseChar))
    }

    val disabledTargetNameTokens = kotlin.targets.withType<KotlinNativeTarget>()
        .filter { it.konanTarget != hostKonanTarget }
        .flatMap { it.taskNameTokens() }
        .toSet()

    tasks.matching { task ->
        disabledTargetNameTokens.any(task.name::contains) && (
            task.name.startsWith("compileKotlin")
                || task.name.startsWith("cinterop")
                || task.name.startsWith("link")
                || task.name.startsWith("run")
        )
    }.configureEach {
        enabled = false
    }
}
