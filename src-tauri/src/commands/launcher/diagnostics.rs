use crate::{diagnostics::HealthCheckItem, ExportDiagnosticsRequest};

#[tauri::command]
pub(crate) fn run_health_check() -> Result<Vec<HealthCheckItem>, String> {
    crate::run_health_check()
}

#[tauri::command]
pub(crate) fn export_diagnostics(input: ExportDiagnosticsRequest) -> Result<(), String> {
    crate::export_diagnostics(input)
}
