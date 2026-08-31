// Cranpose Orbit, built through the Cranpose Gradle plugin.
//
// The native build, the ABIs, the Cargo profiles, the JNI packaging, the
// framework's Java and its manifest contributions all come from the plugin.
// What remains is what is specific to this application.
plugins {
    id("com.android.application")
    id("dev.cranpose.android")
}

// Debug builds default to the classic emulator ABI. On-device or
// Apple-Silicon-emulator work overrides it:
//   -PorbitAbi=arm64-v8a   build the arm64 library instead
val debugBuildAbis: List<String> = (project.findProperty("orbitAbi") as String?)
    ?.split(",")?.map(String::trim)?.filter(String::isNotEmpty)
    ?: listOf("x86_64")

cranpose {
    // This app is its own Cargo workspace, two directories above the Gradle
    // project.
    workspaceRoot.set("../..")
    cargoPackage.set("cranpose-orbit")
    label.set("Orbit")
    debugAbis.set(debugBuildAbis)
}

android {
    namespace = "com.cranpose.orbit"
    compileSdk = 36

    defaultConfig {
        applicationId = "com.cranpose.orbit"
        minSdk = 26
        targetSdk = 36
        versionCode = 2
        versionName = "0.1.1"
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
            signingConfig = signingConfigs.getByName("debug")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}
