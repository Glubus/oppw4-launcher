mod config;
mod installer;
mod steam;

use config::{load_config as read_config, save_config as write_config, LaunchMode, LauncherConfig, STEAM_APP_ID};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, path::PathBuf, process::Command};

const API_BASE: &str = "https://oppw4.prism.am/api";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LauncherState {
  config: LauncherConfig,
  detected_game: Option<steam::DetectedGame>,
  modloader_status: String,
  latest_release: Option<installer::ReleaseInfo>,
  needs_patcher_update: bool,
  installed_mods: Vec<InstalledMod>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InstalledMod {
  name: String,
  kind: String,
  path: String,
  enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToggleModRequest {
  path: String,
  enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiRequest {
  method: String,
  path: String,
  body: Option<String>,
  token: Option<String>,
}

#[tauri::command]
fn get_launcher_state() -> Result<LauncherState, String> {
  let mut config = read_config()?;
  let detected_game = steam::detect_oppw4();
  if config.game_folder.is_none() {
    if let Some(game) = &detected_game {
      config.game_folder = Some(game.game_folder.clone());
      config.game_executable_path = game.executable_path.clone();
      write_config(&config)?;
    }
  }
  let modloader_status = modloader_status(&config);
  let installed_mods = installed_mods(&config);
  let latest_release = installer::latest_release_info(&config.modloader_repo).ok().flatten();
  let needs_patcher_update = latest_release
    .as_ref()
    .is_some_and(|release| config.modloader_release.as_deref() != Some(release.tag_name.as_str()));
  Ok(LauncherState { config, detected_game, modloader_status, latest_release, needs_patcher_update, installed_mods })
}

#[tauri::command]
fn save_launcher_config(config: LauncherConfig) -> Result<LauncherConfig, String> {
  write_config(&config)?;
  Ok(config)
}

#[tauri::command]
fn detect_game() -> Result<Option<steam::DetectedGame>, String> {
  Ok(steam::detect_oppw4())
}

#[tauri::command]
fn launch_game() -> Result<(), String> {
  let mut config = read_config()?;
  match config.launch_mode {
    LaunchMode::Steam => open_steam_uri()?,
    LaunchMode::Executable => {
      let executable = config.game_executable_path.clone().ok_or_else(|| "Set a game executable path first.".to_string())?;
      let executable = PathBuf::from(executable);
      if !executable.exists() {
        return Err("Configured executable does not exist.".to_string());
      }
      Command::new(&executable)
        .current_dir(executable.parent().unwrap_or_else(|| std::path::Path::new(".")))
        .spawn()
        .map_err(|err| format!("Could not launch executable: {err}"))?;
    }
  }
  config.last_launch_at = Some(now_label());
  write_config(&config)
}

#[tauri::command]
fn install_modloader() -> Result<LauncherConfig, String> {
  let mut config = read_config()?;
  installer::install_from_latest_release(&mut config)?;
  write_config(&config)?;
  Ok(config)
}

#[tauri::command]
fn restore_modloader() -> Result<LauncherConfig, String> {
  let mut config = read_config()?;
  installer::restore(&mut config)?;
  write_config(&config)?;
  Ok(config)
}

#[tauri::command]
fn set_mod_enabled(input: ToggleModRequest) -> Result<(), String> {
  let config = read_config()?;
  let game_folder = config
    .game_folder
    .ok_or_else(|| "Set a game folder first.".to_string())?;
  let mods_dir = PathBuf::from(game_folder).join("mods");
  let mods_dir = mods_dir
    .canonicalize()
    .map_err(|_| "Mods folder does not exist.".to_string())?;
  let path = PathBuf::from(&input.path);
  if !path.exists() {
    return Err("Mod path does not exist.".to_string());
  }
  let path = path
    .canonicalize()
    .map_err(|err| format!("Invalid mod path: {err}"))?;
  if !path.starts_with(&mods_dir) || path.parent() != Some(mods_dir.as_path()) {
    return Err("Mod must be inside the configured mods folder.".to_string());
  }

  let file_name = path
    .file_name()
    .and_then(|value| value.to_str())
    .ok_or_else(|| "Invalid mod path.".to_string())?;

  if input.enabled {
    let enabled_name = file_name.trim_end_matches(".disabled");
    if enabled_name == file_name {
      return Ok(());
    }
    let target = path.with_file_name(enabled_name);
    if target.exists() {
      return Err("A mod with this enabled name already exists.".to_string());
    }
    fs::rename(&path, &target).map_err(|err| format!("Could not enable mod: {err}"))?;
  } else {
    if file_name.ends_with(".disabled") {
      return Ok(());
    }
    let target = path.with_file_name(format!("{file_name}.disabled"));
    if target.exists() {
      return Err("A disabled copy of this mod already exists.".to_string());
    }
    fs::rename(&path, &target).map_err(|err| format!("Could not disable mod: {err}"))?;
  }

  Ok(())
}

#[tauri::command]
fn api_request(input: ApiRequest) -> Result<Value, String> {
  let client = reqwest::blocking::Client::new();
  let method = input.method.parse().map_err(|err| format!("Invalid API method: {err}"))?;
  let url = if input.path.starts_with("http://") || input.path.starts_with("https://") {
    input.path
  } else {
    format!("{API_BASE}{}", input.path)
  };
  let mut request = client
    .request(method, url)
    .header("accept", "application/json")
    .header("user-agent", "oppw4-launcher");

  if let Some(token) = input.token.filter(|value| !value.trim().is_empty()) {
    request = request.bearer_auth(token);
  }
  if let Some(body) = input.body {
    request = request.header("content-type", "application/json").body(body);
  }

  let response = request.send().map_err(|err| format!("API request failed: {err}"))?;
  let status = response.status();
  let text = response.text().map_err(|err| format!("Could not read API response: {err}"))?;
  let json = serde_json::from_str::<Value>(&text).unwrap_or_else(|_| serde_json::json!({ "error": text }));
  if !status.is_success() {
    let message = json
      .get("error")
      .and_then(Value::as_str)
      .unwrap_or("API request failed");
    return Err(message.to_string());
  }
  Ok(json)
}

fn open_steam_uri() -> Result<(), String> {
  let uri = format!("steam://run/{STEAM_APP_ID}");

  #[cfg(target_os = "windows")]
  {
    Command::new("cmd")
      .args(["/C", "start", "", &uri])
      .spawn()
      .map_err(|err| format!("Could not open Steam URI: {err}"))?;
  }

  #[cfg(target_os = "linux")]
  {
    Command::new("xdg-open")
      .arg(&uri)
      .spawn()
      .map_err(|err| format!("Could not open Steam URI with xdg-open: {err}"))?;
  }

  #[cfg(not(any(target_os = "windows", target_os = "linux")))]
  {
    return Err("Steam launch is only implemented for Windows and Linux.".to_string());
  }

  Ok(())
}

fn modloader_status(config: &LauncherConfig) -> String {
  if !config.installed_files.is_empty() {
    return "Installed".to_string();
  }
  let Some(game_folder) = &config.game_folder else {
    return "Missing game folder".to_string();
  };
  if PathBuf::from(game_folder).join("dinput8.dll").exists() {
    return "Detected unmanaged dinput8.dll".to_string();
  }
  "Missing".to_string()
}

fn installed_mods(config: &LauncherConfig) -> Vec<InstalledMod> {
  let Some(game_folder) = &config.game_folder else {
    return Vec::new();
  };
  let mods_dir = PathBuf::from(game_folder).join("mods");
  let Ok(entries) = fs::read_dir(mods_dir) else {
    return Vec::new();
  };

  let mut mods = Vec::new();
  for entry in entries.flatten() {
    let path = entry.path();
    let Some(name) = path
      .file_name()
      .and_then(|value| value.to_str())
      .map(ToOwned::to_owned)
    else {
      continue;
    };
    if name == "_oppw4" || name.starts_with('.') {
      continue;
    }
    let enabled = !name.ends_with(".disabled");
    let display_name = name.trim_end_matches(".disabled").to_string();
    let kind = if path.is_dir() {
      "folder"
    } else if path
      .extension()
      .and_then(|value| value.to_str())
      .is_some_and(|ext| ext.eq_ignore_ascii_case("zip") || ext.eq_ignore_ascii_case("disabled"))
    {
      "zip"
    } else {
      continue;
    };
    mods.push(InstalledMod {
      name: display_name,
      kind: kind.to_string(),
      path: path.to_string_lossy().to_string(),
      enabled,
    });
  }
  mods.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
  mods
}

fn now_label() -> String {
  use std::time::{SystemTime, UNIX_EPOCH};
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|duration| duration.as_secs().to_string())
    .unwrap_or_else(|_| "0".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .invoke_handler(tauri::generate_handler![
      get_launcher_state,
      save_launcher_config,
      detect_game,
      launch_game,
      install_modloader,
      restore_modloader,
      set_mod_enabled,
      api_request
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
