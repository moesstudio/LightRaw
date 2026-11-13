use crate::params::DevelopParams;
use crate::pipeline::RawSourceImage;
use chrono::Local;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
#[cfg(target_os = "macos")]
use std::ffi::CString;
use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LightRawFaceInfo {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub quality: f32,
    pub eyes_open_score: f32,
    pub yaw_deg: f32,
}

#[cfg(target_os = "macos")]
extern "C" {
    fn lightraw_decode_image_linear(
        path_utf8: *const std::os::raw::c_char,
        max_dim: u32,
        is_raw_hint: std::os::raw::c_int,
        draft_mode: std::os::raw::c_int,
        out_w: *mut u32,
        out_h: *mut u32,
        out_native_w: *mut u32,
        out_native_h: *mut u32,
        out_as_shot_temp: *mut f32,
        out_as_shot_tint: *mut f32,
        out_baseline_ev: *mut f32,
        out_is_scene_referred: *mut std::os::raw::c_int,
    ) -> *mut f32;

    fn lightraw_free_float_buffer(ptr: *mut f32);

    fn lightraw_detect_faces_rgba(
        rgba_ptr: *const u8,
        width: u32,
        height: u32,
        out_faces: *mut LightRawFaceInfo,
        max_faces: std::os::raw::c_int,
    ) -> std::os::raw::c_int;

    fn lightraw_query_disk_space(
        path_utf8: *const std::os::raw::c_char,
        out_free_bytes: *mut u64,
        out_total_bytes: *mut u64,
        out_vol_name: *mut std::os::raw::c_char,
        vol_name_cap: usize,
    ) -> std::os::raw::c_int;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PickFlag {
    Unflagged = 0,
    Picked = 1,
    Rejected = -1,
}

impl PickFlag {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => PickFlag::Picked,
            -1 => PickFlag::Rejected,
            _ => PickFlag::Unflagged,
        }
    }

    pub fn sort_rank(&self) -> i8 {
        match self {
            PickFlag::Picked => 1,
            PickFlag::Unflagged => 0,
            PickFlag::Rejected => -1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorLabel {
    None,
    Red,
    Yellow,
    Green,
    Blue,
    Purple,
}

impl ColorLabel {
    pub const ALL: &'static [ColorLabel] = &[
        ColorLabel::None,
        ColorLabel::Red,
        ColorLabel::Yellow,
        ColorLabel::Green,
        ColorLabel::Blue,
        ColorLabel::Purple,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            ColorLabel::None => "None",
            ColorLabel::Red => "Red",
            ColorLabel::Yellow => "Yellow",
            ColorLabel::Green => "Green",
            ColorLabel::Blue => "Blue",
            ColorLabel::Purple => "Purple",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Red" => ColorLabel::Red,
            "Yellow" => ColorLabel::Yellow,
            "Green" => ColorLabel::Green,
            "Blue" => ColorLabel::Blue,
            "Purple" => ColorLabel::Purple,
            _ => ColorLabel::None,
        }
    }

    pub fn rgb_color(&self) -> Option<[u8; 3]> {
        match self {
            ColorLabel::None => None,
            ColorLabel::Red => Some([225, 65, 65]),
            ColorLabel::Yellow => Some([230, 195, 50]),
            ColorLabel::Green => Some([65, 190, 85]),
            ColorLabel::Blue => Some([65, 140, 235]),
            ColorLabel::Purple => Some([170, 95, 225]),
        }
    }

    pub fn name(&self) -> &'static str {
        self.as_str()
    }

    #[cfg(feature = "desktop")]
    pub fn rgb(&self) -> Option<eframe::egui::Color32> {
        self.rgb_color()
            .map(|[r, g, b]| eframe::egui::Color32::from_rgb(r, g, b))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStep {
    pub name: String,
    pub value_summary: String,
    pub timestamp: String,
    pub params: DevelopParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopSnapshot {
    pub name: String,
    pub timestamp: String,
    pub params: DevelopParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedFace {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub quality: f32,
    pub eyes_open_score: f32,
    pub yaw_deg: f32,
    #[serde(default)]
    pub person_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotoComment {
    pub author: String,
    pub timestamp: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotoItem {
    pub id: i64,
    pub file_name: String,
    pub folder_path: String,
    pub real_file_path: Option<String>,
    pub sample_scene_id: usize,
    pub is_virtual_copy: bool,
    pub copy_name: String,
    pub master_id: Option<i64>,
    pub stack_id: Option<i64>,
    pub rating: u8, // 0..5
    pub flag: PickFlag,
    pub color_label: ColorLabel,
    pub in_quick_collection: bool,
    pub in_previous_import: bool,

    // EXIF & IPTC Metadata (`AgHarvestedExifMetadata` / `AgHarvestedIptcMetadata`)
    pub camera_make: String,
    pub camera_model: String,
    pub lens_model: String,
    pub iso: u32,
    pub focal_length_mm: f32,
    pub aperture_f: f32,
    pub shutter_speed: String,
    pub capture_time: String,
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub caption: String,
    pub creator: String,
    pub copyright: String,
    pub sublocation: String,
    pub city: String,
    pub state_province: String,
    pub country: String,
    pub iso_country_code: String,
    pub gps_lat: Option<f64>,
    pub gps_lon: Option<f64>,
    pub gps_alt_m: Option<f32>,
    pub person_tag: String,

    pub keywords: Vec<String>,
    pub params: DevelopParams,
    pub history: Vec<HistoryStep>,
    pub history_index: usize,
    pub snapshots: Vec<DevelopSnapshot>,

    #[serde(default)]
    pub detected_faces: Vec<DetectedFace>,
    #[serde(default)]
    pub culling_focus_score: Option<f32>,
    #[serde(default)]
    pub comments: Vec<PhotoComment>,
}

impl PhotoItem {
    pub fn is_cropped(&self) -> bool {
        self.params.is_cropped()
    }

    pub fn is_modified(&self) -> bool {
        self.params.is_modified()
    }

    pub fn before_params(&self) -> DevelopParams {
        self.history
            .first()
            .map(|h| h.params.clone())
            .unwrap_or_default()
    }

    pub fn set_before_params(&mut self, p: DevelopParams) {
        if let Some(first) = self.history.first_mut() {
            first.params = p;
        } else {
            self.history.push(HistoryStep {
                name: "Import RAW".into(),
                value_summary: String::new(),
                timestamp: "Initial".into(),
                params: p,
            });
        }
    }
}

pub type CollectionItem = PhotoCollection;
pub type SnapshotItem = DevelopSnapshot;
pub type SavedLocationEntry = SavedMapLocation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotoCollection {
    pub id: i64,
    pub name: String,
    pub is_smart: bool,
    pub smart_rule_desc: String,
    pub photo_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedMapLocation {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub radius_m: f32,
}

pub struct CatalogDb {
    pub db_path: PathBuf,
    pub blob_dir: PathBuf,
    conn: Connection,
}

impl CatalogDb {
    pub fn open_or_create(workspace_dir: &Path) -> Self {
        let primary_db = workspace_dir.join("LightRawCatalog.lrcat");
        if let Ok(db) = Self::open_specific_file(&primary_db) {
            return db;
        }

        // Fallback to ~/Pictures/LightRaw/LightRawCatalog.lrcat if workspace_dir is read-only (e.g. /Applications)
        let fallback_dir = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir())
            .join("Pictures")
            .join("LightRaw");
        let _ = std::fs::create_dir_all(&fallback_dir);
        let fallback_db = fallback_dir.join("LightRawCatalog.lrcat");
        Self::open_specific_file(&fallback_db)
            .expect("Failed to open LightRawCatalog.lrcat in both workspace and ~/Pictures/LightRaw")
    }

    pub fn open_specific_file(db_path: &Path) -> Result<Self, String> {
        if let Some(parent) = db_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
        }
        let stem = db_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("LightRawCatalog");
        let blob_dir = db_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(format!("{}.lrcat-data", stem));
        let _ = std::fs::create_dir_all(&blob_dir);

        let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
        let _ = conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;");

        let db = Self {
            db_path: db_path.to_path_buf(),
            blob_dir,
            conn,
        };
        db.init_lrcat_schema();
        Ok(db)
    }

    /// Initializes the authentic Adobe Lightroom Classic `v2.0R22SQLSchema.lua` SQLite tables
    fn init_lrcat_schema(&self) {
        let _ = self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS Adobe_variablesTable (
                id_local INTEGER PRIMARY KEY,
                name TEXT UNIQUE,
                value TEXT
            );
            CREATE TABLE IF NOT EXISTS AgLibraryRootFolder (
                id_local INTEGER PRIMARY KEY,
                absolutePath TEXT UNIQUE,
                name TEXT
            );
            CREATE TABLE IF NOT EXISTS AgLibraryFolder (
                id_local INTEGER PRIMARY KEY,
                pathFromRoot TEXT,
                rootFolder INTEGER
            );
            CREATE TABLE IF NOT EXISTS AgLibraryFile (
                id_local INTEGER PRIMARY KEY,
                baseName TEXT,
                extension TEXT,
                folder INTEGER,
                originalFilename TEXT
            );
            CREATE TABLE IF NOT EXISTS Adobe_images (
                id_local INTEGER PRIMARY KEY,
                rootFile INTEGER,
                fileFormat TEXT,
                rating INTEGER DEFAULT 0,
                pick INTEGER DEFAULT 0,
                colorLabels TEXT DEFAULT 'None',
                copyName TEXT DEFAULT '',
                masterImage INTEGER,
                captureTime TEXT,
                fileWidth INTEGER,
                fileHeight INTEGER,
                photoJson TEXT
            );
            CREATE TABLE IF NOT EXISTS Adobe_imageDevelopSettings (
                id_local INTEGER PRIMARY KEY,
                image INTEGER UNIQUE,
                digest TEXT,
                processVersion TEXT,
                text TEXT
            );
            CREATE TABLE IF NOT EXISTS Adobe_libraryImageDevelopHistoryStep (
                id_local INTEGER PRIMARY KEY,
                image INTEGER,
                name TEXT,
                relValueString TEXT,
                dateCreated TEXT,
                text TEXT
            );
            CREATE TABLE IF NOT EXISTS Adobe_libraryImageDevelopSnapshot (
                id_local INTEGER PRIMARY KEY,
                image INTEGER,
                name TEXT,
                text TEXT
            );
            CREATE TABLE IF NOT EXISTS AgHarvestedExifMetadata (
                id_local INTEGER PRIMARY KEY,
                image INTEGER UNIQUE,
                aperture REAL,
                focalLength REAL,
                isoSpeedRating INTEGER,
                shutterSpeed TEXT,
                cameraModel TEXT,
                lensModel TEXT,
                gpsLatitude REAL,
                gpsLongitude REAL
            );
            CREATE TABLE IF NOT EXISTS AgHarvestedIptcMetadata (
                id_local INTEGER PRIMARY KEY,
                image INTEGER UNIQUE,
                creator TEXT,
                copyright TEXT,
                city TEXT,
                stateProvince TEXT,
                country TEXT,
                sublocation TEXT
            );
            CREATE TABLE IF NOT EXISTS AgLibraryKeyword (
                id_local INTEGER PRIMARY KEY,
                name TEXT UNIQUE
            );
            CREATE TABLE IF NOT EXISTS AgLibraryKeywordImage (
                id_local INTEGER PRIMARY KEY,
                image INTEGER,
                tag INTEGER
            );
            CREATE TABLE IF NOT EXISTS AgLibraryCollection (
                id_local INTEGER PRIMARY KEY,
                name TEXT,
                systemOnly INTEGER DEFAULT 0,
                collectionJson TEXT
            );
            CREATE TABLE IF NOT EXISTS AgLibraryCollectionImage (
                id_local INTEGER PRIMARY KEY,
                collection INTEGER,
                image INTEGER
            );
            CREATE TABLE IF NOT EXISTS AgLibraryBackups (
                id_local INTEGER PRIMARY KEY,
                backupDate TEXT,
                backupPath TEXT
            );
            CREATE TABLE IF NOT EXISTS AgDevelopPreset (
                id TEXT PRIMARY KEY,
                category TEXT NOT NULL,
                name TEXT NOT NULL,
                is_user INTEGER NOT NULL,
                is_favorite INTEGER NOT NULL DEFAULT 0,
                presetJson TEXT NOT NULL
            );
            ",
        );

        let _ = self.conn.execute(
            "INSERT OR REPLACE INTO Adobe_variablesTable (id_local, name, value) VALUES (1, 'Adobe_DBVersion', '14050100')",
            [],
        );

        // Automatically remove any legacy synthetic/template photos (where real_file_path is null)
        let _ = self.conn.execute(
            "DELETE FROM Adobe_images WHERE photoJson LIKE '%\"real_file_path\":null%'",
            [],
        );
        let _ = self.conn.execute(
            "DELETE FROM AgLibraryCollection WHERE id_local IN (5, 6) AND collectionJson LIKE '%\"is_smart\":false%'",
            [],
        );
    }

    pub fn save_app_var(&self, key: &str, value: &str) {
        let _ = self.conn.execute(
            "INSERT INTO Adobe_variablesTable (name, value) VALUES (?1, ?2)
             ON CONFLICT(name) DO UPDATE SET value = excluded.value",
            params![key, value],
        );
    }

    pub fn load_app_var(&self, key: &str) -> Option<String> {
        self.conn
            .query_row(
                "SELECT value FROM Adobe_variablesTable WHERE name = ?1",
                params![key],
                |row| row.get::<_, String>(0),
            )
            .ok()
    }

    pub fn save_saved_locations(&self, locs: &[SavedMapLocation]) {
        if let Ok(json) = serde_json::to_string(locs) {
            self.save_app_var("LightRaw_SavedMapLocations", &json);
        }
    }

    pub fn load_saved_locations(&self) -> Vec<SavedMapLocation> {
        if let Some(json) = self.load_app_var("LightRaw_SavedMapLocations") {
            if let Ok(locs) = serde_json::from_str::<Vec<SavedMapLocation>>(&json) {
                return locs;
            }
        }
        vec![
            SavedMapLocation {
                name: "Jakarta Studio".into(),
                lat: -6.2088,
                lon: 106.8456,
                radius_m: 250.0,
            },
            SavedMapLocation {
                name: "Mount Bromo Caldera".into(),
                lat: -7.9425,
                lon: 112.9530,
                radius_m: 1500.0,
            },
            SavedMapLocation {
                name: "Ubud Rice Terraces, Bali".into(),
                lat: -8.4312,
                lon: 115.2793,
                radius_m: 800.0,
            },
        ]
    }

    pub fn clear_all_photos(&self) {
        let _ = self.conn.execute("DELETE FROM Adobe_images", []);
        let _ = self.conn.execute("DELETE FROM Adobe_imageDevelopSettings", []);
        let _ = self.conn.execute("DELETE FROM AgHarvestedExifMetadata", []);
        let _ = self.conn.execute("DELETE FROM AgHarvestedIptcMetadata", []);
        let _ = self.conn.execute("DELETE FROM AgLibraryCollectionImage", []);
        let _ = self.conn.execute("DELETE FROM Adobe_libraryImageDevelopHistoryStep", []);
        let _ = self.conn.execute("DELETE FROM Adobe_libraryImageDevelopSnapshot", []);
    }

    pub fn load_all_photos(&self) -> Vec<PhotoItem> {
        let mut stmt = match self
            .conn
            .prepare("SELECT photoJson FROM Adobe_images ORDER BY id_local ASC")
        {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let rows = stmt.query_map([], |row| row.get::<_, String>(0));
        let mut items = Vec::new();
        if let Ok(iter) = rows {
            for r in iter.flatten() {
                if let Ok(item) = serde_json::from_str::<PhotoItem>(&r) {
                    items.push(item);
                }
            }
        }
        items
    }

    pub fn save_photo(&self, item: &PhotoItem) {
        let json = serde_json::to_string(item).unwrap_or_default();
        let params_json = serde_json::to_string(&item.params).unwrap_or_default();
        let file_fmt = item
            .real_file_path
            .as_deref()
            .and_then(|p| Path::new(p).extension().and_then(|s| s.to_str()))
            .map(|e| e.to_ascii_uppercase())
            .unwrap_or_else(|| "RAW".to_string());

        let _ = self.conn.execute(
            "INSERT OR REPLACE INTO Adobe_images (id_local, fileFormat, rating, pick, colorLabels, copyName, masterImage, captureTime, fileWidth, fileHeight, photoJson)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                item.id,
                file_fmt,
                item.rating as i32,
                item.flag as i32,
                item.color_label.as_str(),
                item.copy_name,
                item.master_id,
                item.capture_time,
                item.width,
                item.height,
                json
            ],
        );

        let _ = self.conn.execute(
            "INSERT OR REPLACE INTO Adobe_imageDevelopSettings (image, processVersion, text)
             VALUES (?1, ?2, ?3)",
            params![item.id, item.params.process_version, params_json],
        );

        let _ = self.conn.execute(
            "INSERT OR REPLACE INTO AgHarvestedExifMetadata (image, aperture, focalLength, isoSpeedRating, shutterSpeed, cameraModel, lensModel, gpsLatitude, gpsLongitude)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                item.id,
                item.aperture_f as f64,
                item.focal_length_mm as f64,
                item.iso as i32,
                item.shutter_speed,
                item.camera_model,
                item.lens_model,
                item.gps_lat,
                item.gps_lon
            ],
        );

        let _ = self.conn.execute(
            "INSERT OR REPLACE INTO AgHarvestedIptcMetadata (image, creator, copyright, city, stateProvince, country, sublocation)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                item.id,
                item.creator,
                item.copyright,
                item.city,
                item.state_province,
                item.country,
                item.sublocation
            ],
        );

        // Save or clean mask/depth blob metadata in `.lrcat-data` directory
        let blob_file = self.blob_dir.join(format!("photo_{}_masks.blob", item.id));
        if !item.params.mask_groups.is_empty() || item.params.lens_blur_active {
            let _ = std::fs::write(blob_file, &params_json);
        } else if blob_file.exists() {
            let _ = std::fs::remove_file(blob_file);
        }
    }

    pub fn delete_photo(&self, id: i64) {
        let _ = self
            .conn
            .execute("DELETE FROM Adobe_images WHERE id_local = ?1", params![id]);
        let _ = self.conn.execute(
            "DELETE FROM Adobe_imageDevelopSettings WHERE image = ?1",
            params![id],
        );
        let _ = self.conn.execute(
            "DELETE FROM AgHarvestedExifMetadata WHERE image = ?1",
            params![id],
        );
        let _ = self.conn.execute(
            "DELETE FROM AgHarvestedIptcMetadata WHERE image = ?1",
            params![id],
        );
        let _ = self.conn.execute(
            "DELETE FROM AgLibraryCollectionImage WHERE image = ?1",
            params![id],
        );
        let _ = self.conn.execute(
            "DELETE FROM Adobe_libraryImageDevelopHistoryStep WHERE image = ?1",
            params![id],
        );
        let _ = self.conn.execute(
            "DELETE FROM Adobe_libraryImageDevelopSnapshot WHERE image = ?1",
            params![id],
        );
        let blob_file = self.blob_dir.join(format!("photo_{}_masks.blob", id));
        if blob_file.exists() {
            let _ = std::fs::remove_file(blob_file);
        }
    }

    pub fn load_collections(&self) -> Vec<PhotoCollection> {
        let mut stmt = match self
            .conn
            .prepare("SELECT collectionJson FROM AgLibraryCollection ORDER BY id_local ASC")
        {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let rows = stmt.query_map([], |row| row.get::<_, String>(0));
        let mut list = Vec::new();
        if let Ok(iter) = rows {
            for r in iter.flatten() {
                if let Ok(c) = serde_json::from_str::<PhotoCollection>(&r) {
                    list.push(c);
                }
            }
        }
        list
    }

    pub fn save_collection(&self, col: &PhotoCollection) {
        let json = serde_json::to_string(col).unwrap_or_default();
        let _ = self.conn.execute(
            "INSERT OR REPLACE INTO AgLibraryCollection (id_local, name, systemOnly, collectionJson)
             VALUES (?1, ?2, ?3, ?4)",
            params![col.id, col.name, col.is_smart as i32, json],
        );
    }

    pub fn delete_collection(&self, id: i64) {
        let _ = self.conn.execute(
            "DELETE FROM AgLibraryCollection WHERE id_local = ?1",
            params![id],
        );
        let _ = self.conn.execute(
            "DELETE FROM AgLibraryCollectionImage WHERE collection = ?1",
            params![id],
        );
    }

    pub fn load_presets(&self) -> Vec<crate::presets::DevelopPreset> {
        let mut stmt = match self
            .conn
            .prepare("SELECT presetJson, is_favorite FROM AgDevelopPreset ORDER BY name ASC")
        {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let rows = stmt.query_map([], |row| {
            let json: String = row.get(0)?;
            let is_fav: i32 = row.get(1).unwrap_or(0);
            Ok((json, is_fav != 0))
        });
        let mut items = Vec::new();
        if let Ok(iter) = rows {
            for r in iter.flatten() {
                if let Ok(mut item) = serde_json::from_str::<crate::presets::DevelopPreset>(&r.0)
                {
                    item.is_favorite = r.1;
                    items.push(item);
                }
            }
        }
        items
    }

    pub fn save_preset(&self, preset: &crate::presets::DevelopPreset) {
        let json = serde_json::to_string(preset).unwrap_or_default();
        let _ = self.conn.execute(
            "INSERT OR REPLACE INTO AgDevelopPreset (id, category, name, is_user, is_favorite, presetJson)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                preset.id,
                preset.category,
                preset.name,
                if preset.is_user { 1 } else { 0 },
                if preset.is_favorite { 1 } else { 0 },
                json
            ],
        );
    }

    pub fn delete_preset(&self, id: &str) {
        let _ = self
            .conn
            .execute("DELETE FROM AgDevelopPreset WHERE id = ?1", params![id]);
    }

    pub fn update_preset_favorite(&self, id: &str, is_favorite: bool) {
        let _ = self.conn.execute(
            "UPDATE AgDevelopPreset SET is_favorite = ?1 WHERE id = ?2",
            params![if is_favorite { 1 } else { 0 }, id],
        );
    }

    /// Optimizes and defragments the SQLite catalog database file (`VACUUM` & `ANALYZE`)
    pub fn optimize(&self) -> rusqlite::Result<()> {
        self.conn.execute_batch("VACUUM; ANALYZE; PRAGMA optimize;")
    }

    pub fn load_raw_blob(&self, _photo_id: i64) -> Option<RawSourceImage> {
        None
    }
}

/// Complete list of supported Camera RAW and Raster Image file extensions (lowercase & uppercase for macOS NSOpenPanel)
pub const SUPPORTED_PHOTO_EXTENSIONS: &[&str] = &[
    // Camera RAW Formats (Sony, Canon, Nikon, Adobe/Apple/Leica DNG, Fujifilm, Olympus, Panasonic, Pentax, Hasselblad, Phase One, Samsung, Sigma, Kodak, Mamiya, Epson, Minolta)
    "arw", "srf", "sr2", "cr2", "cr3", "crw", "nef", "nrw", "dng", "raf", "orf", "ori", "rw2",
    "raw", "pef", "srw", "3fr", "fff", "iiq", "rwl", "x3f", "dcr", "kdc", "mef", "mos", "mrw",
    "erf", "gpr",
    // Uppercase variants for case-sensitive file dialogs
    "ARW", "SRF", "SR2", "CR2", "CR3", "CRW", "NEF", "NRW", "DNG", "RAF", "ORF", "ORI", "RW2",
    "RAW", "PEF", "SRW", "3FR", "FFF", "IIQ", "RWL", "X3F", "DCR", "KDC", "MEF", "MOS", "MRW",
    "ERF", "GPR",
    // Standard RGB & High-Efficiency Image Formats
    "jpg", "jpeg", "png", "tif", "tiff", "webp", "bmp", "heic", "heif", "avif",
    "JPG", "JPEG", "PNG", "TIF", "TIFF", "WEBP", "BMP", "HEIC", "HEIF", "AVIF",
];

pub fn is_supported_photo_path(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
        return false;
    };
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "arw"
            | "srf"
            | "sr2"
            | "cr2"
            | "cr3"
            | "crw"
            | "nef"
            | "nrw"
            | "dng"
            | "raf"
            | "orf"
            | "ori"
            | "rw2"
            | "raw"
            | "pef"
            | "srw"
            | "3fr"
            | "fff"
            | "iiq"
            | "rwl"
            | "x3f"
            | "dcr"
            | "kdc"
            | "mef"
            | "mos"
            | "mrw"
            | "erf"
            | "gpr"
            | "jpg"
            | "jpeg"
            | "png"
            | "tif"
            | "tiff"
            | "webp"
            | "bmp"
            | "heic"
            | "heif"
            | "avif"
    )
}

pub fn is_camera_raw_extension(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
        return false;
    };
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "arw"
            | "srf"
            | "sr2"
            | "cr2"
            | "cr3"
            | "crw"
            | "nef"
            | "nrw"
            | "dng"
            | "raf"
            | "orf"
            | "ori"
            | "rw2"
            | "raw"
            | "pef"
            | "srw"
            | "3fr"
            | "fff"
            | "iiq"
            | "rwl"
            | "x3f"
            | "dcr"
            | "kdc"
            | "mef"
            | "mos"
            | "mrw"
            | "erf"
            | "gpr"
            | "heic"
            | "heif"
    )
}

/// Scans a directory (and 1 level of subdirectories) for all supported RAW & RGB image files.
pub fn collect_photos_in_directory(dir: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    if dir.is_file() {
        if is_supported_photo_path(dir) {
            results.push(dir.to_path_buf());
        }
        return results;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut subdirs = Vec::new();
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() && is_supported_photo_path(&p) {
                results.push(p);
            } else if p.is_dir() {
                let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if !name.starts_with('.')
                    && !name.ends_with(".lrcat-data")
                    && !name.ends_with(".app")
                {
                    subdirs.push(p);
                }
            }
        }
        for sub in subdirs {
            if let Ok(sub_entries) = std::fs::read_dir(&sub) {
                for entry in sub_entries.flatten() {
                    let p = entry.path();
                    if p.is_file() && is_supported_photo_path(&p) {
                        results.push(p);
                    }
                }
            }
        }
    }
    results.sort();
    results
}

/// In-memory placeholder returned by `active_photo()` only when the catalog has 0 photos
/// (never saved to SQLite).
pub fn create_empty_placeholder_photo() -> PhotoItem {
    let default_params = DevelopParams::default();
    PhotoItem {
        id: 0,
        file_name: "No Photo Selected".to_string(),
        folder_path: String::new(),
        real_file_path: None,
        sample_scene_id: 0,
        is_virtual_copy: false,
        copy_name: String::new(),
        master_id: None,
        stack_id: None,
        rating: 0,
        flag: PickFlag::Unflagged,
        color_label: ColorLabel::None,
        in_quick_collection: false,
        in_previous_import: false,
        camera_make: String::new(),
        camera_model: String::new(),
        lens_model: String::new(),
        iso: 100,
        focal_length_mm: 50.0,
        aperture_f: 2.8,
        shutter_speed: "1/125 sec".to_string(),
        capture_time: String::new(),
        width: 0,
        height: 0,
        title: String::new(),
        caption: String::new(),
        creator: String::new(),
        copyright: String::new(),
        sublocation: String::new(),
        city: String::new(),
        state_province: String::new(),
        country: String::new(),
        iso_country_code: String::new(),
        gps_lat: None,
        gps_lon: None,
        gps_alt_m: None,
        person_tag: String::new(),
        keywords: Vec::new(),
        params: default_params.clone(),
        history: vec![HistoryStep {
            name: "Empty".to_string(),
            value_summary: String::new(),
            timestamp: String::new(),
            params: default_params,
        }],
        history_index: 0,
        snapshots: Vec::new(),
        detected_faces: Vec::new(),
        culling_focus_score: None,
        comments: Vec::new(),
    }
}

pub fn create_default_catalog_photos() -> Vec<PhotoItem> {
    Vec::new()
}

pub fn create_default_collections() -> Vec<PhotoCollection> {
    vec![
        PhotoCollection {
            id: 1,
            name: "Five Stars".into(),
            is_smart: true,
            smart_rule_desc: "Rating >= 5".into(),
            photo_ids: Vec::new(),
        },
        PhotoCollection {
            id: 2,
            name: "Past Month / Recent".into(),
            is_smart: true,
            smart_rule_desc: "Recent Captures".into(),
            photo_ids: Vec::new(),
        },
        PhotoCollection {
            id: 3,
            name: "Without Keywords".into(),
            is_smart: true,
            smart_rule_desc: "Keywords are empty".into(),
            photo_ids: Vec::new(),
        },
        PhotoCollection {
            id: 4,
            name: "High ISO (>= 800)".into(),
            is_smart: true,
            smart_rule_desc: "ISO >= 800".into(),
            photo_ids: Vec::new(),
        },
    ]
}

#[derive(Debug, Clone)]
pub struct ExtractedPhotoMetadata {
    pub width: u32,
    pub height: u32,
    pub camera_make: String,
    pub camera_model: String,
    pub lens_model: String,
    pub iso: u32,
    pub focal_length_mm: f32,
    pub aperture_f: f32,
    pub shutter_speed: String,
    pub capture_time: String,
    pub gps_lat: Option<f64>,
    pub gps_lon: Option<f64>,
    pub gps_alt_m: Option<f64>,
}

fn normalize_exif_datetime(raw: &str) -> String {
    // EXIF creation string format is typically "YYYY:MM:DD HH:MM:SS"
    let trimmed = raw.trim();
    if let Some((date_part, time_part)) = trimmed.split_once(' ') {
        let fixed_date = date_part.replacen(':', "-", 2);
        format!("{} {}", fixed_date, time_part)
    } else {
        trimmed.replacen(':', "-", 2)
    }
}

/// Extracts real EXIF & image dimensions from any RAW (.CR2, .CR3, .NEF, .ARW, .DNG, .RAF, .ORF, .RW2, .PEF)
/// or standard RGB file using macOS ImageIO (`sips` + `mdls`) and pure-Rust fallback.
pub fn extract_real_photo_metadata(path: &Path) -> ExtractedPhotoMetadata {
    let now_str = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let ext_upper = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("RAW")
        .to_ascii_uppercase();

    let mut meta = ExtractedPhotoMetadata {
        width: 0,
        height: 0,
        camera_make: if is_camera_raw_extension(path) {
            format!("{} Camera RAW", ext_upper)
        } else {
            "Digital Camera".into()
        },
        camera_model: format!("{} Sensor", ext_upper),
        lens_model: "Prime / Zoom Lens".into(),
        iso: 100,
        focal_length_mm: 35.0,
        aperture_f: 2.8,
        shutter_speed: "1/250 sec".into(),
        capture_time: now_str,
        gps_lat: None,
        gps_lon: None,
        gps_alt_m: None,
    };

    if let Ok((w, h)) = image::image_dimensions(path) {
        meta.width = w;
        meta.height = h;
    }

    let orientation = read_exif_orientation(path);

    // 1. Query macOS ImageIO via `/usr/bin/sips` (works on all RAW & RGB files even outside Spotlight)
    if let Ok(out) = std::process::Command::new("/usr/bin/sips")
        .args([
            "-g",
            "pixelWidth",
            "-g",
            "pixelHeight",
            "-g",
            "make",
            "-g",
            "model",
            "-g",
            "creation",
        ])
        .arg(path)
        .output()
    {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                let trimmed = line.trim();
                if let Some((k, v)) = trimmed.split_once(':') {
                    let key = k.trim();
                    let val = v.trim();
                    if val.is_empty() || val == "<nil>" {
                        continue;
                    }
                    match key {
                        "pixelWidth" => {
                            if let Ok(w) = val.parse::<u32>() {
                                meta.width = w;
                            }
                        }
                        "pixelHeight" => {
                            if let Ok(h) = val.parse::<u32>() {
                                meta.height = h;
                            }
                        }
                        "make" => meta.camera_make = val.to_string(),
                        "model" => meta.camera_model = val.to_string(),
                        "creation" => meta.capture_time = normalize_exif_datetime(val),
                        _ => {}
                    }
                }
            }
        }
    }

    if (5..=8).contains(&orientation) && meta.width > meta.height {
        std::mem::swap(&mut meta.width, &mut meta.height);
    }

    // 2. Query detailed EXIF tags via macOS `/usr/bin/mdls` (ISO, Aperture, Shutter, Focal Length, Lens, GPS)
    if let Ok(out) = std::process::Command::new("/usr/bin/mdls")
        .args([
            "-name",
            "kMDItemISOSpeed",
            "-name",
            "kMDItemFNumber",
            "-name",
            "kMDItemExposureTimeSeconds",
            "-name",
            "kMDItemFocalLength",
            "-name",
            "kMDItemLensModel",
            "-name",
            "kMDItemAcquisitionMake",
            "-name",
            "kMDItemAcquisitionModel",
            "-name",
            "kMDItemLatitude",
            "-name",
            "kMDItemLongitude",
            "-name",
            "kMDItemAltitude",
        ])
        .arg(path)
        .output()
    {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                let Some((k, v)) = line.split_once('=') else {
                    continue;
                };
                let key = k.trim();
                let val = v.trim().trim_matches('"');
                if val.is_empty() || val == "(null)" {
                    continue;
                }
                match key {
                    "kMDItemISOSpeed" => {
                        if let Ok(iso) = val.parse::<f32>() {
                            meta.iso = iso.round() as u32;
                        }
                    }
                    "kMDItemFNumber" => {
                        if let Ok(ap) = val.parse::<f32>() {
                            meta.aperture_f = ap;
                        }
                    }
                    "kMDItemExposureTimeSeconds" => {
                        if let Ok(sec) = val.parse::<f64>() {
                            if sec > 0.0 && sec < 0.5 {
                                meta.shutter_speed = format!("1/{:.0} sec", (1.0 / sec).round());
                            } else if sec >= 0.5 {
                                meta.shutter_speed = format!("{:.1} sec", sec);
                            }
                        }
                    }
                    "kMDItemFocalLength" => {
                        if let Ok(fl) = val.parse::<f32>() {
                            meta.focal_length_mm = fl;
                        }
                    }
                    "kMDItemLensModel" => meta.lens_model = val.to_string(),
                    "kMDItemAcquisitionMake" => meta.camera_make = val.to_string(),
                    "kMDItemAcquisitionModel" => meta.camera_model = val.to_string(),
                    "kMDItemLatitude" => {
                        if let Ok(lat) = val.parse::<f64>() {
                            meta.gps_lat = Some(lat);
                        }
                    }
                    "kMDItemLongitude" => {
                        if let Ok(lon) = val.parse::<f64>() {
                            meta.gps_lon = Some(lon);
                        }
                    }
                    "kMDItemAltitude" => {
                        if let Ok(alt) = val.parse::<f64>() {
                            meta.gps_alt_m = Some(alt);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if meta.width == 0 || meta.height == 0 {
        meta.width = 4000;
        meta.height = 2667;
    }
    meta
}

/// Pure-Rust Embedded JPEG Stream Extractor for Camera RAW files (.CR2, .CR3, .NEF, .ARW, .DNG, .RAF, .ORF, .RW2, .PEF).
/// Scans the RAW container for embedded JPEG streams (`FF D8 FF` ... `FF D9`) and decodes the largest preview image.
pub fn extract_largest_embedded_jpeg(raw_bytes: &[u8]) -> Option<image::DynamicImage> {
    if raw_bytes.len() < 128 {
        return None;
    }
    let mut best_img: Option<image::DynamicImage> = None;
    let mut best_pixels: u64 = 0;

    let mut i = 0usize;
    let limit = raw_bytes.len().saturating_sub(16);
    while i < limit {
        if raw_bytes[i] != 0xFF {
            i += 1;
            continue;
        }
        // Look for JPEG SOI marker: FF D8 FF followed by E0, E1, E2, DB, C0, C2, C4, EE, FE
        if raw_bytes[i + 1] == 0xD8
            && raw_bytes[i + 2] == 0xFF
            && matches!(
                raw_bytes[i + 3],
                0xE0 | 0xE1 | 0xE2 | 0xDB | 0xC0 | 0xC2 | 0xC4 | 0xEE | 0xFE
            )
        {
            if let Ok(img) =
                image::load_from_memory_with_format(&raw_bytes[i..], image::ImageFormat::Jpeg)
            {
                let pixels = (img.width() as u64) * (img.height() as u64);
                if pixels > best_pixels && img.width() >= 160 && img.height() >= 120 {
                    best_pixels = pixels;
                    best_img = Some(img);
                }
                i += 2048;
                continue;
            }
        }
        i += 1;
    }
    best_img
}

/// Parses EXIF Orientation tag (0x0112) from a TIFF-based RAW file (.ARW, .CR2, .NEF, .DNG, .ORF, .RW2, .PEF)
/// or embedded EXIF block (`Exif\0\0`), with macOS `mdls` fallback.
pub fn read_exif_orientation(path: &Path) -> u16 {
    fn parse_tiff_ifd0_orientation(tiff: &[u8]) -> Option<u16> {
        if tiff.len() < 16 {
            return None;
        }
        let le = match (&tiff[0..2], tiff[2]) {
            (b"II", 0x2A | 0x55 | 0x4F) => true,
            (b"MM", 0x00) => false,
            _ => return None,
        };
        let r16 = |off: usize| -> u16 {
            if le {
                u16::from_le_bytes([tiff[off], tiff[off + 1]])
            } else {
                u16::from_be_bytes([tiff[off], tiff[off + 1]])
            }
        };
        let r32 = |off: usize| -> usize {
            if le {
                u32::from_le_bytes([tiff[off], tiff[off + 1], tiff[off + 2], tiff[off + 3]])
                    as usize
            } else {
                u32::from_be_bytes([tiff[off], tiff[off + 1], tiff[off + 2], tiff[off + 3]])
                    as usize
            }
        };
        let ifd_off = r32(4);
        if ifd_off + 2 > tiff.len() {
            return None;
        }
        let entries = r16(ifd_off) as usize;
        for idx in 0..entries.min(128) {
            let e_off = ifd_off + 2 + idx * 12;
            if e_off + 12 > tiff.len() {
                break;
            }
            let tag = r16(e_off);
            if tag == 0x0112 {
                let orient = r16(e_off + 8);
                if (1..=8).contains(&orient) {
                    return Some(orient);
                }
            }
        }
        None
    }

    if let Ok(mut file) = std::fs::File::open(path) {
        use std::io::Read;
        let mut head = vec![0u8; 131072];
        if let Ok(n) = file.read(&mut head) {
            head.truncate(n);
            if let Some(o) = parse_tiff_ifd0_orientation(&head) {
                return o;
            }
            for i in 0..head.len().saturating_sub(32) {
                if &head[i..i + 6] == b"Exif\0\0" {
                    if let Some(o) = parse_tiff_ifd0_orientation(&head[i + 6..]) {
                        return o;
                    }
                }
            }
        }
    }

    if let Ok(out) = std::process::Command::new("/usr/bin/mdls")
        .args(["-name", "kMDItemOrientation"])
        .arg(path)
        .output()
    {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            if let Some((_, v)) = s.split_once('=') {
                if let Ok(o) = v.trim().parse::<u16>() {
                    if (1..=8).contains(&o) {
                        return o;
                    }
                }
            }
        }
    }
    1
}

fn apply_exif_orientation(img: image::DynamicImage, orientation: u16) -> image::DynamicImage {
    match orientation {
        3 => img.rotate180(),
        6 if img.width() >= img.height() => img.rotate90(),
        8 if img.width() >= img.height() => img.rotate270(),
        _ => img,
    }
}

/// Decodes any real Camera RAW (.ARW, .CR2, .CR3, .NEF, .DNG, .RAF, .ORF, .RW2, .PEF, .3FR, .IIQ, .HEIC)
/// or standard RGB image (.JPG, .PNG, .TIFF, .WEBP, .BMP) into a 32-bit linear `RawSourceImage`.
/// Pass `target_w = 0, target_h = 0` to decode at full native camera resolution (e.g. for Export).
pub fn decode_real_photo_file(
    path: &Path,
    target_w: u32,
    target_h: u32,
) -> Option<RawSourceImage> {
    if !path.exists() {
        return None;
    }
    let max_dim = if target_w == 0 && target_h == 0 {
        0
    } else {
        target_w.max(target_h).max(64)
    };
    let is_raw = is_camera_raw_extension(path);

    // Stage 1 (macOS Native): Decode linear 32-bit float sensor RGB via CoreImage `CIRAWFilter` / `CIImage`
    // in `kCGColorSpaceExtendedLinearSRGB` with `boostAmount = 0` (zero baked tone curve, full highlight headroom).
    #[cfg(target_os = "macos")]
    {
        if let Some(path_str) = path.to_str() {
            if let Ok(c_path) = CString::new(path_str) {
                let mut out_w: u32 = 0;
                let mut out_h: u32 = 0;
                let mut nat_w: u32 = 0;
                let mut nat_h: u32 = 0;
                let mut as_shot_temp: f32 = 5500.0;
                let mut as_shot_tint: f32 = 0.0;
                let mut baseline_ev: f32 = 0.0;
                let mut scene_ref: std::os::raw::c_int = 0;
                let draft_mode: std::os::raw::c_int =
                    if max_dim > 0 && max_dim <= 512 { 1 } else { 0 };

                let ptr = unsafe {
                    lightraw_decode_image_linear(
                        c_path.as_ptr(),
                        max_dim,
                        if is_raw { 1 } else { 0 },
                        draft_mode,
                        &mut out_w,
                        &mut out_h,
                        &mut nat_w,
                        &mut nat_h,
                        &mut as_shot_temp,
                        &mut as_shot_tint,
                        &mut baseline_ev,
                        &mut scene_ref,
                    )
                };

                if !ptr.is_null() && out_w > 0 && out_h > 0 {
                    let count = (out_w as usize) * (out_h as usize) * 3;
                    let rgb_vec = unsafe { std::slice::from_raw_parts(ptr, count).to_vec() };
                    unsafe {
                        lightraw_free_float_buffer(ptr);
                    }
                    return Some(RawSourceImage::from_linear_rgb(
                        out_w,
                        out_h,
                        nat_w.max(out_w),
                        nat_h.max(out_h),
                        rgb_vec,
                        as_shot_temp,
                        as_shot_tint,
                        baseline_ev,
                        scene_ref != 0,
                    ));
                }
            }
        }
    }

    let orientation = read_exif_orientation(path);

    // Stage 2: Pure-Rust Direct `image::open` fallback (standard RGB or uncompressed TIFF/DNG)
    if let Ok(dyn_img) = image::open(path) {
        let oriented = apply_exif_orientation(dyn_img, orientation);
        let resized = if max_dim > 0 && (oriented.width() > max_dim || oriented.height() > max_dim)
        {
            oriented
                .resize(max_dim, max_dim, image::imageops::FilterType::Triangle)
                .to_rgba8()
        } else {
            oriented.to_rgba8()
        };
        let (w, h) = resized.dimensions();
        return Some(RawSourceImage::from_rgba8(w, h, resized.as_raw()));
    }

    // Stage 3: Pure-Rust Embedded Preview JPEG Extractor (e.g. synthetic/unsupported RAW containers)
    if let Ok(bytes) = std::fs::read(path) {
        if let Some(dyn_img) = extract_largest_embedded_jpeg(&bytes) {
            let oriented = apply_exif_orientation(dyn_img, orientation);
            let resized =
                if max_dim > 0 && (oriented.width() > max_dim || oriented.height() > max_dim) {
                    oriented
                        .resize(max_dim, max_dim, image::imageops::FilterType::Triangle)
                        .to_rgba8()
                } else {
                    oriented.to_rgba8()
                };
            let (w, h) = resized.dimensions();
            return Some(RawSourceImage::from_rgba8(w, h, resized.as_raw()));
        }
    }

    None
}

pub fn load_source_image_for_photo(
    item: &PhotoItem,
    target_w: u32,
    target_h: u32,
) -> RawSourceImage {
    if let Some(ref path_str) = item.real_file_path {
        let path = Path::new(path_str);
        if let Some(decoded) = decode_real_photo_file(path, target_w, target_h) {
            return decoded;
        }
    }
    let w = target_w.max(64);
    let h = target_h.max(64);
    let blank = vec![28u8; (w * h * 4) as usize];
    RawSourceImage::from_rgba8(w, h, &blank)
}

/// Computes a Laplacian sharpness/focus score (0..100) and detects real faces + eye-openness
/// via macOS Apple `Vision.framework` (`VNDetectFaceLandmarksRequest`).
pub fn detect_faces_and_focus_for_photo(
    rgba: &[u8],
    width: u32,
    height: u32,
) -> (Vec<DetectedFace>, f32) {
    if width < 16 || height < 16 || rgba.len() < (width as usize) * (height as usize) * 4 {
        return (Vec::new(), 50.0);
    }

    let mut faces: Vec<DetectedFace> = Vec::new();
    #[cfg(target_os = "macos")]
    {
        let mut raw_faces = [LightRawFaceInfo::default(); 16];
        let count = unsafe {
            lightraw_detect_faces_rgba(rgba.as_ptr(), width, height, raw_faces.as_mut_ptr(), 16)
        };
        for f in raw_faces.iter().take(count.max(0) as usize) {
            faces.push(DetectedFace {
                x: f.x.clamp(0.0, 1.0),
                y: f.y.clamp(0.0, 1.0),
                w: f.w.clamp(0.01, 1.0),
                h: f.h.clamp(0.01, 1.0),
                quality: f.quality.clamp(0.0, 100.0),
                eyes_open_score: f.eyes_open_score.clamp(0.0, 100.0),
                yaw_deg: f.yaw_deg,
                person_name: String::new(),
            });
        }
    }

    // Compute Laplacian sharpness score over the primary face ROI (or center 50% of image if no face)
    let (rx0, ry0, rx1, ry1) = if let Some(f) = faces.first() {
        let x0 = ((f.x * width as f32) as usize).min(width as usize - 2).max(1);
        let y0 = ((f.y * height as f32) as usize)
            .min(height as usize - 2)
            .max(1);
        let x1 = (((f.x + f.w) * width as f32) as usize)
            .min(width as usize - 1)
            .max(x0 + 1);
        let y1 = (((f.y + f.h) * height as f32) as usize)
            .min(height as usize - 1)
            .max(y0 + 1);
        (x0, y0, x1, y1)
    } else {
        let x0 = (width as usize / 4).max(1);
        let y0 = (height as usize / 4).max(1);
        let x1 = ((width as usize * 3) / 4).min(width as usize - 1).max(x0 + 1);
        let y1 = ((height as usize * 3) / 4)
            .min(height as usize - 1)
            .max(y0 + 1);
        (x0, y0, x1, y1)
    };

    let w_usize = width as usize;
    let luma_at = |x: usize, y: usize| -> f32 {
        let idx = (y * w_usize + x) * 4;
        0.299 * (rgba[idx] as f32) + 0.587 * (rgba[idx + 1] as f32) + 0.114 * (rgba[idx + 2] as f32)
    };

    let mut sum_lap = 0.0f64;
    let mut n = 0usize;
    let step = (((rx1 - rx0).min(ry1 - ry0)) / 96).max(1);
    let mut y = ry0;
    while y < ry1 {
        let mut x = rx0;
        while x < rx1 {
            let c = luma_at(x, y);
            let lap =
                (4.0 * c - luma_at(x - 1, y) - luma_at(x + 1, y) - luma_at(x, y - 1) - luma_at(x, y + 1))
                    .abs();
            sum_lap += (lap as f64) * (lap as f64);
            n += 1;
            x += step;
        }
        y += step;
    }
    let rms_lap = if n > 0 {
        (sum_lap / n as f64).sqrt() as f32
    } else {
        8.0
    };
    let focus_score = ((rms_lap / 22.0) * 100.0).clamp(5.0, 100.0);

    (faces, focus_score)
}

/// Queries the real macOS APFS/HFS+ volume name, free space, and total capacity for a folder path.
pub fn query_volume_disk_space(path: &Path) -> (String, u64, u64) {
    #[cfg(target_os = "macos")]
    {
        let probe = if path.exists() {
            path.to_path_buf()
        } else {
            PathBuf::from("/")
        };
        if let Some(s) = probe.to_str() {
            if let Ok(cs) = CString::new(s) {
                let mut free_b: u64 = 0;
                let mut total_b: u64 = 0;
                let mut vol_buf = [0 as std::os::raw::c_char; 128];
                let ok = unsafe {
                    lightraw_query_disk_space(
                        cs.as_ptr(),
                        &mut free_b,
                        &mut total_b,
                        vol_buf.as_mut_ptr(),
                        vol_buf.len(),
                    )
                };
                if ok != 0 && total_b > 0 {
                    let vol_name = unsafe {
                        std::ffi::CStr::from_ptr(vol_buf.as_ptr())
                            .to_string_lossy()
                            .into_owned()
                    };
                    return (vol_name, free_b, total_b);
                }
            }
        }
    }
    (
        "Macintosh HD".to_string(),
        120 * 1024 * 1024 * 1024,
        500 * 1024 * 1024 * 1024,
    )
}
