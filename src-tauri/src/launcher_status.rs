use crate::{
    config::LauncherConfig,
    diagnostics::{health_item, latest_loader_log, HealthCheckItem},
    installer,
    mod_inventory::{installed_dependency_keys, installed_mods},
    models::InstalledMod,
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
    let mut items = Vec::new();
    let Some(game_folder) = &config.game_folder else {
        items.push(health_item(
            "error",
            "Game folder",
            "No game folder selected.",
        ));
        return items;
    };
    let game_folder = PathBuf::from(game_folder);
    if game_folder.is_dir() {
        items.push(health_item(
            "ok",
            "Game folder",
            &format!("Using {}.", game_folder.display()),
        ));
    } else {
        items.push(health_item(
            "error",
            "Game folder",
            "Selected game folder does not exist.",
        ));
        return items;
    }

    let local_hash = installer::installed_dinput8_sha256(config).ok().flatten();
    items.push(
        match modloader_status(config, local_hash.as_deref()).as_str() {
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
        },
    );

    let mods_dir = game_folder.join("mods");
    if mods_dir.is_dir() {
        items.push(health_item(
            "ok",
            "Mods folder",
            &format!("Found {}.", mods_dir.display()),
        ));
    } else {
        items.push(health_item(
            "warn",
            "Mods folder",
            "No mods folder found yet.",
        ));
    }

    let mods = installed_mods(config);
    if mods.is_empty() {
        items.push(health_item(
            "warn",
            "Installed mods",
            "No local mods were detected.",
        ));
    } else {
        let enabled = mods.iter().filter(|mod_info| mod_info.enabled).count();
        items.push(health_item(
            "ok",
            "Installed mods",
            &format!("{enabled}/{} mods enabled.", mods.len()),
        ));
    }

    let missing_metadata = mods
        .iter()
        .filter(|mod_info| {
            mod_info.kind == "zip" && mod_info.mod_id.is_none() && mod_info.slug.is_none()
        })
        .count();
    if missing_metadata > 0 {
        items.push(health_item(
            "warn",
            "Metadata",
            &format!("{missing_metadata} ZIP mod(s) have no usable metadata identity."),
        ));
    } else if !mods.is_empty() {
        items.push(health_item(
            "ok",
            "Metadata",
            "Installed ZIP mods have usable metadata.",
        ));
    }

    let missing_dependencies = missing_enabled_dependencies(&mods);
    if missing_dependencies.is_empty() {
        items.push(health_item(
            "ok",
            "Dependencies",
            "No missing enabled mod dependencies detected.",
        ));
    } else {
        items.push(health_item(
            "error",
            "Dependencies",
            &missing_dependencies.join("; "),
        ));
    }

    if let Some(log_path) = latest_loader_log(config) {
        items.push(health_item(
            "ok",
            "Loader log",
            &format!("Latest log: {}.", log_path.display()),
        ));
    } else {
        items.push(health_item(
            "warn",
            "Loader log",
            "No loader log found in mods/_oppw4/logs.",
        ));
    }

    items
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
