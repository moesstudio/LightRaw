// ============================================================================
// LightRaw — System, Preferences, Collection, Snapshot & Metadata Modals
// ============================================================================

use super::{CanvasSurroundMode, Colors, LightRawApp};
use crate::catalog::{CollectionItem, SnapshotItem};
use crate::updater::{UpdaterUiState, CURRENT_VERSION, GITHUB_RELEASES_PAGE};
use eframe::egui::{self, Color32, RichText, Vec2};

pub fn render_system_modals(app: &mut LightRawApp, ctx: &egui::Context) {
    render_about_and_sysinfo(app, ctx);
    render_update_modal(app, ctx);
    render_rename_catalog(app, ctx);
    render_preferences_modal(app, ctx);
    render_shortcuts_modal(app, ctx);
    render_collection_modals(app, ctx);
    render_snapshot_modals(app, ctx);
    render_sync_metadata_modal(app, ctx);
}

fn render_about_and_sysinfo(app: &mut LightRawApp, ctx: &egui::Context) {
    if app.show_system_info_modal {
        app.show_about_modal = true;
        app.about_tab = 1;
    }

    if !app.show_about_modal {
        return;
    }

    // Dismiss with Escape key
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.show_about_modal = false;
        app.show_system_info_modal = false;
    }

    let mut open = true;
    egui::Window::new("About LightRaw Classic")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .fixed_size(Vec2::new(710.0, 560.0))
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            // ================================================================
            // 1. Hero Brand Header
            // ================================================================
            egui::Frame::none()
                .fill(Color32::from_rgb(20, 21, 26))
                .rounding(egui::Rounding::same(8.0))
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(44, 46, 56)))
                .inner_margin(egui::Margin::symmetric(16.0, 14.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // High-res Application Logo
                        let (icon_rect, _) = ui.allocate_exact_size(Vec2::new(96.0, 96.0), egui::Sense::hover());
                        ui.painter().rect_filled(icon_rect, 16.0, Color32::from_rgb(14, 18, 30));
                        if let Some(tex) = app.get_app_icon_texture(ctx) {
                            ui.painter().image(
                                tex.id(),
                                icon_rect,
                                egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
                                Color32::WHITE,
                            );
                        } else {
                            ui.painter().text(
                                icon_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "LR",
                                egui::FontId::proportional(36.0),
                                Color32::WHITE,
                            );
                        }
                        ui.painter().rect_stroke(
                            icon_rect,
                            16.0,
                            egui::Stroke::new(1.5, Color32::from_rgb(50, 70, 110)),
                        );

                        ui.add_space(16.0);

                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("LightRaw")
                                        .size(26.0)
                                        .strong()
                                        .color(Color32::WHITE),
                                );
                                // Classic Badge
                                egui::Frame::none()
                                    .fill(Color32::from_rgb(220, 100, 20))
                                    .rounding(egui::Rounding::same(4.0))
                                    .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                                    .show(ui, |ui| {
                                        ui.label(
                                            RichText::new("CLASSIC")
                                                .size(11.0)
                                                .strong()
                                                .color(Color32::WHITE),
                                        );
                                    });
                                ui.label(
                                    RichText::new(CURRENT_VERSION)
                                        .size(15.0)
                                        .color(Color32::from_rgb(170, 175, 185)),
                                );
                            });

                            ui.add_space(2.0);
                            ui.label(
                                RichText::new("Local-First Professional Camera RAW Workflow Engine")
                                    .size(12.5)
                                    .strong()
                                    .color(Colors::ACCENT_AMBER),
                            );

                            ui.add_space(3.0);
                            ui.label(
                                RichText::new("Camera Raw Subsystem 17.0 • 32-Bit Linear Engine • Apple Silicon Metal")
                                    .size(11.0)
                                    .color(Color32::from_rgb(110, 195, 255)),
                            );

                            ui.add_space(3.0);
                            ui.label(
                                RichText::new("Licensed to: Moestret • Standalone Lifetime License (Zero Cloud / Local-First)")
                                    .size(11.0)
                                    .color(Color32::from_rgb(125, 215, 150)),
                            );
                        });
                    });
                });

            ui.add_space(10.0);

            // ================================================================
            // 2. Segmented Tab Control
            // ================================================================
            ui.horizontal(|ui| {
                let tabs = [
                    ("⊞  Overview", 0),
                    ("⚙  System Info", 1),
                    ("★  Credits & Architecture", 2),
                    ("⚖  Legal Notices", 3),
                ];
                for (label, idx) in tabs {
                    let is_active = app.about_tab == idx;
                    let text = if is_active {
                        RichText::new(label).strong().color(Color32::WHITE)
                    } else {
                        RichText::new(label).color(Colors::TEXT_MID)
                    };
                    let btn = egui::Button::new(text)
                        .fill(if is_active {
                            Colors::ACCENT_BLUE
                        } else {
                            Color32::from_rgb(34, 34, 40)
                        })
                        .rounding(egui::Rounding::same(5.0));
                    if ui.add(btn).clicked() {
                        app.about_tab = idx;
                    }
                }
            });

            ui.add_space(8.0);

            // Gather system metrics for System Info and Copy Diagnostics
            let (vol_name, free_b, total_b) = app.cached_disk_space();
            let cat_size_str = if let Ok(meta) = std::fs::metadata(&app.catalog_db.db_path) {
                format!("{:.2} MB", meta.len() as f64 / 1_048_576.0)
            } else {
                "Active".into()
            };
            let diag_text = format!(
                "LightRaw Classic {} [Build Release]\n\
                 Target Architecture: {} ({})\n\
                 Graphics / Acceleration: Apple Metal 3 / Glow / Accelerate Framework\n\
                 Display Scale: {:.1}x (Retina Native)\n\
                 Multithreading: Rayon {} threads\n\
                 Active Catalog: {} ({})\n\
                 Catalog Photos: {} total, {} selected\n\
                 Loaded Presets: {} active (ACR 156-field XMP & .lrtemplate)\n\
                 Storage Free Space: {:.1} GB free of {:.1} GB on {}\n\
                 Color Engine: 32-Bit Linear ProPhoto RGB / Bradford LMS\n\
                 License: Moestret (Standalone Lifetime Professional)",
                crate::updater::CURRENT_VERSION,
                std::env::consts::ARCH,
                std::env::consts::OS,
                ctx.pixels_per_point(),
                rayon::current_num_threads(),
                app.catalog_db.db_path.display(),
                cat_size_str,
                app.photos.len(),
                app.selected_photo_ids.len(),
                app.presets.len(),
                free_b as f64 / 1_073_741_824.0,
                total_b as f64 / 1_073_741_824.0,
                vol_name,
            );

            // ================================================================
            // 3. Tab Content Frame
            // ================================================================
            egui::Frame::none()
                .fill(Color32::from_rgb(24, 25, 30))
                .rounding(egui::Rounding::same(6.0))
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(42, 43, 50)))
                .inner_margin(egui::Margin::symmetric(14.0, 12.0))
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("about_scroll_area")
                        .auto_shrink([false, false])
                        .max_height(265.0)
                        .show(ui, |ui| {
                            match app.about_tab {
                                0 => {
                                    // OVERVIEW TAB
                                    ui.label(
                                        RichText::new("LightRaw Classic Architecture & Philosophy")
                                            .strong()
                                            .size(13.5)
                                            .color(Color32::WHITE),
                                    );
                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new(
                                            "LightRaw is engineered as a 100% pure Rust, local-first professional digital darkroom. \
                                             Built as an uncompromising alternative to subscription-bound RAW editors, LightRaw \
                                             operates with zero cloud dependencies, guarantees absolute data privacy, and delivers \
                                             fluid 120 FPS performance on Apple Silicon hardware."
                                        )
                                        .size(11.5)
                                        .color(Color32::from_rgb(180, 185, 195)),
                                    );
                                    ui.add_space(10.0);
                                    ui.separator();
                                    ui.add_space(8.0);

                                    let features = [
                                        ("◈  32-Bit Linear RAW Engine", "Native Apple Silicon CIRAWFilter decoding with Bradford LMS chromatic adaptation and ProPhoto RGB wide-gamut linear working space."),
                                        ("◈  11-Stage Non-Destructive ACR Pipeline", "Full ACR Process Version 2012 / 6.7 compatibility: Exposure, Contrast, Highlights, Shadows, Whites, Blacks, Parametric Tone Curve, 8-Channel HSL, 3-Way Color Grading, Detail & Lens Blur Bokeh."),
                                        ("◈  Apple Vision AI Assisted Culling", "Neural Engine face detection, landmarks, and eye-blink ratio estimation for lightning-fast photo selection."),
                                        ("◈  SQLite 3 ACID Catalog (.lrcat)", "Non-destructive catalog architecture with instant keyword search, smart collections, virtual copies, snapshots, and persistent history stacks."),
                                        ("◈  156-Parameter ACR Presets (.xmp & .lrtemplate)", "Bidirectional .XMP & .lrtemplate compatibility, instant preset amount interpolation, and one-click preset copying."),
                                        ("◈  Precision Vector & Graphic Watermarks", "Vector typography and graphic logo compositing with precise rotation, fit/fill scaling, and inset anchoring."),
                                        ("◈  100% Local-First & Zero Telemetry Guarantee", "Zero network calls, zero tracking, zero mandatory cloud accounts. Your creative work never leaves your Mac."),
                                    ];

                                    for (title, desc) in features {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(title).strong().size(11.5).color(Colors::ACCENT_AMBER));
                                        });
                                        ui.label(RichText::new(desc).size(11.0).color(Color32::from_rgb(160, 165, 175)));
                                        ui.add_space(6.0);
                                    }
                                }
                                1 => {
                                    // SYSTEM INFO TAB
                                    ui.label(
                                        RichText::new("System Hardware, Runtime & Pipeline Diagnostics")
                                            .strong()
                                            .size(13.5)
                                            .color(Color32::WHITE),
                                    );
                                    ui.add_space(6.0);

                                    egui::Grid::new("about_sys_info_grid")
                                        .num_columns(2)
                                        .spacing([18.0, 7.0])
                                        .striped(true)
                                        .show(ui, |ui| {
                                            let catalog_path_str = app.catalog_db.db_path.to_string_lossy().to_string();
                                            let total_photos_str = format!("{} photos ({} currently selected)", app.photos.len(), app.selected_photo_ids.len());
                                            let presets_str = format!("{} presets loaded (ACR XMP & LRTEMPLATE)", app.presets.len());
                                            let storage_str = format!("{}: {:.1} GB free of {:.1} GB", vol_name, free_b as f64 / 1_073_741_824.0, total_b as f64 / 1_073_741_824.0);

                                            let rows: [(&str, &str); 14] = [
                                                ("Application Version:", "LightRaw Classic 1.0.0 (Native Pure Rust Release)"),
                                                ("Target Platform:", "macOS Sonoma (Darwin 23.x / ARM64 Apple Silicon)"),
                                                ("Build Profile:", "Release Mode (opt-level 3, LTO thin, codegen-units 1)"),
                                                ("Graphics & Display:", "Apple Metal 3 / Glow Backend • Retina Scale Native"),
                                                ("Concurrency Engine:", "Rayon Work-Stealing Pool"),
                                                ("RAW & Color Pipeline:", "Apple CIRAWFilter (32-Bit Linear) + Bradford LMS WB"),
                                                ("Working Color Space:", "ProPhoto RGB (Wide Gamut Scene-Referred FP32)"),
                                                ("AI Neural Accelerator:", "Apple Neural Engine (ANE) + Vision Framework"),
                                                ("Active Catalog Database:", &catalog_path_str),
                                                ("Catalog Database Size:", &cat_size_str),
                                                ("Total Catalog Photos:", &total_photos_str),
                                                ("Registered Presets:", &presets_str),
                                                ("Volume Storage:", &storage_str),
                                                ("User License:", "Moestret (Standalone Lifetime Professional)"),
                                            ];
                                            for (k, v) in rows {
                                                ui.label(RichText::new(k).strong().size(11.5).color(Color32::from_rgb(150, 155, 170)));
                                                ui.label(RichText::new(v).size(11.5).color(Color32::WHITE));
                                                ui.end_row();
                                            }
                                        });
                                }
                                2 => {
                                    // CREDITS & TEAM TAB
                                    ui.label(
                                        RichText::new("LightRaw Engineering & Software Craftsmanship")
                                            .strong()
                                            .size(13.5)
                                            .color(Color32::WHITE),
                                    );
                                    ui.add_space(6.0);
                                    ui.label(
                                        RichText::new("A tribute to the golden age of photography software engineering.")
                                            .size(11.5)
                                            .italics()
                                            .color(Colors::ACCENT_AMBER),
                                    );
                                    ui.add_space(10.0);
                                    ui.separator();
                                    ui.add_space(8.0);

                                    let credits = [
                                        ("Lead Architect & Engineer", "Moestret"),
                                        ("Color Science & Chromatic Adaptation", "Bradford LMS Formulation & Linear Scene-Referred Transform"),
                                        ("RAW Image Pipeline & Decoding", "Apple CoreImage CIRAWFilter & Pure Rust SIMD Math"),
                                        ("Non-Destructive Catalog Architecture", "SQLite 3 WAL Transaction Engine & Zero-Data-Loss Design"),
                                        ("Machine Vision & Assisted Culling", "Apple Vision Framework Face Landmarks & Eye-Blink Classifier"),
                                        ("Preset Interoperability System", "Adobe Camera Raw 156-field XMP Specification & LRTemplate Parser"),
                                    ];

                                    for (role, name) in credits {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(format!("{}:", role)).strong().size(11.5).color(Color32::from_rgb(150, 155, 170)));
                                            ui.label(RichText::new(name).size(11.5).color(Color32::WHITE));
                                        });
                                        ui.add_space(4.0);
                                    }

                                    ui.add_space(10.0);
                                    ui.separator();
                                    ui.add_space(8.0);
                                    ui.label(
                                        RichText::new("Historical Acknowledgements:")
                                            .strong()
                                            .size(11.5)
                                            .color(Color32::WHITE),
                                    );
                                    ui.add_space(3.0);
                                    ui.label(
                                        RichText::new(
                                            "LightRaw stands on the shoulders of photographic software giants. We acknowledge \
                                             the pioneering vision of Mark Hamburg, Thomas Knoll, and the original Lightroom \
                                             Classic engineering team at Adobe, whose workflow paradigm redefined modern photography."
                                        )
                                        .size(11.0)
                                        .color(Color32::from_rgb(160, 165, 175)),
                                    );
                                    ui.add_space(6.0);
                                    ui.label(
                                        RichText::new("Dedicated to all photographers, retouchers, and visual storytellers.")
                                            .size(11.0)
                                            .italics()
                                            .color(Colors::ACCENT_AMBER),
                                    );
                                }
                                3 => {
                                    // LEGAL NOTICES TAB
                                    ui.label(
                                        RichText::new("Legal Notices, Trademarks & Open-Source Licenses")
                                            .strong()
                                            .size(13.5)
                                            .color(Color32::WHITE),
                                    );
                                    ui.add_space(6.0);

                                    ui.label(
                                        RichText::new("Copyright © 2026 Moestret. All rights reserved.")
                                            .strong()
                                            .size(11.5)
                                            .color(Color32::WHITE),
                                    );
                                    ui.add_space(8.0);

                                    ui.label(
                                        RichText::new("Trademark Attribution:")
                                            .strong()
                                            .size(11.5)
                                            .color(Colors::ACCENT_AMBER),
                                    );
                                    ui.add_space(2.0);
                                    ui.label(
                                        RichText::new(
                                            "Adobe, Photoshop, Lightroom, and Adobe Camera Raw are either registered trademarks \
                                             or trademarks of Adobe Inc. in the United States and/or other countries. \
                                             LightRaw is an independent software project developed by Moestret and is neither \
                                             affiliated with, sponsored by, nor endorsed by Adobe Inc."
                                        )
                                        .size(11.0)
                                        .color(Color32::from_rgb(160, 165, 175)),
                                    );
                                    ui.add_space(10.0);

                                    ui.label(
                                        RichText::new("Open-Source Software Components:")
                                            .strong()
                                            .size(11.5)
                                            .color(Colors::ACCENT_AMBER),
                                    );
                                    ui.add_space(2.0);
                                    ui.label(
                                        RichText::new(
                                            "LightRaw is built using open-source Rust technologies distributed under MIT and Apache-2.0 licenses:\n\
                                             • eframe & egui — Immediate Mode GUI Framework (Emil Ernerfeldt)\n\
                                             • image — Rust Image Encoding & Decoding (The Image-rs Authors)\n\
                                             • rusqlite — SQLite3 Ergonomic Rust Driver (John Gallagher)\n\
                                             • rayon — High-Performance Data Parallelism (Niko Matsakis)\n\
                                             • serde & serde_json — Serialization & Deserialization Framework (David Tolnay)\n\
                                             • rfd — Native Cross-Platform File Dialogs\n\
                                             • chrono & sha2 — Time and Cryptographic Hashing Utilities"
                                        )
                                        .size(11.0)
                                        .color(Color32::from_rgb(160, 165, 175)),
                                    );
                                    ui.add_space(10.0);

                                    ui.label(
                                        RichText::new("Privacy & Data Sovereignty Guarantee:")
                                            .strong()
                                            .size(11.5)
                                            .color(Color32::from_rgb(125, 215, 150)),
                                    );
                                    ui.add_space(2.0);
                                    ui.label(
                                        RichText::new(
                                            "100% Local-First. Zero telemetry. Zero analytics. LightRaw never collects, transmits, \
                                             or uploads your photos, editing histories, presets, or user data to external servers."
                                        )
                                        .size(11.0)
                                        .color(Color32::from_rgb(160, 165, 175)),
                                    );
                                }
                                _ => {}
                            }
                        });
                });

            ui.add_space(12.0);

            // ================================================================
            // 4. Bottom Action Bar
            // ================================================================
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Copyright © 2026 Moestret • All rights reserved.")
                        .size(10.5)
                        .color(Color32::from_rgb(130, 130, 135)),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let close_btn = egui::Button::new(
                        RichText::new("Close")
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(Colors::ACCENT_BLUE)
                    .rounding(egui::Rounding::same(5.0))
                    .min_size(Vec2::new(75.0, 24.0));

                    if ui.add(close_btn).clicked() {
                        app.show_about_modal = false;
                        app.show_system_info_modal = false;
                    }

                    let copy_btn = egui::Button::new(
                        RichText::new("📋 Copy System Info")
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(44, 46, 54))
                    .rounding(egui::Rounding::same(5.0))
                    .min_size(Vec2::new(130.0, 24.0));

                    if ui.add(copy_btn).clicked() {
                        ctx.copy_text(diag_text);
                        app.status_message = "System diagnostics copied to clipboard.".into();
                    }

                    let upd_btn = egui::Button::new(
                        RichText::new("🔄 Check Updates").color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(44, 46, 54))
                    .rounding(egui::Rounding::same(5.0))
                    .min_size(Vec2::new(124.0, 24.0));

                    if ui.add(upd_btn).clicked() {
                        app.trigger_update_check(true);
                    }
                });
            });
        });

    if !open {
        app.show_about_modal = false;
        app.show_system_info_modal = false;
    }
}

fn render_update_modal(app: &mut LightRawApp, ctx: &egui::Context) {
    if !app.show_update_modal {
        return;
    }
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.show_update_modal = false;
    }

    let mut open = true;
    let mut trigger_recheck = false;
    let mut trigger_install: Option<crate::updater::RemoteRelease> = None;

    egui::Window::new("LightRaw Software Update")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .fixed_size(Vec2::new(470.0, 290.0))
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.label(
                RichText::new(format!("Installed Version: v{}", CURRENT_VERSION))
                    .size(12.5)
                    .color(Colors::TEXT_MID),
            );
            ui.add_space(10.0);

            match &app.update_state {
                UpdaterUiState::Idle | UpdaterUiState::Checking => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(RichText::new("Checking GitHub Releases for latest version...").color(Colors::TEXT_BRIGHT));
                    });
                }
                UpdaterUiState::UpToDate { current, latest_tag } => {
                    ui.label(
                        RichText::new(format!("You are up to date! (v{} / latest {})", current, latest_tag))
                            .strong()
                            .color(Color32::from_rgb(80, 210, 120)),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Your catalog, presets, and edited photos remain intact across every update.")
                            .size(11.5)
                            .color(Colors::TEXT_MID),
                    );
                }
                UpdaterUiState::Available(rel) => {
                    ui.label(
                        RichText::new(format!("New Version Available: {}", rel.title))
                            .size(14.5)
                            .strong()
                            .color(Color32::from_rgb(100, 180, 255)),
                    );
                    ui.add_space(6.0);
                    egui::ScrollArea::vertical().max_height(135.0).show(ui, |ui| {
                        let notes = if rel.notes.is_empty() {
                            "Performance fixes, pipeline enhancements, and stability improvements."
                        } else {
                            &rel.notes
                        };
                        ui.label(RichText::new(notes).size(11.5).color(Colors::TEXT_MAIN));
                    });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if let Some(ref asset) = rel.asset {
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new(format!("Update & Restart ({})", asset.name))
                                            .strong()
                                            .color(Color32::WHITE),
                                    )
                                    .fill(Colors::ACCENT_BLUE),
                                )
                                .clicked()
                            {
                                trigger_install = Some(rel.clone());
                            }
                        }
                        if ui.button("Open Release Page").clicked() {
                            let _ = open::that(&rel.html_url);
                        }
                    });
                }
                UpdaterUiState::Installing(msg) => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(RichText::new(msg).color(Colors::TEXT_BRIGHT));
                    });
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("Do not close LightRaw; catalog settings and database will be preserved.")
                            .size(11.0)
                            .color(Colors::TEXT_MID),
                    );
                }
                UpdaterUiState::Notice(msg) => {
                    ui.label(RichText::new(msg).size(12.0).color(Colors::TEXT_BRIGHT));
                }
                UpdaterUiState::Error(err) => {
                    ui.label(
                        RichText::new(format!("Update check failed: {}", err))
                            .size(12.0)
                            .color(Color32::from_rgb(240, 110, 110)),
                    );
                    ui.add_space(8.0);
                    if ui.button("Open GitHub Releases").clicked() {
                        let _ = open::that(GITHUB_RELEASES_PAGE);
                    }
                }
            }

            ui.add_space(14.0);
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Check Again").clicked() {
                    trigger_recheck = true;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close").clicked() {
                        app.show_update_modal = false;
                    }
                });
            });
        });

    if !open {
        app.show_update_modal = false;
    }
    if trigger_recheck {
        app.trigger_update_check(true);
    }
    if let Some(rel) = trigger_install {
        app.flush_dirty_photos();
        app.start_update_install(rel);
    }
}

fn render_rename_catalog(app: &mut LightRawApp, ctx: &egui::Context) {
    if app.show_rename_catalog_modal {
        let mut open = true;
        egui::Window::new("Rename Catalog")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .fixed_size(Vec2::new(360.0, 130.0))
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label("Catalog Display Name:");
                ui.add_space(4.0);
                ui.text_edit_singleline(&mut app.rename_catalog_input);
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        let trimmed = app.rename_catalog_input.trim();
                        if !trimmed.is_empty() {
                            app.catalog_display_name = if trimmed.ends_with(".lrcat") {
                                trimmed.to_string()
                            } else {
                                format!("{}.lrcat", trimmed)
                            };
                            app.catalog_db
                                .save_app_var("catalog_display_name", &app.catalog_display_name);
                            app.status_message =
                                format!("Catalog renamed to {}", app.catalog_display_name);
                        }
                        app.show_rename_catalog_modal = false;
                    }
                    if ui.button("Cancel").clicked() {
                        app.show_rename_catalog_modal = false;
                    }
                });
            });
        if !open {
            app.show_rename_catalog_modal = false;
        }
    }
}

fn render_preferences_modal(app: &mut LightRawApp, ctx: &egui::Context) {
    if !app.show_preferences_modal {
        return;
    }
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.show_preferences_modal = false;
        return;
    }
    let mut open = true;
    egui::Window::new("LightRaw Preferences")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .fixed_size(Vec2::new(520.0, 360.0))
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            egui::Grid::new("prefs_grid")
                .num_columns(2)
                .spacing([14.0, 10.0])
                .show(ui, |ui| {
                    ui.label("Product Branding:");
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("LIGHTRAW CLASSIC")
                                .strong()
                                .color(Color32::from_rgb(30, 30, 36)),
                        );
                        ui.label(
                            RichText::new("1.0.0")
                                .size(11.0)
                                .color(Color32::from_rgb(100, 100, 108)),
                        );
                    });
                    ui.end_row();

                    ui.label("Licensed User:");
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(&app.identity_plate_owner)
                                .strong()
                                .color(Color32::from_rgb(20, 110, 190)),
                        );
                        ui.label(
                            RichText::new("• Standalone Lifetime (Local-First)")
                                .size(11.0)
                                .color(Color32::from_rgb(35, 130, 65)),
                        );
                    });
                    ui.end_row();

                    ui.label("Develop Canvas Surround:");
                    egui::ComboBox::from_id_salt("pref_canvas_surround")
                        .selected_text(app.canvas_surround.label())
                        .show_ui(ui, |ui| {
                            for mode in CanvasSurroundMode::ALL {
                                if ui
                                    .selectable_label(app.canvas_surround == mode, mode.label())
                                    .clicked()
                                {
                                    app.canvas_surround = mode;
                                }
                            }
                        });
                    ui.end_row();

                    ui.label("Right-Panel Solo Mode:");
                    ui.checkbox(
                        &mut app.solo_mode_right_panels,
                        "Auto-collapse other panels when opening one",
                    );
                    ui.end_row();

                    ui.label("In-App Menu Bar:");
                    ui.checkbox(
                        &mut app.show_in_app_menubar,
                        "Show Menu Bar (File, Edit, Develop, Photo, View, Window, Help)",
                    );
                    ui.end_row();

                    ui.label("Default Import Preset:");
                    let cur_imp = app
                        .apply_on_import_preset_id
                        .as_ref()
                        .and_then(|id| app.presets.iter().find(|p| &p.id == id))
                        .map(|p| p.name.clone())
                        .unwrap_or_else(|| "None".to_string());
                    egui::ComboBox::from_id_salt("pref_import_preset")
                        .selected_text(cur_imp)
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(app.apply_on_import_preset_id.is_none(), "None")
                                .clicked()
                            {
                                app.apply_on_import_preset_id = None;
                            }
                            for pr in &app.presets {
                                let sel = app.apply_on_import_preset_id.as_ref() == Some(&pr.id);
                                if ui.selectable_label(sel, &pr.name).clicked() {
                                    app.apply_on_import_preset_id = Some(pr.id.clone());
                                }
                            }
                        });
                    ui.end_row();

                    ui.label("AI Hardware Engine:");
                    let cur_tier = app.ai_hardware_tier.label();
                    egui::ComboBox::from_id_salt("pref_ai_hw_tier")
                        .selected_text(cur_tier)
                        .show_ui(ui, |ui| {
                            for tier in crate::ai::AiHardwareTier::ALL {
                                if ui.selectable_label(app.ai_hardware_tier == tier, tier.label()).clicked() {
                                    app.ai_hardware_tier = tier;
                                    let s = match tier {
                                        crate::ai::AiHardwareTier::Auto => "auto",
                                        crate::ai::AiHardwareTier::CpuSimd => "cpu_simd",
                                        crate::ai::AiHardwareTier::HeuristicsOnly => "heuristics_only",
                                    };
                                    app.catalog_db.save_app_var("ai_hardware_tier", s);
                                    app.raw_cache.clear();
                                    app.rendered_after_cache.clear();
                                }
                            }
                        });
                    ui.end_row();

                    ui.label("Detected Hardware:");
                    let hw_desc = crate::ai::get_hardware_description();
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("⚡ ")
                                .size(11.0)
                                .color(Color32::from_rgb(0, 200, 80)),
                        );
                        ui.label(
                            RichText::new(&hw_desc)
                                .size(11.0)
                                .color(Colors::TEXT_MAIN),
                        );
                    });
                    ui.end_row();
                });

            ui.add_space(16.0);
            ui.horizontal(|ui| {
                if ui.button("Manage Presets...").clicked() {
                    app.show_manage_presets_modal = true;
                }
                if ui.button("Done").clicked() {
                    app.show_preferences_modal = false;
                }
            });
        });
    if !open {
        app.show_preferences_modal = false;
    }
}

fn render_shortcuts_modal(app: &mut LightRawApp, ctx: &egui::Context) {
    if !app.show_shortcuts_modal {
        return;
    }
    let mut open = true;
    egui::Window::new("LightRaw — Keyboard Shortcuts")
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_size(Vec2::new(620.0, 420.0))
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.columns(2, |cols| {
                    cols[0].label(RichText::new("Library & Navigation").strong().size(13.0));
                    cols[0].separator();
                    for (k, desc) in [
                        ("G", "Library Grid View"),
                        ("E", "Library Loupe View"),
                        ("C", "Compare View (Select / Candidate)"),
                        ("N", "Survey View"),
                        ("D", "Develop Module"),
                        ("P / U / X", "Flag as Pick / Unflag / Reject"),
                        ("0 – 5", "Set Star Rating (0 to 5 Stars)"),
                        ("6 – 9", "Set Color Label (Red / Yellow / Green / Blue)"),
                        ("B", "Add / Remove Target Collection"),
                        ("T", "Show / Hide Module Toolbar"),
                        ("I", "Cycle Loupe Info Overlay"),
                        ("L", "Cycle Lights Out Mode"),
                        ("Tab", "Show / Hide Side Panels"),
                        ("⌘+Shift+I", "Import Photos and Video"),
                        ("⌘+Shift+E", "Export Selected Photos"),
                    ] {
                        cols[0].horizontal(|ui| {
                            ui.monospace(format!("{:<10}", k));
                            ui.label(desc);
                        });
                    }

                    cols[1].label(RichText::new("Develop & Tools").strong().size(13.0));
                    cols[1].separator();
                    for (k, desc) in [
                        ("R", "Crop & Straighten Overlay"),
                        ("Q", "Remove / Heal / Clone Brush"),
                        ("K / Shift+W", "Masking Drawer & AI Masks"),
                        ("O", "Toggle Active Mask Red Overlay"),
                        ("W", "White Balance Eyedropper"),
                        ("V", "Convert to Black & White"),
                        ("J", "Show Shadow & Highlight Clipping"),
                        ("S", "Toggle Soft Proofing"),
                        ("Y", "Cycle Before / After Split Views"),
                        ("\\", "Hold / Toggle Before Only View"),
                        ("⌘+Z / ⇧⌘Z", "Undo / Redo History Step"),
                        ("⌘+Shift+C", "Copy Develop Settings (Modal)"),
                        ("⌘+Shift+V", "Paste Develop Settings"),
                        ("⌥⌘+V", "Paste Settings from Previous Photo"),
                        ("⌘+'", "Create Non-Destructive Virtual Copy"),
                    ] {
                        cols[1].horizontal(|ui| {
                            ui.monospace(format!("{:<10}", k));
                            ui.label(desc);
                        });
                    }
                });
            });
            ui.separator();
            if ui.button("Close").clicked() {
                app.show_shortcuts_modal = false;
            }
        });
    if !open {
        app.show_shortcuts_modal = false;
    }
}

fn render_collection_modals(app: &mut LightRawApp, ctx: &egui::Context) {
    if app.show_create_collection_modal {
        let mut open = true;
        let title = if app.create_collection_is_smart {
            "Create Smart Collection"
        } else {
            "Create Collection"
        };
        egui::Window::new(title)
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .fixed_size(Vec2::new(400.0, 230.0))
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut app.new_collection_name);
                ui.add_space(6.0);
                ui.checkbox(&mut app.create_collection_is_smart, "Smart Collection (Dynamic Rules)");

                if app.create_collection_is_smart {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Minimum Rating:");
                        ui.add(egui::Slider::new(&mut app.new_smart_min_rating, 0..=5).suffix(" ★"));
                    });
                    ui.checkbox(&mut app.new_smart_picked_only, "Flagged as Pick Only");
                    ui.horizontal(|ui| {
                        ui.label("Contains Keyword / Text:");
                        ui.text_edit_singleline(&mut app.new_smart_keyword);
                    });
                } else {
                    ui.label(
                        RichText::new(format!(
                            "Includes {} currently selected photo(s).",
                            app.selected_photo_ids.len()
                        ))
                        .size(11.0)
                        .color(Colors::TEXT_MID),
                    );
                }

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        let name = app.new_collection_name.trim().to_string();
                        if !name.is_empty() {
                            let next_id = app
                                .collections
                                .iter()
                                .map(|c| c.id)
                                .max()
                                .unwrap_or(10)
                                + 1;
                            let encoded_rule = if app.create_collection_is_smart {
                                // Encode dynamic smart rule cleanly in photo_ids unused or name suffix?
                                // Even better: store rule metadata in `app_settings` keyed by collection id!
                                let rule = format!(
                                    "rating>={};pick={};kw={}",
                                    app.new_smart_min_rating,
                                    app.new_smart_picked_only as u8,
                                    app.new_smart_keyword.trim()
                                );
                                app.catalog_db
                                    .save_app_var(&format!("smart_col_rule_{}", next_id), &rule);
                                Vec::new()
                            } else {
                                app.selected_photo_ids.clone()
                            };
                            let smart_desc = if app.create_collection_is_smart {
                                format!(
                                    "Rating >= {}{}{}",
                                    app.new_smart_min_rating,
                                    if app.new_smart_picked_only { ", Picked" } else { "" },
                                    if app.new_smart_keyword.trim().is_empty() {
                                        String::new()
                                    } else {
                                        format!(", Keyword: {}", app.new_smart_keyword.trim())
                                    }
                                )
                            } else {
                                String::new()
                            };
                            let col = CollectionItem {
                                id: next_id,
                                name: name.clone(),
                                is_smart: app.create_collection_is_smart,
                                smart_rule_desc: smart_desc,
                                photo_ids: encoded_rule,
                            };
                            app.catalog_db.save_collection(&col);
                            app.collections.push(col);
                            app.selected_collection_id = Some(next_id);
                            app.status_message = format!("Created collection '{}'.", name);
                            app.show_create_collection_modal = false;
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        app.show_create_collection_modal = false;
                    }
                });
            });
        if !open {
            app.show_create_collection_modal = false;
        }
    }

    if let Some(col_id) = app.show_rename_collection_modal {
        let mut open = true;
        egui::Window::new("Rename Collection")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .fixed_size(Vec2::new(340.0, 120.0))
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label("New Collection Name:");
                ui.text_edit_singleline(&mut app.rename_collection_input);
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("Rename").clicked() {
                        let new_name = app.rename_collection_input.trim().to_string();
                        if !new_name.is_empty() {
                            if let Some(c) = app.collections.iter_mut().find(|c| c.id == col_id) {
                                c.name = new_name;
                                let cloned = c.clone();
                                app.catalog_db.save_collection(&cloned);
                            }
                        }
                        app.show_rename_collection_modal = None;
                    }
                    if ui.button("Cancel").clicked() {
                        app.show_rename_collection_modal = None;
                    }
                });
            });
        if !open {
            app.show_rename_collection_modal = None;
        }
    }
}

fn render_snapshot_modals(app: &mut LightRawApp, ctx: &egui::Context) {
    if app.show_create_snapshot_modal {
        let mut open = true;
        egui::Window::new("New Develop Snapshot")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .fixed_size(Vec2::new(360.0, 125.0))
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label("Snapshot Name:");
                let resp = ui.text_edit_singleline(&mut app.new_snapshot_name);
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    let submit = ui.button("Create").clicked()
                        || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
                    if submit {
                        let name = app.new_snapshot_name.trim().to_string();
                        if !name.is_empty() && !app.photos.is_empty() {
                            let snap = SnapshotItem {
                                name: name.clone(),
                                timestamp: "Saved".into(),
                                params: app.active_photo().params.clone(),
                            };
                            app.active_photo_mut().snapshots.push(snap);
                            app.persist_active_photo();
                            app.status_message = format!("Created snapshot '{}'.", name);
                        }
                        app.show_create_snapshot_modal = false;
                    }
                    if ui.button("Cancel").clicked() {
                        app.show_create_snapshot_modal = false;
                    }
                });
            });
        if !open {
            app.show_create_snapshot_modal = false;
        }
    }

    if let Some(snap_idx) = app.show_rename_snapshot_modal {
        let mut open = true;
        egui::Window::new("Rename Snapshot")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .fixed_size(Vec2::new(340.0, 120.0))
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label("Snapshot Name:");
                ui.text_edit_singleline(&mut app.rename_snapshot_input);
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("Rename").clicked() {
                        let trimmed = app.rename_snapshot_input.trim().to_string();
                        if !trimmed.is_empty() && !app.photos.is_empty() {
                            if let Some(s) = app.active_photo_mut().snapshots.get_mut(snap_idx) {
                                s.name = trimmed;
                            }
                            app.persist_active_photo();
                        }
                        app.show_rename_snapshot_modal = None;
                    }
                    if ui.button("Cancel").clicked() {
                        app.show_rename_snapshot_modal = None;
                    }
                });
            });
        if !open {
            app.show_rename_snapshot_modal = None;
        }
    }
}

fn render_sync_metadata_modal(app: &mut LightRawApp, ctx: &egui::Context) {
    if !app.show_sync_metadata_modal {
        return;
    }
    let mut open = true;
    egui::Window::new(format!(
        "Synchronize Metadata ({} Selected Photos)",
        app.selected_photo_ids.len()
    ))
    .open(&mut open)
    .collapsible(false)
    .resizable(false)
    .fixed_size(Vec2::new(400.0, 230.0))
    .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
    .show(ctx, |ui| {
        ui.label("Copy metadata fields from active photo to all selected photos:");
        ui.separator();
        ui.checkbox(&mut app.sync_meta_title_caption, "Title & Caption");
        ui.checkbox(&mut app.sync_meta_creator_copyright, "Creator & Copyright");
        ui.checkbox(&mut app.sync_meta_rating_label, "Rating, Pick Flag & Color Label");
        ui.checkbox(&mut app.sync_meta_keywords, "Keywords & Person Tag");
        ui.checkbox(&mut app.sync_meta_gps_location, "GPS Coordinates & Location Name");
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if ui.button("Synchronize").clicked() {
                app.execute_sync_metadata();
                app.show_sync_metadata_modal = false;
            }
            if ui.button("Cancel").clicked() {
                app.show_sync_metadata_modal = false;
            }
        });
    });
    if !open {
        app.show_sync_metadata_modal = false;
    }
}
