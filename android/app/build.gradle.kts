import java.util.Properties

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

fun parseSemverCode(ver: String): Int {
    val parts = ver.removePrefix("v").removePrefix("V")
        .substringBefore("-")
        .split(".")
        .take(3)
        .map { it.toIntOrNull() ?: 0 }
    val major = parts.getOrElse(0) { 1 }
    val minor = parts.getOrElse(1) { 0 }
    val patch = parts.getOrElse(2) { 0 }
    return (major * 10000 + minor * 100 + patch).coerceAtLeast(120)
}

val appVersionName: String = (project.findProperty("LIGHTRAW_VERSION_NAME") as String?)
    ?.takeIf { it.isNotBlank() }
    ?: System.getenv("LIGHTRAW_VERSION_NAME")?.takeIf { it.isNotBlank() }
    ?: "1.2.0"
val appVersionCode: Int = parseSemverCode(appVersionName)

val localKeystorePropsFile = rootProject.file("keystore.properties")
val localKeystoreProps = Properties().apply {
    if (localKeystorePropsFile.isFile) {
        localKeystorePropsFile.inputStream().use { load(it) }
    }
}

val releaseKeystorePath = System.getenv("LIGHTRAW_KEYSTORE_PATH")
    ?.takeIf { it.isNotBlank() }
    ?: localKeystoreProps.getProperty("storeFile")?.takeIf { it.isNotBlank() }
val releaseKeystoreFile = releaseKeystorePath?.let { path ->
    val f = file(path)
    if (f.isFile) f else rootProject.file(path).takeIf { it.isFile }
}
val releaseKeystorePassword = System.getenv("LIGHTRAW_KEYSTORE_PASSWORD")
    ?.takeIf { it.isNotBlank() }
    ?: localKeystoreProps.getProperty("storePassword")
val releaseKeyAlias = System.getenv("LIGHTRAW_KEY_ALIAS")
    ?.takeIf { it.isNotBlank() }
    ?: localKeystoreProps.getProperty("keyAlias")
val releaseKeyPassword = System.getenv("LIGHTRAW_KEY_PASSWORD")
    ?.takeIf { it.isNotBlank() }
    ?: localKeystoreProps.getProperty("keyPassword")

android {
    namespace = "com.moestret.lightraw"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.moestret.lightraw"
        minSdk = 26
        targetSdk = 35
        versionCode = appVersionCode
        versionName = appVersionName

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    signingConfigs {
        if (
            releaseKeystoreFile != null &&
            !releaseKeystorePassword.isNullOrBlank() &&
            !releaseKeyAlias.isNullOrBlank() &&
            !releaseKeyPassword.isNullOrBlank()
        ) {
            create("release") {
                storeFile = releaseKeystoreFile
                storePassword = releaseKeystorePassword
                keyAlias = releaseKeyAlias
                keyPassword = releaseKeyPassword
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
            signingConfig = signingConfigs.findByName("release")
                ?: signingConfigs.getByName("debug")
        }
        debug {
            applicationIdSuffix = ".debug"
            isDebuggable = true
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    buildFeatures {
        viewBinding = true
    }

    packaging {
        resources {
            excludes += "/META-INF/{AL2.0,LGPL2.1}"
        }
    }
}

dependencies {
    implementation("androidx.core:core-ktx:1.15.0")
    implementation("androidx.appcompat:appcompat:1.7.0")
    implementation("com.google.android.material:material:1.12.0")
    implementation("androidx.constraintlayout:constraintlayout:2.2.0")
    implementation("androidx.activity:activity:1.9.3")
    implementation("androidx.recyclerview:recyclerview:1.3.2")
    implementation("androidx.documentfile:documentfile:1.0.0")
    implementation("androidx.exifinterface:exifinterface:1.3.7")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.6.4")

    testImplementation("junit:junit:4.13.2")
}
