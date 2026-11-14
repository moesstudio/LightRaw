// ============================================================================
// LightRaw — Core UI State, Two-Tier RAW LRU Cache & Workspace Shell
// ============================================================================

pub mod develop;
pub mod develop_canvas;
pub mod export_engine;
pub mod library;
pub mod menu_dispatch;
pub mod other_modules;
pub mod presets_dialogs;
pub mod system_modals;
pub mod watermark_editor;
pub mod widgets;

pub use widgets::*;

use crate::catalog::{
    create_default_collections, create_empty_placeholder_photo, decode_real_photo_file,
    detect_faces_and_focus_for_photo, extract_real_photo_metadata, is_supported_photo_path,
    query_volume_disk_space, CatalogDb, CollectionItem, ColorLabel, HistoryStep, PhotoItem,
    PickFlag, SavedLocationEntry,
};
use crate::params::{DevelopParams, RedEyeMode, RetouchMode};
use crate::pipeline::{
    estimate_auto_wb, render_develop_pipeline, RawSourceImage, RenderOptions, RenderedFrame,
};
use crate::presets::{
    apply_settings_with_mask, builtin_develop_presets, builtin_localized_mask_presets,
    export_preset_to_xmp, interpolate_preset_amount, load_presets_from_folder, parse_xmp_preset,
    CopySettingsMask, DevelopPreset, LocalMaskPreset,
};
use crate::updater::{
    check_latest_release, download_and_install_release, ApplyUpdateOutcome, RemoteRelease,
    UpdateCheckOutcome, UpdaterUiState, CURRENT_VERSION,
};
use eframe::egui::{self, Color32, Pos2, Rect, RichText, Sense, Stroke, TextureHandle, Ui, Vec2};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct Colors;
impl Colors {
    pub const BG_DARK: Color32 = Color32::from_rgb(22, 22, 25);
    pub const BG_PANEL: Color32 = Color32::from_rgb(38, 38, 42);
    pub const BG_HEADER: Color32 = Color32::from_rgb(46, 46, 52);
    pub const BG_CANVAS: Color32 = Color32::from_rgb(28, 28, 32);
    /// Lightroom Classic's signature neutral Medium Gray `#808080` Develop surround!
    pub const BG_DEVELOP_CANVAS: Color32 = Color32::from_rgb(128, 128, 128);
    pub const BG_TOOLBAR: Color32 = Color32::from_rgb(34, 34, 38);
    pub const BORDER: Color32 = Color32::from_rgb(20, 20, 24);
    pub const ACCENT_BLUE: Color32 = Color32::from_rgb(10, 132, 255);
    pub const ACCENT_AMBER: Color32 = Color32::from_rgb(235, 175, 65);
    pub const TEXT_BRIGHT: Color32 = Color32::from_rgb(238, 238, 242);
    pub const TEXT_MAIN: Color32 = Color32::from_rgb(208, 208, 214);
    pub const TEXT_MID: Color32 = Color32::from_rgb(152, 152, 160);
    pub const TEXT_DIM: Color32 = Color32::from_rgb(108, 108, 116);
    pub const GOLD_STAR: Color32 = Color32::from_rgb(245, 195, 50);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanvasSurroundMode {
    MediumGray,
    DarkGray,
    VeryDarkGray,
    LightGray,
    Black,
    White,
}

impl CanvasSurroundMode {
    pub const ALL: [Self; 6] = [
        Self::MediumGray,
        Self::DarkGray,
        Self::VeryDarkGray,
        Self::LightGray,
        Self::Black,
        Self::White,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::MediumGray => "Medium Gray (Default LrC #808080)",
            Self::DarkGray => "Dark Gray (#404040)",
            Self::VeryDarkGray => "Charcoal (#1C1C20)",
            Self::LightGray => "Light Gray (#B8B8B8)",
            Self::Black => "Black (#000000)",
            Self::White => "White (#FFFFFF)",
        }
    }

    pub fn color(self) -> Color32 {
        match self {
            Self::MediumGray => Color32::from_rgb(128, 128, 128),
            Self::DarkGray => Color32::from_rgb(64, 64, 64),
            Self::VeryDarkGray => Color32::from_rgb(28, 28, 32),
            Self::LightGray => Color32::from_rgb(184, 184, 184),
            Self::Black => Color32::BLACK,
            Self::White => Color32::WHITE,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspaceModule {
    Library,
    Develop,
    Map,
    Book,
    Slideshow,
    Print,
    Web,
}

impl WorkspaceModule {
    pub const ALL: [Self; 7] = [
        Self::Library,
        Self::Develop,
        Self::Map,
        Self::Book,
        Self::Slideshow,
        Self::Print,
        Self::Web,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Library => "Library",
            Self::Develop => "Develop",
            Self::Map => "Map",
            Self::Book => "Book",
            Self::Slideshow => "Slideshow",
            Self::Print => "Print",
            Self::Web => "Web",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibraryViewMode {
    Grid,
    Loupe,
    Compare,
    Survey,
    People,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DevelopTool {
    None,
    Crop,
    Remove,
    RedEye,
    Masking,
    GuidedUpright,
    TargetAdjust,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetAdjustKind {
    ParametricCurve,
    Hue,
    Saturation,
    Luminance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BeforeAfterMode {
    Single,
    SplitLeftRight,
    SplitTopBottom,
    SideBySide,
    BeforeOnly,
    ReferenceView,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LightsOutMode {
    Normal,
    LightsDim,
    LightsOff,
}

impl LightsOutMode {
    pub fn next(self) -> Self {
        match self {
            Self::Normal => Self::LightsDim,
            Self::LightsDim => Self::LightsOff,
            Self::LightsOff => Self::Normal,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PainterMode {
    Rating,
    Flag,
    ColorLabel,
    Preset,
}

impl PainterMode {
    pub const ALL: &'static [Self] = &[Self::Rating, Self::Flag, Self::ColorLabel, Self::Preset];
    pub fn label(self) -> &'static str {
        match self {
            Self::Rating => "Rating",
            Self::Flag => "Flag",
            Self::ColorLabel => "Label",
            Self::Preset => "Settings",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibrarySortMode {
    CaptureTime,
    ImportOrder,
    Rating,
    PickFlag,
    FileName,
    AspectRatio,
}

impl LibrarySortMode {
    pub const ALL: &'static [Self] = &[
        Self::CaptureTime,
        Self::ImportOrder,
        Self::Rating,
        Self::PickFlag,
        Self::FileName,
        Self::AspectRatio,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::CaptureTime => "Capture Time",
            Self::ImportOrder => "Added Order",
            Self::Rating => "Rating",
            Self::PickFlag => "Pick",
            Self::FileName => "File Name",
            Self::AspectRatio => "Aspect Ratio",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibraryLeftTab {
    Folders,
    AssistedCulling,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetadataViewPreset {
    Default,
    Exif,
    Iptc,
    LargeCaption,
}

impl MetadataViewPreset {
    pub const ALL: &'static [Self] = &[Self::Default, Self::Exif, Self::Iptc, Self::LargeCaption];
    pub fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Exif => "EXIF",
            Self::Iptc => "IPTC",
            Self::LargeCaption => "Large Caption",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibraryFilterMode {
    None,
    Text,
    Attribute,
    Metadata,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportActionChoice {
    CopyAsDng,
    Copy,
    Move,
    Add,
}

impl ImportActionChoice {
    pub fn label(self) -> &'static str {
        match self {
            Self::CopyAsDng => "Copy as DNG",
            Self::Copy => "Copy",
            Self::Move => "Move",
            Self::Add => "Add",
        }
    }
    pub fn subtitle(self) -> &'static str {
        match self {
            Self::CopyAsDng => "Copy photos to a new location and convert to DNG",
            Self::Copy => "Copy photos to a new location and add to catalog",
            Self::Move => "Move photos to a new location and add to catalog",
            Self::Add => "Add photos to catalog without moving them",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportViewMode {
    Grid,
    Loupe,
}

#[derive(Clone, Debug)]
pub struct ImportCandidate {
    pub path: PathBuf,
    pub file_name: String,
    pub file_size_bytes: u64,
    pub capture_time: String,
    pub is_checked: bool,
    pub is_duplicate: bool,
}

#[derive(Clone, Debug)]
pub struct GpxTrackPoint {
    pub lat: f64,
    pub lon: f64,
    pub time: String,
}

pub type SavedMapLocation = SavedLocationEntry;

pub struct LightRawApp {
    pub workspace_dir: PathBuf,
    pub catalog_db: CatalogDb,
    pub recent_catalogs: Vec<PathBuf>,
    pub catalog_display_name: String,
    pub identity_plate_owner: String,
    pub identity_plate_subtitle: String,

    pub photos: Vec<PhotoItem>,
    pub collections: Vec<CollectionItem>,
    pub target_collection_id: i64,
    pub presets: Vec<DevelopPreset>,
    pub local_presets: Vec<LocalMaskPreset>,

    // Two-Tier Bounded RAW & Texture Caches
    pub thumb_raw_cache: HashMap<i64, Arc<RawSourceImage>>,
    pub raw_cache: HashMap<i64, Arc<RawSourceImage>>,
    pub raw_cache_lru: VecDeque<i64>,
    pub rendered_thumb_cache: HashMap<i64, TextureHandle>,
    pub rendered_after_cache: HashMap<(i64, bool), (TextureHandle, RenderedFrame, RenderOptions)>,
    pub rendered_before_cache: HashMap<(i64, bool), TextureHandle>,
    pub nav_hover_preview_cache: Option<(String, TextureHandle)>,
    pub disk_space_cache: Option<(String, u64, u64, Instant)>,

    // Active Selection & Workspace Navigation
    pub active_photo_id: i64,
    pub was_pointer_down: bool,
    pub selected_photo_ids: Vec<i64>,
    pub compare_candidate_id: Option<i64>,
    pub reference_photo_id: Option<i64>,
    pub active_module: WorkspaceModule,
    pub library_view_mode: LibraryViewMode,
    pub selected_collection_id: Option<i64>,
    pub selected_folder: Option<String>,

    // Global Panels & Appearance
    pub show_in_app_menubar: bool,
    pub show_top_module_picker: bool,
    pub show_left_panel: bool,
    pub show_right_panel: bool,
    pub show_bottom_filmstrip: bool,
    pub show_toolbar: bool,
    pub lights_out: LightsOutMode,
    pub canvas_surround: CanvasSurroundMode,
    pub solo_mode_right_panels: bool,

    // Library Filter, Sort, Painter & Culling
    pub library_filter_mode: LibraryFilterMode,
    pub filter_text: String,
    pub filter_min_rating: u8,
    pub filter_only_picks: bool,
    pub filter_unflagged_only: bool,
    pub filter_color_label: Option<ColorLabel>,
    pub filter_camera: Option<String>,
    pub grid_thumb_size: f32,
    pub loupe_zoom: f32,
    pub loupe_pan: Vec2,
    pub zoom_drag_start: Option<Pos2>,
    pub show_info_overlay: bool,
    pub info_overlay_cycle: u8,
    pub painter_active: bool,
    pub painter_mode: PainterMode,
    pub painter_rating: u8,
    pub painter_flag: PickFlag,
    pub painter_color: Option<ColorLabel>,
    pub painter_preset: String,
    pub lib_sort_mode: LibrarySortMode,
    pub lib_sort_ascending: bool,
    pub lib_left_tab: LibraryLeftTab,

    pub lib_nav_open: bool,
    pub lib_catalog_open: bool,
    pub lib_folders_open: bool,
    pub lib_collections_open: bool,
    pub lib_publish_open: bool,
    pub lib_hist_open: bool,
    pub lib_culling_info_open: bool,
    pub lib_quick_dev_open: bool,
    pub lib_quick_dev_tone_expanded: bool,
    pub lib_keywording_open: bool,
    pub lib_keyword_list_open: bool,
    pub lib_metadata_open: bool,
    pub lib_comments_open: bool,

    pub culling_eye_focus_thresh: f32,
    pub culling_eyes_open_thresh: f32,
    pub culling_auto_stack: bool,
    pub culling_reject_blinks: bool,
    pub culling_filter_passed_only: bool,
    pub metadata_view_preset: MetadataViewPreset,
    pub metadata_apply_selected: bool,
    pub keyword_input_buffer: String,
    pub keyword_input_photo_id: i64,
    pub new_keyword_list_input: String,
    pub comment_input_buffer: String,
    pub rename_file_buffer: String,
    pub rename_file_photo_id: i64,

    // Develop State & Interactive Tools
    pub develop_tool: DevelopTool,
    pub target_adjust_kind: TargetAdjustKind,
    pub target_adjust_drag_last: Option<Pos2>,
    pub before_after_mode: BeforeAfterMode,
    pub show_shadow_clipping: bool,
    pub show_highlight_clipping: bool,
    pub soft_proofing: bool,
    pub soft_proof_profile: u8,
    pub soft_proof_perceptual: bool,
    pub simulate_paper_ink: bool,
    pub gamut_warning: bool,
    pub auto_sync_enabled: bool,

    pub wb_eyedropper_active: bool,
    pub point_color_eyedropper_active: bool,
    pub defringe_eyedropper_active: Option<bool>,
    pub lens_blur_focus_picker_active: bool,
    pub lens_blur_brush_mode: Option<bool>,
    pub lens_blur_brush_size: f32,
    pub lens_blur_brush_feather: f32,
    pub lens_blur_brush_flow: f32,
    pub active_stroke_in_progress: bool,

    pub detail_loupe_center: Vec2,
    pub detail_sample_tool_active: bool,
    pub sharpen_mask_preview_active: bool,
    pub guided_upright_drag_start: Option<(f32, f32)>,

    pub selected_retouch_idx: Option<usize>,
    pub dragging_retouch_source: Option<usize>,
    pub selected_redeye_idx: Option<usize>,
    pub selected_mask_idx: Option<usize>,
    pub selected_component_idx: usize,
    pub show_mask_overlay: bool,
    pub mask_brush_size: f32,
    pub mask_brush_feather: f32,
    pub mask_brush_flow: f32,

    pub retouch_mode_ui: RetouchMode,
    pub retouch_brush_radius: f32,
    pub retouch_feather: f32,
    pub retouch_opacity: f32,
    pub redeye_mode_ui: RedEyeMode,
    pub redeye_pupil_size: f32,
    pub redeye_darken: f32,
    pub redeye_catchlight: bool,
    pub crop_drag_handle: Option<u8>,
    pub crop_drag_last_pos: Option<Pos2>,
    pub crop_sync_all_photos: bool,
    pub hist_drag_zone: Option<u8>,
    pub hist_hover_zone: Option<u8>,
    pub ai_hardware_tier: crate::ai::AiHardwareTier,

    pub active_curve_channel: usize,
    pub curve_point_mode: bool,
    pub active_hsl_tab: usize,
    pub hsl_color_view_by_color: bool,
    pub hsl_selected_color_band: usize,
    pub selected_point_color_idx: usize,
    pub color_grade_view_tab: usize,
    pub lens_corrections_manual_tab: bool,
    pub show_profile_browser: bool,
    pub detail_nr_expanded: bool,
    pub lens_blur_refinement_expanded: bool,

    pub preset_search: String,
    pub hovered_preset_preview: Option<(String, DevelopParams)>,
    pub hovered_history_preview: Option<(String, DevelopParams)>,
    pub preset_amount: f32,
    pub expanded_preset_groups: HashSet<String>,
    pub apply_on_import_preset_id: Option<String>,
    pub bypassed_panel: Option<&'static str>,

    // Clipboard & Modals
    pub clipboard_params: Option<DevelopParams>,
    pub clipboard_mask: CopySettingsMask,
    pub previous_edited_params: Option<DevelopParams>,
    pub show_copy_settings_modal: bool,
    pub copy_settings_subset: String,
    pub show_sync_settings_modal: bool,
    pub sync_settings_mask: CopySettingsMask,
    pub sync_settings_subset: String,
    pub show_create_preset_modal: bool,
    pub new_preset_name: String,
    pub new_preset_group: String,
    pub create_preset_mask: CopySettingsMask,
    pub create_preset_subset: String,
    pub show_rename_preset_modal: Option<String>,
    pub rename_preset_input: String,
    pub show_move_preset_modal: Option<String>,
    pub move_preset_group_input: String,
    pub show_manage_presets_modal: bool,

    pub show_create_snapshot_modal: bool,
    pub new_snapshot_name: String,
    pub show_rename_snapshot_modal: Option<usize>,
    pub rename_snapshot_input: String,
    pub show_create_collection_modal: bool,
    pub create_collection_is_smart: bool,
    pub new_collection_name: String,
    pub new_smart_min_rating: u8,
    pub new_smart_picked_only: bool,
    pub new_smart_keyword: String,
    pub show_rename_collection_modal: Option<i64>,
    pub rename_collection_input: String,
    pub show_sync_metadata_modal: bool,
    pub sync_meta_title_caption: bool,
    pub sync_meta_creator_copyright: bool,
    pub sync_meta_rating_label: bool,
    pub sync_meta_keywords: bool,
    pub sync_meta_gps_location: bool,

    pub show_preferences_modal: bool,
    pub show_shortcuts_modal: bool,
    pub show_import_modal: bool,
    pub show_export_modal: bool,
    pub show_watermark_editor_modal: bool,
    pub show_about_modal: bool,
    pub about_tab: usize,
    pub show_system_info_modal: bool,
    pub show_rename_catalog_modal: bool,
    pub rename_catalog_input: String,

    // Develop Panel Open Flags
    pub dev_nav_open: bool,
    pub dev_presets_open: bool,
    pub dev_snapshots_open: bool,
    pub dev_history_open: bool,
    pub dev_collections_open: bool,
    pub dev_hist_open: bool,
    pub dev_basic_open: bool,
    pub dev_curve_open: bool,
    pub dev_hsl_open: bool,
    pub dev_color_grading_open: bool,
    pub dev_detail_open: bool,
    pub dev_lens_blur_open: bool,
    pub dev_lens_open: bool,
    pub dev_transform_open: bool,
    pub dev_effects_open: bool,
    pub dev_calibration_open: bool,

    // Import & Export State
    pub import_folder_input: String,
    pub import_copy_to_catalog: bool,
    pub import_apply_auto_tone: bool,
    pub import_keywords: String,
    pub import_action: ImportActionChoice,
    pub import_view_mode: ImportViewMode,
    pub import_selected_source: Option<PathBuf>,
    pub import_expanded_folders: HashSet<PathBuf>,
    pub import_candidates: Vec<ImportCandidate>,
    pub import_thumb_cache: HashMap<PathBuf, TextureHandle>,
    pub import_grid_thumb_size: f32,
    pub import_sort_mode: String,
    pub import_build_previews: String,
    pub import_build_smart_previews: bool,
    pub import_dont_import_duplicates: bool,
    pub import_make_second_copy: bool,
    pub import_destination_folder: String,
    pub import_subfolder: String,
    pub import_develop_preset: String,
    pub import_file_handling_open: bool,
    pub import_apply_during_open: bool,
    pub import_destination_open: bool,

    pub export_folder: String,
    pub export_subfolder: String,
    pub export_format: String,
    pub export_color_space: String,
    pub export_quality: u8,
    pub export_resize_enabled: bool,
    pub export_long_edge_px: u32,
    pub export_sharpening: String,
    pub export_watermark_enabled: bool,
    pub export_watermark_text: String,
    pub watermark_settings: crate::watermark::WatermarkSettings,
    pub watermark_presets: Vec<crate::watermark::WatermarkPreset>,
    pub active_watermark_preset_name: String,
    pub watermark_graphic_texture: Option<(String, TextureHandle, (u32, u32))>,
    pub app_icon_texture: Option<TextureHandle>,
    pub export_xmp_sidecar: bool,
    pub export_rename_enabled: bool,
    pub export_rename_prefix: String,

    // Map, Book, Slideshow, Print & Web State
    pub map_style: String,
    pub map_zoom: f32,
    pub map_center_lat: f64,
    pub map_center_lon: f64,
    pub saved_locations: Vec<SavedMapLocation>,
    pub new_location_name: String,
    pub new_location_radius: f32,
    pub gpx_track_points: Vec<GpxTrackPoint>,
    pub map_left_open: bool,
    pub map_right_open: bool,

    pub book_title: String,
    pub book_subtitle: String,
    pub book_author: String,
    pub book_size: String,
    pub book_cover: String,
    pub book_layout: String,
    pub book_page_padding: f32,
    pub book_show_page_numbers: bool,
    pub book_bg_white: bool,
    pub book_current_spread: usize,
    pub book_open_settings: bool,
    pub book_open_layout: bool,
    pub book_open_text: bool,

    pub slideshow_playing: bool,
    pub slideshow_fullscreen: bool,
    pub slideshow_slide_duration: f32,
    pub slideshow_crossfade: bool,
    pub slideshow_crossfade_duration: f32,
    pub slideshow_last_tick: Option<Instant>,
    pub slideshow_prev_photo_id: Option<i64>,
    pub slideshow_transition_t: f32,
    pub slideshow_show_identity: bool,
    pub slideshow_show_caption: bool,
    pub slideshow_stroke_border: bool,
    pub slideshow_backdrop_dark: bool,
    pub slideshow_loop: bool,
    pub slideshow_random_order: bool,
    pub slideshow_open_options: bool,

    pub print_layout_style: String,
    pub print_rows: usize,
    pub print_cols: usize,
    pub print_margin_mm: f32,
    pub print_cell_spacing: f32,
    pub print_show_guides: bool,
    pub print_show_dimensions: bool,
    pub print_stroke_border: bool,
    pub print_show_page_info: bool,
    pub print_show_watermark: bool,
    pub print_dpi: f32,
    pub print_current_page: usize,
    pub print_open_layout: bool,
    pub print_open_guides: bool,
    pub print_open_job: bool,

    pub web_site_title: String,
    pub web_collection_title: String,
    pub web_description: String,
    pub web_columns: usize,
    pub web_dark_theme: bool,
    pub web_show_exif: bool,
    pub web_quality: u8,
    pub web_open_settings: bool,

    // Coalescing, Debouncing & Idle CPU Guard
    pub status_message: String,
    pub last_window_title: String,
    pub dirty_photo_ids: HashSet<i64>,
    pub last_edit_instant: Option<Instant>,
    pub last_history_step_meta: Option<(i64, String, Instant)>,
    pub last_frame_instant: Instant,
    pub rolling_frame_time_ms: f32,

    // In-App Auto-Updater State
    pub show_update_modal: bool,
    pub auto_update_checked_startup: bool,
    pub update_state: UpdaterUiState,
    pub update_rx: Option<std::sync::mpsc::Receiver<UpdaterUiState>>,
}

fn merge_and_dedup_presets(
    builtins: Vec<DevelopPreset>,
    folder_presets: Vec<DevelopPreset>,
    db_presets: Vec<DevelopPreset>,
) -> Vec<DevelopPreset> {
    let mut out: Vec<DevelopPreset> = builtins;
    for incoming in folder_presets.into_iter().chain(db_presets.into_iter()) {
        if let Some(existing) = out.iter_mut().find(|p| {
            p.id == incoming.id
                || (p.name.eq_ignore_ascii_case(&incoming.name)
                    && p.category.eq_ignore_ascii_case(&incoming.category))
        }) {
            *existing = incoming;
        } else {
            out.push(incoming);
        }
    }
    out
}

impl LightRawApp {
    pub fn new(workspace_dir: PathBuf) -> Self {
        let catalog_db = CatalogDb::open_or_create(&workspace_dir);
        let photos = catalog_db.load_all_photos();

        let mut collections = catalog_db.load_collections();
        if collections.is_empty() {
            collections = create_default_collections();
            for c in &collections {
                catalog_db.save_collection(c);
            }
        }

        let presets_dir = workspace_dir.join("Presets");
        let _ = std::fs::create_dir_all(&presets_dir);
        let folder_presets = load_presets_from_folder(&presets_dir);
        let db_presets = catalog_db.load_presets();
        let presets =
            merge_and_dedup_presets(builtin_develop_presets(), folder_presets, db_presets);

        let mut expanded_preset_groups = HashSet::new();
        expanded_preset_groups.insert("Favorites".into());
        expanded_preset_groups.insert("Adaptive: Portrait".into());
        expanded_preset_groups.insert("Color".into());
        expanded_preset_groups.insert("User Presets".into());

        let mut saved_locations = catalog_db.load_saved_locations();
        if saved_locations.is_empty() {
            saved_locations = vec![
                SavedLocationEntry {
                    name: "Jakarta Studio".into(),
                    lat: -6.2088,
                    lon: 106.8456,
                    radius_m: 150.0,
                },
                SavedLocationEntry {
                    name: "Mount Bromo Caldera".into(),
                    lat: -7.9425,
                    lon: 112.9530,
                    radius_m: 800.0,
                },
                SavedLocationEntry {
                    name: "Labuan Bajo Harbor".into(),
                    lat: -8.4964,
                    lon: 119.8877,
                    radius_m: 400.0,
                },
            ];
            catalog_db.save_saved_locations(&saved_locations);
        }

        let catalog_display_name = catalog_db
            .load_app_var("catalog_display_name")
            .unwrap_or_else(|| "LightRawCatalog.lrcat".into());
        let identity_plate_owner = catalog_db
            .load_app_var("identity_plate_owner")
            .unwrap_or_else(|| "Moestret".into());
        let identity_plate_subtitle = catalog_db
            .load_app_var("identity_plate_subtitle")
            .unwrap_or_else(|| "LIGHTRAW".into());
        let ai_hardware_tier = match catalog_db.load_app_var("ai_hardware_tier").as_deref() {
            Some("cpu_simd") => crate::ai::AiHardwareTier::CpuSimd,
            Some("heuristics_only") => crate::ai::AiHardwareTier::HeuristicsOnly,
            _ => crate::ai::AiHardwareTier::Auto,
        };

        let active_photo_id = photos.first().map(|p| p.id).unwrap_or(1);
        let selected_photo_ids = if photos.is_empty() {
            Vec::new()
        } else {
            vec![active_photo_id]
        };

        let default_export_dir = std::env::var_os("HOME")
            .map(|h| PathBuf::from(h).join("Pictures").join("LightRaw_Exports"))
            .unwrap_or_else(|| workspace_dir.join("Exports"))
            .to_string_lossy()
            .to_string();
        let default_import_dest = std::env::var_os("HOME")
            .map(|h| PathBuf::from(h).join("Pictures").join("LightRaw_Imported"))
            .unwrap_or_else(|| workspace_dir.join("Imported"))
            .to_string_lossy()
            .to_string();

        let mut import_expanded_folders = HashSet::new();
        import_expanded_folders.insert(PathBuf::from("/"));
        import_expanded_folders.insert(PathBuf::from("/Users"));
        if let Some(home) = std::env::var_os("HOME") {
            let hp = PathBuf::from(&home);
            import_expanded_folders.insert(hp.clone());
            import_expanded_folders.insert(hp.join("Pictures"));
        }

        Self {
            workspace_dir,
            recent_catalogs: vec![catalog_db.db_path.clone()],
            catalog_db,
            catalog_display_name,
            identity_plate_owner,
            identity_plate_subtitle,
            photos,
            collections,
            target_collection_id: 1,
            presets,
            local_presets: builtin_localized_mask_presets(),

            thumb_raw_cache: HashMap::new(),
            raw_cache: HashMap::new(),
            raw_cache_lru: VecDeque::new(),
            rendered_thumb_cache: HashMap::new(),
            rendered_after_cache: HashMap::new(),
            rendered_before_cache: HashMap::new(),
            nav_hover_preview_cache: None,
            disk_space_cache: None,

            active_photo_id,
            was_pointer_down: false,
            selected_photo_ids,
            compare_candidate_id: None,
            reference_photo_id: None,
            active_module: WorkspaceModule::Library,
            library_view_mode: LibraryViewMode::Grid,
            selected_collection_id: None,
            selected_folder: None,

            show_in_app_menubar: if cfg!(target_os = "macos") { false } else { true },
            show_top_module_picker: true,
            show_left_panel: true,
            show_right_panel: true,
            show_bottom_filmstrip: true,
            show_toolbar: true,
            lights_out: LightsOutMode::Normal,
            canvas_surround: CanvasSurroundMode::MediumGray,
            solo_mode_right_panels: false,
            ai_hardware_tier,

            library_filter_mode: LibraryFilterMode::None,
            filter_text: String::new(),
            filter_min_rating: 0,
            filter_only_picks: false,
            filter_unflagged_only: false,
            filter_color_label: None,
            filter_camera: None,
            grid_thumb_size: 190.0,
            loupe_zoom: 1.0,
            loupe_pan: Vec2::ZERO,
            zoom_drag_start: None,
            show_info_overlay: false,
            info_overlay_cycle: 0,
            painter_active: false,
            painter_mode: PainterMode::Rating,
            painter_rating: 5,
            painter_flag: PickFlag::Picked,
            painter_color: Some(ColorLabel::Green),
            painter_preset: "Flattering Skin".into(),
            lib_sort_mode: LibrarySortMode::CaptureTime,
            lib_sort_ascending: true,
            lib_left_tab: LibraryLeftTab::Folders,

            lib_nav_open: true,
            lib_catalog_open: true,
            lib_folders_open: true,
            lib_collections_open: true,
            lib_publish_open: false,
            lib_hist_open: true,
            lib_culling_info_open: true,
            lib_quick_dev_open: true,
            lib_quick_dev_tone_expanded: true,
            lib_keywording_open: true,
            lib_keyword_list_open: false,
            lib_metadata_open: true,
            lib_comments_open: false,

            culling_eye_focus_thresh: 55.0,
            culling_eyes_open_thresh: 50.0,
            culling_auto_stack: true,
            culling_reject_blinks: true,
            culling_filter_passed_only: false,
            metadata_view_preset: MetadataViewPreset::Default,
            metadata_apply_selected: false,
            keyword_input_buffer: String::new(),
            keyword_input_photo_id: -1,
            new_keyword_list_input: String::new(),
            comment_input_buffer: String::new(),
            rename_file_buffer: String::new(),
            rename_file_photo_id: -1,

            develop_tool: DevelopTool::None,
            target_adjust_kind: TargetAdjustKind::ParametricCurve,
            target_adjust_drag_last: None,
            before_after_mode: BeforeAfterMode::Single,
            show_shadow_clipping: false,
            show_highlight_clipping: false,
            soft_proofing: false,
            soft_proof_profile: 0,
            soft_proof_perceptual: true,
            simulate_paper_ink: false,
            gamut_warning: false,
            auto_sync_enabled: false,

            wb_eyedropper_active: false,
            point_color_eyedropper_active: false,
            defringe_eyedropper_active: None,
            lens_blur_focus_picker_active: false,
            lens_blur_brush_mode: None,
            lens_blur_brush_size: 0.06,
            lens_blur_brush_feather: 50.0,
            lens_blur_brush_flow: 80.0,
            active_stroke_in_progress: false,

            detail_loupe_center: Vec2::new(0.5, 0.35),
            detail_sample_tool_active: false,
            sharpen_mask_preview_active: false,
            guided_upright_drag_start: None,

            selected_retouch_idx: None,
            dragging_retouch_source: None,
            selected_redeye_idx: None,
            selected_mask_idx: None,
            selected_component_idx: 0,
            show_mask_overlay: false,
            mask_brush_size: 0.08,
            mask_brush_feather: 55.0,
            mask_brush_flow: 85.0,

            retouch_mode_ui: RetouchMode::Remove,
            retouch_brush_radius: 0.035,
            retouch_feather: 50.0,
            retouch_opacity: 100.0,
            redeye_mode_ui: RedEyeMode::RedEye,
            redeye_pupil_size: 50.0,
            redeye_darken: 50.0,
            redeye_catchlight: true,
            crop_drag_handle: None,
            crop_drag_last_pos: None,
            crop_sync_all_photos: false,
            hist_drag_zone: None,
            hist_hover_zone: None,

            active_curve_channel: 0,
            curve_point_mode: false,
            active_hsl_tab: 0,
            hsl_color_view_by_color: false,
            hsl_selected_color_band: 0,
            selected_point_color_idx: 0,
            color_grade_view_tab: 0,
            lens_corrections_manual_tab: false,
            show_profile_browser: false,
            detail_nr_expanded: true,
            lens_blur_refinement_expanded: false,

            preset_search: String::new(),
            hovered_preset_preview: None,
            hovered_history_preview: None,
            preset_amount: 100.0,
            expanded_preset_groups,
            apply_on_import_preset_id: None,
            bypassed_panel: None,

            clipboard_params: None,
            clipboard_mask: CopySettingsMask::default(),
            previous_edited_params: None,
            show_copy_settings_modal: false,
            copy_settings_subset: "Default".into(),
            show_sync_settings_modal: false,
            sync_settings_mask: CopySettingsMask::default(),
            sync_settings_subset: "Default".into(),
            show_create_preset_modal: false,
            new_preset_name: "Untitled Preset".into(),
            new_preset_group: "User Presets".into(),
            create_preset_mask: CopySettingsMask::default(),
            create_preset_subset: "Default".into(),
            show_rename_preset_modal: None,
            rename_preset_input: String::new(),
            show_move_preset_modal: None,
            move_preset_group_input: "User Presets".into(),
            show_manage_presets_modal: false,

            show_create_snapshot_modal: false,
            new_snapshot_name: "Snapshot 1".into(),
            show_rename_snapshot_modal: None,
            rename_snapshot_input: String::new(),
            show_create_collection_modal: false,
            create_collection_is_smart: false,
            new_collection_name: "New Collection".into(),
            new_smart_min_rating: 3,
            new_smart_picked_only: false,
            new_smart_keyword: String::new(),
            show_rename_collection_modal: None,
            rename_collection_input: String::new(),
            show_sync_metadata_modal: false,
            sync_meta_title_caption: true,
            sync_meta_creator_copyright: true,
            sync_meta_rating_label: true,
            sync_meta_keywords: true,
            sync_meta_gps_location: false,

            show_preferences_modal: false,
            show_shortcuts_modal: false,
            show_import_modal: false,
            show_export_modal: false,
            show_watermark_editor_modal: false,
            show_about_modal: false,
            about_tab: 0,
            show_system_info_modal: false,
            show_rename_catalog_modal: false,
            rename_catalog_input: String::new(),

            dev_nav_open: true,
            dev_presets_open: true,
            dev_snapshots_open: false,
            dev_history_open: false,
            dev_collections_open: false,
            dev_hist_open: true,
            dev_basic_open: true,
            dev_curve_open: false,
            dev_hsl_open: false,
            dev_color_grading_open: false,
            dev_detail_open: true,
            dev_lens_blur_open: false,
            dev_lens_open: false,
            dev_transform_open: false,
            dev_effects_open: false,
            dev_calibration_open: false,

            import_folder_input: String::new(),
            import_copy_to_catalog: false,
            import_apply_auto_tone: false,
            import_keywords: String::new(),
            import_action: ImportActionChoice::Add,
            import_view_mode: ImportViewMode::Grid,
            import_selected_source: None,
            import_expanded_folders,
            import_candidates: Vec::new(),
            import_thumb_cache: HashMap::new(),
            import_grid_thumb_size: 120.0,
            import_sort_mode: "Capture Time".into(),
            import_build_previews: "Standard".into(),
            import_build_smart_previews: false,
            import_dont_import_duplicates: true,
            import_make_second_copy: false,
            import_destination_folder: default_import_dest,
            import_subfolder: "2026_Shoot".into(),
            import_develop_preset: "None".into(),
            import_file_handling_open: true,
            import_apply_during_open: true,
            import_destination_open: true,

            export_folder: default_export_dir,
            export_subfolder: "LightRaw_Processed".into(),
            export_format: "JPEG".into(),
            export_color_space: "sRGB".into(),
            export_quality: 92,
            export_resize_enabled: false,
            export_long_edge_px: 2048,
            export_sharpening: "Screen — Standard".into(),
            export_watermark_enabled: false,
            export_watermark_text: "© Moestret Photography".into(),
            watermark_settings: {
                let wm_list = crate::watermark::WatermarkPreset::default_presets();
                wm_list
                    .first()
                    .map(|p| p.settings.clone())
                    .unwrap_or_default()
            },
            watermark_presets: crate::watermark::WatermarkPreset::default_presets(),
            active_watermark_preset_name: {
                let wm_list = crate::watermark::WatermarkPreset::default_presets();
                wm_list
                    .first()
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| "Simple Copyright Watermark".into())
            },
            watermark_graphic_texture: None,
            app_icon_texture: None,
            export_xmp_sidecar: false,
            export_rename_enabled: false,
            export_rename_prefix: "Moestret".into(),

            map_style: "Dark Satellite Hybrid".into(),
            map_zoom: 1.4,
            map_center_lat: -5.0,
            map_center_lon: 112.0,
            saved_locations,
            new_location_name: "New Shoot Location".into(),
            new_location_radius: 200.0,
            gpx_track_points: Vec::new(),
            map_left_open: true,
            map_right_open: true,

            book_title: "Portfolio Monograph".into(),
            book_subtitle: "Selected Photographs".into(),
            book_author: "Moestret".into(),
            book_size: "Standard Landscape (10x8 in)".into(),
            book_cover: "Hardcover Image Wrap".into(),
            book_layout: "Two-Page Spread".into(),
            book_page_padding: 22.0,
            book_show_page_numbers: true,
            book_bg_white: true,
            book_current_spread: 0,
            book_open_settings: true,
            book_open_layout: true,
            book_open_text: true,

            slideshow_playing: false,
            slideshow_fullscreen: false,
            slideshow_slide_duration: 4.0,
            slideshow_crossfade: true,
            slideshow_crossfade_duration: 1.0,
            slideshow_last_tick: None,
            slideshow_prev_photo_id: None,
            slideshow_transition_t: 1.0,
            slideshow_show_identity: true,
            slideshow_show_caption: true,
            slideshow_stroke_border: true,
            slideshow_backdrop_dark: true,
            slideshow_loop: true,
            slideshow_random_order: false,
            slideshow_open_options: true,

            print_layout_style: "Single Image / Contact Sheet".into(),
            print_rows: 2,
            print_cols: 2,
            print_margin_mm: 12.0,
            print_cell_spacing: 10.0,
            print_show_guides: true,
            print_show_dimensions: true,
            print_stroke_border: true,
            print_show_page_info: true,
            print_show_watermark: true,
            print_dpi: 300.0,
            print_current_page: 0,
            print_open_layout: true,
            print_open_guides: true,
            print_open_job: true,

            web_site_title: "Moestret Visual Archive".into(),
            web_collection_title: "Selected Works".into(),
            web_description: "Processed in LightRaw".into(),
            web_columns: 4,
            web_dark_theme: true,
            web_show_exif: true,
            web_quality: 88,
            web_open_settings: true,

            status_message: "Ready — 32-Bit Linear RAW & ACR PV2012 Engine Active".into(),
            last_window_title: String::new(),
            dirty_photo_ids: HashSet::new(),
            last_edit_instant: None,
            last_history_step_meta: None,
            last_frame_instant: Instant::now(),
            rolling_frame_time_ms: 1.0,
            show_update_modal: false,
            auto_update_checked_startup: false,
            update_state: UpdaterUiState::Idle,
            update_rx: None,
        }
    }

    pub fn open_or_create_catalog_at(&mut self, path: &Path, clear_existing: bool) {
        self.flush_dirty_photos();
        if clear_existing && path.exists() {
            let _ = std::fs::remove_file(path);
        }
        let Ok(db) = CatalogDb::open_specific_file(path) else {
            self.status_message = format!("Failed to open catalog: {}", path.display());
            return;
        };
        if !self.recent_catalogs.iter().any(|p| p == path) {
            self.recent_catalogs.insert(0, path.to_path_buf());
            self.recent_catalogs.truncate(5);
        }
        self.catalog_display_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Catalog.lrcat")
            .to_string();
        self.photos = db.load_all_photos();
        self.collections = db.load_collections();
        if self.collections.is_empty() {
            self.collections = create_default_collections();
            for c in &self.collections {
                db.save_collection(c);
            }
        }
        self.catalog_db = db;
        self.thumb_raw_cache.clear();
        self.raw_cache.clear();
        self.raw_cache_lru.clear();
        self.rendered_thumb_cache.clear();
        self.rendered_after_cache.clear();
        self.rendered_before_cache.clear();
        self.active_photo_id = self.photos.first().map(|p| p.id).unwrap_or(1);
        self.selected_photo_ids = if self.photos.is_empty() {
            Vec::new()
        } else {
            vec![self.active_photo_id]
        };
        self.selected_folder = None;
        self.selected_collection_id = None;
        self.status_message = format!(
            "Opened catalog {} ({} photos).",
            self.catalog_display_name,
            self.photos.len()
        );
    }

    pub fn import_from_external_catalog_dialog(&mut self) {
        let Some(ext_path) = rfd::FileDialog::new()
            .set_title("Import From Another LightRaw Catalog (.lrcat)")
            .add_filter("Lightroom Catalog", &["lrcat", "sqlite", "db"])
            .pick_file()
        else {
            return;
        };
        let Ok(ext_db) = CatalogDb::open_specific_file(&ext_path) else {
            self.status_message = format!("Failed to open external catalog: {}", ext_path.display());
            return;
        };
        let incoming = ext_db.load_all_photos();
        let mut next_id = self.photos.iter().map(|p| p.id).max().unwrap_or(0) + 1;
        let mut added = 0usize;
        for mut ph in incoming {
            let dup = ph.real_file_path.as_ref().map_or(false, |rp| {
                self.photos
                    .iter()
                    .any(|e| e.real_file_path.as_ref() == Some(rp) && !e.is_virtual_copy)
            });
            if dup {
                continue;
            }
            ph.id = next_id;
            next_id += 1;
            self.catalog_db.save_photo(&ph);
            self.photos.push(ph);
            added += 1;
        }
        for pr in ext_db.load_presets() {
            if !self.presets.iter().any(|x| x.name == pr.name && x.category == pr.category) {
                self.catalog_db.save_preset(&pr);
                self.presets.push(pr);
            }
        }
        self.status_message = format!(
            "Imported {} photo(s) from catalog {}.",
            added,
            ext_path.display()
        );
    }

    pub fn export_selection_as_catalog_dialog(&mut self) {
        let Some(dest_path) = rfd::FileDialog::new()
            .set_title("Export Selected Photos as Catalog (.lrcat)")
            .set_file_name("ExportedSelection.lrcat")
            .add_filter("Lightroom Catalog", &["lrcat"])
            .save_file()
        else {
            return;
        };
        if dest_path.exists() {
            let _ = std::fs::remove_file(&dest_path);
        }
        let Ok(out_db) = CatalogDb::open_specific_file(&dest_path) else {
            self.status_message = format!("Failed to create catalog: {}", dest_path.display());
            return;
        };
        let ids = if self.selected_photo_ids.is_empty() {
            vec![self.active_photo_id]
        } else {
            self.selected_photo_ids.clone()
        };
        let mut count = 0usize;
        for id in ids {
            if let Some(p) = self.photos.iter().find(|x| x.id == id) {
                out_db.save_photo(p);
                count += 1;
            }
        }
        for c in create_default_collections() {
            out_db.save_collection(&c);
        }
        self.status_message = format!(
            "Exported {} photo(s) as standalone catalog {}",
            count,
            dest_path.display()
        );
    }

    pub fn cached_disk_space(&mut self) -> (String, u64, u64) {
        if let Some((ref vol, free_b, total_b, ts)) = self.disk_space_cache {
            if ts.elapsed() < Duration::from_secs(12) {
                return (vol.clone(), free_b, total_b);
            }
        }
        let target_path = self
            .selected_folder
            .as_deref()
            .map(Path::new)
            .filter(|p| p.exists())
            .unwrap_or(&self.workspace_dir);
        let (vol, free_b, total_b) = query_volume_disk_space(target_path);
        self.disk_space_cache = Some((vol.clone(), free_b, total_b, Instant::now()));
        (vol, free_b, total_b)
    }

    pub fn canvas_bg_color(&self) -> Color32 {
        match self.lights_out {
            LightsOutMode::LightsOff => Color32::BLACK,
            LightsOutMode::LightsDim => Color32::from_rgb(16, 16, 18),
            LightsOutMode::Normal => {
                if self.active_module == WorkspaceModule::Develop {
                    self.canvas_surround.color()
                } else {
                    Colors::BG_CANVAS
                }
            }
        }
    }

    pub fn active_photo(&self) -> &PhotoItem {
        static EMPTY_FALLBACK: std::sync::OnceLock<PhotoItem> = std::sync::OnceLock::new();
        self.photos
            .iter()
            .find(|p| p.id == self.active_photo_id)
            .or_else(|| self.photos.first())
            .unwrap_or_else(|| EMPTY_FALLBACK.get_or_init(create_empty_placeholder_photo))
    }

    pub fn active_photo_mut(&mut self) -> &mut PhotoItem {
        if self.photos.is_empty() {
            let mut p = create_empty_placeholder_photo();
            p.id = 1;
            self.active_photo_id = 1;
            self.photos.push(p);
        }
        let active_id = self.active_photo_id;
        let pos = self
            .photos
            .iter()
            .position(|p| p.id == active_id)
            .unwrap_or(0);
        &mut self.photos[pos]
    }

    pub fn active_photo_pixel_size(&mut self) -> (f32, f32) {
        let pid = self.active_photo_id;
        if let Some(raw) = self.ensure_loupe_raw(pid) {
            (raw.width.max(1) as f32, raw.height.max(1) as f32)
        } else {
            let ph = self.active_photo();
            (ph.width.max(1) as f32, ph.height.max(1) as f32)
        }
    }

    pub fn effective_target_ids(&self) -> Vec<i64> {
        if self.selected_photo_ids.len() > 1
            && (self.active_module != WorkspaceModule::Develop || self.auto_sync_enabled)
        {
            self.selected_photo_ids.clone()
        } else if !self.selected_photo_ids.is_empty()
            && self.active_module == WorkspaceModule::Library
        {
            self.selected_photo_ids.clone()
        } else {
            vec![self.active_photo_id]
        }
    }

    pub fn select_photo(&mut self, id: i64, add_to_selection: bool) {
        self.flush_dirty_photos();
        if self.active_module == WorkspaceModule::Library
            && self.library_view_mode == LibraryViewMode::Compare
            && add_to_selection
        {
            self.compare_candidate_id = Some(id);
            return;
        }
        self.active_photo_id = id;
        if add_to_selection {
            if !self.selected_photo_ids.contains(&id) {
                self.selected_photo_ids.push(id);
            }
        } else {
            self.selected_photo_ids = vec![id];
        }
        self.preset_amount = 100.0;
    }

    pub fn toggle_select_photo(&mut self, id: i64) {
        if self.selected_photo_ids.contains(&id) {
            if self.selected_photo_ids.len() > 1 {
                self.selected_photo_ids.retain(|&x| x != id);
                if self.active_photo_id == id {
                    self.active_photo_id = self.selected_photo_ids[0];
                }
            }
        } else {
            self.selected_photo_ids.push(id);
            self.active_photo_id = id;
        }
    }

    pub fn select_photo_range(&mut self, end_id: i64) {
        let visible = self.filtered_photo_ids();
        let Some(p0) = visible.iter().position(|&x| x == self.active_photo_id) else {
            self.select_photo(end_id, false);
            return;
        };
        let Some(p1) = visible.iter().position(|&x| x == end_id) else {
            self.select_photo(end_id, false);
            return;
        };
        let lo = p0.min(p1);
        let hi = p0.max(p1);
        for &pid in &visible[lo..=hi] {
            if !self.selected_photo_ids.contains(&pid) {
                self.selected_photo_ids.push(pid);
            }
        }
        self.active_photo_id = end_id;
    }

    pub fn select_all(&mut self) {
        let visible = self.filtered_photo_ids();
        if let Some(&first) = visible.first() {
            if !visible.contains(&self.active_photo_id) {
                self.active_photo_id = first;
            }
            self.selected_photo_ids = visible;
        }
    }

    pub fn deselect_all(&mut self) {
        if !self.photos.is_empty() {
            self.selected_photo_ids = vec![self.active_photo_id];
        } else {
            self.selected_photo_ids.clear();
        }
    }

    pub fn step_active_photo(&mut self, delta: isize) {
        let visible = self.filtered_photo_ids();
        if visible.is_empty() {
            return;
        }
        let cur_pos = visible
            .iter()
            .position(|&id| id == self.active_photo_id)
            .unwrap_or(0) as isize;
        let next_pos = (cur_pos + delta).rem_euclid(visible.len() as isize) as usize;
        let next_id = visible[next_pos];
        self.select_photo(next_id, false);
    }

    pub fn next_photo_id(&self, current_id: i64) -> i64 {
        let visible = self.filtered_photo_ids();
        if visible.is_empty() {
            return current_id;
        }
        let pos = visible.iter().position(|&x| x == current_id).unwrap_or(0);
        visible[(pos + 1) % visible.len()]
    }

    pub fn smart_collection_matches(&self, col: &CollectionItem, p: &PhotoItem) -> bool {
        if !col.is_smart {
            return col.photo_ids.contains(&p.id);
        }
        // Built-in smart collections
        match col.id {
            2 => return p.rating >= 5,
            3 => {
                let lc = format!("{} {}", p.camera_model, p.file_name).to_lowercase();
                return p.keywords.iter().any(|k| {
                    let kl = k.to_lowercase();
                    kl.contains("portrait") || kl.contains("siswa") || kl.contains("smk")
                }) || !p.detected_faces.is_empty()
                    || lc.contains("dsc");
            }
            4 => {
                return p.params.hdr_edit_mode
                    || p.params.exposure_2012.abs() > 0.3
                    || p.rating >= 4;
            }
            _ => {}
        }
        // User-created dynamic smart collection rule stored in `app_settings`
        if let Some(rule_str) = self
            .catalog_db
            .load_app_var(&format!("smart_col_rule_{}", col.id))
        {
            let mut min_r = 0u8;
            let mut pick_only = false;
            let mut kw = String::new();
            for part in rule_str.split(';') {
                if let Some(v) = part.strip_prefix("rating>=") {
                    min_r = v.parse::<u8>().unwrap_or(0);
                } else if let Some(v) = part.strip_prefix("pick=") {
                    pick_only = v == "1";
                } else if let Some(v) = part.strip_prefix("kw=") {
                    kw = v.to_lowercase();
                }
            }
            if p.rating < min_r {
                return false;
            }
            if pick_only && p.flag != PickFlag::Picked {
                return false;
            }
            if !kw.is_empty() {
                let hay = format!(
                    "{} {} {} {}",
                    p.file_name,
                    p.title,
                    p.caption,
                    p.keywords.join(" ")
                )
                .to_lowercase();
                if !hay.contains(&kw) {
                    return false;
                }
            }
            return true;
        }
        p.rating >= 3
    }

    pub fn filtered_photo_ids(&self) -> Vec<i64> {
        let q = self.filter_text.trim().to_lowercase();
        let active_col = self
            .selected_collection_id
            .and_then(|cid| self.collections.iter().find(|c| c.id == cid));

        let mut matched: Vec<&PhotoItem> = self
            .photos
            .iter()
            .filter(|p| {
                if let Some(ref folder) = self.selected_folder {
                    if &p.folder_path != folder {
                        return false;
                    }
                }
                if let Some(col) = active_col {
                    if !self.smart_collection_matches(col, p) {
                        return false;
                    }
                }
                if self.culling_filter_passed_only && self.lib_left_tab == LibraryLeftTab::AssistedCulling {
                    let focus_ok = p.culling_focus_score.unwrap_or(0.0) >= self.culling_eye_focus_thresh;
                    let eyes_ok = p
                        .detected_faces
                        .first()
                        .map(|f| f.eyes_open_score >= self.culling_eyes_open_thresh)
                        .unwrap_or(true);
                    if !(focus_ok && eyes_ok) {
                        return false;
                    }
                }
                if p.rating < self.filter_min_rating {
                    return false;
                }
                if self.filter_only_picks && p.flag != PickFlag::Picked {
                    return false;
                }
                if self.filter_unflagged_only && p.flag != PickFlag::Unflagged {
                    return false;
                }
                if let Some(col_lbl) = self.filter_color_label {
                    if p.color_label != col_lbl {
                        return false;
                    }
                }
                if let Some(ref cam) = self.filter_camera {
                    let full_cam = format!("{} {}", p.camera_make, p.camera_model);
                    if &p.camera_model != cam && &full_cam != cam {
                        return false;
                    }
                }
                if !q.is_empty() {
                    let hay = format!(
                        "{} {} {} {} {} {} {}",
                        p.file_name,
                        p.folder_path,
                        p.camera_make,
                        p.camera_model,
                        p.title,
                        p.caption,
                        p.keywords.join(" ")
                    )
                    .to_lowercase();
                    if !hay.contains(&q) {
                        return false;
                    }
                }
                true
            })
            .collect();

        matched.sort_by(|a, b| {
            let ord = match self.lib_sort_mode {
                LibrarySortMode::CaptureTime => a
                    .capture_time
                    .cmp(&b.capture_time)
                    .then_with(|| a.file_name.cmp(&b.file_name)),
                LibrarySortMode::ImportOrder => a.id.cmp(&b.id),
                LibrarySortMode::Rating => a.rating.cmp(&b.rating).then_with(|| a.id.cmp(&b.id)),
                LibrarySortMode::PickFlag => a
                    .flag
                    .sort_rank()
                    .cmp(&b.flag.sort_rank())
                    .then_with(|| a.id.cmp(&b.id)),
                LibrarySortMode::FileName => a.file_name.cmp(&b.file_name),
                LibrarySortMode::AspectRatio => {
                    let ra = a.width as f32 / (a.height.max(1) as f32);
                    let rb = b.width as f32 / (b.height.max(1) as f32);
                    ra.partial_cmp(&rb).unwrap_or(std::cmp::Ordering::Equal)
                }
            };
            if self.lib_sort_ascending {
                ord
            } else {
                ord.reverse()
            }
        });

        matched.into_iter().map(|p| p.id).collect()
    }

    pub fn unique_folders(&self) -> Vec<(String, usize)> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for p in &self.photos {
            *counts.entry(p.folder_path.clone()).or_insert(0) += 1;
        }
        let mut out: Vec<(String, usize)> = counts.into_iter().collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    pub fn unique_cameras(&self) -> Vec<String> {
        let mut set: HashSet<String> = HashSet::new();
        for p in &self.photos {
            if !p.camera_model.trim().is_empty() {
                set.insert(p.camera_model.clone());
            }
        }
        let mut out: Vec<String> = set.into_iter().collect();
        out.sort();
        out
    }

    pub fn unique_keywords(&self) -> Vec<(String, usize)> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for p in &self.photos {
            for kw in &p.keywords {
                let k = kw.trim();
                if !k.is_empty() {
                    *counts.entry(k.to_string()).or_insert(0) += 1;
                }
            }
        }
        let mut out: Vec<(String, usize)> = counts.into_iter().collect();
        out.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
        out
    }

    // ========================================================================
    // Two-Tier Bounded RAW & Texture Rendering Cache
    // ========================================================================
    pub fn invalidate_photo_render_caches(&mut self, photo_id: i64) {
        self.rendered_thumb_cache.remove(&photo_id);
        self.rendered_after_cache.retain(|&(id, _), _| id != photo_id);
        self.rendered_before_cache.retain(|&(id, _), _| id != photo_id);
        self.nav_hover_preview_cache = None;
    }

    pub fn ensure_thumb_raw(&mut self, photo_id: i64) -> Option<Arc<RawSourceImage>> {
        if !self.thumb_raw_cache.contains_key(&photo_id) {
            let photo = self.photos.iter().find(|p| p.id == photo_id)?.clone();
            let raw = if let Some(ref p_str) = photo.real_file_path {
                let path = Path::new(p_str);
                if path.exists() {
                    decode_real_photo_file(path, 360, 360)
                } else {
                    None
                }
            } else {
                self.catalog_db.load_raw_blob(photo_id)
            }
            .unwrap_or_else(|| {
                RawSourceImage::generate_sample_raw((photo_id.max(0) as usize) % 6, 320, 240)
            });

            if self.thumb_raw_cache.len() >= 256 {
                if let Some(&evict_id) = self.thumb_raw_cache.keys().next() {
                    self.thumb_raw_cache.remove(&evict_id);
                }
            }
            self.thumb_raw_cache.insert(photo_id, Arc::new(raw));
        }
        self.thumb_raw_cache.get(&photo_id).cloned()
    }

    pub fn ensure_loupe_raw(&mut self, photo_id: i64) -> Option<Arc<RawSourceImage>> {
        let is_active = photo_id == self.active_photo_id;
        // In Develop module or when active photo is inspected in Loupe, load at
        // FULL NATIVE CAMERA RESOLUTION (target_dim = 0, 6000x4000 / 24MP-60MP) so the canvas
        // has 1:1 sensor pixel fidelity, razor-sharp edges, and zero blur!
        let needs_full_res = is_active;

        let should_reload = if let Some(existing) = self.raw_cache.get(&photo_id) {
            needs_full_res && existing.width < existing.native_width
        } else {
            false
        };

        if should_reload {
            self.raw_cache.remove(&photo_id);
            self.rendered_after_cache.retain(|&(id, _), _| id != photo_id);
            self.rendered_before_cache.retain(|&(id, _), _| id != photo_id);
        }

        if !self.raw_cache.contains_key(&photo_id) {
            let photo = self.photos.iter().find(|p| p.id == photo_id)?.clone();
            let target_dim = if needs_full_res { 0 } else { 2560 };
            let raw = if let Some(ref p_str) = photo.real_file_path {
                let path = Path::new(p_str);
                if path.exists() {
                    decode_real_photo_file(path, target_dim, target_dim)
                } else {
                    None
                }
            } else {
                self.catalog_db.load_raw_blob(photo_id)
            }
            .unwrap_or_else(|| {
                RawSourceImage::generate_sample_raw((photo_id.max(0) as usize) % 6, 960, 640)
            });

            // Bounded 14-entry LRU eviction so RAM stays crisp and low even in 500-photo catalogs!
            while self.raw_cache_lru.len() >= 14 {
                if let Some(old_id) = self.raw_cache_lru.pop_front() {
                    self.raw_cache.remove(&old_id);
                }
            }
            self.raw_cache_lru.push_back(photo_id);
            self.raw_cache.insert(photo_id, Arc::new(raw));
        } else {
            self.raw_cache_lru.retain(|&x| x != photo_id);
            self.raw_cache_lru.push_back(photo_id);
        }
        self.raw_cache.get(&photo_id).cloned()
    }

    pub fn get_thumb_texture(&mut self, ctx: &egui::Context, photo_id: i64) -> Option<TextureHandle> {
        if let Some(tex) = self.rendered_thumb_cache.get(&photo_id) {
            return Some(tex.clone());
        }
        let params = self.photos.iter().find(|p| p.id == photo_id)?.params.clone();
        let raw = self.ensure_thumb_raw(photo_id)?;
        let frame = render_develop_pipeline(&raw, &params, RenderOptions::default());
        let img = egui::ColorImage::from_rgba_unmultiplied(
            [frame.width as usize, frame.height as usize],
            &frame.rgba,
        );
        let tex = ctx.load_texture(
            format!("thumb_tex_{}", photo_id),
            img,
            egui::TextureOptions::LINEAR,
        );
        if self.rendered_thumb_cache.len() >= 320 {
            self.rendered_thumb_cache.clear();
        }
        self.rendered_thumb_cache.insert(photo_id, tex.clone());
        Some(tex)
    }

    pub fn build_active_render_options(&self, apply_crop: bool) -> RenderOptions {
        let is_dev = self.active_module == WorkspaceModule::Develop;
        let mask_overlay = if is_dev && self.develop_tool == DevelopTool::Masking && self.show_mask_overlay {
            self.selected_mask_idx
        } else {
            None
        };
        RenderOptions {
            show_shadow_clipping: is_dev && self.show_shadow_clipping,
            show_highlight_clipping: is_dev && self.show_highlight_clipping,
            soft_proofing: is_dev && self.soft_proofing,
            soft_proof_profile: self.soft_proof_profile,
            soft_proof_perceptual: self.soft_proof_perceptual,
            simulate_paper_ink: is_dev && self.soft_proofing && self.simulate_paper_ink,
            gamut_warning: is_dev && self.soft_proofing && self.gamut_warning,
            active_mask_overlay: mask_overlay,
            apply_crop,
            show_sharpen_mask_preview: is_dev && self.sharpen_mask_preview_active,
            is_draft_interactive: false,
        }
    }

    pub fn params_with_panel_bypass(&self, base: &DevelopParams) -> DevelopParams {
        let Some(panel) = self.bypassed_panel else {
            return base.clone();
        };
        let mut p = base.clone();
        let def = DevelopParams::default();
        match panel {
            "Basic" => {
                p.exposure_2012 = def.exposure_2012;
                p.contrast_2012 = def.contrast_2012;
                p.highlights_2012 = def.highlights_2012;
                p.shadows_2012 = def.shadows_2012;
                p.whites_2012 = def.whites_2012;
                p.blacks_2012 = def.blacks_2012;
                p.texture = def.texture;
                p.clarity_2012 = def.clarity_2012;
                p.dehaze = def.dehaze;
                p.vibrance = def.vibrance;
                p.saturation = def.saturation;
            }
            "Tone Curve" => {
                p.parametric_shadows = 0.0;
                p.parametric_darks = 0.0;
                p.parametric_lights = 0.0;
                p.parametric_highlights = 0.0;
                p.tone_curve_pv2012 = def.tone_curve_pv2012;
                p.tone_curve_pv2012_red = def.tone_curve_pv2012_red;
                p.tone_curve_pv2012_green = def.tone_curve_pv2012_green;
                p.tone_curve_pv2012_blue = def.tone_curve_pv2012_blue;
            }
            "Color Mixer" => {
                p.hue_adjustment = [0.0; 8];
                p.saturation_adjustment = [0.0; 8];
                p.luminance_adjustment = [0.0; 8];
                p.point_colors.clear();
            }
            "Color Grading" => {
                p.split_toning_shadow_saturation = 0.0;
                p.split_toning_highlight_saturation = 0.0;
                p.color_grade_midtone_sat = 0.0;
                p.color_grade_global_sat = 0.0;
                p.color_grade_shadow_lum = 0.0;
                p.color_grade_midtone_lum = 0.0;
                p.color_grade_highlight_lum = 0.0;
                p.color_grade_global_lum = 0.0;
            }
            "Detail" => {
                p.sharpness = 0.0;
                p.luminance_smoothing = 0.0;
                p.color_noise_reduction = 0.0;
                p.ai_denoise_amount = 0.0;
            }
            "Lens Blur" => p.lens_blur_active = false,
            "Lens Corrections" => {
                p.lens_profile_enable = false;
                p.lens_manual_distortion_amount = 0.0;
                p.vignette_amount = 0.0;
                p.defringe_purple_amount = 0.0;
                p.defringe_green_amount = 0.0;
            }
            "Transform" => {
                p.perspective_upright = crate::params::UprightMode::Off;
                p.perspective_vertical = 0.0;
                p.perspective_horizontal = 0.0;
                p.perspective_rotate = 0.0;
                p.perspective_scale = 100.0;
                p.perspective_aspect = 0.0;
                p.perspective_x = 0.0;
                p.perspective_y = 0.0;
            }
            "Effects" => {
                p.post_crop_vignette_amount = 0.0;
                p.grain_amount = 0.0;
            }
            "Calibration" => {
                p.shadow_tint = 0.0;
                p.red_hue = 0.0;
                p.red_saturation = 0.0;
                p.green_hue = 0.0;
                p.green_saturation = 0.0;
                p.blue_hue = 0.0;
                p.blue_saturation = 0.0;
            }
            _ => {}
        }
        p
    }

    pub fn get_photo_texture(
        &mut self,
        ctx: &egui::Context,
        photo_id: i64,
        apply_crop: bool,
    ) -> Option<TextureHandle> {
        // Fast path: Library Grid & Web/Print thumbnails use the dedicated 360px thumb cache!
        let is_grid_thumb = apply_crop
            && ((self.active_module == WorkspaceModule::Library
                && self.library_view_mode == LibraryViewMode::Grid)
                || self.active_module == WorkspaceModule::Web);
        if is_grid_thumb {
            return self.get_thumb_texture(ctx, photo_id);
        }

        let is_pointer_down = ctx.input(|i| i.pointer.primary_down());
        let is_active_dev = photo_id == self.active_photo_id && self.active_module == WorkspaceModule::Develop;
        let is_canvas_interaction = self.zoom_drag_start.is_some() || self.loupe_zoom > 1.02;
        let is_active_edit = self
            .last_edit_instant
            .map(|t| t.elapsed() < Duration::from_millis(220))
            .unwrap_or(false);
        let use_draft = is_active_dev
            && is_pointer_down
            && self.zoom_drag_start.is_none()
            && (is_active_edit || (ctx.input(|i| i.pointer.is_decidedly_dragging()) && !is_canvas_interaction));

        let mut opts = if photo_id == self.active_photo_id {
            self.build_active_render_options(apply_crop)
        } else {
            RenderOptions {
                apply_crop,
                ..RenderOptions::default()
            }
        };
        opts.is_draft_interactive = use_draft;

        let has_live_override = photo_id == self.active_photo_id
            && (self.bypassed_panel.is_some()
                || self.hovered_preset_preview.is_some()
                || self.hovered_history_preview.is_some());

        // Ensure loupe raw buffer is loaded and matches current zoom demand (Fit 4K vs 1:1 Native 6K)
        let _ = self.ensure_loupe_raw(photo_id);

        let cache_key = (photo_id, apply_crop);

        if !has_live_override {
            if let Some((tex, _, cached_opts)) = self.rendered_after_cache.get(&cache_key) {
                // When in Crop Mode with apply_crop=false, the uncropped texture is ALWAYS byte-identical!
                let is_crop_tool_active = self.active_module == WorkspaceModule::Develop
                    && self.develop_tool == DevelopTool::Crop;
                if is_crop_tool_active && !apply_crop {
                    return Some(tex.clone());
                }
                if *cached_opts == opts {
                    return Some(tex.clone());
                }
            }
        }

        let base_params = self.photos.iter().find(|p| p.id == photo_id)?.params.clone();
        let eff_params = if photo_id == self.active_photo_id {
            if let Some((_, ref hov_p)) = self.hovered_preset_preview {
                hov_p.clone()
            } else if let Some((_, ref hist_p)) = self.hovered_history_preview {
                hist_p.clone()
            } else {
                self.params_with_panel_bypass(&base_params)
            }
        } else {
            base_params
        };

        let raw = self.ensure_loupe_raw(photo_id)?;
        let frame = render_develop_pipeline(&raw, &eff_params, opts);
        let img = egui::ColorImage::from_rgba_unmultiplied(
            [frame.width as usize, frame.height as usize],
            &frame.rgba,
        );
        let tex = ctx.load_texture(
            format!("loupe_after_{}_{}", photo_id, apply_crop),
            img,
            egui::TextureOptions::LINEAR,
        );
        if !has_live_override {
            if self.rendered_after_cache.len() >= 24 {
                self.rendered_after_cache.clear();
            }
            self.rendered_after_cache
                .insert(cache_key, (tex.clone(), frame, opts));
        }
        Some(tex)
    }

    pub fn get_before_texture(&mut self, ctx: &egui::Context, photo_id: i64) -> Option<TextureHandle> {
        let is_dev_crop = self.active_module == WorkspaceModule::Develop
            && self.develop_tool == DevelopTool::Crop;
        let apply_crop = !is_dev_crop;
        let cache_key = (photo_id, apply_crop);
        let _ = self.ensure_loupe_raw(photo_id);
        if let Some(tex) = self.rendered_before_cache.get(&cache_key) {
            return Some(tex.clone());
        }
        let before_params = self
            .photos
            .iter()
            .find(|p| p.id == photo_id)?
            .before_params();
        let raw = self.ensure_loupe_raw(photo_id)?;
        let mut opts = RenderOptions::default();
        opts.apply_crop = apply_crop;
        let frame = render_develop_pipeline(&raw, &before_params, opts);
        let img = egui::ColorImage::from_rgba_unmultiplied(
            [frame.width as usize, frame.height as usize],
            &frame.rgba,
        );
        let tex = ctx.load_texture(
            format!("loupe_before_{}_{}", photo_id, apply_crop),
            img,
            egui::TextureOptions::LINEAR,
        );
        self.rendered_before_cache.insert(cache_key, tex.clone());
        Some(tex)
    }

    pub fn get_navigator_preview_texture(&mut self, ctx: &egui::Context) -> Option<TextureHandle> {
        if self.photos.is_empty() {
            return None;
        }
        let pid = self.active_photo_id;
        if let Some((ref name, ref hov_params)) = self.hovered_preset_preview {
            let key = format!("{}:{}", pid, name);
            if let Some((ref k, ref tex)) = self.nav_hover_preview_cache {
                if k == &key {
                    return Some(tex.clone());
                }
            }
            let params_clone = hov_params.clone();
            let raw = self.ensure_thumb_raw(pid)?;
            let frame = render_develop_pipeline(&raw, &params_clone, RenderOptions::default());
            let img = egui::ColorImage::from_rgba_unmultiplied(
                [frame.width as usize, frame.height as usize],
                &frame.rgba,
            );
            let tex = ctx.load_texture("nav_hover_preview", img, egui::TextureOptions::LINEAR);
            self.nav_hover_preview_cache = Some((key, tex.clone()));
            return Some(tex);
        }
        self.get_thumb_texture(ctx, pid)
    }

    pub fn active_rendered_frame(&self) -> Option<&RenderedFrame> {
        let is_crop = self.active_module == WorkspaceModule::Develop
            && self.develop_tool == DevelopTool::Crop;
        let apply_crop = !is_crop;
        self.rendered_after_cache
            .get(&(self.active_photo_id, apply_crop))
            .or_else(|| self.rendered_after_cache.get(&(self.active_photo_id, !apply_crop)))
            .map(|(_, f, _)| f)
    }

    pub fn get_watermark_graphic_texture(
        &mut self,
        ctx: &egui::Context,
        path_str: &str,
    ) -> Option<(TextureHandle, (u32, u32))> {
        if let Some((ref cached_path, ref tex, size)) = self.watermark_graphic_texture {
            if cached_path == path_str {
                return Some((tex.clone(), size));
            }
        }

        let p = std::path::Path::new(path_str);
        if !p.exists() {
            return None;
        }

        let Ok(img) = image::open(p) else {
            return None;
        };
        let rgba = img.to_rgba8();
        let (w, h) = (rgba.width(), rgba.height());
        if w == 0 || h == 0 {
            return None;
        }

        let color_img = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
        let tex = ctx.load_texture("wm_graphic_preview", color_img, egui::TextureOptions::LINEAR);
        self.watermark_graphic_texture = Some((path_str.to_string(), tex.clone(), (w, h)));
        Some((tex, (w, h)))
    }

    pub fn get_app_icon_texture(&mut self, ctx: &egui::Context) -> Option<TextureHandle> {
        if let Some(ref tex) = self.app_icon_texture {
            return Some(tex.clone());
        }
        if let Ok(img) = image::load_from_memory(include_bytes!("../../assets/icon.png")) {
            let rgba = img.to_rgba8();
            let (w, h) = (rgba.width(), rgba.height());
            if w > 0 && h > 0 {
                let color_img = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
                let tex = ctx.load_texture("about_app_icon", color_img, egui::TextureOptions::LINEAR);
                self.app_icon_texture = Some(tex.clone());
                return Some(tex);
            }
        }
        None
    }

    // ========================================================================
    // Non-Destructive History, Coalesced Dragging & Debounced SQLite Saves
    // ========================================================================
    pub fn commit_params_change(&mut self, step_name: &str, new_params: DevelopParams) {
        if self.photos.is_empty() {
            return;
        }
        let pid = self.active_photo_id;
        let now = Instant::now();
        let coalesce = self
            .last_history_step_meta
            .as_ref()
            .map(|(last_id, last_lbl, ts)| {
                *last_id == pid && last_lbl == step_name && ts.elapsed() < Duration::from_millis(650)
            })
            .unwrap_or(false);

        {
            let photo = self.active_photo_mut();
            photo.params = new_params.clone();
            if photo.history.is_empty() {
                photo.history.push(HistoryStep {
                    name: "Import RAW".into(),
                    value_summary: String::new(),
                    timestamp: "Initial".into(),
                    params: DevelopParams::default(),
                });
                photo.history_index = 0;
            }
            // Truncate any undone forward branch before appending/updating!
            if photo.history_index + 1 < photo.history.len() {
                photo.history.truncate(photo.history_index + 1);
            }
            if coalesce && photo.history.len() > 1 {
                if let Some(last) = photo.history.last_mut() {
                    last.params = new_params.clone();
                }
            } else {
                photo.history.push(HistoryStep {
                    name: step_name.to_string(),
                    value_summary: String::new(),
                    timestamp: "Edited".into(),
                    params: new_params.clone(),
                });
                photo.history_index = photo.history.len() - 1;
            }
        }

        self.previous_edited_params = Some(new_params.clone());
        self.last_history_step_meta = Some((pid, step_name.to_string(), now));
        self.dirty_photo_ids.insert(pid);
        self.last_edit_instant = Some(now);
        let is_only_crop = step_name.contains("Crop") || step_name.contains("Straighten");
        if is_only_crop {
            self.rendered_after_cache.retain(|&(id, apply_crop), _| id != pid || !apply_crop);
            self.rendered_before_cache.retain(|&(id, apply_crop), _| id != pid || !apply_crop);
        } else {
            self.rendered_after_cache.retain(|&(id, _), _| id != pid);
            self.rendered_before_cache.retain(|&(id, _), _| id != pid);
        }
        self.nav_hover_preview_cache = None;
        if !self.was_pointer_down {
            self.rendered_thumb_cache.remove(&pid);
        }

        // Auto-Sync across multi-selection when enabled in Develop!
        if self.auto_sync_enabled && self.selected_photo_ids.len() > 1 {
            let sync_mask = CopySettingsMask::default();
            let others: Vec<i64> = self
                .selected_photo_ids
                .iter()
                .copied()
                .filter(|&id| id != pid)
                .collect();
            for oid in others {
                if let Some(op) = self.photos.iter_mut().find(|p| p.id == oid) {
                    apply_settings_with_mask(&mut op.params, &new_params, &sync_mask);
                    self.dirty_photo_ids.insert(oid);
                }
                self.invalidate_photo_render_caches(oid);
            }
        }
    }

    pub fn persist_active_photo(&mut self) {
        if self.photos.is_empty() {
            return;
        }
        let pid = self.active_photo_id;
        if let Some(p) = self.photos.iter().find(|p| p.id == pid) {
            self.catalog_db.save_photo(p);
        }
        self.dirty_photo_ids.remove(&pid);
        self.invalidate_photo_render_caches(pid);
    }

    pub fn flush_dirty_photos(&mut self) {
        if self.dirty_photo_ids.is_empty() {
            return;
        }
        let ids: Vec<i64> = self.dirty_photo_ids.drain().collect();
        for id in ids {
            if let Some(p) = self.photos.iter().find(|x| x.id == id) {
                self.catalog_db.save_photo(p);
            }
        }
        self.last_edit_instant = None;
    }

    pub fn trigger_update_check(&mut self, open_modal: bool) {
        if open_modal {
            self.show_update_modal = true;
        }
        if matches!(
            self.update_state,
            UpdaterUiState::Checking | UpdaterUiState::Installing(_)
        ) {
            return;
        }
        self.update_state = UpdaterUiState::Checking;
        let (tx, rx) = std::sync::mpsc::channel();
        self.update_rx = Some(rx);
        std::thread::spawn(move || {
            let next = match check_latest_release() {
                Ok(UpdateCheckOutcome::Available(rel)) => UpdaterUiState::Available(rel),
                Ok(UpdateCheckOutcome::UpToDate { current, latest_tag }) => {
                    UpdaterUiState::UpToDate { current, latest_tag }
                }
                Err(err) => UpdaterUiState::Error(err),
            };
            let _ = tx.send(next);
        });
    }

    pub fn start_update_install(&mut self, release: RemoteRelease) {
        if matches!(self.update_state, UpdaterUiState::Installing(_)) {
            return;
        }
        self.show_update_modal = true;
        self.update_state =
            UpdaterUiState::Installing(format!("Downloading LightRaw {}...", release.tag_name));
        let (tx, rx) = std::sync::mpsc::channel();
        self.update_rx = Some(rx);
        std::thread::spawn(move || {
            let next = match download_and_install_release(&release) {
                Ok(ApplyUpdateOutcome::Relaunching(msg)) => {
                    let _ = tx.send(UpdaterUiState::Installing(msg));
                    std::thread::sleep(Duration::from_millis(600));
                    std::process::exit(0);
                }
                Ok(ApplyUpdateOutcome::ManualStep(msg)) => UpdaterUiState::Notice(msg),
                Err(err) => UpdaterUiState::Error(err),
            };
            let _ = tx.send(next);
        });
    }

    pub fn poll_updater(&mut self, ctx: &egui::Context) {
        if !self.auto_update_checked_startup {
            self.auto_update_checked_startup = true;
            self.trigger_update_check(false);
        }
        if let Some(ref rx) = self.update_rx {
            match rx.try_recv() {
                Ok(next) => {
                    if let UpdaterUiState::Available(ref rel) = next {
                        self.status_message = format!("Update available: LightRaw {}", rel.tag_name);
                    }
                    self.update_state = next;
                    self.update_rx = None;
                    ctx.request_repaint();
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(Duration::from_millis(250));
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.update_rx = None;
                }
            }
        }
    }

    pub fn undo_active_photo(&mut self) {
        if self.photos.is_empty() {
            return;
        }
        let pid = self.active_photo_id;
        let stepped = {
            let p = self.active_photo_mut();
            if p.history_index > 0 && !p.history.is_empty() {
                p.history_index -= 1;
                p.params = p.history[p.history_index].params.clone();
                Some(p.history[p.history_index].name.clone())
            } else {
                None
            }
        };
        if let Some(name) = stepped {
            self.persist_active_photo();
            self.invalidate_photo_render_caches(pid);
            self.status_message = format!("Undo -> {}", name);
        }
    }

    pub fn redo_active_photo(&mut self) {
        if self.photos.is_empty() {
            return;
        }
        let pid = self.active_photo_id;
        let stepped = {
            let p = self.active_photo_mut();
            if p.history_index + 1 < p.history.len() {
                p.history_index += 1;
                p.params = p.history[p.history_index].params.clone();
                Some(p.history[p.history_index].name.clone())
            } else {
                None
            }
        };
        if let Some(name) = stepped {
            self.persist_active_photo();
            self.invalidate_photo_render_caches(pid);
            self.status_message = format!("Redo -> {}", name);
        }
    }
}

impl LightRawApp {
    // ========================================================================
    // Presets, Amount Scaling, Copy / Paste / Sync & Sidecar XMP
    // ========================================================================
    pub fn compute_preset_applied_params(
        base: &DevelopParams,
        preset: &DevelopPreset,
    ) -> DevelopParams {
        let mut target = base.clone();
        let eff_mask = preset
            .mask
            .clone()
            .unwrap_or_else(CopySettingsMask::default);
        apply_settings_with_mask(&mut target, &preset.params, &eff_mask);
        target.applied_preset_name = preset.name.clone();
        target.preset_base_snapshot = Some(Box::new(base.clone()));
        let mut snap_target = target.clone();
        snap_target.preset_base_snapshot = None;
        snap_target.preset_target_snapshot = None;
        target.preset_target_snapshot = Some(Box::new(snap_target));
        target
    }

    pub fn apply_preset_to_selection(&mut self, preset_name: &str) {
        let Some(preset) = self
            .presets
            .iter()
            .find(|p| p.name == preset_name)
            .cloned()
        else {
            return;
        };
        let ids = if self.selected_photo_ids.is_empty() {
            vec![self.active_photo_id]
        } else {
            self.selected_photo_ids.clone()
        };

        for id in &ids {
            if let Some(photo) = self.photos.iter_mut().find(|p| p.id == *id) {
                let next_params = Self::compute_preset_applied_params(&photo.params, &preset);
                if photo.history_index + 1 < photo.history.len() {
                    photo.history.truncate(photo.history_index + 1);
                }
                photo.params = next_params.clone();
                photo.history.push(HistoryStep {
                    name: format!("Preset: {}", preset.name),
                    value_summary: String::new(),
                    timestamp: "Preset".into(),
                    params: next_params,
                });
                photo.history_index = photo.history.len() - 1;
                self.catalog_db.save_photo(photo);
            }
            self.invalidate_photo_render_caches(*id);
        }

        self.preset_amount = 100.0;
        self.status_message = format!(
            "Applied preset '{}' to {} photo(s).",
            preset.name,
            ids.len()
        );
    }

    pub fn set_preset_amount(&mut self, amount_pct: f32) {
        self.preset_amount = amount_pct.clamp(0.0, 200.0);
        if self.photos.is_empty() {
            return;
        }
        let pid = self.active_photo_id;
        let interpolated_opt = {
            let cur = &self.active_photo().params;
            if let (Some(ref base), Some(ref target)) =
                (&cur.preset_base_snapshot, &cur.preset_target_snapshot)
            {
                let mut scaled = interpolate_preset_amount(base, target, self.preset_amount);
                scaled.applied_preset_name = cur.applied_preset_name.clone();
                scaled.preset_base_snapshot = cur.preset_base_snapshot.clone();
                scaled.preset_target_snapshot = cur.preset_target_snapshot.clone();
                Some(scaled)
            } else {
                None
            }
        };
        if let Some(scaled) = interpolated_opt {
            self.active_photo_mut().params = scaled;
            self.dirty_photo_ids.insert(pid);
            self.last_edit_instant = Some(Instant::now());
            self.invalidate_photo_render_caches(pid);
        }
    }

    pub fn import_xmp_presets_dialog(&mut self) {
        let Some(files) = rfd::FileDialog::new()
            .set_title("Import Adobe Camera Raw / Lightroom Presets (.xmp)")
            .add_filter("XMP Preset", &["xmp", "XMP", "lrtemplate"])
            .pick_files()
        else {
            return;
        };
        let mut imported = 0usize;
        for file_path in files {
            let Ok(xml) = std::fs::read_to_string(&file_path) else {
                continue;
            };
            let fallback = file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Imported Preset");
            if let Some(mut pr) = parse_xmp_preset(&xml, fallback) {
                let grp_dir = self
                    .workspace_dir
                    .join("Presets")
                    .join(pr.category.replace(['/', '\\', ':'], "_"));
                let _ = std::fs::create_dir_all(&grp_dir);
                let dst_xmp = grp_dir.join(format!(
                    "{}.xmp",
                    pr.name.replace(['/', '\\', ':'], "_")
                ));
                let _ = std::fs::write(&dst_xmp, export_preset_to_xmp(&pr));
                pr.xmp_file_path = Some(dst_xmp.to_string_lossy().to_string());
                self.expanded_preset_groups.insert(pr.category.clone());
                self.catalog_db.save_preset(&pr);
                if let Some(existing) = self.presets.iter_mut().find(|x| {
                    x.name.eq_ignore_ascii_case(&pr.name)
                        && x.category.eq_ignore_ascii_case(&pr.category)
                }) {
                    *existing = pr;
                } else {
                    self.presets.push(pr);
                }
                imported += 1;
            }
        }
        self.status_message = format!("Imported {} .xmp preset(s).", imported);
    }

    pub fn export_preset_dialog(&mut self, preset_id: &str) {
        let Some(preset) = self.presets.iter().find(|p| p.id == preset_id).cloned() else {
            return;
        };
        let default_name = format!("{}.xmp", preset.name.replace(['/', '\\', ':'], "_"));
        if let Some(dest) = rfd::FileDialog::new()
            .set_title("Export Develop Preset (.xmp)")
            .set_file_name(&default_name)
            .add_filter("Adobe XMP Preset", &["xmp"])
            .save_file()
        {
            let xml = export_preset_to_xmp(&preset);
            if std::fs::write(&dest, xml).is_ok() {
                self.status_message =
                    format!("Exported preset '{}' to {}", preset.name, dest.display());
            }
        }
    }

    pub fn export_preset_group_dialog(&mut self, group_name: &str) {
        let Some(dir) = rfd::FileDialog::new()
            .set_title(&format!("Export Preset Group '{}'", group_name))
            .pick_folder()
        else {
            return;
        };
        let out_dir = dir.join(group_name.replace(['/', '\\', ':'], "_"));
        let _ = std::fs::create_dir_all(&out_dir);
        let mut count = 0usize;
        for pr in self.presets.iter().filter(|p| p.category == group_name) {
            let p = out_dir.join(format!("{}.xmp", pr.name.replace(['/', '\\', ':'], "_")));
            if std::fs::write(p, export_preset_to_xmp(pr)).is_ok() {
                count += 1;
            }
        }
        self.status_message = format!(
            "Exported {} preset(s) from '{}' to {}",
            count,
            group_name,
            out_dir.display()
        );
    }

    pub fn paste_clipboard_settings(&mut self) {
        let Some(src_params) = self.clipboard_params.clone() else {
            self.status_message = "Clipboard is empty — Copy settings first (⇧⌘C).".into();
            return;
        };
        let mask = self.clipboard_mask.clone();
        let ids = if self.selected_photo_ids.is_empty() {
            vec![self.active_photo_id]
        } else {
            self.selected_photo_ids.clone()
        };
        for id in &ids {
            if let Some(p) = self.photos.iter_mut().find(|x| x.id == *id) {
                apply_settings_with_mask(&mut p.params, &src_params, &mask);
                if p.history_index + 1 < p.history.len() {
                    p.history.truncate(p.history_index + 1);
                }
                let snap = p.params.clone();
                p.history.push(HistoryStep {
                    name: "Paste Settings".into(),
                    value_summary: String::new(),
                    timestamp: "Pasted".into(),
                    params: snap,
                });
                p.history_index = p.history.len() - 1;
                self.catalog_db.save_photo(p);
            }
            self.invalidate_photo_render_caches(*id);
        }
        self.status_message = format!("Pasted Develop settings to {} photo(s).", ids.len());
    }

    pub fn paste_from_previous_photo(&mut self) {
        let src_opt = self.previous_edited_params.clone().or_else(|| {
            let vis = self.filtered_photo_ids();
            let pos = vis.iter().position(|&x| x == self.active_photo_id)?;
            let prev_id = if pos > 0 { vis[pos - 1] } else { *vis.last()? };
            self.photos
                .iter()
                .find(|p| p.id == prev_id)
                .map(|p| p.params.clone())
        });
        let Some(src_params) = src_opt else {
            self.status_message = "No previous photo settings available.".into();
            return;
        };
        let mask = CopySettingsMask::default();
        let mut next = self.active_photo().params.clone();
        apply_settings_with_mask(&mut next, &src_params, &mask);
        self.commit_params_change("Paste From Previous", next);
        self.status_message = "Applied settings from previous photo.".into();
    }

    pub fn sync_selected_photos_with_mask(&mut self, mask: &CopySettingsMask) {
        if self.photos.is_empty() {
            return;
        }
        let src_params = self.active_photo().params.clone();
        let active_id = self.active_photo_id;
        let targets: Vec<i64> = self
            .selected_photo_ids
            .iter()
            .copied()
            .filter(|&id| id != active_id)
            .collect();
        if targets.is_empty() {
            self.status_message = "Select 2 or more photos to Synchronize Settings.".into();
            return;
        }
        for id in &targets {
            if let Some(p) = self.photos.iter_mut().find(|x| x.id == *id) {
                apply_settings_with_mask(&mut p.params, &src_params, mask);
                if p.history_index + 1 < p.history.len() {
                    p.history.truncate(p.history_index + 1);
                }
                let snap = p.params.clone();
                p.history.push(HistoryStep {
                    name: "Sync Settings".into(),
                    value_summary: String::new(),
                    timestamp: "Synced".into(),
                    params: snap,
                });
                p.history_index = p.history.len() - 1;
                self.catalog_db.save_photo(p);
            }
            self.invalidate_photo_render_caches(*id);
        }
        self.status_message = format!("Synchronized settings across {} photo(s).", targets.len());
    }

    pub fn sync_crop_to_library(&mut self, only_selected: bool) {
        if self.photos.is_empty() {
            return;
        }
        let src_params = self.active_photo().params.clone();
        let active_id = self.active_photo_id;
        let targets: Vec<i64> = self
            .photos
            .iter()
            .map(|p| p.id)
            .filter(|&id| id != active_id && (!only_selected || self.selected_photo_ids.contains(&id)))
            .collect();
        let count = targets.len();

        for id in targets {
            if let Some(photo) = self.photos.iter_mut().find(|p| p.id == id) {
                photo.params.crop_left = src_params.crop_left;
                photo.params.crop_top = src_params.crop_top;
                photo.params.crop_right = src_params.crop_right;
                photo.params.crop_bottom = src_params.crop_bottom;
                photo.params.crop_angle = src_params.crop_angle;
                photo.params.crop_aspect_preset = src_params.crop_aspect_preset;
                photo.params.crop_constrain_aspect_ratio = src_params.crop_constrain_aspect_ratio;
                photo.params.crop_aspect_flipped = src_params.crop_aspect_flipped;
                photo.params.crop_custom_ratio = src_params.crop_custom_ratio;

                if photo.history_index + 1 < photo.history.len() {
                    photo.history.truncate(photo.history_index + 1);
                }
                let snap = photo.params.clone();
                photo.history.push(HistoryStep {
                    name: "Sync Crop".into(),
                    value_summary: format!(
                        "{:.0}% x {:.0}%",
                        (snap.crop_right - snap.crop_left) * 100.0,
                        (snap.crop_bottom - snap.crop_top) * 100.0
                    ),
                    timestamp: "Synced".into(),
                    params: snap,
                });
                photo.history_index = photo.history.len() - 1;
                self.catalog_db.save_photo(photo);
            }
            self.dirty_photo_ids.insert(id);
            self.invalidate_photo_render_caches(id);
        }

        if only_selected {
            self.status_message = format!("Crop synchronized across {} selected photo(s).", count);
        } else {
            self.status_message = format!("Crop synchronized across all {} photo(s) in Library.", count);
        }
    }

    pub fn reset_selected_photos_settings(&mut self) {
        let ids = self.effective_target_ids();
        for id in &ids {
            if let Some(p) = self.photos.iter_mut().find(|x| x.id == *id) {
                let mut def = DevelopParams::default();
                if let Some(raw) = self.thumb_raw_cache.get(id).or_else(|| self.raw_cache.get(id)) {
                    def.temperature = raw.as_shot_temp;
                    def.tint = raw.as_shot_tint;
                }
                p.params = def.clone();
                p.history.push(HistoryStep {
                    name: "Reset Settings".into(),
                    value_summary: String::new(),
                    timestamp: "Reset".into(),
                    params: def,
                });
                p.history_index = p.history.len() - 1;
                self.catalog_db.save_photo(p);
            }
            self.invalidate_photo_render_caches(*id);
        }
        self.preset_amount = 100.0;
        self.status_message = format!("Reset Develop settings for {} photo(s).", ids.len());
    }

    pub fn apply_auto_tone_to_selection(&mut self) {
        let ids = self.effective_target_ids();
        for id in &ids {
            let raw_opt = self.ensure_thumb_raw(*id);
            if let Some(p) = self.photos.iter_mut().find(|x| x.id == *id) {
                p.params.auto_tone = true;
                if let Some(ref raw) = raw_opt {
                    let mut lums: Vec<f32> = raw
                        .rgb
                        .chunks_exact(3)
                        .step_by(4)
                        .map(|px| 0.2126 * px[0] + 0.7152 * px[1] + 0.0722 * px[2])
                        .collect();
                    if !lums.is_empty() {
                        lums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                        let mid = lums[lums.len() / 2].max(0.005);
                        let p98 = lums[(lums.len() * 98 / 100).min(lums.len() - 1)];
                        let p02 = lums[lums.len() * 2 / 100];
                        let target_mid = if raw.is_scene_referred { 0.16 } else { 0.42 };
                        p.params.exposure_2012 =
                            (target_mid / mid).log2().clamp(-1.5, 1.8);
                        p.params.highlights_2012 = if p98 > 0.75 { -38.0 } else { -12.0 };
                        p.params.shadows_2012 = if p02 < 0.04 { 28.0 } else { 14.0 };
                        p.params.whites_2012 = 12.0;
                        p.params.blacks_2012 = -8.0;
                        p.params.contrast_2012 = 8.0;
                        p.params.vibrance = 10.0;
                    }
                }
                let snap = p.params.clone();
                p.history.push(HistoryStep {
                    name: "Auto Tone".into(),
                    value_summary: String::new(),
                    timestamp: "Auto".into(),
                    params: snap,
                });
                p.history_index = p.history.len() - 1;
                self.catalog_db.save_photo(p);
            }
            self.invalidate_photo_render_caches(*id);
        }
        self.status_message = format!("Applied Auto Tone to {} photo(s).", ids.len());
    }

    pub fn apply_auto_wb_to_selection(&mut self) {
        let ids = self.effective_target_ids();
        for id in &ids {
            let wb_opt = self.ensure_thumb_raw(*id).map(|r| estimate_auto_wb(&r));
            if let (Some((temp, tint)), Some(p)) =
                (wb_opt, self.photos.iter_mut().find(|x| x.id == *id))
            {
                p.params.white_balance = crate::params::WhiteBalanceMode::Auto;
                p.params.temperature = temp;
                p.params.tint = tint;
                let snap = p.params.clone();
                p.history.push(HistoryStep {
                    name: format!("Auto WB ({:.0}K {:+.0})", temp, tint),
                    value_summary: String::new(),
                    timestamp: "Auto WB".into(),
                    params: snap,
                });
                p.history_index = p.history.len() - 1;
                self.catalog_db.save_photo(p);
            }
            self.invalidate_photo_render_caches(*id);
        }
        self.status_message = "Calculated scene Auto White Balance.".into();
    }

    pub fn auto_match_color_to_reference(&mut self) {
        if self.photos.is_empty() {
            return;
        }

        let ref_id = self
            .reference_photo_id
            .filter(|&id| id != self.active_photo_id)
            .or_else(|| {
                self.selected_photo_ids
                    .iter()
                    .copied()
                    .find(|&id| id != self.active_photo_id)
            })
            .or_else(|| {
                self.photos.iter().find(|p| p.id != self.active_photo_id).map(|p| p.id)
            });

        let Some(ref_id) = ref_id else {
            self.status_message = "Please select or set a Reference Photo first.".into();
            return;
        };

        self.reference_photo_id = Some(ref_id);

        let ref_raw = self.ensure_thumb_raw(ref_id);
        let active_raw = self.ensure_thumb_raw(self.active_photo_id);

        let (Some(ref_raw), Some(active_raw)) = (ref_raw, active_raw) else {
            self.status_message = "Could not decode reference or target image data for color matching.".into();
            return;
        };

        let ref_params = self.photos.iter().find(|p| p.id == ref_id).map(|p| p.params.clone());
        let ref_name = self
            .photos
            .iter()
            .find(|p| p.id == ref_id)
            .map(|p| p.file_name.clone())
            .unwrap_or_else(|| "Reference".into());

        let target_params = self.active_photo().params.clone();

        let matched = crate::ai::color_match::match_color_and_tone(
            &ref_raw,
            ref_params.as_ref(),
            &active_raw,
            &target_params,
        );

        let step_desc = format!("Auto Match Color to {}", ref_name);
        self.commit_params_change(&step_desc, matched);
        self.status_message = format!("⚡ Matched color, white balance & tone to reference: {}", ref_name);
    }

    pub fn save_xmp_sidecars_for_selection(&mut self) {
        let ids = if self.selected_photo_ids.is_empty() {
            vec![self.active_photo_id]
        } else {
            self.selected_photo_ids.clone()
        };
        let mut saved = 0usize;
        for id in ids {
            if let Some(photo) = self.photos.iter().find(|p| p.id == id) {
                if let Some(ref rp) = photo.real_file_path {
                    let xmp_path = Path::new(rp).with_extension("xmp");
                    let pr = DevelopPreset {
                        id: format!("sidecar_{}", photo.id),
                        name: photo.file_name.clone(),
                        category: "Sidecar".into(),
                        is_user: true,
                        is_favorite: false,
                        is_visible: true,
                        params: photo.params.clone(),
                        mask: None,
                        xmp_file_path: None,
                    };
                    if std::fs::write(&xmp_path, export_preset_to_xmp(&pr)).is_ok() {
                        saved += 1;
                    }
                }
            }
        }
        self.status_message = format!("Saved {} companion .xmp sidecar file(s) to disk.", saved);
    }

    pub fn read_xmp_sidecars_for_selection(&mut self) {
        let ids = if self.selected_photo_ids.is_empty() {
            vec![self.active_photo_id]
        } else {
            self.selected_photo_ids.clone()
        };
        let mut loaded = 0usize;
        for id in ids {
            let xmp_opt = self
                .photos
                .iter()
                .find(|p| p.id == id)
                .and_then(|p| p.real_file_path.as_ref())
                .map(|rp| Path::new(rp).with_extension("xmp"))
                .filter(|xp| xp.exists())
                .and_then(|xp| std::fs::read_to_string(xp).ok());
            if let Some(xml) = xmp_opt {
                if let Some(parsed) = parse_xmp_preset(&xml, "Sidecar") {
                    if let Some(p) = self.photos.iter_mut().find(|x| x.id == id) {
                        p.params = parsed.params.clone();
                        p.history.push(HistoryStep {
                            name: "Read Metadata from XMP".into(),
                            value_summary: String::new(),
                            timestamp: "XMP".into(),
                            params: parsed.params,
                        });
                        p.history_index = p.history.len() - 1;
                        self.catalog_db.save_photo(p);
                        loaded += 1;
                    }
                    self.invalidate_photo_render_caches(id);
                }
            }
        }
        self.status_message = format!("Read {} .xmp sidecar file(s) from disk.", loaded);
    }

    pub fn execute_sync_metadata(&mut self) {
        if self.photos.is_empty() {
            return;
        }
        let src = self.active_photo().clone();
        let targets: Vec<i64> = self
            .selected_photo_ids
            .iter()
            .copied()
            .filter(|&id| id != src.id)
            .collect();
        for id in &targets {
            if let Some(p) = self.photos.iter_mut().find(|x| x.id == *id) {
                if self.sync_meta_title_caption {
                    p.title = src.title.clone();
                    p.caption = src.caption.clone();
                }
                if self.sync_meta_creator_copyright {
                    p.creator = src.creator.clone();
                    p.copyright = src.copyright.clone();
                }
                if self.sync_meta_rating_label {
                    p.rating = src.rating;
                    p.flag = src.flag;
                    p.color_label = src.color_label;
                }
                if self.sync_meta_keywords {
                    p.keywords = src.keywords.clone();
                    p.person_tag = src.person_tag.clone();
                }
                if self.sync_meta_gps_location {
                    p.gps_lat = src.gps_lat;
                    p.gps_lon = src.gps_lon;
                    p.city = src.city.clone();
                    p.country = src.country.clone();
                }
                self.catalog_db.save_photo(p);
            }
        }
        self.status_message = format!("Synchronized metadata to {} photo(s).", targets.len());
    }

    // ========================================================================
    // Ratings, Flags, Rotation, Virtual Copy, Culling & Photo Merge
    // ========================================================================
    pub fn set_rating_on_selection(&mut self, rating: u8) {
        let ids = if self.selected_photo_ids.is_empty() {
            vec![self.active_photo_id]
        } else {
            self.selected_photo_ids.clone()
        };
        for id in &ids {
            if let Some(p) = self.photos.iter_mut().find(|x| x.id == *id) {
                p.rating = rating.min(5);
                self.catalog_db.save_photo(p);
            }
        }
        self.status_message = format!("Set rating {}★ on {} photo(s).", rating, ids.len());
    }

    pub fn set_flag_on_selection(&mut self, flag: PickFlag) {
        let ids = if self.selected_photo_ids.is_empty() {
            vec![self.active_photo_id]
        } else {
            self.selected_photo_ids.clone()
        };
        for id in &ids {
            if let Some(p) = self.photos.iter_mut().find(|x| x.id == *id) {
                p.flag = flag;
                self.catalog_db.save_photo(p);
            }
        }
        self.status_message = format!("Updated Pick Flag on {} photo(s).", ids.len());
    }

    pub fn set_color_label_on_selection(&mut self, label: ColorLabel) {
        let ids = if self.selected_photo_ids.is_empty() {
            vec![self.active_photo_id]
        } else {
            self.selected_photo_ids.clone()
        };
        for id in &ids {
            if let Some(p) = self.photos.iter_mut().find(|x| x.id == *id) {
                p.color_label = label;
                self.catalog_db.save_photo(p);
            }
        }
        self.status_message = format!("Set Color Label {} on {} photo(s).", label.name(), ids.len());
    }

    pub fn rotate_selection(&mut self, delta_deg: f32) {
        let ids = if self.selected_photo_ids.is_empty() {
            vec![self.active_photo_id]
        } else {
            self.selected_photo_ids.clone()
        };
        for id in &ids {
            if let Some(p) = self.photos.iter_mut().find(|x| x.id == *id) {
                p.params.rotation_degrees =
                    (p.params.rotation_degrees + delta_deg).rem_euclid(360.0);
                self.catalog_db.save_photo(p);
            }
            self.invalidate_photo_render_caches(*id);
        }
    }

    pub fn flip_selection(&mut self, horizontal: bool) {
        let ids = if self.selected_photo_ids.is_empty() {
            vec![self.active_photo_id]
        } else {
            self.selected_photo_ids.clone()
        };
        for id in &ids {
            if let Some(p) = self.photos.iter_mut().find(|x| x.id == *id) {
                if horizontal {
                    p.params.flip_horizontal = !p.params.flip_horizontal;
                } else {
                    p.params.flip_vertical = !p.params.flip_vertical;
                }
                self.catalog_db.save_photo(p);
            }
            self.invalidate_photo_render_caches(*id);
        }
    }

    pub fn apply_painter_spray_to_photo(&mut self, photo_id: i64) {
        match self.painter_mode {
            PainterMode::Rating => {
                let r = self.painter_rating;
                if let Some(p) = self.photos.iter_mut().find(|x| x.id == photo_id) {
                    if p.rating != r {
                        p.rating = r;
                        self.catalog_db.save_photo(p);
                    }
                }
            }
            PainterMode::Flag => {
                let f = self.painter_flag;
                if let Some(p) = self.photos.iter_mut().find(|x| x.id == photo_id) {
                    if p.flag != f {
                        p.flag = f;
                        self.catalog_db.save_photo(p);
                    }
                }
            }
            PainterMode::ColorLabel => {
                let c = self.painter_color.unwrap_or(ColorLabel::None);
                if let Some(p) = self.photos.iter_mut().find(|x| x.id == photo_id) {
                    if p.color_label != c {
                        p.color_label = c;
                        self.catalog_db.save_photo(p);
                    }
                }
            }
            PainterMode::Preset => {
                let pr_name = self.painter_preset.clone();
                if let Some(pr) = self.presets.iter().find(|x| x.name == pr_name).cloned() {
                    let already = self
                        .photos
                        .iter()
                        .find(|x| x.id == photo_id)
                        .map(|p| p.params.applied_preset_name.as_str() == pr_name.as_str())
                        .unwrap_or(false);
                    if !already {
                        if let Some(p) = self.photos.iter_mut().find(|x| x.id == photo_id) {
                            p.params = Self::compute_preset_applied_params(&p.params, &pr);
                            self.catalog_db.save_photo(p);
                        }
                        self.invalidate_photo_render_caches(photo_id);
                    }
                }
            }
        }
    }

    pub fn toggle_target_collection_for_selection(&mut self) {
        let target_cid = self.target_collection_id;
        let ids = if self.selected_photo_ids.is_empty() {
            vec![self.active_photo_id]
        } else {
            self.selected_photo_ids.clone()
        };
        if let Some(col) = self.collections.iter_mut().find(|c| c.id == target_cid && !c.is_smart) {
            let all_present = ids.iter().all(|id| col.photo_ids.contains(id));
            if all_present {
                col.photo_ids.retain(|id| !ids.contains(id));
                self.status_message =
                    format!("Removed {} photo(s) from '{}'.", ids.len(), col.name);
            } else {
                for id in &ids {
                    if !col.photo_ids.contains(id) {
                        col.photo_ids.push(*id);
                    }
                }
                self.status_message = format!("Added {} photo(s) to '{}'.", ids.len(), col.name);
            }
            let cloned = col.clone();
            self.catalog_db.save_collection(&cloned);
        }
    }

    pub fn create_virtual_copy(&mut self) {
        if self.photos.is_empty() {
            return;
        }
        let next_id = self.photos.iter().map(|p| p.id).max().unwrap_or(0) + 1;
        let mut copy = self.active_photo().clone();
        copy.id = next_id;
        copy.is_virtual_copy = true;
        let copy_num = self
            .photos
            .iter()
            .filter(|p| p.file_name == copy.file_name && p.is_virtual_copy)
            .count()
            + 1;
        copy.copy_name = format!("Copy {}", copy_num);
        self.catalog_db.save_photo(&copy);
        self.photos.push(copy);
        self.select_photo(next_id, false);
        self.status_message = format!("Created non-destructive Virtual Copy (Copy {}).", copy_num);
    }

    pub fn remove_selected_photos_from_catalog(&mut self) {
        if self.photos.is_empty() {
            return;
        }
        let ids = if self.selected_photo_ids.is_empty() {
            vec![self.active_photo_id]
        } else {
            self.selected_photo_ids.clone()
        };
        for id in &ids {
            self.catalog_db.delete_photo(*id);
            self.thumb_raw_cache.remove(id);
            self.raw_cache.remove(id);
            self.invalidate_photo_render_caches(*id);
        }
        self.photos.retain(|p| !ids.contains(&p.id));
        for col in &mut self.collections {
            col.photo_ids.retain(|id| !ids.contains(id));
            self.catalog_db.save_collection(col);
        }
        if let Some(first) = self.photos.first() {
            self.active_photo_id = first.id;
            self.selected_photo_ids = vec![first.id];
        } else {
            self.selected_photo_ids.clear();
        }
        self.status_message =
            format!("Removed {} photo(s) from catalog (files on disk preserved).", ids.len());
    }

    pub fn delete_rejected_photos_from_catalog(&mut self) {
        let rej_ids: Vec<i64> = self
            .photos
            .iter()
            .filter(|p| p.flag == PickFlag::Rejected)
            .map(|p| p.id)
            .collect();
        if rej_ids.is_empty() {
            self.status_message = "No Rejected (✕) photos in catalog.".into();
            return;
        }
        for id in &rej_ids {
            self.catalog_db.delete_photo(*id);
            self.invalidate_photo_render_caches(*id);
        }
        self.photos.retain(|p| !rej_ids.contains(&p.id));
        if let Some(first) = self.photos.first() {
            if !self.photos.iter().any(|p| p.id == self.active_photo_id) {
                self.active_photo_id = first.id;
                self.selected_photo_ids = vec![first.id];
            }
        } else {
            self.selected_photo_ids.clear();
        }
        self.status_message =
            format!("Removed {} Rejected photo(s) from catalog.", rej_ids.len());
    }

    pub fn reveal_active_photo_in_finder(&mut self) {
        if let Some(ref p) = self.active_photo().real_file_path {
            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open").arg("-R").arg(p).spawn();
            }
            self.status_message = format!("Revealed in Finder: {}", p);
        }
    }

    pub fn run_assisted_culling_analysis(&mut self) {
        let ids = self.filtered_photo_ids();
        if ids.is_empty() {
            self.status_message = "Assisted Culling: No photos in current view.".into();
            return;
        }
        let mut passed = 0usize;
        let mut rejected_blinks = 0usize;
        for id in &ids {
            let raw_opt = self.ensure_thumb_raw(*id);
            let Some(raw) = raw_opt else {
                continue;
            };
            let frame = render_develop_pipeline(&raw, &DevelopParams::default(), RenderOptions::default());
            let (faces, focus_score) =
                detect_faces_and_focus_for_photo(&frame.rgba, frame.width, frame.height);
            let eyes_score = faces.first().map(|f| f.eyes_open_score).unwrap_or(85.0);
            let is_pass = focus_score >= self.culling_eye_focus_thresh
                && eyes_score >= self.culling_eyes_open_thresh;

            if let Some(p) = self.photos.iter_mut().find(|x| x.id == *id) {
                p.detected_faces = faces;
                p.culling_focus_score = Some(focus_score);
                if self.culling_reject_blinks && eyes_score < self.culling_eyes_open_thresh {
                    p.flag = PickFlag::Rejected;
                    rejected_blinks += 1;
                } else if is_pass && p.flag == PickFlag::Unflagged {
                    p.flag = PickFlag::Picked;
                }
                if is_pass {
                    passed += 1;
                }
                self.catalog_db.save_photo(p);
            }
        }
        self.status_message = format!(
            "Vision AI Assisted Culling analyzed {} photos: {} sharp/open Picks, {} blinks flagged.",
            ids.len(),
            passed,
            rejected_blinks
        );
    }

    pub fn merge_selected_hdr_or_pano(&mut self, is_hdr: bool) {
        let ids = if self.selected_photo_ids.len() >= 2 {
            self.selected_photo_ids.clone()
        } else {
            self.filtered_photo_ids().into_iter().take(3).collect()
        };
        if ids.len() < 2 {
            self.status_message = "Select at least 2 photos to Merge.".into();
            return;
        }

        let mut raws = Vec::new();
        for id in &ids {
            if let Some(p) = self.photos.iter().find(|x| x.id == *id) {
                raws.push(self.decode_photo_for_output(p, 1600));
            }
        }
        if raws.len() < 2 {
            return;
        }

        let out_dir = PathBuf::from(&self.export_folder).join("Merged");
        let _ = std::fs::create_dir_all(&out_dir);
        let ts_id = self.photos.iter().map(|p| p.id).max().unwrap_or(0) + 1;

        if is_hdr {
            let w = raws[0].width;
            let h = raws[0].height;
            let mut accum = vec![0.0_f32; (w as usize) * (h as usize) * 3];
            let count = raws.len() as f32;
            for r in &raws {
                for y in 0..h {
                    let sy = (y * r.height / h.max(1)).min(r.height.saturating_sub(1));
                    for x in 0..w {
                        let sx = (x * r.width / w.max(1)).min(r.width.saturating_sub(1));
                        let si = ((sy * r.width + sx) * 3) as usize;
                        let di = ((y * w + x) * 3) as usize;
                        accum[di] += r.rgb[si] / count;
                        accum[di + 1] += r.rgb[si + 1] / count;
                        accum[di + 2] += r.rgb[si + 2] / count;
                    }
                }
            }
            let mut merged_raw = raws[0].clone();
            merged_raw.rgb = accum;
            let mut hdr_params = DevelopParams::default();
            hdr_params.hdr_edit_mode = true;
            hdr_params.highlights_2012 = -45.0;
            hdr_params.shadows_2012 = 38.0;
            let frame = render_develop_pipeline(&merged_raw, &hdr_params, RenderOptions::default());
            let dst = out_dir.join(format!("HDR_Merge_{:04}.tif", ts_id));
            if let Ok(f) = std::fs::File::create(&dst) {
                let enc = image::codecs::tiff::TiffEncoder::new(std::io::BufWriter::new(f));
                use image::ImageEncoder;
                let _ = enc.write_image(&frame.rgba, w, h, image::ExtendedColorType::Rgba8);
                if let Some(new_id) = self.import_single_file_path(&dst, None) {
                    if let Some(np) = self.photos.iter_mut().find(|x| x.id == new_id) {
                        np.params.hdr_edit_mode = true;
                        self.catalog_db.save_photo(np);
                    }
                    self.select_photo(new_id, false);
                }
                self.status_message = format!("Merged {} bracketed photos into {}", ids.len(), dst.display());
            }
        } else {
            let h = raws[0].height.max(240);
            let overlap = (raws[0].width as usize / 10).max(16);
            let mut total_w = 0usize;
            let scaled_widths: Vec<u32> = raws
                .iter()
                .map(|r| ((r.width as f32 * (h as f32 / r.height.max(1) as f32)).round() as u32).max(64))
                .collect();
            for (idx, &sw) in scaled_widths.iter().enumerate() {
                total_w += sw as usize;
                if idx > 0 {
                    total_w = total_w.saturating_sub(overlap);
                }
            }
            let out_w = total_w.max(128) as u32;
            let mut pano_rgba = vec![0u8; (out_w as usize) * (h as usize) * 4];
            let mut cur_x = 0usize;
            for (idx, r) in raws.iter().enumerate() {
                let frame = render_develop_pipeline(r, &DevelopParams::default(), RenderOptions::default());
                let sw = scaled_widths[idx] as usize;
                for y in 0..h as usize {
                    let sy = (y * frame.height as usize / h as usize).min(frame.height as usize - 1);
                    for lx in 0..sw {
                        let gx = cur_x + lx;
                        if gx >= out_w as usize {
                            break;
                        }
                        let sx = (lx * frame.width as usize / sw.max(1)).min(frame.width as usize - 1);
                        let si = (sy * frame.width as usize + sx) * 4;
                        let di = (y * out_w as usize + gx) * 4;
                        let alpha = if idx > 0 && lx < overlap {
                            lx as f32 / overlap as f32
                        } else {
                            1.0
                        };
                        for c in 0..3 {
                            pano_rgba[di + c] = (pano_rgba[di + c] as f32 * (1.0 - alpha)
                                + frame.rgba[si + c] as f32 * alpha)
                                as u8;
                        }
                        pano_rgba[di + 3] = 255;
                    }
                }
                cur_x = (cur_x + sw).saturating_sub(overlap);
            }
            let dst = out_dir.join(format!("Pano_Merge_{:04}.jpg", ts_id));
            if let Some(img) = image::RgbaImage::from_raw(out_w, h, pano_rgba) {
                let _ = image::DynamicImage::ImageRgba8(img).to_rgb8().save(&dst);
                if let Some(new_id) = self.import_single_file_path(&dst, None) {
                    self.select_photo(new_id, false);
                }
                self.status_message = format!("Stitched {}-photo Panorama to {}", ids.len(), dst.display());
            }
        }
    }

    // ========================================================================
    // Import Workspace, Folder Sync & GPX Geotagging
    // ========================================================================
    pub fn open_native_folder_picker(&mut self) {
        if let Some(folder) = rfd::FileDialog::new()
            .set_title("Select Source Folder or Camera Card")
            .pick_folder()
        {
            self.scan_import_source(&folder);
        }
    }

    pub fn scan_import_source(&mut self, folder: &Path) {
        self.import_selected_source = Some(folder.to_path_buf());
        self.import_candidates.clear();
        self.import_thumb_cache.clear();
        let Ok(entries) = std::fs::read_dir(folder) else {
            return;
        };
        let mut files: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_file() && is_supported_photo_path(p))
            .collect();
        files.sort();

        for path in files {
            let file_name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("Photo")
                .to_string();
            let file_size_bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            let full_str = path.to_string_lossy().to_string();
            let is_duplicate = self
                .photos
                .iter()
                .any(|p| p.real_file_path.as_deref() == Some(full_str.as_str()));
            let is_checked = !(is_duplicate && self.import_dont_import_duplicates);
            self.import_candidates.push(ImportCandidate {
                path,
                file_name,
                file_size_bytes,
                capture_time: String::new(),
                is_checked,
                is_duplicate,
            });
        }
    }

    pub fn get_import_candidate_texture(
        &mut self,
        ctx: &egui::Context,
        path: &Path,
    ) -> Option<TextureHandle> {
        if let Some(tex) = self.import_thumb_cache.get(path) {
            return Some(tex.clone());
        }
        // Limit per-frame decodes so scanning 400+ RAWs stays buttery 60 FPS!
        let decoded_this_frame = ctx.data_mut(|d| {
            let k = egui::Id::new("imp_decodes_this_frame");
            let c = d.get_temp::<usize>(k).unwrap_or(0);
            d.insert_temp(k, c + 1);
            c
        });
        if decoded_this_frame >= 4 {
            ctx.request_repaint();
            return None;
        }

        let raw = decode_real_photo_file(path, 220, 220)?;
        let frame = render_develop_pipeline(&raw, &DevelopParams::default(), RenderOptions::default());
        let img = egui::ColorImage::from_rgba_unmultiplied(
            [frame.width as usize, frame.height as usize],
            &frame.rgba,
        );
        let tex = ctx.load_texture(
            format!("imp_cand_{}", path.display()),
            img,
            egui::TextureOptions::LINEAR,
        );
        if self.import_thumb_cache.len() >= 500 {
            self.import_thumb_cache.clear();
        }
        self.import_thumb_cache.insert(path.to_path_buf(), tex.clone());
        Some(tex)
    }

    pub fn import_single_file_path(
        &mut self,
        src_path: &Path,
        preset_name_opt: Option<&str>,
    ) -> Option<i64> {
        if !src_path.exists() || !is_supported_photo_path(src_path) {
            return None;
        }
        let final_path = match self.import_action {
            ImportActionChoice::Add => src_path.to_path_buf(),
            ImportActionChoice::Copy | ImportActionChoice::CopyAsDng => {
                let dst_dir =
                    PathBuf::from(&self.import_destination_folder).join(self.import_subfolder.trim());
                let _ = std::fs::create_dir_all(&dst_dir);
                let dst = dst_dir.join(src_path.file_name()?);
                if dst != src_path {
                    let _ = std::fs::copy(src_path, &dst);
                }
                dst
            }
            ImportActionChoice::Move => {
                let dst_dir =
                    PathBuf::from(&self.import_destination_folder).join(self.import_subfolder.trim());
                let _ = std::fs::create_dir_all(&dst_dir);
                let dst = dst_dir.join(src_path.file_name()?);
                if dst != src_path {
                    if std::fs::rename(src_path, &dst).is_err() {
                        if std::fs::copy(src_path, &dst).is_ok() {
                            let _ = std::fs::remove_file(src_path);
                        }
                    }
                }
                dst
            }
        };

        let full_str = final_path.to_string_lossy().to_string();
        if self.import_dont_import_duplicates
            && self
                .photos
                .iter()
                .any(|p| p.real_file_path.as_deref() == Some(full_str.as_str()))
        {
            return None;
        }

        let next_id = self.photos.iter().map(|p| p.id).max().unwrap_or(0) + 1;
        let meta = extract_real_photo_metadata(&final_path);
        let mut p = create_empty_placeholder_photo();
        p.id = next_id;
        p.file_name = final_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Photo")
            .to_string();
        p.folder_path = final_path
            .parent()
            .map(|d| d.to_string_lossy().to_string())
            .unwrap_or_else(|| "/".into());
        p.real_file_path = Some(full_str);
        p.capture_time = meta.capture_time;
        p.camera_make = meta.camera_make;
        p.camera_model = meta.camera_model;
        p.lens_model = meta.lens_model;
        p.focal_length_mm = meta.focal_length_mm;
        p.aperture_f = meta.aperture_f;
        p.shutter_speed = meta.shutter_speed;
        p.iso = meta.iso;
        if meta.width > 0 && meta.height > 0 {
            p.width = meta.width;
            p.height = meta.height;
        }
        p.gps_lat = meta.gps_lat;
        p.gps_lon = meta.gps_lon;

        // Parse import keywords
        for kw in self.import_keywords.split(',') {
            let trimmed = kw.trim();
            if !trimmed.is_empty() && !p.keywords.iter().any(|x| x == trimmed) {
                p.keywords.push(trimmed.to_string());
            }
        }

        // Apply on Import preset if configured
        let chosen_preset = preset_name_opt
            .filter(|s| *s != "None" && !s.is_empty())
            .map(|s| s.to_string())
            .or_else(|| {
                self.apply_on_import_preset_id
                    .as_ref()
                    .and_then(|id| self.presets.iter().find(|pr| &pr.id == id))
                    .map(|pr| pr.name.clone())
            });
        if let Some(pr_name) = chosen_preset {
            if pr_name == "Auto Tone" {
                p.params.auto_tone = true;
            } else if let Some(pr) = self.presets.iter().find(|x| x.name == pr_name).cloned() {
                p.params = Self::compute_preset_applied_params(&p.params, &pr);
            }
        }
        p.history = vec![HistoryStep {
            name: "Import RAW".into(),
            value_summary: String::new(),
            timestamp: p.capture_time.clone(),
            params: p.params.clone(),
        }];
        p.history_index = 0;

        self.catalog_db.save_photo(&p);
        self.photos.push(p);
        Some(next_id)
    }

    pub fn execute_current_import(&mut self) {
        let paths: Vec<PathBuf> = self
            .import_candidates
            .iter()
            .filter(|c| c.is_checked)
            .map(|c| c.path.clone())
            .collect();
        let preset_name = self.import_develop_preset.clone();
        let mut first_new = None;
        let mut count = 0usize;
        for path in paths {
            if let Some(id) = self.import_single_file_path(&path, Some(&preset_name)) {
                if first_new.is_none() {
                    first_new = Some(id);
                }
                count += 1;
            }
        }
        if let Some(fid) = first_new {
            self.select_photo(fid, false);
            if let Some(p) = self.photos.iter().find(|x| x.id == fid) {
                self.selected_folder = Some(p.folder_path.clone());
                self.selected_collection_id = None;
            }
        }
        self.show_import_modal = false;
        self.import_candidates.clear();
        self.import_thumb_cache.clear();
        self.active_module = WorkspaceModule::Library;
        self.library_view_mode = LibraryViewMode::Grid;
        self.status_message = format!("Imported {} photo(s) into catalog.", count);
    }

    pub fn synchronize_folder(&mut self, folder_path: &str) {
        let dir = Path::new(folder_path);
        if !dir.exists() {
            self.status_message = format!("Folder missing on disk: {}", folder_path);
            return;
        }
        // 1. Remove catalog entries whose files were deleted on disk
        let missing_ids: Vec<i64> = self
            .photos
            .iter()
            .filter(|p| p.folder_path == folder_path)
            .filter(|p| {
                p.real_file_path
                    .as_ref()
                    .map(|rp| !Path::new(rp).exists())
                    .unwrap_or(false)
            })
            .map(|p| p.id)
            .collect();
        for id in &missing_ids {
            self.catalog_db.delete_photo(*id);
        }
        self.photos.retain(|p| !missing_ids.contains(&p.id));

        // 2. Import newly added files in folder
        let mut added = 0usize;
        if let Ok(entries) = std::fs::read_dir(dir) {
            let prev_action = self.import_action;
            self.import_action = ImportActionChoice::Add;
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && is_supported_photo_path(&p) {
                    let s = p.to_string_lossy().to_string();
                    if !self
                        .photos
                        .iter()
                        .any(|ph| ph.real_file_path.as_deref() == Some(s.as_str()))
                    {
                        if self.import_single_file_path(&p, None).is_some() {
                            added += 1;
                        }
                    }
                }
            }
            self.import_action = prev_action;
        }
        self.status_message = format!(
            "Synchronized '{}': +{} new photo(s), -{} missing removed.",
            folder_path,
            added,
            missing_ids.len()
        );
    }

    pub fn remove_folder_from_catalog(&mut self, folder_path: &str) {
        let ids: Vec<i64> = self
            .photos
            .iter()
            .filter(|p| p.folder_path == folder_path)
            .map(|p| p.id)
            .collect();
        for id in &ids {
            self.catalog_db.delete_photo(*id);
            self.invalidate_photo_render_caches(*id);
        }
        self.photos.retain(|p| p.folder_path != folder_path);
        if self.selected_folder.as_deref() == Some(folder_path) {
            self.selected_folder = None;
        }
        if let Some(first) = self.photos.first() {
            self.active_photo_id = first.id;
            self.selected_photo_ids = vec![first.id];
        } else {
            self.selected_photo_ids.clear();
        }
        self.status_message = format!(
            "Removed folder '{}' ({} photos) from catalog (files on disk untouched).",
            folder_path,
            ids.len()
        );
    }

    pub fn load_gpx_tracklog_dialog(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Load GPS Tracklog (.gpx)")
            .add_filter("GPX Tracklog", &["gpx", "GPX", "xml"])
            .pick_file()
        else {
            return;
        };
        let Ok(xml) = std::fs::read_to_string(&path) else {
            return;
        };
        let mut pts = Vec::new();
        for seg in xml.split("<trkpt") {
            let lat = seg
                .split("lat=\"")
                .nth(1)
                .and_then(|s| s.split('"').next())
                .and_then(|v| v.parse::<f64>().ok());
            let lon = seg
                .split("lon=\"")
                .nth(1)
                .and_then(|s| s.split('"').next())
                .and_then(|v| v.parse::<f64>().ok());
            let time = seg
                .split("<time>")
                .nth(1)
                .and_then(|s| s.split("</time>").next())
                .unwrap_or("")
                .to_string();
            if let (Some(la), Some(lo)) = (lat, lon) {
                pts.push(GpxTrackPoint {
                    lat: la,
                    lon: lo,
                    time,
                });
            }
        }
        if let Some(first) = pts.first() {
            self.map_center_lat = first.lat;
            self.map_center_lon = first.lon;
        }
        let n = pts.len();
        self.gpx_track_points = pts;
        self.status_message = format!(
            "Loaded {} GPS trackpoint(s) from {}",
            n,
            path.display()
        );
    }

    // ========================================================================
    // Photo Context Menu, Top Module Bar & Virtualized Bottom Filmstrip
    // ========================================================================
    pub fn render_photo_context_menu(&mut self, ui: &mut Ui, photo_id: i64) {
        if !self.selected_photo_ids.contains(&photo_id) {
            self.select_photo(photo_id, false);
        }
        if ui.button("Develop Photo (D)").clicked() {
            self.select_photo(photo_id, false);
            self.active_module = WorkspaceModule::Develop;
            ui.close_menu();
        }
        if ui.button("Set as Reference Photo (Develop)").clicked() {
            self.reference_photo_id = Some(photo_id);
            self.active_module = WorkspaceModule::Develop;
            self.before_after_mode = BeforeAfterMode::ReferenceView;
            ui.close_menu();
        }
        ui.separator();
        ui.menu_button("Set Rating", |ui| {
            for r in 0..=5u8 {
                let lbl = if r == 0 {
                    "None (0★)".to_string()
                } else {
                    "★".repeat(r as usize)
                };
                if ui.button(lbl).clicked() {
                    self.set_rating_on_selection(r);
                    ui.close_menu();
                }
            }
        });
        ui.menu_button("Set Flag", |ui| {
            if ui.button("⚑ Flag as Pick (P)").clicked() {
                self.set_flag_on_selection(PickFlag::Picked);
                ui.close_menu();
            }
            if ui.button("Unflagged (U)").clicked() {
                self.set_flag_on_selection(PickFlag::Unflagged);
                ui.close_menu();
            }
            if ui.button("✕ Rejected (X)").clicked() {
                self.set_flag_on_selection(PickFlag::Rejected);
                ui.close_menu();
            }
        });
        ui.menu_button("Set Color Label", |ui| {
            for col in [
                ColorLabel::Red,
                ColorLabel::Yellow,
                ColorLabel::Green,
                ColorLabel::Blue,
                ColorLabel::Purple,
                ColorLabel::None,
            ] {
                if ui.button(col.name()).clicked() {
                    self.set_color_label_on_selection(col);
                    ui.close_menu();
                }
            }
        });
        ui.separator();
        if ui.button("Copy Develop Settings... (⇧⌘C)").clicked() {
            self.show_copy_settings_modal = true;
            ui.close_menu();
        }
        if ui
            .add_enabled(self.clipboard_params.is_some(), egui::Button::new("Paste Develop Settings (⇧⌘V)"))
            .clicked()
        {
            self.paste_clipboard_settings();
            ui.close_menu();
        }
        if ui.button("Create Virtual Copy (⌘')").clicked() {
            self.create_virtual_copy();
            ui.close_menu();
        }
        if ui.button("Export... (⇧⌘E)").clicked() {
            self.show_export_modal = true;
            ui.close_menu();
        }
        if ui.button("Show in Finder (⌘R)").clicked() {
            self.reveal_active_photo_in_finder();
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Remove from Catalog...").clicked() {
            self.remove_selected_photos_from_catalog();
            ui.close_menu();
        }
    }

    fn render_top_module_picker(&mut self, ctx: &egui::Context) {
        if !self.show_top_module_picker || self.slideshow_fullscreen {
            return;
        }
        egui::TopBottomPanel::top("lr_top_identity_module_bar")
            .exact_height(38.0)
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(24, 24, 27))
                    .inner_margin(egui::Margin::symmetric(14.0, 5.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    // Left Identity Plate (App icon badge + subtitle + owner — click for About)
                    let (badge_rect, badge_resp) =
                        ui.allocate_exact_size(Vec2::new(26.0, 26.0), Sense::click());
                    let badge_resp = badge_resp.on_hover_text(format!(
                        "LightRaw Classic {} — Click for About & System Info",
                        CURRENT_VERSION
                    ));
                    if badge_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if badge_resp.clicked() {
                        self.show_about_modal = true;
                        self.about_tab = 0;
                    }
                    if let Some(tex) = self.get_app_icon_texture(ctx) {
                        ui.painter().image(
                            tex.id(),
                            badge_rect,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    } else {
                        ui.painter()
                            .rect_filled(badge_rect, 4.0, Color32::from_rgb(20, 48, 82));
                        ui.painter().rect_stroke(
                            badge_rect,
                            4.0,
                            Stroke::new(1.0, Color32::from_rgb(55, 135, 235)),
                        );
                        ui.painter().text(
                            badge_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "LR",
                            egui::FontId::proportional(12.0),
                            Color32::from_rgb(120, 195, 255),
                        );
                    }

                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(&self.identity_plate_subtitle)
                            .size(9.5)
                            .color(Colors::TEXT_DIM),
                    );
                    ui.add_space(4.0);
                    let id_resp = ui.add(
                        egui::Label::new(
                            RichText::new(&self.identity_plate_owner)
                                .size(14.0)
                                .strong()
                                .color(Colors::TEXT_BRIGHT),
                        )
                        .sense(Sense::click()),
                    );
                    if id_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if id_resp.clicked() {
                        self.show_preferences_modal = true;
                    }

                    // Lightroom Classic Activity / Status indicator (placed safely on left side, zero collision with module switcher!)
                    if !self.status_message.is_empty() {
                        ui.add_space(16.0);
                        ui.label(RichText::new("•").size(10.0).color(Color32::from_rgb(0, 150, 255)));
                        ui.add_space(4.0);
                        let max_len = 45;
                        let msg = if self.status_message.chars().count() > max_len {
                            let truncated: String = self.status_message.chars().take(max_len - 3).collect();
                            format!("{}...", truncated)
                        } else {
                            self.status_message.clone()
                        };
                        ui.label(RichText::new(msg).size(10.5).color(Colors::TEXT_MID));
                    }

                    let fps = (1000.0 / self.rolling_frame_time_ms.max(0.5)).round() as u32;
                    let perf_col = if fps >= 55 {
                        Color32::from_rgb(50, 205, 90)
                    } else if fps >= 30 {
                        Colors::ACCENT_AMBER
                    } else {
                        Color32::from_rgb(235, 75, 75)
                    };
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(format!("{:.1}ms", self.rolling_frame_time_ms))
                            .size(9.0)
                            .color(perf_col),
                    )
                    .on_hover_text("Real-time Develop & UI Frame Render Latency");

                    if let UpdaterUiState::Available(ref rel) = self.update_state {
                        ui.add_space(10.0);
                        let upd_btn = egui::Button::new(
                            RichText::new(format!("⬆ Update {}", rel.tag_name))
                                .size(10.5)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(Color32::from_rgb(37, 99, 235))
                        .rounding(egui::Rounding::same(10.0));
                        if ui.add(upd_btn).clicked() {
                            self.show_update_modal = true;
                        }
                    }

                    // Right-aligned Lightroom Classic Module Picker: Library | Develop | Map | Book | Slideshow | Print | Web
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        for (idx, module) in WorkspaceModule::ALL.iter().rev().enumerate() {
                            let is_active = self.active_module == *module;
                            let text = module.label();
                            let font_id = egui::FontId::proportional(14.0);
                            let text_layout = ui.painter().layout_no_wrap(
                                text.to_string(),
                                font_id.clone(),
                                Color32::WHITE,
                            );
                            let btn_w = text_layout.rect.width() + 20.0;
                            let btn_h = 28.0;
                            let (btn_rect, btn_resp) =
                                ui.allocate_exact_size(Vec2::new(btn_w, btn_h), Sense::click());
                            if btn_resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            if btn_resp.clicked() {
                                self.active_module = *module;
                            }

                            // Render background pill
                            if is_active {
                                ui.painter().rect_filled(btn_rect, 4.0, Color32::from_rgb(52, 52, 58));
                                ui.painter().rect_stroke(
                                    btn_rect,
                                    4.0,
                                    Stroke::new(1.0, Color32::from_rgb(70, 70, 78)),
                                );
                            } else if btn_resp.hovered() {
                                ui.painter().rect_filled(
                                    btn_rect,
                                    4.0,
                                    Color32::from_rgba_unmultiplied(255, 255, 255, 18),
                                );
                            }

                            // Render text centered
                            let text_col = if is_active {
                                Color32::WHITE
                            } else if btn_resp.hovered() {
                                Color32::from_rgb(230, 230, 235)
                            } else {
                                Color32::from_rgb(148, 148, 156)
                            };
                            ui.painter().text(
                                btn_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                text,
                                font_id,
                                text_col,
                            );

                            // Divider separator between tabs with PointingHand so transitions are seamless
                            if idx + 1 < WorkspaceModule::ALL.len() {
                                let (sep_rect, sep_resp) = ui.allocate_exact_size(Vec2::new(10.0, btn_h), Sense::hover());
                                if sep_resp.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                ui.painter().line_segment(
                                    [
                                        Pos2::new(sep_rect.center().x, sep_rect.min.y + 6.0),
                                        Pos2::new(sep_rect.center().x, sep_rect.max.y - 6.0),
                                    ],
                                    Stroke::new(1.0, Color32::from_rgb(55, 55, 62)),
                                );
                            }
                        }
                    });
                });
            });
    }

    fn render_bottom_filmstrip(&mut self, ctx: &egui::Context) {
        if !self.show_bottom_filmstrip || self.slideshow_fullscreen {
            return;
        }
        egui::TopBottomPanel::bottom("lr_bottom_filmstrip")
            .exact_height(114.0)
            .frame(egui::Frame::none().fill(Color32::from_rgb(24, 24, 27)))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                let visible_ids = self.filtered_photo_ids();

                // Breadcrumb & Filter status bar above thumbnails
                egui::Frame::none()
                    .fill(Color32::from_rgb(32, 32, 36))
                    .inner_margin(egui::Margin::symmetric(10.0, 3.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui
                                .small_button("⊞")
                                .on_hover_text("Return to Library Grid (G)")
                                .clicked()
                            {
                                self.active_module = WorkspaceModule::Library;
                                self.library_view_mode = LibraryViewMode::Grid;
                            }
                            if ui.small_button("◀").on_hover_text("Previous Photo").clicked() {
                                self.step_active_photo(-1);
                            }
                            if ui.small_button("▶").on_hover_text("Next Photo").clicked() {
                                self.step_active_photo(1);
                            }
                            ui.add_space(6.0);
                            let folder_lbl = self
                                .selected_folder
                                .as_deref()
                                .and_then(|p| Path::new(p).file_name())
                                .and_then(|s| s.to_str())
                                .unwrap_or("All Photographs");
                            let active_fname = if self.photos.is_empty() {
                                "No Photos".to_string()
                            } else {
                                self.active_photo().file_name.clone()
                            };
                            ui.label(
                                RichText::new(format!(
                                    "Folder : {}     {} photos / {} selected / {}",
                                    folder_lbl,
                                    visible_ids.len(),
                                    self.selected_photo_ids.len(),
                                    active_fname
                                ))
                                .size(11.0)
                                .color(Colors::TEXT_MAIN),
                            );

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let filter_lbl = if self.filter_only_picks {
                                        "Flagged"
                                    } else if self.filter_min_rating > 0 {
                                        "Rated"
                                    } else {
                                        "Filters Off"
                                    };
                                    egui::ComboBox::from_id_salt("filmstrip_quick_filter")
                                        .selected_text(filter_lbl)
                                        .width(95.0)
                                        .show_ui(ui, |ui| {
                                            if ui.selectable_label(filter_lbl == "Filters Off", "Filters Off").clicked() {
                                                self.filter_only_picks = false;
                                                self.filter_min_rating = 0;
                                                self.filter_color_label = None;
                                            }
                                            if ui.selectable_label(self.filter_only_picks, "Flagged Picks").clicked() {
                                                self.filter_only_picks = true;
                                            }
                                            if ui.selectable_label(self.filter_min_rating >= 1, "Rated (≥ 1★)").clicked() {
                                                self.filter_min_rating = 1;
                                            }
                                        });
                                    ui.label(
                                        RichText::new("Filter :")
                                            .size(10.5)
                                            .color(Colors::TEXT_DIM),
                                    );
                                },
                            );
                        });
                    });

                // Virtualized Horizontal Filmstrip ScrollArea (O(visible) even with 1,000+ photos!)
                let cell_w = 88.0_f32;
                let cell_h = 82.0_f32;
                let spacing = 4.0_f32;
                let step = cell_w + spacing;
                let total_count = visible_ids.len();

                egui::ScrollArea::horizontal()
                    .id_salt("lr_filmstrip_scroll")
                    .auto_shrink([false, false])
                    .show_viewport(ui, |ui, viewport| {
                        let total_w = (total_count as f32 * step + 12.0).max(ui.available_width());
                        let (strip_rect, _) =
                            ui.allocate_exact_size(Vec2::new(total_w, cell_h + 4.0), Sense::hover());

                        if total_count == 0 {
                            return;
                        }
                        let first_idx = ((viewport.min.x / step).floor() as isize - 1).max(0) as usize;
                        let last_idx =
                            (((viewport.max.x / step).ceil() as usize) + 2).min(total_count);

                        for idx in first_idx..last_idx {
                            let pid = visible_ids[idx];
                            let x0 = strip_rect.min.x + 6.0 + (idx as f32) * step;
                            let cell_rect = Rect::from_min_size(
                                Pos2::new(x0, strip_rect.min.y + 2.0),
                                Vec2::new(cell_w, cell_h),
                            );

                            let cell_resp = ui.interact(
                                cell_rect,
                                ui.make_persistent_id(("filmstrip_cell", pid)),
                                Sense::click(),
                            );
                            if cell_resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            if cell_resp.clicked() {
                                let mods = ui.input(|i| i.modifiers);
                                if mods.shift {
                                    self.select_photo_range(pid);
                                } else if mods.command || mods.ctrl {
                                    self.toggle_select_photo(pid);
                                } else {
                                    self.select_photo(pid, false);
                                }
                            }
                            if cell_resp.double_clicked() {
                                self.select_photo(pid, false);
                                self.active_module = WorkspaceModule::Develop;
                            }

                            let is_active = self.active_photo_id == pid;
                            let is_sel = self.selected_photo_ids.contains(&pid);
                            let bg = if is_active {
                                Color32::from_rgb(112, 112, 118)
                            } else if is_sel {
                                Color32::from_rgb(76, 76, 82)
                            } else {
                                Color32::from_rgb(44, 44, 48)
                            };
                            ui.painter().rect_filled(cell_rect, 2.5, bg);
                            if is_active {
                                ui.painter().rect_stroke(
                                    cell_rect,
                                    2.5,
                                    Stroke::new(1.5, Color32::WHITE),
                                );
                            }

                            ui.painter().text(
                                cell_rect.min + Vec2::new(4.0, 2.0),
                                egui::Align2::LEFT_TOP,
                                format!("{}", idx + 1),
                                egui::FontId::proportional(9.0),
                                if is_active {
                                    Color32::WHITE
                                } else {
                                    Colors::TEXT_MID
                                },
                            );

                            let img_box = Rect::from_min_max(
                                Pos2::new(cell_rect.min.x + 5.0, cell_rect.min.y + 13.0),
                                Pos2::new(cell_rect.max.x - 5.0, cell_rect.max.y - 4.0),
                            );
                            if let Some(tex) = self.get_thumb_texture(ctx, pid) {
                                let ts = tex.size_vec2();
                                let sc = (img_box.width() / ts.x).min(img_box.height() / ts.y);
                                let dst = Rect::from_center_size(img_box.center(), ts * sc);
                                ui.painter().image(
                                    tex.id(),
                                    dst,
                                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                    Color32::WHITE,
                                );
                                if let Some(ph) = self.photos.iter().find(|p| p.id == pid) {
                                    paint_thumbnail_corner_badges(ui, dst, ph);
                                }
                            }

                            cell_resp.context_menu(|ui| {
                                self.render_photo_context_menu(ui, pid);
                            });
                        }
                    });
            });
    }
}

impl eframe::App for LightRawApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame_instant).as_secs_f32() * 1000.0;
        self.last_frame_instant = now;
        self.rolling_frame_time_ms = self.rolling_frame_time_ms * 0.92 + dt.clamp(0.1, 150.0) * 0.08;

        // Reset per-frame import thumbnail decode counter
        ctx.data_mut(|d| d.insert_temp(egui::Id::new("imp_decodes_this_frame"), 0usize));

        // Track pointer release to immediately trigger high-acutance Retina pass and update navigator/filmstrip!
        let is_pointer_down = ctx.input(|i| i.pointer.primary_down());
        if self.was_pointer_down && !is_pointer_down {
            self.rendered_thumb_cache.remove(&self.active_photo_id);
            ctx.request_repaint();
        }
        self.was_pointer_down = is_pointer_down;

        // 1. Poll background auto-updater & register wakeup handle
        self.poll_updater(ctx);
        menu_dispatch::register_egui_context_for_menu(ctx);
        #[cfg(target_os = "macos")]
        unsafe {
            static INIT_MENU: std::sync::Once = std::sync::Once::new();
            INIT_MENU.call_once(|| {
                extern "C" {
                    fn setup_macos_native_menu();
                }
                setup_macos_native_menu();
            });
            menu_dispatch::lightraw_set_text_input_focused(ctx.wants_keyboard_input() as i32);
        }

        // 2. Clear transient hover previews at start of frame (re-populated if still hovered)
        self.hovered_preset_preview = None;
        self.hovered_history_preview = None;
        self.bypassed_panel = None;

        // 3. Process native macOS menu actions & keyboard shortcuts
        self.drain_native_menu_actions();
        self.handle_global_shortcuts(ctx);

        // 4. Handle drag-and-drop files / folders onto the LightRaw window
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .collect()
        });
        if !dropped.is_empty() {
            for dp in dropped {
                if dp.is_dir() {
                    self.scan_import_source(&dp);
                    self.show_import_modal = true;
                } else if is_supported_photo_path(&dp) {
                    if let Some(id) = self.import_single_file_path(&dp, None) {
                        self.select_photo(id, false);
                    }
                } else if dp.extension().and_then(|s| s.to_str()).map(|s| s.eq_ignore_ascii_case("xmp")).unwrap_or(false) {
                    if let Ok(xml) = std::fs::read_to_string(&dp) {
                        if let Some(pr) = parse_xmp_preset(&xml, "Imported") {
                            self.catalog_db.save_preset(&pr);
                            self.presets.push(pr);
                            self.status_message = "Imported dropped .xmp preset.".into();
                        }
                    }
                }
            }
        }

        // 5. Debounced SQLite save flush (zero disk thrash during live slider dragging, zero idle CPU!)
        if !self.dirty_photo_ids.is_empty() {
            if let Some(ts) = self.last_edit_instant {
                if ts.elapsed() >= Duration::from_millis(350)
                    && !ctx.input(|i| i.pointer.primary_down())
                {
                    self.flush_dirty_photos();
                } else {
                    ctx.request_repaint_after(Duration::from_millis(360));
                }
            }
        }

        // 6. Guarded Window Title update — ONLY send ViewportCommand::Title when changed!
        // (Fixes the 94% idle CPU bug on macOS winit!)
        let new_title = format!(
            "{} - LightRaw {} - {} - {}",
            self.catalog_display_name,
            CURRENT_VERSION,
            self.active_module.label(),
            self.identity_plate_owner
        );
        if new_title != self.last_window_title {
            self.last_window_title = new_title.clone();
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(new_title));
        }

        // 7. Render Full-Screen Import Workspace OR Active Module Shell
        if self.show_import_modal {
            other_modules::render_import_workspace(self, ctx);
        } else {
            self.render_system_menu_bar(ctx);
            self.render_top_module_picker(ctx);
            self.render_bottom_filmstrip(ctx);

            match self.active_module {
                WorkspaceModule::Library => library::render_library_module(self, ctx),
                WorkspaceModule::Develop => develop::render_develop_module(self, ctx),
                WorkspaceModule::Map => other_modules::render_map_module(self, ctx),
                WorkspaceModule::Book => other_modules::render_book_module(self, ctx),
                WorkspaceModule::Slideshow => other_modules::render_slideshow_module(self, ctx),
                WorkspaceModule::Print => other_modules::render_print_module(self, ctx),
                WorkspaceModule::Web => other_modules::render_web_module(self, ctx),
            }
        }

        // 8. Global Modal Dialogs
        other_modules::render_modals(self, ctx);
        presets_dialogs::render_presets_and_copy_modals(self, ctx);
        system_modals::render_system_modals(self, ctx);
        watermark_editor::render_watermark_editor_modal(self, ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.flush_dirty_photos();
    }
}
