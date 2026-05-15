pub(crate) mod exports;
pub(crate) mod logs;
pub(crate) mod status;

use crate::config::load_config as read_config;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HealthCheckItem {
    pub(crate) level: String,
    pub(crate) title: String,
    pub(crate) detail: String,
}

pub(crate) fn health_item(level: &str, title: &str, detail: &str) -> HealthCheckItem {
    HealthCheckItem {
        level: level.to_string(),
        title: title.to_string(),
        detail: detail.to_string(),
    }
}

#[tauri::command]
pub(crate) fn run_health_check() -> Result<Vec<HealthCheckItem>, String> {
    let config = read_config()?;
    Ok(status::build_health_check(&config))
}
