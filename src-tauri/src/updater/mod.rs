mod github;
mod platform;
mod types;

pub use types::{UpdateInfo, UpdateInstallResult};

use crate::error::{UpdaterError, UpdaterResult};
use sha2::{Digest, Sha256};
use std::fs;
use types::GithubRelease;

const LAUNCHER_REPO: &str = "Glubus/oppw4-launcher";

pub fn check() -> UpdaterResult<UpdateInfo> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let release = github::fetch_latest_release(LAUNCHER_REPO)?;
    let asset = release
        .assets
        .iter()
        .find(|asset| platform::is_supported_asset(&asset.name));
    Ok(UpdateInfo {
        available: normalize_version(&release.tag_name) != normalize_version(&current_version)
            && asset.is_some(),
        current_version,
        latest_version: release.tag_name,
        release_name: release.name,
        html_url: release.html_url,
        asset_name: asset.map(|asset| asset.name.clone()),
        asset_size: asset.map(|asset| asset.size),
        published_at: release
            .published_at
            .and_then(|value| value.split('T').next().map(str::to_string)),
    })
}

pub fn install_latest() -> UpdaterResult<UpdateInstallResult> {
    let release = github::fetch_latest_release(LAUNCHER_REPO)?;
    let asset = installable_asset(&release)?;
    let bytes = github::download_asset(&asset.browser_download_url)?;
    verify_digest(&bytes, asset.digest.as_deref())?;

    let update_dir = platform::launcher_dir()?.join("updates");
    fs::create_dir_all(&update_dir)
        .map_err(|source| UpdaterError::io("Could not create launcher update folder", source))?;
    let target = update_dir.join(safe_file_name(&asset.name));
    fs::write(&target, bytes).map_err(|source| {
        UpdaterError::io(format!("Could not write {}", target.display()), source)
    })?;
    platform::make_executable_if_needed(&target)?;
    platform::open_path(&target)?;

    Ok(UpdateInstallResult {
        path: target.to_string_lossy().to_string(),
    })
}

fn installable_asset(release: &GithubRelease) -> UpdaterResult<&types::GithubAsset> {
    release
        .assets
        .iter()
        .find(|asset| platform::is_supported_asset(&asset.name))
        .ok_or(UpdaterError::NoSupportedAsset)
}

fn verify_digest(bytes: &[u8], digest: Option<&str>) -> UpdaterResult<()> {
    let Some(digest) = digest else {
        return Ok(());
    };
    let expected = digest.strip_prefix("sha256:").unwrap_or(digest);
    let actual = Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    if expected.eq_ignore_ascii_case(&actual) {
        Ok(())
    } else {
        Err(UpdaterError::HashMismatch)
    }
}

fn normalize_version(version: &str) -> String {
    version.trim().trim_start_matches('v').to_string()
}

fn safe_file_name(name: &str) -> String {
    std::path::Path::new(name)
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("oppw4-launcher-update")
        .to_string()
}
