// ============================================================================
// LightRaw — Full-Resolution Export, Multi-Page PDF 1.4, Print & Web Engine
// ============================================================================

use super::LightRawApp;
use crate::catalog::{decode_real_photo_file, PhotoItem};
use crate::pipeline::{render_develop_pipeline, RawSourceImage, RenderOptions};
use crate::presets::{export_preset_to_xmp, DevelopPreset};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
extern "C" {
    fn lightraw_run_page_setup();
    fn lightraw_print_rgba_sheet(
        rgba_ptr: *const u8,
        width: u32,
        height: u32,
        job_title_utf8: *const std::os::raw::c_char,
    ) -> std::os::raw::c_int;
}

pub use crate::watermark::draw_bitmap_text_rgba;

pub fn apply_output_sharpening_rgba(rgba: &mut [u8], w: u32, h: u32, mode: &str) {
    let strength: f32 = match mode {
        "Screen — Standard" => 0.35,
        "Screen — High" => 0.65,
        "Matte Paper — Standard" => 0.55,
        "Glossy Paper — Standard" => 0.42,
        _ => return,
    };
    if w < 4 || h < 4 {
        return;
    }
    let src = rgba.to_vec();
    let wu = w as usize;
    let hu = h as usize;
    for y in 1..(hu - 1) {
        for x in 1..(wu - 1) {
            let i = (y * wu + x) * 4;
            for c in 0..3 {
                let center = src[i + c] as f32;
                let blur = (src[((y - 1) * wu + x) * 4 + c] as f32
                    + src[((y + 1) * wu + x) * 4 + c] as f32
                    + src[(y * wu + (x - 1)) * 4 + c] as f32
                    + src[(y * wu + (x + 1)) * 4 + c] as f32)
                    * 0.25;
                let v = (center + (center - blur) * strength).clamp(0.0, 255.0);
                rgba[i + c] = v as u8;
            }
        }
    }
}

pub fn draw_watermark_rgba(rgba: &mut [u8], w: u32, h: u32, text: &str) {
    let trimmed = text.trim();
    if trimmed.is_empty() || w < 64 || h < 64 {
        return;
    }
    let scale = ((w.min(h) as i32) / 420).clamp(1, 6);
    let char_count = trimmed.chars().count() as i32;
    let text_w = char_count * 6 * scale;
    let text_h = 7 * scale;
    let margin = (18 * scale).min(w as i32 / 8);
    let x0 = (w as i32 - text_w - margin).max(8);
    let y0 = (h as i32 - text_h - margin).max(8);
    draw_bitmap_text_rgba(
        rgba,
        w,
        h,
        x0 + scale.max(1),
        y0 + scale.max(1),
        trimmed,
        scale,
        [0, 0, 0, 150],
    );
    draw_bitmap_text_rgba(rgba, w, h, x0, y0, trimmed, scale, [255, 255, 255, 215]);
}

/// Pure-Rust multi-page PDF 1.4 writer embedding high-quality DCT (JPEG) image streams.
pub fn write_multipage_jpeg_pdf(
    path: &Path,
    pages_rgba: &[(u32, u32, Vec<u8>)],
    dpi: f32,
) -> std::io::Result<()> {
    if pages_rgba.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "No pages to export",
        ));
    }
    let eff_dpi = dpi.clamp(72.0, 1200.0);
    // Object layout:
    // 1: Catalog
    // 2: Pages
    // For page i in 0..N:
    //   page_obj = 3 + i * 3
    //   content_obj = 4 + i * 3
    //   image_obj = 5 + i * 3
    let n = pages_rgba.len();
    let total_objs = 2 + n * 3;
    let mut buf: Vec<u8> = Vec::with_capacity(n * 256 * 1024);
    let mut offsets: Vec<usize> = vec![0; total_objs + 1];

    buf.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");

    // 1: Catalog
    offsets[1] = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    // 2: Pages
    offsets[2] = buf.len();
    let kids: Vec<String> = (0..n).map(|i| format!("{} 0 R", 3 + i * 3)).collect();
    write!(
        &mut buf,
        "2 0 obj\n<< /Type /Pages /Kids [{}] /Count {} >>\nendobj\n",
        kids.join(" "),
        n
    )?;

    for (i, (w, h, rgba)) in pages_rgba.iter().enumerate() {
        let page_obj = 3 + i * 3;
        let content_obj = 4 + i * 3;
        let image_obj = 5 + i * 3;

        let pt_w = (*w as f32 / eff_dpi * 72.0).max(72.0);
        let pt_h = (*h as f32 / eff_dpi * 72.0).max(72.0);

        // Encode RGBA to RGB JPEG quality 92
        let mut rgb = Vec::with_capacity((*w as usize) * (*h as usize) * 3);
        for px in rgba.chunks_exact(4) {
            rgb.push(px[0]);
            rgb.push(px[1]);
            rgb.push(px[2]);
        }
        let mut jpeg_bytes = Vec::new();
        {
            let mut enc =
                image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_bytes, 92);
            enc.encode(&rgb, *w, *h, image::ExtendedColorType::Rgb8)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        }

        // Page object
        offsets[page_obj] = buf.len();
        write!(
            &mut buf,
            "{} 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {:.2} {:.2}] /Resources << /XObject << /Im0 {} 0 R >> >> /Contents {} 0 R >>\nendobj\n",
            page_obj, pt_w, pt_h, image_obj, content_obj
        )?;

        // Content stream
        let content_str = format!("q\n{:.2} 0 0 {:.2} 0 0 cm\n/Im0 Do\nQ\n", pt_w, pt_h);
        offsets[content_obj] = buf.len();
        write!(
            &mut buf,
            "{} 0 obj\n<< /Length {} >>\nstream\n{}endstream\nendobj\n",
            content_obj,
            content_str.len(),
            content_str
        )?;

        // Image XObject stream
        offsets[image_obj] = buf.len();
        write!(
            &mut buf,
            "{} 0 obj\n<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length {} >>\nstream\n",
            image_obj,
            w,
            h,
            jpeg_bytes.len()
        )?;
        buf.extend_from_slice(&jpeg_bytes);
        buf.extend_from_slice(b"\nendstream\nendobj\n");
    }

    let xref_offset = buf.len();
    write!(&mut buf, "xref\n0 {}\n0000000000 65535 f \n", total_objs + 1)?;
    for obj_id in 1..=total_objs {
        write!(&mut buf, "{:010} 00000 n \n", offsets[obj_id])?;
    }
    write!(
        &mut buf,
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
        total_objs + 1,
        xref_offset
    )?;

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(path, buf)
}

fn blit_rgba_fit(
    canvas: &mut [u8],
    cw: u32,
    ch: u32,
    src: &[u8],
    sw: u32,
    sh: u32,
    box_x: i32,
    box_y: i32,
    box_w: i32,
    box_h: i32,
    draw_border: bool,
) {
    if sw == 0 || sh == 0 || box_w <= 2 || box_h <= 2 {
        return;
    }
    let sc = (box_w as f32 / sw as f32).min(box_h as f32 / sh as f32);
    let dw = ((sw as f32 * sc).round() as i32).max(1);
    let dh = ((sh as f32 * sc).round() as i32).max(1);
    let dx0 = box_x + (box_w - dw) / 2;
    let dy0 = box_y + (box_h - dh) / 2;

    for dy in 0..dh {
        let py = dy0 + dy;
        if py < 0 || py >= ch as i32 {
            continue;
        }
        let sy = (((dy as f32 + 0.5) / dh as f32) * sh as f32)
            .floor()
            .clamp(0.0, (sh - 1) as f32) as u32;
        for dx in 0..dw {
            let px = dx0 + dx;
            if px < 0 || px >= cw as i32 {
                continue;
            }
            let sx = (((dx as f32 + 0.5) / dw as f32) * sw as f32)
                .floor()
                .clamp(0.0, (sw - 1) as f32) as u32;
            let si = ((sy * sw + sx) * 4) as usize;
            let di = ((py as u32 * cw + px as u32) * 4) as usize;
            canvas[di..di + 4].copy_from_slice(&src[si..si + 4]);
        }
    }

    if draw_border {
        for bx in dx0..(dx0 + dw) {
            for by in [dy0, dy0 + dh - 1] {
                if bx >= 0 && by >= 0 && (bx as u32) < cw && (by as u32) < ch {
                    let di = ((by as u32 * cw + bx as u32) * 4) as usize;
                    canvas[di..di + 4].copy_from_slice(&[30, 30, 30, 255]);
                }
            }
        }
        for by in dy0..(dy0 + dh) {
            for bx in [dx0, dx0 + dw - 1] {
                if bx >= 0 && by >= 0 && (bx as u32) < cw && (by as u32) < ch {
                    let di = ((by as u32 * cw + bx as u32) * 4) as usize;
                    canvas[di..di + 4].copy_from_slice(&[30, 30, 30, 255]);
                }
            }
        }
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

impl LightRawApp {
    /// Decodes a photo specifically for export/print without polluting the interactive LRU cache.
    pub fn decode_photo_for_output(
        &self,
        photo: &PhotoItem,
        max_edge: u32,
    ) -> RawSourceImage {
        if let Some(ref p_str) = photo.real_file_path {
            let p = Path::new(p_str);
            if p.exists() {
                if let Some(raw) = decode_real_photo_file(p, max_edge, max_edge) {
                    return raw;
                }
            }
        }
        if let Some(blob) = self.catalog_db.load_raw_blob(photo.id) {
            return blob;
        }
        RawSourceImage::generate_sample_raw(
            (photo.id.max(0) as usize) % 6,
            photo.width.clamp(320, 1600),
            photo.height.clamp(240, 1200),
        )
    }

    pub fn execute_batch_export(&mut self) {
        let out_root = PathBuf::from(&self.export_folder).join(self.export_subfolder.trim());
        if let Err(err) = std::fs::create_dir_all(&out_root) {
            self.status_message = format!("Export failed to create folder: {}", err);
            return;
        }

        let target_ids = if self.selected_photo_ids.is_empty() {
            vec![self.active_photo_id]
        } else {
            self.selected_photo_ids.clone()
        };

        let photos_to_export: Vec<PhotoItem> = target_ids
            .iter()
            .filter_map(|id| self.photos.iter().find(|p| p.id == *id).cloned())
            .collect();

        let mut exported_count = 0usize;
        for (seq_idx, photo) in photos_to_export.iter().enumerate() {
            let orig_stem = Path::new(&photo.file_name)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Photo");
            let stem = if self.export_rename_enabled && !self.export_rename_prefix.trim().is_empty()
            {
                format!("{}_{:04}", self.export_rename_prefix.trim(), seq_idx + 1)
            } else {
                orig_stem.to_string()
            };

            // Original + XMP mode
            if self.export_format.starts_with("Original") {
                if let Some(ref src_path_str) = photo.real_file_path {
                    let src_path = Path::new(src_path_str);
                    if src_path.exists() {
                        let ext = src_path
                            .extension()
                            .and_then(|s| s.to_str())
                            .unwrap_or("RAW");
                        let dst_raw = out_root.join(format!("{}.{}", stem, ext));
                        let _ = std::fs::copy(src_path, &dst_raw);
                    }
                }
                let xmp_preset = DevelopPreset {
                    id: format!("sidecar_{}", photo.id),
                    name: stem.clone(),
                    category: "Export Sidecar".into(),
                    is_user: true,
                    is_favorite: false,
                    is_visible: true,
                    params: photo.params.clone(),
                    mask: None,
                    xmp_file_path: None,
                };
                let _ = std::fs::write(
                    out_root.join(format!("{}.xmp", stem)),
                    export_preset_to_xmp(&xmp_preset),
                );
                exported_count += 1;
                continue;
            }

            // Full native resolution when `export_resize_enabled` is false (`0` = full sensor dimensions)
            let max_edge = if self.export_resize_enabled {
                self.export_long_edge_px.max(64)
            } else {
                0
            };
            let raw = self.decode_photo_for_output(photo, max_edge);

            let soft_profile = match self.export_color_space.as_str() {
                "Display P3" => 1,
                "AdobeRGB (1998)" | "ProPhoto RGB" => 2,
                _ => 0,
            };
            let mut frame = render_develop_pipeline(
                &raw,
                &photo.params,
                RenderOptions {
                    apply_crop: true,
                    soft_proofing: soft_profile > 0,
                    soft_proof_profile: soft_profile,
                    ..RenderOptions::default()
                },
            );

            // Resize exact post-crop long edge if requested
            if self.export_resize_enabled {
                let cur_long = frame.width.max(frame.height);
                if cur_long > 0 && cur_long != self.export_long_edge_px {
                    if let Some(rgba_img) =
                        image::RgbaImage::from_raw(frame.width, frame.height, frame.rgba.clone())
                    {
                        let scale = self.export_long_edge_px as f32 / cur_long as f32;
                        let nw = ((frame.width as f32 * scale).round() as u32).max(1);
                        let nh = ((frame.height as f32 * scale).round() as u32).max(1);
                        let resized = image::imageops::resize(
                            &rgba_img,
                            nw,
                            nh,
                            image::imageops::FilterType::Lanczos3,
                        );
                        frame.width = nw;
                        frame.height = nh;
                        frame.rgba = resized.into_raw();
                    }
                }
            }

            // Output Sharpening
            apply_output_sharpening_rgba(
                &mut frame.rgba,
                frame.width,
                frame.height,
                &self.export_sharpening,
            );

            // Full Lightroom Classic Watermark Compositor (Graphic PNG/JPG & Text + Shadow + Inset + Anchor)
            if self.export_watermark_enabled {
                crate::watermark::composite_watermark_rgba(
                    &mut frame.rgba,
                    frame.width,
                    frame.height,
                    &self.watermark_settings,
                );
            }

            let write_ok = if self.export_format.starts_with("PNG") {
                let out_file = out_root.join(format!("{}.png", stem));
                File::create(&out_file)
                    .and_then(|f| {
                        let enc = image::codecs::png::PngEncoder::new(BufWriter::new(f));
                        use image::ImageEncoder;
                        enc.write_image(
                            &frame.rgba,
                            frame.width,
                            frame.height,
                            image::ExtendedColorType::Rgba8,
                        )
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                    })
                    .is_ok()
            } else if self.export_format.starts_with("TIFF") {
                let out_file = out_root.join(format!("{}.tif", stem));
                File::create(&out_file)
                    .and_then(|f| {
                        let enc = image::codecs::tiff::TiffEncoder::new(BufWriter::new(f));
                        use image::ImageEncoder;
                        enc.write_image(
                            &frame.rgba,
                            frame.width,
                            frame.height,
                            image::ExtendedColorType::Rgba8,
                        )
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                    })
                    .is_ok()
            } else {
                // Real quality-controlled RGB8 JPEG encoding!
                let out_file = out_root.join(format!("{}.jpg", stem));
                let mut rgb =
                    Vec::with_capacity((frame.width as usize) * (frame.height as usize) * 3);
                for px in frame.rgba.chunks_exact(4) {
                    rgb.push(px[0]);
                    rgb.push(px[1]);
                    rgb.push(px[2]);
                }
                File::create(&out_file)
                    .and_then(|f| {
                        let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(
                            BufWriter::new(f),
                            self.export_quality.clamp(10, 100),
                        );
                        enc.encode(
                            &rgb,
                            frame.width,
                            frame.height,
                            image::ExtendedColorType::Rgb8,
                        )
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                    })
                    .is_ok()
            };

            if write_ok {
                exported_count += 1;
            }

            if self.export_xmp_sidecar {
                let xmp_preset = DevelopPreset {
                    id: format!("sidecar_{}", photo.id),
                    name: stem.clone(),
                    category: "Export Sidecar".into(),
                    is_user: true,
                    is_favorite: false,
                    is_visible: true,
                    params: photo.params.clone(),
                    mask: None,
                    xmp_file_path: None,
                };
                let _ = std::fs::write(
                    out_root.join(format!("{}.xmp", stem)),
                    export_preset_to_xmp(&xmp_preset),
                );
            }
        }

        self.status_message = format!(
            "Exported {} full-pipeline photo(s) ({}, Q={}) to {}",
            exported_count,
            self.export_format,
            self.export_quality,
            out_root.display()
        );
    }

    /// Renders all paginated Print module sheets at print resolution (non-repeating pagination!).
    pub fn render_print_sheets(&self) -> Vec<(u32, u32, Vec<u8>)> {
        let dpi = self.print_dpi.clamp(150.0, 360.0);
        // Standard A4 / Letter 8.5 x 11 in scaled by DPI
        let page_w = ((8.5 * dpi).round() as u32).clamp(1200, 3060);
        let page_h = ((11.0 * dpi).round() as u32).clamp(1550, 3960);

        let rows = self.print_rows.max(1);
        let cols = self.print_cols.max(1);
        let per_page = rows * cols;

        let ids = {
            let mut list = self.filtered_photo_ids();
            if list.is_empty() && !self.photos.is_empty() {
                list.push(self.active_photo_id);
            }
            list
        };
        if ids.is_empty() {
            return Vec::new();
        }

        let total_pages = (ids.len() + per_page - 1) / per_page;
        let margin_px = ((self.print_margin_mm / 25.4) * dpi).round() as i32;
        let spacing_px = ((self.print_cell_spacing / 72.0) * dpi).round() as i32;

        let content_w = (page_w as i32 - margin_px * 2).max(100);
        let content_h = (page_h as i32 - margin_px * 2).max(100);
        let cell_w = ((content_w - spacing_px * (cols as i32 - 1)) / cols as i32).max(40);
        let cell_h = ((content_h - spacing_px * (rows as i32 - 1)) / rows as i32).max(40);

        let mut sheets = Vec::with_capacity(total_pages);
        for page_idx in 0..total_pages {
            let mut canvas = vec![255u8; (page_w as usize) * (page_h as usize) * 4];
            for slot in 0..per_page {
                let global_idx = page_idx * per_page + slot;
                let Some(&pid) = ids.get(global_idx) else {
                    break;
                };
                let Some(photo) = self.photos.iter().find(|p| p.id == pid) else {
                    continue;
                };

                let r = slot / cols;
                let c = slot % cols;
                let cx = margin_px + (c as i32) * (cell_w + spacing_px);
                let cy = margin_px + (r as i32) * (cell_h + spacing_px);
                let caption_reserve = if self.print_show_page_info { 28 } else { 0 };

                let raw = self.decode_photo_for_output(photo, (cell_w.max(cell_h) as u32).max(600));
                let rendered =
                    render_develop_pipeline(&raw, &photo.params, RenderOptions::default());

                blit_rgba_fit(
                    &mut canvas,
                    page_w,
                    page_h,
                    &rendered.rgba,
                    rendered.width,
                    rendered.height,
                    cx,
                    cy,
                    cell_w,
                    (cell_h - caption_reserve).max(24),
                    self.print_stroke_border,
                );

                if self.print_show_page_info {
                    let cap = format!("{} ISO {}", photo.file_name, photo.iso);
                    draw_bitmap_text_rgba(
                        &mut canvas,
                        page_w,
                        page_h,
                        cx + 4,
                        cy + cell_h - 20,
                        &cap,
                        2,
                        [50, 50, 50, 255],
                    );
                }
            }

            if self.print_show_watermark {
                let wm = format!(
                    "LIGHTRAW PRINT  PAGE {}/{}  {:.0} PPI",
                    page_idx + 1,
                    total_pages,
                    dpi
                );
                draw_bitmap_text_rgba(
                    &mut canvas,
                    page_w,
                    page_h,
                    margin_px,
                    page_h as i32 - (margin_px / 2).max(24),
                    &wm,
                    2,
                    [90, 90, 90, 255],
                );
            }

            sheets.push((page_w, page_h, canvas));
        }

        sheets
    }

    pub fn export_print_sheet_to_disk(&mut self, as_pdf: bool) {
        let sheets = self.render_print_sheets();
        if sheets.is_empty() {
            self.status_message = "Print Export: No photos available to print.".into();
            return;
        }

        if as_pdf {
            let default_path = PathBuf::from(&self.export_folder).join("LightRaw_Print_Sheet.pdf");
            let dest = rfd::FileDialog::new()
                .set_file_name("LightRaw_Print_Sheet.pdf")
                .add_filter("PDF Document", &["pdf"])
                .save_file()
                .unwrap_or(default_path);
            match write_multipage_jpeg_pdf(&dest, &sheets, self.print_dpi) {
                Ok(()) => {
                    self.status_message = format!(
                        "Exported {}-page Print PDF ({:.0} PPI) to {}",
                        sheets.len(),
                        self.print_dpi,
                        dest.display()
                    );
                    let _ = open::that(&dest);
                }
                Err(e) => {
                    self.status_message = format!("Failed to write Print PDF: {}", e);
                }
            }
        } else {
            let out_dir = PathBuf::from(&self.export_folder).join("Print_Sheets");
            let _ = std::fs::create_dir_all(&out_dir);
            for (idx, (w, h, rgba)) in sheets.iter().enumerate() {
                if let Some(img) = image::RgbaImage::from_raw(*w, *h, rgba.clone()) {
                    let p = out_dir.join(format!("Print_Sheet_Page_{:02}.jpg", idx + 1));
                    let _ = image::DynamicImage::ImageRgba8(img).to_rgb8().save(&p);
                }
            }
            self.status_message = format!(
                "Exported {} Print JPEG sheet(s) ({:.0} PPI) to {}",
                sheets.len(),
                self.print_dpi,
                out_dir.display()
            );
            let _ = open::that(&out_dir);
        }
    }

    pub fn run_native_page_setup(&mut self) {
        #[cfg(target_os = "macos")]
        unsafe {
            lightraw_run_page_setup();
        }
        self.status_message = "Page Setup updated.".into();
    }

    pub fn print_sheet_to_printer(&mut self) {
        let sheets = self.render_print_sheets();
        let Some((w, h, rgba)) = sheets
            .get(self.print_current_page.min(sheets.len().saturating_sub(1)))
            .or_else(|| sheets.first())
        else {
            self.status_message = "No photos in current view to send to printer.".into();
            return;
        };
        #[cfg(target_os = "macos")]
        {
            if let Ok(title) = std::ffi::CString::new("LightRaw Fine Art Print") {
                let ok = unsafe {
                    lightraw_print_rgba_sheet(rgba.as_ptr(), *w, *h, title.as_ptr())
                };
                if ok != 0 {
                    self.status_message = "Sent print sheet to macOS Printer Queue.".into();
                } else {
                    self.status_message = "Print dialog cancelled.".into();
                }
            }
        }
    }

    pub fn export_book_pages_to_disk(&mut self, as_pdf: bool) {
        let ids = self.filtered_photo_ids();
        if ids.is_empty() {
            self.status_message = "Book Export: No photos in current view.".into();
            return;
        }
        let page_w: u32 = match self.book_size.as_str() {
            "Small Square (7x7 in)" => 1600,
            "Large Square (12x12 in)" => 2200,
            _ => 2200,
        };
        let page_h: u32 = match self.book_size.as_str() {
            "Small Square (7x7 in)" => 1600,
            "Large Square (12x12 in)" => 2200,
            "Standard Portrait (8x10 in)" => 1800,
            _ => 1500,
        };

        let bg_val = if self.book_bg_white { 248u8 } else { 24u8 };
        let fg_col = if self.book_bg_white {
            [32u8, 32, 36, 255]
        } else {
            [232u8, 232, 238, 255]
        };

        let mut pages: Vec<(u32, u32, Vec<u8>)> = Vec::new();

        // 1. Cover Page
        let mut cover = vec![bg_val; (page_w as usize) * (page_h as usize) * 4];
        for px in cover.chunks_exact_mut(4) {
            px[3] = 255;
        }
        draw_bitmap_text_rgba(
            &mut cover,
            page_w,
            page_h,
            120,
            110,
            &self.book_title,
            5,
            fg_col,
        );
        draw_bitmap_text_rgba(
            &mut cover,
            page_w,
            page_h,
            120,
            175,
            &format!("{}  -  {}", self.book_subtitle, self.book_author),
            3,
            fg_col,
        );
        if let Some(&first_id) = ids.first() {
            if let Some(photo) = self.photos.iter().find(|p| p.id == first_id) {
                let raw = self.decode_photo_for_output(photo, 1400);
                let rendered =
                    render_develop_pipeline(&raw, &photo.params, RenderOptions::default());
                blit_rgba_fit(
                    &mut cover,
                    page_w,
                    page_h,
                    &rendered.rgba,
                    rendered.width,
                    rendered.height,
                    120,
                    240,
                    page_w as i32 - 240,
                    page_h as i32 - 340,
                    true,
                );
            }
        }
        pages.push((page_w, page_h, cover));

        // 2. Interior Photo Spreads
        let per_spread = match self.book_layout.as_str() {
            "One Photo Per Page" => 1usize,
            "Grid 4-Up" => 4usize,
            _ => 2usize,
        };
        let pad = (self.book_page_padding.round() as i32 * 3).clamp(24, 180);

        for (sp_idx, chunk) in ids.chunks(per_spread).enumerate() {
            let mut spread = vec![bg_val; (page_w as usize) * (page_h as usize) * 4];
            for px in spread.chunks_exact_mut(4) {
                px[3] = 255;
            }

            let cols = if per_spread == 1 { 1 } else { 2 };
            let rows = (per_spread + cols - 1) / cols;
            let cw = ((page_w as i32 - pad * (cols as i32 + 1)) / cols as i32).max(60);
            let ch = ((page_h as i32 - pad * (rows as i32 + 1) - 50) / rows as i32).max(60);

            for (slot, &pid) in chunk.iter().enumerate() {
                if let Some(photo) = self.photos.iter().find(|p| p.id == pid) {
                    let r = slot / cols;
                    let c = slot % cols;
                    let bx = pad + (c as i32) * (cw + pad);
                    let by = pad + (r as i32) * (ch + pad);
                    let raw = self.decode_photo_for_output(photo, 1200);
                    let rendered =
                        render_develop_pipeline(&raw, &photo.params, RenderOptions::default());
                    blit_rgba_fit(
                        &mut spread,
                        page_w,
                        page_h,
                        &rendered.rgba,
                        rendered.width,
                        rendered.height,
                        bx,
                        by,
                        cw,
                        ch,
                        true,
                    );
                }
            }

            if self.book_show_page_numbers {
                let pg = format!("SPREAD {}", sp_idx + 1);
                draw_bitmap_text_rgba(
                    &mut spread,
                    page_w,
                    page_h,
                    page_w as i32 / 2 - 48,
                    page_h as i32 - 36,
                    &pg,
                    2,
                    fg_col,
                );
            }
            pages.push((page_w, page_h, spread));
        }

        if as_pdf {
            let default_path = PathBuf::from(&self.export_folder).join("LightRaw_Photo_Book.pdf");
            let dest = rfd::FileDialog::new()
                .set_file_name("LightRaw_Photo_Book.pdf")
                .add_filter("PDF Document", &["pdf"])
                .save_file()
                .unwrap_or(default_path);
            match write_multipage_jpeg_pdf(&dest, &pages, 200.0) {
                Ok(()) => {
                    self.status_message = format!(
                        "Exported {}-page Photo Book PDF to {}",
                        pages.len(),
                        dest.display()
                    );
                    let _ = open::that(&dest);
                }
                Err(e) => {
                    self.status_message = format!("Book PDF export failed: {}", e);
                }
            }
        } else {
            let out_dir = PathBuf::from(&self.export_folder).join("Book_Spreads");
            let _ = std::fs::create_dir_all(&out_dir);
            for (idx, (w, h, rgba)) in pages.iter().enumerate() {
                if let Some(img) = image::RgbaImage::from_raw(*w, *h, rgba.clone()) {
                    let p = out_dir.join(format!("Book_Page_{:02}.jpg", idx + 1));
                    let _ = image::DynamicImage::ImageRgba8(img).to_rgb8().save(&p);
                }
            }
            self.status_message = format!(
                "Exported {} Book JPEG spread(s) to {}",
                pages.len(),
                out_dir.display()
            );
            let _ = open::that(&out_dir);
        }
    }

    pub fn export_slideshow_to_pdf(&mut self) {
        let ids = self.filtered_photo_ids();
        if ids.is_empty() {
            self.status_message = "Slideshow Export: No photos in current view.".into();
            return;
        }
        let page_w = 1920u32;
        let page_h = 1080u32;
        let bg_val = if self.slideshow_backdrop_dark {
            16u8
        } else {
            242u8
        };
        let fg_col = if self.slideshow_backdrop_dark {
            [235u8, 235, 240, 255]
        } else {
            [30u8, 30, 35, 255]
        };

        let mut slides = Vec::with_capacity(ids.len());
        for pid in ids {
            let Some(photo) = self.photos.iter().find(|p| p.id == pid) else {
                continue;
            };
            let mut canvas = vec![bg_val; (page_w as usize) * (page_h as usize) * 4];
            for px in canvas.chunks_exact_mut(4) {
                px[3] = 255;
            }
            let raw = self.decode_photo_for_output(photo, 1920);
            let rendered = render_develop_pipeline(&raw, &photo.params, RenderOptions::default());
            blit_rgba_fit(
                &mut canvas,
                page_w,
                page_h,
                &rendered.rgba,
                rendered.width,
                rendered.height,
                80,
                70,
                page_w as i32 - 160,
                page_h as i32 - 150,
                self.slideshow_stroke_border,
            );
            if self.slideshow_show_identity {
                draw_bitmap_text_rgba(
                    &mut canvas,
                    page_w,
                    page_h,
                    48,
                    28,
                    &format!("{} - {}", self.identity_plate_subtitle, self.identity_plate_owner),
                    3,
                    fg_col,
                );
            }
            if self.slideshow_show_caption {
                draw_bitmap_text_rgba(
                    &mut canvas,
                    page_w,
                    page_h,
                    80,
                    page_h as i32 - 44,
                    &format!("{}  |  {} ISO {}", photo.file_name, photo.shutter_speed, photo.iso),
                    2,
                    fg_col,
                );
            }
            slides.push((page_w, page_h, canvas));
        }

        let default_path = PathBuf::from(&self.export_folder).join("LightRaw_Slideshow.pdf");
        let dest = rfd::FileDialog::new()
            .set_file_name("LightRaw_Slideshow.pdf")
            .add_filter("PDF Presentation", &["pdf"])
            .save_file()
            .unwrap_or(default_path);
        match write_multipage_jpeg_pdf(&dest, &slides, 144.0) {
            Ok(()) => {
                self.status_message = format!(
                    "Exported {}-slide PDF Slideshow to {}",
                    slides.len(),
                    dest.display()
                );
                let _ = open::that(&dest);
            }
            Err(e) => {
                self.status_message = format!("Slideshow PDF export failed: {}", e);
            }
        }
    }

    pub fn export_and_open_web_gallery(&mut self) {
        let base_dir = rfd::FileDialog::new()
            .set_title("Choose Destination Folder for HTML5 Web Gallery")
            .pick_folder()
            .unwrap_or_else(|| PathBuf::from(&self.export_folder));
        let gallery_dir = base_dir.join("LightRaw_Web_Gallery");
        let img_dir = gallery_dir.join("images");
        let _ = std::fs::create_dir_all(&img_dir);

        let ids = self.filtered_photo_ids();
        let bg = if self.web_dark_theme { "#14161a" } else { "#f5f5f7" };
        let fg = if self.web_dark_theme { "#ebebf0" } else { "#1c1c20" };
        let card = if self.web_dark_theme { "#202228" } else { "#ffffff" };

        let mut cards_html = String::new();
        let mut count = 0usize;
        for (idx, pid) in ids.into_iter().enumerate() {
            if let Some(photo) = self.photos.iter().find(|p| p.id == pid).cloned() {
                let raw = self.decode_photo_for_output(&photo, 1600);
                let frame = render_develop_pipeline(&raw, &photo.params, RenderOptions::default());
                let img_name = format!("photo_{:04}.jpg", idx + 1);
                let mut rgb =
                    Vec::with_capacity((frame.width as usize) * (frame.height as usize) * 3);
                for px in frame.rgba.chunks_exact(4) {
                    rgb.push(px[0]);
                    rgb.push(px[1]);
                    rgb.push(px[2]);
                }
                if let Ok(f) = File::create(img_dir.join(&img_name)) {
                    let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(
                        BufWriter::new(f),
                        self.web_quality.clamp(40, 100),
                    );
                    let _ = enc.encode(
                        &rgb,
                        frame.width,
                        frame.height,
                        image::ExtendedColorType::Rgb8,
                    );
                }

                let title_esc = html_escape(if photo.title.trim().is_empty() {
                    &photo.file_name
                } else {
                    &photo.title
                });
                let exif_str = if self.web_show_exif {
                    html_escape(&format!(
                        "{} • {} at f/{:.1} • ISO {}",
                        photo.camera_model, photo.shutter_speed, photo.aperture_f, photo.iso
                    ))
                } else {
                    String::new()
                };
                cards_html.push_str(&format!(
                    "<div class='card' onclick='openLightbox({})'><img loading='lazy' src='images/{}' alt='{}'><h3>{}</h3><p>{}</p></div>\n",
                    count, img_name, title_esc, title_esc, exif_str
                ));
                count += 1;
            }
        }

        let html = format!(
            "<!DOCTYPE html><html lang='en'><head><meta charset='utf-8'><meta name='viewport' content='width=device-width,initial-scale=1'>\
            <title>{}</title><style>\
            body{{background:{};color:{};font-family:-apple-system,BlinkMacSystemFont,sans-serif;margin:0;padding:36px;}}\
            header{{text-align:center;margin-bottom:32px;}}\
            .grid{{display:grid;grid-template-columns:repeat({},minmax(220px,1fr));gap:18px;max-width:1600px;margin:0 auto;}}\
            @media(max-width:900px){{.grid{{grid-template-columns:repeat(auto-fill,minmax(220px,1fr));}}}}\
            .card{{background:{};border-radius:10px;padding:12px;box-shadow:0 6px 18px rgba(0,0,0,0.22);cursor:pointer;transition:transform .16s ease;}}\
            .card:hover{{transform:translateY(-3px);}}\
            .card img{{width:100%;aspect-ratio:4/3;object-fit:contain;border-radius:6px;background:#0d0e11;}}\
            .card h3{{margin:10px 0 4px;font-size:14px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;}}\
            .card p{{margin:0;font-size:12px;opacity:0.72;}}\
            #lb{{display:none;position:fixed;inset:0;background:rgba(0,0,0,0.92);z-index:999;align-items:center;justify-content:center;flex-direction:column;padding:24px;}}\
            #lb.open{{display:flex;}}\
            #lb img{{max-width:92vw;max-height:82vh;object-fit:contain;border-radius:4px;}}\
            #lb-cap{{color:#eee;margin-top:12px;font-size:14px;text-align:center;}}\
            .lb-nav{{position:fixed;top:50%;transform:translateY(-50%);background:rgba(255,255,255,0.14);color:#fff;border:none;font-size:28px;padding:12px 18px;border-radius:8px;cursor:pointer;}}\
            #lb-prev{{left:20px;}}#lb-next{{right:20px;}}#lb-close{{position:fixed;top:18px;right:24px;background:none;border:none;color:#fff;font-size:30px;cursor:pointer;}}\
            </style></head><body>\
            <header><h1>{}</h1><p>{} — {} ({} photos)</p></header>\
            <div class='grid'>{}</div>\
            <div id='lb' onclick='if(event.target.id===\"lb\")closeLightbox()'>\
            <button id='lb-close' onclick='closeLightbox()'>&times;</button>\
            <button class='lb-nav' id='lb-prev' onclick='stepLightbox(-1)'>&#10094;</button>\
            <img id='lb-img' src=''><div id='lb-cap'></div>\
            <button class='lb-nav' id='lb-next' onclick='stepLightbox(1)'>&#10095;</button>\
            </div>\
            <script>\
            const cards=Array.from(document.querySelectorAll('.card'));let cur=0;\
            function openLightbox(i){{cur=i;showSlide();document.getElementById('lb').classList.add('open');}}\
            function closeLightbox(){{document.getElementById('lb').classList.remove('open');}}\
            function stepLightbox(d){{if(!cards.length)return;cur=(cur+d+cards.length)%cards.length;showSlide();}}\
            function showSlide(){{const c=cards[cur];document.getElementById('lb-img').src=c.querySelector('img').src;\
            document.getElementById('lb-cap').textContent=c.querySelector('h3').textContent+'  '+c.querySelector('p').textContent;}}\
            document.addEventListener('keydown',e=>{{if(!document.getElementById('lb').classList.contains('open'))return;\
            if(e.key==='Escape')closeLightbox();if(e.key==='ArrowLeft')stepLightbox(-1);if(e.key==='ArrowRight')stepLightbox(1);}});\
            </script></body></html>",
            html_escape(&self.web_site_title),
            bg,
            fg,
            self.web_columns.max(1),
            card,
            html_escape(&self.web_site_title),
            html_escape(&self.web_collection_title),
            html_escape(&self.web_description),
            count,
            cards_html
        );

        let index_path = gallery_dir.join("index.html");
        let _ = std::fs::write(&index_path, html);
        self.status_message = format!(
            "Exported responsive HTML5 Web Gallery ({} photos) to {}",
            count,
            index_path.display()
        );
        let _ = open::that(&index_path);
    }
}
