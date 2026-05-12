use crate::config::{backup_dir, InstalledFile, LauncherConfig};
use serde::Deserialize;
use std::{
  fs,
  io::{Cursor, Read},
  path::{Component, Path, PathBuf},
};

#[derive(Debug, Deserialize)]
struct GithubRelease {
  tag_name: String,
  assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
  name: String,
  browser_download_url: String,
}

pub fn install_from_latest_release(config: &mut LauncherConfig) -> Result<(), String> {
  let repo = config.modloader_repo.trim();
  if repo.is_empty() || !repo.contains('/') {
    return Err("Set a GitHub repository as owner/name before installing.".to_string());
  }
  let game_folder = config.game_folder.clone().ok_or_else(|| "Set the game folder before installing the modloader.".to_string())?;
  let game_folder = PathBuf::from(game_folder);
  if !game_folder.is_dir() {
    return Err("Game folder does not exist.".to_string());
  }

  let release = fetch_latest_release(repo)?;
  let asset = release.assets.iter()
    .find(|asset| asset.name.to_lowercase().ends_with(".zip"))
    .ok_or_else(|| "Latest GitHub release does not contain a zip asset.".to_string())?;
  let bytes = reqwest::blocking::Client::new()
    .get(&asset.browser_download_url)
    .header("User-Agent", "oppw4-launcher")
    .send()
    .map_err(|err| format!("Could not download modloader asset: {err}"))?
    .error_for_status()
    .map_err(|err| format!("Modloader download failed: {err}"))?
    .bytes()
    .map_err(|err| format!("Could not read modloader download: {err}"))?;

  let installed_files = install_zip(&bytes, &game_folder)?;
  config.modloader_release = Some(release.tag_name);
  config.installed_files = installed_files;
  Ok(())
}

pub fn restore(config: &mut LauncherConfig) -> Result<(), String> {
  let game_folder = config.game_folder.clone().ok_or_else(|| "Set the game folder before restoring.".to_string())?;
  let game_folder = PathBuf::from(game_folder);

  for installed in config.installed_files.iter().rev() {
    let target = game_folder.join(&installed.relative_path);
    if target.exists() {
      fs::remove_file(&target).map_err(|err| format!("Could not remove {}: {err}", target.display()))?;
    }
    if let Some(backup) = &installed.backup_path {
      let backup = PathBuf::from(backup);
      if backup.exists() {
        if let Some(parent) = target.parent() {
          fs::create_dir_all(parent).map_err(|err| format!("Could not create {}: {err}", parent.display()))?;
        }
        fs::copy(&backup, &target).map_err(|err| format!("Could not restore {}: {err}", target.display()))?;
      }
    }
  }

  config.installed_files.clear();
  config.modloader_release = None;
  Ok(())
}

fn fetch_latest_release(repo: &str) -> Result<GithubRelease, String> {
  let url = format!("https://api.github.com/repos/{repo}/releases/latest");
  reqwest::blocking::Client::new()
    .get(url)
    .header("User-Agent", "oppw4-launcher")
    .send()
    .map_err(|err| format!("Could not contact GitHub: {err}"))?
    .error_for_status()
    .map_err(|err| format!("GitHub release request failed: {err}"))?
    .json()
    .map_err(|err| format!("Could not parse GitHub release: {err}"))
}

fn install_zip(bytes: &[u8], game_folder: &Path) -> Result<Vec<InstalledFile>, String> {
  let reader = Cursor::new(bytes);
  let mut archive = zip::ZipArchive::new(reader).map_err(|err| format!("Could not read modloader zip: {err}"))?;
  let mut installed = Vec::new();
  let backup_root = backup_dir()?.join(timestamp());
  fs::create_dir_all(&backup_root).map_err(|err| format!("Could not create backup directory: {err}"))?;

  for index in 0..archive.len() {
    let mut entry = archive.by_index(index).map_err(|err| format!("Could not read zip entry: {err}"))?;
    if entry.is_dir() {
      continue;
    }
    let relative = safe_zip_path(entry.name())?;
    let target = game_folder.join(&relative);
    let backup_path = if target.exists() {
      let backup_path = backup_root.join(&relative);
      if let Some(parent) = backup_path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("Could not create backup parent: {err}"))?;
      }
      fs::copy(&target, &backup_path).map_err(|err| format!("Could not backup {}: {err}", target.display()))?;
      Some(backup_path.to_string_lossy().to_string())
    } else {
      None
    };

    if let Some(parent) = target.parent() {
      fs::create_dir_all(parent).map_err(|err| format!("Could not create {}: {err}", parent.display()))?;
    }
    let mut content = Vec::new();
    entry.read_to_end(&mut content).map_err(|err| format!("Could not read {}: {err}", entry.name()))?;
    fs::write(&target, content).map_err(|err| format!("Could not write {}: {err}", target.display()))?;

    installed.push(InstalledFile {
      relative_path: relative.to_string_lossy().to_string(),
      backup_path,
    });
  }

  if installed.is_empty() {
    return Err("Modloader zip did not contain installable files.".to_string());
  }

  Ok(installed)
}

fn safe_zip_path(name: &str) -> Result<PathBuf, String> {
  let path = PathBuf::from(name);
  if path.is_absolute() {
    return Err(format!("Unsafe absolute zip path: {name}"));
  }
  for component in path.components() {
    if matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)) {
      return Err(format!("Unsafe zip path: {name}"));
    }
  }
  Ok(path)
}

fn timestamp() -> String {
  use std::time::{SystemTime, UNIX_EPOCH};
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|duration| duration.as_secs().to_string())
    .unwrap_or_else(|_| "0".to_string())
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
    assert_eq!(safe_zip_path("loader/config.json").unwrap(), PathBuf::from("loader/config.json"));
  }
}

