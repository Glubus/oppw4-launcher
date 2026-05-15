use crate::{config::LauncherConfig, installer, models::InstalledMod, steam};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LauncherState {
    pub(crate) config: LauncherConfig,
    pub(crate) detected_game: Option<steam::DetectedGame>,
    pub(crate) modloader_status: String,
    pub(crate) latest_release: Option<installer::ReleaseInfo>,
    pub(crate) needs_patcher_update: bool,
    pub(crate) local_modloader_sha256: Option<String>,
    pub(crate) remote_modloader_sha256: Option<String>,
    pub(crate) installed_mods: Vec<InstalledMod>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExportDiagnosticsRequest {
    pub(crate) path: String,
}
