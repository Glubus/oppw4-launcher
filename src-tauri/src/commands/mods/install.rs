use crate::{
    ImportExternalZipRequest, InstallHostedModRequest, InstallHostedModResult, InstalledMod,
    InstalledModLookupRequest,
};

#[tauri::command]
pub(crate) fn import_external_zip(input: ImportExternalZipRequest) -> Result<InstalledMod, String> {
    crate::import_external_zip(input)
}

#[tauri::command]
pub(crate) fn install_hosted_mod(
    input: InstallHostedModRequest,
) -> Result<InstallHostedModResult, String> {
    crate::install_hosted_mod(input)
}

#[tauri::command]
pub(crate) fn installed_mod_for_skin(
    input: InstalledModLookupRequest,
) -> Result<Option<InstalledMod>, String> {
    crate::installed_mod_for_skin(input)
}
