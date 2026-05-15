use std::{path::Path, process::Command};

pub(crate) fn reveal_path(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(format!("/select,{}", path.display()))
            .spawn()
            .map_err(|err| format!("Could not open folder: {err}"))?;
    }

    #[cfg(target_os = "linux")]
    {
        let folder = path
            .parent()
            .ok_or_else(|| "Could not resolve mod folder.".to_string())?;
        Command::new("xdg-open")
            .arg(folder)
            .spawn()
            .map_err(|err| format!("Could not open folder: {err}"))?;
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        return Err("Show in folder is only implemented for Windows and Linux.".to_string());
    }

    Ok(())
}

pub(crate) fn now_label() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}
