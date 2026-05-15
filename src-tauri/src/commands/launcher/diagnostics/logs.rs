use crate::config::LauncherConfig;
use std::{fs, path::PathBuf};

pub(crate) fn latest_loader_log(config: &LauncherConfig) -> Option<PathBuf> {
    let logs_dir = PathBuf::from(config.game_folder.as_ref()?)
        .join("mods")
        .join("_oppw4")
        .join("logs");
    latest_file_in_dir(logs_dir)
}

pub(crate) fn crash_log(config: &LauncherConfig) -> Option<PathBuf> {
    let path = PathBuf::from(config.game_folder.as_ref()?)
        .join("logs")
        .join("crash.log");
    path.is_file().then_some(path)
}

fn latest_file_in_dir(path: PathBuf) -> Option<PathBuf> {
    fs::read_dir(path)
        .ok()?
        .flatten()
        .filter_map(file_with_modified_time)
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
}

fn file_with_modified_time(entry: fs::DirEntry) -> Option<(std::time::SystemTime, PathBuf)> {
    let path = entry.path();
    path.is_file()
        .then(|| {
            entry
                .metadata()
                .ok()?
                .modified()
                .ok()
                .map(|modified| (modified, path))
        })
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LauncherConfig;
    use std::{thread, time::Duration};

    #[test]
    fn crash_log_returns_existing_game_crash_log_only() {
        let temp = tempfile::tempdir().unwrap();
        let config = LauncherConfig {
            game_folder: Some(temp.path().to_string_lossy().to_string()),
            ..LauncherConfig::default()
        };
        assert!(crash_log(&config).is_none());

        let crash_path = temp.path().join("logs").join("crash.log");
        fs::create_dir_all(crash_path.parent().unwrap()).unwrap();
        fs::write(&crash_path, b"crash").unwrap();

        assert_eq!(crash_log(&config).as_deref(), Some(crash_path.as_path()));
    }

    #[test]
    fn latest_loader_log_returns_newest_file_and_ignores_directories() {
        let temp = tempfile::tempdir().unwrap();
        let logs_dir = temp.path().join("mods").join("_oppw4").join("logs");
        fs::create_dir_all(logs_dir.join("nested")).unwrap();
        let old_log = logs_dir.join("old.log");
        let new_log = logs_dir.join("new.log");
        fs::write(&old_log, b"old").unwrap();
        thread::sleep(Duration::from_millis(5));
        fs::write(&new_log, b"new").unwrap();
        let config = LauncherConfig {
            game_folder: Some(temp.path().to_string_lossy().to_string()),
            ..LauncherConfig::default()
        };

        assert_eq!(latest_loader_log(&config).as_deref(), Some(new_log.as_path()));
    }

    #[test]
    fn latest_loader_log_returns_none_without_game_folder_or_logs_dir() {
        assert!(latest_loader_log(&LauncherConfig::default()).is_none());

        let temp = tempfile::tempdir().unwrap();
        let config = LauncherConfig {
            game_folder: Some(temp.path().to_string_lossy().to_string()),
            ..LauncherConfig::default()
        };

        assert!(latest_loader_log(&config).is_none());
    }
}
