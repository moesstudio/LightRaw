// ============================================================================
// LightRaw — Map, Book, Slideshow, Print, Web, Import Workspace & Export Modal
// ============================================================================

use super::{
    lr_bevel_button, lr_left_panel_header, lr_right_panel_header, lr_selectable_row,
    lr_slider, Colors, ImportActionChoice, ImportViewMode, LightRawApp, SavedMapLocation,
};
use eframe::egui::{self, Color32, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use std::path::PathBuf;
use std::time::Instant;

fn lat_lon_to_screen(
    lat: f64,
    lon: f64,
    center_lat: f64,
    center_lon: f64,
    zoom: f32,
    rect: Rect,
) -> Pos2 {
    let scale = zoom.max(0.5) as f64;
    let dx = ((lon - center_lon) / 360.0) * (rect.width() as f64) * scale;
    let dy = ((center_lat - lat) / 180.0) * (rect.height() as f64) * scale;
    Pos2::new(rect.center().x + dx as f32, rect.center().y + dy as f32)
}

fn screen_to_lat_lon(
    pos: Pos2,
    center_lat: f64,
    center_lon: f64,
    zoom: f32,
    rect: Rect,
) -> (f64, f64) {
    let scale = zoom.max(0.5) as f64;
    let lon = center_lon + ((pos.x - rect.center().x) as f64 / ((rect.width() as f64) * scale)) * 360.0;
    let lat = center_lat - ((pos.y - rect.center().y) as f64 / ((rect.height() as f64) * scale)) * 180.0;
    (lat.clamp(-85.0, 85.0), ((lon + 180.0).rem_euclid(360.0)) - 180.0)
}

// ============================================================================
// 1. MAP MODULE — Vector World Map, Saved Locations (SQLite) & GPX Tracklogs
// ============================================================================
pub fn render_map_module(app: &mut LightRawApp, ctx: &egui::Context) {
    if app.show_left_panel {
        egui::SidePanel::left("map_left_panel")
            .resizable(true)
            .default_width(250.0)
            .min_width(220.0)
            .max_width(360.0)
            .frame(egui::Frame::none().fill(Colors::BG_PANEL))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let _ = lr_left_panel_header(ui, "Saved Locations", &mut app.map_left_open, &[]);
                    if app.map_left_open {
                        egui::Frame::none()
                            .fill(Color32::from_rgb(33, 33, 37))
                            .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                            .show(ui, |ui| {
                                let mut jump_to: Option<(f64, f64)> = None;
                                let mut del_idx: Option<usize> = None;
                                for (idx, loc) in app.saved_locations.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        let label = format!("📍 {} ({:.0}m)", loc.name, loc.radius_m);
                                        if lr_selectable_row(ui, false, &label, Some(&format!("{:.2},{:.2}", loc.lat, loc.lon))).clicked() {
                                            jump_to = Some((loc.lat, loc.lon));
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        if ui.small_button("Center").clicked() {
                                            jump_to = Some((loc.lat, loc.lon));
                                        }
                                        if ui.small_button("Apply to Selected").clicked() {
                                            let (la, lo, nm) = (loc.lat, loc.lon, loc.name.clone());
                                            let ids = if app.selected_photo_ids.is_empty() {
                                                vec![app.active_photo_id]
                                            } else {
                                                app.selected_photo_ids.clone()
                                            };
                                            for id in ids {
                                                if let Some(p) = app.photos.iter_mut().find(|x| x.id == id) {
                                                    p.gps_lat = Some(la);
                                                    p.gps_lon = Some(lo);
                                                    if p.sublocation.is_empty() {
                                                        p.sublocation = nm.clone();
                                                    }
                                                    app.catalog_db.save_photo(p);
                                                }
                                            }
                                            app.status_message = format!("Geotagged selection to '{}'.", nm);
                                        }
                                        if ui.small_button("✕").on_hover_text("Delete Location").clicked() {
                                            del_idx = Some(idx);
                                        }
                                    });
                                    ui.separator();
                                }
                                if let Some((la, lo)) = jump_to {
                                    app.map_center_lat = la;
                                    app.map_center_lon = lo;
                                    app.map_zoom = app.map_zoom.max(3.2);
                                }
                                if let Some(di) = del_idx {
                                    app.saved_locations.remove(di);
                                    app.catalog_db.save_saved_locations(&app.saved_locations);
                                }

                                ui.add_space(6.0);
                                ui.label(RichText::new("Save Current Map Center:").size(10.0).color(Colors::TEXT_MID));
                                ui.add(
                                    egui::TextEdit::singleline(&mut app.new_location_name)
                                        .hint_text("Location Name")
                                        .desired_width(f32::INFINITY),
                                );
                                lr_slider(ui, "Radius (m)", &mut app.new_location_radius, 25.0..=5000.0, 200.0, "%.0f");
                                if lr_bevel_button(ui, "+ Create Saved Location", ui.available_width(), 24.0).clicked() {
                                    let nm = app.new_location_name.trim().to_string();
                                    if !nm.is_empty() {
                                        app.saved_locations.push(SavedMapLocation {
                                            name: nm,
                                            lat: app.map_center_lat,
                                            lon: app.map_center_lon,
                                            radius_m: app.new_location_radius,
                                        });
                                        app.catalog_db.save_saved_locations(&app.saved_locations);
                                    }
                                }
                            });
                    }
                });
            });
    }

    if app.show_right_panel {
        egui::SidePanel::right("map_right_panel")
            .resizable(true)
            .default_width(275.0)
            .min_width(260.0)
            .max_width(380.0)
            .frame(egui::Frame::none().fill(Colors::BG_PANEL))
            .show(ctx, |ui| {
                if lr_right_panel_header(ui, "Location & Tracklogs", &mut app.map_right_open, None, None) {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(34, 34, 38))
                        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing.y = 6.0;
                            let snap = app.active_photo().clone();
                            ui.label(RichText::new(&snap.file_name).size(11.0).strong().color(Colors::TEXT_BRIGHT));

                            let mut lat_v = snap.gps_lat.unwrap_or(app.map_center_lat) as f32;
                            let mut lon_v = snap.gps_lon.unwrap_or(app.map_center_lon) as f32;
                            let mut gps_edited = false;
                            if lr_slider(ui, "Latitude", &mut lat_v, -85.0..=85.0, 0.0, "%.2f") {
                                gps_edited = true;
                            }
                            if lr_slider(ui, "Longitude", &mut lon_v, -180.0..=180.0, 0.0, "%.2f") {
                                gps_edited = true;
                            }
                            if gps_edited {
                                let p = app.active_photo_mut();
                                p.gps_lat = Some(lat_v as f64);
                                p.gps_lon = Some(lon_v as f64);
                                let cloned = p.clone();
                                app.catalog_db.save_photo(&cloned);
                            }

                            let mut subloc = snap.sublocation.clone();
                            let mut city = snap.city.clone();
                            let mut state = snap.state_province.clone();
                            let mut country = snap.country.clone();
                            let mut meta_ch = false;
                            ui.horizontal(|ui| {
                                ui.add_sized([68.0, 18.0], egui::Label::new(RichText::new("Sublocation").size(10.0).color(Colors::TEXT_MID)));
                                meta_ch |= ui.text_edit_singleline(&mut subloc).changed();
                            });
                            ui.horizontal(|ui| {
                                ui.add_sized([68.0, 18.0], egui::Label::new(RichText::new("City").size(10.0).color(Colors::TEXT_MID)));
                                meta_ch |= ui.text_edit_singleline(&mut city).changed();
                            });
                            ui.horizontal(|ui| {
                                ui.add_sized([68.0, 18.0], egui::Label::new(RichText::new("State").size(10.0).color(Colors::TEXT_MID)));
                                meta_ch |= ui.text_edit_singleline(&mut state).changed();
                            });
                            ui.horizontal(|ui| {
                                ui.add_sized([68.0, 18.0], egui::Label::new(RichText::new("Country").size(10.0).color(Colors::TEXT_MID)));
                                meta_ch |= ui.text_edit_singleline(&mut country).changed();
                            });
                            if meta_ch {
                                let p = app.active_photo_mut();
                                p.sublocation = subloc;
                                p.city = city;
                                p.state_province = state;
                                p.country = country;
                                let cloned = p.clone();
                                app.catalog_db.save_photo(&cloned);
                            }

                            if snap.gps_lat.is_some()
                                && lr_bevel_button(ui, "Clear Photo GPS Coordinates", ui.available_width(), 22.0).clicked()
                            {
                                let p = app.active_photo_mut();
                                p.gps_lat = None;
                                p.gps_lon = None;
                                let cloned = p.clone();
                                app.catalog_db.save_photo(&cloned);
                            }

                            ui.separator();
                            ui.label(RichText::new("GPS Tracklogs (.gpx)").size(10.5).strong().color(Colors::TEXT_BRIGHT));
                            if lr_bevel_button(ui, "Load GPX Tracklog...", ui.available_width(), 24.0).clicked() {
                                app.load_gpx_tracklog_dialog();
                            }
                            if !app.gpx_track_points.is_empty() {
                                ui.label(
                                    RichText::new(format!("Active track: {} points loaded", app.gpx_track_points.len()))
                                        .size(10.0)
                                        .color(Color32::from_rgb(80, 210, 245)),
                                );
                                if lr_bevel_button(ui, "Auto-Tag Selected Photos Along Track", ui.available_width(), 22.0).clicked() {
                                    let ids = if app.selected_photo_ids.is_empty() {
                                        vec![app.active_photo_id]
                                    } else {
                                        app.selected_photo_ids.clone()
                                    };
                                    let n_pts = app.gpx_track_points.len();
                                    for (i, id) in ids.iter().enumerate() {
                                        let pt = &app.gpx_track_points[(i * n_pts / ids.len().max(1)).min(n_pts - 1)];
                                        if let Some(p) = app.photos.iter_mut().find(|x| x.id == *id) {
                                            p.gps_lat = Some(pt.lat);
                                            p.gps_lon = Some(pt.lon);
                                            app.catalog_db.save_photo(p);
                                        }
                                    }
                                    app.status_message = format!("Auto-tagged {} photo(s) from GPX tracklog.", ids.len());
                                }
                            }
                        });
                }
            });
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(Color32::from_rgb(18, 24, 34)))
        .show(ctx, |ui| {
            // Top Map Toolbar
            egui::Frame::none()
                .fill(Colors::BG_TOOLBAR)
                .inner_margin(egui::Margin::symmetric(12.0, 5.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Map Style:").size(10.5).color(Colors::TEXT_MID));
                        egui::ComboBox::from_id_salt("map_style_combo")
                            .selected_text(RichText::new(&app.map_style).size(10.5))
                            .show_ui(ui, |ui| {
                                for st in [
                                    "Dark Satellite Hybrid",
                                    "Road Map",
                                    "Terrain Topo",
                                    "Light Atlas",
                                ] {
                                    ui.selectable_value(&mut app.map_style, st.to_string(), st);
                                }
                            });
                        ui.separator();
                        ui.label(RichText::new("Zoom").size(10.5).color(Colors::TEXT_MID));
                        ui.add(egui::Slider::new(&mut app.map_zoom, 0.8..=14.0).show_value(false));
                        if ui.small_button("Reset View").clicked() {
                            app.map_zoom = 1.4;
                            app.map_center_lat = -2.5;
                            app.map_center_lon = 115.0;
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new("Drag to pan • Scroll to zoom • Click map to geotag selected photo(s)")
                                    .size(9.8)
                                    .color(Colors::TEXT_DIM),
                            );
                        });
                    });
                });

            let (map_rect, resp) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
            let (ocean_col, land_col, border_col, grid_col) = match app.map_style.as_str() {
                "Road Map" => (
                    Color32::from_rgb(168, 204, 232),
                    Color32::from_rgb(238, 236, 228),
                    Color32::from_rgb(165, 162, 154),
                    Color32::from_rgba_unmultiplied(80, 100, 120, 45),
                ),
                "Terrain Topo" => (
                    Color32::from_rgb(132, 178, 206),
                    Color32::from_rgb(192, 206, 174),
                    Color32::from_rgb(122, 138, 108),
                    Color32::from_rgba_unmultiplied(60, 80, 60, 50),
                ),
                "Light Atlas" => (
                    Color32::from_rgb(212, 226, 238),
                    Color32::from_rgb(248, 248, 250),
                    Color32::from_rgb(190, 195, 204),
                    Color32::from_rgba_unmultiplied(100, 110, 125, 40),
                ),
                _ => (
                    Color32::from_rgb(16, 26, 40),
                    Color32::from_rgb(38, 48, 60),
                    Color32::from_rgb(72, 92, 114),
                    Color32::from_rgba_unmultiplied(120, 160, 200, 28),
                ),
            };
            ui.painter().rect_filled(map_rect, 0.0, ocean_col);
            ui.set_clip_rect(map_rect);

            if resp.dragged() {
                let d = resp.drag_delta();
                let scale = app.map_zoom.max(0.5) as f64;
                app.map_center_lon -= (d.x as f64 / (map_rect.width() as f64 * scale)) * 360.0;
                app.map_center_lat += (d.y as f64 / (map_rect.height() as f64 * scale)) * 180.0;
                app.map_center_lat = app.map_center_lat.clamp(-80.0, 80.0);
            }
            if resp.hovered() {
                let sc = ui.input(|i| i.smooth_scroll_delta.y);
                if sc.abs() > 0.5 {
                    app.map_zoom = (app.map_zoom * (1.0 + sc * 0.0025)).clamp(0.8, 14.0);
                }
            }

            // Lat/Lon Graticule
            for lon_g in (-180..=180).step_by(30) {
                let p0 = lat_lon_to_screen(82.0, lon_g as f64, app.map_center_lat, app.map_center_lon, app.map_zoom, map_rect);
                let p1 = lat_lon_to_screen(-82.0, lon_g as f64, app.map_center_lat, app.map_center_lon, app.map_zoom, map_rect);
                ui.painter().line_segment([p0, p1], Stroke::new(0.8, grid_col));
            }
            for lat_g in (-60..=60).step_by(30) {
                let p0 = lat_lon_to_screen(lat_g as f64, -180.0, app.map_center_lat, app.map_center_lon, app.map_zoom, map_rect);
                let p1 = lat_lon_to_screen(lat_g as f64, 180.0, app.map_center_lat, app.map_center_lon, app.map_zoom, map_rect);
                ui.painter().line_segment([p0, p1], Stroke::new(0.8, grid_col));
            }

            // Detailed Multi-Continent & Archipelago Land Polygons
            let landmasses: &[&[(f64, f64)]] = &[
                // North America
                &[(70.0, -165.0), (71.0, -130.0), (60.0, -65.0), (45.0, -66.0), (25.0, -80.0), (18.0, -96.0), (9.0, -79.0), (19.0, -105.0), (33.0, -118.0), (58.0, -136.0)],
                // South America
                &[(11.0, -75.0), (6.0, -52.0), (-6.0, -35.0), (-23.0, -42.0), (-54.0, -68.0), (-42.0, -74.0), (-18.0, -71.0), (-4.0, -81.0)],
                // Europe & Eurasia
                &[(36.0, -9.0), (44.0, -8.0), (58.0, 8.0), (70.0, 26.0), (72.0, 95.0), (66.0, 170.0), (52.0, 140.0), (38.0, 122.0), (22.0, 114.0), (9.0, 105.0), (2.0, 103.0), (16.0, 96.0), (8.0, 77.0), (24.0, 67.0), (13.0, 44.0), (31.0, 34.0), (41.0, 28.0), (38.0, 15.0)],
                // Africa
                &[(35.0, -6.0), (37.0, 10.0), (31.0, 32.0), (11.0, 51.0), (-2.0, 41.0), (-26.0, 33.0), (-34.5, 19.0), (-17.0, 12.0), (4.0, 9.0), (5.0, -8.0), (15.0, -17.0)],
                // Sumatra, Java, Kalimantan, Sulawesi & Papua (Indonesia Archipelago)
                &[(5.6, 95.3), (-5.9, 105.8), (-3.0, 104.8), (3.6, 98.7)],
                &[(-6.0, 106.0), (-7.8, 114.4), (-8.5, 114.0), (-6.8, 106.2)],
                &[(7.0, 116.8), (1.2, 118.8), (-3.5, 116.2), (-3.0, 110.2), (1.8, 109.5)],
                &[(1.5, 125.0), (-5.5, 120.4), (-4.0, 119.5), (0.5, 120.0)],
                &[(-0.8, 131.2), (-2.6, 140.8), (-9.0, 149.0), (-8.4, 138.0)],
                // Japan
                &[(44.0, 144.0), (35.5, 140.8), (31.5, 130.5), (39.0, 139.8)],
                // Australia
                &[(-12.0, 131.0), (-11.0, 142.5), (-28.0, 153.5), (-38.0, 145.0), (-34.0, 115.0), (-21.0, 114.0)],
            ];
            for poly in landmasses {
                let pts: Vec<Pos2> = poly
                    .iter()
                    .map(|&(la, lo)| lat_lon_to_screen(la, lo, app.map_center_lat, app.map_center_lon, app.map_zoom, map_rect))
                    .collect();
                ui.painter().add(egui::Shape::convex_polygon(
                    pts.clone(),
                    land_col,
                    Stroke::new(1.1, border_col),
                ));
            }

            // Saved Location Radius Rings
            for loc in &app.saved_locations {
                let p = lat_lon_to_screen(loc.lat, loc.lon, app.map_center_lat, app.map_center_lon, app.map_zoom, map_rect);
                if map_rect.contains(p) {
                    let r_px = (loc.radius_m * 0.03 * app.map_zoom).clamp(8.0, 64.0);
                    ui.painter().circle_filled(p, r_px, Color32::from_rgba_unmultiplied(10, 132, 255, 36));
                    ui.painter().circle_stroke(p, r_px, Stroke::new(1.2, Colors::ACCENT_BLUE));
                    ui.painter().text(
                        p + Vec2::new(0.0, -r_px - 4.0),
                        egui::Align2::CENTER_BOTTOM,
                        &loc.name,
                        egui::FontId::proportional(9.8),
                        Color32::from_rgb(160, 215, 255),
                    );
                }
            }

            // GPX Tracklog Polyline
            if app.gpx_track_points.len() >= 2 {
                let track_pts: Vec<Pos2> = app
                    .gpx_track_points
                    .iter()
                    .map(|tp| lat_lon_to_screen(tp.lat, tp.lon, app.map_center_lat, app.map_center_lon, app.map_zoom, map_rect))
                    .collect();
                ui.painter().add(egui::Shape::line(
                    track_pts,
                    Stroke::new(2.2, Color32::from_rgb(55, 225, 245)),
                ));
            }

            // Geotagged Photo Pins
            let mut clicked_pin: Option<i64> = None;
            for p in &app.photos {
                if let (Some(la), Some(lo)) = (p.gps_lat, p.gps_lon) {
                    let pos = lat_lon_to_screen(la, lo, app.map_center_lat, app.map_center_lon, app.map_zoom, map_rect);
                    if !map_rect.contains(pos) {
                        continue;
                    }
                    let is_active = p.id == app.active_photo_id;
                    let col = if is_active {
                        Colors::ACCENT_AMBER
                    } else {
                        Colors::ACCENT_BLUE
                    };
                    ui.painter().circle_filled(pos, if is_active { 7.5 } else { 5.5 }, col);
                    ui.painter().circle_stroke(pos, if is_active { 7.5 } else { 5.5 }, Stroke::new(1.5, Color32::WHITE));
                    if let Some(mp) = resp.interact_pointer_pos() {
                        if resp.clicked() && mp.distance(pos) <= 10.0 {
                            clicked_pin = Some(p.id);
                        }
                    }
                }
            }

            if let Some(pid) = clicked_pin {
                app.select_photo(pid, false);
            } else if resp.clicked() {
                if let Some(pos) = resp.interact_pointer_pos() {
                    let (la, lo) = screen_to_lat_lon(pos, app.map_center_lat, app.map_center_lon, app.map_zoom, map_rect);
                    let ids = if app.selected_photo_ids.is_empty() {
                        vec![app.active_photo_id]
                    } else {
                        app.selected_photo_ids.clone()
                    };
                    for id in &ids {
                        if let Some(p) = app.photos.iter_mut().find(|x| x.id == *id) {
                            p.gps_lat = Some(la);
                            p.gps_lon = Some(lo);
                            app.catalog_db.save_photo(p);
                        }
                    }
                    app.status_message = format!(
                        "Geotagged {} photo(s) at {:.4}, {:.4}",
                        ids.len(),
                        la,
                        lo
                    );
                }
            }
        });
}

// ============================================================================
// 2. BOOK MODULE — Multi-Page Spread Designer, PDF 1.4 & JPEG Spread Export
// ============================================================================
pub fn render_book_module(app: &mut LightRawApp, ctx: &egui::Context) {
    let ids = app.filtered_photo_ids();
    let per_spread = match app.book_layout.as_str() {
        "One Photo Per Page" => 1usize,
        "Grid 4-Up" => 4usize,
        _ => 2usize,
    };
    let interior_spreads = (ids.len() + per_spread - 1) / per_spread.max(1);
    let total_spreads = 1 + interior_spreads.max(1); // 0 = Cover, 1.. = Interior spreads
    if app.book_current_spread >= total_spreads {
        app.book_current_spread = total_spreads - 1;
    }

    if app.show_right_panel {
        egui::SidePanel::right("book_right_panel")
            .resizable(true)
            .default_width(285.0)
            .min_width(260.0)
            .max_width(380.0)
            .frame(egui::Frame::none().fill(Colors::BG_PANEL))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if lr_right_panel_header(ui, "Book Settings", &mut app.book_open_settings, None, Some("PDF / JPEG")) {
                        egui::Frame::none()
                            .fill(Color32::from_rgb(34, 34, 38))
                            .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                            .show(ui, |ui| {
                                ui.spacing_mut().item_spacing.y = 6.0;
                                egui::ComboBox::from_label("Size")
                                    .selected_text(RichText::new(&app.book_size).size(10.5))
                                    .show_ui(ui, |ui| {
                                        for sz in [
                                            "Standard Landscape (10x8 in)",
                                            "Standard Portrait (8x10 in)",
                                            "Small Square (7x7 in)",
                                            "Large Square (12x12 in)",
                                        ] {
                                            ui.selectable_value(&mut app.book_size, sz.to_string(), sz);
                                        }
                                    });
                                egui::ComboBox::from_label("Cover")
                                    .selected_text(RichText::new(&app.book_cover).size(10.5))
                                    .show_ui(ui, |ui| {
                                        for cv in ["Hardcover Image Wrap", "Hardcover Dust Jacket", "Softcover"] {
                                            ui.selectable_value(&mut app.book_cover, cv.to_string(), cv);
                                        }
                                    });
                            });
                    }

                    if lr_right_panel_header(ui, "Auto Layout & Page", &mut app.book_open_layout, None, None) {
                        egui::Frame::none()
                            .fill(Color32::from_rgb(34, 34, 38))
                            .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                            .show(ui, |ui| {
                                egui::ComboBox::from_label("Preset")
                                    .selected_text(RichText::new(&app.book_layout).size(10.5))
                                    .show_ui(ui, |ui| {
                                        for ly in ["Two-Page Spread", "One Photo Per Page", "Grid 4-Up"] {
                                            ui.selectable_value(&mut app.book_layout, ly.to_string(), ly);
                                        }
                                    });
                                lr_slider(ui, "Cell Padding", &mut app.book_page_padding, 6.0..=60.0, 22.0, "%.0f");
                                ui.checkbox(&mut app.book_show_page_numbers, "Show Page Numbers");
                                ui.checkbox(&mut app.book_bg_white, "White Paper Background");
                            });
                    }

                    if lr_right_panel_header(ui, "Title & Author Typography", &mut app.book_open_text, None, None) {
                        egui::Frame::none()
                            .fill(Color32::from_rgb(34, 34, 38))
                            .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                            .show(ui, |ui| {
                                ui.label(RichText::new("Book Title:").size(10.0).color(Colors::TEXT_MID));
                                ui.text_edit_singleline(&mut app.book_title);
                                ui.label(RichText::new("Subtitle:").size(10.0).color(Colors::TEXT_MID));
                                ui.text_edit_singleline(&mut app.book_subtitle);
                                ui.label(RichText::new("Author:").size(10.0).color(Colors::TEXT_MID));
                                ui.text_edit_singleline(&mut app.book_author);
                            });
                    }

                    ui.add_space(10.0);
                    egui::Frame::none()
                        .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                        .show(ui, |ui| {
                            if lr_bevel_button(ui, "Export Book to PDF...", ui.available_width(), 28.0).clicked() {
                                app.export_book_pages_to_disk(true);
                            }
                            ui.add_space(6.0);
                            if lr_bevel_button(ui, "Export JPEG Spreads...", ui.available_width(), 26.0).clicked() {
                                app.export_book_pages_to_disk(false);
                            }
                        });
                });
            });
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(Colors::BG_CANVAS))
        .show(ctx, |ui| {
            // Top Spread Navigation Bar
            egui::Frame::none()
                .fill(Colors::BG_TOOLBAR)
                .inner_margin(egui::Margin::symmetric(12.0, 5.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if lr_bevel_button(ui, "◀ Prev Spread", 104.0, 22.0).clicked() && app.book_current_spread > 0 {
                            app.book_current_spread -= 1;
                        }
                        let lbl = if app.book_current_spread == 0 {
                            format!("Front Cover  (1 of {} pages)", total_spreads)
                        } else {
                            format!("Spread {}  ({} of {} pages)", app.book_current_spread, app.book_current_spread + 1, total_spreads)
                        };
                        ui.label(RichText::new(lbl).size(11.0).strong().color(Colors::TEXT_BRIGHT));
                        if lr_bevel_button(ui, "Next Spread ▶", 104.0, 22.0).clicked()
                            && app.book_current_spread + 1 < total_spreads
                        {
                            app.book_current_spread += 1;
                        }
                    });
                });

            let avail = ui.available_rect_before_wrap().shrink(26.0);
            let aspect = match app.book_size.as_str() {
                "Small Square (7x7 in)" | "Large Square (12x12 in)" => 1.0,
                "Standard Portrait (8x10 in)" => 0.85,
                _ => 1.48,
            };
            let mut sw = avail.width();
            let mut sh = sw / aspect;
            if sh > avail.height() {
                sh = avail.height();
                sw = sh * aspect;
            }
            let page_rect = Rect::from_center_size(avail.center(), Vec2::new(sw, sh));
            let paper_bg = if app.book_bg_white {
                Color32::from_rgb(246, 246, 244)
            } else {
                Color32::from_rgb(24, 24, 28)
            };
            let fg_text = if app.book_bg_white {
                Color32::from_rgb(28, 28, 32)
            } else {
                Color32::from_rgb(235, 235, 240)
            };
            ui.painter().rect_filled(
                page_rect.translate(Vec2::new(4.0, 6.0)),
                3.0,
                Color32::from_rgba_unmultiplied(0, 0, 0, 110),
            );
            ui.painter().rect_filled(page_rect, 2.0, paper_bg);
            ui.painter().rect_stroke(page_rect, 2.0, Stroke::new(1.0, Color32::from_rgb(90, 90, 96)));

            if app.book_current_spread == 0 {
                // Cover Layout
                ui.painter().text(
                    Pos2::new(page_rect.center().x, page_rect.min.y + 26.0),
                    egui::Align2::CENTER_TOP,
                    &app.book_title,
                    egui::FontId::proportional(20.0),
                    fg_text,
                );
                ui.painter().text(
                    Pos2::new(page_rect.center().x, page_rect.min.y + 52.0),
                    egui::Align2::CENTER_TOP,
                    format!("{} — {}", app.book_subtitle, app.book_author),
                    egui::FontId::proportional(12.0),
                    fg_text.gamma_multiply(0.75),
                );
                if let Some(&first_id) = ids.first() {
                    if let Some(tex) = app.get_photo_texture(ctx, first_id, true) {
                        let hero = Rect::from_min_max(
                            Pos2::new(page_rect.min.x + 36.0, page_rect.min.y + 82.0),
                            Pos2::new(page_rect.max.x - 36.0, page_rect.max.y - 30.0),
                        );
                        let ts = tex.size_vec2();
                        let sc = (hero.width() / ts.x.max(1.0)).min(hero.height() / ts.y.max(1.0));
                        let dst = Rect::from_center_size(hero.center(), ts * sc);
                        ui.painter().image(
                            tex.id(),
                            dst,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    }
                }
            } else {
                // Center spine crease
                ui.painter().line_segment(
                    [
                        Pos2::new(page_rect.center().x, page_rect.min.y),
                        Pos2::new(page_rect.center().x, page_rect.max.y),
                    ],
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(120, 120, 128, 90)),
                );
                let sp_i = app.book_current_spread - 1;
                let start = sp_i * per_spread;
                let end = (start + per_spread).min(ids.len());
                let cols = if per_spread == 1 { 1 } else { 2 };
                let rows = (per_spread + cols - 1) / cols;
                let pad = app.book_page_padding;
                let cw = (page_rect.width() - pad * ((cols + 1) as f32)) / (cols as f32);
                let ch = (page_rect.height() - pad * ((rows + 1) as f32) - 18.0) / (rows as f32);

                if start < end {
                    for (slot, &pid) in ids[start..end].iter().enumerate() {
                        if let Some(tex) = app.get_photo_texture(ctx, pid, true) {
                            let r = slot / cols;
                            let c = slot % cols;
                            let cell = Rect::from_min_size(
                                Pos2::new(
                                    page_rect.min.x + pad + (c as f32) * (cw + pad),
                                    page_rect.min.y + pad + (r as f32) * (ch + pad),
                                ),
                                Vec2::new(cw.max(30.0), ch.max(30.0)),
                            );
                            let ts = tex.size_vec2();
                            let sc = (cell.width() / ts.x.max(1.0)).min(cell.height() / ts.y.max(1.0));
                            let dst = Rect::from_center_size(cell.center(), ts * sc);
                            ui.painter().image(
                                tex.id(),
                                dst,
                                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                Color32::WHITE,
                            );
                        }
                    }
                }
                if app.book_show_page_numbers {
                    ui.painter().text(
                        Pos2::new(page_rect.center().x, page_rect.max.y - 10.0),
                        egui::Align2::CENTER_BOTTOM,
                        format!("— {} —", app.book_current_spread),
                        egui::FontId::proportional(10.5),
                        fg_text.gamma_multiply(0.7),
                    );
                }
            }
        });
}

// ============================================================================
// 3. SLIDESHOW MODULE — Real Crossfade Player, Fullscreen & PDF Presentation
// ============================================================================
pub fn render_slideshow_module(app: &mut LightRawApp, ctx: &egui::Context) {
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) && app.slideshow_fullscreen {
        app.slideshow_fullscreen = false;
        app.slideshow_playing = false;
    }

    // Advance Slideshow Timer & Crossfade Envelope
    if app.slideshow_playing {
        let now = Instant::now();
        let last = app.slideshow_last_tick.unwrap_or(now);
        let elapsed = now.duration_since(last).as_secs_f32();
        if app.slideshow_last_tick.is_none() {
            app.slideshow_last_tick = Some(now);
        }
        if app.slideshow_crossfade && app.slideshow_transition_t < 1.0 {
            let cf_dur = app.slideshow_crossfade_duration.max(0.15);
            app.slideshow_transition_t = (elapsed / cf_dur).clamp(0.0, 1.0);
        } else {
            app.slideshow_transition_t = 1.0;
        }
        if elapsed >= app.slideshow_slide_duration.max(0.8) {
            let prev_id = app.active_photo_id;
            let vis = app.filtered_photo_ids();
            if !vis.is_empty() {
                if app.slideshow_random_order && vis.len() > 1 {
                    let jump = ((now.elapsed().subsec_nanos() as usize) % (vis.len() - 1)) + 1;
                    let cur = vis.iter().position(|&x| x == prev_id).unwrap_or(0);
                    app.select_photo(vis[(cur + jump) % vis.len()], false);
                } else {
                    let cur = vis.iter().position(|&x| x == prev_id).unwrap_or(0);
                    if cur + 1 < vis.len() || app.slideshow_loop {
                        app.select_photo(vis[(cur + 1) % vis.len()], false);
                    } else {
                        app.slideshow_playing = false;
                    }
                }
                app.slideshow_prev_photo_id = Some(prev_id);
                app.slideshow_transition_t = if app.slideshow_crossfade { 0.0 } else { 1.0 };
            }
            app.slideshow_last_tick = Some(now);
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(24));
    }

    if app.show_right_panel && !app.slideshow_fullscreen {
        egui::SidePanel::right("slideshow_right_panel")
            .resizable(true)
            .default_width(275.0)
            .min_width(260.0)
            .max_width(380.0)
            .frame(egui::Frame::none().fill(Colors::BG_PANEL))
            .show(ctx, |ui| {
                if lr_right_panel_header(ui, "Options, Overlays & Playback", &mut app.slideshow_open_options, None, None) {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(34, 34, 38))
                        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing.y = 6.0;
                            ui.checkbox(&mut app.slideshow_show_identity, "Show Identity Plate");
                            if app.slideshow_show_identity {
                                ui.text_edit_singleline(&mut app.identity_plate_owner);
                            }
                            ui.checkbox(&mut app.slideshow_show_caption, "Show EXIF & Caption Overlay");
                            ui.checkbox(&mut app.slideshow_stroke_border, "Stroke Photo Border");
                            ui.checkbox(&mut app.slideshow_backdrop_dark, "Dark Cinema Backdrop");
                            ui.separator();
                            lr_slider(ui, "Slide Length", &mut app.slideshow_slide_duration, 1.0..=20.0, 4.0, "%.1f");
                            ui.checkbox(&mut app.slideshow_crossfade, "Crossfade Between Slides");
                            if app.slideshow_crossfade {
                                lr_slider(ui, "Fade Length", &mut app.slideshow_crossfade_duration, 0.2..=4.0, 1.0, "%.1f");
                            }
                            ui.checkbox(&mut app.slideshow_loop, "Repeat / Loop Slideshow");
                            ui.checkbox(&mut app.slideshow_random_order, "Random Shuffle Order");
                            ui.separator();
                            let play_lbl = if app.slideshow_playing {
                                "⏸ Pause Preview"
                            } else {
                                "▶ Preview Slideshow"
                            };
                            if lr_bevel_button(ui, play_lbl, ui.available_width(), 26.0).clicked() {
                                app.slideshow_playing = !app.slideshow_playing;
                                app.slideshow_last_tick = Some(Instant::now());
                            }
                            if lr_bevel_button(ui, "⛶ Play Fullscreen (Esc to Exit)", ui.available_width(), 26.0).clicked() {
                                app.slideshow_fullscreen = true;
                                app.slideshow_playing = true;
                                app.slideshow_last_tick = Some(Instant::now());
                            }
                            if lr_bevel_button(ui, "Export PDF Slideshow...", ui.available_width(), 26.0).clicked() {
                                app.export_slideshow_to_pdf();
                            }
                        });
                }
            });
    }

    let bg = if app.slideshow_backdrop_dark {
        Color32::from_rgb(12, 12, 14)
    } else {
        Color32::from_rgb(240, 240, 242)
    };
    let fg = if app.slideshow_backdrop_dark {
        Color32::from_rgb(235, 235, 240)
    } else {
        Color32::from_rgb(28, 28, 32)
    };

    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(bg))
        .show(ctx, |ui| {
            let stage = ui.available_rect_before_wrap().shrink(28.0);
            let cur_id = app.active_photo_id;
            let photo = app.active_photo().clone();

            // Outgoing crossfade photo
            if app.slideshow_crossfade && app.slideshow_transition_t < 0.999 {
                if let Some(prev_id) = app.slideshow_prev_photo_id {
                    if let Some(prev_tex) = app.get_photo_texture(ctx, prev_id, true) {
                        let ts = prev_tex.size_vec2();
                        let sc = (stage.width() / ts.x.max(1.0)).min((stage.height() - 48.0) / ts.y.max(1.0));
                        let dst = Rect::from_center_size(stage.center(), ts * sc);
                        let a = ((1.0 - app.slideshow_transition_t) * 255.0).round() as u8;
                        ui.painter().image(
                            prev_tex.id(),
                            dst,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::from_rgba_unmultiplied(255, 255, 255, a),
                        );
                    }
                }
            }

            if let Some(tex) = app.get_photo_texture(ctx, cur_id, true) {
                let ts = tex.size_vec2();
                let sc = (stage.width() / ts.x.max(1.0)).min((stage.height() - 48.0) / ts.y.max(1.0));
                let dst = Rect::from_center_size(stage.center(), ts * sc);
                let a = (app.slideshow_transition_t.clamp(0.0, 1.0) * 255.0).round() as u8;
                ui.painter().image(
                    tex.id(),
                    dst,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::from_rgba_unmultiplied(255, 255, 255, a),
                );
                if app.slideshow_stroke_border {
                    ui.painter().rect_stroke(dst, 0.0, Stroke::new(1.5, fg.gamma_multiply(0.65)));
                }
            }

            if app.slideshow_show_identity {
                ui.painter().text(
                    stage.min + Vec2::new(4.0, 0.0),
                    egui::Align2::LEFT_TOP,
                    format!("{}  •  {}", app.identity_plate_subtitle, app.identity_plate_owner),
                    egui::FontId::proportional(15.0),
                    fg,
                );
            }
            if app.slideshow_show_caption {
                let cap = if photo.caption.trim().is_empty() {
                    format!(
                        "{}  —  {} at f/{:.1}, ISO {}",
                        photo.file_name, photo.shutter_speed, photo.aperture_f, photo.iso
                    )
                } else {
                    format!("{} — {}", photo.title, photo.caption)
                };
                ui.painter().text(
                    Pos2::new(stage.center().x, stage.max.y),
                    egui::Align2::CENTER_BOTTOM,
                    cap,
                    egui::FontId::proportional(12.5),
                    fg.gamma_multiply(0.85),
                );
            }
        });
}

// ============================================================================
// 4. PRINT MODULE — Paginated Contact Sheet, 300 PPI PDF/JPEG & macOS Printer
// ============================================================================
pub fn render_print_module(app: &mut LightRawApp, ctx: &egui::Context) {
    let ids = if app.selected_photo_ids.len() > 1 {
        app.selected_photo_ids.clone()
    } else {
        app.filtered_photo_ids()
    };
    let (rows, cols) = if app.print_layout_style == "Picture Package" {
        (2usize, 2usize)
    } else {
        (app.print_rows.clamp(1, 8), app.print_cols.clamp(1, 8))
    };
    let per_page = (rows * cols).max(1);
    let total_pages = if app.print_layout_style == "Picture Package" {
        ids.len().max(1)
    } else {
        ((ids.len() + per_page - 1) / per_page).max(1)
    };
    if app.print_current_page >= total_pages {
        app.print_current_page = total_pages - 1;
    }

    if app.show_right_panel {
        egui::SidePanel::right("print_right_panel")
            .resizable(true)
            .default_width(285.0)
            .min_width(260.0)
            .max_width(380.0)
            .frame(egui::Frame::none().fill(Colors::BG_PANEL))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    if lr_right_panel_header(ui, "Layout Style & Grid", &mut app.print_open_layout, None, None) {
                        egui::Frame::none()
                            .fill(Color32::from_rgb(34, 34, 38))
                            .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                            .show(ui, |ui| {
                                ui.spacing_mut().item_spacing.y = 6.0;
                                for style in ["Single Image / Contact Sheet", "Picture Package", "Custom Package"] {
                                    ui.radio_value(&mut app.print_layout_style, style.to_string(), style);
                                }
                                ui.separator();
                                let mut r_f = app.print_rows as f32;
                                let mut c_f = app.print_cols as f32;
                                if lr_slider(ui, "Rows", &mut r_f, 1.0..=6.0, 2.0, "%.0f") {
                                    app.print_rows = r_f.round() as usize;
                                }
                                if lr_slider(ui, "Columns", &mut c_f, 1.0..=6.0, 2.0, "%.0f") {
                                    app.print_cols = c_f.round() as usize;
                                }
                                lr_slider(ui, "Margins (mm)", &mut app.print_margin_mm, 4.0..=40.0, 12.0, "%.0f");
                                lr_slider(ui, "Cell Spacing", &mut app.print_cell_spacing, 0.0..=32.0, 10.0, "%.0f");
                            });
                    }

                    if lr_right_panel_header(ui, "Guides & Page Overlays", &mut app.print_open_guides, None, None) {
                        egui::Frame::none()
                            .fill(Color32::from_rgb(34, 34, 38))
                            .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                            .show(ui, |ui| {
                                ui.checkbox(&mut app.print_show_guides, "Show Rulers & Bleed Guides");
                                ui.checkbox(&mut app.print_show_dimensions, "Show Cell Dimensions");
                                ui.checkbox(&mut app.print_stroke_border, "Photo Stroke Border");
                                ui.checkbox(&mut app.print_show_page_info, "Print Photo Filename Info");
                                ui.checkbox(&mut app.print_show_watermark, "Identity Plate Watermark");
                            });
                    }

                    if lr_right_panel_header(ui, "Print Job", &mut app.print_open_job, None, Some("300 PPI")) {
                        egui::Frame::none()
                            .fill(Color32::from_rgb(34, 34, 38))
                            .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                            .show(ui, |ui| {
                                lr_slider(ui, "Resolution", &mut app.print_dpi, 72.0..=600.0, 300.0, "%.0f");
                                ui.add_space(6.0);
                                ui.horizontal(|ui| {
                                    let bw = ((ui.available_width() - 6.0) * 0.5).max(40.0);
                                    if lr_bevel_button(ui, "Page Setup...", bw, 24.0).clicked() {
                                        app.run_native_page_setup();
                                    }
                                    if lr_bevel_button(ui, "Print...", bw, 24.0).clicked() {
                                        app.print_sheet_to_printer();
                                    }
                                });
                                ui.add_space(5.0);
                                let full_bw = ui.available_width().max(80.0);
                                if lr_bevel_button(ui, "Print to PDF File...", full_bw, 26.0).clicked() {
                                    app.export_print_sheet_to_disk(true);
                                }
                                ui.add_space(4.0);
                                if lr_bevel_button(ui, "Print to JPEG Sheets...", full_bw, 24.0).clicked() {
                                    app.export_print_sheet_to_disk(false);
                                }
                            });
                    }
                });
            });
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(Colors::BG_CANVAS))
        .show(ctx, |ui| {
            // Top Page Navigation Bar
            egui::Frame::none()
                .fill(Colors::BG_TOOLBAR)
                .inner_margin(egui::Margin::symmetric(12.0, 5.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if lr_bevel_button(ui, "◀ Prev Page", 92.0, 22.0).clicked() && app.print_current_page > 0 {
                            app.print_current_page -= 1;
                        }
                        ui.label(
                            RichText::new(format!("Page {} of {}  ({:.0} PPI)", app.print_current_page + 1, total_pages, app.print_dpi))
                                .size(11.0)
                                .strong()
                                .color(Colors::TEXT_BRIGHT),
                        );
                        if lr_bevel_button(ui, "Next Page ▶", 92.0, 22.0).clicked()
                            && app.print_current_page + 1 < total_pages
                        {
                            app.print_current_page += 1;
                        }
                    });
                });

            let avail = ui.available_rect_before_wrap().shrink(26.0);
            let a4_aspect = 210.0 / 297.0;
            let mut pw = avail.width();
            let mut ph = pw / a4_aspect;
            if ph > avail.height() {
                ph = avail.height();
                pw = ph * a4_aspect;
            }
            let paper = Rect::from_center_size(avail.center(), Vec2::new(pw, ph));
            ui.painter().rect_filled(paper, 1.5, Color32::WHITE);
            ui.painter().rect_stroke(paper, 1.5, Stroke::new(1.0, Color32::from_rgb(100, 100, 108)));

            let margin = (app.print_margin_mm * 1.4).clamp(8.0, 64.0);
            let inner = paper.shrink(margin);
            if app.print_show_guides {
                ui.painter().rect_stroke(
                    inner,
                    0.0,
                    Stroke::new(0.8, Color32::from_rgba_unmultiplied(40, 150, 235, 130)),
                );
            }

            let gap = app.print_cell_spacing;
            let cw = (inner.width() - gap * ((cols.saturating_sub(1)) as f32)) / (cols as f32);
            let ch = (inner.height() - gap * ((rows.saturating_sub(1)) as f32)) / (rows as f32);

            for r in 0..rows {
                for c in 0..cols {
                    let slot = r * cols + c;
                    let pid_opt = if app.print_layout_style == "Picture Package" {
                        ids.get(app.print_current_page).copied()
                    } else {
                        ids.get(app.print_current_page * per_page + slot).copied()
                    };
                    let cell = Rect::from_min_size(
                        Pos2::new(
                            inner.min.x + (c as f32) * (cw + gap),
                            inner.min.y + (r as f32) * (ch + gap),
                        ),
                        Vec2::new(cw.max(20.0), ch.max(20.0)),
                    );
                    if app.print_show_guides {
                        ui.painter().rect_stroke(cell, 0.0, Stroke::new(0.5, Color32::from_rgb(215, 215, 220)));
                    }
                    if let Some(pid) = pid_opt {
                        let img_area = if app.print_show_page_info {
                            Rect::from_min_max(cell.min, Pos2::new(cell.max.x, cell.max.y - 14.0))
                        } else {
                            cell
                        };
                        if let Some(tex) = app.get_photo_texture(ctx, pid, true) {
                            let ts = tex.size_vec2();
                            let sc = (img_area.width() / ts.x.max(1.0)).min(img_area.height() / ts.y.max(1.0));
                            let dst = Rect::from_center_size(img_area.center(), ts * sc);
                            ui.painter().image(
                                tex.id(),
                                dst,
                                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                Color32::WHITE,
                            );
                            if app.print_stroke_border {
                                ui.painter().rect_stroke(dst, 0.0, Stroke::new(1.0, Color32::BLACK));
                            }
                        }
                        if app.print_show_page_info {
                            if let Some(pm) = app.photos.iter().find(|p| p.id == pid) {
                                ui.painter().text(
                                    Pos2::new(cell.min.x + 2.0, cell.max.y - 2.0),
                                    egui::Align2::LEFT_BOTTOM,
                                    &pm.file_name,
                                    egui::FontId::proportional(9.0),
                                    Color32::from_rgb(55, 55, 60),
                                );
                            }
                        }
                    }
                }
            }
        });
}

// ============================================================================
// 5. WEB MODULE — Unlimited Responsive HTML5 Lightbox Gallery Builder
// ============================================================================
pub fn render_web_module(app: &mut LightRawApp, ctx: &egui::Context) {
    if app.show_right_panel {
        egui::SidePanel::right("web_right_panel")
            .resizable(true)
            .default_width(280.0)
            .min_width(260.0)
            .max_width(380.0)
            .frame(egui::Frame::none().fill(Colors::BG_PANEL))
            .show(ctx, |ui| {
                if lr_right_panel_header(ui, "Site Info, Appearance & Output", &mut app.web_open_settings, None, Some("HTML5")) {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(34, 34, 38))
                        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing.y = 6.0;
                            ui.label(RichText::new("Site Title:").size(10.0).color(Colors::TEXT_MID));
                            ui.text_edit_singleline(&mut app.web_site_title);
                            ui.label(RichText::new("Collection Title:").size(10.0).color(Colors::TEXT_MID));
                            ui.text_edit_singleline(&mut app.web_collection_title);
                            ui.label(RichText::new("Description:").size(10.0).color(Colors::TEXT_MID));
                            ui.text_edit_singleline(&mut app.web_description);
                            ui.separator();
                            let mut cols_f = app.web_columns as f32;
                            if lr_slider(ui, "Grid Columns", &mut cols_f, 2.0..=6.0, 4.0, "%.0f") {
                                app.web_columns = cols_f.round() as usize;
                            }
                            let mut q_f = app.web_quality as f32;
                            if lr_slider(ui, "JPEG Quality", &mut q_f, 50.0..=100.0, 88.0, "%.0f") {
                                app.web_quality = q_f.round() as u8;
                            }
                            ui.checkbox(&mut app.web_dark_theme, "Dark Portfolio Theme");
                            ui.checkbox(&mut app.web_show_exif, "Show Camera EXIF Captions");
                            ui.add_space(8.0);
                            if lr_bevel_button(ui, "Export & Open HTML5 Web Gallery...", ui.available_width(), 28.0).clicked() {
                                app.export_and_open_web_gallery();
                            }
                        });
                }
            });
    }

    let bg = if app.web_dark_theme {
        Color32::from_rgb(20, 22, 26)
    } else {
        Color32::from_rgb(245, 245, 247)
    };
    let card_bg = if app.web_dark_theme {
        Color32::from_rgb(32, 34, 40)
    } else {
        Color32::WHITE
    };
    let fg = if app.web_dark_theme {
        Color32::from_rgb(235, 235, 240)
    } else {
        Color32::from_rgb(28, 28, 32)
    };

    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(bg))
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(18.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(&app.web_site_title).size(22.0).strong().color(fg));
                    ui.label(
                        RichText::new(format!("{} — {}", app.web_collection_title, app.web_description))
                            .size(12.5)
                            .color(fg.gamma_multiply(0.75)),
                    );
                });
                ui.add_space(16.0);

                let ids = app.filtered_photo_ids();
                let cols = app.web_columns.clamp(2, 6);
                let pad = 20.0;
                let avail_w = (ui.available_width() - pad * 2.0).max(200.0);
                let cw = (avail_w - ((cols - 1) as f32) * 14.0) / (cols as f32);

                for row in ids.chunks(cols) {
                    ui.horizontal(|ui| {
                        ui.add_space(pad);
                        ui.spacing_mut().item_spacing.x = 14.0;
                        for &pid in row {
                            let tex_opt = app.get_thumb_texture(ctx, pid);
                            let p_opt = app.photos.iter().find(|p| p.id == pid).cloned();
                            egui::Frame::none()
                                .fill(card_bg)
                                .rounding(6.0)
                                .inner_margin(egui::Margin::same(8.0))
                                .show(ui, |ui| {
                                    ui.set_width((cw - 16.0).max(80.0));
                                    let (ir, _) = ui.allocate_exact_size(
                                        Vec2::new((cw - 16.0).max(80.0), (cw * 0.72).max(64.0)),
                                        Sense::hover(),
                                    );
                                    ui.painter().rect_filled(ir, 4.0, Color32::from_rgb(14, 15, 18));
                                    if let Some(tex) = tex_opt {
                                        let ts = tex.size_vec2();
                                        let sc = (ir.width() / ts.x.max(1.0)).min(ir.height() / ts.y.max(1.0));
                                        let dst = Rect::from_center_size(ir.center(), ts * sc);
                                        ui.painter().image(
                                            tex.id(),
                                            dst,
                                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                            Color32::WHITE,
                                        );
                                    }
                                    if let Some(pm) = p_opt {
                                        ui.add_space(4.0);
                                        ui.label(RichText::new(&pm.file_name).size(10.5).strong().color(fg));
                                        if app.web_show_exif {
                                            ui.label(
                                                RichText::new(format!("{} • ISO {}", pm.shutter_speed, pm.iso))
                                                    .size(9.5)
                                                    .color(fg.gamma_multiply(0.68)),
                                            );
                                        }
                                    }
                                });
                        }
                    });
                    ui.add_space(14.0);
                }
            });
        });
}

// ============================================================================
// 6. FULL-SCREEN IMPORT WORKSPACE (FROM / Copy-Move-Add / TO)
// ============================================================================
pub fn render_import_workspace(app: &mut LightRawApp, ctx: &egui::Context) {
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.show_import_modal = false;
        return;
    }

    // Bottom Pinned Action Bar (`Cancel` | `Import Preset` | `Import`)
    egui::TopBottomPanel::bottom("import_bottom_bar")
        .exact_height(46.0)
        .frame(egui::Frame::none().fill(Color32::from_rgb(28, 28, 32)))
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.add_space(12.0);
                if lr_bevel_button(ui, "Select a Source Folder...", 168.0, 28.0).clicked() {
                    app.open_native_folder_picker();
                }
                ui.separator();
                if ui.small_button("Check All").clicked() {
                    for c in &mut app.import_candidates {
                        c.is_checked = true;
                    }
                }
                if ui.small_button("Uncheck All").clicked() {
                    for c in &mut app.import_candidates {
                        c.is_checked = false;
                    }
                }

                let checked_n = app.import_candidates.iter().filter(|c| c.is_checked).count();
                ui.label(
                    RichText::new(format!(
                        "{} of {} candidate file(s) selected",
                        checked_n,
                        app.import_candidates.len()
                    ))
                    .size(11.0)
                    .color(Colors::TEXT_MID),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(14.0);
                    let imp_btn = ui.add_sized(
                        [116.0, 30.0],
                        egui::Button::new(RichText::new("Import").size(12.5).strong().color(Color32::WHITE))
                            .fill(Colors::ACCENT_BLUE),
                    );
                    if imp_btn.clicked() && checked_n > 0 {
                        app.execute_current_import();
                    }
                    if lr_bevel_button(ui, "Cancel", 86.0, 30.0).clicked() {
                        app.show_import_modal = false;
                    }
                });
            });
        });

    // Left Source Panel (`FROM`)
    egui::SidePanel::left("import_left_source")
        .default_width(250.0)
        .frame(egui::Frame::none().fill(Colors::BG_PANEL))
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(Colors::BG_HEADER)
                .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("FROM").size(13.0).strong().color(Colors::TEXT_BRIGHT));
                });

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(6.0);
                let home = std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/Users"));
                let mut roots = vec![
                    home.join("Pictures"),
                    home.join("Desktop"),
                    home.join("Downloads"),
                    app.workspace_dir.clone(),
                ];
                if let Ok(vols) = std::fs::read_dir("/Volumes") {
                    for v in vols.flatten() {
                        roots.push(v.path());
                    }
                }
                let mut scan_req: Option<PathBuf> = None;
                for root in roots {
                    if !root.exists() {
                        continue;
                    }
                    let name = root.file_name().and_then(|s| s.to_str()).unwrap_or("Volume");
                    let sel = app.import_selected_source.as_ref() == Some(&root);
                    if lr_selectable_row(ui, sel, &format!("📁 {}", name), None).clicked() {
                        scan_req = Some(root.clone());
                    }
                    if let Ok(subs) = std::fs::read_dir(&root) {
                        let mut sub_dirs: Vec<PathBuf> = subs
                            .flatten()
                            .map(|e| e.path())
                            .filter(|p| p.is_dir())
                            .take(12)
                            .collect();
                        sub_dirs.sort();
                        for sd in sub_dirs {
                            let sname = sd.file_name().and_then(|s| s.to_str()).unwrap_or("Folder");
                            if sname.starts_with('.') {
                                continue;
                            }
                            let ssel = app.import_selected_source.as_ref() == Some(&sd);
                            if lr_selectable_row(ui, ssel, &format!("    📂 {}", sname), None).clicked() {
                                scan_req = Some(sd);
                            }
                        }
                    }
                }
                if let Some(folder) = scan_req {
                    app.scan_import_source(&folder);
                }
            });
        });

    // Right Destination & Preset Panel (`TO`)
    egui::SidePanel::right("import_right_dest")
        .default_width(280.0)
        .frame(egui::Frame::none().fill(Colors::BG_PANEL))
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(Colors::BG_HEADER)
                .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("TO — CATALOG & DISK").size(13.0).strong().color(Colors::TEXT_BRIGHT));
                });

            egui::ScrollArea::vertical().show(ui, |ui| {
                if lr_right_panel_header(ui, "File Handling", &mut app.import_file_handling_open, None, None) {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(34, 34, 38))
                        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                        .show(ui, |ui| {
                            egui::ComboBox::from_label("Build Previews")
                                .selected_text(&app.import_build_previews)
                                .show_ui(ui, |ui| {
                                    for bp in ["Minimal", "Embedded & Sidecar", "Standard", "1:1"] {
                                        ui.selectable_value(&mut app.import_build_previews, bp.to_string(), bp);
                                    }
                                });
                            ui.checkbox(&mut app.import_build_smart_previews, "Build Smart Previews");
                            ui.checkbox(&mut app.import_dont_import_duplicates, "Don't Import Suspected Duplicates");
                        });
                }

                if lr_right_panel_header(ui, "Apply During Import", &mut app.import_apply_during_open, None, None) {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(34, 34, 38))
                        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                        .show(ui, |ui| {
                            let preset_names: Vec<String> = std::iter::once("None".to_string())
                                .chain(std::iter::once("Auto Tone".to_string()))
                                .chain(app.presets.iter().filter(|p| p.is_visible).map(|p| p.name.clone()))
                                .collect();
                            ui.label(RichText::new("Develop Settings:").size(10.0).color(Colors::TEXT_MID));
                            egui::ComboBox::from_id_salt("imp_dev_preset_combo")
                                .selected_text(RichText::new(&app.import_develop_preset).size(10.5))
                                .width(ui.available_width())
                                .show_ui(ui, |ui| {
                                    for nm in preset_names {
                                        ui.selectable_value(&mut app.import_develop_preset, nm.clone(), nm);
                                    }
                                });
                            ui.add_space(6.0);
                            ui.label(RichText::new("Keywords (comma-separated):").size(10.0).color(Colors::TEXT_MID));
                            ui.text_edit_multiline(&mut app.import_keywords);
                        });
                }

                if app.import_action != ImportActionChoice::Add
                    && lr_right_panel_header(ui, "Destination", &mut app.import_destination_open, None, None)
                {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(34, 34, 38))
                        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new("Destination Root:").size(10.0).color(Colors::TEXT_MID));
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut app.import_destination_folder)
                                        .desired_width(ui.available_width() - 62.0),
                                );
                                if ui.small_button("Choose...").clicked() {
                                    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                                        app.import_destination_folder = dir.to_string_lossy().to_string();
                                    }
                                }
                            });
                            ui.label(RichText::new("Into Subfolder:").size(10.0).color(Colors::TEXT_MID));
                            ui.text_edit_singleline(&mut app.import_subfolder);
                        });
                }
            });
        });

    // Center Action Mode Header + Candidate Grid/Loupe
    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(Colors::BG_CANVAS))
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(Colors::BG_HEADER)
                .inner_margin(egui::Margin::symmetric(16.0, 8.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for act in [
                            ImportActionChoice::CopyAsDng,
                            ImportActionChoice::Copy,
                            ImportActionChoice::Move,
                            ImportActionChoice::Add,
                        ] {
                            let active = app.import_action == act;
                            if ui
                                .selectable_label(
                                    active,
                                    RichText::new(act.label()).size(12.5).strong().color(if active {
                                        Color32::WHITE
                                    } else {
                                        Colors::TEXT_MID
                                    }),
                                )
                                .on_hover_text(act.subtitle())
                                .clicked()
                            {
                                app.import_action = act;
                            }
                            ui.add_space(10.0);
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.selectable_label(app.import_view_mode == ImportViewMode::Loupe, "▣ Loupe").clicked() {
                                app.import_view_mode = ImportViewMode::Loupe;
                            }
                            if ui.selectable_label(app.import_view_mode == ImportViewMode::Grid, "⊞ Grid").clicked() {
                                app.import_view_mode = ImportViewMode::Grid;
                            }
                            egui::ComboBox::from_id_salt("imp_sort_combo")
                                .selected_text(format!("Sort: {}", app.import_sort_mode))
                                .show_ui(ui, |ui| {
                                    for sm in ["File Name", "File Size"] {
                                        if ui.selectable_value(&mut app.import_sort_mode, sm.to_string(), sm).clicked() {
                                            if sm == "File Size" {
                                                app.import_candidates.sort_by_key(|c| c.file_size_bytes);
                                            } else {
                                                app.import_candidates.sort_by(|a, b| a.file_name.cmp(&b.file_name));
                                            }
                                        }
                                    }
                                });
                        });
                    });
                });

            if app.import_candidates.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        RichText::new("Choose a folder on the left or click 'Select a Source Folder...' below to preview RAW/JPEG/TIFF files.")
                            .size(12.5)
                            .color(Colors::TEXT_MID),
                    );
                });
                return;
            }

            if app.import_view_mode == ImportViewMode::Loupe {
                if let Some(first_checked) = app
                    .import_candidates
                    .iter()
                    .find(|c| c.is_checked)
                    .or_else(|| app.import_candidates.first())
                    .cloned()
                {
                    let avail = ui.available_rect_before_wrap().shrink(20.0);
                    if let Some(tex) = app.get_import_candidate_texture(ctx, &first_checked.path) {
                        let ts = tex.size_vec2();
                        let sc = (avail.width() / ts.x.max(1.0)).min(avail.height() / ts.y.max(1.0));
                        let dst = Rect::from_center_size(avail.center(), ts * sc);
                        ui.painter().image(
                            tex.id(),
                            dst,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    }
                }
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(10.0);
                    let cw = app.import_grid_thumb_size.clamp(100.0, 220.0);
                    let cols = ((ui.available_width() - 20.0) / (cw + 10.0)).floor().max(1.0) as usize;
                    let total = app.import_candidates.len();
                    for r_start in (0..total).step_by(cols) {
                        let r_end = (r_start + cols).min(total);
                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            for idx in r_start..r_end {
                                let cand_path = app.import_candidates[idx].path.clone();
                                let tex_opt = app.get_import_candidate_texture(ctx, &cand_path);
                                let cand = &mut app.import_candidates[idx];
                                egui::Frame::none()
                                    .fill(if cand.is_checked {
                                        Color32::from_rgb(54, 54, 60)
                                    } else {
                                        Color32::from_rgb(34, 34, 38)
                                    })
                                    .rounding(3.0)
                                    .inner_margin(egui::Margin::same(6.0))
                                    .show(ui, |ui| {
                                        ui.set_width(cw);
                                        ui.horizontal(|ui| {
                                            ui.checkbox(&mut cand.is_checked, "");
                                            ui.label(
                                                RichText::new(&cand.file_name)
                                                    .size(9.5)
                                                    .color(if cand.is_duplicate {
                                                        Colors::TEXT_DIM
                                                    } else {
                                                        Colors::TEXT_MAIN
                                                    }),
                                            );
                                        });
                                        let (tr, _) = ui.allocate_exact_size(Vec2::new(cw, cw * 0.75), Sense::hover());
                                        ui.painter().rect_filled(tr, 2.0, Color32::from_rgb(20, 20, 24));
                                        if let Some(tex) = tex_opt {
                                            let ts = tex.size_vec2();
                                            let sc = (tr.width() / ts.x.max(1.0)).min(tr.height() / ts.y.max(1.0));
                                            let dst = Rect::from_center_size(tr.center(), ts * sc);
                                            ui.painter().image(
                                                tex.id(),
                                                dst,
                                                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                                Color32::WHITE,
                                            );
                                        }
                                    });
                            }
                        });
                        ui.add_space(8.0);
                    }
                });
            }
        });
}

// ============================================================================
// 7. EXPORT MODAL (Full Sensor Res JPEG / PNG / TIFF / Original + Sidecar XMP)
// ============================================================================
pub fn render_modals(app: &mut LightRawApp, ctx: &egui::Context) {
    if !app.show_export_modal {
        return;
    }

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.show_export_modal = false;
        return;
    }

    let mut open = app.show_export_modal;
    let mut trigger_export = false;
    let sel_count = app.selected_photo_ids.len().max(1);

    egui::Window::new(format!("Export {} Photo(s)", sel_count))
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_width(540.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;

            // 1. Export Location
            ui.label(RichText::new("Export Location").size(12.0).strong().color(Colors::TEXT_BRIGHT));
            ui.horizontal(|ui| {
                ui.label("Export To:");
                ui.add(egui::TextEdit::singleline(&mut app.export_folder).desired_width(310.0));
                if ui.button("Choose...").clicked() {
                    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                        app.export_folder = dir.to_string_lossy().to_string();
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label("Subfolder:");
                ui.add(egui::TextEdit::singleline(&mut app.export_subfolder).desired_width(240.0));
            });

            ui.separator();

            // 2. File Naming
            ui.label(RichText::new("File Naming").size(12.0).strong().color(Colors::TEXT_BRIGHT));
            ui.horizontal(|ui| {
                ui.checkbox(&mut app.export_rename_enabled, "Rename To Custom Sequence:");
                if app.export_rename_enabled {
                    ui.add(
                        egui::TextEdit::singleline(&mut app.export_rename_prefix)
                            .hint_text("Prefix")
                            .desired_width(180.0),
                    );
                }
            });

            ui.separator();

            // 3. File Settings
            ui.label(RichText::new("File Settings").size(12.0).strong().color(Colors::TEXT_BRIGHT));
            ui.horizontal(|ui| {
                ui.label("Image Format:");
                egui::ComboBox::from_id_salt("exp_fmt_combo")
                    .selected_text(&app.export_format)
                    .show_ui(ui, |ui| {
                        for fmt in [
                            "JPEG",
                            "PNG",
                            "TIFF (16-bit)",
                            "TIFF (8-bit)",
                            "Original + Sidecar XMP",
                        ] {
                            ui.selectable_value(&mut app.export_format, fmt.to_string(), fmt);
                        }
                    });
                ui.label("Color Space:");
                egui::ComboBox::from_id_salt("exp_cs_combo")
                    .selected_text(&app.export_color_space)
                    .show_ui(ui, |ui| {
                        for cs in ["sRGB", "Display P3", "AdobeRGB (1998)", "ProPhoto RGB"] {
                            ui.selectable_value(&mut app.export_color_space, cs.to_string(), cs);
                        }
                    });
            });
            if app.export_format == "JPEG" {
                let mut q = app.export_quality as f32;
                if lr_slider(ui, "Quality", &mut q, 10.0..=100.0, 92.0, "%.0f") {
                    app.export_quality = q.round() as u8;
                }
            }

            ui.separator();

            // 4. Image Sizing & Output Sharpening
            ui.label(RichText::new("Image Sizing & Output Sharpening").size(12.0).strong().color(Colors::TEXT_BRIGHT));
            ui.horizontal(|ui| {
                ui.checkbox(
                    &mut app.export_resize_enabled,
                    "Resize to Fit Long Edge (uncheck for Full Native Sensor Resolution):",
                );
                if app.export_resize_enabled {
                    let mut px = app.export_long_edge_px as f32;
                    if ui.add(egui::DragValue::new(&mut px).range(256.0..=12000.0).suffix(" px")).changed() {
                        app.export_long_edge_px = px.round() as u32;
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label("Sharpen For:");
                egui::ComboBox::from_id_salt("exp_sharp_combo")
                    .selected_text(&app.export_sharpening)
                    .show_ui(ui, |ui| {
                        for sh in [
                            "None",
                            "Screen — Low",
                            "Screen — Standard",
                            "Screen — High",
                            "Matte Paper — Standard",
                            "Glossy Paper — High",
                        ] {
                            ui.selectable_value(&mut app.export_sharpening, sh.to_string(), sh);
                        }
                    });
            });

            ui.separator();

            // 5. Metadata Sidecar & Watermarking
            ui.checkbox(&mut app.export_xmp_sidecar, "Write Companion .xmp Develop Sidecar File");
            ui.horizontal(|ui| {
                ui.checkbox(&mut app.export_watermark_enabled, "Watermark:");
                if app.export_watermark_enabled {
                    egui::ComboBox::from_id_salt("export_wm_preset_select")
                        .selected_text(&app.active_watermark_preset_name)
                        .width(200.0)
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
                            ui.separator();
                            if ui.selectable_label(false, "Edit Watermarks...").clicked() {
                                app.show_watermark_editor_modal = true;
                            }
                        });

                    if ui.button("Edit Watermarks...").clicked() {
                        app.show_watermark_editor_modal = true;
                    }
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add_sized(
                            [112.0, 28.0],
                            egui::Button::new(RichText::new("Export").strong().color(Color32::WHITE))
                                .fill(Colors::ACCENT_BLUE),
                        )
                        .clicked()
                    {
                        trigger_export = true;
                    }
                    if lr_bevel_button(ui, "Cancel", 86.0, 28.0).clicked() {
                        app.show_export_modal = false;
                    }
                });
            });
        });

    if trigger_export {
        app.execute_batch_export();
        app.show_export_modal = false;
    } else if !open {
        app.show_export_modal = false;
    }
}
