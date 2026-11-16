// ============================================================================
// LightRaw AI — Intelligent Reference Photo Color, White Balance & Tone Transfer
// Perceptual OKLab Luminance-CDF + 8-Zone Asymmetric 2x2 Monge-Kantorovich Color Transfer
// Encodes a smooth 25x25x25 3D Color Look LUT with Targeted Skin Safety & .cube Export
// ============================================================================

use crate::params::DevelopParams;
use crate::pipeline::{render_develop_pipeline, RawSourceImage, RenderOptions};

pub const MATCH_LUT_DIM: usize = 25;

#[derive(Debug, Clone, Default)]
pub struct ImageColorStats {
    pub mean_lum: f32,
    pub p05_lum: f32,
    pub p25_lum: f32,
    pub p50_lum: f32,
    pub p75_lum: f32,
    pub p95_lum: f32,
    pub mean_sat: f32,
    pub mid_rg_ratio: f32,
    pub mid_bg_ratio: f32,
    pub shadow_hue: f32,
    pub shadow_sat: f32,
    pub highlight_hue: f32,
    pub highlight_sat: f32,

    // Portrait & Skin-Tone Intelligence (Face & Body)
    pub skin_pixel_count: usize,
    pub skin_median_lum: f32,
    pub skin_mean_rg: f32,
    pub skin_mean_bg: f32,
    pub skin_mean_hue: f32,
    pub skin_mean_sat: f32,

    // Studio Backdrop / Background Color
    pub bg_pixel_count: usize,
    pub bg_median_lum: f32,
    pub bg_mean_hue: f32,
    pub bg_mean_sat: f32,
}

#[inline]
pub fn srgb_to_linear_ch(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[inline]
pub fn linear_to_srgb_ch(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

#[inline]
fn linear_to_perceptual(c: f32) -> f32 {
    let c = c.max(0.0);
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Converts display sRGB `(r, g, b)` in `0..=1` into perceptual OKLab `(L, a, b)`.
#[inline]
pub fn srgb_to_oklab(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let rl = srgb_to_linear_ch(r);
    let gl = srgb_to_linear_ch(g);
    let bl = srgb_to_linear_ch(b);

    let l = (0.41222147 * rl + 0.53633255 * gl + 0.05144599 * bl).max(0.0);
    let m = (0.21190350 * rl + 0.68069955 * gl + 0.10739696 * bl).max(0.0);
    let s = (0.08830246 * rl + 0.28171884 * gl + 0.62997870 * bl).max(0.0);

    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    let ok_l = 0.21045426 * l_ + 0.79361779 * m_ - 0.00407205 * s_;
    let ok_a = 1.97799850 * l_ - 2.42859220 * m_ + 0.45059371 * s_;
    let ok_b = 0.02590404 * l_ + 0.78277177 * m_ - 0.80867577 * s_;
    (ok_l, ok_a, ok_b)
}

#[inline]
fn oklab_to_linear_raw(ok_l: f32, ok_a: f32, ok_b: f32) -> (f32, f32, f32) {
    let l_ = ok_l + 0.39633778 * ok_a + 0.21580376 * ok_b;
    let m_ = ok_l - 0.10556135 * ok_a - 0.06385417 * ok_b;
    let s_ = ok_l - 0.08948418 * ok_a - 1.29148555 * ok_b;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    let r = 4.07674166 * l - 3.30771159 * m + 0.23096993 * s;
    let g = -1.26843800 * l + 2.60975740 * m - 0.34131940 * s;
    let b = -0.00419609 * l - 0.70341861 * m + 1.70761470 * s;
    (r, g, b)
}

/// Converts OKLab `(L, a, b)` back to in-gamut display sRGB `[0..=1]` with luminance-preserving
/// soft chroma projection so highlights and saturated tones never hue-shift on clamp.
#[inline]
pub fn oklab_to_srgb_gamut(ok_l: f32, ok_a: f32, ok_b: f32) -> (f32, f32, f32) {
    let l_clamped = ok_l.clamp(0.0, 1.0);
    let (mut r, mut g, mut b) = oklab_to_linear_raw(l_clamped, ok_a, ok_b);
    if r < 0.0 || g < 0.0 || b < 0.0 || r > 1.0 || g > 1.0 || b > 1.0 {
        let mut lo = 0.0f32;
        let mut hi = 1.0f32;
        for _ in 0..6 {
            let mid = 0.5 * (lo + hi);
            let (tr, tg, tb) = oklab_to_linear_raw(l_clamped, ok_a * mid, ok_b * mid);
            if tr >= -1e-4 && tg >= -1e-4 && tb >= -1e-4 && tr <= 1.0001 && tg <= 1.0001 && tb <= 1.0001 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let (cr, cg, cb) = oklab_to_linear_raw(l_clamped, ok_a * lo, ok_b * lo);
        r = cr;
        g = cg;
        b = cb;
    }
    (linear_to_srgb_ch(r), linear_to_srgb_ch(g), linear_to_srgb_ch(b))
}

/// Fast trilinear 3D LUT lookup for flattened `dim^3 * 3` sRGB LUT buffers.
#[inline]
pub fn sample_lut_3d(lut: &[f32], dim: usize, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    if dim < 2 || lut.len() != dim * dim * dim * 3 {
        return (r, g, b);
    }
    let scale = (dim - 1) as f32;
    let rf = (r.clamp(0.0, 1.0) * scale).clamp(0.0, scale);
    let gf = (g.clamp(0.0, 1.0) * scale).clamp(0.0, scale);
    let bf = (b.clamp(0.0, 1.0) * scale).clamp(0.0, scale);

    let r0 = (rf.floor() as usize).min(dim - 1);
    let g0 = (gf.floor() as usize).min(dim - 1);
    let b0 = (bf.floor() as usize).min(dim - 1);
    let r1 = (r0 + 1).min(dim - 1);
    let g1 = (g0 + 1).min(dim - 1);
    let b1 = (b0 + 1).min(dim - 1);

    let dr = rf - r0 as f32;
    let dg = gf - g0 as f32;
    let db = bf - b0 as f32;

    let d2 = dim * dim;
    let at = |ri: usize, gi: usize, bi: usize| -> (f32, f32, f32) {
        let base = (bi * d2 + gi * dim + ri) * 3;
        (lut[base], lut[base + 1], lut[base + 2])
    };
    let lerp3 = |c0: (f32, f32, f32), c1: (f32, f32, f32), t: f32| -> (f32, f32, f32) {
        (
            c0.0 + (c1.0 - c0.0) * t,
            c0.1 + (c1.1 - c0.1) * t,
            c0.2 + (c1.2 - c0.2) * t,
        )
    };

    let c00 = lerp3(at(r0, g0, b0), at(r1, g0, b0), dr);
    let c10 = lerp3(at(r0, g1, b0), at(r1, g1, b0), dr);
    let c01 = lerp3(at(r0, g0, b1), at(r1, g0, b1), dr);
    let c11 = lerp3(at(r0, g1, b1), at(r1, g1, b1), dr);

    let c0 = lerp3(c00, c10, dg);
    let c1 = lerp3(c01, c11, dg);
    lerp3(c0, c1, db)
}

/// Applies the matched 3D Color Look LUT with independent `Tonal` (0..100) and `Warna` (0..100)
/// OKLab blend weights so users can dial contrast/curve and palette strength live in real time.
#[inline]
pub fn apply_color_match_lut(
    r: f32,
    g: f32,
    b: f32,
    lut: &[f32],
    dim: usize,
    tone_amount_pct: f32,
    color_amount_pct: f32,
) -> (f32, f32, f32) {
    let wt = (tone_amount_pct / 100.0).clamp(0.0, 1.0);
    let wc = (color_amount_pct / 100.0).clamp(0.0, 1.0);
    if (wt <= 1e-4 && wc <= 1e-4) || dim < 2 || lut.len() != dim * dim * dim * 3 {
        return (r, g, b);
    }

    let (lr, lg, lb) = sample_lut_3d(lut, dim, r, g, b);
    if wt >= 0.998 && wc >= 0.998 {
        return (lr, lg, lb);
    }

    let (lin_l, lin_a, lin_b) = srgb_to_oklab(r, g, b);
    let (lut_l, lut_a, lut_b) = srgb_to_oklab(lr, lg, lb);

    let out_l = lin_l + (lut_l - lin_l) * wt;
    let out_a = lin_a + (lut_a - lin_a) * wc;
    let out_b = lin_b + (lut_b - lin_b) * wc;
    oklab_to_srgb_gamut(out_l, out_a, out_b)
}

/// Exports the matched Reference LUT (including current Tonal & Warna strength) as an Adobe `.cube` file.
pub fn export_match_cube_lut(params: &DevelopParams) -> Option<String> {
    if !params.has_color_match_lut() {
        return None;
    }
    let d = params.match_lut_dim as usize;
    let mut out = String::with_capacity(d * d * d * 28 + 256);
    let title = if params.match_ref_name.trim().is_empty() {
        "LightRaw Reference Match"
    } else {
        params.match_ref_name.trim()
    };
    out.push_str(&format!("TITLE \"{}\" (LightRaw)\n", title.replace('"', "")));
    out.push_str(&format!("LUT_3D_SIZE {}\n", d));
    out.push_str("DOMAIN_MIN 0.0 0.0 0.0\n");
    out.push_str("DOMAIN_MAX 1.0 1.0 1.0\n\n");

    let wt = (params.match_tone_amount / 100.0).clamp(0.0, 1.0);
    let wc = (params.match_color_amount / 100.0).clamp(0.0, 1.0);
    let denom = (d - 1) as f32;
    let d2 = d * d;

    for bi in 0..d {
        let bf = bi as f32 / denom;
        for gi in 0..d {
            let gf = gi as f32 / denom;
            for ri in 0..d {
                let rf = ri as f32 / denom;
                let base = (bi * d2 + gi * d + ri) * 3;
                let (mr, mg, mb) = (
                    params.match_lut_data[base],
                    params.match_lut_data[base + 1],
                    params.match_lut_data[base + 2],
                );
                let (or, og, ob) = if wt >= 0.998 && wc >= 0.998 {
                    (mr, mg, mb)
                } else {
                    let (il, ia, ib) = srgb_to_oklab(rf, gf, bf);
                    let (ll, la, lb) = srgb_to_oklab(mr, mg, mb);
                    oklab_to_srgb_gamut(il + (ll - il) * wt, ia + (la - ia) * wc, ib + (lb - ib) * wc)
                };
                out.push_str(&format!("{:.6} {:.6} {:.6}\n", or, og, ob));
            }
        }
    }
    Some(out)
}

#[inline]
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0).max(1e-5)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Converts RGB chromaticity deviation into a Hue [0..360) and Saturation [0..100]
#[inline]
fn rgb_to_hue_sat(r: f32, g: f32, b: f32) -> (f32, f32) {
    let max_c = r.max(g).max(b);
    let min_c = r.min(g).min(b);
    let delta = max_c - min_c;
    if delta < 1e-4 || max_c < 1e-4 {
        return (0.0, 0.0);
    }

    let hue = if (max_c - r).abs() < 1e-5 {
        60.0 * (((g - b) / delta) % 6.0)
    } else if (max_c - g).abs() < 1e-5 {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let hue = (hue + 360.0) % 360.0;
    let sat = (delta / max_c * 100.0).clamp(0.0, 100.0);
    (hue, sat)
}

#[inline]
fn is_skin_srgb(r: f32, g: f32, b: f32, lenient_flat: bool) -> bool {
    let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let (hue, sat) = rgb_to_hue_sat(r, g, b);
    if lenient_flat {
        (0.16..=0.90).contains(&lum)
            && r > g
            && g > b
            && (r - g) >= 0.006
            && (g - b) >= 0.004
            && (r - b) >= 0.015
            && (r - b) <= 0.65
            && (8.0..=52.0).contains(&hue)
            && (4.0..=75.0).contains(&sat)
    } else {
        (0.15..=0.90).contains(&lum)
            && r > g
            && g > b
            && (r - g) >= 0.02
            && (g - b) >= 0.012
            && (r - b) >= 0.05
            && (r - b) <= 0.65
            && (10.0..=48.0).contains(&hue)
            && (12.0..=75.0).contains(&sat)
    }
}

/// Computes comprehensive statistical color and tone descriptors in a normalized perceptual space.
/// Accurately bridges 32-bit linear RAW buffers and display-referred JPEG/TIFF buffers.
pub fn extract_image_color_stats(raw: &RawSourceImage) -> ImageColorStats {
    let total_pixels = (raw.width * raw.height) as usize;
    if total_pixels == 0 || raw.rgb.is_empty() {
        return ImageColorStats::default();
    }

    let sample_target = 12000usize;
    let step = (total_pixels / sample_target).max(1);

    let base_gain = if raw.is_scene_referred {
        let raw_ev_lift = raw.baseline_ev.clamp(-1.0, 1.5) * 0.35 + 0.42;
        2.0f32.powf(raw_ev_lift)
    } else {
        1.0f32
    };

    let cap = total_pixels / step + 16;
    let mut lums: Vec<f32> = Vec::with_capacity(cap);
    let mut sats: Vec<f32> = Vec::with_capacity(cap);

    let mut mid_sum_r = 0.0f64;
    let mut mid_sum_g = 0.0f64;
    let mut mid_sum_b = 0.0f64;
    let mut mid_count = 0u64;

    let mut shd_sum_r = 0.0f64;
    let mut shd_sum_g = 0.0f64;
    let mut shd_sum_b = 0.0f64;
    let mut shd_count = 0u64;

    let mut hl_sum_r = 0.0f64;
    let mut hl_sum_g = 0.0f64;
    let mut hl_sum_b = 0.0f64;
    let mut hl_count = 0u64;

    let mut skin_lums: Vec<f32> = Vec::with_capacity(cap / 4);
    let mut skin_sum_rg = 0.0f64;
    let mut skin_sum_bg = 0.0f64;
    let mut skin_sum_hue = 0.0f64;
    let mut skin_sum_sat = 0.0f64;

    let mut bg_lums: Vec<f32> = Vec::with_capacity(cap / 3);
    let mut bg_sum_hue = 0.0f64;
    let mut bg_sum_sat = 0.0f64;

    let width = raw.width.max(1) as usize;
    let height = raw.height.max(1) as usize;

    for px_idx in (0..total_pixels).step_by(step) {
        let buf_idx = px_idx * 3;
        if buf_idx + 2 >= raw.rgb.len() {
            break;
        }

        let r_lin = raw.rgb[buf_idx] * base_gain;
        let g_lin = raw.rgb[buf_idx + 1] * base_gain;
        let b_lin = raw.rgb[buf_idx + 2] * base_gain;

        let r = linear_to_perceptual(r_lin).clamp(0.0, 1.0);
        let g = linear_to_perceptual(g_lin).clamp(0.0, 1.0);
        let b = linear_to_perceptual(b_lin).clamp(0.0, 1.0);

        let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        let (hue, sat) = rgb_to_hue_sat(r, g, b);

        lums.push(lum);
        sats.push(sat);

        let px_x = px_idx % width;
        let px_y = px_idx / width;
        let nx = px_x as f32 / width as f32;
        let ny = px_y as f32 / height as f32;

        let face_part = raw.sample_face_part(nx, ny);
        let is_ai_skin = face_part == 1;
        let is_skin = is_ai_skin || is_skin_srgb(r, g, b, false);

        if is_skin {
            skin_lums.push(lum);
            if g > 1e-4 {
                skin_sum_rg += (r / g) as f64;
                skin_sum_bg += (b / g) as f64;
            }
            skin_sum_hue += hue as f64;
            skin_sum_sat += sat as f64;
        }

        let is_studio_backdrop = (180.0..=250.0).contains(&hue) && sat > 25.0 && lum > 0.10;
        if is_studio_backdrop && !is_skin {
            bg_lums.push(lum);
            bg_sum_hue += hue as f64;
            bg_sum_sat += sat as f64;
        }

        if lum > 0.02 && lum < 0.30 {
            shd_sum_r += r as f64;
            shd_sum_g += g as f64;
            shd_sum_b += b as f64;
            shd_count += 1;
        } else if lum > 0.70 && lum < 0.98 {
            hl_sum_r += r as f64;
            hl_sum_g += g as f64;
            hl_sum_b += b as f64;
            hl_count += 1;
        }

        if (0.20..=0.80).contains(&lum) {
            mid_sum_r += r as f64;
            mid_sum_g += g as f64;
            mid_sum_b += b as f64;
            mid_count += 1;
        }
    }

    if lums.is_empty() {
        return ImageColorStats::default();
    }

    lums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = lums.len();
    let p05 = lums[(n as f32 * 0.05) as usize];
    let p25 = lums[(n as f32 * 0.25) as usize];
    let p50 = lums[(n as f32 * 0.50) as usize];
    let p75 = lums[(n as f32 * 0.75) as usize];
    let p95 = lums[((n as f32 * 0.95) as usize).min(n - 1)];

    let sum_l: f32 = lums.iter().sum();
    let mean_lum = sum_l / n as f32;

    let sum_s: f32 = sats.iter().sum();
    let mean_sat = sum_s / sats.len().max(1) as f32;

    let (mid_rg, mid_bg) = if mid_count > 0 && mid_sum_g > 1e-6 {
        (
            (mid_sum_r / mid_sum_g) as f32,
            (mid_sum_b / mid_sum_g) as f32,
        )
    } else {
        (1.0, 1.0)
    };

    let (shadow_hue, shadow_sat) = if shd_count > 0 {
        let sr = (shd_sum_r / shd_count as f64) as f32;
        let sg = (shd_sum_g / shd_count as f64) as f32;
        let sb = (shd_sum_b / shd_count as f64) as f32;
        rgb_to_hue_sat(sr, sg, sb)
    } else {
        (0.0, 0.0)
    };

    let (highlight_hue, highlight_sat) = if hl_count > 0 {
        let hr = (hl_sum_r / hl_count as f64) as f32;
        let hg = (hl_sum_g / hl_count as f64) as f32;
        let hb = (hl_sum_b / hl_count as f64) as f32;
        rgb_to_hue_sat(hr, hg, hb)
    } else {
        (0.0, 0.0)
    };

    let skin_count = skin_lums.len();
    let (skin_median_lum, skin_mean_rg, skin_mean_bg, skin_mean_hue, skin_mean_sat) =
        if skin_count > 0 {
            skin_lums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let med = skin_lums[skin_count / 2];
            (
                med,
                (skin_sum_rg / skin_count as f64) as f32,
                (skin_sum_bg / skin_count as f64) as f32,
                (skin_sum_hue / skin_count as f64) as f32,
                (skin_sum_sat / skin_count as f64) as f32,
            )
        } else {
            (0.0, 1.0, 1.0, 0.0, 0.0)
        };

    let bg_count = bg_lums.len();
    let (bg_median_lum, bg_mean_hue, bg_mean_sat) = if bg_count > 0 {
        bg_lums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let med = bg_lums[bg_count / 2];
        (
            med,
            (bg_sum_hue / bg_count as f64) as f32,
            (bg_sum_sat / bg_count as f64) as f32,
        )
    } else {
        (0.0, 0.0, 0.0)
    };

    ImageColorStats {
        mean_lum,
        p05_lum: p05,
        p25_lum: p25,
        p50_lum: p50,
        p75_lum: p75,
        p95_lum: p95,
        mean_sat,
        mid_rg_ratio: mid_rg,
        mid_bg_ratio: mid_bg,
        shadow_hue,
        shadow_sat,
        highlight_hue,
        highlight_sat,
        skin_pixel_count: skin_count,
        skin_median_lum,
        skin_mean_rg,
        skin_mean_bg,
        skin_mean_hue,
        skin_mean_sat,
        bg_pixel_count: bg_count,
        bg_median_lum,
        bg_mean_hue,
        bg_mean_sat,
    }
}

/// Downsamples a `RawSourceImage` to at most `max_dim` before rendering pre-LUT baseline pixels.
fn downsample_raw_for_match(src: &RawSourceImage, max_dim: u32) -> RawSourceImage {
    let cur_max = src.width.max(src.height);
    if cur_max <= max_dim || cur_max == 0 {
        return src.clone();
    }
    let scale = max_dim as f32 / cur_max as f32;
    let nw = ((src.width as f32 * scale).round() as u32).max(16);
    let nh = ((src.height as f32 * scale).round() as u32).max(16);
    let mut rgb = vec![0.0f32; (nw * nh * 3) as usize];
    for y in 0..nh {
        let sy = ((y as f32 + 0.5) / nh as f32 * src.height as f32) as usize;
        let sy = sy.min(src.height.saturating_sub(1) as usize);
        for x in 0..nw {
            let sx = ((x as f32 + 0.5) / nw as f32 * src.width as f32) as usize;
            let sx = sx.min(src.width.saturating_sub(1) as usize);
            let si = (sy * src.width as usize + sx) * 3;
            let di = (y as usize * nw as usize + x as usize) * 3;
            rgb[di] = src.rgb[si];
            rgb[di + 1] = src.rgb[si + 1];
            rgb[di + 2] = src.rgb[si + 2];
        }
    }
    RawSourceImage::from_linear_rgb(
        nw,
        nh,
        nw,
        nh,
        rgb,
        src.as_shot_temp,
        src.as_shot_tint,
        src.baseline_ev,
        src.is_scene_referred,
    )
}

/// Renders up to ~12,000 representative perceptual sRGB triplets for Color Match fitting.
fn sample_rendered_srgb_samples(raw: &RawSourceImage, params: Option<&DevelopParams>) -> Vec<(f32, f32, f32)> {
    let small = downsample_raw_for_match(raw, 160);
    let total = (small.width * small.height) as usize;
    if total == 0 {
        return Vec::new();
    }

    if let Some(p_orig) = params {
        let mut p = p_orig.clone();
        p.clear_color_match_lut();
        p.post_crop_vignette_amount = 0.0;
        p.grain_amount = 0.0;
        let frame = render_develop_pipeline(
            &small,
            &p,
            RenderOptions {
                apply_crop: false,
                is_draft_interactive: true,
                ..RenderOptions::default()
            },
        );
        let n = frame.rgba.len() / 4;
        let step = (n / 12000).max(1);
        let mut out = Vec::with_capacity(n / step + 4);
        for i in (0..n).step_by(step) {
            let o = i * 4;
            out.push((
                frame.rgba[o] as f32 / 255.0,
                frame.rgba[o + 1] as f32 / 255.0,
                frame.rgba[o + 2] as f32 / 255.0,
            ));
        }
        return out;
    }

    let step = (total / 12000).max(1);
    let base_gain = if small.is_scene_referred {
        2.0f32.powf(small.baseline_ev.clamp(-1.0, 1.5) * 0.35 + 0.42)
    } else {
        1.0
    };
    let mut out = Vec::with_capacity(total / step + 4);
    for i in (0..total).step_by(step) {
        let o = i * 3;
        if o + 2 >= small.rgb.len() {
            break;
        }
        out.push((
            linear_to_perceptual(small.rgb[o] * base_gain).clamp(0.0, 1.0),
            linear_to_perceptual(small.rgb[o + 1] * base_gain).clamp(0.0, 1.0),
            linear_to_perceptual(small.rgb[o + 2] * base_gain).clamp(0.0, 1.0),
        ));
    }
    out
}

/// Extracts 10 representative RGB swatches sorted from deep shadows through midtones to highlights.
pub fn extract_reference_palette(samples: &[(f32, f32, f32)], count: usize) -> Vec<[u8; 3]> {
    if samples.is_empty() || count == 0 {
        return Vec::new();
    }
    let mut labs: Vec<(f32, f32, f32, f32, f32, f32)> = samples
        .iter()
        .map(|&(r, g, b)| {
            let (l, a, okb) = srgb_to_oklab(r, g, b);
            (l, a, okb, r, g, b)
        })
        .collect();
    labs.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap_or(std::cmp::Ordering::Equal));

    let n = labs.len();
    let mut palette = Vec::with_capacity(count);
    for k in 0..count {
        let start = (k * n) / count;
        let end = (((k + 1) * n) / count).max(start + 1).min(n);
        let mut sum_w = 0.0f64;
        let mut sum_r = 0.0f64;
        let mut sum_g = 0.0f64;
        let mut sum_b = 0.0f64;
        for &(_l, a, okb, r, g, b) in &labs[start..end] {
            let chroma = (a * a + okb * okb).sqrt();
            // Slightly prefer chromatic swatches in midtones so accents appear clearly
            let w = (0.35 + chroma * 6.0) as f64;
            sum_w += w;
            sum_r += r as f64 * w;
            sum_g += g as f64 * w;
            sum_b += b as f64 * w;
        }
        let mr = ((sum_r / sum_w.max(1e-6)) * 255.0).round().clamp(0.0, 255.0) as u8;
        let mg = ((sum_g / sum_w.max(1e-6)) * 255.0).round().clamp(0.0, 255.0) as u8;
        let mb = ((sum_b / sum_w.max(1e-6)) * 255.0).round().clamp(0.0, 255.0) as u8;
        palette.push([mr, mg, mb]);
    }
    palette
}

#[derive(Clone, Copy, Default)]
struct Mat2 {
    m00: f32,
    m01: f32,
    m10: f32,
    m11: f32,
}

impl Mat2 {
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self {
            m00: self.m00 * rhs.m00 + self.m01 * rhs.m10,
            m01: self.m00 * rhs.m01 + self.m01 * rhs.m11,
            m10: self.m10 * rhs.m00 + self.m11 * rhs.m10,
            m11: self.m10 * rhs.m01 + self.m11 * rhs.m11,
        }
    }

    #[inline]
    fn apply(self, x: f32, y: f32) -> (f32, f32) {
        (self.m00 * x + self.m01 * y, self.m10 * x + self.m11 * y)
    }
}

/// Eigen-decomposition of a 2x2 symmetric matrix `[[a, b], [b, c]]`, returning `(cos, sin, lam1, lam2)`.
#[inline]
fn eigen_sym_2x2(a: f32, b: f32, c: f32) -> (f32, f32, f32, f32) {
    let tr = 0.5 * (a + c);
    let diff = 0.5 * (a - c);
    let rad = (diff * diff + b * b).sqrt();
    let l1 = (tr + rad).max(0.0);
    let l2 = (tr - rad).max(0.0);
    let theta = if b.abs() < 1e-10 && diff.abs() < 1e-10 {
        0.0
    } else {
        0.5 * b.atan2(diff)
    };
    (theta.cos(), theta.sin(), l1, l2)
}

#[inline]
fn reconstruct_sym_2x2(cos_t: f32, sin_t: f32, d1: f32, d2: f32) -> Mat2 {
    let c2 = cos_t * cos_t;
    let s2 = sin_t * sin_t;
    let cs = cos_t * sin_t;
    Mat2 {
        m00: d1 * c2 + d2 * s2,
        m01: (d1 - d2) * cs,
        m10: (d1 - d2) * cs,
        m11: d1 * s2 + d2 * c2,
    }
}

/// Computes regularized 2x2 Monge-Kantorovich chromatic covariance transport matrix:
/// `T = Sigma_t^(-1/2) * (Sigma_t^(1/2) * Sigma_r * Sigma_t^(1/2))^(1/2) * Sigma_t^(-1/2)`.
fn compute_mk_2x2(
    t_aa: f32,
    t_ab: f32,
    t_bb: f32,
    r_aa: f32,
    r_ab: f32,
    r_bb: f32,
    eps_t: f32,
    eps_r: f32,
    min_gain: f32,
    max_gain: f32,
) -> Mat2 {
    let (ct, st, tl1, tl2) = eigen_sym_2x2(t_aa + eps_t, t_ab, t_bb + eps_t);
    let sqrt_t = reconstruct_sym_2x2(ct, st, tl1.sqrt(), tl2.sqrt());
    let inv_sqrt_t = reconstruct_sym_2x2(ct, st, 1.0 / tl1.sqrt(), 1.0 / tl2.sqrt());

    let sig_r = Mat2 {
        m00: r_aa + eps_r,
        m01: r_ab,
        m10: r_ab,
        m11: r_bb + eps_r,
    };
    let inner = sqrt_t.mul(sig_r).mul(sqrt_t);
    let (cp, sp, pl1, pl2) = eigen_sym_2x2(inner.m00, 0.5 * (inner.m01 + inner.m10), inner.m11);
    let sqrt_inner = reconstruct_sym_2x2(cp, sp, pl1.sqrt(), pl2.sqrt());

    let t_mat = inv_sqrt_t.mul(sqrt_inner).mul(inv_sqrt_t);
    // Softly bound principal gains to prevent amplifying shadow banding / compression noise
    let sym_ab = 0.5 * (t_mat.m01 + t_mat.m10);
    let (cm, sm, ml1, ml2) = eigen_sym_2x2(t_mat.m00, sym_ab, t_mat.m11);
    reconstruct_sym_2x2(
        cm,
        sm,
        ml1.clamp(min_gain, max_gain),
        ml2.clamp(min_gain, max_gain),
    )
}

#[inline]
fn quantile_from_sorted(sorted: &[f32], q: f32) -> f32 {
    if sorted.is_empty() {
        return 0.5;
    }
    let idx_f = q.clamp(0.0, 1.0) * (sorted.len() - 1) as f32;
    let i0 = idx_f.floor() as usize;
    let i1 = (i0 + 1).min(sorted.len() - 1);
    let frac = idx_f - i0 as f32;
    sorted[i0] * (1.0 - frac) + sorted[i1] * frac
}

#[inline]
fn cdf_rank(sorted: &[f32], x: f32) -> f32 {
    if sorted.len() <= 1 {
        return x.clamp(0.0, 1.0);
    }
    let lo = sorted.partition_point(|&v| v < x);
    let hi = sorted.partition_point(|&v| v <= x);
    if lo < hi {
        // Midpoint quantile for tied plateaus so flat patches/skin map smoothly at center rank
        return (0.5 * (lo + hi - 1) as f32) / (sorted.len() - 1) as f32;
    }
    if lo == 0 {
        return 0.0;
    }
    if lo >= sorted.len() {
        return 1.0;
    }
    let v0 = sorted[lo - 1];
    let v1 = sorted[lo];
    let frac = if (v1 - v0).abs() > 1e-6 {
        ((x - v0) / (v1 - v0)).clamp(0.0, 1.0)
    } else {
        0.5
    };
    ((lo - 1) as f32 + frac) / (sorted.len() - 1) as f32
}

/// Computes weighted median (`q = 0.50`) of `(val, weight)` pairs cleanly.
fn weighted_median(pairs: &mut Vec<(f32, f32)>) -> f32 {
    if pairs.is_empty() {
        return 0.0;
    }
    pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let total_w: f64 = pairs.iter().map(|p| p.1 as f64).sum();
    if total_w <= 1e-6 {
        return pairs[pairs.len() / 2].0;
    }
    let half = 0.5 * total_w;
    let mut acc = 0.0f64;
    for &(v, w) in pairs.iter() {
        acc += w as f64;
        if acc >= half {
            return v;
        }
    }
    pairs.last().map(|p| p.0).unwrap_or(0.0)
}

/// Core 3D LUT generator matching OKLab luminance CDF (with highlight rolloff protection)
/// and 8-zone median-anchored Monge-Kantorovich chromatic covariance transport.
fn build_match_3d_lut(
    ref_srgb: &[(f32, f32, f32)],
    tgt_srgb: &[(f32, f32, f32)],
    target_is_scene_referred: bool,
    force_raw_flat: Option<bool>,
) -> (Vec<f32>, bool) {
    let dim = MATCH_LUT_DIM;
    if ref_srgb.is_empty() || tgt_srgb.is_empty() {
        return (Vec::new(), false);
    }

    let mut ref_labs = Vec::with_capacity(ref_srgb.len());
    let mut ref_l_sorted = Vec::with_capacity(ref_srgb.len());
    let mut ref_skin_l = Vec::new();
    let mut ref_skin_a_sum = 0.0f64;
    let mut ref_skin_b_sum = 0.0f64;

    for &(r, g, b) in ref_srgb {
        let (l, a, okb) = srgb_to_oklab(r, g, b);
        ref_labs.push((l, a, okb));
        ref_l_sorted.push(l);
        if is_skin_srgb(r, g, b, false) {
            ref_skin_l.push(l);
            ref_skin_a_sum += a as f64;
            ref_skin_b_sum += okb as f64;
        }
    }
    ref_l_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mut tgt_l_sorted = Vec::with_capacity(tgt_srgb.len());
    for &(r, g, b) in tgt_srgb {
        let (l, _, _) = srgb_to_oklab(r, g, b);
        tgt_l_sorted.push(l);
    }
    tgt_l_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let tgt_p05 = quantile_from_sorted(&tgt_l_sorted, 0.05);
    let tgt_p95 = quantile_from_sorted(&tgt_l_sorted, 0.95);
    let tgt_spread = (tgt_p95 - tgt_p05).max(0.02);
    let is_flat = force_raw_flat.unwrap_or_else(|| {
        target_is_scene_referred || tgt_spread < 0.48 || tgt_p05 > 0.14
    });

    let mut tgt_labs = Vec::with_capacity(tgt_srgb.len());
    let mut tgt_skin_l = Vec::new();
    for &(r, g, b) in tgt_srgb {
        let (l, a, okb) = srgb_to_oklab(r, g, b);
        tgt_labs.push((l, a, okb));
        if is_skin_srgb(r, g, b, is_flat) {
            tgt_skin_l.push(l);
        }
    }

    // --- Step 1: 256-entry Smooth Filmic Histogram CDF Luminance Curve ---
    // Matches Reference p05/p50 cleanly while preserving smooth upper highlight rolloff (zero knee kinks).
    const L_BINS: usize = 256;
    let mut l_raw = [0.0f32; L_BINS];
    let t005 = quantile_from_sorted(&tgt_l_sorted, 0.005);
    let t01 = quantile_from_sorted(&tgt_l_sorted, 0.01);
    let t99 = quantile_from_sorted(&tgt_l_sorted, 0.99);
    let t995 = quantile_from_sorted(&tgt_l_sorted, 0.995);
    let r01 = quantile_from_sorted(&ref_l_sorted, 0.01);
    let r05 = quantile_from_sorted(&ref_l_sorted, 0.05);
    let r92 = quantile_from_sorted(&ref_l_sorted, 0.92);
    let r99 = quantile_from_sorted(&ref_l_sorted, 0.99);
    let span_t = (t995 - t005).max(0.08);

    for i in 0..L_BINS {
        let x = i as f32 / (L_BINS - 1) as f32;
        let q = cdf_rank(&tgt_l_sorted, x);
        let l_norm = ((x - t005) / span_t).clamp(0.0, 1.0);
        let l_cdf = if x < t01 {
            let k = (x / t01.max(0.015)).clamp(0.0, 1.0);
            let k_smooth = k * k * (3.0 - 2.0 * k);
            let target_lo = (r01 * 0.55).min(0.035);
            target_lo + (r01 - target_lo) * k_smooth
        } else if x > t99 {
            let k = ((x - t99) / (1.0 - t99).max(0.015)).clamp(0.0, 1.0);
            let k_smooth = k * k * (3.0 - 2.0 * k);
            let target_hi = r99.max(0.92);
            r99 + (target_hi - r99) * k_smooth
        } else {
            quantile_from_sorted(&ref_l_sorted, q)
        };

        // Smooth filmic highlight headroom ("dorongan highlight dilembutkan"):
        // Blends gently across [0..1] (with progressive highlight relief) so bright hair/skin never kinks!
        let l_goal = if !is_flat {
            // Normal mode: faithfully match shadows/mids while letting highlights expand smoothly to preserve sparkle
            let hi_w = 0.42 * q * q * q;
            let airy = l_cdf.max(l_norm * 0.95 + 0.04);
            l_cdf + (airy - l_cdf) * hi_w
        } else if r92 < 0.62 {
            // Dim cinema reference in Raw/Flat mode: faithfully match shadows/mids while lifting skin/highlights naturally
            let hi_w = 0.36 * q * q;
            let airy = l_cdf.max(l_norm * 0.78 + 0.05);
            l_cdf + (airy - l_cdf) * hi_w
        } else if l_cdf > 0.70 {
            // Bright outlier speculars in moody Raw/Flat refs (e.g. Joker bulbs): soft filmic shoulder
            let over = l_cdf - 0.70;
            0.70 + over / (1.0 + over * 1.95)
        } else {
            l_cdf
        };

        let w_tone = if x < t01 {
            1.0
        } else if is_flat {
            0.99
        } else {
            0.95
        };
        l_raw[i] = x + (l_goal - x) * w_tone;
    }

    // Minimal dark-floor safety ONLY if a near-black silhouette reference crushed faces < 0.34
    if tgt_skin_l.len() >= 20 {
        tgt_skin_l.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let t_skin_med = tgt_skin_l[tgt_skin_l.len() / 2];
        let idx = ((t_skin_med * (L_BINS - 1) as f32).round() as usize).min(L_BINS - 1);
        if l_raw[idx] < 0.34 {
            let lift = 0.34 - l_raw[idx];
            for i in 0..L_BINS {
                let x = i as f32 / (L_BINS - 1) as f32;
                let w = (-((x - t_skin_med) / 0.14).powi(2)).exp();
                l_raw[i] = (l_raw[i] + lift * w * 0.68).min(0.86);
            }
        }
    }

    // Smooth with C^2 Gaussian kernel (sigma = 5.5 bins ~ 0.021 L) so skin/hair gradients stay silky
    let mut l_map = [0.0f32; L_BINS];
    let g_sigma = 5.5f32;
    let g_rad = 14isize;
    for i in 0..L_BINS {
        let mut wsum = 0.0f32;
        let mut vsum = 0.0f32;
        for d in -g_rad..=g_rad {
            let j = (i as isize + d).clamp(0, (L_BINS - 1) as isize) as usize;
            let w = (-((d as f32) / g_sigma).powi(2) * 0.5).exp();
            wsum += w;
            vsum += w * l_raw[j];
        }
        l_map[i] = vsum / wsum;
    }

    // Very small min_step on empty pedestal [0..t01] so lifted LOG blacks crush cleanly down to r05!
    let min_step = 0.04 / (L_BINS - 1) as f32;
    let max_step = (if is_flat { 4.8 } else { 3.2 }) / (L_BINS - 1) as f32;
    let start_lo = (r05 * 0.45).clamp(0.0, 0.09);
    l_map[0] = l_map[0].clamp(0.0, start_lo.max(0.03));
    for i in 1..L_BINS {
        let hi = (l_map[i - 1] + max_step).min(1.0);
        let lo = (l_map[i - 1] + min_step).min(hi);
        l_map[i] = l_map[i].clamp(lo, hi);
    }
    if t005 > 0.15 {
        l_map[L_BINS - 1] = l_map[L_BINS - 1].clamp(0.78, 1.0);
    } else {
        l_map[L_BINS - 1] = l_map[L_BINS - 1].clamp(0.86, 1.0);
    }

    let eval_l_map = |l_val: f32| -> f32 {
        let f = (l_val.clamp(0.0, 1.0) * (L_BINS - 1) as f32).clamp(0.0, (L_BINS - 1) as f32);
        let i0 = (f.floor() as usize).min(L_BINS - 1);
        let i1 = (i0 + 1).min(L_BINS - 1);
        let frac = f - i0 as f32;
        l_map[i0] * (1.0 - frac) + l_map[i1] * frac
    };

    // --- Step 2: 8-Zone Weighted-Median Atmospheric Anchor + Noise-Safe Covariance Transfer ---
    const ZONES: usize = 8;
    let centers = [0.06f32, 0.18, 0.30, 0.42, 0.54, 0.66, 0.78, 0.90];
    let zone_sigma = 0.13f32;

    #[derive(Clone, Default)]
    struct ZoneStats {
        w_sum: f64,
        a_sum: f64,
        b_sum: f64,
        aa_sum: f64,
        ab_sum: f64,
        bb_sum: f64,
        a_samples: Vec<(f32, f32)>,
        b_samples: Vec<(f32, f32)>,
    }
    impl ZoneStats {
        fn add(&mut self, a: f32, b: f32, w: f32, collect_median: bool) {
            let w64 = w as f64;
            let a64 = a as f64;
            let b64 = b as f64;
            self.w_sum += w64;
            self.a_sum += w64 * a64;
            self.b_sum += w64 * b64;
            self.aa_sum += w64 * a64 * a64;
            self.ab_sum += w64 * a64 * b64;
            self.bb_sum += w64 * b64 * b64;
            if collect_median {
                self.a_samples.push((a, w));
                self.b_samples.push((b, w));
            }
        }
        fn finalize(&mut self, use_median_anchor: bool) -> (f32, f32, f32, f32, f32) {
            if self.w_sum < 1e-4 {
                return (0.0, 0.0, 0.0005, 0.0, 0.0005);
            }
            let mu_raw_a = (self.a_sum / self.w_sum) as f32;
            let mu_raw_b = (self.b_sum / self.w_sum) as f32;
            let (mu_a, mu_b) = if use_median_anchor && !self.a_samples.is_empty() {
                // Weighted median captures dominant ambient atmosphere & resists vivid subject outliers (e.g. Joker suit)
                let med_a = weighted_median(&mut self.a_samples);
                let med_b = weighted_median(&mut self.b_samples);
                (med_a * 0.90 + mu_raw_a * 0.10, med_b * 0.90 + mu_raw_b * 0.10)
            } else {
                (mu_raw_a, mu_raw_b)
            };
            let vaa = ((self.aa_sum / self.w_sum) as f32 - mu_raw_a * mu_raw_a).max(1e-7);
            let vab = (self.ab_sum / self.w_sum) as f32 - mu_raw_a * mu_raw_b;
            let vbb = ((self.bb_sum / self.w_sum) as f32 - mu_raw_b * mu_raw_b).max(1e-7);
            (mu_a, mu_b, vaa, vab, vbb)
        }
    }

    let mut t_zones: [ZoneStats; ZONES] = std::array::from_fn(|_| ZoneStats::default());
    let mut r_zones: [ZoneStats; ZONES] = std::array::from_fn(|_| ZoneStats::default());

    for &(l, a, okb) in &tgt_labs {
        let l_eff = eval_l_map(l);
        for z in 0..ZONES {
            let w = (-((l_eff - centers[z]) / zone_sigma).powi(2)).exp();
            if w > 0.008 {
                t_zones[z].add(a, okb, w, false);
            }
        }
    }
    for &(l, a, okb) in &ref_labs {
        for z in 0..ZONES {
            let w = (-((l - centers[z]) / zone_sigma).powi(2)).exp();
            if w > 0.008 {
                r_zones[z].add(a, okb, w, true);
            }
        }
    }

    // Regularize orthogonal JPEG/H.264 chroma floor while gating near-neutral deltas cleanly
    let max_gain = if is_flat { 1.85 } else { 1.60 };
    let min_gain = 0.35f32;

    let mut mu_t = [(0.0f32, 0.0f32); ZONES];
    let mut mu_r = [(0.0f32, 0.0f32); ZONES];
    let mut t_mats = [Mat2::default(); ZONES];

    for z in 0..ZONES {
        let (ta, tb, taa, tab, tbb) = t_zones[z].finalize(false);
        let (ra, rb, raa, rab, rbb) = r_zones[z].finalize(true);
        mu_t[z] = (ta, tb);
        mu_r[z] = (ra, rb);
        let eps_t = if is_flat {
            ((taa + tbb) * 0.30).max(1.2e-4)
        } else {
            ((taa + tbb) * 0.40).max(1.5e-4)
        };
        let eps_r = ((raa + rbb) * 0.14).max(6.0e-5);
        t_mats[z] = compute_mk_2x2(taa, tab, tbb, raa, rab, rbb, eps_t, eps_r, min_gain, max_gain);
    }

    let has_tgt_skin = tgt_skin_l.len() >= 20;
    let ref_skin_mean = if ref_skin_l.len() >= 20 {
        Some((
            (ref_skin_a_sum / ref_skin_l.len() as f64) as f32,
            (ref_skin_b_sum / ref_skin_l.len() as f64) as f32,
        ))
    } else {
        None
    };

    // --- Step 3: Populate 25x25x25 3D LUT ---
    let mut lut = vec![0.0f32; dim * dim * dim * 3];
    let denom = (dim - 1) as f32;
    let d2 = dim * dim;

    for bi in 0..dim {
        let b = bi as f32 / denom;
        for gi in 0..dim {
            let g = gi as f32 / denom;
            for ri in 0..dim {
                let r = ri as f32 / denom;
                let (l_in, a_in, b_in) = srgb_to_oklab(r, g, b);
                let l_out = eval_l_map(l_in);

                let mut w = [0.0f32; ZONES];
                let mut w_sum = 0.0f32;
                for z in 0..ZONES {
                    w[z] = (-((l_out - centers[z]) / zone_sigma).powi(2)).exp();
                    w_sum += w[z];
                }
                let inv_w = if w_sum > 1e-6 { 1.0 / w_sum } else { 1.0 / ZONES as f32 };

                let c_in = (a_in * a_in + b_in * b_in).sqrt();
                let mut a_mk = 0.0f32;
                let mut b_mk = 0.0f32;
                for z in 0..ZONES {
                    let wz = w[z] * inv_w;
                    let da = a_in - mu_t[z].0;
                    let db = b_in - mu_t[z].1;
                    let dc = (da * da + db * db).sqrt();
                    let (mk_a, mk_b) = t_mats[z].apply(da, db);
                    // Gate by TRUE pixel chroma (`c_in`) as well as offset (`dc`) so neutral grey stone/concrete
                    // with tiny JPEG compression jitter never explodes via 3D LUT cube corners into cyan/blue blotches,
                    // while genuine subject colors (foliage, skin, wardrobe) bloom smoothly!
                    let signal_t = smoothstep(0.0085, 0.017, c_in) * smoothstep(0.006, 0.015, dc);
                    let bloom = if is_flat { 1.55 } else { 1.20 };
                    let ta = (da * 0.28) * (1.0 - signal_t) + (mk_a * bloom) * signal_t;
                    let tb = (db * 0.28) * (1.0 - signal_t) + (mk_b * bloom) * signal_t;
                    // Restrain excess cast inside both deep blacks (<0.24) and upper highlights (>0.62)
                    // so blacks stay inky and brights/coats stay clean rather than muddy or green-tinted.
                    let shadow_clean = 0.50 + 0.50 * smoothstep(0.04, 0.24, l_out);
                    let hi_clean = 1.0 - 0.50 * smoothstep(0.62, 0.92, l_out);
                    let zone_cast_scale = shadow_clean * hi_clean;
                    a_mk += wz * (mu_r[z].0 * zone_cast_scale + ta);
                    b_mk += wz * (mu_r[z].1 * zone_cast_scale + tb);
                }

                // Additional pure specular white highlight guard so crisp white fabrics stay clean
                if l_out > 0.78 && c_in < 0.035 {
                    let hi_guard = smoothstep(0.78, 0.96, l_out) * (1.0 - smoothstep(0.01, 0.035, c_in));
                    a_mk *= 1.0 - hi_guard * 0.55;
                    b_mk *= 1.0 - hi_guard * 0.55;
                }

                // Targeted natural skin vibrancy & anti-green safety ONLY on warm face pixels (including bright LOG faces)
                let (mut a_out, mut b_out) = (a_mk, b_mk);
                if has_tgt_skin && a_in > 0.0068 && b_in > -0.001 && l_in > 0.42 && l_in < 0.97 {
                    let ang_in = b_in.atan2(a_in);
                    if ang_in > 0.12 && ang_in < 0.92 {
                        let w_sk = smoothstep(0.0068, 0.0105, a_in)
                            * smoothstep(0.42, 0.56, l_in)
                            * (1.0 - smoothstep(0.91, 0.98, l_in))
                            * (1.0 - smoothstep(0.72, 0.92, ang_in));
                        if w_sk > 0.02 {
                            let min_a = if let Some((ra, _)) = ref_skin_mean {
                                ra.clamp(0.030, 0.044)
                            } else {
                                0.032
                            };
                            if a_out < min_a {
                                a_out += (min_a - a_out) * w_sk * 0.85;
                            }
                            if b_out < 0.008 {
                                b_out += (0.011 - b_out) * w_sk * 0.75;
                            }
                            let c_sk = (a_out * a_out + b_out * b_out).sqrt();
                            if c_sk > 0.100 {
                                let sc = 0.100 / c_sk;
                                a_out = a_out * (1.0 - w_sk) + a_out * sc * w_sk;
                                b_out = b_out * (1.0 - w_sk) + b_out * sc * w_sk;
                            }
                        }
                    }
                }

                let (ro, go, bo) = oklab_to_srgb_gamut(l_out, a_out, b_out);
                let idx = (bi * d2 + gi * dim + ri) * 3;
                lut[idx] = ro;
                lut[idx + 1] = go;
                lut[idx + 2] = bo;
            }
        }
    }

    (lut, is_flat)
}

/// Computes matched `DevelopParams` using a perceptual 25x25x25 OKLab 3D Color Look LUT
/// (`Tonal 0..100%`, `Warna 0..100%`, `Raw/Flat` mode + Palette) without hijacking Basic sliders.
pub fn match_color_and_tone(
    ref_raw: &RawSourceImage,
    ref_params: Option<&DevelopParams>,
    target_raw: &RawSourceImage,
    target_params: &DevelopParams,
) -> DevelopParams {
    match_color_and_tone_ex(ref_raw, ref_params, target_raw, target_params, None, "Reference")
}

pub fn match_color_and_tone_ex(
    ref_raw: &RawSourceImage,
    ref_params: Option<&DevelopParams>,
    target_raw: &RawSourceImage,
    target_params: &DevelopParams,
    force_raw_flat: Option<bool>,
    ref_name: &str,
) -> DevelopParams {
    let mut matched = target_params.clone();
    matched.clear_color_match_lut();

    // Optional reference-edit presence nuance if Reference also has explicit Develop metadata
    if let Some(rp) = ref_params {
        matched.texture = (matched.texture * 0.5 + rp.texture * 0.5).clamp(-100.0, 100.0);
        matched.clarity_2012 = (matched.clarity_2012 * 0.5 + rp.clarity_2012 * 0.5).clamp(-100.0, 100.0);
        matched.dehaze = (matched.dehaze * 0.5 + rp.dehaze * 0.5).clamp(-100.0, 100.0);
    }

    // Sample ACTUAL pixels arriving at Stage 11.5 (rendered baseline with `matched` pre-LUT!)
    // so the 3D LUT aligns pixel-for-pixel with the live Develop pipeline while Basic sliders stay at user's own settings!
    let ref_samples = sample_rendered_srgb_samples(ref_raw, ref_params);
    let tgt_samples = sample_rendered_srgb_samples(target_raw, Some(&matched));
    let palette = extract_reference_palette(&ref_samples, 10);
    let (lut_data, is_flat) = build_match_3d_lut(
        &ref_samples,
        &tgt_samples,
        target_raw.is_scene_referred,
        force_raw_flat,
    );

    if lut_data.len() == MATCH_LUT_DIM * MATCH_LUT_DIM * MATCH_LUT_DIM * 3 {
        matched.match_lut_dim = MATCH_LUT_DIM as u8;
        matched.match_lut_data = lut_data;
        matched.match_tone_amount = if target_params.has_color_match_lut() {
            target_params.match_tone_amount.clamp(0.0, 100.0)
        } else {
            100.0
        };
        matched.match_color_amount = if target_params.has_color_match_lut() {
            target_params.match_color_amount.clamp(0.0, 100.0)
        } else {
            100.0
        };
        matched.match_raw_flat = is_flat;
        matched.match_palette = palette;
        matched.match_ref_name = ref_name.to_string();
    }

    matched
}
