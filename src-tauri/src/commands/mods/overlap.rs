use super::types::InstalledMod;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PotentialOverlapGroup {
    pub(crate) character_label: String,
    pub(crate) mod_type: String,
    pub(crate) mod_names: Vec<String>,
}

pub(crate) fn potential_enabled_overlaps(mods: &[InstalledMod]) -> Vec<PotentialOverlapGroup> {
    potential_overlaps(mods.iter().filter(|mod_info| mod_info.enabled))
}

pub(crate) fn potential_overlaps<'a>(
    mods: impl IntoIterator<Item = &'a InstalledMod>,
) -> Vec<PotentialOverlapGroup> {
    let mut groups: BTreeMap<(String, String), Vec<&InstalledMod>> = BTreeMap::new();
    for mod_info in mods {
        let Some(key) = overlap_key(mod_info) else {
            continue;
        };
        groups.entry(key).or_default().push(mod_info);
    }

    groups
        .into_values()
        .filter(|mods| mods.len() > 1)
        .map(overlap_group)
        .collect()
}

fn overlap_key(mod_info: &InstalledMod) -> Option<(String, String)> {
    let character = normalized_value(
        mod_info
            .character_slug
            .as_deref()
            .or(mod_info.character_name.as_deref()),
    )?;
    let mod_type = normalized_value(mod_info.mod_type.as_deref())?;
    Some((character, mod_type))
}

fn normalized_value(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase)
}

fn overlap_group(mods: Vec<&InstalledMod>) -> PotentialOverlapGroup {
    let first = mods[0];
    PotentialOverlapGroup {
        character_label: first
            .character_name
            .clone()
            .or_else(|| first.character_slug.clone())
            .unwrap_or_else(|| "Unknown character".to_string()),
        mod_type: first.mod_type.clone().unwrap_or_else(|| "mod".to_string()),
        mod_names: mods
            .into_iter()
            .map(|mod_info| mod_info.name.clone())
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mod_info(
        name: &str,
        character_slug: Option<&str>,
        mod_type: Option<&str>,
        enabled: bool,
    ) -> InstalledMod {
        InstalledMod {
            name: name.to_string(),
            kind: "zip".to_string(),
            path: format!("/mods/{name}.zip"),
            mod_key: format!("local:{name}"),
            enabled,
            mod_id: None,
            version: None,
            source_url: None,
            slug: None,
            character_name: character_slug.map(str::to_string),
            character_slug: character_slug.map(str::to_string),
            mod_type: mod_type.map(str::to_string),
            dependencies: Vec::new(),
            changelog: None,
            cover_data_url: None,
        }
    }

    #[test]
    fn detects_enabled_mods_with_same_character_and_type() {
        let mods = vec![
            mod_info("Law A", Some("law"), Some("skin"), true),
            mod_info("Law B", Some(" LAW "), Some(" Skin "), true),
            mod_info("Luffy", Some("luffy"), Some("skin"), true),
        ];

        let overlaps = potential_enabled_overlaps(&mods);

        assert_eq!(overlaps.len(), 1);
        assert_eq!(overlaps[0].character_label, "law");
        assert_eq!(overlaps[0].mod_type, "skin");
        assert_eq!(overlaps[0].mod_names, vec!["Law A", "Law B"]);
    }

    #[test]
    fn ignores_disabled_or_incomplete_metadata() {
        let mods = vec![
            mod_info("Disabled A", Some("law"), Some("skin"), false),
            mod_info("Enabled A", Some("law"), Some("skin"), true),
            mod_info("No type", Some("law"), None, true),
            mod_info("No character", None, Some("skin"), true),
        ];

        assert!(potential_enabled_overlaps(&mods).is_empty());
    }
}
