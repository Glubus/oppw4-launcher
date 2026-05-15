use crate::config::{backup_dir, InstalledFile};
use crate::error::{InstallerError, InstallerResult};
use crate::installer::{paths::safe_zip_path, time::timestamp};
use std::{
    fs,
    io::{Cursor, Read},
    path::Path,
};

pub(crate) fn install_dll(bytes: &[u8], game_folder: &Path) -> InstallerResult<Vec<InstalledFile>> {
    install_dll_with_backup_root(bytes, game_folder, backup_root()?)
}

fn install_dll_with_backup_root(
    bytes: &[u8],
    game_folder: &Path,
    backup_root: std::path::PathBuf,
) -> InstallerResult<Vec<InstalledFile>> {
    let target = game_folder.join("dinput8.dll");
    fs::create_dir_all(&backup_root)
        .map_err(|source| InstallerError::io("Could not create backup directory", source))?;

    let backup_path = if target.exists() {
        let backup_path = backup_root.join("dinput8.dll");
        fs::copy(&target, &backup_path)
            .map_err(|source| InstallerError::path_io("Could not backup", &target, source))?;
        Some(backup_path.to_string_lossy().to_string())
    } else {
        None
    };

    fs::write(&target, bytes)
        .map_err(|source| InstallerError::path_io("Could not write", &target, source))?;
    Ok(vec![InstalledFile {
        relative_path: "dinput8.dll".to_string(),
        backup_path,
    }])
}

pub(crate) fn install_zip(bytes: &[u8], game_folder: &Path) -> InstallerResult<Vec<InstalledFile>> {
    install_zip_with_backup_root(bytes, game_folder, backup_root()?)
}

fn install_zip_with_backup_root(
    bytes: &[u8],
    game_folder: &Path,
    backup_root: std::path::PathBuf,
) -> InstallerResult<Vec<InstalledFile>> {
    let reader = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader).map_err(|source| InstallerError::Zip {
        context: "Could not read modloader zip",
        source,
    })?;
    let mut installed = Vec::new();
    fs::create_dir_all(&backup_root)
        .map_err(|source| InstallerError::io("Could not create backup directory", source))?;

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
        let relative = safe_zip_path(entry.name())?;
        let target = game_folder.join(&relative);
        let backup_path = if target.exists() {
            let backup_path = backup_root.join(&relative);
            if let Some(parent) = backup_path.parent() {
                fs::create_dir_all(parent).map_err(|source| {
                    InstallerError::io("Could not create backup parent", source)
                })?;
            }
            fs::copy(&target, &backup_path)
                .map_err(|source| InstallerError::path_io("Could not backup", &target, source))?;
            Some(backup_path.to_string_lossy().to_string())
        } else {
            None
        };

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .map_err(|source| InstallerError::path_io("Could not create", parent, source))?;
        }
        let mut content = Vec::new();
        let entry_name = entry.name().to_string();
        entry
            .read_to_end(&mut content)
            .map_err(|source| InstallerError::io(format!("Could not read {entry_name}"), source))?;
        fs::write(&target, content)
            .map_err(|source| InstallerError::path_io("Could not write", &target, source))?;

        installed.push(InstalledFile {
            relative_path: relative.to_string_lossy().to_string(),
            backup_path,
        });
    }

    if installed.is_empty() {
        return Err(InstallerError::ZipDidNotContainInstallableFiles);
    }

    Ok(installed)
}

fn backup_root() -> InstallerResult<std::path::PathBuf> {
    Ok(backup_dir()
        .map_err(InstallerError::Config)?
        .join(timestamp()))
}

pub fn restore(config: &mut crate::config::LauncherConfig) -> InstallerResult<()> {
    let game_folder = config
        .game_folder
        .clone()
        .ok_or(InstallerError::MissingGameFolder {
            action: "restoring",
        })?;
    let game_folder = std::path::PathBuf::from(game_folder);

    for installed in config.installed_files.iter().rev() {
        let target = game_folder.join(&installed.relative_path);
        if target.exists() {
            fs::remove_file(&target)
                .map_err(|source| InstallerError::path_io("Could not remove", &target, source))?;
        }
        if let Some(backup) = &installed.backup_path {
            let backup = std::path::PathBuf::from(backup);
            if backup.exists() {
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent).map_err(|source| {
                        InstallerError::path_io("Could not create", parent, source)
                    })?;
                }
                fs::copy(&backup, &target).map_err(|source| {
                    InstallerError::path_io("Could not restore", &target, source)
                })?;
            }
        }
    }

    config.installed_files.clear();
    config.modloader_release = None;
    config.modloader_sha256 = None;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

    fn zip_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = ZipWriter::new(&mut cursor);
            let options =
                SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
            for (name, bytes) in entries {
                writer.start_file(*name, options).unwrap();
                writer.write_all(bytes).unwrap();
            }
            writer.finish().unwrap();
        }
        cursor.into_inner()
    }

    #[test]
    fn install_zip_writes_files_and_backs_up_existing_targets() {
        let temp = tempfile::tempdir().unwrap();
        let game_folder = temp.path().join("game");
        let backup_root = temp.path().join("backup");
        fs::create_dir_all(game_folder.join("loader")).unwrap();
        fs::write(game_folder.join("dinput8.dll"), b"old dll").unwrap();
        fs::write(game_folder.join("loader/config.toml"), b"old config").unwrap();
        let bytes = zip_bytes(&[
            ("dinput8.dll", b"new dll"),
            ("loader/config.toml", b"new config"),
        ]);

        let installed = install_zip_with_backup_root(&bytes, &game_folder, backup_root).unwrap();

        assert_eq!(installed.len(), 2);
        assert_eq!(fs::read(game_folder.join("dinput8.dll")).unwrap(), b"new dll");
        assert_eq!(
            fs::read(game_folder.join("loader/config.toml")).unwrap(),
            b"new config"
        );
        assert_eq!(
            fs::read(installed[0].backup_path.as_ref().unwrap()).unwrap(),
            b"old dll"
        );
        assert_eq!(
            fs::read(installed[1].backup_path.as_ref().unwrap()).unwrap(),
            b"old config"
        );
    }

    #[test]
    fn install_zip_rejects_parent_paths() {
        let temp = tempfile::tempdir().unwrap();
        let game_folder = temp.path().join("game");
        fs::create_dir_all(&game_folder).unwrap();
        let bytes = zip_bytes(&[("../outside.txt", b"nope")]);

        let err = install_zip_with_backup_root(&bytes, &game_folder, temp.path().join("backup"))
            .unwrap_err();

        assert!(matches!(err, InstallerError::UnsafeZipPath { .. }));
        assert!(!temp.path().join("outside.txt").exists());
    }

    #[test]
    fn install_dll_backs_up_existing_dinput8() {
        let temp = tempfile::tempdir().unwrap();
        let game_folder = temp.path().join("game");
        fs::create_dir_all(&game_folder).unwrap();
        fs::write(game_folder.join("dinput8.dll"), b"old").unwrap();

        let installed =
            install_dll_with_backup_root(b"new", &game_folder, temp.path().join("backup")).unwrap();

        assert_eq!(fs::read(game_folder.join("dinput8.dll")).unwrap(), b"new");
        assert_eq!(
            fs::read(installed[0].backup_path.as_ref().unwrap()).unwrap(),
            b"old"
        );
    }

    #[test]
    fn restore_removes_installed_files_and_restores_backups_in_reverse_order() {
        let temp = tempfile::tempdir().unwrap();
        let game_folder = temp.path().join("game");
        let backup_dir = temp.path().join("backup");
        fs::create_dir_all(game_folder.join("loader")).unwrap();
        fs::create_dir_all(backup_dir.join("loader")).unwrap();
        fs::write(game_folder.join("dinput8.dll"), b"installed dll").unwrap();
        fs::write(game_folder.join("loader/config.toml"), b"installed config").unwrap();
        fs::write(backup_dir.join("dinput8.dll"), b"old dll").unwrap();
        fs::write(backup_dir.join("loader/config.toml"), b"old config").unwrap();
        let mut config = crate::config::LauncherConfig {
            game_folder: Some(game_folder.to_string_lossy().to_string()),
            modloader_release: Some("v1".to_string()),
            modloader_sha256: Some("hash".to_string()),
            installed_files: vec![
                InstalledFile {
                    relative_path: "dinput8.dll".to_string(),
                    backup_path: Some(backup_dir.join("dinput8.dll").to_string_lossy().to_string()),
                },
                InstalledFile {
                    relative_path: "loader/config.toml".to_string(),
                    backup_path: Some(
                        backup_dir
                            .join("loader/config.toml")
                            .to_string_lossy()
                            .to_string(),
                    ),
                },
            ],
            ..crate::config::LauncherConfig::default()
        };

        restore(&mut config).unwrap();

        assert_eq!(fs::read(game_folder.join("dinput8.dll")).unwrap(), b"old dll");
        assert_eq!(
            fs::read(game_folder.join("loader/config.toml")).unwrap(),
            b"old config"
        );
        assert!(config.installed_files.is_empty());
        assert!(config.modloader_release.is_none());
        assert!(config.modloader_sha256.is_none());
    }

    #[test]
    fn restore_requires_game_folder() {
        let mut config = crate::config::LauncherConfig::default();

        assert!(matches!(
            restore(&mut config),
            Err(InstallerError::MissingGameFolder {
                action: "restoring"
            })
        ));
    }
}
