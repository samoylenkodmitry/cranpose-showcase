// The Wear OS app, built through the Cranpose Gradle plugin from the same
// Rust library as the phone app. The `watch` Cargo feature makes the watch
// app (`src/watch`) the root composable.
plugins {
    id("com.android.application")
    id("dev.cranpose.android")
}

// Debug builds default to the emulator ABI. A watch takes another one:
//   -PshowcaseAbi=armeabi-v7a   Pixel Watch and other 32-bit Wear OS watches
//   -PshowcaseAbi=arm64-v8a     64-bit watches, Apple Silicon emulators
val debugBuildAbis: List<String> = (project.findProperty("showcaseAbi") as String?)
    ?.split(",")?.map(String::trim)?.filter(String::isNotEmpty)
    ?: listOf("x86_64")

cranpose {
    workspaceRoot.set("../..")
    cargoPackage.set("cranpose-showcase")
    label.set("Showcase")
    features.set(listOf("android", "renderer-wgpu", "watch"))
    debugAbis.set(debugBuildAbis)
    // Many Wear OS watches run 32-bit ARM. Google Play takes a Wear OS app
    // with native code only when it also carries the 64-bit library.
    releaseAbis.set(listOf("armeabi-v7a", "arm64-v8a"))
    // The manifest asks for a watch, so Play offers this APK to watches only.
    requiredFeatures.add("android.hardware.type.watch")
}

android {
    namespace = "com.cranpose.showcase.wear"
    compileSdk = 36

    defaultConfig {
        // The phone app's id: Play lists the phone and the watch app as one
        // app, with the Wear OS build in its own release track.
        applicationId = "com.cranpose.showcase"
        // Wear OS 3.
        minSdk = 30
        targetSdk = 36
        // Play numbers every build of one application id in one sequence,
        // so the watch build adds a million to the phone's version code.
        versionCode = 1_000_131
        versionName = "0.1.31"
    }

    // The phone app's launcher icon.
    sourceSets.getByName("main").res.srcDir("../app/src/main/res")

    buildTypes {
        release {
            isMinifyEnabled = false
            signingConfig = signingConfigs.getByName("debug")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}
