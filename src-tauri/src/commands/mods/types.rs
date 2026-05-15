use crate::models::InstalledMod;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ToggleModRequest {
    pub(crate) path: String,
    pub(crate) enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImportExternalZipRequest {
    pub(crate) path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApplyProfileRequest {
    pub(crate) profile_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApplyMetadataRequest {
    pub(crate) skin_id: String,
    pub(crate) zip_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InstallHostedModRequest {
    pub(crate) file_id: String,
    pub(crate) file_name: String,
    #[serde(default)]
    pub(crate) install_as_new: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InstalledModLookupRequest {
    pub(crate) mod_id: Option<String>,
    pub(crate) slug: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RevealModRequest {
    pub(crate) path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RemoveModRequest {
    pub(crate) path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InstallHostedModResult {
    pub(crate) mod_info: InstalledMod,
    pub(crate) already_up_to_date: bool,
}
