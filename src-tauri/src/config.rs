use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

pub const STEAM_APP_ID: &str = "1089090";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum LaunchMode {
    Steam,
    Executable,
}

impl Default for LaunchMode {
    fn default() -> Self {
        Self::Steam
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledFile {
    pub relative_path: String,
    pub backup_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModProfile {
    pub id: String,
    pub name: String,
    #[serde(default = "default_profile_icon")]
    pub icon: String,
    #[serde(default = "default_profile_color")]
    pub color: String,
    pub enabled_mod_keys: Vec<String>,
}

fn default_profile_icon() -> String {
    "sparkles".to_string()
}

fn default_profile_color() -> String {
    "violet".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LauncherConfig {
    pub launch_mode: LaunchMode,
    pub game_folder: Option<String>,
    pub game_executable_path: Option<String>,
    pub modloader_repo: String,
    pub modloader_release: Option<String>,
    #[serde(default)]
    pub modloader_sha256: Option<String>,
    #[serde(default)]
    pub latest_modloader_sha256: Option<String>,
    #[serde(default)]
    pub latest_modloader_sha256_checked_at: Option<String>,
    pub installed_files: Vec<InstalledFile>,
    pub last_launch_at: Option<String>,
    #[serde(default)]
    pub mod_profiles: Vec<ModProfile>,
    #[serde(default)]
    pub debug_logs: bool,
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            launch_mode: LaunchMode::Steam,
            game_folder: None,
            game_executable_path: None,
            modloader_repo: "Glubus/oppw4-patcher".to_string(),
            modloader_release: None,
            modloader_sha256: None,
            latest_modloader_sha256: None,
            latest_modloader_sha256_checked_at: None,
            installed_files: Vec::new(),
            last_launch_at: None,
            mod_profiles: Vec::new(),
            debug_logs: false,
        }
    }
}

pub fn app_data_dir() -> Result<PathBuf, String> {
    let base =
        dirs::data_dir().ok_or_else(|| "Could not resolve user data directory".to_string())?;
    Ok(base.join("oppw4-launcher"))
}

pub fn config_path() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("config.json"))
}

pub fn backup_dir() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("backups"))
}

pub fn load_config() -> Result<LauncherConfig, String> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(LauncherConfig::default());
    }
    let raw = fs::read_to_string(&path).map_err(|err| format!("Could not read config: {err}"))?;
    serde_json::from_str(&raw).map_err(|err| format!("Could not parse config: {err}"))
}

pub fn save_config(config: &LauncherConfig) -> Result<(), String> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("Could not create config directory: {err}"))?;
    }
    let raw = serde_json::to_string_pretty(config)
        .map_err(|err| format!("Could not serialize config: {err}"))?;
    fs::write(path, raw).map_err(|err| format!("Could not write config: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launcher_config_defaults_to_steam_and_patcher_repo() {
        let config = LauncherConfig::default();

        assert_eq!(config.launch_mode, LaunchMode::Steam);
        assert_eq!(config.modloader_repo, "Glubus/oppw4-patcher");
        assert!(!config.debug_logs);
        assert!(config.installed_files.is_empty());
        assert!(config.mod_profiles.is_empty());
    }

    #[test]
    fn mod_profile_deserialization_fills_visual_defaults() {
        let raw = r##"{
            "id": "default",
            "name": "Default",
            "enabledModKeys": ["id:law"]
        }"##;

        let profile: ModProfile = serde_json::from_str(raw).unwrap();

        assert_eq!(profile.icon, "sparkles");
        assert_eq!(profile.color, "violet");
        assert_eq!(profile.enabled_mod_keys, vec!["id:law"]);
    }
}
