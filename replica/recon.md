# Recon map: Adobe Lightroom Classic (macOS Desktop)

Scope: Library Catalog & Culling, Develop Module (Tone, Color Mixer, Color Grading, Detail, Curves, Geometry), Presets Management (.xmp Import/Export), Copy/Paste Settings Dialog, Native macOS Menu Bar.
For: Moestret Studio / High-Throughput School & Portrait Photography RAW Workflow.
Date: 2026-11-07

## Sources

| # | source | URL / Path | notes |
| --- | --- | --- | --- |
| 1 | Reference App | `/Users/mustamar/Documents/APLIKASI/LightRaw/Adobe Lightroom Classic.app` | Installed binary and resources |
| 2 | Reference Screenshots | `/Users/mustamar/Documents/APLIKASI/LightRaw/Screenshoot/` | 31 full-panel reference PNG screenshots |
| 3 | User Presets UI Capture | Conversation uploads (Attachments 7 & 8) | Presets '+' Menu, Preset Item Context Menu, Copy Settings Dialog |
| 4 | Sample RAW & Presets | `/Users/mustamar/Pictures/Raw Foto SMA-SMK/DSC00005.ARW` & `XII SMK 26.xmp` | Sony A7 24MP RAW files and studio preset |
| 5 | Adobe Lightroom Classic Specs | Official Adobe Camera Raw / Lightroom Classic Documentation | Process Version 6, Adobe Color profiles, HSL color science |

## Core loop

Import hundreds of high-resolution Camera RAW files, rapidly cull and organize in Library, apply calibrated studio presets with 100% accurate skin-tone color science, fine-tune tone/color grading, and batch copy/paste settings across portrait sessions.

## Screens

| ID | screen | route / how to reach | purpose | key components | states seen |
| --- | --- | --- | --- | --- | --- |
| S01 | Library Module | Top bar "Library" or key `G`/`E` | Catalog browsing, filtering, assisted culling, quick develop | Grid/Loupe canvas, Filter bar, Folders tree, Quick Develop, Metadata panel | empty, loaded (415 photos), filtered |
| S02 | Develop Main Workspace | Top bar "Develop" or key `D` | Interactive RAW canvas, before/after compare, live preview | Viewport canvas, Filmstrip, Toolstrip, Zoom (Fit/100%/1:1), Soft Proofing | default, zoomed, split before/after |
| S03 | Develop Right Inspection Panels | Right sidebar in Develop | Parametric image tuning across all stages | Histogram, Basic, Tone Curve, Color Mixer, Color Grading, Detail, Lens Corrections, Transform, Lens Blur, Effects, Calibration | collapsed, expanded, slider active |
| S04 | Develop Left Drawer | Left sidebar in Develop | Navigation, Presets hierarchy, Snapshots, History, Copy/Paste action bar | Presets tree, Favorites, Search filter, Snapshots list, Undo history list, Copy... / Paste buttons | expanded, filtered, preset applied |
| S05 | Develop Toolstrip Drawers | Top of right panel beneath histogram | Local adjustments and geometry corrections | Crop & Straighten drawer, Remove/Heal drawer, Red Eye drawer, Masking drawer (Subject/Sky/Brush) | closed, active tool overlay |
| S06 | Presets Menus & Copy Settings Dialog | '+' button on Presets, right click preset item, or 'Copy...' button | Creating/importing/exporting presets and granular selective copy/paste | Popup context menus (`Create...`, `Import...`, `Manage...`), Item context menu (`Export...`, etc.), Modal checkbox dialog with `Check All`, `Check None`, `Subset` | open, checklist toggled, file picker |
| S07 | macOS Native Top Menu Bar | Top screen menu bar (Cocoa `NSMenu`) | System-level shortcuts, standard macOS integration | File, Edit, Develop, Photo, Settings, Tools, View, Window, Help menus | native system state |

## Flows

```
F01 Import & Culling
    S01 Library -> Filter bar (Flagged / Star Rating / Color Label) -> Grid selection -> Assisted Culling
    happy path clicks: 3
    edge: missing raw files, non-standard orientations, large folders (500+ photos)

F02 Non-destructive RAW Develop
    S01 -> S02 Develop -> S03 Basic (WB, Exposure, Highlights, Shadows) -> Color Mixer (HSL) -> Color Grading -> Detail
    happy path clicks: 4
    edge: extreme over-exposure, chalky skin tone collapse, high ISO sensor noise

F03 Preset Management & Application
    S04 Presets Drawer -> Hover Preview -> Click Preset -> S06 '+' Menu -> Import Presets... (.xmp) -> Export Preset (.xmp)
    happy path clicks: 2
    edge: missing .xmp tags, invalid XML, overwriting existing preset, missing camera profile

F04 Granular Copy & Paste Settings
    S04 "Copy..." -> S06 Copy Settings Dialog -> Select categories -> Click "Copy" -> S02 Next Photo (Arrow Right) -> "Paste"
    happy path clicks: 4
    edge: copying crop/orientation across mixed orientations, paste to multi-selection

F05 Batch Processing & Export
    S01/S02 Multi-selection (Cmd+A / Shift+Click) -> File -> Export... -> Quality & format selection -> Write to disk
    happy path clicks: 3
    edge: destination folder unwritable, disk full, concurrent multi-threaded rendering
```

## Components

| component | variants | states | used on |
| --- | --- | --- | --- |
| Dark App Shell & Tab Bar | macOS titlebar integrated, breadcrumb bar | default, active tab, hover | all screens |
| Filmstrip | 1-row, adjustable thumbnail size, selection border | unselected, single select, multi-select, active key photo | S01, S02 |
| Develop Slider | centered zero (e.g. -100..+100), absolute (0..100), Kelvin | default, dragging, double-click reset, numeric edit | S03 |
| Tone Curve Graph | Point Curve, Parametric 4-region curve, RGB channels | default, node dragging, curve hover, TAT active | S03 |
| Color Mixer HSL | 8-band sliders (Hue, Sat, Lum, All), Point Color eyedropper | default, band active, swatch picked | S03 |
| Color Grading 3-Way | Shadows, Midtones, Highlights, Global color wheels | default, puck dragging, lum slider dragging | S03 |
| Presets Tree | Category collapsible headers, star favorites, preset row | default, hover preview, active/selected, right-click menu | S04 |
| Presets '+' Menu | macOS native / egui popup menu: `Create Preset...`, `Import Presets...`, `Manage Presets...` | hidden, visible | S04, S06 |
| Preset Item Menu | Right-click popup: `Add to Favorites`, `Apply on Import`, `Rename`, `Move`, `Update with Current Settings`, `Show in Finder`, `Delete`, `Export...` | hidden, visible | S04, S06 |
| Copy Settings Modal | Multi-category checklist (Treatment, Basic, Curve, Mixer, Grading, Detail, Lens, Transform, Effects, Calibration), Check All / None, Subset | closed, open modal, saving | S06 |
| macOS Menu Bar | Native `NSMenu` bridge: File, Edit, Develop, Photo, Settings, Tools, View, Window, Help | active system menu | S07 |

## Inferred data model

```
Catalog               sqlite database file (LightRawCatalog.lrcat)
                      fields: schema_version, root_folder, current_selection
                      evidence: LightRawCatalog.lrcat sqlite structure
                      confidence: high

Adobe_images          fields: id, local_path, filename, width, height, capture_time, rating, label, flag
                      evidence: S01 filmstrip, metadata panel, sqlite table
                      confidence: high

Adobe_imageDevelopSettings
                      fields: id, image_id, params_json (exposure, wb, tone, hsl, detail, curve, etc.)
                      evidence: S03 sliders, sqlite table
                      confidence: high

AgDevelopPreset       fields: id, category, name, is_user, is_favorite, presetJson
                      evidence: S04 presets panel, sqlite table AgDevelopPreset
                      confidence: high

HistoryStep           fields: id, image_id, name, value_summary, timestamp, params_snapshot
                      evidence: S04 History panel, undo/redo stack
                      confidence: high
```

Relationships:
- `Catalog` 1-n `Adobe_images`
- `Adobe_images` 1-1 `Adobe_imageDevelopSettings`
- `Adobe_images` 1-n `HistoryStep`
- `Catalog` 1-n `AgDevelopPreset`

## Feature matrix

See `replica/features.csv`. Must: 22, should: 14, could: 8, skip: 4.

## Out of scope (cannot or should not be cloned)

- Adobe Creative Cloud Sync & cloud storage account integration (proprietary Adobe servers).
- Firefly Cloud Generative AI fill server (requires Adobe API authentication & paid credits).
- Adobe Exchange Plugins Store.
- Proprietary proprietary camera tethering drivers (e.g. Hasselblad proprietary SDK).

## Size

Screens: 7, Flows: 5, Entities: 5. Hard parts: Color science pipeline (demosaicing, chromaticity preservation, Mired Planckian WB, 8-band HSL), native macOS Cocoa menu bridge, 60fps real-time GPU/Metal preview on 24MP RAW files. Size: M.
