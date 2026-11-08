use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

fn default_fifty() -> f32 {
    50.0
}

fn default_hundred() -> f32 {
    100.0
}

fn default_crop_custom_ratio() -> (f32, f32) {
    (1.0, 1.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WhiteBalanceMode {
    AsShot,
    Auto,
    Daylight,
    Cloudy,
    Shade,
    Tungsten,
    Fluorescent,
    Flash,
    Custom,
}

impl WhiteBalanceMode {
    pub const ALL: &'static [WhiteBalanceMode] = &[
        WhiteBalanceMode::AsShot,
        WhiteBalanceMode::Auto,
        WhiteBalanceMode::Daylight,
        WhiteBalanceMode::Cloudy,
        WhiteBalanceMode::Shade,
        WhiteBalanceMode::Tungsten,
        WhiteBalanceMode::Fluorescent,
        WhiteBalanceMode::Flash,
        WhiteBalanceMode::Custom,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            WhiteBalanceMode::AsShot => "As Shot",
            WhiteBalanceMode::Auto => "Auto",
            WhiteBalanceMode::Daylight => "Daylight (5500 K)",
            WhiteBalanceMode::Cloudy => "Cloudy (6500 K)",
            WhiteBalanceMode::Shade => "Shade (7500 K)",
            WhiteBalanceMode::Tungsten => "Tungsten (2850 K)",
            WhiteBalanceMode::Fluorescent => "Fluorescent (3800 K)",
            WhiteBalanceMode::Flash => "Flash (5500 K)",
            WhiteBalanceMode::Custom => "Custom",
        }
    }

    pub fn preset_temp_tint(&self) -> Option<(f32, f32)> {
        match self {
            WhiteBalanceMode::AsShot => Some((5500.0, 0.0)),
            WhiteBalanceMode::Auto => None,
            WhiteBalanceMode::Daylight => Some((5500.0, 10.0)),
            WhiteBalanceMode::Cloudy => Some((6500.0, 10.0)),
            WhiteBalanceMode::Shade => Some((7500.0, 10.0)),
            WhiteBalanceMode::Tungsten => Some((2850.0, 0.0)),
            WhiteBalanceMode::Fluorescent => Some((3800.0, 21.0)),
            WhiteBalanceMode::Flash => Some((5500.0, 0.0)),
            WhiteBalanceMode::Custom => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CameraProfile {
    AdobeColor,
    AdobeVivid,
    AdobeLandscape,
    AdobePortrait,
    AdobeNeutral,
    AdobeStandard,
    AdobeMonochrome,
    CameraStandard,
    CameraFaithful,
    CameraPositiveFilm,
}

impl CameraProfile {
    pub const ALL: &'static [CameraProfile] = &[
        CameraProfile::AdobeColor,
        CameraProfile::AdobeVivid,
        CameraProfile::AdobeLandscape,
        CameraProfile::AdobePortrait,
        CameraProfile::AdobeNeutral,
        CameraProfile::AdobeStandard,
        CameraProfile::AdobeMonochrome,
        CameraProfile::CameraStandard,
        CameraProfile::CameraFaithful,
        CameraProfile::CameraPositiveFilm,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            CameraProfile::AdobeColor => "Adobe Color",
            CameraProfile::AdobeVivid => "Adobe Vivid",
            CameraProfile::AdobeLandscape => "Adobe Landscape",
            CameraProfile::AdobePortrait => "Adobe Portrait",
            CameraProfile::AdobeNeutral => "Adobe Neutral",
            CameraProfile::AdobeStandard => "Adobe Standard",
            CameraProfile::AdobeMonochrome => "Adobe Monochrome",
            CameraProfile::CameraStandard => "Camera Standard",
            CameraProfile::CameraFaithful => "Camera Faithful",
            CameraProfile::CameraPositiveFilm => "Camera Positive Film",
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            CameraProfile::AdobeColor
            | CameraProfile::AdobeVivid
            | CameraProfile::AdobeLandscape
            | CameraProfile::AdobePortrait
            | CameraProfile::AdobeNeutral
            | CameraProfile::AdobeStandard => "Adobe Raw",
            CameraProfile::AdobeMonochrome => "B&W",
            CameraProfile::CameraStandard
            | CameraProfile::CameraFaithful
            | CameraProfile::CameraPositiveFilm => "Camera Matching",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UprightMode {
    Off,
    Auto,
    Level,
    Vertical,
    Guided,
    Full,
}

impl UprightMode {
    pub const ALL: &'static [UprightMode] = &[
        UprightMode::Off,
        UprightMode::Auto,
        UprightMode::Level,
        UprightMode::Vertical,
        UprightMode::Guided,
        UprightMode::Full,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            UprightMode::Off => "Off",
            UprightMode::Auto => "Auto",
            UprightMode::Level => "Level",
            UprightMode::Vertical => "Vertical",
            UprightMode::Guided => "Guided",
            UprightMode::Full => "Full",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VignetteStyle {
    HighlightPriority,
    ColorPriority,
    PaintOverlay,
}

impl VignetteStyle {
    pub const ALL: &'static [VignetteStyle] = &[
        VignetteStyle::HighlightPriority,
        VignetteStyle::ColorPriority,
        VignetteStyle::PaintOverlay,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            VignetteStyle::HighlightPriority => "Highlight Priority",
            VignetteStyle::ColorPriority => "Color Priority",
            VignetteStyle::PaintOverlay => "Paint Overlay",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BokehShape {
    Circle,
    SoapBubble,
    Blade,
    Ring,
    CatEye,
    Anamorphic,
}

impl BokehShape {
    pub const ALL: &'static [BokehShape] = &[
        BokehShape::Circle,
        BokehShape::SoapBubble,
        BokehShape::Blade,
        BokehShape::Ring,
        BokehShape::CatEye,
        BokehShape::Anamorphic,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            BokehShape::Circle => "Circle (Modern circular lens)",
            BokehShape::SoapBubble => "Bubble (Standard circular with bright rim)",
            BokehShape::Blade => "5-Blade (Vintage 5-blade aperture)",
            BokehShape::Ring => "Ring (Reflex / Mirror lens)",
            BokehShape::CatEye => "Cat Eye (Optical mechanical vignetting)",
            BokehShape::Anamorphic => "Anamorphic (Oval cinema bokeh)",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            BokehShape::Circle => "Circle",
            BokehShape::SoapBubble => "Bubble",
            BokehShape::Blade => "5-Blade",
            BokehShape::Ring => "Ring",
            BokehShape::CatEye => "Cat Eye",
            BokehShape::Anamorphic => "Anamorphic",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LensBlurStroke {
    pub is_focus: bool,
    pub amount: f32,  // 0..100
    pub radius: f32,  // normalized 0.005..0.35
    pub feather: f32, // 0..100
    pub points: Vec<(f32, f32)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolOverlayMode {
    Always,
    Auto,
    Never,
}

impl Default for ToolOverlayMode {
    fn default() -> Self {
        ToolOverlayMode::Auto
    }
}

impl ToolOverlayMode {
    pub const ALL: &'static [ToolOverlayMode] = &[
        ToolOverlayMode::Always,
        ToolOverlayMode::Auto,
        ToolOverlayMode::Never,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            ToolOverlayMode::Always => "Always",
            ToolOverlayMode::Auto => "Auto",
            ToolOverlayMode::Never => "Never",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            ToolOverlayMode::Always => ToolOverlayMode::Auto,
            ToolOverlayMode::Auto => ToolOverlayMode::Never,
            ToolOverlayMode::Never => ToolOverlayMode::Always,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AspectRatioPreset {
    Original,
    Custom,
    Ratio1x1,
    Ratio4x5,
    Ratio8_5x11,
    Ratio5x7,
    Ratio2x3,
    Ratio4x3,
    Ratio16x9,
    Ratio16x10,
}

impl AspectRatioPreset {
    pub const ALL: &'static [AspectRatioPreset] = &[
        AspectRatioPreset::Original,
        AspectRatioPreset::Custom,
        AspectRatioPreset::Ratio1x1,
        AspectRatioPreset::Ratio4x5,
        AspectRatioPreset::Ratio8_5x11,
        AspectRatioPreset::Ratio5x7,
        AspectRatioPreset::Ratio2x3,
        AspectRatioPreset::Ratio4x3,
        AspectRatioPreset::Ratio16x9,
        AspectRatioPreset::Ratio16x10,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            AspectRatioPreset::Original => "As Shot / Original",
            AspectRatioPreset::Custom => "Enter Custom (Unlocked)",
            AspectRatioPreset::Ratio1x1 => "1 x 1",
            AspectRatioPreset::Ratio4x5 => "4 x 5 / 8 x 10",
            AspectRatioPreset::Ratio8_5x11 => "8.5 x 11",
            AspectRatioPreset::Ratio5x7 => "5 x 7",
            AspectRatioPreset::Ratio2x3 => "2 x 3 / 4 x 6",
            AspectRatioPreset::Ratio4x3 => "4 x 3",
            AspectRatioPreset::Ratio16x9 => "16 x 9",
            AspectRatioPreset::Ratio16x10 => "16 x 10",
        }
    }

    pub fn ratio(&self) -> Option<f32> {
        match self {
            AspectRatioPreset::Original | AspectRatioPreset::Custom => None,
            AspectRatioPreset::Ratio1x1 => Some(1.0),
            AspectRatioPreset::Ratio4x5 => Some(5.0 / 4.0),
            AspectRatioPreset::Ratio8_5x11 => Some(11.0 / 8.5),
            AspectRatioPreset::Ratio5x7 => Some(7.0 / 5.0),
            AspectRatioPreset::Ratio2x3 => Some(3.0 / 2.0),
            AspectRatioPreset::Ratio4x3 => Some(4.0 / 3.0),
            AspectRatioPreset::Ratio16x9 => Some(16.0 / 9.0),
            AspectRatioPreset::Ratio16x10 => Some(16.0 / 10.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CropGuideOverlay {
    RuleOfThirds,
    Grid,
    Diagonal,
    Triangle,
    GoldenRatio,
    GoldenSpiral,
    AspectRatios,
}

impl CropGuideOverlay {
    pub const ALL: &'static [CropGuideOverlay] = &[
        CropGuideOverlay::RuleOfThirds,
        CropGuideOverlay::Grid,
        CropGuideOverlay::Diagonal,
        CropGuideOverlay::Triangle,
        CropGuideOverlay::GoldenRatio,
        CropGuideOverlay::GoldenSpiral,
        CropGuideOverlay::AspectRatios,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            CropGuideOverlay::RuleOfThirds => "Rule of Thirds",
            CropGuideOverlay::Grid => "Grid",
            CropGuideOverlay::Diagonal => "Diagonal",
            CropGuideOverlay::Triangle => "Triangle",
            CropGuideOverlay::GoldenRatio => "Golden Ratio",
            CropGuideOverlay::GoldenSpiral => "Golden Spiral",
            CropGuideOverlay::AspectRatios => "Aspect Ratios",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            CropGuideOverlay::RuleOfThirds => CropGuideOverlay::Grid,
            CropGuideOverlay::Grid => CropGuideOverlay::Diagonal,
            CropGuideOverlay::Diagonal => CropGuideOverlay::Triangle,
            CropGuideOverlay::Triangle => CropGuideOverlay::GoldenRatio,
            CropGuideOverlay::GoldenRatio => CropGuideOverlay::GoldenSpiral,
            CropGuideOverlay::GoldenSpiral => CropGuideOverlay::AspectRatios,
            CropGuideOverlay::AspectRatios => CropGuideOverlay::RuleOfThirds,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetouchMode {
    Remove, // Generative / Content-Aware Remove
    Heal,   // Texture + Lighting blend
    Clone,  // Direct pixel stamp
}

impl RetouchMode {
    pub fn label(&self) -> &'static str {
        match self {
            RetouchMode::Remove => "Remove (Content-Aware)",
            RetouchMode::Heal => "Heal",
            RetouchMode::Clone => "Clone",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetouchArea {
    pub mode: RetouchMode,
    pub target_x: f32,
    pub target_y: f32,
    pub source_x: f32,
    pub source_y: f32,
    pub radius: f32,
    pub feather: f32,
    pub opacity: f32,
    #[serde(default)]
    pub points: Vec<(f32, f32)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RedEyeMode {
    RedEye,
    PetEye,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RedEyeSpot {
    pub mode: RedEyeMode,
    pub center_x: f32,
    pub center_y: f32,
    pub radius: f32,
    pub pupil_size: f32, // 0..100
    pub darken: f32,     // 0..100
    #[serde(default = "default_true")]
    pub add_catchlight: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PointColorSwatch {
    pub name: String,
    pub src_hue: f32,        // 0..360
    pub src_sat: f32,        // 0..1
    pub src_lum: f32,        // 0..1
    pub hue_shift: f32,      // -100..100
    pub sat_shift: f32,      // -100..100
    pub lum_shift: f32,      // -100..100
    pub range_amount: f32,   // 0..100
    pub hue_range: f32,      // 0..100
    pub sat_range: f32,      // 0..100
    pub lum_range: f32,      // 0..100
}

impl Default for PointColorSwatch {
    fn default() -> Self {
        Self {
            name: "Swatch 1".to_string(),
            src_hue: 25.0,
            src_sat: 0.65,
            src_lum: 0.55,
            hue_shift: 0.0,
            sat_shift: 0.0,
            lum_shift: 0.0,
            range_amount: 50.0,
            hue_range: 50.0,
            sat_range: 50.0,
            lum_range: 50.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaskOp {
    Add,
    Subtract,
    Intersect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LandscapePart {
    Sky,
    Architecture,
    Vegetation,
    Water,
    Mountains,
    NaturalGround,
    ArtificialGround,
}

impl LandscapePart {
    pub const ALL: &'static [LandscapePart] = &[
        LandscapePart::Sky,
        LandscapePart::Mountains,
        LandscapePart::Architecture,
        LandscapePart::Vegetation,
        LandscapePart::Water,
        LandscapePart::NaturalGround,
        LandscapePart::ArtificialGround,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            LandscapePart::Sky => "Sky",
            LandscapePart::Architecture => "Architecture",
            LandscapePart::Vegetation => "Vegetation",
            LandscapePart::Water => "Water",
            LandscapePart::Mountains => "Mountains",
            LandscapePart::NaturalGround => "Natural Ground",
            LandscapePart::ArtificialGround => "Artificial Ground",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersonPart {
    EntirePerson,
    FaceSkin,
    BodySkin,
    Eyebrows,
    EyeSclera,
    IrisPupil,
    Lips,
    Teeth,
    Hair,
    FacialHair,
    Clothes,
}

impl PersonPart {
    pub const ALL: &'static [PersonPart] = &[
        PersonPart::EntirePerson,
        PersonPart::FaceSkin,
        PersonPart::BodySkin,
        PersonPart::Eyebrows,
        PersonPart::EyeSclera,
        PersonPart::IrisPupil,
        PersonPart::Lips,
        PersonPart::Teeth,
        PersonPart::Hair,
        PersonPart::FacialHair,
        PersonPart::Clothes,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            PersonPart::EntirePerson => "Entire Person",
            PersonPart::FaceSkin => "Face Skin",
            PersonPart::BodySkin => "Body Skin",
            PersonPart::Eyebrows => "Eyebrows",
            PersonPart::EyeSclera => "Eye Sclera",
            PersonPart::IrisPupil => "Iris and Pupil",
            PersonPart::Lips => "Lips",
            PersonPart::Teeth => "Teeth",
            PersonPart::Hair => "Hair",
            PersonPart::FacialHair => "Facial Hair",
            PersonPart::Clothes => "Clothes",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MaskPrimitiveKind {
    AiSubject,
    AiSky,
    AiBackground,
    AiObjects { center_x: f32, center_y: f32, radius: f32 },
    AiPeople,
    PersonFeature { person_index: usize, part: PersonPart },
    LandscapeFeature(LandscapePart),
    Brush { points: Vec<(f32, f32)>, radius: f32, feather: f32, flow: f32 },
    LinearGradient { x1: f32, y1: f32, x2: f32, y2: f32 },
    RadialGradient { cx: f32, cy: f32, rx: f32, ry: f32, feather: f32, invert: bool },
    ColorRange { target_hue: f32, target_sat: f32, tolerance: f32 },
    LuminanceRange { min_lum: f32, max_lum: f32, feather: f32 },
    DepthRange { min_depth: f32, max_depth: f32, feather: f32 },
}

impl MaskPrimitiveKind {
    pub fn label(&self) -> &'static str {
        match self {
            MaskPrimitiveKind::AiSubject => "Subject (AI)",
            MaskPrimitiveKind::AiSky => "Sky (AI)",
            MaskPrimitiveKind::AiBackground => "Background (AI)",
            MaskPrimitiveKind::AiObjects { .. } => "Objects (AI)",
            MaskPrimitiveKind::AiPeople => "People (AI)",
            MaskPrimitiveKind::PersonFeature { part, .. } => part.label(),
            MaskPrimitiveKind::LandscapeFeature(part) => part.label(),
            MaskPrimitiveKind::Brush { .. } => "Brush",
            MaskPrimitiveKind::LinearGradient { .. } => "Linear Gradient",
            MaskPrimitiveKind::RadialGradient { .. } => "Radial Gradient",
            MaskPrimitiveKind::ColorRange { .. } => "Color Range",
            MaskPrimitiveKind::LuminanceRange { .. } => "Luminance Range",
            MaskPrimitiveKind::DepthRange { .. } => "Depth Range",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaskComponent {
    pub name: String,
    pub op: MaskOp,
    pub invert: bool,
    pub kind: MaskPrimitiveKind,
}

/// Exact local correction channels from CameraRaw `getCRLocalCorrectionsInfo` (0x62694)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalAdjustParams {
    pub amount: f32,      // 0..200 (default 100)
    pub exposure: f32,    // -4.0..+4.0
    pub contrast: f32,    // -100..+100
    pub highlights: f32,  // -100..+100
    pub shadows: f32,     // -100..+100
    pub whites: f32,      // -100..+100
    pub blacks: f32,      // -100..+100
    pub temperature: f32, // -100..+100
    pub tint: f32,        // -100..+100
    pub hue: f32,         // -180..+180
    pub saturation: f32,  // -100..+100
    pub texture: f32,     // -100..+100
    pub clarity: f32,     // -100..+100
    pub dehaze: f32,      // -100..+100
    pub sharpness: f32,   // -100..+100
    pub noise: f32,       // -100..+100
    pub moire: f32,       // -100..+100
    pub defringe: f32,    // -100..+100
    pub tint_color_hue: f32, // 0..360
    pub tint_color_sat: f32, // 0..100
}

impl Default for LocalAdjustParams {
    fn default() -> Self {
        Self {
            amount: 100.0,
            exposure: 0.0,
            contrast: 0.0,
            highlights: 0.0,
            shadows: 0.0,
            whites: 0.0,
            blacks: 0.0,
            temperature: 0.0,
            tint: 0.0,
            hue: 0.0,
            saturation: 0.0,
            texture: 0.0,
            clarity: 0.0,
            dehaze: 0.0,
            sharpness: 0.0,
            noise: 0.0,
            moire: 0.0,
            defringe: 0.0,
            tint_color_hue: 0.0,
            tint_color_sat: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaskGroup {
    pub id: u64,
    pub name: String,
    pub enabled: bool,
    pub invert: bool,
    pub components: Vec<MaskComponent>,
    pub local: LocalAdjustParams,
}

/// Complete Adobe Camera Raw / Lightroom Classic 15.4.1 Develop Parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DevelopParams {
    // --- 17 Boolean Panel & Mode Switches ---
    pub convert_to_grayscale: bool,
    pub auto_grayscale_mix: bool,
    pub auto_tone: bool,
    pub override_look_vignette: bool,
    pub enable_tone_curve: bool,
    pub enable_split_toning: bool, // Color Grading
    pub enable_grayscale_mix: bool,
    pub enable_color_adjustments: bool,
    pub enable_detail: bool,
    pub enable_lens_corrections: bool,
    pub enable_transform: bool,
    pub enable_effects: bool,
    pub enable_calibration: bool,
    pub enable_retouch: bool,
    pub enable_red_eye: bool,
    pub enable_mask_group_based_corrections: bool,
    pub enable_distraction_removal: bool,

    // --- Profile, Preset Amount & White Balance ---
    pub process_version: String, // "Version 6 (Current)"
    pub camera_profile: CameraProfile,
    pub look_amount: f32, // 0..200 (default 100)
    pub preset_amount: f32,
    #[serde(default)]
    pub applied_preset_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset_base_snapshot: Option<Box<DevelopParams>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset_target_snapshot: Option<Box<DevelopParams>>,
    pub white_balance: WhiteBalanceMode,
    pub temperature: f32, // 2000..50000 K
    pub tint: f32,        // -150..+150

    // --- Basic Tone (PV2012) ---
    pub exposure_2012: f32,   // -5.00..+5.00
    pub contrast_2012: f32,   // -100..+100
    pub highlights_2012: f32, // -100..+100
    pub shadows_2012: f32,    // -100..+100
    pub whites_2012: f32,     // -100..+100
    pub blacks_2012: f32,     // -100..+100

    // --- Basic Presence ---
    pub texture: f32,      // -100..+100
    pub clarity_2012: f32, // -100..+100
    pub dehaze: f32,       // -100..+100
    pub vibrance: f32,     // -100..+100
    pub saturation: f32,   // -100..+100

    // --- Tone Curve (Parametric + Point Curves) ---
    pub parametric_highlights: f32,      // -100..+100
    pub parametric_lights: f32,          // -100..+100
    pub parametric_darks: f32,           // -100..+100
    pub parametric_shadows: f32,         // -100..+100
    pub parametric_shadow_split: f32,    // 10..70 (default 25)
    pub parametric_midtone_split: f32,   // 20..80 (default 50)
    pub parametric_highlight_split: f32, // 30..90 (default 75)
    pub curve_refine_saturation: f32,    // 0..100 (default 100)
    pub tone_curve_name_2012: String,    // "Linear", "Medium Contrast", "Strong Contrast", "Custom"
    pub tone_curve_pv2012: Vec<(f32, f32)>,
    pub tone_curve_pv2012_red: Vec<(f32, f32)>,
    pub tone_curve_pv2012_green: Vec<(f32, f32)>,
    pub tone_curve_pv2012_blue: Vec<(f32, f32)>,

    // --- Color Mixer: 8-Band HSL & B&W Gray Mixer (Red, Orange, Yellow, Green, Aqua, Blue, Purple, Magenta) ---
    pub hue_adjustment: [f32; 8],        // -100..+100
    pub saturation_adjustment: [f32; 8], // -100..+100
    pub luminance_adjustment: [f32; 8],  // -100..+100
    pub gray_mixer: [f32; 8],            // -100..+100
    pub point_colors: Vec<PointColorSwatch>,
    #[serde(default)]
    pub point_color_visualize_range: bool,

    // --- Color Grading (Shadows, Midtones, Highlights, Global) ---
    pub split_toning_shadow_hue: f32,           // 0..360
    pub split_toning_shadow_saturation: f32,    // 0..100
    pub color_grade_shadow_lum: f32,            // -100..+100
    pub color_grade_midtone_hue: f32,           // 0..360
    pub color_grade_midtone_sat: f32,           // 0..100
    pub color_grade_midtone_lum: f32,           // -100..+100
    pub split_toning_highlight_hue: f32,        // 0..360
    pub split_toning_highlight_saturation: f32, // 0..100
    pub color_grade_highlight_lum: f32,         // -100..+100
    pub color_grade_global_hue: f32,            // 0..360
    pub color_grade_global_sat: f32,            // 0..100
    pub color_grade_global_lum: f32,            // -100..+100
    pub color_grade_blending: f32,              // 0..100 (default 50)
    pub split_toning_balance: f32,              // -100..+100 (default 0)
    #[serde(default = "default_true")]
    pub color_grade_enable_shadows: bool,
    #[serde(default = "default_true")]
    pub color_grade_enable_midtones: bool,
    #[serde(default = "default_true")]
    pub color_grade_enable_highlights: bool,
    #[serde(default = "default_true")]
    pub color_grade_enable_global: bool,

    // --- Detail (AI Denoise, Manual Noise Reduction, Sharpening) ---
    pub ai_denoise_amount: f32,                  // 0..100 (Enhance Denoise)
    pub super_resolution_enabled: bool,          // Enhance Super Resolution (2x)
    pub raw_details_enabled: bool,               // Enhance Raw Details
    pub sharpness: f32,                          // 0..150 (default 40)
    pub sharpen_radius: f32,                     // 0.5..3.0 (default 1.0)
    pub sharpen_detail: f32,                     // 0..100 (default 25)
    pub sharpen_edge_masking: f32,               // 0..100 (default 0)
    pub luminance_smoothing: f32,                // 0..100 (default 0)
    pub luminance_noise_reduction_detail: f32,   // 0..100 (default 50)
    pub luminance_noise_reduction_contrast: f32, // 0..100 (default 0)
    pub color_noise_reduction: f32,              // 0..100 (default 25)
    pub color_noise_reduction_detail: f32,       // 0..100 (default 50)
    pub color_noise_reduction_smoothness: f32,   // 0..100 (default 50)

    // --- Lens Blur (`LensBlur.*`) ---
    pub lens_blur_active: bool,
    pub lens_blur_amount: f32,           // 0..100 (default 50)
    pub lens_blur_bokeh: BokehShape,     // Circle, SoapBubble, Blade, Ring, CatEye, Anamorphic
    pub lens_blur_highlights_boost: f32, // 0..100 (default 50)
    #[serde(default = "default_fifty")]
    pub lens_blur_bokeh_boost_threshold: f32, // 0..100 (default 50)
    pub lens_blur_cat_eye_amount: f32,   // 0..100 (default 0)
    #[serde(default = "default_hundred")]
    pub lens_blur_cat_eye_scale: f32,    // 0..100 (default 100)
    pub lens_blur_focal_range: [f32; 4], // [near_fade, near_focus, far_focus, far_fade] in 0..100
    pub lens_blur_visualize_depth: bool,
    #[serde(default)]
    pub lens_blur_refinement_strokes: Vec<LensBlurStroke>,

    // --- Lens Corrections ---
    pub lens_profile_enable: bool,
    pub auto_lateral_ca: bool,
    pub lens_profile_name: String,
    pub lens_profile_distortion_scale: f32, // 0..200 (default 100)
    pub lens_profile_vignetting_scale: f32, // 0..200 (default 100)
    pub lens_manual_distortion_amount: f32, // -100..+100
    pub vignette_amount: f32,               // -100..+100
    pub vignette_midpoint: f32,             // 0..100 (default 50)
    pub defringe_purple_amount: f32,        // 0..20
    pub defringe_purple_hue_lo: f32,        // 0..90 (default 30)
    pub defringe_purple_hue_hi: f32,        // 10..100 (default 70)
    pub defringe_green_amount: f32,         // 0..20
    pub defringe_green_hue_lo: f32,         // 0..90 (default 40)
    pub defringe_green_hue_hi: f32,         // 10..100 (default 60)
    pub chromatic_aberration_r: f32,        // -100..+100
    pub chromatic_aberration_b: f32,        // -100..+100

    // --- Transform / Upright ---
    pub perspective_upright: UprightMode,
    pub perspective_vertical: f32,   // -100..+100
    pub perspective_horizontal: f32, // -100..+100
    pub perspective_rotate: f32,     // -10.0..+10.0
    pub perspective_aspect: f32,     // -100..+100
    pub perspective_scale: f32,      // 50..150 (default 100)
    pub perspective_x: f32,          // -100..+100
    pub perspective_y: f32,          // -100..+100
    pub crop_constrain_to_warp: bool,
    pub guided_upright_lines: Vec<([f32; 2], [f32; 2])>,

    // --- Effects (Post-Crop Vignetting & Film Grain) ---
    pub post_crop_vignette_style: VignetteStyle,
    pub post_crop_vignette_amount: f32,             // -100..+100
    pub post_crop_vignette_midpoint: f32,           // 0..100 (default 50)
    pub post_crop_vignette_roundness: f32,          // -100..+100 (default 0)
    pub post_crop_vignette_feather: f32,            // 0..100 (default 50)
    pub post_crop_vignette_highlight_contrast: f32, // 0..100 (default 0)
    pub grain_amount: f32,                          // 0..100 (default 0)
    pub grain_size: f32,                            // 0..100 (default 25)
    pub grain_frequency: f32,                       // 0..100 (default 50 - Roughness)

    // --- Calibration ---
    pub shadow_tint: f32,      // -100..+100
    pub red_hue: f32,          // -100..+100
    pub red_saturation: f32,   // -100..+100
    pub green_hue: f32,        // -100..+100
    pub green_saturation: f32, // -100..+100
    pub blue_hue: f32,         // -100..+100
    pub blue_saturation: f32,  // -100..+100

    // --- HDR / SDR Rendition Settings ---
    pub hdr_edit_mode: bool,
    pub hdr_max_value: f32, // 1.0..4.0 stops
    pub visualize_hdr_ranges: bool,
    pub preview_for_sdr_display: bool,
    pub sdr_brightness: f32, // -100..+100
    pub sdr_contrast: f32,   // -100..+100
    pub sdr_clarity: f32,    // -100..+100
    pub sdr_highlights: f32, // -100..+100
    pub sdr_shadows: f32,    // -100..+100
    pub sdr_whites: f32,     // -100..+100
    pub sdr_blend: f32,      // -100..+100

    // --- Crop & Straighten ---
    pub crop_top: f32,    // 0.0..1.0
    pub crop_left: f32,   // 0.0..1.0
    pub crop_bottom: f32, // 0.0..1.0
    pub crop_right: f32,  // 0.0..1.0
    pub crop_angle: f32,  // -45.0..+45.0 degrees
    pub crop_constrain_aspect_ratio: bool,
    pub crop_aspect_preset: AspectRatioPreset,
    #[serde(default)]
    pub crop_aspect_flipped: bool,
    #[serde(default = "default_crop_custom_ratio")]
    pub crop_custom_ratio: (f32, f32),
    pub crop_guide_overlay: CropGuideOverlay,
    #[serde(default)]
    pub crop_tool_overlay_mode: ToolOverlayMode,
    #[serde(default)]
    pub rotation_degrees: f32, // 0.0, 90.0, 180.0, 270.0
    #[serde(default)]
    pub flip_horizontal: bool,
    #[serde(default)]
    pub flip_vertical: bool,

    // --- Remove / Heal / Clone & Distraction Removal ---
    pub retouch_areas: Vec<RetouchArea>,
    pub distraction_removal_people: bool,
    pub distraction_removal_reflections: bool,
    #[serde(default)]
    pub retouch_visualize_spots: bool,
    #[serde(default = "default_fifty")]
    pub retouch_visualize_threshold: f32,

    // --- Red Eye Correction ---
    pub red_eye_spots: Vec<RedEyeSpot>,

    // --- Local Masking (`kMaskGroupBasedCorrectionsKey`) ---
    pub mask_groups: Vec<MaskGroup>,
}

impl Default for DevelopParams {
    fn default() -> Self {
        let linear_curve = vec![(0.0, 0.0), (1.0, 1.0)];
        Self {
            convert_to_grayscale: false,
            auto_grayscale_mix: false,
            auto_tone: false,
            override_look_vignette: false,
            enable_tone_curve: true,
            enable_split_toning: true,
            enable_grayscale_mix: true,
            enable_color_adjustments: true,
            enable_detail: true,
            enable_lens_corrections: true,
            enable_transform: true,
            enable_effects: true,
            enable_calibration: true,
            enable_retouch: true,
            enable_red_eye: true,
            enable_mask_group_based_corrections: true,
            enable_distraction_removal: true,

            process_version: "Version 6 (Current)".to_string(),
            camera_profile: CameraProfile::AdobeColor,
            look_amount: 100.0,
            preset_amount: 100.0,
            applied_preset_name: String::new(),
            preset_base_snapshot: None,
            preset_target_snapshot: None,
            white_balance: WhiteBalanceMode::AsShot,
            temperature: 5500.0,
            tint: 0.0,

            exposure_2012: 0.0,
            contrast_2012: 0.0,
            highlights_2012: 0.0,
            shadows_2012: 0.0,
            whites_2012: 0.0,
            blacks_2012: 0.0,

            texture: 0.0,
            clarity_2012: 0.0,
            dehaze: 0.0,
            vibrance: 0.0,
            saturation: 0.0,

            parametric_highlights: 0.0,
            parametric_lights: 0.0,
            parametric_darks: 0.0,
            parametric_shadows: 0.0,
            parametric_shadow_split: 25.0,
            parametric_midtone_split: 50.0,
            parametric_highlight_split: 75.0,
            curve_refine_saturation: 100.0,
            tone_curve_name_2012: "Linear".to_string(),
            tone_curve_pv2012: linear_curve.clone(),
            tone_curve_pv2012_red: linear_curve.clone(),
            tone_curve_pv2012_green: linear_curve.clone(),
            tone_curve_pv2012_blue: linear_curve,

            hue_adjustment: [0.0; 8],
            saturation_adjustment: [0.0; 8],
            luminance_adjustment: [0.0; 8],
            gray_mixer: [0.0; 8],
            point_colors: Vec::new(),
            point_color_visualize_range: false,

            split_toning_shadow_hue: 0.0,
            split_toning_shadow_saturation: 0.0,
            color_grade_shadow_lum: 0.0,
            color_grade_midtone_hue: 0.0,
            color_grade_midtone_sat: 0.0,
            color_grade_midtone_lum: 0.0,
            split_toning_highlight_hue: 0.0,
            split_toning_highlight_saturation: 0.0,
            color_grade_highlight_lum: 0.0,
            color_grade_global_hue: 0.0,
            color_grade_global_sat: 0.0,
            color_grade_global_lum: 0.0,
            color_grade_blending: 50.0,
            split_toning_balance: 0.0,
            color_grade_enable_shadows: true,
            color_grade_enable_midtones: true,
            color_grade_enable_highlights: true,
            color_grade_enable_global: true,

            ai_denoise_amount: 0.0,
            super_resolution_enabled: false,
            raw_details_enabled: false,
            sharpness: 40.0,
            sharpen_radius: 1.0,
            sharpen_detail: 25.0,
            sharpen_edge_masking: 0.0,
            luminance_smoothing: 0.0,
            luminance_noise_reduction_detail: 50.0,
            luminance_noise_reduction_contrast: 0.0,
            color_noise_reduction: 25.0,
            color_noise_reduction_detail: 50.0,
            color_noise_reduction_smoothness: 50.0,

            lens_blur_active: false,
            lens_blur_amount: 50.0,
            lens_blur_bokeh: BokehShape::Circle,
            lens_blur_highlights_boost: 50.0,
            lens_blur_bokeh_boost_threshold: 50.0,
            lens_blur_cat_eye_amount: 0.0,
            lens_blur_cat_eye_scale: 100.0,
            lens_blur_focal_range: [15.0, 25.0, 55.0, 70.0],
            lens_blur_visualize_depth: false,
            lens_blur_refinement_strokes: Vec::new(),

            lens_profile_enable: false,
            auto_lateral_ca: false,
            lens_profile_name: "Built-in Lens Profile Applied".to_string(),
            lens_profile_distortion_scale: 100.0,
            lens_profile_vignetting_scale: 100.0,
            lens_manual_distortion_amount: 0.0,
            vignette_amount: 0.0,
            vignette_midpoint: 50.0,
            defringe_purple_amount: 0.0,
            defringe_purple_hue_lo: 30.0,
            defringe_purple_hue_hi: 70.0,
            defringe_green_amount: 0.0,
            defringe_green_hue_lo: 40.0,
            defringe_green_hue_hi: 60.0,
            chromatic_aberration_r: 0.0,
            chromatic_aberration_b: 0.0,

            perspective_upright: UprightMode::Off,
            perspective_vertical: 0.0,
            perspective_horizontal: 0.0,
            perspective_rotate: 0.0,
            perspective_aspect: 0.0,
            perspective_scale: 100.0,
            perspective_x: 0.0,
            perspective_y: 0.0,
            crop_constrain_to_warp: false,
            guided_upright_lines: Vec::new(),

            post_crop_vignette_style: VignetteStyle::HighlightPriority,
            post_crop_vignette_amount: 0.0,
            post_crop_vignette_midpoint: 50.0,
            post_crop_vignette_roundness: 0.0,
            post_crop_vignette_feather: 50.0,
            post_crop_vignette_highlight_contrast: 0.0,
            grain_amount: 0.0,
            grain_size: 25.0,
            grain_frequency: 50.0,

            shadow_tint: 0.0,
            red_hue: 0.0,
            red_saturation: 0.0,
            green_hue: 0.0,
            green_saturation: 0.0,
            blue_hue: 0.0,
            blue_saturation: 0.0,

            hdr_edit_mode: false,
            hdr_max_value: 2.0,
            visualize_hdr_ranges: false,
            preview_for_sdr_display: false,
            sdr_brightness: 0.0,
            sdr_contrast: 0.0,
            sdr_clarity: 0.0,
            sdr_highlights: 0.0,
            sdr_shadows: 0.0,
            sdr_whites: 0.0,
            sdr_blend: 0.0,

            crop_top: 0.0,
            crop_left: 0.0,
            crop_bottom: 1.0,
            crop_right: 1.0,
            crop_angle: 0.0,
            crop_constrain_aspect_ratio: false,
            crop_aspect_preset: AspectRatioPreset::Original,
            crop_aspect_flipped: false,
            crop_custom_ratio: (1.0, 1.0),
            crop_guide_overlay: CropGuideOverlay::RuleOfThirds,
            crop_tool_overlay_mode: ToolOverlayMode::Auto,
            rotation_degrees: 0.0,
            flip_horizontal: false,
            flip_vertical: false,

            retouch_areas: Vec::new(),
            distraction_removal_people: false,
            distraction_removal_reflections: false,
            retouch_visualize_spots: false,
            retouch_visualize_threshold: 50.0,

            red_eye_spots: Vec::new(),
            mask_groups: Vec::new(),
        }
    }
}

pub const HSL_BAND_HUES: [f32; 8] = DevelopParams::HSL_BAND_HUES;

impl DevelopParams {
    pub const HSL_BAND_NAMES: [&'static str; 8] = [
        "Red", "Orange", "Yellow", "Green", "Aqua", "Blue", "Purple", "Magenta",
    ];

    /// Calibrated Adobe Camera Raw 8-band HSL center hues (degrees)
    pub const HSL_BAND_HUES: [f32; 8] = [
        0.0, 28.0, 58.0, 120.0, 180.0, 228.0, 275.0, 320.0,
    ];

    pub fn is_color_mixer_modified(&self) -> bool {
        self.is_mixer_modified()
    }

    pub fn is_color_grading_modified(&self) -> bool {
        self.is_grading_modified()
    }

    pub fn is_lens_corrections_modified(&self) -> bool {
        self.is_lens_corr_modified()
    }

    pub fn is_basic_modified(&self) -> bool {
        self.camera_profile != CameraProfile::AdobeColor
            || (self.look_amount - 100.0).abs() > 0.05
            || self.convert_to_grayscale
            || self.hdr_edit_mode
            || self.white_balance != WhiteBalanceMode::AsShot
            || self.exposure_2012.abs() > 0.005
            || self.contrast_2012.abs() > 0.05
            || self.highlights_2012.abs() > 0.05
            || self.shadows_2012.abs() > 0.05
            || self.whites_2012.abs() > 0.05
            || self.blacks_2012.abs() > 0.05
            || self.texture.abs() > 0.05
            || self.clarity_2012.abs() > 0.05
            || self.dehaze.abs() > 0.05
            || self.vibrance.abs() > 0.05
            || self.saturation.abs() > 0.05
    }

    pub fn is_curve_modified(&self) -> bool {
        let lin = [(0.0, 0.0), (1.0, 1.0)];
        self.parametric_highlights.abs() > 0.05
            || self.parametric_lights.abs() > 0.05
            || self.parametric_darks.abs() > 0.05
            || self.parametric_shadows.abs() > 0.05
            || (self.parametric_shadow_split - 25.0).abs() > 0.05
            || (self.parametric_midtone_split - 50.0).abs() > 0.05
            || (self.parametric_highlight_split - 75.0).abs() > 0.05
            || (self.curve_refine_saturation - 100.0).abs() > 0.05
            || self.tone_curve_pv2012.as_slice() != lin
            || self.tone_curve_pv2012_red.as_slice() != lin
            || self.tone_curve_pv2012_green.as_slice() != lin
            || self.tone_curve_pv2012_blue.as_slice() != lin
    }

    pub fn is_mixer_modified(&self) -> bool {
        self.hue_adjustment.iter().any(|v| v.abs() > 0.05)
            || self.saturation_adjustment.iter().any(|v| v.abs() > 0.05)
            || self.luminance_adjustment.iter().any(|v| v.abs() > 0.05)
            || (self.convert_to_grayscale && self.gray_mixer.iter().any(|v| v.abs() > 0.05))
            || self.point_colors.iter().any(|sw| {
                sw.hue_shift.abs() > 0.05 || sw.sat_shift.abs() > 0.05 || sw.lum_shift.abs() > 0.05
            })
    }

    pub fn is_grading_modified(&self) -> bool {
        self.split_toning_shadow_saturation > 0.05
            || self.color_grade_midtone_sat > 0.05
            || self.split_toning_highlight_saturation > 0.05
            || self.color_grade_global_sat > 0.05
            || self.color_grade_shadow_lum.abs() > 0.05
            || self.color_grade_midtone_lum.abs() > 0.05
            || self.color_grade_highlight_lum.abs() > 0.05
            || self.color_grade_global_lum.abs() > 0.05
    }

    pub fn is_detail_modified(&self) -> bool {
        self.ai_denoise_amount > 0.05
            || self.super_resolution_enabled
            || self.raw_details_enabled
            || (self.sharpness - 40.0).abs() > 0.05
            || (self.sharpen_radius - 1.0).abs() > 0.02
            || (self.sharpen_detail - 25.0).abs() > 0.05
            || self.sharpen_edge_masking > 0.05
            || self.luminance_smoothing > 0.05
            || (self.color_noise_reduction - 25.0).abs() > 0.05
    }

    pub fn is_lens_corr_modified(&self) -> bool {
        self.lens_profile_enable
            || self.auto_lateral_ca
            || self.lens_manual_distortion_amount.abs() > 0.05
            || self.vignette_amount.abs() > 0.05
            || self.defringe_purple_amount > 0.05
            || self.defringe_green_amount > 0.05
            || self.chromatic_aberration_r.abs() > 0.05
            || self.chromatic_aberration_b.abs() > 0.05
    }

    pub fn is_transform_modified(&self) -> bool {
        self.perspective_upright != UprightMode::Off
            || self.perspective_vertical.abs() > 0.05
            || self.perspective_horizontal.abs() > 0.05
            || self.perspective_rotate.abs() > 0.02
            || self.perspective_aspect.abs() > 0.05
            || (self.perspective_scale - 100.0).abs() > 0.05
            || self.perspective_x.abs() > 0.05
            || self.perspective_y.abs() > 0.05
            || !self.guided_upright_lines.is_empty()
    }

    pub fn is_lens_blur_modified(&self) -> bool {
        self.lens_blur_active
    }

    pub fn is_effects_modified(&self) -> bool {
        self.post_crop_vignette_amount.abs() > 0.05 || self.grain_amount > 0.05
    }

    pub fn is_calibration_modified(&self) -> bool {
        self.shadow_tint.abs() > 0.05
            || self.red_hue.abs() > 0.05
            || self.red_saturation.abs() > 0.05
            || self.green_hue.abs() > 0.05
            || self.green_saturation.abs() > 0.05
            || self.blue_hue.abs() > 0.05
            || self.blue_saturation.abs() > 0.05
    }

    pub fn is_cropped(&self) -> bool {
        self.crop_top > 0.001
            || self.crop_left > 0.001
            || self.crop_bottom < 0.999
            || self.crop_right < 0.999
            || self.crop_angle.abs() > 0.01
            || self.rotation_degrees.rem_euclid(360.0).abs() > 0.01
            || self.flip_horizontal
            || self.flip_vertical
    }

    pub fn has_masks(&self) -> bool {
        self.enable_mask_group_based_corrections && !self.mask_groups.is_empty()
    }

    pub fn is_modified(&self) -> bool {
        self.is_basic_modified()
            || self.is_curve_modified()
            || self.is_mixer_modified()
            || self.is_grading_modified()
            || self.is_detail_modified()
            || self.is_lens_corr_modified()
            || self.is_transform_modified()
            || self.is_lens_blur_modified()
            || self.is_effects_modified()
            || self.is_calibration_modified()
            || self.is_cropped()
            || !self.retouch_areas.is_empty()
            || self.distraction_removal_people
            || self.distraction_removal_reflections
            || !self.red_eye_spots.is_empty()
            || !self.mask_groups.is_empty()
    }

    pub fn apply_auto_tone(&mut self, mean_lum: f32, p5_lum: f32, p95_lum: f32) {
        self.auto_tone = true;
        let target_mean = 0.48;
        let ev_delta = ((target_mean / mean_lum.max(0.05)).log2() * 0.65).clamp(-2.5, 2.5);
        self.exposure_2012 = (ev_delta * 100.0).round() / 100.0;
        self.contrast_2012 = 12.0;
        self.highlights_2012 = if p95_lum > 0.82 { -45.0 } else { -22.0 };
        self.shadows_2012 = if p5_lum < 0.12 { 48.0 } else { 28.0 };
        self.whites_2012 = ((0.95 - p95_lum) * 80.0).clamp(-25.0, 35.0).round();
        self.blacks_2012 = ((0.06 - p5_lum) * 80.0).clamp(-35.0, 20.0).round();
        self.vibrance = 15.0;
        self.saturation = 3.0;
    }
}
