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
