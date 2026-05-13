use crate::{config::LauncherConfig, InstalledMod};
use serde::Serialize;
use std::{
  fs,
  io::{Seek, Write},
  path::PathBuf,
};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckItem {
  pub level: String,
  pub title: String,
  pub detail: String,
}

pub fn health_item(level: &str, title: &str, detail: &str) -> HealthCheckItem {
  HealthCheckItem {
    level: level.to_string(),
    title: title.to_string(),
    detail: detail.to_string(),
  }
}

pub fn latest_loader_log(config: &LauncherConfig) -> Option<PathBuf> {
  let logs_dir = PathBuf::from(config.game_folder.as_ref()?).join("mods").join("_oppw4").join("logs");
  let entries = fs::read_dir(logs_dir).ok()?;
  entries
    .flatten()
    .filter_map(|entry| {
      let path = entry.path();
      if !path.is_file() {
        return None;
      }
      let modified = entry.metadata().ok()?.modified().ok()?;
      Some((modified, path))
    })
    .max_by_key(|(modified, _)| *modified)
    .map(|(_, path)| path)
}

pub fn export_diagnostics_zip(path: PathBuf, config: &LauncherConfig, mods: &[InstalledMod], health: &[HealthCheckItem]) -> Result<(), String> {
  let file = fs::File::create(&path).map_err(|err| format!("Could not create diagnostics ZIP: {err}"))?;
  let mut writer = ZipWriter::new(file);
  let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

  write_zip_text(&mut writer, options, "summary.txt", &diagnostics_summary(config, mods, health))?;
  write_zip_text(&mut writer, options, "config.json", &serde_json::to_string_pretty(config).map_err(|err| format!("Could not serialize config: {err}"))?)?;
  write_zip_text(&mut writer, options, "installed-mods.json", &serde_json::to_string_pretty(mods).map_err(|err| format!("Could not serialize mods: {err}"))?)?;
  write_zip_text(&mut writer, options, "health-check.json", &serde_json::to_string_pretty(health).map_err(|err| format!("Could not serialize health check: {err}"))?)?;

  if let Some(log_path) = latest_loader_log(config) {
    if let Ok(bytes) = fs::read(&log_path) {
      writer
        .start_file("latest-loader-log.txt", options)
        .map_err(|err| format!("Could not write diagnostics ZIP: {err}"))?;
      writer
        .write_all(&bytes)
        .map_err(|err| format!("Could not write diagnostics ZIP: {err}"))?;
    }
  }

  writer.finish().map_err(|err| format!("Could not finish diagnostics ZIP: {err}"))?;
  Ok(())
}

fn diagnostics_summary(config: &LauncherConfig, mods: &[InstalledMod], health: &[HealthCheckItem]) -> String {
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

fn write_zip_text<W: Write + Seek>(writer: &mut ZipWriter<W>, options: SimpleFileOptions, name: &str, content: &str) -> Result<(), String> {
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
