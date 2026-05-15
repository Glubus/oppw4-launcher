use super::types::ExportDiagnosticsRequest;
use crate::{
    config::load_config as read_config,
    diagnostics::{export_diagnostics_zip, HealthCheckItem},
    launcher_status, mod_inventory,
};
use std::path::PathBuf;

#[tauri::command]
pub(crate) fn run_health_check() -> Result<Vec<HealthCheckItem>, String> {
    let config = read_config()?;
    Ok(launcher_status::build_health_check(&config))
}

#[tauri::command]
pub(crate) fn export_diagnostics(input: ExportDiagnosticsRequest) -> Result<(), String> {
    let config = read_config()?;
    let mods = mod_inventory::installed_mods(&config);
    let health = launcher_status::build_health_check(&config);
    export_diagnostics_zip(PathBuf::from(input.path), &config, &mods, &health)
}
