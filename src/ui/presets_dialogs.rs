use super::LightRawApp;
use crate::params::DevelopParams;
use crate::presets::{
    apply_settings_with_mask, export_preset_to_xmp, CopySettingsMask, DevelopPreset,
};
use egui::{Color32, Pos2, Rect, RichText, Sense, Stroke, Vec2};

pub fn render_presets_and_copy_modals(app: &mut LightRawApp, ctx: &egui::Context) {
    if app.show_copy_settings_modal {
        render_copy_settings_modal(app, ctx);
    }
    if app.show_sync_settings_modal {
        render_sync_settings_modal(app, ctx);
    }
    if app.show_create_preset_modal {
        render_create_preset_modal(app, ctx);
    }
    if app.show_rename_preset_modal.is_some() {
        render_rename_preset_modal(app, ctx);
    }
    if app.show_move_preset_modal.is_some() {
        render_move_preset_modal(app, ctx);
    }
    if app.show_manage_presets_modal {
        render_manage_presets_modal(app, ctx);
    }
}

pub fn render_copy_settings_modal(app: &mut LightRawApp, ctx: &egui::Context) {
    let mut is_open = true;

    egui::Window::new("Copy Settings")
        .open(&mut is_open)
        .collapsible(false)
        .resizable(true)
        .default_width(860.0)
        .default_height(560.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(Color32::from_rgb(38, 38, 42))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(72, 72, 80)))
                .inner_margin(egui::Margin::symmetric(18.0, 14.0))
                .rounding(8.0),
        )
        .show(ctx, |ui| {
            render_settings_columns(ui, &mut app.clipboard_mask);

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                if ui.button(RichText::new("Check All").size(12.0)).clicked() {
                    app.clipboard_mask.set_all(true);
                    app.copy_settings_subset = "All".into();
                }
                if ui.button(RichText::new("Check None").size(12.0)).clicked() {
                    app.clipboard_mask.set_all(false);
                    app.copy_settings_subset = "None".into();
                }

                ui.add_space(10.0);
                ui.label(
                    RichText::new("Subset:")
                        .size(12.0)
                        .color(Color32::from_rgb(190, 190, 195)),
                );
                egui::ComboBox::from_id_salt("copy_settings_subset_combo")
                    .selected_text(&app.copy_settings_subset)
                    .width(160.0)
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_value(
                                &mut app.copy_settings_subset,
                                "Default".into(),
                                "Default",
                            )
                            .clicked()
                        {
                            app.clipboard_mask = CopySettingsMask::default();
                        }
                        if ui
                            .selectable_value(&mut app.copy_settings_subset, "All".into(), "All")
                            .clicked()
                        {
                            app.clipboard_mask = CopySettingsMask::all();
                        }
                        if ui
                            .selectable_value(&mut app.copy_settings_subset, "None".into(), "None")
                            .clicked()
                        {
                            app.clipboard_mask = CopySettingsMask::none();
                        }
                        if ui
                            .selectable_value(
                                &mut app.copy_settings_subset,
                                "Modified Settings".into(),
                                "Modified Settings",
                            )
                            .clicked()
                        {
                            if !app.photos.is_empty() {
                                app.clipboard_mask =
                                    CopySettingsMask::modified(&app.active_photo().params);
                            }
                        }
                    });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let copy_btn = egui::Button::new(
                        RichText::new("   Copy   ")
                            .size(12.5)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(0, 122, 255));

                    if ui.add(copy_btn).clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        if !app.photos.is_empty() {
                            app.clipboard_params = Some(app.active_photo().params.clone());
                            app.status_message = format!(
                                "Copied develop settings from '{}'",
                                app.active_photo().file_name
                            );
                        }
                        app.show_copy_settings_modal = false;
                    }

                    ui.add_space(6.0);
                    if ui.button(RichText::new(" Cancel ").size(12.5)).clicked() {
                        app.show_copy_settings_modal = false;
                    }
                });
            });
        });

    if !is_open {
        app.show_copy_settings_modal = false;
    }
}

pub fn render_sync_settings_modal(app: &mut LightRawApp, ctx: &egui::Context) {
    let mut is_open = true;

    egui::Window::new("Synchronize Settings")
        .open(&mut is_open)
        .collapsible(false)
        .resizable(true)
        .default_width(860.0)
        .default_height(560.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(Color32::from_rgb(38, 38, 42))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(72, 72, 80)))
                .inner_margin(egui::Margin::symmetric(18.0, 14.0))
                .rounding(8.0),
        )
        .show(ctx, |ui| {
            render_settings_columns(ui, &mut app.sync_settings_mask);

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                if ui.button(RichText::new("Check All").size(12.0)).clicked() {
                    app.sync_settings_mask.set_all(true);
                    app.sync_settings_subset = "All".into();
                }
                if ui.button(RichText::new("Check None").size(12.0)).clicked() {
                    app.sync_settings_mask.set_all(false);
                    app.sync_settings_subset = "None".into();
                }

                ui.add_space(10.0);
                ui.label(
                    RichText::new("Subset:")
                        .size(12.0)
                        .color(Color32::from_rgb(190, 190, 195)),
                );
                egui::ComboBox::from_id_salt("sync_settings_subset_combo")
                    .selected_text(&app.sync_settings_subset)
                    .width(160.0)
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_value(
                                &mut app.sync_settings_subset,
                                "Default".into(),
                                "Default",
                            )
                            .clicked()
                        {
                            app.sync_settings_mask = CopySettingsMask::default();
                        }
                        if ui
                            .selectable_value(&mut app.sync_settings_subset, "All".into(), "All")
                            .clicked()
                        {
                            app.sync_settings_mask = CopySettingsMask::all();
                        }
                        if ui
                            .selectable_value(&mut app.sync_settings_subset, "None".into(), "None")
                            .clicked()
                        {
                            app.sync_settings_mask = CopySettingsMask::none();
                        }
                        if ui
                            .selectable_value(
                                &mut app.sync_settings_subset,
                                "Modified Settings".into(),
                                "Modified Settings",
                            )
                            .clicked()
                        {
                            if !app.photos.is_empty() {
                                app.sync_settings_mask =
                                    CopySettingsMask::modified(&app.active_photo().params);
                            }
                        }
                    });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let sync_btn = egui::Button::new(
                        RichText::new("  Synchronize  ")
                            .size(12.5)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(0, 122, 255));

                    if ui.add(sync_btn).clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        let mask = app.sync_settings_mask.clone();
                        app.sync_selected_photos_with_mask(&mask);
                        app.show_sync_settings_modal = false;
                    }

                    ui.add_space(6.0);
                    if ui.button(RichText::new(" Cancel ").size(12.5)).clicked() {
                        app.show_sync_settings_modal = false;
                    }
                });
            });
        });

    if !is_open {
        app.show_sync_settings_modal = false;
    }
}

pub fn render_create_preset_modal(app: &mut LightRawApp, ctx: &egui::Context) {
    let mut is_open = true;

    let mut existing_groups: Vec<String> = app
        .presets
        .iter()
        .map(|p| p.category.clone())
        .collect();
    if !existing_groups.iter().any(|g| g == "User Presets") {
        existing_groups.insert(0, "User Presets".into());
    }
    existing_groups.sort();
    existing_groups.dedup();

    egui::Window::new("New Develop Preset")
        .open(&mut is_open)
        .collapsible(false)
        .resizable(true)
        .default_width(860.0)
        .default_height(620.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(Color32::from_rgb(38, 38, 42))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(72, 72, 80)))
                .inner_margin(egui::Margin::symmetric(18.0, 14.0))
                .rounding(8.0),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Preset Name:").size(12.5).strong());
                ui.add_sized(
                    Vec2::new(260.0, 22.0),
                    egui::TextEdit::singleline(&mut app.new_preset_name)
                        .hint_text("e.g. Moody Warm, Studio Clean, Golden Hour"),
                );

                ui.add_space(16.0);
                ui.label(RichText::new("Group:").size(12.5).strong());
                ui.add_sized(
                    Vec2::new(150.0, 22.0),
                    egui::TextEdit::singleline(&mut app.new_preset_group)
                        .hint_text("User Presets"),
                );
                egui::ComboBox::from_id_salt("create_preset_group_combo")
                    .selected_text("▾")
                    .width(34.0)
                    .show_ui(ui, |ui| {
                        for grp in &existing_groups {
                            if ui
                                .selectable_label(&app.new_preset_group == grp, grp)
                                .clicked()
                            {
                                app.new_preset_group = grp.clone();
                            }
                        }
                    });
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            render_settings_columns(ui, &mut app.create_preset_mask);

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                if ui.button(RichText::new("Check All").size(12.0)).clicked() {
                    app.create_preset_mask.set_all(true);
                }
                if ui.button(RichText::new("Check None").size(12.0)).clicked() {
                    app.create_preset_mask.set_all(false);
                }
                if ui
                    .button(RichText::new("Modified Only").size(12.0))
                    .clicked()
                {
                    if !app.photos.is_empty() {
                        app.create_preset_mask =
                            CopySettingsMask::modified(&app.active_photo().params);
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let create_btn = egui::Button::new(
                        RichText::new("   Create   ")
                            .size(12.5)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(0, 122, 255));

                    if ui.add(create_btn).clicked() {
                        let count = app.presets.len() + 1;
                        let raw_name = app.new_preset_name.trim();
                        let name = if raw_name.is_empty() {
                            format!("User Preset {}", count)
                        } else {
                            raw_name.to_string()
                        };

                        let raw_group = app.new_preset_group.trim();
                        let group = if raw_group.is_empty() {
                            "User Presets".to_string()
                        } else {
                            raw_group.to_string()
                        };

                        let params = if !app.photos.is_empty() {
                            let mut p = DevelopParams::default();
                            apply_settings_with_mask(
                                &mut p,
                                &app.active_photo().params,
                                &app.create_preset_mask,
                            );
                            p
                        } else {
                            DevelopParams::default()
                        };

                        let id = format!(
                            "user_{}_{}",
                            group.to_lowercase().replace(' ', "_"),
                            name.to_lowercase().replace(' ', "_")
                        );

                        // Materialize .xmp file in workspace Presets folder
                        let safe_group = group.replace('/', "-");
                        let safe_name = name.replace('/', "-");
                        let out_dir = app.workspace_dir.join("Presets").join(&safe_group);
                        let _ = std::fs::create_dir_all(&out_dir);
                        let out_file = out_dir.join(format!("{}.xmp", safe_name));

                        let mut preset = DevelopPreset {
                            id,
                            category: group.clone(),
                            name: name.clone(),
                            is_user: true,
                            is_favorite: false,
                            params,
                            mask: Some(app.create_preset_mask.clone()),
                            is_visible: true,
                            xmp_file_path: Some(out_file.display().to_string()),
                        };

                        let xml = export_preset_to_xmp(&preset);
                        if std::fs::write(&out_file, xml).is_err() {
                            preset.xmp_file_path = None;
                        }

                        app.catalog_db.save_preset(&preset);
                        if !app.photos.is_empty() {
                            app.active_photo_mut().params.applied_preset_name = name.clone();
                        }
                        if let Some(pos) = app.presets.iter().position(|p| {
                            p.id == preset.id
                                || (p.category.eq_ignore_ascii_case(&group)
                                    && p.name.eq_ignore_ascii_case(&name))
                        }) {
                            app.presets[pos] = preset;
                        } else {
                            app.presets.push(preset);
                        }
                        app.status_message = format!("Created preset '{}' in '{}'.", name, group);

                        app.new_preset_name.clear();
                        app.show_create_preset_modal = false;
                    }

                    ui.add_space(6.0);
                    if ui.button(RichText::new(" Cancel ").size(12.5)).clicked() {
                        app.show_create_preset_modal = false;
                    }
                });
            });
        });

    if !is_open {
        app.show_create_preset_modal = false;
    }
}

pub fn render_rename_preset_modal(app: &mut LightRawApp, ctx: &egui::Context) {
    let Some(target_id) = app.show_rename_preset_modal.clone() else {
        return;
    };
    let mut is_open = true;
    let mut close_modal = false;

    egui::Window::new("Rename Preset")
        .open(&mut is_open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(Color32::from_rgb(34, 36, 42))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(70, 75, 85)))
                .inner_margin(egui::Margin::symmetric(20.0, 16.0))
                .rounding(8.0),
        )
        .show(ctx, |ui| {
            ui.label(RichText::new("Preset Name:").size(12.5));
            ui.add_space(6.0);
            ui.add_sized(
                Vec2::new(320.0, 24.0),
                egui::TextEdit::singleline(&mut app.rename_preset_input),
            );
            ui.add_space(14.0);

            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(
                            RichText::new(" Rename ")
                                .size(12.5)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        let new_name = app.rename_preset_input.trim().to_string();
                        if !new_name.is_empty() {
                            if let Some(grp_name) = target_id.strip_prefix("GROUP:") {
                                let mut updated = Vec::new();
                                for p in &mut app.presets {
                                    if p.category == grp_name {
                                        p.category = new_name.clone();
                                        p.is_user = true;
                                        updated.push(p.clone());
                                    }
                                }
                                for up in &updated {
                                    app.catalog_db.save_preset(up);
                                }
                                app.status_message = format!(
                                    "Renamed preset group '{}' to '{}'.",
                                    grp_name, new_name
                                );
                            } else if let Some(p) = app
                                .presets
                                .iter_mut()
                                .find(|p| p.id == target_id || p.name == target_id)
                            {
                                let old_name = p.name.clone();
                                p.name = new_name.clone();
                                p.is_user = true;
                                if let Some(ref old_xmp) = p.xmp_file_path.clone() {
                                    let old_p = std::path::Path::new(old_xmp);
                                    if let Some(parent) = old_p.parent() {
                                        let new_p = parent
                                            .join(format!("{}.xmp", new_name.replace('/', "-")));
                                        let xml = export_preset_to_xmp(p);
                                        if std::fs::write(&new_p, xml).is_ok() {
                                            if new_p != old_p && old_p.exists() {
                                                let _ = std::fs::remove_file(old_p);
                                            }
                                            p.xmp_file_path = Some(new_p.display().to_string());
                                        }
                                    }
                                }
                                let clone = p.clone();
                                app.catalog_db.save_preset(&clone);
                                if !app.photos.is_empty()
                                    && app.active_photo().params.applied_preset_name == old_name
                                {
                                    app.active_photo_mut().params.applied_preset_name =
                                        new_name.clone();
                                }
                                app.status_message = format!("Renamed preset to '{}'.", new_name);
                            }
                        }
                        close_modal = true;
                    }
                    ui.add_space(8.0);
                    if ui.button(RichText::new("Cancel").size(12.5)).clicked() {
                        close_modal = true;
                    }
                });
            });
        });

    if !is_open || close_modal {
        app.show_rename_preset_modal = None;
    }
}

pub fn render_move_preset_modal(app: &mut LightRawApp, ctx: &egui::Context) {
    let Some(target_id) = app.show_move_preset_modal.clone() else {
        return;
    };
    let mut is_open = true;
    let mut close_modal = false;

    let mut existing_groups: Vec<String> =
        app.presets.iter().map(|p| p.category.clone()).collect();
    existing_groups.sort();
    existing_groups.dedup();

    egui::Window::new("Move Preset")
        .open(&mut is_open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(Color32::from_rgb(34, 36, 42))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(70, 75, 85)))
                .inner_margin(egui::Margin::symmetric(20.0, 16.0))
                .rounding(8.0),
        )
        .show(ctx, |ui| {
            ui.label(RichText::new("Group:").size(12.5));
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.add_sized(
                    Vec2::new(265.0, 24.0),
                    egui::TextEdit::singleline(&mut app.move_preset_group_input),
                );
                egui::ComboBox::from_id_salt("move_preset_group_combo")
                    .selected_text("▾")
                    .width(36.0)
                    .show_ui(ui, |ui| {
                        for grp in &existing_groups {
                            if ui
                                .selectable_label(&app.move_preset_group_input == grp, grp)
                                .clicked()
                            {
                                app.move_preset_group_input = grp.clone();
                            }
                        }
                    });
            });
            ui.add_space(14.0);

            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(
                            RichText::new(" Move ")
                                .size(12.5)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        let new_group = app.move_preset_group_input.trim().to_string();
                        if !new_group.is_empty() {
                            if let Some(p) = app
                                .presets
                                .iter_mut()
                                .find(|p| p.id == target_id || p.name == target_id)
                            {
                                p.category = new_group.clone();
                                p.is_user = true;
                                let out_dir = app
                                    .workspace_dir
                                    .join("Presets")
                                    .join(new_group.replace('/', "-"));
                                let _ = std::fs::create_dir_all(&out_dir);
                                let new_xmp =
                                    out_dir.join(format!("{}.xmp", p.name.replace('/', "-")));
                                let xml = export_preset_to_xmp(p);
                                if std::fs::write(&new_xmp, xml).is_ok() {
                                    p.xmp_file_path = Some(new_xmp.display().to_string());
                                }
                                let clone = p.clone();
                                app.catalog_db.save_preset(&clone);
                                app.status_message = format!(
                                    "Moved preset '{}' to group '{}'.",
                                    clone.name, new_group
                                );
                            }
                        }
                        close_modal = true;
                    }
                    ui.add_space(8.0);
                    if ui.button(RichText::new("Cancel").size(12.5)).clicked() {
                        close_modal = true;
                    }
                });
            });
        });

    if !is_open || close_modal {
        app.show_move_preset_modal = None;
    }
}

pub fn render_manage_presets_modal(app: &mut LightRawApp, ctx: &egui::Context) {
    let mut is_open = true;

    let mut groups: Vec<String> = app.presets.iter().map(|p| p.category.clone()).collect();
    groups.sort();
    groups.dedup();

    egui::Window::new("Manage Presets")
        .open(&mut is_open)
        .collapsible(false)
        .resizable(true)
        .default_width(420.0)
        .default_height(460.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(Color32::from_rgb(36, 36, 40))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(72, 72, 80)))
                .inner_margin(egui::Margin::symmetric(18.0, 14.0))
                .rounding(8.0),
        )
        .show(ctx, |ui| {
            ui.label(
                RichText::new("Check preset groups to show in the Presets panel:")
                    .size(12.0)
                    .color(Color32::from_rgb(195, 195, 200)),
            );
            ui.add_space(8.0);

            egui::ScrollArea::vertical()
                .max_height(320.0)
                .show(ui, |ui| {
                    for grp in &groups {
                        let any_vis = app
                            .presets
                            .iter()
                            .filter(|p| &p.category == grp)
                            .any(|p| p.is_visible);
                        let count = app.presets.iter().filter(|p| &p.category == grp).count();
                        let mut checked = any_vis;
                        if ui
                            .checkbox(
                                &mut checked,
                                RichText::new(format!("{}  ({})", grp, count)).size(12.5),
                            )
                            .changed()
                        {
                            for p in &mut app.presets {
                                if &p.category == grp {
                                    p.is_visible = checked;
                                    app.catalog_db.save_preset(p);
                                }
                            }
                        }
                    }
                });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                if ui.button("Check All").clicked() {
                    for p in &mut app.presets {
                        p.is_visible = true;
                        app.catalog_db.save_preset(p);
                    }
                }
                if ui.button("Check None").clicked() {
                    for p in &mut app.presets {
                        p.is_visible = false;
                        app.catalog_db.save_preset(p);
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let save_btn = egui::Button::new(
                        RichText::new("   Save   ")
                            .size(12.5)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(0, 122, 255));
                    if ui.add(save_btn).clicked() {
                        app.show_manage_presets_modal = false;
                    }
                });
            });
        });

    if !is_open {
        app.show_manage_presets_modal = false;
    }
}

/// Paints a macOS parent group checkbox supporting `checked` (✓), `unchecked` ( ), and `partial` (—) states.
fn parent_group_checkbox(
    ui: &mut egui::Ui,
    label: &str,
    children: &[bool],
) -> Option<bool> {
    let all_true = !children.is_empty() && children.iter().all(|&b| b);
    let any_true = children.iter().any(|&b| b);
    let partial = any_true && !all_true;

    let mut toggled_to = None;
    ui.horizontal(|ui| {
        let (box_rect, resp) = ui.allocate_exact_size(Vec2::new(15.0, 15.0), Sense::click());
        let lbl_resp = ui.add(
            egui::Label::new(
                RichText::new(label)
                    .size(12.0)
                    .strong()
                    .color(Color32::from_rgb(235, 235, 240)),
            )
            .sense(Sense::click()),
        );
        if resp.clicked() || lbl_resp.clicked() {
            toggled_to = Some(!all_true);
        }

        if all_true || partial {
            ui.painter()
                .rect_filled(box_rect, 3.5, Color32::from_rgb(10, 132, 255));
            if partial {
                ui.painter().line_segment(
                    [
                        Pos2::new(box_rect.min.x + 3.5, box_rect.center().y),
                        Pos2::new(box_rect.max.x - 3.5, box_rect.center().y),
                    ],
                    Stroke::new(2.0_f32, Color32::WHITE),
                );
            } else {
                let p1 = Pos2::new(box_rect.min.x + 3.2, box_rect.center().y + 0.3);
                let p2 = Pos2::new(box_rect.min.x + 6.2, box_rect.max.y - 3.5);
                let p3 = Pos2::new(box_rect.max.x - 3.0, box_rect.min.y + 3.5);
                ui.painter()
                    .line_segment([p1, p2], Stroke::new(1.8_f32, Color32::WHITE));
                ui.painter()
                    .line_segment([p2, p3], Stroke::new(1.8_f32, Color32::WHITE));
            }
        } else {
            let _ = Rect::from_min_max(box_rect.min, box_rect.max);
            ui.painter()
                .rect_filled(box_rect, 3.5, Color32::from_rgb(52, 52, 58));
            ui.painter().rect_stroke(
                box_rect,
                3.5,
                Stroke::new(1.0_f32, Color32::from_rgb(100, 100, 110)),
            );
        }
    });
    toggled_to
}

fn child_checkbox(ui: &mut egui::Ui, val: &mut bool, label: &str) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.add_space(16.0);
        if ui.checkbox(val, RichText::new(label).size(11.5)).changed() {
            changed = true;
        }
    });
    changed
}

fn render_settings_columns(ui: &mut egui::Ui, mask: &mut CopySettingsMask) {
    let mut any_child_changed = false;

    ui.columns(4, |columns| {
        // COLUMN 1: Treatment & Profile, Basic, Curve
        let c1 = &mut columns[0];
        c1.checkbox(
            &mut mask.treatment_and_profile,
            RichText::new("Treatment & Profile").size(12.0).strong(),
        );

        c1.add_space(5.0);
        let basic_children = [
            mask.basic_white_balance,
            mask.basic_exposure,
            mask.basic_contrast,
            mask.basic_highlights,
            mask.basic_shadows,
            mask.basic_whites,
            mask.basic_blacks,
            mask.basic_texture,
            mask.basic_clarity,
            mask.basic_dehaze,
            mask.basic_vibrance,
            mask.basic_saturation,
        ];
        if let Some(val) = parent_group_checkbox(c1, "Basic", &basic_children) {
            mask.basic_group = val;
            mask.basic_white_balance = val;
            mask.basic_exposure = val;
            mask.basic_contrast = val;
            mask.basic_highlights = val;
            mask.basic_shadows = val;
            mask.basic_whites = val;
            mask.basic_blacks = val;
            mask.basic_texture = val;
            mask.basic_clarity = val;
            mask.basic_dehaze = val;
            mask.basic_vibrance = val;
            mask.basic_saturation = val;
        }
        any_child_changed |= child_checkbox(c1, &mut mask.basic_white_balance, "White Balance");
        any_child_changed |= child_checkbox(c1, &mut mask.basic_exposure, "Exposure");
        any_child_changed |= child_checkbox(c1, &mut mask.basic_contrast, "Contrast");
        any_child_changed |= child_checkbox(c1, &mut mask.basic_highlights, "Highlights");
        any_child_changed |= child_checkbox(c1, &mut mask.basic_shadows, "Shadows");
        any_child_changed |= child_checkbox(c1, &mut mask.basic_whites, "Whites");
        any_child_changed |= child_checkbox(c1, &mut mask.basic_blacks, "Blacks");
        any_child_changed |= child_checkbox(c1, &mut mask.basic_texture, "Texture");
        any_child_changed |= child_checkbox(c1, &mut mask.basic_clarity, "Clarity");
        any_child_changed |= child_checkbox(c1, &mut mask.basic_dehaze, "Dehaze");
        any_child_changed |= child_checkbox(c1, &mut mask.basic_vibrance, "Vibrance");
        any_child_changed |= child_checkbox(c1, &mut mask.basic_saturation, "Saturation");

        c1.add_space(6.0);
        let curve_children = [mask.curve_parametric, mask.curve_point];
        if let Some(val) = parent_group_checkbox(c1, "Curve", &curve_children) {
            mask.curve_group = val;
            mask.curve_parametric = val;
            mask.curve_point = val;
        }
        any_child_changed |= child_checkbox(c1, &mut mask.curve_parametric, "Parametric Curve");
        any_child_changed |= child_checkbox(c1, &mut mask.curve_point, "Point Curve");

        // COLUMN 2: Color Mixer, Color Grading, Detail, Lens Corrections, Transform
        let c2 = &mut columns[1];
        let mixer_children = [mask.color_mixer_mixer, mask.color_mixer_point_color];
        if let Some(val) = parent_group_checkbox(c2, "Color Mixer", &mixer_children) {
            mask.color_mixer_group = val;
            mask.color_mixer_mixer = val;
            mask.color_mixer_point_color = val;
        }
        any_child_changed |= child_checkbox(c2, &mut mask.color_mixer_mixer, "Mixer");
        any_child_changed |= child_checkbox(c2, &mut mask.color_mixer_point_color, "Point Color");

        c2.add_space(6.0);
        c2.checkbox(
            &mut mask.color_grading,
            RichText::new("Color Grading").size(12.0).strong(),
        );

        c2.add_space(6.0);
        let detail_children = [
            mask.detail_denoise_raw,
            mask.detail_sharpening,
            mask.detail_luminance_nr,
            mask.detail_color_nr,
        ];
        if let Some(val) = parent_group_checkbox(c2, "Detail", &detail_children) {
            mask.detail_group = val;
            mask.detail_denoise_raw = val;
            mask.detail_sharpening = val;
            mask.detail_luminance_nr = val;
            mask.detail_color_nr = val;
        }
        any_child_changed |= child_checkbox(
            c2,
            &mut mask.detail_denoise_raw,
            "Denoise, Raw Details, Super Resolution",
        );
        any_child_changed |= child_checkbox(c2, &mut mask.detail_sharpening, "Sharpening");
        any_child_changed |=
            child_checkbox(c2, &mut mask.detail_luminance_nr, "Luminance Noise Reduction");
        any_child_changed |=
            child_checkbox(c2, &mut mask.detail_color_nr, "Color Noise Reduction");

        c2.add_space(6.0);
        let lens_children = [
            mask.lens_remove_ca,
            mask.lens_profile,
            mask.lens_manual_distortion,
            mask.lens_manual_vignetting,
        ];
        if let Some(val) = parent_group_checkbox(c2, "Lens Corrections", &lens_children) {
            mask.lens_corrections_group = val;
            mask.lens_remove_ca = val;
            mask.lens_profile = val;
            mask.lens_manual_distortion = val;
            mask.lens_manual_vignetting = val;
        }
        any_child_changed |=
            child_checkbox(c2, &mut mask.lens_remove_ca, "Remove Chromatic Aberration");
        any_child_changed |=
            child_checkbox(c2, &mut mask.lens_profile, "Enable Profile Corrections");
        any_child_changed |=
            child_checkbox(c2, &mut mask.lens_manual_distortion, "Manual Distortion");
        any_child_changed |=
            child_checkbox(c2, &mut mask.lens_manual_vignetting, "Manual Vignetting");

        c2.add_space(6.0);
        let trans_children = [
            mask.transform_upright_mode,
            mask.transform_upright_transforms,
            mask.transform_manual,
        ];
        if let Some(val) = parent_group_checkbox(c2, "Transform", &trans_children) {
            mask.transform_group = val;
            mask.transform_upright_mode = val;
            mask.transform_upright_transforms = val;
            mask.transform_manual = val;
        }
        any_child_changed |= child_checkbox(c2, &mut mask.transform_upright_mode, "Upright Mode");
        any_child_changed |= child_checkbox(
            c2,
            &mut mask.transform_upright_transforms,
            "Upright Transforms",
        );
        any_child_changed |= child_checkbox(c2, &mut mask.transform_manual, "Manual Transform");

        // COLUMN 3: Lens Blur, Effects, Remove, Crop, Process Version, Calibration, HDR
        let c3 = &mut columns[2];
        let blur_children = [mask.lens_blur_focus_subject];
        if let Some(val) = parent_group_checkbox(c3, "Lens Blur", &blur_children) {
            mask.lens_blur_group = val;
            mask.lens_blur_focus_subject = val;
        }
        any_child_changed |=
            child_checkbox(c3, &mut mask.lens_blur_focus_subject, "Focus on Subject");

        c3.add_space(6.0);
        let effects_children = [mask.effects_post_crop_vignette, mask.effects_grain];
        if let Some(val) = parent_group_checkbox(c3, "Effects", &effects_children) {
            mask.effects_group = val;
            mask.effects_post_crop_vignette = val;
            mask.effects_grain = val;
        }
        any_child_changed |=
            child_checkbox(c3, &mut mask.effects_post_crop_vignette, "Post-Crop Vignetting");
        any_child_changed |= child_checkbox(c3, &mut mask.effects_grain, "Grain");

        c3.add_space(6.0);
        let remove_children = [
            mask.remove_manual,
            mask.remove_reflections,
            mask.remove_people,
            mask.remove_dust,
        ];
        if let Some(val) = parent_group_checkbox(c3, "Remove", &remove_children) {
            mask.remove_group = val;
            mask.remove_manual = val;
            mask.remove_reflections = val;
            mask.remove_people = val;
            mask.remove_dust = val;
        }
        any_child_changed |= child_checkbox(c3, &mut mask.remove_manual, "Manual");
        any_child_changed |= child_checkbox(c3, &mut mask.remove_reflections, "Reflections");
        any_child_changed |= child_checkbox(c3, &mut mask.remove_people, "People");
        any_child_changed |= child_checkbox(c3, &mut mask.remove_dust, "Dust");

        c3.add_space(6.0);
        let crop_children = [mask.crop_straighten_angle, mask.crop_aspect_ratio];
        if let Some(val) = parent_group_checkbox(c3, "Crop", &crop_children) {
            mask.crop_group = val;
            mask.crop_straighten_angle = val;
            mask.crop_aspect_ratio = val;
        }
        any_child_changed |=
            child_checkbox(c3, &mut mask.crop_straighten_angle, "Straighten Angle");
        any_child_changed |= child_checkbox(c3, &mut mask.crop_aspect_ratio, "Aspect Ratio");

        c3.add_space(6.0);
        c3.checkbox(
            &mut mask.process_version,
            RichText::new("Process Version").size(12.0).strong(),
        );
        c3.checkbox(
            &mut mask.calibration,
            RichText::new("Calibration").size(12.0).strong(),
        );

        c3.add_space(6.0);
        let hdr_children = [mask.hdr_mode, mask.hdr_sdr_settings];
        if let Some(val) = parent_group_checkbox(c3, "High Dynamic Range", &hdr_children) {
            mask.hdr_group = val;
            mask.hdr_mode = val;
            mask.hdr_sdr_settings = val;
        }
        any_child_changed |= child_checkbox(c3, &mut mask.hdr_mode, "HDR Mode");
        any_child_changed |= child_checkbox(c3, &mut mask.hdr_sdr_settings, "SDR Settings");

        // COLUMN 4: Masking
        let c4 = &mut columns[3];
        c4.checkbox(
            &mut mask.masking,
            RichText::new("Masking").size(12.0).strong(),
        );
    });

    if any_child_changed {
        mask.sync_groups_from_children();
    }
}
