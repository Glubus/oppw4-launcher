use std::path::PathBuf;

use super::types::RevealPathRequest;
use crate::system_utils;

#[tauri::command]
pub(crate) fn reveal_path_in_folder(input: RevealPathRequest) -> Result<(), String> {
    let path = PathBuf::from(input.path);
    if !path.exists() {
        return Err("Selected path does not exist.".to_string());
    }
    system_utils::reveal_path(&path)
}
