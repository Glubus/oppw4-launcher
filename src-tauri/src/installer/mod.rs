mod github;
mod hash;
mod install;
mod paths;
mod time;
mod types;

pub use hash::installed_dinput8_sha256;
pub use install::restore;
pub use time::today_label;
pub use types::ReleaseInfo;

use crate::{
    config::LauncherConfig,
    error::{InstallerError, InstallerResult},
};
use std::path::PathBuf;

pub fn install_from_latest_release(config: &mut LauncherConfig) -> InstallerResult<()> {
    let repo = config.modloader_repo.trim();
    github::validate_repo(repo)?;
    let game_folder = game_folder(config, "installing the modloader")?;

    let release = github::fetch_latest_release(repo)?;
    let asset = github::installable_asset(&release)?;
    let bytes = github::download_asset(&asset.browser_download_url)?;

    let installed_files = if asset.name.to_lowercase().ends_with(".dll") {
        install::install_dll(&bytes, &game_folder)?
    } else {
        install::install_zip(&bytes, &game_folder)?
    };
    config.modloader_release = Some(release.tag_name);
    config.installed_files = installed_files;
    if let Some(hash) = installed_dinput8_sha256(config)? {
        config.modloader_sha256 = Some(hash.clone());
        config.latest_modloader_sha256 = Some(hash);
        config.latest_modloader_sha256_checked_at = Some(today_label());
    }
    Ok(())
}

pub fn latest_release_info(repo: &str) -> InstallerResult<Option<ReleaseInfo>> {
    if repo.trim().is_empty() || !repo.contains('/') {
        return Ok(None);
    }
    Ok(Some(github::release_info(github::fetch_latest_release(
        repo,
    )?)))
}

pub fn refresh_latest_modloader_hash(
    config: &mut LauncherConfig,
    force: bool,
) -> InstallerResult<Option<String>> {
    if !force
        && config.latest_modloader_sha256_checked_at.as_deref() == Some(today_label().as_str())
    {
        return Ok(config.latest_modloader_sha256.clone());
    }
    let repo = config.modloader_repo.trim();
    if repo.is_empty() || !repo.contains('/') {
        return Ok(None);
    }
    let release = github::fetch_latest_release(repo)?;
    let asset = github::installable_asset(&release)?;
    let bytes = github::download_asset(&asset.browser_download_url)?;
    let hash = if asset.name.to_lowercase().ends_with(".dll") {
        hash::sha256_hex(&bytes)
    } else {
        hash::zip_dinput8_sha256(&bytes)?
    };
    config.latest_modloader_sha256 = Some(hash.clone());
    config.latest_modloader_sha256_checked_at = Some(today_label());
    Ok(Some(hash))
}

fn game_folder(config: &LauncherConfig, action: &'static str) -> InstallerResult<PathBuf> {
    let game_folder = config
        .game_folder
        .clone()
        .ok_or(InstallerError::MissingGameFolder { action })?;
    let game_folder = PathBuf::from(game_folder);
    if !game_folder.is_dir() {
        return Err(InstallerError::GameFolderDoesNotExist);
    }
    Ok(game_folder)
}
