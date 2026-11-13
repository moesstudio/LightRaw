use jni::objects::{JByteArray, JClass, JString};
use jni::sys::{jboolean, jint, jstring, JNI_TRUE};
use jni::JNIEnv;
use lightraw::ai::color_match::match_color_and_tone;
use lightraw::params::DevelopParams;
use lightraw::pipeline::{render_develop_pipeline, RawSourceImage, RenderOptions};
use lightraw::presets::{builtin_develop_presets, export_preset_to_xmp, parse_xmp_preset, DevelopPreset};
use lightraw::watermark::{composite_watermark_rgba, WatermarkSettings};

fn get_string(env: &mut JNIEnv, s: &JString) -> Option<String> {
    env.get_string(s).ok().map(|js| js.into())
}

fn new_jstring(env: &mut JNIEnv, s: &str) -> jstring {
    env.new_string(s)
        .map(|js| js.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "system" fn Java_com_moestret_lightraw_engine_NativeBridge_nativeVersion(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    new_jstring(&mut { env }, "LightRaw Rust Core 1.1.0 (PV2012)")
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
    let params: DevelopParams = serde_json::from_str(&json_str).unwrap_or_default();
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
    let tgt_params: DevelopParams = serde_json::from_str(&tgt_json).unwrap_or_default();

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
