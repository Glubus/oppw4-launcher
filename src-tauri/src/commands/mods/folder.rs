use crate::{RemoveModRequest, RevealModRequest, ToggleModRequest};

#[tauri::command]
pub(crate) fn set_mod_enabled(input: ToggleModRequest) -> Result<(), String> {
    crate::set_mod_enabled(input)
}

#[tauri::command]
pub(crate) fn reveal_mod_in_folder(input: RevealModRequest) -> Result<(), String> {
    crate::reveal_mod_in_folder(input)
}

#[tauri::command]
pub(crate) fn remove_installed_mod(input: RemoveModRequest) -> Result<(), String> {
    crate::remove_installed_mod(input)
}
