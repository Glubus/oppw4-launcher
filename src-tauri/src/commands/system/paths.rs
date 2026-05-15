use crate::RevealPathRequest;

#[tauri::command]
pub(crate) fn reveal_path_in_folder(input: RevealPathRequest) -> Result<(), String> {
    crate::reveal_path_in_folder(input)
}
