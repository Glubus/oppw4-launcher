use crate::error::{InstallerError, InstallerResult};
use crate::installer::types::{GithubAsset, GithubRelease, ReleaseInfo};

pub(crate) fn validate_repo(repo: &str) -> InstallerResult<()> {
    if repo.trim().is_empty() || !repo.contains('/') {
        return Err(InstallerError::InvalidRepository);
    }
    Ok(())
}

pub(crate) fn installable_asset(release: &GithubRelease) -> InstallerResult<&GithubAsset> {
    release
        .assets
        .iter()
        .find(|asset| is_installable_asset(&asset.name))
        .ok_or(InstallerError::MissingInstallableAsset)
}

pub(crate) fn is_installable_asset(name: &str) -> bool {
    let name = name.to_lowercase();
    name.ends_with(".zip") || name.ends_with(".dll")
}

pub(crate) fn fetch_latest_release(repo: &str) -> InstallerResult<GithubRelease> {
    let url = format!("https://api.github.com/repos/{repo}/releases?per_page=10");
    let response = reqwest::blocking::Client::new()
        .get(url)
        .header("User-Agent", "oppw4-launcher")
        .send()
        .map_err(|source| InstallerError::Network {
            context: "Could not contact GitHub",
            source,
        })?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(InstallerError::RepositoryHasNoReleases {
            repo: repo.to_string(),
        });
    }

    let releases = response
        .error_for_status()
        .map_err(|source| InstallerError::Network {
            context: "GitHub release request failed",
            source,
        })?
        .json::<Vec<GithubRelease>>()
        .map_err(|source| InstallerError::Json {
            context: "Could not parse GitHub releases",
            source,
        })?;

    releases
        .into_iter()
        .find(|release| !release.assets.is_empty())
        .ok_or_else(|| InstallerError::RepositoryHasNoDownloadableAssets {
            repo: repo.to_string(),
        })
}

pub(crate) fn download_asset(url: &str) -> InstallerResult<Vec<u8>> {
    reqwest::blocking::Client::new()
        .get(url)
        .header("User-Agent", "oppw4-launcher")
        .send()
        .map_err(|source| InstallerError::Network {
            context: "Could not download patcher asset",
            source,
        })?
        .error_for_status()
        .map_err(|source| InstallerError::Network {
            context: "Patcher download failed",
            source,
        })?
        .bytes()
        .map(|bytes| bytes.to_vec())
        .map_err(|source| InstallerError::Network {
            context: "Could not read patcher download",
            source,
        })
}

pub(crate) fn release_info(release: GithubRelease) -> ReleaseInfo {
    let asset_name = release
        .assets
        .iter()
        .find(|asset| is_installable_asset(&asset.name))
        .map(|asset| asset.name.clone());
    ReleaseInfo {
        tag_name: release.tag_name,
        name: release.name,
        body: release.body,
        html_url: release.html_url,
        prerelease: release.prerelease,
        asset_name,
        published_at: release
            .published_at
            .and_then(|value| value.split('T').next().map(str::to_string)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release_with_assets(assets: Vec<&str>) -> GithubRelease {
        GithubRelease {
            tag_name: "v1.2.3".to_string(),
            name: Some("Release".to_string()),
            body: Some("Body".to_string()),
            html_url: "https://github.com/owner/repo/releases/tag/v1.2.3".to_string(),
            prerelease: false,
            published_at: Some("2026-05-15T12:34:56Z".to_string()),
            assets: assets
                .into_iter()
                .map(|name| GithubAsset {
                    name: name.to_string(),
                    browser_download_url: format!("https://example.com/{name}"),
                })
                .collect(),
        }
    }

    #[test]
    fn validate_repo_requires_owner_and_name() {
        assert!(matches!(
            validate_repo(""),
            Err(InstallerError::InvalidRepository)
        ));
        assert!(matches!(
            validate_repo("owner-only"),
            Err(InstallerError::InvalidRepository)
        ));
        assert!(validate_repo("owner/repo").is_ok());
    }

    #[test]
    fn installable_asset_prefers_zip_or_dll() {
        let release = release_with_assets(vec!["readme.txt", "patcher.dll", "patcher.zip"]);

        let asset = installable_asset(&release).unwrap();

        assert_eq!(asset.name, "patcher.dll");
    }

    #[test]
    fn release_info_formats_date_and_asset_name() {
        let release = release_with_assets(vec!["notes.md", "patcher.zip"]);

        let info = release_info(release);

        assert_eq!(info.tag_name, "v1.2.3");
        assert_eq!(info.asset_name.as_deref(), Some("patcher.zip"));
        assert_eq!(info.published_at.as_deref(), Some("2026-05-15"));
    }
}
