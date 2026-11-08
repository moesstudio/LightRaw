# Test plan: LightRaw vs Adobe Lightroom Classic

Build: release-v1.0.0  Date: 2026-11-07  Env: macOS Darwin Sonoma/Sequoia, Apple Silicon, SQLite Catalog, Sony A7 ARW RAW assets

| case | flow | type | steps | expected | auto | result |
| --- | --- | --- | --- | --- | --- | --- |
| F01-H1 | F01 Culling & Selection | happy | Open catalog with 415 photos, press 1..5 for star ratings, 6..9 for color labels, right arrow to navigate | Active photo updates immediately, star badges and color labels persist in SQLite database | unit/live | pass |
| F01-E1 | F01 Culling & Selection | edge | Rapid keypress navigation (>10 photos per second) | No dropped events, cache remains consistent, no memory leak | unit/live | pass |
| F01-N1 | F01 Culling & Selection | negative | RAW file missing or renamed on disk | Displays neutral blank canvas with clear status error without crashing | unit | pass |
| F02-H1 | F02 RAW Develop & Skin Color | happy | Select photo 5 (`DSC00005.ARW`), apply preset `XII SMK 26`, observe viewport preview | Skin tone renders warm peach hue (`R ~ 205..220, G ~ 150..165, B ~ 145..155`), background blue is saturated, zero chalky collapse | unit/live | pass |
| F02-E1 | F02 RAW Develop & Skin Color | edge | Extreme exposure (+5 EV) with positive orange luminance | Highlights compress smoothly, chromaticity preserved, skin tone remains peach rather than bleaching to pure white | unit/live | pass |
| F02-E2 | F02 RAW Develop & Skin Color | edge | Toggle Before/After split view (`Y` / `Shift+Y`) | Viewport renders side-by-side or split line without flicker or buffer overrun | unit/live | pass |
| F03-H1 | F03 Preset Management | happy | Click '+' in Presets header -> Select `Import Presets...` -> Choose `XII_SMK_26.xmp` | Preset parses XML, creates entry under category 'A7', appears in list | unit/live | pass |
| F03-H2 | F03 Preset Management | happy | Right click preset item -> Click `Export...` | Serializes complete preset parameters to valid Adobe Camera Raw XML format and writes `.xmp` file | unit | pass |
| F03-E1 | F03 Preset Management | edge | Preset with non-ASCII or Unicode name and missing optional XML tags | Defaults missing tags gracefully without panic or corruption | unit | pass |
| F04-H1 | F04 Copy & Paste Settings | happy | Click `Copy...` -> modal opens -> click `Check None` -> check `Basic` & `Color Mixer` -> click `Copy` -> select photo 2 -> click `Paste` | Only selected subset parameters are transferred; unselected parameters (e.g. Crop, Curve) remain untouched | unit/live | pass |
| F04-E1 | F04 Copy & Paste Settings | edge | Copy portrait crop to landscape image | Crop aspect ratio handles bounds safely without inverted coordinate panic | unit | pass |
| F05-H1 | F05 Batch Export | happy | Multi-select photos 1..5 -> Menu `File` -> `Export...` -> 100% quality JPEG | Exports 5 files to destination directory with sRGB ICC color management | unit | pass |
