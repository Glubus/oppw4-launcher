#[tauri::command]
pub(crate) fn detect_game() -> Result<Option<crate::steam::DetectedGame>, String> {
    crate::detect_game()
}

#[tauri::command]
pub(crate) fn launch_game() -> Result<(), String> {
    crate::launch_game()
}
