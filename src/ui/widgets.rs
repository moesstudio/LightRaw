// ============================================================================
// LightRaw — Native Lightroom Classic Widget Toolkit & Interactive Navigator
// ============================================================================

use super::{Colors, LightRawApp};
use crate::catalog::{PhotoItem, PickFlag};
use eframe::egui::{self, Color32, Pos2, Rect, RichText, Sense, Stroke, Ui, Vec2};

pub fn format_lr_val(val: f32, fmt: &str) -> String {
    let suffix = if fmt.ends_with('%') {
        "%"
    } else if fmt.ends_with(" K") {
        " K"
    } else if fmt.ends_with(" px") {
        " px"
    } else if fmt.ends_with('°') {
        "°"
    } else if fmt.ends_with(" mm") {
        " mm"
    } else if fmt.ends_with(" s") {
        " s"
    } else if fmt.ends_with(" PPI") {
        " PPI"
    } else {
        ""
    };

    if fmt.contains(":+.2") || fmt.contains("%+.2") {
        if val.abs() < 0.005 {
            format!("0.00{}", suffix)
        } else {
            format!("{:+.2}{}", val, suffix)
        }
    } else if fmt.contains(":.2") || fmt.contains("%.2") {
        format!("{:.2}{}", val.max(0.0 - 1e9), suffix)
    } else if fmt.contains(":+.1") || fmt.contains("%+.1") {
        if val.abs() < 0.05 {
            format!("0.0{}", suffix)
        } else {
            format!("{:+.1}{}", val, suffix)
        }
    } else if fmt.contains(":.1") || fmt.contains("%.1") {
        format!("{:.1}{}", val, suffix)
    } else if fmt.contains(":+.0") || fmt.contains("%+.0") {
        if val.abs() < 0.49 {
            format!("0{}", suffix)
        } else {
            format!("{:+.0}{}", val, suffix)
        }
    } else {
        format!("{:.0}{}", val, suffix)
    }
}

fn raw_numeric_string(val: f32, fmt: &str) -> String {
    if fmt.contains(".2") {
        format!("{:.2}", val)
    } else if fmt.contains(".1") {
        format!("{:.1}", val)
    } else {
        format!("{:.0}", val)
    }
}

fn render_clickable_value_slot(
    ui: &mut Ui,
    edit_id: egui::Id,
    val: &mut f32,
    range: &std::ops::RangeInclusive<f32>,
    default_val: f32,
    fmt: &str,
    width: f32,
) -> bool {
    let mut changed = false;
    let active_key = egui::Id::new("lr_active_numeric_edit");
    let buf_key = egui::Id::new("lr_active_numeric_buf");
    let just_opened_key = egui::Id::new("lr_active_numeric_just_opened");

    let is_editing = ui
        .ctx()
        .data_mut(|d| d.get_temp::<egui::Id>(active_key) == Some(edit_id));

    if is_editing {
        let mut buf = ui
            .ctx()
            .data_mut(|d| d.get_temp::<String>(buf_key).unwrap_or_default());
        let just_opened = ui
            .ctx()
            .data_mut(|d| d.get_temp::<bool>(just_opened_key).unwrap_or(false));

        let te = ui.add_sized(
            [width, 16.0],
            egui::TextEdit::singleline(&mut buf)
                .font(egui::FontId::monospace(10.5))
                .horizontal_align(egui::Align::RIGHT),
        );
        if just_opened {
            te.request_focus();
            ui.ctx()
                .data_mut(|d| d.insert_temp(just_opened_key, false));
        }
        ui.ctx().data_mut(|d| d.insert_temp(buf_key, buf.clone()));

        let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
        let esc = ui.input(|i| i.key_pressed(egui::Key::Escape));
        if esc {
            ui.ctx().data_mut(|d| d.remove::<egui::Id>(active_key));
        } else if enter || (!just_opened && te.lost_focus()) {
            let cleaned = buf
                .trim()
                .replace(',', ".")
                .trim_end_matches(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != '+')
                .to_string();
            if let Ok(parsed) = cleaned.parse::<f32>() {
                let clamped = parsed.clamp(*range.start(), *range.end());
                if (*val - clamped).abs() > 1e-6 {
                    *val = clamped;
                    changed = true;
                }
            }
            ui.ctx().data_mut(|d| d.remove::<egui::Id>(active_key));
        }
    } else {
        let val_str = format_lr_val(*val, fmt);
        let is_mod = (*val - default_val).abs() > 1e-4;
        let (v_rect, v_resp) = ui.allocate_exact_size(Vec2::new(width, 16.0), Sense::click());
        let v_resp = v_resp.on_hover_text("Click to type exact value • Double-click to reset");
        if v_resp.double_clicked() {
            if (*val - default_val).abs() > 1e-6 {
                *val = default_val;
                changed = true;
            }
        } else if v_resp.clicked() {
            let init_str = raw_numeric_string(*val, fmt);
            ui.ctx().data_mut(|d| {
                d.insert_temp(active_key, edit_id);
                d.insert_temp(buf_key, init_str);
                d.insert_temp(just_opened_key, true);
            });
        }
        if v_resp.hovered() {
            ui.painter().rect_filled(
                v_rect,
                2.5,
                Color32::from_rgba_unmultiplied(255, 255, 255, 18),
            );
        }
        ui.painter().text(
            Pos2::new(v_rect.right() - 1.0, v_rect.center().y),
            egui::Align2::RIGHT_CENTER,
            val_str,
            egui::FontId::monospace(10.5),
            if is_mod {
                Colors::TEXT_BRIGHT
            } else {
                Colors::TEXT_MID
            },
        );
    }

    changed
}

pub fn lr_panel_header(ui: &mut Ui, title: &str, open: &mut bool, badge: Option<&str>) -> bool {
    let desired = Vec2::new(ui.available_width(), 24.0);
    let (rect, resp) = ui.allocate_exact_size(desired, Sense::click());
    if resp.clicked() {
        *open = !*open;
    }
    let bg = if resp.hovered() {
        Color32::from_rgb(56, 56, 62)
    } else {
        Colors::BG_HEADER
    };
    ui.painter().rect_filled(rect, 3.0, bg);
    ui.painter().line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0, Colors::BORDER),
    );
    let arrow = if *open { "▼" } else { "▶" };
    ui.painter().text(
        Pos2::new(rect.min.x + 8.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        format!("{}  {}", arrow, title),
        egui::FontId::proportional(11.5),
        Colors::TEXT_BRIGHT,
    );
    if let Some(b) = badge {
        ui.painter().text(
            Pos2::new(rect.max.x - 8.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            b,
            egui::FontId::proportional(10.5),
            Colors::TEXT_MID,
        );
    }
    *open
}

pub fn lr_left_panel_header(
    ui: &mut Ui,
    title: &str,
    open: &mut bool,
    right_buttons: &[&str],
) -> Option<usize> {
    let mut clicked_btn = None;
    let desired = Vec2::new(ui.available_width(), 24.0);
    let (rect, _) = ui.allocate_exact_size(desired, Sense::hover());
    ui.painter()
        .rect_filled(rect, 0.0, Color32::from_rgb(38, 38, 42));
    ui.painter().line_segment(
        [rect.left_top(), rect.right_top()],
        Stroke::new(1.0, Color32::from_rgb(52, 52, 58)),
    );
    ui.painter().line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0, Color32::from_rgb(24, 24, 28)),
    );

    let btn_area_w = (right_buttons.len() as f32) * 24.0;
    let title_rect = Rect::from_min_max(
        rect.min,
        Pos2::new((rect.max.x - btn_area_w - 4.0).max(rect.min.x + 40.0), rect.max.y),
    );
    let title_resp = ui.interact(
        title_rect,
        ui.make_persistent_id(("left_hdr", title)),
        Sense::click(),
    );
    if title_resp.clicked() {
        *open = !*open;
    }

    let arrow = if *open { "▼" } else { "▶" };
    ui.painter().text(
        Pos2::new(rect.min.x + 8.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        arrow,
        egui::FontId::proportional(9.5),
        Colors::TEXT_MID,
    );
    ui.painter().text(
        Pos2::new(rect.min.x + 22.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        title,
        egui::FontId::proportional(12.0),
        if title_resp.hovered() {
            Color32::WHITE
        } else {
            Colors::TEXT_BRIGHT
        },
    );

    for (idx, lbl) in right_buttons.iter().enumerate() {
        let bx = rect.max.x - (right_buttons.len() - idx) as f32 * 23.0;
        let brect = Rect::from_min_size(Pos2::new(bx, rect.min.y + 3.0), Vec2::new(20.0, 18.0));
        let bresp = ui.interact(
            brect,
            ui.make_persistent_id(("left_hdr_btn", title, idx)),
            Sense::click(),
        );
        if bresp.hovered() {
            ui.painter()
                .rect_filled(brect, 3.0, Color32::from_rgb(64, 64, 72));
        }
        ui.painter().text(
            brect.center(),
            egui::Align2::CENTER_CENTER,
            *lbl,
            egui::FontId::proportional(13.0),
            if bresp.hovered() {
                Color32::WHITE
            } else {
                Colors::TEXT_MAIN
            },
        );
        if bresp.clicked() {
            clicked_btn = Some(idx);
        }
    }
    clicked_btn
}

pub fn lr_right_panel_header(
    ui: &mut Ui,
    title: &str,
    open: &mut bool,
    has_eye: Option<bool>,
    right_badge: Option<&str>,
) -> bool {
    let (is_open, _eye_held, _toggled, _right_clicked) =
        lr_right_panel_header_ex(ui, title, open, has_eye, right_badge);
    is_open
}

/// Extended Lightroom Classic right-panel header returning:
/// `(is_open, eye_is_held_for_bypass, was_just_opened, header_right_clicked)`
pub fn lr_right_panel_header_ex(
    ui: &mut Ui,
    title: &str,
    open: &mut bool,
    eye_modified: Option<bool>,
    right_badge: Option<&str>,
) -> (bool, bool, bool, bool) {
    let desired = Vec2::new(ui.available_width(), 25.0);
    let (rect, _) = ui.allocate_exact_size(desired, Sense::hover());
    ui.painter()
        .rect_filled(rect, 0.0, Color32::from_rgb(38, 38, 42));
    ui.painter().line_segment(
        [rect.left_top(), rect.right_top()],
        Stroke::new(1.0, Color32::from_rgb(52, 52, 58)),
    );
    ui.painter().line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0, Color32::from_rgb(24, 24, 28)),
    );

    let mut eye_held = false;
    let left_offset = if let Some(is_mod) = eye_modified {
        let eye_rect =
            Rect::from_min_size(Pos2::new(rect.min.x + 6.0, rect.min.y + 3.0), Vec2::new(22.0, 19.0));
        let eye_resp = ui
            .interact(
                eye_rect,
                ui.make_persistent_id(("right_hdr_eye", title)),
                Sense::click_and_drag(),
            )
            .on_hover_text(if is_mod {
                "Press and hold to preview without this panel's adjustments"
            } else {
                "No adjustments made in this panel"
            });
        if eye_resp.is_pointer_button_down_on() {
            eye_held = true;
        }
        let eye_col = if eye_held {
            Colors::ACCENT_AMBER
        } else if is_mod {
            Color32::from_rgb(235, 235, 240)
        } else {
            Color32::from_rgb(90, 90, 96)
        };
        ui.painter().text(
            eye_rect.center(),
            egui::Align2::CENTER_CENTER,
            "👁",
            egui::FontId::proportional(11.5),
            eye_col,
        );
        if is_mod {
            ui.painter().circle_filled(
                Pos2::new(eye_rect.right() - 1.5, eye_rect.top() + 3.5),
                2.2,
                Colors::ACCENT_BLUE,
            );
        }
        28.0
    } else {
        8.0
    };

    let main_rect = Rect::from_min_max(Pos2::new(rect.min.x + left_offset, rect.min.y), rect.max);
    let main_resp = ui.interact(
        main_rect,
        ui.make_persistent_id(("right_hdr_main", title)),
        Sense::click(),
    );
    let mut just_opened = false;
    if main_resp.clicked() {
        *open = !*open;
        if *open {
            just_opened = true;
        }
    }

    if let Some(badge) = right_badge {
        ui.painter().text(
            Pos2::new(rect.max.x - 95.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            badge,
            egui::FontId::proportional(10.0),
            Colors::ACCENT_AMBER,
        );
    }

    let arrow = if *open { "▼" } else { "◀" };
    ui.painter().text(
        Pos2::new(rect.max.x - 8.0, rect.center().y),
        egui::Align2::RIGHT_CENTER,
        format!("{}  {}", title, arrow),
        egui::FontId::proportional(12.0),
        if main_resp.hovered() {
            Color32::WHITE
        } else {
            Colors::TEXT_BRIGHT
        },
    );

    (
        *open,
        eye_held,
        just_opened,
        main_resp.secondary_clicked(),
    )
}

pub fn lr_slider(
    ui: &mut Ui,
    label: &str,
    val: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    default_val: f32,
    fmt: &str,
) -> bool {
    let mut changed = false;
    let edit_id = ui.make_persistent_id(("lr_slider_num", label));

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        let lbl_resp = ui.add_sized(
            [74.0, 16.0],
            egui::Label::new(RichText::new(label).size(11.0).color(Colors::TEXT_MAIN))
                .truncate()
                .sense(Sense::click()),
        );
        if lbl_resp.double_clicked() {
            if (*val - default_val).abs() > 1e-6 {
                *val = default_val;
                changed = true;
            }
        }

        let track_w = (ui.available_width() - 52.0).max(48.0);
        let (t_rect, t_resp) =
            ui.allocate_exact_size(Vec2::new(track_w, 16.0), Sense::click_and_drag());
        if t_resp.double_clicked() {
            if (*val - default_val).abs() > 1e-6 {
                *val = default_val;
                changed = true;
            }
        } else if (t_resp.dragged() || t_resp.clicked()) && ui.input(|i| i.pointer.primary_down()) {
            if let Some(pos) = t_resp.interact_pointer_pos() {
                let t = ((pos.x - (t_rect.min.x + 5.0)) / (t_rect.width() - 10.0)).clamp(0.0, 1.0);
                let new_v = *range.start() + t * (*range.end() - *range.start());
                if (*val - new_v).abs() > 1e-6 {
                    *val = new_v;
                    changed = true;
                }
            }
        }

        let groove = Rect::from_min_max(
            Pos2::new(t_rect.min.x + 4.0, t_rect.center().y - 1.5),
            Pos2::new(t_rect.max.x - 4.0, t_rect.center().y + 1.5),
        );
        ui.painter()
            .rect_filled(groove, 1.5, Color32::from_rgb(24, 24, 28));
        ui.painter().rect_stroke(
            groove,
            1.5,
            Stroke::new(0.6, Color32::from_rgb(56, 56, 62)),
        );

        let span = (*range.end() - *range.start()).max(1e-6);
        let def_t = ((default_val - *range.start()) / span).clamp(0.0, 1.0);
        let cur_t = ((*val - *range.start()) / span).clamp(0.0, 1.0);
        let def_x = groove.min.x + def_t * groove.width();
        let cur_x = groove.min.x + cur_t * groove.width();

        if (*val - default_val).abs() > 1e-4 {
            let fill_rect = Rect::from_min_max(
                Pos2::new(def_x.min(cur_x), groove.min.y),
                Pos2::new(def_x.max(cur_x), groove.max.y),
            );
            ui.painter()
                .rect_filled(fill_rect, 1.0, Color32::from_rgb(135, 145, 160));
        }

        // Center notch
        ui.painter().line_segment(
            [
                Pos2::new(def_x, groove.min.y - 2.0),
                Pos2::new(def_x, groove.max.y + 2.0),
            ],
            Stroke::new(1.0, Color32::from_rgb(80, 80, 88)),
        );

        let knob_c = Pos2::new(cur_x, groove.center().y);
        ui.painter().circle_filled(
            knob_c + Vec2::new(0.0, 0.8),
            5.0,
            Color32::from_rgba_unmultiplied(0, 0, 0, 110),
        );
        ui.painter()
            .circle_filled(knob_c, 4.3, Color32::from_rgb(232, 232, 236));
        ui.painter().circle_stroke(
            knob_c,
            4.3,
            Stroke::new(0.8, Color32::from_rgb(40, 40, 45)),
        );

        if render_clickable_value_slot(ui, edit_id, val, &range, default_val, fmt, 48.0) {
            changed = true;
        }
    });

    changed
}

pub fn lr_gradient_slider(
    ui: &mut Ui,
    label: &str,
    val: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    default_val: f32,
    left_col: Color32,
    right_col: Color32,
    fmt: &str,
) -> bool {
    let mut changed = false;
    let edit_id = ui.make_persistent_id(("lr_grad_slider_num", label));

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        let lbl_resp = ui.add_sized(
            [74.0, 16.0],
            egui::Label::new(RichText::new(label).size(11.0).color(Colors::TEXT_MAIN))
                .truncate()
                .sense(Sense::click()),
        );
        if lbl_resp.double_clicked() {
            if (*val - default_val).abs() > 1e-6 {
                *val = default_val;
                changed = true;
            }
        }

        let track_w = (ui.available_width() - 52.0).max(48.0);
        let (t_rect, t_resp) =
            ui.allocate_exact_size(Vec2::new(track_w, 16.0), Sense::click_and_drag());
        if t_resp.double_clicked() {
            if (*val - default_val).abs() > 1e-6 {
                *val = default_val;
                changed = true;
            }
        } else if (t_resp.dragged() || t_resp.clicked()) && ui.input(|i| i.pointer.primary_down()) {
            if let Some(pos) = t_resp.interact_pointer_pos() {
                let t = ((pos.x - (t_rect.min.x + 5.0)) / (t_rect.width() - 10.0)).clamp(0.0, 1.0);
                let new_v = *range.start() + t * (*range.end() - *range.start());
                if (*val - new_v).abs() > 1e-6 {
                    *val = new_v;
                    changed = true;
                }
            }
        }

        let groove = Rect::from_min_max(
            Pos2::new(t_rect.min.x + 4.0, t_rect.center().y - 2.2),
            Pos2::new(t_rect.max.x - 4.0, t_rect.center().y + 2.2),
        );
        let mut mesh = egui::Mesh::default();
        let i0 = mesh.vertices.len() as u32;
        mesh.vertices.push(egui::epaint::Vertex {
            pos: groove.left_top(),
            uv: Pos2::ZERO,
            color: left_col,
        });
        mesh.vertices.push(egui::epaint::Vertex {
            pos: groove.right_top(),
            uv: Pos2::ZERO,
            color: right_col,
        });
        mesh.vertices.push(egui::epaint::Vertex {
            pos: groove.right_bottom(),
            uv: Pos2::ZERO,
            color: right_col,
        });
        mesh.vertices.push(egui::epaint::Vertex {
            pos: groove.left_bottom(),
            uv: Pos2::ZERO,
            color: left_col,
        });
        mesh.add_triangle(i0, i0 + 1, i0 + 2);
        mesh.add_triangle(i0, i0 + 2, i0 + 3);
        ui.painter().add(egui::Shape::mesh(mesh));
        ui.painter().rect_stroke(
            groove,
            1.5,
            Stroke::new(0.6, Color32::from_rgb(20, 20, 24)),
        );

        let span = (*range.end() - *range.start()).max(1e-6);
        let cur_t = ((*val - *range.start()) / span).clamp(0.0, 1.0);
        let cur_x = groove.min.x + cur_t * groove.width();
        let knob_c = Pos2::new(cur_x, groove.center().y);
        ui.painter().circle_filled(
            knob_c + Vec2::new(0.0, 0.8),
            5.2,
            Color32::from_rgba_unmultiplied(0, 0, 0, 130),
        );
        ui.painter()
            .circle_filled(knob_c, 4.4, Color32::from_rgb(240, 240, 244));
        ui.painter().circle_stroke(
            knob_c,
            4.4,
            Stroke::new(0.9, Color32::from_rgb(28, 28, 32)),
        );

        if render_clickable_value_slot(ui, edit_id, val, &range, default_val, fmt, 48.0) {
            changed = true;
        }
    });

    changed
}

pub fn lr_bevel_button(ui: &mut Ui, label: &str, width: f32, height: f32) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
    let bg = if resp.is_pointer_button_down_on() {
        Color32::from_rgb(36, 36, 40)
    } else if resp.hovered() {
        Color32::from_rgb(68, 68, 74)
    } else {
        Color32::from_rgb(54, 54, 60)
    };
    ui.painter().rect_filled(rect, 3.5, bg);
    ui.painter().rect_stroke(
        rect,
        3.5,
        Stroke::new(1.0, Color32::from_rgb(24, 24, 28)),
    );
    ui.painter().line_segment(
        [
            Pos2::new(rect.min.x + 2.0, rect.min.y + 1.0),
            Pos2::new(rect.max.x - 2.0, rect.min.y + 1.0),
        ],
        Stroke::new(0.8, Color32::from_rgba_unmultiplied(255, 255, 255, 24)),
    );
    let font_size = if label.len() > 12 && width < 125.0 {
        10.0
    } else if width < 90.0 {
        9.5
    } else {
        11.5
    };
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(font_size),
        Colors::TEXT_BRIGHT,
    );
    resp
}

pub fn lr_selectable_row(
    ui: &mut Ui,
    selected: bool,
    left_label: &str,
    right_badge: Option<&str>,
) -> egui::Response {
    let desired = Vec2::new(ui.available_width(), 20.0);
    let (rect, resp) = ui.allocate_exact_size(desired, Sense::click());
    if selected {
        ui.painter()
            .rect_filled(rect, 3.0, Color32::from_rgb(92, 92, 98));
    } else if resp.hovered() {
        ui.painter()
            .rect_filled(rect, 3.0, Color32::from_rgb(50, 50, 56));
    }
    ui.painter().text(
        Pos2::new(rect.min.x + 8.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        left_label,
        egui::FontId::proportional(11.0),
        if selected {
            Color32::WHITE
        } else {
            Colors::TEXT_MAIN
        },
    );
    if let Some(badge) = right_badge {
        ui.painter().text(
            Pos2::new(rect.max.x - 8.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            badge,
            egui::FontId::monospace(10.0),
            if selected {
                Color32::WHITE
            } else {
                Colors::TEXT_DIM
            },
        );
    }
    resp
}

pub fn draw_histogram_widget(
    app: &mut LightRawApp,
    ui: &mut Ui,
) {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 108.0), Sense::click_and_drag());
    ui.painter()
        .rect_filled(rect, 2.0, Color32::from_rgb(24, 24, 26));
    ui.painter().rect_stroke(
        rect,
        2.0,
        Stroke::new(1.0, Color32::from_rgb(16, 16, 18)),
    );

    // 5 Tonal Zones: Blacks (0-18%), Shadows (18-40%), Exposure (40-70%), Highlights (70-88%), Whites (88-100%)
    const ZONE_BOUNDS: [f32; 6] = [0.0, 0.18, 0.40, 0.70, 0.88, 1.0];

    // Top-left Shadow Clipping Triangle `[▲]` & Top-right Highlight Clipping Triangle `[▲]`
    let sh_rect = Rect::from_min_size(rect.min + Vec2::new(4.0, 4.0), Vec2::new(16.0, 16.0));
    let hi_rect = Rect::from_min_size(
        Pos2::new(rect.max.x - 20.0, rect.min.y + 4.0),
        Vec2::new(16.0, 16.0),
    );

    // Detect hovered zone (avoid clipping buttons)
    let mut current_hover_zone = None;
    if let Some(pos) = resp.hover_pos() {
        if rect.contains(pos) && !sh_rect.contains(pos) && !hi_rect.contains(pos) {
            let u = ((pos.x - rect.min.x) / rect.width().max(1.0)).clamp(0.0, 0.999);
            let z = if u < ZONE_BOUNDS[1] {
                0
            } else if u < ZONE_BOUNDS[2] {
                1
            } else if u < ZONE_BOUNDS[3] {
                2
            } else if u < ZONE_BOUNDS[4] {
                3
            } else {
                4
            };
            current_hover_zone = Some(z);
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        }
    }

    if resp.dragged() && app.hist_drag_zone.is_some() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
    }

    let active_zone = app.hist_drag_zone.or(current_hover_zone);
    app.hist_hover_zone = active_zone;

    // Highlight the active tonal zone background & top accent line
    if let Some(z) = active_zone {
        let z_idx = z as usize;
        if z_idx < 5 {
            let x0 = rect.min.x + rect.width() * ZONE_BOUNDS[z_idx];
            let x1 = rect.min.x + rect.width() * ZONE_BOUNDS[z_idx + 1];
            let z_rect = Rect::from_min_max(Pos2::new(x0, rect.min.y), Pos2::new(x1, rect.max.y));
            ui.painter().rect_filled(
                z_rect,
                0.0,
                Color32::from_rgba_unmultiplied(255, 255, 255, 22),
            );
            ui.painter().line_segment(
                [Pos2::new(x0, rect.min.y + 1.0), Pos2::new(x1, rect.min.y + 1.0)],
                Stroke::new(1.8, Color32::from_rgba_unmultiplied(255, 255, 255, 90)),
            );
        }
    }

    // Subtle 5-Zone divider lines
    for &b in &ZONE_BOUNDS[1..5] {
        let gx = rect.min.x + rect.width() * b;
        ui.painter().line_segment(
            [Pos2::new(gx, rect.min.y), Pos2::new(gx, rect.max.y)],
            Stroke::new(0.8, Color32::from_rgb(38, 38, 42)),
        );
    }

    // Render Histogram Curves
    let hist_data = app.active_rendered_frame().map(|f| (f.hist_r, f.hist_g, f.hist_b, f.hist_lum));
    if let Some((hist_r, hist_g, hist_b, hist_lum)) = hist_data {
        let max_bin = hist_r
            .iter()
            .chain(hist_g.iter())
            .chain(hist_b.iter())
            .skip(2)
            .take(252)
            .copied()
            .max()
            .unwrap_or(1)
            .max(1) as f32;

        let plot_h = rect.height() - 6.0;
        let base_y = rect.max.y - 2.0;

        for (bins, col) in [
            (&hist_r, Color32::from_rgba_unmultiplied(220, 55, 55, 130)),
            (&hist_g, Color32::from_rgba_unmultiplied(55, 205, 85, 130)),
            (&hist_b, Color32::from_rgba_unmultiplied(60, 125, 235, 130)),
            (
                &hist_lum,
                Color32::from_rgba_unmultiplied(225, 225, 230, 145),
            ),
        ] {
            let mut pts = Vec::with_capacity(130);
            pts.push(Pos2::new(rect.min.x + 2.0, base_y));
            for step in 0..128 {
                let b0 = step * 2;
                let avg = (bins[b0] + bins[b0 + 1]) as f32 * 0.5;
                let norm = (avg / max_bin).sqrt().clamp(0.0, 1.0);
                let x = rect.min.x + 2.0 + (step as f32 / 127.0) * (rect.width() - 4.0);
                let y = base_y - norm * plot_h;
                pts.push(Pos2::new(x, y));
            }
            pts.push(Pos2::new(rect.max.x - 2.0, base_y));
            ui.painter().add(egui::Shape::line(pts, Stroke::new(1.2, col)));
        }
    }

    // Interactive Dragging on the 5 Tonal Zones
    if resp.drag_started() {
        if let Some(z) = current_hover_zone {
            app.hist_drag_zone = Some(z);
        }
    }

    if resp.dragged() {
        if let Some(dz) = app.hist_drag_zone {
            let dx = resp.drag_delta().x;
            if dx.abs() > 0.001 && !app.photos.is_empty() {
                let mut p = app.active_photo().params.clone();
                let desc = match dz {
                    0 => {
                        p.blacks_2012 = (p.blacks_2012 + dx * 0.5).clamp(-100.0, 100.0);
                        "Adjust Blacks"
                    }
                    1 => {
                        p.shadows_2012 = (p.shadows_2012 + dx * 0.5).clamp(-100.0, 100.0);
                        "Adjust Shadows"
                    }
                    2 => {
                        p.exposure_2012 = (p.exposure_2012 + dx * 0.01).clamp(-5.0, 5.0);
                        "Adjust Exposure"
                    }
                    3 => {
                        p.highlights_2012 = (p.highlights_2012 + dx * 0.5).clamp(-100.0, 100.0);
                        "Adjust Highlights"
                    }
                    4 => {
                        p.whites_2012 = (p.whites_2012 + dx * 0.5).clamp(-100.0, 100.0);
                        "Adjust Whites"
                    }
                    _ => "Adjust Tone",
                };
                app.commit_params_change(desc, p);
            }
        }
    }

    if resp.drag_stopped() {
        app.hist_drag_zone = None;
    }

    // Double-click resets that specific zone to 0.0
    if resp.double_clicked() {
        if let Some(z) = current_hover_zone {
            if !app.photos.is_empty() {
                let mut p = app.active_photo().params.clone();
                let desc = match z {
                    0 => {
                        p.blacks_2012 = 0.0;
                        "Reset Blacks"
                    }
                    1 => {
                        p.shadows_2012 = 0.0;
                        "Reset Shadows"
                    }
                    2 => {
                        p.exposure_2012 = 0.0;
                        "Reset Exposure"
                    }
                    3 => {
                        p.highlights_2012 = 0.0;
                        "Reset Highlights"
                    }
                    4 => {
                        p.whites_2012 = 0.0;
                        "Reset Whites"
                    }
                    _ => "Reset Tone",
                };
                app.commit_params_change(desc, p);
            }
        }
    }

    // Shadow & Highlight Clipping Toggles
    let sh_resp = ui
        .interact(
            sh_rect,
            ui.make_persistent_id("hist_sh_clip"),
            Sense::click(),
        )
        .on_hover_text("Show Shadow Clipping (J)");
    let hi_resp = ui
        .interact(
            hi_rect,
            ui.make_persistent_id("hist_hi_clip"),
            Sense::click(),
        )
        .on_hover_text("Show Highlight Clipping (J)");
    if sh_resp.clicked() {
        app.show_shadow_clipping = !app.show_shadow_clipping;
    }
    if hi_resp.clicked() {
        app.show_highlight_clipping = !app.show_highlight_clipping;
    }

    for (r, active, active_col) in [
        (sh_rect, app.show_shadow_clipping, Color32::from_rgb(70, 155, 255)),
        (hi_rect, app.show_highlight_clipping, Color32::WHITE),
    ] {
        ui.painter()
            .rect_filled(r, 2.0, Color32::from_rgb(38, 38, 42));
        if active {
            ui.painter()
                .rect_stroke(r, 2.0, Stroke::new(1.0, active_col));
        }
        ui.painter().text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            "▲",
            egui::FontId::proportional(9.5),
            if active {
                active_col
            } else {
                Color32::from_rgb(135, 135, 140)
            },
        );
    }
}

pub fn render_navigator_panel(app: &mut LightRawApp, ctx: &egui::Context, ui: &mut Ui, open: &mut bool) {
    let w = ui.available_width();
    let (hdr_rect, _) = ui.allocate_exact_size(Vec2::new(w, 24.0), Sense::hover());
    ui.painter()
        .rect_filled(hdr_rect, 0.0, Color32::from_rgb(38, 38, 42));
    ui.painter().line_segment(
        [hdr_rect.left_bottom(), hdr_rect.right_bottom()],
        Stroke::new(1.0, Color32::from_rgb(24, 24, 28)),
    );

    let left_hit = Rect::from_min_max(
        hdr_rect.min,
        Pos2::new(hdr_rect.min.x + 100.0, hdr_rect.max.y),
    );
    if ui
        .interact(left_hit, ui.make_persistent_id("nav_hdr_toggle"), Sense::click())
        .clicked()
    {
        *open = !*open;
    }
    ui.painter().text(
        Pos2::new(hdr_rect.min.x + 8.0, hdr_rect.center().y),
        egui::Align2::LEFT_CENTER,
        format!("{}  Navigator", if *open { "▼" } else { "▶" }),
        egui::FontId::proportional(12.0),
        Colors::TEXT_BRIGHT,
    );

    // Zoom presets: FIT | 100% | 200%
    for (idx, (lbl, z)) in [("FIT", 1.0_f32), ("FILL", 1.35_f32), ("100%", 2.5_f32), ("200%", 4.0_f32)]
        .into_iter()
        .enumerate()
    {
        let bx = hdr_rect.max.x - (4 - idx) as f32 * 33.0 - 2.0;
        let br = Rect::from_min_size(Pos2::new(bx, hdr_rect.min.y + 3.0), Vec2::new(31.0, 18.0));
        let bresp = ui.interact(br, ui.make_persistent_id(("nav_z", idx)), Sense::click());
        let active = (app.loupe_zoom - z).abs() < 0.08;
        if bresp.clicked() {
            app.loupe_zoom = z;
            if z <= 1.01 {
                app.loupe_pan = Vec2::ZERO;
            }
        }
        ui.painter().text(
            br.center(),
            egui::Align2::CENTER_CENTER,
            lbl,
            egui::FontId::proportional(9.5),
            if active {
                Color32::WHITE
            } else if bresp.hovered() {
                Colors::TEXT_MAIN
            } else {
                Colors::TEXT_DIM
            },
        );
    }

    if !*open {
        return;
    }

    let (box_rect, box_resp) =
        ui.allocate_exact_size(Vec2::new(w, 168.0), Sense::click_and_drag());
    ui.painter()
        .rect_filled(box_rect, 0.0, Color32::from_rgb(32, 32, 34));

    if app.photos.is_empty() {
        return;
    }

    // Check if hovering a preset or history step for instant live preview!
    let preview_tex = app.get_navigator_preview_texture(ctx);
    let Some(tex) = preview_tex else {
        return;
    };

    let inner = box_rect.shrink(8.0);
    let ts = tex.size_vec2();
    let sc = (inner.width() / ts.x).min(inner.height() / ts.y);
    let dst = Rect::from_center_size(inner.center(), ts * sc);

    ui.painter().image(
        tex.id(),
        dst,
        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        Color32::WHITE,
    );
    ui.painter()
        .rect_stroke(dst, 0.0, Stroke::new(1.0, Color32::from_rgb(18, 18, 20)));

    let is_zoomed = app.loupe_zoom > 1.02;
    if box_resp.hovered() {
        if is_zoomed {
            if box_resp.is_pointer_button_down_on() || box_resp.dragged() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            } else {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
            }
        } else {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ZoomIn);
        }
    }

    // Interactive Zoom Pan Viewport Box inside Navigator
    if is_zoomed {
        let frac = (1.0 / app.loupe_zoom).clamp(0.08, 1.0);
        let vp_size = dst.size() * frac;
        // Map `loupe_pan` (approx canvas pixels) to Navigator normalized offset
        let approx_canvas_scale = 760.0 * app.loupe_zoom;
        let norm_off = Vec2::new(
            (-app.loupe_pan.x / approx_canvas_scale).clamp(-0.5 + frac * 0.5, 0.5 - frac * 0.5),
            (-app.loupe_pan.y / approx_canvas_scale).clamp(-0.5 + frac * 0.5, 0.5 - frac * 0.5),
        );
        let vp_center = dst.center() + norm_off * dst.size();
        let vp_rect = Rect::from_center_size(vp_center, vp_size).intersect(dst);
        ui.painter()
            .rect_stroke(vp_rect, 1.0, Stroke::new(1.5, Color32::WHITE));

        if (box_resp.dragged() || box_resp.clicked()) && ui.input(|i| i.pointer.primary_down()) {
            if let Some(pos) = box_resp.interact_pointer_pos() {
                let rel = Vec2::new(
                    ((pos.x - dst.center().x) / dst.width().max(1.0)).clamp(-0.48, 0.48),
                    ((pos.y - dst.center().y) / dst.height().max(1.0)).clamp(-0.48, 0.48),
                );
                app.loupe_pan = -rel * approx_canvas_scale;
            }
        }
    } else if box_resp.clicked() {
        app.loupe_zoom = 2.5;
        let approx_canvas_scale = 760.0 * app.loupe_zoom;
        if let Some(pos) = box_resp.interact_pointer_pos() {
            let rel = Vec2::new(
                ((pos.x - dst.center().x) / dst.width().max(1.0)).clamp(-0.48, 0.48),
                ((pos.y - dst.center().y) / dst.height().max(1.0)).clamp(-0.48, 0.48),
            );
            app.loupe_pan = -rel * approx_canvas_scale;
        }
    }
}

pub fn paint_thumbnail_corner_badges(ui: &mut Ui, dst: Rect, photo: &PhotoItem) {
    let cropped = photo.is_cropped();
    let modified = photo.is_modified();

    // Top-left pick/reject flag indicator
    if photo.flag != PickFlag::Unflagged {
        let f_rect = Rect::from_min_size(dst.min + Vec2::new(3.0, 3.0), Vec2::new(14.0, 14.0));
        ui.painter().rect_filled(
            f_rect,
            2.0,
            Color32::from_rgba_unmultiplied(15, 15, 18, 195),
        );
        let (sym, col) = match photo.flag {
            PickFlag::Picked => ("⚑", Color32::WHITE),
            PickFlag::Rejected => ("✕", Color32::from_rgb(235, 85, 85)),
            PickFlag::Unflagged => ("", Color32::WHITE),
        };
        ui.painter().text(
            f_rect.center(),
            egui::Align2::CENTER_CENTER,
            sym,
            egui::FontId::proportional(9.5),
            col,
        );
    }

    // Bottom-left star rating pill if > 0
    if photo.rating > 0 && dst.width() > 54.0 {
        let stars = "★".repeat(photo.rating.min(5) as usize);
        let s_rect = Rect::from_min_size(
            Pos2::new(dst.min.x + 3.0, dst.max.y - 15.0),
            Vec2::new((photo.rating as f32 * 8.5 + 6.0).min(dst.width() * 0.55), 12.0),
        );
        ui.painter().rect_filled(
            s_rect,
            2.0,
            Color32::from_rgba_unmultiplied(15, 15, 18, 185),
        );
        ui.painter().text(
            s_rect.center(),
            egui::Align2::CENTER_CENTER,
            stars,
            egui::FontId::proportional(8.5),
            Colors::GOLD_STAR,
        );
    }

    // Bottom-right `[⌑]` Crop & `[±]` Develop badges — ONLY drawn when actually modified!
    let mut badges: Vec<&str> = Vec::new();
    if cropped {
        badges.push("⌑");
    }
    if modified {
        badges.push("±");
    }
    for (idx, sym) in badges.iter().rev().enumerate() {
        let bx = dst.max.x - 17.0 - (idx as f32) * 17.0;
        let by = dst.max.y - 16.0;
        let b_rect = Rect::from_min_size(Pos2::new(bx, by), Vec2::new(14.0, 13.0));
        ui.painter().rect_filled(
            b_rect,
            2.0,
            Color32::from_rgba_unmultiplied(18, 18, 20, 210),
        );
        ui.painter().rect_stroke(
            b_rect,
            2.0,
            Stroke::new(0.6, Color32::from_rgba_unmultiplied(255, 255, 255, 55)),
        );
        ui.painter().text(
            b_rect.center(),
            egui::Align2::CENTER_CENTER,
            *sym,
            egui::FontId::proportional(9.5),
            Color32::from_rgb(230, 230, 235),
        );
    }
}
