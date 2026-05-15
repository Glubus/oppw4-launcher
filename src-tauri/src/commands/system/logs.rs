use crate::{now_label, LauncherLogRequest};
use std::{fs, io::Write};

#[tauri::command]
pub(crate) fn write_launcher_log(input: LauncherLogRequest) -> Result<(), String> {
    let safe_stamp = input
        .file_stamp
        .chars()
        .filter(|value| value.is_ascii_alphanumeric() || matches!(value, '-' | '_'))
        .collect::<String>();
    let safe_stamp = if safe_stamp.is_empty() {
        now_label()
    } else {
        safe_stamp
    };
    let exe = std::env::current_exe()
        .map_err(|err| format!("Could not resolve launcher executable: {err}"))?;
    let log_dir = exe
        .parent()
        .ok_or_else(|| "Could not resolve launcher directory.".to_string())?
        .join("logs");
    fs::create_dir_all(&log_dir).map_err(|err| format!("Could not create logs folder: {err}"))?;

    let level = input.level.to_uppercase();
    let prefix = if input.debug { "DEBUG" } else { level.as_str() };
    let line = format!(
        "[{}] {} {}\n",
        now_label(),
        prefix,
        input.message.replace('\n', " ")
    );
    let path = log_dir.join(format!("{safe_stamp}.log"));
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|err| format!("Could not open launcher log: {err}"))?;
    file.write_all(line.as_bytes())
        .map_err(|err| format!("Could not write launcher log: {err}"))
}
