use super::{
    keys,
    metadata::reader,
    types::{InstalledMod, LocalModMetadata},
};
use crate::config::LauncherConfig;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn installed_mods(config: &LauncherConfig) -> Vec<InstalledMod> {
    let Some(mods_dir) = mods_dir(config) else {
        return Vec::new();
    };
    let Ok(entries) = fs::read_dir(mods_dir) else {
        return Vec::new();
    };

    let mut mods = entries
        .flatten()
        .filter_map(|entry| installed_mod_from_path(&entry.path()))
        .collect::<Vec<_>>();
    mods.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    mods
}

fn mods_dir(config: &LauncherConfig) -> Option<PathBuf> {
    config
        .game_folder
        .as_ref()
        .map(|game_folder| PathBuf::from(game_folder).join("mods"))
}

pub(crate) fn installed_mod_from_path(path: &Path) -> Option<InstalledMod> {
    let name = path.file_name()?.to_str()?.to_string();
    if should_skip_mod_entry(&name) {
        return None;
    }
    let kind = installed_mod_kind(path)?;
    let enabled = !name.ends_with(".disabled");
    let display_name = name.trim_end_matches(".disabled").to_string();
    installed_mod_from_parts(path.to_path_buf(), display_name, kind, enabled)
}

fn should_skip_mod_entry(name: &str) -> bool {
    name == "_oppw4" || name.starts_with('.')
}

fn installed_mod_kind(path: &Path) -> Option<String> {
    if path.is_dir() {
        return Some("folder".to_string());
    }
    path.extension()
        .and_then(|value| value.to_str())
        .filter(|ext| ext.eq_ignore_ascii_case("zip") || ext.eq_ignore_ascii_case("disabled"))
        .map(|_| "zip".to_string())
}

fn installed_mod_from_parts(
    path: PathBuf,
    display_name: String,
    kind: String,
    enabled: bool,
) -> Option<InstalledMod> {
    let metadata = metadata_for_installed_mod(&path, &kind);
    let mod_key = keys::mod_key_for(&display_name, &metadata);
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

fn metadata_for_installed_mod(path: &Path, kind: &str) -> LocalModMetadata {
    if kind == "zip" {
        reader::read_local_mod_metadata(path).unwrap_or_default()
    } else {
        LocalModMetadata::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LauncherConfig;

    #[test]
    fn installed_mods_ignores_hidden_and_internal_entries() {
        let temp = tempfile::tempdir().unwrap();
        let mods_dir = temp.path().join("mods");
        fs::create_dir_all(mods_dir.join("_oppw4")).unwrap();
        fs::write(mods_dir.join(".hidden.zip"), b"hidden").unwrap();
        fs::write(mods_dir.join("visible.disabled"), b"zip-ish").unwrap();

        let mods = installed_mods(&LauncherConfig {
            game_folder: Some(temp.path().to_string_lossy().to_string()),
            ..LauncherConfig::default()
        });

        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].name, "visible");
        assert!(!mods[0].enabled);
    }

    #[test]
    fn installed_mods_returns_empty_without_game_folder_or_mods_dir() {
        assert!(installed_mods(&LauncherConfig::default()).is_empty());

        let temp = tempfile::tempdir().unwrap();
        let config = LauncherConfig {
            game_folder: Some(temp.path().to_string_lossy().to_string()),
            ..LauncherConfig::default()
        };

        assert!(installed_mods(&config).is_empty());
    }

    #[test]
    fn installed_mod_from_path_detects_folder_zip_and_disabled_zip() {
        let temp = tempfile::tempdir().unwrap();
        let folder = temp.path().join("Folder Mod");
        fs::create_dir_all(&folder).unwrap();
        let zip = temp.path().join("Zip Mod.ZIP");
        fs::write(&zip, b"not a real zip").unwrap();
        let disabled = temp.path().join("Disabled.zip.disabled");
        fs::write(&disabled, b"disabled").unwrap();

        let folder_mod = installed_mod_from_path(&folder).unwrap();
        let zip_mod = installed_mod_from_path(&zip).unwrap();
        let disabled_mod = installed_mod_from_path(&disabled).unwrap();

        assert_eq!(folder_mod.kind, "folder");
        assert!(folder_mod.enabled);
        assert_eq!(zip_mod.kind, "zip");
        assert!(zip_mod.enabled);
        assert_eq!(disabled_mod.name, "Disabled.zip");
        assert_eq!(disabled_mod.mod_key, "local:disabled");
        assert!(!disabled_mod.enabled);
    }

    #[test]
    fn installed_mods_sorts_case_insensitively() {
        let temp = tempfile::tempdir().unwrap();
        let mods_dir = temp.path().join("mods");
        fs::create_dir_all(&mods_dir).unwrap();
        fs::create_dir_all(mods_dir.join("beta")).unwrap();
        fs::create_dir_all(mods_dir.join("Alpha")).unwrap();
        let config = LauncherConfig {
            game_folder: Some(temp.path().to_string_lossy().to_string()),
            ..LauncherConfig::default()
        };

        let mods = installed_mods(&config);

        assert_eq!(
            mods.iter().map(|mod_info| mod_info.name.as_str()).collect::<Vec<_>>(),
            vec!["Alpha", "beta"]
        );
    }
}
