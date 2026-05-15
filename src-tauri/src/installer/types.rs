use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub html_url: String,
    pub prerelease: bool,
    pub asset_name: Option<String>,
    pub published_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GithubRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub html_url: String,
    pub prerelease: bool,
    pub published_at: Option<String>,
    pub assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GithubAsset {
    pub name: String,
    pub browser_download_url: String,
}
