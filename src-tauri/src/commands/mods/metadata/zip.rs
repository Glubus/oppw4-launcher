use std::{
    fs,
    io::{Cursor, Read, Seek, Write},
    path::Path,
};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

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

fn replace_file(source: &Path, target: &Path) -> Result<(), String> {
    if target.exists() {
        fs::remove_file(target).map_err(|err| format!("Could not replace selected ZIP: {err}"))?;
    }
    fs::rename(source, target).map_err(|err| format!("Could not replace selected ZIP: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};

    #[test]
    fn reads_only_metadata_entries_and_rejects_unsafe_paths() {
        let safe = metadata_bundle(vec![
            ("metadata.toml", b"title = \"Safe\"".as_slice()),
            (".metadata/cover.png", b"png".as_slice()),
            ("mod/file.txt", b"ignore".as_slice()),
        ]);

        let entries = read_metadata_entries(&safe).unwrap();

        assert_eq!(entries.len(), 2);
        assert!(entries.iter().any(|entry| entry.0 == "metadata.toml"));
        assert!(entries.iter().any(|entry| entry.0 == ".metadata/cover.png"));

        let unsafe_zip = metadata_bundle(vec![("../metadata.toml", b"bad".as_slice())]);
        assert!(read_metadata_entries(&unsafe_zip).is_err());
    }

    #[test]
    fn inject_metadata_replaces_existing_metadata_only() {
        let temp = tempfile::tempdir().unwrap();
        let zip_path = temp.path().join("mod.zip");
        write_zip_file(
            &zip_path,
            vec![
                ("mod/file.txt", b"keep".as_slice()),
                ("metadata.toml", b"old".as_slice()),
                (".metadata/cover.png", b"old-cover".as_slice()),
            ],
        );

        inject_metadata_entries(
            &zip_path,
            vec![
                ("metadata.toml".to_string(), b"title = \"New\"".to_vec()),
                (".metadata/cover.png".to_string(), b"new-cover".to_vec()),
            ],
        )
        .unwrap();

        assert_eq!(zip_text(&zip_path, "mod/file.txt"), "keep");
        assert_eq!(zip_text(&zip_path, "metadata.toml"), "title = \"New\"");
        assert_eq!(zip_bytes(&zip_path, ".metadata/cover.png"), b"new-cover");
    }

    #[test]
    fn read_metadata_entries_accepts_backslash_metadata_paths() {
        let bytes = metadata_bundle(vec![(".metadata\\cover.png", b"png".as_slice())]);

        let entries = read_metadata_entries(&bytes).unwrap();

        assert_eq!(entries[0].0, ".metadata/cover.png");
        assert_eq!(entries[0].1, b"png");
    }

    #[test]
    fn read_metadata_entries_rejects_absolute_paths() {
        let bytes = metadata_bundle(vec![("/metadata.toml", b"bad".as_slice())]);

        assert!(read_metadata_entries(&bytes).is_err());
    }

    #[test]
    fn inject_metadata_can_add_metadata_to_zip_without_existing_metadata() {
        let temp = tempfile::tempdir().unwrap();
        let zip_path = temp.path().join("mod.zip");
        write_zip_file(&zip_path, vec![("mod/file.txt", b"keep".as_slice())]);

        inject_metadata_entries(
            &zip_path,
            vec![("metadata.toml".to_string(), b"title = \"Added\"".to_vec())],
        )
        .unwrap();

        assert_eq!(zip_text(&zip_path, "mod/file.txt"), "keep");
        assert_eq!(zip_text(&zip_path, "metadata.toml"), "title = \"Added\"");
    }

    #[test]
    fn inject_metadata_rejects_invalid_target_zip() {
        let temp = tempfile::tempdir().unwrap();
        let zip_path = temp.path().join("mod.zip");
        fs::write(&zip_path, b"not a zip").unwrap();

        let err = inject_metadata_entries(
            &zip_path,
            vec![("metadata.toml".to_string(), b"title = \"Nope\"".to_vec())],
        )
        .unwrap_err();

        assert!(err.contains("Could not read selected ZIP"));
    }

    fn metadata_bundle(entries: Vec<(&str, &[u8])>) -> Vec<u8> {
        let mut writer = ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        for (name, content) in entries {
            writer.start_file(name, options).unwrap();
            writer.write_all(content).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    fn write_zip_file(path: &Path, entries: Vec<(&str, &[u8])>) {
        let file = fs::File::create(path).unwrap();
        let mut writer = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        for (name, content) in entries {
            writer.start_file(name, options).unwrap();
            writer.write_all(content).unwrap();
        }
        writer.finish().unwrap();
    }

    fn zip_text(path: &Path, name: &str) -> String {
        String::from_utf8(zip_bytes(path, name)).unwrap()
    }

    fn zip_bytes(path: &Path, name: &str) -> Vec<u8> {
        let file = fs::File::open(path).unwrap();
        let mut archive = ZipArchive::new(file).unwrap();
        let mut entry = archive.by_name(name).unwrap();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        bytes
    }
}
