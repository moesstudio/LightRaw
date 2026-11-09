// ============================================================================
// LightRaw AI — Intelligent Reference Photo Color, White Balance & Tone Transfer
// Analyzes Reference Image (RAW/JPEG/TIFF) & Matches Target Image Look & Feel
// ============================================================================

use crate::params::{DevelopParams, WhiteBalanceMode};
use crate::pipeline::RawSourceImage;

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
fn linear_to_perceptual(c: f32) -> f32 {
    let c = c.max(0.0);
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
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

/// Computes comprehensive statistical color and tone descriptors in a normalized perceptual space.
/// Accurately bridges 32-bit linear RAW buffers and display-referred JPEG/TIFF buffers.
pub fn extract_image_color_stats(raw: &RawSourceImage) -> ImageColorStats {
    let total_pixels = (raw.width * raw.height) as usize;
    if total_pixels == 0 || raw.rgb.is_empty() {
        return ImageColorStats::default();
    }

    // Step by stride to sample around 12,000 representative points evenly across the frame
    let sample_target = 12000usize;
    let step = (total_pixels / sample_target).max(1);

    // If image is scene-referred RAW, apply its baseline sensor exposure gain so linear values
    // map accurately to perceptual 18% middle-gray before statistics are calculated.
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

    // Skin metrics
    let mut skin_lums: Vec<f32> = Vec::with_capacity(cap / 4);
    let mut skin_sum_rg = 0.0f64;
    let mut skin_sum_bg = 0.0f64;
    let mut skin_sum_hue = 0.0f64;
    let mut skin_sum_sat = 0.0f64;

    // Background backdrop metrics
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

        // Apply sensor exposure calibration gain
        let r_lin = raw.rgb[buf_idx] * base_gain;
        let g_lin = raw.rgb[buf_idx + 1] * base_gain;
        let b_lin = raw.rgb[buf_idx + 2] * base_gain;

        // Convert to normalized perceptual display space [0.0 .. 1.0]
        let r = linear_to_perceptual(r_lin).clamp(0.0, 1.0);
        let g = linear_to_perceptual(g_lin).clamp(0.0, 1.0);
        let b = linear_to_perceptual(b_lin).clamp(0.0, 1.0);

        let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        let (hue, sat) = rgb_to_hue_sat(r, g, b);

        lums.push(lum);
        sats.push(sat);

        // Pixel coordinates normalized for semantic lookup
        let px_x = px_idx % width;
        let px_y = px_idx / width;
        let nx = px_x as f32 / width as f32;
        let ny = px_y as f32 / height as f32;

        let face_part = raw.sample_face_part(nx, ny);
        let is_ai_skin = face_part == 1; // 1 = Face Skin

        // Robust colorimetric skin detector (works across Asian, Caucasian, African, Hispanic skin tones)
        let is_color_skin = lum > 0.15
            && lum < 0.90
            && r > g
            && g > b
            && (r - g) >= 0.02
            && (g - b) >= 0.012
            && (r - b) >= 0.05
            && (r - b) <= 0.65
            && (10.0..=48.0).contains(&hue)
            && (12.0..=75.0).contains(&sat);

        let is_skin = is_ai_skin || is_color_skin;

        if is_skin {
            skin_lums.push(lum);
            if g > 1e-4 {
                skin_sum_rg += (r / g) as f64;
                skin_sum_bg += (b / g) as f64;
            }
            skin_sum_hue += hue as f64;
            skin_sum_sat += sat as f64;
        }

        // Studio backdrop detection (e.g. blue studio backdrop or saturated background wall)
        let is_studio_backdrop = (180.0..=250.0).contains(&hue) && sat > 25.0 && lum > 0.10;
        if is_studio_backdrop && !is_skin {
            bg_lums.push(lum);
            bg_sum_hue += hue as f64;
            bg_sum_sat += sat as f64;
        }

        // Classify shadows and highlights for tone curve / split toning
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

        // Classify midtones (20% to 80% perceptual luminance) for general white balance
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

    // Sort luminance to compute exact percentiles
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

    // Midtone chromaticity ratios
    let (mid_rg, mid_bg) = if mid_count > 0 && mid_sum_g > 1e-6 {
        (
            (mid_sum_r / mid_sum_g) as f32,
            (mid_sum_b / mid_sum_g) as f32,
        )
    } else {
        (1.0, 1.0)
    };

    // Calculate shadow tint hue and saturation
    let (shadow_hue, shadow_sat) = if shd_count > 0 {
        let sr = (shd_sum_r / shd_count as f64) as f32;
        let sg = (shd_sum_g / shd_count as f64) as f32;
        let sb = (shd_sum_b / shd_count as f64) as f32;
        rgb_to_hue_sat(sr, sg, sb)
    } else {
        (0.0, 0.0)
    };

    // Calculate highlight tint hue and saturation
    let (highlight_hue, highlight_sat) = if hl_count > 0 {
        let hr = (hl_sum_r / hl_count as f64) as f32;
        let hg = (hl_sum_g / hl_count as f64) as f32;
        let hb = (hl_sum_b / hl_count as f64) as f32;
        rgb_to_hue_sat(hr, hg, hb)
    } else {
        (0.0, 0.0)
    };

    // Compute skin statistics
    let skin_count = skin_lums.len();
    let (skin_median_lum, skin_mean_rg, skin_mean_bg, skin_mean_hue, skin_mean_sat) = if skin_count > 0 {
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

    // Compute background backdrop statistics
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

/// Computes matched `DevelopParams` by analyzing the reference image and transferring
/// its exposure, tone curve contrast, white balance color temperature, vibrance, and color grading.
/// Applies intelligent portrait skin-tone protection and gentle photographic bounds.
pub fn match_color_and_tone(
    ref_raw: &RawSourceImage,
    ref_params: Option<&DevelopParams>,
    target_raw: &RawSourceImage,
    target_params: &DevelopParams,
) -> DevelopParams {
    let ref_stats = extract_image_color_stats(ref_raw);
    let tgt_stats = extract_image_color_stats(target_raw);

    let mut matched = target_params.clone();

    // -------------------------------------------------------------------------
    // 1. Exposure Matching with Portrait Face Protection
    // -------------------------------------------------------------------------
    let delta_ev = if ref_stats.skin_pixel_count >= 30 && tgt_stats.skin_pixel_count >= 30 {
        // Both images contain human subjects: Match skin luminance directly so faces are
        // exposed consistently and whites/skin are never blown out!
        let ref_skin = ref_stats.skin_median_lum.max(0.12);
        let tgt_skin = tgt_stats.skin_median_lum.max(0.12);
        let ev = (ref_skin - tgt_skin) * 1.8;
        ev.clamp(-0.65, 0.65)
    } else {
        // General scene: use median perceptual midtone luminance difference
        let ref_m = ref_stats.p50_lum.max(0.05);
        let tgt_m = tgt_stats.p50_lum.max(0.05);
        let ev = (ref_m - tgt_m) * 1.5;
        ev.clamp(-0.75, 0.75)
    };

    matched.exposure_2012 = (target_params.exposure_2012 + delta_ev).clamp(-3.0, 3.0);

    // -------------------------------------------------------------------------
    // 2. Dynamic Range & Contrast Matching
    // -------------------------------------------------------------------------
    let ref_spread = (ref_stats.p95_lum - ref_stats.p05_lum).max(0.08);
    let tgt_spread = (tgt_stats.p95_lum - tgt_stats.p05_lum).max(0.08);
    let contrast_ratio = ref_spread / tgt_spread;
    let delta_contrast = ((contrast_ratio - 1.0) * 30.0).clamp(-18.0, 22.0);
    matched.contrast_2012 = (target_params.contrast_2012 + delta_contrast).clamp(-50.0, 50.0);

    // Whites: subtle natural adjustment (NEVER slam to -100!)
    let delta_whites = ((ref_stats.p95_lum - tgt_stats.p95_lum) * 25.0).clamp(-15.0, 15.0);
    matched.whites_2012 = (target_params.whites_2012 + delta_whites).clamp(-25.0, 25.0);

    // Blacks: subtle shadow lift or crush
    let delta_blacks = ((ref_stats.p05_lum - tgt_stats.p05_lum) * 30.0).clamp(-15.0, 15.0);
    matched.blacks_2012 = (target_params.blacks_2012 + delta_blacks).clamp(-25.0, 25.0);

    // Highlights & Shadows
    let delta_hl = ((tgt_stats.p95_lum - ref_stats.p95_lum) * -30.0).clamp(-25.0, 20.0);
    matched.highlights_2012 = (target_params.highlights_2012 + delta_hl).clamp(-50.0, 30.0);

    let delta_shd = ((ref_stats.p25_lum - tgt_stats.p25_lum) * 35.0).clamp(-20.0, 25.0);
    matched.shadows_2012 = (target_params.shadows_2012 + delta_shd).clamp(-40.0, 50.0);

    // -------------------------------------------------------------------------
    // 3. White Balance & Color Temperature Matching
    // -------------------------------------------------------------------------
    matched.white_balance = WhiteBalanceMode::Custom;

    let (delta_temp, delta_tint) = if ref_stats.skin_pixel_count >= 30 && tgt_stats.skin_pixel_count >= 30 {
        // Portrait mode: Match skin chromatic warmth rather than whole-frame clothing/backdrop colors!
        let rg_diff = ref_stats.skin_mean_rg - tgt_stats.skin_mean_rg;
        let bg_diff = ref_stats.skin_mean_bg - tgt_stats.skin_mean_bg;
        let d_k = ((rg_diff - bg_diff) * 850.0).clamp(-300.0, 300.0);
        let d_t = ((rg_diff + bg_diff - 2.0) * 8.0).clamp(-5.0, 5.0);
        (d_k, d_t)
    } else {
        // General scenes or synthetic buffers:
        let rg_diff = ref_stats.mid_rg_ratio - tgt_stats.mid_rg_ratio;
        let bg_diff = ref_stats.mid_bg_ratio - tgt_stats.mid_bg_ratio;
        let d_k = ((rg_diff - bg_diff) * 750.0).clamp(-400.0, 400.0);
        let d_t = ((rg_diff + bg_diff - 2.0) * 8.0).clamp(-6.0, 6.0);
        (d_k, d_t)
    };

    matched.temperature = (target_params.temperature + delta_temp).clamp(2800.0, 12000.0);
    matched.tint = (target_params.tint + delta_tint).clamp(-50.0, 50.0);

    // -------------------------------------------------------------------------
    // 4. Vibrance & Saturation Matching
    // -------------------------------------------------------------------------
    let sat_diff = (ref_stats.mean_sat - tgt_stats.mean_sat) * 30.0;
    matched.vibrance = (target_params.vibrance + sat_diff).clamp(-20.0, 30.0);
    matched.saturation = (target_params.saturation + sat_diff * 0.5).clamp(-15.0, 20.0);

    // -------------------------------------------------------------------------
    // 5. Shadow & Highlight Color Grading (Split Toning)
    // -------------------------------------------------------------------------
    if ref_stats.shadow_sat > 2.0 {
        matched.split_toning_shadow_hue = ref_stats.shadow_hue;
        matched.split_toning_shadow_saturation = (ref_stats.shadow_sat * 0.35).clamp(0.0, 35.0);
    }
    if ref_stats.highlight_sat > 2.0 {
        matched.split_toning_highlight_hue = ref_stats.highlight_hue;
        matched.split_toning_highlight_saturation = (ref_stats.highlight_sat * 0.30).clamp(0.0, 30.0);
    }

    // -------------------------------------------------------------------------
    // 6. Inherit Detailed Creative Tone Curve, Calibration & HSL if Reference has Develop adjustments
    // -------------------------------------------------------------------------
    if let Some(rp) = ref_params {
        if !rp.tone_curve_pv2012.is_empty() {
            matched.tone_curve_pv2012 = rp.tone_curve_pv2012.clone();
            matched.tone_curve_name_2012 = rp.tone_curve_name_2012.clone();
        }
        if !rp.tone_curve_pv2012_red.is_empty() {
            matched.tone_curve_pv2012_red = rp.tone_curve_pv2012_red.clone();
        }
        if !rp.tone_curve_pv2012_green.is_empty() {
            matched.tone_curve_pv2012_green = rp.tone_curve_pv2012_green.clone();
        }
        if !rp.tone_curve_pv2012_blue.is_empty() {
            matched.tone_curve_pv2012_blue = rp.tone_curve_pv2012_blue.clone();
        }
        matched.parametric_highlights = rp.parametric_highlights;
        matched.parametric_lights = rp.parametric_lights;
        matched.parametric_darks = rp.parametric_darks;
        matched.parametric_shadows = rp.parametric_shadows;

        // Copy 8-Band HSL color shifts
        matched.hue_adjustment = rp.hue_adjustment;
        matched.saturation_adjustment = rp.saturation_adjustment;
        matched.luminance_adjustment = rp.luminance_adjustment;

        // Copy Color Grading Midtones and Global
        matched.color_grade_midtone_hue = rp.color_grade_midtone_hue;
        matched.color_grade_midtone_sat = rp.color_grade_midtone_sat;
        matched.color_grade_midtone_lum = rp.color_grade_midtone_lum;
        matched.color_grade_global_hue = rp.color_grade_global_hue;
        matched.color_grade_global_sat = rp.color_grade_global_sat;
        matched.color_grade_global_lum = rp.color_grade_global_lum;
        matched.color_grade_blending = rp.color_grade_blending;
        matched.split_toning_balance = rp.split_toning_balance;

        // Copy Presence nuance
        matched.texture = (matched.texture * 0.5 + rp.texture * 0.5).clamp(-100.0, 100.0);
        matched.clarity_2012 = (matched.clarity_2012 * 0.5 + rp.clarity_2012 * 0.5).clamp(-100.0, 100.0);
        matched.dehaze = (matched.dehaze * 0.5 + rp.dehaze * 0.5).clamp(-100.0, 100.0);

        // Copy camera profile if applicable
        matched.camera_profile = rp.camera_profile;
    }

    matched
}
