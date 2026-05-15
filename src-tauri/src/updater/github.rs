use crate::error::{UpdaterError, UpdaterResult};
use crate::updater::types::GithubRelease;
use reqwest::StatusCode;

pub(crate) fn fetch_latest_release(repo: &str) -> UpdaterResult<GithubRelease> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let response = reqwest::blocking::Client::new()
        .get(url)
        .header("User-Agent", "oppw4-launcher")
        .send()
        .map_err(|source| UpdaterError::Network {
            context: "Could not contact GitHub",
            source,
        })?;
    if matches!(
        response.status(),
        StatusCode::FORBIDDEN | StatusCode::TOO_MANY_REQUESTS
    ) {
        return Err(UpdaterError::RateLimited);
    }
    response
        .error_for_status()
        .map_err(|source| UpdaterError::Network {
            context: "Launcher update request failed",
            source,
        })?
        .json::<GithubRelease>()
        .map_err(|source| UpdaterError::Json {
            context: "Could not parse launcher update release",
            source,
        })
}

pub(crate) fn download_asset(url: &str) -> UpdaterResult<Vec<u8>> {
    reqwest::blocking::Client::new()
        .get(url)
        .header("User-Agent", "oppw4-launcher")
        .send()
        .map_err(|source| UpdaterError::Network {
            context: "Could not download launcher update",
            source,
        })?
        .error_for_status()
        .map_err(|source| UpdaterError::Network {
            context: "Launcher update download failed",
            source,
        })?
        .bytes()
        .map(|bytes| bytes.to_vec())
        .map_err(|source| UpdaterError::Network {
            context: "Could not read launcher update download",
            source,
        })
}
