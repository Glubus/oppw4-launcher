use crate::error::{InstallerError, InstallerResult};
use std::path::{Component, PathBuf};

pub(crate) fn safe_zip_path(name: &str) -> InstallerResult<PathBuf> {
    let path = PathBuf::from(name);
    if path.is_absolute() {
        return Err(InstallerError::UnsafeAbsoluteZipPath {
            path: name.to_string(),
        });
    }
    for component in path.components() {
        if matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        ) {
            return Err(InstallerError::UnsafeZipPath {
                path: name.to_string(),
            });
        }
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_parent_zip_paths() {
        assert!(safe_zip_path("../dinput8.dll").is_err());
        assert!(safe_zip_path("mods/../../dinput8.dll").is_err());
    }

    #[test]
    fn accepts_nested_zip_paths() {
        assert_eq!(
            safe_zip_path("loader/config.json").unwrap(),
            PathBuf::from("loader/config.json")
        );
    }
}
