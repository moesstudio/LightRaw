// ============================================================================
// LightRaw — 100% Pure Rust Clone of Adobe Lightroom Classic 14.5.1
// Local-First Professional RAW Photography Workflow & Non-Destructive Engine
// (Zero Cloud / Zero Oz / Zero Nimbus Telemetry Dependencies)
// ============================================================================

#![allow(float_literal_f32_fallback)]

pub mod ai;
pub mod catalog;
pub mod params;
pub mod pipeline;
pub mod presets;
pub mod ui;
pub mod watermark;

use eframe::egui;
use std::path::PathBuf;
use ui::{Colors, LightRawApp};

/// Applies Adobe Lightroom Classic's signature neutral dark charcoal color scheme
/// and loads macOS system Unicode fonts (Arial Unicode / SF) so all symbols (▼, ▶, ⊞, ⚑, ★, ⌑, 🩹, etc.)
/// render crisply without tofu boxes (`□`).
fn configure_lightroom_visuals(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    // 1. Native Apple San Francisco Pro System Font
    if let Ok(sf_data) = std::fs::read("/System/Library/Fonts/SFNS.ttf") {
        fonts.font_data.insert(
            "sf_pro".to_owned(),
            egui::FontData::from_owned(sf_data),
        );
        if let Some(prop) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            prop.insert(0, "sf_pro".to_owned());
        }
    }
    // 2. Native Apple San Francisco Mono Font
    if let Ok(sf_mono) = std::fs::read("/System/Library/Fonts/SFNSMono.ttf") {
        fonts.font_data.insert(
            "sf_mono".to_owned(),
            egui::FontData::from_owned(sf_mono),
        );
        if let Some(mono) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            mono.insert(0, "sf_mono".to_owned());
        }
    }
    // 3. Fallback for Unicode glyphs & symbols
    if let Ok(font_data) = std::fs::read("/System/Library/Fonts/Supplemental/Arial Unicode.ttf") {
        fonts.font_data.insert(
            "arial_unicode".to_owned(),
            egui::FontData::from_owned(font_data),
        );
        if let Some(prop) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            prop.push("arial_unicode".to_owned());
        }
        if let Some(mono) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            mono.push("arial_unicode".to_owned());
        }
    }
    ctx.set_fonts(fonts);

    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = Colors::BG_PANEL;
    visuals.window_fill = Colors::BG_DARK;
    visuals.faint_bg_color = Colors::BG_HEADER;
    visuals.extreme_bg_color = Colors::BG_DARK;

    // Native macOS Apple HIG Corner Roundings
    visuals.window_rounding = egui::Rounding::same(10.0);
    visuals.menu_rounding = egui::Rounding::same(8.0);
    visuals.widgets.noninteractive.rounding = egui::Rounding::same(6.0);
    visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);
    visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);
    visuals.widgets.active.rounding = egui::Rounding::same(6.0);
    visuals.widgets.open.rounding = egui::Rounding::same(6.0);

    visuals.widgets.noninteractive.bg_fill = Colors::BG_PANEL;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, Colors::TEXT_MAIN);
    visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(46, 46, 52));

    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(44, 44, 50);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, Colors::TEXT_MAIN);
    visuals.widgets.inactive.bg_stroke =
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(56, 56, 62));

    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(56, 56, 64);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, Colors::ACCENT_BLUE);

    visuals.widgets.active.bg_fill = Colors::ACCENT_BLUE;
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);

    // Native macOS System Blue Selection Halo
    visuals.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(10, 132, 255, 185);
    visuals.selection.stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(6.0, 5.0);
    style.spacing.button_padding = egui::vec2(8.0, 3.5);
    style.spacing.window_margin = egui::Margin::same(10.0);
    style.spacing.scroll.bar_width = 7.0;
    style.spacing.scroll.floating = true;
    style.spacing.scroll.bar_inner_margin = 2.0;
    ctx.set_style(style);
}

/// Resolves the workspace directory reliably whether launched from terminal or double-clicked as `LightRaw.app`.
fn resolve_workspace_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        // Check if running inside `<workspace>/LightRaw.app/Contents/MacOS/LightRaw`
        if let Some(macos_dir) = exe.parent() {
            if macos_dir.file_name().and_then(|s| s.to_str()) == Some("MacOS") {
                if let Some(app_parent) = macos_dir
                    .parent()
                    .and_then(|c| c.parent())
                    .and_then(|app| app.parent())
                {
                    return app_parent.to_path_buf();
                }
            }
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

#[cfg(target_os = "macos")]
extern "C" {
    fn setup_macos_native_menu();
}

fn main() -> eframe::Result<()> {
    #[cfg(target_os = "macos")]
    unsafe {
        setup_macos_native_menu();
    }

    let workspace_dir = resolve_workspace_dir();

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("LightRaw 1.0.0 — Moestret [LightRawCatalog.lrcat]")
        .with_inner_size([1600.0, 960.0])
        .with_min_inner_size([1150.0, 720.0])
        .with_drag_and_drop(true);

    if let Ok(img) = image::load_from_memory(include_bytes!("../assets/icon.png")) {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        viewport = viewport.with_icon(std::sync::Arc::new(egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        }));
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "LightRaw",
        native_options,
        Box::new(move |cc| {
            configure_lightroom_visuals(&cc.egui_ctx);
            Ok(Box::new(LightRawApp::new(workspace_dir)))
        }),
    )
}

#[cfg(test)]
mod tests {
    use crate::catalog::{
        create_default_catalog_photos, create_default_collections, create_empty_placeholder_photo,
        decode_real_photo_file, is_camera_raw_extension, is_supported_photo_path, CatalogDb,
        ColorLabel, PickFlag,
    };
    use crate::params::{
        BokehShape, DevelopParams, LocalAdjustParams, MaskComponent, MaskGroup, MaskOp,
        MaskPrimitiveKind, PointColorSwatch, RedEyeMode, RedEyeSpot, RetouchArea, RetouchMode,
        UprightMode,
    };
    use crate::pipeline::{render_develop_pipeline, RawSourceImage, RenderOptions};
    use crate::presets::{builtin_develop_presets, builtin_localized_mask_presets};
    use std::path::Path;

    #[test]
    fn test_ag_develop_params_defaults_and_serialization() {
        let mut params = DevelopParams::default();
        assert!(!params.is_modified());
        assert_eq!(params.process_version, "Version 6 (Current)");
        assert_eq!(params.temperature, 5500.0);
        assert_eq!(params.exposure_2012, 0.0);

        params.exposure_2012 = 1.25;
        params.contrast_2012 = 22.0;
        params.lens_blur_active = true;
        params.lens_blur_amount = 45.0;
        params.lens_blur_bokeh = BokehShape::CatEye;
        assert!(params.is_modified());

        let json = serde_json::to_string(&params).expect("serialize DevelopParams");
        let restored: DevelopParams =
            serde_json::from_str(&json).expect("deserialize DevelopParams");
        assert_eq!(restored.exposure_2012, 1.25);
        assert_eq!(restored.contrast_2012, 22.0);
        assert!(restored.lens_blur_active);
        assert_eq!(restored.lens_blur_bokeh, BokehShape::CatEye);
    }

    #[test]
    fn test_full_11_stage_camera_raw_pipeline_execution() {
        let src = RawSourceImage::generate_sample_raw(0, 240, 160);
        let mut params = DevelopParams::default();

        // 1. Basic Panel + Dehaze + Clarity + Vibrance (PV2012 / PV6.7)
        params.temperature = 6200.0;
        params.tint = 8.0;
        params.exposure_2012 = 0.45;
        params.contrast_2012 = 18.0;
        params.highlights_2012 = -35.0;
        params.shadows_2012 = 40.0;
        params.whites_2012 = 15.0;
        params.blacks_2012 = -12.0;
        params.texture = 20.0;
        params.clarity_2012 = 25.0;
        params.dehaze = 18.0;
        params.vibrance = 22.0;
        params.saturation = 8.0;

        // 2. Parametric Tone Curve
        params.parametric_lights = 15.0;
        params.parametric_darks = -10.0;

        // 3. HSL & Point Color
        params.hue_adjustment[5] = -12.0;
        params.saturation_adjustment[1] = 18.0;
        params.luminance_adjustment[5] = -15.0;
        params.point_colors.push(PointColorSwatch {
            name: "Sky Blue".into(),
            src_hue: 210.0,
            src_sat: 0.6,
            src_lum: 0.5,
            hue_shift: 10.0,
            sat_shift: 20.0,
            lum_shift: -10.0,
            range_amount: 50.0,
            ..Default::default()
        });

        // 4. 4-Way Color Grading
        params.split_toning_shadow_hue = 220.0;
        params.split_toning_shadow_saturation = 25.0;
        params.split_toning_highlight_hue = 42.0;
        params.split_toning_highlight_saturation = 20.0;

        // 5. Detail (Sharpening & AI Denoise)
        params.sharpness = 55.0;
        params.luminance_smoothing = 25.0;
        params.color_noise_reduction = 30.0;

        // 6. AI Lens Blur & Bokeh
        params.lens_blur_active = true;
        params.lens_blur_amount = 40.0;
        params.lens_blur_highlights_boost = 35.0;

        // 7. Lens Corrections & Transform
        params.lens_manual_distortion_amount = -8.0;
        params.defringe_purple_amount = 4.0;
        params.perspective_vertical = -6.0;
        params.perspective_upright = UprightMode::Auto;

        // 8. Effects (Post-Crop Vignetting & Film Grain)
        params.post_crop_vignette_amount = -25.0;
        params.grain_amount = 18.0;

        // 9. Camera Calibration
        params.blue_hue = -10.0;
        params.blue_saturation = 15.0;

        // 10. Retouch & Red-Eye
        params.retouch_areas.push(RetouchArea {
            mode: RetouchMode::Heal,
            target_x: 0.3,
            target_y: 0.3,
            source_x: 0.4,
            source_y: 0.3,
            radius: 0.04,
            feather: 50.0,
            opacity: 100.0,
            points: Vec::new(),
        });
        params.red_eye_spots.push(RedEyeSpot {
            mode: RedEyeMode::RedEye,
            center_x: 0.5,
            center_y: 0.5,
            radius: 0.02,
            pupil_size: 50.0,
            darken: 50.0,
            add_catchlight: true,
        });

        // 11. Local AI Masking
        params.mask_groups.push(MaskGroup {
            id: 1,
            name: "AI Sky Mask".into(),
            enabled: true,
            invert: false,
            components: vec![MaskComponent {
                name: "Select Sky".into(),
                op: MaskOp::Add,
                invert: false,
                kind: MaskPrimitiveKind::AiSky,
            }],
            local: LocalAdjustParams {
                exposure: -0.45,
                dehaze: 25.0,
                saturation: 15.0,
                ..Default::default()
            },
        });

        let frame = render_develop_pipeline(
            &src,
            &params,
            RenderOptions {
                show_shadow_clipping: true,
                show_highlight_clipping: true,
                soft_proofing: true,
                gamut_warning: false,
                active_mask_overlay: None,
                apply_crop: true,
                ..RenderOptions::default()
            },
        );

        assert_eq!(frame.width, 240);
        assert_eq!(frame.height, 160);
        assert_eq!(frame.rgba.len(), 240 * 160 * 4);
        assert!(frame.hist_lum.iter().sum::<u32>() > 0);
    }

    #[test]
    fn test_lrcat_sqlite_catalog_clean_and_raw_import() {
        let temp_dir = std::env::temp_dir().join(format!(
            "lightraw_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&temp_dir).expect("create temp test dir");

        let db = CatalogDb::open_or_create(&temp_dir);
        assert!(db.db_path.exists());
        assert!(db.blob_dir.exists());

        // Verify catalog starts 100% clean with 0 template photos
        let default_photos = create_default_catalog_photos();
        assert!(
            default_photos.is_empty(),
            "Catalog must not seed any template photos"
        );
        assert!(db.load_all_photos().is_empty());

        let default_cols = create_default_collections();
        assert_eq!(default_cols.len(), 4);

        // Verify RAW extension recognition (.CR2, .CR3, .NEF, .ARW, .DNG, .RAF, .ORF, .RW2, .PEF)
        assert!(is_camera_raw_extension(Path::new("/tmp/photo.CR3")));
        assert!(is_camera_raw_extension(Path::new("/tmp/photo.ARW")));
        assert!(is_camera_raw_extension(Path::new("/tmp/photo.NEF")));
        assert!(is_camera_raw_extension(Path::new("/tmp/photo.DNG")));
        assert!(is_camera_raw_extension(Path::new("/tmp/photo.RAF")));
        assert!(is_camera_raw_extension(Path::new("/tmp/photo.RW2")));
        assert!(is_camera_raw_extension(Path::new("/tmp/photo.ORF")));
        assert!(is_camera_raw_extension(Path::new("/tmp/photo.PEF")));

        // Create a simulated RAW file with an embedded full-resolution JPEG preview stream
        let sim_raw_path = temp_dir.join("TEST_CAMERA_SHOOT.CR3");
        let mut img_buf = image::RgbaImage::new(320, 240);
        for (x, y, px) in img_buf.enumerate_pixels_mut() {
            *px = image::Rgba([(x % 255) as u8, (y % 255) as u8, 180, 255]);
        }
        let mut jpeg_bytes = Vec::new();
        image::DynamicImage::ImageRgba8(img_buf)
            .write_to(
                &mut std::io::Cursor::new(&mut jpeg_bytes),
                image::ImageFormat::Jpeg,
            )
            .expect("encode embedded jpeg");
        // Wrap inside a mock CR3 RAW header + embedded JPEG preview
        let mut raw_container = vec![0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'c', b'r', b'x', b' '];
        raw_container.extend_from_slice(&jpeg_bytes);
        std::fs::write(&sim_raw_path, &raw_container).expect("write simulated CR3 file");

        assert!(is_supported_photo_path(&sim_raw_path));
        let decoded = decode_real_photo_file(&sim_raw_path, 320, 240)
            .expect("decode simulated CR3 RAW file");
        assert_eq!(decoded.width, 320);
        assert_eq!(decoded.height, 240);

        // Save imported real RAW photo to SQLite catalog and verify persistence
        let mut p0 = create_empty_placeholder_photo();
        p0.id = 1;
        p0.file_name = "TEST_CAMERA_SHOOT.CR3".into();
        p0.real_file_path = Some(sim_raw_path.display().to_string());
        p0.rating = 5;
        p0.flag = PickFlag::Picked;
        p0.color_label = ColorLabel::Green;
        p0.params.exposure_2012 = 0.85;
        p0.params.lens_blur_active = true;
        db.save_photo(&p0);

        let reloaded = db.load_all_photos();
        assert_eq!(reloaded.len(), 1);
        assert_eq!(reloaded[0].file_name, "TEST_CAMERA_SHOOT.CR3");
        assert_eq!(reloaded[0].rating, 5);
        assert_eq!(reloaded[0].flag, PickFlag::Picked);
        assert_eq!(reloaded[0].color_label, ColorLabel::Green);
        assert!((reloaded[0].params.exposure_2012 - 0.85).abs() < 1e-4);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_builtin_presets_and_local_presets() {
        let presets = builtin_develop_presets();
        assert!(presets.len() >= 14);
        for preset in &presets {
            assert!(!preset.name.is_empty());
            assert!(!preset.category.is_empty());
        }

        let local_presets = builtin_localized_mask_presets();
        assert_eq!(local_presets.len(), 7);
    }

    #[test]
    fn test_multi_selection_and_preset_application() {
        let temp_dir = std::env::temp_dir().join(format!("lightraw_test_multisel_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let mut app = crate::ui::LightRawApp::new(temp_dir.clone());

        // Add 5 mock photos
        for i in 1..=5 {
            let mut p = create_empty_placeholder_photo();
            p.id = i;
            p.file_name = format!("DSC_{:04}.ARW", i);
            p.folder_path = "/Mock/Folder".into();
            p.real_file_path = Some(format!("/Mock/Folder/DSC_{:04}.ARW", i));
            app.catalog_db.save_photo(&p);
            app.photos.push(p);
        }

        // Test normal single click
        app.select_photo(1, false);
        assert_eq!(app.selected_photo_ids, vec![1]);
        assert_eq!(app.active_photo_id, 1);

        // Test Shift+Click range selection from 1 to 4
        app.select_photo_range(4);
        assert_eq!(app.selected_photo_ids, vec![1, 2, 3, 4]);
        assert_eq!(app.active_photo_id, 4);

        // Test Cmd+Click toggle photo 2 off
        app.toggle_select_photo(2);
        assert_eq!(app.selected_photo_ids, vec![1, 3, 4]);

        // Test Cmd+Click toggle photo 5 on
        app.toggle_select_photo(5);
        assert_eq!(app.selected_photo_ids, vec![1, 3, 4, 5]);

        // Test Cmd+A (Select All)
        app.select_all();
        assert_eq!(app.selected_photo_ids.len(), 5);

        // Test applying preset to selection
        app.apply_preset_to_selection("Flattering Skin");
        for p in &app.photos {
            assert_eq!(p.params.exposure_2012, 0.15);
            assert_eq!(p.params.highlights_2012, -25.0);
            assert_eq!(p.params.shadows_2012, 22.0);
            assert_eq!(p.history.last().unwrap().name, "Preset: Flattering Skin");
        }

        // Test Cmd+D (Deselect All)
        app.deselect_all();
        assert_eq!(app.selected_photo_ids.len(), 1);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_native_macos_menu_actions_dispatch() {
        let temp_dir = std::env::temp_dir().join(format!(
            "lightraw_menu_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(&temp_dir);

        let mut app = crate::ui::LightRawApp::new(temp_dir.clone());
        for i in 1..=5 {
            let mut p = create_empty_placeholder_photo();
            p.id = i;
            p.file_name = format!("IMG_{}.CR3", i);
            p.real_file_path = Some(
                temp_dir
                    .join(format!("IMG_{}.CR3", i))
                    .to_string_lossy()
                    .to_string(),
            );
            app.catalog_db.save_photo(&p);
            app.photos.push(p);
        }
        app.active_photo_id = 1;
        app.selected_photo_ids = vec![1];

        // 1. App Menu actions
        app.execute_menu_action(1); // ACTION_ABOUT
        assert!(app.show_about_modal);
        assert_eq!(app.about_tab, 0);
        app.execute_menu_action(2); // ACTION_SYSTEM_INFO
        assert!(app.show_system_info_modal);
        assert!(app.show_about_modal);
        assert_eq!(app.about_tab, 1);

        // 2. File Menu actions
        app.execute_menu_action(12); // ACTION_OPTIMIZE_CATALOG
        assert!(app.status_message.contains("optimized"));
        app.execute_menu_action(14); // ACTION_IMPORT_PHOTOS
        assert!(app.show_import_modal);
        app.show_import_modal = false;
        app.execute_menu_action(15); // ACTION_EXPORT_PHOTOS
        assert!(app.show_export_modal);
        app.show_export_modal = false;

        // 3. Edit Menu actions
        app.execute_menu_action(36); // ACTION_SELECT_ALL
        assert_eq!(app.selected_photo_ids.len(), 5);
        app.execute_menu_action(37); // ACTION_SELECT_NONE
        assert_eq!(app.selected_photo_ids.len(), 1);

        // 4. Photo Rating, Flag & Label
        app.execute_menu_action(67); // ACTION_SET_FLAG_PICK
        assert_eq!(app.active_photo().flag, PickFlag::Picked);
        app.execute_menu_action(75); // ACTION_SET_RATING_5
        assert_eq!(app.active_photo().rating, 5);
        app.execute_menu_action(77); // ACTION_SET_COLOR_RED
        assert_eq!(app.active_photo().color_label, ColorLabel::Red);

        // 5. Rotation & Flip
        app.execute_menu_action(64); // ACTION_ROTATE_RIGHT (+90)
        assert_eq!(app.active_photo().params.rotation_degrees, 90.0);
        app.execute_menu_action(63); // ACTION_ROTATE_LEFT (-90)
        assert_eq!(app.active_photo().params.rotation_degrees, 0.0);
        app.execute_menu_action(65); // ACTION_FLIP_HORIZONTAL
        assert!(app.active_photo().params.flip_horizontal);

        // 6. Settings actions
        app.execute_menu_action(94); // ACTION_AUTO_SETTINGS
        assert!(app.active_photo().params.auto_tone);
        app.execute_menu_action(96); // ACTION_CONVERT_BW
        assert!(app.active_photo().params.convert_to_grayscale);
        app.execute_menu_action(90); // ACTION_RESET_ALL_SETTINGS
        assert!(!app.active_photo().params.convert_to_grayscale);

        // 7. Tools actions
        app.execute_menu_action(110); // ACTION_TOOL_CROP
        assert_eq!(app.active_module, crate::ui::WorkspaceModule::Develop);
        assert_eq!(app.develop_tool, crate::ui::DevelopTool::Crop);
        app.execute_menu_action(112); // ACTION_TOOL_REMOVE
        assert_eq!(app.develop_tool, crate::ui::DevelopTool::Remove);
        app.execute_menu_action(113); // ACTION_TOOL_MASKING
        assert_eq!(app.develop_tool, crate::ui::DevelopTool::Masking);

        // 8. View actions
        app.execute_menu_action(129); // ACTION_VIEW_GRID
        assert_eq!(app.active_module, crate::ui::WorkspaceModule::Library);
        assert_eq!(app.library_view_mode, crate::ui::LibraryViewMode::Grid);
        app.execute_menu_action(130); // ACTION_VIEW_LOUPE
        assert_eq!(app.library_view_mode, crate::ui::LibraryViewMode::Loupe);
        app.execute_menu_action(131); // ACTION_VIEW_COMPARE
        assert_eq!(app.library_view_mode, crate::ui::LibraryViewMode::Compare);

        // 9. Window / Modules actions
        app.execute_menu_action(151); // ACTION_MODULE_DEVELOP
        assert_eq!(app.active_module, crate::ui::WorkspaceModule::Develop);
        app.execute_menu_action(152); // ACTION_MODULE_MAP
        assert_eq!(app.active_module, crate::ui::WorkspaceModule::Map);
        app.execute_menu_action(153); // ACTION_MODULE_BOOK
        assert_eq!(app.active_module, crate::ui::WorkspaceModule::Book);
        app.execute_menu_action(154); // ACTION_MODULE_SLIDESHOW
        assert_eq!(app.active_module, crate::ui::WorkspaceModule::Slideshow);
        app.execute_menu_action(155); // ACTION_MODULE_PRINT
        assert_eq!(app.active_module, crate::ui::WorkspaceModule::Print);
        app.execute_menu_action(156); // ACTION_MODULE_WEB
        assert_eq!(app.active_module, crate::ui::WorkspaceModule::Web);
        app.execute_menu_action(150); // ACTION_MODULE_LIBRARY
        assert_eq!(app.active_module, crate::ui::WorkspaceModule::Library);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_presets_xmp_import_export_and_copy_paste_workflow() {
        use crate::catalog::CatalogDb;
        use crate::params::DevelopParams;
        use crate::presets::{apply_settings_with_mask, export_preset_to_xmp, parse_xmp_preset, CopySettingsMask};
        use crate::LightRawApp;

        let temp_dir = std::env::temp_dir().join(format!(
            "lightraw_test_presets_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(&temp_dir);

        // 1. Test XMP Preset Parsing from raw XML
        let sample_xmp = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="Adobe XMP Core 7.0-c000 1.000000, 0000/00/00-00:00:00        ">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/"
   crs:Version="18.4"
   crs:ProcessVersion="15.4"
   crs:WhiteBalance="Custom"
   crs:Temperature="3936"
   crs:Tint="+5"
   crs:Exposure2012="+0.50"
   crs:Contrast2012="-7"
   crs:Highlights2012="-59"
   crs:Shadows2012="+32"
   crs:Whites2012="-59"
   crs:Blacks2012="+9"
   crs:Texture="-12"
   crs:Clarity2012="+1"
   crs:Dehaze="-2"
   crs:Vibrance="0"
   crs:Saturation="0"
   crs:Sharpness="21"
   crs:Name="XII SMK 26"
   crs:Group="A7">
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#;

        let parsed = parse_xmp_preset(sample_xmp, "XII SMK 26").expect("Failed to parse sample XMP");
        assert_eq!(parsed.name, "XII SMK 26");
        assert_eq!(parsed.category, "A7");
        assert_eq!(parsed.params.temperature, 3936.0);
        assert_eq!(parsed.params.tint, 5.0);
        assert_eq!(parsed.params.exposure_2012, 0.50);
        assert_eq!(parsed.params.contrast_2012, -7.0);
        assert_eq!(parsed.params.highlights_2012, -59.0);
        assert_eq!(parsed.params.shadows_2012, 32.0);
        assert_eq!(parsed.params.whites_2012, -59.0);
        assert_eq!(parsed.params.blacks_2012, 9.0);
        assert_eq!(parsed.params.texture, -12.0);
        assert_eq!(parsed.params.clarity_2012, 1.0);
        assert_eq!(parsed.params.dehaze, -2.0);
        assert_eq!(parsed.params.sharpness, 21.0);

        // 2. Test XMP Preset Exporting & Round-trip
        let exported_xmp = export_preset_to_xmp(&parsed);
        assert!(exported_xmp.contains("crs:Name=\"XII SMK 26\""));
        assert!(exported_xmp.contains("crs:Group=\"A7\""));
        assert!(exported_xmp.contains("crs:Temperature=\"3936\""));
        assert!(exported_xmp.contains("crs:Exposure2012=\"+0.50\""));

        let round_trip = parse_xmp_preset(&exported_xmp, "XII SMK 26").expect("Failed to round-trip XMP");
        assert_eq!(round_trip.params.temperature, 3936.0);
        assert_eq!(round_trip.params.exposure_2012, 0.50);
        assert_eq!(round_trip.params.highlights_2012, -59.0);

        // 3. Test Selective Copy Settings Mask
        let mut dest_params = DevelopParams::default();
        dest_params.crop_angle = 15.0; // Existing crop
        dest_params.crop_top = 0.1;

        let mask = CopySettingsMask::default();
        assert!(mask.basic_exposure);
        assert!(!mask.crop_group); // Crop is unchecked in default Lightroom copy mask

        apply_settings_with_mask(&mut dest_params, &parsed.params, &mask);
        assert_eq!(dest_params.exposure_2012, 0.50);
        assert_eq!(dest_params.temperature, 3936.0);
        assert_eq!(dest_params.crop_angle, 15.0); // Crop was preserved
        assert_eq!(dest_params.crop_top, 0.1);

        // 4. Test Catalog Database AgDevelopPreset Table CRUD
        let catalog_db = CatalogDb::open_or_create(&temp_dir);
        let mut test_preset = parsed.clone();
        test_preset.id = "preset_test_001".to_string();
        test_preset.is_user = true;
        test_preset.is_favorite = true;

        catalog_db.save_preset(&test_preset);
        let loaded = catalog_db.load_presets();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "XII SMK 26");
        assert!(loaded[0].is_favorite);

        catalog_db.update_preset_favorite("preset_test_001", false);
        let loaded_after_unfav = catalog_db.load_presets();
        assert!(!loaded_after_unfav[0].is_favorite);

        catalog_db.delete_preset("preset_test_001");
        assert!(catalog_db.load_presets().is_empty());

        // 5. Test Multi-Select Selective Paste Workflow in LightRawApp
        let mut app = LightRawApp::new(temp_dir.clone());
        for i in 1..=4 {
            let mut p = crate::catalog::create_empty_placeholder_photo();
            p.id = i;
            p.file_name = format!("IMG_{:04}.CR2", i);
            p.real_file_path = Some(format!("/Photos/IMG_{:04}.CR2", i));
            app.catalog_db.save_photo(&p);
            app.photos.push(p);
        }

        // Copy settings from photo 1
        app.active_photo_id = 1;
        app.active_photo_mut().params.exposure_2012 = 1.25;
        app.active_photo_mut().params.contrast_2012 = 25.0;
        app.active_photo_mut().params.crop_angle = 5.0;

        app.clipboard_params = Some(app.active_photo().params.clone());
        app.clipboard_mask = CopySettingsMask::default(); // Excludes crop

        // Multi-select photos 2, 3, 4
        app.selected_photo_ids = vec![2, 3, 4];
        app.paste_clipboard_settings();

        for id in [2, 3, 4] {
            let p = app.photos.iter().find(|x| x.id == id).unwrap();
            assert_eq!(p.params.exposure_2012, 1.25);
            assert_eq!(p.params.contrast_2012, 25.0);
            assert_eq!(p.params.crop_angle, 0.0); // Crop was NOT overwritten
            assert_eq!(p.history.last().unwrap().name, "Paste Settings");
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_decode_dsc00005() {
        let path = std::path::Path::new("/Users/mustamar/Pictures/Raw Foto SMA-SMK/DSC00005.ARW");
        if path.exists() {
            let t_dec = std::time::Instant::now();
            let img = crate::catalog::decode_real_photo_file(path, 0, 0).expect("failed decode");
            println!("Decoded DSC00005: {}x{}, rgb len={} in {:?}", img.width, img.height, img.rgb.len(), t_dec.elapsed());
            let w = img.width as usize;
            let bg_idx = (img.height as usize / 10 * w + w / 2) * 3;
            println!("Top bg pixel: ({}, {}, {})", img.rgb[bg_idx], img.rgb[bg_idx+1], img.rgb[bg_idx+2]);

            // Test pipeline with XII SMK 26 preset
            let xmp_path = std::path::Path::new("/Users/mustamar/.gemini/antigravity/brain/0686adf6-62dc-43eb-8cf0-1cf9fe7449ca/.user_uploaded/media_1793982082797_8bced4b9.xmp");
            if xmp_path.exists() {
                let xml = std::fs::read_to_string(xmp_path).unwrap();
                let preset = crate::presets::parse_xmp_preset(&xml, "XII SMK 26").unwrap();
                let mut draft_opts = crate::pipeline::RenderOptions::default();
                draft_opts.is_draft_interactive = true;
                let t_draft = std::time::Instant::now();
                let draft_rendered = crate::pipeline::render_develop_pipeline(
                    &img,
                    &preset.params,
                    draft_opts,
                );
                println!("Rendered draft {}x{} pipeline in {:?}", draft_rendered.width, draft_rendered.height, t_draft.elapsed());
                let t_ren = std::time::Instant::now();
                let rendered = crate::pipeline::render_develop_pipeline(
                    &img,
                    &preset.params,
                    crate::pipeline::RenderOptions::default(),
                );
                println!("Rendered full {}x{} pipeline in {:?}", rendered.width, rendered.height, t_ren.elapsed());
                assert_eq!(rendered.width, img.width);
                assert_eq!(rendered.height, img.height);
                let out_w = rendered.width as usize;
                let out_h = rendered.height as usize;
                let sample_roi = |nx: f32, ny: f32| -> (u8, u8, u8) {
                    let sx = ((nx * out_w as f32) as usize).clamp(2, out_w.saturating_sub(3));
                    let sy = ((ny * out_h as f32) as usize).clamp(2, out_h.saturating_sub(3));
                    let (mut r, mut g, mut b, mut cnt) = (0u32, 0u32, 0u32, 0u32);
                    for dy in -2i32..=2 {
                        for dx in -2i32..=2 {
                            let x = (sx as i32 + dx) as usize;
                            let y = (sy as i32 + dy) as usize;
                            let idx = (y * out_w + x) * 4;
                            r += rendered.rgba[idx] as u32;
                            g += rendered.rgba[idx + 1] as u32;
                            b += rendered.rgba[idx + 2] as u32;
                            cnt += 1;
                        }
                    }
                    ((r / cnt) as u8, (g / cnt) as u8, (b / cnt) as u8)
                };

                let top_bg = sample_roi(0.50, 0.08);
                let left_cheek = sample_roi(0.46, 0.30);
                let right_cheek = sample_roi(0.53, 0.30);
                let hijab = sample_roi(0.42, 0.45);
                let shirt = sample_roi(0.36, 0.65);
                let tie = sample_roi(0.49, 0.62);
                println!("=== DSC00005.ARW + XII SMK 26.xmp ROI sRGB ===");
                println!("  Top Blue BG : {:?} (LrC target ~(0, 138, 225))", top_bg);
                println!("  Left Cheek  : {:?} (LrC target ~(217, 186, 188))", left_cheek);
                println!("  Right Cheek : {:?}", right_cheek);
                println!("  White Hijab : {:?} (LrC target ~(192, 208, 233))", hijab);
                println!("  White Shirt : {:?} (LrC target ~(194, 215, 237))", shirt);
                println!("  Teal Tie    : {:?} (LrC target ~(92, 155, 184))", tie);

                // Save rendered frame to JPEG
                let dyn_img = image::RgbaImage::from_raw(rendered.width, rendered.height, rendered.rgba).unwrap();
                let out_path = "/Users/mustamar/.gemini/antigravity/brain/0686adf6-62dc-43eb-8cf0-1cf9fe7449ca/lightraw_pipeline_rendered.jpg";
                image::DynamicImage::ImageRgba8(dyn_img).to_rgb8().save(out_path).unwrap();
                println!("Saved rendered test image to {}", out_path);
            }
        }
    }

    #[test]
    fn test_preset_amount_interpolation_full_156_xmp_and_multipage_pdf() {
        use crate::params::{
            CameraProfile, DevelopParams, LocalAdjustParams, MaskComponent, MaskGroup, MaskOp,
            MaskPrimitiveKind, PointColorSwatch, VignetteStyle, WhiteBalanceMode,
        };
        use crate::presets::{
            export_preset_to_xmp, interpolate_preset_amount, parse_xmp_preset, CopySettingsMask,
            DevelopPreset,
        };
        use crate::ui::export_engine::write_multipage_jpeg_pdf;

        // 1. Preset Amount (0..200) live delta scaling
        let mut base = DevelopParams::default();
        base.exposure_2012 = 0.20;
        base.highlights_2012 = -10.0;

        let mut target = base.clone();
        target.exposure_2012 = 1.20; // delta +1.00
        target.highlights_2012 = -70.0; // delta -60.0
        target.hue_adjustment[1] = -14.0;
        target.grain_amount = 30.0;

        let at_0 = interpolate_preset_amount(&base, &target, 0.0);
        let at_100 = interpolate_preset_amount(&base, &target, 100.0);
        let at_200 = interpolate_preset_amount(&base, &target, 200.0);

        assert!((at_0.exposure_2012 - 0.20).abs() < 1e-4);
        assert!((at_0.highlights_2012 - (-10.0)).abs() < 1e-4);
        assert!((at_100.exposure_2012 - 1.20).abs() < 1e-4);
        assert!((at_100.highlights_2012 - (-70.0)).abs() < 1e-4);
        assert!((at_200.exposure_2012 - 2.20).abs() < 1e-4);
        assert!((at_200.highlights_2012 - (-130.0_f32).clamp(-100.0, 100.0)).abs() < 1e-3);
        assert!((at_200.hue_adjustment[1] - (-28.0)).abs() < 1e-3);

        // 2. Full 156-field .xmp round-trip (Point Curves, Point Color, Color Grading, Calibration, Masks)
        let mut rich = DevelopParams::default();
        rich.camera_profile = CameraProfile::AdobeVivid;
        rich.look_amount = 115.0;
        rich.white_balance = WhiteBalanceMode::Custom;
        rich.temperature = 4850.0;
        rich.tint = 12.0;
        rich.exposure_2012 = 0.65;
        rich.tone_curve_pv2012 = vec![(0.0, 0.03), (0.25, 0.20), (0.75, 0.82), (1.0, 0.98)];
        rich.hue_adjustment = [5.0, -10.0, 8.0, -4.0, 12.0, -18.0, 3.0, -6.0];
        rich.point_colors.push(PointColorSwatch {
            name: "Skin".into(),
            src_hue: 22.0,
            src_sat: 0.55,
            src_lum: 0.62,
            hue_shift: -6.0,
            sat_shift: -12.0,
            lum_shift: 14.0,
            range_amount: 60.0,
            hue_range: 45.0,
            sat_range: 50.0,
            lum_range: 55.0,
        });
        rich.color_grade_midtone_hue = 36.0;
        rich.color_grade_midtone_sat = 18.0;
        rich.post_crop_vignette_style = VignetteStyle::ColorPriority;
        rich.post_crop_vignette_amount = -24.0;
        rich.grain_amount = 22.0;
        rich.blue_hue = -14.0;
        rich.blue_saturation = 26.0;
        rich.mask_groups.push(MaskGroup {
            id: 1,
            name: "Subject Pop".into(),
            enabled: true,
            invert: false,
            components: vec![MaskComponent {
                name: "Subject".into(),
                op: MaskOp::Add,
                invert: false,
                kind: MaskPrimitiveKind::AiSubject,
            }],
            local: LocalAdjustParams {
                exposure: 0.35,
                clarity: 18.0,
                ..Default::default()
            },
        });

        let preset = DevelopPreset {
            id: "rich_rt".into(),
            category: "Pro Suite".into(),
            name: "Full 156 Field Test".into(),
            is_user: true,
            params: rich,
            is_favorite: true,
            mask: Some(CopySettingsMask::all()),
            is_visible: true,
            xmp_file_path: None,
        };
        let xml = export_preset_to_xmp(&preset);
        let reparsed = parse_xmp_preset(&xml, "Full 156 Field Test").expect("reparse rich XMP");
        assert_eq!(reparsed.params.camera_profile, CameraProfile::AdobeVivid);
        assert!((reparsed.params.temperature - 4850.0).abs() < 0.5);
        assert_eq!(reparsed.params.tone_curve_pv2012.len(), 4);
        assert!((reparsed.params.tone_curve_pv2012[0].1 - 0.03).abs() < 0.01);
        assert!((reparsed.params.hue_adjustment[5] - (-18.0)).abs() < 0.5);
        assert!(!reparsed.params.point_colors.is_empty());
        assert!((reparsed.params.blue_saturation - 26.0).abs() < 0.5);
        assert!(!reparsed.params.mask_groups.is_empty());

        // 3. Multi-page PDF 1.4 export verification
        let pdf_path = std::env::temp_dir().join(format!("lightraw_book_test_{}.pdf", std::process::id()));
        let page1 = (320u32, 240u32, vec![210u8; 320 * 240 * 4]);
        let page2 = (320u32, 240u32, vec![90u8; 320 * 240 * 4]);
        write_multipage_jpeg_pdf(&pdf_path, &[page1, page2], 300.0).expect("write multi-page PDF");
        let pdf_bytes = std::fs::read(&pdf_path).expect("read PDF");
        assert!(pdf_bytes.starts_with(b"%PDF-1.4"));
        let pdf_str = String::from_utf8_lossy(&pdf_bytes);
        assert!(pdf_str.contains("/Count 2"));
        let _ = std::fs::remove_file(&pdf_path);
    }

    #[test]
    fn test_ai_hardware_tier_and_description() {
        let desc = crate::ai::get_hardware_description();
        assert!(!desc.is_empty());
        println!("Active Hardware Engine: {}", desc);

        for tier in crate::ai::AiHardwareTier::ALL {
            assert!(!tier.label().is_empty());
            let json = serde_json::to_string(&tier).expect("serialize AiHardwareTier");
            let deserialized: crate::ai::AiHardwareTier =
                serde_json::from_str(&json).expect("deserialize AiHardwareTier");
            assert_eq!(tier, deserialized);
        }
    }

    #[test]
    fn test_ai_semantics_and_masking() {
        let width = 64;
        let height = 64;
        let sem_w = 32;
        let sem_h = 32;
        let rgb = vec![0.5f32; width * height * 3];

        let maps = crate::ai::compute_ai_semantics(
            &rgb,
            width as u32,
            height as u32,
            sem_w as u32,
            sem_h as u32,
            crate::ai::AiHardwareTier::Auto,
            None,
        );

        assert_eq!(maps.subject_q.len(), (sem_w * sem_h) as usize);
        assert_eq!(maps.sky_q.len(), (sem_w * sem_h) as usize);
        assert_eq!(maps.people_q.len(), (sem_w * sem_h) as usize);
        assert_eq!(maps.depth_q.len(), (sem_w * sem_h) as usize);
    }

    #[test]
    fn test_ai_generative_remove_and_denoise() {
        // 1. Inpaint Generative Remove
        let spot = RetouchArea {
            mode: RetouchMode::Remove,
            target_x: 0.5,
            target_y: 0.5,
            source_x: 0.5,
            source_y: 0.5,
            radius: 0.1,
            feather: 50.0,
            opacity: 100.0,
            points: Vec::new(),
        };
        let sample_fn = |_u: f32, _v: f32| (0.8f32, 0.4f32, 0.2f32);
        let (r, g, b) = crate::ai::sample_generative_remove(0.5, 0.5, &spot, sample_fn);
        assert!(!r.is_nan() && r >= 0.0);
        assert!(!g.is_nan() && g >= 0.0);
        assert!(!b.is_nan() && b >= 0.0);

        // 2. AI Denoise
        let mut noisy_rgb = vec![0.5f32; 16 * 16 * 3];
        crate::ai::run_ai_denoise(&mut noisy_rgb, 16, 16, 50.0, true);
        for &val in &noisy_rgb {
            assert!(!val.is_nan());
            assert!(val >= 0.0);
        }
    }

    #[test]
    fn test_watermark_engine_and_presets() {
        use crate::watermark::*;

        // 1. Presets & Defaults
        let presets = WatermarkPreset::default_presets();
        assert!(presets.len() >= 3);
        assert_eq!(presets[0].name, "Simple Copyright Watermark");

        let mut s = WatermarkSettings::default();
        assert_eq!(s.style, WatermarkStyle::Text);
        assert_eq!(s.anchor, WatermarkAnchor::BottomCenter);
        assert_eq!(s.proportional_size, 26.0);
        assert_eq!(s.opacity, 100.0);
        assert!(s.shadow_enabled);

        // 2. Rotation
        assert_eq!(s.rotation.degrees(), 0.0);
        s.rotation = s.rotation.rotate_cw();
        assert_eq!(s.rotation, WatermarkRotation::Deg90);
        assert_eq!(s.rotation.degrees(), 90.0);
        s.rotation = s.rotation.rotate_ccw();
        assert_eq!(s.rotation, WatermarkRotation::Deg0);

        // 3. Anchor Points Grid 3x3
        for r in 0..3 {
            for c in 0..3 {
                let a = WatermarkAnchor::from_row_col(r, c);
                assert_eq!(a.row_col(), (r, c));
            }
        }

        // 4. Compositing on test buffer
        let w = 200u32;
        let h = 200u32;
        let mut buf = vec![50u8; (w * h * 4) as usize];
        composite_watermark_rgba(&mut buf, w, h, &s);
        // Verify pixels were painted (not all still 50)
        let changed = buf.iter().any(|&px| px != 50);
        assert!(changed, "Watermark compositor should modify target pixel buffer");
    }

    #[test]
    fn test_crop_synchronization_across_library() {
        use crate::params::AspectRatioPreset;
        use crate::ui::LightRawApp;

        let tmp = std::env::temp_dir().join(format!(
            "lightraw_crop_sync_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        let mut app = LightRawApp::new(tmp.clone());

        // Create 3 dummy photos
        let mut p1 = create_empty_placeholder_photo();
        p1.id = 101;
        p1.file_name = "photo1.dng".into();

        let mut p2 = create_empty_placeholder_photo();
        p2.id = 102;
        p2.file_name = "photo2.dng".into();

        let mut p3 = create_empty_placeholder_photo();
        p3.id = 103;
        p3.file_name = "photo3.dng".into();

        app.photos = vec![p1, p2, p3];
        app.active_photo_id = 101;

        // Configure active photo crop
        let mut crop_params = app.active_photo().params.clone();
        crop_params.crop_left = 0.15;
        crop_params.crop_top = 0.20;
        crop_params.crop_right = 0.85;
        crop_params.crop_bottom = 0.90;
        crop_params.crop_angle = 3.5;
        crop_params.crop_aspect_preset = AspectRatioPreset::Ratio16x9;
        crop_params.crop_constrain_aspect_ratio = true;
        crop_params.crop_aspect_flipped = true;
        app.active_photo_mut().params = crop_params.clone();

        // 1. Sync to All Photos
        app.sync_crop_to_library(false);

        for p in &app.photos {
            assert_eq!(p.params.crop_left, 0.15);
            assert_eq!(p.params.crop_top, 0.20);
            assert_eq!(p.params.crop_right, 0.85);
            assert_eq!(p.params.crop_bottom, 0.90);
            assert_eq!(p.params.crop_angle, 3.5);
            assert_eq!(p.params.crop_aspect_preset, AspectRatioPreset::Ratio16x9);
            assert!(p.params.crop_constrain_aspect_ratio);
            assert!(p.params.crop_aspect_flipped);
            if p.id != 101 {
                assert_eq!(p.history.last().unwrap().name, "Sync Crop");
            }
        }

        // 2. Modify active photo crop and sync only to selected (photo 102 only)
        app.active_photo_mut().params.crop_angle = -12.0;
        app.active_photo_mut().params.crop_aspect_preset = AspectRatioPreset::Ratio1x1;
        app.selected_photo_ids.clear();
        app.selected_photo_ids.push(101);
        app.selected_photo_ids.push(102);

        app.sync_crop_to_library(true);

        assert_eq!(app.photos[1].params.crop_angle, -12.0);
        assert_eq!(app.photos[1].params.crop_aspect_preset, AspectRatioPreset::Ratio1x1);
        // Photo 103 was not selected, so its crop remains unchanged (angle = 3.5)
        assert_eq!(app.photos[2].params.crop_angle, 3.5);
        assert_eq!(app.photos[2].params.crop_aspect_preset, AspectRatioPreset::Ratio16x9);

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
