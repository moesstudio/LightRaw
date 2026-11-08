// LightRaw AI Engine — Local Hardware-Accelerated Machine Learning & Neural Processing
// Native Apple Neural Engine (ANE), Metal GPU, Vectorized SIMD & Multi-Tier Fallback

pub mod engine;
pub mod inpaint;
pub mod denoise;

pub use engine::{AiHardwareTier, AiSemanticMaps, compute_ai_semantics, get_hardware_description};
pub use inpaint::sample_generative_remove;
pub use denoise::run_ai_denoise;
