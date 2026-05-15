mod api;
mod commands;
mod config;
mod error;
mod installer;
mod system_utils;
mod updater;

const API_BASE: &str = "https://oppw4.prism.am/api";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// Runs the Tauri launcher application.
///
/// # Panics
///
/// Panics when Tauri fails to initialize or run the application context.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::launcher::state::get_launcher_state,
            commands::launcher::state::save_launcher_config,
            commands::launcher::game::detect_game,
            commands::launcher::game::launch_game,
            commands::launcher::patcher::install_modloader,
            commands::launcher::patcher::restore_modloader,
            commands::launcher::patcher::check_modloader_integrity,
            commands::launcher::diagnostics::run_health_check,
            commands::launcher::diagnostics::exports::export_diagnostics,
            commands::mods::folder::set_mod_enabled,
            commands::mods::install::import_external_zip,
            commands::mods::profiles::apply_mod_profile,
            commands::mods::metadata::apply_metadata_to_zip,
            commands::mods::install::install_hosted_mod,
            commands::mods::install::installed_mod_for_skin,
            commands::mods::folder::reveal_mod_in_folder,
            commands::mods::folder::remove_installed_mod,
            commands::system::paths::reveal_path_in_folder,
            commands::system::logs::write_launcher_log,
            commands::system::updates::check_launcher_update,
            commands::system::updates::install_launcher_update,
            commands::system::links::open_external_url,
            api::api_request
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use crate::{
        commands::{
            launcher::diagnostics::status::modloader_status,
            mods::{
                inventory::installed_mods,
                metadata::{reader::read_local_mod_metadata, zip::inject_metadata_entries},
            },
        },
        config::LauncherConfig,
    };
    use std::{
        fs,
        io::{Read, Write},
    };
    use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

    #[test]
    fn inject_metadata_replaces_metadata_entries_only() {
        let temp = tempfile::tempdir().unwrap();
        let zip_path = temp.path().join("mod.zip");
        {
            let file = fs::File::create(&zip_path).unwrap();
            let mut writer = ZipWriter::new(file);
            let options =
                SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
            writer.start_file("mod/file.txt", options).unwrap();
            writer.write_all(b"keep").unwrap();
            writer.start_file("metadata.toml", options).unwrap();
            writer.write_all(b"old").unwrap();
            writer.start_file(".metadata/cover.png", options).unwrap();
            writer.write_all(b"old-cover").unwrap();
            writer.finish().unwrap();
        }

        inject_metadata_entries(&zip_path, vec![
      ("metadata.toml".to_string(), b"title = \"Installed Mod\"\nversion = \"1.2.3\"\nslug = \"installed-mod\"\ncover = \".metadata/cover.png\"\n".to_vec()),
      (".metadata/cover.png".to_string(), b"new-cover".to_vec()),
    ]).unwrap();

        let file = fs::File::open(&zip_path).unwrap();
        let mut archive = ZipArchive::new(file).unwrap();
        let mut kept = String::new();
        archive
            .by_name("mod/file.txt")
            .unwrap()
            .read_to_string(&mut kept)
            .unwrap();
        let mut metadata = String::new();
        archive
            .by_name("metadata.toml")
            .unwrap()
            .read_to_string(&mut metadata)
            .unwrap();

        assert_eq!(kept, "keep");
        assert!(metadata.contains("Installed Mod"));
        assert!(archive.by_name(".metadata/cover.png").is_ok());

        drop(archive);
        let local_metadata = read_local_mod_metadata(&zip_path).unwrap();
        assert_eq!(local_metadata.title.as_deref(), Some("Installed Mod"));
        assert_eq!(local_metadata.version.as_deref(), Some("1.2.3"));
        assert_eq!(local_metadata.slug.as_deref(), Some("installed-mod"));
        assert!(local_metadata
            .cover_data_url
            .as_deref()
            .unwrap_or("")
            .starts_with("data:image/png;base64,"));
    }

    #[test]
    fn modloader_status_detects_unmanaged_dll() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("dinput8.dll"), b"foreign").unwrap();
        let config = LauncherConfig {
            game_folder: Some(temp.path().to_string_lossy().to_string()),
            ..LauncherConfig::default()
        };

        assert_eq!(
            modloader_status(&config, None),
            "Detected unmanaged dinput8.dll"
        );
    }

    #[test]
    fn modloader_status_detects_modified_dll() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("dinput8.dll"), b"modified").unwrap();
        let config = LauncherConfig {
            game_folder: Some(temp.path().to_string_lossy().to_string()),
            modloader_sha256: Some("expected".to_string()),
            installed_files: vec![crate::config::InstalledFile {
                relative_path: "dinput8.dll".to_string(),
                backup_path: None,
            }],
            ..LauncherConfig::default()
        };

        assert_eq!(
            modloader_status(&config, Some("actual")),
            "Modified dinput8.dll"
        );
    }

    #[test]
    fn modloader_status_detects_available_update() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("dinput8.dll"), b"current").unwrap();
        let config = LauncherConfig {
            game_folder: Some(temp.path().to_string_lossy().to_string()),
            modloader_sha256: Some("current-hash".to_string()),
            latest_modloader_sha256: Some("new-hash".to_string()),
            installed_files: vec![crate::config::InstalledFile {
                relative_path: "dinput8.dll".to_string(),
                backup_path: None,
            }],
            ..LauncherConfig::default()
        };

        assert_eq!(
            modloader_status(&config, Some("current-hash")),
            "Update available"
        );
    }

    #[test]
    fn installed_mods_reads_zip_metadata_and_cover() {
        let temp = tempfile::tempdir().unwrap();
        let mods_dir = temp.path().join("mods");
        fs::create_dir_all(&mods_dir).unwrap();
        let zip_path = mods_dir.join("law.zip");
        {
            let file = fs::File::create(&zip_path).unwrap();
            let mut writer = ZipWriter::new(file);
            let options =
                SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
            writer.start_file("metadata.toml", options).unwrap();
            writer.write_all(b"title = \"Casual Law\"\nversion = \"1.0.0\"\nmod_id = \"casual-law\"\ncharacter_name = \"Trafalgar Law\"\ndependencies = [\"base-law\"]\nchangelog = \"Initial release\"\ncover = \".metadata/cover.png\"\n").unwrap();
            writer.start_file(".metadata/cover.png", options).unwrap();
            writer.write_all(b"png").unwrap();
            writer.finish().unwrap();
        }
        let config = LauncherConfig {
            game_folder: Some(temp.path().to_string_lossy().to_string()),
            ..LauncherConfig::default()
        };

        let mods = installed_mods(&config);

        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].name, "Casual Law");
        assert_eq!(mods[0].version.as_deref(), Some("1.0.0"));
        assert_eq!(mods[0].mod_key, "id:casual-law");
        assert_eq!(mods[0].character_name.as_deref(), Some("Trafalgar Law"));
        assert_eq!(mods[0].dependencies, vec!["base-law"]);
        assert_eq!(mods[0].changelog.as_deref(), Some("Initial release"));
        assert!(mods[0]
            .cover_data_url
            .as_deref()
            .unwrap_or("")
            .starts_with("data:image/png;base64,"));
    }
}
