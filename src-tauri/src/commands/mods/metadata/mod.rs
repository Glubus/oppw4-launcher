pub(crate) mod reader;
pub(crate) mod zip;

use super::{
    inventory, paths,
    types::{ApplyMetadataRequest, InstalledMod},
};
use crate::{config::load_config as read_config, API_BASE};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[tauri::command]
pub(crate) fn apply_metadata_to_zip(input: ApplyMetadataRequest) -> Result<InstalledMod, String> {
    let config = read_config()?;
    let mods_dir = ensure_mods_dir(&config)?;
    let target_path = PathBuf::from(input.zip_path);
    validate_selected_zip(&target_path)?;

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

    let metadata_entries = zip::read_metadata_entries(bytes.as_ref())?;
    if !metadata_entries
        .iter()
        .any(|entry| entry.0 == "metadata.toml")
    {
        return Err("Downloaded metadata ZIP does not contain metadata.toml.".to_string());
    }

    zip::inject_metadata_entries(&target_path, metadata_entries)?;
    let installed_path = move_zip_into_mods_dir(&target_path, &mods_dir)?;
    inventory::installed_mod_from_path(&installed_path)
        .ok_or_else(|| "Linked ZIP could not be scanned in the mods folder.".to_string())
}

fn ensure_mods_dir(config: &crate::config::LauncherConfig) -> Result<PathBuf, String> {
    let game_folder = config
        .game_folder
        .clone()
        .ok_or_else(|| "Set a game folder first.".to_string())?;
    let mods_dir = PathBuf::from(game_folder).join("mods");
    fs::create_dir_all(&mods_dir).map_err(|err| format!("Could not create mods folder: {err}"))?;
    Ok(mods_dir)
}

fn validate_selected_zip(path: &Path) -> Result<(), String> {
    if !path.exists() || !path.is_file() {
        return Err("Selected ZIP does not exist.".to_string());
    }
    if !path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
    {
        return Err("Select a ZIP archive.".to_string());
    }
    Ok(())
}

fn move_zip_into_mods_dir(target_path: &Path, mods_dir: &Path) -> Result<PathBuf, String> {
    if target_path
        .parent()
        .and_then(|parent| same_path(parent, mods_dir).ok())
        .unwrap_or(false)
    {
        return Ok(target_path.to_path_buf());
    }

    let file_name = target_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Invalid ZIP file name.".to_string())?;
    let destination = paths::available_mod_path(mods_dir, file_name);
    fs::rename(target_path, &destination)
        .or_else(|_| {
            fs::copy(target_path, &destination)
                .and_then(|_| fs::remove_file(target_path))
                .map(|_| ())
        })
        .map_err(|err| format!("Could not move ZIP into mods folder: {err}"))?;
    Ok(destination)
}

fn same_path(first: &Path, second: &Path) -> Result<bool, String> {
    let first = first
        .canonicalize()
        .map_err(|err| format!("Could not resolve selected ZIP folder: {err}"))?;
    let second = second
        .canonicalize()
        .map_err(|err| format!("Could not resolve mods folder: {err}"))?;
    Ok(first == second)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};
    use std::io::Write;

    #[test]
    fn move_zip_into_mods_dir_keeps_zip_already_in_mods_dir() {
        let temp = tempfile::tempdir().unwrap();
        let mods_dir = temp.path().join("mods");
        fs::create_dir_all(&mods_dir).unwrap();
        let source = mods_dir.join("law.zip");
        write_zip(&source);

        let moved = move_zip_into_mods_dir(&source, &mods_dir).unwrap();

        assert_eq!(moved, source);
        assert!(moved.exists());
    }

    #[test]
    fn move_zip_into_mods_dir_moves_external_zip_to_available_name() {
        let temp = tempfile::tempdir().unwrap();
        let mods_dir = temp.path().join("mods");
        fs::create_dir_all(&mods_dir).unwrap();
        fs::write(mods_dir.join("law.zip"), b"existing").unwrap();
        let source = temp.path().join("law.zip");
        write_zip(&source);

        let moved = move_zip_into_mods_dir(&source, &mods_dir).unwrap();

        assert_eq!(
            moved.file_name().and_then(|name| name.to_str()),
            Some("law-1.zip")
        );
        assert!(moved.exists());
        assert!(!source.exists());
    }

    fn write_zip(path: &Path) {
        let file = fs::File::create(path).unwrap();
        let mut writer = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        writer.start_file("metadata.toml", options).unwrap();
        writer.write_all(b"title = \"Law\"").unwrap();
        writer.finish().unwrap();
    }
}
