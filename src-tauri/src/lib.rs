mod config;
mod installer;
mod steam;

use base64::{engine::general_purpose, Engine as _};
use config::{load_config as read_config, save_config as write_config, LaunchMode, LauncherConfig, STEAM_APP_ID};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, io::{Cursor, Read, Seek, Write}, path::{Path, PathBuf}, process::Command};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

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
  mod_id: Option<String>,
  version: Option<String>,
  source_url: Option<String>,
  slug: Option<String>,
  character_name: Option<String>,
  character_slug: Option<String>,
  mod_type: Option<String>,
  cover_data_url: Option<String>,
}

#[derive(Debug, Default)]
struct LocalModMetadata {
  mod_id: Option<String>,
  title: Option<String>,
  version: Option<String>,
  source_url: Option<String>,
  slug: Option<String>,
  character_name: Option<String>,
  character_slug: Option<String>,
  mod_type: Option<String>,
  cover_data_url: Option<String>,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApplyMetadataRequest {
  skin_id: String,
  zip_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallHostedModRequest {
  file_id: String,
  file_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstalledModLookupRequest {
  mod_id: Option<String>,
  slug: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RevealModRequest {
  path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InstallHostedModResult {
  mod_info: InstalledMod,
  already_up_to_date: bool,
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
    .clone()
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

#[tauri::command]
fn apply_metadata_to_zip(input: ApplyMetadataRequest) -> Result<(), String> {
  let target_path = PathBuf::from(input.zip_path);
  if !target_path.exists() {
    return Err("Selected ZIP does not exist.".to_string());
  }
  if !target_path
    .extension()
    .and_then(|value| value.to_str())
    .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
  {
    return Err("Select a ZIP archive.".to_string());
  }

  let url = format!("{API_BASE}/skins/{}/metadata.zip", input.skin_id);
  let bytes = reqwest::blocking::Client::new()
    .get(url)
    .header("accept", "application/zip")
    .header("user-agent", "oppw4-launcher")
    .send()
    .map_err(|err| format!("Could not download metadata: {err}"))?
    .error_for_status()
    .map_err(|err| format!("Metadata download failed: {err}"))?
    .bytes()
    .map_err(|err| format!("Could not read metadata download: {err}"))?;

  let metadata_entries = read_metadata_entries(bytes.as_ref())?;
  if !metadata_entries.iter().any(|entry| entry.0 == "metadata.toml") {
    return Err("Downloaded metadata ZIP does not contain metadata.toml.".to_string());
  }

  inject_metadata_entries(&target_path, metadata_entries)
}

#[tauri::command]
fn install_hosted_mod(input: InstallHostedModRequest) -> Result<InstallHostedModResult, String> {
  let config = read_config()?;
  let game_folder = config
    .game_folder
    .clone()
    .ok_or_else(|| "Set a game folder first.".to_string())?;
  let mods_dir = PathBuf::from(game_folder).join("mods");
  fs::create_dir_all(&mods_dir).map_err(|err| format!("Could not create mods folder: {err}"))?;

  let url = format!("{API_BASE}/files/{}/download", input.file_id);
  let bytes = reqwest::blocking::Client::new()
    .get(url)
    .header("accept", "application/zip")
    .header("user-agent", "oppw4-launcher")
    .send()
    .map_err(|err| format!("Could not download mod: {err}"))?
    .error_for_status()
    .map_err(|err| format!("Mod download failed: {err}"))?
    .bytes()
    .map_err(|err| format!("Could not read mod download: {err}"))?;

  if !bytes.starts_with(b"PK") {
    return Err("Downloaded file is not a ZIP archive.".to_string());
  }

  let downloaded_metadata = read_mod_metadata_from_bytes(bytes.as_ref()).unwrap_or_default();
  if let Some(existing) = installed_mods(&config).into_iter().find(|mod_info| same_mod_version(mod_info, &downloaded_metadata)) {
    return Ok(InstallHostedModResult {
      mod_info: existing,
      already_up_to_date: true,
    });
  }

  let target = available_mod_path(&mods_dir, &input.file_name);
  fs::write(&target, bytes).map_err(|err| format!("Could not write mod ZIP: {err}"))?;
  let name = target
    .file_name()
    .and_then(|value| value.to_str())
    .unwrap_or("installed.zip")
    .to_string();

  Ok(InstallHostedModResult {
    mod_info: InstalledMod {
      name: downloaded_metadata.title.unwrap_or_else(|| name.trim_end_matches(".disabled").to_string()),
      kind: "zip".to_string(),
      path: target.to_string_lossy().to_string(),
      enabled: true,
      mod_id: downloaded_metadata.mod_id,
      version: downloaded_metadata.version,
      source_url: downloaded_metadata.source_url,
      slug: downloaded_metadata.slug,
      character_name: downloaded_metadata.character_name,
      character_slug: downloaded_metadata.character_slug,
      mod_type: downloaded_metadata.mod_type,
      cover_data_url: downloaded_metadata.cover_data_url,
    },
    already_up_to_date: false,
  })
}

#[tauri::command]
fn installed_mod_for_skin(input: InstalledModLookupRequest) -> Result<Option<InstalledMod>, String> {
  let config = read_config()?;
  Ok(installed_mods(&config).into_iter().find(|mod_info| {
    input.mod_id.as_ref().is_some_and(|id| mod_info.mod_id.as_ref() == Some(id))
      || input.slug.as_ref().is_some_and(|slug| mod_info.slug.as_ref() == Some(slug))
  }))
}

#[tauri::command]
fn reveal_mod_in_folder(input: RevealModRequest) -> Result<(), String> {
  let config = read_config()?;
  let game_folder = config
    .game_folder
    .ok_or_else(|| "Set a game folder first.".to_string())?;
  let mods_dir = PathBuf::from(game_folder).join("mods");
  let mods_dir = mods_dir
    .canonicalize()
    .map_err(|_| "Mods folder does not exist.".to_string())?;
  let path = PathBuf::from(input.path);
  if !path.exists() {
    return Err("Mod path does not exist.".to_string());
  }
  let path = path
    .canonicalize()
    .map_err(|err| format!("Invalid mod path: {err}"))?;
  if !path.starts_with(&mods_dir) || path.parent() != Some(mods_dir.as_path()) {
    return Err("Mod must be inside the configured mods folder.".to_string());
  }

  reveal_path(&path)
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
    let metadata = if kind == "zip" {
      read_local_mod_metadata(&path).unwrap_or_default()
    } else {
      LocalModMetadata::default()
    };
    mods.push(InstalledMod {
      name: metadata.title.unwrap_or(display_name),
      kind: kind.to_string(),
      path: path.to_string_lossy().to_string(),
      enabled,
      mod_id: metadata.mod_id,
      version: metadata.version,
      source_url: metadata.source_url,
      slug: metadata.slug,
      character_name: metadata.character_name,
      character_slug: metadata.character_slug,
      mod_type: metadata.mod_type,
      cover_data_url: metadata.cover_data_url,
    });
  }
  mods.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
  mods
}

fn read_local_mod_metadata(path: &Path) -> Result<LocalModMetadata, String> {
  let file = fs::File::open(path).map_err(|err| format!("Could not open mod ZIP: {err}"))?;
  let mut archive = ZipArchive::new(file).map_err(|err| format!("Could not read mod ZIP: {err}"))?;
  read_mod_metadata_from_archive(&mut archive)
}

fn read_mod_metadata_from_bytes(bytes: &[u8]) -> Result<LocalModMetadata, String> {
  let reader = Cursor::new(bytes);
  let mut archive = ZipArchive::new(reader).map_err(|err| format!("Could not read mod ZIP: {err}"))?;
  read_mod_metadata_from_archive(&mut archive)
}

fn read_mod_metadata_from_archive<R: Read + Seek>(archive: &mut ZipArchive<R>) -> Result<LocalModMetadata, String> {
  let mut metadata = LocalModMetadata::default();
  let content = match archive.by_name("metadata.toml") {
    Ok(mut entry) => {
      let mut content = String::new();
      entry
        .read_to_string(&mut content)
        .map_err(|err| format!("Could not read metadata.toml: {err}"))?;
      content
    }
    Err(_) => return Ok(metadata),
  };

  {
    metadata.mod_id = toml_value(&content, "mod_id");
    metadata.title = toml_value(&content, "title");
    metadata.version = toml_value(&content, "version");
    metadata.source_url = toml_value(&content, "source_url");
    metadata.slug = toml_value(&content, "slug");
    metadata.character_name = toml_value(&content, "character_name");
    metadata.character_slug = toml_value(&content, "character_slug");
    metadata.mod_type = toml_value(&content, "mod_type");
    if let Some(cover_path) = toml_value(&content, "cover").filter(|value| value.starts_with(".metadata/")) {
      metadata.cover_data_url = zip_image_data_url(archive, &cover_path).ok();
    }
  }

  Ok(metadata)
}

fn same_mod_version(mod_info: &InstalledMod, metadata: &LocalModMetadata) -> bool {
  if metadata.version.is_none() {
    return false;
  }
  let same_identity = metadata.mod_id.as_ref().is_some_and(|id| mod_info.mod_id.as_ref() == Some(id))
    || metadata.slug.as_ref().is_some_and(|slug| mod_info.slug.as_ref() == Some(slug))
    || metadata.source_url.as_ref().is_some_and(|url| mod_info.source_url.as_ref() == Some(url));
  same_identity && mod_info.version.as_ref() == metadata.version.as_ref()
}

fn toml_value(content: &str, key: &str) -> Option<String> {
  let prefix = format!("{key} = ");
  content.lines().find_map(|line| {
    let value = line.trim().strip_prefix(&prefix)?.trim();
    if value == "\"\"" {
      return None;
    }
    if value.starts_with('"') && value.ends_with('"') {
      serde_json::from_str::<String>(value).ok().filter(|value| !value.trim().is_empty())
    } else {
      Some(value.to_string()).filter(|value| !value.trim().is_empty())
    }
  })
}

fn zip_image_data_url<R: Read + Seek>(archive: &mut ZipArchive<R>, path: &str) -> Result<String, String> {
  let mut entry = archive.by_name(path).map_err(|err| format!("Could not read cover image: {err}"))?;
  let mut bytes = Vec::new();
  entry
    .read_to_end(&mut bytes)
    .map_err(|err| format!("Could not read cover image: {err}"))?;
  let mime = if path.ends_with(".png") {
    "image/png"
  } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
    "image/jpeg"
  } else if path.ends_with(".webp") {
    "image/webp"
  } else {
    return Err("Unsupported cover image type.".to_string());
  };
  Ok(format!("data:{mime};base64,{}", general_purpose::STANDARD.encode(bytes)))
}

fn read_metadata_entries(bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>, String> {
  let reader = Cursor::new(bytes);
  let mut archive = ZipArchive::new(reader).map_err(|err| format!("Could not open metadata ZIP: {err}"))?;
  let mut entries = Vec::new();

  for index in 0..archive.len() {
    let mut entry = archive.by_index(index).map_err(|err| format!("Could not read metadata ZIP: {err}"))?;
    if entry.is_dir() {
      continue;
    }
    let name = entry.name().replace('\\', "/");
    if name != "metadata.toml" && !name.starts_with(".metadata/") {
      continue;
    }
    if name.contains("..") || name.starts_with('/') {
      return Err("Metadata ZIP contains an unsafe path.".to_string());
    }
    let mut content = Vec::new();
    entry
      .read_to_end(&mut content)
      .map_err(|err| format!("Could not read metadata entry: {err}"))?;
    entries.push((name, content));
  }

  Ok(entries)
}

fn inject_metadata_entries(target_path: &Path, metadata_entries: Vec<(String, Vec<u8>)>) -> Result<(), String> {
  let source_file = fs::File::open(target_path).map_err(|err| format!("Could not open selected ZIP: {err}"))?;
  let mut source = ZipArchive::new(source_file).map_err(|err| format!("Could not read selected ZIP: {err}"))?;
  let temp_path = target_path.with_extension("zip.metadata-tmp");
  let backup_path = target_path.with_extension("zip.metadata-backup");
  let temp_file = fs::File::create(&temp_path).map_err(|err| format!("Could not create temporary ZIP: {err}"))?;
  let mut writer = ZipWriter::new(temp_file);

  for index in 0..source.len() {
    let entry = source.by_index(index).map_err(|err| format!("Could not read selected ZIP entry: {err}"))?;
    let name = entry.name().replace('\\', "/");
    if name == "metadata.toml" || name.starts_with(".metadata/") {
      continue;
    }
    writer
      .raw_copy_file(entry)
      .map_err(|err| format!("Could not copy selected ZIP entry: {err}"))?;
  }

  let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
  for (name, content) in metadata_entries {
    writer
      .start_file(name, options)
      .map_err(|err| format!("Could not write metadata entry: {err}"))?;
    writer
      .write_all(&content)
      .map_err(|err| format!("Could not write metadata entry: {err}"))?;
  }
  writer.finish().map_err(|err| format!("Could not finish ZIP: {err}"))?;

  fs::copy(target_path, &backup_path).map_err(|err| format!("Could not create ZIP backup: {err}"))?;
  if let Err(err) = replace_file(&temp_path, target_path) {
    let _ = fs::copy(&backup_path, target_path);
    let _ = fs::remove_file(&temp_path);
    return Err(err);
  }
  let _ = fs::remove_file(&backup_path);
  Ok(())
}

fn available_mod_path(mods_dir: &Path, file_name: &str) -> PathBuf {
  let safe_name = Path::new(file_name)
    .file_name()
    .and_then(|value| value.to_str())
    .filter(|value| !value.trim().is_empty())
    .unwrap_or("mod.zip");
  let stem = safe_name
    .trim_end_matches(".zip")
    .trim_end_matches(".ZIP")
    .trim()
    .replace(['/', '\\'], "-");
  let stem = if stem.is_empty() { "mod".to_string() } else { stem };

  for index in 0..1000 {
    let name = if index == 0 {
      format!("{stem}.zip")
    } else {
      format!("{stem}-{index}.zip")
    };
    let candidate = mods_dir.join(name);
    if !candidate.exists() {
      return candidate;
    }
  }

  mods_dir.join(format!("{stem}-{}.zip", now_label()))
}

fn replace_file(source: &Path, target: &Path) -> Result<(), String> {
  if target.exists() {
    fs::remove_file(target).map_err(|err| format!("Could not replace selected ZIP: {err}"))?;
  }
  fs::rename(source, target).map_err(|err| format!("Could not replace selected ZIP: {err}"))
}

fn reveal_path(path: &Path) -> Result<(), String> {
  #[cfg(target_os = "windows")]
  {
    Command::new("explorer")
      .arg(format!("/select,{}", path.display()))
      .spawn()
      .map_err(|err| format!("Could not open folder: {err}"))?;
  }

  #[cfg(target_os = "linux")]
  {
    let folder = path.parent().ok_or_else(|| "Could not resolve mod folder.".to_string())?;
    Command::new("xdg-open")
      .arg(folder)
      .spawn()
      .map_err(|err| format!("Could not open folder: {err}"))?;
  }

  #[cfg(not(any(target_os = "windows", target_os = "linux")))]
  {
    return Err("Show in folder is only implemented for Windows and Linux.".to_string());
  }

  Ok(())
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
      apply_metadata_to_zip,
      install_hosted_mod,
      installed_mod_for_skin,
      reveal_mod_in_folder,
      api_request
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn inject_metadata_replaces_metadata_entries_only() {
    let temp = tempfile::tempdir().unwrap();
    let zip_path = temp.path().join("mod.zip");
    {
      let file = fs::File::create(&zip_path).unwrap();
      let mut writer = ZipWriter::new(file);
      let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
      writer.start_file("mod/file.txt", options).unwrap();
      writer.write_all(b"keep").unwrap();
      writer.start_file("metadata.toml", options).unwrap();
      writer.write_all(b"old").unwrap();
      writer.start_file(".metadata/cover.png", options).unwrap();
      writer.write_all(b"old-cover").unwrap();
      writer.finish().unwrap();
    }

    inject_metadata_entries(&zip_path, vec![
      ("metadata.toml".to_string(), b"title = \"Installed Mod\"\nversion = \"1.2.3\"\nslug = \"installed-mod\"\ncover = \".metadata/cover.png\"\n".to_vec()),
      (".metadata/cover.png".to_string(), b"new-cover".to_vec()),
    ]).unwrap();

    let file = fs::File::open(&zip_path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    let mut kept = String::new();
    archive.by_name("mod/file.txt").unwrap().read_to_string(&mut kept).unwrap();
    let mut metadata = String::new();
    archive.by_name("metadata.toml").unwrap().read_to_string(&mut metadata).unwrap();

    assert_eq!(kept, "keep");
    assert!(metadata.contains("Installed Mod"));
    assert!(archive.by_name(".metadata/cover.png").is_ok());

    drop(archive);
    let local_metadata = read_local_mod_metadata(&zip_path).unwrap();
    assert_eq!(local_metadata.title.as_deref(), Some("Installed Mod"));
    assert_eq!(local_metadata.version.as_deref(), Some("1.2.3"));
    assert_eq!(local_metadata.slug.as_deref(), Some("installed-mod"));
    assert!(local_metadata.cover_data_url.as_deref().unwrap_or("").starts_with("data:image/png;base64,"));
  }
}
