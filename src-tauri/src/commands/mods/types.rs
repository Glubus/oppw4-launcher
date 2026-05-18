use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InstalledMod {
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) path: String,
    pub(crate) mod_key: String,
    pub(crate) enabled: bool,
    pub(crate) mod_id: Option<String>,
    pub(crate) version: Option<String>,
    pub(crate) source_url: Option<String>,
    pub(crate) slug: Option<String>,
    pub(crate) content_kind: String,
    pub(crate) character_name: Option<String>,
    pub(crate) character_slug: Option<String>,
    pub(crate) mod_type: Option<String>,
    pub(crate) dependencies: Vec<String>,
    pub(crate) changelog: Option<String>,
    pub(crate) cover_data_url: Option<String>,
}

#[derive(Debug, Default)]
pub(crate) struct LocalModMetadata {
    pub(crate) mod_id: Option<String>,
    pub(crate) title: Option<String>,
    pub(crate) version: Option<String>,
    pub(crate) source_url: Option<String>,
    pub(crate) slug: Option<String>,
    pub(crate) content_kind: Option<String>,
    pub(crate) character_name: Option<String>,
    pub(crate) character_slug: Option<String>,
    pub(crate) mod_type: Option<String>,
    pub(crate) dependencies: Vec<String>,
    pub(crate) changelog: Option<String>,
    pub(crate) cover_data_url: Option<String>,
}

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
    pub(crate) content_kind: Option<String>,
    pub(crate) slug: Option<String>,
    pub(crate) title: Option<String>,
    pub(crate) version: Option<String>,
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
