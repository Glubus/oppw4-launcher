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

#[cfg(test)]
mod tests {
    use super::*;

    fn release_with_assets(assets: Vec<types::GithubAsset>) -> GithubRelease {
        GithubRelease {
            tag_name: "v9.9.9".to_string(),
            name: Some("Release".to_string()),
            html_url: "https://example.com/release".to_string(),
            published_at: Some("2026-05-15T10:20:30Z".to_string()),
            assets,
        }
    }

    fn asset(name: &str) -> types::GithubAsset {
        types::GithubAsset {
            name: name.to_string(),
            browser_download_url: format!("https://example.com/{name}"),
            size: 123,
            digest: None,
        }
    }

    #[test]
    fn normalize_version_trims_space_and_leading_v_only() {
        assert_eq!(normalize_version(" v1.2.3 "), "1.2.3");
        assert_eq!(normalize_version("1.2.3"), "1.2.3");
        assert_eq!(normalize_version("release-1.2.3"), "release-1.2.3");
    }

    #[test]
    fn verify_digest_accepts_plain_and_prefixed_sha256() {
        let bytes = b"launcher update";
        let hash = Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();

        assert!(verify_digest(bytes, Some(&hash)).is_ok());
        assert!(verify_digest(bytes, Some(&format!("sha256:{hash}"))).is_ok());
        assert!(verify_digest(bytes, None).is_ok());
    }

    #[test]
    fn verify_digest_rejects_mismatches_case_insensitively() {
        assert!(matches!(
            verify_digest(b"actual", Some("sha256:deadbeef")),
            Err(UpdaterError::HashMismatch)
        ));
    }

    #[test]
    fn installable_asset_returns_first_platform_supported_asset() {
        let release = release_with_assets(vec![asset("checksum.sha256"), asset("launcher.zip")]);

        let chosen = installable_asset(&release).unwrap();

        assert_eq!(chosen.name, "launcher.zip");
    }

    #[test]
    fn installable_asset_errors_when_release_has_only_hash_assets() {
        let release = release_with_assets(vec![asset("launcher.zip.sha256"), asset("notes.txt")]);

        assert!(matches!(
            installable_asset(&release),
            Err(UpdaterError::NoSupportedAsset)
        ));
    }

    #[test]
    fn safe_file_name_strips_directories_and_uses_fallback() {
        assert_eq!(safe_file_name("../launcher.zip"), "launcher.zip");
        assert_eq!(safe_file_name(""), "oppw4-launcher-update");
        assert_eq!(safe_file_name("/"), "oppw4-launcher-update");
    }
}
