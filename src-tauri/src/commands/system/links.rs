#[tauri::command]
pub(crate) fn open_external_url(url: String) -> Result<(), String> {
    crate::open_external_url(url)
}
