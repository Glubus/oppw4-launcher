use super::types::{RemoveModRequest, RevealModRequest, ToggleModRequest};
use crate::{
    config::{load_config as read_config, LauncherConfig},
    system_utils,
};
use std::{fs, path::PathBuf};

#[tauri::command]
pub(crate) fn set_mod_enabled(input: ToggleModRequest) -> Result<(), String> {
    let config = read_config()?;
    set_mod_path_enabled(&config, &input.path, input.enabled)
}

#[tauri::command]
pub(crate) fn reveal_mod_in_folder(input: RevealModRequest) -> Result<(), String> {
    let path = checked_mod_path(input.path)?;
    system_utils::reveal_path(&path)
}

#[tauri::command]
pub(crate) fn remove_installed_mod(input: RemoveModRequest) -> Result<(), String> {
    let path = checked_mod_path(input.path)?;
    if path.is_dir() {
        fs::remove_dir_all(&path).map_err(|err| format!("Could not remove mod folder: {err}"))?;
    } else {
        fs::remove_file(&path).map_err(|err| format!("Could not remove mod file: {err}"))?;
    }
    Ok(())
}

pub(super) fn set_mod_path_enabled(
    config: &LauncherConfig,
    mod_path: &str,
    enabled: bool,
) -> Result<(), String> {
    let game_folder = config
        .game_folder
        .clone()
        .ok_or_else(|| "Set a game folder first.".to_string())?;
    let mods_dir = PathBuf::from(game_folder).join("mods");
    let mods_dir = mods_dir
        .canonicalize()
        .map_err(|_| "Mods folder does not exist.".to_string())?;
    let path = PathBuf::from(mod_path);
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

    if enabled {
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

fn checked_mod_path(path: String) -> Result<PathBuf, String> {
    let config = read_config()?;
    let game_folder = config
        .game_folder
        .ok_or_else(|| "Set a game folder first.".to_string())?;
    let mods_dir = PathBuf::from(game_folder).join("mods");
    let mods_dir = mods_dir
        .canonicalize()
        .map_err(|_| "Mods folder does not exist.".to_string())?;
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err("Mod path does not exist.".to_string());
    }
    let path = path
        .canonicalize()
        .map_err(|err| format!("Invalid mod path: {err}"))?;
    if !path.starts_with(&mods_dir) || path.parent() != Some(mods_dir.as_path()) {
        return Err("Mod must be inside the configured mods folder.".to_string());
    }
    Ok(path)
}
