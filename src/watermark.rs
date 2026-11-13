// ============================================================================
// LightRaw — Watermark Engine & Preset System (Adobe Lightroom Classic Replica)
// Pure Rust Graphic (PNG/JPG Alpha) and Text Watermark Compositor
// ============================================================================

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatermarkStyle {
    Text,
    Graphic,
}

impl WatermarkStyle {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Text => "Text",
            Self::Graphic => "Graphic",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatermarkTextAlign {
    Left,
    Center,
    Right,
}

impl WatermarkTextAlign {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Center => "Center",
            Self::Right => "Right",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatermarkSizeMode {
    Proportional,
    Fit,
    Fill,
}

impl WatermarkSizeMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Proportional => "Proportional",
            Self::Fit => "Fit",
            Self::Fill => "Fill",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatermarkAnchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl WatermarkAnchor {
    pub const ALL: [Self; 9] = [
        Self::TopLeft,
        Self::TopCenter,
        Self::TopRight,
        Self::CenterLeft,
        Self::Center,
        Self::CenterRight,
        Self::BottomLeft,
        Self::BottomCenter,
        Self::BottomRight,
    ];

    pub fn row_col(&self) -> (usize, usize) {
        match self {
            Self::TopLeft => (0, 0),
            Self::TopCenter => (0, 1),
            Self::TopRight => (0, 2),
            Self::CenterLeft => (1, 0),
            Self::Center => (1, 1),
            Self::CenterRight => (1, 2),
            Self::BottomLeft => (2, 0),
            Self::BottomCenter => (2, 1),
            Self::BottomRight => (2, 2),
        }
    }

    pub fn from_row_col(row: usize, col: usize) -> Self {
        match (row, col) {
            (0, 0) => Self::TopLeft,
            (0, 1) => Self::TopCenter,
            (0, 2) => Self::TopRight,
            (1, 0) => Self::CenterLeft,
            (1, 1) => Self::Center,
            (1, 2) => Self::CenterRight,
            (2, 0) => Self::BottomLeft,
            (2, 1) => Self::BottomCenter,
            _ => Self::BottomRight,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatermarkRotation {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

impl WatermarkRotation {
    pub fn rotate_cw(&self) -> Self {
        match self {
            Self::Deg0 => Self::Deg90,
            Self::Deg90 => Self::Deg180,
            Self::Deg180 => Self::Deg270,
            Self::Deg270 => Self::Deg0,
        }
    }

    pub fn rotate_ccw(&self) -> Self {
        match self {
            Self::Deg0 => Self::Deg270,
            Self::Deg90 => Self::Deg0,
            Self::Deg180 => Self::Deg90,
            Self::Deg270 => Self::Deg180,
        }
    }

    pub fn degrees(&self) -> f32 {
        match self {
            Self::Deg0 => 0.0,
            Self::Deg90 => 90.0,
            Self::Deg180 => 180.0,
            Self::Deg270 => 270.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatermarkSettings {
    pub style: WatermarkStyle,
    // Graphic Options
    pub graphic_path: Option<String>,
    // Text Options
    pub text: String,
    pub font_family: String,
    pub font_style: String,
    pub text_align: WatermarkTextAlign,
    pub text_color: [u8; 3], // RGB [0..255]
    pub shadow_enabled: bool,
    pub shadow_opacity: f32, // 0..100
    pub shadow_offset: f32,  // 0..100
    pub shadow_radius: f32,  // 0..100
    pub shadow_angle: f32,   // -180..180 (degrees)
    // Watermark Effects
    pub opacity: f32,        // 0..100
    pub size_mode: WatermarkSizeMode,
    pub proportional_size: f32, // 0..100 (%)
    pub inset_horizontal: f32,  // 0..100 (%)
    pub inset_vertical: f32,    // 0..100 (%)
    pub anchor: WatermarkAnchor,
    pub rotation: WatermarkRotation,
}

impl Default for WatermarkSettings {
    fn default() -> Self {
        Self {
            style: WatermarkStyle::Text,
            graphic_path: None,
            text: "© Moestret Photography".into(),
            font_family: "Adobe Clean".into(),
            font_style: "Regular".into(),
            text_align: WatermarkTextAlign::Center,
            text_color: [255, 255, 255],
            shadow_enabled: true,
            shadow_opacity: 80.0,
            shadow_offset: 10.0,
            shadow_radius: 20.0,
            shadow_angle: -90.0,
            opacity: 100.0,
            size_mode: WatermarkSizeMode::Proportional,
            proportional_size: 26.0,
            inset_horizontal: 0.0,
            inset_vertical: 0.0,
            anchor: WatermarkAnchor::BottomCenter,
            rotation: WatermarkRotation::Deg0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatermarkPreset {
    pub name: String,
    pub settings: WatermarkSettings,
}

impl WatermarkPreset {
    pub fn default_presets() -> Vec<Self> {
        let mut list = vec![
            Self {
                name: "Simple Copyright Watermark".into(),
                settings: WatermarkSettings {
                    style: WatermarkStyle::Text,
                    graphic_path: None,
                    text: "© Moestret Photography".into(),
                    font_family: "Adobe Clean".into(),
                    font_style: "Regular".into(),
                    text_align: WatermarkTextAlign::Left,
                    text_color: [255, 255, 255],
                    shadow_enabled: true,
                    shadow_opacity: 80.0,
                    shadow_offset: 10.0,
                    shadow_radius: 20.0,
                    shadow_angle: -90.0,
                    opacity: 100.0,
                    size_mode: WatermarkSizeMode::Proportional,
                    proportional_size: 20.0,
                    inset_horizontal: 3.0,
                    inset_vertical: 3.0,
                    anchor: WatermarkAnchor::BottomLeft,
                    rotation: WatermarkRotation::Deg0,
                },
            },
            Self {
                name: "Centered Moestret Signature".into(),
                settings: WatermarkSettings {
                    style: WatermarkStyle::Text,
                    graphic_path: None,
                    text: "Moestret Studio".into(),
                    font_family: "Adobe Clean".into(),
                    font_style: "Bold".into(),
                    text_align: WatermarkTextAlign::Center,
                    text_color: [255, 255, 255],
                    shadow_enabled: true,
                    shadow_opacity: 90.0,
                    shadow_offset: 12.0,
                    shadow_radius: 25.0,
                    shadow_angle: -90.0,
                    opacity: 85.0,
                    size_mode: WatermarkSizeMode::Proportional,
                    proportional_size: 32.0,
                    inset_horizontal: 0.0,
                    inset_vertical: 0.0,
                    anchor: WatermarkAnchor::Center,
                    rotation: WatermarkRotation::Deg0,
                },
            },
            Self {
                name: "Bottom Right Stamp".into(),
                settings: WatermarkSettings {
                    style: WatermarkStyle::Text,
                    graphic_path: None,
                    text: "LIGHTRAW 1.0".into(),
                    font_family: "Adobe Clean".into(),
                    font_style: "Regular".into(),
                    text_align: WatermarkTextAlign::Right,
                    text_color: [240, 240, 240],
                    shadow_enabled: true,
                    shadow_opacity: 75.0,
                    shadow_offset: 8.0,
                    shadow_radius: 15.0,
                    shadow_angle: -90.0,
                    opacity: 90.0,
                    size_mode: WatermarkSizeMode::Proportional,
                    proportional_size: 18.0,
                    inset_horizontal: 2.5,
                    inset_vertical: 2.5,
                    anchor: WatermarkAnchor::BottomRight,
                    rotation: WatermarkRotation::Deg0,
                },
            },
        ];

        // Auto-discover user's existing Adobe Lightroom Classic .lrtemplate watermarks!
        let home = std::env::var("HOME").unwrap_or_default();
        let lr_wm_dir = std::path::PathBuf::from(home)
            .join("Library/Application Support/Adobe/Lightroom/Watermarks");
        if lr_wm_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(lr_wm_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("lrtemplate") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Some(p) = parse_lrtemplate_watermark(&content) {
                                // Add if not already present
                                if !list.iter().any(|existing| existing.name == p.name) {
                                    list.push(p);
                                }
                            }
                        }
                    }
                }
            }
        }

        list
    }
}

pub fn parse_lrtemplate_watermark(content: &str) -> Option<WatermarkPreset> {
    let mut title = String::new();
    let mut image_path = None;
    let mut kind = "text".to_string();
    let mut text = "© Moestret Photography".to_string();
    let mut anchor = WatermarkAnchor::BottomCenter;
    let mut size_relative = 26.0_f32;
    let mut opacity = 100.0_f32;
    let mut inset_h = 0.0_f32;
    let mut inset_v = 0.0_f32;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("title =") {
            if let Some(val) = extract_quoted_value(trimmed) {
                title = val;
            }
        } else if trimmed.starts_with("imagePath =") {
            if let Some(val) = extract_quoted_value(trimmed) {
                image_path = Some(val);
            }
        } else if trimmed.starts_with("kind =") {
            if let Some(val) = extract_quoted_value(trimmed) {
                kind = val;
            }
        } else if trimmed.starts_with("text =") {
            if let Some(val) = extract_quoted_value(trimmed) {
                text = val;
            }
        } else if trimmed.starts_with("anchor =") {
            if let Some(val) = extract_quoted_value(trimmed) {
                anchor = match val.to_lowercase().as_str() {
                    "top-left" | "topleft" => WatermarkAnchor::TopLeft,
                    "top" | "topcenter" => WatermarkAnchor::TopCenter,
                    "top-right" | "topright" => WatermarkAnchor::TopRight,
                    "left" | "centerleft" => WatermarkAnchor::CenterLeft,
                    "center" => WatermarkAnchor::Center,
                    "right" | "centerright" => WatermarkAnchor::CenterRight,
                    "bottom-left" | "bottomleft" => WatermarkAnchor::BottomLeft,
                    "bottom" | "bottomcenter" => WatermarkAnchor::BottomCenter,
                    "bottom-right" | "bottomright" => WatermarkAnchor::BottomRight,
                    _ => WatermarkAnchor::BottomCenter,
                };
            }
        } else if trimmed.starts_with("sizeRelativeAmount =") {
            if let Some(num_str) = trimmed.split('=').nth(1) {
                let clean = num_str.trim().trim_end_matches(',');
                if let Ok(val) = clean.parse::<f32>() {
                    size_relative = (val * 100.0).clamp(1.0, 100.0);
                }
            }
        } else if trimmed.starts_with("opacity =") {
            if let Some(num_str) = trimmed.split('=').nth(1) {
                let clean = num_str.trim().trim_end_matches(',');
                if let Ok(val) = clean.parse::<f32>() {
                    opacity = (val * 100.0).clamp(0.0, 100.0);
                }
            }
        } else if trimmed.starts_with("horizontalPadding =") {
            if let Some(num_str) = trimmed.split('=').nth(1) {
                let clean = num_str.trim().trim_end_matches(',');
                if let Ok(val) = clean.parse::<f32>() {
                    inset_h = (val * 100.0).clamp(0.0, 100.0);
                }
            }
        } else if trimmed.starts_with("verticalPadding =") {
            if let Some(num_str) = trimmed.split('=').nth(1) {
                let clean = num_str.trim().trim_end_matches(',');
                if let Ok(val) = clean.parse::<f32>() {
                    inset_v = (val * 100.0).clamp(0.0, 100.0);
                }
            }
        }
    }

    if title.is_empty() {
        return None;
    }

    let style = if kind == "image" || image_path.is_some() {
        WatermarkStyle::Graphic
    } else {
        WatermarkStyle::Text
    };

    Some(WatermarkPreset {
        name: title,
        settings: WatermarkSettings {
            style,
            graphic_path: image_path,
            text,
            font_family: "Adobe Clean".into(),
            font_style: "Regular".into(),
            text_align: WatermarkTextAlign::Center,
            text_color: [255, 255, 255],
            shadow_enabled: true,
            shadow_opacity: 80.0,
            shadow_offset: 10.0,
            shadow_radius: 20.0,
            shadow_angle: -90.0,
            opacity,
            size_mode: WatermarkSizeMode::Proportional,
            proportional_size: size_relative,
            inset_horizontal: inset_h,
            inset_vertical: inset_v,
            anchor,
            rotation: WatermarkRotation::Deg0,
        },
    })
}

fn extract_quoted_value(line: &str) -> Option<String> {
    let start = line.find('"')?;
    let end = line[start + 1..].find('"')?;
    Some(line[start + 1..start + 1 + end].to_string())
}

// ============================================================================
// Compositing Engine: Full Sensor & Export Buffer Rendering
// ============================================================================

/// Composites the configured watermark onto target RGBA buffer with full alpha channel blending,
/// drop shadow, rotation, and proportional/fit/fill positioning.
pub fn composite_watermark_rgba(
    target_rgba: &mut [u8],
    target_w: u32,
    target_h: u32,
    settings: &WatermarkSettings,
) {
    if target_w < 32 || target_h < 32 {
        return;
    }

    match settings.style {
        WatermarkStyle::Graphic => {
            if let Some(ref path_str) = settings.graphic_path {
                composite_graphic_watermark_rgba(target_rgba, target_w, target_h, settings, path_str);
            }
        }
        WatermarkStyle::Text => {
            composite_text_watermark_rgba(target_rgba, target_w, target_h, settings);
        }
    }
}

fn composite_graphic_watermark_rgba(
    target_rgba: &mut [u8],
    target_w: u32,
    target_h: u32,
    settings: &WatermarkSettings,
    path_str: &str,
) {
    let p = Path::new(path_str);
    if !p.exists() {
        return;
    }

    let Ok(img) = image::open(p) else {
        return;
    };
    let mut rgba_img = img.to_rgba8();

    // 1. Rotation (90, 180, 270)
    match settings.rotation {
        WatermarkRotation::Deg0 => {}
        WatermarkRotation::Deg90 => {
            rgba_img = image::imageops::rotate90(&rgba_img);
        }
        WatermarkRotation::Deg180 => {
            rgba_img = image::imageops::rotate180(&rgba_img);
        }
        WatermarkRotation::Deg270 => {
            rgba_img = image::imageops::rotate270(&rgba_img);
        }
    }

    let (src_w, src_h) = (rgba_img.width(), rgba_img.height());
    if src_w == 0 || src_h == 0 {
        return;
    }

    // 2. Size calculation
    let (final_w, final_h) = match settings.size_mode {
        WatermarkSizeMode::Proportional => {
            let pct = (settings.proportional_size / 100.0).clamp(0.02, 1.0);
            let w = (target_w as f32 * pct).max(16.0);
            let h = w * (src_h as f32 / src_w as f32);
            (w as u32, h as u32)
        }
        WatermarkSizeMode::Fit => {
            let max_w = target_w as f32 * 0.95;
            let max_h = target_h as f32 * 0.95;
            let scale = (max_w / src_w as f32).min(max_h / src_h as f32);
            (
                (src_w as f32 * scale).max(16.0) as u32,
                (src_h as f32 * scale).max(16.0) as u32,
            )
        }
        WatermarkSizeMode::Fill => {
            let scale = (target_w as f32 / src_w as f32).max(target_h as f32 / src_h as f32);
            (
                (src_w as f32 * scale).max(16.0) as u32,
                (src_h as f32 * scale).max(16.0) as u32,
            )
        }
    };

    let resized = if final_w != src_w || final_h != src_h {
        image::imageops::resize(&rgba_img, final_w, final_h, image::imageops::FilterType::Triangle)
    } else {
        rgba_img
    };

    // 3. Anchor & Inset Positioning
    let inset_x = ((settings.inset_horizontal / 100.0) * target_w as f32) as i32;
    let inset_y = ((settings.inset_vertical / 100.0) * target_h as f32) as i32;

    let (pos_x, pos_y) = calculate_anchor_coords(
        settings.anchor,
        target_w as i32,
        target_h as i32,
        final_w as i32,
        final_h as i32,
        inset_x,
        inset_y,
    );

    // 4. Alpha Blending
    let global_opacity = (settings.opacity / 100.0).clamp(0.0, 1.0);
    blend_rgba_buffer(target_rgba, target_w, target_h, &resized, pos_x, pos_y, global_opacity);
}

fn composite_text_watermark_rgba(
    target_rgba: &mut [u8],
    target_w: u32,
    target_h: u32,
    settings: &WatermarkSettings,
) {
    let trimmed = settings.text.trim();
    if trimmed.is_empty() {
        return;
    }

    let char_count = trimmed.chars().count().max(1) as i32;
    // Scale proportional to target width
    let pct = (settings.proportional_size / 100.0).clamp(0.05, 1.0);
    let desired_text_w = target_w as f32 * pct;
    let scale = ((desired_text_w / (char_count as f32 * 6.0)).round() as i32).clamp(1, 40);

    let text_w = char_count * 6 * scale;
    let text_h = 7 * scale;

    let inset_x = ((settings.inset_horizontal / 100.0) * target_w as f32) as i32;
    let inset_y = ((settings.inset_vertical / 100.0) * target_h as f32) as i32;

    let (base_x, base_y) = calculate_anchor_coords(
        settings.anchor,
        target_w as i32,
        target_h as i32,
        text_w,
        text_h,
        inset_x,
        inset_y,
    );

    let global_opacity = (settings.opacity / 100.0).clamp(0.0, 1.0);

    // Render Shadow if enabled
    if settings.shadow_enabled {
        let shadow_alpha = (settings.shadow_opacity / 100.0 * global_opacity * 255.0).clamp(0.0, 255.0) as u8;
        let angle_rad = settings.shadow_angle.to_radians();
        let dist = (settings.shadow_offset / 100.0 * 24.0 * scale as f32 * 0.15).max(1.0);
        let s_dx = (dist * angle_rad.cos()).round() as i32;
        let s_dy = (-dist * angle_rad.sin()).round() as i32;

        canvas_draw_text_rgba(
            target_rgba,
            target_w,
            target_h,
            base_x + s_dx,
            base_y + s_dy,
            trimmed,
            scale,
            [0, 0, 0, shadow_alpha],
        );
    }

    // Render Foreground Text
    let fg_alpha = (global_opacity * 255.0).clamp(0.0, 255.0) as u8;
    let color_rgba = [
        settings.text_color[0],
        settings.text_color[1],
        settings.text_color[2],
        fg_alpha,
    ];

    canvas_draw_text_rgba(
        target_rgba,
        target_w,
        target_h,
        base_x,
        base_y,
        trimmed,
        scale,
        color_rgba,
    );
}

fn glyph_5x7(ch: char) -> [u8; 7] {
    match ch.to_ascii_uppercase() {
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        'C' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
        'D' => [0b11100, 0b10010, 0b10001, 0b10001, 0b10001, 0b10010, 0b11100],
        'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'G' => [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110],
        'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'I' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'J' => [0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100],
        'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001],
        'X' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
        'Y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
        '3' => [0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => [0b01110, 0b10000, 0b11110, 0b10001, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
        '-' => [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
        '_' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111],
        '.' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100],
        ':' => [0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b01100, 0b00000],
        '/' => [0b00001, 0b00010, 0b00100, 0b00100, 0b01000, 0b10000, 0b00000],
        '(' => [0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010],
        ')' => [0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000],
        '©' => [0b01110, 0b10001, 0b10111, 0b10100, 0b10111, 0b10001, 0b01110],
        '•' | '|' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        _ => [0; 7],
    }
}

/// Renders crisp scaled 5x7 ASCII bitmap text onto an RGBA buffer with alpha compositing.
pub fn draw_bitmap_text_rgba(
    rgba: &mut [u8],
    w: u32,
    h: u32,
    x0: i32,
    y0: i32,
    text: &str,
    scale: i32,
    color: [u8; 4],
) {
    let sc = scale.max(1);
    let mut cx = x0;
    for ch in text.chars() {
        let rows = glyph_5x7(ch);
        for (ry, row_bits) in rows.iter().enumerate() {
            for rx in 0..5 {
                if (row_bits >> (4 - rx)) & 1 == 1 {
                    for sy in 0..sc {
                        for sx in 0..sc {
                            let px = cx + rx * sc + sx;
                            let py = y0 + (ry as i32) * sc + sy;
                            if px >= 0 && py >= 0 && (px as u32) < w && (py as u32) < h {
                                let idx = ((py as u32 * w + px as u32) * 4) as usize;
                                let a = color[3] as f32 / 255.0;
                                rgba[idx] =
                                    (rgba[idx] as f32 * (1.0 - a) + color[0] as f32 * a) as u8;
                                rgba[idx + 1] =
                                    (rgba[idx + 1] as f32 * (1.0 - a) + color[1] as f32 * a) as u8;
                                rgba[idx + 2] =
                                    (rgba[idx + 2] as f32 * (1.0 - a) + color[2] as f32 * a) as u8;
                                rgba[idx + 3] = 255;
                            }
                        }
                    }
                }
            }
        }
        cx += 6 * sc;
    }
}

#[inline]
fn canvas_draw_text_rgba(
    rgba: &mut [u8],
    w: u32,
    h: u32,
    x0: i32,
    y0: i32,
    text: &str,
    scale: i32,
    color: [u8; 4],
) {
    draw_bitmap_text_rgba(rgba, w, h, x0, y0, text, scale, color);
}

fn calculate_anchor_coords(
    anchor: WatermarkAnchor,
    dst_w: i32,
    dst_h: i32,
    src_w: i32,
    src_h: i32,
    inset_x: i32,
    inset_y: i32,
) -> (i32, i32) {
    match anchor {
        WatermarkAnchor::TopLeft => (inset_x, inset_y),
        WatermarkAnchor::TopCenter => ((dst_w - src_w) / 2, inset_y),
        WatermarkAnchor::TopRight => (dst_w - src_w - inset_x, inset_y),
        WatermarkAnchor::CenterLeft => (inset_x, (dst_h - src_h) / 2),
        WatermarkAnchor::Center => ((dst_w - src_w) / 2, (dst_h - src_h) / 2),
        WatermarkAnchor::CenterRight => (dst_w - src_w - inset_x, (dst_h - src_h) / 2),
        WatermarkAnchor::BottomLeft => (inset_x, dst_h - src_h - inset_y),
        WatermarkAnchor::BottomCenter => ((dst_w - src_w) / 2, dst_h - src_h - inset_y),
        WatermarkAnchor::BottomRight => (dst_w - src_w - inset_x, dst_h - src_h - inset_y),
    }
}

fn blend_rgba_buffer(
    target_rgba: &mut [u8],
    target_w: u32,
    target_h: u32,
    src_img: &image::RgbaImage,
    start_x: i32,
    start_y: i32,
    global_opacity: f32,
) {
    let (sw, sh) = (src_img.width(), src_img.height());
    for sy in 0..sh {
        let dy = start_y + sy as i32;
        if dy < 0 || (dy as u32) >= target_h {
            continue;
        }
        for sx in 0..sw {
            let dx = start_x + sx as i32;
            if dx < 0 || (dx as u32) >= target_w {
                continue;
            }

            let spx = src_img.get_pixel(sx, sy);
            let src_a = (spx[3] as f32 / 255.0) * global_opacity;
            if src_a <= 0.001 {
                continue;
            }

            let tidx = ((dy as u32 * target_w + dx as u32) * 4) as usize;
            let inv_a = 1.0 - src_a;

            target_rgba[tidx] = (target_rgba[tidx] as f32 * inv_a + spx[0] as f32 * src_a) as u8;
            target_rgba[tidx + 1] = (target_rgba[tidx + 1] as f32 * inv_a + spx[1] as f32 * src_a) as u8;
            target_rgba[tidx + 2] = (target_rgba[tidx + 2] as f32 * inv_a + spx[2] as f32 * src_a) as u8;
            target_rgba[tidx + 3] = 255;
        }
    }
}
