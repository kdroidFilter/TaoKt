import org.gradle.internal.os.OperatingSystem
import org.gradle.jvm.toolchain.JavaLanguageVersion
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.gobleyRust)
    kotlin("jvm")
    application
}

group = "io.github.kdroidfilter.taokt"
version = "0.1.0-SNAPSHOT"

dependencies {
    implementation(projects.taoktBindings)
}

application {
    mainClass.set("io.github.kdroidfilter.taokt.examples.MainKt")
    if (OperatingSystem.current().isMacOsX) {
        applicationDefaultJvmArgs = listOf("-XstartOnFirstThread")
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
