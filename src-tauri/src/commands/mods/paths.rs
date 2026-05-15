use crate::system_utils::now_label;
use std::path::{Path, PathBuf};

pub(crate) fn available_mod_path(mods_dir: &Path, file_name: &str) -> PathBuf {
    let stem = safe_mod_stem(file_name);
    first_available_mod_path(mods_dir, &stem)
        .unwrap_or_else(|| mods_dir.join(format!("{stem}-{}.zip", now_label())))
}

fn safe_mod_stem(file_name: &str) -> String {
    let safe_name = Path::new(file_name)
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("mod.zip");
    let stem = safe_name
        .trim_end_matches(".zip")
        .trim_end_matches(".ZIP")
        .trim()
        .replace(['/', '\\'], "-");
    if stem.is_empty() {
        "mod".to_string()
    } else {
        stem
    }
}

fn first_available_mod_path(mods_dir: &Path, stem: &str) -> Option<PathBuf> {
    (0..1000)
        .map(|index| candidate_mod_path(mods_dir, stem, index))
        .find(|candidate| !candidate.exists())
}

fn candidate_mod_path(mods_dir: &Path, stem: &str, index: usize) -> PathBuf {
    let name = if index == 0 {
        format!("{stem}.zip")
    } else {
        format!("{stem}-{index}.zip")
    };
    mods_dir.join(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn available_mod_path_sanitizes_name_and_increments_collisions() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("cool-mod.zip"), b"existing").unwrap();

        let path = available_mod_path(temp.path(), "../cool-mod.zip");

        assert_eq!(
            path.file_name().and_then(|name| name.to_str()),
            Some("cool-mod-1.zip")
        );
    }

    #[test]
    fn available_mod_path_uses_default_for_empty_name() {
        let temp = tempfile::tempdir().unwrap();

        let path = available_mod_path(temp.path(), "");

        assert_eq!(
            path.file_name().and_then(|name| name.to_str()),
            Some("mod.zip")
        );
    }

    #[test]
    fn safe_mod_stem_removes_only_zip_extension_case_insensitively() {
        assert_eq!(safe_mod_stem("Cool Mod.ZIP"), "Cool Mod");
        assert_eq!(
            safe_mod_stem("archive.zip.disabled"),
            "archive.zip.disabled"
        );
        assert_eq!(safe_mod_stem("   .zip"), "mod");
    }

    #[test]
    fn candidate_mod_path_uses_plain_name_for_zero_and_suffix_afterward() {
        let root = Path::new("/mods");

        assert_eq!(
            candidate_mod_path(root, "law", 0),
            PathBuf::from("/mods/law.zip")
        );
        assert_eq!(
            candidate_mod_path(root, "law", 12),
            PathBuf::from("/mods/law-12.zip")
        );
    }
}
