use crate::{config::LauncherConfig, LauncherState};

#[tauri::command]
pub(crate) fn get_launcher_state() -> Result<LauncherState, String> {
    crate::get_launcher_state()
}

#[tauri::command]
pub(crate) fn save_launcher_config(config: LauncherConfig) -> Result<LauncherConfig, String> {
    crate::save_launcher_config(config)
}
