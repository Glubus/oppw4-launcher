use crate::ApplyProfileRequest;

#[tauri::command]
pub(crate) fn apply_mod_profile(input: ApplyProfileRequest) -> Result<(), String> {
    crate::apply_mod_profile(input)
}
