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

// jpackage configuration for macOS .app bundle with modern UI support
if (OperatingSystem.current().isMacOsX) {
    tasks.register<Exec>("jpackageApp") {
        dependsOn("installDist")

        val appName = "TaoKtExamples"
        val outputDir = layout.buildDirectory.dir("jpackage").get().asFile
        val inputDir = layout.buildDirectory.dir("install/taokt-examples/lib").get().asFile
        val resourceDir = file("src/main/resources/macos")
        val mainJar = "taokt-examples-${version}.jar"

        doFirst {
            outputDir.mkdirs()
        }

        commandLine(
            "jpackage",
            "--type", "app-image",
            "--name", appName,
            "--input", inputDir.absolutePath,
            "--main-jar", mainJar,
            "--main-class", "io.github.kdroidfilter.taokt.examples.MainKt",
            "--dest", outputDir.absolutePath,
            "--java-options", "-XstartOnFirstThread",
            "--mac-package-identifier", "io.github.kdroidfilter.taokt.examples",
            "--mac-package-name", "TaoKt Examples",
            "--resource-dir", resourceDir.absolutePath,
            "--verbose"
        )
    }

    tasks.register<Exec>("runApp") {
        dependsOn("jpackageApp")

        val appPath = layout.buildDirectory.file("jpackage/TaoKtExamples.app/Contents/MacOS/TaoKtExamples").get().asFile

        commandLine(appPath.absolutePath, *project.findProperty("appArgs")?.toString()?.split(" ")?.toTypedArray() ?: arrayOf("window"))
    }
}
