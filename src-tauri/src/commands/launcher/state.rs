use super::{diagnostics::status, game::steam, types::LauncherState};
use crate::{
    commands::mods::inventory,
    config::{load_config as read_config, save_config as write_config, LauncherConfig},
    installer,
};

#[tauri::command]
pub(crate) fn get_launcher_state() -> Result<LauncherState, String> {
    let mut config = read_config()?;
    let detected_game = steam::detect_oppw4();
    if config.game_folder.is_none() {
        if let Some(game) = &detected_game {
            config.game_folder = Some(game.game_folder.clone());
            config.game_executable_path = game.executable_path.clone();
            write_config(&config)?;
        }
    }
    let local_modloader_sha256 = installer::installed_dinput8_sha256(&config).ok().flatten();
    let _ = installer::refresh_latest_modloader_hash(&mut config, false);
    write_config(&config)?;
    let remote_modloader_sha256 = config.latest_modloader_sha256.clone();
    let modloader_status = status::modloader_status(&config, local_modloader_sha256.as_deref());
    let installed_mods = inventory::installed_mods(&config);
    let latest_release = installer::latest_release_info(&config.modloader_repo)
        .ok()
        .flatten();
    let needs_patcher_update = latest_release.as_ref().is_some_and(|release| {
        config.modloader_release.as_deref() != Some(release.tag_name.as_str())
    }) || remote_modloader_sha256
        .as_ref()
        .is_some_and(|hash| config.modloader_sha256.as_ref() != Some(hash));
    Ok(LauncherState {
        config,
        detected_game,
        modloader_status,
        latest_release,
        needs_patcher_update,
        local_modloader_sha256,
        remote_modloader_sha256,
        installed_mods,
    })
}

#[tauri::command]
pub(crate) fn save_launcher_config(config: LauncherConfig) -> Result<LauncherConfig, String> {
    write_config(&config)?;
    Ok(config)
}
