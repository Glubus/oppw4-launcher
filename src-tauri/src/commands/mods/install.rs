use super::{
    inventory, keys,
    metadata::reader,
    paths,
    types::{
        ImportExternalZipRequest, InstallHostedModRequest, InstallHostedModResult, InstalledMod,
        InstalledModLookupRequest, LocalModMetadata,
    },
};
use crate::{config::load_config as read_config, API_BASE};
use std::{fs, path::PathBuf};

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn import_external_zip(input: ImportExternalZipRequest) -> Result<InstalledMod, String> {
    let config = read_config()?;
    import_external_zip_to_game(&config, &input.path)
}

fn import_external_zip_to_game(
    config: &crate::config::LauncherConfig,
    selected: &str,
) -> Result<InstalledMod, String> {
    let selected = valid_external_zip_path(selected)?;
    let metadata = reader::read_local_mod_metadata(&selected).unwrap_or_default();
    let target_dir = ensure_content_dir(config, metadata.content_kind.as_deref())?;
    let target = copy_external_zip_to_content_dir(&target_dir, &selected)?;
    scan_imported_zip(&target)
}

fn valid_external_zip_path(path: &str) -> Result<PathBuf, String> {
    let selected = PathBuf::from(path);
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
    Ok(selected)
}

fn copy_external_zip_to_content_dir(
    content_dir: &std::path::Path,
    selected: &std::path::Path,
) -> Result<PathBuf, String> {
    let file_name = selected
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Invalid ZIP file name.".to_string())?;
    let target = paths::available_mod_path(content_dir, file_name);
    fs::copy(selected, &target).map_err(|err| format!("Could not import ZIP: {err}"))?;
    Ok(target)
}

fn scan_imported_zip(target: &std::path::Path) -> Result<InstalledMod, String> {
    inventory::installed_mod_from_path(target)
        .ok_or_else(|| "Imported ZIP could not be scanned.".to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn install_hosted_mod(
    input: InstallHostedModRequest,
) -> Result<InstallHostedModResult, String> {
    let config = read_config()?;
    let bytes = download_hosted_file(&input.file_id)?;

    if input.content_kind.as_deref() == Some("plugin") {
        validate_downloaded_dll(&bytes)?;
        return install_hosted_plugin_dll(&config, &input, &bytes);
    }

    validate_downloaded_zip(&bytes)?;
    let metadata = reader::read_mod_metadata_from_bytes(&bytes).unwrap_or_default();
    let content_dir = ensure_content_dir(&config, metadata.content_kind.as_deref())?;

    if !input.install_as_new {
        if let Some(result) =
            existing_hosted_mod_result(&config, &input.file_name, &bytes, &metadata)?
        {
            return Ok(result);
        }
    }

    install_new_hosted_mod(&content_dir, &input.file_name, &bytes, &metadata)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn installed_mod_for_skin(
    input: InstalledModLookupRequest,
) -> Result<Option<InstalledMod>, String> {
    let config = read_config()?;
    Ok(inventory::installed_mods(&config)
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

fn ensure_content_dir(
    config: &crate::config::LauncherConfig,
    content_kind: Option<&str>,
) -> Result<PathBuf, String> {
    let game_folder = config
        .game_folder
        .clone()
        .ok_or_else(|| "Set a game folder first.".to_string())?;
    let folder_name = if content_kind == Some("plugin") { "plugins" } else { "mods" };
    let content_dir = PathBuf::from(game_folder).join(folder_name);
    fs::create_dir_all(&content_dir)
        .map_err(|err| format!("Could not create {folder_name} folder: {err}"))?;
    Ok(content_dir)
}

fn download_hosted_file(file_id: &str) -> Result<Vec<u8>, String> {
    let url = format!("{API_BASE}/files/{file_id}/download");
    reqwest::blocking::Client::new()
        .get(url)
        .header("accept", "application/octet-stream,application/zip")
        .header("user-agent", "oppw4-launcher")
        .send()
        .map_err(|err| format!("Could not download mod: {err}"))?
        .error_for_status()
        .map_err(|err| format!("Mod download failed: {err}"))?
        .bytes()
        .map(|bytes| bytes.to_vec())
        .map_err(|err| format!("Could not read download: {err}"))
}

fn validate_downloaded_zip(bytes: &[u8]) -> Result<(), String> {
    bytes
        .starts_with(b"PK")
        .then_some(())
        .ok_or_else(|| "Downloaded file is not a ZIP archive.".to_string())
}

fn validate_downloaded_dll(bytes: &[u8]) -> Result<(), String> {
    bytes
        .starts_with(b"MZ")
        .then_some(())
        .ok_or_else(|| "Downloaded plugin is not a DLL.".to_string())
}

fn install_hosted_plugin_dll(
    config: &crate::config::LauncherConfig,
    input: &InstallHostedModRequest,
    bytes: &[u8],
) -> Result<InstallHostedModResult, String> {
    let plugins_dir = ensure_content_dir(config, Some("plugin"))?;
    let slug = input
        .slug
        .as_deref()
        .map(safe_folder_name)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| safe_folder_name(&input.file_name.trim_end_matches(".dll")));
    let plugin_dir = plugins_dir.join(&slug);
    fs::create_dir_all(&plugin_dir)
        .map_err(|err| format!("Could not create plugin folder: {err}"))?;

    let dll_name = safe_dll_name(&input.file_name)?;
    let target = plugin_dir.join(&dll_name);
    let already_up_to_date = target.exists()
        && fs::read(&target)
            .map(|existing| existing == bytes)
            .unwrap_or(false);
    if !already_up_to_date {
        fs::write(&target, bytes).map_err(|err| format!("Could not write plugin DLL: {err}"))?;
    }

    let metadata = LocalModMetadata {
        mod_id: None,
        title: input.title.clone(),
        version: input.version.clone(),
        source_url: input.source_code_url.clone(),
        slug: Some(slug.clone()),
        content_kind: Some("plugin".to_string()),
        character_name: None,
        character_slug: None,
        mod_type: Some("plugin".to_string()),
        dependencies: Vec::new(),
        changelog: None,
        cover_data_url: None,
    };
    write_plugin_metadata_toml(&plugin_dir, &metadata, &dll_name)?;

    Ok(InstallHostedModResult {
        mod_info: installed_mod_from_metadata(
            plugin_dir.to_string_lossy().to_string(),
            "folder".to_string(),
            true,
            input.title.clone().unwrap_or(slug.clone()),
            format!("plugin:{slug}"),
            &metadata,
        ),
        already_up_to_date,
    })
}

fn write_plugin_metadata_toml(
    plugin_dir: &std::path::Path,
    metadata: &LocalModMetadata,
    dll_name: &str,
) -> Result<(), String> {
    let mut lines = Vec::new();
    push_toml_string(&mut lines, "title", metadata.title.as_deref());
    push_toml_string(&mut lines, "version", metadata.version.as_deref());
    push_toml_string(&mut lines, "source_url", metadata.source_url.as_deref());
    push_toml_string(&mut lines, "slug", metadata.slug.as_deref());
    push_toml_string(&mut lines, "content_kind", metadata.content_kind.as_deref());
    push_toml_string(&mut lines, "mod_type", metadata.mod_type.as_deref());
    push_toml_string(&mut lines, "plugin_dll", Some(dll_name));
    let content = format!("{}\n", lines.join("\n"));
    fs::write(plugin_dir.join("metadata.toml"), content)
        .map_err(|err| format!("Could not write plugin metadata: {err}"))
}

fn push_toml_string(lines: &mut Vec<String>, key: &str, value: Option<&str>) {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return;
    };
    let encoded = serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string());
    lines.push(format!("{key} = {encoded}"));
}

fn safe_dll_name(file_name: &str) -> Result<String, String> {
    let path = PathBuf::from(file_name);
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Invalid DLL file name.".to_string())?;
    if !name.to_lowercase().ends_with(".dll") {
        return Err("Plugin file must be a DLL.".to_string());
    }
    Ok(name.to_string())
}

fn safe_folder_name(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn existing_hosted_mod_result(
    config: &crate::config::LauncherConfig,
    file_name: &str,
    bytes: &[u8],
    metadata: &LocalModMetadata,
) -> Result<Option<InstallHostedModResult>, String> {
    let installed_mods = inventory::installed_mods(config);
    if let Some(existing) = installed_mods
        .iter()
        .find(|mod_info| keys::same_mod_version(mod_info, metadata))
    {
        return Ok(Some(InstallHostedModResult {
            mod_info: clone_installed_mod(existing),
            already_up_to_date: true,
        }));
    }
    installed_mods
        .into_iter()
        .find(|mod_info| keys::same_mod_identity(mod_info, metadata))
        .map(|existing| update_existing_hosted_mod(existing, file_name, bytes, metadata))
        .transpose()
}

fn update_existing_hosted_mod(
    existing: InstalledMod,
    file_name: &str,
    bytes: &[u8],
    metadata: &LocalModMetadata,
) -> Result<InstallHostedModResult, String> {
    fs::write(&existing.path, bytes).map_err(|err| format!("Could not update mod ZIP: {err}"))?;
    Ok(InstallHostedModResult {
        mod_info: installed_mod_from_metadata(
            existing.path,
            existing.kind,
            existing.enabled,
            metadata
                .title
                .clone()
                .unwrap_or_else(|| existing.name.clone()),
            keys::mod_key_for(file_name, metadata),
            metadata,
        ),
        already_up_to_date: false,
    })
}

fn install_new_hosted_mod(
    mods_dir: &std::path::Path,
    file_name: &str,
    bytes: &[u8],
    metadata: &LocalModMetadata,
) -> Result<InstallHostedModResult, String> {
    let target = paths::available_mod_path(mods_dir, file_name);
    fs::write(&target, bytes).map_err(|err| format!("Could not write mod ZIP: {err}"))?;
    let name = target
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("installed.zip")
        .to_string();
    Ok(InstallHostedModResult {
        mod_info: installed_mod_from_metadata(
            target.to_string_lossy().to_string(),
            "zip".to_string(),
            true,
            metadata
                .title
                .clone()
                .unwrap_or_else(|| name.trim_end_matches(".disabled").to_string()),
            keys::mod_key_for(&name, metadata),
            metadata,
        ),
        already_up_to_date: false,
    })
}

fn installed_mod_from_metadata(
    path: String,
    kind: String,
    enabled: bool,
    name: String,
    mod_key: String,
    metadata: &LocalModMetadata,
) -> InstalledMod {
    InstalledMod {
        name,
        kind,
        path,
        mod_key,
        enabled,
        mod_id: metadata.mod_id.clone(),
        version: metadata.version.clone(),
        source_url: metadata.source_url.clone(),
        slug: metadata.slug.clone(),
        content_kind: metadata.content_kind.clone().unwrap_or_else(|| "mod".to_string()),
        character_name: metadata.character_name.clone(),
        character_slug: metadata.character_slug.clone(),
        mod_type: metadata.mod_type.clone(),
        dependencies: metadata.dependencies.clone(),
        changelog: metadata.changelog.clone(),
        cover_data_url: metadata.cover_data_url.clone(),
    }
}

fn clone_installed_mod(mod_info: &InstalledMod) -> InstalledMod {
    InstalledMod {
        name: mod_info.name.clone(),
        kind: mod_info.kind.clone(),
        path: mod_info.path.clone(),
        mod_key: mod_info.mod_key.clone(),
        enabled: mod_info.enabled,
        mod_id: mod_info.mod_id.clone(),
        version: mod_info.version.clone(),
        source_url: mod_info.source_url.clone(),
        slug: mod_info.slug.clone(),
        content_kind: mod_info.content_kind.clone(),
        character_name: mod_info.character_name.clone(),
        character_slug: mod_info.character_slug.clone(),
        mod_type: mod_info.mod_type.clone(),
        dependencies: mod_info.dependencies.clone(),
        changelog: mod_info.changelog.clone(),
        cover_data_url: mod_info.cover_data_url.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

    #[test]
    fn valid_external_zip_path_accepts_zip_case_insensitively() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("mod.ZIP");
        fs::write(&path, b"zip").unwrap();

        assert_eq!(
            valid_external_zip_path(&path.to_string_lossy()).unwrap(),
            path
        );
    }

    #[test]
    fn valid_external_zip_path_rejects_missing_non_file_and_non_zip() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().join("folder.zip");
        fs::create_dir_all(&dir).unwrap();
        let txt = temp.path().join("mod.txt");
        fs::write(&txt, b"text").unwrap();

        assert_eq!(
            valid_external_zip_path(&temp.path().join("missing.zip").to_string_lossy())
                .unwrap_err(),
            "Selected ZIP does not exist."
        );
        assert_eq!(
            valid_external_zip_path(&dir.to_string_lossy()).unwrap_err(),
            "Selected ZIP does not exist."
        );
        assert_eq!(
            valid_external_zip_path(&txt.to_string_lossy()).unwrap_err(),
            "Select a .zip mod archive."
        );
    }

    #[test]
    fn import_external_zip_to_dir_copies_to_available_path_and_scans_mod() {
        let temp = tempfile::tempdir().unwrap();
        let mods_dir = temp.path().join("mods");
        fs::create_dir_all(&mods_dir).unwrap();
        fs::write(mods_dir.join("law.zip"), b"existing").unwrap();
        let source = temp.path().join("law.zip");
        write_metadata_zip(&source, "title = \"Imported Law\"\nmod_id = \"law\"\n");

        let config = crate::config::LauncherConfig {
            game_folder: Some(temp.path().to_string_lossy().to_string()),
            ..crate::config::LauncherConfig::default()
        };
        let imported = import_external_zip_to_game(&config, &source.to_string_lossy()).unwrap();

        assert_eq!(imported.name, "Imported Law");
        assert_eq!(imported.mod_key, "id:law");
        assert!(imported.path.ends_with("law-1.zip"));
        assert!(mods_dir.join("law-1.zip").exists());
    }

    #[test]
    fn import_external_zip_installs_plugins_to_plugins_folder() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("lua-core.zip");
        write_metadata_zip(&source, "title = \"Lua Core\"\ncontent_kind = \"plugin\"\n");
        let config = crate::config::LauncherConfig {
            game_folder: Some(temp.path().to_string_lossy().to_string()),
            ..crate::config::LauncherConfig::default()
        };

        let imported = import_external_zip_to_game(&config, &source.to_string_lossy()).unwrap();

        assert_eq!(imported.content_kind, "plugin");
        assert!(temp.path().join("plugins").join("lua-core.zip").exists());
    }

    #[test]
    fn validate_downloaded_zip_requires_zip_signature() {
        assert!(validate_downloaded_zip(b"PK\x03\x04payload").is_ok());
        assert_eq!(
            validate_downloaded_zip(b"not zip").unwrap_err(),
            "Downloaded file is not a ZIP archive."
        );
    }

    #[test]
    fn installed_mod_from_metadata_copies_metadata_fields() {
        let metadata = LocalModMetadata {
            mod_id: Some("id".to_string()),
            version: Some("1.0.0".to_string()),
            source_url: Some("https://example.test".to_string()),
            slug: Some("slug".to_string()),
            content_kind: Some("mod".to_string()),
            character_name: Some("Law".to_string()),
            character_slug: Some("law".to_string()),
            mod_type: Some("skin".to_string()),
            dependencies: vec!["base".to_string()],
            changelog: Some("Changed".to_string()),
            cover_data_url: Some("data:image/png;base64,abc".to_string()),
            ..LocalModMetadata::default()
        };

        let mod_info = installed_mod_from_metadata(
            "/mods/law.zip".to_string(),
            "zip".to_string(),
            true,
            "Law".to_string(),
            "id:id".to_string(),
            &metadata,
        );

        assert_eq!(mod_info.name, "Law");
        assert_eq!(mod_info.mod_id.as_deref(), Some("id"));
        assert_eq!(mod_info.content_kind, "mod");
        assert_eq!(mod_info.dependencies, vec!["base"]);
        assert_eq!(mod_info.cover_data_url, metadata.cover_data_url);
    }

    #[test]
    fn install_new_hosted_mod_uses_metadata_title_and_generates_path() {
        let temp = tempfile::tempdir().unwrap();
        let metadata = LocalModMetadata {
            title: Some("Hosted Mod".to_string()),
            slug: Some("hosted-mod".to_string()),
            ..LocalModMetadata::default()
        };

        let result =
            install_new_hosted_mod(temp.path(), "hosted.zip", b"PK zip bytes", &metadata).unwrap();

        assert!(!result.already_up_to_date);
        assert_eq!(result.mod_info.name, "Hosted Mod");
        assert_eq!(result.mod_info.mod_key, "slug:hosted-mod");
        assert_eq!(
            fs::read(temp.path().join("hosted.zip")).unwrap(),
            b"PK zip bytes"
        );
    }

    #[test]
    fn install_hosted_plugin_dll_writes_folder_metadata() {
        let temp = tempfile::tempdir().unwrap();
        let config = crate::config::LauncherConfig {
            game_folder: Some(temp.path().to_string_lossy().to_string()),
            ..crate::config::LauncherConfig::default()
        };
        let input = InstallHostedModRequest {
            file_id: "file".to_string(),
            file_name: "fx_director.dll".to_string(),
            content_kind: Some("plugin".to_string()),
            slug: Some("fx-director".to_string()),
            title: Some("FX Director".to_string()),
            version: Some("1.2.0".to_string()),
            source_code_url: Some("https://github.com/creator/fx-director".to_string()),
            install_as_new: false,
        };

        let result = install_hosted_plugin_dll(&config, &input, b"MZ dll bytes").unwrap();
        let plugin_dir = temp.path().join("plugins").join("fx-director");
        let metadata = fs::read_to_string(plugin_dir.join("metadata.toml")).unwrap();

        assert_eq!(result.mod_info.name, "FX Director");
        assert_eq!(result.mod_info.content_kind, "plugin");
        assert!(plugin_dir.join("fx_director.dll").exists());
        assert!(metadata.contains("title = \"FX Director\""));
        assert!(metadata.contains("source_url = \"https://github.com/creator/fx-director\""));

        let scanned = inventory::installed_mod_from_path(&plugin_dir).unwrap();
        assert_eq!(scanned.name, "FX Director");
        assert_eq!(scanned.version.as_deref(), Some("1.2.0"));
        assert_eq!(scanned.source_url.as_deref(), Some("https://github.com/creator/fx-director"));
        assert_eq!(scanned.slug.as_deref(), Some("fx-director"));
        assert_eq!(scanned.content_kind, "plugin");
    }

    fn write_metadata_zip(path: &std::path::Path, metadata: &str) {
        let file = fs::File::create(path).unwrap();
        let mut writer = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        writer.start_file("metadata.toml", options).unwrap();
        writer.write_all(metadata.as_bytes()).unwrap();
        writer.finish().unwrap();
    }
}
