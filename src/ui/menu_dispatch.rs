// ============================================================================
// LightRaw — Native macOS Menu Dispatcher (Actions 1..=194) & Global Shortcuts
// ============================================================================

use super::{
    BeforeAfterMode, Colors, DevelopTool, LibraryFilterMode, LibraryViewMode, LightRawApp,
    WorkspaceModule,
};
use crate::catalog::{ColorLabel, PickFlag};
use crate::params::DevelopParams;
use crate::presets::CopySettingsMask;
use eframe::egui::{self, Color32, Stroke, Vec2};
use std::path::Path;
use std::sync::{Mutex, OnceLock};

#[cfg(target_os = "macos")]
extern "C" {
    pub fn lightraw_set_text_input_focused(focused: std::os::raw::c_int);
}

static NATIVE_MENU_QUEUE: Mutex<Vec<u32>> = Mutex::new(Vec::new());
static EGUI_CTX: OnceLock<Mutex<Option<egui::Context>>> = OnceLock::new();

pub fn register_egui_context_for_menu(ctx: &egui::Context) {
    let lock = EGUI_CTX.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = lock.lock() {
        if guard.is_none() {
            *guard = Some(ctx.clone());
        }
    }
}

#[no_mangle]
pub extern "C" fn lightraw_dispatch_menu_action(action_id: u32) {
    if let Ok(mut q) = NATIVE_MENU_QUEUE.lock() {
        q.push(action_id);
    }
    if let Some(lock) = EGUI_CTX.get() {
        if let Ok(guard) = lock.lock() {
            if let Some(ref ctx) = *guard {
                ctx.request_repaint();
            }
        }
    }
}

impl LightRawApp {
    pub fn drain_native_menu_actions(&mut self) {
        let pending: Vec<u32> = if let Ok(mut q) = NATIVE_MENU_QUEUE.lock() {
            q.drain(..).collect()
        } else {
            Vec::new()
        };
        for action_id in pending {
            self.execute_menu_action(action_id);
        }
    }

    pub fn execute_menu_action(&mut self, action_id: u32) {
        match action_id {
            // 0. App Menu
            1 => {
                self.show_about_modal = true;
                self.about_tab = 0;
            }
            2 => {
                self.show_system_info_modal = true;
                self.show_about_modal = true;
                self.about_tab = 1;
            }
            3 => self.show_preferences_modal = true,

            // 1. File Menu
            10 => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Create New LightRaw Catalog (.lrcat)")
                    .set_file_name("NewLightRawCatalog.lrcat")
                    .add_filter("Lightroom Catalog", &["lrcat"])
                    .save_file()
                {
                    self.open_or_create_catalog_at(&path, true);
                }
            }
            11 => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Open LightRaw Catalog (.lrcat)")
                    .add_filter("Lightroom Catalog", &["lrcat", "sqlite", "db"])
                    .pick_file()
                {
                    self.open_or_create_catalog_at(&path, false);
                }
            }
            12 => {
                match self.catalog_db.optimize() {
                    Ok(()) => {
                        self.status_message =
                            "Catalog optimized (SQLite VACUUM & ANALYZE completed).".into();
                    }
                    Err(e) => {
                        self.status_message = format!("Catalog optimize warning: {}", e);
                    }
                }
            }
            13 => {
                self.rename_catalog_input = self.catalog_display_name.clone();
                self.show_rename_catalog_modal = true;
            }
            14 => self.show_import_modal = true,
            15 => self.show_export_modal = true,
            16 | 173 => self.execute_batch_export(),
            17 => {
                self.selected_collection_id = Some(1);
                self.selected_folder = None;
                self.active_module = WorkspaceModule::Library;
                self.status_message = "Showing Quick Collection.".into();
            }
            18 => {
                if let Some(col) = self.collections.iter_mut().find(|c| c.id == 1) {
                    col.photo_ids.clear();
                    let cloned = col.clone();
                    self.catalog_db.save_collection(&cloned);
                }
                self.status_message = "Cleared Quick Collection.".into();
            }
            19 => self.run_native_page_setup(),
            20 => {
                self.active_module = WorkspaceModule::Print;
                self.print_sheet_to_printer();
            }
            21 => {
                self.create_collection_is_smart = false;
                self.new_collection_name = "Saved Quick Collection".into();
                self.show_create_collection_modal = true;
            }

            // 2. Edit Menu
            30 => self.undo_active_photo(),
            31 => self.redo_active_photo(),
            32 => {
                if !self.photos.is_empty() {
                    self.clipboard_params = Some(self.active_photo().params.clone());
                    self.clipboard_mask = CopySettingsMask::all_checked();
                    self.reset_selected_photos_settings();
                    self.status_message = "Cut Develop settings to clipboard.".into();
                }
            }
            33 => {
                if !self.photos.is_empty() {
                    self.clipboard_params = Some(self.active_photo().params.clone());
                    self.show_copy_settings_modal = true;
                }
            }
            34 => self.paste_clipboard_settings(),
            35 => self.delete_active_tool_item_or_photo(),
            36 => self.select_all(),
            37 => self.deselect_all(),
            38 => {
                let all_ids = self.filtered_photo_ids();
                let inverted: Vec<i64> = all_ids
                    .into_iter()
                    .filter(|id| !self.selected_photo_ids.contains(id))
                    .collect();
                if let Some(&first) = inverted.first() {
                    self.active_photo_id = first;
                    self.selected_photo_ids = inverted;
                }
            }
            39 => {
                let flagged: Vec<i64> = self
                    .filtered_photo_ids()
                    .into_iter()
                    .filter(|id| {
                        self.photos
                            .iter()
                            .find(|p| p.id == *id)
                            .map(|p| p.flag == PickFlag::Picked)
                            .unwrap_or(false)
                    })
                    .collect();
                if let Some(&first) = flagged.first() {
                    self.active_photo_id = first;
                    self.selected_photo_ids = flagged;
                }
            }
            40 => {
                self.selected_photo_ids.retain(|id| {
                    self.photos
                        .iter()
                        .find(|p| p.id == *id)
                        .map(|p| p.flag != PickFlag::Unflagged)
                        .unwrap_or(false)
                });
                if self.selected_photo_ids.is_empty() && !self.photos.is_empty() {
                    self.selected_photo_ids.push(self.active_photo_id);
                }
            }
            41 => self.deselect_all(),
            42 => {
                if self.selected_photo_ids.len() > 1 {
                    let old = self.active_photo_id;
                    self.selected_photo_ids.retain(|&x| x != old);
                    if let Some(&first) = self.selected_photo_ids.first() {
                        self.active_photo_id = first;
                    }
                }
            }
            43 => self.show_watermark_editor_modal = true,

            // 3. Develop / Library Menu
            50 => {
                if !self.photos.is_empty() {
                    let count = self.active_photo().snapshots.len() + 1;
                    self.new_snapshot_name = format!("Snapshot {}", count);
                    self.show_create_snapshot_modal = true;
                }
            }
            51 => {
                self.new_preset_name = "Untitled Preset".into();
                self.create_preset_mask = CopySettingsMask::default();
                self.show_create_preset_modal = true;
            }
            52 => {
                if !self.photos.is_empty() {
                    let cur = self.active_photo().params.clone();
                    let p = self.active_photo_mut();
                    p.history.truncate(1);
                    if p.history.is_empty() {
                        p.history.push(crate::catalog::HistoryStep {
                            name: "Import RAW".into(),
                            value_summary: String::new(),
                            timestamp: "Initial".into(),
                            params: DevelopParams::default(),
                        });
                    }
                    p.params = cur;
                    p.history_index = p.history.len() - 1;
                    self.persist_active_photo();
                    self.status_message = "Cleared Develop history.".into();
                }
            }
            53 => self.step_active_photo(-1),
            54 => self.step_active_photo(1),

            // 4. Photo Menu
            60 => {
                self.toggle_target_collection_for_selection();
                self.step_active_photo(1);
            }
            61 => self.reveal_active_photo_in_finder(),
            62 => self.create_virtual_copy(),
            63 => self.rotate_selection(-90.0),
            64 => self.rotate_selection(90.0),
            65 => self.flip_selection(true),
            66 => self.flip_selection(false),
            67 => self.set_flag_on_selection(PickFlag::Picked),
            68 => self.set_flag_on_selection(PickFlag::Unflagged),
            69 => self.set_flag_on_selection(PickFlag::Rejected),
            70..=75 => self.set_rating_on_selection((action_id - 70) as u8),
            76 => self.set_color_label_on_selection(ColorLabel::None),
            77 => self.set_color_label_on_selection(ColorLabel::Red),
            78 => self.set_color_label_on_selection(ColorLabel::Yellow),
            79 => self.set_color_label_on_selection(ColorLabel::Green),
            80 => self.set_color_label_on_selection(ColorLabel::Blue),
            81 => self.set_color_label_on_selection(ColorLabel::Purple),
            82 | 184 => self.remove_selected_photos_from_catalog(),
            83 => self.save_xmp_sidecars_for_selection(),
            84 => self.delete_rejected_photos_from_catalog(),
            85 => {
                if !self.photos.is_empty() {
                    let mut p = self.active_photo().params.clone();
                    p.ai_denoise_amount = 50.0;
                    p.sharpness = p.sharpness.max(45.0);
                    self.commit_params_change("AI Enhance & Denoise", p);
                }
            }

            // 5. Settings Menu
            90 => self.reset_selected_photos_settings(),
            91 => {
                if !self.photos.is_empty() {
                    self.show_copy_settings_modal = true;
                }
            }
            92 => self.paste_clipboard_settings(),
            93 => {
                if !self.photos.is_empty() {
                    self.show_sync_settings_modal = true;
                }
            }
            94 => self.apply_auto_tone_to_selection(),
            95 => self.apply_auto_wb_to_selection(),
            96 => {
                if !self.photos.is_empty() {
                    let next_bw = !self.active_photo().params.convert_to_grayscale;
                    let ids = self.effective_target_ids();
                    for id in ids {
                        if let Some(p) = self.photos.iter_mut().find(|x| x.id == id) {
                            p.params.convert_to_grayscale = next_bw;
                            self.catalog_db.save_photo(p);
                            self.invalidate_photo_render_caches(id);
                        }
                    }
                    self.status_message = if next_bw {
                        "Converted to Black & White.".into()
                    } else {
                        "Restored Color treatment.".into()
                    };
                }
            }
            97 => {
                if !self.photos.is_empty() {
                    let mut p = self.active_photo().params.clone();
                    p.hdr_edit_mode = !p.hdr_edit_mode;
                    self.commit_params_change("Toggle HDR Edit Mode", p);
                }
            }
            98 => {
                if !self.photos.is_empty() {
                    let mut p = self.active_photo().params.clone();
                    p.crop_left = 0.0;
                    p.crop_top = 0.0;
                    p.crop_right = 1.0;
                    p.crop_bottom = 1.0;
                    p.crop_angle = 0.0;
                    self.commit_params_change("Reset Crop", p);
                }
            }
            99 => {
                if !self.photos.is_empty() {
                    let next_constrain = !self.active_photo().params.crop_constrain_aspect_ratio;
                    self.active_photo_mut().params.crop_constrain_aspect_ratio = next_constrain;
                    self.persist_active_photo();
                }
            }
            100 => {
                if !self.photos.is_empty() {
                    let id = self.active_photo_id;
                    let p = self.active_photo_mut();
                    let old_before = p.before_params();
                    let old_after = p.params.clone();
                    p.params = old_before;
                    p.set_before_params(old_after);
                    let cloned = p.clone();
                    self.catalog_db.save_photo(&cloned);
                    self.invalidate_photo_render_caches(id);
                    self.status_message = "Swapped Before and After settings.".into();
                }
            }

            // 6. Tools Menu
            110 => {
                self.active_module = WorkspaceModule::Develop;
                self.develop_tool = if self.develop_tool == DevelopTool::Crop {
                    DevelopTool::None
                } else {
                    DevelopTool::Crop
                };
            }
            111 => {
                self.active_module = WorkspaceModule::Develop;
                self.develop_tool = if self.develop_tool == DevelopTool::RedEye {
                    DevelopTool::None
                } else {
                    DevelopTool::RedEye
                };
            }
            112 => {
                self.active_module = WorkspaceModule::Develop;
                self.develop_tool = if self.develop_tool == DevelopTool::Remove {
                    DevelopTool::None
                } else {
                    DevelopTool::Remove
                };
            }
            113 => {
                self.active_module = WorkspaceModule::Develop;
                self.develop_tool = if self.develop_tool == DevelopTool::Masking {
                    DevelopTool::None
                } else {
                    DevelopTool::Masking
                };
            }
            114 => {
                self.active_module = WorkspaceModule::Develop;
                self.develop_tool = DevelopTool::None;
                self.wb_eyedropper_active = !self.wb_eyedropper_active;
            }
            115 => {
                self.active_module = WorkspaceModule::Develop;
                self.develop_tool = if self.develop_tool == DevelopTool::TargetAdjust {
                    DevelopTool::None
                } else {
                    DevelopTool::TargetAdjust
                };
            }
            116 => {
                self.active_module = WorkspaceModule::Develop;
                self.dev_transform_open = true;
                if !self.photos.is_empty() {
                    self.active_photo_mut().params.perspective_upright =
                        crate::params::UprightMode::Guided;
                }
                self.develop_tool = DevelopTool::GuidedUpright;
            }

            // 7. View Menu
            120 => self.show_toolbar = !self.show_toolbar,
            121 => {
                self.info_overlay_cycle = (self.info_overlay_cycle + 1) % 3;
                self.show_info_overlay = self.info_overlay_cycle > 0;
            }
            122 => {
                if self.loupe_zoom > 1.05 {
                    self.loupe_zoom = 1.0;
                    self.loupe_pan = egui::Vec2::ZERO;
                } else {
                    self.loupe_zoom = 2.5;
                }
            }
            123 => self.loupe_zoom = (self.loupe_zoom * 1.25).clamp(0.5, 8.0),
            124 => {
                self.loupe_zoom = (self.loupe_zoom / 1.25).clamp(0.5, 8.0);
                if self.loupe_zoom <= 1.02 {
                    self.loupe_pan = egui::Vec2::ZERO;
                }
            }
            125 => self.loupe_zoom = 2.5,
            126 => {
                self.active_module = WorkspaceModule::Develop;
                self.before_after_mode = match self.before_after_mode {
                    BeforeAfterMode::Single => BeforeAfterMode::SplitLeftRight,
                    BeforeAfterMode::SplitLeftRight => BeforeAfterMode::SplitTopBottom,
                    BeforeAfterMode::SplitTopBottom => BeforeAfterMode::SideBySide,
                    BeforeAfterMode::SideBySide => BeforeAfterMode::BeforeOnly,
                    BeforeAfterMode::BeforeOnly | BeforeAfterMode::ReferenceView => {
                        BeforeAfterMode::Single
                    }
                };
            }
            127 => self.soft_proofing = !self.soft_proofing,
            128 => {
                let next = !(self.show_shadow_clipping || self.show_highlight_clipping);
                self.show_shadow_clipping = next;
                self.show_highlight_clipping = next;
            }
            129 => {
                self.active_module = WorkspaceModule::Library;
                self.library_view_mode = LibraryViewMode::Grid;
            }
            130 => {
                self.active_module = WorkspaceModule::Library;
                self.library_view_mode = LibraryViewMode::Loupe;
            }
            131 => {
                self.active_module = WorkspaceModule::Library;
                self.library_view_mode = LibraryViewMode::Compare;
            }
            132 => {
                self.active_module = WorkspaceModule::Library;
                self.library_view_mode = LibraryViewMode::Survey;
            }
            133 => {
                self.active_module = WorkspaceModule::Library;
                self.library_view_mode = LibraryViewMode::People;
            }

            // 8. Window & Modules Menu
            140 => {
                self.active_module = WorkspaceModule::Slideshow;
                self.slideshow_playing = true;
                self.slideshow_fullscreen = true;
                self.slideshow_last_tick = Some(std::time::Instant::now());
            }
            141 => self.lights_out = self.lights_out.next(),
            142 => self.show_left_panel = !self.show_left_panel,
            143 => self.show_right_panel = !self.show_right_panel,
            144 => self.show_bottom_filmstrip = !self.show_bottom_filmstrip,
            150 => self.active_module = WorkspaceModule::Library,
            151 => self.active_module = WorkspaceModule::Develop,
            152 => self.active_module = WorkspaceModule::Map,
            153 => self.active_module = WorkspaceModule::Book,
            154 => self.active_module = WorkspaceModule::Slideshow,
            155 => self.active_module = WorkspaceModule::Print,
            156 => self.active_module = WorkspaceModule::Web,

            // 9. Help Menu
            160 => {
                self.show_about_modal = true;
                self.about_tab = 0;
            },
            161 | 162 => self.show_shortcuts_modal = true,
            163 => self.trigger_update_check(true),

            // 10. Extended Menu Actions (170..=194)
            170 => {
                if let Some(prev_cat) = self.recent_catalogs.first().cloned() {
                    if prev_cat.exists() {
                        self.open_or_create_catalog_at(&prev_cat, false);
                    }
                } else {
                    self.status_message = format!(
                        "Active Catalog: {}",
                        self.catalog_db.db_path.display()
                    );
                }
            }
            171 => self.import_from_external_catalog_dialog(),
            172 => self.import_xmp_presets_dialog(),
            174 => self.export_selection_as_catalog_dialog(),
            175 => {
                self.export_resize_enabled = true;
                self.export_long_edge_px = 1600;
                self.export_format = "JPEG".into();
                self.execute_batch_export();
                let out_root =
                    std::path::PathBuf::from(&self.export_folder).join(self.export_subfolder.trim());
                let _ = open::that(&out_root);
            }
            176 => self.show_system_info_modal = true,
            177 => {
                self.target_collection_id = 1;
                self.status_message = "Set Quick Collection as Target Collection (B).".into();
            }
            178 => {
                if !self.photos.is_empty() {
                    let folder = self.active_photo().folder_path.clone();
                    self.selected_folder = Some(folder.clone());
                    self.selected_collection_id = None;
                    self.active_module = WorkspaceModule::Library;
                    self.library_view_mode = LibraryViewMode::Grid;
                    self.status_message = format!("Navigated to folder: {}", folder);
                }
            }
            179 => self.edit_active_photo_in_external_app(),
            180 => self.merge_selected_hdr_or_pano(true),
            181 => self.merge_selected_hdr_or_pano(false),
            182 => self.read_xmp_sidecars_for_selection(),
            183 => {
                let ids = self.effective_target_ids();
                for id in &ids {
                    self.invalidate_photo_render_caches(*id);
                }
                self.status_message =
                    format!("Updated previews for {} photo(s).", ids.len());
            }
            185 => self.paste_from_previous_photo(),
            186 => {
                let ids = self.effective_target_ids();
                for id in ids {
                    if let Some(p) = self.photos.iter_mut().find(|x| x.id == id) {
                        p.params.process_version = "Version 6 (Current)".into();
                        self.catalog_db.save_photo(p);
                    }
                }
                self.status_message = "Updated to Process Version 6 (Current).".into();
            }
            187 => {
                self.active_module = WorkspaceModule::Develop;
                self.dev_hsl_open = true;
                self.active_hsl_tab = 4;
                self.point_color_eyedropper_active = true;
                self.status_message =
                    "Point Color Eyedropper active — click a color in the photo.".into();
            }
            188 => {
                self.active_module = WorkspaceModule::Develop;
                self.before_after_mode = BeforeAfterMode::ReferenceView;
                if self.reference_photo_id.is_none() && !self.photos.is_empty() {
                    self.reference_photo_id = Some(self.active_photo_id);
                }
            }
            189 => {
                self.library_filter_mode = LibraryFilterMode::Attribute;
                self.filter_only_picks = true;
            }
            190 => {
                self.library_filter_mode = LibraryFilterMode::Attribute;
                self.filter_min_rating = self.filter_min_rating.max(1);
            }
            191 => {
                self.library_filter_mode = LibraryFilterMode::None;
                self.filter_text.clear();
                self.filter_min_rating = 0;
                self.filter_only_picks = false;
                self.filter_unflagged_only = false;
                self.filter_color_label = None;
                self.filter_camera = None;
            }
            192 | 193 => {
                self.open_native_folder_picker();
                if !self.import_candidates.is_empty() {
                    self.execute_current_import();
                }
            }
            194 => self.show_manage_presets_modal = true,
            195 => self.sync_crop_to_library(false),
            196 => self.auto_match_color_to_reference(),
            _ => {}
        }
    }

    pub fn handle_global_shortcuts(&mut self, ctx: &egui::Context) {
        // Track Alt key while dragging Detail sliders for real-time Sharpening Mask preview!
        let alt_down = ctx.input(|i| i.modifiers.alt && i.pointer.primary_down());
        if self.sharpen_mask_preview_active != alt_down && self.dev_detail_open {
            self.sharpen_mask_preview_active = alt_down;
        } else if !alt_down {
            self.sharpen_mask_preview_active = false;
        }

        if ctx.wants_keyboard_input() {
            return;
        }

        // Escape exits fullscreen slideshow, active eyedropper, or active tool
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.slideshow_fullscreen || self.slideshow_playing {
                self.slideshow_fullscreen = false;
                self.slideshow_playing = false;
                return;
            }
            if self.wb_eyedropper_active
                || self.point_color_eyedropper_active
                || self.defringe_eyedropper_active.is_some()
                || self.lens_blur_focus_picker_active
                || self.lens_blur_brush_mode.is_some()
            {
                self.wb_eyedropper_active = false;
                self.point_color_eyedropper_active = false;
                self.defringe_eyedropper_active = None;
                self.lens_blur_focus_picker_active = false;
                self.lens_blur_brush_mode = None;
                return;
            }
            if self.painter_active {
                self.painter_active = false;
                return;
            }
            if self.develop_tool != DevelopTool::None {
                self.develop_tool = DevelopTool::None;
                return;
            }
        }

        let mods = ctx.input(|i| i.modifiers);
        let cmd = mods.command || mods.ctrl;

        // Arrow Left / Right photo navigation (when not dragging crop)
        if !cmd && !mods.alt {
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                self.step_active_photo(-1);
            } else if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                self.step_active_photo(1);
            }
        }

        // Lightroom Classic Signature Single-Key Workflow Shortcuts
        if !cmd && !mods.alt && !mods.shift {
            if ctx.input(|i| i.key_pressed(egui::Key::D)) {
                self.active_module = WorkspaceModule::Develop;
            } else if ctx.input(|i| i.key_pressed(egui::Key::G)) {
                self.active_module = WorkspaceModule::Library;
                self.library_view_mode = LibraryViewMode::Grid;
            } else if ctx.input(|i| i.key_pressed(egui::Key::E)) {
                self.active_module = WorkspaceModule::Library;
                self.library_view_mode = LibraryViewMode::Loupe;
            } else if ctx.input(|i| i.key_pressed(egui::Key::C)) {
                self.active_module = WorkspaceModule::Library;
                self.library_view_mode = LibraryViewMode::Compare;
            } else if ctx.input(|i| i.key_pressed(egui::Key::N)) {
                self.active_module = WorkspaceModule::Library;
                self.library_view_mode = LibraryViewMode::Survey;
            } else if ctx.input(|i| i.key_pressed(egui::Key::R)) {
                if self.active_module == WorkspaceModule::Develop {
                    let old_tool = self.develop_tool;
                    self.develop_tool = if self.develop_tool == DevelopTool::Crop {
                        DevelopTool::None
                    } else {
                        DevelopTool::Crop
                    };
                    if old_tool == DevelopTool::Crop && self.develop_tool != DevelopTool::Crop {
                        self.invalidate_photo_render_caches(self.active_photo_id);
                    }
                }
            } else if ctx.input(|i| i.key_pressed(egui::Key::Q)) {
                if self.active_module == WorkspaceModule::Develop {
                    self.develop_tool = if self.develop_tool == DevelopTool::Remove {
                        DevelopTool::None
                    } else {
                        DevelopTool::Remove
                    };
                }
            } else if ctx.input(|i| i.key_pressed(egui::Key::K)) {
                if self.active_module == WorkspaceModule::Develop {
                    self.develop_tool = if self.develop_tool == DevelopTool::Masking {
                        DevelopTool::None
                    } else {
                        DevelopTool::Masking
                    };
                }
            } else if ctx.input(|i| i.key_pressed(egui::Key::W)) {
                if self.active_module == WorkspaceModule::Develop {
                    self.wb_eyedropper_active = !self.wb_eyedropper_active;
                }
            } else if ctx.input(|i| i.key_pressed(egui::Key::Y)) {
                if self.active_module == WorkspaceModule::Develop {
                    self.execute_menu_action(126);
                }
            } else if ctx.input(|i| i.key_pressed(egui::Key::S)) {
                if self.active_module == WorkspaceModule::Develop {
                    self.soft_proofing = !self.soft_proofing;
                }
            } else if ctx.input(|i| i.key_pressed(egui::Key::L)) {
                self.lights_out = self.lights_out.next();
            } else if ctx.input(|i| i.key_pressed(egui::Key::P)) {
                self.execute_menu_action(67);
            } else if ctx.input(|i| i.key_pressed(egui::Key::U)) {
                self.execute_menu_action(68);
            } else if ctx.input(|i| i.key_pressed(egui::Key::X)) {
                if !(self.active_module == WorkspaceModule::Develop && self.develop_tool == DevelopTool::Crop) {
                    self.execute_menu_action(69);
                }
            } else if ctx.input(|i| i.key_pressed(egui::Key::Num0)) {
                self.execute_menu_action(70);
            } else if ctx.input(|i| i.key_pressed(egui::Key::Num1)) {
                self.execute_menu_action(71);
            } else if ctx.input(|i| i.key_pressed(egui::Key::Num2)) {
                self.execute_menu_action(72);
            } else if ctx.input(|i| i.key_pressed(egui::Key::Num3)) {
                self.execute_menu_action(73);
            } else if ctx.input(|i| i.key_pressed(egui::Key::Num4)) {
                self.execute_menu_action(74);
            } else if ctx.input(|i| i.key_pressed(egui::Key::Num5)) {
                self.execute_menu_action(75);
            } else if ctx.input(|i| i.key_pressed(egui::Key::Num6)) {
                self.execute_menu_action(77);
            } else if ctx.input(|i| i.key_pressed(egui::Key::Num7)) {
                self.execute_menu_action(78);
            } else if ctx.input(|i| i.key_pressed(egui::Key::Num8)) {
                self.execute_menu_action(79);
            } else if ctx.input(|i| i.key_pressed(egui::Key::Num9)) {
                self.execute_menu_action(80);
            }
        }

        // Tab hides/shows left & right side panels simultaneously
        if !cmd && ctx.input(|i| i.key_pressed(egui::Key::Tab)) {
            let show_both = !(self.show_left_panel && self.show_right_panel);
            self.show_left_panel = show_both;
            self.show_right_panel = show_both;
        }

        // Backslash `\` toggles Before Only comparison in Develop
        if !cmd && ctx.input(|i| i.key_pressed(egui::Key::Backslash)) {
            if self.active_module == WorkspaceModule::Develop {
                self.before_after_mode = if self.before_after_mode == BeforeAfterMode::BeforeOnly {
                    BeforeAfterMode::Single
                } else {
                    BeforeAfterMode::BeforeOnly
                };
            }
        }

        // `O` toggles Mask Overlay when Masking tool is open
        if !cmd
            && self.active_module == WorkspaceModule::Develop
            && self.develop_tool == DevelopTool::Masking
            && ctx.input(|i| i.key_pressed(egui::Key::O))
        {
            self.show_mask_overlay = !self.show_mask_overlay;
        }

        // Backspace / Delete removes selected Retouch spot, Red-Eye spot, or Mask
        if !cmd
            && (ctx.input(|i| i.key_pressed(egui::Key::Backspace))
                || ctx.input(|i| i.key_pressed(egui::Key::Delete)))
        {
            self.delete_active_tool_item_or_photo();
        }

        // Lightroom Classic Standard Cmd & Cmd+Shift Shortcuts
        if cmd {
            if mods.shift {
                if ctx.input(|i| i.key_pressed(egui::Key::C)) {
                    self.execute_menu_action(91); // Copy Settings Modal
                } else if ctx.input(|i| i.key_pressed(egui::Key::V)) {
                    self.execute_menu_action(92); // Paste Settings
                } else if ctx.input(|i| i.key_pressed(egui::Key::S)) {
                    self.execute_menu_action(93); // Sync Settings Modal
                } else if ctx.input(|i| i.key_pressed(egui::Key::I)) {
                    self.execute_menu_action(14); // Import Modal
                } else if ctx.input(|i| i.key_pressed(egui::Key::E)) {
                    self.execute_menu_action(15); // Export Modal
                } else if ctx.input(|i| i.key_pressed(egui::Key::N)) {
                    self.execute_menu_action(51); // New Preset Modal
                } else if ctx.input(|i| i.key_pressed(egui::Key::U)) {
                    self.execute_menu_action(95); // Auto White Balance
                } else if ctx.input(|i| i.key_pressed(egui::Key::Z)) {
                    self.execute_menu_action(31); // Redo
                }
            } else {
                if ctx.input(|i| i.key_pressed(egui::Key::Z)) {
                    self.execute_menu_action(30); // Undo
                } else if ctx.input(|i| i.key_pressed(egui::Key::A)) {
                    self.execute_menu_action(36); // Select All
                } else if ctx.input(|i| i.key_pressed(egui::Key::D)) {
                    self.execute_menu_action(37); // Deselect All
                } else if ctx.input(|i| i.key_pressed(egui::Key::U)) {
                    self.execute_menu_action(94); // Auto Settings
                } else if ctx.input(|i| i.key_pressed(egui::Key::N)) {
                    self.execute_menu_action(50); // New Snapshot
                }
            }
        }

        // `B` adds/removes from Target Collection
        if !cmd && !mods.shift && ctx.input(|i| i.key_pressed(egui::Key::B)) {
            self.toggle_target_collection_for_selection();
        }
    }

    pub fn delete_active_tool_item_or_photo(&mut self) {
        if self.active_module == WorkspaceModule::Develop && !self.photos.is_empty() {
            if self.develop_tool == DevelopTool::Remove {
                if let Some(idx) = self.selected_retouch_idx {
                    let mut p = self.active_photo().params.clone();
                    if idx < p.retouch_areas.len() {
                        p.retouch_areas.remove(idx);
                        self.selected_retouch_idx = None;
                        self.commit_params_change("Delete Heal/Clone Spot", p);
                        return;
                    }
                }
            } else if self.develop_tool == DevelopTool::RedEye {
                if let Some(idx) = self.selected_redeye_idx {
                    let mut p = self.active_photo().params.clone();
                    if idx < p.red_eye_spots.len() {
                        p.red_eye_spots.remove(idx);
                        self.selected_redeye_idx = None;
                        self.commit_params_change("Delete Red-Eye Correction", p);
                        return;
                    }
                }
            } else if self.develop_tool == DevelopTool::Masking {
                if let Some(idx) = self.selected_mask_idx {
                    let mut p = self.active_photo().params.clone();
                    if idx < p.mask_groups.len() {
                        p.mask_groups.remove(idx);
                        self.selected_mask_idx = if p.mask_groups.is_empty() {
                            None
                        } else {
                            Some(idx.saturating_sub(1))
                        };
                        self.commit_params_change("Delete Mask Group", p);
                        return;
                    }
                }
            }
        }
        self.remove_selected_photos_from_catalog();
    }

    pub fn edit_active_photo_in_external_app(&mut self) {
        if self.photos.is_empty() {
            return;
        }
        let photo = self.active_photo().clone();
        let raw = self.decode_photo_for_output(&photo, 0);
        let frame = crate::pipeline::render_develop_pipeline(
            &raw,
            &photo.params,
            crate::pipeline::RenderOptions::default(),
        );
        let out_dir = Path::new(&self.export_folder).join("External_Edits");
        let _ = std::fs::create_dir_all(&out_dir);
        let stem = Path::new(&photo.file_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Edit");
        let tif_path = out_dir.join(format!("{}_Edit.tif", stem));
        if let Ok(f) = std::fs::File::create(&tif_path) {
            let enc = image::codecs::tiff::TiffEncoder::new(std::io::BufWriter::new(f));
            use image::ImageEncoder;
            if enc
                .write_image(
                    &frame.rgba,
                    frame.width,
                    frame.height,
                    image::ExtendedColorType::Rgba8,
                )
                .is_ok()
            {
                // Automatically import the TIFF derivative into the catalog alongside the RAW!
                self.import_single_file_path(&tif_path, None);
                let _ = open::that(&tif_path);
                self.status_message =
                    format!("Created & opened TIFF derivative: {}", tif_path.display());
            }
        }
    }

    pub fn render_system_menu_bar(&mut self, ctx: &egui::Context) {
        if !self.show_in_app_menubar || self.slideshow_fullscreen {
            return;
        }

        let mod_k = if cfg!(target_os = "macos") { "⌘" } else { "Ctrl+" };
        let alt_k = if cfg!(target_os = "macos") { "⌥" } else { "Alt+" };
        let shift_k = if cfg!(target_os = "macos") { "⇧" } else { "Shift+" };

        egui::TopBottomPanel::top("lr_in_app_system_menu_bar")
            .exact_height(24.0)
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(26, 26, 30))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(38, 38, 44)))
                    .inner_margin(egui::Margin::symmetric(8.0, 2.0)),
            )
            .show(ctx, |ui| {
                ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
                ui.style_mut().visuals.widgets.hovered.bg_fill = Color32::from_rgb(45, 45, 52);
                ui.style_mut().visuals.widgets.active.bg_fill = Color32::from_rgb(55, 55, 65);
                ui.style_mut().visuals.widgets.inactive.fg_stroke.color = Colors::TEXT_MAIN;
                ui.style_mut().visuals.widgets.hovered.fg_stroke.color = Colors::TEXT_BRIGHT;
                ui.style_mut().visuals.widgets.active.fg_stroke.color = Color32::WHITE;

                egui::menu::bar(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(6.0, 0.0);

                    // 1. FILE
                    ui.menu_button("File", |ui| {
                        ui.set_min_width(240.0);
                        if ui.add(egui::Button::new("New Catalog...").shortcut_text(format!("{}N", mod_k))).clicked() {
                            self.execute_menu_action(10);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Open Catalog...").shortcut_text(format!("{}O", mod_k))).clicked() {
                            self.execute_menu_action(11);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Open Recent Catalog")).clicked() {
                            self.execute_menu_action(170);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Optimize Catalog...")).clicked() {
                            self.execute_menu_action(12);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Rename Catalog...")).clicked() {
                            self.execute_menu_action(13);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.add(egui::Button::new("Import Photos and Videos...").shortcut_text(format!("{}{}I", mod_k, shift_k))).clicked() {
                            self.execute_menu_action(14);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Import from Another Catalog...")).clicked() {
                            self.execute_menu_action(171);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Import Presets (.xmp)...")).clicked() {
                            self.execute_menu_action(172);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.add(egui::Button::new("Export...").shortcut_text(format!("{}{}E", mod_k, shift_k))).clicked() {
                            self.execute_menu_action(15);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Export with Previous").shortcut_text(format!("{}{}{}E", mod_k, alt_k, shift_k))).clicked() {
                            self.execute_menu_action(16);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Export as Catalog...")).clicked() {
                            self.execute_menu_action(174);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Email Photos...")).clicked() {
                            self.execute_menu_action(175);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.add(egui::Button::new("Page Setup...").shortcut_text(format!("{}{}P", mod_k, shift_k))).clicked() {
                            self.execute_menu_action(19);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Print...").shortcut_text(format!("{}P", mod_k))).clicked() {
                            self.execute_menu_action(20);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.add(egui::Button::new("Exit").shortcut_text(if cfg!(target_os = "windows") { "Alt+F4" } else { "Ctrl+Q" })).clicked() {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                            ui.close_menu();
                        }
                    });

                    // 2. EDIT
                    ui.menu_button("Edit", |ui| {
                        ui.set_min_width(240.0);
                        if ui.add(egui::Button::new("Undo").shortcut_text(format!("{}Z", mod_k))).clicked() {
                            self.execute_menu_action(30);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Redo").shortcut_text(if cfg!(target_os = "macos") { "⌘⇧Z" } else { "Ctrl+Y" })).clicked() {
                            self.execute_menu_action(31);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.add(egui::Button::new("Cut Develop Settings").shortcut_text(format!("{}X", mod_k))).clicked() {
                            self.execute_menu_action(32);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Copy Develop Settings...").shortcut_text(format!("{}{}C", mod_k, shift_k))).clicked() {
                            self.execute_menu_action(33);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Paste Develop Settings").shortcut_text(format!("{}{}V", mod_k, shift_k))).clicked() {
                            self.execute_menu_action(34);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Delete Selected Photo(s)").shortcut_text("Del")).clicked() {
                            self.execute_menu_action(35);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.add(egui::Button::new("Select All").shortcut_text(format!("{}A", mod_k))).clicked() {
                            self.execute_menu_action(36);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Select None").shortcut_text(format!("{}D", mod_k))).clicked() {
                            self.execute_menu_action(37);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Invert Selection").shortcut_text(format!("{}{}I", mod_k, shift_k))).clicked() {
                            self.execute_menu_action(38);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Select Flagged Photos")).clicked() {
                            self.execute_menu_action(39);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Deselect Unflagged Photos")).clicked() {
                            self.execute_menu_action(40);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.add(egui::Button::new("Edit Watermarks...")).clicked() {
                            self.execute_menu_action(43);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Preferences...").shortcut_text(format!("{},", mod_k))).clicked() {
                            self.execute_menu_action(3);
                            ui.close_menu();
                        }
                    });

                    // 3. DEVELOP
                    ui.menu_button("Develop", |ui| {
                        ui.set_min_width(250.0);
                        if ui.add(egui::Button::new("New Snapshot...").shortcut_text(format!("{}N", mod_k))).clicked() {
                            self.execute_menu_action(50);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("New Preset...").shortcut_text(format!("{}{}N", mod_k, shift_k))).clicked() {
                            self.execute_menu_action(51);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Clear History")).clicked() {
                            self.execute_menu_action(52);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.add(egui::Button::new("Crop & Straighten").shortcut_text("R")).clicked() {
                            self.execute_menu_action(110);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Spot Removal & Generative AI").shortcut_text("Q")).clicked() {
                            self.execute_menu_action(112);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Red Eye Correction")).clicked() {
                            self.execute_menu_action(111);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Masking & AI Segmentation").shortcut_text("K")).clicked() {
                            self.execute_menu_action(113);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("White Balance Selector").shortcut_text("W")).clicked() {
                            self.execute_menu_action(114);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Targeted Adjustment Tool")).clicked() {
                            self.execute_menu_action(115);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Guided Upright Perspective")).clicked() {
                            self.execute_menu_action(116);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.add(egui::Button::new("Auto Tone").shortcut_text(format!("{}U", mod_k))).clicked() {
                            self.execute_menu_action(94);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Auto White Balance").shortcut_text(format!("{}{}U", mod_k, shift_k))).clicked() {
                            self.execute_menu_action(95);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("⚡ Match Color to Reference...").shortcut_text(format!("{}{}{}M", mod_k, alt_k, shift_k))).clicked() {
                            self.execute_menu_action(196);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Convert to Black & White").shortcut_text("V")).clicked() {
                            self.execute_menu_action(96);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Toggle HDR Edit Mode")).clicked() {
                            self.execute_menu_action(97);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Reset Crop")).clicked() {
                            self.execute_menu_action(98);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Swap Before & After").shortcut_text(format!("{}Y", alt_k))).clicked() {
                            self.execute_menu_action(100);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.add(egui::Button::new("Copy Settings...").shortcut_text(format!("{}{}C", mod_k, shift_k))).clicked() {
                            self.execute_menu_action(91);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Paste Settings").shortcut_text(format!("{}{}V", mod_k, shift_k))).clicked() {
                            self.execute_menu_action(92);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Sync Settings...").shortcut_text(format!("{}{}S", mod_k, shift_k))).clicked() {
                            self.execute_menu_action(93);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Reset All Settings").shortcut_text(format!("{}{}R", mod_k, shift_k))).clicked() {
                            self.execute_menu_action(90);
                            ui.close_menu();
                        }
                    });

                    // 4. PHOTO
                    ui.menu_button("Photo", |ui| {
                        ui.set_min_width(250.0);
                        if ui.add(egui::Button::new("Add to Quick Collection").shortcut_text("B")).clicked() {
                            self.execute_menu_action(60);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Set Quick Collection as Target")).clicked() {
                            self.execute_menu_action(177);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Show in File Explorer / Finder").shortcut_text(format!("{}R", mod_k))).clicked() {
                            self.execute_menu_action(61);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Go to Folder in Library")).clicked() {
                            self.execute_menu_action(178);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Create Virtual Copy").shortcut_text(format!("{}'", mod_k))).clicked() {
                            self.execute_menu_action(62);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Edit in External Application")).clicked() {
                            self.execute_menu_action(179);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.add(egui::Button::new("Rotate Left (CCW)").shortcut_text(format!("{}[", mod_k))).clicked() {
                            self.execute_menu_action(63);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Rotate Right (CW)").shortcut_text(format!("{}]", mod_k))).clicked() {
                            self.execute_menu_action(64);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Flip Horizontal")).clicked() {
                            self.execute_menu_action(65);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Flip Vertical")).clicked() {
                            self.execute_menu_action(66);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.add(egui::Button::new("Set Flag: Pick").shortcut_text("P")).clicked() {
                            self.execute_menu_action(67);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Set Flag: Unflag").shortcut_text("U")).clicked() {
                            self.execute_menu_action(68);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Set Flag: Reject").shortcut_text("X")).clicked() {
                            self.execute_menu_action(69);
                            ui.close_menu();
                        }
                        ui.separator();
                        ui.menu_button("Set Rating", |ui| {
                            for r in 0..=5 {
                                let label = if r == 0 { "None (0)".to_string() } else { format!("{} Star{}", r, if r > 1 { "s" } else { "" }) };
                                if ui.add(egui::Button::new(label).shortcut_text(format!("{}", r))).clicked() {
                                    self.execute_menu_action(70 + r as u32);
                                    ui.close_menu();
                                }
                            }
                        });
                        ui.menu_button("Set Color Label", |ui| {
                            let labels = [("None", 76), ("Red", 77), ("Yellow", 78), ("Green", 79), ("Blue", 80), ("Purple", 81)];
                            for (name, id) in labels {
                                if ui.button(name).clicked() {
                                    self.execute_menu_action(id);
                                    ui.close_menu();
                                }
                            }
                        });
                        ui.separator();
                        if ui.add(egui::Button::new("AI Enhance & Denoise")).clicked() {
                            self.execute_menu_action(85);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Photo Merge: HDR").shortcut_text(format!("{}H", mod_k))).clicked() {
                            self.execute_menu_action(180);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Photo Merge: Panorama").shortcut_text(format!("{}M", mod_k))).clicked() {
                            self.execute_menu_action(181);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Save Metadata to File").shortcut_text(format!("{}S", mod_k))).clicked() {
                            self.execute_menu_action(83);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Read Metadata from File")).clicked() {
                            self.execute_menu_action(182);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Update Preview Caches")).clicked() {
                            self.execute_menu_action(183);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.add(egui::Button::new("Remove from Catalog").shortcut_text("Del")).clicked() {
                            self.execute_menu_action(82);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Delete Rejected Photos...").shortcut_text(format!("{}Backspace", mod_k))).clicked() {
                            self.execute_menu_action(84);
                            ui.close_menu();
                        }
                    });

                    // 5. VIEW
                    ui.menu_button("View", |ui| {
                        ui.set_min_width(240.0);
                        if ui.add(egui::Button::new("Grid").shortcut_text("G")).clicked() {
                            self.execute_menu_action(129);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Loupe").shortcut_text("E")).clicked() {
                            self.execute_menu_action(130);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Compare").shortcut_text("C")).clicked() {
                            self.execute_menu_action(131);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Survey").shortcut_text("N")).clicked() {
                            self.execute_menu_action(132);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("People (Face Detection)").shortcut_text("O")).clicked() {
                            self.execute_menu_action(133);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.add(egui::Button::new("Toggle Zoom").shortcut_text("Z")).clicked() {
                            self.execute_menu_action(122);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Zoom In").shortcut_text(format!("{}+", mod_k))).clicked() {
                            self.execute_menu_action(123);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Zoom Out").shortcut_text(format!("{}-", mod_k))).clicked() {
                            self.execute_menu_action(124);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Zoom 1:1 (100%)")).clicked() {
                            self.execute_menu_action(125);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.add(egui::Button::new("Before / After View").shortcut_text("Y")).clicked() {
                            self.execute_menu_action(126);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Soft Proofing").shortcut_text("S")).clicked() {
                            self.execute_menu_action(127);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Show Clipping Warnings").shortcut_text("J")).clicked() {
                            self.execute_menu_action(128);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Toggle Toolbar").shortcut_text("T")).clicked() {
                            self.execute_menu_action(120);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Cycle Info Overlay").shortcut_text("I")).clicked() {
                            self.execute_menu_action(121);
                            ui.close_menu();
                        }
                    });

                    // 6. WINDOW
                    ui.menu_button("Window", |ui| {
                        ui.set_min_width(240.0);
                        if ui.add(egui::Button::new("Library Module").shortcut_text(format!("{}{}1", mod_k, alt_k))).clicked() {
                            self.execute_menu_action(150);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Develop Module").shortcut_text(format!("{}{}2", mod_k, alt_k))).clicked() {
                            self.execute_menu_action(151);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Map Module").shortcut_text(format!("{}{}3", mod_k, alt_k))).clicked() {
                            self.execute_menu_action(152);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Book Module").shortcut_text(format!("{}{}4", mod_k, alt_k))).clicked() {
                            self.execute_menu_action(153);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Slideshow Module").shortcut_text(format!("{}{}5", mod_k, alt_k))).clicked() {
                            self.execute_menu_action(154);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Print Module").shortcut_text(format!("{}{}6", mod_k, alt_k))).clicked() {
                            self.execute_menu_action(155);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Web Module").shortcut_text(format!("{}{}7", mod_k, alt_k))).clicked() {
                            self.execute_menu_action(156);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.add(egui::Button::new("Toggle Left Panel").shortcut_text("Tab")).clicked() {
                            self.execute_menu_action(142);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Toggle Right Panel").shortcut_text("Shift+Tab")).clicked() {
                            self.execute_menu_action(143);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Toggle Filmstrip").shortcut_text("F6")).clicked() {
                            self.execute_menu_action(144);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Lights Out Mode").shortcut_text("L")).clicked() {
                            self.execute_menu_action(141);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Impromptu Slideshow").shortcut_text(format!("{}Enter", mod_k))).clicked() {
                            self.execute_menu_action(140);
                            ui.close_menu();
                        }
                    });

                    // 7. HELP
                    ui.menu_button("Help", |ui| {
                        ui.set_min_width(220.0);
                        if ui.add(egui::Button::new("LightRaw Documentation").shortcut_text("F1")).clicked() {
                            self.execute_menu_action(161);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("Keyboard Shortcuts Guide").shortcut_text(format!("{}/", mod_k))).clicked() {
                            self.execute_menu_action(162);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("System Info & Hardware Diagnostic")).clicked() {
                            self.execute_menu_action(2);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.add(egui::Button::new("Check for Updates...")).clicked() {
                            self.execute_menu_action(163);
                            ui.close_menu();
                        }
                        if ui.add(egui::Button::new("About LightRaw Classic...").shortcut_text(if cfg!(target_os = "macos") { "⌘I" } else { "" })).clicked() {
                            self.execute_menu_action(1);
                            ui.close_menu();
                        }
                    });
                });
            });
    }
}
