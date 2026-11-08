// ============================================================================
// LightRaw — Complete Adobe Lightroom Classic Library Module (Virtualized)
// ============================================================================

use super::{
    draw_histogram_widget, lr_bevel_button, lr_left_panel_header, lr_right_panel_header,
    lr_selectable_row, paint_thumbnail_corner_badges, render_navigator_panel, Colors,
    LibraryFilterMode, LibraryLeftTab, LibrarySortMode, LibraryViewMode, LightRawApp,
    MetadataViewPreset, PainterMode,
};
use crate::catalog::{ColorLabel, HistoryStep, PhotoComment, PickFlag};
use eframe::egui::{self, Color32, Pos2, Rect, RichText, Sense, Stroke, Ui, Vec2};
use std::path::Path;

pub fn render_library_module(app: &mut LightRawApp, ctx: &egui::Context) {
    if app.show_left_panel {
        egui::SidePanel::left("library_left_panel")
            .resizable(true)
            .default_width(256.0)
            .min_width(220.0)
            .max_width(360.0)
            .frame(egui::Frame::none().fill(Colors::BG_PANEL))
            .show(ctx, |ui| {
                render_library_left_panel(app, ctx, ui);
            });
    }

    if app.show_right_panel {
        egui::SidePanel::right("library_right_panel")
            .resizable(true)
            .default_width(288.0)
            .min_width(255.0)
            .max_width(390.0)
            .frame(egui::Frame::none().fill(Colors::BG_PANEL))
            .show(ctx, |ui| {
                render_library_right_panel(app, ctx, ui);
            });
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(app.canvas_bg_color()))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            if app.library_view_mode == LibraryViewMode::Grid {
                render_library_filter_bar(app, ui);
            }

            let tb_h = if app.show_toolbar { 32.0 } else { 0.0 };
            let main_h = (ui.available_height() - tb_h).max(100.0);
            ui.allocate_ui(Vec2::new(ui.available_width(), main_h), |ui| {
                match app.library_view_mode {
                    LibraryViewMode::Grid => render_grid_view(app, ctx, ui),
                    LibraryViewMode::Loupe => render_loupe_view(app, ctx, ui),
                    LibraryViewMode::Compare => render_compare_view(app, ctx, ui),
                    LibraryViewMode::Survey => render_survey_view(app, ctx, ui),
                    LibraryViewMode::People => render_people_view(app, ctx, ui),
                }
            });

            if app.show_toolbar {
                render_library_toolbar(app, ui);
            }
        });
}

// ============================================================================
// 1. LIBRARY LEFT PANEL: Navigator, Folders/Culling Tabs, Disk Space & Collections
// ============================================================================
fn render_library_left_panel(app: &mut LightRawApp, ctx: &egui::Context, ui: &mut Ui) {
    let total_h = ui.available_height();
    let bottom_bar_h = 38.0;

    ui.allocate_ui(
        Vec2::new(ui.available_width(), (total_h - bottom_bar_h).max(80.0)),
        |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            let mut nav_open = app.lib_nav_open;
            render_navigator_panel(app, ctx, ui, &mut nav_open);
            app.lib_nav_open = nav_open;

            // Lightroom Classic 14.5+ `Folders | Assisted Culling` Tab Switcher
            egui::Frame::none()
                .fill(Color32::from_rgb(34, 34, 38))
                .inner_margin(egui::Margin::symmetric(8.0, 5.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let hw = (ui.available_width() - 6.0) * 0.5;
                        for (tab, lbl) in [
                            (LibraryLeftTab::Folders, "Folders"),
                            (LibraryLeftTab::AssistedCulling, "Assisted Culling"),
                        ] {
                            let active = app.lib_left_tab == tab;
                            if ui
                                .add_sized(
                                    [hw, 22.0],
                                    egui::SelectableLabel::new(
                                        active,
                                        RichText::new(lbl).size(11.5).strong(),
                                    ),
                                )
                                .clicked()
                            {
                                app.lib_left_tab = tab;
                            }
                        }
                    });
                });

            egui::ScrollArea::vertical()
                .id_salt("lib_left_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    match app.lib_left_tab {
                        LibraryLeftTab::Folders => render_folders_and_collections_tree(app, ui),
                        LibraryLeftTab::AssistedCulling => render_assisted_culling_tab(app, ui),
                    }
                });
        },
    );

    // Bottom Pinned `[ Import... ]` | `[ Export... ]`
    egui::Frame::none()
        .fill(Color32::from_rgb(32, 32, 36))
        .inner_margin(egui::Margin::symmetric(8.0, 6.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let bw = (ui.available_width() - 8.0) * 0.5;
                if lr_bevel_button(ui, "Import...", bw, 24.0).clicked() {
                    app.show_import_modal = true;
                }
                ui.add_space(6.0);
                if lr_bevel_button(ui, "Export...", bw, 24.0).clicked() {
                    app.show_export_modal = true;
                }
            });
        });
}

fn render_folders_and_collections_tree(app: &mut LightRawApp, ui: &mut Ui) {
    // 1. Catalog Section
    let _ = lr_left_panel_header(ui, "Catalog", &mut app.lib_catalog_open, &[]);
    if app.lib_catalog_open {
        egui::Frame::none()
            .fill(Color32::from_rgb(42, 42, 46))
            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
            .show(ui, |ui| {
                let total_str = format!("{}", app.photos.len());
                let all_sel = app.selected_folder.is_none() && app.selected_collection_id.is_none();
                if lr_selectable_row(ui, all_sel, "All Photographs", Some(&total_str)).clicked() {
                    app.selected_folder = None;
                    app.selected_collection_id = None;
                }

                let qc_count = app
                    .collections
                    .iter()
                    .find(|c| c.id == 1)
                    .map(|c| c.photo_ids.len())
                    .unwrap_or(0);
                let qc_sel = app.selected_collection_id == Some(1);
                let qc_lbl = if app.target_collection_id == 1 {
                    "Quick Collection +"
                } else {
                    "Quick Collection"
                };
                if lr_selectable_row(ui, qc_sel, qc_lbl, Some(&format!("{}", qc_count))).clicked() {
                    app.selected_collection_id = Some(1);
                    app.selected_folder = None;
                }
            });
    }

    // 2. Folders Section (with Real Volume Free Space Indicator!)
    let folder_btn = lr_left_panel_header(ui, "Folders", &mut app.lib_folders_open, &["+", "−"]);
    if folder_btn == Some(0) {
        app.show_import_modal = true;
    } else if folder_btn == Some(1) {
        if let Some(sel_f) = app.selected_folder.clone() {
            app.remove_folder_from_catalog(&sel_f);
        }
    }
    if app.lib_folders_open {
        egui::Frame::none()
            .fill(Color32::from_rgb(42, 42, 46))
            .inner_margin(egui::Margin::symmetric(8.0, 6.0))
            .show(ui, |ui| {
                let (vol_name, free_b, total_b) = app.cached_disk_space();
                let used_frac = if total_b > 0 {
                    1.0 - (free_b as f32 / total_b as f32)
                } else {
                    0.35
                }
                .clamp(0.02, 0.98);

                // Volume Header Bar with Real Free GB Meter
                let (v_rect, _) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), 22.0), Sense::hover());
                ui.painter()
                    .rect_filled(v_rect, 2.5, Color32::from_rgb(32, 32, 36));
                let bar_fill = Rect::from_min_max(
                    Pos2::new(v_rect.max.x - 54.0, v_rect.center().y - 3.5),
                    Pos2::new(
                        v_rect.max.x - 54.0 + 46.0 * (1.0 - used_frac),
                        v_rect.center().y + 3.5,
                    ),
                );
                let bar_bg = Rect::from_min_max(
                    Pos2::new(v_rect.max.x - 54.0, v_rect.center().y - 3.5),
                    Pos2::new(v_rect.max.x - 8.0, v_rect.center().y + 3.5),
                );
                ui.painter()
                    .rect_filled(bar_bg, 2.0, Color32::from_rgb(20, 20, 24));
                ui.painter().rect_filled(
                    bar_fill,
                    2.0,
                    if used_frac > 0.9 {
                        Color32::from_rgb(225, 75, 75)
                    } else {
                        Color32::from_rgb(65, 190, 95)
                    },
                );
                ui.painter().text(
                    Pos2::new(v_rect.min.x + 6.0, v_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    format!(
                        "▾ {} ({:.0} GB free)",
                        vol_name,
                        free_b as f64 / 1_073_741_824.0
                    ),
                    egui::FontId::proportional(10.5),
                    Colors::TEXT_BRIGHT,
                );
                ui.add_space(3.0);

                let folders = app.unique_folders();
                if folders.is_empty() {
                    ui.label(
                        RichText::new("No folders imported yet")
                            .size(10.5)
                            .color(Colors::TEXT_DIM),
                    );
                }
                for (fpath, count) in folders {
                    let short = Path::new(&fpath)
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or(&fpath);
                    let is_sel = app.selected_folder.as_deref() == Some(fpath.as_str());
                    let r = lr_selectable_row(
                        ui,
                        is_sel,
                        &format!("  📁 {}", short),
                        Some(&format!("{}", count)),
                    )
                    .on_hover_text(&fpath);
                    if r.clicked() {
                        app.selected_folder = if is_sel { None } else { Some(fpath.clone()) };
                        app.selected_collection_id = None;
                    }
                    r.context_menu(|ui| {
                        if ui.button("Synchronize Folder...").clicked() {
                            app.synchronize_folder(&fpath);
                            ui.close_menu();
                        }
                        if ui.button("Import into this Folder...").clicked() {
                            app.scan_import_source(Path::new(&fpath));
                            app.show_import_modal = true;
                            ui.close_menu();
                        }
                        if ui.button("Show in Finder").clicked() {
                            let _ = open::that(&fpath);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Remove Folder from Catalog").clicked() {
                            app.remove_folder_from_catalog(&fpath);
                            ui.close_menu();
                        }
                    });
                }
            });
    }

    // 3. Collections Section
    let col_btn =
        lr_left_panel_header(ui, "Collections", &mut app.lib_collections_open, &["+", "−"]);
    if col_btn == Some(0) {
        app.create_collection_is_smart = false;
        app.new_collection_name = "New Collection".into();
        app.show_create_collection_modal = true;
    } else if col_btn == Some(1) {
        if let Some(cid) = app.selected_collection_id {
            if cid > 4 {
                app.catalog_db.delete_collection(cid);
                app.collections.retain(|c| c.id != cid);
                app.selected_collection_id = None;
            }
        }
    }
    if app.lib_collections_open {
        egui::Frame::none()
            .fill(Color32::from_rgb(42, 42, 46))
            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.small_button("+ Collection").clicked() {
                        app.create_collection_is_smart = false;
                        app.new_collection_name = "New Collection".into();
                        app.show_create_collection_modal = true;
                    }
                    if ui.small_button("+ Smart").clicked() {
                        app.create_collection_is_smart = true;
                        app.new_collection_name = "Smart Selects".into();
                        app.show_create_collection_modal = true;
                    }
                });

                let cols = app.collections.clone();
                for col in cols {
                    let count = app
                        .photos
                        .iter()
                        .filter(|p| app.smart_collection_matches(&col, p))
                        .count();
                    let is_sel = app.selected_collection_id == Some(col.id);
                    let icon = if col.is_smart { "⚙" } else { "▤" };
                    let suffix = if !col.is_smart && app.target_collection_id == col.id {
                        " +"
                    } else {
                        ""
                    };
                    let r = lr_selectable_row(
                        ui,
                        is_sel,
                        &format!("{} {}{}", icon, col.name, suffix),
                        Some(&format!("{}", count)),
                    );
                    if r.clicked() {
                        app.selected_collection_id = if is_sel { None } else { Some(col.id) };
                        app.selected_folder = None;
                    }
                    r.context_menu(|ui| {
                        if !col.is_smart {
                            if ui.button("Set as Target Collection (B)").clicked() {
                                app.target_collection_id = col.id;
                                ui.close_menu();
                            }
                            if ui.button("Add Selected Photos").clicked() {
                                let sel = app.selected_photo_ids.clone();
                                if let Some(target) =
                                    app.collections.iter_mut().find(|c| c.id == col.id)
                                {
                                    for id in sel {
                                        if !target.photo_ids.contains(&id) {
                                            target.photo_ids.push(id);
                                        }
                                    }
                                    let cloned = target.clone();
                                    app.catalog_db.save_collection(&cloned);
                                }
                                ui.close_menu();
                            }
                        }
                        if ui.button("Rename...").clicked() {
                            app.rename_collection_input = col.name.clone();
                            app.show_rename_collection_modal = Some(col.id);
                            ui.close_menu();
                        }
                        if col.id > 1 && ui.button("Delete").clicked() {
                            app.catalog_db.delete_collection(col.id);
                            app.collections.retain(|c| c.id != col.id);
                            if app.selected_collection_id == Some(col.id) {
                                app.selected_collection_id = None;
                            }
                            ui.close_menu();
                        }
                    });
                }
            });
    }

    // 4. Local Publish Service
    let _ = lr_left_panel_header(ui, "Publish Services", &mut app.lib_publish_open, &[]);
    if app.lib_publish_open {
        egui::Frame::none()
            .fill(Color32::from_rgb(42, 42, 46))
            .inner_margin(egui::Margin::symmetric(8.0, 6.0))
            .show(ui, |ui| {
                if lr_selectable_row(ui, false, "💾 Hard Drive — Quick Export", None).clicked() {
                    app.show_export_modal = true;
                }
            });
    }
}

fn render_assisted_culling_tab(app: &mut LightRawApp, ui: &mut Ui) {
    egui::Frame::none()
        .fill(Color32::from_rgb(42, 42, 46))
        .inner_margin(egui::Margin::symmetric(10.0, 10.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new("Apple Vision AI Assisted Culling")
                    .strong()
                    .size(12.0)
                    .color(Colors::TEXT_BRIGHT),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new(
                    "Detects faces, eye aspect ratio (blinks), head pose, and Laplacian sharpness locally.",
                )
                .size(10.5)
                .color(Colors::TEXT_MID),
            );
            ui.add_space(8.0);

            if lr_bevel_button(
                ui,
                "✨ Analyze Visible Photos (Vision AI)",
                ui.available_width(),
                26.0,
            )
            .clicked()
            {
                app.run_assisted_culling_analysis();
            }

            ui.add_space(10.0);
            super::lr_slider(
                ui,
                "Eye Focus",
                &mut app.culling_eye_focus_thresh,
                0.0..=100.0,
                55.0,
                "{:.0}",
            );
            super::lr_slider(
                ui,
                "Eyes Open",
                &mut app.culling_eyes_open_thresh,
                0.0..=100.0,
                50.0,
                "{:.0}",
            );

            ui.add_space(6.0);
            ui.checkbox(
                &mut app.culling_reject_blinks,
                "Auto-Reject Closed Eyes / Blinks (✕)",
            );
            ui.checkbox(
                &mut app.culling_filter_passed_only,
                "Filter Grid to Passed Selects Only",
            );

            ui.separator();
            let analyzed = app
                .photos
                .iter()
                .filter(|p| p.culling_focus_score.unwrap_or(0.0) > 0.0)
                .count();
            let picks = app
                .photos
                .iter()
                .filter(|p| p.flag == PickFlag::Picked)
                .count();
            let rejects = app
                .photos
                .iter()
                .filter(|p| p.flag == PickFlag::Rejected)
                .count();
            ui.label(
                RichText::new(format!(
                    "Analyzed: {}   •   Picks (⚑): {}   •   Rejects (✕): {}",
                    analyzed, picks, rejects
                ))
                .size(10.5)
                .color(Colors::TEXT_MAIN),
            );
        });
}

// ============================================================================
// 2. LIBRARY RIGHT PANEL: Histogram, Culling Info, Quick Develop, Keywords, Metadata
// ============================================================================
fn render_library_right_panel(app: &mut LightRawApp, ctx: &egui::Context, ui: &mut Ui) {
    let full_rect = ui.available_rect_before_wrap();
    let bottom_bar_h = 38.0;
    let split_y = (full_rect.max.y - bottom_bar_h).max(full_rect.min.y);

    let top_rect = Rect::from_min_max(full_rect.min, Pos2::new(full_rect.max.x, split_y));
    let bottom_rect = Rect::from_min_max(Pos2::new(full_rect.min.x, split_y), full_rect.max);

    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(top_rect), |ui| {
        egui::ScrollArea::vertical()
            .id_salt("lib_right_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;

                // 1. Histogram
                if lr_right_panel_header(ui, "Histogram", &mut app.lib_hist_open, None, None) {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(34, 34, 38))
                        .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                        .show(ui, |ui| {
                            draw_histogram_widget(app, ui);

                            if !app.photos.is_empty() {
                                let ph = app.active_photo();
                                ui.add_space(4.0);
                                if let Some(zone) = app.hist_hover_zone {
                                    let p = &ph.params;
                                    let (name, val_str) = match zone {
                                        0 => ("Blacks", format!("{:+.0}", p.blacks_2012)),
                                        1 => ("Shadows", format!("{:+.0}", p.shadows_2012)),
                                        2 => ("Exposure", format!("{:+.2}", p.exposure_2012)),
                                        3 => ("Highlights", format!("{:+.0}", p.highlights_2012)),
                                        4 => ("Whites", format!("{:+.0}", p.whites_2012)),
                                        _ => ("", String::new()),
                                    };
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(format!("◀   {} : {}   ▶", name, val_str))
                                                .size(11.0)
                                                .color(Color32::WHITE)
                                                .strong(),
                                        );
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                ui.label(
                                                    RichText::new("Drag to slide • Double-click reset")
                                                        .size(9.5)
                                                        .color(Colors::TEXT_MID),
                                                );
                                            },
                                        );
                                    });
                                } else {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(format!(
                                                "ISO {}   {:.0} mm   f / {:.1}",
                                                ph.iso, ph.focal_length_mm, ph.aperture_f
                                            ))
                                            .size(10.5)
                                            .color(Colors::TEXT_MID),
                                        );
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                ui.label(
                                                    RichText::new(&ph.shutter_speed)
                                                        .size(10.5)
                                                        .color(Colors::TEXT_MID),
                                                );
                                            },
                                        );
                                    });
                                }
                            }
                        });
                }

                // 2. Assisted Culling Face & Focus Diagnostics
                if (app.lib_left_tab == LibraryLeftTab::AssistedCulling
                    || !app.active_photo().detected_faces.is_empty())
                    && lr_right_panel_header(
                        ui,
                        "Assisted Culling Info",
                        &mut app.lib_culling_info_open,
                        None,
                        Some("VISION"),
                    )
                {
                    render_culling_info_panel(app, ctx, ui);
                }

                // 3. Quick Develop (Multi-Photo Batch Relative Stepping)
                let qd_badge_str = if !app.lib_quick_dev_open {
                    app.active_photo().params.applied_preset_name.clone()
                } else {
                    String::new()
                };
                let qd_badge = if !qd_badge_str.is_empty() {
                    Some(qd_badge_str.as_str())
                } else {
                    None
                };
                if lr_right_panel_header(
                    ui,
                    "Quick Develop",
                    &mut app.lib_quick_dev_open,
                    None,
                    qd_badge,
                ) {
                    render_quick_develop_panel(app, ui);
                }

                // 4. Keywording & Keyword Suggestions
                if lr_right_panel_header(
                    ui,
                    "Keywording",
                    &mut app.lib_keywording_open,
                    None,
                    None,
                ) {
                    render_keywording_panel(app, ui);
                }

                // 5. Keyword List
                if lr_right_panel_header(
                    ui,
                    "Keyword List",
                    &mut app.lib_keyword_list_open,
                    None,
                    None,
                ) {
                    render_keyword_list_panel(app, ui);
                }

                // 6. Metadata (Default / EXIF / IPTC / Large Caption + Rename on Disk)
                if lr_right_panel_header(
                    ui,
                    "Metadata",
                    &mut app.lib_metadata_open,
                    None,
                    None,
                ) {
                    render_metadata_panel(app, ui);
                }

                // 7. Comments
                if lr_right_panel_header(
                    ui,
                    "Comments",
                    &mut app.lib_comments_open,
                    None,
                    None,
                ) {
                    render_comments_panel(app, ui);
                }
            });
    });

    // Bottom Pinned `[ Sync Settings... ]` | `[ Sync Metadata... ]` strictly in bottom_rect
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(bottom_rect), |ui| {
        egui::Frame::none()
            .fill(Color32::from_rgb(32, 32, 36))
            .inner_margin(egui::Margin::symmetric(8.0, 6.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let multi = app.selected_photo_ids.len() > 1;
                    let avail = ui.available_width();
                    let bw = ((avail - 6.0) * 0.5).max(40.0);
                    ui.add_enabled_ui(multi, |ui| {
                        if lr_bevel_button(ui, "Sync Settings", bw, 24.0).clicked() {
                            app.show_sync_settings_modal = true;
                        }
                        ui.add_space(4.0);
                        if lr_bevel_button(ui, "Sync Metadata", bw, 24.0).clicked() {
                            app.show_sync_metadata_modal = true;
                        }
                    });
                });
            });
    });
}


// ============================================================================
// 2A. LIBRARY RIGHT SUB-PANELS
// ============================================================================
fn render_culling_info_panel(app: &mut LightRawApp, ctx: &egui::Context, ui: &mut Ui) {
    let photo = app.active_photo().clone();
    let tex_opt = app.get_thumb_texture(ctx, photo.id);

    egui::Frame::none()
        .fill(Color32::from_rgb(33, 33, 37))
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            let focus_val = photo.culling_focus_score.unwrap_or(0.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Focus Sharpness:").size(10.5).color(Colors::TEXT_MID));
                if photo.culling_focus_score.is_some() {
                    let col = if focus_val >= app.culling_eye_focus_thresh {
                        Color32::from_rgb(80, 210, 125)
                    } else {
                        Color32::from_rgb(235, 110, 80)
                    };
                    ui.label(RichText::new(format!("{:.0}%", focus_val)).size(11.0).strong().color(col));
                } else {
                    ui.label(RichText::new("Not analyzed").size(10.5).color(Colors::TEXT_DIM));
                }
            });

            if photo.culling_focus_score.is_some() {
                let (bar, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 6.0), Sense::hover());
                ui.painter().rect_filled(bar, 2.0, Color32::from_rgb(22, 22, 26));
                let fill_w = bar.width() * (focus_val / 100.0).clamp(0.0, 1.0);
                let col = if focus_val >= app.culling_eye_focus_thresh {
                    Colors::ACCENT_BLUE
                } else {
                    Color32::from_rgb(220, 95, 70)
                };
                ui.painter().rect_filled(
                    Rect::from_min_size(bar.min, Vec2::new(fill_w, bar.height())),
                    2.0,
                    col,
                );
            }

            ui.add_space(6.0);
            if photo.detected_faces.is_empty() {
                if photo.culling_focus_score.is_none() {
                    ui.label(
                        RichText::new("Click 'Analyze Visible Photos' in Assisted Culling tab to run Apple Vision face & blink detection.")
                            .size(10.0)
                            .color(Colors::TEXT_DIM),
                    );
                } else {
                    ui.label(
                        RichText::new("No human faces detected in frame (scene sharpness scored).")
                            .size(10.0)
                            .color(Colors::TEXT_DIM),
                    );
                }
            } else {
                for (idx, face) in photo.detected_faces.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let (cr, _) = ui.allocate_exact_size(Vec2::new(48.0, 48.0), Sense::hover());
                        ui.painter().rect_filled(cr, 3.0, Color32::from_rgb(22, 22, 25));
                        if let Some(ref tex) = tex_opt {
                            let u0 = (face.x - face.w * 0.12).clamp(0.0, 0.95);
                            let v0 = (face.y - face.h * 0.15).clamp(0.0, 0.95);
                            let u1 = (face.x + face.w * 1.12).clamp(u0 + 0.04, 1.0);
                            let v1 = (face.y + face.h * 1.15).clamp(v0 + 0.04, 1.0);
                            ui.painter().image(
                                tex.id(),
                                cr,
                                Rect::from_min_max(Pos2::new(u0, v0), Pos2::new(u1, v1)),
                                Color32::WHITE,
                            );
                        }
                        ui.painter().rect_stroke(cr, 3.0, Stroke::new(1.0, Color32::from_rgb(65, 65, 72)));

                        ui.vertical(|ui| {
                            let title = if face.person_name.trim().is_empty() {
                                format!("Subject Face #{}", idx + 1)
                            } else {
                                face.person_name.clone()
                            };
                            ui.label(RichText::new(title).size(10.5).strong().color(Colors::TEXT_BRIGHT));
                            let eyes_ok = face.eyes_open_score >= app.culling_eyes_open_thresh;
                            let eye_col = if eyes_ok {
                                Color32::from_rgb(85, 215, 130)
                            } else {
                                Color32::from_rgb(240, 95, 85)
                            };
                            ui.label(
                                RichText::new(format!(
                                    "Eyes Open: {:.0}%  •  Quality: {:.0}%",
                                    face.eyes_open_score, face.quality
                                ))
                                .size(9.8)
                                .color(eye_col),
                            );
                            ui.label(
                                RichText::new(format!("Head Yaw: {:+.0}°", face.yaw_deg))
                                    .size(9.5)
                                    .color(Colors::TEXT_DIM),
                            );
                        });
                    });
                    ui.add_space(4.0);
                }
            }
        });
}

fn step_quick_develop_field(app: &mut LightRawApp, label: &str, f: impl Fn(&mut crate::params::DevelopParams)) {
    let ids = if app.selected_photo_ids.is_empty() {
        vec![app.active_photo_id]
    } else {
        app.selected_photo_ids.clone()
    };
    for id in &ids {
        if let Some(p) = app.photos.iter_mut().find(|x| x.id == *id) {
            f(&mut p.params);
            let snap = p.params.clone();
            if p.history_index + 1 < p.history.len() {
                p.history.truncate(p.history_index + 1);
            }
            p.history.push(HistoryStep {
                name: format!("Quick Develop: {}", label),
                value_summary: String::new(),
                timestamp: "Quick Dev".into(),
                params: snap,
            });
            p.history_index = p.history.len() - 1;
            app.catalog_db.save_photo(p);
        }
        app.invalidate_photo_render_caches(*id);
    }
    app.status_message = format!("Quick Develop '{}' applied to {} photo(s).", label, ids.len());
}

fn quick_step_row(
    ui: &mut Ui,
    label: &str,
    big_neg_tip: &str,
    small_neg_tip: &str,
    small_pos_tip: &str,
    big_pos_tip: &str,
) -> Option<i8> {
    let mut out = None;
    let total_w = ui.available_width();
    let row_h = 20.0;
    let (row_rect, _) = ui.allocate_exact_size(Vec2::new(total_w, row_h), Sense::hover());

    // Left label: fixed width 68.0 pt
    let label_w = 68.0;
    let label_rect = Rect::from_min_size(row_rect.min, Vec2::new(label_w, row_h));
    ui.painter().text(
        Pos2::new(label_rect.min.x, label_rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(10.5),
        Colors::TEXT_MAIN,
    );

    // Right segmented button group
    let btns_x = label_rect.max.x + 4.0;
    let btns_w = (row_rect.max.x - btns_x).max(60.0);
    let gap = 2.0;
    let bw = ((btns_w - 3.0 * gap) / 4.0).floor().max(16.0);

    let buttons = [
        (-2, "«", big_neg_tip),
        (-1, "<", small_neg_tip),
        (1, ">", small_pos_tip),
        (2, "»", big_pos_tip),
    ];

    for (i, (step_val, glyph, tip)) in buttons.iter().enumerate() {
        let x0 = btns_x + (i as f32) * (bw + gap);
        let x1 = if i == 3 { row_rect.max.x } else { x0 + bw };
        let b_rect = Rect::from_min_max(Pos2::new(x0, row_rect.min.y), Pos2::new(x1, row_rect.max.y));
        let id = ui.make_persistent_id(("qd_step_btn", label, i));
        let resp = ui.interact(b_rect, id, Sense::click()).on_hover_text(*tip);

        let bg = if resp.is_pointer_button_down_on() {
            Color32::from_rgb(26, 26, 30)
        } else if resp.hovered() {
            Color32::from_rgb(58, 58, 64)
        } else {
            Color32::from_rgb(44, 44, 48)
        };

        ui.painter().rect_filled(b_rect, 2.0, bg);
        let border_col = if resp.hovered() {
            Color32::from_rgb(76, 76, 84)
        } else {
            Color32::from_rgb(30, 30, 34)
        };
        ui.painter().rect_stroke(b_rect, 2.0, Stroke::new(1.0, border_col));

        // Subtle 3D top highlight bevel
        if !resp.is_pointer_button_down_on() {
            ui.painter().line_segment(
                [b_rect.left_top() + Vec2::new(1.0, 1.0), b_rect.right_top() + Vec2::new(-1.0, 1.0)],
                Stroke::new(1.0, Color32::from_rgb(60, 60, 66)),
            );
        }

        ui.painter().text(
            b_rect.center(),
            egui::Align2::CENTER_CENTER,
            *glyph,
            egui::FontId::proportional(10.0),
            if resp.hovered() { Color32::WHITE } else { Colors::TEXT_BRIGHT },
        );

        if resp.clicked() {
            out = Some(*step_val);
        }
    }

    out
}

fn render_quick_develop_panel(app: &mut LightRawApp, ui: &mut Ui) {
    egui::Frame::none()
        .fill(Color32::from_rgb(34, 34, 38))
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            let label_w = 68.0;

            // 1. Saved Preset Dropdown
            let raw_preset_name = app.active_photo().params.applied_preset_name.clone();
            let cur_preset = if raw_preset_name.is_empty() {
                "Default Settings".to_string()
            } else {
                raw_preset_name
            };
            let preset_names: Vec<String> = app
                .presets
                .iter()
                .filter(|p| p.is_visible)
                .map(|p| p.name.clone())
                .collect();
            let mut chosen_preset: Option<String> = None;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                ui.add_sized(
                    [label_w, 20.0],
                    egui::Label::new(RichText::new("Saved Preset").size(10.5).color(Colors::TEXT_MID)),
                );
                let cb_w = (ui.available_width() - 2.0).max(80.0);
                egui::ComboBox::from_id_salt("qd_saved_preset")
                    .selected_text(RichText::new(&cur_preset).size(10.5))
                    .width(cb_w)
                    .show_ui(ui, |ui| {
                        for name in &preset_names {
                            if ui.selectable_label(name == &cur_preset, name).clicked() {
                                chosen_preset = Some(name.clone());
                            }
                        }
                    });
            });
            if let Some(pr_name) = chosen_preset {
                app.apply_preset_to_selection(&pr_name);
            }

            // 2. Treatment: Color | Black & White (Clean Segmented Bevel Pill, Zero Cyan)
            let is_bw = app.active_photo().params.convert_to_grayscale;
            let total_w = ui.available_width();
            let (t_row, _) = ui.allocate_exact_size(Vec2::new(total_w, 20.0), Sense::hover());
            let t_label_rect = Rect::from_min_size(t_row.min, Vec2::new(label_w, 20.0));
            ui.painter().text(
                Pos2::new(t_label_rect.min.x, t_label_rect.center().y),
                egui::Align2::LEFT_CENTER,
                "Treatment",
                egui::FontId::proportional(10.5),
                Colors::TEXT_MID,
            );
            let t_btns_x = t_label_rect.max.x + 4.0;
            let t_btns_w = (t_row.max.x - t_btns_x).max(60.0);
            let t_gap = 2.0;
            let half_w = ((t_btns_w - t_gap) * 0.5).floor();

            let r_col = Rect::from_min_max(Pos2::new(t_btns_x, t_row.min.y), Pos2::new(t_btns_x + half_w, t_row.max.y));
            let r_bw = Rect::from_min_max(Pos2::new(t_btns_x + half_w + t_gap, t_row.min.y), Pos2::new(t_row.max.x, t_row.max.y));

            let resp_c = ui.interact(r_col, ui.make_persistent_id("qd_treat_col"), Sense::click()).on_hover_text("Color Treatment");
            let resp_bw = ui.interact(r_bw, ui.make_persistent_id("qd_treat_bw"), Sense::click()).on_hover_text("Black & White Treatment");

            if resp_c.clicked() && is_bw {
                step_quick_develop_field(app, "Color Treatment", |p| p.convert_to_grayscale = false);
            }
            if resp_bw.clicked() && !is_bw {
                step_quick_develop_field(app, "B&W Treatment", |p| p.convert_to_grayscale = true);
            }

            // Paint Color Segment
            let c_bg = if !is_bw {
                Color32::from_rgb(62, 62, 70)
            } else if resp_c.hovered() {
                Color32::from_rgb(48, 48, 54)
            } else {
                Color32::from_rgb(38, 38, 42)
            };
            ui.painter().rect_filled(r_col, 2.0, c_bg);
            ui.painter().rect_stroke(
                r_col,
                2.0,
                Stroke::new(1.0, if !is_bw { Color32::from_rgb(85, 85, 95) } else { Color32::from_rgb(30, 30, 34) }),
            );
            ui.painter().text(
                r_col.center(),
                egui::Align2::CENTER_CENTER,
                "Color",
                egui::FontId::proportional(10.5),
                if !is_bw { Color32::WHITE } else { Colors::TEXT_MID },
            );

            // Paint Black & White Segment
            let bw_bg = if is_bw {
                Color32::from_rgb(62, 62, 70)
            } else if resp_bw.hovered() {
                Color32::from_rgb(48, 48, 54)
            } else {
                Color32::from_rgb(38, 38, 42)
            };
            ui.painter().rect_filled(r_bw, 2.0, bw_bg);
            ui.painter().rect_stroke(
                r_bw,
                2.0,
                Stroke::new(1.0, if is_bw { Color32::from_rgb(85, 85, 95) } else { Color32::from_rgb(30, 30, 34) }),
            );
            ui.painter().text(
                r_bw.center(),
                egui::Align2::CENTER_CENTER,
                "Black & White",
                egui::FontId::proportional(10.5),
                if is_bw { Color32::WHITE } else { Colors::TEXT_MID },
            );

            // 3. White Balance Preset Dropdown
            let cur_wb = app.active_photo().params.white_balance;
            let mut next_wb: Option<crate::params::WhiteBalanceMode> = None;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                ui.add_sized(
                    [label_w, 20.0],
                    egui::Label::new(RichText::new("White Balance").size(10.5).color(Colors::TEXT_MID)),
                );
                let cb_w = (ui.available_width() - 2.0).max(80.0);
                egui::ComboBox::from_id_salt("qd_wb_combo")
                    .selected_text(RichText::new(cur_wb.label()).size(10.5))
                    .width(cb_w)
                    .show_ui(ui, |ui| {
                        for &mode in crate::params::WhiteBalanceMode::ALL {
                            if ui.selectable_label(cur_wb == mode, mode.label()).clicked() {
                                next_wb = Some(mode);
                            }
                        }
                    });
            });
            if let Some(mode) = next_wb {
                if mode == crate::params::WhiteBalanceMode::Auto {
                    app.apply_auto_wb_to_selection();
                } else if mode == crate::params::WhiteBalanceMode::AsShot {
                    let ids = if app.selected_photo_ids.is_empty() {
                        vec![app.active_photo_id]
                    } else {
                        app.selected_photo_ids.clone()
                    };
                    for id in ids {
                        let shot = app.ensure_thumb_raw(id).map(|r| (r.as_shot_temp, r.as_shot_tint));
                        if let Some(p) = app.photos.iter_mut().find(|x| x.id == id) {
                            p.params.white_balance = crate::params::WhiteBalanceMode::AsShot;
                            if let Some((t, ti)) = shot {
                                p.params.temperature = t;
                                p.params.tint = ti;
                            }
                            app.catalog_db.save_photo(p);
                        }
                        app.invalidate_photo_render_caches(id);
                    }
                } else {
                    step_quick_develop_field(app, mode.label(), |p| {
                        p.white_balance = mode;
                        if let Some((t, ti)) = mode.preset_temp_tint() {
                            p.temperature = t;
                            p.tint = ti;
                        }
                    });
                }
            }

            ui.add_space(2.0);
            let sep_rect = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover()).0;
            ui.painter().line_segment(
                [sep_rect.left_center(), sep_rect.right_center()],
                Stroke::new(1.0, Color32::from_rgb(46, 46, 52)),
            );
            ui.add_space(2.0);

            // 4. Tone Control Header Row (Lightroom Classic Structure)
            let tone_total_w = ui.available_width();
            let (tone_row, _) = ui.allocate_exact_size(Vec2::new(tone_total_w, 20.0), Sense::hover());
            let tone_label_rect = Rect::from_min_size(tone_row.min, Vec2::new(label_w, 20.0));
            let tone_label_resp = ui.interact(tone_label_rect, ui.make_persistent_id("qd_tone_hdr_lbl"), Sense::click())
                .on_hover_text("Click to toggle expanded Tone controls");
            ui.painter().text(
                Pos2::new(tone_label_rect.min.x, tone_label_rect.center().y),
                egui::Align2::LEFT_CENTER,
                "Tone Control",
                egui::FontId::proportional(10.5),
                if tone_label_resp.hovered() { Color32::WHITE } else { Colors::TEXT_MID },
            );
            if tone_label_resp.clicked() {
                app.lib_quick_dev_tone_expanded = !app.lib_quick_dev_tone_expanded;
            }

            let auto_w = 46.0;
            let arrow_w = 16.0;
            let auto_rect = Rect::from_min_max(
                Pos2::new(tone_row.max.x - auto_w - arrow_w - 4.0, tone_row.min.y),
                Pos2::new(tone_row.max.x - arrow_w - 4.0, tone_row.max.y),
            );
            let arrow_rect = Rect::from_min_max(
                Pos2::new(tone_row.max.x - arrow_w, tone_row.min.y),
                tone_row.max,
            );

            let resp_auto = ui.interact(auto_rect, ui.make_persistent_id("qd_auto_tone_top"), Sense::click())
                .on_hover_text("Automatically calculate optimum tone adjustments");
            let resp_arrow = ui.interact(arrow_rect, ui.make_persistent_id("qd_tone_arrow"), Sense::click())
                .on_hover_text(if app.lib_quick_dev_tone_expanded { "Collapse Tone Controls" } else { "Expand All Tone Controls" });

            if resp_auto.clicked() {
                app.apply_auto_tone_to_selection();
            }
            if resp_arrow.clicked() {
                app.lib_quick_dev_tone_expanded = !app.lib_quick_dev_tone_expanded;
            }

            let auto_bg = if resp_auto.is_pointer_button_down_on() {
                Color32::from_rgb(32, 32, 36)
            } else if resp_auto.hovered() {
                Color32::from_rgb(58, 58, 64)
            } else {
                Color32::from_rgb(44, 44, 48)
            };
            ui.painter().rect_filled(auto_rect, 2.0, auto_bg);
            ui.painter().rect_stroke(auto_rect, 2.0, Stroke::new(1.0, Color32::from_rgb(30, 30, 34)));
            ui.painter().text(
                auto_rect.center(),
                egui::Align2::CENTER_CENTER,
                "Auto",
                egui::FontId::proportional(10.0),
                if resp_auto.hovered() { Color32::WHITE } else { Colors::TEXT_BRIGHT },
            );

            let arrow_glyph = if app.lib_quick_dev_tone_expanded { "▼" } else { "◀" };
            ui.painter().text(
                arrow_rect.center(),
                egui::Align2::CENTER_CENTER,
                arrow_glyph,
                egui::FontId::proportional(10.0),
                if resp_arrow.hovered() { Color32::WHITE } else { Colors::TEXT_MID },
            );

            // 5. Relative Multi-Photo Stepping Rows (Lightroom Classic « < > »)
            if let Some(step) = quick_step_row(
                ui,
                "Exposure",
                "Decrease Exposure 1 Stop (-1.0 EV)",
                "Decrease Exposure 1/3 Stop (-0.33 EV)",
                "Increase Exposure 1/3 Stop (+0.33 EV)",
                "Increase Exposure 1 Stop (+1.0 EV)",
            ) {
                let d = match step {
                    -2 => -1.0,
                    -1 => -0.33,
                    1 => 0.33,
                    _ => 1.0,
                };
                step_quick_develop_field(app, "Exposure", |p| {
                    p.exposure_2012 = (p.exposure_2012 + d).clamp(-5.0, 5.0);
                });
            }

            if app.lib_quick_dev_tone_expanded {
                if let Some(step) = quick_step_row(
                    ui,
                    "Contrast",
                    "Decrease Contrast (-20)",
                    "Decrease Contrast (-5)",
                    "Increase Contrast (+5)",
                    "Increase Contrast (+20)",
                ) {
                    let d = if step.abs() == 2 { 20.0 * (step.signum() as f32) } else { 5.0 * (step.signum() as f32) };
                    step_quick_develop_field(app, "Contrast", |p| {
                        p.contrast_2012 = (p.contrast_2012 + d).clamp(-100.0, 100.0);
                    });
                }
                if let Some(step) = quick_step_row(
                    ui,
                    "Highlights",
                    "Decrease Highlights (-20)",
                    "Decrease Highlights (-5)",
                    "Increase Highlights (+5)",
                    "Increase Highlights (+20)",
                ) {
                    let d = if step.abs() == 2 { 20.0 * (step.signum() as f32) } else { 5.0 * (step.signum() as f32) };
                    step_quick_develop_field(app, "Highlights", |p| {
                        p.highlights_2012 = (p.highlights_2012 + d).clamp(-100.0, 100.0);
                    });
                }
                if let Some(step) = quick_step_row(
                    ui,
                    "Shadows",
                    "Decrease Shadows (-20)",
                    "Decrease Shadows (-5)",
                    "Increase Shadows (+5)",
                    "Increase Shadows (+20)",
                ) {
                    let d = if step.abs() == 2 { 20.0 * (step.signum() as f32) } else { 5.0 * (step.signum() as f32) };
                    step_quick_develop_field(app, "Shadows", |p| {
                        p.shadows_2012 = (p.shadows_2012 + d).clamp(-100.0, 100.0);
                    });
                }
                if let Some(step) = quick_step_row(
                    ui,
                    "Whites",
                    "Decrease Whites (-20)",
                    "Decrease Whites (-5)",
                    "Increase Whites (+5)",
                    "Increase Whites (+20)",
                ) {
                    let d = if step.abs() == 2 { 20.0 * (step.signum() as f32) } else { 5.0 * (step.signum() as f32) };
                    step_quick_develop_field(app, "Whites", |p| {
                        p.whites_2012 = (p.whites_2012 + d).clamp(-100.0, 100.0);
                    });
                }
                if let Some(step) = quick_step_row(
                    ui,
                    "Blacks",
                    "Decrease Blacks (-20)",
                    "Decrease Blacks (-5)",
                    "Increase Blacks (+5)",
                    "Increase Blacks (+20)",
                ) {
                    let d = if step.abs() == 2 { 20.0 * (step.signum() as f32) } else { 5.0 * (step.signum() as f32) };
                    step_quick_develop_field(app, "Blacks", |p| {
                        p.blacks_2012 = (p.blacks_2012 + d).clamp(-100.0, 100.0);
                    });
                }
            }

            if let Some(step) = quick_step_row(
                ui,
                "Clarity",
                "Decrease Clarity (-20)",
                "Decrease Clarity (-5)",
                "Increase Clarity (+5)",
                "Increase Clarity (+20)",
            ) {
                let d = if step.abs() == 2 { 20.0 * (step.signum() as f32) } else { 5.0 * (step.signum() as f32) };
                step_quick_develop_field(app, "Clarity", |p| {
                    p.clarity_2012 = (p.clarity_2012 + d).clamp(-100.0, 100.0);
                });
            }
            if let Some(step) = quick_step_row(
                ui,
                "Vibrance",
                "Decrease Vibrance (-20)",
                "Decrease Vibrance (-5)",
                "Increase Vibrance (+5)",
                "Increase Vibrance (+20)",
            ) {
                let d = if step.abs() == 2 { 20.0 * (step.signum() as f32) } else { 5.0 * (step.signum() as f32) };
                step_quick_develop_field(app, "Vibrance", |p| {
                    p.vibrance = (p.vibrance + d).clamp(-100.0, 100.0);
                });
            }

            // 6. Action Bar: Auto Tone & Reset All (Exact Fit, Zero Overflow)
            ui.add_space(4.0);
            let bar_w = ui.available_width();
            let gap = 6.0;
            let bw = ((bar_w - gap) * 0.5).floor();
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                if lr_bevel_button(ui, "Auto Tone", bw, 22.0)
                    .on_hover_text("Automatically calculate optimum tone adjustments")
                    .clicked()
                {
                    app.apply_auto_tone_to_selection();
                }
                if lr_bevel_button(ui, "Reset All", bw, 22.0)
                    .on_hover_text("Reset all develop settings for selected photos")
                    .clicked()
                {
                    app.reset_selected_photos_settings();
                }
            });
        });
}

fn commit_keyword_buffer_to_selection(app: &mut LightRawApp) {
    let parsed: Vec<String> = app
        .keyword_input_buffer
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let ids = if app.selected_photo_ids.is_empty() {
        vec![app.active_photo_id]
    } else {
        app.selected_photo_ids.clone()
    };
    for id in &ids {
        if let Some(p) = app.photos.iter_mut().find(|x| x.id == *id) {
            p.keywords = parsed.clone();
            app.catalog_db.save_photo(p);
        }
    }
    app.keyword_input_buffer = parsed.join(", ");
}

fn render_keywording_panel(app: &mut LightRawApp, ui: &mut Ui) {
    if app.keyword_input_photo_id != app.active_photo_id {
        app.keyword_input_photo_id = app.active_photo_id;
        app.keyword_input_buffer = app.active_photo().keywords.join(", ");
    }

    egui::Frame::none()
        .fill(Color32::from_rgb(34, 34, 38))
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.label(RichText::new("Keyword Tags (comma-separated, press Enter):").size(10.0).color(Colors::TEXT_MID));
            let resp = ui.add(
                egui::TextEdit::multiline(&mut app.keyword_input_buffer)
                    .desired_rows(2)
                    .desired_width(f32::INFINITY)
                    .font(egui::FontId::proportional(11.0)),
            );
            let enter_commit = resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift);
            if resp.lost_focus() || enter_commit {
                app.keyword_input_buffer = app.keyword_input_buffer.replace('\n', ", ");
                commit_keyword_buffer_to_selection(app);
            }

            ui.add_space(6.0);
            ui.label(RichText::new("Keyword Suggestions:").size(10.0).color(Colors::TEXT_MID));
            ui.horizontal_wrapped(|ui| {
                for sug in [
                    "Portrait",
                    "Studio",
                    "SMK",
                    "Yearbook",
                    "Landscape",
                    "Wedding",
                    "Select",
                    "Portfolio",
                ] {
                    let has_kw = app.active_photo().keywords.iter().any(|k| k.eq_ignore_ascii_case(sug));
                    if ui.selectable_label(has_kw, RichText::new(sug).size(10.0)).clicked() {
                        let ids = if app.selected_photo_ids.is_empty() {
                            vec![app.active_photo_id]
                        } else {
                            app.selected_photo_ids.clone()
                        };
                        for id in &ids {
                            if let Some(p) = app.photos.iter_mut().find(|x| x.id == *id) {
                                if has_kw {
                                    p.keywords.retain(|k| !k.eq_ignore_ascii_case(sug));
                                } else {
                                    p.keywords.push(sug.to_string());
                                }
                                app.catalog_db.save_photo(p);
                            }
                        }
                        app.keyword_input_buffer = app.active_photo().keywords.join(", ");
                    }
                }
            });
        });
}

fn render_keyword_list_panel(app: &mut LightRawApp, ui: &mut Ui) {
    egui::Frame::none()
        .fill(Color32::from_rgb(34, 34, 38))
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut app.new_keyword_list_input)
                        .hint_text("Add new keyword...")
                        .desired_width(ui.available_width() - 34.0),
                );
                let add_now = ui.small_button("+").clicked()
                    || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
                if add_now {
                    let kw = app.new_keyword_list_input.trim().to_string();
                    if !kw.is_empty() {
                        let ids = if app.selected_photo_ids.is_empty() {
                            vec![app.active_photo_id]
                        } else {
                            app.selected_photo_ids.clone()
                        };
                        for id in &ids {
                            if let Some(p) = app.photos.iter_mut().find(|x| x.id == *id) {
                                if !p.keywords.iter().any(|k| k.eq_ignore_ascii_case(&kw)) {
                                    p.keywords.push(kw.clone());
                                    app.catalog_db.save_photo(p);
                                }
                            }
                        }
                        app.keyword_input_buffer = app.active_photo().keywords.join(", ");
                        app.new_keyword_list_input.clear();
                    }
                }
            });

            ui.add_space(4.0);
            let kws = app.unique_keywords();
            if kws.is_empty() {
                ui.label(RichText::new("No keywords in catalog yet.").size(10.0).color(Colors::TEXT_DIM));
            }
            for (kw, count) in kws {
                let mut checked = app.active_photo().keywords.iter().any(|k| k == &kw);
                let mut filter_kw = false;
                let mut delete_kw = false;
                let row = ui.horizontal(|ui| {
                    if ui.checkbox(&mut checked, RichText::new(&kw).size(10.5)).changed() {
                        let ids = if app.selected_photo_ids.is_empty() {
                            vec![app.active_photo_id]
                        } else {
                            app.selected_photo_ids.clone()
                        };
                        for id in &ids {
                            if let Some(p) = app.photos.iter_mut().find(|x| x.id == *id) {
                                if checked {
                                    if !p.keywords.contains(&kw) {
                                        p.keywords.push(kw.clone());
                                    }
                                } else {
                                    p.keywords.retain(|x| x != &kw);
                                }
                                app.catalog_db.save_photo(p);
                            }
                        }
                        app.keyword_input_buffer = app.active_photo().keywords.join(", ");
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("→").on_hover_text("Filter by this keyword").clicked() {
                            filter_kw = true;
                        }
                        ui.label(RichText::new(format!("{}", count)).size(10.0).color(Colors::TEXT_DIM));
                    });
                });
                row.response.context_menu(|ui| {
                    if ui.button(format!("Delete Keyword '{}'", kw)).clicked() {
                        delete_kw = true;
                        ui.close_menu();
                    }
                });
                if filter_kw {
                    app.library_filter_mode = LibraryFilterMode::Text;
                    app.filter_text = kw.clone();
                }
                if delete_kw {
                    for p in &mut app.photos {
                        if p.keywords.contains(&kw) {
                            p.keywords.retain(|x| x != &kw);
                            app.catalog_db.save_photo(p);
                        }
                    }
                    app.keyword_input_buffer = app.active_photo().keywords.join(", ");
                }
            }
        });
}

fn update_metadata_field_on_targets(
    app: &mut LightRawApp,
    mut f: impl FnMut(&mut crate::catalog::PhotoItem),
) {
    let ids = if app.metadata_apply_selected && !app.selected_photo_ids.is_empty() {
        app.selected_photo_ids.clone()
    } else {
        vec![app.active_photo_id]
    };
    for id in ids {
        if let Some(p) = app.photos.iter_mut().find(|x| x.id == id) {
            f(p);
            app.catalog_db.save_photo(p);
        }
    }
}

fn render_metadata_panel(app: &mut LightRawApp, ui: &mut Ui) {
    if app.rename_file_photo_id != app.active_photo_id {
        app.rename_file_photo_id = app.active_photo_id;
        app.rename_file_buffer = app.active_photo().file_name.clone();
    }

    egui::Frame::none()
        .fill(Color32::from_rgb(34, 34, 38))
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 5.0;

            ui.horizontal(|ui| {
                ui.label(RichText::new("Preset:").size(10.5).color(Colors::TEXT_MID));
                egui::ComboBox::from_id_salt("meta_preset_combo")
                    .selected_text(RichText::new(app.metadata_view_preset.label()).size(10.5))
                    .show_ui(ui, |ui| {
                        for pr in MetadataViewPreset::ALL {
                            ui.selectable_value(&mut app.metadata_view_preset, *pr, pr.label());
                        }
                    });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.checkbox(&mut app.metadata_apply_selected, RichText::new("Selected").size(9.8))
                        .on_hover_text("Apply metadata edits to all selected photos");
                });
            });

            ui.separator();

            // File Name + Real Disk Rename (including companion .xmp sidecar if present)
            ui.horizontal(|ui| {
                ui.add_sized([72.0, 18.0], egui::Label::new(RichText::new("File Name").size(10.2).color(Colors::TEXT_MID)));
                ui.add(
                    egui::TextEdit::singleline(&mut app.rename_file_buffer)
                        .desired_width(ui.available_width() - 56.0)
                        .font(egui::FontId::proportional(10.5)),
                );
                if ui.small_button("Rename").clicked() {
                    let new_name = app.rename_file_buffer.trim().to_string();
                    if !new_name.is_empty() && new_name != app.active_photo().file_name {
                        let pid = app.active_photo_id;
                        if let Some(p) = app.photos.iter_mut().find(|x| x.id == pid) {
                            if let Some(ref old_path_str) = p.real_file_path.clone() {
                                let old_path = Path::new(old_path_str);
                                if let Some(parent) = old_path.parent() {
                                    let new_path = parent.join(&new_name);
                                    if !new_path.exists() && std::fs::rename(old_path, &new_path).is_ok() {
                                        let old_xmp = old_path.with_extension("xmp");
                                        if old_xmp.exists() {
                                            let _ = std::fs::rename(&old_xmp, new_path.with_extension("xmp"));
                                        }
                                        p.real_file_path = Some(new_path.to_string_lossy().to_string());
                                        p.file_name = new_name.clone();
                                        app.catalog_db.save_photo(p);
                                        app.status_message = format!("Renamed file on disk to {}", new_name);
                                    } else {
                                        p.file_name = new_name.clone();
                                        app.catalog_db.save_photo(p);
                                    }
                                }
                            } else {
                                p.file_name = new_name.clone();
                                app.catalog_db.save_photo(p);
                            }
                        }
                    }
                }
            });

            let snap = app.active_photo().clone();

            if app.metadata_view_preset != MetadataViewPreset::Exif {
                let mut title = snap.title.clone();
                ui.horizontal(|ui| {
                    ui.add_sized([72.0, 18.0], egui::Label::new(RichText::new("Title").size(10.2).color(Colors::TEXT_MID)));
                    if ui.add(egui::TextEdit::singleline(&mut title).desired_width(f32::INFINITY)).changed() {
                        update_metadata_field_on_targets(app, |p| p.title = title.clone());
                    }
                });

                let mut caption = snap.caption.clone();
                if app.metadata_view_preset == MetadataViewPreset::LargeCaption {
                    ui.label(RichText::new("Caption:").size(10.2).color(Colors::TEXT_MID));
                    if ui
                        .add(egui::TextEdit::multiline(&mut caption).desired_rows(5).desired_width(f32::INFINITY))
                        .changed()
                    {
                        update_metadata_field_on_targets(app, |p| p.caption = caption.clone());
                    }
                } else {
                    ui.horizontal(|ui| {
                        ui.add_sized([72.0, 18.0], egui::Label::new(RichText::new("Caption").size(10.2).color(Colors::TEXT_MID)));
                        if ui.add(egui::TextEdit::singleline(&mut caption).desired_width(f32::INFINITY)).changed() {
                            update_metadata_field_on_targets(app, |p| p.caption = caption.clone());
                        }
                    });
                }

                let mut creator = snap.creator.clone();
                ui.horizontal(|ui| {
                    ui.add_sized([72.0, 18.0], egui::Label::new(RichText::new("Creator").size(10.2).color(Colors::TEXT_MID)));
                    if ui.add(egui::TextEdit::singleline(&mut creator).desired_width(f32::INFINITY)).changed() {
                        update_metadata_field_on_targets(app, |p| p.creator = creator.clone());
                    }
                });

                let mut copyright = snap.copyright.clone();
                ui.horizontal(|ui| {
                    ui.add_sized([72.0, 18.0], egui::Label::new(RichText::new("Copyright").size(10.2).color(Colors::TEXT_MID)));
                    if ui.add(egui::TextEdit::singleline(&mut copyright).desired_width(f32::INFINITY)).changed() {
                        update_metadata_field_on_targets(app, |p| p.copyright = copyright.clone());
                    }
                });

                let mut person = snap.person_tag.clone();
                ui.horizontal(|ui| {
                    ui.add_sized([72.0, 18.0], egui::Label::new(RichText::new("Person").size(10.2).color(Colors::TEXT_MID)));
                    if ui.add(egui::TextEdit::singleline(&mut person).desired_width(f32::INFINITY)).changed() {
                        update_metadata_field_on_targets(app, |p| p.person_tag = person.clone());
                    }
                });
            }

            if app.metadata_view_preset == MetadataViewPreset::Iptc
                || app.metadata_view_preset == MetadataViewPreset::Default
            {
                let mut city = snap.city.clone();
                ui.horizontal(|ui| {
                    ui.add_sized([72.0, 18.0], egui::Label::new(RichText::new("City").size(10.2).color(Colors::TEXT_MID)));
                    if ui.add(egui::TextEdit::singleline(&mut city).desired_width(f32::INFINITY)).changed() {
                        update_metadata_field_on_targets(app, |p| p.city = city.clone());
                    }
                });
                let mut country = snap.country.clone();
                ui.horizontal(|ui| {
                    ui.add_sized([72.0, 18.0], egui::Label::new(RichText::new("Country").size(10.2).color(Colors::TEXT_MID)));
                    if ui.add(egui::TextEdit::singleline(&mut country).desired_width(f32::INFINITY)).changed() {
                        update_metadata_field_on_targets(app, |p| p.country = country.clone());
                    }
                });
            }

            // Rating & Color Label inline controls
            ui.horizontal(|ui| {
                ui.add_sized([72.0, 18.0], egui::Label::new(RichText::new("Rating").size(10.2).color(Colors::TEXT_MID)));
                for r in 1..=5u8 {
                    let sym = if snap.rating >= r { "★" } else { "☆" };
                    let col = if snap.rating >= r { Colors::GOLD_STAR } else { Colors::TEXT_DIM };
                    if ui.add(egui::Button::new(RichText::new(sym).size(12.0).color(col)).frame(false)).clicked() {
                        let next_r = if snap.rating == r { 0 } else { r };
                        app.set_rating_on_selection(next_r);
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.add_sized([72.0, 18.0], egui::Label::new(RichText::new("Label").size(10.2).color(Colors::TEXT_MID)));
                for lbl in [
                    ColorLabel::Red,
                    ColorLabel::Yellow,
                    ColorLabel::Green,
                    ColorLabel::Blue,
                    ColorLabel::Purple,
                ] {
                    let (r, resp) = ui.allocate_exact_size(Vec2::new(14.0, 14.0), Sense::click());
                    ui.painter().rect_filled(r, 2.5, lbl.rgb().unwrap_or(Color32::GRAY));
                    if snap.color_label == lbl {
                        ui.painter().rect_stroke(r.expand(1.2), 3.0, Stroke::new(1.5, Color32::WHITE));
                    }
                    if resp.clicked() {
                        let next = if snap.color_label == lbl { ColorLabel::None } else { lbl };
                        app.set_color_label_on_selection(next);
                    }
                }
            });

            ui.separator();

            // Read-Only Harvested EXIF Rows
            let exif_rows = [
                ("Capture Time", snap.capture_time.clone()),
                ("Dimensions", format!("{} × {} px", snap.width, snap.height)),
                (
                    "Exposure",
                    format!(
                        "{} sec at f/{:.1}, ISO {}",
                        snap.shutter_speed, snap.aperture_f, snap.iso
                    ),
                ),
                ("Focal Length", format!("{:.0} mm", snap.focal_length_mm)),
                ("Camera", format!("{} {}", snap.camera_make, snap.camera_model).trim().to_string()),
                ("Lens", snap.lens_model.clone()),
                (
                    "GPS",
                    match (snap.gps_lat, snap.gps_lon) {
                        (Some(la), Some(lo)) => format!("{:.5}, {:.5}", la, lo),
                        _ => "No GPS coordinates".into(),
                    },
                ),
            ];
            for (k, v) in exif_rows {
                ui.horizontal(|ui| {
                    ui.add_sized([78.0, 15.0], egui::Label::new(RichText::new(k).size(9.8).color(Colors::TEXT_DIM)));
                    ui.label(RichText::new(v).size(10.0).color(Colors::TEXT_MAIN));
                });
            }
        });
}

fn render_comments_panel(app: &mut LightRawApp, ui: &mut Ui) {
    egui::Frame::none()
        .fill(Color32::from_rgb(34, 34, 38))
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            let comments = app.active_photo().comments.clone();
            let mut del_idx: Option<usize> = None;
            if comments.is_empty() {
                ui.label(RichText::new("No review notes or client comments yet.").size(10.0).color(Colors::TEXT_DIM));
            }
            for (idx, c) in comments.iter().enumerate() {
                egui::Frame::none()
                    .fill(Color32::from_rgb(27, 27, 31))
                    .rounding(3.0)
                    .inner_margin(egui::Margin::symmetric(7.0, 5.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&c.author).size(10.0).strong().color(Colors::TEXT_BRIGHT));
                            ui.label(RichText::new(&c.timestamp).size(9.2).color(Colors::TEXT_DIM));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("✕").clicked() {
                                    del_idx = Some(idx);
                                }
                            });
                        });
                        ui.label(RichText::new(&c.text).size(10.2).color(Colors::TEXT_MAIN));
                    });
                ui.add_space(4.0);
            }
            if let Some(i) = del_idx {
                let p = app.active_photo_mut();
                if i < p.comments.len() {
                    p.comments.remove(i);
                    let cloned = p.clone();
                    app.catalog_db.save_photo(&cloned);
                }
            }

            ui.horizontal(|ui| {
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut app.comment_input_buffer)
                        .hint_text("Write a comment...")
                        .desired_width(ui.available_width() - 48.0),
                );
                let post = ui.small_button("Post").clicked()
                    || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
                if post && !app.comment_input_buffer.trim().is_empty() {
                    let author = app.identity_plate_owner.clone();
                    let txt = app.comment_input_buffer.trim().to_string();
                    let p = app.active_photo_mut();
                    p.comments.push(PhotoComment {
                        author,
                        timestamp: "Local".into(),
                        text: txt,
                    });
                    let cloned = p.clone();
                    app.catalog_db.save_photo(&cloned);
                    app.comment_input_buffer.clear();
                }
            });
        });
}

// ============================================================================
// 3. LIBRARY FILTER BAR (Text / Attribute / Metadata / None)
// ============================================================================
fn render_library_filter_bar(app: &mut LightRawApp, ui: &mut Ui) {
    egui::Frame::none()
        .fill(Color32::from_rgb(31, 31, 35))
        .inner_margin(egui::Margin::symmetric(10.0, 5.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Library Filter:").size(10.5).strong().color(Colors::TEXT_MID));
                for (mode, lbl) in [
                    (LibraryFilterMode::Text, "Text"),
                    (LibraryFilterMode::Attribute, "Attribute"),
                    (LibraryFilterMode::Metadata, "Metadata"),
                    (LibraryFilterMode::None, "None"),
                ] {
                    if ui.selectable_label(app.library_filter_mode == mode, RichText::new(lbl).size(10.5)).clicked() {
                        app.library_filter_mode = mode;
                        if mode == LibraryFilterMode::None {
                            app.filter_text.clear();
                            app.filter_min_rating = 0;
                            app.filter_only_picks = false;
                            app.filter_unflagged_only = false;
                            app.filter_color_label = None;
                            app.filter_camera = None;
                        }
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let shown = app.filtered_photo_ids().len();
                    ui.label(
                        RichText::new(format!("{} of {} photos", shown, app.photos.len()))
                            .size(10.0)
                            .color(Colors::TEXT_DIM),
                    );
                });
            });

            match app.library_filter_mode {
                LibraryFilterMode::None => {}
                LibraryFilterMode::Text => {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🔍").size(11.0));
                        ui.add(
                            egui::TextEdit::singleline(&mut app.filter_text)
                                .hint_text("Search filename, folder, camera, title, caption, or keywords...")
                                .desired_width(360.0),
                        );
                        if !app.filter_text.is_empty() && ui.small_button("✕ Clear").clicked() {
                            app.filter_text.clear();
                        }
                    });
                }
                LibraryFilterMode::Attribute => {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if ui.selectable_label(app.filter_only_picks, "⚑ Flagged").clicked() {
                            app.filter_only_picks = !app.filter_only_picks;
                            if app.filter_only_picks {
                                app.filter_unflagged_only = false;
                            }
                        }
                        if ui.selectable_label(app.filter_unflagged_only, "⚐ Unflagged").clicked() {
                            app.filter_unflagged_only = !app.filter_unflagged_only;
                            if app.filter_unflagged_only {
                                app.filter_only_picks = false;
                            }
                        }
                        ui.separator();
                        ui.label(RichText::new("Rating ≥").size(10.0).color(Colors::TEXT_DIM));
                        for r in 1..=5u8 {
                            let on = app.filter_min_rating >= r;
                            let col = if on { Colors::GOLD_STAR } else { Colors::TEXT_DIM };
                            if ui.add(egui::Button::new(RichText::new("★").size(11.5).color(col)).frame(false)).clicked() {
                                app.filter_min_rating = if app.filter_min_rating == r { 0 } else { r };
                            }
                        }
                        ui.separator();
                        for lbl in [
                            ColorLabel::Red,
                            ColorLabel::Yellow,
                            ColorLabel::Green,
                            ColorLabel::Blue,
                            ColorLabel::Purple,
                        ] {
                            let (rect, resp) = ui.allocate_exact_size(Vec2::new(13.0, 13.0), Sense::click());
                            ui.painter().rect_filled(rect, 2.0, lbl.rgb().unwrap_or(Color32::GRAY));
                            if app.filter_color_label == Some(lbl) {
                                ui.painter().rect_stroke(rect.expand(1.5), 2.5, Stroke::new(1.5, Color32::WHITE));
                            }
                            if resp.clicked() {
                                app.filter_color_label = if app.filter_color_label == Some(lbl) {
                                    None
                                } else {
                                    Some(lbl)
                                };
                            }
                        }
                    });
                }
                LibraryFilterMode::Metadata => {
                    ui.add_space(4.0);
                    let cams = app.unique_cameras();
                    let folders = app.unique_folders();
                    let kws = app.unique_keywords();
                    ui.columns(3, |cols| {
                        cols[0].label(RichText::new("Camera").size(10.0).strong().color(Colors::TEXT_MID));
                        if cols[0].selectable_label(app.filter_camera.is_none(), "All Cameras").clicked() {
                            app.filter_camera = None;
                        }
                        for c in cams.iter().take(5) {
                            if cols[0]
                                .selectable_label(app.filter_camera.as_ref() == Some(c), c)
                                .clicked()
                            {
                                app.filter_camera = Some(c.clone());
                            }
                        }

                        cols[1].label(RichText::new("Folder").size(10.0).strong().color(Colors::TEXT_MID));
                        if cols[1].selectable_label(app.selected_folder.is_none(), "All Folders").clicked() {
                            app.selected_folder = None;
                        }
                        for (f, n) in folders.iter().take(5) {
                            let short = Path::new(f).file_name().and_then(|s| s.to_str()).unwrap_or(f);
                            if cols[1]
                                .selectable_label(
                                    app.selected_folder.as_ref() == Some(f),
                                    format!("{} ({})", short, n),
                                )
                                .clicked()
                            {
                                app.selected_folder = Some(f.clone());
                            }
                        }

                        cols[2].label(RichText::new("Keyword").size(10.0).strong().color(Colors::TEXT_MID));
                        if cols[2].selectable_label(app.filter_text.is_empty(), "All Keywords").clicked() {
                            app.filter_text.clear();
                        }
                        for (k, n) in kws.iter().take(5) {
                            if cols[2]
                                .selectable_label(app.filter_text == *k, format!("{} ({})", k, n))
                                .clicked()
                            {
                                app.filter_text = k.clone();
                            }
                        }
                    });
                }
            }
        });
}

// ============================================================================
// 4. VIRTUALIZED GRID, LOUPE, COMPARE, SURVEY, PEOPLE & LIBRARY TOOLBAR
// ============================================================================
fn render_grid_view(app: &mut LightRawApp, ctx: &egui::Context, ui: &mut Ui) {
    let ids = app.filtered_photo_ids();
    if ids.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.35);
                ui.label(
                    RichText::new("No photos match the current Library filter or folder.")
                        .size(13.5)
                        .color(Colors::TEXT_MID),
                );
                ui.add_space(8.0);
                if lr_bevel_button(ui, "Import Photos... (⇧⌘I)", 168.0, 28.0).clicked() {
                    app.show_import_modal = true;
                }
            });
        });
        return;
    }

    let pad = 10.0;
    let avail_w = (ui.available_width() - pad * 2.0).max(160.0);
    let cell_w = app.grid_thumb_size.clamp(110.0, 320.0);
    let cols = ((avail_w / (cell_w + 8.0)).floor() as usize).max(1);
    let actual_cell_w = ((avail_w - ((cols.saturating_sub(1)) as f32) * 8.0) / (cols as f32)).max(105.0);
    let cell_h = actual_cell_w * 0.88 + 38.0;
    let total_rows = (ids.len() + cols - 1) / cols;

    egui::ScrollArea::vertical()
        .id_salt("lib_virtual_grid_scroll")
        .auto_shrink([false, false])
        .show_rows(ui, cell_h + 8.0, total_rows, |ui, row_range| {
            ui.add_space(6.0);
            for row_idx in row_range {
                ui.horizontal(|ui| {
                    ui.add_space(pad);
                    ui.spacing_mut().item_spacing.x = 8.0;
                    let start = row_idx * cols;
                    let end = (start + cols).min(ids.len());
                    for (local_i, &photo_id) in ids[start..end].iter().enumerate() {
                        let seq_num = start + local_i + 1;
                        render_grid_cell(
                            app,
                            ctx,
                            ui,
                            photo_id,
                            seq_num,
                            Vec2::new(actual_cell_w, cell_h),
                        );
                    }
                });
                ui.add_space(8.0);
            }
        });
}

fn render_grid_cell(
    app: &mut LightRawApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    photo_id: i64,
    seq_num: usize,
    cell_size: Vec2,
) {
    let Some(photo) = app.photos.iter().find(|p| p.id == photo_id).cloned() else {
        return;
    };
    let is_active = app.active_photo_id == photo_id;
    let is_selected = app.selected_photo_ids.contains(&photo_id);

    let (rect, resp) = ui.allocate_exact_size(cell_size, Sense::click_and_drag());

    let bg = if is_active {
        Color32::from_rgb(104, 104, 112)
    } else if is_selected {
        Color32::from_rgb(76, 76, 84)
    } else if let Some(lbl_rgb) = photo.color_label.rgb() {
        Color32::from_rgba_unmultiplied(lbl_rgb.r() / 3 + 28, lbl_rgb.g() / 3 + 28, lbl_rgb.b() / 3 + 30, 255)
    } else {
        Color32::from_rgb(50, 50, 56)
    };
    ui.painter().rect_filled(rect, 2.5, bg);
    let border_col = if is_active {
        Color32::from_rgb(225, 225, 232)
    } else if is_selected {
        Color32::from_rgb(155, 155, 165)
    } else {
        Color32::from_rgb(28, 28, 32)
    };
    ui.painter().rect_stroke(rect, 2.5, Stroke::new(1.0, border_col));

    // Top cell header: Sequence # + File Name + Target Collection quick dot
    ui.painter().text(
        rect.min + Vec2::new(6.0, 4.0),
        egui::Align2::LEFT_TOP,
        format!("{}", seq_num),
        egui::FontId::proportional(10.5),
        if is_active { Color32::WHITE } else { Colors::TEXT_DIM },
    );
    let disp_name = if photo.is_virtual_copy {
        format!("{} ({})", photo.file_name, photo.copy_name)
    } else {
        photo.file_name.clone()
    };
    ui.painter().text(
        Pos2::new(rect.max.x - 18.0, rect.min.y + 5.0),
        egui::Align2::RIGHT_TOP,
        disp_name,
        egui::FontId::proportional(9.5),
        if is_active { Color32::from_rgb(240, 240, 245) } else { Colors::TEXT_MID },
    );

    // Quick Target Collection circle top-right
    let tc_center = Pos2::new(rect.max.x - 9.0, rect.min.y + 10.0);
    let in_target = app
        .collections
        .iter()
        .find(|c| c.id == app.target_collection_id)
        .map(|c| c.photo_ids.contains(&photo_id))
        .unwrap_or(false);
    if in_target || resp.hovered() {
        ui.painter().circle_filled(
            tc_center,
            4.5,
            if in_target {
                Colors::ACCENT_BLUE
            } else {
                Color32::from_rgba_unmultiplied(20, 20, 24, 180)
            },
        );
        ui.painter()
            .circle_stroke(tc_center, 4.5, Stroke::new(1.0, Color32::from_rgb(210, 210, 218)));
    }

    // Inner thumbnail well
    let well = Rect::from_min_max(
        Pos2::new(rect.min.x + 7.0, rect.min.y + 20.0),
        Pos2::new(rect.max.x - 7.0, rect.max.y - 18.0),
    );
    ui.painter().rect_filled(well, 1.5, Color32::from_rgb(24, 24, 27));

    if let Some(tex) = app.get_thumb_texture(ctx, photo_id) {
        let ts = tex.size_vec2();
        let sc = (well.width() / ts.x.max(1.0)).min(well.height() / ts.y.max(1.0));
        let dst = Rect::from_center_size(well.center(), ts * sc);
        let tint = if photo.flag == PickFlag::Rejected {
            Color32::from_rgba_unmultiplied(150, 150, 150, 165)
        } else {
            Color32::WHITE
        };
        ui.painter().image(
            tex.id(),
            dst,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            tint,
        );
        paint_thumbnail_corner_badges(ui, dst, &photo);
    }

    // Bottom cell strip: Interactive 5-star row + Color Label chip
    for s in 1..=5u8 {
        let sx = rect.min.x + 8.0 + ((s - 1) as f32) * 11.5;
        let sy = rect.max.y - 14.0;
        let s_rect = Rect::from_min_size(Pos2::new(sx, sy), Vec2::new(11.0, 12.0));
        let sym = if photo.rating >= s { "★" } else { "·" };
        let col = if photo.rating >= s {
            Colors::GOLD_STAR
        } else {
            Color32::from_rgb(115, 115, 125)
        };
        ui.painter().text(
            s_rect.center(),
            egui::Align2::CENTER_CENTER,
            sym,
            egui::FontId::proportional(9.5),
            col,
        );
    }
    if let Some(lbl_col) = photo.color_label.rgb() {
        let chip = Rect::from_min_size(Pos2::new(rect.max.x - 18.0, rect.max.y - 13.0), Vec2::new(10.0, 9.0));
        ui.painter().rect_filled(chip, 1.5, lbl_col);
    }

    // Interactions: Painter Spray Can, Target Collection dot, Multi-select, Double-click to Loupe
    if resp.hovered() {
        if app.painter_active {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
        } else {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    }

    if app.painter_active && (resp.clicked() || (resp.hovered() && ui.input(|i| i.pointer.primary_down()))) {
        app.apply_painter_spray_to_photo(photo_id);
    } else if resp.double_clicked() {
        app.select_photo(photo_id, false);
        app.library_view_mode = LibraryViewMode::Loupe;
    } else if resp.clicked() {
        if let Some(pos) = resp.interact_pointer_pos() {
            if pos.distance(tc_center) <= 7.0 {
                app.select_photo(photo_id, false);
                app.toggle_target_collection_for_selection();
                return;
            }
        }
        let mods = ui.input(|i| i.modifiers);
        if mods.shift {
            app.select_photo_range(photo_id);
        } else if mods.command || mods.ctrl {
            app.toggle_select_photo(photo_id);
        } else {
            app.select_photo(photo_id, false);
        }
    }

    resp.context_menu(|ui| {
        app.render_photo_context_menu(ui, photo_id);
    });
}

fn render_loupe_view(app: &mut LightRawApp, ctx: &egui::Context, ui: &mut Ui) {
    let (canvas_rect, resp) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
    ui.painter().rect_filled(canvas_rect, 0.0, app.canvas_bg_color());
    ui.set_clip_rect(canvas_rect);

    let is_zoomed = app.loupe_zoom > 1.01;
    let cmd_down = ui.input(|i| i.modifiers.command || i.modifiers.ctrl);
    let alt_down = ui.input(|i| i.modifiers.alt);
    let can_marquee_zoom = !is_zoomed || cmd_down;

    let ptr_opt = resp.interact_pointer_pos().or_else(|| ui.input(|i| i.pointer.hover_pos()));

    if resp.hovered() {
        if can_marquee_zoom {
            if alt_down {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ZoomOut);
            } else {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ZoomIn);
            }
        } else if is_zoomed {
            if alt_down {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ZoomOut);
            } else if resp.is_pointer_button_down_on() || resp.dragged() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            } else {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
            }
        } else {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ZoomIn);
        }
    }

    if resp.hovered() {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll.abs() > 0.5 {
            app.loupe_zoom = (app.loupe_zoom * (1.0 + scroll * 0.0025)).clamp(0.5, 8.0);
            if (app.loupe_zoom - 1.0).abs() < 0.04 {
                app.loupe_pan = Vec2::ZERO;
            }
        }
    }

    let pid = app.active_photo_id;
    let photo = app.active_photo().clone();

    let dst_opt = if let Some(tex) = app.get_photo_texture(ctx, pid, true) {
        let pad = canvas_rect.shrink(14.0);
        let ts = tex.size_vec2();
        let fit_sc = (pad.width() / ts.x.max(1.0)).min(pad.height() / ts.y.max(1.0));
        let draw_size = ts * fit_sc * app.loupe_zoom;
        let center = canvas_rect.center() + if app.loupe_zoom > 1.01 { app.loupe_pan } else { Vec2::ZERO };
        let dst = Rect::from_center_size(center, draw_size);
        ui.painter().image(
            tex.id(),
            dst,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
        Some(dst)
    } else {
        None
    };

    let dst = dst_opt.unwrap_or(canvas_rect);

    // Marquee Drag Zoom: Drag to select an area to zoom into!
    if can_marquee_zoom
        && resp.hovered()
        && ui.input(|i| i.pointer.primary_down())
        && app.zoom_drag_start.is_none()
    {
        app.zoom_drag_start = ui.input(|i| i.pointer.press_origin()).or(ptr_opt);
    }

    if let Some(start_pos) = app.zoom_drag_start {
        if !ui.input(|i| i.pointer.primary_down()) || resp.drag_stopped() {
            app.zoom_drag_start = None;
            let end_pos = ptr_opt.unwrap_or(start_pos);
            let marquee_rect = Rect::from_two_pos(start_pos, end_pos);
            if marquee_rect.width() > 8.0 && marquee_rect.height() > 8.0 {
                let crop_box = marquee_rect.intersect(dst);
                if crop_box.width() > 4.0 && crop_box.height() > 4.0 {
                    let u_min = ((crop_box.min.x - dst.min.x) / dst.width()).clamp(0.0, 1.0);
                    let v_min = ((crop_box.min.y - dst.min.y) / dst.height()).clamp(0.0, 1.0);
                    let u_max = ((crop_box.max.x - dst.min.x) / dst.width()).clamp(0.0, 1.0);
                    let v_max = ((crop_box.max.y - dst.min.y) / dst.height()).clamp(0.0, 1.0);

                    let uv_center = Pos2::new((u_min + u_max) * 0.5, (v_min + v_max) * 0.5);
                    let scale_factor = (canvas_rect.width() * 0.94 / crop_box.width())
                        .min(canvas_rect.height() * 0.94 / crop_box.height());
                    let new_zoom = (app.loupe_zoom * scale_factor).clamp(1.0, 16.0);

                    let new_dst_size = Vec2::new(
                        dst.width() / app.loupe_zoom.max(0.1) * new_zoom,
                        dst.height() / app.loupe_zoom.max(0.1) * new_zoom,
                    );
                    let mut new_pan = -Vec2::new(
                        (uv_center.x - 0.5) * new_dst_size.x,
                        (uv_center.y - 0.5) * new_dst_size.y,
                    );
                    let max_pan = (new_dst_size - canvas_rect.size() * 0.5).max(Vec2::ZERO);
                    new_pan = new_pan.clamp(-max_pan, max_pan);

                    app.loupe_zoom = new_zoom;
                    app.loupe_pan = new_pan;
                }
            } else {
                // Short click toggle
                if alt_down {
                    app.loupe_zoom = (app.loupe_zoom / 2.0).max(1.0);
                    if app.loupe_zoom <= 1.02 {
                        app.loupe_pan = Vec2::ZERO;
                    }
                } else if is_zoomed {
                    app.loupe_zoom = 1.0;
                    app.loupe_pan = Vec2::ZERO;
                } else {
                    app.loupe_zoom = 2.0;
                    if let Some(pos) = ptr_opt {
                        let center = dst.center();
                        app.loupe_pan = (center - pos) * (app.loupe_zoom - 1.0);
                    }
                }
            }
        } else {
            // Still holding primary mouse: paint live marquee selection
            let curr_pos = ptr_opt.unwrap_or(start_pos);
            let marquee_rect = Rect::from_two_pos(start_pos, curr_pos);
            if marquee_rect.width() > 3.0 || marquee_rect.height() > 3.0 {
                ui.painter().rect_filled(
                    marquee_rect,
                    0.0,
                    Color32::from_rgba_unmultiplied(255, 255, 255, 25),
                );
                ui.painter().rect_stroke(
                    marquee_rect.expand(1.0),
                    0.0,
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 0, 0, 180)),
                );
                ui.painter().rect_stroke(
                    marquee_rect,
                    0.0,
                    Stroke::new(1.5, Color32::WHITE),
                );

                // Corner tick marks
                let tick_len = 8.0_f32.min(marquee_rect.width() * 0.3).min(marquee_rect.height() * 0.3);
                let tick_stroke = Stroke::new(2.0, Color32::WHITE);
                ui.painter().line_segment([marquee_rect.min, marquee_rect.min + Vec2::new(tick_len, 0.0)], tick_stroke);
                ui.painter().line_segment([marquee_rect.min, marquee_rect.min + Vec2::new(0.0, tick_len)], tick_stroke);
                ui.painter().line_segment([Pos2::new(marquee_rect.max.x, marquee_rect.min.y), Pos2::new(marquee_rect.max.x - tick_len, marquee_rect.min.y)], tick_stroke);
                ui.painter().line_segment([Pos2::new(marquee_rect.max.x, marquee_rect.min.y), Pos2::new(marquee_rect.max.x, marquee_rect.min.y + tick_len)], tick_stroke);
                ui.painter().line_segment([Pos2::new(marquee_rect.min.x, marquee_rect.max.y), Pos2::new(marquee_rect.min.x + tick_len, marquee_rect.max.y)], tick_stroke);
                ui.painter().line_segment([Pos2::new(marquee_rect.min.x, marquee_rect.max.y), Pos2::new(marquee_rect.min.x, marquee_rect.max.y - tick_len)], tick_stroke);
                ui.painter().line_segment([marquee_rect.max, marquee_rect.max - Vec2::new(tick_len, 0.0)], tick_stroke);
                ui.painter().line_segment([marquee_rect.max, marquee_rect.max - Vec2::new(0.0, tick_len)], tick_stroke);

                // Dynamic zoom ratio floating pill
                let scale_w = canvas_rect.width() * 0.94 / marquee_rect.width().max(1.0);
                let scale_h = canvas_rect.height() * 0.94 / marquee_rect.height().max(1.0);
                let preview_zoom = (app.loupe_zoom * scale_w.min(scale_h)).clamp(1.0, 16.0);
                let badge_str = format!("{:.0}%", preview_zoom * 100.0);
                let badge_pos = Pos2::new(
                    (marquee_rect.max.x + 8.0).min(canvas_rect.max.x - 56.0),
                    (marquee_rect.max.y - 18.0).max(canvas_rect.min.y + 6.0),
                );
                let b_rect = Rect::from_min_size(badge_pos, Vec2::new(48.0, 18.0));
                ui.painter().rect_filled(b_rect, 3.0, Color32::from_rgba_unmultiplied(20, 20, 24, 220));
                ui.painter().rect_stroke(b_rect, 3.0, Stroke::new(1.0, Color32::from_rgb(80, 80, 90)));
                ui.painter().text(b_rect.center(), egui::Align2::CENTER_CENTER, badge_str, egui::FontId::proportional(10.5), Color32::WHITE);
            }
        }
    } else if (resp.clicked() || resp.double_clicked()) && (!resp.dragged() || resp.drag_delta().length() < 4.0) {
        if alt_down {
            app.loupe_zoom = (app.loupe_zoom / 2.0).max(1.0);
            if app.loupe_zoom <= 1.02 {
                app.loupe_pan = Vec2::ZERO;
            }
        } else if is_zoomed {
            app.loupe_zoom = 1.0;
            app.loupe_pan = Vec2::ZERO;
        } else {
            app.loupe_zoom = 2.0;
            if let Some(pos) = ptr_opt {
                let center = dst.center();
                app.loupe_pan = (center - pos) * (app.loupe_zoom - 1.0);
            }
        }
    } else if resp.dragged() && is_zoomed && !cmd_down {
        app.loupe_pan += resp.drag_delta();
    }

    if app.show_info_overlay && app.info_overlay_cycle > 0 {
        let lines = if app.info_overlay_cycle == 1 {
            vec![
                photo.file_name.clone(),
                photo.capture_time.clone(),
                format!("{} × {} px", photo.width, photo.height),
            ]
        } else {
            vec![
                format!(
                    "{} sec at f/{:.1}, ISO {}",
                    photo.shutter_speed, photo.aperture_f, photo.iso
                ),
                format!("{:.0} mm ({})", photo.focal_length_mm, photo.lens_model),
                format!("{} {}", photo.camera_make, photo.camera_model),
            ]
        };
        let box_rect = Rect::from_min_size(canvas_rect.min + Vec2::new(16.0, 14.0), Vec2::new(300.0, 58.0));
        ui.painter().rect_filled(
            box_rect,
            4.0,
            Color32::from_rgba_unmultiplied(12, 12, 15, 175),
        );
        for (i, line) in lines.iter().enumerate() {
            ui.painter().text(
                box_rect.min + Vec2::new(10.0, 7.0 + (i as f32) * 16.0),
                egui::Align2::LEFT_TOP,
                line,
                egui::FontId::proportional(if i == 0 { 12.0 } else { 11.0 }),
                Color32::WHITE,
            );
        }
    }

    resp.context_menu(|ui| {
        app.render_photo_context_menu(ui, pid);
    });
}

fn render_compare_view(app: &mut LightRawApp, ctx: &egui::Context, ui: &mut Ui) {
    let select_id = app.active_photo_id;
    let cand_id = app
        .compare_candidate_id
        .filter(|&cid| cid != select_id && app.photos.iter().any(|p| p.id == cid))
        .unwrap_or_else(|| {
            app.selected_photo_ids
                .iter()
                .copied()
                .find(|&id| id != select_id)
                .unwrap_or_else(|| app.next_photo_id(select_id))
        });
    app.compare_candidate_id = Some(cand_id);

    // Top Compare Action Strip
    egui::Frame::none()
        .fill(Color32::from_rgb(31, 31, 35))
        .inner_margin(egui::Margin::symmetric(12.0, 5.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Compare View (Synced Zoom/Pan)").size(11.0).strong().color(Colors::TEXT_BRIGHT));
                ui.separator();
                if lr_bevel_button(ui, "⇄ Swap", 68.0, 21.0).clicked() {
                    app.active_photo_id = cand_id;
                    app.compare_candidate_id = Some(select_id);
                }
                if lr_bevel_button(ui, "✓ Make Candidate Select", 148.0, 21.0).clicked() {
                    app.select_photo(cand_id, false);
                    app.compare_candidate_id = Some(app.next_photo_id(cand_id));
                }
                if lr_bevel_button(ui, "◀ Prev Candidate", 112.0, 21.0).clicked() {
                    let vis = app.filtered_photo_ids();
                    if vis.len() > 1 {
                        let p = vis.iter().position(|&x| x == cand_id).unwrap_or(0);
                        let mut prev = if p == 0 { vis.len() - 1 } else { p - 1 };
                        if vis[prev] == select_id {
                            prev = if prev == 0 { vis.len() - 1 } else { prev - 1 };
                        }
                        app.compare_candidate_id = Some(vis[prev]);
                    }
                }
                if lr_bevel_button(ui, "Next Candidate ▶", 112.0, 21.0).clicked() {
                    let next = app.next_photo_id(cand_id);
                    app.compare_candidate_id = Some(if next == select_id {
                        app.next_photo_id(next)
                    } else {
                        next
                    });
                }
            });
        });

    let avail = ui.available_size();
    let half_w = (avail.x - 4.0) * 0.5;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (pane_label, pid) in [("SELECT", select_id), ("CANDIDATE", cand_id)] {
            let (pane_rect, resp) = ui.allocate_exact_size(Vec2::new(half_w, avail.y), Sense::click_and_drag());
            ui.painter().rect_filled(pane_rect, 0.0, app.canvas_bg_color());
            if resp.dragged() && app.loupe_zoom > 1.01 {
                app.loupe_pan += resp.drag_delta();
            }
            if resp.double_clicked() {
                app.loupe_zoom = if app.loupe_zoom > 1.05 { 1.0 } else { 2.0 };
                if app.loupe_zoom <= 1.05 {
                    app.loupe_pan = Vec2::ZERO;
                }
            }
            ui.set_clip_rect(pane_rect);
            if let Some(tex) = app.get_photo_texture(ctx, pid, true) {
                let inner = pane_rect.shrink2(Vec2::new(10.0, 26.0));
                let ts = tex.size_vec2();
                let sc = (inner.width() / ts.x.max(1.0)).min(inner.height() / ts.y.max(1.0));
                let dst = Rect::from_center_size(
                    pane_rect.center() + if app.loupe_zoom > 1.01 { app.loupe_pan } else { Vec2::ZERO },
                    ts * sc * app.loupe_zoom,
                );
                ui.painter().image(
                    tex.id(),
                    dst,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
            let p_meta = app.photos.iter().find(|p| p.id == pid).cloned();
            let badge = Rect::from_min_size(pane_rect.min + Vec2::new(10.0, 8.0), Vec2::new(220.0, 22.0));
            ui.painter()
                .rect_filled(badge, 3.0, Color32::from_rgba_unmultiplied(15, 15, 18, 190));
            let txt = if let Some(pm) = p_meta {
                format!("{} — {} ({}★)", pane_label, pm.file_name, pm.rating)
            } else {
                pane_label.to_string()
            };
            ui.painter().text(
                badge.min + Vec2::new(8.0, 4.0),
                egui::Align2::LEFT_TOP,
                txt,
                egui::FontId::proportional(10.5),
                Color32::WHITE,
            );
        }
    });
}

fn render_survey_view(app: &mut LightRawApp, ctx: &egui::Context, ui: &mut Ui) {
    let ids: Vec<i64> = if app.selected_photo_ids.len() >= 2 {
        app.selected_photo_ids.clone()
    } else {
        app.filtered_photo_ids().into_iter().take(6).collect()
    };
    if ids.is_empty() {
        return;
    }

    let n = ids.len();
    let cols = (n as f32).sqrt().ceil().max(1.0) as usize;
    let rows = (n + cols - 1) / cols;
    let avail = ui.available_size();
    let cw = (avail.x - ((cols + 1) as f32) * 10.0) / (cols as f32);
    let ch = (avail.y - ((rows + 1) as f32) * 10.0) / (rows as f32);

    let mut drop_id: Option<i64> = None;
    ui.add_space(10.0);
    for r in 0..rows {
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.spacing_mut().item_spacing.x = 10.0;
            let start = r * cols;
            let end = (start + cols).min(n);
            for &pid in &ids[start..end] {
                let (cell_rect, resp) = ui.allocate_exact_size(Vec2::new(cw.max(80.0), ch.max(80.0)), Sense::click());
                let is_active = pid == app.active_photo_id;
                ui.painter().rect_filled(cell_rect, 3.0, Color32::from_rgb(28, 28, 32));
                if is_active {
                    ui.painter().rect_stroke(cell_rect, 3.0, Stroke::new(2.0, Color32::WHITE));
                }
                if let Some(tex) = app.get_photo_texture(ctx, pid, true) {
                    let inner = cell_rect.shrink(10.0);
                    let ts = tex.size_vec2();
                    let sc = (inner.width() / ts.x.max(1.0)).min(inner.height() / ts.y.max(1.0));
                    let dst = Rect::from_center_size(inner.center(), ts * sc);
                    ui.painter().image(
                        tex.id(),
                        dst,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }

                // Deselect `✕` corner button on hover
                let x_rect = Rect::from_min_size(
                    Pos2::new(cell_rect.max.x - 24.0, cell_rect.min.y + 6.0),
                    Vec2::new(18.0, 18.0),
                );
                if resp.hovered() && ids.len() > 1 {
                    ui.painter().rect_filled(
                        x_rect,
                        3.0,
                        Color32::from_rgba_unmultiplied(15, 15, 18, 210),
                    );
                    ui.painter().text(
                        x_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "✕",
                        egui::FontId::proportional(10.5),
                        Color32::WHITE,
                    );
                }
                if resp.clicked() {
                    if let Some(pos) = resp.interact_pointer_pos() {
                        if x_rect.contains(pos) && ids.len() > 1 {
                            drop_id = Some(pid);
                        } else {
                            app.active_photo_id = pid;
                        }
                    }
                }
            }
        });
        ui.add_space(10.0);
    }

    if let Some(did) = drop_id {
        app.selected_photo_ids.retain(|&x| x != did);
        if app.active_photo_id == did {
            if let Some(&first) = app.selected_photo_ids.first() {
                app.active_photo_id = first;
            }
        }
    }
}

fn render_people_view(app: &mut LightRawApp, ctx: &egui::Context, ui: &mut Ui) {
    let ids = app.filtered_photo_ids();
    egui::Frame::none()
        .fill(Color32::from_rgb(31, 31, 35))
        .inner_margin(egui::Margin::symmetric(12.0, 6.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("People View — Apple Vision Face Detection & Naming")
                        .size(11.5)
                        .strong()
                        .color(Colors::TEXT_BRIGHT),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if lr_bevel_button(ui, "Scan Faces (Vision AI)", 155.0, 22.0).clicked() {
                        app.run_assisted_culling_analysis();
                    }
                });
            });
        });

    egui::ScrollArea::vertical()
        .id_salt("people_view_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(14.0, 14.0);
                for pid in ids {
                    let Some(p_snap) = app.photos.iter().find(|p| p.id == pid).cloned() else {
                        continue;
                    };
                    let tex_opt = app.get_thumb_texture(ctx, pid);
                    ui.allocate_ui(Vec2::new(148.0, 178.0), |ui| {
                        egui::Frame::none()
                            .fill(Color32::from_rgb(42, 42, 47))
                            .rounding(4.0)
                            .inner_margin(egui::Margin::same(8.0))
                            .show(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    let (cr, resp) =
                                        ui.allocate_exact_size(Vec2::new(112.0, 112.0), Sense::click());
                                    ui.painter().rect_filled(cr, 56.0, Color32::from_rgb(24, 24, 28));
                                    if let Some(tex) = tex_opt {
                                        let uv = if let Some(f) = p_snap.detected_faces.first() {
                                            let u0 = (f.x - f.w * 0.18).clamp(0.0, 0.92);
                                            let v0 = (f.y - f.h * 0.20).clamp(0.0, 0.92);
                                            let u1 = (f.x + f.w * 1.18).clamp(u0 + 0.05, 1.0);
                                            let v1 = (f.y + f.h * 1.20).clamp(v0 + 0.05, 1.0);
                                            Rect::from_min_max(Pos2::new(u0, v0), Pos2::new(u1, v1))
                                        } else {
                                            Rect::from_min_max(Pos2::new(0.15, 0.08), Pos2::new(0.85, 0.78))
                                        };
                                        ui.painter().image(tex.id(), cr.shrink(2.0), uv, Color32::WHITE);
                                    }
                                    ui.painter().rect_stroke(
                                        cr,
                                        4.0,
                                        Stroke::new(
                                            if pid == app.active_photo_id { 2.0 } else { 1.0 },
                                            if pid == app.active_photo_id {
                                                Colors::ACCENT_BLUE
                                            } else {
                                                Color32::from_rgb(70, 70, 78)
                                            },
                                        ),
                                    );
                                    if resp.double_clicked() {
                                        app.select_photo(pid, false);
                                        app.library_view_mode = LibraryViewMode::Loupe;
                                    } else if resp.clicked() {
                                        app.select_photo(pid, false);
                                    }

                                    ui.add_space(4.0);
                                    let mut name_buf = p_snap.person_tag.clone();
                                    let te = ui.add(
                                        egui::TextEdit::singleline(&mut name_buf)
                                            .hint_text("Unnamed Person?")
                                            .desired_width(126.0)
                                            .font(egui::FontId::proportional(10.5)),
                                    );
                                    if te.changed() {
                                        if let Some(pm) = app.photos.iter_mut().find(|x| x.id == pid) {
                                            pm.person_tag = name_buf.clone();
                                            if let Some(f0) = pm.detected_faces.first_mut() {
                                                f0.person_name = name_buf;
                                            }
                                            app.catalog_db.save_photo(pm);
                                        }
                                    }
                                    ui.label(
                                        RichText::new(&p_snap.file_name)
                                            .size(9.2)
                                            .color(Colors::TEXT_DIM),
                                    );
                                });
                            });
                    });
                }
            });
        });
}

fn render_library_toolbar(app: &mut LightRawApp, ui: &mut Ui) {
    egui::Frame::none()
        .fill(Colors::BG_TOOLBAR)
        .inner_margin(egui::Margin::symmetric(10.0, 4.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                for (mode, sym, tip) in [
                    (LibraryViewMode::Grid, "⊞", "Grid View (G)"),
                    (LibraryViewMode::Loupe, "▣", "Loupe View (E)"),
                    (LibraryViewMode::Compare, "X|Y", "Compare View (C)"),
                    (LibraryViewMode::Survey, "⊞⊞", "Survey View (N)"),
                    (LibraryViewMode::People, "☺", "People View (O)"),
                ] {
                    let active = app.library_view_mode == mode;
                    if ui
                        .selectable_label(active, RichText::new(sym).size(11.5).strong())
                        .on_hover_text(tip)
                        .clicked()
                    {
                        app.library_view_mode = mode;
                    }
                }

                ui.separator();

                // Painter Spray Can Tool
                let spray_btn = ui
                    .selectable_label(
                        app.painter_active,
                        RichText::new("🖌 Painter").size(10.5).color(if app.painter_active {
                            Colors::ACCENT_AMBER
                        } else {
                            Colors::TEXT_MAIN
                        }),
                    )
                    .on_hover_text("Painter Tool: Click or drag across Grid thumbnails to apply attribute");
                if spray_btn.clicked() {
                    app.painter_active = !app.painter_active;
                }
                if app.painter_active {
                    egui::ComboBox::from_id_salt("painter_mode_combo")
                        .selected_text(RichText::new(app.painter_mode.label()).size(10.2))
                        .width(74.0)
                        .show_ui(ui, |ui| {
                            for m in PainterMode::ALL {
                                ui.selectable_value(&mut app.painter_mode, *m, m.label());
                            }
                        });
                    match app.painter_mode {
                        PainterMode::Rating => {
                            for r in 1..=5u8 {
                                let on = app.painter_rating >= r;
                                if ui
                                    .add(
                                        egui::Button::new(RichText::new("★").size(11.0).color(if on {
                                            Colors::GOLD_STAR
                                        } else {
                                            Colors::TEXT_DIM
                                        }))
                                        .frame(false),
                                    )
                                    .clicked()
                                {
                                    app.painter_rating = r;
                                }
                            }
                        }
                        PainterMode::Flag => {
                            for (fl, s) in [
                                (PickFlag::Picked, "⚑"),
                                (PickFlag::Unflagged, "⚐"),
                                (PickFlag::Rejected, "✕"),
                            ] {
                                if ui.selectable_label(app.painter_flag == fl, s).clicked() {
                                    app.painter_flag = fl;
                                }
                            }
                        }
                        PainterMode::ColorLabel => {
                            for lbl in [
                                ColorLabel::Red,
                                ColorLabel::Yellow,
                                ColorLabel::Green,
                                ColorLabel::Blue,
                                ColorLabel::Purple,
                            ] {
                                let (r, resp) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), Sense::click());
                                ui.painter().rect_filled(r, 2.0, lbl.rgb().unwrap_or(Color32::GRAY));
                                if resp.clicked() {
                                    app.painter_color = Some(lbl);
                                }
                            }
                        }
                        PainterMode::Preset => {
                            let names: Vec<String> =
                                app.presets.iter().filter(|p| p.is_visible).map(|p| p.name.clone()).collect();
                            if app.painter_preset.is_empty() {
                                if let Some(first) = names.first() {
                                    app.painter_preset = first.clone();
                                }
                            }
                            egui::ComboBox::from_id_salt("painter_preset_combo")
                                .selected_text(RichText::new(&app.painter_preset).size(10.0))
                                .width(120.0)
                                .show_ui(ui, |ui| {
                                    for nm in names {
                                        ui.selectable_value(&mut app.painter_preset, nm.clone(), nm);
                                    }
                                });
                        }
                    }
                }

                ui.separator();

                // Sort Mode + Direction
                ui.label(RichText::new("Sort:").size(10.2).color(Colors::TEXT_DIM));
                egui::ComboBox::from_id_salt("lib_sort_combo")
                    .selected_text(RichText::new(app.lib_sort_mode.label()).size(10.2))
                    .width(98.0)
                    .show_ui(ui, |ui| {
                        for sm in LibrarySortMode::ALL {
                            ui.selectable_value(&mut app.lib_sort_mode, *sm, sm.label());
                        }
                    });
                let dir_lbl = if app.lib_sort_ascending { "A↓" } else { "Z↑" };
                if ui.small_button(dir_lbl).on_hover_text("Toggle Sort Direction").clicked() {
                    app.lib_sort_ascending = !app.lib_sort_ascending;
                }

                ui.separator();
                if ui.small_button("⟲").on_hover_text("Rotate Left (⌘[)").clicked() {
                    app.rotate_selection(-90.0);
                }
                if ui.small_button("⟳").on_hover_text("Rotate Right (⌘])").clicked() {
                    app.rotate_selection(90.0);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if app.library_view_mode == LibraryViewMode::Grid {
                        ui.add(
                            egui::Slider::new(&mut app.grid_thumb_size, 110.0..=300.0)
                                .show_value(false)
                                .trailing_fill(true),
                        );
                        ui.label(RichText::new("Thumbnails").size(10.0).color(Colors::TEXT_DIM));
                    } else {
                        ui.add(
                            egui::Slider::new(&mut app.loupe_zoom, 0.5..=8.0)
                                .show_value(false)
                                .trailing_fill(true),
                        );
                        ui.label(RichText::new(format!("Zoom {:.0}%", app.loupe_zoom * 100.0)).size(10.0).color(Colors::TEXT_DIM));
                    }
                });
            });
        });
}
