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
pub struct LauncherConfig {
  pub launch_mode: LaunchMode,
  #[serde(default = "default_site_url")]
  pub site_url: String,
  pub game_folder: Option<String>,
  pub game_executable_path: Option<String>,
  pub modloader_repo: String,
  pub modloader_release: Option<String>,
  pub installed_files: Vec<InstalledFile>,
  pub last_launch_at: Option<String>,
}

impl Default for LauncherConfig {
  fn default() -> Self {
    Self {
      launch_mode: LaunchMode::Steam,
      site_url: default_site_url(),
      game_folder: None,
      game_executable_path: None,
      modloader_repo: "Glubus/oppw4-modloader".to_string(),
      modloader_release: None,
      installed_files: Vec::new(),
      last_launch_at: None,
    }
  }
}

fn default_site_url() -> String {
  "https://oppw4.prism.am".to_string()
}

pub fn app_data_dir() -> Result<PathBuf, String> {
  let base = dirs::data_dir().ok_or_else(|| "Could not resolve user data directory".to_string())?;
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
    fs::create_dir_all(parent).map_err(|err| format!("Could not create config directory: {err}"))?;
  }
  let raw = serde_json::to_string_pretty(config).map_err(|err| format!("Could not serialize config: {err}"))?;
  fs::write(path, raw).map_err(|err| format!("Could not write config: {err}"))
}
