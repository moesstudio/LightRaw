// ============================================================================
// LightRaw — Complete Adobe Lightroom Classic Develop Module
// ============================================================================

use super::develop_canvas::{render_active_tool_drawer, render_develop_canvas};
use super::{
    draw_histogram_widget, lr_bevel_button, lr_gradient_slider, lr_left_panel_header,
    lr_right_panel_header_ex, lr_selectable_row, lr_slider, render_navigator_panel,
    BeforeAfterMode, Colors, DevelopTool, LightRawApp,
};
use crate::params::{BokehShape, CameraProfile, UprightMode, VignetteStyle, WhiteBalanceMode};
use crate::presets::export_preset_to_xmp;
use eframe::egui::{self, Color32, Pos2, Rect, RichText, Sense, Stroke, Ui, Vec2};

pub fn render_develop_module(app: &mut LightRawApp, ctx: &egui::Context) {
    if app.show_left_panel {
        egui::SidePanel::left("develop_left_panel")
            .resizable(true)
            .default_width(256.0)
            .min_width(220.0)
            .max_width(360.0)
            .frame(egui::Frame::none().fill(Colors::BG_PANEL))
            .show(ctx, |ui| {
                render_develop_left_panel(app, ctx, ui);
            });
    }

    if app.show_right_panel {
        egui::SidePanel::right("develop_right_panel")
            .resizable(true)
            .default_width(292.0)
            .min_width(260.0)
            .max_width(400.0)
            .frame(egui::Frame::none().fill(Colors::BG_PANEL))
            .show(ctx, |ui| {
                render_develop_right_panel(app, ctx, ui);
            });
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(app.canvas_bg_color()))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            render_develop_canvas(app, ctx, ui);
        });
}

// ============================================================================
// DEVELOP LEFT PANEL: Navigator, Preset Amount, Presets, Snapshots, History
// ============================================================================
fn render_develop_left_panel(app: &mut LightRawApp, ctx: &egui::Context, ui: &mut Ui) {
    let total_h = ui.available_height();
    let bottom_bar_h = 38.0;

    ui.allocate_ui(Vec2::new(ui.available_width(), (total_h - bottom_bar_h).max(80.0)), |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        let mut nav_open = app.dev_nav_open;
        render_navigator_panel(app, ctx, ui, &mut nav_open);
        app.dev_nav_open = nav_open;

        // Preset Amount Live Scaling Bar (`Preset : <Name>` + `Amount 0..200`)
        egui::Frame::none()
            .fill(Color32::from_rgb(34, 34, 38))
            .inner_margin(egui::Margin::symmetric(8.0, 6.0))
            .show(ui, |ui| {
                let applied_name = if app.photos.is_empty() {
                    String::new()
                } else {
                    app.active_photo().params.applied_preset_name.clone()
                };
                let has_preset = !applied_name.is_empty();
                ui.label(
                    RichText::new(format!(
                        "Preset : {}",
                        if has_preset {
                            applied_name.as_str()
                        } else {
                            "None"
                        }
                    ))
                    .size(11.0)
                    .color(if has_preset {
                        Colors::TEXT_BRIGHT
                    } else {
                        Colors::TEXT_DIM
                    }),
                );
                ui.add_enabled_ui(has_preset, |ui| {
                    let mut amt = app.preset_amount;
                    if lr_slider(ui, "Amount", &mut amt, 0.0..=200.0, 100.0, "{:.0}") {
                        app.set_preset_amount(amt);
                    }
                });
            });

        egui::ScrollArea::vertical()
            .id_salt("dev_left_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                render_presets_section(app, ui);
                render_snapshots_section(app, ui);
                render_history_section(app, ui);
                render_dev_collections_section(app, ui);
            });
    });

    // Bottom Pinned Copy... | Paste Buttons
    egui::Frame::none()
        .fill(Color32::from_rgb(32, 32, 36))
        .inner_margin(egui::Margin::symmetric(8.0, 6.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let bw = (ui.available_width() - 8.0) * 0.5;
                if lr_bevel_button(ui, "Copy...", bw, 24.0).clicked() {
                    app.show_copy_settings_modal = true;
                }
                ui.add_space(6.0);
                if lr_bevel_button(ui, "Paste", bw, 24.0).clicked() {
                    app.paste_clipboard_settings();
                }
            });
        });
}

fn render_presets_section(app: &mut LightRawApp, ui: &mut Ui) {
    // Header with `+` popup menu (`Create Preset...`, `Import Presets...`, `Manage Presets...`)
    let desired = Vec2::new(ui.available_width(), 25.0);
    let (hdr_rect, _) = ui.allocate_exact_size(desired, Sense::hover());
    ui.painter()
        .rect_filled(hdr_rect, 0.0, Color32::from_rgb(38, 38, 42));
    ui.painter().line_segment(
        [hdr_rect.left_bottom(), hdr_rect.right_bottom()],
        Stroke::new(1.0, Color32::from_rgb(24, 24, 28)),
    );
    let title_hit = Rect::from_min_max(
        hdr_rect.min,
        Pos2::new(hdr_rect.max.x - 36.0, hdr_rect.max.y),
    );
    if ui
        .interact(title_hit, ui.make_persistent_id("dev_presets_hdr"), Sense::click())
        .clicked()
    {
        app.dev_presets_open = !app.dev_presets_open;
    }
    ui.painter().text(
        Pos2::new(hdr_rect.min.x + 8.0, hdr_rect.center().y),
        egui::Align2::LEFT_CENTER,
        format!("{}  Presets", if app.dev_presets_open { "▼" } else { "▶" }),
        egui::FontId::proportional(12.0),
        Colors::TEXT_BRIGHT,
    );

    let plus_rect = Rect::from_min_size(
        Pos2::new(hdr_rect.max.x - 30.0, hdr_rect.min.y + 3.0),
        Vec2::new(24.0, 19.0),
    );
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(plus_rect), |ui| {
        ui.menu_button(RichText::new("+▾").size(11.5), |ui| {
            ui.set_min_width(155.0);
            if ui.button("Create Preset...").clicked() {
                app.new_preset_name = "Untitled Preset".into();
                app.show_create_preset_modal = true;
                ui.close_menu();
            }
            if ui.button("Import Presets...").clicked() {
                app.import_xmp_presets_dialog();
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Manage Presets...").clicked() {
                app.show_manage_presets_modal = true;
                ui.close_menu();
            }
        });
    });

    if !app.dev_presets_open {
        return;
    }

    egui::Frame::none()
        .fill(Color32::from_rgb(42, 42, 46))
        .inner_margin(egui::Margin::symmetric(8.0, 6.0))
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut app.preset_search)
                    .hint_text("🔍 Filter Presets")
                    .desired_width(ui.available_width()),
            );
            ui.add_space(4.0);

            let q = app.preset_search.trim().to_lowercase();
            let presets_snapshot = app.presets.clone();

            // 1. Favorites Group (if any preset is marked favorite)
            let favs: Vec<_> = presets_snapshot
                .iter()
                .filter(|p| p.is_favorite && (q.is_empty() || p.name.to_lowercase().contains(&q)))
                .cloned()
                .collect();
            if !favs.is_empty() {
                render_preset_group_block(app, ui, "Favorites", &favs, !q.is_empty());
            }

            // 2. Ordered Visible Categories (A7 & User groups prioritized at top like user screenshot!)
            let mut groups: Vec<String> = Vec::new();
            for p in &presets_snapshot {
                if !p.is_visible {
                    continue;
                }
                if !q.is_empty()
                    && !p.name.to_lowercase().contains(&q)
                    && !p.category.to_lowercase().contains(&q)
                {
                    continue;
                }
                if !groups.contains(&p.category) {
                    groups.push(p.category.clone());
                }
            }
            groups.sort_by(|a, b| {
                let rank = |g: &str| -> u8 {
                    if g == "A7" {
                        0
                    } else if g == "User Presets" {
                        1
                    } else if g.starts_with("Adaptive:") {
                        2
                    } else {
                        3
                    }
                };
                rank(a).cmp(&rank(b)).then_with(|| a.cmp(b))
            });

            for grp in groups {
                let items: Vec<_> = presets_snapshot
                    .iter()
                    .filter(|p| {
                        p.is_visible
                            && p.category == grp
                            && (q.is_empty()
                                || p.name.to_lowercase().contains(&q)
                                || p.category.to_lowercase().contains(&q))
                    })
                    .cloned()
                    .collect();
                if !items.is_empty() {
                    render_preset_group_block(app, ui, &grp, &items, !q.is_empty());
                }
            }
        });
}

fn render_preset_group_block(
    app: &mut LightRawApp,
    ui: &mut Ui,
    grp: &str,
    items: &[crate::presets::DevelopPreset],
    force_open: bool,
) {
    let is_open = force_open || app.expanded_preset_groups.contains(grp);
    let arrow = if is_open { "▼" } else { "▶" };
    let grp_resp = lr_selectable_row(ui, false, &format!("{}  {}", arrow, grp), None);
    if grp_resp.clicked() {
        if is_open {
            app.expanded_preset_groups.remove(grp);
        } else {
            app.expanded_preset_groups.insert(grp.to_string());
        }
    }
    if grp != "Favorites" {
        grp_resp.context_menu(|ui| {
            if ui.button("Rename Group...").clicked() {
                app.rename_preset_input = grp.to_string();
                app.show_rename_preset_modal = Some(format!("GROUP:{}", grp));
                ui.close_menu();
            }
            if ui.button("Export Group...").clicked() {
                app.export_preset_group_dialog(grp);
                ui.close_menu();
            }
            if ui.button("Hide Group").clicked() {
                for p in &mut app.presets {
                    if p.category == grp {
                        p.is_visible = false;
                    }
                }
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Delete Group").clicked() {
                let ids: Vec<String> = app
                    .presets
                    .iter()
                    .filter(|p| p.category == grp && p.is_user)
                    .map(|p| p.id.clone())
                    .collect();
                for id in &ids {
                    app.catalog_db.delete_preset(id);
                }
                app.presets.retain(|p| !(p.category == grp && p.is_user));
                ui.close_menu();
            }
        });
    }

    if !is_open {
        return;
    }

    let active_applied = if app.photos.is_empty() {
        String::new()
    } else {
        app.active_photo().params.applied_preset_name.clone()
    };

    for pr in items {
        let is_selected = active_applied == pr.name;
        let fav_mark = if pr.is_favorite && grp != "Favorites" {
            Some("★")
        } else if app.apply_on_import_preset_id.as_ref() == Some(&pr.id) {
            Some("📥")
        } else {
            None
        };
        let row = lr_selectable_row(ui, is_selected, &format!("    ▤ {}", pr.name), fav_mark);

        // Live Hover Preview on Navigator & Develop Canvas!
        if row.hovered() && !app.photos.is_empty() {
            let preview_p =
                LightRawApp::compute_preset_applied_params(&app.active_photo().params, pr);
            app.hovered_preset_preview = Some((pr.name.clone(), preview_p));
        }

        if row.clicked() {
            app.apply_preset_to_selection(&pr.name);
        }

        // Exact Lightroom Classic Right-Click Context Menu (User Screenshot 2)
        row.context_menu(|ui| {
            ui.set_min_width(195.0);
            let fav_lbl = if pr.is_favorite {
                "Remove from Favorites"
            } else {
                "Add to Favorites"
            };
            if ui.button(fav_lbl).clicked() {
                if let Some(target) = app.presets.iter_mut().find(|x| x.id == pr.id) {
                    target.is_favorite = !target.is_favorite;
                    let nf = target.is_favorite;
                    app.catalog_db.update_preset_favorite(&pr.id, nf);
                }
                ui.close_menu();
            }
            ui.separator();

            let is_imp = app.apply_on_import_preset_id.as_ref() == Some(&pr.id);
            let imp_lbl = if is_imp {
                "✓ Apply on Import"
            } else {
                "Apply on Import"
            };
            if ui.button(imp_lbl).clicked() {
                app.apply_on_import_preset_id = if is_imp { None } else { Some(pr.id.clone()) };
                app.import_develop_preset = if is_imp {
                    "None".into()
                } else {
                    pr.name.clone()
                };
                ui.close_menu();
            }
            ui.separator();

            if ui.button("Rename").clicked() {
                app.rename_preset_input = pr.name.clone();
                app.show_rename_preset_modal = Some(pr.id.clone());
                ui.close_menu();
            }
            if ui.button("Move").clicked() {
                app.move_preset_group_input = pr.category.clone();
                app.show_move_preset_modal = Some(pr.id.clone());
                ui.close_menu();
            }
            if ui.button("Update with Current Settings").clicked() {
                if !app.photos.is_empty() {
                    let cur = app.active_photo().params.clone();
                    if let Some(target) = app.presets.iter_mut().find(|x| x.id == pr.id) {
                        target.params = cur;
                        target.is_user = true;
                        let cloned = target.clone();
                        app.catalog_db.save_preset(&cloned);
                        if let Some(ref xp) = cloned.xmp_file_path {
                            let _ = std::fs::write(xp, export_preset_to_xmp(&cloned));
                        }
                        app.status_message =
                            format!("Updated preset '{}' with current settings.", cloned.name);
                    }
                }
                ui.close_menu();
            }
            if ui.button("Show in Finder").clicked() {
                let xmp_path = if let Some(ref existing) = pr.xmp_file_path {
                    std::path::PathBuf::from(existing)
                } else {
                    let dir = app
                        .workspace_dir
                        .join("Presets")
                        .join(pr.category.replace(['/', '\\', ':'], "_"));
                    let _ = std::fs::create_dir_all(&dir);
                    let p = dir.join(format!("{}.xmp", pr.name.replace(['/', '\\', ':'], "_")));
                    let _ = std::fs::write(&p, export_preset_to_xmp(pr));
                    p
                };
                #[cfg(target_os = "macos")]
                {
                    let _ = std::process::Command::new("open")
                        .arg("-R")
                        .arg(&xmp_path)
                        .spawn();
                }
                ui.close_menu();
            }
            if ui.button("Delete").clicked() {
                if let Some(ref xp) = pr.xmp_file_path {
                    let _ = std::fs::remove_file(xp);
                }
                app.catalog_db.delete_preset(&pr.id);
                app.presets.retain(|x| x.id != pr.id);
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Export...").clicked() {
                app.export_preset_dialog(&pr.id);
                ui.close_menu();
            }
        });
    }
}

fn render_snapshots_section(app: &mut LightRawApp, ui: &mut Ui) {
    if lr_left_panel_header(ui, "Snapshots", &mut app.dev_snapshots_open, &["+"]) == Some(0) {
        let count = if app.photos.is_empty() {
            1
        } else {
            app.active_photo().snapshots.len() + 1
        };
        app.new_snapshot_name = format!("Snapshot {}", count);
        app.show_create_snapshot_modal = true;
    }
    if !app.dev_snapshots_open || app.photos.is_empty() {
        return;
    }
    egui::Frame::none()
        .fill(Color32::from_rgb(42, 42, 46))
        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
        .show(ui, |ui| {
            let snaps = app.active_photo().snapshots.clone();
            if snaps.is_empty() {
                ui.label(RichText::new("No snapshots yet (+ to save)").size(10.5).color(Colors::TEXT_DIM));
                return;
            }
            let mut delete_idx = None;
            for (idx, snap) in snaps.iter().enumerate() {
                let r = lr_selectable_row(ui, false, &snap.name, Some(&snap.timestamp));
                if r.hovered() {
                    app.hovered_history_preview = Some((snap.name.clone(), snap.params.clone()));
                }
                if r.clicked() {
                    app.commit_params_change(&format!("Snapshot: {}", snap.name), snap.params.clone());
                }
                r.context_menu(|ui| {
                    if ui.button("Update with Current Settings").clicked() {
                        let cur = app.active_photo().params.clone();
                        if let Some(s) = app.active_photo_mut().snapshots.get_mut(idx) {
                            s.params = cur;
                        }
                        app.persist_active_photo();
                        ui.close_menu();
                    }
                    if ui.button("Rename...").clicked() {
                        app.rename_snapshot_input = snap.name.clone();
                        app.show_rename_snapshot_modal = Some(idx);
                        ui.close_menu();
                    }
                    if ui.button("Delete").clicked() {
                        delete_idx = Some(idx);
                        ui.close_menu();
                    }
                });
            }
            if let Some(di) = delete_idx {
                app.active_photo_mut().snapshots.remove(di);
                app.persist_active_photo();
            }
        });
}

fn render_history_section(app: &mut LightRawApp, ui: &mut Ui) {
    if lr_left_panel_header(ui, "History", &mut app.dev_history_open, &["✕"]) == Some(0) {
        app.execute_menu_action(52); // Clear History
    }
    if !app.dev_history_open || app.photos.is_empty() {
        return;
    }
    egui::Frame::none()
        .fill(Color32::from_rgb(42, 42, 46))
        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
        .show(ui, |ui| {
            let hist = app.active_photo().history.clone();
            let cur_idx = app.active_photo().history_index;
            for (idx, step) in hist.iter().enumerate().rev() {
                let r = lr_selectable_row(ui, idx == cur_idx, &step.name, None);
                if r.hovered() {
                    app.hovered_history_preview = Some((step.name.clone(), step.params.clone()));
                }
                if r.clicked() {
                    let pid = app.active_photo_id;
                    let p = app.active_photo_mut();
                    p.history_index = idx;
                    p.params = step.params.clone();
                    app.persist_active_photo();
                    app.invalidate_photo_render_caches(pid);
                }
            }
        });
}

fn render_dev_collections_section(app: &mut LightRawApp, ui: &mut Ui) {
    if lr_left_panel_header(ui, "Collections", &mut app.dev_collections_open, &["+"]) == Some(0) {
        app.create_collection_is_smart = false;
        app.new_collection_name = "New Collection".into();
        app.show_create_collection_modal = true;
    }
    if !app.dev_collections_open {
        return;
    }
    egui::Frame::none()
        .fill(Color32::from_rgb(42, 42, 46))
        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
        .show(ui, |ui| {
            let cols = app.collections.clone();
            for col in cols {
                let is_sel = app.selected_collection_id == Some(col.id);
                let icon = if col.is_smart { "⚙" } else { "▤" };
                if lr_selectable_row(ui, is_sel, &format!("{} {}", icon, col.name), None).clicked()
                {
                    app.selected_collection_id = if is_sel { None } else { Some(col.id) };
                }
            }
        });
}

// ============================================================================
// DEVELOP RIGHT PANEL: Histogram, 5-Tool Strip, All 10 ACR Panels & Sync/Reset
// ============================================================================
fn collapse_other_panels_if_solo(app: &mut LightRawApp, opened_panel: &'static str) {
    if !app.solo_mode_right_panels {
        return;
    }
    app.dev_basic_open = opened_panel == "Basic";
    app.dev_curve_open = opened_panel == "Tone Curve";
    app.dev_hsl_open = opened_panel == "Color Mixer";
    app.dev_color_grading_open = opened_panel == "Color Grading";
    app.dev_detail_open = opened_panel == "Detail";
    app.dev_lens_open = opened_panel == "Lens Corrections";
    app.dev_transform_open = opened_panel == "Transform";
    app.dev_lens_blur_open = opened_panel == "Lens Blur";
    app.dev_effects_open = opened_panel == "Effects";
    app.dev_calibration_open = opened_panel == "Calibration";
}

fn render_develop_right_panel(app: &mut LightRawApp, ctx: &egui::Context, ui: &mut Ui) {
    let full_rect = ui.available_rect_before_wrap();
    let bottom_bar_h = 38.0;
    let split_y = (full_rect.max.y - bottom_bar_h).max(full_rect.min.y);

    let top_rect = Rect::from_min_max(full_rect.min, Pos2::new(full_rect.max.x, split_y));
    let bottom_rect = Rect::from_min_max(Pos2::new(full_rect.min.x, split_y), full_rect.max);

    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(top_rect), |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;

            // 1. Histogram + EXIF Strip + Original Photo Checkbox
            let (hist_open, _, _, _) =
                lr_right_panel_header_ex(ui, "Histogram", &mut app.dev_hist_open, None, None);
            if hist_open {
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
                            let mut orig = app.before_after_mode == BeforeAfterMode::BeforeOnly;
                            if ui
                                .checkbox(
                                    &mut orig,
                                    RichText::new("Original Photo")
                                        .size(10.5)
                                        .color(Colors::TEXT_MID),
                                )
                                .changed()
                            {
                                app.before_after_mode = if orig {
                                    BeforeAfterMode::BeforeOnly
                                } else {
                                    BeforeAfterMode::Single
                                };
                            }
                        }
                    });
            }

            // 2. Lightroom Classic 5-Icon Tool Strip (`Sliders | Crop | Remove | Red-Eye | Masking`)
            egui::Frame::none()
                .fill(Color32::from_rgb(42, 42, 46))
                .inner_margin(egui::Margin::symmetric(8.0, 5.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let tools = [
                            (DevelopTool::None, "sliders", "☰", "Edit Sliders (D)"),
                            (DevelopTool::Crop, "crop", "⌑", "Crop & Straighten (R)"),
                            (DevelopTool::Remove, "remove", "🩹", "Remove / Heal / Clone (Q)"),
                            (DevelopTool::RedEye, "redeye", "👁", "Red Eye / Pet Eye"),
                            (DevelopTool::Masking, "mask", "◎", "Masking (K)"),
                        ];
                        let bw = (ui.available_width() - 24.0) / 5.0;
                        for (tool, _id, icon, tip) in tools {
                            let active = app.develop_tool == tool;
                            let (r, resp) =
                                ui.allocate_exact_size(Vec2::new(bw, 24.0), Sense::click());
                            let resp = resp.on_hover_text(tip);
                            if resp.clicked() {
                                let old_tool = app.develop_tool;
                                app.develop_tool = if active && tool != DevelopTool::None {
                                    DevelopTool::None
                                } else {
                                    tool
                                };
                                if old_tool == DevelopTool::Crop && app.develop_tool != DevelopTool::Crop {
                                    app.invalidate_photo_render_caches(app.active_photo_id);
                                }
                            }
                            if active {
                                ui.painter()
                                    .rect_filled(r, 4.0, Color32::from_rgb(84, 84, 92));
                            } else if resp.hovered() {
                                ui.painter()
                                    .rect_filled(r, 4.0, Color32::from_rgb(56, 56, 62));
                            }
                            ui.painter().text(
                                r.center(),
                                egui::Align2::CENTER_CENTER,
                                icon,
                                egui::FontId::proportional(13.5),
                                if active {
                                    Color32::WHITE
                                } else {
                                    Colors::TEXT_MID
                                },
                            );
                        }
                    });
                });

            // 2.5 Active Tool Drawer (Pinned directly under the toolstrip, matching Lightroom Classic)
            if app.develop_tool != DevelopTool::None && !app.photos.is_empty() {
                render_active_tool_drawer(app, ui);
                ui.add_space(2.0);
            }

            // 3. Scrollable 10 Develop Panels in Exact Lightroom Classic Order
            egui::ScrollArea::vertical()
                .id_salt("dev_right_panels_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    if !app.photos.is_empty() {
                        render_panel_basic(app, ui);
                        render_panel_tone_curve(app, ui);
                        render_panel_color_mixer(app, ui);
                        render_panel_color_grading(app, ui);
                        render_panel_detail(app, ctx, ui);
                        render_panel_lens_corrections(app, ui);
                        render_panel_transform(app, ui);
                        render_panel_lens_blur(app, ui);
                        render_panel_effects(app, ui);
                        render_panel_calibration(app, ui);
                    }
                });
        });

    // 4. Bottom Pinned `[Previous]` vs `[⚡ Auto-Sync | Sync...]` | `[Reset]` Bar strictly in bottom_rect
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(bottom_rect), |ui| {
        egui::Frame::none()
            .fill(Color32::from_rgb(32, 32, 36))
            .inner_margin(egui::Margin::symmetric(8.0, 6.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let multi = app.selected_photo_ids.len() > 1;
                    if multi {
                        let sw_resp = ui
                            .selectable_label(app.auto_sync_enabled, "⚡")
                            .on_hover_text("Toggle Auto-Sync across all selected photos");
                        if sw_resp.clicked() {
                            app.auto_sync_enabled = !app.auto_sync_enabled;
                        }
                        let bw = ((ui.available_width() - 34.0) * 0.5).max(40.0);
                        let sync_lbl = if app.auto_sync_enabled {
                            "Auto-Sync"
                        } else {
                            "Sync..."
                        };
                        if lr_bevel_button(ui, sync_lbl, bw, 24.0).clicked() {
                            app.show_sync_settings_modal = true;
                        }
                        ui.add_space(4.0);
                        if lr_bevel_button(ui, "Reset", bw, 24.0).clicked() {
                            app.reset_selected_photos_settings();
                        }
                    } else {
                        let bw = ((ui.available_width() - 8.0) * 0.5).max(40.0);
                        if lr_bevel_button(ui, "Previous", bw, 24.0)
                            .on_hover_text("Apply settings from previously selected/edited photo")
                            .clicked()
                        {
                            app.paste_from_previous_photo();
                        }
                        ui.add_space(6.0);
                        if lr_bevel_button(ui, "Reset", bw, 24.0).clicked() {
                            app.reset_selected_photos_settings();
                        }
                    }
                });
            });
    });
}

fn panel_wrap(
    app: &mut LightRawApp,
    ui: &mut Ui,
    name: &'static str,
    mut open_val: bool,
    is_mod: bool,
    badge: Option<&str>,
    body: impl FnOnce(&mut LightRawApp, &mut Ui),
) -> bool {
    let (is_open, eye_held, just_opened, right_clicked) =
        lr_right_panel_header_ex(ui, name, &mut open_val, Some(is_mod), badge);
    if eye_held {
        app.bypassed_panel = Some(name);
    }
    if right_clicked {
        app.solo_mode_right_panels = !app.solo_mode_right_panels;
        app.status_message = format!(
            "Right-Panel Solo Mode: {}",
            if app.solo_mode_right_panels {
                "Enabled"
            } else {
                "Disabled"
            }
        );
    }
    if just_opened {
        collapse_other_panels_if_solo(app, name);
    }
    if is_open {
        egui::Frame::none()
            .fill(Color32::from_rgb(44, 44, 48))
            .inner_margin(egui::Margin::symmetric(10.0, 8.0))
            .show(ui, |ui| {
                body(app, ui);
            });
    }
    open_val
}

fn render_panel_basic(app: &mut LightRawApp, ui: &mut Ui) {
    let is_mod = app.active_photo().params.is_basic_modified();
    let open = app.dev_basic_open;
    let next_open = panel_wrap(app, ui, "Basic", open, is_mod, None, |app, ui| {
        let mut p = app.active_photo().params.clone();
        let mut changed = false;
        let mut step_name = "Basic Tone";

        // Treatment Row: Auto | B&W | HDR
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.selectable_label(p.hdr_edit_mode, "HDR").clicked() {
                    p.hdr_edit_mode = !p.hdr_edit_mode;
                    changed = true;
                    step_name = "Toggle HDR";
                }
                if ui.selectable_label(p.convert_to_grayscale, "B&W").clicked() {
                    p.convert_to_grayscale = !p.convert_to_grayscale;
                    changed = true;
                    step_name = "Treatment B&W";
                }
                if ui.button("Auto").clicked() {
                    app.apply_auto_tone_to_selection();
                    return;
                }
                if ui
                    .button("⚡ Match Ref")
                    .on_hover_text("Auto Match Color & Tone from Reference Photo")
                    .clicked()
                {
                    app.auto_match_color_to_reference();
                    return;
                }
            });
        });
        ui.add_space(4.0);

        // Profile Row + Browser Grid Toggle + Profile Amount
        ui.horizontal(|ui| {
            ui.label(RichText::new("Profile :").size(11.0).color(Colors::TEXT_MID));
            egui::ComboBox::from_id_salt("basic_profile_cb")
                .selected_text(p.camera_profile.label())
                .width(145.0)
                .show_ui(ui, |ui| {
                    for &prof in CameraProfile::ALL {
                        if ui
                            .selectable_label(p.camera_profile == prof, prof.label())
                            .clicked()
                        {
                            p.camera_profile = prof;
                            if prof == CameraProfile::AdobeMonochrome {
                                p.convert_to_grayscale = true;
                            }
                            changed = true;
                            step_name = "Camera Profile";
                        }
                    }
                });
            if ui
                .selectable_label(app.show_profile_browser, "⊞")
                .on_hover_text("Browse Camera & Creative Profiles")
                .clicked()
            {
                app.show_profile_browser = !app.show_profile_browser;
            }
        });

        if app.show_profile_browser {
            egui::Frame::none()
                .fill(Color32::from_rgb(34, 34, 38))
                .rounding(4.0)
                .inner_margin(6.0)
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for &prof in CameraProfile::ALL {
                            if ui
                                .selectable_label(p.camera_profile == prof, prof.label())
                                .clicked()
                            {
                                p.camera_profile = prof;
                                changed = true;
                                step_name = "Camera Profile";
                            }
                        }
                    });
                });
        }

        if p.camera_profile != CameraProfile::AdobeStandard {
            if lr_slider(ui, "Amount", &mut p.look_amount, 0.0..=200.0, 100.0, "{:.0}") {
                changed = true;
                step_name = "Profile Amount";
            }
        }
        ui.add_space(4.0);

        // White Balance Eyedropper + Presets + Temp/Tint Gradient Sliders
        ui.horizontal(|ui| {
            if ui
                .selectable_label(app.wb_eyedropper_active, "🖌")
                .on_hover_text("White Balance Selector (W) — Click neutral gray/white in photo")
                .clicked()
            {
                app.wb_eyedropper_active = !app.wb_eyedropper_active;
            }
            ui.label(RichText::new("WB :").size(11.0).color(Colors::TEXT_MID));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::ComboBox::from_id_salt("wb_preset_cb")
                    .selected_text(p.white_balance.label())
                    .width(120.0)
                    .show_ui(ui, |ui| {
                        for &wb in WhiteBalanceMode::ALL {
                            if ui.selectable_label(p.white_balance == wb, wb.label()).clicked() {
                                p.white_balance = wb;
                                match wb {
                                    WhiteBalanceMode::AsShot => {
                                        let pid = app.active_photo_id;
                                        if let Some(r) = app.thumb_raw_cache.get(&pid).or_else(|| app.raw_cache.get(&pid)) {
                                            p.temperature = r.as_shot_temp;
                                            p.tint = r.as_shot_tint;
                                        } else {
                                            p.temperature = 5500.0;
                                            p.tint = 0.0;
                                        }
                                    }
                                    WhiteBalanceMode::Auto => {
                                        app.apply_auto_wb_to_selection();
                                        return;
                                    }
                                    _ => {
                                        if let Some((t, ti)) = wb.preset_temp_tint() {
                                            p.temperature = t;
                                            p.tint = ti;
                                        }
                                    }
                                }
                                changed = true;
                                step_name = "White Balance";
                            }
                        }
                    });
            });
        });

        if lr_gradient_slider(
            ui,
            "Temp",
            &mut p.temperature,
            2000.0..=50000.0,
            5500.0,
            Color32::from_rgb(55, 115, 235),
            Color32::from_rgb(245, 195, 55),
            "{:.0}",
        ) {
            p.white_balance = WhiteBalanceMode::Custom;
            changed = true;
            step_name = "Temperature";
        }
        if lr_gradient_slider(
            ui,
            "Tint",
            &mut p.tint,
            -150.0..=150.0,
            0.0,
            Color32::from_rgb(45, 195, 85),
            Color32::from_rgb(215, 65, 195),
            "{:+.0}",
        ) {
            p.white_balance = WhiteBalanceMode::Custom;
            changed = true;
            step_name = "Tint";
        }

        ui.add_space(5.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Tone").size(10.5).color(Colors::TEXT_MID));
        });
        if lr_slider(ui, "Exposure", &mut p.exposure_2012, -5.0..=5.0, 0.0, "{:+.2}") {
            changed = true;
            step_name = "Exposure";
        }
        if lr_slider(ui, "Contrast", &mut p.contrast_2012, -100.0..=100.0, 0.0, "{:+.0}") {
            changed = true;
            step_name = "Contrast";
        }
        ui.add_space(2.0);
        if lr_slider(ui, "Highlights", &mut p.highlights_2012, -100.0..=100.0, 0.0, "{:+.0}") {
            changed = true;
            step_name = "Highlights";
        }
        if lr_slider(ui, "Shadows", &mut p.shadows_2012, -100.0..=100.0, 0.0, "{:+.0}") {
            changed = true;
            step_name = "Shadows";
        }
        if lr_slider(ui, "Whites", &mut p.whites_2012, -100.0..=100.0, 0.0, "{:+.0}") {
            changed = true;
            step_name = "Whites";
        }
        if lr_slider(ui, "Blacks", &mut p.blacks_2012, -100.0..=100.0, 0.0, "{:+.0}") {
            changed = true;
            step_name = "Blacks";
        }

        ui.add_space(5.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Presence").size(10.5).color(Colors::TEXT_MID));
        });
        if lr_slider(ui, "Texture", &mut p.texture, -100.0..=100.0, 0.0, "{:+.0}") {
            changed = true;
            step_name = "Texture";
        }
        if lr_slider(ui, "Clarity", &mut p.clarity_2012, -100.0..=100.0, 0.0, "{:+.0}") {
            changed = true;
            step_name = "Clarity";
        }
        if lr_slider(ui, "Dehaze", &mut p.dehaze, -100.0..=100.0, 0.0, "{:+.0}") {
            changed = true;
            step_name = "Dehaze";
        }
        if !p.convert_to_grayscale {
            if lr_slider(ui, "Vibrance", &mut p.vibrance, -100.0..=100.0, 0.0, "{:+.0}") {
                changed = true;
                step_name = "Vibrance";
            }
            if lr_slider(ui, "Saturation", &mut p.saturation, -100.0..=100.0, 0.0, "{:+.0}") {
                changed = true;
                step_name = "Saturation";
            }
        }

        if p.hdr_edit_mode {
            ui.separator();
            ui.label(RichText::new("SDR Display Settings").size(10.5).color(Colors::ACCENT_AMBER));
            changed |= lr_slider(ui, "SDR Brightness", &mut p.sdr_brightness, -100.0..=100.0, 0.0, "{:+.0}");
            changed |= lr_slider(ui, "SDR Contrast", &mut p.sdr_contrast, -100.0..=100.0, 0.0, "{:+.0}");
            changed |= lr_slider(ui, "SDR Clarity", &mut p.sdr_clarity, 0.0..=100.0, 0.0, "{:.0}");
        }

        if changed {
            app.commit_params_change(step_name, p);
        }
    });
    app.dev_basic_open = next_open;
}

fn render_panel_tone_curve(app: &mut LightRawApp, ui: &mut Ui) {
    let is_mod = app.active_photo().params.is_curve_modified();
    let open = app.dev_curve_open;
    let next_open = panel_wrap(app, ui, "Tone Curve", open, is_mod, None, |app, ui| {
        let mut p = app.active_photo().params.clone();
        let mut changed = false;

        ui.horizontal(|ui| {
            if ui
                .selectable_label(
                    app.develop_tool == DevelopTool::TargetAdjust
                        && app.target_adjust_kind == super::TargetAdjustKind::ParametricCurve,
                    "🎯",
                )
                .on_hover_text("Target Adjustment Tool — Drag vertically on photo")
                .clicked()
            {
                app.target_adjust_kind = super::TargetAdjustKind::ParametricCurve;
                app.develop_tool = if app.develop_tool == DevelopTool::TargetAdjust {
                    DevelopTool::None
                } else {
                    DevelopTool::TargetAdjust
                };
            }
            if ui
                .selectable_label(!app.curve_point_mode, "Parametric")
                .clicked()
            {
                app.curve_point_mode = false;
            }
            if ui.selectable_label(app.curve_point_mode, "Point").clicked() {
                app.curve_point_mode = true;
            }
            if app.curve_point_mode {
                for (ch, lbl, col) in [
                    (0usize, "RGB", Color32::WHITE),
                    (1, "R", Color32::from_rgb(235, 80, 80)),
                    (2, "G", Color32::from_rgb(80, 215, 100)),
                    (3, "B", Color32::from_rgb(90, 150, 245)),
                ] {
                    if ui
                        .selectable_label(app.active_curve_channel == ch, RichText::new(lbl).color(col))
                        .clicked()
                    {
                        app.active_curve_channel = ch;
                    }
                }
            }
        });

        // Interactive 190x190 Tone Curve Box
        let plot_sz = ui.available_width().min(210.0);
        let (rect, resp) = ui.allocate_exact_size(Vec2::splat(plot_sz), Sense::click_and_drag());
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
        }
        ui.painter()
            .rect_filled(rect, 3.0, Color32::from_rgb(26, 26, 30));
        ui.painter()
            .rect_stroke(rect, 3.0, Stroke::new(1.0, Color32::from_rgb(56, 56, 62)));
        for i in 1..4 {
            let f = i as f32 * 0.25;
            let gx = rect.min.x + rect.width() * f;
            let gy = rect.min.y + rect.height() * f;
            ui.painter().line_segment(
                [Pos2::new(gx, rect.min.y), Pos2::new(gx, rect.max.y)],
                Stroke::new(0.7, Color32::from_rgb(40, 40, 45)),
            );
            ui.painter().line_segment(
                [Pos2::new(rect.min.x, gy), Pos2::new(rect.max.x, gy)],
                Stroke::new(0.7, Color32::from_rgb(40, 40, 45)),
            );
        }
        // Baseline diagonal
        ui.painter().line_segment(
            [rect.left_bottom(), rect.right_top()],
            Stroke::new(0.8, Color32::from_rgb(70, 70, 76)),
        );

        let curve_pts = match app.active_curve_channel {
            1 => &mut p.tone_curve_pv2012_red,
            2 => &mut p.tone_curve_pv2012_green,
            3 => &mut p.tone_curve_pv2012_blue,
            _ => &mut p.tone_curve_pv2012,
        };
        let stroke_col = match app.active_curve_channel {
            1 => Color32::from_rgb(235, 85, 85),
            2 => Color32::from_rgb(85, 220, 105),
            3 => Color32::from_rgb(95, 155, 250),
            _ => Color32::WHITE,
        };

        // Draw curve polyline & control points (normalized 0.0..=1.0)
        let screen_pts: Vec<Pos2> = curve_pts
            .iter()
            .map(|&(x, y)| {
                Pos2::new(
                    rect.min.x + x.clamp(0.0, 1.0) * rect.width(),
                    rect.max.y - y.clamp(0.0, 1.0) * rect.height(),
                )
            })
            .collect();
        ui.painter()
            .add(egui::Shape::line(screen_pts.clone(), Stroke::new(1.6, stroke_col)));
        if app.curve_point_mode {
            for sp in &screen_pts {
                ui.painter().circle_filled(*sp, 4.0, stroke_col);
                ui.painter()
                    .circle_stroke(*sp, 4.0, Stroke::new(1.0, Color32::BLACK));
            }

            if let Some(pos) = resp.interact_pointer_pos() {
                if rect.contains(pos) {
                    let cx = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
                    let cy = ((rect.max.y - pos.y) / rect.height()).clamp(0.0, 1.0);

                    if resp.secondary_clicked() && curve_pts.len() > 2 {
                        // Right-click deletes nearest interior point!
                        if let Some((idx, _)) = screen_pts
                            .iter()
                            .enumerate()
                            .skip(1)
                            .take(screen_pts.len() - 2)
                            .find(|(_, sp)| sp.distance(pos) < 14.0)
                        {
                            curve_pts.remove(idx);
                            changed = true;
                        }
                    } else if resp.dragged() || resp.clicked() {
                        if let Some((idx, _)) = screen_pts
                            .iter()
                            .enumerate()
                            .min_by(|a, b| a.1.distance(pos).partial_cmp(&b.1.distance(pos)).unwrap())
                            .filter(|(_, sp)| sp.distance(pos) < 16.0)
                        {
                            let new_x = if idx == 0 {
                                0.0
                            } else if idx + 1 == curve_pts.len() {
                                1.0
                            } else {
                                cx.clamp(curve_pts[idx - 1].0 + 0.01, curve_pts[idx + 1].0 - 0.01)
                            };
                            curve_pts[idx] = (new_x, cy);
                            changed = true;
                        } else if resp.clicked() && curve_pts.len() < 16 {
                            curve_pts.push((cx, cy));
                            curve_pts.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                            changed = true;
                        }
                    }
                }
            }
        }

        if !app.curve_point_mode {
            changed |= lr_slider(ui, "Highlights", &mut p.parametric_highlights, -100.0..=100.0, 0.0, "{:+.0}");
            changed |= lr_slider(ui, "Lights", &mut p.parametric_lights, -100.0..=100.0, 0.0, "{:+.0}");
            changed |= lr_slider(ui, "Darks", &mut p.parametric_darks, -100.0..=100.0, 0.0, "{:+.0}");
            changed |= lr_slider(ui, "Shadows", &mut p.parametric_shadows, -100.0..=100.0, 0.0, "{:+.0}");
            ui.horizontal(|ui| {
                ui.label(RichText::new("Splits:").size(10.5).color(Colors::TEXT_DIM));
                changed |= ui
                    .add(egui::DragValue::new(&mut p.parametric_shadow_split).range(10.0..=45.0).speed(0.5))
                    .changed();
                changed |= ui
                    .add(egui::DragValue::new(&mut p.parametric_midtone_split).range(30.0..=70.0).speed(0.5))
                    .changed();
                changed |= ui
                    .add(egui::DragValue::new(&mut p.parametric_highlight_split).range(55.0..=90.0).speed(0.5))
                    .changed();
            });
        } else {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Point Curve:").size(11.0));
                egui::ComboBox::from_id_salt("pt_curve_preset")
                    .selected_text(&p.tone_curve_name_2012)
                    .width(135.0)
                    .show_ui(ui, |ui| {
                        for (nm, pts) in [
                            ("Linear", vec![(0.0, 0.0), (1.0, 1.0)]),
                            (
                                "Medium Contrast",
                                vec![(0.0, 0.0), (0.25, 0.20), (0.75, 0.80), (1.0, 1.0)],
                            ),
                            (
                                "Strong Contrast",
                                vec![(0.0, 0.0), (0.25, 0.16), (0.75, 0.85), (1.0, 1.0)],
                            ),
                        ] {
                            if ui.selectable_label(p.tone_curve_name_2012 == nm, nm).clicked() {
                                p.tone_curve_name_2012 = nm.to_string();
                                *curve_pts = pts;
                                changed = true;
                            }
                        }
                    });
            });
        }

        changed |= lr_slider(
            ui,
            "Refine Sat",
            &mut p.curve_refine_saturation,
            0.0..=200.0,
            100.0,
            "{:.0}",
        );

        if changed {
            app.commit_params_change("Tone Curve", p);
        }
    });
    app.dev_curve_open = next_open;
}

fn render_panel_color_mixer(app: &mut LightRawApp, ui: &mut Ui) {
    let is_mod = app.active_photo().params.is_color_mixer_modified();
    let open = app.dev_hsl_open;
    let next_open = panel_wrap(app, ui, "Color Mixer", open, is_mod, None, |app, ui| {
        let mut p = app.active_photo().params.clone();
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label(RichText::new("Adjust :").size(11.0).color(Colors::TEXT_MID));
            if ui
                .selectable_label(app.active_hsl_tab < 4 && !app.hsl_color_view_by_color, "HSL")
                .clicked()
            {
                app.hsl_color_view_by_color = false;
                if app.active_hsl_tab == 4 {
                    app.active_hsl_tab = 0;
                }
            }
            if ui
                .selectable_label(app.active_hsl_tab < 4 && app.hsl_color_view_by_color, "Color")
                .clicked()
            {
                app.hsl_color_view_by_color = true;
                if app.active_hsl_tab == 4 {
                    app.active_hsl_tab = 0;
                }
            }
            if ui
                .selectable_label(app.active_hsl_tab == 4, "Point Color")
                .clicked()
            {
                app.active_hsl_tab = 4;
            }
        });
        ui.separator();

        let band_names = [
            "Red", "Orange", "Yellow", "Green", "Aqua", "Blue", "Purple", "Magenta",
        ];
        let band_colors = [
            Color32::from_rgb(225, 55, 55),
            Color32::from_rgb(235, 135, 45),
            Color32::from_rgb(235, 210, 55),
            Color32::from_rgb(65, 195, 85),
            Color32::from_rgb(55, 200, 195),
            Color32::from_rgb(65, 120, 235),
            Color32::from_rgb(155, 85, 225),
            Color32::from_rgb(220, 75, 185),
        ];

        if app.active_hsl_tab == 4 {
            // 3D Point Color Swatch Selector & Shift Controls
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(app.point_color_eyedropper_active, "🖌 Pick Color")
                    .on_hover_text("Click a color in the photo to create a 3D Point Color swatch")
                    .clicked()
                {
                    app.point_color_eyedropper_active = !app.point_color_eyedropper_active;
                }
                if p.point_colors.len() < 8 && ui.small_button("+ Swatch").clicked() {
                    p.point_colors.push(crate::params::PointColorSwatch {
                        name: format!("Swatch {}", p.point_colors.len() + 1),
                        src_hue: 210.0,
                        src_sat: 0.65,
                        src_lum: 0.5,
                        range_amount: 50.0,
                        ..Default::default()
                    });
                    app.selected_point_color_idx = p.point_colors.len() - 1;
                    changed = true;
                }
            });

            if !p.point_colors.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    let mut del_sw = None;
                    for (idx, sw) in p.point_colors.iter().enumerate() {
                        let c = egui::epaint::Hsva::new(
                            sw.src_hue / 360.0,
                            sw.src_sat.clamp(0.15, 1.0),
                            sw.src_lum.clamp(0.25, 1.0),
                            1.0,
                        );
                        let sel = app.selected_point_color_idx == idx;
                        let btn = egui::Button::new(if sel { "✓" } else { "  " }).fill(Color32::from(c));
                        let r = ui.add(btn);
                        if r.clicked() {
                            app.selected_point_color_idx = idx;
                        }
                        if r.secondary_clicked() {
                            del_sw = Some(idx);
                        }
                    }
                    if let Some(di) = del_sw {
                        p.point_colors.remove(di);
                        app.selected_point_color_idx = app.selected_point_color_idx.saturating_sub(1);
                        changed = true;
                    }
                });

                let s_idx = app
                    .selected_point_color_idx
                    .min(p.point_colors.len().saturating_sub(1));
                if let Some(sw) = p.point_colors.get_mut(s_idx) {
                    changed |= lr_slider(ui, "Hue Shift", &mut sw.hue_shift, -100.0..=100.0, 0.0, "{:+.0}");
                    changed |= lr_slider(ui, "Sat. Shift", &mut sw.sat_shift, -100.0..=100.0, 0.0, "{:+.0}");
                    changed |= lr_slider(ui, "Lum. Shift", &mut sw.lum_shift, -100.0..=100.0, 0.0, "{:+.0}");
                    changed |= lr_slider(ui, "Range", &mut sw.range_amount, 0.0..=100.0, 50.0, "{:.0}");
                    changed |= ui
                        .checkbox(&mut p.point_color_visualize_range, "Visualize Range")
                        .changed();
                }
            } else {
                ui.label(
                    RichText::new("Click [🖌 Pick Color] and sample any hue on the canvas.")
                        .size(10.5)
                        .color(Colors::TEXT_DIM),
                );
            }
        } else if app.hsl_color_view_by_color {
            // Single-Color Band Swatch View (`Color` mode)
            ui.horizontal(|ui| {
                for (bi, &bcol) in band_colors.iter().enumerate() {
                    let sel = app.hsl_selected_color_band == bi;
                    let (r, resp) = ui.allocate_exact_size(Vec2::new(22.0, 18.0), Sense::click());
                    if resp.clicked() {
                        app.hsl_selected_color_band = bi;
                    }
                    ui.painter().rect_filled(r, 3.0, bcol);
                    if sel {
                        ui.painter()
                            .rect_stroke(r, 3.0, Stroke::new(1.8, Color32::WHITE));
                    }
                }
            });
            let bi = app.hsl_selected_color_band.min(7);
            ui.label(RichText::new(band_names[bi]).strong().size(11.0));
            changed |= lr_slider(ui, "Hue", &mut p.hue_adjustment[bi], -100.0..=100.0, 0.0, "{:+.0}");
            changed |= lr_slider(
                ui,
                "Saturation",
                &mut p.saturation_adjustment[bi],
                -100.0..=100.0,
                0.0,
                "{:+.0}",
            );
            changed |= lr_slider(
                ui,
                "Luminance",
                &mut p.luminance_adjustment[bi],
                -100.0..=100.0,
                0.0,
                "{:+.0}",
            );
        } else {
            // Classic HSL View: Hue | Saturation | Luminance | All
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(app.develop_tool == DevelopTool::TargetAdjust, "🎯")
                    .on_hover_text("Target Adjustment Tool on Photo")
                    .clicked()
                {
                    app.target_adjust_kind = match app.active_hsl_tab {
                        0 => super::TargetAdjustKind::Hue,
                        1 => super::TargetAdjustKind::Saturation,
                        _ => super::TargetAdjustKind::Luminance,
                    };
                    app.develop_tool = if app.develop_tool == DevelopTool::TargetAdjust {
                        DevelopTool::None
                    } else {
                        DevelopTool::TargetAdjust
                    };
                }
                for (ti, lbl) in [(0, "Hue"), (1, "Saturation"), (2, "Luminance"), (3, "All")] {
                    if ui
                        .selectable_label(app.active_hsl_tab == ti, lbl)
                        .clicked()
                    {
                        app.active_hsl_tab = ti;
                    }
                }
            });

            for (sec_idx, sec_title) in [(0, "Hue"), (1, "Saturation"), (2, "Luminance")] {
                if app.active_hsl_tab != 3 && app.active_hsl_tab != sec_idx {
                    continue;
                }
                if app.active_hsl_tab == 3 {
                    ui.label(RichText::new(sec_title).size(10.5).strong().color(Colors::TEXT_MID));
                }
                let arr = match sec_idx {
                    0 => &mut p.hue_adjustment,
                    1 => &mut p.saturation_adjustment,
                    _ => &mut p.luminance_adjustment,
                };
                for i in 0..8 {
                    changed |= lr_slider(ui, band_names[i], &mut arr[i], -100.0..=100.0, 0.0, "{:+.0}");
                }
            }
        }

        if changed {
            app.commit_params_change("Color Mixer", p);
        }
    });
    app.dev_hsl_open = next_open;
}

fn render_panel_color_grading(app: &mut LightRawApp, ui: &mut Ui) {
    let is_mod = app.active_photo().params.is_color_grading_modified();
    let open = app.dev_color_grading_open;
    let next_open = panel_wrap(app, ui, "Color Grading", open, is_mod, None, |app, ui| {
        let mut p = app.active_photo().params.clone();
        let mut changed = false;

        ui.horizontal(|ui| {
            for (idx, lbl) in [
                (0, "3-Way"),
                (1, "Shadows"),
                (2, "Midtones"),
                (3, "Highlights"),
                (4, "Global"),
            ] {
                if ui
                    .selectable_label(app.color_grade_view_tab == idx, lbl)
                    .clicked()
                {
                    app.color_grade_view_tab = idx;
                }
            }
        });
        ui.add_space(4.0);

        if app.color_grade_view_tab == 0 {
            ui.horizontal(|ui| {
                let r = (ui.available_width() - 20.0) / 6.0;
                changed |= draw_grade_wheel(
                    ui,
                    "Shadows",
                    r,
                    &mut p.split_toning_shadow_hue,
                    &mut p.split_toning_shadow_saturation,
                    &mut p.color_grade_enable_shadows,
                );
                changed |= draw_grade_wheel(
                    ui,
                    "Midtones",
                    r,
                    &mut p.color_grade_midtone_hue,
                    &mut p.color_grade_midtone_sat,
                    &mut p.color_grade_enable_midtones,
                );
                changed |= draw_grade_wheel(
                    ui,
                    "Highlights",
                    r,
                    &mut p.split_toning_highlight_hue,
                    &mut p.split_toning_highlight_saturation,
                    &mut p.color_grade_enable_highlights,
                );
            });
            changed |= lr_slider(ui, "Shadow Lum", &mut p.color_grade_shadow_lum, -100.0..=100.0, 0.0, "{:+.0}");
            changed |= lr_slider(ui, "Midtone Lum", &mut p.color_grade_midtone_lum, -100.0..=100.0, 0.0, "{:+.0}");
            changed |= lr_slider(ui, "High. Lum", &mut p.color_grade_highlight_lum, -100.0..=100.0, 0.0, "{:+.0}");
        } else {
            let (lbl, hue, sat, lum, en) = match app.color_grade_view_tab {
                1 => (
                    "Shadows",
                    &mut p.split_toning_shadow_hue,
                    &mut p.split_toning_shadow_saturation,
                    &mut p.color_grade_shadow_lum,
                    &mut p.color_grade_enable_shadows,
                ),
                2 => (
                    "Midtones",
                    &mut p.color_grade_midtone_hue,
                    &mut p.color_grade_midtone_sat,
                    &mut p.color_grade_midtone_lum,
                    &mut p.color_grade_enable_midtones,
                ),
                3 => (
                    "Highlights",
                    &mut p.split_toning_highlight_hue,
                    &mut p.split_toning_highlight_saturation,
                    &mut p.color_grade_highlight_lum,
                    &mut p.color_grade_enable_highlights,
                ),
                _ => (
                    "Global",
                    &mut p.color_grade_global_hue,
                    &mut p.color_grade_global_sat,
                    &mut p.color_grade_global_lum,
                    &mut p.color_grade_enable_global,
                ),
            };
            ui.vertical_centered(|ui| {
                changed |= draw_grade_wheel(ui, lbl, 64.0, hue, sat, en);
            });
            changed |= lr_slider(ui, "Hue", hue, 0.0..=360.0, 0.0, "{:.0}");
            changed |= lr_slider(ui, "Saturation", sat, 0.0..=100.0, 0.0, "{:.0}");
            changed |= lr_slider(ui, "Luminance", lum, -100.0..=100.0, 0.0, "{:+.0}");
        }

        ui.separator();
        changed |= lr_slider(ui, "Blending", &mut p.color_grade_blending, 0.0..=100.0, 50.0, "{:.0}");
        changed |= lr_slider(ui, "Balance", &mut p.split_toning_balance, -100.0..=100.0, 0.0, "{:+.0}");

        if changed {
            app.commit_params_change("Color Grading", p);
        }
    });
    app.dev_color_grading_open = next_open;
}

fn draw_grade_wheel(
    ui: &mut Ui,
    label: &str,
    radius: f32,
    hue: &mut f32,
    sat: &mut f32,
    enabled: &mut bool,
) -> bool {
    let mut changed = false;
    let r = radius.max(28.0);
    ui.vertical_centered(|ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(label).size(10.0).color(Colors::TEXT_MID));
            if ui
                .selectable_label(*enabled, "👁")
                .on_hover_text("Toggle wheel")
                .clicked()
            {
                *enabled = !*enabled;
                changed = true;
            }
        });
        let (rect, resp) = ui.allocate_exact_size(Vec2::splat(r * 2.0 + 4.0), Sense::click_and_drag());
        let c = rect.center();
        for ring in (1..=4).rev() {
            let rf = ring as f32 / 4.0;
            for seg in 0..24 {
                let a0 = (seg as f32 / 24.0) * std::f32::consts::TAU;
                let pt = c + Vec2::new(a0.cos(), a0.sin()) * (r * rf * 0.85);
                let col = Color32::from(egui::epaint::Hsva::new(
                    seg as f32 / 24.0,
                    rf * 0.75,
                    if *enabled { 0.82 } else { 0.35 },
                    0.55,
                ));
                ui.painter().circle_filled(pt, (r * 0.24).max(4.0), col);
            }
        }
        ui.painter()
            .circle_stroke(c, r, Stroke::new(1.2, Color32::from_rgb(75, 75, 82)));

        if resp.double_clicked() {
            *sat = 0.0;
            changed = true;
        } else if (resp.dragged() || resp.clicked()) && ui.input(|i| i.pointer.primary_down()) {
            if let Some(pos) = resp.interact_pointer_pos() {
                let d = pos - c;
                let mods = ui.input(|i| i.modifiers);
                if !mods.shift {
                    *hue = d.y.atan2(d.x).to_degrees().rem_euclid(360.0);
                }
                if !mods.command {
                    *sat = ((d.length() / r) * 100.0).clamp(0.0, 100.0);
                }
                changed = true;
            }
        }

        let rad = hue.to_radians();
        let handle = c + Vec2::new(rad.cos(), rad.sin()) * ((*sat / 100.0) * r);
        ui.painter().line_segment([c, handle], Stroke::new(1.0, Color32::WHITE));
        ui.painter().circle_filled(handle, 4.5, Color32::WHITE);
        ui.painter()
            .circle_stroke(handle, 4.5, Stroke::new(1.0, Color32::BLACK));
    });
    changed
}

fn render_panel_detail(app: &mut LightRawApp, ctx: &egui::Context, ui: &mut Ui) {
    let is_mod = app.active_photo().params.is_detail_modified();
    let open = app.dev_detail_open;
    let next_open = panel_wrap(app, ui, "Detail", open, is_mod, None, |app, ui| {
        let mut p = app.active_photo().params.clone();
        let mut changed = false;

        // Denoise section at top (matching Lightroom Classic)
        ui.horizontal(|ui| {
            let mut denoise_on = p.ai_denoise_amount > 0.0;
            if ui
                .checkbox(&mut denoise_on, RichText::new("Denoise (AI)").size(10.5).color(Colors::TEXT_MAIN))
                .changed()
            {
                p.ai_denoise_amount = if denoise_on { 50.0 } else { 0.0 };
                changed = true;
            }
            ui.label(
                RichText::new("⚡ ANE")
                    .size(9.0)
                    .color(Color32::from_rgb(0, 200, 80)),
            )
            .on_hover_text("Local Apple Neural Engine / SIMD Hardware Accelerated");
            if denoise_on {
                changed |= lr_slider(ui, "", &mut p.ai_denoise_amount, 0.0..=100.0, 50.0, "{:.0}");
            }
        });
        ui.horizontal(|ui| {
            changed |= ui.checkbox(&mut p.raw_details_enabled, RichText::new("Raw Details").size(10.5).color(Colors::TEXT_MAIN)).changed();
        });
        ui.horizontal(|ui| {
            changed |= ui.checkbox(&mut p.super_resolution_enabled, RichText::new("Super Resolution").size(10.5).color(Colors::TEXT_MAIN)).changed();
        });
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // 1:1 Detail Loupe Preview Window (Centered Square, Exact 1:1 Pixel Aspect Ratio)
        let pid = app.active_photo_id;
        let is_crop = app.develop_tool == DevelopTool::Crop;
        let apply_crop = !is_crop;
        if let Some(tex) = app.get_photo_texture(ctx, pid, apply_crop) {
            let tex_size = tex.size_vec2();
            let side = 164.0_f32; // Lightroom Classic centered square loupe

            ui.horizontal(|ui| {
                // Left sample crosshair icon
                let (pick_r, pick_resp) = ui.allocate_exact_size(Vec2::new(20.0, 20.0), Sense::click());
                let pick_resp = pick_resp.on_hover_text("Click to sample a region for 1:1 detail inspection");
                if pick_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                let is_picking = app.detail_sample_tool_active;
                if pick_resp.clicked() {
                    app.detail_sample_tool_active = !app.detail_sample_tool_active;
                }
                let pick_bg = if is_picking {
                    Color32::from_rgb(0, 122, 255)
                } else if pick_resp.hovered() {
                    Color32::from_rgb(52, 52, 58)
                } else {
                    Color32::TRANSPARENT
                };
                ui.painter().rect_filled(pick_r, 3.0, pick_bg);
                ui.painter().text(
                    pick_r.center(),
                    egui::Align2::CENTER_CENTER,
                    "⌖",
                    egui::FontId::proportional(14.0),
                    if is_picking { Color32::WHITE } else { Colors::TEXT_MID },
                );

                // Center pad
                let avail_w = ui.available_width() - 28.0;
                let pad = ((avail_w - side) * 0.5).max(0.0);
                ui.add_space(pad);

                let (l_rect, l_resp) = ui.allocate_exact_size(Vec2::splat(side), Sense::click_and_drag());
                if l_resp.hovered() {
                    if l_resp.is_pointer_button_down_on() || l_resp.dragged() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                    } else {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                    }
                }
                if (l_resp.dragged() || l_resp.clicked()) && ui.input(|i| i.pointer.primary_down()) {
                    let delta = l_resp.drag_delta();
                    // True 1:1 pixel panning matching mouse drag exactly!
                    app.detail_loupe_center.x =
                        (app.detail_loupe_center.x - delta.x / tex_size.x.max(1.0)).clamp(0.01, 0.99);
                    app.detail_loupe_center.y =
                        (app.detail_loupe_center.y - delta.y / tex_size.y.max(1.0)).clamp(0.01, 0.99);
                }

                // Mathematical 1:1 Square Pixel Aspect Ratio (Zero distortion!)
                let du = (side / tex_size.x.max(1.0)).min(1.0);
                let dv = (side / tex_size.y.max(1.0)).min(1.0);
                let u_min = (app.detail_loupe_center.x - du * 0.5).clamp(0.0, 1.0 - du);
                let v_min = (app.detail_loupe_center.y - dv * 0.5).clamp(0.0, 1.0 - dv);
                let uv_rect = Rect::from_min_max(Pos2::new(u_min, v_min), Pos2::new(u_min + du, v_min + dv));

                ui.painter().image(
                    tex.id(),
                    l_rect,
                    uv_rect,
                    Color32::WHITE,
                );
                ui.painter().rect_stroke(l_rect, 0.0, Stroke::new(1.0, Color32::from_rgb(18, 18, 22)));

                // Subtle 1:1 badge on top
                ui.painter().text(
                    l_rect.min + Vec2::new(5.0, 4.0),
                    egui::Align2::LEFT_TOP,
                    "1:1",
                    egui::FontId::proportional(9.0),
                    Color32::from_rgba_unmultiplied(255, 255, 255, 175),
                );

                // Right collapse arrow
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(4.0);
                    ui.label(RichText::new("▼").size(10.0).color(Colors::TEXT_DIM));
                });
            });
        }

        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Sharpening").size(11.0).strong().color(Colors::TEXT_MID));
        });
        ui.add_space(2.0);
        changed |= lr_slider(ui, "Amount", &mut p.sharpness, 0.0..=150.0, 40.0, "{:.0}");
        changed |= lr_slider(ui, "Radius", &mut p.sharpen_radius, 0.5..=3.0, 1.0, "{:.1}");
        changed |= lr_slider(ui, "Detail", &mut p.sharpen_detail, 0.0..=100.0, 25.0, "{:.0}");
        if lr_slider(ui, "Masking", &mut p.sharpen_edge_masking, 0.0..=100.0, 0.0, "{:.0}") {
            changed = true;
            if ui.input(|i| i.modifiers.alt) {
                app.sharpen_mask_preview_active = true;
            }
        }

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // Manual Noise Reduction (Clean dark section header, zero blue highlight)
        let (h_rect, h_resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 22.0), Sense::click());
        if h_resp.clicked() {
            app.detail_nr_expanded = !app.detail_nr_expanded;
        }
        let h_bg = if h_resp.hovered() {
            Color32::from_rgb(46, 46, 52)
        } else {
            Color32::from_rgb(36, 36, 40)
        };
        ui.painter().rect_filled(h_rect, 2.0, h_bg);
        ui.painter().text(
            h_rect.min + Vec2::new(8.0, 11.0),
            egui::Align2::LEFT_CENTER,
            "Manual Noise Reduction",
            egui::FontId::proportional(11.0),
            Colors::TEXT_MAIN,
        );
        let arrow = if app.detail_nr_expanded { "▼" } else { "◀" };
        ui.painter().text(
            Pos2::new(h_rect.max.x - 8.0, h_rect.center().y),
            egui::Align2::RIGHT_CENTER,
            arrow,
            egui::FontId::proportional(10.0),
            Colors::TEXT_DIM,
        );

        if app.detail_nr_expanded {
            ui.add_space(4.0);
            changed |= lr_slider(
                ui,
                "Luminance",
                &mut p.luminance_smoothing,
                0.0..=100.0,
                0.0,
                "{:.0}",
            );
            changed |= lr_slider(
                ui,
                "Lum Detail",
                &mut p.luminance_noise_reduction_detail,
                0.0..=100.0,
                50.0,
                "{:.0}",
            );
            changed |= lr_slider(
                ui,
                "Contrast",
                &mut p.luminance_noise_reduction_contrast,
                0.0..=100.0,
                0.0,
                "{:.0}",
            );
            changed |= lr_slider(
                ui,
                "Color",
                &mut p.color_noise_reduction,
                0.0..=100.0,
                25.0,
                "{:.0}",
            );
            changed |= lr_slider(
                ui,
                "Col Detail",
                &mut p.color_noise_reduction_detail,
                0.0..=100.0,
                50.0,
                "{:.0}",
            );
            changed |= lr_slider(
                ui,
                "Smoothness",
                &mut p.color_noise_reduction_smoothness,
                0.0..=100.0,
                50.0,
                "{:.0}",
            );
        }

        if changed {
            app.commit_params_change("Detail & Noise Reduction", p);
        }
    });
    app.dev_detail_open = next_open;
}

fn render_panel_lens_corrections(app: &mut LightRawApp, ui: &mut Ui) {
    let is_mod = app.active_photo().params.is_lens_corrections_modified();
    let open = app.dev_lens_open;
    let next_open = panel_wrap(app, ui, "Lens Corrections", open, is_mod, None, |app, ui| {
        let mut p = app.active_photo().params.clone();
        let mut changed = false;

        ui.horizontal(|ui| {
            if ui
                .selectable_label(!app.lens_corrections_manual_tab, "Profile")
                .clicked()
            {
                app.lens_corrections_manual_tab = false;
            }
            if ui
                .selectable_label(app.lens_corrections_manual_tab, "Manual")
                .clicked()
            {
                app.lens_corrections_manual_tab = true;
            }
        });
        ui.separator();

        if !app.lens_corrections_manual_tab {
            changed |= ui
                .checkbox(&mut p.auto_lateral_ca, "Remove Chromatic Aberration")
                .changed();
            changed |= ui
                .checkbox(&mut p.lens_profile_enable, "Enable Profile Corrections")
                .changed();
            if p.lens_profile_enable {
                let lens_lbl = app.active_photo().lens_model.clone();
                ui.label(
                    RichText::new(format!("Lens Profile: {}", lens_lbl))
                        .size(10.5)
                        .color(Colors::TEXT_MID),
                );
                changed |= lr_slider(
                    ui,
                    "Distortion",
                    &mut p.lens_profile_distortion_scale,
                    0.0..=200.0,
                    100.0,
                    "{:.0}",
                );
                changed |= lr_slider(
                    ui,
                    "Vignetting",
                    &mut p.lens_profile_vignetting_scale,
                    0.0..=200.0,
                    100.0,
                    "{:.0}",
                );
            }
        } else {
            ui.label(RichText::new("Distortion").size(10.5).strong().color(Colors::TEXT_MID));
            changed |= lr_slider(
                ui,
                "Distortion",
                &mut p.lens_manual_distortion_amount,
                -100.0..=100.0,
                0.0,
                "{:+.0}",
            );

            ui.separator();
            ui.horizontal(|ui| {
                ui.label(RichText::new("Defringe").size(10.5).strong().color(Colors::TEXT_MID));
                if ui
                    .selectable_label(app.defringe_eyedropper_active == Some(true), "🖌 Purple")
                    .clicked()
                {
                    app.defringe_eyedropper_active = if app.defringe_eyedropper_active == Some(true) {
                        None
                    } else {
                        Some(true)
                    };
                }
                if ui
                    .selectable_label(app.defringe_eyedropper_active == Some(false), "🖌 Green")
                    .clicked()
                {
                    app.defringe_eyedropper_active = if app.defringe_eyedropper_active == Some(false) {
                        None
                    } else {
                        Some(false)
                    };
                }
            });
            changed |= lr_slider(
                ui,
                "Purple Amt",
                &mut p.defringe_purple_amount,
                0.0..=20.0,
                0.0,
                "{:.0}",
            );
            changed |= lr_slider(
                ui,
                "Purple Lo",
                &mut p.defringe_purple_hue_lo,
                0.0..=360.0,
                30.0,
                "{:.0}",
            );
            changed |= lr_slider(
                ui,
                "Purple Hi",
                &mut p.defringe_purple_hue_hi,
                0.0..=360.0,
                70.0,
                "{:.0}",
            );
            changed |= lr_slider(
                ui,
                "Green Amt",
                &mut p.defringe_green_amount,
                0.0..=20.0,
                0.0,
                "{:.0}",
            );
            changed |= lr_slider(
                ui,
                "Green Lo",
                &mut p.defringe_green_hue_lo,
                0.0..=360.0,
                40.0,
                "{:.0}",
            );
            changed |= lr_slider(
                ui,
                "Green Hi",
                &mut p.defringe_green_hue_hi,
                0.0..=360.0,
                60.0,
                "{:.0}",
            );

            ui.separator();
            ui.label(RichText::new("Vignetting").size(10.5).strong().color(Colors::TEXT_MID));
            changed |= lr_slider(ui, "Amount", &mut p.vignette_amount, -100.0..=100.0, 0.0, "{:+.0}");
            changed |= lr_slider(ui, "Midpoint", &mut p.vignette_midpoint, 0.0..=100.0, 50.0, "{:.0}");
        }

        if changed {
            app.commit_params_change("Lens Corrections", p);
        }
    });
    app.dev_lens_open = next_open;
}

fn render_panel_transform(app: &mut LightRawApp, ui: &mut Ui) {
    let is_mod = app.active_photo().params.is_transform_modified();
    let open = app.dev_transform_open;
    let next_open = panel_wrap(app, ui, "Transform", open, is_mod, None, |app, ui| {
        let mut p = app.active_photo().params.clone();
        let mut changed = false;

        ui.horizontal_wrapped(|ui| {
            for (mode, lbl) in [
                (UprightMode::Off, "Off"),
                (UprightMode::Auto, "Auto"),
                (UprightMode::Level, "Level"),
                (UprightMode::Vertical, "Vertical"),
                (UprightMode::Full, "Full"),
                (UprightMode::Guided, "Guided"),
            ] {
                if ui
                    .selectable_label(p.perspective_upright == mode, lbl)
                    .clicked()
                {
                    p.perspective_upright = mode;
                    if mode == UprightMode::Guided {
                        app.develop_tool = DevelopTool::GuidedUpright;
                    } else if app.develop_tool == DevelopTool::GuidedUpright {
                        app.develop_tool = DevelopTool::None;
                    }
                    changed = true;
                }
            }
            if ui
                .small_button("Update")
                .on_hover_text("Re-analyze Upright geometry")
                .clicked()
            {
                if p.perspective_upright == UprightMode::Guided && p.guided_upright_lines.len() >= 2 {
                    // Solve rotation/vertical alignment from drawn guide lines!
                    let mut rot_sum = 0.0_f32;
                    let mut rot_n = 0usize;
                    for &([x1, y1], [x2, y2]) in &p.guided_upright_lines {
                        let dx = x2 - x1;
                        let dy = y2 - y1;
                        if dx.abs() > dy.abs() {
                            rot_sum += -(dy.atan2(dx).to_degrees());
                            rot_n += 1;
                        }
                    }
                    if rot_n > 0 {
                        p.perspective_rotate = (rot_sum / rot_n as f32).clamp(-10.0, 10.0);
                    }
                }
                changed = true;
            }
        });
        ui.separator();

        changed |= lr_slider(ui, "Vertical", &mut p.perspective_vertical, -100.0..=100.0, 0.0, "{:+.0}");
        changed |= lr_slider(ui, "Horizontal", &mut p.perspective_horizontal, -100.0..=100.0, 0.0, "{:+.0}");
        changed |= lr_slider(ui, "Rotate", &mut p.perspective_rotate, -10.0..=10.0, 0.0, "{:+.1}°");
        changed |= lr_slider(ui, "Aspect", &mut p.perspective_aspect, -100.0..=100.0, 0.0, "{:+.0}");
        changed |= lr_slider(ui, "Scale", &mut p.perspective_scale, 50.0..=150.0, 100.0, "{:.0}");
        changed |= lr_slider(ui, "X Offset", &mut p.perspective_x, -100.0..=100.0, 0.0, "{:+.1}");
        changed |= lr_slider(ui, "Y Offset", &mut p.perspective_y, -100.0..=100.0, 0.0, "{:+.1}");
        changed |= ui
            .checkbox(&mut p.crop_constrain_to_warp, "Constrain Crop")
            .changed();

        if changed {
            app.commit_params_change("Transform", p);
        }
    });
    app.dev_transform_open = next_open;
}

fn render_panel_lens_blur(app: &mut LightRawApp, ui: &mut Ui) {
    let is_mod = app.active_photo().params.is_lens_blur_modified();
    let open = app.dev_lens_blur_open;
    let next_open = panel_wrap(
        app,
        ui,
        "Lens Blur",
        open,
        is_mod,
        Some("DEPTH AI"),
        |app, ui| {
            let mut p = app.active_photo().params.clone();
            let mut changed = false;

            ui.horizontal(|ui| {
                if ui.checkbox(&mut p.lens_blur_active, "Apply").changed() {
                    changed = true;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .selectable_label(app.lens_blur_focus_picker_active, "🎯 Point Focus")
                        .on_hover_text("Click subject in photo to set focal plane")
                        .clicked()
                    {
                        app.lens_blur_focus_picker_active = !app.lens_blur_focus_picker_active;
                    }
                });
            });

            ui.add_enabled_ui(p.lens_blur_active, |ui| {
                changed |= lr_slider(
                    ui,
                    "Blur Amount",
                    &mut p.lens_blur_amount,
                    0.0..=100.0,
                    50.0,
                    "{:.0}",
                );

                ui.label(RichText::new("Bokeh").size(10.5).strong().color(Colors::TEXT_MID));
                ui.horizontal_wrapped(|ui| {
                    for (shape, sym, tip) in [
                        (BokehShape::Circle, "●", "Modern Circular"),
                        (BokehShape::SoapBubble, "◯", "Soap Bubble"),
                        (BokehShape::Blade, "⬠", "5-Blade Vintage"),
                        (BokehShape::Ring, "◎", "Mirror / Reflex Ring"),
                        (BokehShape::CatEye, "◖", "Optical Vignette Cat Eye"),
                        (BokehShape::Anamorphic, "⬭", "2x Anamorphic Oval"),
                    ] {
                        if ui
                            .selectable_label(p.lens_blur_bokeh == shape, sym)
                            .on_hover_text(tip)
                            .clicked()
                        {
                            p.lens_blur_bokeh = shape;
                            changed = true;
                        }
                    }
                });

                changed |= lr_slider(
                    ui,
                    "Boost",
                    &mut p.lens_blur_highlights_boost,
                    0.0..=100.0,
                    50.0,
                    "{:.0}",
                );
                changed |= lr_slider(
                    ui,
                    "Cat Eye",
                    &mut p.lens_blur_cat_eye_amount,
                    0.0..=100.0,
                    0.0,
                    "{:.0}",
                );

                // Interactive Focal Range Bar (Near <-> Far)
                ui.label(RichText::new("Focal Range").size(10.5).strong().color(Colors::TEXT_MID));
                let (bar_rect, bar_resp) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), 22.0),
                    Sense::click_and_drag(),
                );
                ui.painter()
                    .rect_filled(bar_rect, 3.0, Color32::from_rgb(28, 28, 32));
                let x_lo = bar_rect.min.x + (p.lens_blur_focal_range[1] / 100.0) * bar_rect.width();
                let x_hi = bar_rect.min.x + (p.lens_blur_focal_range[2] / 100.0) * bar_rect.width();
                let active_zone = Rect::from_min_max(
                    Pos2::new(x_lo, bar_rect.min.y + 3.0),
                    Pos2::new(x_hi.max(x_lo + 4.0), bar_rect.max.y - 3.0),
                );
                ui.painter()
                    .rect_filled(active_zone, 2.0, Color32::from_rgb(225, 180, 65));
                if (bar_resp.dragged() || bar_resp.clicked()) && ui.input(|i| i.pointer.primary_down()) {
                    if let Some(pos) = bar_resp.interact_pointer_pos() {
                        let pct = ((pos.x - bar_rect.min.x) / bar_rect.width() * 100.0).clamp(0.0, 100.0);
                        if (pos.x - x_lo).abs() <= (pos.x - x_hi).abs() {
                            let new_min = pct.min(p.lens_blur_focal_range[2] - 2.0);
                            p.lens_blur_focal_range[1] = new_min;
                            p.lens_blur_focal_range[0] = (new_min - 10.0).max(0.0);
                        } else {
                            let new_max = pct.max(p.lens_blur_focal_range[1] + 2.0);
                            p.lens_blur_focal_range[2] = new_max;
                            p.lens_blur_focal_range[3] = (new_max + 15.0).min(100.0);
                        }
                        changed = true;
                    }
                }

                // Refinement Focus / Blur Brushes
                let ref_arrow = if app.lens_blur_refinement_expanded { "▼" } else { "▶" };
                if ui
                    .selectable_label(
                        app.lens_blur_refinement_expanded,
                        format!("{} Refinement", ref_arrow),
                    )
                    .clicked()
                {
                    app.lens_blur_refinement_expanded = !app.lens_blur_refinement_expanded;
                }
                if app.lens_blur_refinement_expanded {
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(app.lens_blur_brush_mode == Some(true), "🖌 Focus")
                            .clicked()
                        {
                            app.lens_blur_brush_mode = if app.lens_blur_brush_mode == Some(true) {
                                None
                            } else {
                                Some(true)
                            };
                        }
                        if ui
                            .selectable_label(app.lens_blur_brush_mode == Some(false), "🖌 Blur")
                            .clicked()
                        {
                            app.lens_blur_brush_mode = if app.lens_blur_brush_mode == Some(false) {
                                None
                            } else {
                                Some(false)
                            };
                        }
                        if !p.lens_blur_refinement_strokes.is_empty() && ui.small_button("Reset").clicked() {
                            p.lens_blur_refinement_strokes.clear();
                            changed = true;
                        }
                    });
                    let mut bsz = app.lens_blur_brush_size * 500.0;
                    if lr_slider(ui, "Brush Size", &mut bsz, 5.0..=150.0, 30.0, "{:.0}") {
                        app.lens_blur_brush_size = bsz / 500.0;
                    }
                }
            });

            if changed {
                app.commit_params_change("Lens Blur", p);
            }
        },
    );
    app.dev_lens_blur_open = next_open;
}

fn render_panel_effects(app: &mut LightRawApp, ui: &mut Ui) {
    let is_mod = app.active_photo().params.is_effects_modified();
    let open = app.dev_effects_open;
    let next_open = panel_wrap(app, ui, "Effects", open, is_mod, None, |app, ui| {
        let mut p = app.active_photo().params.clone();
        let mut changed = false;

        ui.label(
            RichText::new("Post-Crop Vignetting")
                .size(10.5)
                .strong()
                .color(Colors::TEXT_MID),
        );
        ui.horizontal(|ui| {
            ui.label(RichText::new("Style:").size(11.0));
            egui::ComboBox::from_id_salt("pcv_style_cb")
                .selected_text(p.post_crop_vignette_style.label())
                .width(145.0)
                .show_ui(ui, |ui| {
                    for &st in VignetteStyle::ALL {
                        if ui
                            .selectable_label(p.post_crop_vignette_style == st, st.label())
                            .clicked()
                        {
                            p.post_crop_vignette_style = st;
                            changed = true;
                        }
                    }
                });
        });
        changed |= lr_slider(
            ui,
            "Amount",
            &mut p.post_crop_vignette_amount,
            -100.0..=100.0,
            0.0,
            "{:+.0}",
        );
        if p.post_crop_vignette_amount.abs() > 0.1 {
            changed |= lr_slider(
                ui,
                "Midpoint",
                &mut p.post_crop_vignette_midpoint,
                0.0..=100.0,
                50.0,
                "{:.0}",
            );
            changed |= lr_slider(
                ui,
                "Roundness",
                &mut p.post_crop_vignette_roundness,
                -100.0..=100.0,
                0.0,
                "{:+.0}",
            );
            changed |= lr_slider(
                ui,
                "Feather",
                &mut p.post_crop_vignette_feather,
                0.0..=100.0,
                50.0,
                "{:.0}",
            );
            if p.post_crop_vignette_style != VignetteStyle::PaintOverlay && p.post_crop_vignette_amount < 0.0 {
                changed |= lr_slider(
                    ui,
                    "Highlights",
                    &mut p.post_crop_vignette_highlight_contrast,
                    0.0..=100.0,
                    0.0,
                    "{:.0}",
                );
            }
        }

        ui.separator();
        ui.label(RichText::new("Grain").size(10.5).strong().color(Colors::TEXT_MID));
        changed |= lr_slider(ui, "Amount", &mut p.grain_amount, 0.0..=100.0, 0.0, "{:.0}");
        if p.grain_amount > 0.1 {
            changed |= lr_slider(ui, "Size", &mut p.grain_size, 0.0..=100.0, 25.0, "{:.0}");
            changed |= lr_slider(
                ui,
                "Roughness",
                &mut p.grain_frequency,
                0.0..=100.0,
                50.0,
                "{:.0}",
            );
        }

        if changed {
            app.commit_params_change("Effects", p);
        }
    });
    app.dev_effects_open = next_open;
}

fn render_panel_calibration(app: &mut LightRawApp, ui: &mut Ui) {
    let is_mod = app.active_photo().params.is_calibration_modified();
    let open = app.dev_calibration_open;
    let next_open = panel_wrap(app, ui, "Calibration", open, is_mod, None, |app, ui| {
        let mut p = app.active_photo().params.clone();
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label(RichText::new("Process :").size(11.0));
            egui::ComboBox::from_id_salt("cal_pv_cb")
                .selected_text(&p.process_version)
                .width(155.0)
                .show_ui(ui, |ui| {
                    for pv in ["Version 6 (Current)", "Version 5", "Version 4"] {
                        if ui.selectable_label(p.process_version == pv, pv).clicked() {
                            p.process_version = pv.to_string();
                            changed = true;
                        }
                    }
                });
        });

        ui.add_space(4.0);
        ui.label(RichText::new("Shadows").size(10.5).strong().color(Colors::TEXT_MID));
        changed |= lr_gradient_slider(
            ui,
            "Tint",
            &mut p.shadow_tint,
            -100.0..=100.0,
            0.0,
            Color32::from_rgb(45, 185, 85),
            Color32::from_rgb(195, 65, 185),
            "{:+.0}",
        );

        ui.label(RichText::new("Red Primary").size(10.5).strong().color(Colors::TEXT_MID));
        changed |= lr_slider(ui, "Red Hue", &mut p.red_hue, -100.0..=100.0, 0.0, "{:+.0}");
        changed |= lr_slider(ui, "Red Sat", &mut p.red_saturation, -100.0..=100.0, 0.0, "{:+.0}");

        ui.label(RichText::new("Green Primary").size(10.5).strong().color(Colors::TEXT_MID));
        changed |= lr_slider(ui, "Green Hue", &mut p.green_hue, -100.0..=100.0, 0.0, "{:+.0}");
        changed |= lr_slider(
            ui,
            "Green Sat",
            &mut p.green_saturation,
            -100.0..=100.0,
            0.0,
            "{:+.0}",
        );

        ui.label(RichText::new("Blue Primary").size(10.5).strong().color(Colors::TEXT_MID));
        changed |= lr_slider(ui, "Blue Hue", &mut p.blue_hue, -100.0..=100.0, 0.0, "{:+.0}");
        changed |= lr_slider(ui, "Blue Sat", &mut p.blue_saturation, -100.0..=100.0, 0.0, "{:+.0}");

        if changed {
            app.commit_params_change("Calibration", p);
        }
    });
    app.dev_calibration_open = next_open;
}
