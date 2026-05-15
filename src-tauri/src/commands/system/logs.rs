use super::types::LauncherLogRequest;
use crate::system_utils;
use std::{fs, io::Write};

#[tauri::command]
pub(crate) fn write_launcher_log(input: LauncherLogRequest) -> Result<(), String> {
    let safe_stamp = sanitize_log_stamp(&input.file_stamp);
    let safe_stamp = if safe_stamp.is_empty() {
        system_utils::now_label()
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

    let line = format_log_line(&input, &system_utils::now_label());
    let path = log_dir.join(format!("{safe_stamp}.log"));
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|err| format!("Could not open launcher log: {err}"))?;
    file.write_all(line.as_bytes())
        .map_err(|err| format!("Could not write launcher log: {err}"))
}

fn sanitize_log_stamp(file_stamp: &str) -> String {
    file_stamp
        .chars()
        .filter(|value| value.is_ascii_alphanumeric() || matches!(value, '-' | '_'))
        .collect()
}

fn format_log_line(input: &LauncherLogRequest, timestamp: &str) -> String {
    let level = input.level.to_uppercase();
    let prefix = if input.debug { "DEBUG" } else { level.as_str() };
    format!(
        "[{timestamp}] {prefix} {}\n",
        input.message.replace('\n', " ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_log_stamp_removes_path_and_shell_characters() {
        assert_eq!(sanitize_log_stamp("../2026:05:15 log!"), "20260515log");
        assert_eq!(sanitize_log_stamp("2026-05-15_120000"), "2026-05-15_120000");
    }

    #[test]
    fn format_log_line_flattens_multiline_messages_and_uses_debug_prefix() {
        let input = LauncherLogRequest {
            level: "error".to_string(),
            message: "first line\nsecond line".to_string(),
            file_stamp: "stamp".to_string(),
            debug: true,
        };

        let line = format_log_line(&input, "123");

        assert_eq!(line, "[123] DEBUG first line second line\n");
    }
}
