use super::types::{
    ImportExternalZipRequest, InstallHostedModRequest, InstallHostedModResult,
    InstalledModLookupRequest,
};
use crate::{config::load_config as read_config, mod_inventory, models::InstalledMod, API_BASE};
use std::{fs, path::PathBuf};

#[tauri::command]
pub(crate) fn import_external_zip(input: ImportExternalZipRequest) -> Result<InstalledMod, String> {
    let config = read_config()?;
    let game_folder = config
        .game_folder
        .clone()
        .ok_or_else(|| "Set a game folder first.".to_string())?;
    let mods_dir = PathBuf::from(game_folder).join("mods");
    fs::create_dir_all(&mods_dir).map_err(|err| format!("Could not create mods folder: {err}"))?;
    let selected = PathBuf::from(input.path);
    if !selected.exists() || !selected.is_file() {
        return Err("Selected ZIP does not exist.".to_string());
    }
    if !selected
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
    {
        return Err("Select a .zip mod archive.".to_string());
    }

    let file_name = selected
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Invalid ZIP file name.".to_string())?;
    let target = mod_inventory::available_mod_path(&mods_dir, file_name);
    fs::copy(&selected, &target).map_err(|err| format!("Could not import ZIP: {err}"))?;
    mod_inventory::installed_mod_from_path(&target)
        .ok_or_else(|| "Imported ZIP could not be scanned.".to_string())
}

#[tauri::command]
pub(crate) fn install_hosted_mod(
    input: InstallHostedModRequest,
) -> Result<InstallHostedModResult, String> {
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

    let downloaded_metadata =
        mod_inventory::read_mod_metadata_from_bytes(bytes.as_ref()).unwrap_or_default();
    if !input.install_as_new {
        if let Some(existing) = mod_inventory::installed_mods(&config)
            .into_iter()
            .find(|mod_info| mod_inventory::same_mod_version(mod_info, &downloaded_metadata))
        {
            return Ok(InstallHostedModResult {
                mod_info: existing,
                already_up_to_date: true,
            });
        }
        if let Some(existing) = mod_inventory::installed_mods(&config)
            .into_iter()
            .find(|mod_info| mod_inventory::same_mod_identity(mod_info, &downloaded_metadata))
        {
            fs::write(&existing.path, bytes)
                .map_err(|err| format!("Could not update mod ZIP: {err}"))?;
            let mod_key = mod_inventory::mod_key_for(&input.file_name, &downloaded_metadata);
            return Ok(InstallHostedModResult {
                mod_info: InstalledMod {
                    name: downloaded_metadata.title.unwrap_or(existing.name),
                    kind: existing.kind,
                    path: existing.path,
                    mod_key,
                    enabled: existing.enabled,
                    mod_id: downloaded_metadata.mod_id,
                    version: downloaded_metadata.version,
                    source_url: downloaded_metadata.source_url,
                    slug: downloaded_metadata.slug,
                    character_name: downloaded_metadata.character_name,
                    character_slug: downloaded_metadata.character_slug,
                    mod_type: downloaded_metadata.mod_type,
                    dependencies: downloaded_metadata.dependencies,
                    changelog: downloaded_metadata.changelog,
                    cover_data_url: downloaded_metadata.cover_data_url,
                },
                already_up_to_date: false,
            });
        }
    }

    let target = mod_inventory::available_mod_path(&mods_dir, &input.file_name);
    fs::write(&target, bytes).map_err(|err| format!("Could not write mod ZIP: {err}"))?;
    let name = target
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("installed.zip")
        .to_string();
    let mod_key = mod_inventory::mod_key_for(&name, &downloaded_metadata);

    Ok(InstallHostedModResult {
        mod_info: InstalledMod {
            name: downloaded_metadata
                .title
                .unwrap_or_else(|| name.trim_end_matches(".disabled").to_string()),
            kind: "zip".to_string(),
            path: target.to_string_lossy().to_string(),
            mod_key,
            enabled: true,
            mod_id: downloaded_metadata.mod_id,
            version: downloaded_metadata.version,
            source_url: downloaded_metadata.source_url,
            slug: downloaded_metadata.slug,
            character_name: downloaded_metadata.character_name,
            character_slug: downloaded_metadata.character_slug,
            mod_type: downloaded_metadata.mod_type,
            dependencies: downloaded_metadata.dependencies,
            changelog: downloaded_metadata.changelog,
            cover_data_url: downloaded_metadata.cover_data_url,
        },
        already_up_to_date: false,
    })
}

#[tauri::command]
pub(crate) fn installed_mod_for_skin(
    input: InstalledModLookupRequest,
) -> Result<Option<InstalledMod>, String> {
    let config = read_config()?;
    Ok(mod_inventory::installed_mods(&config)
        .into_iter()
        .find(|mod_info| {
            input
                .mod_id
                .as_ref()
                .is_some_and(|id| mod_info.mod_id.as_ref() == Some(id))
                || input
                    .slug
                    .as_ref()
                    .is_some_and(|slug| mod_info.slug.as_ref() == Some(slug))
        }))
}
