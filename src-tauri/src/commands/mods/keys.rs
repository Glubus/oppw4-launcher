use super::types::{InstalledMod, LocalModMetadata};
use std::collections::HashSet;

pub(crate) fn installed_dependency_keys(mods: &[InstalledMod]) -> HashSet<String> {
    mods.iter()
        .flat_map(installed_dependency_keys_for_mod)
        .map(|key| key.to_lowercase())
        .collect()
}

fn installed_dependency_keys_for_mod(mod_info: &InstalledMod) -> Vec<String> {
    let mut keys = vec![mod_info.mod_key.clone()];
    push_optional_dependency_keys(&mut keys, "id", &mod_info.mod_id);
    push_optional_dependency_keys(&mut keys, "slug", &mod_info.slug);
    push_optional_dependency_keys(&mut keys, "source", &mod_info.source_url);
    keys
}

fn push_optional_dependency_keys(keys: &mut Vec<String>, prefix: &str, value: &Option<String>) {
    if let Some(value) = value {
        keys.push(value.clone());
        keys.push(format!("{prefix}:{value}"));
    }
}

pub(crate) fn mod_key_for(display_name: &str, metadata: &LocalModMetadata) -> String {
    metadata
        .mod_id
        .as_ref()
        .map(|value| format!("id:{value}"))
        .or_else(|| metadata.slug.as_ref().map(|value| format!("slug:{value}")))
        .or_else(|| {
            metadata
                .source_url
                .as_ref()
                .map(|value| format!("source:{value}"))
        })
        .unwrap_or_else(|| local_mod_key(display_name))
}

fn local_mod_key(display_name: &str) -> String {
    format!(
        "local:{}",
        display_name.trim_end_matches(".zip").to_lowercase()
    )
}

pub(crate) fn same_mod_version(mod_info: &InstalledMod, metadata: &LocalModMetadata) -> bool {
    metadata.version.is_some()
        && same_mod_identity(mod_info, metadata)
        && mod_info.version.as_ref() == metadata.version.as_ref()
}

pub(crate) fn same_mod_identity(mod_info: &InstalledMod, metadata: &LocalModMetadata) -> bool {
    metadata
        .mod_id
        .as_ref()
        .is_some_and(|id| mod_info.mod_id.as_ref() == Some(id))
        || metadata
            .slug
            .as_ref()
            .is_some_and(|slug| mod_info.slug.as_ref() == Some(slug))
        || metadata
            .source_url
            .as_ref()
            .is_some_and(|url| mod_info.source_url.as_ref() == Some(url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mod_key_prefers_id_then_slug_then_source_then_local_name() {
        assert_eq!(
            mod_key_for("Fallback.zip", &metadata_with_id("abc")),
            "id:abc"
        );
        assert_eq!(
            mod_key_for(
                "Fallback.zip",
                &LocalModMetadata {
                    slug: Some("sluggy".to_string()),
                    ..LocalModMetadata::default()
                }
            ),
            "slug:sluggy"
        );
        assert_eq!(
            mod_key_for("Fallback.zip", &LocalModMetadata::default()),
            "local:fallback"
        );
    }

    #[test]
    fn same_mod_version_requires_identity_and_version_match() {
        let mod_info = installed_mod("abc", "1.0.0");

        assert!(same_mod_version(
            &mod_info,
            &LocalModMetadata {
                mod_id: Some("abc".to_string()),
                version: Some("1.0.0".to_string()),
                ..LocalModMetadata::default()
            }
        ));
        assert!(!same_mod_version(
            &mod_info,
            &LocalModMetadata {
                mod_id: Some("abc".to_string()),
                version: Some("2.0.0".to_string()),
                ..LocalModMetadata::default()
            }
        ));
    }

    #[test]
    fn installed_dependency_keys_includes_identity_aliases_lowercased() {
        let mut mod_info = installed_mod("ABC", "1.0.0");
        mod_info.mod_key = "ID:ABC".to_string();
        mod_info.slug = Some("Cool-Mod".to_string());
        mod_info.source_url = Some("HTTPS://Example.test/Mod".to_string());

        let keys = installed_dependency_keys(&[mod_info]);

        assert!(keys.contains("id:abc"));
        assert!(keys.contains("abc"));
        assert!(keys.contains("slug:cool-mod"));
        assert!(keys.contains("cool-mod"));
        assert!(keys.contains("source:https://example.test/mod"));
        assert!(keys.contains("https://example.test/mod"));
    }

    #[test]
    fn same_mod_identity_accepts_slug_or_source_when_id_is_absent() {
        let mut mod_info = installed_mod("abc", "1.0.0");
        mod_info.mod_id = None;
        mod_info.slug = Some("sluggy".to_string());
        mod_info.source_url = Some("https://example.test/mod".to_string());

        assert!(same_mod_identity(
            &mod_info,
            &LocalModMetadata {
                slug: Some("sluggy".to_string()),
                ..LocalModMetadata::default()
            }
        ));
        assert!(same_mod_identity(
            &mod_info,
            &LocalModMetadata {
                source_url: Some("https://example.test/mod".to_string()),
                ..LocalModMetadata::default()
            }
        ));
        assert!(!same_mod_identity(
            &mod_info,
            &LocalModMetadata {
                slug: Some("other".to_string()),
                ..LocalModMetadata::default()
            }
        ));
    }

    fn metadata_with_id(id: &str) -> LocalModMetadata {
        LocalModMetadata {
            mod_id: Some(id.to_string()),
            ..LocalModMetadata::default()
        }
    }

    fn installed_mod(id: &str, version: &str) -> InstalledMod {
        InstalledMod {
            name: "Test".to_string(),
            kind: "zip".to_string(),
            path: "/tmp/test.zip".to_string(),
            mod_key: format!("id:{id}"),
            enabled: true,
            mod_id: Some(id.to_string()),
            version: Some(version.to_string()),
            source_url: None,
            slug: None,
            character_name: None,
            character_slug: None,
            mod_type: None,
            dependencies: Vec::new(),
            changelog: None,
            cover_data_url: None,
        }
    }
}
