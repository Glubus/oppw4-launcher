use super::{logs, status, HealthCheckItem};
use crate::{
    commands::{
        launcher::types::ExportDiagnosticsRequest,
        mods::{inventory, types::InstalledMod},
    },
    config::load_config as read_config,
    config::LauncherConfig,
};
use std::{
    fs,
    io::{Seek, Write},
    path::PathBuf,
};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

#[tauri::command]
pub(crate) fn export_diagnostics(input: ExportDiagnosticsRequest) -> Result<(), String> {
    let config = read_config()?;
    let mods = inventory::installed_mods(&config);
    let health = status::build_health_check(&config);
    export_diagnostics_zip(PathBuf::from(input.path), &config, &mods, &health)
}

fn export_diagnostics_zip(
    path: PathBuf,
    config: &LauncherConfig,
    mods: &[InstalledMod],
    health: &[HealthCheckItem],
) -> Result<(), String> {
    let file = fs::File::create(&path)
        .map_err(|err| format!("Could not create diagnostics ZIP: {err}"))?;
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    write_diagnostics_json_files(&mut writer, options, config, mods, health)?;
    write_optional_log_file(
        &mut writer,
        options,
        "latest-loader-log.txt",
        logs::latest_loader_log(config),
    )?;
    write_optional_log_file(&mut writer, options, "crash.log", logs::crash_log(config))?;
    writer
        .finish()
        .map_err(|err| format!("Could not finish diagnostics ZIP: {err}"))?;
    Ok(())
}

fn write_diagnostics_json_files<W: Write + Seek>(
    writer: &mut ZipWriter<W>,
    options: SimpleFileOptions,
    config: &LauncherConfig,
    mods: &[InstalledMod],
    health: &[HealthCheckItem],
) -> Result<(), String> {
    write_zip_text(
        writer,
        options,
        "summary.txt",
        &diagnostics_summary(config, mods, health),
    )?;
    write_zip_json(writer, options, "config.json", config)?;
    write_zip_json(writer, options, "installed-mods.json", mods)?;
    write_zip_json(writer, options, "health-check.json", health)
}

fn write_zip_json<W, T>(
    writer: &mut ZipWriter<W>,
    options: SimpleFileOptions,
    name: &str,
    value: &T,
) -> Result<(), String>
where
    W: Write + Seek,
    T: serde::Serialize + ?Sized,
{
    let content = serde_json::to_string_pretty(value)
        .map_err(|err| format!("Could not serialize {name}: {err}"))?;
    write_zip_text(writer, options, name, &content)
}

fn write_optional_log_file<W: Write + Seek>(
    writer: &mut ZipWriter<W>,
    options: SimpleFileOptions,
    name: &str,
    path: Option<PathBuf>,
) -> Result<(), String> {
    let Some(path) = path else {
        return Ok(());
    };
    let Ok(bytes) = fs::read(path) else {
        return Ok(());
    };
    writer
        .start_file(name, options)
        .map_err(|err| format!("Could not write diagnostics ZIP: {err}"))?;
    writer
        .write_all(&bytes)
        .map_err(|err| format!("Could not write diagnostics ZIP: {err}"))
}

fn diagnostics_summary(
    config: &LauncherConfig,
    mods: &[InstalledMod],
    health: &[HealthCheckItem],
) -> String {
    let error_count = health.iter().filter(|item| item.level == "error").count();
    let warning_count = health.iter().filter(|item| item.level == "warn").count();
    format!(
    "OPPW4 Launcher diagnostics\nGenerated: {}\nLauncher version: {}\nGame folder: {}\nPatcher release: {}\nInstalled mods: {}\nHealth: {error_count} error(s), {warning_count} warning(s)\n",
    now_label(),
    env!("CARGO_PKG_VERSION"),
    config.game_folder.as_deref().unwrap_or("Not selected"),
    config.modloader_release.as_deref().unwrap_or("Not installed"),
    mods.len()
  )
}

fn write_zip_text<W: Write + Seek>(
    writer: &mut ZipWriter<W>,
    options: SimpleFileOptions,
    name: &str,
    content: &str,
) -> Result<(), String> {
    writer
        .start_file(name, options)
        .map_err(|err| format!("Could not write diagnostics ZIP: {err}"))?;
    writer
        .write_all(content.as_bytes())
        .map_err(|err| format!("Could not write diagnostics ZIP: {err}"))
}

fn now_label() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}
