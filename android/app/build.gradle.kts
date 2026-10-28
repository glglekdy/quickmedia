plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
}

// 릴리즈 태그(v1.1.0)를 -PappVersion=1.1.0 으로 받는다. versionCode 는 1.1.0 → 10100.
val appVersion = (findProperty("appVersion") as String?) ?: "0.0.0-dev"
val versionParts = Regex("""^(\d+)\.(\d+)\.(\d+)""").find(appVersion)?.destructured
// 버전이 없으면(0.0.0-dev) 1. versionCode 는 0 이 될 수 없다.
val appVersionCode = versionParts?.let { (a, b, c) -> a.toInt() * 10000 + b.toInt() * 100 + c.toInt() }
    ?.takeIf { it > 0 } ?: 1

android {
    namespace = "com.glglekdy.quickmedia"
    compileSdk = 37

    defaultConfig {
        applicationId = "com.glglekdy.quickmedia"
        minSdk = 29
        targetSdk = 37
        versionCode = appVersionCode
        versionName = appVersion
    }

    buildTypes {
        debug {
            // x86_64 는 PC 에뮬레이터용
            ndk { abiFilters += listOf("arm64-v8a", "x86_64") }
        }
        release {
            isMinifyEnabled = false
            ndk { abiFilters += listOf("arm64-v8a") }
        }
    }

    buildFeatures {
        compose = true
        buildConfig = true
    }

    // youtubedl-android 는 압축된 파이썬·ffmpeg 를 .so 로 넣어 두고 설치 폴더에서 푼다
    packaging {
        jniLibs { useLegacyPackaging = true }
    }

    // 16KB 페이지 기기에서 ffmpeg 가 뜨게 바꿔 넣을 libwebp. third_party/libwebp-16k/README.md
    sourceSets["main"].jniLibs.directories.add("../third_party/libwebp-16k/jniLibs")
}

dependencies {
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.material3)
    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.lifecycle.runtime.compose)
    implementation(libs.kotlinx.coroutines.android)
    implementation(libs.youtubedl.library)
    implementation(libs.youtubedl.ffmpeg)

    testImplementation(libs.junit)
    // 안드로이드의 org.json 은 JVM 단위 테스트에서 빈 껍데기다
    testImplementation(libs.json)
}
