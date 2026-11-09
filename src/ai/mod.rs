// LightRaw AI Engine — Local Hardware-Accelerated Machine Learning & Neural Processing
// Native Apple Neural Engine (ANE), Metal GPU, Vectorized SIMD & Multi-Tier Fallback

pub mod engine;
pub mod inpaint;
pub mod denoise;
pub mod color_match;

pub use engine::{AiHardwareTier, AiSemanticMaps, compute_ai_semantics, get_hardware_description};
pub use inpaint::sample_generative_remove;
pub use denoise::run_ai_denoise;
pub use color_match::{extract_image_color_stats, match_color_and_tone, ImageColorStats};
