use crate::{config::LauncherConfig, error::CommandResult};

#[tauri::command]
pub(crate) fn install_modloader() -> CommandResult<LauncherConfig> {
    crate::install_modloader()
}

#[tauri::command]
pub(crate) fn restore_modloader() -> CommandResult<LauncherConfig> {
    crate::restore_modloader()
}

#[tauri::command]
pub(crate) fn check_modloader_integrity() -> CommandResult<LauncherConfig> {
    crate::check_modloader_integrity()
}
