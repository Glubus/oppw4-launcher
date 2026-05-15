use crate::{
    config::LauncherConfig,
    models::{InstalledMod, LocalModMetadata},
    system_utils::now_label,
};
use base64::{engine::general_purpose, Engine as _};
use std::{
    collections::HashSet,
    fs,
    io::{Cursor, Read, Seek, Write},
    path::{Path, PathBuf},
};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

pub(crate) fn installed_dependency_keys(mods: &[InstalledMod]) -> HashSet<String> {
    mods.iter()
        .flat_map(installed_dependency_keys_for_mod)
        .map(|key| key.to_lowercase())
        .collect()
}

fn installed_dependency_keys_for_mod(mod_info: &InstalledMod) -> Vec<String> {
    let mut keys = vec![mod_info.mod_key.clone()];
    push_optional_dependency_keys(&mut keys, "id", &mod_info.mod_id);
    push_optional_dependency_keys(&mut keys, "slug", &mod_info.slug);
    push_optional_dependency_keys(&mut keys, "source", &mod_info.source_url);
    keys
}

fn push_optional_dependency_keys(keys: &mut Vec<String>, prefix: &str, value: &Option<String>) {
    if let Some(value) = value {
        keys.push(value.clone());
        keys.push(format!("{prefix}:{value}"));
    }
}

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

fn metadata_for_installed_mod(path: &Path, kind: &str) -> LocalModMetadata {
    if kind == "zip" {
        read_local_mod_metadata(path).unwrap_or_default()
    } else {
        LocalModMetadata::default()
    }
}

pub(crate) fn mod_key_for(display_name: &str, metadata: &LocalModMetadata) -> String {
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
        .unwrap_or_else(|| local_mod_key(display_name))
}

fn local_mod_key(display_name: &str) -> String {
    format!(
        "local:{}",
        display_name.trim_end_matches(".zip").to_lowercase()
    )
}

pub(crate) fn read_local_mod_metadata(path: &Path) -> Result<LocalModMetadata, String> {
    let file = fs::File::open(path).map_err(|err| format!("Could not open mod ZIP: {err}"))?;
    let mut archive =
        ZipArchive::new(file).map_err(|err| format!("Could not read mod ZIP: {err}"))?;
    read_mod_metadata_from_archive(&mut archive)
}

pub(crate) fn read_mod_metadata_from_bytes(bytes: &[u8]) -> Result<LocalModMetadata, String> {
    let reader = Cursor::new(bytes);
    let mut archive =
        ZipArchive::new(reader).map_err(|err| format!("Could not read mod ZIP: {err}"))?;
    read_mod_metadata_from_archive(&mut archive)
}

fn read_mod_metadata_from_archive<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
) -> Result<LocalModMetadata, String> {
    let Some(content) = metadata_toml_content(archive)? else {
        return Ok(LocalModMetadata::default());
    };
    let mut metadata = parse_mod_metadata_toml(&content);
    metadata.cover_data_url = cover_data_url_from_metadata(archive, &content);
    Ok(metadata)
}

fn metadata_toml_content<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
) -> Result<Option<String>, String> {
    let Ok(mut entry) = archive.by_name("metadata.toml") else {
        return Ok(None);
    };
    let mut content = String::new();
    entry
        .read_to_string(&mut content)
        .map_err(|err| format!("Could not read metadata.toml: {err}"))?;
    Ok(Some(content))
}

fn parse_mod_metadata_toml(content: &str) -> LocalModMetadata {
    LocalModMetadata {
        mod_id: toml_value(content, "mod_id"),
        title: toml_value(content, "title"),
        version: toml_value(content, "version"),
        source_url: toml_value(content, "source_url"),
        slug: toml_value(content, "slug"),
        character_name: toml_value(content, "character_name"),
        character_slug: toml_value(content, "character_slug"),
        mod_type: toml_value(content, "mod_type"),
        dependencies: toml_array(content, "dependencies"),
        changelog: toml_value(content, "changelog"),
        cover_data_url: None,
    }
}

fn cover_data_url_from_metadata<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    content: &str,
) -> Option<String> {
    let cover_path = toml_value(content, "cover")?;
    cover_path
        .starts_with(".metadata/")
        .then(|| zip_image_data_url(archive, &cover_path).ok())
        .flatten()
}

pub(crate) fn same_mod_version(mod_info: &InstalledMod, metadata: &LocalModMetadata) -> bool {
    metadata.version.is_some()
        && same_mod_identity(mod_info, metadata)
        && mod_info.version.as_ref() == metadata.version.as_ref()
}

pub(crate) fn same_mod_identity(mod_info: &InstalledMod, metadata: &LocalModMetadata) -> bool {
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
        parse_toml_scalar(value)
    })
}

fn parse_toml_scalar(value: &str) -> Option<String> {
    if value == "\"\"" {
        return None;
    }
    if value.starts_with('"') && value.ends_with('"') {
        return serde_json::from_str::<String>(value)
            .ok()
            .filter(|value| !value.trim().is_empty());
    }
    Some(value.to_string()).filter(|value| !value.trim().is_empty())
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
    let bytes = zip_entry_bytes(archive, path, "cover image")?;
    let mime = cover_image_mime(path)?;
    Ok(format!(
        "data:{mime};base64,{}",
        general_purpose::STANDARD.encode(bytes)
    ))
}

fn zip_entry_bytes<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
    label: &str,
) -> Result<Vec<u8>, String> {
    let mut entry = archive
        .by_name(path)
        .map_err(|err| format!("Could not read {label}: {err}"))?;
    let mut bytes = Vec::new();
    entry
        .read_to_end(&mut bytes)
        .map_err(|err| format!("Could not read {label}: {err}"))?;
    Ok(bytes)
}

fn cover_image_mime(path: &str) -> Result<&'static str, String> {
    if path.ends_with(".png") {
        Ok("image/png")
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        Ok("image/jpeg")
    } else if path.ends_with(".webp") {
        Ok("image/webp")
    } else {
        Err("Unsupported cover image type.".to_string())
    }
}

pub(crate) fn read_metadata_entries(bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>, String> {
    let reader = Cursor::new(bytes);
    let mut archive =
        ZipArchive::new(reader).map_err(|err| format!("Could not open metadata ZIP: {err}"))?;
    (0..archive.len())
        .filter_map(|index| read_metadata_entry(&mut archive, index).transpose())
        .collect()
}

fn read_metadata_entry<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    index: usize,
) -> Result<Option<(String, Vec<u8>)>, String> {
    let mut entry = archive
        .by_index(index)
        .map_err(|err| format!("Could not read metadata ZIP: {err}"))?;
    if entry.is_dir() {
        return Ok(None);
    }
    let name = entry.name().replace('\\', "/");
    validate_metadata_entry_name(&name)?;
    if !is_metadata_entry_name(&name) {
        return Ok(None);
    }
    let mut content = Vec::new();
    entry
        .read_to_end(&mut content)
        .map_err(|err| format!("Could not read metadata entry: {err}"))?;
    Ok(Some((name, content)))
}

fn validate_metadata_entry_name(name: &str) -> Result<(), String> {
    if name.contains("..") || name.starts_with('/') {
        Err("Metadata ZIP contains an unsafe path.".to_string())
    } else {
        Ok(())
    }
}

fn is_metadata_entry_name(name: &str) -> bool {
    name == "metadata.toml" || name.starts_with(".metadata/")
}

pub(crate) fn inject_metadata_entries(
    target_path: &Path,
    metadata_entries: Vec<(String, Vec<u8>)>,
) -> Result<(), String> {
    let temp_path = target_path.with_extension("zip.metadata-tmp");
    let backup_path = target_path.with_extension("zip.metadata-backup");
    write_metadata_replaced_zip(target_path, &temp_path, metadata_entries)?;
    replace_zip_with_backup(target_path, &temp_path, &backup_path)
}

fn write_metadata_replaced_zip(
    target_path: &Path,
    temp_path: &Path,
    metadata_entries: Vec<(String, Vec<u8>)>,
) -> Result<(), String> {
    let mut source = open_zip_archive(target_path, "selected ZIP")?;
    let temp_file = fs::File::create(temp_path)
        .map_err(|err| format!("Could not create temporary ZIP: {err}"))?;
    let mut writer = ZipWriter::new(temp_file);
    copy_non_metadata_entries(&mut source, &mut writer)?;
    write_metadata_entries(&mut writer, metadata_entries)?;
    writer
        .finish()
        .map_err(|err| format!("Could not finish ZIP: {err}"))?;
    Ok(())
}

fn open_zip_archive(path: &Path, label: &str) -> Result<ZipArchive<fs::File>, String> {
    let file = fs::File::open(path).map_err(|err| format!("Could not open {label}: {err}"))?;
    ZipArchive::new(file).map_err(|err| format!("Could not read {label}: {err}"))
}

fn copy_non_metadata_entries<R: Read + Seek, W: Write + Seek>(
    source: &mut ZipArchive<R>,
    writer: &mut ZipWriter<W>,
) -> Result<(), String> {
    (0..source.len()).try_for_each(|index| copy_non_metadata_entry(source, writer, index))
}

fn copy_non_metadata_entry<R: Read + Seek, W: Write + Seek>(
    source: &mut ZipArchive<R>,
    writer: &mut ZipWriter<W>,
    index: usize,
) -> Result<(), String> {
    let entry = source
        .by_index(index)
        .map_err(|err| format!("Could not read selected ZIP entry: {err}"))?;
    let name = entry.name().replace('\\', "/");
    if is_metadata_entry_name(&name) {
        return Ok(());
    }
    writer
        .raw_copy_file(entry)
        .map_err(|err| format!("Could not copy selected ZIP entry: {err}"))
}

fn write_metadata_entries<W: Write + Seek>(
    writer: &mut ZipWriter<W>,
    metadata_entries: Vec<(String, Vec<u8>)>,
) -> Result<(), String> {
    metadata_entries
        .into_iter()
        .try_for_each(|(name, content)| write_metadata_entry(writer, name, &content))
}

fn write_metadata_entry<W: Write + Seek>(
    writer: &mut ZipWriter<W>,
    name: String,
    content: &[u8],
) -> Result<(), String> {
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    writer
        .start_file(name, options)
        .map_err(|err| format!("Could not write metadata entry: {err}"))?;
    writer
        .write_all(content)
        .map_err(|err| format!("Could not write metadata entry: {err}"))
}

fn replace_zip_with_backup(
    target_path: &Path,
    temp_path: &Path,
    backup_path: &Path,
) -> Result<(), String> {
    fs::copy(target_path, backup_path)
        .map_err(|err| format!("Could not create ZIP backup: {err}"))?;
    if let Err(err) = replace_file(temp_path, target_path) {
        let _ = fs::copy(backup_path, target_path);
        let _ = fs::remove_file(temp_path);
        return Err(err);
    }
    let _ = fs::remove_file(backup_path);
    Ok(())
}

pub(crate) fn available_mod_path(mods_dir: &Path, file_name: &str) -> PathBuf {
    let stem = safe_mod_stem(file_name);
    first_available_mod_path(mods_dir, &stem)
        .unwrap_or_else(|| mods_dir.join(format!("{stem}-{}.zip", now_label())))
}

fn safe_mod_stem(file_name: &str) -> String {
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
    if stem.is_empty() {
        "mod".to_string()
    } else {
        stem
    }
}

fn first_available_mod_path(mods_dir: &Path, stem: &str) -> Option<PathBuf> {
    (0..1000)
        .map(|index| candidate_mod_path(mods_dir, stem, index))
        .find(|candidate| !candidate.exists())
}

fn candidate_mod_path(mods_dir: &Path, stem: &str, index: usize) -> PathBuf {
    let name = if index == 0 {
        format!("{stem}.zip")
    } else {
        format!("{stem}-{index}.zip")
    };
    mods_dir.join(name)
}

fn replace_file(source: &Path, target: &Path) -> Result<(), String> {
    if target.exists() {
        fs::remove_file(target).map_err(|err| format!("Could not replace selected ZIP: {err}"))?;
    }
    fs::rename(source, target).map_err(|err| format!("Could not replace selected ZIP: {err}"))
}
