use super::super::types::LocalModMetadata;
use base64::{engine::general_purpose, Engine as _};
use std::{
    fs,
    io::{Cursor, Read, Seek},
    path::Path,
};
use zip::ZipArchive;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

    #[test]
    fn reads_metadata_toml_and_cover_data_url() {
        let bytes = metadata_zip(
            "title = \"Casual Law\"\nversion = \"1.0.0\"\nmod_id = \"casual-law\"\ndependencies = [\"base-law\"]\ncover = \".metadata/cover.png\"\n",
            Some(("png", ".metadata/cover.png")),
        );

        let metadata = read_mod_metadata_from_bytes(&bytes).unwrap();

        assert_eq!(metadata.title.as_deref(), Some("Casual Law"));
        assert_eq!(metadata.version.as_deref(), Some("1.0.0"));
        assert_eq!(metadata.mod_id.as_deref(), Some("casual-law"));
        assert_eq!(metadata.dependencies, vec!["base-law"]);
        assert!(metadata
            .cover_data_url
            .as_deref()
            .unwrap_or_default()
            .starts_with("data:image/png;base64,"));
    }

    #[test]
    fn returns_empty_metadata_when_zip_has_no_metadata_toml() {
        let bytes = zip_without_metadata();

        let metadata = read_mod_metadata_from_bytes(&bytes).unwrap();

        assert!(metadata.title.is_none());
        assert!(metadata.mod_id.is_none());
        assert!(metadata.dependencies.is_empty());
    }

    fn metadata_zip(metadata: &str, cover: Option<(&str, &str)>) -> Vec<u8> {
        let mut writer = ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        writer.start_file("metadata.toml", options).unwrap();
        writer.write_all(metadata.as_bytes()).unwrap();
        if let Some((bytes, path)) = cover {
            writer.start_file(path, options).unwrap();
            writer.write_all(bytes.as_bytes()).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    fn zip_without_metadata() -> Vec<u8> {
        let mut writer = ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        writer.start_file("mod/file.txt", options).unwrap();
        writer.write_all(b"content").unwrap();
        writer.finish().unwrap().into_inner()
    }
}
