# Android Components Specification: LightRaw Mobile

All components use clean-room Material 3 XML + custom Canvas views, `>= 48dp` touch targets, OLED darkroom surfaces (`#080808` / `#1E1E1E` / `#2C2C2C`), and original vector paths.

## 1. Library Home Shell, Bottom Navigation & Filter Strip (`S01`)
- **Top Bar (`56dp`, `#080808`)**: "LR" accent badge (`#2563EB`) + active section title, Search toggle, Sort menu (`Capture Time`, `File Name`, `Star Rating`, `Modified Date`), Column toggle (`2 / 3 / 4` columns), Multi-Select toggle, and About dialog.
- **Filter Chip Row**: Horizontal scroll chips (`All`, `Picked ⚑`, `Rejected`, `3★+`, `5★`, `RAW`, `Edited`, plus Color Label chips) and dedicated **Albums strip** (`+ New Album`, rename/delete) & **Device Browser strip** (direct MediaStore storage scan & tap-to-edit).
- **Grid Cell**: Square center-crop thumbnail, `2dp` selection border (`#2563EB`), top-left Pick/Reject flag, top-right Color Label dot + Star rating pill, bottom-right RAW/DNG + Edited badges.
- **4-Tab Bottom Navigation (`#080808`)**: `Photos`, `Albums`, `Device`, `Camera` (clean-room vector icons), swapping dynamically to contextual **Batch Action Dock** (`Rate`, `Flag`, `Color`, `Album`, `Copy`, `Paste`, `Collage`, `Export`, `Remove`) during multi-select.

## 2. Touch Loupe Studio Shell (`S03`)
- **Top Overlay Toolbar (`56dp`, `#080808`)**: Crisp vector icons for Back, Undo, Redo, Info HUD toggle, Histogram HUD toggle, Rate & Review toggle, More (Versions / History / EXIF / Reset), and primary `Export` pill (`#2563EB`).
- **Filmstrip Stepper & Info HUD**: Overlay Prev `<` / Index (`1 / N`) / Next `>` switcher at bottom of canvas + toggleable `loupeInfoHud` card displaying filename, RAW badge, resolution, rating, flag, color label, album, and active preset.
- **Histogram Overlay**: Toggleable RGB+Luma chart (`#B8080808` translucent card).
- **Rate & Review Bar**: Expandable horizontal pill strip below canvas with `1★..5★`, `⚑ Pick`, `✕ Reject`, and `Red`/`Yellow`/`Green`/`Blue`/`Purple` Color Label toggles persisted to SQLite.
- **Interactive Loupe Canvas (`LoupeImageView`)**: Pinch-to-zoom (`1x..8x`), pan, double-tap zoom, press-and-hold **Before** split view, plus interactive Crop handles, Mask handles/brush, and Spot Heal circles.
- **Bottom Horizontal Tool Rail**: Scrollable pills for `AI Match`, `Presets`, `Light`, `Color`, `Mixer` (8 colored dots + `B&W`), `Grading` (interactive 2D `ColorWheelView`), `Curve`, `Effects`, `Detail` (`Sharpening` / `Noise Reduction` sub-tabs), `Optics`, `Crop` (`Crop & Straighten` / `Geometry Warp` keystone sub-tabs), `Mask`, and `Heal`.

## 3. Interactive 2D Color Grading Wheel (`ColorWheelView`, `S04`)
- Custom Canvas HSV hue/saturation circular wheel (`156dp`) with crosshairs, active drag handle, touch tracking, and double-tap reset, paired with Luminance, Blending, and Balance sliders across `Shadows`, `Midtones`, `Highlights`, and `Global` zones.

## 4. Pro Camera HUD (`S11`, `activity_camera.xml`)
- OLED `#080808` top HUD (`Close`, `Flash Off/Shot/Torch`, `PRO · DNG+JPEG` badge, `Grid` 3x3 toggle, `Flip`) and viewfinder with optional `CameraGridOverlayView`.
- Bottom control dock with White Balance pills (`AWB`, `Daylight`, `Cloudy`, `Tungsten`, `Fluor`), `DNG` toggle switch, EV compensation SeekBar (`Locale.US`), `Last Capture` thumbnail shortcut, circular double-ring shutter button (`bg_shutter_ring`), and active WB indicator.

## 5. Presets Filmstrip & `.xmp` Codec (`S08`)
- Category tabs (`All`, `Favorites ★`, `User`, `Portrait`, `Landscape`, `Cinematic`, `B&W`), `72dp` live-rendered thumbnail filmstrip, Intensity slider (`0..200%`), `Save Current`, `Import .xmp`, and long-press actions (`Favorite ★`, `Rename`, `Update with Current Settings`, `Export .xmp`, `Delete`).
