use crate::ApplyMetadataRequest;

#[tauri::command]
pub(crate) fn apply_metadata_to_zip(input: ApplyMetadataRequest) -> Result<(), String> {
    crate::apply_metadata_to_zip(input)
}
