use crate::error::{UpdaterError, UpdaterResult};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub(crate) fn is_supported_asset(name: &str) -> bool {
    if has_extension(name, &["sha256", "sha2", "sig"]) {
        return false;
    }
    #[cfg(target_os = "windows")]
    return has_extension(name, &["msi", "exe", "zip"]);
    #[cfg(target_os = "linux")]
    return has_extension(name, &["appimage", "deb", "rpm", "zip"])
        || ends_with_ignore_ascii_case(name, ".tar.gz");
    #[cfg(target_os = "macos")]
    return has_extension(name, &["dmg", "zip"])
        || ends_with_ignore_ascii_case(name, ".app.tar.gz");
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    false
}

fn has_extension(name: &str, extensions: &[&str]) -> bool {
    Path::new(name)
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| {
            extensions
                .iter()
                .any(|expected| extension.eq_ignore_ascii_case(expected))
        })
}

fn ends_with_ignore_ascii_case(value: &str, suffix: &str) -> bool {
    value
        .get(value.len().saturating_sub(suffix.len())..)
        .is_some_and(|ending| ending.eq_ignore_ascii_case(suffix))
}

pub(crate) fn launcher_dir() -> UpdaterResult<PathBuf> {
    let exe = std::env::current_exe()
        .map_err(|source| UpdaterError::io("Could not resolve launcher executable", source))?;
    exe.parent().map(PathBuf::from).ok_or_else(|| {
        UpdaterError::io(
            "Could not resolve launcher directory",
            std::io::Error::new(std::io::ErrorKind::NotFound, "missing parent"),
        )
    })
}

pub(crate) fn make_executable_if_needed(path: &Path) -> UpdaterResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("appimage"))
        {
            let mut permissions = fs::metadata(path)
                .map_err(|source| {
                    UpdaterError::io(format!("Could not inspect {}", path.display()), source)
                })?
                .permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(path, permissions).map_err(|source| {
                UpdaterError::io(
                    format!("Could not make {} executable", path.display()),
                    source,
                )
            })?;
        }
    }
    Ok(())
}

pub(crate) fn open_path(path: &Path) -> UpdaterResult<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new(path).spawn().map_err(|source| {
            UpdaterError::io(format!("Could not open {}", path.display()), source)
        })?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(path).spawn().map_err(|source| {
            UpdaterError::io(format!("Could not open {}", path.display()), source)
        })?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|source| {
                UpdaterError::io(format!("Could not open {}", path.display()), source)
            })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_asset_rejects_hashes_and_signatures() {
        assert!(!is_supported_asset("launcher.AppImage.sha256"));
        assert!(!is_supported_asset("launcher.zip.sha2"));
        assert!(!is_supported_asset("launcher.AppImage.sig"));
    }

    #[test]
    fn supported_asset_accepts_current_platform_package() {
        #[cfg(target_os = "linux")]
        {
            assert!(is_supported_asset("oppw4-launcher.AppImage"));
            assert!(is_supported_asset("oppw4-launcher.deb"));
            assert!(!is_supported_asset("oppw4-launcher.exe"));
        }

        #[cfg(target_os = "windows")]
        {
            assert!(is_supported_asset("oppw4-launcher.exe"));
            assert!(is_supported_asset("oppw4-launcher.msi"));
            assert!(!is_supported_asset("oppw4-launcher.AppImage"));
        }

        #[cfg(target_os = "macos")]
        {
            assert!(is_supported_asset("oppw4-launcher.dmg"));
            assert!(is_supported_asset("oppw4-launcher.app.tar.gz"));
            assert!(!is_supported_asset("oppw4-launcher.exe"));
        }
    }
}
