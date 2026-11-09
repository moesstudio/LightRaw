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
}

/// Computes comprehensive statistical color and tone descriptors for an image's linear RGB buffer
pub fn extract_image_color_stats(raw: &RawSourceImage) -> ImageColorStats {
    let total_pixels = (raw.width * raw.height) as usize;
    if total_pixels == 0 || raw.rgb.is_empty() {
        return ImageColorStats::default();
    }

    // Step by stride to sample around 12,000 representative points evenly across the frame
    let sample_target = 12000usize;
    let step = (total_pixels / sample_target).max(1);

    let mut lums: Vec<f32> = Vec::with_capacity(total_pixels / step + 10);
    let mut sats: Vec<f32> = Vec::with_capacity(total_pixels / step + 10);

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

    for px in raw.rgb.chunks_exact(3).step_by(step) {
        let r = px[0].max(0.0);
        let g = px[1].max(0.0);
        let b = px[2].max(0.0);

        let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        let max_c = r.max(g).max(b);
        let min_c = r.min(g).min(b);
        let sat = if max_c > 1e-4 {
            (max_c - min_c) / max_c
        } else {
            0.0
        };

        lums.push(lum);
        sats.push(sat);

        // Classify into Shadows (lower 33%), Midtones, and Highlights (upper 33%)
        if lum > 0.02 && lum < 0.33 {
            shd_sum_r += r as f64;
            shd_sum_g += g as f64;
            shd_sum_b += b as f64;
            shd_count += 1;
        } else if lum >= 0.33 && lum <= 0.67 {
            mid_sum_r += r as f64;
            mid_sum_g += g as f64;
            mid_sum_b += b as f64;
            mid_count += 1;
        } else if lum > 0.67 && lum < 0.98 {
            hl_sum_r += r as f64;
            hl_sum_g += g as f64;
            hl_sum_b += b as f64;
            hl_count += 1;
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
    let p95 = lums[(n as f32 * 0.95).min((n - 1) as f32) as usize];

    let sum_l: f32 = lums.iter().sum();
    let mean_lum = sum_l / n as f32;

    let sum_s: f32 = sats.iter().sum();
    let mean_sat = sum_s / sats.len().max(1) as f32;

    let mid_rg = if mid_count > 0 && mid_sum_g > 1e-6 {
        (mid_sum_r / mid_sum_g) as f32
    } else {
        1.0
    };
    let mid_bg = if mid_count > 0 && mid_sum_g > 1e-6 {
        (mid_sum_b / mid_sum_g) as f32
    } else {
        1.0
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
    }
}

/// Converts RGB chromaticity deviation into a Hue [0..360) and Saturation [0..100]
fn rgb_to_hue_sat(r: f32, g: f32, b: f32) -> (f32, f32) {
    let max_c = r.max(g).max(b);
    let min_c = r.min(g).min(b);
    let delta = max_c - min_c;
    if delta < 1e-4 || max_c < 1e-4 {
        return (0.0, 0.0);
    }

    let hue = if max_c == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max_c == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let hue = (hue + 360.0) % 360.0;
    let sat = (delta / max_c * 100.0).clamp(0.0, 100.0);
    (hue, sat)
}

/// Computes matched `DevelopParams` by analyzing the reference image and transferring
/// its exposure, tone curve contrast, white balance color temperature, vibrance, and color grading.
pub fn match_color_and_tone(
    ref_raw: &RawSourceImage,
    ref_params: Option<&DevelopParams>,
    target_raw: &RawSourceImage,
    target_params: &DevelopParams,
) -> DevelopParams {
    let ref_stats = extract_image_color_stats(ref_raw);
    let tgt_stats = extract_image_color_stats(target_raw);

    let mut matched = target_params.clone();

    // 1. Exposure Matching (EV difference based on median and trimmed mean midtone luminance)
    let ref_m = ref_stats.p50_lum.max(0.01);
    let tgt_m = tgt_stats.p50_lum.max(0.01);
    let delta_ev = (ref_m / tgt_m).log2().clamp(-3.0, 3.0);
    matched.exposure_2012 = (target_params.exposure_2012 + delta_ev).clamp(-5.0, 5.0);

    // 2. Contrast & Dynamic Range Matching
    let ref_spread = (ref_stats.p95_lum - ref_stats.p05_lum).max(0.05);
    let tgt_spread = (tgt_stats.p95_lum - tgt_stats.p05_lum).max(0.05);
    let contrast_ratio = ref_spread / tgt_spread;
    let delta_contrast = (contrast_ratio - 1.0) * 55.0;
    matched.contrast_2012 = (target_params.contrast_2012 + delta_contrast).clamp(-100.0, 100.0);

    // Matte Blacks / Faded Film vs Deep Punchy Blacks
    if ref_stats.p05_lum > 0.04 {
        // Reference has lifted matte blacks (vintage / film look)
        let lift = (ref_stats.p05_lum - 0.04) * 350.0;
        matched.blacks_2012 = (target_params.blacks_2012 + lift).clamp(-100.0, 100.0);
    } else if ref_stats.p05_lum < 0.015 {
        // Deep crushed blacks
        matched.blacks_2012 = (target_params.blacks_2012 - 15.0).clamp(-100.0, 100.0);
    }

    // Rolled-off soft highlights vs crisp bright highlights
    if ref_stats.p95_lum < 0.90 {
        let pull = (0.90 - ref_stats.p95_lum) * 250.0;
        matched.whites_2012 = (target_params.whites_2012 - pull).clamp(-100.0, 100.0);
    }

    // Shadows & Highlights balance
    let shd_diff = (ref_stats.p25_lum - tgt_stats.p25_lum) * 110.0;
    matched.shadows_2012 = (target_params.shadows_2012 + shd_diff).clamp(-100.0, 100.0);

    let hl_diff = (ref_stats.p75_lum - tgt_stats.p75_lum) * -110.0;
    matched.highlights_2012 = (target_params.highlights_2012 + hl_diff).clamp(-100.0, 100.0);

    // 3. White Balance & Color Temperature Matching
    let rg_diff = ref_stats.mid_rg_ratio - tgt_stats.mid_rg_ratio;
    let bg_diff = ref_stats.mid_bg_ratio - tgt_stats.mid_bg_ratio;

    let delta_temp = (rg_diff - bg_diff) * 3200.0;
    let delta_tint = (rg_diff + bg_diff) * 28.0;

    matched.white_balance = WhiteBalanceMode::Custom;
    matched.temperature = (target_params.temperature + delta_temp).clamp(2000.0, 50000.0);
    matched.tint = (target_params.tint + delta_tint).clamp(-150.0, 150.0);

    // 4. Vibrance & Saturation Matching
    let sat_diff = ref_stats.mean_sat - tgt_stats.mean_sat;
    matched.vibrance = (target_params.vibrance + sat_diff * 60.0).clamp(-100.0, 100.0);
    matched.saturation = (target_params.saturation + sat_diff * 40.0).clamp(-100.0, 100.0);

    // 5. Shadow & Highlight Color Grading (Split Toning)
    if ref_stats.shadow_sat > 2.0 {
        matched.split_toning_shadow_hue = ref_stats.shadow_hue;
        matched.split_toning_shadow_saturation = (ref_stats.shadow_sat * 0.6).clamp(0.0, 80.0);
    }
    if ref_stats.highlight_sat > 2.0 {
        matched.split_toning_highlight_hue = ref_stats.highlight_hue;
        matched.split_toning_highlight_saturation = (ref_stats.highlight_sat * 0.5).clamp(0.0, 70.0);
    }

    // 6. Inherit detailed creative tone curve, calibration & HSL if reference has Develop adjustments
    if let Some(rp) = ref_params {
        // If reference has customized tone curves or color grading:
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
        matched.texture = (matched.texture * 0.4 + rp.texture * 0.6).clamp(-100.0, 100.0);
        matched.clarity_2012 = (matched.clarity_2012 * 0.4 + rp.clarity_2012 * 0.6).clamp(-100.0, 100.0);
        matched.dehaze = (matched.dehaze * 0.4 + rp.dehaze * 0.6).clamp(-100.0, 100.0);

        // Copy camera profile if applicable
        matched.camera_profile = rp.camera_profile;
    }

    matched
}
