// ============================================================================
// LightRaw — Develop Interactive Canvas, Tool Drawers & Before/After Views
// ============================================================================

use super::{
    lr_bevel_button, lr_slider, BeforeAfterMode, CanvasSurroundMode, Colors, DevelopTool,
    LightRawApp, TargetAdjustKind,
};
use crate::params::{
    AspectRatioPreset, CropGuideOverlay, LandscapePart, LensBlurStroke, LocalAdjustParams,
    MaskComponent, MaskGroup, MaskOp, MaskPrimitiveKind, PersonPart, PointColorSwatch, RedEyeMode,
    RedEyeSpot, RetouchArea, RetouchMode, ToolOverlayMode, WhiteBalanceMode, HSL_BAND_HUES,
};
use eframe::egui::{self, Color32, Pos2, Rect, RichText, Sense, Stroke, Ui, Vec2};

/// Solves approximate Color Temperature (K) and Tint from a sampled linear RGB neutral patch.
fn solve_wb_from_linear_rgb(r: f32, g: f32, b: f32, as_shot_temp: f32, as_shot_tint: f32) -> (f32, f32) {
    let rg = (r.max(1e-4) / g.max(1e-4)).clamp(0.25, 4.0);
    let bg = (b.max(1e-4) / g.max(1e-4)).clamp(0.25, 4.0);
    // Warm patch (r > b) requires warmer Temperature slider in ACR to neutralize a cool cast,
    // wait: in Lightroom, clicking a yellowish patch (high R, low B) COOLS the image (lowers Temp),
    // while clicking a bluish patch (low R, high B) WARMS the image (raises Temp)!
    // Wait: let's verify! If a white shirt looks too blue (B > R), we want to warm the image (raise Temp > as_shot_temp).
    // Yes! `bg / rg > 1.0` -> raise Temperature; `rg / bg > 1.0` -> lower Temperature!
    let rb_ratio = (bg / rg).ln();
    let new_temp = (as_shot_temp * (1.0 + rb_ratio * 0.42)).clamp(2000.0, 50000.0);
    // Green/Magenta tint: if G is higher than average of R & B (green cast), add +Magenta (+Tint)!
    let rb_avg = (r.max(1e-4) * b.max(1e-4)).sqrt();
    let gm_ratio = (g.max(1e-4) / rb_avg).ln();
    let new_tint = (as_shot_tint + gm_ratio * 65.0).clamp(-150.0, 150.0);
    (new_temp, new_tint)
}

fn rgb_to_hsl_deg(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max_c = r.max(g).max(b);
    let min_c = r.min(g).min(b);
    let l = 0.5 * (max_c + min_c);
    let d = max_c - min_c;
    if d < 1e-5 {
        return (0.0, 0.0, l);
    }
    let s = if l > 0.5 {
        d / (2.0 - max_c - min_c).max(1e-5)
    } else {
        d / (max_c + min_c).max(1e-5)
    };
    let h = if (max_c - r).abs() < 1e-5 {
        60.0 * (((g - b) / d).rem_euclid(6.0))
    } else if (max_c - g).abs() < 1e-5 {
        60.0 * (((b - r) / d) + 2.0)
    } else {
        60.0 * (((r - g) / d) + 4.0)
    };
    (h.rem_euclid(360.0), s.clamp(0.0, 1.0), l.clamp(0.0, 1.0))
}

pub fn render_soft_proofing_top_bar(app: &mut LightRawApp, ui: &mut Ui) {
    if !app.soft_proofing {
        return;
    }
    egui::Frame::none()
        .fill(Color32::from_rgb(238, 238, 242))
        .inner_margin(egui::Margin::symmetric(12.0, 4.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("PROOF PREVIEW")
                        .size(10.5)
                        .strong()
                        .color(Color32::from_rgb(30, 30, 35)),
                );
                ui.separator();
                ui.label(RichText::new("Profile:").size(11.0).color(Color32::from_rgb(45, 45, 50)));
                let prof_lbl = match app.soft_proof_profile {
                    1 => "Display P3",
                    2 => "Adobe RGB (1998)",
                    _ => "sRGB IEC61966-2.1",
                };
                egui::ComboBox::from_id_salt("soft_proof_profile_cb")
                    .selected_text(prof_lbl)
                    .width(145.0)
                    .show_ui(ui, |ui| {
                        for (idx, lbl) in [
                            (0u8, "sRGB IEC61966-2.1"),
                            (1u8, "Display P3"),
                            (2u8, "Adobe RGB (1998)"),
                        ] {
                            if ui
                                .selectable_label(app.soft_proof_profile == idx, lbl)
                                .clicked()
                            {
                                app.soft_proof_profile = idx;
                            }
                        }
                    });

                ui.add_space(6.0);
                ui.label(RichText::new("Intent:").size(11.0).color(Color32::from_rgb(45, 45, 50)));
                if ui
                    .selectable_label(app.soft_proof_perceptual, "Perceptual")
                    .clicked()
                {
                    app.soft_proof_perceptual = true;
                }
                if ui
                    .selectable_label(!app.soft_proof_perceptual, "Relative")
                    .clicked()
                {
                    app.soft_proof_perceptual = false;
                }

                ui.add_space(8.0);
                ui.checkbox(
                    &mut app.simulate_paper_ink,
                    RichText::new("Simulate Paper & Ink")
                        .size(11.0)
                        .color(Color32::from_rgb(35, 35, 40)),
                );
                ui.checkbox(
                    &mut app.gamut_warning,
                    RichText::new("Gamut Warning")
                        .size(11.0)
                        .color(Color32::from_rgb(35, 35, 40)),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Create Proof Copy").clicked() {
                        app.create_virtual_copy();
                    }
                });
            });
        });
}

pub fn render_active_tool_drawer(app: &mut LightRawApp, ui: &mut Ui) {
    match app.develop_tool {
        DevelopTool::None => {}
        DevelopTool::Crop => render_crop_drawer(app, ui),
        DevelopTool::Remove => render_remove_drawer(app, ui),
        DevelopTool::RedEye => render_redeye_drawer(app, ui),
        DevelopTool::Masking => render_masking_drawer(app, ui),
        DevelopTool::GuidedUpright => {
            egui::Frame::none()
                .fill(Color32::from_rgb(44, 44, 48))
                .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                .show(ui, |ui| {
                    let count = app.active_photo().params.guided_upright_lines.len();
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("Guided Upright ({}/4 Guides)", count))
                                .strong()
                                .size(11.5),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("Done").clicked() {
                                app.develop_tool = DevelopTool::None;
                            }
                            if count > 0 && ui.small_button("Clear Guides").clicked() {
                                let mut p = app.active_photo().params.clone();
                                p.guided_upright_lines.clear();
                                app.commit_params_change("Clear Upright Guides", p);
                            }
                        });
                    });
                    ui.label(
                        RichText::new(
                            "Drag 2 to 4 lines along vertical or horizontal architectural edges on the photo.",
                        )
                        .size(10.5)
                        .color(Colors::TEXT_MID),
                    );
                });
        }
        DevelopTool::TargetAdjust => {
            egui::Frame::none()
                .fill(Color32::from_rgb(44, 44, 48))
                .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Target Adjustment:").strong().size(11.0));
                        for (kind, lbl) in [
                            (TargetAdjustKind::ParametricCurve, "Tone Curve"),
                            (TargetAdjustKind::Hue, "Hue"),
                            (TargetAdjustKind::Saturation, "Sat"),
                            (TargetAdjustKind::Luminance, "Lum"),
                        ] {
                            if ui
                                .selectable_label(app.target_adjust_kind == kind, lbl)
                                .clicked()
                            {
                                app.target_adjust_kind = kind;
                            }
                        }
                        if ui.small_button("Done").clicked() {
                            app.develop_tool = DevelopTool::None;
                        }
                    });
                });
        }
    }
}

fn render_crop_drawer(app: &mut LightRawApp, ui: &mut Ui) {
    egui::Frame::none()
        .fill(Color32::from_rgb(44, 44, 48))
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            let mut p = app.active_photo().params.clone();
            let mut changed = false;

            ui.horizontal(|ui| {
                ui.label(RichText::new("Aspect:").size(11.0));
                let current_label = match p.crop_aspect_preset {
                    AspectRatioPreset::Custom => {
                        let (img_w, img_h) = app.active_photo_pixel_size();
                        let cur_w_px = (p.crop_right - p.crop_left) * img_w;
                        let cur_h_px = (p.crop_bottom - p.crop_top) * img_h;
                        format!("Custom ({:.2}:1)", cur_w_px / cur_h_px.max(1.0))
                    }
                    other => {
                        let base = other.label();
                        if p.crop_aspect_flipped {
                            format!("{} (Flipped)", base)
                        } else {
                            base.to_string()
                        }
                    }
                };

                egui::ComboBox::from_id_salt("crop_aspect_cb")
                    .selected_text(current_label)
                    .width(135.0)
                    .show_ui(ui, |ui| {
                        for &preset in AspectRatioPreset::ALL {
                            if ui
                                .selectable_label(p.crop_aspect_preset == preset, preset.label())
                                .clicked()
                            {
                                p.crop_aspect_preset = preset;
                                p.crop_constrain_aspect_ratio = preset != AspectRatioPreset::Custom;
                                p.crop_aspect_flipped = false;
                                apply_aspect_ratio_to_crop(app, &mut p);
                                changed = true;
                            }
                        }
                    });

                let lock_lbl = if p.crop_constrain_aspect_ratio { "🔒" } else { "🔓" };
                if ui
                    .small_button(lock_lbl)
                    .on_hover_text(if p.crop_constrain_aspect_ratio {
                        "Constrain Aspect Ratio: LOCKED (Click to Unlock for freeform cropping)"
                    } else {
                        "Constrain Aspect Ratio: UNLOCKED (Click to Lock aspect ratio)"
                    })
                    .clicked()
                {
                    p.crop_constrain_aspect_ratio = !p.crop_constrain_aspect_ratio;
                    changed = true;
                }

                if ui
                    .small_button("Flip Aspect (X)")
                    .on_hover_text("Flip crop aspect ratio between Portrait & Landscape (Shortcut: X)")
                    .clicked()
                {
                    flip_crop_aspect_ratio(app, &mut p);
                    changed = true;
                }
            });

            if lr_slider(ui, "Angle", &mut p.crop_angle, -45.0..=45.0, 0.0, "{:+.1}°") {
                changed = true;
            }

            ui.horizontal(|ui| {
                ui.label(RichText::new("Overlay:").size(11.0));
                egui::ComboBox::from_id_salt("crop_overlay_cb")
                    .selected_text(p.crop_guide_overlay.label())
                    .width(105.0)
                    .show_ui(ui, |ui| {
                        for &ov in CropGuideOverlay::ALL {
                            if ui
                                .selectable_label(p.crop_guide_overlay == ov, ov.label())
                                .clicked()
                            {
                                p.crop_guide_overlay = ov;
                                changed = true;
                            }
                        }
                    });
                for (mode, lbl) in [
                    (ToolOverlayMode::Auto, "Auto"),
                    (ToolOverlayMode::Always, "Always"),
                    (ToolOverlayMode::Never, "Never"),
                ] {
                    if ui
                        .selectable_label(p.crop_tool_overlay_mode == mode, lbl)
                        .clicked()
                    {
                        p.crop_tool_overlay_mode = mode;
                        changed = true;
                    }
                }
            });

            ui.horizontal(|ui| {
                if lr_bevel_button(ui, "Reset Crop", 95.0, 22.0).clicked() {
                    p.crop_left = 0.0;
                    p.crop_top = 0.0;
                    p.crop_right = 1.0;
                    p.crop_bottom = 1.0;
                    p.crop_angle = 0.0;
                    p.crop_aspect_flipped = false;
                    changed = true;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if lr_bevel_button(ui, "Close (R)", 85.0, 22.0).clicked() {
                        app.develop_tool = DevelopTool::None;
                        app.invalidate_photo_render_caches(app.active_photo_id);
                    }
                });
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.checkbox(&mut app.crop_sync_all_photos, "Auto-Sync to Library")
                    .on_hover_text("When enabled, all crop and aspect ratio adjustments automatically synchronize to every photo in the Library");

                let total = app.photos.len();
                let sel_len = app.selected_photo_ids.len();

                if lr_bevel_button(ui, &format!("⇋ Sync to All ({})", total), 125.0, 22.0)
                    .on_hover_text("Synchronize current crop framing, aspect ratio, and angle across all photos in Library")
                    .clicked()
                {
                    app.sync_crop_to_library(false);
                }

                if sel_len > 1 {
                    if lr_bevel_button(ui, &format!("⇋ Sync Selected ({})", sel_len), 130.0, 22.0)
                        .on_hover_text("Synchronize current crop settings to only selected photos")
                        .clicked()
                    {
                        app.sync_crop_to_library(true);
                    }
                }
            });

            if changed {
                app.active_photo_mut().params = p.clone();
                if !ui.input(|i| i.pointer.primary_down()) {
                    app.commit_params_change("Crop & Straighten", p);
                    if app.crop_sync_all_photos {
                        app.sync_crop_to_library(false);
                    }
                } else {
                    ui.ctx().request_repaint();
                }
            }
        });
}

pub fn resolve_aspect_ratio_preset(
    preset: AspectRatioPreset,
    img_w: f32,
    img_h: f32,
    flipped: bool,
    custom_ratio: (f32, f32),
) -> Option<f32> {
    let is_portrait = img_h >= img_w;
    let want_portrait = if flipped { !is_portrait } else { is_portrait };

    match preset {
        AspectRatioPreset::Original => {
            let r = img_w / img_h.max(1.0);
            if flipped {
                Some(1.0 / r)
            } else {
                Some(r)
            }
        }
        AspectRatioPreset::Custom => {
            let (cw, ch) = custom_ratio;
            let base = (cw / ch.max(0.01)).max(0.01);
            let r = if want_portrait {
                base.min(1.0 / base)
            } else {
                base.max(1.0 / base)
            };
            Some(r)
        }
        AspectRatioPreset::Ratio1x1 => Some(1.0),
        AspectRatioPreset::Ratio4x5 => {
            if want_portrait {
                Some(4.0 / 5.0)
            } else {
                Some(5.0 / 4.0)
            }
        }
        AspectRatioPreset::Ratio8_5x11 => {
            if want_portrait {
                Some(8.5 / 11.0)
            } else {
                Some(11.0 / 8.5)
            }
        }
        AspectRatioPreset::Ratio5x7 => {
            if want_portrait {
                Some(5.0 / 7.0)
            } else {
                Some(7.0 / 5.0)
            }
        }
        AspectRatioPreset::Ratio2x3 => {
            if want_portrait {
                Some(2.0 / 3.0)
            } else {
                Some(3.0 / 2.0)
            }
        }
        AspectRatioPreset::Ratio4x3 => {
            if want_portrait {
                Some(3.0 / 4.0)
            } else {
                Some(4.0 / 3.0)
            }
        }
        AspectRatioPreset::Ratio16x9 => {
            if want_portrait {
                Some(9.0 / 16.0)
            } else {
                Some(16.0 / 9.0)
            }
        }
        AspectRatioPreset::Ratio16x10 => {
            if want_portrait {
                Some(10.0 / 16.0)
            } else {
                Some(16.0 / 10.0)
            }
        }
    }
}

pub fn apply_aspect_ratio_to_crop(app: &mut LightRawApp, p: &mut crate::params::DevelopParams) {
    let (img_w, img_h) = app.active_photo_pixel_size();
    let Some(target_px_aspect) = resolve_aspect_ratio_preset(
        p.crop_aspect_preset,
        img_w,
        img_h,
        p.crop_aspect_flipped,
        p.crop_custom_ratio,
    ) else {
        return;
    };

    let norm_ratio = target_px_aspect * (img_h / img_w);
    let cx = 0.5 * (p.crop_left + p.crop_right);
    let cy = 0.5 * (p.crop_top + p.crop_bottom);

    let (w, h) = if norm_ratio >= 1.0 {
        let w = 1.0_f32;
        let h = (w / norm_ratio).min(1.0);
        (w, h)
    } else {
        let h = 1.0_f32;
        let w = (h * norm_ratio).min(1.0);
        (w, h)
    };

    let mut left = cx - w * 0.5;
    if left < 0.0 { left = 0.0; }
    if left + w > 1.0 { left = 1.0 - w; }
    let mut top = cy - h * 0.5;
    if top < 0.0 { top = 0.0; }
    if top + h > 1.0 { top = 1.0 - h; }

    p.crop_left = left.clamp(0.0, (1.0 - w).max(0.0));
    p.crop_right = (p.crop_left + w).min(1.0);
    p.crop_top = top.clamp(0.0, (1.0 - h).max(0.0));
    p.crop_bottom = (p.crop_top + h).min(1.0);
}

pub fn flip_crop_aspect_ratio(app: &mut LightRawApp, p: &mut crate::params::DevelopParams) {
    let (img_w, img_h) = app.active_photo_pixel_size();
    p.crop_aspect_flipped = !p.crop_aspect_flipped;

    let cur_w_px = (p.crop_right - p.crop_left).max(0.02) * img_w;
    let cur_h_px = (p.crop_bottom - p.crop_top).max(0.02) * img_h;
    let target_px_aspect = cur_h_px / cur_w_px;
    let norm_ratio = target_px_aspect * (img_h / img_w);
    let cx = 0.5 * (p.crop_left + p.crop_right);
    let cy = 0.5 * (p.crop_top + p.crop_bottom);

    let cur_norm_w = p.crop_right - p.crop_left;
    let cur_norm_h = p.crop_bottom - p.crop_top;
    let geom_mean = (cur_norm_w * cur_norm_h).sqrt();

    let mut w = (geom_mean * norm_ratio.sqrt()).min(1.0);
    let mut h = (w / norm_ratio).min(1.0);
    w = (h * norm_ratio).min(1.0);
    h = (w / norm_ratio).min(1.0);

    let mut left = cx - w * 0.5;
    if left < 0.0 { left = 0.0; }
    if left + w > 1.0 { left = 1.0 - w; }
    let mut top = cy - h * 0.5;
    if top < 0.0 { top = 0.0; }
    if top + h > 1.0 { top = 1.0 - h; }

    p.crop_left = left.clamp(0.0, (1.0 - w).max(0.0));
    p.crop_right = (p.crop_left + w).min(1.0);
    p.crop_top = top.clamp(0.0, (1.0 - h).max(0.0));
    p.crop_bottom = (p.crop_top + h).min(1.0);
}

fn render_remove_drawer(app: &mut LightRawApp, ui: &mut Ui) {
    egui::Frame::none()
        .fill(Color32::from_rgb(44, 44, 48))
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Mode:").size(11.0));
                for (m, lbl) in [
                    (RetouchMode::Remove, "Generative / Content Remove"),
                    (RetouchMode::Heal, "Heal"),
                    (RetouchMode::Clone, "Clone"),
                ] {
                    if ui
                        .selectable_label(app.retouch_mode_ui == m, lbl)
                        .clicked()
                    {
                        app.retouch_mode_ui = m;
                    }
                }
            });

            let mut size_pct = app.retouch_brush_radius * 1000.0;
            if lr_slider(ui, "Size", &mut size_pct, 5.0..=150.0, 35.0, "{:.0}") {
                app.retouch_brush_radius = size_pct / 1000.0;
                if let Some(idx) = app.selected_retouch_idx {
                    let mut p = app.active_photo().params.clone();
                    if let Some(spot) = p.retouch_areas.get_mut(idx) {
                        spot.radius = app.retouch_brush_radius;
                        app.commit_params_change("Update Heal Brush Size", p);
                    }
                }
            }
            if lr_slider(ui, "Feather", &mut app.retouch_feather, 0.0..=100.0, 50.0, "{:.0}") {
                if let Some(idx) = app.selected_retouch_idx {
                    let mut p = app.active_photo().params.clone();
                    if let Some(spot) = p.retouch_areas.get_mut(idx) {
                        spot.feather = app.retouch_feather;
                        app.commit_params_change("Update Heal Feather", p);
                    }
                }
            }
            if lr_slider(ui, "Opacity", &mut app.retouch_opacity, 0.0..=100.0, 100.0, "{:.0}") {
                if let Some(idx) = app.selected_retouch_idx {
                    let mut p = app.active_photo().params.clone();
                    if let Some(spot) = p.retouch_areas.get_mut(idx) {
                        spot.opacity = app.retouch_opacity;
                        app.commit_params_change("Update Heal Opacity", p);
                    }
                }
            }

            let mut p = app.active_photo().params.clone();
            let mut vis_changed = false;
            ui.horizontal(|ui| {
                if ui
                    .checkbox(&mut p.retouch_visualize_spots, "Visualize Spots")
                    .changed()
                {
                    vis_changed = true;
                }
                if p.retouch_visualize_spots {
                    if ui
                        .add(egui::Slider::new(&mut p.retouch_visualize_threshold, 0.0..=100.0).show_value(false))
                        .changed()
                    {
                        vis_changed = true;
                    }
                }
            });
            if vis_changed {
                app.commit_params_change("Toggle Visualize Spots", p.clone());
            }

            ui.horizontal(|ui| {
                if let Some(idx) = app.selected_retouch_idx {
                    if ui.small_button("Delete Selected").clicked() && idx < p.retouch_areas.len() {
                        p.retouch_areas.remove(idx);
                        app.selected_retouch_idx = None;
                        app.commit_params_change("Delete Spot", p.clone());
                    }
                }
                if !p.retouch_areas.is_empty() && ui.small_button("Reset All").clicked() {
                    p.retouch_areas.clear();
                    app.selected_retouch_idx = None;
                    app.commit_params_change("Clear All Spots", p);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Close (Q)").clicked() {
                        app.develop_tool = DevelopTool::None;
                    }
                });
            });
        });
}

fn render_redeye_drawer(app: &mut LightRawApp, ui: &mut Ui) {
    egui::Frame::none()
        .fill(Color32::from_rgb(44, 44, 48))
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                for (m, lbl) in [(RedEyeMode::RedEye, "Red Eye"), (RedEyeMode::PetEye, "Pet Eye")] {
                    if ui
                        .selectable_label(app.redeye_mode_ui == m, lbl)
                        .clicked()
                    {
                        app.redeye_mode_ui = m;
                    }
                }
            });

            let mut updated_spot = false;
            if lr_slider(ui, "Pupil Size", &mut app.redeye_pupil_size, 5.0..=100.0, 50.0, "{:.0}") {
                updated_spot = true;
            }
            if app.redeye_mode_ui == RedEyeMode::RedEye {
                if lr_slider(ui, "Darken", &mut app.redeye_darken, 0.0..=100.0, 50.0, "{:.0}") {
                    updated_spot = true;
                }
            } else if ui
                .checkbox(&mut app.redeye_catchlight, "Add Catchlight")
                .changed()
            {
                updated_spot = true;
            }

            if updated_spot {
                if let Some(idx) = app.selected_redeye_idx {
                    let mut p = app.active_photo().params.clone();
                    if let Some(sp) = p.red_eye_spots.get_mut(idx) {
                        sp.pupil_size = app.redeye_pupil_size;
                        sp.darken = app.redeye_darken;
                        sp.add_catchlight = app.redeye_catchlight;
                        app.commit_params_change("Adjust Red-Eye", p);
                    }
                }
            }

            ui.horizontal(|ui| {
                let mut p = app.active_photo().params.clone();
                if !p.red_eye_spots.is_empty() && ui.small_button("Reset Eyes").clicked() {
                    p.red_eye_spots.clear();
                    app.selected_redeye_idx = None;
                    app.commit_params_change("Reset Red-Eye", p);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Close").clicked() {
                        app.develop_tool = DevelopTool::None;
                    }
                });
            });
        });
}

fn render_masking_drawer(app: &mut LightRawApp, ui: &mut Ui) {
    egui::Frame::none()
        .fill(Color32::from_rgb(42, 42, 46))
        .inner_margin(egui::Margin::symmetric(8.0, 8.0))
        .show(ui, |ui| {
            let mut p = app.active_photo().params.clone();
            let mut changed = false;
            let mut step_lbl = "Local Masking";

            ui.horizontal(|ui| {
                ui.label(RichText::new("Masks").strong().size(12.0));
                ui.checkbox(&mut app.show_mask_overlay, "Overlay (O)");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Done").clicked() {
                        app.develop_tool = DevelopTool::None;
                    }
                });
            });

            // Create New Mask Menu (`+ Create New Mask`)
            ui.menu_button("➕ Create New Mask...", |ui| {
                let mut add_prim: Option<(&str, MaskPrimitiveKind)> = None;
                if ui.button("🎯 Subject (AI)").clicked() {
                    add_prim = Some(("Subject", MaskPrimitiveKind::AiSubject));
                }
                if ui.button("☁ Sky (AI)").clicked() {
                    add_prim = Some(("Sky", MaskPrimitiveKind::AiSky));
                }
                if ui.button("🖼 Background (AI)").clicked() {
                    add_prim = Some(("Background", MaskPrimitiveKind::AiBackground));
                }
                ui.menu_button("🧑 People (AI Parts)", |ui| {
                    for (part, lbl) in [
                        (PersonPart::EntirePerson, "Entire Person"),
                        (PersonPart::FaceSkin, "Face Skin"),
                        (PersonPart::BodySkin, "Body Skin"),
                        (PersonPart::Eyebrows, "Eyebrows"),
                        (PersonPart::EyeSclera, "Eye Sclera"),
                        (PersonPart::IrisPupil, "Iris and Pupil"),
                        (PersonPart::Lips, "Lips"),
                        (PersonPart::Teeth, "Teeth"),
                        (PersonPart::Hair, "Hair"),
                        (PersonPart::Clothes, "Clothes"),
                    ] {
                        if ui.button(lbl).clicked() {
                            add_prim = Some((
                                lbl,
                                MaskPrimitiveKind::PersonFeature {
                                    person_index: 0,
                                    part,
                                },
                            ));
                        }
                    }
                });
                ui.menu_button("🏞 Landscape (AI Parts)", |ui| {
                    for (part, lbl) in [
                        (LandscapePart::Sky, "Sky"),
                        (LandscapePart::Mountains, "Mountains"),
                        (LandscapePart::Architecture, "Architecture"),
                        (LandscapePart::Vegetation, "Vegetation"),
                        (LandscapePart::Water, "Water"),
                        (LandscapePart::NaturalGround, "Natural Ground"),
                        (LandscapePart::ArtificialGround, "Artificial Ground"),
                    ] {
                        if ui.button(lbl).clicked() {
                            add_prim = Some((lbl, MaskPrimitiveKind::LandscapeFeature(part)));
                        }
                    }
                });
                ui.separator();
                if ui.button("🖌 Brush").clicked() {
                    add_prim = Some((
                        "Brush",
                        MaskPrimitiveKind::Brush {
                            points: Vec::new(),
                            radius: app.mask_brush_size,
                            feather: app.mask_brush_feather / 100.0,
                            flow: app.mask_brush_flow / 100.0,
                        },
                    ));
                }
                if ui.button("▤ Linear Gradient").clicked() {
                    add_prim = Some((
                        "Linear Gradient",
                        MaskPrimitiveKind::LinearGradient {
                            x1: 0.5,
                            y1: 0.18,
                            x2: 0.5,
                            y2: 0.68,
                        },
                    ));
                }
                if ui.button("◎ Radial Gradient").clicked() {
                    add_prim = Some((
                        "Radial Gradient",
                        MaskPrimitiveKind::RadialGradient {
                            cx: 0.5,
                            cy: 0.42,
                            rx: 0.28,
                            ry: 0.28,
                            feather: 60.0,
                            invert: false,
                        },
                    ));
                }
                ui.menu_button("🎨 Range Mask", |ui| {
                    if ui.button("Color Range (Click photo to sample)").clicked() {
                        add_prim = Some((
                            "Color Range",
                            MaskPrimitiveKind::ColorRange {
                                target_hue: 210.0,
                                target_sat: 0.65,
                                tolerance: 35.0,
                            },
                        ));
                    }
                    if ui.button("Luminance Range").clicked() {
                        add_prim = Some((
                            "Luminance Range",
                            MaskPrimitiveKind::LuminanceRange {
                                min_lum: 0.55,
                                max_lum: 1.0,
                                feather: 35.0,
                            },
                        ));
                    }
                    if ui.button("Depth Range").clicked() {
                        add_prim = Some((
                            "Depth Range",
                            MaskPrimitiveKind::DepthRange {
                                min_depth: 0.0,
                                max_depth: 0.45,
                                feather: 30.0,
                            },
                        ));
                    }
                });

                if let Some((lbl, kind)) = add_prim {
                    let next_id = p.mask_groups.len() as u64 + 1;
                    p.mask_groups.push(MaskGroup {
                        id: next_id,
                        name: format!("Mask {}: {}", next_id, lbl),
                        enabled: true,
                        invert: false,
                        components: vec![MaskComponent {
                            name: lbl.to_string(),
                            op: MaskOp::Add,
                            invert: false,
                            kind,
                        }],
                        local: LocalAdjustParams::default(),
                    });
                    app.selected_mask_idx = Some(p.mask_groups.len() - 1);
                    app.selected_component_idx = 0;
                    app.show_mask_overlay = true;
                    changed = true;
                    step_lbl = "Create Mask";
                    ui.close_menu();
                }
            });

            // List existing Mask Groups
            let mut del_mask = None;
            for (idx, mg) in p.mask_groups.iter_mut().enumerate() {
                let is_sel = app.selected_mask_idx == Some(idx);
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut mg.enabled, "").changed() {
                        changed = true;
                    }
                    if ui.selectable_label(is_sel, &mg.name).clicked() {
                        app.selected_mask_idx = Some(idx);
                        app.selected_component_idx = 0;
                    }
                    if ui.small_button("Invert").clicked() {
                        mg.invert = !mg.invert;
                        changed = true;
                    }
                    if ui.small_button("✕").clicked() {
                        del_mask = Some(idx);
                    }
                });
            }
            if let Some(d_idx) = del_mask {
                p.mask_groups.remove(d_idx);
                app.selected_mask_idx = if p.mask_groups.is_empty() {
                    None
                } else {
                    Some(d_idx.saturating_sub(1))
                };
                changed = true;
            }

            // Active Mask Group Boolean Add / Subtract / Intersect + Local Sliders
            if let Some(m_idx) = app.selected_mask_idx {
                if let Some(mg) = p.mask_groups.get_mut(m_idx) {
                    ui.separator();
                    ui.horizontal(|ui| {
                        for (op, op_lbl) in [
                            (MaskOp::Add, "+ Add"),
                            (MaskOp::Subtract, "- Subtract"),
                            (MaskOp::Intersect, "∩ Intersect"),
                        ] {
                            ui.menu_button(op_lbl, |ui| {
                                let mut new_comp = None;
                                if ui.button("Brush").clicked() {
                                    new_comp = Some((
                                        "Brush",
                                        MaskPrimitiveKind::Brush {
                                            points: Vec::new(),
                                            radius: app.mask_brush_size,
                                            feather: app.mask_brush_feather / 100.0,
                                            flow: app.mask_brush_flow / 100.0,
                                        },
                                    ));
                                }
                                if ui.button("Linear Gradient").clicked() {
                                    new_comp = Some((
                                        "Linear Gradient",
                                        MaskPrimitiveKind::LinearGradient {
                                            x1: 0.5,
                                            y1: 0.2,
                                            x2: 0.5,
                                            y2: 0.7,
                                        },
                                    ));
                                }
                                if ui.button("Radial Gradient").clicked() {
                                    new_comp = Some((
                                        "Radial Gradient",
                                        MaskPrimitiveKind::RadialGradient {
                                            cx: 0.5,
                                            cy: 0.5,
                                            rx: 0.25,
                                            ry: 0.25,
                                            feather: 60.0,
                                            invert: false,
                                        },
                                    ));
                                }
                                if ui.button("Subject (AI)").clicked() {
                                    new_comp = Some(("Subject", MaskPrimitiveKind::AiSubject));
                                }
                                if ui.button("Sky (AI)").clicked() {
                                    new_comp = Some(("Sky", MaskPrimitiveKind::AiSky));
                                }
                                if let Some((nm, kind)) = new_comp {
                                    mg.components.push(MaskComponent {
                                        name: nm.to_string(),
                                        op,
                                        invert: false,
                                        kind,
                                    });
                                    app.selected_component_idx = mg.components.len() - 1;
                                    changed = true;
                                    ui.close_menu();
                                }
                            });
                        }
                    });

                    // Component list & Brush/Range controls
                    for (c_idx, comp) in mg.components.iter_mut().enumerate() {
                        let c_sel = app.selected_component_idx == c_idx;
                        ui.horizontal(|ui| {
                            let prefix = match comp.op {
                                MaskOp::Add => "+",
                                MaskOp::Subtract => "−",
                                MaskOp::Intersect => "∩",
                            };
                            if ui
                                .selectable_label(c_sel, format!("{} {}", prefix, comp.name))
                                .clicked()
                            {
                                app.selected_component_idx = c_idx;
                            }
                            if ui.small_button("Inv").clicked() {
                                comp.invert = !comp.invert;
                                changed = true;
                            }
                        });
                    }

                    if let Some(comp) = mg.components.get_mut(app.selected_component_idx) {
                        match &mut comp.kind {
                            MaskPrimitiveKind::Brush {
                                radius,
                                feather,
                                flow,
                                ..
                            } => {
                                let mut sz = *radius * 500.0;
                                if lr_slider(ui, "Brush Size", &mut sz, 5.0..=150.0, 40.0, "{:.0}") {
                                    *radius = sz / 500.0;
                                    app.mask_brush_size = *radius;
                                    changed = true;
                                }
                                let mut ft = *feather * 100.0;
                                if lr_slider(ui, "Feather", &mut ft, 0.0..=100.0, 55.0, "{:.0}") {
                                    *feather = ft / 100.0;
                                    changed = true;
                                }
                                let mut fl = *flow * 100.0;
                                if lr_slider(ui, "Flow", &mut fl, 5.0..=100.0, 85.0, "{:.0}") {
                                    *flow = fl / 100.0;
                                    changed = true;
                                }
                            }
                            MaskPrimitiveKind::ColorRange { tolerance, .. } => {
                                changed |= lr_slider(ui, "Amount", tolerance, 5.0..=100.0, 35.0, "{:.0}");
                            }
                            MaskPrimitiveKind::LuminanceRange {
                                min_lum,
                                max_lum,
                                feather,
                            } => {
                                changed |= lr_slider(ui, "Lum Min", min_lum, 0.0..=1.0, 0.5, "{:.2}");
                                changed |= lr_slider(ui, "Lum Max", max_lum, 0.0..=1.0, 1.0, "{:.2}");
                                changed |=
                                    lr_slider(ui, "Feather", feather, 0.0..=100.0, 35.0, "{:.0}");
                            }
                            MaskPrimitiveKind::DepthRange {
                                min_depth,
                                max_depth,
                                feather,
                            } => {
                                changed |= lr_slider(ui, "Depth Near", min_depth, 0.0..=1.0, 0.0, "{:.2}");
                                changed |= lr_slider(ui, "Depth Far", max_depth, 0.0..=1.0, 0.5, "{:.2}");
                                changed |=
                                    lr_slider(ui, "Feather", feather, 0.0..=100.0, 30.0, "{:.0}");
                            }
                            _ => {}
                        }
                    }

                    // LocalPreset quick picker
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Effect:").size(10.5));
                        egui::ComboBox::from_id_salt("local_preset_cb")
                            .selected_text("Custom")
                            .width(140.0)
                            .show_ui(ui, |ui| {
                                for lp in &app.local_presets {
                                    if ui.selectable_label(false, lp.name).clicked() {
                                        mg.local = lp.local.clone();
                                        changed = true;
                                    }
                                }
                            });
                    });

                    changed |= lr_slider(ui, "Amount", &mut mg.local.amount, 0.0..=200.0, 100.0, "{:.0}");
                    changed |= lr_slider(ui, "Exposure", &mut mg.local.exposure, -4.0..=4.0, 0.0, "{:+.2}");
                    changed |= lr_slider(ui, "Contrast", &mut mg.local.contrast, -100.0..=100.0, 0.0, "{:+.0}");
                    changed |= lr_slider(ui, "Highlights", &mut mg.local.highlights, -100.0..=100.0, 0.0, "{:+.0}");
                    changed |= lr_slider(ui, "Shadows", &mut mg.local.shadows, -100.0..=100.0, 0.0, "{:+.0}");
                    changed |= lr_slider(ui, "Whites", &mut mg.local.whites, -100.0..=100.0, 0.0, "{:+.0}");
                    changed |= lr_slider(ui, "Blacks", &mut mg.local.blacks, -100.0..=100.0, 0.0, "{:+.0}");
                    changed |= lr_slider(ui, "Temp", &mut mg.local.temperature, -100.0..=100.0, 0.0, "{:+.0}");
                    changed |= lr_slider(ui, "Tint", &mut mg.local.tint, -100.0..=100.0, 0.0, "{:+.0}");
                    changed |= lr_slider(ui, "Texture", &mut mg.local.texture, -100.0..=100.0, 0.0, "{:+.0}");
                    changed |= lr_slider(ui, "Clarity", &mut mg.local.clarity, -100.0..=100.0, 0.0, "{:+.0}");
                    changed |= lr_slider(ui, "Dehaze", &mut mg.local.dehaze, -100.0..=100.0, 0.0, "{:+.0}");
                    changed |= lr_slider(ui, "Saturation", &mut mg.local.saturation, -100.0..=100.0, 0.0, "{:+.0}");
                    changed |= lr_slider(ui, "Sharpness", &mut mg.local.sharpness, -100.0..=100.0, 0.0, "{:+.0}");
                }
            }

            if changed {
                app.commit_params_change(step_lbl, p);
            }
        });
}

pub fn paint_photo_mesh_rotated(
    painter: &egui::Painter,
    tex_id: egui::TextureId,
    dst: Rect,
    angle_deg: f32,
) {
    if angle_deg.abs() <= 0.01 {
        painter.image(
            tex_id,
            dst,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
        return;
    }
    let rot_rad = -angle_deg.to_radians();
    let cos_a = rot_rad.cos();
    let sin_a = rot_rad.sin();
    let c = dst.center();
    let rot_pt = |pt: Pos2| -> Pos2 {
        let rel = pt - c;
        Pos2::new(
            c.x + rel.x * cos_a - rel.y * sin_a,
            c.y + rel.x * sin_a + rel.y * cos_a,
        )
    };

    let p0 = rot_pt(dst.left_top());
    let p1 = rot_pt(dst.right_top());
    let p2 = rot_pt(dst.right_bottom());
    let p3 = rot_pt(dst.left_bottom());

    let mut mesh = egui::Mesh::with_texture(tex_id);
    let idx = mesh.vertices.len() as u32;
    mesh.vertices.push(egui::epaint::Vertex {
        pos: p0,
        uv: Pos2::new(0.0, 0.0),
        color: Color32::WHITE,
    });
    mesh.vertices.push(egui::epaint::Vertex {
        pos: p1,
        uv: Pos2::new(1.0, 0.0),
        color: Color32::WHITE,
    });
    mesh.vertices.push(egui::epaint::Vertex {
        pos: p2,
        uv: Pos2::new(1.0, 1.0),
        color: Color32::WHITE,
    });
    mesh.vertices.push(egui::epaint::Vertex {
        pos: p3,
        uv: Pos2::new(0.0, 1.0),
        color: Color32::WHITE,
    });
    mesh.add_triangle(idx, idx + 1, idx + 2);
    mesh.add_triangle(idx, idx + 2, idx + 3);
    painter.add(egui::Shape::mesh(mesh));
}

pub fn render_develop_canvas(app: &mut LightRawApp, ctx: &egui::Context, ui: &mut Ui) {
    render_soft_proofing_top_bar(app, ui);

    let toolbar_h = if app.show_toolbar { 32.0 } else { 0.0 };
    let canvas_h = (ui.available_height() - toolbar_h).max(120.0);
    let (canvas_rect, canvas_resp) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), canvas_h), Sense::click_and_drag());

    let surround_col = if app.soft_proofing && app.simulate_paper_ink {
        Color32::from_rgb(232, 230, 224)
    } else {
        app.canvas_bg_color()
    };
    ui.painter().rect_filled(canvas_rect, 0.0, surround_col);

    // Right-click Canvas Surround Menu (Lightroom Classic Background Color Switcher)
    canvas_resp.context_menu(|ui| {
        ui.label(RichText::new("Develop Canvas Background").strong().size(11.0));
        for mode in CanvasSurroundMode::ALL {
            if ui
                .radio_value(&mut app.canvas_surround, mode, mode.label())
                .clicked()
            {
                ui.close_menu();
            }
        }
        ui.separator();
        app.render_photo_context_menu(ui, app.active_photo_id);
    });

    if app.photos.is_empty() {
        ui.painter().text(
            canvas_rect.center(),
            egui::Align2::CENTER_CENTER,
            "No Photo Selected — Import RAW or RGB Photos (⇧⌘I)",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(225, 225, 230),
        );
        if app.show_toolbar {
            render_develop_toolbar(app, ui);
        }
        return;
    }

    // Spacebar toggle Zoom between Fit (1.0x) and 100% Detail (2.5x)
    if ui.input(|i| i.key_pressed(egui::Key::Space)) {
        if app.loupe_zoom > 1.02 {
            app.loupe_zoom = 1.0;
            app.loupe_pan = Vec2::ZERO;
        } else {
            app.loupe_zoom = 2.5;
            let ptr_pos = canvas_resp.hover_pos().unwrap_or(canvas_rect.center());
            let center = canvas_rect.center();
            app.loupe_pan = (center - ptr_pos) * (app.loupe_zoom - 1.0);
        }
    }

    // Mouse-wheel / Trackpad Pinch Zoom inside Develop Canvas
    if canvas_resp.hovered() {
        let scroll_y = ui.input(|i| i.smooth_scroll_delta.y);
        let zoom_delta = ui.input(|i| i.zoom_delta());
        let ptr_pos = canvas_resp.hover_pos().unwrap_or(canvas_rect.center());
        if (zoom_delta - 1.0).abs() > 0.002 {
            let old_zoom = app.loupe_zoom;
            let new_zoom = (old_zoom * zoom_delta).clamp(0.5, 8.0);
            if (new_zoom - old_zoom).abs() > 0.0001 {
                let factor = new_zoom / old_zoom;
                let c = canvas_rect.center();
                app.loupe_pan = ptr_pos - c - (ptr_pos - c - app.loupe_pan) * factor;
                app.loupe_zoom = new_zoom;
                if app.loupe_zoom <= 1.02 {
                    app.loupe_pan = Vec2::ZERO;
                }
            }
        } else if scroll_y.abs() > 0.5 && ui.input(|i| i.modifiers.command || i.modifiers.alt) {
            let old_zoom = app.loupe_zoom;
            let factor = (1.0 + scroll_y * 0.004).clamp(0.8, 1.25);
            let new_zoom = (old_zoom * factor).clamp(0.5, 8.0);
            if (new_zoom - old_zoom).abs() > 0.0001 {
                let actual_factor = new_zoom / old_zoom;
                let c = canvas_rect.center();
                app.loupe_pan = ptr_pos - c - (ptr_pos - c - app.loupe_pan) * actual_factor;
                app.loupe_zoom = new_zoom;
                if app.loupe_zoom <= 1.02 {
                    app.loupe_pan = Vec2::ZERO;
                }
            }
        }
    }

    let apply_crop = app.develop_tool != DevelopTool::Crop;
    let active_id = app.active_photo_id;
    let Some(after_tex) = app.get_photo_texture(ctx, active_id, apply_crop) else {
        return;
    };

    let ts = after_tex.size_vec2();
    let fit_scale = (canvas_rect.width() / ts.x).min(canvas_rect.height() / ts.y) * 0.94;
    let dst = Rect::from_center_size(
        canvas_rect.center() + app.loupe_pan,
        ts * fit_scale * app.loupe_zoom,
    );

    // Clip image drawing strictly inside `canvas_rect`
    let painter = ui.painter().with_clip_rect(canvas_rect);

    if app.develop_tool == DevelopTool::Crop {
        paint_photo_mesh_rotated(
            &painter,
            after_tex.id(),
            dst,
            app.active_photo().params.crop_angle,
        );
    } else {
        match app.before_after_mode {
            BeforeAfterMode::Single => {
                painter.image(
                    after_tex.id(),
                    dst,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
        BeforeAfterMode::BeforeOnly => {
            if let Some(before_tex) = app.get_before_texture(ctx, active_id) {
                painter.image(
                    before_tex.id(),
                    dst,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
            paint_corner_pill(&painter, canvas_rect.min + Vec2::new(14.0, 14.0), "BEFORE");
        }
        BeforeAfterMode::SplitLeftRight => {
            if let Some(before_tex) = app.get_before_texture(ctx, active_id) {
                let mid_x = dst.center().x;
                let left_r = Rect::from_min_max(dst.min, Pos2::new(mid_x, dst.max.y));
                let right_r = Rect::from_min_max(Pos2::new(mid_x, dst.min.y), dst.max);
                painter.image(
                    before_tex.id(),
                    left_r,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(0.5, 1.0)),
                    Color32::WHITE,
                );
                painter.image(
                    after_tex.id(),
                    right_r,
                    Rect::from_min_max(Pos2::new(0.5, 0.0), Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
                painter.line_segment(
                    [Pos2::new(mid_x, dst.min.y), Pos2::new(mid_x, dst.max.y)],
                    Stroke::new(1.8, Color32::WHITE),
                );
                paint_corner_pill(&painter, dst.min + Vec2::new(10.0, 10.0), "BEFORE");
                paint_corner_pill(
                    &painter,
                    Pos2::new(dst.max.x - 74.0, dst.min.y + 10.0),
                    "AFTER",
                );
            }
        }
        BeforeAfterMode::SplitTopBottom => {
            if let Some(before_tex) = app.get_before_texture(ctx, active_id) {
                let mid_y = dst.center().y;
                let top_r = Rect::from_min_max(dst.min, Pos2::new(dst.max.x, mid_y));
                let bot_r = Rect::from_min_max(Pos2::new(dst.min.x, mid_y), dst.max);
                painter.image(
                    before_tex.id(),
                    top_r,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 0.5)),
                    Color32::WHITE,
                );
                painter.image(
                    after_tex.id(),
                    bot_r,
                    Rect::from_min_max(Pos2::new(0.0, 0.5), Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
                painter.line_segment(
                    [Pos2::new(dst.min.x, mid_y), Pos2::new(dst.max.x, mid_y)],
                    Stroke::new(1.8, Color32::WHITE),
                );
                paint_corner_pill(&painter, dst.min + Vec2::new(10.0, 10.0), "BEFORE");
                paint_corner_pill(
                    &painter,
                    Pos2::new(dst.min.x + 10.0, dst.max.y - 32.0),
                    "AFTER",
                );
            }
        }
        BeforeAfterMode::SideBySide => {
            let half_w = canvas_rect.width() * 0.5 - 6.0;
            let l_box = Rect::from_min_size(
                canvas_rect.min + Vec2::new(4.0, 4.0),
                Vec2::new(half_w, canvas_rect.height() - 8.0),
            );
            let r_box = Rect::from_min_size(
                Pos2::new(canvas_rect.center().x + 2.0, canvas_rect.min.y + 4.0),
                Vec2::new(half_w, canvas_rect.height() - 8.0),
            );
            let sc = (l_box.width() / ts.x).min(l_box.height() / ts.y) * 0.94;
            if let Some(before_tex) = app.get_before_texture(ctx, active_id) {
                let d_l = Rect::from_center_size(l_box.center(), ts * sc);
                painter.image(
                    before_tex.id(),
                    d_l,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
                paint_corner_pill(&painter, d_l.min + Vec2::new(8.0, 8.0), "BEFORE");
            }
            let d_r = Rect::from_center_size(r_box.center(), ts * sc);
            painter.image(
                after_tex.id(),
                d_r,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
            paint_corner_pill(&painter, d_r.min + Vec2::new(8.0, 8.0), "AFTER");
        }
        BeforeAfterMode::ReferenceView => {
            let ref_id = app.reference_photo_id.unwrap_or(active_id);
            let half_w = canvas_rect.width() * 0.5 - 6.0;
            let l_box = Rect::from_min_size(
                canvas_rect.min + Vec2::new(4.0, 4.0),
                Vec2::new(half_w, canvas_rect.height() - 8.0),
            );
            let r_box = Rect::from_min_size(
                Pos2::new(canvas_rect.center().x + 2.0, canvas_rect.min.y + 4.0),
                Vec2::new(half_w, canvas_rect.height() - 8.0),
            );
            if let Some(ref_tex) = app.get_photo_texture(ctx, ref_id, true) {
                let rts = ref_tex.size_vec2();
                let rsc = (l_box.width() / rts.x).min(l_box.height() / rts.y) * 0.94;
                let d_l = Rect::from_center_size(l_box.center(), rts * rsc);
                painter.image(
                    ref_tex.id(),
                    d_l,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
                paint_corner_pill(&painter, d_l.min + Vec2::new(8.0, 8.0), "REFERENCE");
            }
            let sc = (r_box.width() / ts.x).min(r_box.height() / ts.y) * 0.94;
            let d_r = Rect::from_center_size(r_box.center(), ts * sc);
            painter.image(
                after_tex.id(),
                d_r,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
            paint_corner_pill(&painter, d_r.min + Vec2::new(8.0, 8.0), "ACTIVE");
        }
    }
}

    // Subtle crisp border around the photo on the medium-gray surround
    if matches!(
        app.before_after_mode,
        BeforeAfterMode::Single
            | BeforeAfterMode::BeforeOnly
            | BeforeAfterMode::SplitLeftRight
            | BeforeAfterMode::SplitTopBottom
    ) {
        painter.rect_stroke(
            dst,
            0.0,
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 0, 0, 90)),
        );
    }

    // Interactive Tool Overlays & Eyedroppers on `dst`
    handle_canvas_interactions(app, ui, &painter, &canvas_resp, dst);

    if app.show_toolbar {
        render_develop_toolbar(app, ui);
    }
}

fn paint_corner_pill(painter: &egui::Painter, pos: Pos2, text: &str) {
    let rect = Rect::from_min_size(pos, Vec2::new(72.0, 20.0));
    painter.rect_filled(rect, 3.0, Color32::from_rgba_unmultiplied(18, 18, 22, 205));
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(10.0),
        Color32::WHITE,
    );
}

fn handle_canvas_interactions(
    app: &mut LightRawApp,
    ui: &mut Ui,
    painter: &egui::Painter,
    resp: &egui::Response,
    dst: Rect,
) {
    let ptr_opt = resp.interact_pointer_pos().or_else(|| ui.input(|i| i.pointer.hover_pos()));
    let norm_uv = ptr_opt.map(|pos| {
        (
            ((pos.x - dst.min.x) / dst.width().max(1.0)).clamp(0.0, 1.0),
            ((pos.y - dst.min.y) / dst.height().max(1.0)).clamp(0.0, 1.0),
        )
    });

    // Global cancellation of active eyedroppers / pickers / develop tools on Escape
    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.wb_eyedropper_active = false;
        app.detail_sample_tool_active = false;
        app.point_color_eyedropper_active = false;
        app.defringe_eyedropper_active = None;
        app.lens_blur_focus_picker_active = false;
        app.develop_tool = DevelopTool::None;
        app.selected_mask_idx = None;
    }

    // 1. White Balance Eyedropper (`W`)
    if app.wb_eyedropper_active {
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
        }
        if let (Some(pos), Some((u, v))) = (ptr_opt, norm_uv) {
            if dst.contains(pos) {
                // 5x5 sampling reticle with high-contrast dual stroke
                painter.rect_stroke(
                    Rect::from_center_size(pos, Vec2::splat(18.0)),
                    2.0,
                    Stroke::new(1.5, Color32::WHITE),
                );
                painter.rect_stroke(
                    Rect::from_center_size(pos, Vec2::splat(18.0)),
                    2.0,
                    Stroke::new(0.5, Color32::BLACK),
                );

                // Floating RGB readout badge next to cursor
                if let Some(frame) = app.active_rendered_frame() {
                    let fx = ((u * (frame.width - 1) as f32).round() as u32).min(frame.width - 1);
                    let fy = ((v * (frame.height - 1) as f32).round() as u32).min(frame.height - 1);
                    let idx = ((fy * frame.width + fx) * 4) as usize;
                    let r_pct = (frame.rgba[idx] as f32 / 2.55).round();
                    let g_pct = (frame.rgba[idx + 1] as f32 / 2.55).round();
                    let b_pct = (frame.rgba[idx + 2] as f32 / 2.55).round();
                    let badge_pos = pos + Vec2::new(16.0, 16.0);
                    let badge_r = Rect::from_min_size(badge_pos, Vec2::new(144.0, 36.0));
                    painter.rect_filled(badge_r, 4.0, Color32::from_rgba_unmultiplied(18, 18, 22, 235));
                    painter.rect_stroke(badge_r, 4.0, Stroke::new(1.0, Color32::from_rgb(60, 60, 68)));
                    painter.text(
                        badge_r.min + Vec2::new(8.0, 5.0),
                        egui::Align2::LEFT_TOP,
                        format!("R:{:.0}%  G:{:.0}%  B:{:.0}%", r_pct, g_pct, b_pct),
                        egui::FontId::proportional(11.0),
                        Color32::WHITE,
                    );
                    painter.text(
                        badge_r.min + Vec2::new(8.0, 20.0),
                        egui::Align2::LEFT_TOP,
                        "Click: Sample WB (Esc)",
                        egui::FontId::proportional(9.0),
                        Colors::TEXT_MID,
                    );
                }

                if resp.clicked() {
                    let pid = app.active_photo_id;
                    if let Some(raw) = app.ensure_loupe_raw(pid) {
                        let rx = ((u * (raw.width - 1) as f32).round() as u32).min(raw.width - 1);
                        let ry = ((v * (raw.height - 1) as f32).round() as u32).min(raw.height - 1);
                        let idx = ((ry * raw.width + rx) * 3) as usize;
                        let (temp, tint) = solve_wb_from_linear_rgb(
                            raw.rgb[idx],
                            raw.rgb[idx + 1],
                            raw.rgb[idx + 2],
                            raw.as_shot_temp,
                            raw.as_shot_tint,
                        );
                        let mut p = app.active_photo().params.clone();
                        p.white_balance = WhiteBalanceMode::Custom;
                        p.temperature = temp;
                        p.tint = tint;
                        app.commit_params_change("White Balance Selector", p);
                        app.wb_eyedropper_active = false;
                    }
                }
            }
        }
        return;
    }

    // 2. Detail 1:1 Loupe Sample Picker
    if app.detail_sample_tool_active {
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
        }
        if let (Some(pos), Some((u, v))) = (ptr_opt, norm_uv) {
            if dst.contains(pos) {
                painter.circle_stroke(pos, 14.0, Stroke::new(1.5, Color32::WHITE));
                painter.circle_stroke(pos, 14.0, Stroke::new(0.5, Color32::BLACK));
                painter.line_segment(
                    [Pos2::new(pos.x - 18.0, pos.y), Pos2::new(pos.x + 18.0, pos.y)],
                    Stroke::new(1.0, Color32::WHITE),
                );
                painter.line_segment(
                    [Pos2::new(pos.x, pos.y - 18.0), Pos2::new(pos.x, pos.y + 18.0)],
                    Stroke::new(1.0, Color32::WHITE),
                );

                let badge_pos = pos + Vec2::new(16.0, 16.0);
                let badge_r = Rect::from_min_size(badge_pos, Vec2::new(140.0, 24.0));
                painter.rect_filled(badge_r, 4.0, Color32::from_rgba_unmultiplied(18, 18, 22, 235));
                painter.rect_stroke(badge_r, 4.0, Stroke::new(1.0, Color32::from_rgb(60, 60, 68)));
                painter.text(
                    badge_r.center(),
                    egui::Align2::CENTER_CENTER,
                    "Sample 1:1 Detail Loupe",
                    egui::FontId::proportional(10.0),
                    Color32::WHITE,
                );

                if resp.clicked() {
                    app.detail_loupe_center = Vec2::new(u, v);
                    app.detail_sample_tool_active = false;
                }
            }
        }
        return;
    }

    // 3. Point Color Eyedropper
    if app.point_color_eyedropper_active {
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
        }
        if let (Some(pos), Some((u, v))) = (ptr_opt, norm_uv) {
            if dst.contains(pos) && resp.clicked() {
                if let Some(frame) = app.active_rendered_frame() {
                    let fx = ((u * (frame.width - 1) as f32).round() as u32).min(frame.width - 1);
                    let fy = ((v * (frame.height - 1) as f32).round() as u32).min(frame.height - 1);
                    let i = ((fy * frame.width + fx) * 4) as usize;
                    let (h, s, l) = rgb_to_hsl_deg(
                        frame.rgba[i] as f32 / 255.0,
                        frame.rgba[i + 1] as f32 / 255.0,
                        frame.rgba[i + 2] as f32 / 255.0,
                    );
                    let mut p = app.active_photo().params.clone();
                    p.point_colors.push(PointColorSwatch {
                        name: format!("Swatch {}", p.point_colors.len() + 1),
                        src_hue: h,
                        src_sat: s,
                        src_lum: l,
                        range_amount: 50.0,
                        ..Default::default()
                    });
                    app.selected_point_color_idx = p.point_colors.len() - 1;
                    app.point_color_eyedropper_active = false;
                    app.commit_params_change("Sample Point Color", p);
                }
            }
        }
        return;
    }

    // 3. Defringe Fringe Eyedropper
    if let Some(is_purple) = app.defringe_eyedropper_active {
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
        }
        if let (Some(pos), Some((u, v))) = (ptr_opt, norm_uv) {
            if dst.contains(pos) && resp.clicked() {
                if let Some(frame) = app.active_rendered_frame() {
                    let fx = ((u * (frame.width - 1) as f32).round() as u32).min(frame.width - 1);
                    let fy = ((v * (frame.height - 1) as f32).round() as u32).min(frame.height - 1);
                    let i = ((fy * frame.width + fx) * 4) as usize;
                    let (h, _, _) = rgb_to_hsl_deg(
                        frame.rgba[i] as f32 / 255.0,
                        frame.rgba[i + 1] as f32 / 255.0,
                        frame.rgba[i + 2] as f32 / 255.0,
                    );
                    let mut p = app.active_photo().params.clone();
                    if is_purple {
                        p.defringe_purple_amount = p.defringe_purple_amount.max(8.0);
                        p.defringe_purple_hue_lo = (h - 20.0).clamp(0.0, 360.0);
                        p.defringe_purple_hue_hi = (h + 20.0).clamp(0.0, 360.0);
                    } else {
                        p.defringe_green_amount = p.defringe_green_amount.max(8.0);
                        p.defringe_green_hue_lo = (h - 20.0).clamp(0.0, 360.0);
                        p.defringe_green_hue_hi = (h + 20.0).clamp(0.0, 360.0);
                    }
                    app.defringe_eyedropper_active = None;
                    app.commit_params_change("Defringe Selector", p);
                }
            }
        }
        return;
    }

    // 4. Lens Blur Subject Point Focus Picker & Depth Refinement Brush
    if app.lens_blur_focus_picker_active {
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
        }
        if let (Some(pos), Some((u, v))) = (ptr_opt, norm_uv) {
            if dst.contains(pos) && resp.clicked() {
                let pid = app.active_photo_id;
                if let Some(raw) = app.ensure_loupe_raw(pid) {
                    let d = raw.sample_depth(u, v) * 100.0;
                    let mut p = app.active_photo().params.clone();
                    p.lens_blur_active = true;
                    p.lens_blur_focal_range = [
                        (d - 22.0).clamp(0.0, 100.0),
                        (d - 12.0).clamp(0.0, 100.0),
                        (d + 12.0).clamp(0.0, 100.0),
                        (d + 22.0).clamp(0.0, 100.0),
                    ];
                    app.lens_blur_focus_picker_active = false;
                    app.commit_params_change("Lens Blur Point Focus", p);
                }
            }
        }
        return;
    }

    if let Some(is_focus) = app.lens_blur_brush_mode {
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
        }
        if let (Some(pos), Some((u, v))) = (ptr_opt, norm_uv) {
            if dst.contains(pos) {
                painter.circle_stroke(
                    pos,
                    app.lens_blur_brush_size * dst.width(),
                    Stroke::new(1.2, if is_focus { Color32::GREEN } else { Colors::ACCENT_AMBER }),
                );
                if resp.drag_started() || resp.clicked() {
                    let mut p = app.active_photo().params.clone();
                    p.lens_blur_refinement_strokes.push(LensBlurStroke {
                        is_focus,
                        amount: app.lens_blur_brush_flow,
                        radius: app.lens_blur_brush_size,
                        feather: app.lens_blur_brush_feather,
                        points: vec![(u, v)],
                    });
                    app.commit_params_change("Lens Blur Refinement", p);
                } else if resp.dragged() {
                    let mut p = app.active_photo().params.clone();
                    if let Some(last) = p.lens_blur_refinement_strokes.last_mut() {
                        last.points.push((u, v));
                        app.commit_params_change("Lens Blur Refinement", p);
                    }
                }
            }
        }
        return;
    }

    // 5. Active Develop Tool Overlays
    match app.develop_tool {
        DevelopTool::None => {
            let is_zoomed = app.loupe_zoom > 1.02;
            let cmd_down = ui.input(|i| i.modifiers.command || i.modifiers.ctrl);
            let alt_down = ui.input(|i| i.modifiers.alt);
            let can_marquee_zoom = !is_zoomed || cmd_down;

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
                            let scale_factor = (resp.rect.width() * 0.94 / crop_box.width())
                                .min(resp.rect.height() * 0.94 / crop_box.height());
                            let new_zoom = (app.loupe_zoom * scale_factor).clamp(1.0, 16.0);

                            let new_dst_size = Vec2::new(
                                dst.width() / app.loupe_zoom.max(0.1) * new_zoom,
                                dst.height() / app.loupe_zoom.max(0.1) * new_zoom,
                            );
                            let mut new_pan = -Vec2::new(
                                (uv_center.x - 0.5) * new_dst_size.x,
                                (uv_center.y - 0.5) * new_dst_size.y,
                            );
                            let max_pan = (new_dst_size - resp.rect.size() * 0.5).max(Vec2::ZERO);
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
                            app.loupe_zoom = 2.5;
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
                        painter.rect_filled(
                            marquee_rect,
                            0.0,
                            Color32::from_rgba_unmultiplied(255, 255, 255, 25),
                        );
                        painter.rect_stroke(
                            marquee_rect.expand(1.0),
                            0.0,
                            Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 0, 0, 180)),
                        );
                        painter.rect_stroke(
                            marquee_rect,
                            0.0,
                            Stroke::new(1.5, Color32::WHITE),
                        );

                        // Corner tick marks (Lightroom Classic styling)
                        let tick_len = 8.0_f32.min(marquee_rect.width() * 0.3).min(marquee_rect.height() * 0.3);
                        let tick_stroke = Stroke::new(2.0, Color32::WHITE);
                        painter.line_segment([marquee_rect.min, marquee_rect.min + Vec2::new(tick_len, 0.0)], tick_stroke);
                        painter.line_segment([marquee_rect.min, marquee_rect.min + Vec2::new(0.0, tick_len)], tick_stroke);
                        painter.line_segment([Pos2::new(marquee_rect.max.x, marquee_rect.min.y), Pos2::new(marquee_rect.max.x - tick_len, marquee_rect.min.y)], tick_stroke);
                        painter.line_segment([Pos2::new(marquee_rect.max.x, marquee_rect.min.y), Pos2::new(marquee_rect.max.x, marquee_rect.min.y + tick_len)], tick_stroke);
                        painter.line_segment([Pos2::new(marquee_rect.min.x, marquee_rect.max.y), Pos2::new(marquee_rect.min.x + tick_len, marquee_rect.max.y)], tick_stroke);
                        painter.line_segment([Pos2::new(marquee_rect.min.x, marquee_rect.max.y), Pos2::new(marquee_rect.min.x, marquee_rect.max.y - tick_len)], tick_stroke);
                        painter.line_segment([marquee_rect.max, marquee_rect.max - Vec2::new(tick_len, 0.0)], tick_stroke);
                        painter.line_segment([marquee_rect.max, marquee_rect.max - Vec2::new(0.0, tick_len)], tick_stroke);

                        // Dynamic zoom percentage pill
                        let scale_w = resp.rect.width() * 0.94 / marquee_rect.width().max(1.0);
                        let scale_h = resp.rect.height() * 0.94 / marquee_rect.height().max(1.0);
                        let preview_zoom = (app.loupe_zoom * scale_w.min(scale_h)).clamp(1.0, 16.0);
                        let badge_str = format!("{:.0}%", preview_zoom * 100.0);
                        let badge_pos = Pos2::new(
                            (marquee_rect.max.x + 8.0).min(resp.rect.max.x - 56.0),
                            (marquee_rect.max.y - 18.0).max(resp.rect.min.y + 6.0),
                        );
                        let b_rect = Rect::from_min_size(badge_pos, Vec2::new(48.0, 18.0));
                        painter.rect_filled(b_rect, 3.0, Color32::from_rgba_unmultiplied(20, 20, 24, 220));
                        painter.rect_stroke(b_rect, 3.0, Stroke::new(1.0, Color32::from_rgb(80, 80, 90)));
                        painter.text(b_rect.center(), egui::Align2::CENTER_CENTER, badge_str, egui::FontId::proportional(10.5), Color32::WHITE);
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
                    app.loupe_zoom = 2.5;
                    if let Some(pos) = ptr_opt {
                        let center = dst.center();
                        app.loupe_pan = (center - pos) * (app.loupe_zoom - 1.0);
                    }
                }
            } else if resp.dragged() && is_zoomed && !cmd_down {
                app.loupe_pan += resp.drag_delta();
            }
        }
        DevelopTool::TargetAdjust => {
            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
            }
            if resp.dragged() {
                if let Some((u, v)) = norm_uv {
                    let dy = -resp.drag_delta().y * 0.45;
                    let mut p = app.active_photo().params.clone();
                    if let Some(frame) = app.active_rendered_frame() {
                        let fx = ((u * (frame.width - 1) as f32).round() as u32).min(frame.width - 1);
                        let fy = ((v * (frame.height - 1) as f32).round() as u32).min(frame.height - 1);
                        let i = ((fy * frame.width + fx) * 4) as usize;
                        let (h, _, l) = rgb_to_hsl_deg(
                            frame.rgba[i] as f32 / 255.0,
                            frame.rgba[i + 1] as f32 / 255.0,
                            frame.rgba[i + 2] as f32 / 255.0,
                        );
                        match app.target_adjust_kind {
                            TargetAdjustKind::ParametricCurve => {
                                if l < 0.25 {
                                    p.parametric_shadows = (p.parametric_shadows + dy).clamp(-100.0, 100.0);
                                } else if l < 0.5 {
                                    p.parametric_darks = (p.parametric_darks + dy).clamp(-100.0, 100.0);
                                } else if l < 0.75 {
                                    p.parametric_lights = (p.parametric_lights + dy).clamp(-100.0, 100.0);
                                } else {
                                    p.parametric_highlights = (p.parametric_highlights + dy).clamp(-100.0, 100.0);
                                }
                            }
                            kind => {
                                let mut best_b = 0usize;
                                let mut best_d = 999.0_f32;
                                for (bi, &bh) in HSL_BAND_HUES.iter().enumerate() {
                                    let mut dh = (h - bh).abs();
                                    if dh > 180.0 {
                                        dh = 360.0 - dh;
                                    }
                                    if dh < best_d {
                                        best_d = dh;
                                        best_b = bi;
                                    }
                                }
                                let arr = match kind {
                                    TargetAdjustKind::Hue => &mut p.hue_adjustment,
                                    TargetAdjustKind::Saturation => &mut p.saturation_adjustment,
                                    _ => &mut p.luminance_adjustment,
                                };
                                arr[best_b] = (arr[best_b] + dy).clamp(-100.0, 100.0);
                            }
                        }
                    }
                    app.commit_params_change("Target Adjustment", p);
                }
            }
        }
        DevelopTool::Crop => draw_and_interact_crop_overlay(app, ui, painter, resp, dst),
        DevelopTool::Remove => draw_and_interact_remove_overlay(app, ui, painter, resp, dst, ptr_opt, norm_uv),
        DevelopTool::RedEye => draw_and_interact_redeye_overlay(app, ui, painter, resp, dst, norm_uv),
        DevelopTool::GuidedUpright => draw_and_interact_guided_upright(app, ui, painter, resp, dst, norm_uv),
        DevelopTool::Masking => draw_and_interact_masking_overlay(app, ui, painter, resp, dst, ptr_opt, norm_uv),
    }
}

fn draw_and_interact_crop_overlay(
    app: &mut LightRawApp,
    ui: &mut Ui,
    painter: &egui::Painter,
    resp: &egui::Response,
    dst: Rect,
) {
    let mut p = app.active_photo().params.clone();
    let crop_rect = Rect::from_min_max(
        Pos2::new(
            dst.min.x + p.crop_left * dst.width(),
            dst.min.y + p.crop_top * dst.height(),
        ),
        Pos2::new(
            dst.min.x + p.crop_right * dst.width(),
            dst.min.y + p.crop_bottom * dst.height(),
        ),
    );

    // 1. Keyboard Shortcuts & Gestures for Crop Mode (Lightroom Classic Workflow)
    let double_clicked_inside = resp.double_clicked() && resp.hover_pos().map_or(false, |pos| crop_rect.contains(pos));
    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
    let esc_pressed = ui.input(|i| i.key_pressed(egui::Key::Escape));
    let x_pressed = ui.input(|i| i.key_pressed(egui::Key::X));
    let o_pressed = ui.input(|i| i.key_pressed(egui::Key::O));
    let a_pressed = ui.input(|i| i.key_pressed(egui::Key::A));

    if enter_pressed || double_clicked_inside {
        app.develop_tool = DevelopTool::None;
        app.invalidate_photo_render_caches(app.active_photo_id);
        app.status_message = "Crop applied (Enter)".to_string();
        return;
    }

    if esc_pressed {
        app.develop_tool = DevelopTool::None;
        app.status_message = "Crop closed (Esc)".to_string();
        return;
    }

    if x_pressed {
        flip_crop_aspect_ratio(app, &mut p);
        app.commit_params_change("Flip Crop Orientation", p.clone());
        if app.crop_sync_all_photos {
            app.sync_crop_to_library(false);
        }
        app.status_message = "Crop orientation flipped (X)".to_string();
    }

    if o_pressed {
        p.crop_guide_overlay = match p.crop_guide_overlay {
            CropGuideOverlay::RuleOfThirds => CropGuideOverlay::Grid,
            CropGuideOverlay::Grid => CropGuideOverlay::Diagonal,
            CropGuideOverlay::Diagonal => CropGuideOverlay::Triangle,
            CropGuideOverlay::Triangle => CropGuideOverlay::GoldenRatio,
            CropGuideOverlay::GoldenRatio => CropGuideOverlay::GoldenSpiral,
            CropGuideOverlay::GoldenSpiral => CropGuideOverlay::AspectRatios,
            CropGuideOverlay::AspectRatios => CropGuideOverlay::RuleOfThirds,
        };
        app.active_photo_mut().params.crop_guide_overlay = p.crop_guide_overlay;
        app.status_message = format!("Crop Overlay: {} (O)", p.crop_guide_overlay.label());
    }

    if a_pressed {
        p.crop_constrain_aspect_ratio = !p.crop_constrain_aspect_ratio;
        app.active_photo_mut().params.crop_constrain_aspect_ratio = p.crop_constrain_aspect_ratio;
        if app.crop_sync_all_photos {
            app.sync_crop_to_library(false);
        }
        app.status_message = if p.crop_constrain_aspect_ratio {
            "Crop Aspect Ratio Locked 🔒 (A)".to_string()
        } else {
            "Crop Aspect Ratio Unlocked 🔓 (A)".to_string()
        };
    }

    // 2. Dim outside crop box
    let dim = Color32::from_rgba_unmultiplied(0, 0, 0, 160);
    painter.rect_filled(Rect::from_min_max(dst.min, Pos2::new(dst.max.x, crop_rect.min.y)), 0.0, dim);
    painter.rect_filled(Rect::from_min_max(Pos2::new(dst.min.x, crop_rect.max.y), dst.max), 0.0, dim);
    painter.rect_filled(
        Rect::from_min_max(
            Pos2::new(dst.min.x, crop_rect.min.y),
            Pos2::new(crop_rect.min.x, crop_rect.max.y),
        ),
        0.0,
        dim,
    );
    painter.rect_filled(
        Rect::from_min_max(
            Pos2::new(crop_rect.max.x, crop_rect.min.y),
            Pos2::new(dst.max.x, crop_rect.max.y),
        ),
        0.0,
        dim,
    );

    // 3. Draw Crisp Outer Crop Frame Border
    painter.rect_stroke(crop_rect, 0.0, Stroke::new(1.5, Color32::WHITE));

    // 4. Composition Grid Overlay (Rule of Thirds, Grid, Diagonal, Golden Ratio)
    let show_grid = match p.crop_tool_overlay_mode {
        ToolOverlayMode::Always => true,
        ToolOverlayMode::Never => false,
        ToolOverlayMode::Auto => true,
    };
    if show_grid {
        let grid_col = Color32::from_rgba_unmultiplied(255, 255, 255, 115);
        match p.crop_guide_overlay {
            CropGuideOverlay::Diagonal | CropGuideOverlay::Triangle => {
                painter.line_segment([crop_rect.left_top(), crop_rect.right_bottom()], Stroke::new(0.9, grid_col));
                painter.line_segment([crop_rect.right_top(), crop_rect.left_bottom()], Stroke::new(0.9, grid_col));
            }
            CropGuideOverlay::GoldenRatio | CropGuideOverlay::GoldenSpiral => {
                for f in [0.382_f32, 0.618_f32] {
                    let gx = crop_rect.min.x + crop_rect.width() * f;
                    let gy = crop_rect.min.y + crop_rect.height() * f;
                    painter.line_segment([Pos2::new(gx, crop_rect.min.y), Pos2::new(gx, crop_rect.max.y)], Stroke::new(0.9, grid_col));
                    painter.line_segment([Pos2::new(crop_rect.min.x, gy), Pos2::new(crop_rect.max.x, gy)], Stroke::new(0.9, grid_col));
                }
            }
            CropGuideOverlay::Grid => {
                for i in 1..6 {
                    let f = i as f32 / 6.0;
                    let gx = crop_rect.min.x + crop_rect.width() * f;
                    let gy = crop_rect.min.y + crop_rect.height() * f;
                    painter.line_segment([Pos2::new(gx, crop_rect.min.y), Pos2::new(gx, crop_rect.max.y)], Stroke::new(0.7, grid_col));
                    painter.line_segment([Pos2::new(crop_rect.min.x, gy), Pos2::new(crop_rect.max.x, gy)], Stroke::new(0.7, grid_col));
                }
            }
            _ => {
                for i in 1..3 {
                    let f = i as f32 / 3.0;
                    let gx = crop_rect.min.x + crop_rect.width() * f;
                    let gy = crop_rect.min.y + crop_rect.height() * f;
                    painter.line_segment([Pos2::new(gx, crop_rect.min.y), Pos2::new(gx, crop_rect.max.y)], Stroke::new(0.9, grid_col));
                    painter.line_segment([Pos2::new(crop_rect.min.x, gy), Pos2::new(crop_rect.max.x, gy)], Stroke::new(0.9, grid_col));
                }
            }
        }
    }

    // 5. Authentic Adobe Lightroom Classic Corner L-Brackets & Midpoint Grab Bars
    let corners = [
        crop_rect.left_top(),
        crop_rect.right_top(),
        crop_rect.right_bottom(),
        crop_rect.left_bottom(),
    ];
    let mid_top = Pos2::new(crop_rect.center().x, crop_rect.min.y);
    let mid_bot = Pos2::new(crop_rect.center().x, crop_rect.max.y);
    let mid_left = Pos2::new(crop_rect.min.x, crop_rect.center().y);
    let mid_right = Pos2::new(crop_rect.max.x, crop_rect.center().y);

    let bracket_len = 16.0_f32;
    let b_stroke = Stroke::new(3.0, Color32::WHITE);
    let s_stroke = Stroke::new(4.5, Color32::from_rgba_unmultiplied(0, 0, 0, 140));

    // TL Corner Bracket
    painter.line_segment([corners[0] - Vec2::new(1.0, 0.0), corners[0] + Vec2::new(bracket_len, 0.0)], s_stroke);
    painter.line_segment([corners[0] - Vec2::new(0.0, 1.0), corners[0] + Vec2::new(0.0, bracket_len)], s_stroke);
    painter.line_segment([corners[0], corners[0] + Vec2::new(bracket_len, 0.0)], b_stroke);
    painter.line_segment([corners[0], corners[0] + Vec2::new(0.0, bracket_len)], b_stroke);

    // TR Corner Bracket
    painter.line_segment([corners[1] + Vec2::new(1.0, 0.0), corners[1] - Vec2::new(bracket_len, 0.0)], s_stroke);
    painter.line_segment([corners[1] - Vec2::new(0.0, 1.0), corners[1] + Vec2::new(0.0, bracket_len)], s_stroke);
    painter.line_segment([corners[1], corners[1] - Vec2::new(bracket_len, 0.0)], b_stroke);
    painter.line_segment([corners[1], corners[1] + Vec2::new(0.0, bracket_len)], b_stroke);

    // BR Corner Bracket
    painter.line_segment([corners[2] + Vec2::new(1.0, 0.0), corners[2] - Vec2::new(bracket_len, 0.0)], s_stroke);
    painter.line_segment([corners[2] + Vec2::new(0.0, 1.0), corners[2] - Vec2::new(0.0, bracket_len)], s_stroke);
    painter.line_segment([corners[2], corners[2] - Vec2::new(bracket_len, 0.0)], b_stroke);
    painter.line_segment([corners[2], corners[2] - Vec2::new(0.0, bracket_len)], b_stroke);

    // BL Corner Bracket
    painter.line_segment([corners[3] - Vec2::new(1.0, 0.0), corners[3] + Vec2::new(bracket_len, 0.0)], s_stroke);
    painter.line_segment([corners[3] + Vec2::new(0.0, 1.0), corners[3] - Vec2::new(0.0, bracket_len)], s_stroke);
    painter.line_segment([corners[3], corners[3] + Vec2::new(bracket_len, 0.0)], b_stroke);
    painter.line_segment([corners[3], corners[3] - Vec2::new(0.0, bracket_len)], b_stroke);

    // Midpoint Edge Grab Bars
    let mid_bar_len = 10.0_f32;
    painter.line_segment([mid_top - Vec2::new(mid_bar_len, 0.0), mid_top + Vec2::new(mid_bar_len, 0.0)], s_stroke);
    painter.line_segment([mid_top - Vec2::new(mid_bar_len, 0.0), mid_top + Vec2::new(mid_bar_len, 0.0)], b_stroke);

    painter.line_segment([mid_bot - Vec2::new(mid_bar_len, 0.0), mid_bot + Vec2::new(mid_bar_len, 0.0)], s_stroke);
    painter.line_segment([mid_bot - Vec2::new(mid_bar_len, 0.0), mid_bot + Vec2::new(mid_bar_len, 0.0)], b_stroke);

    painter.line_segment([mid_left - Vec2::new(0.0, mid_bar_len), mid_left + Vec2::new(0.0, mid_bar_len)], s_stroke);
    painter.line_segment([mid_left - Vec2::new(0.0, mid_bar_len), mid_left + Vec2::new(0.0, mid_bar_len)], b_stroke);

    painter.line_segment([mid_right - Vec2::new(0.0, mid_bar_len), mid_right + Vec2::new(0.0, mid_bar_len)], s_stroke);
    painter.line_segment([mid_right - Vec2::new(0.0, mid_bar_len), mid_right + Vec2::new(0.0, mid_bar_len)], b_stroke);

    // 6. Dynamic Context-Sensitive Cursor Icon with Exact Distance Gating
    let detected_handle = app.crop_drag_handle.or_else(|| {
        if !resp.hovered() {
            return None;
        }
        let pos = resp.hover_pos()?;
        for (idx, c) in corners.iter().enumerate() {
            if pos.distance(*c) < 18.0 {
                return Some(idx as u8);
            }
        }
        if pos.distance(mid_top) < 14.0 {
            return Some(5);
        }
        if pos.distance(mid_bot) < 14.0 {
            return Some(6);
        }
        if pos.distance(mid_left) < 14.0 {
            return Some(7);
        }
        if pos.distance(mid_right) < 14.0 {
            return Some(8);
        }
        if crop_rect.shrink(12.0).contains(pos) {
            return Some(4);
        }
        // Rotate zone: near corners outside crop_rect
        for c in &corners {
            let d = pos.distance(*c);
            if d >= 18.0 && d < 42.0 {
                return Some(9);
            }
        }
        None
    });

    if resp.hovered() {
        match detected_handle {
            Some(0) => ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNorthWest),
            Some(1) => ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNorthEast),
            Some(2) => ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeSouthEast),
            Some(3) => ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeSouthWest),
            Some(5) => ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNorth),
            Some(6) => ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeSouth),
            Some(7) => ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeWest),
            Some(8) => ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeEast),
            Some(4) => {
                if resp.is_pointer_button_down_on() || resp.dragged() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                } else {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Move);
                }
            }
            Some(9) => ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair),
            _ => ui.ctx().set_cursor_icon(egui::CursorIcon::Default),
        }
    }

    // 7. Handle Drag Initiation
    if resp.drag_started() {
        app.crop_drag_handle = detected_handle;
    }

    // 7. Interactive Dragging with Exact Mathematical Anchoring
    let (img_w, img_h) = app.active_photo_pixel_size();
    let norm_ratio_opt = if p.crop_constrain_aspect_ratio {
        resolve_aspect_ratio_preset(
            p.crop_aspect_preset,
            img_w,
            img_h,
            p.crop_aspect_flipped,
            p.crop_custom_ratio,
        )
        .map(|px_asp| px_asp * (img_h / img_w))
    } else {
        None
    };

    if resp.dragged() {
        let dx = resp.drag_delta().x / dst.width().max(1.0);
        let dy = resp.drag_delta().y / dst.height().max(1.0);
        let min_sz = 0.05_f32;
        let mut changed = false;

        match app.crop_drag_handle {
            // 0 = Top-Left (Anchor: Bottom-Right)
            Some(0) => {
                let x1 = p.crop_right;
                let y1 = p.crop_bottom;
                if let Some(r) = norm_ratio_opt {
                    let old_w = x1 - p.crop_left;
                    let max_w = x1.min(y1 * r);
                    let min_w = min_sz.max(min_sz * r);
                    let cand_w = if dx.abs() >= (dy * r).abs() {
                        old_w - dx
                    } else {
                        old_w - dy * r
                    };
                    let w = cand_w.clamp(min_w, max_w);
                    let h = w / r;
                    p.crop_left = x1 - w;
                    p.crop_top = y1 - h;
                } else {
                    p.crop_left = (p.crop_left + dx).clamp(0.0, x1 - min_sz);
                    p.crop_top = (p.crop_top + dy).clamp(0.0, y1 - min_sz);
                    p.crop_aspect_preset = AspectRatioPreset::Custom;
                }
                changed = true;
            }
            // 1 = Top-Right (Anchor: Bottom-Left)
            Some(1) => {
                let x0 = p.crop_left;
                let y1 = p.crop_bottom;
                if let Some(r) = norm_ratio_opt {
                    let old_w = p.crop_right - x0;
                    let max_w = (1.0 - x0).min(y1 * r);
                    let min_w = min_sz.max(min_sz * r);
                    let cand_w = if dx.abs() >= (dy * r).abs() {
                        old_w + dx
                    } else {
                        old_w - dy * r
                    };
                    let w = cand_w.clamp(min_w, max_w);
                    let h = w / r;
                    p.crop_right = x0 + w;
                    p.crop_top = y1 - h;
                } else {
                    p.crop_right = (p.crop_right + dx).clamp(x0 + min_sz, 1.0);
                    p.crop_top = (p.crop_top + dy).clamp(0.0, y1 - min_sz);
                    p.crop_aspect_preset = AspectRatioPreset::Custom;
                }
                changed = true;
            }
            // 2 = Bottom-Right (Anchor: Top-Left)
            Some(2) => {
                let x0 = p.crop_left;
                let y0 = p.crop_top;
                if let Some(r) = norm_ratio_opt {
                    let old_w = p.crop_right - x0;
                    let max_w = (1.0 - x0).min((1.0 - y0) * r);
                    let min_w = min_sz.max(min_sz * r);
                    let cand_w = if dx.abs() >= (dy * r).abs() {
                        old_w + dx
                    } else {
                        old_w + dy * r
                    };
                    let w = cand_w.clamp(min_w, max_w);
                    let h = w / r;
                    p.crop_right = x0 + w;
                    p.crop_bottom = y0 + h;
                } else {
                    p.crop_right = (p.crop_right + dx).clamp(x0 + min_sz, 1.0);
                    p.crop_bottom = (p.crop_bottom + dy).clamp(y0 + min_sz, 1.0);
                    p.crop_aspect_preset = AspectRatioPreset::Custom;
                }
                changed = true;
            }
            // 3 = Bottom-Left (Anchor: Top-Right)
            Some(3) => {
                let x1 = p.crop_right;
                let y0 = p.crop_top;
                if let Some(r) = norm_ratio_opt {
                    let old_w = x1 - p.crop_left;
                    let max_w = x1.min((1.0 - y0) * r);
                    let min_w = min_sz.max(min_sz * r);
                    let cand_w = if dx.abs() >= (dy * r).abs() {
                        old_w - dx
                    } else {
                        old_w + dy * r
                    };
                    let w = cand_w.clamp(min_w, max_w);
                    let h = w / r;
                    p.crop_left = x1 - w;
                    p.crop_bottom = y0 + h;
                } else {
                    p.crop_left = (p.crop_left + dx).clamp(0.0, x1 - min_sz);
                    p.crop_bottom = (p.crop_bottom + dy).clamp(y0 + min_sz, 1.0);
                    p.crop_aspect_preset = AspectRatioPreset::Custom;
                }
                changed = true;
            }
            // 4 = Move entire crop box
            Some(4) => {
                let w = p.crop_right - p.crop_left;
                let h = p.crop_bottom - p.crop_top;
                p.crop_left = (p.crop_left + dx).clamp(0.0, 1.0 - w);
                p.crop_right = p.crop_left + w;
                p.crop_top = (p.crop_top + dy).clamp(0.0, 1.0 - h);
                p.crop_bottom = p.crop_top + h;
                changed = true;
            }
            // 5 = Top Edge (Anchor: Bottom edge)
            Some(5) => {
                let y1 = p.crop_bottom;
                if let Some(r) = norm_ratio_opt {
                    let old_h = y1 - p.crop_top;
                    let max_h = y1.min(1.0 / r);
                    let min_h = min_sz.max(min_sz / r);
                    let h = (old_h - dy).clamp(min_h, max_h);
                    let w = h * r;
                    let mut cx = 0.5 * (p.crop_left + p.crop_right);
                    if cx - w * 0.5 < 0.0 { cx = w * 0.5; }
                    if cx + w * 0.5 > 1.0 { cx = 1.0 - w * 0.5; }
                    p.crop_left = (cx - w * 0.5).clamp(0.0, (1.0 - w).max(0.0));
                    p.crop_right = (p.crop_left + w).min(1.0);
                    p.crop_top = y1 - h;
                } else {
                    p.crop_top = (p.crop_top + dy).clamp(0.0, y1 - min_sz);
                    p.crop_aspect_preset = AspectRatioPreset::Custom;
                }
                changed = true;
            }
            // 6 = Bottom Edge (Anchor: Top edge)
            Some(6) => {
                let y0 = p.crop_top;
                if let Some(r) = norm_ratio_opt {
                    let old_h = p.crop_bottom - y0;
                    let max_h = (1.0 - y0).min(1.0 / r);
                    let min_h = min_sz.max(min_sz / r);
                    let h = (old_h + dy).clamp(min_h, max_h);
                    let w = h * r;
                    let mut cx = 0.5 * (p.crop_left + p.crop_right);
                    if cx - w * 0.5 < 0.0 { cx = w * 0.5; }
                    if cx + w * 0.5 > 1.0 { cx = 1.0 - w * 0.5; }
                    p.crop_left = (cx - w * 0.5).clamp(0.0, (1.0 - w).max(0.0));
                    p.crop_right = (p.crop_left + w).min(1.0);
                    p.crop_bottom = y0 + h;
                } else {
                    p.crop_bottom = (p.crop_bottom + dy).clamp(y0 + min_sz, 1.0);
                    p.crop_aspect_preset = AspectRatioPreset::Custom;
                }
                changed = true;
            }
            // 7 = Left Edge (Anchor: Right edge)
            Some(7) => {
                let x1 = p.crop_right;
                if let Some(r) = norm_ratio_opt {
                    let old_w = x1 - p.crop_left;
                    let max_w = x1.min(r);
                    let min_w = min_sz.max(min_sz * r);
                    let w = (old_w - dx).clamp(min_w, max_w);
                    let h = w / r;
                    let mut cy = 0.5 * (p.crop_top + p.crop_bottom);
                    if cy - h * 0.5 < 0.0 { cy = h * 0.5; }
                    if cy + h * 0.5 > 1.0 { cy = 1.0 - h * 0.5; }
                    p.crop_top = (cy - h * 0.5).clamp(0.0, (1.0 - h).max(0.0));
                    p.crop_bottom = (p.crop_top + h).min(1.0);
                    p.crop_left = x1 - w;
                } else {
                    p.crop_left = (p.crop_left + dx).clamp(0.0, x1 - min_sz);
                    p.crop_aspect_preset = AspectRatioPreset::Custom;
                }
                changed = true;
            }
            // 8 = Right Edge (Anchor: Left edge)
            Some(8) => {
                let x0 = p.crop_left;
                if let Some(r) = norm_ratio_opt {
                    let old_w = p.crop_right - x0;
                    let max_w = (1.0 - x0).min(r);
                    let min_w = min_sz.max(min_sz * r);
                    let w = (old_w + dx).clamp(min_w, max_w);
                    let h = w / r;
                    let mut cy = 0.5 * (p.crop_top + p.crop_bottom);
                    if cy - h * 0.5 < 0.0 { cy = h * 0.5; }
                    if cy + h * 0.5 > 1.0 { cy = 1.0 - h * 0.5; }
                    p.crop_top = (cy - h * 0.5).clamp(0.0, (1.0 - h).max(0.0));
                    p.crop_bottom = (p.crop_top + h).min(1.0);
                    p.crop_right = x0 + w;
                } else {
                    p.crop_right = (p.crop_right + dx).clamp(x0 + min_sz, 1.0);
                    p.crop_aspect_preset = AspectRatioPreset::Custom;
                }
                changed = true;
            }
            // 9 = Rotate / Straighten
            Some(9) => {
                p.crop_angle = (p.crop_angle + dx * 30.0).clamp(-45.0, 45.0);
                changed = true;
            }
            _ => {}
        }

        if changed {
            if !p.crop_constrain_aspect_ratio {
                p.crop_custom_ratio = (
                    (p.crop_right - p.crop_left) * img_w,
                    (p.crop_bottom - p.crop_top) * img_h,
                );
            }
            // Live update without flushing heavy render caches during active mouse drag!
            app.active_photo_mut().params = p;
            ui.ctx().request_repaint();
        }
    }

    // 8. Commit Single History Step on Drag Release
    if resp.drag_stopped() {
        if app.crop_drag_handle.is_some() {
            let p_commit = app.active_photo().params.clone();
            app.commit_params_change("Adjust Crop", p_commit);
            if app.crop_sync_all_photos {
                app.sync_crop_to_library(false);
            }
            app.crop_drag_handle = None;
        }
    }
}

fn draw_and_interact_remove_overlay(
    app: &mut LightRawApp,
    ui: &mut Ui,
    painter: &egui::Painter,
    resp: &egui::Response,
    dst: Rect,
    ptr_opt: Option<Pos2>,
    norm_uv: Option<(f32, f32)>,
) {
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
    }
    if let Some(pos) = ptr_opt {
        if dst.contains(pos) {
            painter.circle_stroke(
                pos,
                app.retouch_brush_radius * dst.width(),
                Stroke::new(1.2, Color32::WHITE),
            );
        }
    }

    let p = app.active_photo().params.clone();
    for (idx, area) in p.retouch_areas.iter().enumerate() {
        let tc = Pos2::new(
            dst.min.x + area.target_x * dst.width(),
            dst.min.y + area.target_y * dst.height(),
        );
        let sc = Pos2::new(
            dst.min.x + area.source_x * dst.width(),
            dst.min.y + area.source_y * dst.height(),
        );
        let is_sel = app.selected_retouch_idx == Some(idx);
        let col = if is_sel { Colors::ACCENT_BLUE } else { Color32::WHITE };
        painter.circle_stroke(tc, area.radius * dst.width(), Stroke::new(1.5, col));
        if is_sel && area.mode != RetouchMode::Remove {
            painter.line_segment([tc, sc], Stroke::new(1.0, Color32::from_rgb(200, 220, 255)));
            painter.circle_stroke(sc, area.radius * dst.width(), Stroke::new(1.2, Color32::GREEN));
        }
    }

    let Some((u, v)) = norm_uv else {
        return;
    };
    let Some(pos) = ptr_opt else {
        return;
    };
    if !dst.contains(pos) {
        return;
    }

    if resp.drag_started() {
        // Check if dragging an existing source pin
        if let Some(sel_idx) = app.selected_retouch_idx {
            if let Some(area) = p.retouch_areas.get(sel_idx) {
                let sc = Pos2::new(
                    dst.min.x + area.source_x * dst.width(),
                    dst.min.y + area.source_y * dst.height(),
                );
                if pos.distance(sc) <= (area.radius * dst.width()).max(14.0) {
                    app.dragging_retouch_source = Some(sel_idx);
                    return;
                }
            }
        }
        // Start new brush stroke
        let mut next = p;
        let src_x = (u + 0.06).clamp(0.05, 0.95);
        next.retouch_areas.push(RetouchArea {
            mode: app.retouch_mode_ui,
            target_x: u,
            target_y: v,
            points: vec![(u, v)],
            source_x: src_x,
            source_y: v,
            radius: app.retouch_brush_radius,
            feather: app.retouch_feather,
            opacity: app.retouch_opacity,
        });
        app.selected_retouch_idx = Some(next.retouch_areas.len() - 1);
        app.active_stroke_in_progress = true;
        app.commit_params_change("Heal / Remove Brush", next);
    } else if resp.dragged() {
        if let Some(src_idx) = app.dragging_retouch_source {
            let mut next = p;
            if let Some(area) = next.retouch_areas.get_mut(src_idx) {
                area.source_x = u;
                area.source_y = v;
                app.commit_params_change("Move Heal Source", next);
            }
        } else if app.active_stroke_in_progress {
            let mut next = p;
            if let Some(last) = next.retouch_areas.last_mut() {
                last.points.push((u, v));
                app.commit_params_change("Heal / Remove Brush", next);
            }
        }
    } else if resp.drag_stopped() {
        app.dragging_retouch_source = None;
        app.active_stroke_in_progress = false;
    } else if resp.clicked() {
        // Click existing pin to select, or place single spot
        for (idx, area) in p.retouch_areas.iter().enumerate() {
            let tc = Pos2::new(
                dst.min.x + area.target_x * dst.width(),
                dst.min.y + area.target_y * dst.height(),
            );
            if pos.distance(tc) <= (area.radius * dst.width()).max(12.0) {
                app.selected_retouch_idx = Some(idx);
                return;
            }
        }
        let mut next = p;
        next.retouch_areas.push(RetouchArea {
            mode: app.retouch_mode_ui,
            target_x: u,
            target_y: v,
            points: vec![(u, v)],
            source_x: (u + 0.06).clamp(0.05, 0.95),
            source_y: v,
            radius: app.retouch_brush_radius,
            feather: app.retouch_feather,
            opacity: app.retouch_opacity,
        });
        app.selected_retouch_idx = Some(next.retouch_areas.len() - 1);
        app.commit_params_change("Remove Spot", next);
    }
}

fn draw_and_interact_redeye_overlay(
    app: &mut LightRawApp,
    ui: &mut Ui,
    painter: &egui::Painter,
    resp: &egui::Response,
    dst: Rect,
    norm_uv: Option<(f32, f32)>,
) {
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
    }
    let p = app.active_photo().params.clone();
    for (idx, sp) in p.red_eye_spots.iter().enumerate() {
        let c = Pos2::new(
            dst.min.x + sp.center_x * dst.width(),
            dst.min.y + sp.center_y * dst.height(),
        );
        let col = if app.selected_redeye_idx == Some(idx) {
            Colors::ACCENT_BLUE
        } else {
            Color32::WHITE
        };
        painter.circle_stroke(c, sp.radius * dst.width(), Stroke::new(1.5, col));
    }

    if let Some((u, v)) = norm_uv {
        if resp.clicked() {
            let mut next = p;
            next.red_eye_spots.push(RedEyeSpot {
                mode: app.redeye_mode_ui,
                center_x: u,
                center_y: v,
                radius: 0.025,
                pupil_size: app.redeye_pupil_size,
                darken: app.redeye_darken,
                add_catchlight: app.redeye_catchlight,
            });
            app.selected_redeye_idx = Some(next.red_eye_spots.len() - 1);
            app.commit_params_change("Red-Eye Correction", next);
        }
    }
}

fn draw_and_interact_guided_upright(
    app: &mut LightRawApp,
    ui: &mut Ui,
    painter: &egui::Painter,
    resp: &egui::Response,
    dst: Rect,
    norm_uv: Option<(f32, f32)>,
) {
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
    }
    let p = app.active_photo().params.clone();
    for &([x1, y1], [x2, y2]) in &p.guided_upright_lines {
        let a = Pos2::new(dst.min.x + x1 * dst.width(), dst.min.y + y1 * dst.height());
        let b = Pos2::new(dst.min.x + x2 * dst.width(), dst.min.y + y2 * dst.height());
        painter.line_segment([a, b], Stroke::new(2.0, Color32::from_rgb(0, 225, 255)));
        painter.circle_filled(a, 4.0, Color32::WHITE);
        painter.circle_filled(b, 4.0, Color32::WHITE);
    }

    if let (Some((sx, sy)), Some((u, v))) = (app.guided_upright_drag_start, norm_uv) {
        let a = Pos2::new(dst.min.x + sx * dst.width(), dst.min.y + sy * dst.height());
        let b = Pos2::new(dst.min.x + u * dst.width(), dst.min.y + v * dst.height());
        painter.line_segment([a, b], Stroke::new(1.5, Colors::ACCENT_AMBER));
    }

    if let Some((u, v)) = norm_uv {
        if resp.drag_started() {
            app.guided_upright_drag_start = Some((u, v));
        } else if resp.drag_stopped() {
            if let Some((sx, sy)) = app.guided_upright_drag_start.take() {
                if ((u - sx).hypot(v - sy)) > 0.03 {
                    let mut next = p;
                    if next.guided_upright_lines.len() >= 4 {
                        next.guided_upright_lines.remove(0);
                    }
                    next.guided_upright_lines.push(([sx, sy], [u, v]));
                    next.perspective_upright = crate::params::UprightMode::Guided;
                    app.commit_params_change("Add Guided Upright Line", next);
                }
            }
        }
    }
}

fn draw_and_interact_masking_overlay(
    app: &mut LightRawApp,
    ui: &mut Ui,
    painter: &egui::Painter,
    resp: &egui::Response,
    dst: Rect,
    ptr_opt: Option<Pos2>,
    norm_uv: Option<(f32, f32)>,
) {
    let Some(m_idx) = app.selected_mask_idx else {
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
        }
        return;
    };
    let mut p = app.active_photo().params.clone();
    let Some(mg) = p.mask_groups.get_mut(m_idx) else {
        return;
    };
    let c_idx = app
        .selected_component_idx
        .min(mg.components.len().saturating_sub(1));
    let Some(comp) = mg.components.get_mut(c_idx) else {
        return;
    };

    match &mut comp.kind {
        MaskPrimitiveKind::Brush { points, radius, .. } => {
            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
            }
            if let Some(pos) = ptr_opt {
                if dst.contains(pos) {
                    painter.circle_stroke(pos, *radius * dst.width(), Stroke::new(1.2, Color32::WHITE));
                }
            }
            if let Some((u, v)) = norm_uv {
                if (resp.drag_started() || resp.dragged() || resp.clicked())
                    && ptr_opt.map_or(false, |pos| dst.contains(pos))
                {
                    let should_add = points
                        .last()
                        .map(|&(lx, ly)| (u - lx).hypot(v - ly) > (*radius * 0.18).max(0.004))
                        .unwrap_or(true);
                    if should_add {
                        points.push((u, v));
                        app.commit_params_change("Paint Mask Brush", p);
                    }
                }
            }
        }
        MaskPrimitiveKind::LinearGradient { x1, y1, x2, y2 } => {
            if resp.hovered() {
                ui.ctx().set_cursor_icon(if resp.dragged() { egui::CursorIcon::Grabbing } else { egui::CursorIcon::Crosshair });
            }
            let p1 = Pos2::new(dst.min.x + *x1 * dst.width(), dst.min.y + *y1 * dst.height());
            let p2 = Pos2::new(dst.min.x + *x2 * dst.width(), dst.min.y + *y2 * dst.height());
            painter.line_segment([p1, p2], Stroke::new(1.6, Color32::WHITE));
            painter.circle_filled(p1, 5.5, Colors::ACCENT_BLUE);
            painter.circle_filled(p2, 5.5, Color32::WHITE);

            if resp.dragged() {
                if let (Some(pos), Some((u, v))) = (ptr_opt, norm_uv) {
                    if pos.distance(p1) <= pos.distance(p2) {
                        *x1 = u;
                        *y1 = v;
                    } else {
                        *x2 = u;
                        *y2 = v;
                    }
                    app.commit_params_change("Move Linear Gradient", p);
                }
            }
        }
        MaskPrimitiveKind::RadialGradient { cx, cy, rx, ry, .. } => {
            if resp.hovered() {
                ui.ctx().set_cursor_icon(if resp.dragged() { egui::CursorIcon::Grabbing } else { egui::CursorIcon::Crosshair });
            }
            let center = Pos2::new(dst.min.x + *cx * dst.width(), dst.min.y + *cy * dst.height());
            let rad_px = Vec2::new(*rx * dst.width(), *ry * dst.height());
            painter.rect_stroke(
                Rect::from_center_size(center, rad_px * 2.0),
                rad_px.x.min(rad_px.y),
                Stroke::new(1.4, Color32::WHITE),
            );
            painter.circle_filled(center, 5.5, Colors::ACCENT_BLUE);
            let edge_handle = Pos2::new(center.x + rad_px.x, center.y);
            painter.circle_filled(edge_handle, 5.0, Color32::WHITE);

            if resp.dragged() {
                if let (Some(pos), Some((u, v))) = (ptr_opt, norm_uv) {
                    if pos.distance(edge_handle) < 28.0 {
                        *rx = (u - *cx).abs().clamp(0.03, 0.95);
                        *ry = (*ry + resp.drag_delta().y / dst.height().max(1.0)).clamp(0.03, 0.95);
                    } else {
                        *cx = u;
                        *cy = v;
                    }
                    app.commit_params_change("Adjust Radial Mask", p);
                }
            }
        }
        MaskPrimitiveKind::ColorRange {
            target_hue,
            target_sat,
            ..
        } => {
            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
            }
            if resp.clicked() {
                if let Some((u, v)) = norm_uv {
                    let pid = app.active_photo_id;
                    if let Some(raw) = app.ensure_loupe_raw(pid) {
                        let rx = ((u * (raw.width - 1) as f32).round() as u32).min(raw.width - 1);
                        let ry = ((v * (raw.height - 1) as f32).round() as u32).min(raw.height - 1);
                        let idx = ((ry * raw.width + rx) * 3) as usize;
                        let sr = crate::pipeline::linear_to_srgb(raw.rgb[idx]);
                        let sg = crate::pipeline::linear_to_srgb(raw.rgb[idx + 1]);
                        let sb = crate::pipeline::linear_to_srgb(raw.rgb[idx + 2]);
                        let (h, s, _) = rgb_to_hsl_deg(sr, sg, sb);
                        *target_hue = h;
                        *target_sat = s;
                        app.commit_params_change("Sample Color Range Mask", p);
                    }
                }
            }
        }
        _ => {}
    }
}

pub fn render_develop_toolbar(app: &mut LightRawApp, ui: &mut Ui) {
    egui::Frame::none()
        .fill(Colors::BG_TOOLBAR)
        .inner_margin(egui::Margin::symmetric(10.0, 4.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;

                let is_single = app.before_after_mode == BeforeAfterMode::Single;
                if ui
                    .selectable_label(is_single, "▣")
                    .on_hover_text("Loupe View (D)")
                    .clicked()
                {
                    app.before_after_mode = BeforeAfterMode::Single;
                }

                let is_ref = app.before_after_mode == BeforeAfterMode::ReferenceView;
                if ui
                    .selectable_label(is_ref, "R|A")
                    .on_hover_text("Reference View (Shift+R)")
                    .clicked()
                {
                    app.before_after_mode = if is_ref {
                        BeforeAfterMode::Single
                    } else {
                        if app.reference_photo_id.is_none() {
                            app.reference_photo_id = Some(app.active_photo_id);
                        }
                        BeforeAfterMode::ReferenceView
                    };
                }

                if app.before_after_mode == BeforeAfterMode::ReferenceView {
                    let cur_ref = app
                        .reference_photo_id
                        .and_then(|id| app.photos.iter().find(|p| p.id == id))
                        .map(|p| p.file_name.clone())
                        .unwrap_or_else(|| "Select Reference".into());
                    egui::ComboBox::from_id_salt("ref_photo_picker")
                        .selected_text(format!("Ref: {}", cur_ref))
                        .width(150.0)
                        .show_ui(ui, |ui| {
                            for p in &app.photos {
                                if ui
                                    .selectable_label(app.reference_photo_id == Some(p.id), &p.file_name)
                                    .clicked()
                                {
                                    app.reference_photo_id = Some(p.id);
                                }
                            }
                        });

                    let has_ref = app.reference_photo_id.is_some()
                        && app.reference_photo_id != Some(app.active_photo_id);
                    let match_btn = ui.add_enabled(
                        has_ref,
                        egui::Button::new(
                            RichText::new("⚡ Match Color to Ref")
                                .size(11.5)
                                .strong()
                                .color(if has_ref {
                                    Color32::from_rgb(255, 215, 0)
                                } else {
                                    Colors::TEXT_DIM
                                }),
                        ),
                    );
                    if match_btn
                        .on_hover_text("Automatically analyze the Reference photo's colors, exposure, white balance, contrast, and color grading, and match this active photo to it.")
                        .clicked()
                    {
                        app.auto_match_color_to_reference();
                    }
                }

                let is_ba = matches!(
                    app.before_after_mode,
                    BeforeAfterMode::SplitLeftRight
                        | BeforeAfterMode::SplitTopBottom
                        | BeforeAfterMode::SideBySide
                        | BeforeAfterMode::BeforeOnly
                );
                if ui
                    .selectable_label(is_ba, "Y|Y")
                    .on_hover_text("Cycle Before / After Views (Y)")
                    .clicked()
                {
                    app.before_after_mode = match app.before_after_mode {
                        BeforeAfterMode::Single | BeforeAfterMode::ReferenceView => {
                            BeforeAfterMode::SplitLeftRight
                        }
                        BeforeAfterMode::SplitLeftRight => BeforeAfterMode::SplitTopBottom,
                        BeforeAfterMode::SplitTopBottom => BeforeAfterMode::SideBySide,
                        BeforeAfterMode::SideBySide => BeforeAfterMode::BeforeOnly,
                        BeforeAfterMode::BeforeOnly => BeforeAfterMode::Single,
                    };
                }

                ui.menu_button("▾", |ui| {
                    if ui.button("Before/After Left/Right Split").clicked() {
                        app.before_after_mode = BeforeAfterMode::SplitLeftRight;
                        ui.close_menu();
                    }
                    if ui.button("Before/After Top/Bottom Split").clicked() {
                        app.before_after_mode = BeforeAfterMode::SplitTopBottom;
                        ui.close_menu();
                    }
                    if ui.button("Before/After Left/Right Side-by-Side").clicked() {
                        app.before_after_mode = BeforeAfterMode::SideBySide;
                        ui.close_menu();
                    }
                    if ui.button("Before Only (\\)").clicked() {
                        app.before_after_mode = BeforeAfterMode::BeforeOnly;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Copy Before's Settings to After").clicked() {
                        if !app.photos.is_empty() {
                            let b = app.active_photo().before_params();
                            app.commit_params_change("Copy Before to After", b);
                        }
                        ui.close_menu();
                    }
                    if ui.button("Copy After's Settings to Before").clicked() {
                        if !app.photos.is_empty() {
                            let id = app.active_photo_id;
                            let cur = app.active_photo().params.clone();
                            app.active_photo_mut().set_before_params(cur);
                            app.persist_active_photo();
                            app.rendered_before_cache.retain(|&(cid, _), _| cid != id);
                            app.status_message = "Updated Before state from After.".into();
                        }
                        ui.close_menu();
                    }
                    if ui.button("Swap Before and After").clicked() {
                        app.execute_menu_action(100);
                        ui.close_menu();
                    }
                });

                ui.separator();
                ui.checkbox(
                    &mut app.soft_proofing,
                    RichText::new("Soft Proofing (S)").size(11.0),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("Zoom: {:.0}%", app.loupe_zoom * 100.0))
                            .size(10.5)
                            .color(Colors::TEXT_MID),
                    );
                });
            });
        });
}
