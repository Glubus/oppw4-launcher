use crate::{
    config::load_config as read_config,
    diagnostics::{export_diagnostics_zip, HealthCheckItem},
    ExportDiagnosticsRequest,
};
use std::path::PathBuf;

#[tauri::command]
pub(crate) fn run_health_check() -> Result<Vec<HealthCheckItem>, String> {
    let config = read_config()?;
    Ok(crate::build_health_check(&config))
}

#[tauri::command]
pub(crate) fn export_diagnostics(input: ExportDiagnosticsRequest) -> Result<(), String> {
    let config = read_config()?;
    let mods = crate::installed_mods(&config);
    let health = crate::build_health_check(&config);
    export_diagnostics_zip(PathBuf::from(input.path), &config, &mods, &health)
}
