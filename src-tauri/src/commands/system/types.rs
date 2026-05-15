use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RevealPathRequest {
    pub(crate) path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LauncherLogRequest {
    pub(crate) level: String,
    pub(crate) message: String,
    pub(crate) file_stamp: String,
    #[serde(default)]
    pub(crate) debug: bool,
}
