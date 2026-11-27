use jni::objects::{JByteArray, JClass, JString};
use jni::sys::{jboolean, jint, jstring, JNI_TRUE};
use jni::JNIEnv;
use lightraw::ai::color_match::match_color_and_tone;
use lightraw::params::{
    AspectRatioPreset, CameraProfile, DevelopParams, LocalAdjustParams, MaskComponent, MaskGroup,
    MaskOp, MaskPrimitiveKind, RetouchArea, RetouchMode, WhiteBalanceMode,
};
use lightraw::pipeline::{render_develop_pipeline, RawSourceImage, RenderOptions};
use lightraw::presets::{builtin_develop_presets, export_preset_to_xmp, parse_xmp_preset, DevelopPreset};
use lightraw::watermark::{composite_watermark_rgba, WatermarkSettings};
use serde_json::Value;

fn get_string(env: &mut JNIEnv, s: &JString) -> Option<String> {
    env.get_string(s).ok().map(|js| js.into())
}

fn new_jstring(env: &mut JNIEnv, s: &str) -> jstring {
    env.new_string(s)
        .map(|js| js.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

fn decode_base64_bytes(input: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity((input.len() * 3) / 4);
    let mut val = 0u32;
    let mut bits = -8i32;
    for b in input.bytes() {
        let d = match b {
            b'A'..=b'Z' => (b - b'A') as u32,
            b'a'..=b'z' => (b - b'a' + 26) as u32,
            b'0'..=b'9' => (b - b'0' + 52) as u32,
            b'+' => 62,
            b'/' => 63,
            b'=' => break,
            b' ' | b'\n' | b'\r' | b'\t' => continue,
            _ => continue,
        };
        val = (val << 6) | d;
        bits += 6;
        if bits >= 0 {
            out.push(((val >> bits) & 0xFF) as u8);
            bits -= 8;
        }
    }
    out
}

fn decode_lut_u16_b64(b64: &str, dim: usize) -> Vec<f32> {
    if dim < 2 || b64.is_empty() {
        return Vec::new();
    }
    let expected = dim * dim * dim * 3;
    let bytes = decode_base64_bytes(b64);
    if bytes.len() != expected * 2 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(expected);
    for chunk in bytes.chunks_exact(2) {
        let q = u16::from_le_bytes([chunk[0], chunk[1]]);
        out.push(q as f32 / 65535.0);
    }
    out
}

fn parse_curve_val(v: Option<&Value>, fallback: Vec<(f32, f32)>) -> Vec<(f32, f32)> {
    let Some(arr) = v.and_then(|x| x.as_array()) else {
        return fallback;
    };
    let mut pts = Vec::with_capacity(arr.len());
    for item in arr {
        if let Some(pair) = item.as_array() {
            if pair.len() >= 2 {
                let x = pair[0].as_f64().unwrap_or(0.0) as f32;
                let y = pair[1].as_f64().unwrap_or(0.0) as f32;
                pts.push((x, y));
            }
        }
    }
    if pts.is_empty() {
        fallback
    } else {
        pts
    }
}

fn parse_f32_array8(v: Option<&Value>) -> [f32; 8] {
    let mut out = [0.0f32; 8];
    if let Some(arr) = v.and_then(|x| x.as_array()) {
        for (i, item) in arr.iter().take(8).enumerate() {
            out[i] = item.as_f64().unwrap_or(0.0) as f32;
        }
    }
    out
}

/// Parses either native Rust serde JSON or Android Kotlin `DevelopParams.toJsonString()` camelCase JSON.
pub fn parse_develop_params_json(json_str: &str) -> DevelopParams {
    let Ok(val) = serde_json::from_str::<Value>(json_str) else {
        return DevelopParams::default();
    };
    if val.get("exposure_2012").is_some() {
        if let Ok(p) = serde_json::from_value::<DevelopParams>(val.clone()) {
            return p;
        }
    }

    let obj = match val.as_object() {
        Some(m) => m,
        None => return DevelopParams::default(),
    };
    let f = |k: &str, def: f32| -> f32 {
        obj.get(k).and_then(|v| v.as_f64()).map(|n| n as f32).unwrap_or(def)
    };
    let b = |k: &str, def: bool| -> bool {
        obj.get(k).and_then(|v| v.as_bool()).unwrap_or(def)
    };
    let s = |k: &str| -> &str {
        obj.get(k).and_then(|v| v.as_str()).unwrap_or("")
    };

    let mut p = DevelopParams::default();
    p.convert_to_grayscale = b("convertToGrayscale", false);
    p.camera_profile = match s("cameraProfile") {
        "VIVID" => CameraProfile::AdobeVivid,
        "PORTRAIT" => CameraProfile::AdobePortrait,
        "LANDSCAPE" => CameraProfile::AdobeLandscape,
        "NEUTRAL" => CameraProfile::AdobeNeutral,
        "MONOCHROME" => CameraProfile::AdobeMonochrome,
        "FILM" => CameraProfile::CameraStandard,
        "ADOBE_STANDARD" => CameraProfile::AdobeStandard,
        _ => CameraProfile::AdobeColor,
    };
    p.look_amount = f("lookAmount", 100.0);
    p.white_balance = match s("whiteBalance") {
        "AUTO" => WhiteBalanceMode::Auto,
        "DAYLIGHT" => WhiteBalanceMode::Daylight,
        "CLOUDY" => WhiteBalanceMode::Cloudy,
        "SHADE" => WhiteBalanceMode::Shade,
        "TUNGSTEN" => WhiteBalanceMode::Tungsten,
        "FLUORESCENT" => WhiteBalanceMode::Fluorescent,
        "CUSTOM" => WhiteBalanceMode::Custom,
        _ => WhiteBalanceMode::AsShot,
    };
    p.temperature = f("temperature", 5500.0);
    p.tint = f("tint", 0.0);

    p.exposure_2012 = f("exposure", 0.0);
    p.contrast_2012 = f("contrast", 0.0);
    p.highlights_2012 = f("highlights", 0.0);
    p.shadows_2012 = f("shadows", 0.0);
    p.whites_2012 = f("whites", 0.0);
    p.blacks_2012 = f("blacks", 0.0);
    p.texture = f("texture", 0.0);
    p.clarity_2012 = f("clarity", 0.0);
    p.dehaze = f("dehaze", 0.0);
    p.vibrance = f("vibrance", 0.0);
    p.saturation = f("saturation", 0.0);

    p.parametric_highlights = f("parametricHighlights", 0.0);
    p.parametric_lights = f("parametricLights", 0.0);
    p.parametric_darks = f("parametricDarks", 0.0);
    p.parametric_shadows = f("parametricShadows", 0.0);
    p.tone_curve_pv2012 = parse_curve_val(obj.get("toneCurveMaster"), p.tone_curve_pv2012.clone());
    p.tone_curve_pv2012_red = parse_curve_val(obj.get("toneCurveRed"), p.tone_curve_pv2012_red.clone());
    p.tone_curve_pv2012_green = parse_curve_val(obj.get("toneCurveGreen"), p.tone_curve_pv2012_green.clone());
    p.tone_curve_pv2012_blue = parse_curve_val(obj.get("toneCurveBlue"), p.tone_curve_pv2012_blue.clone());

    p.hue_adjustment = parse_f32_array8(obj.get("hueAdjustment"));
    p.saturation_adjustment = parse_f32_array8(obj.get("saturationAdjustment"));
    p.luminance_adjustment = parse_f32_array8(obj.get("luminanceAdjustment"));
    p.gray_mixer = parse_f32_array8(obj.get("grayMixer"));

    p.split_toning_shadow_hue = f("cgShadowHue", 0.0);
    p.split_toning_shadow_saturation = f("cgShadowSat", 0.0);
    p.color_grade_shadow_lum = f("cgShadowLum", 0.0);
    p.color_grade_midtone_hue = f("cgMidHue", 0.0);
    p.color_grade_midtone_sat = f("cgMidSat", 0.0);
    p.color_grade_midtone_lum = f("cgMidLum", 0.0);
    p.split_toning_highlight_hue = f("cgHighHue", 0.0);
    p.split_toning_highlight_saturation = f("cgHighSat", 0.0);
    p.color_grade_highlight_lum = f("cgHighLum", 0.0);
    p.color_grade_global_hue = f("cgGlobalHue", 0.0);
    p.color_grade_global_sat = f("cgGlobalSat", 0.0);
    p.color_grade_global_lum = f("cgGlobalLum", 0.0);
    p.color_grade_blending = f("cgBlending", 50.0);
    p.split_toning_balance = f("cgBalance", 0.0);

    p.sharpness = f("sharpness", 20.0);
    p.sharpen_radius = f("sharpenRadius", 1.0);
    p.luminance_smoothing = f("luminanceNr", 0.0);
    p.color_noise_reduction = f("colorNr", 15.0);
    p.lens_manual_distortion_amount = f("lensDistortion", 0.0);
    p.vignette_amount = f("lensVignetting", 0.0);
    let lens_blur = f("lensBlurAmount", 0.0);
    if lens_blur > 0.5 {
        p.lens_blur_active = true;
        p.lens_blur_amount = lens_blur;
    }
    p.post_crop_vignette_amount = f("vignetteAmount", 0.0);
    p.post_crop_vignette_midpoint = f("vignetteMidpoint", 50.0);
    p.post_crop_vignette_roundness = f("vignetteRoundness", 0.0);
    p.post_crop_vignette_feather = f("vignetteFeather", 50.0);
    p.grain_amount = f("grainAmount", 0.0);
    p.grain_size = f("grainSize", 25.0);
    p.grain_frequency = f("grainRoughness", 50.0);

    p.crop_left = f("cropLeft", 0.0);
    p.crop_top = f("cropTop", 0.0);
    p.crop_right = f("cropRight", 1.0);
    p.crop_bottom = f("cropBottom", 1.0);
    p.crop_angle = f("cropAngle", 0.0);
    p.crop_aspect_preset = match s("cropAspect") {
        "FREE" | "CUSTOM" => AspectRatioPreset::Custom,
        "RATIO_1_1" => AspectRatioPreset::Ratio1x1,
        "RATIO_4_5" => AspectRatioPreset::Ratio4x5,
        "RATIO_2_3" => AspectRatioPreset::Ratio2x3,
        "RATIO_4_3" => AspectRatioPreset::Ratio4x3,
        "RATIO_16_9" => AspectRatioPreset::Ratio16x9,
        _ => AspectRatioPreset::Original,
    };
    p.rotation_degrees = f("rotationDegrees", 0.0);
    p.flip_horizontal = b("flipHorizontal", false);
    p.flip_vertical = b("flipVertical", false);
    p.perspective_vertical = f("perspectiveVertical", 0.0);
    p.perspective_horizontal = f("perspectiveHorizontal", 0.0);
    p.perspective_aspect = f("perspectiveAspect", 0.0);
    p.perspective_scale = f("geometryScale", 100.0);
    p.perspective_x = f("geometryOffsetX", 0.0);
    p.perspective_y = f("geometryOffsetY", 0.0);

    if let Some(m_arr) = obj.get("masks").and_then(|v| v.as_array()) {
        for (idx, mv) in m_arr.iter().enumerate() {
            let Some(mo) = mv.as_object() else { continue };
            let mf = |k: &str, d: f32| mo.get(k).and_then(|x| x.as_f64()).map(|n| n as f32).unwrap_or(d);
            let kind_str = mo.get("kind").and_then(|x| x.as_str()).unwrap_or("RADIAL_GRADIENT");
            let feather = mf("feather", 50.0);
            let x1 = mf("x1", 0.5);
            let y1 = mf("y1", 0.5);
            let x2 = mf("x2", 0.5);
            let y2 = mf("y2", 0.85);
            let rx = mf("radiusX", 0.28);
            let ry = mf("radiusY", 0.28);
            let kind = match kind_str {
                "SUBJECT" => MaskPrimitiveKind::AiSubject,
                "SKY" => MaskPrimitiveKind::AiSky,
                "LINEAR_GRADIENT" => MaskPrimitiveKind::LinearGradient { x1, y1, x2, y2 },
                "BRUSH" => MaskPrimitiveKind::Brush {
                    points: parse_curve_val(mo.get("brushPoints"), Vec::new()),
                    radius: (feather / 100.0 * 0.05 + 0.03).max(0.02),
                    feather,
                    flow: 100.0,
                },
                _ => MaskPrimitiveKind::RadialGradient {
                    cx: x1,
                    cy: y1,
                    rx,
                    ry,
                    feather,
                    invert: false,
                },
            };
            let mut local = LocalAdjustParams::default();
            local.exposure = mf("exposure", 0.0);
            local.contrast = mf("contrast", 0.0);
            local.highlights = mf("highlights", 0.0);
            local.shadows = mf("shadows", 0.0);
            local.temperature = mf("temperature", 0.0);
            local.tint = mf("tint", 0.0);
            local.clarity = mf("clarity", 0.0);
            local.saturation = mf("saturation", 0.0);
            p.mask_groups.push(MaskGroup {
                id: idx as u64 + 1,
                name: mo.get("name").and_then(|x| x.as_str()).unwrap_or("Mask").to_string(),
                enabled: true,
                invert: mo.get("invert").and_then(|x| x.as_bool()).unwrap_or(false),
                components: vec![MaskComponent {
                    name: kind_str.to_string(),
                    op: MaskOp::Add,
                    invert: false,
                    kind,
                }],
                local,
            });
        }
    }

    if let Some(r_arr) = obj.get("retouchSpots").and_then(|v| v.as_array()) {
        for rv in r_arr {
            let Some(ro) = rv.as_object() else { continue };
            let rf = |k: &str, d: f32| ro.get(k).and_then(|x| x.as_f64()).map(|n| n as f32).unwrap_or(d);
            let x = rf("x", 0.5);
            let y = rf("y", 0.5);
            p.retouch_areas.push(RetouchArea {
                mode: RetouchMode::Heal,
                target_x: x,
                target_y: y,
                source_x: (x + 0.08).clamp(0.05, 0.95),
                source_y: (y + 0.06).clamp(0.05, 0.95),
                radius: rf("radius", 0.035),
                feather: 50.0,
                opacity: rf("opacity", 100.0),
                points: Vec::new(),
            });
        }
    }

    p.applied_preset_name = s("appliedPresetName").to_string();
    p.preset_amount = f("presetAmount", 100.0);

    let dim = obj.get("matchLutDim").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let lut = decode_lut_u16_b64(s("matchLutB64"), dim);
    if dim >= 2 && lut.len() == dim * dim * dim * 3 {
        p.match_lut_dim = dim as u8;
        p.match_lut_data = lut;
        let bg_lut = decode_lut_u16_b64(s("matchBgLutB64"), dim);
        if bg_lut.len() == dim * dim * dim * 3 {
            p.match_bg_lut_data = bg_lut;
        }
    }
    p.match_tone_amount = f("matchToneAmount", 100.0);
    p.match_color_amount = f("matchColorAmount", 100.0);
    p.match_raw_flat = b("matchRawFlat", false);
    p.match_ref_name = s("matchRefName").to_string();

    p
}

#[no_mangle]
pub extern "system" fn Java_com_moestret_lightraw_engine_NativeBridge_nativeVersion(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    new_jstring(&mut { env }, "LightRaw Rust Core 1.2.6 (PV2012)")
}

/// Processes RGBA8 input using LightRaw's full non-destructive PV2012 Rust pipeline.
/// Output format: [outW(u32 LE), outH(u32 LE), histR(256*u32 LE), histG(256*u32 LE),
///                 histB(256*u32 LE), histLum(256*u32 LE), rgbaBytes...]
#[no_mangle]
pub extern "system" fn Java_com_moestret_lightraw_engine_NativeBridge_nativeProcessRgba(
    mut env: JNIEnv,
    _class: JClass,
    rgba_bytes: JByteArray,
    width: jint,
    height: jint,
    params_json: JString,
    draft_interactive: jboolean,
    apply_crop: jboolean,
) -> jni::sys::jbyteArray {
    if width <= 0 || height <= 0 {
        return std::ptr::null_mut();
    }
    let Ok(input_i8) = env.convert_byte_array(&rgba_bytes) else {
        return std::ptr::null_mut();
    };
    let expected = (width as usize) * (height as usize) * 4;
    if input_i8.len() < expected {
        return std::ptr::null_mut();
    }

    let json_str = get_string(&mut env, &params_json).unwrap_or_default();
    let params = parse_develop_params_json(&json_str);
    let src = RawSourceImage::from_rgba8(width as u32, height as u32, &input_i8[..expected]);

    let opts = RenderOptions {
        apply_crop: apply_crop == JNI_TRUE,
        is_draft_interactive: draft_interactive == JNI_TRUE,
        ..Default::default()
    };

    let frame = render_develop_pipeline(&src, &params, opts);

    let header_bytes = 8 + 4 * 256 * 4;
    let mut out = Vec::with_capacity(header_bytes + frame.rgba.len());
    out.extend_from_slice(&frame.width.to_le_bytes());
    out.extend_from_slice(&frame.height.to_le_bytes());
    for v in frame.hist_r.iter() {
        out.extend_from_slice(&v.to_le_bytes());
    }
    for v in frame.hist_g.iter() {
        out.extend_from_slice(&v.to_le_bytes());
    }
    for v in frame.hist_b.iter() {
        out.extend_from_slice(&v.to_le_bytes());
    }
    for v in frame.hist_lum.iter() {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out.extend_from_slice(&frame.rgba);

    env.byte_array_from_slice(&out)
        .map(|arr| arr.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "system" fn Java_com_moestret_lightraw_engine_NativeBridge_nativeMatchColorRef(
    mut env: JNIEnv,
    _class: JClass,
    ref_rgba: JByteArray,
    ref_w: jint,
    ref_h: jint,
    tgt_rgba: JByteArray,
    tgt_w: jint,
    tgt_h: jint,
    tgt_params_json: JString,
) -> jstring {
    let (Ok(ref_bytes), Ok(tgt_bytes)) = (
        env.convert_byte_array(&ref_rgba),
        env.convert_byte_array(&tgt_rgba),
    ) else {
        return std::ptr::null_mut();
    };
    let tgt_json = get_string(&mut env, &tgt_params_json).unwrap_or_default();
    let tgt_params = parse_develop_params_json(&tgt_json);

    let ref_raw = RawSourceImage::from_rgba8(ref_w.max(1) as u32, ref_h.max(1) as u32, &ref_bytes);
    let tgt_raw = RawSourceImage::from_rgba8(tgt_w.max(1) as u32, tgt_h.max(1) as u32, &tgt_bytes);
    let matched = match_color_and_tone(&ref_raw, None, &tgt_raw, &tgt_params);

    let out = serde_json::to_string(&matched).unwrap_or_else(|_| "{}".to_string());
    new_jstring(&mut env, &out)
}

#[no_mangle]
pub extern "system" fn Java_com_moestret_lightraw_engine_NativeBridge_nativeParseXmp(
    mut env: JNIEnv,
    _class: JClass,
    xmp_xml: JString,
    fallback_name: JString,
) -> jstring {
    let xml = get_string(&mut env, &xmp_xml).unwrap_or_default();
    let name = get_string(&mut env, &fallback_name).unwrap_or_else(|| "Imported Preset".to_string());
    if let Some(preset) = parse_xmp_preset(&xml, &name) {
        let json = serde_json::to_string(&preset).unwrap_or_default();
        new_jstring(&mut env, &json)
    } else {
        std::ptr::null_mut()
    }
}

#[no_mangle]
pub extern "system" fn Java_com_moestret_lightraw_engine_NativeBridge_nativeExportXmp(
    mut env: JNIEnv,
    _class: JClass,
    preset_json: JString,
) -> jstring {
    let json = get_string(&mut env, &preset_json).unwrap_or_default();
    let Ok(preset) = serde_json::from_str::<DevelopPreset>(&json) else {
        return std::ptr::null_mut();
    };
    let xmp = export_preset_to_xmp(&preset);
    new_jstring(&mut env, &xmp)
}

#[no_mangle]
pub extern "system" fn Java_com_moestret_lightraw_engine_NativeBridge_nativeBuiltinPresetsJson(
    mut env: JNIEnv,
    _class: JClass,
) -> jstring {
    let presets = builtin_develop_presets();
    let json = serde_json::to_string(&presets).unwrap_or_else(|_| "[]".to_string());
    new_jstring(&mut env, &json)
}

#[no_mangle]
pub extern "system" fn Java_com_moestret_lightraw_engine_NativeBridge_nativeApplyWatermarkRgba(
    mut env: JNIEnv,
    _class: JClass,
    rgba_bytes: JByteArray,
    width: jint,
    height: jint,
    settings_json: JString,
) -> jni::sys::jbyteArray {
    let Ok(mut rgba) = env.convert_byte_array(&rgba_bytes) else {
        return std::ptr::null_mut();
    };
    let json = get_string(&mut env, &settings_json).unwrap_or_default();
    let settings: WatermarkSettings = serde_json::from_str(&json).unwrap_or_default();
    composite_watermark_rgba(&mut rgba, width.max(1) as u32, height.max(1) as u32, &settings);
    env.byte_array_from_slice(&rgba)
        .map(|arr| arr.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kotlin_camel_case_json_and_lut_decode() {
        // Build tiny 2x2x2 LUT (24 u16 values = 48 bytes -> base64)
        let mut raw = Vec::new();
        for idx in 0..24 {
            let v = if idx % 3 == 0 { 65535u16 } else { 0u16 };
            raw.extend_from_slice(&v.to_le_bytes());
        }
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut b64 = String::new();
        for chunk in raw.chunks(3) {
            let b0 = chunk[0] as usize;
            let b1 = *chunk.get(1).unwrap_or(&0) as usize;
            let b2 = *chunk.get(2).unwrap_or(&0) as usize;
            let n = (b0 << 16) | (b1 << 8) | b2;
            b64.push(CHARS[(n >> 18) & 63] as char);
            b64.push(CHARS[(n >> 12) & 63] as char);
            b64.push(if chunk.len() > 1 { CHARS[(n >> 6) & 63] as char } else { '=' });
            b64.push(if chunk.len() > 2 { CHARS[n & 63] as char } else { '=' });
        }

        let kotlin_json = format!(
            r#"{{
                "exposure": 1.35,
                "contrast": 28,
                "whiteBalance": "CUSTOM",
                "temperature": 6850,
                "tint": 12,
                "matchLutDim": 2,
                "matchLutB64": "{0}",
                "matchBgLutB64": "{0}",
                "matchToneAmount": 82,
                "matchColorAmount": 91,
                "matchRawFlat": true,
                "matchRefName": "Joker_Ref"
            }}"#,
            b64
        );

        let parsed = parse_develop_params_json(&kotlin_json);
        assert!((parsed.exposure_2012 - 1.35).abs() < 1e-4);
        assert!((parsed.contrast_2012 - 28.0).abs() < 1e-4);
        assert_eq!(parsed.white_balance, WhiteBalanceMode::Custom);
        assert!((parsed.temperature - 6850.0).abs() < 1e-2);
        assert!(parsed.has_color_match_lut());
        assert!(parsed.has_color_match_bg_lut());
        assert_eq!(parsed.match_lut_dim, 2);
        assert!((parsed.match_lut_data[0] - 1.0).abs() < 1e-4);
        assert!((parsed.match_bg_lut_data[0] - 1.0).abs() < 1e-4);
        assert!((parsed.match_lut_data[1] - 0.0).abs() < 1e-4);
        assert!((parsed.match_tone_amount - 82.0).abs() < 1e-4);
        assert!((parsed.match_color_amount - 91.0).abs() < 1e-4);
        assert!(parsed.match_raw_flat);
        assert_eq!(parsed.match_ref_name, "Joker_Ref");
    }
}
