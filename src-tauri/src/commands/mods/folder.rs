use super::types::{RemoveModRequest, RevealModRequest, ToggleModRequest};
use crate::{
    config::{load_config as read_config, LauncherConfig},
    system_utils,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn set_mod_enabled(input: ToggleModRequest) -> Result<(), String> {
    let config = read_config()?;
    set_mod_path_enabled(&config, &input.path, input.enabled)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn reveal_mod_in_folder(input: RevealModRequest) -> Result<(), String> {
    let path = checked_mod_path(&input.path)?;
    system_utils::reveal_path(&path)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn remove_installed_mod(input: RemoveModRequest) -> Result<(), String> {
    let path = checked_mod_path(&input.path)?;
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
    let path = checked_mod_path_for_config(config, mod_path)?;
    set_checked_mod_path_enabled(&path, enabled)
}

fn checked_mod_path(path: &str) -> Result<PathBuf, String> {
    let config = read_config()?;
    checked_mod_path_for_config(&config, path)
}

fn checked_mod_path_for_config(config: &LauncherConfig, path: &str) -> Result<PathBuf, String> {
    let content_dirs = canonical_content_dirs(config)?;
    let path = canonical_existing_mod_path(path)?;
    require_top_level_mod_path(&path, &content_dirs)?;
    Ok(path)
}

fn canonical_content_dirs(config: &LauncherConfig) -> Result<Vec<PathBuf>, String> {
    let game_folder = config
        .game_folder
        .clone()
        .ok_or_else(|| "Set a game folder first.".to_string())?;
    let game_folder = PathBuf::from(game_folder);
    let dirs = ["mods", "plugins"]
        .into_iter()
        .filter_map(|name| game_folder.join(name).canonicalize().ok())
        .collect::<Vec<_>>();
    if dirs.is_empty() {
        return Err("Mods or plugins folder does not exist.".to_string());
    }
    Ok(dirs)
}

fn canonical_existing_mod_path(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err("Mod path does not exist.".to_string());
    }
    path.canonicalize()
        .map_err(|err| format!("Invalid mod path: {err}"))
}

fn require_top_level_mod_path(path: &Path, content_dirs: &[PathBuf]) -> Result<(), String> {
    if content_dirs
        .iter()
        .any(|dir| path.starts_with(dir) && path.parent() == Some(dir.as_path()))
    {
        Ok(())
    } else {
        Err("Mod must be inside the configured mods or plugins folder.".to_string())
    }
}

fn set_checked_mod_path_enabled(path: &Path, enabled: bool) -> Result<(), String> {
    let file_name = mod_file_name(path)?;
    if enabled {
        enable_mod_path(path, file_name)
    } else {
        disable_mod_path(path, file_name)
    }
}

fn mod_file_name(path: &Path) -> Result<&str, String> {
    path.file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Invalid mod path.".to_string())
}

fn enable_mod_path(path: &Path, file_name: &str) -> Result<(), String> {
    let enabled_name = file_name.trim_end_matches(".disabled");
    if enabled_name == file_name {
        return Ok(());
    }
    let target = path.with_file_name(enabled_name);
    if target.exists() {
        return Err("A mod with this enabled name already exists.".to_string());
    }
    fs::rename(path, &target).map_err(|err| format!("Could not enable mod: {err}"))
}

fn disable_mod_path(path: &Path, file_name: &str) -> Result<(), String> {
    if file_name.ends_with(".disabled") {
        return Ok(());
    }
    let target = path.with_file_name(format!("{file_name}.disabled"));
    if target.exists() {
        return Err("A disabled copy of this mod already exists.".to_string());
    }
    fs::rename(path, &target).map_err(|err| format!("Could not disable mod: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LauncherConfig;

    #[test]
    fn set_mod_path_enabled_disables_and_enables_top_level_mod() {
        let temp = tempfile::tempdir().unwrap();
        let mods_dir = temp.path().join("mods");
        fs::create_dir_all(&mods_dir).unwrap();
        let mod_path = mods_dir.join("law.zip");
        fs::write(&mod_path, b"zip").unwrap();
        let config = LauncherConfig {
            game_folder: Some(temp.path().to_string_lossy().to_string()),
            ..LauncherConfig::default()
        };

        set_mod_path_enabled(&config, &mod_path.to_string_lossy(), false).unwrap();

        let disabled_path = mods_dir.join("law.zip.disabled");
        assert!(!mod_path.exists());
        assert!(disabled_path.exists());

        set_mod_path_enabled(&config, &disabled_path.to_string_lossy(), true).unwrap();

        assert!(mod_path.exists());
        assert!(!disabled_path.exists());
    }

    #[test]
    fn set_mod_path_enabled_rejects_nested_mod_paths() {
        let temp = tempfile::tempdir().unwrap();
        let mods_dir = temp.path().join("mods");
        let nested_dir = mods_dir.join("pack");
        fs::create_dir_all(&nested_dir).unwrap();
        let nested_mod = nested_dir.join("nested.zip");
        fs::write(&nested_mod, b"zip").unwrap();
        let config = LauncherConfig {
            game_folder: Some(temp.path().to_string_lossy().to_string()),
            ..LauncherConfig::default()
        };

        let err = set_mod_path_enabled(&config, &nested_mod.to_string_lossy(), false).unwrap_err();

        assert_eq!(err, "Mod must be inside the configured mods or plugins folder.");
        assert!(nested_mod.exists());
    }

    #[test]
    fn set_mod_path_enabled_is_idempotent_when_state_already_matches() {
        let temp = tempfile::tempdir().unwrap();
        let mods_dir = temp.path().join("mods");
        fs::create_dir_all(&mods_dir).unwrap();
        let enabled_mod = mods_dir.join("enabled.zip");
        let disabled_mod = mods_dir.join("disabled.zip.disabled");
        fs::write(&enabled_mod, b"enabled").unwrap();
        fs::write(&disabled_mod, b"disabled").unwrap();
        let config = LauncherConfig {
            game_folder: Some(temp.path().to_string_lossy().to_string()),
            ..LauncherConfig::default()
        };

        set_mod_path_enabled(&config, &enabled_mod.to_string_lossy(), true).unwrap();
        set_mod_path_enabled(&config, &disabled_mod.to_string_lossy(), false).unwrap();

        assert!(enabled_mod.exists());
        assert!(disabled_mod.exists());
    }

    #[test]
    fn set_mod_path_enabled_rejects_name_conflicts() {
        let temp = tempfile::tempdir().unwrap();
        let mods_dir = temp.path().join("mods");
        fs::create_dir_all(&mods_dir).unwrap();
        let enabled_mod = mods_dir.join("law.zip");
        let disabled_mod = mods_dir.join("law.zip.disabled");
        fs::write(&enabled_mod, b"enabled").unwrap();
        fs::write(&disabled_mod, b"disabled").unwrap();
        let config = LauncherConfig {
            game_folder: Some(temp.path().to_string_lossy().to_string()),
            ..LauncherConfig::default()
        };

        let enable_err =
            set_mod_path_enabled(&config, &disabled_mod.to_string_lossy(), true).unwrap_err();
        let disable_err =
            set_mod_path_enabled(&config, &enabled_mod.to_string_lossy(), false).unwrap_err();

        assert_eq!(enable_err, "A mod with this enabled name already exists.");
        assert_eq!(disable_err, "A disabled copy of this mod already exists.");
    }

    #[test]
    fn set_mod_path_enabled_requires_game_folder_and_existing_mods_folder() {
        let temp = tempfile::tempdir().unwrap();
        let mod_path = temp.path().join("law.zip");
        fs::write(&mod_path, b"zip").unwrap();

        assert_eq!(
            set_mod_path_enabled(
                &LauncherConfig::default(),
                &mod_path.to_string_lossy(),
                false
            )
            .unwrap_err(),
            "Set a game folder first."
        );

        let config = LauncherConfig {
            game_folder: Some(temp.path().to_string_lossy().to_string()),
            ..LauncherConfig::default()
        };
        assert_eq!(
            set_mod_path_enabled(&config, &mod_path.to_string_lossy(), false).unwrap_err(),
            "Mods or plugins folder does not exist."
        );
    }

    #[test]
    fn checked_mod_path_for_config_returns_canonical_top_level_path() {
        let temp = tempfile::tempdir().unwrap();
        let mods_dir = temp.path().join("mods");
        fs::create_dir_all(&mods_dir).unwrap();
        let mod_path = mods_dir.join("law.zip");
        fs::write(&mod_path, b"zip").unwrap();
        let config = LauncherConfig {
            game_folder: Some(temp.path().to_string_lossy().to_string()),
            ..LauncherConfig::default()
        };

        let checked = checked_mod_path_for_config(&config, &mod_path.to_string_lossy()).unwrap();

        assert_eq!(checked, mod_path.canonicalize().unwrap());
    }

    #[test]
    fn canonical_existing_mod_path_rejects_missing_path() {
        let err = canonical_existing_mod_path("/path/that/does/not/exist").unwrap_err();

        assert_eq!(err, "Mod path does not exist.");
    }

    #[test]
    fn require_top_level_mod_path_rejects_sibling_and_nested_paths() {
        let content_dirs = vec![PathBuf::from("/game/mods"), PathBuf::from("/game/plugins")];
        let sibling = Path::new("/game/other.zip");
        let nested = Path::new("/game/mods/folder/mod.zip");

        assert!(require_top_level_mod_path(sibling, &content_dirs).is_err());
        assert!(require_top_level_mod_path(nested, &content_dirs).is_err());
        assert!(require_top_level_mod_path(Path::new("/game/mods/mod.zip"), &content_dirs).is_ok());
        assert!(require_top_level_mod_path(Path::new("/game/plugins/plugin.zip"), &content_dirs).is_ok());
    }

    #[test]
    fn mod_file_name_rejects_paths_without_file_name() {
        assert_eq!(
            mod_file_name(Path::new("/")).unwrap_err(),
            "Invalid mod path."
        );
    }
}
