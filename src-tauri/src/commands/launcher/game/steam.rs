use crate::config::STEAM_APP_ID;
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedGame {
    pub game_folder: String,
    pub executable_path: Option<String>,
    pub source: String,
}

pub fn detect_oppw4() -> Option<DetectedGame> {
    for steam_root in steam_roots() {
        for library in steam_libraries(&steam_root) {
            if let Some(game) = detect_oppw4_in_library(&library) {
                return Some(game);
            }
        }
    }
    None
}

fn detect_oppw4_in_library(library: &Path) -> Option<DetectedGame> {
    let manifest = app_manifest_path(library);
    if !manifest.exists() {
        return None;
    }
    let game_folder = game_folder_from_manifest(library, &manifest)?;
    if !game_folder.exists() {
        return None;
    }
    Some(DetectedGame {
        executable_path: find_game_executable(&game_folder).map(path_to_string),
        game_folder: path_to_string(game_folder),
        source: "Steam".to_string(),
    })
}

fn app_manifest_path(library: &Path) -> PathBuf {
    library
        .join("steamapps")
        .join(format!("appmanifest_{STEAM_APP_ID}.acf"))
}

fn game_folder_from_manifest(library: &Path, manifest: &Path) -> Option<PathBuf> {
    parse_install_dir(manifest)
        .map(|folder_name| library.join("steamapps").join("common").join(folder_name))
}

pub fn steam_libraries(steam_root: &Path) -> Vec<PathBuf> {
    let mut libraries = vec![steam_root.to_path_buf()];
    let vdf = steam_root.join("steamapps").join("libraryfolders.vdf");
    let Ok(raw) = fs::read_to_string(vdf) else {
        return libraries;
    };

    for line in raw.lines() {
        let mut quoted = line.split('"').skip(1).step_by(2);
        let key = quoted.next();
        let value = quoted.next();
        if key == Some("path") {
            if let Some(path) = value {
                libraries.push(PathBuf::from(path.replace("\\\\", "\\")));
            }
        }
    }

    libraries.sort();
    libraries.dedup();
    libraries
}

fn steam_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    #[cfg(target_os = "windows")]
    {
        use std::env;
        if let Ok(program_files_x86) = env::var("ProgramFiles(x86)") {
            roots.push(PathBuf::from(program_files_x86).join("Steam"));
        }
        if let Ok(program_files) = env::var("ProgramFiles") {
            roots.push(PathBuf::from(program_files).join("Steam"));
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(home) = dirs::home_dir() {
            roots.push(home.join(".steam").join("steam"));
            roots.push(home.join(".local").join("share").join("Steam"));
            roots.push(
                home.join(".var")
                    .join("app")
                    .join("com.valvesoftware.Steam")
                    .join(".local")
                    .join("share")
                    .join("Steam"),
            );
        }
    }

    roots.into_iter().filter(|path| path.exists()).collect()
}

fn parse_install_dir(manifest: &Path) -> Option<String> {
    let raw = fs::read_to_string(manifest).ok()?;
    for line in raw.lines() {
        let mut quoted = line.split('"').skip(1).step_by(2);
        let key = quoted.next();
        let value = quoted.next();
        if key == Some("installdir") {
            return value.map(ToOwned::to_owned);
        }
    }
    None
}

fn find_game_executable(game_folder: &Path) -> Option<PathBuf> {
    let candidates = ["OPPW4.exe", "ONE PIECE PIRATE WARRIORS 4.exe", "oppw4.exe"];
    for candidate in candidates {
        let path = game_folder.join(candidate);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

fn path_to_string(path: PathBuf) -> String {
    path.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_libraryfolders_paths() {
        let temp = tempfile::tempdir().unwrap();
        let steamapps = temp.path().join("steamapps");
        fs::create_dir_all(&steamapps).unwrap();
        fs::write(
            steamapps.join("libraryfolders.vdf"),
            r#""libraryfolders"
{
  "0"
  {
    "path" "/mnt/games/Steam"
  }
}"#,
        )
        .unwrap();

        let libraries = steam_libraries(temp.path());
        assert!(libraries.contains(&temp.path().to_path_buf()));
        assert!(libraries.contains(&PathBuf::from("/mnt/games/Steam")));
    }

    #[test]
    fn parse_install_dir_returns_manifest_installdir() {
        let temp = tempfile::tempdir().unwrap();
        let manifest = temp.path().join("appmanifest.acf");
        fs::write(
            &manifest,
            r#""AppState"
{
  "appid" "1089090"
  "installdir" "OPPW4"
}"#,
        )
        .unwrap();

        assert_eq!(parse_install_dir(&manifest).as_deref(), Some("OPPW4"));
    }

    #[test]
    fn detect_oppw4_in_library_requires_manifest_and_existing_folder() {
        let temp = tempfile::tempdir().unwrap();
        assert!(detect_oppw4_in_library(temp.path()).is_none());

        let manifest = app_manifest_path(temp.path());
        fs::create_dir_all(manifest.parent().unwrap()).unwrap();
        fs::write(&manifest, "\"installdir\" \"OPPW4\"").unwrap();
        assert!(detect_oppw4_in_library(temp.path()).is_none());

        let game_folder = temp.path().join("steamapps").join("common").join("OPPW4");
        fs::create_dir_all(&game_folder).unwrap();
        fs::write(game_folder.join("OPPW4.exe"), b"exe").unwrap();

        let detected = detect_oppw4_in_library(temp.path()).unwrap();

        assert_eq!(detected.source, "Steam");
        assert!(detected.game_folder.ends_with("steamapps/common/OPPW4"));
        assert!(detected
            .executable_path
            .as_deref()
            .unwrap_or_default()
            .ends_with("OPPW4.exe"));
    }

    #[test]
    fn find_game_executable_uses_known_candidate_order() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("oppw4.exe"), b"lower").unwrap();
        fs::write(temp.path().join("ONE PIECE PIRATE WARRIORS 4.exe"), b"long").unwrap();

        let executable = find_game_executable(temp.path()).unwrap();

        assert!(executable.ends_with("ONE PIECE PIRATE WARRIORS 4.exe"));
    }
}
