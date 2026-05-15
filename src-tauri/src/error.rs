use std::path::PathBuf;
use thiserror::Error;

pub type CommandResult<T> = Result<T, String>;
pub type InstallerResult<T> = Result<T, InstallerError>;
pub type UpdaterResult<T> = Result<T, UpdaterError>;

#[derive(Debug, Error)]
pub enum InstallerError {
    #[error("Set a GitHub repository as owner/name before installing.")]
    InvalidRepository,
    #[error("Set the game folder before {action}.")]
    MissingGameFolder { action: &'static str },
    #[error("Game folder does not exist.")]
    GameFolderDoesNotExist,
    #[error("Latest GitHub release does not contain a .zip or .dll asset.")]
    MissingInstallableAsset,
    #[error("{repo} has no GitHub releases yet.")]
    RepositoryHasNoReleases { repo: String },
    #[error("{repo} has releases, but none of them has downloadable assets.")]
    RepositoryHasNoDownloadableAssets { repo: String },
    #[error("Modloader zip does not contain dinput8.dll.")]
    ZipMissingDinput8,
    #[error("Modloader zip did not contain installable files.")]
    ZipDidNotContainInstallableFiles,
    #[error("Unsafe absolute zip path: {path}")]
    UnsafeAbsoluteZipPath { path: String },
    #[error("Unsafe zip path: {path}")]
    UnsafeZipPath { path: String },
    #[error("{context}: {source}")]
    Network {
        context: &'static str,
        #[source]
        source: reqwest::Error,
    },
    #[error("{context}: {source}")]
    Zip {
        context: &'static str,
        #[source]
        source: zip::result::ZipError,
    },
    #[error("{context}: {source}")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },
    #[error("{context}: {source}")]
    Json {
        context: &'static str,
        #[source]
        source: reqwest::Error,
    },
    #[error("{0}")]
    Config(String),
}

impl InstallerError {
    pub fn io(context: impl Into<String>, source: std::io::Error) -> Self {
        Self::Io {
            context: context.into(),
            source,
        }
    }

    pub fn path_io(action: &'static str, path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        let path = path.into();
        Self::io(format!("{action} {}", path.display()), source)
    }
}

#[derive(Debug, Error)]
pub enum UpdaterError {
    #[error("Latest launcher release does not contain a supported asset for this platform.")]
    NoSupportedAsset,
    #[error("Downloaded launcher update hash does not match GitHub digest.")]
    HashMismatch,
    #[error("{context}: {source}")]
    Network {
        context: &'static str,
        #[source]
        source: reqwest::Error,
    },
    #[error("{context}: {source}")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },
    #[error("{context}: {source}")]
    Json {
        context: &'static str,
        #[source]
        source: reqwest::Error,
    },
}

impl UpdaterError {
    pub fn io(context: impl Into<String>, source: std::io::Error) -> Self {
        Self::Io {
            context: context.into(),
            source,
        }
    }
}
