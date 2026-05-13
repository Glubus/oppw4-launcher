use crate::config::{backup_dir, InstalledFile, LauncherConfig};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
  fs,
  io::{Cursor, Read},
  path::{Component, Path, PathBuf},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseInfo {
  pub tag_name: String,
  pub name: Option<String>,
  pub body: Option<String>,
  pub html_url: String,
  pub prerelease: bool,
  pub asset_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
  tag_name: String,
  name: Option<String>,
  body: Option<String>,
  html_url: String,
  prerelease: bool,
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
    .find(|asset| {
      let name = asset.name.to_lowercase();
      name.ends_with(".zip") || name.ends_with(".dll")
    })
    .ok_or_else(|| "Latest GitHub release does not contain a .zip or .dll asset.".to_string())?;
  let bytes = download_asset(&asset.browser_download_url)?;

  let installed_files = if asset.name.to_lowercase().ends_with(".dll") {
    install_dll(&bytes, &game_folder)?
  } else {
    install_zip(&bytes, &game_folder)?
  };
  config.modloader_release = Some(release.tag_name);
  config.installed_files = installed_files;
  if let Some(hash) = installed_dinput8_sha256(config)? {
    config.modloader_sha256 = Some(hash.clone());
    config.latest_modloader_sha256 = Some(hash);
    config.latest_modloader_sha256_checked_at = Some(today_label());
  }
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
  config.modloader_sha256 = None;
  Ok(())
}

pub fn latest_release_info(repo: &str) -> Result<Option<ReleaseInfo>, String> {
  if repo.trim().is_empty() || !repo.contains('/') {
    return Ok(None);
  }
  let release = fetch_latest_release(repo)?;
  let asset_name = release.assets.iter()
    .find(|asset| {
      let name = asset.name.to_lowercase();
      name.ends_with(".zip") || name.ends_with(".dll")
    })
    .map(|asset| asset.name.clone());

  Ok(Some(ReleaseInfo {
    tag_name: release.tag_name,
    name: release.name,
    body: release.body,
    html_url: release.html_url,
    prerelease: release.prerelease,
    asset_name,
  }))
}

pub fn refresh_latest_modloader_hash(config: &mut LauncherConfig, force: bool) -> Result<Option<String>, String> {
  if !force && config.latest_modloader_sha256_checked_at.as_deref() == Some(today_label().as_str()) {
    return Ok(config.latest_modloader_sha256.clone());
  }
  let repo = config.modloader_repo.trim();
  if repo.is_empty() || !repo.contains('/') {
    return Ok(None);
  }
  let release = fetch_latest_release(repo)?;
  let asset = release.assets.iter()
    .find(|asset| {
      let name = asset.name.to_lowercase();
      name.ends_with(".zip") || name.ends_with(".dll")
    })
    .ok_or_else(|| "Latest GitHub release does not contain a .zip or .dll asset.".to_string())?;
  let bytes = download_asset(&asset.browser_download_url)?;
  let hash = if asset.name.to_lowercase().ends_with(".dll") {
    sha256_hex(&bytes)
  } else {
    zip_dinput8_sha256(&bytes)?
  };
  config.latest_modloader_sha256 = Some(hash.clone());
  config.latest_modloader_sha256_checked_at = Some(today_label());
  Ok(Some(hash))
}

pub fn installed_dinput8_sha256(config: &LauncherConfig) -> Result<Option<String>, String> {
  let Some(game_folder) = &config.game_folder else {
    return Ok(None);
  };
  let path = PathBuf::from(game_folder).join("dinput8.dll");
  if !path.exists() {
    return Ok(None);
  }
  let bytes = fs::read(&path).map_err(|err| format!("Could not read {}: {err}", path.display()))?;
  Ok(Some(sha256_hex(&bytes)))
}

fn fetch_latest_release(repo: &str) -> Result<GithubRelease, String> {
  let url = format!("https://api.github.com/repos/{repo}/releases?per_page=10");
  let response = reqwest::blocking::Client::new()
    .get(url)
    .header("User-Agent", "oppw4-launcher")
    .send()
    .map_err(|err| format!("Could not contact GitHub: {err}"))?;

  if response.status() == reqwest::StatusCode::NOT_FOUND {
    return Err(format!("{repo} has no GitHub releases yet."));
  }

  let releases = response
    .error_for_status()
    .map_err(|err| format!("GitHub release request failed: {err}"))?
    .json::<Vec<GithubRelease>>()
    .map_err(|err| format!("Could not parse GitHub releases: {err}"))?;

  releases
    .into_iter()
    .find(|release| !release.assets.is_empty())
    .ok_or_else(|| format!("{repo} has releases, but none of them has downloadable assets."))
}

fn download_asset(url: &str) -> Result<Vec<u8>, String> {
  reqwest::blocking::Client::new()
    .get(url)
    .header("User-Agent", "oppw4-launcher")
    .send()
    .map_err(|err| format!("Could not download patcher asset: {err}"))?
    .error_for_status()
    .map_err(|err| format!("Patcher download failed: {err}"))?
    .bytes()
    .map(|bytes| bytes.to_vec())
    .map_err(|err| format!("Could not read patcher download: {err}"))
}

fn sha256_hex(bytes: &[u8]) -> String {
  let digest = Sha256::digest(bytes);
  digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn zip_dinput8_sha256(bytes: &[u8]) -> Result<String, String> {
  let reader = Cursor::new(bytes);
  let mut archive = zip::ZipArchive::new(reader).map_err(|err| format!("Could not read modloader zip: {err}"))?;
  for index in 0..archive.len() {
    let mut entry = archive.by_index(index).map_err(|err| format!("Could not read zip entry: {err}"))?;
    if entry.is_dir() {
      continue;
    }
    let path = safe_zip_path(entry.name())?;
    if path.file_name().and_then(|value| value.to_str()).is_some_and(|name| name.eq_ignore_ascii_case("dinput8.dll")) {
      let mut content = Vec::new();
      entry.read_to_end(&mut content).map_err(|err| format!("Could not read dinput8.dll: {err}"))?;
      return Ok(sha256_hex(&content));
    }
  }
  Err("Modloader zip does not contain dinput8.dll.".to_string())
}

fn install_dll(bytes: &[u8], game_folder: &Path) -> Result<Vec<InstalledFile>, String> {
  let target = game_folder.join("dinput8.dll");
  let backup_root = backup_dir()?.join(timestamp());
  fs::create_dir_all(&backup_root).map_err(|err| format!("Could not create backup directory: {err}"))?;

  let backup_path = if target.exists() {
    let backup_path = backup_root.join("dinput8.dll");
    fs::copy(&target, &backup_path).map_err(|err| format!("Could not backup {}: {err}", target.display()))?;
    Some(backup_path.to_string_lossy().to_string())
  } else {
    None
  };

  fs::write(&target, bytes).map_err(|err| format!("Could not write {}: {err}", target.display()))?;
  Ok(vec![InstalledFile {
    relative_path: "dinput8.dll".to_string(),
    backup_path,
  }])
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

pub fn today_label() -> String {
  use std::time::{SystemTime, UNIX_EPOCH};
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|duration| (duration.as_secs() / 86_400).to_string())
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

  #[test]
  fn hashes_installed_dinput8_dll() {
    let temp = tempfile::tempdir().unwrap();
    let dll_path = temp.path().join("dinput8.dll");
    fs::write(&dll_path, b"patcher").unwrap();
    let config = LauncherConfig {
      game_folder: Some(temp.path().to_string_lossy().to_string()),
      ..LauncherConfig::default()
    };

    assert_eq!(
      installed_dinput8_sha256(&config).unwrap().as_deref(),
      Some("242d2f23a194483a0aea19c60f86ca2fb887d97edfd2cdfdcf4e2d650a2f79f3")
    );
  }

  #[test]
  fn installed_dinput8_hash_is_absent_when_dll_is_missing() {
    let temp = tempfile::tempdir().unwrap();
    let config = LauncherConfig {
      game_folder: Some(temp.path().to_string_lossy().to_string()),
      ..LauncherConfig::default()
    };

    assert_eq!(installed_dinput8_sha256(&config).unwrap(), None);
  }
}
