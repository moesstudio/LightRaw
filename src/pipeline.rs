use crate::params::{
    BokehShape, CameraProfile, DevelopParams, LandscapePart, LocalAdjustParams, MaskGroup, MaskOp,
    MaskPrimitiveKind, PersonPart, RedEyeMode, RetouchMode, UprightMode, VignetteStyle,
    WhiteBalanceMode,
};
use rayon::prelude::*;

#[derive(Clone)]
pub struct RawSourceImage {
    pub width: u32,
    pub height: u32,
    pub native_width: u32,
    pub native_height: u32,
    /// Unclipped 32-bit linear extended sRGB pixels in `[0.0 .. ∞)`, length = `width * height * 3`
    pub rgb: Vec<f32>,
    pub as_shot_temp: f32,
    pub as_shot_tint: f32,
    pub baseline_ev: f32,
    pub is_scene_referred: bool,
    sem_w: u32,
    sem_h: u32,
    depth_q: Vec<u8>,
    subject_q: Vec<u8>,
    sky_q: Vec<u8>,
    people_q: Vec<u8>,
    pub face_parts_q: Option<Vec<u8>>,
}

#[derive(Clone)]
pub struct RenderedFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub hist_r: [u32; 256],
    pub hist_g: [u32; 256],
    pub hist_b: [u32; 256],
    pub hist_lum: [u32; 256],
    pub shadow_clipped_pct: f32,
    pub highlight_clipped_pct: f32,
    pub mean_lum: f32,
    pub p5_lum: f32,
    pub p95_lum: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderOptions {
    pub show_shadow_clipping: bool,
    pub show_highlight_clipping: bool,
    pub soft_proofing: bool,
    pub gamut_warning: bool,
    pub active_mask_overlay: Option<usize>,
    pub apply_crop: bool,
    pub show_sharpen_mask_preview: bool,
    pub soft_proof_profile: u8, // 0=sRGB, 1=AdobeRGB, 2=DisplayP3, 3=ProPhoto, 4=Matte Paper
    pub soft_proof_perceptual: bool,
    pub simulate_paper_ink: bool,
    pub is_draft_interactive: bool,
}

#[inline]
pub fn srgb_to_linear(c: f32) -> f32 {
    let x = c.max(0.0);
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

#[inline]
pub fn linear_to_srgb(c: f32) -> f32 {
    let x = c.max(0.0);
    if x <= 0.0031308 {
        x * 12.92
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    }
}

/// Computes CIE 1960 UCS Planckian + Duv Isotherm (`duv = tint / 3000`) -> CIE XYZ (Y = 1)
fn cct_tint_to_xyz(temp_k: f32, tint: f32) -> [f64; 3] {
    let calc_uv = |tk: f64| -> (f64, f64) {
        let t = tk.clamp(1667.0, 50000.0);
        let t2 = t * t;
        let u = (0.860117757 + 1.54118254e-4 * t + 1.28641212e-7 * t2)
            / (1.0 + 8.42420235e-4 * t + 7.08145163e-7 * t2);
        let v = (0.317398726 + 4.22806245e-5 * t + 4.20481691e-8 * t2)
            / (1.0 - 2.89741816e-5 * t + 1.61456053e-7 * t2);
        (u, v)
    };
    let t0 = (temp_k as f64).clamp(1667.0, 50000.0);
    let (u0, v0) = calc_uv(t0);
    let (u1, v1) = calc_uv(t0 + 10.0);
    let du = u1 - u0;
    let dv = v1 - v0;
    let len = (du * du + dv * dv).sqrt().max(1e-12);
    // Unit normal pointing toward +v (above Planckian locus = greenish illuminant)
    let nu = dv / len;
    let nv = -du / len;
    let duv = (tint as f64).clamp(-150.0, 150.0) / 3000.0;
    let u = u0 + duv * nu;
    let v = (v0 + duv * nv).max(1e-5);

    // CIE 1960 (u, v) -> CIE 1931 (x, y) -> XYZ (Y = 1)
    let denom = (2.0 * u - 8.0 * v + 4.0).max(1e-6);
    let x = (3.0 * u) / denom;
    let y = (2.0 * v) / denom;
    let y_safe = y.max(1e-5);
    [x / y_safe, 1.0, (1.0 - x - y).max(0.0) / y_safe]
}

/// Computes the 3x3 linear sRGB Bradford Chromatic Adaptation Matrix adapting from
/// `(src_temp, src_tint)` (camera As-Shot WB) to `(dst_temp, dst_tint)` (user WB).
pub fn compute_bradford_wb_matrix(
    src_temp: f32,
    src_tint: f32,
    dst_temp: f32,
    dst_tint: f32,
) -> [[f32; 3]; 3] {
    if (src_temp - dst_temp).abs() < 0.5 && (src_tint - dst_tint).abs() < 0.05 {
        return [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    }

    // sRGB (D65) <-> XYZ & Bradford M / M^-1
    const RGB_TO_XYZ: [[f64; 3]; 3] = [
        [0.4124564, 0.3575761, 0.1804375],
        [0.2126729, 0.7151522, 0.0721750],
        [0.0193339, 0.1191920, 0.9503041],
    ];
    const XYZ_TO_RGB: [[f64; 3]; 3] = [
        [3.2404542, -1.5371385, -0.4985314],
        [-0.9692660, 1.8760108, 0.0415560],
        [0.0556434, -0.2040259, 1.0572252],
    ];
    const BRADFORD: [[f64; 3]; 3] = [
        [0.8951, 0.2664, -0.1614],
        [-0.7502, 1.7135, 0.0367],
        [0.0389, -0.0685, 1.0296],
    ];
    const BRADFORD_INV: [[f64; 3]; 3] = [
        [0.9869929, -0.1470543, 0.1599627],
        [0.4323053, 0.5183603, 0.0492912],
        [-0.0085287, 0.0400428, 0.9684867],
    ];

    let mul_mv = |m: &[[f64; 3]; 3], v: &[f64; 3]| -> [f64; 3] {
        [
            m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
            m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
            m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
        ]
    };
    let mul_mm = |a: &[[f64; 3]; 3], b: &[[f64; 3]; 3]| -> [[f64; 3]; 3] {
        let mut out = [[0.0; 3]; 3];
        for r in 0..3 {
            for c in 0..3 {
                out[r][c] = a[r][0] * b[0][c] + a[r][1] * b[1][c] + a[r][2] * b[2][c];
            }
        }
        out
    };

    let xyz_as = cct_tint_to_xyz(src_temp, src_tint);
    let xyz_dst = cct_tint_to_xyz(dst_temp, dst_tint);
    let lms_as = mul_mv(&BRADFORD, &xyz_as);
    let lms_dst = mul_mv(&BRADFORD, &xyz_dst);

    // Moving Temp slider lower than As-Shot cools the scene (reduces Red, boosts Blue):
    // scale = LMS(as_shot) / LMS(dst)
    let diag = [
        [(lms_as[0] / lms_dst[0].max(1e-6)).clamp(0.15, 6.0), 0.0, 0.0],
        [0.0, (lms_as[1] / lms_dst[1].max(1e-6)).clamp(0.15, 6.0), 0.0],
        [0.0, 0.0, (lms_as[2] / lms_dst[2].max(1e-6)).clamp(0.15, 6.0)],
    ];

    let adapt_xyz = mul_mm(&BRADFORD_INV, &mul_mm(&diag, &BRADFORD));
    let mut m = mul_mm(&XYZ_TO_RGB, &mul_mm(&adapt_xyz, &RGB_TO_XYZ));

    // Preserve photopic luminance Y of a D65 neutral gray (1, 1, 1) so WB changes chromaticity cleanly
    let w_rgb = mul_mv(&m, &[1.0, 1.0, 1.0]);
    let y_after = (0.2126729 * w_rgb[0] + 0.7151522 * w_rgb[1] + 0.0721750 * w_rgb[2]).max(0.2);
    let norm = 1.0 / y_after;
    for r in 0..3 {
        for c in 0..3 {
            m[r][c] *= norm;
        }
    }

    [
        [m[0][0] as f32, m[0][1] as f32, m[0][2] as f32],
        [m[1][0] as f32, m[1][1] as f32, m[1][2] as f32],
        [m[2][0] as f32, m[2][1] as f32, m[2][2] as f32],
    ]
}

/// Estimates Auto White Balance `(Temperature, Tint)` from unclipped linear pixels (Gray-World / Shades-of-Gray)
pub fn estimate_auto_wb(src: &RawSourceImage) -> (f32, f32) {
    let mut sum_r = 0.0f64;
    let mut sum_g = 0.0f64;
    let mut sum_b = 0.0f64;
    let mut count = 0u64;
    let step = ((src.width * src.height) as usize / 8000).max(1);
    for px in src.rgb.chunks_exact(3).step_by(step) {
        let (r, g, b) = (px[0], px[1], px[2]);
        let mx = r.max(g).max(b);
        let mn = r.min(g).min(b);
        if mx > 0.015 && mx < 0.88 && mn > 0.002 {
            // Weight muted/midtone regions higher so solid colored backdrops don't skew AWB excessively
            let sat = (mx - mn) / mx.max(1e-4);
            let w = (1.0 - sat * 0.75).max(0.12) as f64;
            sum_r += r as f64 * w;
            sum_g += g as f64 * w;
            sum_b += b as f64 * w;
            count += 1;
        }
    }
    if count < 16 || sum_g < 1e-6 {
        return (src.as_shot_temp, src.as_shot_tint);
    }
    let rg = (sum_r / sum_g).clamp(0.4, 2.5);
    let bg = (sum_b / sum_g).clamp(0.4, 2.5);
    // Search CCT/Tint that best neutralizes `(rg, bg)` toward `(1, 1)`
    let mut best_t = src.as_shot_temp;
    let mut best_ti = src.as_shot_tint;
    let mut best_err = f64::MAX;
    for tk in (2400..=9500).step_by(200) {
        for ti in (-45..=45).step_by(6) {
            let m = compute_bradford_wb_matrix(src.as_shot_temp, src.as_shot_tint, tk as f32, ti as f32);
            let nr = m[0][0] as f64 * rg + m[0][1] as f64 + m[0][2] as f64 * bg;
            let ng = m[1][0] as f64 * rg + m[1][1] as f64 + m[1][2] as f64 * bg;
            let nb = m[2][0] as f64 * rg + m[2][1] as f64 + m[2][2] as f64 * bg;
            if ng > 0.05 {
                let err = (nr / ng - 1.0).powi(2) + (nb / ng - 1.0).powi(2);
                if err < best_err {
                    best_err = err;
                    best_t = tk as f32;
                    best_ti = ti as f32;
                }
            }
        }
    }
    // Damp 65% toward estimated neutral so studio backdrops preserve natural ambiance
    let final_t = (src.as_shot_temp * 0.35 + best_t * 0.65).round();
    let final_ti = (src.as_shot_tint * 0.35 + best_ti * 0.65).round();
    (final_t.clamp(2000.0, 50000.0), final_ti.clamp(-150.0, 150.0))
}

impl RawSourceImage {
    pub fn from_linear_rgb(
        width: u32,
        height: u32,
        native_width: u32,
        native_height: u32,
        rgb: Vec<f32>,
        as_shot_temp: f32,
        as_shot_tint: f32,
        baseline_ev: f32,
        is_scene_referred: bool,
    ) -> Self {
        let sem_w = width.min(320).max(1);
        let sem_h = height.min(320).max(1);

        // Hardware-Accelerated Local AI Semantic Extraction (Apple Neural Engine / Metal / SIMD)
        let maps = crate::ai::compute_ai_semantics(
            &rgb,
            width,
            height,
            sem_w,
            sem_h,
            crate::ai::AiHardwareTier::Auto,
            None,
        );

        Self {
            width,
            height,
            native_width: native_width.max(width),
            native_height: native_height.max(height),
            rgb,
            as_shot_temp: if as_shot_temp > 1000.0 { as_shot_temp } else { 5500.0 },
            as_shot_tint: as_shot_tint.clamp(-150.0, 150.0),
            baseline_ev,
            is_scene_referred,
            sem_w,
            sem_h,
            depth_q: maps.depth_q,
            subject_q: maps.subject_q,
            sky_q: maps.sky_q,
            people_q: maps.people_q,
            face_parts_q: maps.face_parts_q,
        }
    }

    #[inline]
    pub fn sample_face_part(&self, nx: f32, ny: f32) -> u8 {
        if let Some(parts) = &self.face_parts_q {
            let sx = (nx.clamp(0.0, 1.0) * (self.sem_w.saturating_sub(1)) as f32) as usize;
            let sy = (ny.clamp(0.0, 1.0) * (self.sem_h.saturating_sub(1)) as f32) as usize;
            let idx = (sy * self.sem_w as usize + sx).min(parts.len().saturating_sub(1));
            parts[idx]
        } else {
            0
        }
    }

    pub fn from_rgba8(width: u32, height: u32, rgba: &[u8]) -> Self {
        let n = (width * height) as usize;
        let mut rgb = vec![0.0f32; n * 3];
        for i in 0..n {
            rgb[i * 3] = srgb_to_linear(rgba[i * 4] as f32 / 255.0);
            rgb[i * 3 + 1] = srgb_to_linear(rgba[i * 4 + 1] as f32 / 255.0);
            rgb[i * 3 + 2] = srgb_to_linear(rgba[i * 4 + 2] as f32 / 255.0);
        }
        Self::from_linear_rgb(width, height, width, height, rgb, 5500.0, 0.0, 0.0, false)
    }

    #[inline]
    pub fn sample_semantics(&self, nx: f32, ny: f32) -> (f32, f32, f32, f32) {
        let sx = (nx.clamp(0.0, 1.0) * (self.sem_w.saturating_sub(1)) as f32) as usize;
        let sy = (ny.clamp(0.0, 1.0) * (self.sem_h.saturating_sub(1)) as f32) as usize;
        let idx = (sy * self.sem_w as usize + sx).min(self.depth_q.len().saturating_sub(1));
        (
            self.depth_q[idx] as f32 / 255.0,
            self.subject_q[idx] as f32 / 255.0,
            self.sky_q[idx] as f32 / 255.0,
            self.people_q[idx] as f32 / 255.0,
        )
    }

    #[inline]
    pub fn sample_depth(&self, nx: f32, ny: f32) -> f32 {
        self.sample_semantics(nx, ny).0
    }

    pub fn generate_sample_raw(scene_id: usize, width: u32, height: u32) -> Self {
        let mut rgba = vec![0u8; (width * height * 4) as usize];
        for y in 0..height {
            let ny = y as f32 / height.max(1) as f32;
            for x in 0..width {
                let nx = x as f32 / width.max(1) as f32;
                let idx = ((y * width + x) * 4) as usize;
                let (r, g, b) = match scene_id % 4 {
                    0 => {
                        let sky = (1.0 - (ny / 0.52).clamp(0.0, 1.0)).powf(0.85);
                        if ny < 0.48 {
                            (0.28 + 0.55 * (1.0 - sky), 0.38 + 0.35 * (1.0 - sky), 0.65 + 0.25 * sky)
                        } else {
                            (0.18 + 0.25 * (1.0 - ny), 0.34 + 0.22 * (1.0 - ny), 0.42 + 0.18 * (1.0 - ny))
                        }
                    }
                    1 => {
                        let face_d = ((nx - 0.5).powi(2) * 2.5 + (ny - 0.46).powi(2) * 1.45).sqrt();
                        if face_d < 0.24 {
                            (0.72, 0.54, 0.44)
                        } else {
                            (0.18, 0.24, 0.34)
                        }
                    }
                    2 => (0.12 + 0.55 * nx, 0.22 + 0.45 * (1.0 - ny), 0.48 + 0.35 * ny),
                    _ => (0.35 + 0.25 * (nx * 12.0).sin().abs(), 0.38, 0.34),
                };
                rgba[idx] = (r.clamp(0.0, 1.0) * 255.0) as u8;
                rgba[idx + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
                rgba[idx + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
                rgba[idx + 3] = 255;
            }
        }
        Self::from_rgba8(width, height, &rgba)
    }
}

#[inline]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0).max(1e-5)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[inline]
pub fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) * 0.5;
    if (max - min).abs() < 1e-6 {
        return (0.0, 0.0, l.clamp(0.0, 1.0));
    }
    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min).max(1e-5)
    } else {
        d / (max + min).max(1e-5)
    };
    let mut h = if (max - r).abs() < 1e-6 {
        (g - b) / d + (if g < b { 6.0 } else { 0.0 })
    } else if (max - g).abs() < 1e-6 {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    h *= 60.0;
    (h.rem_euclid(360.0), s.clamp(0.0, 1.0), l.clamp(0.0, 1.0))
}

#[inline]
pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s <= 1e-5 {
        return (l, l, l);
    }
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let hk = h.rem_euclid(360.0) / 360.0;
    let hue2rgb = |mut t: f32| -> f32 {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };
    (
        hue2rgb(hk + 1.0 / 3.0),
        hue2rgb(hk),
        hue2rgb(hk - 1.0 / 3.0),
    )
}

/// Builds a 256-entry Tone Curve LUT in perceptual sRGB domain
pub fn build_tone_curve_lut(
    points: &[(f32, f32)],
    param_shadows: f32,
    param_darks: f32,
    param_lights: f32,
    param_highlights: f32,
    shadow_split: f32,
    midtone_split: f32,
    highlight_split: f32,
    include_parametric: bool,
) -> [f32; 256] {
    let mut lut = [0.0f32; 256];
    let s_split = (shadow_split / 100.0).clamp(0.1, 0.7);
    let m_split = (midtone_split / 100.0).clamp(0.2, 0.8);
    let h_split = (highlight_split / 100.0).clamp(0.3, 0.9);

    for i in 0..256 {
        let mut x = i as f32 / 255.0;

        if include_parametric
            && (param_shadows.abs() > 0.01
                || param_darks.abs() > 0.01
                || param_lights.abs() > 0.01
                || param_highlights.abs() > 0.01)
        {
            let w_sh = (1.0 - (x / s_split).clamp(0.0, 1.0)).powi(2) * 4.0 * x * (1.0 - x);
            let w_dk = (-((x - s_split * 0.9) / (m_split - s_split).max(0.1)).powi(2) * 2.5).exp()
                * 4.0
                * x
                * (1.0 - x);
            let w_lt = (-((x - (m_split + h_split) * 0.5) / (h_split - m_split).max(0.1)).powi(2)
                * 2.5)
                .exp()
                * 4.0
                * x
                * (1.0 - x);
            let w_hi = (((x - h_split) / (1.0 - h_split).max(0.1)).clamp(0.0, 1.0)).powi(2)
                * 4.0
                * x
                * (1.0 - x);

            x += (param_shadows * 0.0025 * w_sh)
                + (param_darks * 0.0025 * w_dk)
                + (param_lights * 0.0025 * w_lt)
                + (param_highlights * 0.0025 * w_hi);
            x = x.clamp(0.0, 1.0);
        }

        let y = evaluate_point_curve(points, x);
        lut[i] = y.clamp(0.0, 1.0);
    }
    lut
}

pub fn evaluate_point_curve(points: &[(f32, f32)], x: f32) -> f32 {
    if points.is_empty() {
        return x;
    }
    if x <= points[0].0 {
        return points[0].1;
    }
    if x >= points[points.len() - 1].0 {
        return points[points.len() - 1].1;
    }
    for w in points.windows(2) {
        let (x0, y0) = w[0];
        let (x1, y1) = w[1];
        if x >= x0 && x <= x1 {
            let t = (x - x0) / (x1 - x0).max(1e-5);
            let st = t * t * (3.0 - 2.0 * t);
            let blend = 0.65 * t + 0.35 * st;
            return y0 + (y1 - y0) * blend;
        }
    }
    x
}

#[inline]
fn sample_lut(lut: &[f32; 256], v: f32) -> f32 {
    let scaled = (v.clamp(0.0, 1.0) * 255.0).clamp(0.0, 255.0);
    let i0 = scaled.floor() as usize;
    let i1 = (i0 + 1).min(255);
    let frac = scaled - i0 as f32;
    lut[i0] * (1.0 - frac) + lut[i1] * frac
}

/// Evaluates a `MaskGroup` weight at normalized coordinate `(nx, ny)` with perceptual sRGB `(r, g, b)`
pub fn evaluate_mask_group(
    group: &MaskGroup,
    nx: f32,
    ny: f32,
    r: f32,
    g: f32,
    b: f32,
    depth: f32,
    subject_prob: f32,
    sky_prob: f32,
    people_prob: f32,
    face_part: u8,
) -> f32 {
    if !group.enabled || group.components.is_empty() {
        return 0.0;
    }
    let mut acc = 0.0f32;
    for (idx, comp) in group.components.iter().enumerate() {
        let mut w = match &comp.kind {
            MaskPrimitiveKind::AiSubject => subject_prob,
            MaskPrimitiveKind::AiSky => sky_prob,
            MaskPrimitiveKind::AiBackground => (1.0 - subject_prob).clamp(0.0, 1.0),
            MaskPrimitiveKind::AiPeople => people_prob,
            MaskPrimitiveKind::PersonFeature { part, .. } => {
                let hw_match = match part {
                    PersonPart::EntirePerson => face_part > 0,
                    PersonPart::FaceSkin => face_part == 1,
                    PersonPart::IrisPupil | PersonPart::EyeSclera => face_part == 2,
                    PersonPart::Eyebrows => face_part == 3,
                    PersonPart::Lips => face_part == 4,
                    PersonPart::Teeth => face_part == 5,
                    PersonPart::BodySkin | PersonPart::Hair | PersonPart::Clothes => face_part == 6,
                    PersonPart::FacialHair => face_part == 1 || face_part == 6,
                };
                if hw_match {
                    1.0
                } else if face_part > 0 && *part != PersonPart::EntirePerson {
                    0.0
                } else {
                    let (h, s, l) = rgb_to_hsl(r, g, b);
                    match part {
                        PersonPart::EntirePerson => people_prob,
                        PersonPart::FaceSkin | PersonPart::BodySkin => {
                            let skin = if (h <= 42.0 || h >= 335.0) && s >= 0.08 && s <= 0.78 { 1.0 } else { 0.15 };
                            (people_prob * skin).clamp(0.0, 1.0)
                        }
                        PersonPart::Lips => {
                            let lip = if (h <= 22.0 || h >= 330.0) && s >= 0.25 && l < 0.72 { 1.0 } else { 0.0 };
                            (people_prob * lip).clamp(0.0, 1.0)
                        }
                        PersonPart::Teeth | PersonPart::EyeSclera => {
                            let wh = if s < 0.22 && l > 0.55 { 1.0 } else { 0.0 };
                            (people_prob * wh).clamp(0.0, 1.0)
                        }
                        PersonPart::IrisPupil | PersonPart::Eyebrows | PersonPart::Hair | PersonPart::FacialHair => {
                            let dk = if l < 0.35 { 1.0 } else { 0.2 };
                            (people_prob * dk).clamp(0.0, 1.0)
                        }
                        PersonPart::Clothes => {
                            let skin = (h <= 42.0 || h >= 335.0) && s >= 0.10 && s <= 0.72;
                            if !skin && ny > 0.45 { subject_prob } else { 0.0 }
                        }
                    }
                }
            }
            MaskPrimitiveKind::LandscapeFeature(part) => {
                let (h, s, l) = rgb_to_hsl(r, g, b);
                match part {
                    LandscapePart::Sky => sky_prob,
                    LandscapePart::Vegetation => {
                        if h >= 65.0 && h <= 165.0 && s > 0.12 {
                            ((1.0 - sky_prob) * (0.4 + 0.6 * s)).clamp(0.0, 1.0)
                        } else {
                            0.0
                        }
                    }
                    LandscapePart::Water => {
                        if h >= 165.0 && h <= 245.0 && ny > 0.45 {
                            (1.0 - sky_prob).clamp(0.0, 1.0)
                        } else {
                            0.0
                        }
                    }
                    LandscapePart::Mountains => {
                        if ny > 0.25 && ny < 0.65 && sky_prob < 0.4 {
                            (1.0 - subject_prob * 0.5).clamp(0.0, 1.0)
                        } else {
                            0.0
                        }
                    }
                    LandscapePart::Architecture => {
                        if s < 0.28 && sky_prob < 0.3 && l > 0.12 && l < 0.88 {
                            0.75
                        } else {
                            0.0
                        }
                    }
                    LandscapePart::NaturalGround | LandscapePart::ArtificialGround => {
                        smoothstep(0.50, 0.72, ny) * (1.0 - subject_prob * 0.5)
                    }
                }
            }
            MaskPrimitiveKind::AiObjects {
                center_x,
                center_y,
                radius,
            } => {
                let d = ((nx - center_x).powi(2) + (ny - center_y).powi(2)).sqrt();
                let spatial = smoothstep(*radius, (*radius * 0.4).max(0.01), d);
                (spatial * (0.5 + 0.5 * subject_prob)).clamp(0.0, 1.0)
            }
            MaskPrimitiveKind::Brush {
                points,
                radius,
                feather,
                flow,
            } => {
                let mut best = 0.0f32;
                let r_outer = radius.max(0.005);
                let r_inner = (r_outer * (1.0 - feather * 0.008)).max(0.001);
                for &(px, py) in points {
                    let d = ((nx - px).powi(2) + (ny - py).powi(2)).sqrt();
                    if d < r_outer {
                        let v = smoothstep(r_outer, r_inner, d) * (flow * 0.01);
                        if v > best {
                            best = v;
                        }
                    }
                }
                best
            }
            MaskPrimitiveKind::LinearGradient { x1, y1, x2, y2 } => {
                let dx = x2 - x1;
                let dy = y2 - y1;
                let len2 = (dx * dx + dy * dy).max(1e-5);
                let t = ((nx - x1) * dx + (ny - y1) * dy) / len2;
                (1.0 - t).clamp(0.0, 1.0)
            }
            MaskPrimitiveKind::RadialGradient {
                cx,
                cy,
                rx,
                ry,
                feather,
                invert,
            } => {
                let nd = (((nx - cx) / rx.max(0.01)).powi(2) + ((ny - cy) / ry.max(0.01)).powi(2))
                    .sqrt();
                let inner = (1.0 - feather * 0.008).clamp(0.05, 0.99);
                let mut v = smoothstep(1.0, inner, nd);
                if *invert {
                    v = 1.0 - v;
                }
                v
            }
            MaskPrimitiveKind::ColorRange {
                target_hue,
                target_sat,
                tolerance,
            } => {
                let (h, s, _) = rgb_to_hsl(r, g, b);
                let mut dh = (h - target_hue).abs();
                if dh > 180.0 {
                    dh = 360.0 - dh;
                }
                let ds = (s - target_sat).abs() * 100.0;
                let dist = (dh * dh * 0.5 + ds * ds).sqrt();
                smoothstep(tolerance.max(5.0), (tolerance * 0.3).max(1.0), dist)
            }
            MaskPrimitiveKind::LuminanceRange {
                min_lum,
                max_lum,
                feather,
            } => {
                let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
                let f = (feather * 0.003).max(0.01);
                let w_in = smoothstep(min_lum - f, *min_lum, lum);
                let w_out = 1.0 - smoothstep(*max_lum, max_lum + f, lum);
                (w_in * w_out).clamp(0.0, 1.0)
            }
            MaskPrimitiveKind::DepthRange {
                min_depth,
                max_depth,
                feather,
            } => {
                let f = (feather * 0.003).max(0.01);
                let w_in = smoothstep(min_depth - f, *min_depth, depth);
                let w_out = 1.0 - smoothstep(*max_depth, max_depth + f, depth);
                (w_in * w_out).clamp(0.0, 1.0)
            }
        };

        if comp.invert {
            w = 1.0 - w;
        }
        if idx == 0 {
            acc = w;
        } else {
            match comp.op {
                MaskOp::Add => acc = (acc + w).clamp(0.0, 1.0),
                MaskOp::Subtract => acc = (acc - w).clamp(0.0, 1.0),
                MaskOp::Intersect => acc = (acc * w).clamp(0.0, 1.0),
            }
        }
    }
    if group.invert {
        1.0 - acc
    } else {
        acc
    }
}

#[inline]
fn apply_local_adjustments(
    mut r: f32,
    mut g: f32,
    mut b: f32,
    hp_r: f32,
    hp_g: f32,
    hp_b: f32,
    local: &LocalAdjustParams,
    weight: f32,
) -> (f32, f32, f32) {
    let w = (weight * (local.amount / 100.0)).clamp(0.0, 2.0);
    if w <= 1e-4 {
        return (r, g, b);
    }

    // Local Exposure (evaluated in linear space then converted back to perceptual)
    if local.exposure.abs() > 0.001 {
        let mult = 2.0f32.powf(local.exposure * w);
        r = linear_to_srgb(srgb_to_linear(r) * mult);
        g = linear_to_srgb(srgb_to_linear(g) * mult);
        b = linear_to_srgb(srgb_to_linear(b) * mult);
    }

    // Local Temperature & Tint
    if local.temperature.abs() > 0.1 || local.tint.abs() > 0.1 {
        let t = local.temperature * 0.0038 * w;
        let ti = local.tint * 0.0032 * w;
        r *= 1.0 + t + ti * 0.3;
        g *= 1.0 - ti * 0.65;
        b *= 1.0 - t + ti * 0.3;
    }

    // Local Detail: Sharpness, Noise, Texture, Clarity
    let detail_gain = (local.sharpness * 0.012 + local.texture * 0.009 + local.clarity * 0.006
        - local.noise * 0.009)
        * w;
    if detail_gain.abs() > 1e-4 {
        r += hp_r * detail_gain;
        g += hp_g * detail_gain;
        b += hp_b * detail_gain;
    }

    // Local Moire & Defringe chroma suppression
    if local.moire > 0.5 || local.defringe > 0.5 {
        let chroma_damp = ((local.moire + local.defringe) * 0.004 * w).clamp(0.0, 0.85);
        let avg = (r + g + b) * 0.3333;
        r = r * (1.0 - chroma_damp) + avg * chroma_damp;
        b = b * (1.0 - chroma_damp) + avg * chroma_damp;
    }

    // Local Highlights / Shadows / Whites / Blacks / Contrast / Dehaze
    let lum = (0.2126 * r + 0.7152 * g + 0.0722 * b).clamp(0.0, 1.0);
    if lum > 1e-4 {
        let shadow_w = (1.0 - lum).powi(2);
        let highlight_w = lum.powi(2);
        let delta = (local.shadows * 0.0032 * shadow_w
            + local.highlights * 0.0032 * highlight_w
            + local.whites * 0.0024 * lum
            + local.blacks * 0.0024 * (1.0 - lum)
            - local.dehaze * 0.0022 * (1.0 - lum * 0.5))
            * w;
        let scale = ((lum + delta).max(0.0) / lum).clamp(0.1, 4.0);
        r *= scale;
        g *= scale;
        b *= scale;
    }

    let mid_w = 4.0 * lum * (1.0 - lum);
    let c_total = (local.contrast + local.clarity * 0.55 + local.dehaze * 0.35) * 0.0055 * w;
    if c_total.abs() > 0.001 {
        r = 0.5 + (r - 0.5) * (1.0 + c_total * (0.5 + 0.5 * mid_w));
        g = 0.5 + (g - 0.5) * (1.0 + c_total * (0.5 + 0.5 * mid_w));
        b = 0.5 + (b - 0.5) * (1.0 + c_total * (0.5 + 0.5 * mid_w));
    }

    // Local Hue, Saturation & Color Swatch Tint (`tint_color_hue` + `tint_color_sat`)
    if local.hue.abs() > 0.1 || local.saturation.abs() > 0.1 || local.tint_color_sat > 0.1 {
        let (mut h, mut s, l) =
            rgb_to_hsl(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0));
        h = (h + local.hue * w).rem_euclid(360.0);
        s = (s * (1.0 + local.saturation * 0.01 * w)).clamp(0.0, 1.0);
        let (mut nr, mut ng, mut nb) = hsl_to_rgb(h, s, l);
        if local.tint_color_sat > 0.1 {
            let (tr, tg, tb) = hsl_to_rgb(local.tint_color_hue, 1.0, 0.5);
            let ta = (local.tint_color_sat * 0.006 * w).clamp(0.0, 1.0);
            nr += (tr - 0.5) * ta;
            ng += (tg - 0.5) * ta;
            nb += (tb - 0.5) * ta;
        }
        r = nr;
        g = ng;
        b = nb;
    }

    (r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
}

#[inline]
fn value_noise_2d(fx: f32, fy: f32, seed: u32) -> f32 {
    let x0 = fx.floor() as i32;
    let y0 = fy.floor() as i32;
    let tx = fx - x0 as f32;
    let ty = fy - y0 as f32;
    let hash = |x: i32, y: i32| -> f32 {
        let mut h = (x as u32)
            .wrapping_mul(374761393)
            .wrapping_add((y as u32).wrapping_mul(668265263))
            ^ seed.wrapping_mul(2246822519);
        h = (h ^ (h >> 13)).wrapping_mul(1274126177);
        ((h & 0xffff) as f32 / 32768.0) - 1.0
    };
    let sx = tx * tx * (3.0 - 2.0 * tx);
    let sy = ty * ty * (3.0 - 2.0 * ty);
    let n00 = hash(x0, y0);
    let n10 = hash(x0 + 1, y0);
    let n01 = hash(x0, y0 + 1);
    let n11 = hash(x0 + 1, y0 + 1);
    let nx0 = n00 + (n10 - n00) * sx;
    let nx1 = n01 + (n11 - n01) * sx;
    nx0 + (nx1 - nx0) * sy
}

/// Estimates Upright perspective corrections `(vert, horiz, rot_deg)` from image gradients & guided lines
fn compute_upright_parameters(src: &RawSourceImage, params: &DevelopParams) -> (f32, f32, f32) {
    if !params.enable_transform {
        return (0.0, 0.0, 0.0);
    }
    match params.perspective_upright {
        UprightMode::Off => (0.0, 0.0, 0.0),
        UprightMode::Guided => {
            if params.guided_upright_lines.is_empty() {
                return (0.0, 0.0, 0.0);
            }
            let mut v_angles = Vec::new();
            let mut h_angles = Vec::new();
            for &(p0, p1) in &params.guided_upright_lines {
                let dx = p1[0] - p0[0];
                let dy = p1[1] - p0[1];
                if (dx * dx + dy * dy) < 1e-4 {
                    continue;
                }
                if dy.abs() >= dx.abs() {
                    // Near-vertical guide line: angle from vertical
                    let ang = (dx / dy).atan().to_degrees();
                    let mid_x = (p0[0] + p1[0]) * 0.5 - 0.5;
                    v_angles.push((ang, mid_x));
                } else {
                    // Near-horizontal guide line: angle from horizontal
                    let ang = (dy / dx).atan().to_degrees();
                    let mid_y = (p0[1] + p1[1]) * 0.5 - 0.5;
                    h_angles.push((ang, mid_y));
                }
            }
            let mut rot = 0.0f32;
            let mut vert = 0.0f32;
            let mut horiz = 0.0f32;
            if !h_angles.is_empty() {
                rot = -h_angles.iter().map(|a| a.0).sum::<f32>() / h_angles.len() as f32;
                if h_angles.len() >= 2 {
                    let diff = h_angles.last().unwrap().0 - h_angles[0].0;
                    horiz = (diff * 2.2).clamp(-60.0, 60.0);
                }
            } else if v_angles.len() == 1 {
                rot = v_angles[0].0;
            }
            if v_angles.len() >= 2 {
                // Converging verticals -> keystone vertical tilt
                let mut conv = 0.0f32;
                for &(ang, mx) in &v_angles {
                    if mx.abs() > 0.05 {
                        conv += ang / mx.signum();
                    }
                }
                vert = (-conv / v_angles.len() as f32 * 2.5).clamp(-65.0, 65.0);
            }
            (vert, horiz, rot.clamp(-10.0, 10.0))
        }
        mode => {
            // Fast Sobel dominant horizon & vertical convergence estimator
            let step = (src.width.min(src.height) / 64).max(2) as usize;
            let w = src.width as usize;
            let h = src.height as usize;
            let mut sum_h_ang = 0.0f32;
            let mut sum_h_w = 0.0f32;
            let mut sum_v_conv = 0.0f32;
            let mut sum_v_w = 0.0f32;
            for y in (step..h.saturating_sub(step)).step_by(step) {
                for x in (step..w.saturating_sub(step)).step_by(step) {
                    let lum_at = |px: usize, py: usize| -> f32 {
                        let i = (py * w + px) * 3;
                        0.2126 * src.rgb[i] + 0.7152 * src.rgb[i + 1] + 0.0722 * src.rgb[i + 2]
                    };
                    let gx = lum_at(x + 1, y) - lum_at(x - 1, y);
                    let gy = lum_at(x, y + 1) - lum_at(x, y - 1);
                    let mag = (gx * gx + gy * gy).sqrt();
                    if mag < 0.025 {
                        continue;
                    }
                    // Edge tangent is (-gy, gx)
                    let tx = -gy;
                    let ty = gx;
                    if tx.abs() > ty.abs() * 2.5 {
                        let ang = (ty / tx).atan().to_degrees();
                        if ang.abs() < 8.0 {
                            sum_h_ang += ang * mag;
                            sum_h_w += mag;
                        }
                    } else if ty.abs() > tx.abs() * 2.5 {
                        let ang = (tx / ty).atan().to_degrees();
                        let mx = (x as f32 / w as f32) - 0.5;
                        if ang.abs() < 12.0 && mx.abs() > 0.1 {
                            sum_v_conv += (ang * mx.signum()) * mag;
                            sum_v_w += mag;
                        }
                    }
                }
            }
            let level_rot = if sum_h_w > 0.1 {
                (-sum_h_ang / sum_h_w).clamp(-4.5, 4.5)
            } else {
                0.0
            };
            let vert_k = if sum_v_w > 0.1 {
                (-sum_v_conv / sum_v_w * 2.0).clamp(-30.0, 30.0)
            } else {
                0.0
            };
            match mode {
                UprightMode::Level => (0.0, 0.0, level_rot),
                UprightMode::Vertical => (vert_k, 0.0, 0.0),
                UprightMode::Auto => (vert_k * 0.75, 0.0, level_rot * 0.85),
                UprightMode::Full => (vert_k, 0.0, level_rot),
                _ => (0.0, 0.0, 0.0),
            }
        }
    }
}

/// Complete Multi-Stage Scene-Referred Linear Adobe Camera Raw (`cr_sdk`) Pipeline (`rayon` parallel)
pub fn render_develop_pipeline(
    src: &RawSourceImage,
    params: &DevelopParams,
    opts: RenderOptions,
) -> RenderedFrame {
    // Handle orthogonal 90° / 180° / 270° rotation cleanly (swapping width & height for 90°/270°)
    let orth_deg = ((params.rotation_degrees.round() as i32).rem_euclid(360) / 90) * 90;
    let swapped_axes = orth_deg == 90 || orth_deg == 270;
    let base_w = if swapped_axes { src.height } else { src.width };
    let base_h = if swapped_axes { src.width } else { src.height };

    let (c_left, c_top, c_right, c_bottom) = if opts.apply_crop {
        (
            params.crop_left.clamp(0.0, 0.95),
            params.crop_top.clamp(0.0, 0.95),
            params.crop_right.clamp(0.05, 1.0),
            params.crop_bottom.clamp(0.05, 1.0),
        )
    } else {
        (0.0, 0.0, 1.0, 1.0)
    };

    let (out_w, out_h) = if opts.is_draft_interactive {
        let (raw_w, raw_h) = if opts.apply_crop {
            (
                ((base_w as f32 * (c_right - c_left)).round() as u32).max(16),
                ((base_h as f32 * (c_bottom - c_top)).round() as u32).max(16),
            )
        } else {
            (base_w.max(16), base_h.max(16))
        };
        let sc = 2560.0 / (raw_w.max(raw_h) as f32).max(2560.0);
        (
            ((raw_w as f32 * sc).round() as u32).max(16),
            ((raw_h as f32 * sc).round() as u32).max(16),
        )
    } else if opts.apply_crop {
        (
            ((base_w as f32 * (c_right - c_left)).round() as u32).max(16),
            ((base_h as f32 * (c_bottom - c_top)).round() as u32).max(16),
        )
    } else {
        (base_w.max(16), base_h.max(16))
    };

    // Precompute Tone Curve LUTs
    let lut_master = if params.enable_tone_curve {
        build_tone_curve_lut(
            &params.tone_curve_pv2012,
            params.parametric_shadows,
            params.parametric_darks,
            params.parametric_lights,
            params.parametric_highlights,
            params.parametric_shadow_split,
            params.parametric_midtone_split,
            params.parametric_highlight_split,
            true,
        )
    } else {
        let mut l = [0.0f32; 256];
        for i in 0..256 {
            l[i] = i as f32 / 255.0;
        }
        l
    };
    let lut_r = if params.enable_tone_curve {
        build_tone_curve_lut(&params.tone_curve_pv2012_red, 0.0, 0.0, 0.0, 0.0, 25.0, 50.0, 75.0, false)
    } else {
        lut_master
    };
    let lut_g = if params.enable_tone_curve {
        build_tone_curve_lut(&params.tone_curve_pv2012_green, 0.0, 0.0, 0.0, 0.0, 25.0, 50.0, 75.0, false)
    } else {
        lut_master
    };
    let lut_b = if params.enable_tone_curve {
        build_tone_curve_lut(&params.tone_curve_pv2012_blue, 0.0, 0.0, 0.0, 0.0, 25.0, 50.0, 75.0, false)
    } else {
        lut_master
    };

    // 3x3 Bradford Chromatic Adaptation Matrix from camera As-Shot WB -> Target WB
    let wb_mat = match params.white_balance {
        WhiteBalanceMode::AsShot => [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        WhiteBalanceMode::Auto => {
            let (at, ati) = estimate_auto_wb(src);
            compute_bradford_wb_matrix(src.as_shot_temp, src.as_shot_tint, at, ati)
        }
        WhiteBalanceMode::Custom => {
            compute_bradford_wb_matrix(src.as_shot_temp, src.as_shot_tint, params.temperature, params.tint)
        }
        mode => {
            let (pt, pti) = mode
                .preset_temp_tint()
                .unwrap_or((params.temperature, params.tint));
            compute_bradford_wb_matrix(src.as_shot_temp, src.as_shot_tint, pt, pti)
        }
    };

    // Linear Exposure multiplier (`2^(EV + sensor_headroom_lift)`)
    // For unboosted scene-referred RAW (`CIRAWFilter` boostAmount=0), ACR applies baseline exposure +
    // middle-gray perceptual anchor so raw 18% gray maps accurately to ACR PV2012 middle gray.
    let raw_ev_lift = if src.is_scene_referred {
        src.baseline_ev.clamp(-1.0, 1.5) * 0.35 + 0.42
    } else {
        0.0
    };
    let ev_mult = 2.0f32.powf(params.exposure_2012 + raw_ev_lift);
    let look_scale = (params.look_amount / 100.0).clamp(0.0, 2.0);

    // Transform, Upright, Constrain-to-Warp & Crop Straighten Angle
    let (auto_v, auto_h, auto_r) = compute_upright_parameters(src, params);
    let (upright_v, upright_h, transform_rot_deg, scale_inv, aspect_x, aspect_y, off_x, off_y) =
        if params.enable_transform {
            let v = (params.perspective_vertical + auto_v) * 0.0035;
            let h = (params.perspective_horizontal + auto_h) * 0.0035;
            let rot = params.perspective_rotate + auto_r;
            let mut s_inv = 100.0 / params.perspective_scale.max(20.0);
            if params.crop_constrain_to_warp {
                let warp_shrink = 1.0
                    - (rot.abs() * 0.014 + v.abs() * 0.42 + h.abs() * 0.42).clamp(0.0, 0.45);
                s_inv *= warp_shrink;
            }
            (
                v,
                h,
                rot,
                s_inv,
                1.0 + params.perspective_aspect * 0.003,
                1.0 - params.perspective_aspect * 0.003,
                params.perspective_x * 0.003,
                params.perspective_y * 0.003,
            )
        } else {
            (0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0)
        };

    let crop_rot_deg = if opts.apply_crop { params.crop_angle } else { 0.0 };
    let total_fine_rad = (transform_rot_deg + crop_rot_deg).to_radians();
    // Auto-zoom slightly when straighten angle is applied so corners don't show black triangles
    let crop_rot_zoom = if opts.apply_crop && params.crop_angle.abs() > 0.02 {
        let a = params.crop_angle.abs().to_radians();
        let ar = (out_w as f32 / out_h.max(1) as f32).max(1.0 / (out_w as f32 / out_h.max(1) as f32));
        1.0 / (a.cos() + ar * a.sin()).max(1.0)
    } else {
        1.0
    };
    let cos_r = total_fine_rad.cos();
    let sin_r = total_fine_rad.sin();

    let sw = src.width as i32;
    let sh = src.height as i32;

    let sample_linear_rgb = |fx: f32, fy: f32| -> (f32, f32, f32) {
        let gx = (fx * (sw - 1) as f32).clamp(0.0, (sw - 1) as f32);
        let gy = (fy * (sh - 1) as f32).clamp(0.0, (sh - 1) as f32);
        let x0 = gx.floor() as usize;
        let y0 = gy.floor() as usize;
        let x1 = (x0 + 1).min(sw as usize - 1);
        let y1 = (y0 + 1).min(sh as usize - 1);
        let tx = gx - x0 as f32;
        let ty = gy - y0 as f32;
        let w_idx = src.width as usize;
        let idx00 = (y0 * w_idx + x0) * 3;
        let idx10 = (y0 * w_idx + x1) * 3;
        let idx01 = (y1 * w_idx + x0) * 3;
        let idx11 = (y1 * w_idx + x1) * 3;
        let w00 = (1.0 - tx) * (1.0 - ty);
        let w10 = tx * (1.0 - ty);
        let w01 = (1.0 - tx) * ty;
        let w11 = tx * ty;
        (
            src.rgb[idx00] * w00 + src.rgb[idx10] * w10 + src.rgb[idx01] * w01 + src.rgb[idx11] * w11,
            src.rgb[idx00 + 1] * w00 + src.rgb[idx10 + 1] * w10 + src.rgb[idx01 + 1] * w01 + src.rgb[idx11 + 1] * w11,
            src.rgb[idx00 + 2] * w00 + src.rgb[idx10 + 2] * w10 + src.rgb[idx01 + 2] * w01 + src.rgb[idx11 + 2] * w11,
        )
    };

    let mut rgba_out = vec![0u8; (out_w * out_h * 4) as usize];
    // Store clean pre-overlay sRGB bytes in parallel if clipping/gamut overlays are active so histogram stays 100% pure!
    let has_overlays = opts.show_highlight_clipping
        || opts.show_shadow_clipping
        || (opts.soft_proofing && opts.gamut_warning)
        || opts.active_mask_overlay.is_some()
        || opts.show_sharpen_mask_preview
        || params.point_color_visualize_range
        || params.visualize_hdr_ranges
        || (params.enable_retouch && params.retouch_visualize_spots)
        || (params.lens_blur_active && params.lens_blur_visualize_depth);
    let mut clean_rgb_for_hist = if has_overlays {
        vec![0u8; (out_w * out_h * 3) as usize]
    } else {
        Vec::new()
    };

    let row_stride = (out_w * 4) as usize;
    let hist_stride = (out_w * 3) as usize;

    let render_row = |oy: usize, row_bytes: &mut [u8], clean_row: Option<&mut [u8]>| {
        let v_out = oy as f32 / out_h.max(1) as f32;
        let ny_crop = c_top + v_out * (c_bottom - c_top);

        let mut clean_slice = clean_row;

        for ox in 0..out_w {
            let u_out = ox as f32 / out_w.max(1) as f32;
            let nx_crop = c_left + u_out * (c_right - c_left);

            // --- 1. Transform / Upright / Crop Angle & Lens Distortion Mapping ---
            let mut dx = (nx_crop - 0.5 - off_x) * scale_inv * crop_rot_zoom * aspect_x;
            let mut dy = (ny_crop - 0.5 - off_y) * scale_inv * crop_rot_zoom * aspect_y;

            let rx = dx * cos_r - dy * sin_r;
            let ry = dx * sin_r + dy * cos_r;

            let denom = (1.0 + rx * upright_h + ry * upright_v).max(0.25);
            dx = rx / denom;
            dy = ry / denom;

            let r2 = dx * dx + dy * dy;
            if params.enable_lens_corrections {
                let k1 = -params.lens_manual_distortion_amount * 0.0035
                    + if params.lens_profile_enable {
                        (params.lens_profile_distortion_scale - 100.0) * -0.0012
                    } else {
                        0.0
                    };
                let radial = 1.0 + k1 * r2;
                dx *= radial;
                dy *= radial;
            }

            let mut u_rot = (dx + 0.5).clamp(0.0, 1.0);
            let mut v_rot = (dy + 0.5).clamp(0.0, 1.0);
            if params.flip_horizontal {
                u_rot = 1.0 - u_rot;
            }
            if params.flip_vertical {
                v_rot = 1.0 - v_rot;
            }
            // Inverse orthogonal rotation back to sensor coordinates
            let (nx, ny) = match orth_deg {
                90 => (v_rot, 1.0 - u_rot),
                180 => (1.0 - u_rot, 1.0 - v_rot),
                270 => (1.0 - v_rot, u_rot),
                _ => (u_rot, v_rot),
            };

            let (mut depth_v, subj_v, sky_v, peop_v) = src.sample_semantics(nx, ny);
            if !params.lens_blur_refinement_strokes.is_empty() {
                for stroke in &params.lens_blur_refinement_strokes {
                    let r_out = stroke.radius.max(0.005);
                    let r_in = (r_out * (1.0 - stroke.feather * 0.008)).max(0.001);
                    for &(px, py) in &stroke.points {
                        let d = ((nx - px).powi(2) + (ny - py).powi(2)).sqrt();
                        if d < r_out {
                            let w = smoothstep(r_out, r_in, d) * (stroke.amount * 0.01);
                            let target_d = if stroke.is_focus {
                                (params.lens_blur_focal_range[1] + params.lens_blur_focal_range[2]) * 0.005
                            } else {
                                0.98
                            };
                            depth_v = depth_v * (1.0 - w) + target_d * w;
                        }
                    }
                }
            }

            // Lateral Chromatic Aberration R/B offset
            let auto_ca_r = if params.enable_lens_corrections && params.auto_lateral_ca { -6.0 } else { 0.0 };
            let auto_ca_b = if params.enable_lens_corrections && params.auto_lateral_ca { 6.0 } else { 0.0 };
            let tot_ca_r = params.chromatic_aberration_r + auto_ca_r;
            let tot_ca_b = params.chromatic_aberration_b + auto_ca_b;

            let (mut r, mut g, mut b) = if params.enable_lens_corrections
                && (tot_ca_r.abs() > 0.1 || tot_ca_b.abs() > 0.1)
            {
                let ca_r = 1.0 + tot_ca_r * 0.00018;
                let ca_b = 1.0 + tot_ca_b * 0.00018;
                let (rr, _, _) = sample_linear_rgb(
                    ((nx - 0.5) * ca_r + 0.5).clamp(0.0, 1.0),
                    ((ny - 0.5) * ca_r + 0.5).clamp(0.0, 1.0),
                );
                let (_, gg, _) = sample_linear_rgb(nx, ny);
                let (_, _, bb) = sample_linear_rgb(
                    ((nx - 0.5) * ca_b + 0.5).clamp(0.0, 1.0),
                    ((ny - 0.5) * ca_b + 0.5).clamp(0.0, 1.0),
                );
                (rr, gg, bb)
            } else {
                sample_linear_rgb(nx, ny)
            };

            // --- 2. Spot Removal / Healing (Perimeter-Ring Lighting Match!) / Clone ---
            if !opts.is_draft_interactive && params.enable_retouch && !params.retouch_areas.is_empty() {
                for spot in &params.retouch_areas {
                    let r_rad = spot.radius.max(0.003);
                    // Support both single-click spots and multi-point brush strokes
                    let (best_d, ref_tx, ref_ty) = if spot.points.is_empty() {
                        let d = ((nx - spot.target_x).powi(2) + (ny - spot.target_y).powi(2)).sqrt();
                        (d, spot.target_x, spot.target_y)
                    } else {
                        let mut bd = f32::MAX;
                        let mut bx = spot.target_x;
                        let mut by = spot.target_y;
                        for &(ptx, pty) in &spot.points {
                            let d = ((nx - ptx).powi(2) + (ny - pty).powi(2)).sqrt();
                            if d < bd {
                                bd = d;
                                bx = ptx;
                                by = pty;
                            }
                        }
                        (bd, bx, by)
                    };
                    if best_d < r_rad {
                        let inner = (r_rad * (1.0 - spot.feather * 0.008)).max(0.001);
                        let w = smoothstep(r_rad, inner, best_d) * (spot.opacity * 0.01);
                        let off_sx = spot.source_x - spot.target_x;
                        let off_sy = spot.source_y - spot.target_y;
                        let s_nx = (nx + off_sx).clamp(0.0, 1.0);
                        let s_ny = (ny + off_sy).clamp(0.0, 1.0);
                        let (sr, sg, sb) = sample_linear_rgb(s_nx, s_ny);
                        match spot.mode {
                            RetouchMode::Clone => {
                                r = r * (1.0 - w) + sr * w;
                                g = g * (1.0 - w) + sg * w;
                                b = b * (1.0 - w) + sb * w;
                            }
                            RetouchMode::Remove => {
                                // Local AI Generative / Content-Aware Object Removal
                                let (cr, cg, cb) = crate::ai::sample_generative_remove(
                                    nx,
                                    ny,
                                    spot,
                                    &sample_linear_rgb,
                                );
                                r = cr;
                                g = cg;
                                b = cb;
                            }
                            RetouchMode::Heal => {
                                // Sample 4 points on the CLEAN perimeter ring around target & source (never the blemish center!)
                                let ring_r = r_rad * 1.15;
                                let mut tr_ring = 0.0f32;
                                let mut tg_ring = 0.0f32;
                                let mut tb_ring = 0.0f32;
                                let mut sr_ring = 0.0f32;
                                let mut sg_ring = 0.0f32;
                                let mut sb_ring = 0.0f32;
                                for &(ox_r, oy_r) in &[(1.0f32, 0.0f32), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
                                    let (trp, tgp, tbp) = sample_linear_rgb(
                                        (ref_tx + ox_r * ring_r).clamp(0.0, 1.0),
                                        (ref_ty + oy_r * ring_r).clamp(0.0, 1.0),
                                    );
                                    let (srp, sgp, sbp) = sample_linear_rgb(
                                        (ref_tx + off_sx + ox_r * ring_r).clamp(0.0, 1.0),
                                        (ref_ty + off_sy + oy_r * ring_r).clamp(0.0, 1.0),
                                    );
                                    tr_ring += trp;
                                    tg_ring += tgp;
                                    tb_ring += tbp;
                                    sr_ring += srp;
                                    sg_ring += sgp;
                                    sb_ring += sbp;
                                }
                                let kr = (tr_ring / sr_ring.max(1e-4)).clamp(0.35, 2.8);
                                let kg = (tg_ring / sg_ring.max(1e-4)).clamp(0.35, 2.8);
                                let kb = (tb_ring / sb_ring.max(1e-4)).clamp(0.35, 2.8);
                                r = r * (1.0 - w) + (sr * kr) * w;
                                g = g * (1.0 - w) + (sg * kg) * w;
                                b = b * (1.0 - w) + (sb * kb) * w;
                            }
                        }
                    }
                }
            }

            // Distraction Removal (Reflections / Background People attenuation)
            if !opts.is_draft_interactive && params.enable_distraction_removal && params.distraction_removal_reflections {
                let hi = r.max(g).max(b);
                if hi > 0.55 {
                    let damp = ((hi - 0.55) * 0.35).clamp(0.0, 0.30);
                    r *= 1.0 - damp;
                    g *= 1.0 - damp;
                    b *= 1.0 - damp;
                }
            }

            // --- 3. Red Eye & Pet Eye Correction (with Catchlight) ---
            if !opts.is_draft_interactive && params.enable_red_eye && !params.red_eye_spots.is_empty() {
                for eye in &params.red_eye_spots {
                    let eff_r = eye.radius * (0.5 + eye.pupil_size * 0.01);
                    let dist = ((nx - eye.center_x).powi(2) + (ny - eye.center_y).powi(2)).sqrt();
                    if dist < eff_r {
                        let w = smoothstep(eff_r, eff_r * 0.38, dist);
                        let dark = 1.0 - (eye.darken * 0.0088) * w;
                        match eye.mode {
                            RedEyeMode::RedEye => {
                                if r > (g + b) * 0.52 {
                                    let neutral = (g + b) * 0.45;
                                    r = (r * (1.0 - w) + neutral * w) * dark;
                                    g *= dark;
                                    b *= dark;
                                }
                            }
                            RedEyeMode::PetEye => {
                                let neutral = (r + g + b) * 0.22;
                                r = (r * (1.0 - w) + neutral * w) * dark;
                                g = (g * (1.0 - w) + neutral * w) * dark;
                                b = (b * (1.0 - w) + neutral * w) * dark;
                                if eye.add_catchlight {
                                    let cd = ((nx - (eye.center_x - eff_r * 0.22)).powi(2)
                                        + (ny - (eye.center_y - eff_r * 0.22)).powi(2))
                                    .sqrt();
                                    if cd < eff_r * 0.18 {
                                        let cw = smoothstep(eff_r * 0.18, eff_r * 0.04, cd) * 0.75;
                                        r += cw;
                                        g += cw;
                                        b += cw;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // --- 4. AI Lens Blur (`cr_lens_blur` Depth-Guided Bokeh Convolution in Linear Light!) ---
            if !opts.is_draft_interactive && params.lens_blur_active && params.lens_blur_amount > 0.5 && !params.lens_blur_visualize_depth {
                let d_pct = depth_v * 100.0;
                let [n_fade, n_foc, f_foc, f_fade] = params.lens_blur_focal_range;
                let coc = if d_pct >= n_foc && d_pct <= f_foc {
                    0.0
                } else if d_pct < n_foc {
                    ((n_foc - d_pct) / (n_foc - n_fade).max(3.0)).clamp(0.0, 1.0)
                } else {
                    ((d_pct - f_foc) / (f_fade - f_foc).max(3.0)).clamp(0.0, 1.0)
                };

                if coc > 0.02 {
                    let radius = coc * (params.lens_blur_amount * 0.00045);
                    let boost = params.lens_blur_highlights_boost * 0.05;
                    let thresh = (params.lens_blur_bokeh_boost_threshold * 0.008).clamp(0.1, 0.9);
                    let cat_amt = (params.lens_blur_cat_eye_amount * 0.01).clamp(0.0, 1.0);
                    let mut sum_r = 0.0f32;
                    let mut sum_g = 0.0f32;
                    let mut sum_b = 0.0f32;
                    let mut sum_w = 0.0f32;

                    const TAPS: [(f32, f32); 12] = [
                        (0.0, 0.0), (1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0),
                        (0.707, 0.707), (-0.707, 0.707), (0.707, -0.707), (-0.707, -0.707),
                        (0.5, 0.866), (-0.866, 0.5), (0.5, -0.866),
                    ];
                    for &(kx, ky) in &TAPS {
                        let (mut sx_off, mut sy_off) = (kx, ky);
                        let mut kw = 1.0f32;
                        match params.lens_blur_bokeh {
                            BokehShape::Circle => {}
                            BokehShape::SoapBubble => {
                                let r_len = (kx * kx + ky * ky).sqrt();
                                kw = 0.55 + 0.95 * r_len;
                            }
                            BokehShape::Ring => {
                                let r_len = (kx * kx + ky * ky).sqrt();
                                if r_len < 0.35 {
                                    continue;
                                }
                                kw = 1.45;
                            }
                            BokehShape::Anamorphic => {
                                sx_off *= 0.52;
                                sy_off *= 1.48;
                            }
                            BokehShape::CatEye => {
                                let eff_cat = cat_amt.max(0.55);
                                let vig = 1.0 - eff_cat * ((nx - 0.5) * kx + (ny - 0.5) * ky).abs() * 1.8;
                                kw = vig.clamp(0.1, 1.2);
                            }
                            BokehShape::Blade => {
                                let ang = ky.atan2(kx);
                                let poly = (5.0 * ang).cos() * 0.18 + 0.82;
                                sx_off *= poly;
                                sy_off *= poly;
                            }
                        }
                        if cat_amt > 0.01 && params.lens_blur_bokeh != BokehShape::CatEye {
                            let vig = 1.0 - cat_amt * ((nx - 0.5) * kx + (ny - 0.5) * ky).abs() * 1.6;
                            kw *= vig.clamp(0.15, 1.2);
                        }
                        let (tr, tg, tb) = sample_linear_rgb(
                            (nx + sx_off * radius).clamp(0.0, 1.0),
                            (ny + sy_off * radius).clamp(0.0, 1.0),
                        );
                        let t_lum = 0.2126 * tr + 0.7152 * tg + 0.0722 * tb;
                        let spec_w = kw * (1.0 + boost * (t_lum - thresh).max(0.0).powi(2) * 14.0);
                        sum_r += tr * spec_w;
                        sum_g += tg * spec_w;
                        sum_b += tb * spec_w;
                        sum_w += spec_w;
                    }
                    if sum_w > 0.0 {
                        r = sum_r / sum_w;
                        g = sum_g / sum_w;
                        b = sum_b / sum_w;
                    }
                }
            }

            // --- 5. Spatial Neighborhood: Sharpening, Luminance & Color NR, Texture & Clarity ---
            let need_spatial = !opts.is_draft_interactive && ((params.enable_detail
                && (params.sharpness > 0.5
                    || params.luminance_smoothing > 0.5
                    || params.color_noise_reduction > 0.5
                    || params.ai_denoise_amount > 0.5
                    || params.raw_details_enabled
                    || params.super_resolution_enabled))
                || params.texture.abs() > 0.5
                || params.clarity_2012.abs() > 0.5
                || (params.enable_mask_group_based_corrections && !params.mask_groups.is_empty())
                || opts.show_sharpen_mask_preview
                || (params.enable_retouch && params.retouch_visualize_spots));

            let (mut hp_r, mut hp_g, mut hp_b, mut edge_gate) = (0.0f32, 0.0f32, 0.0f32, 1.0f32);
            let mut spot_vis_mag = 0.0f32;

            if need_spatial {
                let rad_px = if params.enable_detail {
                    params.sharpen_radius.clamp(0.5, 3.0)
                } else {
                    1.0
                };
                let step_x = (rad_px / sw.max(1) as f32).max(1.0 / sw.max(1) as f32);
                let step_y = (rad_px / sh.max(1) as f32).max(1.0 / sh.max(1) as f32);
                let (r_l, g_l, b_l) = sample_linear_rgb((nx - step_x).max(0.0), ny);
                let (r_r, g_r, b_r) = sample_linear_rgb((nx + step_x).min(1.0), ny);
                let (r_u, g_u, b_u) = sample_linear_rgb(nx, (ny - step_y).max(0.0));
                let (r_d, g_d, b_d) = sample_linear_rgb(nx, (ny + step_y).min(1.0));

                let avg_r = (r_l + r_r + r_u + r_d) * 0.25;
                let avg_g = (g_l + g_r + g_u + g_d) * 0.25;
                let avg_b = (b_l + b_r + b_u + b_d) * 0.25;

                hp_r = r - avg_r;
                hp_g = g - avg_g;
                hp_b = b - avg_b;
                let edge_mag = (hp_r.abs() + hp_g.abs() + hp_b.abs()) * 0.3333;
                spot_vis_mag = edge_mag;

                if params.enable_detail {
                    // Luminance & AI Denoise (with Detail & Contrast preservation)
                    let det_pres = 0.04 + params.luminance_noise_reduction_detail * 0.0014;
                    let denoise_str = ((params.luminance_smoothing + params.ai_denoise_amount * 1.15) * 0.0085)
                        .clamp(0.0, 0.94);
                    if denoise_str > 0.005 && edge_mag < det_pres {
                        let contrast_keep = params.luminance_noise_reduction_contrast * 0.005;
                        let smooth_w = (denoise_str * (1.0 - edge_mag / det_pres) * (1.0 - contrast_keep))
                            .clamp(0.0, 0.94);
                        r = r * (1.0 - smooth_w) + avg_r * smooth_w;
                        g = g * (1.0 - smooth_w) + avg_g * smooth_w;
                        b = b * (1.0 - smooth_w) + avg_b * smooth_w;
                    }

                    // Color Noise Reduction (chrominance smoothing preserving luminance)
                    if params.color_noise_reduction > 0.5 {
                        let cnr_w = (params.color_noise_reduction * 0.0075
                            * (0.6 + params.color_noise_reduction_smoothness * 0.006)
                            * (1.0 - params.color_noise_reduction_detail * 0.003 * (edge_mag * 8.0).min(1.0)))
                        .clamp(0.0, 0.88);
                        let y_cur = 0.2126 * r + 0.7152 * g + 0.0722 * b;
                        let y_avg = (0.2126 * avg_r + 0.7152 * avg_g + 0.0722 * avg_b).max(1e-5);
                        let r_chroma_smoothed = avg_r * (y_cur / y_avg);
                        let g_chroma_smoothed = avg_g * (y_cur / y_avg);
                        let b_chroma_smoothed = avg_b * (y_cur / y_avg);
                        r = r * (1.0 - cnr_w) + r_chroma_smoothed * cnr_w;
                        g = g * (1.0 - cnr_w) + g_chroma_smoothed * cnr_w;
                        b = b * (1.0 - cnr_w) + b_chroma_smoothed * cnr_w;
                    }

                    // Edge-Masked Unsharp Mask (`sharpness == 0` = zero sharpening!)
                    let mask_thresh = params.sharpen_edge_masking * 0.0014;
                    edge_gate = smoothstep(mask_thresh * 0.65, mask_thresh + 0.004, edge_mag);
                    let detail_boost = 0.65 + params.sharpen_detail * 0.007;
                    let raw_enh = if params.raw_details_enabled || params.super_resolution_enabled {
                        0.15
                    } else {
                        0.0
                    };
                    let sharp_gain = (params.sharpness * 0.0085 * detail_boost + raw_enh) * edge_gate;
                    let halo_cap = 0.12 + params.sharpen_detail * 0.003;
                    r += (hp_r * sharp_gain).clamp(-halo_cap, halo_cap);
                    g += (hp_g * sharp_gain).clamp(-halo_cap, halo_cap);
                    b += (hp_b * sharp_gain).clamp(-halo_cap, halo_cap);
                }

                // Basic Texture & Clarity are ALWAYS independent of `enable_detail`!
                if params.texture.abs() > 0.1 || params.clarity_2012.abs() > 0.1 {
                    let pres_gain = params.texture * 0.0075 + params.clarity_2012 * 0.0045;
                    r = (r + hp_r * pres_gain).max(0.0);
                    g = (g + hp_g * pres_gain).max(0.0);
                    b = (b + hp_b * pres_gain).max(0.0);
                }
            }

            // --- 6. Optical Lens Vignetting & Axial Defringe (Purple / Green) ---
            if params.enable_lens_corrections {
                let prof_vig = if params.lens_profile_enable {
                    (params.lens_profile_vignetting_scale / 100.0) * 14.0
                } else {
                    0.0
                };
                let tot_vig = params.vignette_amount + prof_vig;
                if tot_vig.abs() > 0.1 {
                    let mid = params.vignette_midpoint * 0.01;
                    let d = ((nx - 0.5).powi(2) + (ny - 0.5).powi(2)).sqrt() * 1.414;
                    let v_w = ((d - mid) / (1.0 - mid).max(0.1)).clamp(0.0, 1.0).powi(2);
                    let mult = (1.0 + tot_vig * 0.0065 * v_w).max(0.1);
                    r *= mult;
                    g *= mult;
                    b *= mult;
                }
            }

            // --- 7. 3x3 Bradford White Balance & Camera Calibration ---
            let wr = (wb_mat[0][0] * r + wb_mat[0][1] * g + wb_mat[0][2] * b).max(0.0);
            let wg = (wb_mat[1][0] * r + wb_mat[1][1] * g + wb_mat[1][2] * b).max(0.0);
            let wb = (wb_mat[2][0] * r + wb_mat[2][1] * g + wb_mat[2][2] * b).max(0.0);
            r = wr;
            g = wg;
            b = wb;

            if params.enable_calibration {
                let lum0 = (0.2126 * r + 0.7152 * g + 0.0722 * b).clamp(0.0, 1.0);
                let sh_w = (1.0 - lum0).powi(2);
                g = (g - params.shadow_tint * 0.0018 * sh_w).max(0.0);
                r = (r + params.shadow_tint * 0.0009 * sh_w).max(0.0);
                b = (b + params.shadow_tint * 0.0009 * sh_w).max(0.0);

                let r_cal = r + (params.red_hue * 0.0016 * (r - g)) + (params.red_saturation * 0.0026 * (r - lum0));
                let g_cal = g + (params.green_hue * 0.0016 * (g - b)) + (params.green_saturation * 0.0026 * (g - lum0));
                let b_cal = b + (params.blue_hue * 0.0016 * (b - r)) + (params.blue_saturation * 0.0026 * (b - lum0));
                r = r_cal.max(0.0);
                g = g_cal.max(0.0);
                b = b_cal.max(0.0);
            }

            // --- 8. Linear Exposure (`2^EV`), Dehaze & ACR PV2012 Tone Mapping ---
            r *= ev_mult;
            g *= ev_mult;
            b *= ev_mult;

            if params.dehaze.abs() > 0.1 {
                let dh = params.dehaze * 0.0042;
                let min_c = r.min(g).min(b);
                r = ((r - min_c * dh * 0.50) * (1.0 + dh * 0.55)).max(0.0);
                g = ((g - min_c * dh * 0.50) * (1.0 + dh * 0.55)).max(0.0);
                b = ((b - min_c * dh * 0.50) * (1.0 + dh * 0.55)).max(0.0);
            }

            // Convert scene-referred linear light into perceptual tone space for ACR PV2012 tone operators:
            // For RAW (`is_scene_referred`), apply ACR's smooth filmic base response curve before sRGB encoding
            // so shadows/midtones have Adobe Color richness while highlight headroom rolls off without clipping!
            let (prof_contrast, prof_sat) = match params.camera_profile {
                CameraProfile::AdobeColor => (1.04, 1.06),
                CameraProfile::AdobeVivid => (1.16, 1.24),
                CameraProfile::AdobeLandscape => (1.14, 1.26),
                CameraProfile::AdobePortrait => (0.95, 0.96),
                CameraProfile::AdobeNeutral => (0.88, 0.89),
                CameraProfile::AdobeStandard => (0.99, 0.98),
                CameraProfile::AdobeMonochrome => (1.06, 0.0),
                CameraProfile::CameraStandard => (1.03, 1.05),
                CameraProfile::CameraFaithful => (0.96, 0.96),
                CameraProfile::CameraPositiveFilm => (1.18, 1.18),
            };

            let hdr_ceil = if params.hdr_edit_mode {
                2.0f32.powf(params.hdr_max_value.clamp(1.0, 4.0))
            } else {
                1.0
            };

            // Highlight compression shoulder in linear light (`Highlights2012` & `Whites2012`)
            let max_lin = r.max(g).max(b);
            let lin_lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            if max_lin > 1e-5 {
                // Soft highlight knee when Highlights or Whites are negative, or white boost when positive
                let hl_pull = (-params.highlights_2012.min(0.0) * 0.0052)
                    + (-params.whites_2012.min(0.0) * 0.0046)
                    + if src.is_scene_referred { 0.14 } else { 0.0 };
                let hl_push = (params.highlights_2012.max(0.0) * 0.0035)
                    + (params.whites_2012.max(0.0) * 0.0045);

                // Smooth Rational Shoulder on `max_lin` preserves highlight color without clipping RGB channels individually!
                let compressed_max = (max_lin * (1.0 + hl_push * smoothstep(0.25, 0.95, max_lin)))
                    / (1.0 + hl_pull * max_lin.powf(1.15));
                let ratio = compressed_max / max_lin;
                r *= ratio;
                g *= ratio;
                b *= ratio;
            }

            // Encode to perceptual sRGB [0..1] for PV2012 Shadows / Blacks / Contrast / Tone Curve / HSL
            let mut pr = linear_to_srgb(r / hdr_ceil);
            let mut pg = linear_to_srgb(g / hdr_ceil);
            let mut pb = linear_to_srgb(b / hdr_ceil);

            // ACR Base Filmic S-Curve for unboosted RAW sensor data
            if src.is_scene_referred {
                let base_curve = |x: f32| -> f32 {
                    let xc = x.clamp(0.0, 1.2);
                    // Lifts midtones gently while anchoring deep blacks
                    let lifted = xc.powf(0.88) * 1.03;
                    let toe = smoothstep(0.0, 0.10, lifted) * 0.04 + 0.96;
                    (lifted * toe).clamp(0.0, 1.05)
                };
                pr = base_curve(pr);
                pg = base_curve(pg);
                pb = base_curve(pb);
            }

            // Perceptual Shadows, Blacks, positive/fine Highlights & Whites with chromaticity preservation
            let plum = (0.2126 * pr + 0.7152 * pg + 0.0722 * pb).clamp(0.0, 1.0);
            if plum > 1e-4 {
                let sh_zone = (1.0 - plum).powi(2) * (0.35 + 2.6 * plum * (1.0 - plum));
                let bl_zone = (1.0 - plum).powi(3);
                let hi_fine = plum.powi(2) * (1.0 - plum) * 2.2;
                let delta = params.shadows_2012 * 0.0028 * sh_zone
                    + params.blacks_2012 * 0.0020 * bl_zone
                    + params.highlights_2012 * 0.0011 * hi_fine
                    + params.whites_2012 * 0.0009 * plum.powi(3);
                let new_plum = (plum + delta).max(0.0);
                let sc = (new_plum / plum).clamp(0.05, 3.5);
                // Blend 85% multiplicative (preserves saturation) + 15% additive (prevents shadow noise chroma explosion)
                pr = (pr * sc * 0.85 + (pr + delta) * 0.15).max(0.0);
                pg = (pg * sc * 0.85 + (pg + delta) * 0.15).max(0.0);
                pb = (pb * sc * 0.85 + (pb + delta) * 0.15).max(0.0);
            }

            // SDR Preview controls when HDR Edit Mode + Preview for SDR Display is enabled
            if params.hdr_edit_mode && params.preview_for_sdr_display {
                let sdr_ev = 1.0 + params.sdr_brightness * 0.004;
                let sdr_c = params.sdr_contrast * 0.005 + params.sdr_clarity * 0.003;
                pr = (0.5 + (pr * sdr_ev - 0.5) * (1.0 + sdr_c)).clamp(0.0, 1.0);
                pg = (0.5 + (pg * sdr_ev - 0.5) * (1.0 + sdr_c)).clamp(0.0, 1.0);
                pb = (0.5 + (pb * sdr_ev - 0.5) * (1.0 + sdr_c)).clamp(0.0, 1.0);
            }

            // Contrast & Profile Contrast S-curve
            let total_contrast = ((params.contrast_2012 * 0.0055 + params.clarity_2012 * 0.0028)
                + (prof_contrast - 1.0) * look_scale)
                .clamp(-0.75, 1.25);
            if total_contrast.abs() > 0.001 {
                let pivot = 0.46;
                pr = (pivot + (pr - pivot) * (1.0 + total_contrast)).max(0.0);
                pg = (pivot + (pg - pivot) * (1.0 + total_contrast)).max(0.0);
                pb = (pivot + (pb - pivot) * (1.0 + total_contrast)).max(0.0);
            }

            // --- 9. Tone Curve LUTs + `curve_refine_saturation` ---
            if params.enable_tone_curve {
                let (_, s_before, _) = rgb_to_hsl(pr.clamp(0.0, 1.0), pg.clamp(0.0, 1.0), pb.clamp(0.0, 1.0));
                pr = sample_lut(&lut_r, sample_lut(&lut_master, pr));
                pg = sample_lut(&lut_g, sample_lut(&lut_master, pg));
                pb = sample_lut(&lut_b, sample_lut(&lut_master, pb));
                let refine = (params.curve_refine_saturation / 100.0).clamp(0.0, 1.0);
                if refine < 0.99 {
                    let (hc, sc, lc) = rgb_to_hsl(pr, pg, pb);
                    let blended_s = s_before + (sc - s_before) * refine;
                    let (rr, rg, rb) = hsl_to_rgb(hc, blended_s, lc);
                    pr = rr;
                    pg = rg;
                    pb = rb;
                }
            }

            // Axial Defringe (Purple / Green fringe suppression on high-contrast edges)
            if params.enable_lens_corrections
                && (params.defringe_purple_amount > 0.1 || params.defringe_green_amount > 0.1)
            {
                let (dh, ds, dl) = rgb_to_hsl(pr.clamp(0.0, 1.0), pg.clamp(0.0, 1.0), pb.clamp(0.0, 1.0));
                let mut damp = 0.0f32;
                if params.defringe_purple_amount > 0.1 {
                    let p_lo = 230.0 + params.defringe_purple_hue_lo * 0.8;
                    let p_hi = 270.0 + params.defringe_purple_hue_hi * 0.9;
                    if dh >= p_lo && dh <= p_hi {
                        damp = damp.max((params.defringe_purple_amount * 0.05).clamp(0.0, 0.95));
                    }
                }
                if params.defringe_green_amount > 0.1 {
                    let g_lo = 70.0 + params.defringe_green_hue_lo * 0.6;
                    let g_hi = 110.0 + params.defringe_green_hue_hi * 0.8;
                    if dh >= g_lo && dh <= g_hi {
                        damp = damp.max((params.defringe_green_amount * 0.05).clamp(0.0, 0.95));
                    }
                }
                if damp > 0.01 {
                    let (nr, ng, nb) = hsl_to_rgb(dh, ds * (1.0 - damp), dl);
                    pr = nr;
                    pg = ng;
                    pb = nb;
                }
            }

            // --- 10. Color Mixer (8-Band HSL + 3D Point Color) & Basic Vibrance/Saturation ---
            let is_bw = params.convert_to_grayscale
                || params.camera_profile == CameraProfile::AdobeMonochrome;
            let (mut h, mut s, mut l) =
                rgb_to_hsl(pr.clamp(0.0, 1.0), pg.clamp(0.0, 1.0), pb.clamp(0.0, 1.0));
            let mut pc_vis_weight = 0.0f32;

            if is_bw {
                let mut bw_boost = 0.0f32;
                if params.enable_grayscale_mix {
                    for band in 0..8 {
                        let gm = if params.auto_grayscale_mix {
                            [18.0, 24.0, 12.0, -10.0, -16.0, -28.0, -12.0, 8.0][band]
                        } else {
                            params.gray_mixer[band]
                        };
                        let mut dh = (h - DevelopParams::HSL_BAND_HUES[band]).abs();
                        if dh > 180.0 {
                            dh = 360.0 - dh;
                        }
                        if dh < 55.0 {
                            let w = (1.0 - dh / 55.0).powi(2) * s;
                            bw_boost += gm * 0.0045 * w;
                        }
                    }
                }
                let gray = (0.2126 * pr + 0.7152 * pg + 0.0722 * pb + bw_boost).clamp(0.0, 1.0);
                pr = gray;
                pg = gray;
                pb = gray;
            } else {
                // 8-Band HSL & 3D Point Color (gated on `enable_color_adjustments`)
                if params.enable_color_adjustments {
                    let mut h_delta = 0.0f32;
                    let mut s_delta = 0.0f32;
                    let mut l_delta = 0.0f32;
                    // Chroma gate prevents neutral grays/whites from shifting hue or luminance
                    let chroma_gate = smoothstep(0.03, 0.16, s);

                    for band in 0..8 {
                        let mut dh = (h - DevelopParams::HSL_BAND_HUES[band]).abs();
                        if dh > 180.0 {
                            dh = 360.0 - dh;
                        }
                        if dh < 54.0 {
                            let w = (1.0 - dh / 54.0).powi(2);
                            h_delta += params.hue_adjustment[band] * 0.85 * w;
                            s_delta += params.saturation_adjustment[band] * 0.0085 * w;
                            l_delta += params.luminance_adjustment[band] * 0.0026 * w * chroma_gate;
                        }
                    }

                    // Full 3D Point Color (`src_hue`, `src_sat`, `src_lum`, `hue/sat/lum_range`, `range_amount`)
                    for sw_col in &params.point_colors {
                        let mut dh = (h - sw_col.src_hue).abs();
                        if dh > 180.0 {
                            dh = 360.0 - dh;
                        }
                        let scale = (sw_col.range_amount * 0.02).clamp(0.15, 2.0);
                        let h_tol = (sw_col.hue_range * 0.55 + 8.0) * scale;
                        let s_tol = (sw_col.sat_range * 0.008 + 0.10) * scale;
                        let l_tol = (sw_col.lum_range * 0.008 + 0.12) * scale;
                        let ds = (s - sw_col.src_sat).abs();
                        let dl = (l - sw_col.src_lum).abs();
                        if dh < h_tol && ds < s_tol && dl < l_tol {
                            let wh = 1.0 - (dh / h_tol).powi(2);
                            let ws = 1.0 - (ds / s_tol).powi(2);
                            let wl = 1.0 - (dl / l_tol).powi(2);
                            let w = (wh * ws * wl).clamp(0.0, 1.0);
                            pc_vis_weight = pc_vis_weight.max(w);
                            h_delta += sw_col.hue_shift * 0.55 * w;
                            s_delta += sw_col.sat_shift * 0.009 * w;
                            l_delta += sw_col.lum_shift * 0.0028 * w * chroma_gate;
                        }
                    }

                    h = (h + h_delta).rem_euclid(360.0);
                    s = (s * (1.0 + s_delta)).clamp(0.0, 1.0);
                    l = (l + l_delta * (1.0 - l * 0.35)).clamp(0.0, 1.0);
                }

                // Basic Vibrance, Saturation & Camera Profile Saturation (ALWAYS active, independent of Color Mixer toggle!)
                let skin_protect = if h <= 45.0 || h >= 340.0 { 0.45 } else { 1.0 };
                let vib_gain = (params.vibrance * 0.01) * (1.0 - s * 0.7) * skin_protect;
                let sat_gain = (params.saturation * 0.01) + (prof_sat - 1.0) * look_scale * (1.0 - s * 0.35);
                s = (s * (1.0 + vib_gain + sat_gain)).clamp(0.0, 1.0);

                let (nr, ng, nb) = hsl_to_rgb(h, s, l);
                pr = nr;
                pg = ng;
                pb = nb;
            }

            // --- 11. Color Grading (Shadows, Midtones, Highlights, Global + per-wheel `👁` switches) ---
            if params.enable_split_toning && params.is_grading_modified() {
                let bal = (0.5 - params.split_toning_balance * 0.0035).clamp(0.15, 0.85);
                let w_sh = (1.0 - (l / bal).clamp(0.0, 1.0)).powf(1.5);
                let w_hi = (((l - bal) / (1.0 - bal).max(0.1)).clamp(0.0, 1.0)).powf(1.5);
                let w_mid = (1.0 - (w_sh + w_hi) * (1.0 - params.color_grade_blending * 0.005))
                    .clamp(0.0, 1.0);

                let mut tint_vec = |enabled: bool, hue_deg: f32, sat_pct: f32, lum_pct: f32, weight: f32| {
                    if !enabled || weight <= 1e-4 {
                        return;
                    }
                    let (tr, tg, tb) = hsl_to_rgb(hue_deg, 1.0, 0.5);
                    let s_amt = (sat_pct * 0.0035) * weight;
                    pr += (tr - 0.5) * s_amt + (lum_pct * 0.0025 * weight);
                    pg += (tg - 0.5) * s_amt + (lum_pct * 0.0025 * weight);
                    pb += (tb - 0.5) * s_amt + (lum_pct * 0.0025 * weight);
                };

                tint_vec(
                    params.color_grade_enable_shadows,
                    params.split_toning_shadow_hue,
                    params.split_toning_shadow_saturation,
                    params.color_grade_shadow_lum,
                    w_sh,
                );
                tint_vec(
                    params.color_grade_enable_midtones,
                    params.color_grade_midtone_hue,
                    params.color_grade_midtone_sat,
                    params.color_grade_midtone_lum,
                    w_mid,
                );
                tint_vec(
                    params.color_grade_enable_highlights,
                    params.split_toning_highlight_hue,
                    params.split_toning_highlight_saturation,
                    params.color_grade_highlight_lum,
                    w_hi,
                );
                tint_vec(
                    params.color_grade_enable_global,
                    params.color_grade_global_hue,
                    params.color_grade_global_sat,
                    params.color_grade_global_lum,
                    1.0,
                );
            }

            // --- 12. Local Masking (`kMaskGroupBasedCorrectionsKey`) ---
            let mut active_mask_w = 0.0f32;
            if params.enable_mask_group_based_corrections && !params.mask_groups.is_empty() {
                for (m_idx, group) in params.mask_groups.iter().enumerate() {
                    let face_part = src.sample_face_part(nx, ny);
                    let mw = evaluate_mask_group(
                        group, nx, ny, pr, pg, pb, depth_v, subj_v, sky_v, peop_v, face_part,
                    );
                    if mw > 1e-4 {
                        let (nr, ng, nb) =
                            apply_local_adjustments(pr, pg, pb, hp_r, hp_g, hp_b, &group.local, mw);
                        pr = nr;
                        pg = ng;
                        pb = nb;
                    }
                    if opts.active_mask_overlay == Some(m_idx) {
                        active_mask_w = mw;
                    }
                }
            }

            // --- 13. Effects: Post-Crop Vignetting (`override_look_vignette`) & Continuous Film Grain ---
            if params.enable_effects {
                let vig_amt = params.post_crop_vignette_amount
                    * if params.override_look_vignette { 0.75 } else { 1.0 };
                if vig_amt.abs() > 0.1 {
                    let mid = params.post_crop_vignette_midpoint * 0.01;
                    let feather = (params.post_crop_vignette_feather * 0.01).max(0.05);
                    let exp_p = 2.0 + params.post_crop_vignette_roundness * 0.012;
                    let vx = ((u_out - 0.5).abs() * 2.0).powf(exp_p.max(0.6));
                    let vy = ((v_out - 0.5).abs() * 2.0).powf(exp_p.max(0.6));
                    let dist = (vx + vy).powf(1.0 / exp_p.max(0.6));
                    let vw = smoothstep(mid * 0.7, (mid + feather).min(1.35), dist);
                    let amt = vig_amt * 0.0085 * vw;

                    match params.post_crop_vignette_style {
                        VignetteStyle::HighlightPriority => {
                            let hi_prot = 1.0
                                - (params.post_crop_vignette_highlight_contrast * 0.007
                                    * pr.max(pg).max(pb));
                            let f = (1.0 + amt * hi_prot).max(0.0);
                            pr *= f;
                            pg *= f;
                            pb *= f;
                        }
                        VignetteStyle::ColorPriority => {
                            pr = (pr + amt * 0.85).max(0.0);
                            pg = (pg + amt * 0.85).max(0.0);
                            pb = (pb + amt * 0.85).max(0.0);
                        }
                        VignetteStyle::PaintOverlay => {
                            let target = if amt >= 0.0 { 1.0 } else { 0.0 };
                            let a = amt.abs().min(1.0);
                            pr = pr * (1.0 - a) + target * a;
                            pg = pg * (1.0 - a) + target * a;
                            pb = pb * (1.0 - a) + target * a;
                        }
                    }
                }

                // Continuous Bilinear Value-Noise Film Grain (`GrainAmount`, `GrainSize`, `GrainFrequency`)
                if params.grain_amount > 0.5 {
                    let g_radius = 0.7 + params.grain_size * 0.028;
                    let freq_oct = 0.5 + params.grain_frequency * 0.012;
                    let n1 = value_noise_2d(ox as f32 / g_radius, oy as f32 / g_radius, 0x9E3779B9);
                    let n2 = value_noise_2d(
                        (ox as f32 / g_radius) * 2.03 * freq_oct,
                        (oy as f32 / g_radius) * 2.03 * freq_oct,
                        0x85EBCA6B,
                    );
                    let noise = n1 * 0.68 + n2 * 0.32;
                    let mid_weight = 4.0 * l * (1.0 - l) * 0.65 + 0.35;
                    let g_val = noise * (params.grain_amount * 0.0024) * mid_weight;
                    pr += g_val;
                    pg += g_val;
                    pb += g_val;
                }
            }

            // --- 14. Soft Proofing ICC Profile & Paper/Ink Simulation ---
            let mut out_of_gamut = false;
            if opts.soft_proofing {
                let (_, ps, _) = rgb_to_hsl(pr.clamp(0.0, 1.0), pg.clamp(0.0, 1.0), pb.clamp(0.0, 1.0));
                let gamut_limit = match opts.soft_proof_profile {
                    0 => 0.96, // sRGB
                    1 => 0.99, // Adobe RGB (1998)
                    2 => 0.98, // Display P3
                    3 => 1.00, // ProPhoto RGB
                    _ => 0.78, // Matte Fine-Art CMYK Paper
                };
                if ps > gamut_limit || pr > 1.0 || pg > 1.0 || pb > 1.0 {
                    out_of_gamut = true;
                    if opts.soft_proof_perceptual {
                        let compress = gamut_limit / ps.max(gamut_limit);
                        let (gh, _, gl) = rgb_to_hsl(pr.clamp(0.0, 1.0), pg.clamp(0.0, 1.0), pb.clamp(0.0, 1.0));
                        let (cr, cg, cb) = hsl_to_rgb(gh, ps * compress, gl);
                        pr = cr;
                        pg = cg;
                        pb = cb;
                    }
                }
                if opts.simulate_paper_ink || opts.soft_proof_profile >= 4 {
                    pr = 0.038 + pr.clamp(0.0, 1.0) * 0.925;
                    pg = 0.036 + pg.clamp(0.0, 1.0) * 0.918;
                    pb = 0.033 + pb.clamp(0.0, 1.0) * 0.888;
                }
            }

            let raw_max = pr.max(pg).max(pb);
            let raw_min = pr.min(pg).min(pb);

            let clean_r8 = (pr.clamp(0.0, 1.0) * 255.0).round() as u8;
            let clean_g8 = (pg.clamp(0.0, 1.0) * 255.0).round() as u8;
            let clean_b8 = (pb.clamp(0.0, 1.0) * 255.0).round() as u8;

            if let Some(ref mut cr) = clean_slice {
                let cp = (ox * 3) as usize;
                cr[cp] = clean_r8;
                cr[cp + 1] = clean_g8;
                cr[cp + 2] = clean_b8;
            }

            // --- 15. Diagnostic & Interactive Canvas Overlays (Never pollute Histogram!) ---
            let (mut r8, mut g8, mut b8) = (clean_r8, clean_g8, clean_b8);

            if params.lens_blur_active && params.lens_blur_visualize_depth {
                let (dr, dg, db) = hsl_to_rgb(
                    (35.0 + (1.0 - depth_v) * 280.0) % 360.0,
                    0.85,
                    0.15 + (1.0 - depth_v) * 0.72,
                );
                r8 = (dr * 255.0) as u8;
                g8 = (dg * 255.0) as u8;
                b8 = (db * 255.0) as u8;
            } else if opts.show_sharpen_mask_preview {
                let m = (edge_gate.clamp(0.0, 1.0) * 255.0) as u8;
                r8 = m;
                g8 = m;
                b8 = m;
            } else if params.enable_retouch && params.retouch_visualize_spots {
                let gain = 6.0 + params.retouch_visualize_threshold * 0.25;
                let s_val = ((spot_vis_mag * gain).clamp(0.0, 1.0) * 255.0) as u8;
                r8 = s_val;
                g8 = s_val;
                b8 = s_val;
            } else if params.point_color_visualize_range && !params.point_colors.is_empty() {
                let gray = ((0.2126 * pr + 0.7152 * pg + 0.0722 * pb).clamp(0.0, 1.0) * 255.0) as u8;
                let w = pc_vis_weight.clamp(0.0, 1.0);
                r8 = (gray as f32 * (1.0 - w) + r8 as f32 * w) as u8;
                g8 = (gray as f32 * (1.0 - w) + g8 as f32 * w) as u8;
                b8 = (gray as f32 * (1.0 - w) + b8 as f32 * w) as u8;
            } else if params.hdr_edit_mode && params.visualize_hdr_ranges && lin_lum > 1.0 {
                // Cyan -> Yellow -> Magenta HDR stop false-color overlay
                let stops = lin_lum.log2();
                let (hr, hg, hb) = if stops < 1.0 {
                    (0.0, 0.92, 0.96)
                } else if stops < 2.0 {
                    (0.98, 0.90, 0.12)
                } else {
                    (0.98, 0.20, 0.85)
                };
                r8 = (hr * 255.0) as u8;
                g8 = (hg * 255.0) as u8;
                b8 = (hb * 255.0) as u8;
            } else {
                if opts.active_mask_overlay.is_some() && active_mask_w > 0.02 {
                    let alpha = (active_mask_w * 0.58).clamp(0.0, 0.75);
                    r8 = (r8 as f32 * (1.0 - alpha) + 245.0 * alpha) as u8;
                    g8 = (g8 as f32 * (1.0 - alpha) + 36.0 * alpha) as u8;
                    b8 = (b8 as f32 * (1.0 - alpha) + 56.0 * alpha) as u8;
                }
                if opts.soft_proofing && opts.gamut_warning && out_of_gamut {
                    r8 = 20;
                    g8 = 128;
                    b8 = 255;
                } else if opts.show_highlight_clipping && raw_max >= 0.995 {
                    r8 = 255;
                    g8 = 24;
                    b8 = 24;
                } else if opts.show_shadow_clipping && raw_min <= 0.005 {
                    r8 = 28;
                    g8 = 110;
                    b8 = 255;
                }
            }

            let px = (ox * 4) as usize;
            row_bytes[px] = r8;
            row_bytes[px + 1] = g8;
            row_bytes[px + 2] = b8;
            row_bytes[px + 3] = 255;
        }
    };

    if has_overlays {
        rgba_out
            .par_chunks_mut(row_stride)
            .zip(clean_rgb_for_hist.par_chunks_mut(hist_stride))
            .enumerate()
            .for_each(|(oy, (row_b, clean_b))| {
                render_row(oy, row_b, Some(clean_b));
            });
    } else {
        rgba_out
            .par_chunks_mut(row_stride)
            .enumerate()
            .for_each(|(oy, row_b)| {
                render_row(oy, row_b, None);
            });
    }

    // Compute 256-bin R, G, B, Luminance Histograms with Rayon parallel map-reduce!
    #[derive(Clone)]
    struct HistAcc {
        hist_r: [u32; 256],
        hist_g: [u32; 256],
        hist_b: [u32; 256],
        hist_lum: [u32; 256],
        sh_clip: u32,
        hi_clip: u32,
        lum_sum: u64,
    }

    impl Default for HistAcc {
        fn default() -> Self {
            Self {
                hist_r: [0u32; 256],
                hist_g: [0u32; 256],
                hist_b: [0u32; 256],
                hist_lum: [0u32; 256],
                sh_clip: 0,
                hi_clip: 0,
                lum_sum: 0,
            }
        }
    }

    let acc = if has_overlays {
        clean_rgb_for_hist
            .par_chunks(32768)
            .fold(HistAcc::default, |mut acc, chunk| {
                for px in chunk.chunks_exact(3) {
                    let (r, g, b) = (px[0] as usize, px[1] as usize, px[2] as usize);
                    let lum = ((r * 54 + g * 183 + b * 19) >> 8).min(255);
                    acc.hist_r[r] += 1;
                    acc.hist_g[g] += 1;
                    acc.hist_b[b] += 1;
                    acc.hist_lum[lum] += 1;
                    acc.lum_sum += lum as u64;
                    if r <= 2 && g <= 2 && b <= 2 { acc.sh_clip += 1; }
                    if r >= 253 || g >= 253 || b >= 253 { acc.hi_clip += 1; }
                }
                acc
            })
            .reduce(HistAcc::default, |mut a, b| {
                for i in 0..256 {
                    a.hist_r[i] += b.hist_r[i];
                    a.hist_g[i] += b.hist_g[i];
                    a.hist_b[i] += b.hist_b[i];
                    a.hist_lum[i] += b.hist_lum[i];
                }
                a.sh_clip += b.sh_clip;
                a.hi_clip += b.hi_clip;
                a.lum_sum += b.lum_sum;
                a
            })
    } else {
        rgba_out
            .par_chunks(32768)
            .fold(HistAcc::default, |mut acc, chunk| {
                for px in chunk.chunks_exact(4) {
                    let (r, g, b) = (px[0] as usize, px[1] as usize, px[2] as usize);
                    let lum = ((r * 54 + g * 183 + b * 19) >> 8).min(255);
                    acc.hist_r[r] += 1;
                    acc.hist_g[g] += 1;
                    acc.hist_b[b] += 1;
                    acc.hist_lum[lum] += 1;
                    acc.lum_sum += lum as u64;
                    if r <= 2 && g <= 2 && b <= 2 { acc.sh_clip += 1; }
                    if r >= 253 || g >= 253 || b >= 253 { acc.hi_clip += 1; }
                }
                acc
            })
            .reduce(HistAcc::default, |mut a, b| {
                for i in 0..256 {
                    a.hist_r[i] += b.hist_r[i];
                    a.hist_g[i] += b.hist_g[i];
                    a.hist_b[i] += b.hist_b[i];
                    a.hist_lum[i] += b.hist_lum[i];
                }
                a.sh_clip += b.sh_clip;
                a.hi_clip += b.hi_clip;
                a.lum_sum += b.lum_sum;
                a
            })
    };

    let hist_r = acc.hist_r;
    let hist_g = acc.hist_g;
    let hist_b = acc.hist_b;
    let hist_lum = acc.hist_lum;
    let sh_clip = acc.sh_clip;
    let hi_clip = acc.hi_clip;
    let lum_sum = acc.lum_sum;
    let total_px = (out_w * out_h).max(1);

    let mean_lum = (lum_sum as f32 / total_px as f32) / 255.0;
    let p5_target = (total_px as f32 * 0.05) as u32;
    let p95_target = (total_px as f32 * 0.95) as u32;
    let mut cum = 0u32;
    let mut p5_bin = 12usize;
    let mut p95_bin = 240usize;
    let mut found_p5 = false;
    for (bin, &cnt) in hist_lum.iter().enumerate() {
        cum += cnt;
        if !found_p5 && cum >= p5_target {
            p5_bin = bin;
            found_p5 = true;
        }
        if cum >= p95_target {
            p95_bin = bin;
            break;
        }
    }

    RenderedFrame {
        width: out_w,
        height: out_h,
        rgba: rgba_out,
        hist_r,
        hist_g,
        hist_b,
        hist_lum,
        shadow_clipped_pct: (sh_clip as f32 / total_px as f32) * 100.0,
        highlight_clipped_pct: (hi_clip as f32 / total_px as f32) * 100.0,
        mean_lum,
        p5_lum: p5_bin as f32 / 255.0,
        p95_lum: p95_bin as f32 / 255.0,
    }
}
