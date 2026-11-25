//! Resolve character aliases from typed definitions and gear from native tables.

use serde_json::Value;
use std::collections::BTreeMap;

pub(super) struct Catalog {
    characters: BTreeMap<String, String>,
    weapons: BTreeMap<String, String>,
    sets: BTreeMap<String, String>,
}

fn key(name: &str) -> String {
    name.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

fn insert(aliases: &mut BTreeMap<String, String>, name: &str, id: &str) -> Result<(), String> {
    let key = key(name);
    if key.is_empty() {
        return Ok(());
    }
    if let Some(previous) = aliases.insert(key, id.into())
        && previous != id
    {
        return Err(format!(
            "Ambiguous catalog alias '{name}': {previous} and {id}."
        ));
    }
    Ok(())
}

fn gear_rows(
    rows: &serde_json::Map<String, Value>,
    prefix: &str,
    label: &str,
) -> Result<BTreeMap<String, String>, String> {
    let mut aliases = BTreeMap::new();
    for (native, row) in rows {
        let id = if prefix == "weapon" {
            format!("weapon:{}", native.to_ascii_lowercase())
        } else {
            format!("set:{native}")
        };
        insert(&mut aliases, native, &id)?;
        if let Some(name) = row[label]["LocalizedString"].as_str() {
            insert(&mut aliases, name, &id)?;
        }
    }
    Ok(aliases)
}

impl Catalog {
    pub fn new(tables: &BTreeMap<String, Value>) -> Result<Self, String> {
        let mut characters = BTreeMap::new();
        for descriptor in crate::registry::character_descriptors()? {
            insert(&mut characters, descriptor.slug, descriptor.character_id)?;
            if let Some(name) =
                ntesim_data::character::definition(descriptor.character_id)?.script_alias
            {
                insert(&mut characters, name, descriptor.character_id)?;
            }
        }
        Ok(Self {
            characters,
            weapons: gear_rows(
                ntesim_data::weapon_definitions::rows(tables)?.1,
                "weapon",
                "ItemName",
            )?,
            sets: gear_rows(
                ntesim_data::set_definitions::rows(tables)?.1,
                "set",
                "SuitTitle",
            )?,
        })
    }

    pub fn resolve(&self, category: &str, name: &str) -> Result<String, String> {
        let aliases = match category {
            "character" => &self.characters,
            "weapon" => &self.weapons,
            "cartridge" => &self.sets,
            _ => return Err(format!("Unknown catalog category '{category}'.")),
        };
        aliases
            .get(&key(name))
            .cloned()
            .ok_or_else(|| format!("Unknown {category} '{name}'."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_character_aliases_preserve_public_names_and_reject_unknown_names() {
        let catalog = Catalog::new(ntesim_data::stat_projection::frozen_tables()).unwrap();
        for (name, id) in [
            ("daffodil", "character:1054"),
            ("Daffodill", "character:1054"),
            ("shinku", "character:1076"),
            ("Crimson", "character:1076"),
            ("akane", "character:1057"),
            ("akanerin", "character:1057"),
            ("Akane", "character:1057"),
            ("Akane Rin", "character:1057"),
            ("zankou", "character:1036"),
        ] {
            assert_eq!(catalog.resolve("character", name).unwrap(), id);
        }
        assert!(catalog.resolve("character", " ").is_err());
    }
}
