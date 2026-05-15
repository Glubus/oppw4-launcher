use crate::config::LauncherConfig;
use crate::error::{InstallerError, InstallerResult};
use crate::installer::paths::safe_zip_path;
use sha2::{Digest, Sha256};
use std::{
    fmt::Write as _,
    fs,
    io::{Cursor, Read},
    path::PathBuf,
};

pub fn installed_dinput8_sha256(config: &LauncherConfig) -> InstallerResult<Option<String>> {
    let Some(game_folder) = &config.game_folder else {
        return Ok(None);
    };
    let path = PathBuf::from(game_folder).join("dinput8.dll");
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path)
        .map_err(|source| InstallerError::path_io("Could not read", path, source))?;
    Ok(Some(sha256_hex(&bytes)))
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut hex, "{byte:02x}").expect("writing to a String cannot fail");
    }
    hex
}

pub(crate) fn zip_dinput8_sha256(bytes: &[u8]) -> InstallerResult<String> {
    let reader = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader).map_err(|source| InstallerError::Zip {
        context: "Could not read modloader zip",
        source,
    })?;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|source| InstallerError::Zip {
                context: "Could not read zip entry",
                source,
            })?;
        if entry.is_dir() {
            continue;
        }
        let path = safe_zip_path(entry.name())?;
        if path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case("dinput8.dll"))
        {
            let mut content = Vec::new();
            entry
                .read_to_end(&mut content)
                .map_err(|source| InstallerError::io("Could not read dinput8.dll", source))?;
            return Ok(sha256_hex(&content));
        }
    }
    Err(InstallerError::ZipMissingDinput8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_installed_dinput8_dll() {
        let temp = tempfile::tempdir().unwrap();
        let dll_path = temp.path().join("dinput8.dll");
        fs::write(&dll_path, b"patcher").unwrap();
        let config = LauncherConfig {
            game_folder: Some(temp.path().to_string_lossy().to_string()),
            ..LauncherConfig::default()
        };

        assert_eq!(
            installed_dinput8_sha256(&config).unwrap().as_deref(),
            Some("242d2f23a194483a0aea19c60f86ca2fb887d97edfd2cdfdcf4e2d650a2f79f3")
        );
    }

    #[test]
    fn installed_dinput8_hash_is_absent_when_dll_is_missing() {
        let temp = tempfile::tempdir().unwrap();
        let config = LauncherConfig {
            game_folder: Some(temp.path().to_string_lossy().to_string()),
            ..LauncherConfig::default()
        };

        assert_eq!(installed_dinput8_sha256(&config).unwrap(), None);
    }

    #[test]
    fn installed_dinput8_hash_is_absent_without_game_folder() {
        assert_eq!(
            installed_dinput8_sha256(&LauncherConfig::default()).unwrap(),
            None
        );
    }

    #[test]
    fn zip_dinput8_hash_finds_nested_case_insensitive_dll() {
        let bytes = zip_bytes(&[("bin/DINPUT8.DLL", b"patcher".as_slice())]);

        assert_eq!(
            zip_dinput8_sha256(&bytes).unwrap(),
            "242d2f23a194483a0aea19c60f86ca2fb887d97edfd2cdfdcf4e2d650a2f79f3"
        );
    }

    #[test]
    fn zip_dinput8_hash_rejects_zip_without_dll() {
        let bytes = zip_bytes(&[("readme.txt", b"readme".as_slice())]);

        assert!(matches!(
            zip_dinput8_sha256(&bytes),
            Err(InstallerError::ZipMissingDinput8)
        ));
    }

    #[test]
    fn zip_dinput8_hash_rejects_unsafe_entry_before_hashing() {
        let bytes = zip_bytes(&[("../dinput8.dll", b"patcher".as_slice())]);

        assert!(matches!(
            zip_dinput8_sha256(&bytes),
            Err(InstallerError::UnsafeZipPath { .. })
        ));
    }

    fn zip_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
        use std::io::Write;
        use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

        let mut writer = ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        for (name, content) in entries {
            writer.start_file(*name, options).unwrap();
            writer.write_all(content).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }
}
