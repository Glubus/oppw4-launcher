use std::process::Command;

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn open_external_url(url: String) -> Result<(), String> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("Only web URLs can be opened externally.".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", &url])
            .spawn()
            .map_err(|err| format!("Could not open URL in browser: {err}"))?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|err| format!("Could not open URL in browser: {err}"))?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|err| format!("Could not open URL in browser: {err}"))?;
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        return Err(
            "Opening URLs externally is only implemented for Windows, macOS, and Linux."
                .to_string(),
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_external_url_rejects_non_http_urls_before_spawning_browser() {
        assert_eq!(
            open_external_url("file:///tmp/mod.zip".to_string()).unwrap_err(),
            "Only web URLs can be opened externally."
        );
        assert_eq!(
            open_external_url("steam://run/1089090".to_string()).unwrap_err(),
            "Only web URLs can be opened externally."
        );
    }
}
