use serde::Serialize;

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
    pub(crate) character_name: Option<String>,
    pub(crate) character_slug: Option<String>,
    pub(crate) mod_type: Option<String>,
    pub(crate) dependencies: Vec<String>,
    pub(crate) changelog: Option<String>,
    pub(crate) cover_data_url: Option<String>,
}
