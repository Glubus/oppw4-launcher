use crate::config::{backup_dir, InstalledFile};
use crate::error::{InstallerError, InstallerResult};
use crate::installer::{paths::safe_zip_path, time::timestamp};
use std::{
    fs,
    io::{Cursor, Read},
    path::Path,
};

pub(crate) fn install_dll(bytes: &[u8], game_folder: &Path) -> InstallerResult<Vec<InstalledFile>> {
    let target = game_folder.join("dinput8.dll");
    let backup_root = backup_dir()
        .map_err(InstallerError::Config)?
        .join(timestamp());
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
    let reader = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader).map_err(|source| InstallerError::Zip {
        context: "Could not read modloader zip",
        source,
    })?;
    let mut installed = Vec::new();
    let backup_root = backup_dir()
        .map_err(InstallerError::Config)?
        .join(timestamp());
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
