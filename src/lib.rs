// ============================================================================
// LightRaw Core Library — Platform-Independent Non-Destructive RAW Engine
// Shared across macOS/Windows/Linux Desktop and Android JNI (`rust/lightraw_jni`)
// ============================================================================

#![allow(float_literal_f32_fallback)]

pub mod ai;
#[cfg(feature = "desktop")]
pub mod catalog;
pub mod params;
pub mod pipeline;
pub mod presets;
pub mod watermark;

#[cfg(feature = "desktop")]
pub mod ui;
