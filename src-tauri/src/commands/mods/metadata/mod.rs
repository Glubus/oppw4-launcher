pub(crate) mod reader;
pub(crate) mod zip;

use super::types::ApplyMetadataRequest;
use crate::API_BASE;
use std::path::PathBuf;

#[tauri::command]
pub(crate) fn apply_metadata_to_zip(input: ApplyMetadataRequest) -> Result<(), String> {
    let target_path = PathBuf::from(input.zip_path);
    if !target_path.exists() {
        return Err("Selected ZIP does not exist.".to_string());
    }
    if !target_path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
    {
        return Err("Select a ZIP archive.".to_string());
    }

    let url = format!("{API_BASE}/skins/{}/metadata.zip", input.skin_id);
    let bytes = reqwest::blocking::Client::new()
        .get(url)
        .header("accept", "application/zip")
        .header("user-agent", "oppw4-launcher")
        .send()
        .map_err(|err| format!("Could not download metadata: {err}"))?
        .error_for_status()
        .map_err(|err| format!("Metadata download failed: {err}"))?
        .bytes()
        .map_err(|err| format!("Could not read metadata download: {err}"))?;

    let metadata_entries = zip::read_metadata_entries(bytes.as_ref())?;
    if !metadata_entries
        .iter()
        .any(|entry| entry.0 == "metadata.toml")
    {
        return Err("Downloaded metadata ZIP does not contain metadata.toml.".to_string());
    }

    zip::inject_metadata_entries(&target_path, metadata_entries)
}
