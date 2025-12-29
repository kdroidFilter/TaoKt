import gobley.gradle.GobleyHost
import gobley.gradle.cargo.dsl.jvm
import org.gradle.jvm.toolchain.JavaLanguageVersion
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.gobleyCargo)
    alias(libs.plugins.gobleyUniffi)
    kotlin("jvm")
    alias(libs.plugins.atomicfu)
}

group = "io.github.kdroidfilter.taokt"
version = "0.1.0-SNAPSHOT"

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
    compilerOptions {
        jvmTarget = JvmTarget.JVM_17
    }
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(17))
    }
}
