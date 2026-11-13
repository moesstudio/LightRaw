# Architecture: LightRaw Android (`com.moestret.lightraw`)

Target Benchmark: Adobe Lightroom Mobile `12.0.01` (`adobe-lightroom-12-0-01.xapk`, clean-room structural analysis).
Goal: Offline-first, zero-telemetry Android photography catalog & non-destructive RAW/photo editor sharing exact math and `.lrcat`/`.xmp` schema compatibility with LightRaw Desktop.

---

## 1. Technology Stack

| Layer | Choice | Rationale |
| --- | --- | --- |
| Language & UI | Kotlin 2.0 + AndroidX (`AppCompat`, `Material 3`, `ConstraintLayout`, `RecyclerView`, `Activity KTX`, `Coroutines`) | Deterministic builds with zero external proprietary runtime bloat; native touch gestures and smooth 60fps Canvas loupe rendering. |
| Package & SDK | `com.moestret.lightraw`, `minSdk = 26`, `compileSdk = 35`, `targetSdk = 35` | Matches observed `minSdk 26` of modern Android RAW photography devices while aligning cleanly with locally installed Android SDK 35 & AGP `8.7.3`. |
| Dual Processing Core | **Tier 1 (Optional NDK Fast-Path)**: Rust `lightraw` shared library + JNI crate (`rust/lightraw_jni` -> `liblightraw_jni.so`)<br>**Tier 2 (Built-in Standalone Engine)**: `KotlinPv2012Engine` + Parallel Coroutine Tile Pipeline | Guarantees the `.apk` compiles cleanly on any machine (even without local Android NDK) while delivering identical 11-stage non-destructive PV2012 color math, Tone Curve splines, 8-band HSL, Color Grading, Vignette/Grain/Sharpen, Masking, AI Auto Tone, and AI Color Match. CI builds bundle native `.so` libraries automatically. |
| RAW / DNG & Bitmap I/O | Android `ImageDecoder` / `BitmapFactory` (ARGB_8888 & RGBA_F16 when hardware supports) + Camera2 `RAW_SENSOR` DNG capture + EXIF (`ExifInterface`) | Native platform decoding on Android 8+ without bundling 70MB+ proprietary native blobs. |
| Local Catalog DB | Android `SQLiteOpenHelper` (`LightRawCatalog.lrcat`, WAL mode) | Preserves core `Adobe_images`, `Adobe_imageDevelopSettings`, `AgDevelopPreset`, `Adobe_libraryImageDevelopHistoryStep`, and `AgLibraryFolder` schema so develop states stay interoperable with desktop LightRaw. |
| Presets & Settings | Adobe Camera Raw `.xmp` XML parser & serializer (`XmpPresetCodec.kt`) | Bi-directional `.xmp` import/export supporting `crs:Temperature`, `crs:Exposure2012`, `crs:ToneCurvePV2012*`, `crs:HueAdjustment*`, `crs:SaturationAdjustment*`, `crs:LuminanceAdjustment*`, `crs:SplitToning*`, `crs:ColorGrade*`, `crs:Sharpness`, `crs:PostCropVignette*`, `crs:Grain*`. |

---

## 2. Module Layout

```
LightRaw/
├── src/                        # Shared Desktop + Core Rust crate (lib.rs + main.rs)
├── rust/
│   └── lightraw_jni/           # Optional Android NDK JNI bridge (cdylib -> liblightraw_jni.so)
├── android/
│   ├── settings.gradle.kts
│   ├── build.gradle.kts
│   └── app/
│       ├── build.gradle.kts
│       └── src/main/
│           ├── AndroidManifest.xml
│           ├── java/com/moestret/lightraw/
│           │   ├── model/      # DevelopParams.kt, PhotoAsset.kt, PresetModels.kt, CopyMask.kt, ExportOptions.kt
│           │   ├── engine/     # KotlinPv2012Engine.kt, AiColorMatchEngine.kt, NativeBridge.kt, WatermarkRenderer.kt
│           │   ├── data/       # CatalogDbHelper.kt (.lrcat SQLite), XmpPresetCodec.kt, BuiltinPresets.kt, MediaStoreImporter.kt
│           │   ├── ui/         # MainActivity.kt (Library Grid), LoupeEditorActivity.kt (Studio),
│           │   │               # LoupeImageView.kt (Pinch/Zoom/Crop/Mask/Before-After Canvas),
│           │   │               # ToneCurveView.kt, HistogramView.kt, ProCameraActivity.kt,
│           │   │               # WatermarkDialog.kt, ExportDialog.kt, CopySettingsDialog.kt, AiMatchDialog.kt
│           └── res/            # Material 3 darkroom themes, vector icons, bottom sheet layouts
└── .github/workflows/android.yml # Builds signed Debug & Release APKs + Rust ARM64/x86_64 JNI .so libs
```

---

## 3. Non-Destructive Pipeline Stages (Android & Rust Shared Math)

1. **White Balance**: Planckian CIE 1960 UCS `(Temperature 2000..50000 K, Tint -150..+150)` relative chromatic adaptation gains `(rScale, gScale, bScale)`.
2. **Exposure & Highlight Roll-off**: `2^(exposure_2012)` linear gain + soft shoulder compression preventing harsh digital clipping on portraits/RAW skies.
3. **Basic Tone (PV2012)**: Highlights, Shadows, Whites, Blacks, S-curve Contrast (`-100..+100`) mapped cleanly in linear/perceptual space.
4. **Presence**: High-frequency Texture/Clarity micro-contrast + Dehaze black-point/contrast recovery + skin-preserving Vibrance & Saturation.
5. **Tone Curve**: 256-entry LUT built from Parametric (`Highlights/Lights/Darks/Shadows`) + Monotone Cubic Hermite Spline (`toneCurveMaster`, `Red`, `Green`, `Blue`).
6. **Color Mixer (8-Band HSL / B&W)**: Smooth angular bell weights across Red, Orange, Yellow, Green, Aqua, Blue, Purple, Magenta + B&W channel luminosity weights.
7. **Color Grading / Split Toning**: Shadows, Midtones, Highlights, Global hue/sat/lum wheels with tonal balance & blending controls.
8. **Detail & Optics**: Unsharp Mask sharpen pass, Edge-preserving luminance noise smoothing, Optical vignette compensation.
9. **Selective Masking**: Composite Linear Gradient, Radial Gradient, and Freehand Brush masks applying per-mask localized Exposure/Temp/Contrast/Shadows/Saturation/Clarity.
10. **Effects**: Post-Crop Vignette (Amount/Midpoint/Roundness/Feather) & film Grain synthesis.
11. **Geometry & Watermark**: Orthogonal 90-deg rotation, H/V flip, arbitrary straighten rotation (`-45..+45 deg`), aspect crop rectangle, and text/anchor watermark compositing on final export.
