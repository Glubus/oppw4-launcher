use crate::{
    config::{load_config as read_config, save_config as write_config, LaunchMode, STEAM_APP_ID},
    now_label, steam,
};
use std::{path::PathBuf, process::Command};

#[tauri::command]
pub(crate) fn detect_game() -> Result<Option<steam::DetectedGame>, String> {
    Ok(steam::detect_oppw4())
}

#[tauri::command]
pub(crate) fn launch_game() -> Result<(), String> {
    let mut config = read_config()?;
    match config.launch_mode {
        LaunchMode::Steam => open_steam_uri()?,
        LaunchMode::Executable => {
            let executable = config
                .game_executable_path
                .clone()
                .ok_or_else(|| "Set a game executable path first.".to_string())?;
            let executable = PathBuf::from(executable);
            if !executable.exists() {
                return Err("Configured executable does not exist.".to_string());
            }
            Command::new(&executable)
                .current_dir(
                    executable
                        .parent()
                        .unwrap_or_else(|| std::path::Path::new(".")),
                )
                .spawn()
                .map_err(|err| format!("Could not launch executable: {err}"))?;
        }
    }
    config.last_launch_at = Some(now_label());
    write_config(&config)
}

fn open_steam_uri() -> Result<(), String> {
    let uri = format!("steam://run/{STEAM_APP_ID}");

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", &uri])
            .spawn()
            .map_err(|err| format!("Could not open Steam URI: {err}"))?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&uri)
            .spawn()
            .map_err(|err| format!("Could not open Steam URI with xdg-open: {err}"))?;
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        return Err("Steam launch is only implemented for Windows and Linux.".to_string());
    }

    Ok(())
}
