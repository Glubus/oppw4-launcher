use crate::error::CommandResult;

#[tauri::command]
pub(crate) fn check_launcher_update() -> CommandResult<crate::updater::UpdateInfo> {
    crate::check_launcher_update()
}

#[tauri::command]
pub(crate) fn install_launcher_update() -> CommandResult<crate::updater::UpdateInstallResult> {
    crate::install_launcher_update()
}
