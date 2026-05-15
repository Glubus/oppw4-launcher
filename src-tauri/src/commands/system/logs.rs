use crate::LauncherLogRequest;

#[tauri::command]
pub(crate) fn write_launcher_log(input: LauncherLogRequest) -> Result<(), String> {
    crate::write_launcher_log(input)
}
