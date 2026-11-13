# LightRaw Android — Clean-Room Recon Map

> Clean-room architectural analysis of `adobe-lightroom-12-0-01.xapk` (`com.adobe.lrmobile`, version `12.0.01`, `versionCode 711200010`) mapped to **LightRaw Android** (`com.moestret.lightraw`).
> Strictly local-first mobile photography workflow: zero proprietary assets, zero Adobe copy/fonts/LUT PNGs, zero cloud/billing/licensing/telemetric dependencies.

---

## 1. XAPK Package & Runtime Findings (`adobe-lightroom-12-0-01.xapk`)

### Archive Split Structure (50 ZIP Entries)
- **Base APK (`com.adobe.lrmobile.apk`, 112.8 MB)**: Java/Kotlin UI shell, 1,021 XML layouts, vector/menu resources, collage templates, shader assets.
  - `minSdkVersion`: `26` (Android 8.0 Oreo)
  - `targetSdkVersion` / `compileSdkVersion`: `36`
  - Graphics requirement: `uses-gl-es 0x20000` (OpenGL ES 2.0+)
- **Native ABI Split (`config.arm64_v8a.apk`, 159.0 MB)**:
  - Core image/RAW engine: `libLrAndroid.so`, `libimaging.so`, `libkernel.so`, `liblua-rapidjson.so`, `libopencv_java4.so`, `libcapture.so` (Camera2 RAW/DNG pipeline)
  - On-device AI runtime: `libLiteRt.so`, `libLiteRtClGlAccelerator.so`, `libLrmModels.so`, `libml.so`
  - Media codecs & storage: `libjxl*.so` (JPEG XL), `libwebp*.so`, `libwfsqlite.so` (local SQLite catalog), `libvfcommonlib-native-lib.so`, `libvfexporterlib-native-lib.so`, `libvfmetadatareaderlib-native-lib.so`
  - **Excluded (Out-of-Scope Cloud/Licensing/Telemetry)**: `liboz_client.so`, `libAdobeGrowthSDK.so`, `libpairipcore.so`, `libadobe_c2pa.so`, `libcrashlytics*.so`
- **Density & Locale Splits**: 6 DPI APKs (`mdpi`..`xxxhdpi`) + 42 locale APKs (`en`, `in`, `ms`, `ja`, etc.).

### Intent Filters & Supported Formats Observed
- Handles `image/*`, `*/*`, and raw/image extensions including `.DNG`, `.ARW`, `.CR2`, `.CR3`, `.CRW`, `.NEF`, `.RAF`, `.ORF`, `.RW2`, `.PEF`, `.SRW`, `.3FR`, `.IIQ`, `.FFF`, `.DCR`, `.ERF`, `.KDC`, `.JXL`, `.AVIF`.
- Supports external share/open intents (`ImportGalleryActivity`, `AddToLrActivity`, `OpenInLrActivity`) and USB camera transfer (`PtpActivity`, `PtpDeviceReceiver`).

### Functional Layout Modules Observed in `res/menu` & `res/layout`
1. **Bottom Navigation (`bottom_navigation_menu.xml`)**: Hub (`Device/Library`, `Catalog/Photos`, `Quick Capture`, `Shared/Export`, `Tools/Learn`) — streamlined in LightRaw Android to 3 focused offline hubs (`Library`, `Studio/Loupe`, `Presets/Tools` + floating `Pro Camera` trigger).
2. **Loupe Editor Shell (`loupe_layout_main.xml`, `loupeview_bottombar.xml`, `loupe_bottom_sheets_container.xml`)**:
   - Top Bar: Back, Histogram HUD toggle, Undo/Redo, Copy/Paste Settings (`dialog_loupe_copyoptions.xml`), Info/EXIF (`loupe_info_view_*`), More menu (`Versions`, `Compare`, `Reset`), Export (`export_settings_editor_layout.xml`).
   - Canvas: Pinch-to-zoom, pan, press-and-hold **Before/After** toggle, clipping overlay indicators, interactive Crop/Straighten frame, interactive Mask overlay.
   - Bottom Horizontal Tool Rail -> Expandable Bottom Sheets:
     - **Match AI / Auto**: One-tap Auto Tone + AI Reference Color Match (`Match Ref`)
     - **Presets (`all_preset_view.xml`, `presets_filmstrip.xml`, `preset_amount_slider_view.xml`)**: Horizontal filmstrip + categories + amount slider + `.xmp` import/export
     - **Crop & Geometry (`crop_controls_view.xml`, `crop_aspect_sheet.xml`, `geometry_layout.xml`)**: Aspect ratio presets (`1:1`, `4:5`, `2:3`, `4:3`, `16:9`, `Custom`), Straighten dial `-45..+45 deg`, 90-deg rotate, flip H/V, Upright/Perspective V/H
     - **Light (`lights_sliders_ev.xml`, `tone_curve_sheet.xml`)**: White Balance pill + Exposure, Contrast, Highlights, Shadows, Whites, Blacks + Interactive Parametric/Point/RGB Tone Curve
     - **Color (`color_sliders_ev.xml`, `colormixer_layout.xml`, `colorgrading_sheet_ev.xml`, `splittone_layout.xml`)**: Temp/Tint, Vibrance/Saturation + 8-band HSL Color Mixer + Point Color + 3-Way Color Grading Wheels + Profiles (`profile_amount_slider_view.xml`)
     - **Effects (`splittone_layout.xml`, vignette, grain)**: Texture, Clarity, Dehaze, Vignette (Amount/Midpoint/Feather/Roundness), Film Grain (Amount/Size/Roughness)
     - **Detail & Optics (`detail_sliders_ev.xml`, `lens_profile_bottomsheet.xml`)**: Sharpening (Amount/Radius/Detail/Masking), Luminance NR, Color NR, Lens Distortion/Vignetting/CA
     - **Lens Blur (`lens_blur_view.xml`, `bokeh_preset_item.xml`)**: Blur amount, bokeh shape, depth focal range
     - **Masking (`masking_group_detail_item.xml`, `masking_add_subtract_context_options.xml`)**: Radial Gradient, Linear Gradient, Subject/Sky AI Heuristic, Brush strokes with per-mask Light/Color/Presence sliders
     - **Remove / Heal (`heal_mode_view.xml`, `heal_brush_prop_sldier_ev.xml`)**: Spot heal / clone circles
     - **Watermark (`watermark_editor.xml`)**: Text/Graphic anchor (3x3 grid), font scale, opacity, shadow, offset
     - **Versions (`loupe_versions.xml`)**: Saved snapshots per photo

---

## 2. LightRaw Android Screen Inventory

| ID | Screen | Route / Activity | Purpose |
| --- | --- | --- | --- |
| S01 | Library Grid & Albums | `MainActivity` (`LibraryFragment`) | Local catalog grid, Star/Flag/Color filters, Search, Multi-select batch Copy/Paste/Export, Quick Collection & Albums |
| S02 | Device Gallery & Import | `ImportBottomSheet` / `DevicePhotos` | MediaStore device photo picker, RAW/DNG/JPEG filter, SAF `.xmp` & folder import |
| S03 | Touch Loupe Editor | `LoupeEditorActivity` | Full-screen non-destructive editor: live RGB Histogram, Before/After hold, Undo/Redo, bottom tool carousel & interactive slider panels |
| S04 | Tone Curve & Mixer Sheet | `CurveSheet` / `MixerSheet` | Multi-touch cubic point curve (RGB + individual R/G/B channels) + 8-band HSL chip selector |
| S05 | Masking & Heal Overlay | `MaskingOverlayMode` | Interactive canvas gestures for Radial, Linear Gradient, Brush, and Spot Heal annotations with localized develop sliders |
| S06 | Crop & Transform Workspace | `CropOverlayMode` | Draggable corners, rule-of-thirds/golden grid, interactive protractor dial, aspect sheet, rotate/flip, vertical/horizontal keystone |
| S07 | AI Color Match Studio | `AiColorMatchSheet` | Pick Reference Photo from catalog/gallery, analyze luminance/skin/WB stats, preview & transfer matched `DevelopParams` |
| S08 | Presets & `.xmp` Drawer | `PresetsBottomSheet` | Built-in + User `.xmp` presets filmstrip, Amount slider (`0..200%`), Create/Rename/Delete, Import/Export Adobe `.xmp` |
| S09 | Selective Copy & Paste Modal | `CopySettingsDialog` | Categorized checklist (Treatment/WB/Exposure/Tone/Curve/HSL/Grading/Detail/Effects/Transform/Crop/Masking) for single/batch paste |
| S10 | Watermark, Export & Collage | `ExportSheetActivity` / `WatermarkSheet` / `CollageActivity` | Format (JPEG/PNG/WEBP), Long-Edge / Original resize, Quality (`10..100`), Color space, Watermark overlay editor, Multi-photo Collage Grid (1x1, 4:5, 9:16) |
| S11 | Pro RAW Capture | `ProCameraActivity` | Camera2 DNG/RAW + JPEG capture with manual ISO/Shutter/WB lock, live grid & histogram, auto-import into LightRaw Catalog |

---

## 3. Core User Flows

- **F01: Import -> Cull -> Organize (`S02 -> S01`)**:
  User opens app -> grants `READ_MEDIA_IMAGES` -> browses Device folders or clicks `+ Import` -> picks RAW/JPEG/PNG -> stored non-destructively in local SQLite catalog (`LightRawCatalog.lrcat` schema) -> swipe/tap to set 1-5 Stars, Pick/Reject flags, Color labels, or assign to Album.
- **F02: Non-Destructive Loupe Develop (`S01 -> S03 -> S04/S05/S06/S08`)**:
  Tap thumbnail in Grid -> opens Loupe Editor (`S03`) -> press Auto or pick Tool (`Light`, `Curve`, `Color`, `Mixer`, `Grading`, `Effects`, `Detail`, `Optics`, `Crop`, `Masks`, `Heal`, `Presets`) -> drag 1-thumb precision slider -> real-time pipeline renders frame + updates RGB Histogram + appends Undo history -> long-press canvas for Before/After comparison.
- **F03: AI Reference Color Match (`S03 -> S07 -> S03`)**:
  Inside Loupe -> tap `Match Ref` -> select reference photo (JPEG/RAW) -> engine computes perceptual luminance percentiles (`p05..p95`), midtone chromatic ratios, and skin-safe colorimetric anchors -> applies matched Exposure/Contrast/WB/Split Toning/Curve/HSL to active photo without blowing highlights or over-warming skin.
- **F04: Selective Copy & Batch Sync (`S03 / S01 -> S09 -> S01`)**:
  Tap `Copy Settings` -> toggle checkboxes per tool group -> confirm -> select 1..N photos in Grid -> `Paste Settings` / `Sync` updates SQLite `DevelopParams` non-destructively across selection.
- **F05: Preset Creation & `.xmp` Roundtrip (`S03 -> S08`)**:
  Open Presets -> tap `+ Create Preset` -> choose subset + name + category -> saved as interoperable Adobe Camera Raw `.xmp` and indexed in SQLite; or `Import .xmp` from device Downloads via SAF so desktop Lightroom / LightRaw presets work identically on Android.
- **F06: Watermark, Export & Collage (`S03 / S01 -> S10`)**:
  Open Export -> configure Format (JPEG/PNG), Resize (`Original`, `2048px`, `1080px`, Custom), Quality, Toggle Watermark (`Text` / `3x3 Anchor` / `Opacity` / `Scale`) -> Render full-resolution non-destructive pipeline -> Save to `Pictures/LightRaw` MediaStore or open Android Share Sheet / Collage Builder.

---

## 4. Entities & Schema Alignment

| Entity | Core Fields | Storage |
| --- | --- | --- |
| `PhotoAsset` | `idLocal`, `idGlobal`, `uri`, `displayName`, `captureTime`, `rating (0..5)`, `pick (-1/0/1)`, `colorLabel`, `width`, `height`, `orientation`, `albumIds` | SQLite (`Adobe_images`) |
| `DevelopState` | All PV2012 fields mirroring `src/params.rs` (`exposure_2012`, `temperature`, `tint`, curves, `hue/sat/lum[8]`, color grading, detail, optics, effects, crop, masks, heal) | SQLite (`Adobe_imageDevelopSettings`) |
| `PresetItem` | `id`, `name`, `category`, `isUser`, `isFavorite`, `xmpPath`, `developJson` | SQLite (`AgDevelopPreset`) + `.xmp` files |
| `MaskLayer` | `id`, `kind (Brush/Radial/Linear/Subject/Sky)`, `invert`, `feather`, `strokes/bounds`, `localParams` | JSON inside `DevelopState.mask_groups` |
| `Album` | `id`, `name`, `isSmart`, `queryRule`, `photoCount` | SQLite (`AgLibraryFolder` / `AgLibraryCollection`) |
| `WatermarkConfig` | `enabled`, `text`, `anchor (0..8)`, `fontSize`, `opacity`, `insetX/Y`, `shadow` | SharedPrefs / SQLite |
