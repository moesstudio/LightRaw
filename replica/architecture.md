# Architecture: LightRaw (High-Performance Rebuild of Adobe Lightroom Classic)

Target: Adobe Lightroom Classic 15.4.1 (macOS Apple Silicon & Intel)
Scope: Pure-Rust Native Desktop Application with SQLite Catalog, 11-Stage Non-Destructive Camera RAW Pipeline, Presets (.xmp), and Native Cocoa Menu Bar.

## Stack

| layer | choice | why |
| --- | --- | --- |
| language | Rust 2021 Edition | Zero-cost abstractions, memory safety, thread safety without garbage collection latency. |
| GUI framework | `eframe` + `egui 0.29.1` | Immediate mode GUI for ultra-smooth 60fps image manipulation and low-overhead UI layout. |
| native OS bridge | Objective-C Cocoa (`macos_menu.m` / `AppKit`) | Native top macOS system menu bar (`NSMenu`), native File Open/Save dialogs, and Finder integration. |
| RAW decoding | Apple CoreImage / ImageIO via `/usr/bin/sips` | Apple-certified hardware-accelerated demosaicing with native sRGB ColorSync ICC profiles. |
| database | SQLite 3 via `rusqlite 0.32` | 100% binary schema compatibility with Adobe Lightroom `.lrcat` catalog files (WAL mode enabled). |
| image math & LUT | SIMD & Rayon multi-threading | Real-time 24MP floating-point tone curves, 8-band HSL modulation, and unsharp masking. |
| configuration / Presets | Adobe Camera Raw CRS XMP Schema | Bi-directional lossless import and export of industry-standard `.xmp` develop presets. |

## Schema

Database: `LightRawCatalog.lrcat` (SQLite 3 with WAL mode enabled).
Direct parity with Lightroom Classic internal catalog structure:

```sql
-- 1. Photos and Images Table
CREATE TABLE IF NOT EXISTS Adobe_images (
    id_local INTEGER PRIMARY KEY AUTOINCREMENT,
    id_global TEXT UNIQUE NOT NULL,
    rootFolder INTEGER NOT NULL,
    pathFromRoot TEXT NOT NULL,
    real_file_path TEXT,
    captureTime TEXT,
    rating INTEGER DEFAULT 0,
    colorLabels TEXT DEFAULT '',
    pick INTEGER DEFAULT 0,
    fileWidth INTEGER,
    fileHeight INTEGER,
    orientation INTEGER DEFAULT 1
);

-- 2. Non-Destructive Develop Settings Table
CREATE TABLE IF NOT EXISTS Adobe_imageDevelopSettings (
    id_local INTEGER PRIMARY KEY AUTOINCREMENT,
    image INTEGER UNIQUE NOT NULL REFERENCES Adobe_images(id_local) ON DELETE CASCADE,
    hasDevelopAdjustments INTEGER DEFAULT 0,
    whiteBalance TEXT DEFAULT 'As Shot',
    temperature REAL DEFAULT 5050.0,
    tint REAL DEFAULT 0.0,
    exposure REAL DEFAULT 0.0,
    contrast REAL DEFAULT 0.0,
    highlights REAL DEFAULT 0.0,
    shadows REAL DEFAULT 0.0,
    whites REAL DEFAULT 0.0,
    blacks REAL DEFAULT 0.0,
    texture REAL DEFAULT 0.0,
    clarity REAL DEFAULT 0.0,
    dehaze REAL DEFAULT 0.0,
    vibrance REAL DEFAULT 0.0,
    saturation REAL DEFAULT 0.0,
    text TEXT NOT NULL -- Comprehensive JSON DevelopParams dump
);

-- 3. Presets Hierarchy Table
CREATE TABLE IF NOT EXISTS AgDevelopPreset (
    id TEXT PRIMARY KEY,
    category TEXT NOT NULL,
    name TEXT NOT NULL,
    is_user INTEGER NOT NULL DEFAULT 1,
    is_favorite INTEGER NOT NULL DEFAULT 0,
    presetJson TEXT NOT NULL
);

-- 4. Develop History Steps
CREATE TABLE IF NOT EXISTS Adobe_libraryImageDevelopHistoryStep (
    id_local INTEGER PRIMARY KEY AUTOINCREMENT,
    image INTEGER NOT NULL REFERENCES Adobe_images(id_local) ON DELETE CASCADE,
    name TEXT NOT NULL,
    valueString TEXT NOT NULL,
    dateCreated REAL NOT NULL,
    digest TEXT
);

-- 5. Folders and Catalog Roots
CREATE TABLE IF NOT EXISTS AgLibraryFolder (
    id_local INTEGER PRIMARY KEY AUTOINCREMENT,
    id_global TEXT UNIQUE NOT NULL,
    rootFolder INTEGER NOT NULL,
    pathFromRoot TEXT NOT NULL
);
```

## 11-Stage Non-Destructive RAW Image Pipeline

```
Raw Camera Sensor (.ARW/.CR3/etc.)
   │
   ├─► [Stage 01] Native Demosaicing (sips + Apple ColorSync sRGB Profile)
   │
   ├─► [Stage 02] Exposure Compensation (2.0^(EV)) & Highlight Recovery
   │
   ├─► [Stage 03] Planckian Mired White Balance ((1e6/Temp - 1e6/5050)/300)
   │
   ├─► [Stage 04] Photometric Tone Mapping (Luminance Y Chromaticity-Preserved)
   │
   ├─► [Stage 05] Spline Tone Curve (Parametric 4-Region & RGB Point Curves)
   │
   ├─► [Stage 06] 8-Band Perceptual HSL & Point Color Eyedropper
   │
   ├─► [Stage 07] 3-Way Color Grading (Shadows, Midtones, Highlights, Global)
   │
   ├─► [Stage 08] Detail Sharpening (Unsharp Mask) & Luminance Denoise
   │
   ├─► [Stage 09] Lens Corrections (CA Removal, Distortion) & Transform Upright
   │
   ├─► [Stage 10] Effects (Post-Crop Cosine Vignette & Film Grain)
   │
   ├─► [Stage 11] Camera Calibration (Process Version 6 Primaries & Shadow Tint)
   │
   ▼
GPU Framebuffer (eframe egui::ColorImage sRGB Texture)
```

## The Parts That Bite

1. **Highlight Chromaticity Collapse (Chalky Face)**:
   - *Problem*: In Adobe Lightroom, boosting orange luminance (`LuminanceAdjustmentOrange = +23`) or pulling down highlights does not bleach skin into white. A naive RGB tone addition destroys the red-to-green ratio, turning human skin pale.
   - *Architecture Fix*: Separate luminance from chromaticity. Calculate CIE 1931 Luminance $Y = 0.2126 R + 0.7152 G + 0.0722 B$. Apply all tone transformations to $Y$, then scale channels by $Y_{\text{target}} / Y$. Apply HSL luminance as a perceptual chroma-preserving multiplier.
2. **Preset XMP Serialization & Namespaces**:
   - *Problem*: Presets exported from Lightroom Classic use Adobe Camera Raw namespace `xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/"`.
   - *Architecture Fix*: Bi-directional serde parser with exact XML element mappings (`crs:Exposure2012`, `crs:HueAdjustmentOrange`, `crs:ToneCurvePV2012`, `crs:Look`).
3. **Selective Copy / Paste Invalidation**:
   - *Problem*: Copying develop settings between portrait and landscape photos must not accidentally invert crop bounding boxes or override unselected adjustments.
   - *Architecture Fix*: Bitmask-driven `CopySettingsMask` passed to `paste_clipboard_settings`.
