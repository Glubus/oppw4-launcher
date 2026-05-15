use super::status;
use crate::{
    commands::launcher::types::ExportDiagnosticsRequest, config::load_config as read_config,
    diagnostics::export_diagnostics_zip, mod_inventory,
};
use std::path::PathBuf;

#[tauri::command]
pub(crate) fn export_diagnostics(input: ExportDiagnosticsRequest) -> Result<(), String> {
    let config = read_config()?;
    let mods = mod_inventory::installed_mods(&config);
    let health = status::build_health_check(&config);
    export_diagnostics_zip(PathBuf::from(input.path), &config, &mods, &health)
}
