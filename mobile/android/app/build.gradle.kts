plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id("dev.flutter.flutter-gradle-plugin")
}

android {
    namespace = "tools.knowme.knowme"
    compileSdk = flutter.compileSdkVersion
    ndkVersion = flutter.ndkVersion

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    defaultConfig {
        applicationId = "tools.knowme.knowme"
        // You can update the following values to match your application needs.
        // For more information, see: https://flutter.dev/to/review-gradle-config.
        // KnowMe targets current-generation phones. Android 10+ lets the app
        // rely on modern storage, media, security, and accessibility behavior.
        minSdk = 29
        targetSdk = flutter.targetSdkVersion
        versionCode = flutter.versionCode
        versionName = flutter.versionName

        // ort (ONNX Runtime, via fastembed) ships prebuilt binaries only for
        // arm64-v8a on Android — see gen_ui_db_graph's embed-native feature.
        // Neither i686 nor x86_64 (emulator ABIs) are supported, so this
        // project targets real arm64 devices only until that changes.
        ndk {
            abiFilters += listOf("arm64-v8a")
        }
    }

    buildTypes {
        release {
            // TODO: Add your own signing config for the release build.
            // Signing with the debug keys for now, so `flutter run --release` works.
            signingConfig = signingConfigs.getByName("debug")
            // R8 runs on release and shrinks anything it cannot prove reachable.
            // The LiteRT-LM bridge is reached only from Rust via JNI reflection,
            // so without these keep rules it is stripped and release builds fail
            // at load with ClassNotFoundException (debug is unaffected because
            // R8 does not run there). See proguard-rules.pro.
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro",
            )
        }
    }

    packaging {
        jniLibs {
            excludes += setOf(
                "lib/armeabi-v7a/**",
                "lib/x86/**",
                "lib/x86_64/**",
            )
        }
    }
}

kotlin {
    compilerOptions {
        jvmTarget = org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17
    }
}

flutter {
    source = "../.."
}

dependencies {
    // Pinned from Google Maven metadata (latest/release 0.14.0, 2026-07-08).
    // Rust remains the inference-policy owner; this AAR supplies the official
    // Android LiteRT-LM classes and native JNI library that Rust calls through
    // the JavaVM captured by gen_ui_ffi.
    implementation("com.google.ai.edge.litertlm:litertlm-android:0.14.0")
}
