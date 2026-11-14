// ============================================================================
// LightRaw — Cross-Platform Self-Updater (GitHub Releases)
// Supports macOS (.dmg / .app in-place), Windows (Inno /SILENT in-place),
// and Linux (AppImage / .deb in-place) without uninstalling existing catalog.
// ============================================================================

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GITHUB_LATEST_API: &str = "https://api.github.com/repos/moesstudio/LightRaw/releases/latest";
pub const GITHUB_RELEASES_PAGE: &str = "https://github.com/moesstudio/LightRaw/releases/latest";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdateAsset {
    pub name: String,
    pub url: String,
    pub size: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteRelease {
    pub tag_name: String,
    pub version: String,
    pub title: String,
    pub notes: String,
    pub html_url: String,
    pub asset: Option<UpdateAsset>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateCheckOutcome {
    UpToDate { current: String, latest_tag: String },
    Available(RemoteRelease),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplyUpdateOutcome {
    /// Detached installer/swap script spawned; caller should flush catalog DB & exit cleanly.
    Relaunching(String),
    /// Downloaded / opened external installer cleanly; no forced exit required.
    ManualStep(String),
}

#[derive(Clone, Debug)]
pub enum UpdaterUiState {
    Idle,
    Checking,
    UpToDate { current: String, latest_tag: String },
    Available(RemoteRelease),
    Installing(String),
    Notice(String),
    Error(String),
}

/// Parses a semantic version string (e.g. "v1.2.1-beta") into numeric components.
pub fn parse_semver(ver: &str) -> Vec<u32> {
    let cleaned = ver
        .trim()
        .trim_start_matches(['v', 'V'])
        .split(['-', '+'])
        .next()
        .unwrap_or("");
    if cleaned.is_empty() {
        return vec![0, 0, 0];
    }
    cleaned
        .split('.')
        .take(4)
        .map(|seg| {
            let digits: String = seg.chars().take_while(|c| c.is_ascii_digit()).collect();
            digits.parse::<u32>().unwrap_or(0)
        })
        .collect()
}

/// Returns `true` strictly when `remote` is a newer SemVer release than `current`.
pub fn is_newer_version(remote: &str, current: &str) -> bool {
    let r = parse_semver(remote);
    let c = parse_semver(current);
    let len = r.len().max(c.len());
    for i in 0..len {
        let rv = *r.get(i).unwrap_or(&0);
        let cv = *c.get(i).unwrap_or(&0);
        if rv != cv {
            return rv > cv;
        }
    }
    false
}

/// Picks the best platform installer / bundle asset from GitHub Release assets.
pub fn pick_desktop_asset(
    assets: &[UpdateAsset],
    os: &str,
    _arch: &str,
    running_appimage: bool,
) -> Option<UpdateAsset> {
    let find_matching = |suffixes: &[&str]| -> Option<UpdateAsset> {
        for s in suffixes {
            if let Some(found) = assets
                .iter()
                .find(|a| a.name.to_ascii_lowercase().ends_with(s))
            {
                return Some(found.clone());
            }
        }
        None
    };

    match os {
        "macos" => find_matching(&["macos.dmg", ".dmg"]),
        "windows" => find_matching(&["windows-setup.exe", ".exe", ".msi"]),
        "linux" => {
            if running_appimage {
                find_matching(&[".appimage", ".deb", ".tar.gz"])
            } else {
                find_matching(&[".deb", ".appimage", ".tar.gz"])
            }
        }
        _ => None,
    }
}

fn fetch_json_text(url: &str) -> Result<String, String> {
    let curl_out = Command::new("curl")
        .args([
            "-fsSL",
            "--connect-timeout",
            "10",
            "--max-time",
            "20",
            "-H",
            "Accept: application/vnd.github+json",
            "-H",
            &format!("User-Agent: LightRaw-Desktop/{CURRENT_VERSION}"),
            url,
        ])
        .output();

    if let Ok(out) = curl_out {
        if out.status.success() {
            return String::from_utf8(out.stdout)
                .map_err(|e| format!("Invalid UTF-8 response: {e}"));
        }
    }

    #[cfg(target_os = "windows")]
    {
        let ps = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; \
                     (Invoke-WebRequest -Uri '{}' -Headers @{{'Accept'='application/vnd.github+json'; 'User-Agent'='LightRaw-Desktop'}} -UseBasicParsing).Content",
                    url.replace('\'', "''")
                ),
            ])
            .output()
            .map_err(|e| format!("Failed invoking curl/PowerShell: {e}"))?;
        if ps.status.success() {
            return String::from_utf8(ps.stdout)
                .map_err(|e| format!("Invalid UTF-8 response: {e}"));
        }
        return Err(String::from_utf8_lossy(&ps.stderr).trim().to_string());
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Unable to contact GitHub Releases (curl request failed).".to_string())
    }
}

fn download_file(url: &str, dest: &Path) -> Result<(), String> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Create dir failed: {e}"))?;
    }
    let curl_res = Command::new("curl")
        .args([
            "-fSL",
            "--connect-timeout",
            "15",
            "--max-time",
            "300",
            "-H",
            &format!("User-Agent: LightRaw-Desktop/{CURRENT_VERSION}"),
            "-o",
        ])
        .arg(dest)
        .arg(url)
        .output();

    if let Ok(out) = curl_res {
        if out.status.success() && dest.exists() {
            return Ok(());
        }
    }

    #[cfg(target_os = "windows")]
    {
        let ps = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; \
                     Invoke-WebRequest -Uri '{}' -OutFile '{}' -UseBasicParsing",
                    url.replace('\'', "''"),
                    dest.display().to_string().replace('\'', "''")
                ),
            ])
            .output()
            .map_err(|e| format!("Download failed: {e}"))?;
        if ps.status.success() && dest.exists() {
            return Ok(());
        }
        return Err(String::from_utf8_lossy(&ps.stderr).trim().to_string());
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Failed downloading update installer from GitHub.".to_string())
    }
}

/// Checks GitHub Releases API for a newer version than `CURRENT_VERSION`.
pub fn check_latest_release() -> Result<UpdateCheckOutcome, String> {
    let raw = fetch_json_text(GITHUB_LATEST_API)?;
    let root: Value = serde_json::from_str(&raw).map_err(|e| format!("JSON parse error: {e}"))?;

    let tag_name = root
        .get("tag_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if tag_name.is_empty() {
        return Err("Latest release did not contain a tag_name.".to_string());
    }
    let version = tag_name.trim_start_matches(['v', 'V']).to_string();
    let title = root
        .get("name")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(&tag_name)
        .to_string();
    let notes = root
        .get("body")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .chars()
        .take(1100)
        .collect::<String>();
    let html_url = root
        .get("html_url")
        .and_then(|v| v.as_str())
        .unwrap_or(GITHUB_RELEASES_PAGE)
        .to_string();

    let mut assets = Vec::new();
    if let Some(arr) = root.get("assets").and_then(|v| v.as_array()) {
        for item in arr {
            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let url = item
                .get("browser_download_url")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let size = item.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
            if !name.is_empty() && !url.is_empty() {
                assets.push(UpdateAsset {
                    name: name.to_string(),
                    url: url.to_string(),
                    size,
                });
            }
        }
    }

    let running_appimage = std::env::var_os("APPIMAGE").is_some();
    let asset = pick_desktop_asset(
        &assets,
        std::env::consts::OS,
        std::env::consts::ARCH,
        running_appimage,
    );

    if is_newer_version(&version, CURRENT_VERSION) {
        Ok(UpdateCheckOutcome::Available(RemoteRelease {
            tag_name,
            version,
            title,
            notes,
            html_url,
            asset,
        }))
    } else {
        Ok(UpdateCheckOutcome::UpToDate {
            current: CURRENT_VERSION.to_string(),
            latest_tag: tag_name,
        })
    }
}

/// Downloads the release asset and applies an in-place upgrade cleanly when supported.
pub fn download_and_install_release(release: &RemoteRelease) -> Result<ApplyUpdateOutcome, String> {
    let asset = release
        .asset
        .as_ref()
        .ok_or_else(|| "No installer asset found for this OS in release.".to_string())?;

    let stage_dir = std::env::temp_dir().join("lightraw_updater");
    let _ = std::fs::remove_dir_all(&stage_dir);
    std::fs::create_dir_all(&stage_dir).map_err(|e| format!("Failed staging dir: {e}"))?;
    let downloaded_path = stage_dir.join(&asset.name);
    download_file(&asset.url, &downloaded_path)?;

    apply_downloaded_installer(&downloaded_path, &stage_dir)
}

#[cfg(target_os = "macos")]
fn find_enclosing_app_bundle() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let mut cur = exe.as_path();
    while let Some(parent) = cur.parent() {
        if parent
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("app"))
            .unwrap_or(false)
        {
            return Some(parent.to_path_buf());
        }
        cur = parent;
    }
    None
}

#[cfg(target_os = "macos")]
fn apply_downloaded_installer(dmg_path: &Path, stage_dir: &Path) -> Result<ApplyUpdateOutcome, String> {
    if let Some(installed_app) = find_enclosing_app_bundle() {
        if let Some(parent_dir) = installed_app.parent() {
            // Verify write permissions to bundle directory before spawning background swapper.
            let probe = parent_dir.join(".lightraw_write_probe");
            if std::fs::write(&probe, b"ok").is_ok() {
                let _ = std::fs::remove_file(&probe);
                let mount_dir = stage_dir.join("mnt");
                std::fs::create_dir_all(&mount_dir)
                    .map_err(|e| format!("Mount dir error: {e}"))?;
                let attach = Command::new("hdiutil")
                    .args(["attach", "-nobrowse", "-quiet", "-mountpoint"])
                    .arg(&mount_dir)
                    .arg(dmg_path)
                    .output()
                    .map_err(|e| format!("Failed running hdiutil: {e}"))?;
                if attach.status.success() {
                    let src_app = mount_dir.join("LightRaw.app");
                    if src_app.is_dir() {
                        let pid = std::process::id();
                        let script_path = stage_dir.join("apply_update.sh");
                        let script = format!(
                            "#!/bin/sh\n\
                             while kill -0 {pid} 2>/dev/null; do sleep 0.2; done\n\
                             rm -rf \"{dst}.new\"\n\
                             ditto \"{src}\" \"{dst}.new\" && rm -rf \"{dst}\" && mv \"{dst}.new\" \"{dst}\"\n\
                             hdiutil detach \"{mnt}\" -quiet >/dev/null 2>&1 || true\n\
                             open \"{dst}\"\n",
                            pid = pid,
                            src = src_app.display(),
                            dst = installed_app.display(),
                            mnt = mount_dir.display(),
                        );
                        std::fs::write(&script_path, script)
                            .map_err(|e| format!("Failed writing update helper: {e}"))?;
                        Command::new("chmod")
                            .arg("+x")
                            .arg(&script_path)
                            .status()
                            .map_err(|e| format!("chmod failed: {e}"))?;
                        Command::new("/bin/sh")
                            .arg(&script_path)
                            .spawn()
                            .map_err(|e| format!("Spawn updater failed: {e}"))?;
                        return Ok(ApplyUpdateOutcome::Relaunching(
                            "Applying update in place & relaunching LightRaw.app...".to_string(),
                        ));
                    }
                    let _ = Command::new("hdiutil")
                        .args(["detach", "-quiet"])
                        .arg(&mount_dir)
                        .status();
                }
            }
        }
    }

    Command::new("open")
        .arg(dmg_path)
        .spawn()
        .map_err(|e| format!("Failed opening DMG: {e}"))?;
    Ok(ApplyUpdateOutcome::ManualStep(format!(
        "Downloaded & opened {}. Drag LightRaw.app to Applications to finish updating without deleting your catalog.",
        dmg_path.file_name().and_then(|s| s.to_str()).unwrap_or("DMG")
    )))
}

#[cfg(target_os = "windows")]
fn apply_downloaded_installer(
    installer_path: &Path,
    _stage_dir: &Path,
) -> Result<ApplyUpdateOutcome, String> {
    let lower = installer_path.to_string_lossy().to_ascii_lowercase();
    if lower.ends_with(".exe") {
        Command::new(installer_path)
            .args(["/SILENT", "/CLOSEAPPLICATIONS", "/RESTARTAPPLICATIONS"])
            .spawn()
            .map_err(|e| format!("Failed launching setup installer: {e}"))?;
        return Ok(ApplyUpdateOutcome::Relaunching(
            "Running silent in-place upgrade & restarting LightRaw...".to_string(),
        ));
    }
    Command::new("msiexec")
        .arg("/i")
        .arg(installer_path)
        .spawn()
        .map_err(|e| format!("Failed launching MSI installer: {e}"))?;
    Ok(ApplyUpdateOutcome::Relaunching(
        "Launching Windows Installer upgrade...".to_string(),
    ))
}

#[cfg(target_os = "linux")]
fn apply_downloaded_installer(
    installer_path: &Path,
    _stage_dir: &Path,
) -> Result<ApplyUpdateOutcome, String> {
    let lower = installer_path.to_string_lossy().to_ascii_lowercase();
    if lower.ends_with(".appimage") {
        let _ = Command::new("chmod").arg("+x").arg(installer_path).status();
        if let Ok(appimage_target) = std::env::var("APPIMAGE") {
            let target_path = PathBuf::from(&appimage_target);
            if std::fs::copy(installer_path, &target_path).is_ok() {
                let _ = Command::new("chmod").arg("+x").arg(&target_path).status();
                let _ = Command::new(&target_path).spawn();
                return Ok(ApplyUpdateOutcome::Relaunching(
                    "Updated AppImage in place & relaunching LightRaw...".to_string(),
                ));
            }
        }
        let _ = Command::new(installer_path).spawn();
        return Ok(ApplyUpdateOutcome::Relaunching(
            "Launching updated LightRaw AppImage...".to_string(),
        ));
    }

    if lower.ends_with(".deb") {
        if Command::new("pkexec")
            .args(["apt", "install", "-y"])
            .arg(installer_path)
            .spawn()
            .is_ok()
        {
            return Ok(ApplyUpdateOutcome::Relaunching(
                "Installing .deb package update in place...".to_string(),
            ));
        }
    }

    let _ = Command::new("xdg-open").arg(installer_path).spawn();
    Ok(ApplyUpdateOutcome::ManualStep(format!(
        "Downloaded update to {}",
        installer_path.display()
    )))
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn apply_downloaded_installer(
    installer_path: &Path,
    _stage_dir: &Path,
) -> Result<ApplyUpdateOutcome, String> {
    Ok(ApplyUpdateOutcome::ManualStep(format!(
        "Downloaded update to {}",
        installer_path.display()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_comparison_and_asset_picking() {
        assert!(is_newer_version("v1.2.1", "1.2.0"));
        assert!(is_newer_version("v1.10.0", "1.2.0"));
        assert!(!is_newer_version("v1.2.0", "1.2.0"));
        assert!(!is_newer_version("1.1.0", "1.2.0"));

        let assets = vec![
            UpdateAsset {
                name: "LightRaw-1.2.1-macOS.dmg".into(),
                url: "https://example.com/dmg".into(),
                size: 100,
            },
            UpdateAsset {
                name: "LightRaw-1.2.1-Windows-Setup.exe".into(),
                url: "https://example.com/exe".into(),
                size: 100,
            },
            UpdateAsset {
                name: "lightraw_1.2.1_amd64.deb".into(),
                url: "https://example.com/deb".into(),
                size: 100,
            },
            UpdateAsset {
                name: "LightRaw-1.2.1-x86_64.AppImage".into(),
                url: "https://example.com/appimage".into(),
                size: 100,
            },
        ];
        assert_eq!(
            pick_desktop_asset(&assets, "macos", "aarch64", false)
                .map(|a| a.name),
            Some("LightRaw-1.2.1-macOS.dmg".into())
        );
        assert_eq!(
            pick_desktop_asset(&assets, "windows", "x86_64", false)
                .map(|a| a.name),
            Some("LightRaw-1.2.1-Windows-Setup.exe".into())
        );
        assert_eq!(
            pick_desktop_asset(&assets, "linux", "x86_64", true)
                .map(|a| a.name),
            Some("LightRaw-1.2.1-x86_64.AppImage".into())
        );
        assert_eq!(
            pick_desktop_asset(&assets, "linux", "x86_64", false)
                .map(|a| a.name),
            Some("lightraw_1.2.1_amd64.deb".into())
        );
    }
}
