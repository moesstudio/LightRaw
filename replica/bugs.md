# Bug Report: LightRaw vs Adobe Lightroom Classic

Build: release-v1.0.0  Date: 2026-11-07  Target: Lightroom Classic Parity

### BUG-001: Chalky Pale Bleached Skin Tone on Preset Application
- Severity: S1 (Core Image Pipeline / Color Science)
- Flow / case: F02 / F02-H1
- Screen: S02 (Develop Viewport)
- Build: release-v1.0.0  Device: Apple Silicon macOS Sonoma/Sequoia

Steps:
1. Open catalog with Sony ARW portrait (`DSC00005.ARW`).
2. Navigate to Develop module (`D`).
3. Apply preset `XII SMK 26` (`Exposure +0.50`, `Highlights -59`, `Whites -59`, `Orange Luminance +23`).

Expected:
Natural, warm peach skin tone matching Adobe Lightroom Classic (`R ~ 205..220, G ~ 150..165, B ~ 145..155`) with smooth highlight roll-off.

Actual:
Subject's face appeared chalky, pale, and bleached white (`249, 246, 246`) due to:
1. Tone curve adding flat scalar offsets to RGB and channel-independent highlight compression `r /= 1 + hl_comp * r`, which severely crushed Red since Red in skin is the highest channel (~0.90).
2. HSL luminance adjustment adding directly to HSL `L`, collapsing color saturation to 0 when `L -> 1.0`.

Fix:
1. Replaced channel-independent compression with Photometric Chromaticity-Preserved Tone Mapping on Luminance `Y`, scaling `(r, g, b)` by `target_Y / Y`.
2. Changed HSL luminance adjustment to a perceptual luminance multiplier `l_mult += params.luminance_adjustment[band] * 0.0025 * w`, preserving chromaticity `R > G > B`.
Status: Fixed in `src/pipeline.rs`.

---

### BUG-002: Preset Application Fallback Omitted HSL and Detail Adjustments
- Severity: S2 (Presets Engine)
- Flow / case: F03 / F03-H1
- Screen: S04 (Presets Panel)
- Build: release-v1.0.0

Steps:
1. Select photo and click preset "XII SMK 26".
2. Inspect `params.hue_adjustment`, `params.saturation_adjustment`, `params.luminance_adjustment`, `texture`, `clarity`, and `sharpening`.

Expected:
All 8-band HSL parameters (`HueOrange: -13`, `HueBlue: -25`, `SatOrange: -17`, `LumOrange: +23`, `LumRed: +17`), `Texture: -12`, `Clarity: +1`, `Dehaze: -2` are applied.

Actual:
Fallback branch in `apply_preset_to_selection` only assigned basic exposure and tone sliders, leaving all HSL adjustments at 0.0.

Fix:
Updated `apply_preset_to_selection` and preset loader to ensure all HSL, presence, detail, and profile attributes are fully populated from XMP/database.
Status: Fixed in `src/ui/mod.rs` & `src/presets.rs`.

---

### BUG-003: Incomplete Presets '+' Context Menu
- Severity: S2 (UI Parity)
- Flow / case: F03 / F03-H1
- Screen: S04 / S06
- Build: release-v1.0.0

Steps:
1. Click the '+' button in the Presets panel header.

Expected:
Context menu opens showing:
- `Create Preset...`
- `Import Presets...`
- `Manage Presets...`
matching Lightroom Classic (Screenshot Attachment 7).

Actual:
Only simple file picker opened directly without the standard 3-option popup menu.

Fix:
Implemented `show_presets_add_menu` popup with `Create Preset...`, `Import Presets...`, and `Manage Presets...`.
Status: Fixed in `src/ui/develop.rs`.

---

### BUG-004: Incomplete Preset Item Right-Click Context Menu
- Severity: S2 (UI Parity)
- Flow / case: F03 / F03-H2
- Screen: S04 / S06
- Build: release-v1.0.0

Steps:
1. Right click on any preset item (e.g. "XII SMK 26") in the Presets tree.

Expected:
Context menu displays 8 actions:
- `Add to Favorites` / `Remove from Favorites`
- `Apply on Import`
- `Rename`
- `Move`
- `Update with Current Settings`
- `Show in Finder`
- `Delete`
- `Export...`
matching Lightroom Classic (Screenshot Attachment 8).

Actual:
Only basic rename/delete was available.

Fix:
Implemented complete 8-item context menu with full macOS Finder integration and XMP serialization.
Status: Fixed in `src/ui/develop.rs` & `src/presets_dialogs.rs`.

---

### BUG-005: Copy Settings Dialog Lacked Granular Categorized Masking
- Severity: S2 (Workflow Parity)
- Flow / case: F04 / F04-H1
- Screen: S06
- Build: release-v1.0.0

Steps:
1. Click `Copy...` button at the bottom of the left Develop panel.

Expected:
Modal dialog opens matching Lightroom Classic (Screenshot Attachment 9) featuring:
- Categorized checkboxes: `Treatment & Profile`, `Basic`, `Curve`, `Color Mixer`, `Color Grading`, `Detail`, `Lens Corrections`, `Transform`, `Effects`, `Calibration`, `HDR`.
- Sub-item checkboxes (e.g. White Balance, Exposure, Contrast, Texture, Clarity, Sharpening, etc.).
- Action buttons: `Check All`, `Check None`, `Subset: Default`, `Cancel`, `Copy`.

Actual:
Clipboard copied entire develop state unconditionally without selective masking.

Fix:
Implemented `CopySettingsMask` struct, full modal UI matching Lightroom Classic, and selective parameter transfer in `paste_clipboard_settings`.
Status: Fixed in `src/ui/presets_dialogs.rs` & `src/ui/mod.rs`.

---

### BUG-006: Viewport Texture Cache Retaining Stale Render
- Severity: S3 (UI Responsiveness)
- Flow / case: F02 / F02-H1
- Screen: S02
- Build: release-v1.0.0

Steps:
1. Apply a preset or adjust a slider.
2. Observe canvas viewport.

Expected:
Canvas immediately repaints with fresh pipeline render.

Actual:
`rendered_after_cache` cached texture handle for the photo ID and did not invalidate until photo selection changed.

Fix:
Explicitly invalidate texture in `rendered_after_cache` on every parameter mutation or preset application and trigger `ctx.request_repaint()`.
Status: Fixed in `src/ui/mod.rs`.

---

### BUG-007: Stale Decoded RAW Cache in Temporary Directory
- Severity: S3 (File Pipeline)
- Flow / case: F01 / F02
- Screen: S01 / S02
- Build: release-v1.0.0

Steps:
1. Change RAW demosaicing parameters or target dimensions.
2. Re-decode RAW file.

Expected:
Pipeline uses updated demosaiced image buffer.

Actual:
Existing cached JPEG in `/tmp/lightraw_raw_decode_*.jpg` was reused because cache hash only hashed path and max_dim without file modification time or profile version.

Fix:
Included file modification timestamp and profile version in cache key hasher.
Status: Fixed in `src/catalog.rs`.

---

### BUG-008: High Idle CPU Usage (~94%) from Continuous egui Repaint Loop
- Severity: S1 (Performance & Battery)
- Flow / case: F01..F07
- Screen: Global (All Modules)
- Build: release-v1.0.0

Steps:
1. Launch LightRaw without user interaction.
2. Monitor CPU usage via `ps -A -o %cpu,%mem,command | grep LightRaw`.

Expected:
Idle CPU usage < 2.0% (like Lightroom Classic).

Actual:
CPU usage was ~94% due to unconstrained repaint requests in `update()` and unthrottled worker polling.

Fix:
1. Replaced unconditional repaint requests with event-driven repaint triggers (`ctx.request_repaint()` only on user input or state change).
2. Added backoff sleep in background image decoding/rendering worker channels.
Status: Fixed in `src/ui/mod.rs` & `src/pipeline.rs`. Live verified at 0.0% CPU.

---

### BUG-009: macOS Top Native Menu Bar Overwritten by Winit Run Loop
- Severity: S1 (macOS System Integration)
- Flow / case: Global Native Menus
- Screen: Top Menu Bar (1..=194 Actions)
- Build: release-v1.0.0

Steps:
1. Launch LightRaw.app bundle on macOS.
2. Inspect top menu bar next to Apple logo.

Expected:
Full 11-menu native Lightroom Classic menu bar: ` LightRaw File Edit Develop Photo Settings Tools View Window Help`.

Actual:
Winit window creation loop replaced `[NSApp setMainMenu:]` with a default single-item menu.

Fix:
Deferred menu installation to `dispatch_async(dispatch_get_main_queue(), ^{ [app setMainMenu:mainMenu]; })` and triggered once on the first egui frame.
Status: Fixed in `src/macos_menu.m` & `src/ui/mod.rs`.

---

### BUG-010: Modal Dialog egui State Trap Resetting Open Flags
- Severity: S2 (UI Dialogs)
- Flow / case: F04 (Copy/Sync Settings & Presets)
- Screen: S06
- Build: release-v1.0.0

Steps:
1. Open Copy Settings dialog, close it, and reopen it via `Settings -> Copy Settings...`.

Expected:
Dialog opens reliably on every user invocation.

Actual:
Binding `let mut is_open = app.show_copy_settings_modal;` directly caused egui's cached window close state to immediately reset `is_open = false` on frame 0.

Fix:
Initialized `let mut is_open = true;` inside `render_*_modal` so each invocation opens cleanly and close handlers synchronize `app.show_* = false`.
Status: Fixed in `src/ui/presets_dialogs.rs`.

---

### BUG-011: Canvas Surround Color Mismatch with Lightroom Classic Default
- Severity: S2 (Visual Parity)
- Flow / case: F02 (Develop Viewport)
- Screen: S02
- Build: release-v1.0.0

Steps:
1. Open Develop module with any portrait.
2. Observe canvas viewport surround background.

Expected:
Medium Gray `#808080` (RGB 128, 128, 128) matching Lightroom Classic default surround with right-click surround switcher.

Actual:
Hardcoded dark charcoal (`#1e1e1e`), creating high contrast difference against reference.

Fix:
Added `canvas_surround_color` setting defaulting to `Color32::from_rgb(128, 128, 128)` with right-click context menu switcher (`Black`, `Dark Gray`, `Medium Gray`, `Light Gray`, `White`).
Status: Fixed in `src/ui/develop_canvas.rs` & `src/ui/mod.rs`.
