use crate::{
    config::{load_config as read_config, save_config as write_config, LauncherConfig},
    error::CommandResult,
    installer,
};

#[tauri::command]
pub(crate) fn install_modloader() -> CommandResult<LauncherConfig> {
    let mut config = read_config()?;
    installer::install_from_latest_release(&mut config).map_err(|err| err.to_string())?;
    write_config(&config)?;
    Ok(config)
}

#[tauri::command]
pub(crate) fn restore_modloader() -> CommandResult<LauncherConfig> {
    let mut config = read_config()?;
    installer::restore(&mut config).map_err(|err| err.to_string())?;
    write_config(&config)?;
    Ok(config)
}

#[tauri::command]
pub(crate) fn check_modloader_integrity() -> CommandResult<LauncherConfig> {
    let mut config = read_config()?;
    installer::refresh_latest_modloader_hash(&mut config, true).map_err(|err| err.to_string())?;
    write_config(&config)?;
    Ok(config)
}
