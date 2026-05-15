use crate::{config::load_config as read_config, ApplyProfileRequest};

#[tauri::command]
pub(crate) fn apply_mod_profile(input: ApplyProfileRequest) -> Result<(), String> {
    let config = read_config()?;
    let profile = config
        .mod_profiles
        .iter()
        .find(|profile| profile.id == input.profile_id)
        .ok_or_else(|| "Mod profile does not exist.".to_string())?;
    for mod_info in crate::installed_mods(&config) {
        super::folder::set_mod_path_enabled(
            &config,
            &mod_info.path,
            profile.enabled_mod_keys.contains(&mod_info.mod_key),
        )?;
    }
    Ok(())
}
