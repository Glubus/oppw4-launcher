use super::{health_item, logs::latest_loader_log, HealthCheckItem};
use crate::{
    commands::mods::{
        inventory::installed_mods, keys::installed_dependency_keys, types::InstalledMod,
    },
    config::LauncherConfig,
    installer,
};
use std::path::PathBuf;

pub(crate) fn modloader_status(config: &LauncherConfig, local_hash: Option<&str>) -> String {
    let Some(game_folder) = &config.game_folder else {
        return "Missing game folder".to_string();
    };
    if !PathBuf::from(game_folder).join("dinput8.dll").exists() {
        return if config.installed_files.is_empty() {
            "Missing".to_string()
        } else {
            "Missing installed dinput8.dll".to_string()
        };
    }
    if config.installed_files.is_empty() || config.modloader_sha256.is_none() {
        return "Detected unmanaged dinput8.dll".to_string();
    }
    if local_hash != config.modloader_sha256.as_deref() {
        return "Modified dinput8.dll".to_string();
    }
    if config
        .latest_modloader_sha256
        .as_ref()
        .is_some_and(|hash| config.modloader_sha256.as_ref() != Some(hash))
    {
        return "Update available".to_string();
    }
    "Installed".to_string()
}

pub(crate) fn build_health_check(config: &LauncherConfig) -> Vec<HealthCheckItem> {
    let Some(game_folder) = valid_game_folder(config) else {
        return vec![game_folder_health(config)];
    };
    let mods = installed_mods(config);
    health_sections(config, &game_folder, &mods)
        .into_iter()
        .flatten()
        .collect()
}

fn valid_game_folder(config: &LauncherConfig) -> Option<PathBuf> {
    let game_folder = PathBuf::from(config.game_folder.as_ref()?);
    game_folder.is_dir().then_some(game_folder)
}

fn health_sections(
    config: &LauncherConfig,
    game_folder: &PathBuf,
    mods: &[InstalledMod],
) -> Vec<Vec<HealthCheckItem>> {
    vec![
        vec![game_folder_health(config)],
        vec![patcher_health(config)],
        vec![mods_folder_health(game_folder)],
        vec![installed_mods_health(mods)],
        metadata_health(mods).into_iter().collect(),
        vec![dependencies_health(mods)],
        vec![loader_log_health(config)],
    ]
}

fn game_folder_health(config: &LauncherConfig) -> HealthCheckItem {
    match config.game_folder.as_ref().map(PathBuf::from) {
        None => health_item("error", "Game folder", "No game folder selected."),
        Some(path) if path.is_dir() => {
            health_item("ok", "Game folder", &format!("Using {}.", path.display()))
        }
        Some(_) => health_item(
            "error",
            "Game folder",
            "Selected game folder does not exist.",
        ),
    }
}

fn patcher_health(config: &LauncherConfig) -> HealthCheckItem {
    let local_hash = installer::installed_dinput8_sha256(config).ok().flatten();
    patcher_health_for_status(&modloader_status(config, local_hash.as_deref()))
}

fn patcher_health_for_status(status: &str) -> HealthCheckItem {
    match status {
        "Installed" => health_item(
            "ok",
            "Patcher",
            "dinput8.dll is installed and matches the tracked hash.",
        ),
        "Update available" => health_item(
            "warn",
            "Patcher",
            "A newer patcher asset is available on GitHub.",
        ),
        "Modified dinput8.dll" => health_item(
            "warn",
            "Patcher",
            "The local dinput8.dll does not match the tracked install hash.",
        ),
        "Detected unmanaged dinput8.dll" => health_item(
            "warn",
            "Patcher",
            "A dinput8.dll exists, but it was not installed by this launcher.",
        ),
        "Missing installed dinput8.dll" => health_item(
            "error",
            "Patcher",
            "The launcher tracks an install, but dinput8.dll is missing.",
        ),
        status => health_item("error", "Patcher", status),
    }
}

fn mods_folder_health(game_folder: &PathBuf) -> HealthCheckItem {
    let mods_dir = game_folder.join("mods");
    if mods_dir.is_dir() {
        health_item(
            "ok",
            "Mods folder",
            &format!("Found {}.", mods_dir.display()),
        )
    } else {
        health_item("warn", "Mods folder", "No mods folder found yet.")
    }
}

fn installed_mods_health(mods: &[InstalledMod]) -> HealthCheckItem {
    if mods.is_empty() {
        return health_item("warn", "Installed mods", "No local mods were detected.");
    }
    let enabled = mods.iter().filter(|mod_info| mod_info.enabled).count();
    health_item(
        "ok",
        "Installed mods",
        &format!("{enabled}/{} mods enabled.", mods.len()),
    )
}

fn metadata_health(mods: &[InstalledMod]) -> Option<HealthCheckItem> {
    let missing_metadata = missing_metadata_count(mods);
    if missing_metadata > 0 {
        return Some(health_item(
            "warn",
            "Metadata",
            &format!("{missing_metadata} ZIP mod(s) have no usable metadata identity."),
        ));
    }
    (!mods.is_empty())
        .then(|| health_item("ok", "Metadata", "Installed ZIP mods have usable metadata."))
}

fn missing_metadata_count(mods: &[InstalledMod]) -> usize {
    mods.iter()
        .filter(|mod_info| {
            mod_info.kind == "zip" && mod_info.mod_id.is_none() && mod_info.slug.is_none()
        })
        .count()
}

fn dependencies_health(mods: &[InstalledMod]) -> HealthCheckItem {
    let missing_dependencies = missing_enabled_dependencies(mods);
    if missing_dependencies.is_empty() {
        health_item(
            "ok",
            "Dependencies",
            "No missing enabled mod dependencies detected.",
        )
    } else {
        health_item("error", "Dependencies", &missing_dependencies.join("; "))
    }
}

fn loader_log_health(config: &LauncherConfig) -> HealthCheckItem {
    latest_loader_log(config)
        .map(|path| {
            health_item(
                "ok",
                "Loader log",
                &format!("Latest log: {}.", path.display()),
            )
        })
        .unwrap_or_else(|| {
            health_item(
                "warn",
                "Loader log",
                "No loader log found in mods/_oppw4/logs.",
            )
        })
}

fn missing_enabled_dependencies(mods: &[InstalledMod]) -> Vec<String> {
    let installed_keys = installed_dependency_keys(mods);
    mods.iter()
        .filter(|mod_info| mod_info.enabled)
        .flat_map(|mod_info| missing_dependencies_for_mod(mod_info, &installed_keys))
        .collect()
}

fn missing_dependencies_for_mod(
    mod_info: &InstalledMod,
    installed_keys: &std::collections::HashSet<String>,
) -> Vec<String> {
    mod_info
        .dependencies
        .iter()
        .filter(|dependency| !installed_keys.contains(&dependency.to_lowercase()))
        .map(|dependency| format!("{} needs {}", mod_info.name, dependency))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::mods::types::InstalledMod;

    fn mod_info(name: &str) -> InstalledMod {
        InstalledMod {
            name: name.to_string(),
            path: format!("/mods/{name}.zip"),
            enabled: true,
            kind: "zip".to_string(),
            version: None,
            mod_id: None,
            slug: None,
            source_url: None,
            mod_key: format!("local:{name}.zip"),
            character_name: None,
            character_slug: None,
            mod_type: None,
            dependencies: Vec::new(),
            changelog: None,
            cover_data_url: None,
        }
    }

    #[test]
    fn metadata_health_warns_only_for_zip_mods_without_identity() {
        let missing = mod_info("missing");
        let mut folder = mod_info("folder");
        folder.kind = "folder".to_string();
        let mut identified = mod_info("identified");
        identified.mod_id = Some("identified".to_string());

        let item = metadata_health(&[missing, folder, identified]).unwrap();

        assert_eq!(item.level, "warn");
        assert_eq!(item.title, "Metadata");
        assert!(item.detail.contains("1 ZIP mod(s)"));
    }

    #[test]
    fn dependencies_health_reports_missing_enabled_dependencies() {
        let mut dependency = mod_info("dependency");
        dependency.mod_id = Some("base-law".to_string());
        dependency.enabled = false;
        let mut consumer = mod_info("consumer");
        consumer.dependencies = vec!["base-law".to_string(), "missing-pack".to_string()];

        let item = dependencies_health(&[dependency, consumer]);

        assert_eq!(item.level, "error");
        assert_eq!(item.title, "Dependencies");
        assert!(item.detail.contains("consumer needs missing-pack"));
        assert!(!item.detail.contains("consumer needs base-law"));
    }

    #[test]
    fn patcher_health_maps_known_statuses_to_user_messages() {
        let installed = patcher_health_for_status("Installed");
        let modified = patcher_health_for_status("Modified dinput8.dll");
        let missing = patcher_health_for_status("Missing installed dinput8.dll");

        assert_eq!(installed.level, "ok");
        assert_eq!(modified.level, "warn");
        assert_eq!(missing.level, "error");
    }
}
