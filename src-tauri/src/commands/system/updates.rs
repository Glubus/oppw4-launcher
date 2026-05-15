use crate::{error::CommandResult, updater};

#[tauri::command]
pub(crate) fn check_launcher_update() -> CommandResult<updater::UpdateInfo> {
    updater::check().map_err(|err| err.to_string())
}

#[tauri::command]
pub(crate) fn install_launcher_update() -> CommandResult<updater::UpdateInstallResult> {
    updater::install_latest().map_err(|err| err.to_string())
}
