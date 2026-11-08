use crate::params::{
    BokehShape, CameraProfile, DevelopParams, LandscapePart, LocalAdjustParams, MaskComponent,
    MaskGroup, MaskOp, MaskPrimitiveKind, PersonPart, PointColorSwatch, UprightMode, VignetteStyle,
    WhiteBalanceMode,
};
use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopPreset {
    pub id: String,
    pub category: String,
    pub name: String,
    pub is_user: bool,
    pub params: DevelopParams,
    #[serde(default)]
    pub is_favorite: bool,
    #[serde(default)]
    pub mask: Option<CopySettingsMask>,
    #[serde(default = "default_true")]
    pub is_visible: bool,
    #[serde(default)]
    pub xmp_file_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CopySettingsMask {
    // 1. Treatment & Profile
    pub treatment_and_profile: bool,

    // 2. Basic
    pub basic_group: bool,
    pub basic_white_balance: bool,
    pub basic_exposure: bool,
    pub basic_contrast: bool,
    pub basic_highlights: bool,
    pub basic_shadows: bool,
    pub basic_whites: bool,
    pub basic_blacks: bool,
    pub basic_texture: bool,
    pub basic_clarity: bool,
    pub basic_dehaze: bool,
    pub basic_vibrance: bool,
    pub basic_saturation: bool,

    // 3. Curve
    pub curve_group: bool,
    pub curve_parametric: bool,
    pub curve_point: bool,

    // 4. Color Mixer
    pub color_mixer_group: bool,
    pub color_mixer_mixer: bool,
    pub color_mixer_point_color: bool,

    // 5. Color Grading
    pub color_grading: bool,

    // 6. Detail
    pub detail_group: bool,
    pub detail_denoise_raw: bool,
    pub detail_sharpening: bool,
    pub detail_luminance_nr: bool,
    pub detail_color_nr: bool,

    // 7. Lens Corrections
    pub lens_corrections_group: bool,
    pub lens_remove_ca: bool,
    pub lens_profile: bool,
    pub lens_manual_distortion: bool,
    pub lens_manual_vignetting: bool,

    // 8. Transform
    pub transform_group: bool,
    pub transform_upright_mode: bool,
    pub transform_upright_transforms: bool,
    pub transform_manual: bool,

    // 9. Lens Blur
    pub lens_blur_group: bool,
    pub lens_blur_focus_subject: bool,

    // 10. Effects
    pub effects_group: bool,
    pub effects_post_crop_vignette: bool,
    pub effects_grain: bool,

    // 11. Remove
    pub remove_group: bool,
    pub remove_manual: bool,
    pub remove_reflections: bool,
    pub remove_people: bool,
    pub remove_dust: bool,

    // 12. Crop
    pub crop_group: bool,
    pub crop_straighten_angle: bool,
    pub crop_aspect_ratio: bool,

    // 13. Process Version
    pub process_version: bool,

    // 14. Calibration
    pub calibration: bool,

    // 15. HDR
    pub hdr_group: bool,
    pub hdr_mode: bool,
    pub hdr_sdr_settings: bool,

    // 16. Masking
    pub masking: bool,
}

impl Default for CopySettingsMask {
    fn default() -> Self {
        Self {
            treatment_and_profile: true,
            basic_group: true,
            basic_white_balance: true,
            basic_exposure: true,
            basic_contrast: true,
            basic_highlights: true,
            basic_shadows: true,
            basic_whites: true,
            basic_blacks: true,
            basic_texture: true,
            basic_clarity: true,
            basic_dehaze: true,
            basic_vibrance: true,
            basic_saturation: true,
            curve_group: true,
            curve_parametric: true,
            curve_point: true,
            color_mixer_group: true,
            color_mixer_mixer: true,
            color_mixer_point_color: true,
            color_grading: true,
            detail_group: true,
            detail_denoise_raw: false,
            detail_sharpening: true,
            detail_luminance_nr: true,
            detail_color_nr: true,
            lens_corrections_group: true,
            lens_remove_ca: true,
            lens_profile: false,
            lens_manual_distortion: true,
            lens_manual_vignetting: true,
            transform_group: false,
            transform_upright_mode: false,
            transform_upright_transforms: false,
            transform_manual: false,
            lens_blur_group: false,
            lens_blur_focus_subject: false,
            effects_group: true,
            effects_post_crop_vignette: true,
            effects_grain: true,
            remove_group: false,
            remove_manual: false,
            remove_reflections: false,
            remove_people: false,
            remove_dust: false,
            crop_group: false,
            crop_straighten_angle: false,
            crop_aspect_ratio: false,
            process_version: true,
            calibration: true,
            hdr_group: true,
            hdr_mode: true,
            hdr_sdr_settings: false,
            masking: false,
        }
    }
}

impl CopySettingsMask {
    pub fn all_checked() -> Self {
        Self::all()
    }

    pub fn all() -> Self {
        Self {
            treatment_and_profile: true,
            basic_group: true,
            basic_white_balance: true,
            basic_exposure: true,
            basic_contrast: true,
            basic_highlights: true,
            basic_shadows: true,
            basic_whites: true,
            basic_blacks: true,
            basic_texture: true,
            basic_clarity: true,
            basic_dehaze: true,
            basic_vibrance: true,
            basic_saturation: true,
            curve_group: true,
            curve_parametric: true,
            curve_point: true,
            color_mixer_group: true,
            color_mixer_mixer: true,
            color_mixer_point_color: true,
            color_grading: true,
            detail_group: true,
            detail_denoise_raw: true,
            detail_sharpening: true,
            detail_luminance_nr: true,
            detail_color_nr: true,
            lens_corrections_group: true,
            lens_remove_ca: true,
            lens_profile: true,
            lens_manual_distortion: true,
            lens_manual_vignetting: true,
            transform_group: true,
            transform_upright_mode: true,
            transform_upright_transforms: true,
            transform_manual: true,
            lens_blur_group: true,
            lens_blur_focus_subject: true,
            effects_group: true,
            effects_post_crop_vignette: true,
            effects_grain: true,
            remove_group: true,
            remove_manual: true,
            remove_reflections: true,
            remove_people: true,
            remove_dust: true,
            crop_group: true,
            crop_straighten_angle: true,
            crop_aspect_ratio: true,
            process_version: true,
            calibration: true,
            hdr_group: true,
            hdr_mode: true,
            hdr_sdr_settings: true,
            masking: true,
        }
    }

    pub fn none() -> Self {
        Self {
            treatment_and_profile: false,
            basic_group: false,
            basic_white_balance: false,
            basic_exposure: false,
            basic_contrast: false,
            basic_highlights: false,
            basic_shadows: false,
            basic_whites: false,
            basic_blacks: false,
            basic_texture: false,
            basic_clarity: false,
            basic_dehaze: false,
            basic_vibrance: false,
            basic_saturation: false,
            curve_group: false,
            curve_parametric: false,
            curve_point: false,
            color_mixer_group: false,
            color_mixer_mixer: false,
            color_mixer_point_color: false,
            color_grading: false,
            detail_group: false,
            detail_denoise_raw: false,
            detail_sharpening: false,
            detail_luminance_nr: false,
            detail_color_nr: false,
            lens_corrections_group: false,
            lens_remove_ca: false,
            lens_profile: false,
            lens_manual_distortion: false,
            lens_manual_vignetting: false,
            transform_group: false,
            transform_upright_mode: false,
            transform_upright_transforms: false,
            transform_manual: false,
            lens_blur_group: false,
            lens_blur_focus_subject: false,
            effects_group: false,
            effects_post_crop_vignette: false,
            effects_grain: false,
            remove_group: false,
            remove_manual: false,
            remove_reflections: false,
            remove_people: false,
            remove_dust: false,
            crop_group: false,
            crop_straighten_angle: false,
            crop_aspect_ratio: false,
            process_version: false,
            calibration: false,
            hdr_group: false,
            hdr_mode: false,
            hdr_sdr_settings: false,
            masking: false,
        }
    }

    pub fn set_all(&mut self, val: bool) {
        if val {
            *self = Self::all();
        } else {
            *self = Self::none();
        }
    }

    /// Synchronizes parent group flags with the OR of their child flags
    pub fn sync_groups_from_children(&mut self) {
        self.basic_group = self.basic_white_balance
            || self.basic_exposure
            || self.basic_contrast
            || self.basic_highlights
            || self.basic_shadows
            || self.basic_whites
            || self.basic_blacks
            || self.basic_texture
            || self.basic_clarity
            || self.basic_dehaze
            || self.basic_vibrance
            || self.basic_saturation;
        self.curve_group = self.curve_parametric || self.curve_point;
        self.color_mixer_group = self.color_mixer_mixer || self.color_mixer_point_color;
        self.detail_group = self.detail_denoise_raw
            || self.detail_sharpening
            || self.detail_luminance_nr
            || self.detail_color_nr;
        self.lens_corrections_group = self.lens_remove_ca
            || self.lens_profile
            || self.lens_manual_distortion
            || self.lens_manual_vignetting;
        self.transform_group = self.transform_upright_mode
            || self.transform_upright_transforms
            || self.transform_manual;
        self.lens_blur_group = self.lens_blur_focus_subject;
        self.effects_group = self.effects_post_crop_vignette || self.effects_grain;
        self.remove_group = self.remove_manual
            || self.remove_reflections
            || self.remove_people
            || self.remove_dust;
        self.crop_group = self.crop_straighten_angle || self.crop_aspect_ratio;
        self.hdr_group = self.hdr_mode || self.hdr_sdr_settings;
    }

    pub fn modified(p: &DevelopParams) -> Self {
        let def = DevelopParams::default();
        let mut m = Self {
            treatment_and_profile: p.camera_profile != def.camera_profile
                || p.convert_to_grayscale != def.convert_to_grayscale
                || (p.look_amount - def.look_amount).abs() > 0.1,
            basic_group: false,
            basic_white_balance: p.white_balance != def.white_balance
                || (p.temperature - def.temperature).abs() > 0.5
                || (p.tint - def.tint).abs() > 0.2,
            basic_exposure: p.exposure_2012.abs() > 0.005,
            basic_contrast: p.contrast_2012.abs() > 0.05,
            basic_highlights: p.highlights_2012.abs() > 0.05,
            basic_shadows: p.shadows_2012.abs() > 0.05,
            basic_whites: p.whites_2012.abs() > 0.05,
            basic_blacks: p.blacks_2012.abs() > 0.05,
            basic_texture: p.texture.abs() > 0.05,
            basic_clarity: p.clarity_2012.abs() > 0.05,
            basic_dehaze: p.dehaze.abs() > 0.05,
            basic_vibrance: p.vibrance.abs() > 0.05,
            basic_saturation: p.saturation.abs() > 0.05,
            curve_group: false,
            curve_parametric: p.parametric_shadows.abs() > 0.05
                || p.parametric_darks.abs() > 0.05
                || p.parametric_lights.abs() > 0.05
                || p.parametric_highlights.abs() > 0.05
                || (p.parametric_shadow_split - 25.0).abs() > 0.05
                || (p.parametric_midtone_split - 50.0).abs() > 0.05
                || (p.parametric_highlight_split - 75.0).abs() > 0.05,
            curve_point: p.tone_curve_pv2012 != def.tone_curve_pv2012
                || p.tone_curve_pv2012_red != def.tone_curve_pv2012_red
                || p.tone_curve_pv2012_green != def.tone_curve_pv2012_green
                || p.tone_curve_pv2012_blue != def.tone_curve_pv2012_blue
                || (p.curve_refine_saturation - 100.0).abs() > 0.05,
            color_mixer_group: false,
            color_mixer_mixer: p.hue_adjustment != def.hue_adjustment
                || p.saturation_adjustment != def.saturation_adjustment
                || p.luminance_adjustment != def.luminance_adjustment
                || p.gray_mixer != def.gray_mixer,
            color_mixer_point_color: !p.point_colors.is_empty(),
            color_grading: p.is_grading_modified(),
            detail_group: false,
            detail_denoise_raw: p.ai_denoise_amount > 0.05
                || p.raw_details_enabled
                || p.super_resolution_enabled,
            detail_sharpening: (p.sharpness - def.sharpness).abs() > 0.1
                || (p.sharpen_radius - def.sharpen_radius).abs() > 0.02
                || (p.sharpen_detail - def.sharpen_detail).abs() > 0.1
                || p.sharpen_edge_masking > 0.1,
            detail_luminance_nr: p.luminance_smoothing > 0.1,
            detail_color_nr: (p.color_noise_reduction - def.color_noise_reduction).abs() > 0.1,
            lens_corrections_group: false,
            lens_remove_ca: p.auto_lateral_ca
                || p.defringe_purple_amount > 0.05
                || p.defringe_green_amount > 0.05
                || p.chromatic_aberration_r.abs() > 0.05
                || p.chromatic_aberration_b.abs() > 0.05,
            lens_profile: p.lens_profile_enable,
            lens_manual_distortion: p.lens_manual_distortion_amount.abs() > 0.05,
            lens_manual_vignetting: p.vignette_amount.abs() > 0.05,
            transform_group: false,
            transform_upright_mode: p.perspective_upright != def.perspective_upright,
            transform_upright_transforms: !p.guided_upright_lines.is_empty()
                || p.crop_constrain_to_warp,
            transform_manual: p.perspective_vertical.abs() > 0.05
                || p.perspective_horizontal.abs() > 0.05
                || p.perspective_rotate.abs() > 0.02
                || p.perspective_aspect.abs() > 0.05
                || (p.perspective_scale - 100.0).abs() > 0.05
                || p.perspective_x.abs() > 0.05
                || p.perspective_y.abs() > 0.05,
            lens_blur_group: p.lens_blur_active,
            lens_blur_focus_subject: p.lens_blur_active,
            effects_group: false,
            effects_post_crop_vignette: p.post_crop_vignette_amount.abs() > 0.05,
            effects_grain: p.grain_amount > 0.05,
            remove_group: false,
            remove_manual: !p.retouch_areas.is_empty(),
            remove_reflections: p.distraction_removal_reflections,
            remove_people: p.distraction_removal_people,
            remove_dust: !p.red_eye_spots.is_empty(),
            crop_group: false,
            crop_straighten_angle: p.crop_angle.abs() > 0.01,
            crop_aspect_ratio: p.crop_top > 0.001
                || p.crop_left > 0.001
                || p.crop_bottom < 0.999
                || p.crop_right < 0.999
                || p.crop_constrain_aspect_ratio,
            process_version: p.process_version != def.process_version,
            calibration: p.is_calibration_modified(),
            hdr_group: false,
            hdr_mode: p.hdr_edit_mode,
            hdr_sdr_settings: p.preview_for_sdr_display
                || p.sdr_brightness.abs() > 0.05
                || p.sdr_contrast.abs() > 0.05,
            masking: !p.mask_groups.is_empty(),
        };
        m.sync_groups_from_children();
        m
    }
}

pub fn apply_settings_with_mask(
    dest: &mut DevelopParams,
    src: &DevelopParams,
    mask: &CopySettingsMask,
) {
    if mask.process_version {
        dest.process_version = src.process_version.clone();
    }
    if mask.treatment_and_profile {
        dest.camera_profile = src.camera_profile;
        dest.look_amount = src.look_amount;
        dest.convert_to_grayscale = src.convert_to_grayscale;
        dest.auto_grayscale_mix = src.auto_grayscale_mix;
    }
    if mask.basic_white_balance {
        dest.white_balance = src.white_balance;
        dest.temperature = src.temperature;
        dest.tint = src.tint;
    }
    if mask.basic_exposure {
        dest.exposure_2012 = src.exposure_2012;
    }
    if mask.basic_contrast {
        dest.contrast_2012 = src.contrast_2012;
    }
    if mask.basic_highlights {
        dest.highlights_2012 = src.highlights_2012;
    }
    if mask.basic_shadows {
        dest.shadows_2012 = src.shadows_2012;
    }
    if mask.basic_whites {
        dest.whites_2012 = src.whites_2012;
    }
    if mask.basic_blacks {
        dest.blacks_2012 = src.blacks_2012;
    }
    if mask.basic_texture {
        dest.texture = src.texture;
    }
    if mask.basic_clarity {
        dest.clarity_2012 = src.clarity_2012;
    }
    if mask.basic_dehaze {
        dest.dehaze = src.dehaze;
    }
    if mask.basic_vibrance {
        dest.vibrance = src.vibrance;
    }
    if mask.basic_saturation {
        dest.saturation = src.saturation;
    }
    if mask.curve_parametric || mask.curve_point {
        dest.enable_tone_curve = src.enable_tone_curve;
    }
    if mask.curve_parametric {
        dest.parametric_shadows = src.parametric_shadows;
        dest.parametric_darks = src.parametric_darks;
        dest.parametric_lights = src.parametric_lights;
        dest.parametric_highlights = src.parametric_highlights;
        dest.parametric_shadow_split = src.parametric_shadow_split;
        dest.parametric_midtone_split = src.parametric_midtone_split;
        dest.parametric_highlight_split = src.parametric_highlight_split;
    }
    if mask.curve_point {
        dest.tone_curve_name_2012 = src.tone_curve_name_2012.clone();
        dest.curve_refine_saturation = src.curve_refine_saturation;
        dest.tone_curve_pv2012 = src.tone_curve_pv2012.clone();
        dest.tone_curve_pv2012_red = src.tone_curve_pv2012_red.clone();
        dest.tone_curve_pv2012_green = src.tone_curve_pv2012_green.clone();
        dest.tone_curve_pv2012_blue = src.tone_curve_pv2012_blue.clone();
    }
    if mask.color_mixer_mixer || mask.color_mixer_point_color {
        dest.enable_color_adjustments = src.enable_color_adjustments;
        dest.enable_grayscale_mix = src.enable_grayscale_mix;
    }
    if mask.color_mixer_mixer {
        dest.hue_adjustment = src.hue_adjustment;
        dest.saturation_adjustment = src.saturation_adjustment;
        dest.luminance_adjustment = src.luminance_adjustment;
        dest.gray_mixer = src.gray_mixer;
    }
    if mask.color_mixer_point_color {
        dest.point_colors = src.point_colors.clone();
    }
    if mask.color_grading {
        dest.enable_split_toning = src.enable_split_toning;
        dest.split_toning_shadow_hue = src.split_toning_shadow_hue;
        dest.split_toning_shadow_saturation = src.split_toning_shadow_saturation;
        dest.split_toning_highlight_hue = src.split_toning_highlight_hue;
        dest.split_toning_highlight_saturation = src.split_toning_highlight_saturation;
        dest.split_toning_balance = src.split_toning_balance;
        dest.color_grade_shadow_lum = src.color_grade_shadow_lum;
        dest.color_grade_midtone_hue = src.color_grade_midtone_hue;
        dest.color_grade_midtone_sat = src.color_grade_midtone_sat;
        dest.color_grade_midtone_lum = src.color_grade_midtone_lum;
        dest.color_grade_highlight_lum = src.color_grade_highlight_lum;
        dest.color_grade_blending = src.color_grade_blending;
        dest.color_grade_global_hue = src.color_grade_global_hue;
        dest.color_grade_global_sat = src.color_grade_global_sat;
        dest.color_grade_global_lum = src.color_grade_global_lum;
        dest.color_grade_enable_shadows = src.color_grade_enable_shadows;
        dest.color_grade_enable_midtones = src.color_grade_enable_midtones;
        dest.color_grade_enable_highlights = src.color_grade_enable_highlights;
        dest.color_grade_enable_global = src.color_grade_enable_global;
    }
    if mask.detail_sharpening || mask.detail_luminance_nr || mask.detail_color_nr || mask.detail_denoise_raw {
        dest.enable_detail = src.enable_detail;
    }
    if mask.detail_sharpening {
        dest.sharpness = src.sharpness;
        dest.sharpen_radius = src.sharpen_radius;
        dest.sharpen_detail = src.sharpen_detail;
        dest.sharpen_edge_masking = src.sharpen_edge_masking;
    }
    if mask.detail_luminance_nr {
        dest.luminance_smoothing = src.luminance_smoothing;
        dest.luminance_noise_reduction_detail = src.luminance_noise_reduction_detail;
        dest.luminance_noise_reduction_contrast = src.luminance_noise_reduction_contrast;
    }
    if mask.detail_color_nr {
        dest.color_noise_reduction = src.color_noise_reduction;
        dest.color_noise_reduction_detail = src.color_noise_reduction_detail;
        dest.color_noise_reduction_smoothness = src.color_noise_reduction_smoothness;
    }
    if mask.detail_denoise_raw {
        dest.ai_denoise_amount = src.ai_denoise_amount;
        dest.super_resolution_enabled = src.super_resolution_enabled;
        dest.raw_details_enabled = src.raw_details_enabled;
    }
    if mask.lens_remove_ca || mask.lens_profile || mask.lens_manual_distortion || mask.lens_manual_vignetting {
        dest.enable_lens_corrections = src.enable_lens_corrections;
    }
    if mask.lens_remove_ca {
        dest.auto_lateral_ca = src.auto_lateral_ca;
        dest.chromatic_aberration_r = src.chromatic_aberration_r;
        dest.chromatic_aberration_b = src.chromatic_aberration_b;
        dest.defringe_purple_amount = src.defringe_purple_amount;
        dest.defringe_purple_hue_lo = src.defringe_purple_hue_lo;
        dest.defringe_purple_hue_hi = src.defringe_purple_hue_hi;
        dest.defringe_green_amount = src.defringe_green_amount;
        dest.defringe_green_hue_lo = src.defringe_green_hue_lo;
        dest.defringe_green_hue_hi = src.defringe_green_hue_hi;
    }
    if mask.lens_profile {
        dest.lens_profile_enable = src.lens_profile_enable;
        dest.lens_profile_name = src.lens_profile_name.clone();
        dest.lens_profile_distortion_scale = src.lens_profile_distortion_scale;
        dest.lens_profile_vignetting_scale = src.lens_profile_vignetting_scale;
    }
    if mask.lens_manual_distortion {
        dest.lens_manual_distortion_amount = src.lens_manual_distortion_amount;
    }
    if mask.lens_manual_vignetting {
        dest.vignette_amount = src.vignette_amount;
        dest.vignette_midpoint = src.vignette_midpoint;
    }
    if mask.transform_upright_mode || mask.transform_upright_transforms || mask.transform_manual {
        dest.enable_transform = src.enable_transform;
    }
    if mask.transform_upright_mode {
        dest.perspective_upright = src.perspective_upright;
    }
    if mask.transform_upright_transforms {
        dest.guided_upright_lines = src.guided_upright_lines.clone();
        dest.crop_constrain_to_warp = src.crop_constrain_to_warp;
    }
    if mask.transform_manual {
        dest.perspective_vertical = src.perspective_vertical;
        dest.perspective_horizontal = src.perspective_horizontal;
        dest.perspective_rotate = src.perspective_rotate;
        dest.perspective_scale = src.perspective_scale;
        dest.perspective_aspect = src.perspective_aspect;
        dest.perspective_x = src.perspective_x;
        dest.perspective_y = src.perspective_y;
    }
    if mask.lens_blur_focus_subject || mask.lens_blur_group {
        dest.lens_blur_active = src.lens_blur_active;
        dest.lens_blur_amount = src.lens_blur_amount;
        dest.lens_blur_bokeh = src.lens_blur_bokeh;
        dest.lens_blur_focal_range = src.lens_blur_focal_range;
        dest.lens_blur_cat_eye_amount = src.lens_blur_cat_eye_amount;
        dest.lens_blur_cat_eye_scale = src.lens_blur_cat_eye_scale;
        dest.lens_blur_highlights_boost = src.lens_blur_highlights_boost;
        dest.lens_blur_bokeh_boost_threshold = src.lens_blur_bokeh_boost_threshold;
        dest.lens_blur_refinement_strokes = src.lens_blur_refinement_strokes.clone();
    }
    if mask.effects_post_crop_vignette || mask.effects_grain {
        dest.enable_effects = src.enable_effects;
    }
    if mask.effects_post_crop_vignette {
        dest.override_look_vignette = src.override_look_vignette;
        dest.post_crop_vignette_style = src.post_crop_vignette_style;
        dest.post_crop_vignette_amount = src.post_crop_vignette_amount;
        dest.post_crop_vignette_midpoint = src.post_crop_vignette_midpoint;
        dest.post_crop_vignette_roundness = src.post_crop_vignette_roundness;
        dest.post_crop_vignette_feather = src.post_crop_vignette_feather;
        dest.post_crop_vignette_highlight_contrast = src.post_crop_vignette_highlight_contrast;
    }
    if mask.effects_grain {
        dest.grain_amount = src.grain_amount;
        dest.grain_size = src.grain_size;
        dest.grain_frequency = src.grain_frequency;
    }
    if mask.remove_manual || mask.remove_dust {
        dest.enable_retouch = src.enable_retouch;
        dest.retouch_areas = src.retouch_areas.clone();
    }
    if mask.remove_dust {
        dest.enable_red_eye = src.enable_red_eye;
        dest.red_eye_spots = src.red_eye_spots.clone();
    }
    if mask.remove_reflections {
        dest.enable_distraction_removal = src.enable_distraction_removal;
        dest.distraction_removal_reflections = src.distraction_removal_reflections;
    }
    if mask.remove_people {
        dest.enable_distraction_removal = src.enable_distraction_removal;
        dest.distraction_removal_people = src.distraction_removal_people;
    }
    if mask.crop_straighten_angle {
        dest.crop_angle = src.crop_angle;
    }
    if mask.crop_aspect_ratio {
        dest.crop_aspect_preset = src.crop_aspect_preset;
        dest.crop_constrain_aspect_ratio = src.crop_constrain_aspect_ratio;
        dest.crop_aspect_flipped = src.crop_aspect_flipped;
        dest.crop_custom_ratio = src.crop_custom_ratio;
        dest.crop_top = src.crop_top;
        dest.crop_left = src.crop_left;
        dest.crop_bottom = src.crop_bottom;
        dest.crop_right = src.crop_right;
    }
    if mask.calibration {
        dest.enable_calibration = src.enable_calibration;
        dest.shadow_tint = src.shadow_tint;
        dest.red_hue = src.red_hue;
        dest.red_saturation = src.red_saturation;
        dest.green_hue = src.green_hue;
        dest.green_saturation = src.green_saturation;
        dest.blue_hue = src.blue_hue;
        dest.blue_saturation = src.blue_saturation;
    }
    if mask.hdr_mode {
        dest.hdr_edit_mode = src.hdr_edit_mode;
    }
    if mask.hdr_sdr_settings {
        dest.hdr_max_value = src.hdr_max_value;
        dest.preview_for_sdr_display = src.preview_for_sdr_display;
        dest.sdr_brightness = src.sdr_brightness;
        dest.sdr_contrast = src.sdr_contrast;
        dest.sdr_clarity = src.sdr_clarity;
        dest.sdr_highlights = src.sdr_highlights;
        dest.sdr_shadows = src.sdr_shadows;
        dest.sdr_whites = src.sdr_whites;
        dest.sdr_blend = src.sdr_blend;
    }
    if mask.masking {
        dest.enable_mask_group_based_corrections = src.enable_mask_group_based_corrections;
        dest.mask_groups = src.mask_groups.clone();
    }
}

/// Scales a preset's adjustments between `base` (`0%`), `target` (`100%`), and `2x` (`200%`)
/// matching Adobe Camera Raw `crs:SupportsAmount2="True"` (`Preset Amount` slider 0..200).
pub fn interpolate_preset_amount(
    base: &DevelopParams,
    target: &DevelopParams,
    amount_pct: f32,
) -> DevelopParams {
    let t = (amount_pct / 100.0).clamp(0.0, 2.0);
    let lerp = |a: f32, b: f32| -> f32 { a + (b - a) * t };

    let mut out = target.clone();
    out.preset_amount = amount_pct.clamp(0.0, 200.0);
    out.look_amount = lerp(100.0, target.look_amount).clamp(0.0, 200.0);

    if target.white_balance != WhiteBalanceMode::AsShot || base.white_balance != WhiteBalanceMode::AsShot {
        out.temperature = lerp(base.temperature, target.temperature).clamp(2000.0, 50000.0);
        out.tint = lerp(base.tint, target.tint).clamp(-150.0, 150.0);
        if t < 0.01 {
            out.white_balance = base.white_balance;
        }
    }

    out.exposure_2012 = lerp(base.exposure_2012, target.exposure_2012).clamp(-5.0, 5.0);
    out.contrast_2012 = lerp(base.contrast_2012, target.contrast_2012).clamp(-100.0, 100.0);
    out.highlights_2012 = lerp(base.highlights_2012, target.highlights_2012).clamp(-100.0, 100.0);
    out.shadows_2012 = lerp(base.shadows_2012, target.shadows_2012).clamp(-100.0, 100.0);
    out.whites_2012 = lerp(base.whites_2012, target.whites_2012).clamp(-100.0, 100.0);
    out.blacks_2012 = lerp(base.blacks_2012, target.blacks_2012).clamp(-100.0, 100.0);
    out.texture = lerp(base.texture, target.texture).clamp(-100.0, 100.0);
    out.clarity_2012 = lerp(base.clarity_2012, target.clarity_2012).clamp(-100.0, 100.0);
    out.dehaze = lerp(base.dehaze, target.dehaze).clamp(-100.0, 100.0);
    out.vibrance = lerp(base.vibrance, target.vibrance).clamp(-100.0, 100.0);
    out.saturation = lerp(base.saturation, target.saturation).clamp(-100.0, 100.0);

    out.parametric_shadows = lerp(base.parametric_shadows, target.parametric_shadows).clamp(-100.0, 100.0);
    out.parametric_darks = lerp(base.parametric_darks, target.parametric_darks).clamp(-100.0, 100.0);
    out.parametric_lights = lerp(base.parametric_lights, target.parametric_lights).clamp(-100.0, 100.0);
    out.parametric_highlights = lerp(base.parametric_highlights, target.parametric_highlights).clamp(-100.0, 100.0);

    // Blend point curves toward identity y=x
    let scale_curve = |pts: &[(f32, f32)]| -> Vec<(f32, f32)> {
        pts.iter()
            .map(|&(x, y)| (x, (x + (y - x) * t).clamp(0.0, 1.0)))
            .collect()
    };
    out.tone_curve_pv2012 = scale_curve(&target.tone_curve_pv2012);
    out.tone_curve_pv2012_red = scale_curve(&target.tone_curve_pv2012_red);
    out.tone_curve_pv2012_green = scale_curve(&target.tone_curve_pv2012_green);
    out.tone_curve_pv2012_blue = scale_curve(&target.tone_curve_pv2012_blue);

    for i in 0..8 {
        out.hue_adjustment[i] = lerp(base.hue_adjustment[i], target.hue_adjustment[i]).clamp(-100.0, 100.0);
        out.saturation_adjustment[i] =
            lerp(base.saturation_adjustment[i], target.saturation_adjustment[i]).clamp(-100.0, 100.0);
        out.luminance_adjustment[i] =
            lerp(base.luminance_adjustment[i], target.luminance_adjustment[i]).clamp(-100.0, 100.0);
        out.gray_mixer[i] = lerp(base.gray_mixer[i], target.gray_mixer[i]).clamp(-100.0, 100.0);
    }

    for sw in &mut out.point_colors {
        sw.hue_shift = (sw.hue_shift * t).clamp(-100.0, 100.0);
        sw.sat_shift = (sw.sat_shift * t).clamp(-100.0, 100.0);
        sw.lum_shift = (sw.lum_shift * t).clamp(-100.0, 100.0);
    }

    out.split_toning_shadow_saturation =
        lerp(base.split_toning_shadow_saturation, target.split_toning_shadow_saturation).clamp(0.0, 100.0);
    out.color_grade_midtone_sat =
        lerp(base.color_grade_midtone_sat, target.color_grade_midtone_sat).clamp(0.0, 100.0);
    out.split_toning_highlight_saturation =
        lerp(base.split_toning_highlight_saturation, target.split_toning_highlight_saturation).clamp(0.0, 100.0);
    out.color_grade_global_sat =
        lerp(base.color_grade_global_sat, target.color_grade_global_sat).clamp(0.0, 100.0);
    out.color_grade_shadow_lum =
        lerp(base.color_grade_shadow_lum, target.color_grade_shadow_lum).clamp(-100.0, 100.0);
    out.color_grade_midtone_lum =
        lerp(base.color_grade_midtone_lum, target.color_grade_midtone_lum).clamp(-100.0, 100.0);
    out.color_grade_highlight_lum =
        lerp(base.color_grade_highlight_lum, target.color_grade_highlight_lum).clamp(-100.0, 100.0);
    out.color_grade_global_lum =
        lerp(base.color_grade_global_lum, target.color_grade_global_lum).clamp(-100.0, 100.0);

    out.sharpness = lerp(base.sharpness, target.sharpness).clamp(0.0, 150.0);
    out.luminance_smoothing = lerp(base.luminance_smoothing, target.luminance_smoothing).clamp(0.0, 100.0);
    out.color_noise_reduction = lerp(base.color_noise_reduction, target.color_noise_reduction).clamp(0.0, 100.0);
    out.lens_blur_amount = lerp(0.0, target.lens_blur_amount).clamp(0.0, 100.0);

    out.post_crop_vignette_amount =
        lerp(base.post_crop_vignette_amount, target.post_crop_vignette_amount).clamp(-100.0, 100.0);
    out.grain_amount = lerp(base.grain_amount, target.grain_amount).clamp(0.0, 100.0);

    out.shadow_tint = lerp(base.shadow_tint, target.shadow_tint).clamp(-100.0, 100.0);
    out.red_hue = lerp(base.red_hue, target.red_hue).clamp(-100.0, 100.0);
    out.red_saturation = lerp(base.red_saturation, target.red_saturation).clamp(-100.0, 100.0);
    out.green_hue = lerp(base.green_hue, target.green_hue).clamp(-100.0, 100.0);
    out.green_saturation = lerp(base.green_saturation, target.green_saturation).clamp(-100.0, 100.0);
    out.blue_hue = lerp(base.blue_hue, target.blue_hue).clamp(-100.0, 100.0);
    out.blue_saturation = lerp(base.blue_saturation, target.blue_saturation).clamp(-100.0, 100.0);

    for mg in &mut out.mask_groups {
        mg.local.amount = (mg.local.amount * t).clamp(0.0, 200.0);
    }

    out
}

fn xml_unescape(s: &str) -> String {
    s.replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Extracts an exact XML tag block `<tag>...</tag>` or `<tag ...>...</tag>` without prefix-matching `<tagRed>`!
fn extract_tag_block(xml: &str, tag: &str) -> Option<String> {
    let start_pat = format!("<{}", tag);
    let close_tag = format!("</{}>", tag);
    let mut search_from = 0;
    while let Some(rel_pos) = xml[search_from..].find(&start_pat) {
        let s_pos = search_from + rel_pos;
        let after_name = s_pos + start_pat.len();
        let next_ch = xml[after_name..].chars().next();
        match next_ch {
            Some('>') | Some(' ') | Some('\t') | Some('\n') | Some('\r') => {
                let rest = &xml[after_name..];
                if let Some(gt_pos) = rest.find('>') {
                    // Self-closing `<tag .../>` has empty inner body
                    if gt_pos > 0 && rest.as_bytes()[gt_pos - 1] == b'/' {
                        return Some(String::new());
                    }
                    let inner_start = after_name + gt_pos + 1;
                    if let Some(e_pos) = xml[inner_start..].find(&close_tag) {
                        return Some(xml[inner_start..inner_start + e_pos].to_string());
                    }
                }
            }
            Some('/') => {
                return Some(String::new());
            }
            _ => {}
        }
        search_from = after_name;
    }
    None
}

/// Extracts the opening tag itself (`<tag ...>` or `<tag .../>`) so attributes on child tags like `<crs:LensBlur .../>` can be read
fn extract_opening_tag(xml: &str, tag: &str) -> Option<String> {
    let start_pat = format!("<{}", tag);
    let mut search_from = 0;
    while let Some(rel_pos) = xml[search_from..].find(&start_pat) {
        let s_pos = search_from + rel_pos;
        let after_name = s_pos + start_pat.len();
        let next_ch = xml[after_name..].chars().next();
        if matches!(
            next_ch,
            Some('>') | Some('/') | Some(' ') | Some('\t') | Some('\n') | Some('\r')
        ) {
            if let Some(gt_pos) = xml[s_pos..].find('>') {
                return Some(xml[s_pos..=s_pos + gt_pos].to_string());
            }
        }
        search_from = after_name;
    }
    None
}

fn extract_all_tag_blocks(xml: &str, tag: &str) -> Vec<String> {
    let mut results = Vec::new();
    let start_pat = format!("<{}", tag);
    let close_tag = format!("</{}>", tag);
    let mut cursor = 0;
    while let Some(s_pos) = xml[cursor..].find(&start_pat) {
        let abs_s = cursor + s_pos;
        let after_name = abs_s + start_pat.len();
        let next_ch = xml[after_name..].chars().next();
        if matches!(
            next_ch,
            Some('>') | Some('/') | Some(' ') | Some('\t') | Some('\n') | Some('\r')
        ) {
            if let Some(gt_pos) = xml[after_name..].find('>') {
                if gt_pos > 0 && xml[after_name..].as_bytes()[gt_pos - 1] == b'/' {
                    results.push(String::new());
                    cursor = after_name + gt_pos + 1;
                    continue;
                }
                let inner_start = after_name + gt_pos + 1;
                if let Some(e_pos) = xml[inner_start..].find(&close_tag) {
                    results.push(xml[inner_start..inner_start + e_pos].trim().to_string());
                    cursor = inner_start + e_pos + close_tag.len();
                    continue;
                }
            }
        }
        cursor = after_name;
    }
    results
}

/// Reads an XMP attribute `crs:Foo="val"` OR element `<crs:Foo>val</crs:Foo>` safely with word-boundary verification
fn extract_attr_str(xml: &str, attr: &str) -> Option<String> {
    for quote in ['"', '\''] {
        let key = format!("{}={}", attr, quote);
        let mut cursor = 0;
        while let Some(rel) = xml[cursor..].find(&key) {
            let pos = cursor + rel;
            let valid_boundary = pos == 0 || {
                let prev = xml.as_bytes()[pos - 1];
                prev.is_ascii_whitespace() || prev == b'<'
            };
            if valid_boundary {
                let rest = &xml[pos + key.len()..];
                if let Some(end) = rest.find(quote) {
                    return Some(xml_unescape(&rest[..end]));
                }
            }
            cursor = pos + key.len();
        }
    }
    if let Some(elem) = extract_tag_block(xml, attr) {
        let trimmed = elem.trim();
        if !trimmed.starts_with('<') && !trimmed.is_empty() {
            return Some(xml_unescape(trimmed));
        }
    }
    None
}

fn extract_attr_f32(xml: &str, attr: &str) -> Option<f32> {
    extract_attr_str(xml, attr).and_then(|s| s.trim().parse::<f32>().ok())
}

fn extract_attr_bool(xml: &str, attr: &str) -> Option<bool> {
    extract_attr_str(xml, attr).map(|s| {
        matches!(
            s.trim().to_ascii_lowercase().as_str(),
            "true" | "1" | "yes"
        )
    })
}

fn parse_curve_seq(xml: &str, tag: &str) -> Option<Vec<(f32, f32)>> {
    let block = extract_tag_block(xml, tag)?;
    let mut pts = Vec::new();
    for li in extract_all_tag_blocks(&block, "rdf:li") {
        let parts: Vec<&str> = li.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            if let (Ok(x), Ok(y)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>()) {
                pts.push(((x / 255.0).clamp(0.0, 1.0), (y / 255.0).clamp(0.0, 1.0)));
            }
        }
    }
    if pts.len() >= 2 {
        Some(pts)
    } else {
        None
    }
}

fn remove_nested_block(xml: &str, tag: &str) -> String {
    let start_pat = format!("<{}", tag);
    let close_tag = format!("</{}>", tag);
    if let Some(s) = xml.find(&start_pat) {
        if let Some(e) = xml[s..].find(&close_tag) {
            let mut cleaned = String::with_capacity(xml.len());
            cleaned.push_str(&xml[..s]);
            cleaned.push_str(&xml[s + e + close_tag.len()..]);
            return cleaned;
        }
    }
    xml.to_string()
}

pub fn parse_xmp_preset(content: &str, fallback_name: &str) -> Option<DevelopPreset> {
    if !content.contains("camera-raw-settings")
        && !content.contains("crs:")
        && !content.contains("xmpmeta")
    {
        return None;
    }

    // Isolate `<crs:Look>` first so its nested `crs:Name="Adobe Color"` & `<crs:Group>Profiles</crs:Group>`
    // never overwrite the preset's own Name or Group!
    let look_block = extract_tag_block(content, "crs:Look").or_else(|| extract_opening_tag(content, "crs:Look"));
    let main_xml = remove_nested_block(content, "crs:Look");

    let mut params = DevelopParams::default();
    let mut mask = CopySettingsMask::none();
    mask.process_version = true;

    // 1. Preset Name
    let mut name = String::new();
    if let Some(n_block) = extract_tag_block(&main_xml, "crs:Name") {
        if let Some(li) = extract_tag_block(&n_block, "rdf:li") {
            name = xml_unescape(li.trim());
        }
    }
    if name.is_empty() {
        if let Some(n) = extract_attr_str(&main_xml, "crs:Name") {
            name = n;
        }
    }
    if name.is_empty() {
        name = fallback_name.to_string();
    }

    // 2. Preset Group
    let mut group = "User Presets".to_string();
    if let Some(g_block) = extract_tag_block(&main_xml, "crs:Group") {
        if let Some(li) = extract_tag_block(&g_block, "rdf:li") {
            let g = xml_unescape(li.trim());
            if !g.is_empty() {
                group = g;
            }
        }
    }
    if group == "User Presets" {
        if let Some(g) = extract_attr_str(&main_xml, "crs:Group") {
            if !g.is_empty() {
                group = g;
            }
        }
    }

    // 3. Look / Camera Profile & Monochrome Treatment
    if let Some(ref lb) = look_block {
        if let Some(prof_name) = extract_attr_str(lb, "crs:Name") {
            mask.treatment_and_profile = true;
            params.camera_profile = match prof_name.trim().to_ascii_lowercase().as_str() {
                "adobe vivid" => CameraProfile::AdobeVivid,
                "adobe landscape" => CameraProfile::AdobeLandscape,
                "adobe portrait" => CameraProfile::AdobePortrait,
                "adobe neutral" => CameraProfile::AdobeNeutral,
                "adobe standard" => CameraProfile::AdobeStandard,
                "adobe monochrome" => {
                    params.convert_to_grayscale = true;
                    CameraProfile::AdobeMonochrome
                }
                "camera standard" => CameraProfile::CameraStandard,
                "camera faithful" => CameraProfile::CameraFaithful,
                "camera positive film" => CameraProfile::CameraPositiveFilm,
                _ => CameraProfile::AdobeColor,
            };
        }
        if let Some(amt) = extract_attr_f32(lb, "crs:Amount") {
            params.look_amount = (amt * 100.0).clamp(0.0, 200.0);
        }
    }
    if let Some(prof_attr) = extract_attr_str(&main_xml, "crs:CameraProfile") {
        mask.treatment_and_profile = true;
        params.camera_profile = match prof_attr.trim().to_ascii_lowercase().as_str() {
            s if s.contains("vivid") => CameraProfile::AdobeVivid,
            s if s.contains("landscape") => CameraProfile::AdobeLandscape,
            s if s.contains("portrait") => CameraProfile::AdobePortrait,
            s if s.contains("neutral") => CameraProfile::AdobeNeutral,
            s if s.contains("monochrome") => CameraProfile::AdobeMonochrome,
            s if s.contains("standard") && s.contains("camera") => CameraProfile::CameraStandard,
            s if s.contains("standard") => CameraProfile::AdobeStandard,
            _ => CameraProfile::AdobeColor,
        };
    }
    if let Some(bw) = extract_attr_bool(&main_xml, "crs:ConvertToGrayscale") {
        params.convert_to_grayscale = bw;
        mask.treatment_and_profile = true;
    }
    if let Some(agm) = extract_attr_bool(&main_xml, "crs:AutoGrayscaleMix") {
        params.auto_grayscale_mix = agm;
    }

    // 4. White Balance (`As Shot`, `AsShot`, `Auto`, `Custom`, presets)
    if let Some(wb) = extract_attr_str(&main_xml, "crs:WhiteBalance") {
        mask.basic_white_balance = true;
        params.white_balance = match wb.trim().to_ascii_lowercase().as_str() {
            "as shot" | "asshot" => WhiteBalanceMode::AsShot,
            "auto" => WhiteBalanceMode::Auto,
            "daylight" => WhiteBalanceMode::Daylight,
            "cloudy" => WhiteBalanceMode::Cloudy,
            "shade" => WhiteBalanceMode::Shade,
            "tungsten" | "incandescent" => WhiteBalanceMode::Tungsten,
            "fluorescent" => WhiteBalanceMode::Fluorescent,
            "flash" => WhiteBalanceMode::Flash,
            _ => WhiteBalanceMode::Custom,
        };
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:Temperature").or_else(|| extract_attr_f32(&main_xml, "crs:IncrementalTemperature")) {
        params.temperature = if v < 500.0 { (5500.0 + v * 25.0).clamp(2000.0, 50000.0) } else { v };
        mask.basic_white_balance = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:Tint").or_else(|| extract_attr_f32(&main_xml, "crs:IncrementalTint")) {
        params.tint = v;
        mask.basic_white_balance = true;
    }

    // 5. Basic Tone & Presence (PV2012)
    if let Some(v) = extract_attr_f32(&main_xml, "crs:Exposure2012") {
        params.exposure_2012 = v;
        mask.basic_exposure = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:Contrast2012") {
        params.contrast_2012 = v;
        mask.basic_contrast = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:Highlights2012") {
        params.highlights_2012 = v;
        mask.basic_highlights = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:Shadows2012") {
        params.shadows_2012 = v;
        mask.basic_shadows = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:Whites2012") {
        params.whites_2012 = v;
        mask.basic_whites = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:Blacks2012") {
        params.blacks_2012 = v;
        mask.basic_blacks = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:Texture") {
        params.texture = v;
        mask.basic_texture = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:Clarity2012") {
        params.clarity_2012 = v;
        mask.basic_clarity = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:Dehaze") {
        params.dehaze = v;
        mask.basic_dehaze = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:Vibrance") {
        params.vibrance = v;
        mask.basic_vibrance = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:Saturation") {
        params.saturation = v;
        mask.basic_saturation = true;
    }

    // 6. Detail: Sharpening & Noise Reduction
    if let Some(v) = extract_attr_f32(&main_xml, "crs:Sharpness") {
        params.sharpness = v;
        mask.detail_sharpening = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:SharpenRadius") {
        params.sharpen_radius = v;
        mask.detail_sharpening = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:SharpenDetail") {
        params.sharpen_detail = v;
        mask.detail_sharpening = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:SharpenEdgeMasking") {
        params.sharpen_edge_masking = v;
        mask.detail_sharpening = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:LuminanceSmoothing") {
        params.luminance_smoothing = v;
        mask.detail_luminance_nr = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:LuminanceNoiseReductionDetail") {
        params.luminance_noise_reduction_detail = v;
        mask.detail_luminance_nr = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:LuminanceNoiseReductionContrast") {
        params.luminance_noise_reduction_contrast = v;
        mask.detail_luminance_nr = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:ColorNoiseReduction") {
        params.color_noise_reduction = v;
        mask.detail_color_nr = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:ColorNoiseReductionDetail") {
        params.color_noise_reduction_detail = v;
        mask.detail_color_nr = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:ColorNoiseReductionSmoothness") {
        params.color_noise_reduction_smoothness = v;
        mask.detail_color_nr = true;
    }
    if let Some(en) = extract_attr_bool(&main_xml, "crs:EnableDetail") {
        params.enable_detail = en;
    }

    // 7. Parametric & 4-Channel Point Curves
    if let Some(v) = extract_attr_f32(&main_xml, "crs:ParametricShadows") {
        params.parametric_shadows = v;
        mask.curve_parametric = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:ParametricDarks") {
        params.parametric_darks = v;
        mask.curve_parametric = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:ParametricLights") {
        params.parametric_lights = v;
        mask.curve_parametric = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:ParametricHighlights") {
        params.parametric_highlights = v;
        mask.curve_parametric = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:ParametricShadowSplit") {
        params.parametric_shadow_split = v;
        mask.curve_parametric = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:ParametricMidtoneSplit") {
        params.parametric_midtone_split = v;
        mask.curve_parametric = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:ParametricHighlightSplit") {
        params.parametric_highlight_split = v;
        mask.curve_parametric = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:CurveRefineSaturation") {
        params.curve_refine_saturation = v;
        mask.curve_point = true;
    }
    if let Some(n) = extract_attr_str(&main_xml, "crs:ToneCurveName2012") {
        params.tone_curve_name_2012 = n;
        mask.curve_point = true;
    }
    if let Some(pts) = parse_curve_seq(&main_xml, "crs:ToneCurvePV2012") {
        params.tone_curve_pv2012 = pts;
        mask.curve_point = true;
    }
    if let Some(pts) = parse_curve_seq(&main_xml, "crs:ToneCurvePV2012Red") {
        params.tone_curve_pv2012_red = pts;
        mask.curve_point = true;
    }
    if let Some(pts) = parse_curve_seq(&main_xml, "crs:ToneCurvePV2012Green") {
        params.tone_curve_pv2012_green = pts;
        mask.curve_point = true;
    }
    if let Some(pts) = parse_curve_seq(&main_xml, "crs:ToneCurvePV2012Blue") {
        params.tone_curve_pv2012_blue = pts;
        mask.curve_point = true;
    }
    if let Some(en) = extract_attr_bool(&main_xml, "crs:EnableToneCurve") {
        params.enable_tone_curve = en;
    }

    // 8. 8-Band HSL, Grayscale Mixer & `<crs:PointColors>`
    let hsl_keys = [
        ("Red", 0),
        ("Orange", 1),
        ("Yellow", 2),
        ("Green", 3),
        ("Aqua", 4),
        ("Blue", 5),
        ("Purple", 6),
        ("Magenta", 7),
    ];
    for (color, idx) in hsl_keys {
        if let Some(v) = extract_attr_f32(&main_xml, &format!("crs:HueAdjustment{}", color)) {
            params.hue_adjustment[idx] = v;
            mask.color_mixer_mixer = true;
        }
        if let Some(v) = extract_attr_f32(&main_xml, &format!("crs:SaturationAdjustment{}", color)) {
            params.saturation_adjustment[idx] = v;
            mask.color_mixer_mixer = true;
        }
        if let Some(v) = extract_attr_f32(&main_xml, &format!("crs:LuminanceAdjustment{}", color)) {
            params.luminance_adjustment[idx] = v;
            mask.color_mixer_mixer = true;
        }
        if let Some(v) = extract_attr_f32(&main_xml, &format!("crs:GrayMixer{}", color)) {
            params.gray_mixer[idx] = v;
            mask.color_mixer_mixer = true;
        }
    }
    if let Some(pc_block) = extract_tag_block(&main_xml, "crs:PointColors") {
        let mut swatches = Vec::new();
        for (idx, li) in extract_all_tag_blocks(&pc_block, "rdf:li").into_iter().enumerate() {
            let nums: Vec<f32> = li
                .split(',')
                .filter_map(|s| s.trim().parse::<f32>().ok())
                .collect();
            if nums.len() >= 7 {
                // ACR stores `src_hue` in radians `[0 .. 2π]` and shifts in `[-1 .. +1]`!
                let src_hue_deg = if nums[0] <= std::f32::consts::TAU + 0.01 {
                    nums[0].to_degrees().rem_euclid(360.0)
                } else {
                    nums[0].rem_euclid(360.0)
                };
                let scale_shift = |v: f32| -> f32 {
                    if v.abs() <= 1.0001 {
                        v * 100.0
                    } else {
                        v.clamp(-100.0, 100.0)
                    }
                };
                swatches.push(PointColorSwatch {
                    name: format!("Swatch {}", idx + 1),
                    src_hue: src_hue_deg,
                    src_sat: nums[1].clamp(0.0, 1.0),
                    src_lum: nums[2].clamp(0.0, 1.0),
                    hue_shift: scale_shift(nums[3]),
                    sat_shift: scale_shift(nums[4]),
                    lum_shift: scale_shift(nums[5]),
                    range_amount: (nums[6] * 100.0).clamp(0.0, 100.0),
                    hue_range: 50.0,
                    sat_range: 50.0,
                    lum_range: 50.0,
                });
            }
        }
        if !swatches.is_empty() {
            params.point_colors = swatches;
            mask.color_mixer_point_color = true;
        }
    }
    if let Some(en) = extract_attr_bool(&main_xml, "crs:EnableColorAdjustments") {
        params.enable_color_adjustments = en;
    }

    // 9. Color Grading (Shadows, Midtones, Highlights, Global)
    for (attr, setter) in [
        ("crs:SplitToningShadowHue", 0usize),
        ("crs:SplitToningShadowSaturation", 1),
        ("crs:ColorGradeShadowLum", 2),
        ("crs:ColorGradeMidtoneHue", 3),
        ("crs:ColorGradeMidtoneSat", 4),
        ("crs:ColorGradeMidtoneLum", 5),
        ("crs:SplitToningHighlightHue", 6),
        ("crs:SplitToningHighlightSaturation", 7),
        ("crs:ColorGradeHighlightLum", 8),
        ("crs:ColorGradeGlobalHue", 9),
        ("crs:ColorGradeGlobalSat", 10),
        ("crs:ColorGradeGlobalLum", 11),
        ("crs:ColorGradeBlending", 12),
        ("crs:SplitToningBalance", 13),
    ] {
        if let Some(v) = extract_attr_f32(&main_xml, attr) {
            mask.color_grading = true;
            match setter {
                0 => params.split_toning_shadow_hue = v,
                1 => params.split_toning_shadow_saturation = v,
                2 => params.color_grade_shadow_lum = v,
                3 => params.color_grade_midtone_hue = v,
                4 => params.color_grade_midtone_sat = v,
                5 => params.color_grade_midtone_lum = v,
                6 => params.split_toning_highlight_hue = v,
                7 => params.split_toning_highlight_saturation = v,
                8 => params.color_grade_highlight_lum = v,
                9 => params.color_grade_global_hue = v,
                10 => params.color_grade_global_sat = v,
                11 => params.color_grade_global_lum = v,
                12 => params.color_grade_blending = v,
                _ => params.split_toning_balance = v,
            }
        }
    }
    if let Some(en) = extract_attr_bool(&main_xml, "crs:EnableSplitToning") {
        params.enable_split_toning = en;
    }

    // 10. Lens Corrections & Defringe
    if let Some(v) = extract_attr_bool(&main_xml, "crs:AutoLateralCA") {
        params.auto_lateral_ca = v;
        mask.lens_remove_ca = true;
    }
    if let Some(v) = extract_attr_bool(&main_xml, "crs:LensProfileEnable") {
        params.lens_profile_enable = v;
        mask.lens_profile = true;
    }
    if let Some(v) = extract_attr_str(&main_xml, "crs:LensProfileName") {
        params.lens_profile_name = v;
        mask.lens_profile = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:LensProfileDistortionScale") {
        params.lens_profile_distortion_scale = v;
        mask.lens_profile = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:LensProfileVignettingScale") {
        params.lens_profile_vignetting_scale = v;
        mask.lens_profile = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:LensManualDistortionAmount") {
        params.lens_manual_distortion_amount = v;
        mask.lens_manual_distortion = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:VignetteAmount") {
        params.vignette_amount = v;
        mask.lens_manual_vignetting = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:VignetteMidpoint") {
        params.vignette_midpoint = v;
        mask.lens_manual_vignetting = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:DefringePurpleAmount") {
        params.defringe_purple_amount = v;
        mask.lens_remove_ca = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:DefringePurpleHueLo") {
        params.defringe_purple_hue_lo = v;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:DefringePurpleHueHi") {
        params.defringe_purple_hue_hi = v;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:DefringeGreenAmount") {
        params.defringe_green_amount = v;
        mask.lens_remove_ca = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:DefringeGreenHueLo") {
        params.defringe_green_hue_lo = v;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:DefringeGreenHueHi") {
        params.defringe_green_hue_hi = v;
    }
    if let Some(en) = extract_attr_bool(&main_xml, "crs:EnableLensCorrections") {
        params.enable_lens_corrections = en;
    }

    // 11. Transform / Upright
    if let Some(v) = extract_attr_f32(&main_xml, "crs:PerspectiveUpright") {
        mask.transform_upright_mode = true;
        params.perspective_upright = match v.round() as i32 {
            1 => UprightMode::Auto,
            2 => UprightMode::Full,
            3 => UprightMode::Level,
            4 => UprightMode::Vertical,
            5 => UprightMode::Guided,
            _ => UprightMode::Off,
        };
    }
    for (attr, idx) in [
        ("crs:PerspectiveVertical", 0usize),
        ("crs:PerspectiveHorizontal", 1),
        ("crs:PerspectiveRotate", 2),
        ("crs:PerspectiveAspect", 3),
        ("crs:PerspectiveScale", 4),
        ("crs:PerspectiveX", 5),
        ("crs:PerspectiveY", 6),
    ] {
        if let Some(v) = extract_attr_f32(&main_xml, attr) {
            mask.transform_manual = true;
            match idx {
                0 => params.perspective_vertical = v,
                1 => params.perspective_horizontal = v,
                2 => params.perspective_rotate = v,
                3 => params.perspective_aspect = v,
                4 => params.perspective_scale = v,
                5 => params.perspective_x = v,
                _ => params.perspective_y = v,
            }
        }
    }
    if let Some(v) = extract_attr_bool(&main_xml, "crs:CropConstrainToWarp") {
        params.crop_constrain_to_warp = v;
        mask.transform_upright_transforms = true;
    }
    if let Some(en) = extract_attr_bool(&main_xml, "crs:EnableTransform") {
        params.enable_transform = en;
    }

    // 12. `<crs:LensBlur .../>`
    if let Some(lb_tag) = extract_opening_tag(&main_xml, "crs:LensBlur") {
        if let Some(act) = extract_attr_bool(&lb_tag, "crs:Active") {
            params.lens_blur_active = act;
            if act {
                mask.lens_blur_group = true;
                mask.lens_blur_focus_subject = true;
            }
        }
        if let Some(v) = extract_attr_f32(&lb_tag, "crs:BlurAmount") {
            params.lens_blur_amount = v;
        }
        if let Some(v) = extract_attr_f32(&lb_tag, "crs:BokehShape") {
            params.lens_blur_bokeh = match v.round() as i32 {
                1 => BokehShape::SoapBubble,
                2 => BokehShape::Blade,
                3 => BokehShape::Ring,
                4 => BokehShape::CatEye,
                5 => BokehShape::Anamorphic,
                _ => BokehShape::Circle,
            };
        }
        if let Some(v) = extract_attr_f32(&lb_tag, "crs:HighlightsBoost") {
            params.lens_blur_highlights_boost = v;
        }
        if let Some(v) = extract_attr_f32(&lb_tag, "crs:CatEyeAmount") {
            params.lens_blur_cat_eye_amount = v;
        }
        if let Some(fr) = extract_attr_str(&lb_tag, "crs:FocalRange") {
            let nums: Vec<f32> = fr
                .split_whitespace()
                .filter_map(|s| s.parse::<f32>().ok())
                .collect();
            if nums.len() == 4 && !(nums[0] == 0.0 && nums[1] == 0.0 && nums[2] == 100.0 && nums[3] == 100.0) {
                params.lens_blur_focal_range = [nums[0], nums[1], nums[2], nums[3]];
            }
        }
    }

    // 13. Effects (Post-Crop Vignetting & Grain)
    if let Some(v) = extract_attr_f32(&main_xml, "crs:GrainAmount") {
        params.grain_amount = v;
        mask.effects_grain = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:GrainSize") {
        params.grain_size = v;
        mask.effects_grain = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:GrainFrequency") {
        params.grain_frequency = v;
        mask.effects_grain = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:PostCropVignetteAmount") {
        params.post_crop_vignette_amount = v;
        mask.effects_post_crop_vignette = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:PostCropVignetteMidpoint") {
        params.post_crop_vignette_midpoint = v;
        mask.effects_post_crop_vignette = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:PostCropVignetteRoundness") {
        params.post_crop_vignette_roundness = v;
        mask.effects_post_crop_vignette = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:PostCropVignetteFeather") {
        params.post_crop_vignette_feather = v;
        mask.effects_post_crop_vignette = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:PostCropVignetteHighlightContrast") {
        params.post_crop_vignette_highlight_contrast = v;
        mask.effects_post_crop_vignette = true;
    }
    if let Some(v) = extract_attr_f32(&main_xml, "crs:PostCropVignetteStyle") {
        params.post_crop_vignette_style = match v.round() as i32 {
            2 => VignetteStyle::ColorPriority,
            3 => VignetteStyle::PaintOverlay,
            _ => VignetteStyle::HighlightPriority,
        };
    }
    if let Some(ov) = extract_attr_bool(&main_xml, "crs:OverrideLookVignette") {
        params.override_look_vignette = ov;
    }
    if let Some(en) = extract_attr_bool(&main_xml, "crs:EnableEffects") {
        params.enable_effects = en;
    }

    // 14. Calibration
    for (attr, idx) in [
        ("crs:ShadowTint", 0usize),
        ("crs:RedHue", 1),
        ("crs:RedSaturation", 2),
        ("crs:GreenHue", 3),
        ("crs:GreenSaturation", 4),
        ("crs:BlueHue", 5),
        ("crs:BlueSaturation", 6),
    ] {
        if let Some(v) = extract_attr_f32(&main_xml, attr) {
            mask.calibration = true;
            match idx {
                0 => params.shadow_tint = v,
                1 => params.red_hue = v,
                2 => params.red_saturation = v,
                3 => params.green_hue = v,
                4 => params.green_saturation = v,
                5 => params.blue_hue = v,
                _ => params.blue_saturation = v,
            }
        }
    }
    if let Some(en) = extract_attr_bool(&main_xml, "crs:EnableCalibration") {
        params.enable_calibration = en;
    }

    // 15. HDR & Crop (only apply Crop if explicitly `crs:HasCrop="True"`)
    if let Some(hdr) = extract_attr_bool(&main_xml, "crs:HDREditMode") {
        params.hdr_edit_mode = hdr;
        mask.hdr_mode = true;
    }
    if extract_attr_bool(&main_xml, "crs:HasCrop") == Some(true) {
        if let Some(v) = extract_attr_f32(&main_xml, "crs:CropTop") {
            params.crop_top = v;
            mask.crop_aspect_ratio = true;
        }
        if let Some(v) = extract_attr_f32(&main_xml, "crs:CropLeft") {
            params.crop_left = v;
            mask.crop_aspect_ratio = true;
        }
        if let Some(v) = extract_attr_f32(&main_xml, "crs:CropBottom") {
            params.crop_bottom = v;
            mask.crop_aspect_ratio = true;
        }
        if let Some(v) = extract_attr_f32(&main_xml, "crs:CropRight") {
            params.crop_right = v;
            mask.crop_aspect_ratio = true;
        }
        if let Some(v) = extract_attr_f32(&main_xml, "crs:CropAngle") {
            params.crop_angle = v;
            mask.crop_straighten_angle = true;
        }
    }

    // 16. Optional LightRaw serialized MaskGroups block for lossless mask preset round-trip
    if let Some(mask_json) = extract_attr_str(&main_xml, "crs:LightRawMasksJson") {
        if let Ok(groups) = serde_json::from_str::<Vec<MaskGroup>>(&mask_json) {
            if !groups.is_empty() {
                params.mask_groups = groups;
                mask.masking = true;
            }
        }
    }

    mask.sync_groups_from_children();

    let slug: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let id = format!("xmp_{}", slug.trim_matches('_'));

    Some(DevelopPreset {
        id,
        category: group,
        name,
        is_user: true,
        is_favorite: false,
        params,
        mask: Some(mask),
        is_visible: true,
        xmp_file_path: None,
    })
}

fn format_curve_seq(tag: &str, pts: &[(f32, f32)]) -> String {
    let mut lines = String::new();
    if pts.is_empty() {
        lines.push_str("     <rdf:li>0, 0</rdf:li>\n     <rdf:li>255, 255</rdf:li>\n");
    } else {
        for &(x, y) in pts {
            lines.push_str(&format!(
                "     <rdf:li>{:.0}, {:.0}</rdf:li>\n",
                (x * 255.0).round().clamp(0.0, 255.0),
                (y * 255.0).round().clamp(0.0, 255.0)
            ));
        }
    }
    format!(
        "   <{tag}>\n    <rdf:Seq>\n{lines}    </rdf:Seq>\n   </{tag}>\n",
        tag = tag,
        lines = lines
    )
}

pub fn export_preset_to_xmp(preset: &DevelopPreset) -> String {
    let p = &preset.params;
    let m = preset.mask.clone().unwrap_or_else(CopySettingsMask::default);

    let esc_name = xml_escape(&preset.name);
    let esc_group = xml_escape(&preset.category);
    let uuid = format!(
        "{:032X}",
        preset
            .name
            .as_bytes()
            .iter()
            .fold(0xA4537336u128, |acc, &b| acc.wrapping_mul(131).wrapping_add(b as u128))
    );

    let wb_str = match p.white_balance {
        WhiteBalanceMode::AsShot => "As Shot",
        WhiteBalanceMode::Auto => "Auto",
        WhiteBalanceMode::Daylight => "Daylight",
        WhiteBalanceMode::Cloudy => "Cloudy",
        WhiteBalanceMode::Shade => "Shade",
        WhiteBalanceMode::Tungsten => "Tungsten",
        WhiteBalanceMode::Fluorescent => "Fluorescent",
        WhiteBalanceMode::Flash => "Flash",
        WhiteBalanceMode::Custom => "Custom",
    };

    let mut attrs = String::new();
    if m.treatment_and_profile {
        attrs.push_str(&format!(
            "   crs:ConvertToGrayscale=\"{}\"\n",
            if p.convert_to_grayscale { "True" } else { "False" }
        ));
    }
    if m.basic_white_balance {
        attrs.push_str(&format!("   crs:WhiteBalance=\"{}\"\n", wb_str));
        if p.white_balance != WhiteBalanceMode::AsShot {
            attrs.push_str(&format!("   crs:Temperature=\"{:.0}\"\n", p.temperature));
            attrs.push_str(&format!("   crs:Tint=\"{:+.0}\"\n", p.tint));
        }
    }
    if m.basic_exposure {
        attrs.push_str(&format!("   crs:Exposure2012=\"{:+.2}\"\n", p.exposure_2012));
    }
    if m.basic_contrast {
        attrs.push_str(&format!("   crs:Contrast2012=\"{:+.0}\"\n", p.contrast_2012));
    }
    if m.basic_highlights {
        attrs.push_str(&format!("   crs:Highlights2012=\"{:+.0}\"\n", p.highlights_2012));
    }
    if m.basic_shadows {
        attrs.push_str(&format!("   crs:Shadows2012=\"{:+.0}\"\n", p.shadows_2012));
    }
    if m.basic_whites {
        attrs.push_str(&format!("   crs:Whites2012=\"{:+.0}\"\n", p.whites_2012));
    }
    if m.basic_blacks {
        attrs.push_str(&format!("   crs:Blacks2012=\"{:+.0}\"\n", p.blacks_2012));
    }
    if m.basic_texture {
        attrs.push_str(&format!("   crs:Texture=\"{:+.0}\"\n", p.texture));
    }
    if m.basic_clarity {
        attrs.push_str(&format!("   crs:Clarity2012=\"{:+.0}\"\n", p.clarity_2012));
    }
    if m.basic_dehaze {
        attrs.push_str(&format!("   crs:Dehaze=\"{:+.0}\"\n", p.dehaze));
    }
    if m.basic_vibrance {
        attrs.push_str(&format!("   crs:Vibrance=\"{:+.0}\"\n", p.vibrance));
    }
    if m.basic_saturation {
        attrs.push_str(&format!("   crs:Saturation=\"{:+.0}\"\n", p.saturation));
    }

    if m.curve_parametric {
        attrs.push_str(&format!("   crs:ParametricShadows=\"{:+.0}\"\n", p.parametric_shadows));
        attrs.push_str(&format!("   crs:ParametricDarks=\"{:+.0}\"\n", p.parametric_darks));
        attrs.push_str(&format!("   crs:ParametricLights=\"{:+.0}\"\n", p.parametric_lights));
        attrs.push_str(&format!("   crs:ParametricHighlights=\"{:+.0}\"\n", p.parametric_highlights));
        attrs.push_str(&format!("   crs:ParametricShadowSplit=\"{:.0}\"\n", p.parametric_shadow_split));
        attrs.push_str(&format!("   crs:ParametricMidtoneSplit=\"{:.0}\"\n", p.parametric_midtone_split));
        attrs.push_str(&format!("   crs:ParametricHighlightSplit=\"{:.0}\"\n", p.parametric_highlight_split));
    }
    if m.curve_point {
        attrs.push_str(&format!("   crs:ToneCurveName2012=\"{}\"\n", xml_escape(&p.tone_curve_name_2012)));
        attrs.push_str(&format!("   crs:CurveRefineSaturation=\"{:.0}\"\n", p.curve_refine_saturation));
    }

    if m.detail_sharpening {
        attrs.push_str(&format!("   crs:Sharpness=\"{:.0}\"\n", p.sharpness));
        attrs.push_str(&format!("   crs:SharpenRadius=\"{:+.1}\"\n", p.sharpen_radius));
        attrs.push_str(&format!("   crs:SharpenDetail=\"{:.0}\"\n", p.sharpen_detail));
        attrs.push_str(&format!("   crs:SharpenEdgeMasking=\"{:.0}\"\n", p.sharpen_edge_masking));
    }
    if m.detail_luminance_nr {
        attrs.push_str(&format!("   crs:LuminanceSmoothing=\"{:.0}\"\n", p.luminance_smoothing));
        attrs.push_str(&format!(
            "   crs:LuminanceNoiseReductionDetail=\"{:.0}\"\n",
            p.luminance_noise_reduction_detail
        ));
        attrs.push_str(&format!(
            "   crs:LuminanceNoiseReductionContrast=\"{:.0}\"\n",
            p.luminance_noise_reduction_contrast
        ));
    }
    if m.detail_color_nr {
        attrs.push_str(&format!("   crs:ColorNoiseReduction=\"{:.0}\"\n", p.color_noise_reduction));
        attrs.push_str(&format!(
            "   crs:ColorNoiseReductionDetail=\"{:.0}\"\n",
            p.color_noise_reduction_detail
        ));
        attrs.push_str(&format!(
            "   crs:ColorNoiseReductionSmoothness=\"{:.0}\"\n",
            p.color_noise_reduction_smoothness
        ));
    }

    if m.color_mixer_mixer {
        let names = DevelopParams::HSL_BAND_NAMES;
        for i in 0..8 {
            attrs.push_str(&format!("   crs:HueAdjustment{}=\"{:+.0}\"\n", names[i], p.hue_adjustment[i]));
        }
        for i in 0..8 {
            attrs.push_str(&format!(
                "   crs:SaturationAdjustment{}=\"{:+.0}\"\n",
                names[i], p.saturation_adjustment[i]
            ));
        }
        for i in 0..8 {
            attrs.push_str(&format!(
                "   crs:LuminanceAdjustment{}=\"{:+.0}\"\n",
                names[i], p.luminance_adjustment[i]
            ));
        }
        if p.convert_to_grayscale {
            for i in 0..8 {
                attrs.push_str(&format!("   crs:GrayMixer{}=\"{:+.0}\"\n", names[i], p.gray_mixer[i]));
            }
        }
    }

    if m.color_grading {
        attrs.push_str(&format!("   crs:SplitToningShadowHue=\"{:.0}\"\n", p.split_toning_shadow_hue));
        attrs.push_str(&format!(
            "   crs:SplitToningShadowSaturation=\"{:.0}\"\n",
            p.split_toning_shadow_saturation
        ));
        attrs.push_str(&format!("   crs:ColorGradeShadowLum=\"{:+.0}\"\n", p.color_grade_shadow_lum));
        attrs.push_str(&format!("   crs:ColorGradeMidtoneHue=\"{:.0}\"\n", p.color_grade_midtone_hue));
        attrs.push_str(&format!("   crs:ColorGradeMidtoneSat=\"{:.0}\"\n", p.color_grade_midtone_sat));
        attrs.push_str(&format!("   crs:ColorGradeMidtoneLum=\"{:+.0}\"\n", p.color_grade_midtone_lum));
        attrs.push_str(&format!("   crs:SplitToningHighlightHue=\"{:.0}\"\n", p.split_toning_highlight_hue));
        attrs.push_str(&format!(
            "   crs:SplitToningHighlightSaturation=\"{:.0}\"\n",
            p.split_toning_highlight_saturation
        ));
        attrs.push_str(&format!("   crs:ColorGradeHighlightLum=\"{:+.0}\"\n", p.color_grade_highlight_lum));
        attrs.push_str(&format!("   crs:ColorGradeGlobalHue=\"{:.0}\"\n", p.color_grade_global_hue));
        attrs.push_str(&format!("   crs:ColorGradeGlobalSat=\"{:.0}\"\n", p.color_grade_global_sat));
        attrs.push_str(&format!("   crs:ColorGradeGlobalLum=\"{:+.0}\"\n", p.color_grade_global_lum));
        attrs.push_str(&format!("   crs:ColorGradeBlending=\"{:.0}\"\n", p.color_grade_blending));
        attrs.push_str(&format!("   crs:SplitToningBalance=\"{:+.0}\"\n", p.split_toning_balance));
    }

    if m.lens_remove_ca {
        attrs.push_str(&format!("   crs:AutoLateralCA=\"{}\"\n", if p.auto_lateral_ca { 1 } else { 0 }));
        attrs.push_str(&format!("   crs:DefringePurpleAmount=\"{:.0}\"\n", p.defringe_purple_amount));
        attrs.push_str(&format!("   crs:DefringePurpleHueLo=\"{:.0}\"\n", p.defringe_purple_hue_lo));
        attrs.push_str(&format!("   crs:DefringePurpleHueHi=\"{:.0}\"\n", p.defringe_purple_hue_hi));
        attrs.push_str(&format!("   crs:DefringeGreenAmount=\"{:.0}\"\n", p.defringe_green_amount));
        attrs.push_str(&format!("   crs:DefringeGreenHueLo=\"{:.0}\"\n", p.defringe_green_hue_lo));
        attrs.push_str(&format!("   crs:DefringeGreenHueHi=\"{:.0}\"\n", p.defringe_green_hue_hi));
    }
    if m.lens_profile {
        attrs.push_str(&format!("   crs:LensProfileEnable=\"{}\"\n", if p.lens_profile_enable { 1 } else { 0 }));
        attrs.push_str(&format!(
            "   crs:LensProfileDistortionScale=\"{:.0}\"\n",
            p.lens_profile_distortion_scale
        ));
        attrs.push_str(&format!(
            "   crs:LensProfileVignettingScale=\"{:.0}\"\n",
            p.lens_profile_vignetting_scale
        ));
    }
    if m.lens_manual_distortion {
        attrs.push_str(&format!(
            "   crs:LensManualDistortionAmount=\"{:+.0}\"\n",
            p.lens_manual_distortion_amount
        ));
    }
    if m.lens_manual_vignetting {
        attrs.push_str(&format!("   crs:VignetteAmount=\"{:+.0}\"\n", p.vignette_amount));
        attrs.push_str(&format!("   crs:VignetteMidpoint=\"{:.0}\"\n", p.vignette_midpoint));
    }

    if m.transform_upright_mode {
        let up_id = match p.perspective_upright {
            UprightMode::Off => 0,
            UprightMode::Auto => 1,
            UprightMode::Full => 2,
            UprightMode::Level => 3,
            UprightMode::Vertical => 4,
            UprightMode::Guided => 5,
        };
        attrs.push_str(&format!("   crs:PerspectiveUpright=\"{}\"\n", up_id));
    }
    if m.transform_manual {
        attrs.push_str(&format!("   crs:PerspectiveVertical=\"{:+.0}\"\n", p.perspective_vertical));
        attrs.push_str(&format!("   crs:PerspectiveHorizontal=\"{:+.0}\"\n", p.perspective_horizontal));
        attrs.push_str(&format!("   crs:PerspectiveRotate=\"{:+.1}\"\n", p.perspective_rotate));
        attrs.push_str(&format!("   crs:PerspectiveAspect=\"{:+.0}\"\n", p.perspective_aspect));
        attrs.push_str(&format!("   crs:PerspectiveScale=\"{:.0}\"\n", p.perspective_scale));
        attrs.push_str(&format!("   crs:PerspectiveX=\"{:+.2}\"\n", p.perspective_x));
        attrs.push_str(&format!("   crs:PerspectiveY=\"{:+.2}\"\n", p.perspective_y));
    }

    if m.effects_grain {
        attrs.push_str(&format!("   crs:GrainAmount=\"{:.0}\"\n", p.grain_amount));
        attrs.push_str(&format!("   crs:GrainSize=\"{:.0}\"\n", p.grain_size));
        attrs.push_str(&format!("   crs:GrainFrequency=\"{:.0}\"\n", p.grain_frequency));
    }
    if m.effects_post_crop_vignette {
        attrs.push_str(&format!("   crs:PostCropVignetteAmount=\"{:+.0}\"\n", p.post_crop_vignette_amount));
        attrs.push_str(&format!(
            "   crs:PostCropVignetteMidpoint=\"{:.0}\"\n",
            p.post_crop_vignette_midpoint
        ));
        attrs.push_str(&format!(
            "   crs:PostCropVignetteRoundness=\"{:+.0}\"\n",
            p.post_crop_vignette_roundness
        ));
        attrs.push_str(&format!(
            "   crs:PostCropVignetteFeather=\"{:.0}\"\n",
            p.post_crop_vignette_feather
        ));
        attrs.push_str(&format!(
            "   crs:PostCropVignetteHighlightContrast=\"{:.0}\"\n",
            p.post_crop_vignette_highlight_contrast
        ));
    }

    if m.calibration {
        attrs.push_str(&format!("   crs:ShadowTint=\"{:+.0}\"\n", p.shadow_tint));
        attrs.push_str(&format!("   crs:RedHue=\"{:+.0}\"\n", p.red_hue));
        attrs.push_str(&format!("   crs:RedSaturation=\"{:+.0}\"\n", p.red_saturation));
        attrs.push_str(&format!("   crs:GreenHue=\"{:+.0}\"\n", p.green_hue));
        attrs.push_str(&format!("   crs:GreenSaturation=\"{:+.0}\"\n", p.green_saturation));
        attrs.push_str(&format!("   crs:BlueHue=\"{:+.0}\"\n", p.blue_hue));
        attrs.push_str(&format!("   crs:BlueSaturation=\"{:+.0}\"\n", p.blue_saturation));
    }

    if m.hdr_mode {
        attrs.push_str(&format!("   crs:HDREditMode=\"{}\"\n", if p.hdr_edit_mode { 1 } else { 0 }));
    }

    if m.crop_aspect_ratio || m.crop_straighten_angle {
        attrs.push_str("   crs:HasCrop=\"True\"\n");
        attrs.push_str(&format!("   crs:CropTop=\"{:.5}\"\n", p.crop_top));
        attrs.push_str(&format!("   crs:CropLeft=\"{:.5}\"\n", p.crop_left));
        attrs.push_str(&format!("   crs:CropBottom=\"{:.5}\"\n", p.crop_bottom));
        attrs.push_str(&format!("   crs:CropRight=\"{:.5}\"\n", p.crop_right));
        attrs.push_str(&format!("   crs:CropAngle=\"{:+.2}\"\n", p.crop_angle));
    }

    if m.masking && !p.mask_groups.is_empty() {
        if let Ok(mj) = serde_json::to_string(&p.mask_groups) {
            attrs.push_str(&format!("   crs:LightRawMasksJson=\"{}\"\n", xml_escape(&mj)));
        }
    }

    let mut body = String::new();
    if m.curve_point {
        body.push_str(&format_curve_seq("crs:ToneCurvePV2012", &p.tone_curve_pv2012));
        body.push_str(&format_curve_seq("crs:ToneCurvePV2012Red", &p.tone_curve_pv2012_red));
        body.push_str(&format_curve_seq("crs:ToneCurvePV2012Green", &p.tone_curve_pv2012_green));
        body.push_str(&format_curve_seq("crs:ToneCurvePV2012Blue", &p.tone_curve_pv2012_blue));
    }

    if m.color_mixer_point_color && !p.point_colors.is_empty() {
        body.push_str("   <crs:PointColors>\n    <rdf:Seq>\n");
        for sw in &p.point_colors {
            let rad = sw.src_hue.to_radians();
            body.push_str(&format!(
                "     <rdf:li>{:.6}, {:.6}, {:.6}, {:.6}, {:.6}, {:.6}, {:.6}, 0.000000, 0.330000, 0.670000, 1.000000, 0.000000, 0.000000, 0.350000, 0.900000, 0.240000, 0.790000, 1.000000, 1.000000</rdf:li>\n",
                rad,
                sw.src_sat,
                sw.src_lum,
                sw.hue_shift / 100.0,
                sw.sat_shift / 100.0,
                sw.lum_shift / 100.0,
                sw.range_amount / 100.0
            ));
        }
        body.push_str("    </rdf:Seq>\n   </crs:PointColors>\n");
    }

    if m.lens_blur_group || p.lens_blur_active {
        let bokeh_id = match p.lens_blur_bokeh {
            BokehShape::Circle => 0,
            BokehShape::SoapBubble => 1,
            BokehShape::Blade => 2,
            BokehShape::Ring => 3,
            BokehShape::CatEye => 4,
            BokehShape::Anamorphic => 5,
        };
        body.push_str(&format!(
            "   <crs:LensBlur\n    crs:Version=\"1\"\n    crs:Active=\"{}\"\n    crs:FocalRange=\"{:.0} {:.0} {:.0} {:.0}\"\n    crs:BlurAmount=\"{:.0}\"\n    crs:BokehShape=\"{}\"\n    crs:HighlightsBoost=\"{:.0}\"\n    crs:CatEyeAmount=\"{:.0}\"/>\n",
            if p.lens_blur_active { "true" } else { "false" },
            p.lens_blur_focal_range[0],
            p.lens_blur_focal_range[1],
            p.lens_blur_focal_range[2],
            p.lens_blur_focal_range[3],
            p.lens_blur_amount,
            bokeh_id,
            p.lens_blur_highlights_boost,
            p.lens_blur_cat_eye_amount
        ));
    }

    if m.treatment_and_profile {
        body.push_str(&format!(
            "   <crs:Look>\n    <rdf:Description\n     crs:Name=\"{}\"\n     crs:Amount=\"{:.2}\"\n     crs:SupportsAmount=\"true\">\n    <crs:Group>\n     <rdf:Alt>\n      <rdf:li xml:lang=\"x-default\">Profiles</rdf:li>\n     </rdf:Alt>\n    </crs:Group>\n    </rdf:Description>\n   </crs:Look>\n",
            xml_escape(p.camera_profile.label()),
            p.look_amount / 100.0
        ));
    }

    format!(
        "<x:xmpmeta xmlns:x=\"adobe:ns:meta/\" x:xmptk=\"Adobe XMP Core 7.0-c000 1.000000, 0000/00/00-00:00:00        \">\n <rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">\n  <rdf:Description rdf:about=\"\"\n    xmlns:crs=\"http://ns.adobe.com/camera-raw-settings/1.0/\"\n   crs:PresetType=\"Normal\"\n   crs:Cluster=\"\"\n   crs:Name=\"{name}\"\n   crs:Group=\"{group}\"\n   crs:UUID=\"{uuid}\"\n   crs:SupportsAmount2=\"True\"\n   crs:SupportsAmount=\"True\"\n   crs:SupportsColor=\"True\"\n   crs:SupportsMonochrome=\"True\"\n   crs:Version=\"18.4\"\n   crs:ProcessVersion=\"15.4\"\n{attrs}   crs:HasSettings=\"True\">\n   <crs:Name>\n    <rdf:Alt>\n     <rdf:li xml:lang=\"x-default\">{name}</rdf:li>\n    </rdf:Alt>\n   </crs:Name>\n   <crs:Group>\n    <rdf:Alt>\n     <rdf:li xml:lang=\"x-default\">{group}</rdf:li>\n    </rdf:Alt>\n   </crs:Group>\n{body}  </rdf:Description>\n </rdf:RDF>\n</x:xmpmeta>\n",
        name = esc_name,
        group = esc_group,
        uuid = uuid,
        attrs = attrs,
        body = body
    )
}

#[derive(Debug, Clone)]
pub struct LocalizedMaskPreset {
    pub name: &'static str,
    pub local: LocalAdjustParams,
}

pub type LocalMaskPreset = LocalizedMaskPreset;

pub fn load_presets_from_folder(folder: &std::path::Path) -> Vec<DevelopPreset> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(folder) else {
        return out;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            if let Ok(sub_entries) = std::fs::read_dir(&p) {
                let cat_fallback = p
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("User Presets")
                    .to_string();
                for sub in sub_entries.flatten() {
                    let sp = sub.path();
                    if sp.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("xmp")).unwrap_or(false) {
                        if let Ok(xml) = std::fs::read_to_string(&sp) {
                            let stem = sp.file_stem().and_then(|s| s.to_str()).unwrap_or("Preset");
                            if let Some(mut pr) = parse_xmp_preset(&xml, stem) {
                                if pr.category == "User Presets" && !cat_fallback.is_empty() {
                                    pr.category = cat_fallback.clone();
                                }
                                pr.xmp_file_path = Some(sp.display().to_string());
                                out.push(pr);
                            }
                        }
                    }
                }
            }
        } else if p.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("xmp")).unwrap_or(false) {
            if let Ok(xml) = std::fs::read_to_string(&p) {
                let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("Preset");
                if let Some(mut pr) = parse_xmp_preset(&xml, stem) {
                    pr.xmp_file_path = Some(p.display().to_string());
                    out.push(pr);
                }
            }
        }
    }
    out
}

pub fn builtin_localized_mask_presets() -> Vec<LocalizedMaskPreset> {
    vec![
        LocalizedMaskPreset {
            name: "Custom",
            local: LocalAdjustParams::default(),
        },
        LocalizedMaskPreset {
            name: "Dodge (Lighten)",
            local: LocalAdjustParams {
                exposure: 0.45,
                shadows: 18.0,
                ..Default::default()
            },
        },
        LocalizedMaskPreset {
            name: "Burn (Darken)",
            local: LocalAdjustParams {
                exposure: -0.45,
                highlights: -15.0,
                ..Default::default()
            },
        },
        LocalizedMaskPreset {
            name: "Iris Enhance",
            local: LocalAdjustParams {
                exposure: 0.35,
                contrast: 15.0,
                clarity: 28.0,
                saturation: 22.0,
                sharpness: 20.0,
                ..Default::default()
            },
        },
        LocalizedMaskPreset {
            name: "Soften Skin",
            local: LocalAdjustParams {
                clarity: -45.0,
                texture: -35.0,
                sharpness: -25.0,
                ..Default::default()
            },
        },
        LocalizedMaskPreset {
            name: "Soften Skin (Lite)",
            local: LocalAdjustParams {
                clarity: -25.0,
                texture: -20.0,
                sharpness: -12.0,
                ..Default::default()
            },
        },
        LocalizedMaskPreset {
            name: "Teeth Whitening",
            local: LocalAdjustParams {
                exposure: 0.35,
                saturation: -55.0,
                temperature: -12.0,
                ..Default::default()
            },
        },
    ]
}

fn make_builtin(id: &str, category: &str, name: &str, params: DevelopParams, mask: Option<CopySettingsMask>) -> DevelopPreset {
    DevelopPreset {
        id: id.into(),
        category: category.into(),
        name: name.into(),
        is_user: false,
        is_favorite: false,
        params,
        mask,
        is_visible: true,
        xmp_file_path: None,
    }
}

pub fn builtin_develop_presets() -> Vec<DevelopPreset> {
    let mut list = Vec::new();

    // 1. Adaptive AI Presets (matching Lightroom Classic Left Presets Tree!)
    {
        let mut mask_only = CopySettingsMask::none();
        mask_only.masking = true;

        let mut p = DevelopParams::default();
        p.mask_groups.push(MaskGroup {
            id: 101,
            name: "Flattering Portrait".into(),
            enabled: true,
            invert: false,
            components: vec![MaskComponent {
                name: "Face Skin".into(),
                op: MaskOp::Add,
                invert: false,
                kind: MaskPrimitiveKind::PersonFeature {
                    person_index: 0,
                    part: PersonPart::FaceSkin,
                },
            }],
            local: LocalAdjustParams {
                exposure: 0.18,
                shadows: 16.0,
                texture: -28.0,
                clarity: -14.0,
                ..Default::default()
            },
        });
        list.push(make_builtin("adaptive_portrait_flattering", "Adaptive: Portrait", "Flattering Portrait", p, Some(mask_only.clone())));

        let mut p = DevelopParams::default();
        p.mask_groups.push(MaskGroup {
            id: 102,
            name: "Enhance Eyes & Teeth".into(),
            enabled: true,
            invert: false,
            components: vec![MaskComponent {
                name: "Teeth".into(),
                op: MaskOp::Add,
                invert: false,
                kind: MaskPrimitiveKind::PersonFeature {
                    person_index: 0,
                    part: PersonPart::Teeth,
                },
            }],
            local: LocalAdjustParams {
                exposure: 0.30,
                saturation: -45.0,
                clarity: 15.0,
                ..Default::default()
            },
        });
        list.push(make_builtin("adaptive_portrait_glamour", "Adaptive: Portrait", "Polished Portrait", p, Some(mask_only.clone())));

        let mut p = DevelopParams::default();
        p.mask_groups.push(MaskGroup {
            id: 103,
            name: "Blue Drama Sky".into(),
            enabled: true,
            invert: false,
            components: vec![MaskComponent {
                name: "Sky".into(),
                op: MaskOp::Add,
                invert: false,
                kind: MaskPrimitiveKind::AiSky,
            }],
            local: LocalAdjustParams {
                exposure: -0.55,
                highlights: -35.0,
                dehaze: 28.0,
                saturation: 22.0,
                temperature: -14.0,
                ..Default::default()
            },
        });
        list.push(make_builtin("adaptive_sky_blue_drama", "Adaptive: Sky", "Blue Drama", p, Some(mask_only.clone())));

        let mut p = DevelopParams::default();
        p.mask_groups.push(MaskGroup {
            id: 104,
            name: "Sunset Glow Sky".into(),
            enabled: true,
            invert: false,
            components: vec![MaskComponent {
                name: "Sky".into(),
                op: MaskOp::Add,
                invert: false,
                kind: MaskPrimitiveKind::AiSky,
            }],
            local: LocalAdjustParams {
                exposure: -0.20,
                temperature: 36.0,
                tint: 14.0,
                saturation: 28.0,
                ..Default::default()
            },
        });
        list.push(make_builtin("adaptive_sky_sunset", "Adaptive: Sky", "Sunset", p, Some(mask_only.clone())));

        let mut p = DevelopParams::default();
        p.mask_groups.push(MaskGroup {
            id: 105,
            name: "Pop Subject".into(),
            enabled: true,
            invert: false,
            components: vec![MaskComponent {
                name: "Subject".into(),
                op: MaskOp::Add,
                invert: false,
                kind: MaskPrimitiveKind::AiSubject,
            }],
            local: LocalAdjustParams {
                exposure: 0.32,
                contrast: 16.0,
                clarity: 18.0,
                texture: 12.0,
                saturation: 10.0,
                ..Default::default()
            },
        });
        list.push(make_builtin("adaptive_subject_pop", "Adaptive: Subject", "Pop", p, Some(mask_only.clone())));

        let mut p = DevelopParams::default();
        p.mask_groups.push(MaskGroup {
            id: 106,
            name: "Lush Foliage".into(),
            enabled: true,
            invert: false,
            components: vec![MaskComponent {
                name: "Vegetation".into(),
                op: MaskOp::Add,
                invert: false,
                kind: MaskPrimitiveKind::LandscapeFeature(LandscapePart::Vegetation),
            }],
            local: LocalAdjustParams {
                exposure: 0.15,
                clarity: 18.0,
                saturation: 22.0,
                hue: 6.0,
                ..Default::default()
            },
        });
        list.push(make_builtin("adaptive_landscape_lush", "Adaptive: Landscape", "Lush", p, Some(mask_only.clone())));

        let mut blur_mask = CopySettingsMask::none();
        blur_mask.lens_blur_group = true;
        blur_mask.lens_blur_focus_subject = true;
        let mut p = DevelopParams::default();
        p.lens_blur_active = true;
        p.lens_blur_amount = 55.0;
        p.lens_blur_bokeh = BokehShape::Circle;
        p.lens_blur_highlights_boost = 45.0;
        list.push(make_builtin("adaptive_blur_subtle", "Adaptive: Blur Background", "Subtle", p, Some(blur_mask.clone())));

        let mut p = DevelopParams::default();
        p.lens_blur_active = true;
        p.lens_blur_amount = 85.0;
        p.lens_blur_bokeh = BokehShape::CatEye;
        p.lens_blur_cat_eye_amount = 45.0;
        p.lens_blur_highlights_boost = 65.0;
        list.push(make_builtin("adaptive_blur_strong", "Adaptive: Blur Background", "Strong Bokeh", p, Some(blur_mask)));
    }

    // 2. Portraits: Deep Skin & Flattering Skin
    {
        let mut p = DevelopParams::default();
        p.camera_profile = CameraProfile::AdobePortrait;
        p.exposure_2012 = 0.18;
        p.highlights_2012 = -28.0;
        p.shadows_2012 = 26.0;
        p.texture = -14.0;
        p.clarity_2012 = -6.0;
        p.vibrance = 14.0;
        p.luminance_adjustment[0] = 10.0;
        p.luminance_adjustment[1] = 16.0;
        let mut pm = CopySettingsMask::modified(&p);
        pm.basic_white_balance = false;
        pm.sync_groups_from_children();
        list.push(make_builtin("portrait_deep_skin_1", "Portraits: Deep Skin", "Warm Rich Glow", p, Some(pm)));

        let mut p = DevelopParams::default();
        p.camera_profile = CameraProfile::AdobePortrait;
        p.exposure_2012 = 0.15;
        p.highlights_2012 = -25.0;
        p.shadows_2012 = 22.0;
        p.texture = -15.0;
        p.clarity_2012 = -8.0;
        p.vibrance = 14.0;
        p.luminance_adjustment[1] = 12.0;
        p.saturation_adjustment[1] = -5.0;
        let mut pm = CopySettingsMask::modified(&p);
        pm.basic_white_balance = false;
        pm.sync_groups_from_children();
        list.push(make_builtin("portrait_flattering", "Portrait", "Flattering Skin", p, Some(pm)));
    }

    // 3. Color
    {
        let mut p = DevelopParams::default();
        p.camera_profile = CameraProfile::AdobeColor;
        p.contrast_2012 = 15.0;
        p.vibrance = 22.0;
        p.saturation = 5.0;
        let mut m = CopySettingsMask::modified(&p);
        m.basic_white_balance = false;
        m.sync_groups_from_children();
        list.push(make_builtin("color_natural", "Color", "Natural", p, Some(m)));

        let mut p = DevelopParams::default();
        p.camera_profile = CameraProfile::AdobeVivid;
        p.exposure_2012 = 0.25;
        p.contrast_2012 = 18.0;
        p.highlights_2012 = -25.0;
        p.shadows_2012 = 28.0;
        p.vibrance = 28.0;
        let mut m = CopySettingsMask::modified(&p);
        m.basic_white_balance = false;
        m.sync_groups_from_children();
        list.push(make_builtin("color_bright", "Color", "Bright", p, Some(m)));

        let mut p = DevelopParams::default();
        p.contrast_2012 = 42.0;
        p.highlights_2012 = -30.0;
        p.shadows_2012 = -15.0;
        p.whites_2012 = 20.0;
        p.blacks_2012 = -18.0;
        p.clarity_2012 = 20.0;
        let mut m = CopySettingsMask::modified(&p);
        m.basic_white_balance = false;
        m.sync_groups_from_children();
        list.push(make_builtin("color_high_contrast", "Color", "High Contrast", p, Some(m)));

        let mut p = DevelopParams::default();
        p.camera_profile = CameraProfile::AdobeVivid;
        p.vibrance = 40.0;
        p.saturation = 15.0;
        p.clarity_2012 = 15.0;
        p.dehaze = 12.0;
        let mut m = CopySettingsMask::modified(&p);
        m.basic_white_balance = false;
        m.sync_groups_from_children();
        list.push(make_builtin("color_vivid", "Color", "Vivid", p, Some(m)));

        let mut p = DevelopParams::default();
        p.camera_profile = CameraProfile::AdobeNeutral;
        p.contrast_2012 = -15.0;
        p.highlights_2012 = -35.0;
        p.shadows_2012 = 35.0;
        p.tone_curve_pv2012 = vec![(0.0, 0.08), (0.25, 0.26), (0.75, 0.74), (1.0, 0.94)];
        let mut m = CopySettingsMask::modified(&p);
        m.basic_white_balance = false;
        m.sync_groups_from_children();
        list.push(make_builtin("color_matte", "Color", "Matte", p, Some(m)));
    }

    // 4. Creative
    {
        let mut p = DevelopParams::default();
        p.saturation = -35.0;
        p.contrast_2012 = 25.0;
        p.clarity_2012 = 22.0;
        p.split_toning_shadow_hue = 210.0;
        p.split_toning_shadow_saturation = 22.0;
        p.split_toning_highlight_hue = 42.0;
        p.split_toning_highlight_saturation = 18.0;
        let mut m = CopySettingsMask::modified(&p);
        m.basic_white_balance = false;
        m.sync_groups_from_children();
        list.push(make_builtin("creative_desat_contrast", "Creative", "Desaturated Contrast", p, Some(m)));

        let mut p = DevelopParams::default();
        p.split_toning_shadow_hue = 195.0;
        p.split_toning_shadow_saturation = 32.0;
        p.split_toning_highlight_hue = 32.0;
        p.split_toning_highlight_saturation = 28.0;
        p.hue_adjustment[1] = -12.0;
        p.hue_adjustment[4] = -20.0;
        p.hue_adjustment[5] = -25.0;
        p.contrast_2012 = 20.0;
        let mut m = CopySettingsMask::modified(&p);
        m.basic_white_balance = false;
        m.sync_groups_from_children();
        list.push(make_builtin("creative_turquoise_red", "Creative", "Turquoise & Red (Teal & Orange)", p, Some(m)));

        let mut p = DevelopParams::default();
        p.highlights_2012 = -20.0;
        p.shadows_2012 = 25.0;
        p.split_toning_shadow_hue = 28.0;
        p.split_toning_shadow_saturation = 18.0;
        p.split_toning_highlight_hue = 48.0;
        p.split_toning_highlight_saturation = 25.0;
        p.tone_curve_pv2012 = vec![(0.0, 0.06), (0.3, 0.28), (0.7, 0.74), (1.0, 0.96)];
        let mut m = CopySettingsMask::modified(&p);
        m.basic_white_balance = false;
        m.sync_groups_from_children();
        list.push(make_builtin("creative_warm_vintage", "Creative", "Warm Vintage", p, Some(m)));
    }

    // 5. B&W
    {
        let mut p = DevelopParams::default();
        p.convert_to_grayscale = true;
        p.camera_profile = CameraProfile::AdobeMonochrome;
        p.contrast_2012 = 38.0;
        p.highlights_2012 = -20.0;
        p.shadows_2012 = -12.0;
        p.whites_2012 = 22.0;
        p.blacks_2012 = -20.0;
        p.clarity_2012 = 25.0;
        let m = CopySettingsMask::modified(&p);
        list.push(make_builtin("bw_contrast_high", "B&W", "B&W High Contrast", p, Some(m)));

        let mut p = DevelopParams::default();
        p.convert_to_grayscale = true;
        p.camera_profile = CameraProfile::AdobeMonochrome;
        p.contrast_2012 = -18.0;
        p.highlights_2012 = -30.0;
        p.shadows_2012 = 35.0;
        let m = CopySettingsMask::modified(&p);
        list.push(make_builtin("bw_contrast_low", "B&W", "B&W Low Contrast", p, Some(m)));

        let mut p = DevelopParams::default();
        p.convert_to_grayscale = true;
        p.camera_profile = CameraProfile::AdobeMonochrome;
        p.gray_mixer = [25.0, 35.0, 20.0, -15.0, -30.0, -45.0, -20.0, 10.0];
        p.contrast_2012 = 24.0;
        p.clarity_2012 = 18.0;
        p.grain_amount = 22.0;
        p.post_crop_vignette_amount = -18.0;
        let m = CopySettingsMask::modified(&p);
        list.push(make_builtin("bw_red_filter", "B&W", "B&W Red Filter (Dramatic Sky)", p, Some(m)));

        let mut p = DevelopParams::default();
        p.convert_to_grayscale = true;
        p.camera_profile = CameraProfile::AdobeMonochrome;
        p.split_toning_shadow_hue = 38.0;
        p.split_toning_shadow_saturation = 16.0;
        p.split_toning_highlight_hue = 44.0;
        p.split_toning_highlight_saturation = 20.0;
        p.contrast_2012 = 15.0;
        let m = CopySettingsMask::modified(&p);
        list.push(make_builtin("bw_selenium_sepia", "B&W", "B&W Classic Sepia Tone", p, Some(m)));
    }

    // 6. Landscape, Grain & Vignetting (strictly partial masks!)
    {
        let mut p = DevelopParams::default();
        p.camera_profile = CameraProfile::AdobeLandscape;
        p.highlights_2012 = -45.0;
        p.shadows_2012 = 38.0;
        p.whites_2012 = 15.0;
        p.blacks_2012 = -12.0;
        p.texture = 18.0;
        p.clarity_2012 = 22.0;
        p.dehaze = 18.0;
        p.vibrance = 25.0;
        let m = CopySettingsMask::modified(&p);
        list.push(make_builtin("landscape_golden_hour", "Landscape", "Landscape Dynamic Pop", p, Some(m)));

        let mut p = DevelopParams::default();
        p.grain_amount = 35.0;
        p.grain_size = 30.0;
        p.grain_frequency = 55.0;
        let mut gm = CopySettingsMask::none();
        gm.effects_group = true;
        gm.effects_grain = true;
        list.push(make_builtin("grain_medium", "Grain", "Medium Film Grain", p, Some(gm)));

        let mut p = DevelopParams::default();
        p.post_crop_vignette_style = VignetteStyle::HighlightPriority;
        p.post_crop_vignette_amount = -32.0;
        p.post_crop_vignette_midpoint = 42.0;
        p.post_crop_vignette_feather = 65.0;
        let mut vm = CopySettingsMask::none();
        vm.effects_group = true;
        vm.effects_post_crop_vignette = true;
        list.push(make_builtin("vignette_heavy", "Vignetting", "Medium Edge Vignette", p, Some(vm)));
    }

    list
}
