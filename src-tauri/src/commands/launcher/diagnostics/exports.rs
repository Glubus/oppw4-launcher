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
    path::{Path, PathBuf},
};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

#[tauri::command]
pub(crate) fn export_diagnostics(input: ExportDiagnosticsRequest) -> Result<(), String> {
    let config = read_config()?;
    let mods = inventory::installed_mods(&config);
    let health = status::build_health_check(&config);
    export_diagnostics_zip(&PathBuf::from(input.path), &config, &mods, &health)
}

fn export_diagnostics_zip(
    path: &Path,
    config: &LauncherConfig,
    mods: &[InstalledMod],
    health: &[HealthCheckItem],
) -> Result<(), String> {
    let file =
        fs::File::create(path).map_err(|err| format!("Could not create diagnostics ZIP: {err}"))?;
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
    "OPPW4 Launcher diagnostics\nGenerated: {}\nLauncher version: {}\nGame folder: {}\nModloader release: {}\nInstalled mods: {}\nHealth: {error_count} error(s), {warning_count} warning(s)\n",
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
    SystemTime::now().duration_since(UNIX_EPOCH).map_or_else(
        |_| "0".to_string(),
        |duration| duration.as_secs().to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use zip::ZipArchive;

    #[test]
    fn export_diagnostics_zip_writes_summary_config_mods_and_health() {
        let temp = tempfile::tempdir().unwrap();
        let export_path = temp.path().join("diagnostics.zip");
        let config = LauncherConfig {
            game_folder: Some(temp.path().join("game").to_string_lossy().to_string()),
            modloader_release: Some("v1.2.3".to_string()),
            ..LauncherConfig::default()
        };
        let mods = vec![InstalledMod {
            name: "Law Outfit".to_string(),
            kind: "zip".to_string(),
            path: "/mods/law.zip".to_string(),
            mod_key: "id:law-outfit".to_string(),
            enabled: true,
            mod_id: Some("law-outfit".to_string()),
            version: Some("1.0.0".to_string()),
            source_url: None,
            slug: None,
            content_kind: "mod".to_string(),
            character_name: Some("Law".to_string()),
            character_slug: Some("law".to_string()),
            mod_type: Some("skin".to_string()),
            dependencies: Vec::new(),
            changelog: None,
            cover_data_url: None,
        }];
        let health = vec![
            super::super::health_item("ok", "Game folder", "Using game folder."),
            super::super::health_item("warn", "Loader log", "Missing log."),
        ];

        export_diagnostics_zip(&export_path, &config, &mods, &health).unwrap();

        let file = fs::File::open(export_path).unwrap();
        let mut archive = ZipArchive::new(file).unwrap();
        let mut summary = String::new();
        archive
            .by_name("summary.txt")
            .unwrap()
            .read_to_string(&mut summary)
            .unwrap();
        let mut installed_mods = String::new();
        archive
            .by_name("installed-mods.json")
            .unwrap()
            .read_to_string(&mut installed_mods)
            .unwrap();
        let mut health_json = String::new();
        archive
            .by_name("health-check.json")
            .unwrap()
            .read_to_string(&mut health_json)
            .unwrap();

        assert!(summary.contains("Modloader release: v1.2.3"));
        assert!(summary.contains("Installed mods: 1"));
        assert!(summary.contains("Health: 0 error(s), 1 warning(s)"));
        assert!(installed_mods.contains("Law Outfit"));
        assert!(health_json.contains("Loader log"));
        assert!(archive.by_name("config.json").is_ok());
        assert!(archive.by_name("latest-loader-log.txt").is_err());
    }
}
