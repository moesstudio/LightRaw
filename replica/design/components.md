# Components Specification: LightRaw (Adobe Lightroom Classic Design System)

Derived from Adobe Lightroom Classic 15.4.1 interface screenshots and Lua resources.

## 1. Top App Shell & Module Bar
- **Dimensions**: Height 36px, full width.
- **Background**: `#181818` with 1px border bottom `#2c2c2c`.
- **Identity Badge**: "LR" rounded badge (`#2572e4` background with bold white typography) + catalog identity plate ("Moestret").
- **Module Tabs**: `Library`, `Develop`, `Map`, `Book`, `Slideshow`, `Print`, `Web`.
  - Inactive tab: `#888888`, hover `#cccccc`.
  - Active tab: `#ffffff` with subtle underline / glow indicator.

## 2. Develop Slider (Parametric Control)
- **Dimensions**: Label width 85px, slider track width 140px, numeric text box 40px.
- **Track**: 2px height, color `#383838`, center tick mark at 0.0 for biphasic sliders (e.g. Exposure, Contrast, Highlights).
- **Thumb**: Circular puck 10px diameter `#dedede` with hover state `#ffffff`.
- **Double Click Action**: Resets parameter instantly to default 0.0 (or 5050K for Temp).
- **Numeric Edit**: Clickable number with direct keyboard text input and up/down arrow step scrubbing.

## 3. Interactive Tone Curve Graph
- **Dimensions**: 220px x 220px square canvas.
- **Grid**: 4x4 subtle dotted grid `#2c2c2c`.
- **Graph Modes**:
  1. *Parametric Curve*: 4 interactive tone split zones (Highlights, Lights, Darks, Shadows).
  2. *Point Curve*: Custom control nodes with smooth Catmull-Rom / cubic spline interpolation.
  3. *Channel Curves*: Red, Green, Blue individual chromatic tone curves.
- **Targeted Adjustment Tool (TAT)**: Eyedropper crosshair icon that drags directly over the image preview to modulate curve heights.

## 4. 8-Band Color Mixer (HSL & Point Color)
- **Tabs**: `Mixer` vs `Point Color`.
- **Mixer Sub-tabs**: `Hue`, `Saturation`, `Luminance`, `All`.
- **Bands**: Red, Orange, Yellow, Green, Aqua, Blue, Purple, Magenta.
- **Color Swatches**: Color circles on left with slider ranges (-100..+100).
- **Point Color**: Eyedropper swatch sampler, Hue Shift (-180..+180), Saturation Shift (-100..+100), Luminance Shift (-100..+100), and Hue Range slider.

## 5. 3-Way Color Grading Wheels
- **Wheels**: Shadows, Midtones, Highlights, and Global.
- **Disc**: 70px diameter circular chromaticity disk (continuous spectral ring).
- **Puck**: Draggable circular dot representing current Hue (angle $\theta$) and Saturation (radius $r$).
- **Luminance Slider**: Horizontal bar beneath each wheel (-100..+100).
- **Global Sliders**: Blending (0..100, default 50) and Balance (-100..+100, default 0).

## 6. Presets Drawer & Context Menus
- **Header**: "Presets" title with search input ("Filter Presets") and right-aligned `+` button.
- **`+` Menu (Context Popup)**:
  - `Create Preset...`
  - `Import Presets...`
  - `Manage Presets...`
- **Tree Hierarchy**:
  - `Favorites` (starred items pinned to top).
  - Categorized disclosure arrows (e.g. `A7`, `Color`, `Creative`, `Portrait`, `B&W`).
- **Item Right-Click Menu**:
  - `Add to Favorites` / `Remove from Favorites`
  - `Apply on Import`
  - `Rename`
  - `Move`
  - `Update with Current Settings`
  - `Show in Finder`
  - `Delete`
  - `Export...`

## 7. Copy Settings Modal Dialog
- **Dimensions**: 680px width, centered modal overlay with dimming backdrop.
- **Header**: "Copy Settings" title.
- **Checklist Columns**:
  - *Column 1*: `Treatment & Profile`, `Basic` (White Balance, Exposure, Contrast, Highlights, Shadows, Whites, Blacks, Texture, Clarity, Dehaze, Vibrance, Saturation), `Curve` (Parametric Curve, Point Curve).
  - *Column 2*: `Color Mixer` (Mixer, Point Color), `Color Grading`, `Detail` (Denoise, Sharpening, Luminance Noise Reduction, Color Noise Reduction), `Lens Corrections` (Remove CA, Enable Profile, Manual Distortion, Manual Vignetting).
  - *Column 3*: `Transform` (Upright Mode, Upright Transforms, Manual), `Effects` (Post-Crop Vignetting, Grain), `Calibration`, `High Dynamic Range` (HDR Mode, SDR Settings).
- **Footer Controls**:
  - Left: `Check All`, `Check None`, `Subset: [Default v]`.
  - Right: `Cancel`, `Copy` (primary accent blue button).

## 8. Filmstrip & Culling Bar
- **Dimensions**: Height 120px at bottom of viewport.
- **Thumbnail View Modes**: 1/2 size toggle, active selection border `#2572e4`.
- **Badges**: Star rating (1-5), Color label tag (Red/Yellow/Green/Blue/Purple), Pick/Reject flag.
- **Toolbar**: Filter toggles, Loupe/Grid icons, Compare view, Soft Proofing toggle.
