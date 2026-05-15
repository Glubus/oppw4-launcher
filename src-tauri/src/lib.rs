mod api;
mod commands;
mod config;
mod diagnostics;
mod error;
mod installer;
mod steam;
mod updater;

use base64::{engine::general_purpose, Engine as _};
use config::LauncherConfig;
use diagnostics::{health_item, latest_loader_log, HealthCheckItem};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Cursor, Read, Seek, Write},
    path::{Path, PathBuf},
    process::Command,
};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

const API_BASE: &str = "https://oppw4.prism.am/api";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LauncherState {
    config: LauncherConfig,
    detected_game: Option<steam::DetectedGame>,
    modloader_status: String,
    latest_release: Option<installer::ReleaseInfo>,
    needs_patcher_update: bool,
    local_modloader_sha256: Option<String>,
    remote_modloader_sha256: Option<String>,
    installed_mods: Vec<InstalledMod>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InstalledMod {
    name: String,
    kind: String,
    path: String,
    mod_key: String,
    enabled: bool,
    mod_id: Option<String>,
    version: Option<String>,
    source_url: Option<String>,
    slug: Option<String>,
    character_name: Option<String>,
    character_slug: Option<String>,
    mod_type: Option<String>,
    dependencies: Vec<String>,
    changelog: Option<String>,
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
    dependencies: Vec<String>,
    changelog: Option<String>,
    cover_data_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ToggleModRequest {
    path: String,
    enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImportExternalZipRequest {
    path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApplyProfileRequest {
    profile_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApplyMetadataRequest {
    skin_id: String,
    zip_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InstallHostedModRequest {
    file_id: String,
    file_name: String,
    #[serde(default)]
    install_as_new: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InstalledModLookupRequest {
    mod_id: Option<String>,
    slug: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RevealModRequest {
    path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RevealPathRequest {
    path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RemoveModRequest {
    path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExportDiagnosticsRequest {
    path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LauncherLogRequest {
    level: String,
    message: String,
    file_stamp: String,
    #[serde(default)]
    debug: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InstallHostedModResult {
    mod_info: InstalledMod,
    already_up_to_date: bool,
}

fn modloader_status(config: &LauncherConfig, local_hash: Option<&str>) -> String {
    let Some(game_folder) = &config.game_folder else {
        return "Missing game folder".to_string();
    };
    if !PathBuf::from(game_folder).join("dinput8.dll").exists() {
        return if config.installed_files.is_empty() {
            "Missing".to_string()
        } else {
            "Missing installed dinput8.dll".to_string()
        };
    }
    if config.installed_files.is_empty() || config.modloader_sha256.is_none() {
        return "Detected unmanaged dinput8.dll".to_string();
    }
    if local_hash != config.modloader_sha256.as_deref() {
        return "Modified dinput8.dll".to_string();
    }
    if config
        .latest_modloader_sha256
        .as_ref()
        .is_some_and(|hash| config.modloader_sha256.as_ref() != Some(hash))
    {
        return "Update available".to_string();
    }
    "Installed".to_string()
}

fn build_health_check(config: &LauncherConfig) -> Vec<HealthCheckItem> {
    let mut items = Vec::new();
    let Some(game_folder) = &config.game_folder else {
        items.push(health_item(
            "error",
            "Game folder",
            "No game folder selected.",
        ));
        return items;
    };
    let game_folder = PathBuf::from(game_folder);
    if game_folder.is_dir() {
        items.push(health_item(
            "ok",
            "Game folder",
            &format!("Using {}.", game_folder.display()),
        ));
    } else {
        items.push(health_item(
            "error",
            "Game folder",
            "Selected game folder does not exist.",
        ));
        return items;
    }

    let local_hash = installer::installed_dinput8_sha256(config).ok().flatten();
    items.push(
        match modloader_status(config, local_hash.as_deref()).as_str() {
            "Installed" => health_item(
                "ok",
                "Patcher",
                "dinput8.dll is installed and matches the tracked hash.",
            ),
            "Update available" => health_item(
                "warn",
                "Patcher",
                "A newer patcher asset is available on GitHub.",
            ),
            "Modified dinput8.dll" => health_item(
                "warn",
                "Patcher",
                "The local dinput8.dll does not match the tracked install hash.",
            ),
            "Detected unmanaged dinput8.dll" => health_item(
                "warn",
                "Patcher",
                "A dinput8.dll exists, but it was not installed by this launcher.",
            ),
            "Missing installed dinput8.dll" => health_item(
                "error",
                "Patcher",
                "The launcher tracks an install, but dinput8.dll is missing.",
            ),
            status => health_item("error", "Patcher", status),
        },
    );

    let mods_dir = game_folder.join("mods");
    if mods_dir.is_dir() {
        items.push(health_item(
            "ok",
            "Mods folder",
            &format!("Found {}.", mods_dir.display()),
        ));
    } else {
        items.push(health_item(
            "warn",
            "Mods folder",
            "No mods folder found yet.",
        ));
    }

    let mods = installed_mods(config);
    if mods.is_empty() {
        items.push(health_item(
            "warn",
            "Installed mods",
            "No local mods were detected.",
        ));
    } else {
        let enabled = mods.iter().filter(|mod_info| mod_info.enabled).count();
        items.push(health_item(
            "ok",
            "Installed mods",
            &format!("{enabled}/{} mods enabled.", mods.len()),
        ));
    }

    let missing_metadata = mods
        .iter()
        .filter(|mod_info| {
            mod_info.kind == "zip" && mod_info.mod_id.is_none() && mod_info.slug.is_none()
        })
        .count();
    if missing_metadata > 0 {
        items.push(health_item(
            "warn",
            "Metadata",
            &format!("{missing_metadata} ZIP mod(s) have no usable metadata identity."),
        ));
    } else if !mods.is_empty() {
        items.push(health_item(
            "ok",
            "Metadata",
            "Installed ZIP mods have usable metadata.",
        ));
    }

    let installed_keys = installed_dependency_keys(&mods);
    let mut missing_dependencies = Vec::new();
    for mod_info in &mods {
        if !mod_info.enabled {
            continue;
        }
        for dependency in &mod_info.dependencies {
            if !installed_keys.contains(&dependency.to_lowercase()) {
                missing_dependencies.push(format!("{} needs {}", mod_info.name, dependency));
            }
        }
    }
    if missing_dependencies.is_empty() {
        items.push(health_item(
            "ok",
            "Dependencies",
            "No missing enabled mod dependencies detected.",
        ));
    } else {
        items.push(health_item(
            "error",
            "Dependencies",
            &missing_dependencies.join("; "),
        ));
    }

    if let Some(log_path) = latest_loader_log(config) {
        items.push(health_item(
            "ok",
            "Loader log",
            &format!("Latest log: {}.", log_path.display()),
        ));
    } else {
        items.push(health_item(
            "warn",
            "Loader log",
            "No loader log found in mods/_oppw4/logs.",
        ));
    }

    items
}

fn installed_dependency_keys(mods: &[InstalledMod]) -> std::collections::HashSet<String> {
    let mut keys = std::collections::HashSet::new();
    for mod_info in mods {
        keys.insert(mod_info.mod_key.to_lowercase());
        if let Some(value) = &mod_info.mod_id {
            keys.insert(value.to_lowercase());
            keys.insert(format!("id:{value}").to_lowercase());
        }
        if let Some(value) = &mod_info.slug {
            keys.insert(value.to_lowercase());
            keys.insert(format!("slug:{value}").to_lowercase());
        }
        if let Some(value) = &mod_info.source_url {
            keys.insert(value.to_lowercase());
            keys.insert(format!("source:{value}").to_lowercase());
        }
    }
    keys
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
            .is_some_and(|ext| {
                ext.eq_ignore_ascii_case("zip") || ext.eq_ignore_ascii_case("disabled")
            })
        {
            "zip"
        } else {
            continue;
        };
        if let Some(mod_info) =
            installed_mod_from_parts(path, display_name, kind.to_string(), enabled)
        {
            mods.push(mod_info);
        }
    }
    mods.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    mods
}

fn installed_mod_from_path(path: &Path) -> Option<InstalledMod> {
    let name = path.file_name()?.to_str()?.to_string();
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
        return None;
    };
    installed_mod_from_parts(path.to_path_buf(), display_name, kind.to_string(), enabled)
}

fn installed_mod_from_parts(
    path: PathBuf,
    display_name: String,
    kind: String,
    enabled: bool,
) -> Option<InstalledMod> {
    let metadata = if kind == "zip" {
        read_local_mod_metadata(&path).unwrap_or_default()
    } else {
        LocalModMetadata::default()
    };
    let mod_key = mod_key_for(&display_name, &metadata);
    Some(InstalledMod {
        name: metadata.title.unwrap_or(display_name),
        kind,
        path: path.to_string_lossy().to_string(),
        mod_key,
        enabled,
        mod_id: metadata.mod_id,
        version: metadata.version,
        source_url: metadata.source_url,
        slug: metadata.slug,
        character_name: metadata.character_name,
        character_slug: metadata.character_slug,
        mod_type: metadata.mod_type,
        dependencies: metadata.dependencies,
        changelog: metadata.changelog,
        cover_data_url: metadata.cover_data_url,
    })
}

fn mod_key_for(display_name: &str, metadata: &LocalModMetadata) -> String {
    metadata
        .mod_id
        .as_ref()
        .map(|value| format!("id:{value}"))
        .or_else(|| metadata.slug.as_ref().map(|value| format!("slug:{value}")))
        .or_else(|| {
            metadata
                .source_url
                .as_ref()
                .map(|value| format!("source:{value}"))
        })
        .unwrap_or_else(|| {
            format!(
                "local:{}",
                display_name.trim_end_matches(".zip").to_lowercase()
            )
        })
}

fn read_local_mod_metadata(path: &Path) -> Result<LocalModMetadata, String> {
    let file = fs::File::open(path).map_err(|err| format!("Could not open mod ZIP: {err}"))?;
    let mut archive =
        ZipArchive::new(file).map_err(|err| format!("Could not read mod ZIP: {err}"))?;
    read_mod_metadata_from_archive(&mut archive)
}

fn read_mod_metadata_from_bytes(bytes: &[u8]) -> Result<LocalModMetadata, String> {
    let reader = Cursor::new(bytes);
    let mut archive =
        ZipArchive::new(reader).map_err(|err| format!("Could not read mod ZIP: {err}"))?;
    read_mod_metadata_from_archive(&mut archive)
}

fn read_mod_metadata_from_archive<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
) -> Result<LocalModMetadata, String> {
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
        metadata.dependencies = toml_array(&content, "dependencies");
        metadata.changelog = toml_value(&content, "changelog");
        if let Some(cover_path) =
            toml_value(&content, "cover").filter(|value| value.starts_with(".metadata/"))
        {
            metadata.cover_data_url = zip_image_data_url(archive, &cover_path).ok();
        }
    }

    Ok(metadata)
}

fn same_mod_version(mod_info: &InstalledMod, metadata: &LocalModMetadata) -> bool {
    if metadata.version.is_none() {
        return false;
    }
    same_mod_identity(mod_info, metadata) && mod_info.version.as_ref() == metadata.version.as_ref()
}

fn same_mod_identity(mod_info: &InstalledMod, metadata: &LocalModMetadata) -> bool {
    metadata
        .mod_id
        .as_ref()
        .is_some_and(|id| mod_info.mod_id.as_ref() == Some(id))
        || metadata
            .slug
            .as_ref()
            .is_some_and(|slug| mod_info.slug.as_ref() == Some(slug))
        || metadata
            .source_url
            .as_ref()
            .is_some_and(|url| mod_info.source_url.as_ref() == Some(url))
}

fn toml_value(content: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    content.lines().find_map(|line| {
        let value = line.trim().strip_prefix(&prefix)?.trim();
        if value == "\"\"" {
            return None;
        }
        if value.starts_with('"') && value.ends_with('"') {
            serde_json::from_str::<String>(value)
                .ok()
                .filter(|value| !value.trim().is_empty())
        } else {
            Some(value.to_string()).filter(|value| !value.trim().is_empty())
        }
    })
}

fn toml_array(content: &str, key: &str) -> Vec<String> {
    let prefix = format!("{key} = ");
    content
        .lines()
        .find_map(|line| line.trim().strip_prefix(&prefix))
        .and_then(|value| serde_json::from_str::<Vec<String>>(value.trim()).ok())
        .unwrap_or_default()
        .into_iter()
        .filter(|value| !value.trim().is_empty())
        .collect()
}

fn zip_image_data_url<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
) -> Result<String, String> {
    let mut entry = archive
        .by_name(path)
        .map_err(|err| format!("Could not read cover image: {err}"))?;
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
    Ok(format!(
        "data:{mime};base64,{}",
        general_purpose::STANDARD.encode(bytes)
    ))
}

fn read_metadata_entries(bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>, String> {
    let reader = Cursor::new(bytes);
    let mut archive =
        ZipArchive::new(reader).map_err(|err| format!("Could not open metadata ZIP: {err}"))?;
    let mut entries = Vec::new();

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|err| format!("Could not read metadata ZIP: {err}"))?;
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

fn inject_metadata_entries(
    target_path: &Path,
    metadata_entries: Vec<(String, Vec<u8>)>,
) -> Result<(), String> {
    let source_file =
        fs::File::open(target_path).map_err(|err| format!("Could not open selected ZIP: {err}"))?;
    let mut source = ZipArchive::new(source_file)
        .map_err(|err| format!("Could not read selected ZIP: {err}"))?;
    let temp_path = target_path.with_extension("zip.metadata-tmp");
    let backup_path = target_path.with_extension("zip.metadata-backup");
    let temp_file = fs::File::create(&temp_path)
        .map_err(|err| format!("Could not create temporary ZIP: {err}"))?;
    let mut writer = ZipWriter::new(temp_file);

    for index in 0..source.len() {
        let entry = source
            .by_index(index)
            .map_err(|err| format!("Could not read selected ZIP entry: {err}"))?;
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
    writer
        .finish()
        .map_err(|err| format!("Could not finish ZIP: {err}"))?;

    fs::copy(target_path, &backup_path)
        .map_err(|err| format!("Could not create ZIP backup: {err}"))?;
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
    let stem = if stem.is_empty() {
        "mod".to_string()
    } else {
        stem
    };

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
        let folder = path
            .parent()
            .ok_or_else(|| "Could not resolve mod folder.".to_string())?;
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
            commands::launcher::state::get_launcher_state,
            commands::launcher::state::save_launcher_config,
            commands::launcher::game::detect_game,
            commands::launcher::game::launch_game,
            commands::launcher::patcher::install_modloader,
            commands::launcher::patcher::restore_modloader,
            commands::launcher::patcher::check_modloader_integrity,
            commands::launcher::diagnostics::run_health_check,
            commands::launcher::diagnostics::export_diagnostics,
            commands::mods::folder::set_mod_enabled,
            commands::mods::install::import_external_zip,
            commands::mods::profiles::apply_mod_profile,
            commands::mods::metadata::apply_metadata_to_zip,
            commands::mods::install::install_hosted_mod,
            commands::mods::install::installed_mod_for_skin,
            commands::mods::folder::reveal_mod_in_folder,
            commands::mods::folder::remove_installed_mod,
            commands::system::paths::reveal_path_in_folder,
            commands::system::logs::write_launcher_log,
            commands::system::updates::check_launcher_update,
            commands::system::updates::install_launcher_update,
            commands::system::links::open_external_url,
            api::api_request
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
            let options =
                SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
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
        archive
            .by_name("mod/file.txt")
            .unwrap()
            .read_to_string(&mut kept)
            .unwrap();
        let mut metadata = String::new();
        archive
            .by_name("metadata.toml")
            .unwrap()
            .read_to_string(&mut metadata)
            .unwrap();

        assert_eq!(kept, "keep");
        assert!(metadata.contains("Installed Mod"));
        assert!(archive.by_name(".metadata/cover.png").is_ok());

        drop(archive);
        let local_metadata = read_local_mod_metadata(&zip_path).unwrap();
        assert_eq!(local_metadata.title.as_deref(), Some("Installed Mod"));
        assert_eq!(local_metadata.version.as_deref(), Some("1.2.3"));
        assert_eq!(local_metadata.slug.as_deref(), Some("installed-mod"));
        assert!(local_metadata
            .cover_data_url
            .as_deref()
            .unwrap_or("")
            .starts_with("data:image/png;base64,"));
    }

    #[test]
    fn modloader_status_detects_unmanaged_dll() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("dinput8.dll"), b"foreign").unwrap();
        let config = LauncherConfig {
            game_folder: Some(temp.path().to_string_lossy().to_string()),
            ..LauncherConfig::default()
        };

        assert_eq!(
            modloader_status(&config, None),
            "Detected unmanaged dinput8.dll"
        );
    }

    #[test]
    fn modloader_status_detects_modified_dll() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("dinput8.dll"), b"modified").unwrap();
        let config = LauncherConfig {
            game_folder: Some(temp.path().to_string_lossy().to_string()),
            modloader_sha256: Some("expected".to_string()),
            installed_files: vec![crate::config::InstalledFile {
                relative_path: "dinput8.dll".to_string(),
                backup_path: None,
            }],
            ..LauncherConfig::default()
        };

        assert_eq!(
            modloader_status(&config, Some("actual")),
            "Modified dinput8.dll"
        );
    }

    #[test]
    fn modloader_status_detects_available_update() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("dinput8.dll"), b"current").unwrap();
        let config = LauncherConfig {
            game_folder: Some(temp.path().to_string_lossy().to_string()),
            modloader_sha256: Some("current-hash".to_string()),
            latest_modloader_sha256: Some("new-hash".to_string()),
            installed_files: vec![crate::config::InstalledFile {
                relative_path: "dinput8.dll".to_string(),
                backup_path: None,
            }],
            ..LauncherConfig::default()
        };

        assert_eq!(
            modloader_status(&config, Some("current-hash")),
            "Update available"
        );
    }

    #[test]
    fn installed_mods_reads_zip_metadata_and_cover() {
        let temp = tempfile::tempdir().unwrap();
        let mods_dir = temp.path().join("mods");
        fs::create_dir_all(&mods_dir).unwrap();
        let zip_path = mods_dir.join("law.zip");
        {
            let file = fs::File::create(&zip_path).unwrap();
            let mut writer = ZipWriter::new(file);
            let options =
                SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
            writer.start_file("metadata.toml", options).unwrap();
            writer.write_all(b"title = \"Casual Law\"\nversion = \"1.0.0\"\nmod_id = \"casual-law\"\ncharacter_name = \"Trafalgar Law\"\ndependencies = [\"base-law\"]\nchangelog = \"Initial release\"\ncover = \".metadata/cover.png\"\n").unwrap();
            writer.start_file(".metadata/cover.png", options).unwrap();
            writer.write_all(b"png").unwrap();
            writer.finish().unwrap();
        }
        let config = LauncherConfig {
            game_folder: Some(temp.path().to_string_lossy().to_string()),
            ..LauncherConfig::default()
        };

        let mods = installed_mods(&config);

        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].name, "Casual Law");
        assert_eq!(mods[0].version.as_deref(), Some("1.0.0"));
        assert_eq!(mods[0].mod_key, "id:casual-law");
        assert_eq!(mods[0].character_name.as_deref(), Some("Trafalgar Law"));
        assert_eq!(mods[0].dependencies, vec!["base-law"]);
        assert_eq!(mods[0].changelog.as_deref(), Some("Initial release"));
        assert!(mods[0]
            .cover_data_url
            .as_deref()
            .unwrap_or("")
            .starts_with("data:image/png;base64,"));
    }
}
