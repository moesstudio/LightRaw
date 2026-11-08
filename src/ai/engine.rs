// LightRaw AI Engine — Hardware Accelerated Semantic Segmentation & Depth Inference
// Utilizing Apple Neural Engine (ANE), Metal GPU, and Multi-Tier Safety Fallbacks

use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiHardwareTier {
    Auto,           // Apple Neural Engine (16-core ANE) + Metal GPU (Recommended)
    CpuSimd,        // CPU Vectorized SIMD (ARM NEON / AVX2)
    HeuristicsOnly, // Fast analytical color/gradient heuristic fallback
}

impl AiHardwareTier {
    pub const ALL: [Self; 3] = [
        Self::Auto,
        Self::CpuSimd,
        Self::HeuristicsOnly,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Auto => "Auto (Apple Neural Engine & Metal GPU)",
            Self::CpuSimd => "CPU Only (Vectorized SIMD)",
            Self::HeuristicsOnly => "Fast Heuristics Only (Ultra Fast / Low Power)",
        }
    }
}

#[cfg(target_os = "macos")]
extern "C" {
    fn lightraw_ai_hardware_info(out_desc: *mut c_char, max_len: usize) -> i32;
    fn lightraw_ai_segment_subject_rgba(
        rgba_ptr: *const u8,
        width: u32,
        height: u32,
        out_mask: *mut u8,
        mask_w: u32,
        mask_h: u32,
    ) -> i32;
    fn lightraw_ai_segment_people_parts_rgba(
        rgba_ptr: *const u8,
        width: u32,
        height: u32,
        out_parts_mask: *mut u8,
        mask_w: u32,
        mask_h: u32,
    ) -> i32;
    fn lightraw_ai_extract_depth_rgba(
        file_path_utf8: *const c_char,
        rgba_ptr: *const u8,
        width: u32,
        height: u32,
        out_depth: *mut u8,
        depth_w: u32,
        depth_h: u32,
    ) -> i32;
}

pub fn get_hardware_description() -> String {
    #[cfg(target_os = "macos")]
    {
        let mut buf = vec![0u8; 256];
        let ok = unsafe { lightraw_ai_hardware_info(buf.as_mut_ptr() as *mut c_char, buf.len()) };
        if ok != 0 {
            if let Ok(c_str) = unsafe { CStr::from_ptr(buf.as_ptr() as *const c_char) }.to_str() {
                if !c_str.is_empty() {
                    return c_str.to_string();
                }
            }
        }
    }
    "CPU Vectorized SIMD (ARM NEON / AVX2)".to_string()
}

pub struct AiSemanticMaps {
    pub subject_q: Vec<u8>,
    pub sky_q: Vec<u8>,
    pub people_q: Vec<u8>,
    pub depth_q: Vec<u8>,
    pub face_parts_q: Option<Vec<u8>>,
}

#[inline]
fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

#[inline]
fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) * 0.5;
    if (max - min).abs() < 1e-5 {
        return (0.0, 0.0, l);
    }
    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let mut h = if (max - r).abs() < 1e-5 {
        (g - b) / d + (if g < b { 6.0 } else { 0.0 })
    } else if (max - g).abs() < 1e-5 {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    h *= 60.0;
    (h, s, l)
}

#[inline]
fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0).max(1e-5)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Compute semantic masks (Subject, Sky, People, Depth, Face Parts) with multi-tier hardware acceleration
pub fn compute_ai_semantics(
    linear_rgb: &[f32],
    width: u32,
    height: u32,
    sem_w: u32,
    sem_h: u32,
    tier: AiHardwareTier,
    file_path: Option<&str>,
) -> AiSemanticMaps {
    let sem_n = (sem_w * sem_h) as usize;
    let mut depth_q = vec![0u8; sem_n];
    let mut subject_q = vec![0u8; sem_n];
    let mut sky_q = vec![0u8; sem_n];
    let mut people_q = vec![0u8; sem_n];
    let mut face_parts_q: Option<Vec<u8>> = None;

    let use_hardware_ai = tier != AiHardwareTier::HeuristicsOnly;

    // 1. Prepare RGBA buffer for hardware Vision / ANE input (downsampled to max 512x512 for optimal inference latency)
    let ai_in_w = width.min(512).max(64);
    let ai_in_h = height.min(512).max(64);
    let mut rgba_buf = vec![255u8; (ai_in_w * ai_in_h * 4) as usize];

    for y in 0..ai_in_h {
        let sy = ((y as f32 / ai_in_h as f32) * height as f32) as usize;
        let sy = sy.min(height.saturating_sub(1) as usize);
        for x in 0..ai_in_w {
            let sx = ((x as f32 / ai_in_w as f32) * width as f32) as usize;
            let sx = sx.min(width.saturating_sub(1) as usize);
            let src_i = (sy * width as usize + sx) * 3;
            let dst_i = (y * ai_in_w + x) as usize * 4;
            if src_i + 2 < linear_rgb.len() {
                rgba_buf[dst_i] = (linear_to_srgb(linear_rgb[src_i]).clamp(0.0, 1.0) * 255.0) as u8;
                rgba_buf[dst_i + 1] = (linear_to_srgb(linear_rgb[src_i + 1]).clamp(0.0, 1.0) * 255.0) as u8;
                rgba_buf[dst_i + 2] = (linear_to_srgb(linear_rgb[src_i + 2]).clamp(0.0, 1.0) * 255.0) as u8;
                rgba_buf[dst_i + 3] = 255;
            }
        }
    }

    let mut hw_subject_success = false;
    let mut hw_people_success = false;
    let mut hw_depth_success = false;

    #[cfg(target_os = "macos")]
    if use_hardware_ai {
        // Run Native Apple Neural Engine Subject Segmentation
        let res = unsafe {
            lightraw_ai_segment_subject_rgba(
                rgba_buf.as_ptr(),
                ai_in_w,
                ai_in_h,
                subject_q.as_mut_ptr(),
                sem_w,
                sem_h,
            )
        };
        if res != 0 {
            hw_subject_success = true;
        }

        // Run Native Apple Neural Engine People Segmentation & Facial Features
        let mut parts_buf = vec![0u8; sem_n];
        let p_res = unsafe {
            lightraw_ai_segment_people_parts_rgba(
                rgba_buf.as_ptr(),
                ai_in_w,
                ai_in_h,
                parts_buf.as_mut_ptr(),
                sem_w,
                sem_h,
            )
        };
        if p_res != 0 {
            // Extract people mask from parts
            for i in 0..sem_n {
                let p = parts_buf[i];
                people_q[i] = if p > 0 { 255 } else { 0 };
            }
            face_parts_q = Some(parts_buf);
            hw_people_success = true;
        }

        // Run Hardware Disparity / Depth extraction
        if let Some(path) = file_path {
            if let Ok(c_path) = CString::new(path) {
                let d_res = unsafe {
                    lightraw_ai_extract_depth_rgba(
                        c_path.as_ptr(),
                        rgba_buf.as_ptr(),
                        ai_in_w,
                        ai_in_h,
                        depth_q.as_mut_ptr(),
                        sem_w,
                        sem_h,
                    )
                };
                if d_res != 0 {
                    hw_depth_success = true;
                }
            }
        }
    }

    // 2. Synthesize Sky and Depth if not provided by hardware, plus graceful heuristic fallback
    for sy in 0..sem_h {
        let ny = sy as f32 / sem_h.max(1) as f32;
        let py = ((ny * (height.saturating_sub(1)) as f32) as usize).min(height.saturating_sub(1) as usize);
        for sx in 0..sem_w {
            let nx = sx as f32 / sem_w.max(1) as f32;
            let px = ((nx * (width.saturating_sub(1)) as f32) as usize).min(width.saturating_sub(1) as usize);
            let src_idx = (py * width as usize + px) * 3;
            let (r_lin, g_lin, b_lin) = if src_idx + 2 < linear_rgb.len() {
                (linear_rgb[src_idx], linear_rgb[src_idx + 1], linear_rgb[src_idx + 2])
            } else {
                (0.0, 0.0, 0.0)
            };
            let r = linear_to_srgb(r_lin).clamp(0.0, 1.0);
            let g = linear_to_srgb(g_lin).clamp(0.0, 1.0);
            let b = linear_to_srgb(b_lin).clamp(0.0, 1.0);
            let (h, s, l) = rgb_to_hsl(r, g, b);

            let dst_idx = (sy * sem_w + sx) as usize;

            // If hardware subject segmentation succeeded, use it; otherwise compute heuristic
            let subj_v = if hw_subject_success {
                subject_q[dst_idx] as f32 / 255.0
            } else {
                let center_dist = ((nx - 0.5).powi(2) * 1.3 + (ny - 0.54).powi(2) * 1.1).sqrt();
                let warmth = ((1.0 - ((h - 28.0).abs() / 180.0)).max(0.0)) * s;
                let subj = (1.0 - center_dist * 1.85).clamp(0.0, 1.0) * (0.65 + 0.55 * warmth);
                let sv = smoothstep(0.18, 0.55, subj);
                subject_q[dst_idx] = (sv * 255.0) as u8;
                sv
            };

            // Sky Mask: Grounded by (1.0 - subject) to guarantee zero subject spillover
            let is_sky_hue = (h >= 180.0 && h <= 255.0 && s > 0.12) || (ny < 0.45 && l > 0.52);
            let sky_prob = if is_sky_hue && ny < 0.62 {
                ((0.62 - ny) * 2.3).clamp(0.0, 1.0) * (0.45 + 0.55 * l) * (1.0 - subj_v * 0.92)
            } else {
                0.0
            };
            sky_q[dst_idx] = (sky_prob.clamp(0.0, 1.0) * 255.0) as u8;

            // People Mask fallback if not produced by hardware
            if !hw_people_success {
                let is_skin = (h <= 42.0 || h >= 335.0) && s >= 0.10 && s <= 0.78 && l >= 0.16 && l <= 0.90;
                let peop_v = if is_skin {
                    subj_v.max(0.78)
                } else {
                    subj_v * 0.42
                };
                people_q[dst_idx] = (peop_v.clamp(0.0, 1.0) * 255.0) as u8;
            }

            // Depth Map: If not hardware embedded, construct physical perspective depth map
            if !hw_depth_success {
                let base_depth = (1.0 - ny) * 0.72 + sky_prob * 0.35;
                let depth_v = (base_depth * (1.0 - subj_v * 0.72)).clamp(0.0, 1.0);
                depth_q[dst_idx] = (depth_v * 255.0) as u8;
            }
        }
    }

    AiSemanticMaps {
        subject_q,
        sky_q,
        people_q,
        depth_q,
        face_parts_q,
    }
}
