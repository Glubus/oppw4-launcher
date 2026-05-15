use std::path::PathBuf;

use crate::RevealPathRequest;

#[tauri::command]
pub(crate) fn reveal_path_in_folder(input: RevealPathRequest) -> Result<(), String> {
    let path = PathBuf::from(input.path);
    if !path.exists() {
        return Err("Selected path does not exist.".to_string());
    }
    crate::reveal_path(&path)
}
