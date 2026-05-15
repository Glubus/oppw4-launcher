pub(crate) mod exports;
pub(crate) mod status;

use crate::{config::load_config as read_config, diagnostics::HealthCheckItem};

#[tauri::command]
pub(crate) fn run_health_check() -> Result<Vec<HealthCheckItem>, String> {
    let config = read_config()?;
    Ok(status::build_health_check(&config))
}
