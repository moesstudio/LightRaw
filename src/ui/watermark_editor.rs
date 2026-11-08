// ============================================================================
// LightRaw — Watermark Editor Modal (Adobe Lightroom Classic Replica)
// Live Photo Preview Canvas + Image Options + Text Options + Watermark Effects
// ============================================================================

use super::{Colors, LightRawApp};
use crate::watermark::{
    WatermarkAnchor, WatermarkPreset, WatermarkSizeMode,
    WatermarkStyle, WatermarkTextAlign,
};
use eframe::egui::{self, Color32, Pos2, Rect, RichText, Stroke, Vec2};

pub fn render_watermark_editor_modal(app: &mut LightRawApp, ctx: &egui::Context) {
    if !app.show_watermark_editor_modal {
        return;
    }

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.show_watermark_editor_modal = false;
        return;
    }

    let mut open = app.show_watermark_editor_modal;
    let mut commit_save = false;

    egui::Window::new("Watermark Editor")
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_size(Vec2::new(1060.0, 740.0))
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(10.0, 8.0);

            let available_height = (ui.available_height() - 48.0).max(400.0); // reserve space for bottom action bar

            // Main Split: Left = Live Photo Preview, Right = Setting Panels
            ui.horizontal(|ui| {
                let preview_w = 540.0_f32.min(ui.available_width() * 0.52);
                let controls_w = (ui.available_width() - preview_w - 16.0).max(340.0);

                // -------------------------------------------------------------
                // 1. LEFT PANE: Live Interactive Preview Canvas
                // -------------------------------------------------------------
                ui.allocate_ui(Vec2::new(preview_w, available_height), |ui| {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(24, 24, 26))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(45, 45, 50)))
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            render_live_preview_canvas(app, ui, preview_w - 16.0, available_height - 16.0);
                        });
                });

                ui.separator();

                // -------------------------------------------------------------
                // 2. RIGHT PANE: Watermark Controls (Matching Lightroom Classic)
                // -------------------------------------------------------------
                ui.allocate_ui(Vec2::new(controls_w, available_height), |ui| {
                    ui.vertical(|ui| {
                        // FIXED HEADER BAR: Never scrolls away!
                        render_fixed_header_bar(app, ui);
                        ui.add_space(4.0);
                        ui.separator();

                        // SCROLLABLE SECTIONS: Image Options, Text Options, Watermark Effects
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    render_sections(app, ui);
                                });
                            });
                    });
                });
            });

            ui.separator();

            // -----------------------------------------------------------------
            // 3. BOTTOM ACTION BAR (Preset Management + Cancel / Save)
            // -----------------------------------------------------------------
            ui.horizontal(|ui| {
                ui.label(RichText::new("Preset:").size(11.5).color(Color32::from_rgb(150, 150, 155)));
                egui::ComboBox::from_id_salt("wm_editor_presets_cb")
                    .selected_text(&app.active_watermark_preset_name)
                    .width(190.0)
                    .show_ui(ui, |ui| {
                        for p in &app.watermark_presets {
                            if ui
                                .selectable_label(app.active_watermark_preset_name == p.name, &p.name)
                                .clicked()
                            {
                                app.active_watermark_preset_name = p.name.clone();
                                app.watermark_settings = p.settings.clone();
                            }
                        }
                    });

                if ui
                    .button("Save as New...")
                    .on_hover_text("Save current settings as a new watermark preset")
                    .clicked()
                {
                    let new_name = format!("Preset {}", app.watermark_presets.len() + 1);
                    app.watermark_presets.push(WatermarkPreset {
                        name: new_name.clone(),
                        settings: app.watermark_settings.clone(),
                    });
                    app.active_watermark_preset_name = new_name;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add_sized(
                            [90.0, 26.0],
                            egui::Button::new(RichText::new("Save").strong().color(Color32::WHITE))
                                .fill(Colors::ACCENT_BLUE),
                        )
                        .clicked()
                    {
                        commit_save = true;
                    }

                    if ui.add_sized([80.0, 26.0], egui::Button::new("Cancel")).clicked() {
                        app.show_watermark_editor_modal = false;
                    }
                });
            });
        });

    if commit_save {
        // Save current active preset settings
        if let Some(p) = app
            .watermark_presets
            .iter_mut()
            .find(|p| p.name == app.active_watermark_preset_name)
        {
            p.settings = app.watermark_settings.clone();
        }
        app.export_watermark_text = app.watermark_settings.text.clone();
        app.status_message = format!("Watermark preset '{}' updated.", app.active_watermark_preset_name);
        app.show_watermark_editor_modal = false;
    } else if !open {
        app.show_watermark_editor_modal = false;
    }
}

fn render_live_preview_canvas(
    app: &mut LightRawApp,
    ui: &mut egui::Ui,
    avail_w: f32,
    avail_h: f32,
) {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(avail_w, avail_h), egui::Sense::click());
    let painter = ui.painter_at(rect);

    // 1. Dark neutral photo canvas background
    painter.rect_filled(rect, 4.0, Color32::from_rgb(18, 18, 20));

    // 2. Render Active Photo Preview
    let tex_opt = app.get_photo_texture(ui.ctx(), app.active_photo_id, false);
    let img_size = tex_opt
        .as_ref()
        .map(|t| t.size_vec2())
        .unwrap_or(Vec2::new(600.0, 400.0));

    let scale = ((rect.width() - 24.0) / img_size.x).min((rect.height() - 24.0) / img_size.y).max(0.01);
    let photo_w = img_size.x * scale;
    let photo_h = img_size.y * scale;
    let photo_rect = Rect::from_center_size(rect.center(), Vec2::new(photo_w, photo_h));

    if let Some(tex) = tex_opt {
        painter.image(
            tex.id(),
            photo_rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    } else {
        painter.rect_filled(photo_rect, 0.0, Color32::from_rgb(40, 40, 45));
    }

    // Border around photo
    painter.rect_stroke(photo_rect, 0.0, Stroke::new(1.0, Color32::from_rgb(70, 70, 75)));

    // 3. Render Real Live Watermark Overlay (Text or Graphic Image)
    let s = app.watermark_settings.clone();
    let global_opacity = (s.opacity / 100.0).clamp(0.0, 1.0);

    match s.style {
        WatermarkStyle::Graphic => {
            if let Some(ref path_str) = s.graphic_path {
                if let Some((tex, (orig_w, orig_h))) = app.get_watermark_graphic_texture(ui.ctx(), path_str) {
                    let (src_w, src_h) = (orig_w as f32, orig_h as f32);

                    // Size Calculation
                    let (wm_w, wm_h) = match s.size_mode {
                        WatermarkSizeMode::Proportional => {
                            let pct = (s.proportional_size / 100.0).clamp(0.02, 1.0);
                            let w = (photo_rect.width() * pct).max(12.0);
                            let h = w * (src_h / src_w.max(1.0));
                            (w, h)
                        }
                        WatermarkSizeMode::Fit => {
                            let max_w = photo_rect.width() * 0.95;
                            let max_h = photo_rect.height() * 0.95;
                            let sc = (max_w / src_w.max(1.0)).min(max_h / src_h.max(1.0));
                            (src_w * sc, src_h * sc)
                        }
                        WatermarkSizeMode::Fill => {
                            let sc = (photo_rect.width() / src_w.max(1.0)).max(photo_rect.height() / src_h.max(1.0));
                            (src_w * sc, src_h * sc)
                        }
                    };

                    // Inset
                    let inset_x = photo_rect.width() * (s.inset_horizontal / 100.0);
                    let inset_y = photo_rect.height() * (s.inset_vertical / 100.0);

                    // Anchor
                    let (px, py) = calculate_preview_anchor(s.anchor, photo_rect, wm_w, wm_h, inset_x, inset_y);
                    let wm_rect = Rect::from_min_size(Pos2::new(px, py), Vec2::new(wm_w, wm_h));

                    let tint = Color32::from_white_alpha((global_opacity * 255.0) as u8);
                    paint_wm_mesh_rotated(&painter, tex.id(), wm_rect, s.rotation.degrees(), tint);
                } else {
                    let font_id = egui::FontId::proportional(12.5);
                    painter.text(
                        photo_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Image could not be loaded",
                        font_id,
                        Color32::from_rgb(220, 80, 80),
                    );
                }
            } else {
                let font_id = egui::FontId::proportional(13.0);
                painter.text(
                    photo_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "No graphic image chosen yet\n(Click 'Choose...' on the right panel)",
                    font_id,
                    Color32::from_white_alpha(150),
                );
            }
        }
        WatermarkStyle::Text => {
            let trimmed = s.text.trim();
            if !trimmed.is_empty() {
                let char_count = trimmed.chars().count().max(1) as f32;
                let pct = (s.proportional_size / 100.0).clamp(0.05, 1.0);
                let target_text_w = photo_rect.width() * pct;
                let font_size = ((target_text_w / (char_count * 0.6)).clamp(10.0, 64.0)) as f32;

                let font_id = egui::FontId::proportional(font_size);
                let text_color = Color32::from_rgba_unmultiplied(
                    s.text_color[0],
                    s.text_color[1],
                    s.text_color[2],
                    (global_opacity * 255.0) as u8,
                );

                let approx_w = char_count * font_size * 0.55;
                let approx_h = font_size * 1.2;

                let inset_x = photo_rect.width() * (s.inset_horizontal / 100.0);
                let inset_y = photo_rect.height() * (s.inset_vertical / 100.0);

                let (px, py) = calculate_preview_anchor(
                    s.anchor,
                    photo_rect,
                    approx_w,
                    approx_h,
                    inset_x,
                    inset_y,
                );

                // Drop Shadow
                if s.shadow_enabled {
                    let s_alpha = (s.shadow_opacity / 100.0 * global_opacity * 255.0) as u8;
                    let angle_rad = s.shadow_angle.to_radians();
                    let dist = (s.shadow_offset / 100.0 * 18.0).max(1.0);
                    let sx = dist * angle_rad.cos();
                    let sy = -dist * angle_rad.sin();
                    let shadow_color = Color32::from_rgba_unmultiplied(0, 0, 0, s_alpha);

                    painter.text(
                        Pos2::new(px + sx, py + sy),
                        egui::Align2::LEFT_TOP,
                        trimmed,
                        font_id.clone(),
                        shadow_color,
                    );
                }

                // Main Text
                painter.text(
                    Pos2::new(px, py),
                    egui::Align2::LEFT_TOP,
                    trimmed,
                    font_id,
                    text_color,
                );
            }
        }
    }

    // 4. Render 9 Interactive Anchor Guide Points on Canvas
    let click_pos = resp.interact_pointer_pos();
    for &anchor in &WatermarkAnchor::ALL {
        let (r, c) = anchor.row_col();
        let ax = photo_rect.min.x + (photo_rect.width() * c as f32 * 0.5);
        let ay = photo_rect.min.y + (photo_rect.height() * r as f32 * 0.5);
        let pt = Pos2::new(ax, ay);

        // Click to set anchor directly on the canvas!
        if let Some(cp) = click_pos {
            if resp.clicked() && cp.distance(pt) <= 16.0 {
                app.watermark_settings.anchor = anchor;
            }
        }

        let is_sel = anchor == app.watermark_settings.anchor;
        let col = if is_sel {
            Colors::ACCENT_BLUE
        } else {
            Color32::from_white_alpha(50)
        };
        let radius = if is_sel { 5.5 } else { 3.0 };
        painter.circle_filled(pt, radius, col);
        if is_sel {
            painter.circle_stroke(pt, radius + 2.0, Stroke::new(1.2, Color32::WHITE));
        }
    }
}

fn paint_wm_mesh_rotated(
    painter: &egui::Painter,
    tex_id: egui::TextureId,
    dst: Rect,
    angle_deg: f32,
    tint: Color32,
) {
    if angle_deg.abs() <= 0.01 {
        painter.image(
            tex_id,
            dst,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            tint,
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
    mesh.vertices.push(egui::epaint::Vertex {
        pos: p0,
        uv: Pos2::new(0.0, 0.0),
        color: tint,
    });
    mesh.vertices.push(egui::epaint::Vertex {
        pos: p1,
        uv: Pos2::new(1.0, 0.0),
        color: tint,
    });
    mesh.vertices.push(egui::epaint::Vertex {
        pos: p2,
        uv: Pos2::new(1.0, 1.0),
        color: tint,
    });
    mesh.vertices.push(egui::epaint::Vertex {
        pos: p3,
        uv: Pos2::new(0.0, 1.0),
        color: tint,
    });
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(mesh);
}

fn calculate_preview_anchor(
    anchor: WatermarkAnchor,
    rect: Rect,
    w: f32,
    h: f32,
    inset_x: f32,
    inset_y: f32,
) -> (f32, f32) {
    match anchor {
        WatermarkAnchor::TopLeft => (rect.min.x + inset_x, rect.min.y + inset_y),
        WatermarkAnchor::TopCenter => (rect.min.x + (rect.width() - w) * 0.5, rect.min.y + inset_y),
        WatermarkAnchor::TopRight => (rect.max.x - w - inset_x, rect.min.y + inset_y),
        WatermarkAnchor::CenterLeft => (rect.min.x + inset_x, rect.min.y + (rect.height() - h) * 0.5),
        WatermarkAnchor::Center => (rect.min.x + (rect.width() - w) * 0.5, rect.min.y + (rect.height() - h) * 0.5),
        WatermarkAnchor::CenterRight => (rect.max.x - w - inset_x, rect.min.y + (rect.height() - h) * 0.5),
        WatermarkAnchor::BottomLeft => (rect.min.x + inset_x, rect.max.y - h - inset_y),
        WatermarkAnchor::BottomCenter => (rect.min.x + (rect.width() - w) * 0.5, rect.max.y - h - inset_y),
        WatermarkAnchor::BottomRight => (rect.max.x - w - inset_x, rect.max.y - h - inset_y),
    }
}

fn render_fixed_header_bar(app: &mut LightRawApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui.button("◀").on_hover_text("Previous photo").clicked() {
            app.step_active_photo(-1);
        }
        if ui.button("▶").on_hover_text("Next photo").clicked() {
            app.step_active_photo(1);
        }
        ui.add_space(8.0);
        ui.label(RichText::new("Watermark Style:").size(12.0).strong().color(Colors::TEXT_BRIGHT));
        ui.radio_value(&mut app.watermark_settings.style, WatermarkStyle::Text, "Text");
        ui.radio_value(&mut app.watermark_settings.style, WatermarkStyle::Graphic, "Graphic");
    });
}

fn render_sections(app: &mut LightRawApp, ui: &mut egui::Ui) {
    let is_graphic = app.watermark_settings.style == WatermarkStyle::Graphic;
    let is_text = app.watermark_settings.style == WatermarkStyle::Text;

    // =========================================================================
    // SECTION 1: Image Options
    // =========================================================================
    egui::CollapsingHeader::new(RichText::new("▼ Image Options").size(12.0).strong())
        .default_open(true)
        .show(ui, |ui| {
            ui.add_enabled_ui(is_graphic, |ui| {
                ui.horizontal(|ui| {
                    if let Some(ref path_str) = app.watermark_settings.graphic_path {
                        let file_name = std::path::Path::new(path_str)
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| path_str.clone());
                        ui.label(RichText::new(file_name).size(11.5).strong().color(Colors::TEXT_BRIGHT));
                    } else {
                        ui.label(RichText::new("Please choose a PNG or JPG image").size(11.5).color(Color32::from_rgb(150, 150, 155)));
                    }

                    if ui.button("Choose...").clicked() {
                        if let Some(f) = rfd::FileDialog::new()
                            .add_filter("Images", &["png", "jpg", "jpeg", "webp", "tiff"])
                            .pick_file()
                        {
                            let path_s = f.to_string_lossy().to_string();
                            app.watermark_settings.graphic_path = Some(path_s);
                            app.watermark_settings.style = WatermarkStyle::Graphic;
                        }
                    }

                    if app.watermark_settings.graphic_path.is_some() && ui.small_button("✕").clicked() {
                        app.watermark_settings.graphic_path = None;
                    }
                });
            });
        });

    ui.add_space(2.0);

    // =========================================================================
    // SECTION 2: Text Options
    // =========================================================================
    egui::CollapsingHeader::new(RichText::new("▼ Text Options").size(12.0).strong())
        .default_open(true)
        .show(ui, |ui| {
            ui.add_enabled_ui(is_text, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Text:").size(11.5));
                    ui.add(egui::TextEdit::singleline(&mut app.watermark_settings.text).desired_width(220.0));
                });

                // Font & Style Dropdown
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Font:").size(11.5));
                    egui::ComboBox::from_id_salt("wm_font_cb")
                        .selected_text(&app.watermark_settings.font_family)
                        .width(170.0)
                        .show_ui(ui, |ui| {
                            for font in ["Adobe Clean", "Helvetica", "San Francisco", "Arial", "Courier"] {
                                ui.selectable_value(&mut app.watermark_settings.font_family, font.to_string(), font);
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Style:").size(11.5));
                    egui::ComboBox::from_id_salt("wm_font_style_cb")
                        .selected_text(&app.watermark_settings.font_style)
                        .width(170.0)
                        .show_ui(ui, |ui| {
                            for style in ["Regular", "Bold", "Italic", "Bold Italic"] {
                                ui.selectable_value(&mut app.watermark_settings.font_style, style.to_string(), style);
                            }
                        });
                });

                // Align & Color
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Align:").size(11.5));
                    for (align, lbl) in [
                        (WatermarkTextAlign::Left, "Left"),
                        (WatermarkTextAlign::Center, "Center"),
                        (WatermarkTextAlign::Right, "Right"),
                    ] {
                        ui.selectable_value(&mut app.watermark_settings.text_align, align, lbl);
                    }

                    ui.add_space(8.0);
                    ui.label(RichText::new("Color:").size(11.5));
                    let mut col_arr = [
                        app.watermark_settings.text_color[0] as f32 / 255.0,
                        app.watermark_settings.text_color[1] as f32 / 255.0,
                        app.watermark_settings.text_color[2] as f32 / 255.0,
                    ];
                    if egui::color_picker::color_edit_button_rgb(ui, &mut col_arr).changed() {
                        app.watermark_settings.text_color = [
                            (col_arr[0] * 255.0) as u8,
                            (col_arr[1] * 255.0) as u8,
                            (col_arr[2] * 255.0) as u8,
                        ];
                    }
                });

                ui.separator();

                // Shadow Controls
                ui.checkbox(&mut app.watermark_settings.shadow_enabled, "Shadow");
                if app.watermark_settings.shadow_enabled {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Opacity:").size(11.0));
                        ui.add(egui::Slider::new(&mut app.watermark_settings.shadow_opacity, 0.0..=100.0).show_value(true));
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Offset:").size(11.0));
                        ui.add(egui::Slider::new(&mut app.watermark_settings.shadow_offset, 0.0..=100.0).show_value(true));
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Radius:").size(11.0));
                        ui.add(egui::Slider::new(&mut app.watermark_settings.shadow_radius, 0.0..=100.0).show_value(true));
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Angle:").size(11.0));
                        ui.add(egui::Slider::new(&mut app.watermark_settings.shadow_angle, -180.0..=180.0).suffix("°"));
                    });
                }
            });
        });

    ui.add_space(2.0);

    // =========================================================================
    // SECTION 3: Watermark Effects
    // =========================================================================
    egui::CollapsingHeader::new(RichText::new("▼ Watermark Effects").size(12.0).strong())
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Opacity:").size(11.5));
                ui.add(egui::Slider::new(&mut app.watermark_settings.opacity, 0.0..=100.0).show_value(true));
            });

            ui.separator();

            // Size Options (Proportional / Fit / Fill)
            ui.horizontal(|ui| {
                ui.label(RichText::new("Size:").size(11.5));
                ui.radio_value(&mut app.watermark_settings.size_mode, WatermarkSizeMode::Proportional, "Proportional");
                ui.radio_value(&mut app.watermark_settings.size_mode, WatermarkSizeMode::Fit, "Fit");
                ui.radio_value(&mut app.watermark_settings.size_mode, WatermarkSizeMode::Fill, "Fill");
            });

            if app.watermark_settings.size_mode == WatermarkSizeMode::Proportional {
                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    ui.add(egui::Slider::new(&mut app.watermark_settings.proportional_size, 1.0..=100.0).show_value(true));
                });
            }

            ui.separator();

            // Inset Controls
            ui.label(RichText::new("Inset:").size(11.5).strong());
            ui.horizontal(|ui| {
                ui.label(RichText::new("Horizontal:").size(11.0));
                ui.add(egui::Slider::new(&mut app.watermark_settings.inset_horizontal, 0.0..=100.0).show_value(true));
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("Vertical:").size(11.0));
                ui.add(egui::Slider::new(&mut app.watermark_settings.inset_vertical, 0.0..=100.0).show_value(true));
            });

            ui.separator();

            // Anchor (3x3 Grid) & Rotate (CCW / CW)
            ui.horizontal(|ui| {
                // 3x3 Anchor Grid
                ui.vertical(|ui| {
                    ui.label(RichText::new("Anchor:").size(11.5).strong());
                    for row in 0..3 {
                        ui.horizontal(|ui| {
                            for col in 0..3 {
                                let anchor = WatermarkAnchor::from_row_col(row, col);
                                let is_active = app.watermark_settings.anchor == anchor;
                                let btn_text = if is_active { "●" } else { "○" };
                                if ui.selectable_label(is_active, btn_text).clicked() {
                                    app.watermark_settings.anchor = anchor;
                                }
                            }
                        });
                    }
                });

                ui.add_space(40.0);

                // Rotate Controls
                ui.vertical(|ui| {
                    ui.label(RichText::new("Rotate:").size(11.5).strong());
                    ui.horizontal(|ui| {
                        if ui
                            .add_sized([32.0, 32.0], egui::Button::new("↶"))
                            .on_hover_text("Rotate 90° Counter-Clockwise")
                            .clicked()
                        {
                            app.watermark_settings.rotation = app.watermark_settings.rotation.rotate_ccw();
                        }
                        if ui
                            .add_sized([32.0, 32.0], egui::Button::new("↷"))
                            .on_hover_text("Rotate 90° Clockwise")
                            .clicked()
                        {
                            app.watermark_settings.rotation = app.watermark_settings.rotation.rotate_cw();
                        }
                    });
                    ui.label(
                        RichText::new(format!("{:.0}°", app.watermark_settings.rotation.degrees()))
                            .size(11.0)
                            .color(Color32::from_rgb(150, 150, 155)),
                    );
                });
            });
        });
}
