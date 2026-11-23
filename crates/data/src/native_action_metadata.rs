//! Exact source-bound native labels for actions and damage rows.
//!
//! This module does not assign gameplay meaning to native tags, categories, or
//! damage types. Consumers own those mappings and any execution policy. The
//! caller supplies `content_id` from its audited package context; these lookup
//! functions validate table shape and row identity but do not audit the package
//! manifest or prove that tables and package ID belong together. Baseline action
//! tags default to consumed typed constants when no explicit tag overlay is
//! supplied. Injected test tables must use an explicit hypothetical package ID.
use ntesim_contracts::hypotheses::HypothesisId;
use ntesim_contracts::sha256;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{
    collections::BTreeMap,
    sync::{Mutex, OnceLock},
};

// Only exact immutable embedded row identities enter this cache. Injected table
// copies continue to hash their own bytes; package labels are never cache keys.
static FROZEN_ROW_HASHES: OnceLock<Mutex<BTreeMap<usize, String>>> = OnceLock::new();

fn row_hash(table: &str, row: &str, value: &Value) -> Result<String, String> {
    let frozen = crate::skill_levels::frozen::table(table)
        .and_then(|table| table["rows"].get(row))
        .is_some_and(|frozen| std::ptr::eq(frozen, value));
    let key = std::ptr::from_ref(value) as usize;
    let cache = FROZEN_ROW_HASHES.get_or_init(Mutex::default);
    if frozen
        && let Some(hash) = cache
            .lock()
            .map_err(|_| "frozen row hash cache poisoned")?
            .get(&key)
    {
        return Ok(hash.clone());
    }
    let bytes =
        serde_json::to_vec(value).map_err(|_| "native metadata row serialization failed")?;
    let hash = sha256(&bytes);
    if frozen {
        cache
            .lock()
            .map_err(|_| "frozen row hash cache poisoned")?
            .insert(key, hash.clone());
    }
    Ok(hash)
}

/// Accepted model mapping from the native skill-level coordinate to action role.
/// It is not a GameplayTag interpretation or evidence of a native trigger.
pub const PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::AbilityIndexPrimaryActionRoleV1;
pub const PRIMARY_ROLE_MAPPING_HYPOTHESIS: &str = PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.as_str();
pub const PRIMARY_ROLE_MAPPING_HYPOTHESIS_DESCRIPTION: &str =
    PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.description();

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NativePrimaryRole {
    Normal,
    RedirectSkill,
    Ultimate,
    Qte,
}

pub fn resolve_primary_role(index: Option<u64>) -> Result<NativePrimaryRole, String> {
    match index {
        Some(1) => Ok(NativePrimaryRole::Normal),
        Some(2) => Ok(NativePrimaryRole::RedirectSkill),
        Some(3) => Ok(NativePrimaryRole::Ultimate),
        Some(4) => Ok(NativePrimaryRole::Qte),
        Some(index) => Err(format!("unsupported native AbilityIndex {index}")),
        None => Err("native AbilityIndex is missing or is not an unsigned integer".into()),
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RowState {
    Present,
    MissingNativeRow,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FieldState {
    Present,
    Empty,
    Missing,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct NativeEvidence {
    pub content_id: String,
    pub table: String,
    pub row: String,
    pub pointer: String,
    /// Hash of the resolved transport row; typed tags contain only consumed tag fields.
    pub row_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct NativeTagName {
    pub value: Option<String>,
    pub state: FieldState,
    pub pointer: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct NativeAbilityMetadata {
    pub content_id: String,
    pub character_id: String,
    pub ability_id: String,
    pub ability_list_index: usize,
    /// Native Value.AbilityIndex, kept separate from the serialized list position.
    pub native_ability_index: Option<u64>,
    pub raw_ability_config_entry: Value,
    pub config_evidence: NativeEvidence,
    pub row_state: RowState,
    pub tag_state: FieldState,
    pub gameplay_tag_names: Vec<NativeTagName>,
    pub raw_ability_gameplay_tag: Option<Value>,
    pub raw_row: Option<Value>,
    pub evidence: Option<NativeEvidence>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct NativeDamageEffectMetadata {
    pub content_id: String,
    pub effect_id: String,
    pub row_state: RowState,
    pub ga_name: Option<String>,
    pub damage_type_ex: Option<String>,
    pub damage_source_category: Option<String>,
    pub raw_damage_type_ex: Option<Value>,
    pub raw_damage_source_category: Option<Value>,
    pub raw_row: Option<Value>,
    pub evidence: Option<NativeEvidence>,
}

fn rows<'a>(
    tables: &'a BTreeMap<String, Value>,
    name: &str,
) -> Result<&'a Map<String, Value>, String> {
    let table = tables
        .get(name)
        .ok_or_else(|| format!("native metadata table missing: {name}"))?;
    if table["schema_version"] != 1 || table["source_id"] != format!("table:{name}") {
        return Err(format!(
            "native metadata table schema/source mismatch: {name}"
        ));
    }
    table["rows"]
        .as_object()
        .ok_or_else(|| format!("native metadata rows missing: {name}"))
}

fn lookup<'a>(
    rows: &'a Map<String, Value>,
    key: &str,
) -> Result<Option<(&'a str, &'a Value)>, String> {
    if !key.is_ascii() {
        return Err("non-ASCII native metadata row IDs are unsupported".into());
    }
    let matching = rows
        .iter()
        .filter(|(candidate, _)| candidate.eq_ignore_ascii_case(key))
        .collect::<Vec<_>>();
    if matching.len() > 1 {
        return Err(format!("ambiguous native metadata row ID: {key}"));
    }
    Ok(matching
        .first()
        .map(|(candidate, value)| (candidate.as_str(), *value)))
}

fn evidence(
    content_id: &str,
    table: &str,
    row: &str,
    value: &Value,
    field_pointer: Option<&str>,
) -> Result<NativeEvidence, String> {
    let escaped = row.replace('~', "~0").replace('/', "~1");
    let pointer = format!("/rows/{escaped}{}", field_pointer.unwrap_or(""));
    Ok(NativeEvidence {
        content_id: content_id.to_owned(),
        table: table.to_owned(),
        row: row.to_owned(),
        pointer,
        row_sha256: row_hash(table, row, value)?,
    })
}

fn collect_tags(value: &Value, pointer: &str, output: &mut Vec<NativeTagName>) {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                let child_pointer =
                    format!("{pointer}/{}", key.replace('~', "~0").replace('/', "~1"));
                if key == "TagName" {
                    let value = child.as_str().map(str::to_owned);
                    let state = match value.as_deref() {
                        Some("") => FieldState::Empty,
                        Some(_) => FieldState::Present,
                        None => FieldState::Missing,
                    };
                    output.push(NativeTagName {
                        value,
                        state,
                        pointer: child_pointer,
                    });
                } else {
                    collect_tags(child, &child_pointer, output);
                }
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                collect_tags(child, &format!("{pointer}/{index}"), output);
            }
        }
        _ => {}
    }
}

/// Resolve configured native action tags independently from damage packets.
/// Missing native effect-config rows and empty/missing tag fields remain explicit.
pub fn resolve_ability(
    content_id: &str,
    tables: &BTreeMap<String, Value>,
    character_id: &str,
    ability_id: &str,
) -> Result<NativeAbilityMetadata, String> {
    if content_id.trim().is_empty() {
        return Err("native metadata content_id must be explicit".into());
    }
    let native_character = character_id
        .strip_prefix("character:")
        .filter(|id| !id.is_empty())
        .ok_or("expected character:native_id")?;
    let (config_table, config_rows) = crate::character_actions::rows(tables)?;
    let (config_id, config) =
        lookup(config_rows, native_character)?.ok_or("configured character ability row missing")?;
    let actions = config["CharacterAbilityList"]
        .as_array()
        .ok_or("character actions missing")?;
    let matching = actions
        .iter()
        .enumerate()
        .filter(|(_, action)| {
            action["Key"]
                .as_str()
                .is_some_and(|key| key.eq_ignore_ascii_case(ability_id))
        })
        .collect::<Vec<_>>();
    let (ability_list_index, action, canonical_ability) = if matching.len() == 1 {
        let (index, action) = matching[0];
        let canonical_ability = action["Key"]
            .as_str()
            .ok_or("configured ability ID missing")?;
        (index, action, canonical_ability)
    } else if let Some(parent) =
        crate::character_actions::resolve_evolved_parent(native_character, ability_id)
    {
        let parent_matching = actions
            .iter()
            .enumerate()
            .filter(|(_, action)| {
                action["Key"]
                    .as_str()
                    .is_some_and(|key| key.eq_ignore_ascii_case(parent))
            })
            .collect::<Vec<_>>();
        if parent_matching.len() != 1 {
            return Err("ability must identify exactly one configured character action".into());
        }
        let (index, action) = parent_matching[0];
        (index, action, ability_id)
    } else {
        return Err("ability must identify exactly one configured character action".into());
    };
    let native_ability_index = action["Value"]["AbilityIndex"].as_u64();
    let config_evidence = evidence(
        content_id,
        config_table,
        config_id,
        config,
        Some(&format!("/CharacterAbilityList/{ability_list_index}")),
    )?;
    let raw_ability_config_entry = action.clone();
    let (effect_table, effect_rows) = if tables.contains_key("DT_CharacterAbilityEffectConfig") {
        (
            "DT_CharacterAbilityEffectConfig",
            rows(tables, "DT_CharacterAbilityEffectConfig")?,
        )
    } else if content_id == "ntesim-current" {
        (
            crate::ability_tags::EVIDENCE_TABLE,
            crate::ability_tags::transport()["rows"]
                .as_object()
                .expect("typed action tag rows"),
        )
    } else {
        return Err(
            "explicit native action tag table required outside the baseline package".into(),
        );
    };
    let Some((row_id, row)) = lookup(effect_rows, canonical_ability)? else {
        return Ok(NativeAbilityMetadata {
            content_id: content_id.into(),
            character_id: character_id.into(),
            ability_id: canonical_ability.into(),
            ability_list_index,
            native_ability_index,
            raw_ability_config_entry,
            config_evidence,
            row_state: RowState::MissingNativeRow,
            tag_state: FieldState::Missing,
            gameplay_tag_names: Vec::new(),
            raw_ability_gameplay_tag: None,
            raw_row: None,
            evidence: None,
        });
    };
    let raw_tag = row.get("AbilityGameplayTag").cloned();
    let mut tags = Vec::new();
    if let Some(value) = raw_tag.as_ref() {
        collect_tags(value, "/AbilityGameplayTag", &mut tags);
    }
    let tag_state = if tags.iter().any(|tag| tag.state == FieldState::Present) {
        FieldState::Present
    } else if tags.iter().any(|tag| tag.state == FieldState::Empty) {
        FieldState::Empty
    } else {
        FieldState::Missing
    };
    Ok(NativeAbilityMetadata {
        content_id: content_id.into(),
        character_id: character_id.into(),
        ability_id: canonical_ability.into(),
        ability_list_index,
        native_ability_index,
        raw_ability_config_entry,
        config_evidence,
        row_state: RowState::Present,
        tag_state,
        gameplay_tag_names: tags,
        raw_ability_gameplay_tag: raw_tag,
        raw_row: Some(row.clone()),
        evidence: Some(evidence(content_id, effect_table, row_id, row, None)?),
    })
}

/// Resolve one explicit native damage row by exact row ID; no ability suffix is inferred.
pub fn resolve_damage_effect(
    content_id: &str,
    tables: &BTreeMap<String, Value>,
    effect_id: &str,
) -> Result<NativeDamageEffectMetadata, String> {
    if content_id.trim().is_empty() {
        return Err("native metadata content_id must be explicit".into());
    }
    if content_id != "ntesim-current" && !tables.contains_key("DT_SkillDamageData") {
        return Err("explicit native damage table required outside the baseline package".into());
    }
    let (damage_table, damage_rows) = crate::native_damage::rows(tables)?;
    let Some((row_id, row)) = lookup(damage_rows, effect_id)? else {
        return Ok(NativeDamageEffectMetadata {
            content_id: content_id.into(),
            effect_id: effect_id.into(),
            row_state: RowState::MissingNativeRow,
            ga_name: None,
            damage_type_ex: None,
            damage_source_category: None,
            raw_damage_type_ex: None,
            raw_damage_source_category: None,
            raw_row: None,
            evidence: None,
        });
    };
    let native_string = |field: &str| row.get(field).and_then(Value::as_str).map(str::to_owned);
    Ok(NativeDamageEffectMetadata {
        content_id: content_id.into(),
        effect_id: row_id.into(),
        row_state: RowState::Present,
        ga_name: native_string("GAName"),
        damage_type_ex: native_string("DamageTypeEX"),
        damage_source_category: native_string("DamageSourceCategory"),
        raw_damage_type_ex: row.get("DamageTypeEX").cloned(),
        raw_damage_source_category: row.get("DamageSourceCategory").cloned(),
        raw_row: Some(row.clone()),
        evidence: Some(evidence(content_id, damage_table, row_id, row, None)?),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn frozen_row_hash_cache_cannot_mask_changed_injected_evidence() {
        let tables = crate::skill_levels::frozen::tables();
        let frozen =
            resolve_damage_effect("ntesim-current", tables, "GE_Player_Mint_Melee1_Damage_New")
                .unwrap();
        let again =
            resolve_damage_effect("ntesim-current", tables, "GE_Player_Mint_Melee1_Damage_New")
                .unwrap();
        assert_eq!(frozen, again);
        let mut changed = crate::skill_levels::frozen::overlay_tables();
        changed.get_mut("DT_SkillDamageData").unwrap()["rows"]["GE_Player_Mint_Melee1_Damage_New"]
            ["FixedCritRate"] = serde_json::json!(0.5);
        let injected =
            resolve_damage_effect("hypothetical", &changed, "GE_Player_Mint_Melee1_Damage_New")
                .unwrap();
        assert_ne!(
            frozen.evidence.unwrap().row_sha256,
            injected.evidence.unwrap().row_sha256
        );
        assert_eq!(injected.raw_row.unwrap()["FixedCritRate"], 0.5);
    }

    #[test]
    fn typed_tags_resolve_ascii_case_without_rewriting_native_row_identity() {
        let tables = super::super::skill_levels::frozen::tables();
        for (ability, source_id, expected) in [
            ("GA_Lacrimosa_Melee", "GA_lacrimosa_Melee", "Ability.Melee"),
            ("ga_lacrimosa_skill", "GA_lacrimosa_Skill", "Ability.Skill"),
            (
                "GA_LACRIMOSA_ULTRASKILL",
                "GA_lacrimosa_UltraSkill",
                "Ability.UltraSkill",
            ),
        ] {
            let metadata =
                resolve_ability("ntesim-current", tables, "character:1004", ability).unwrap();
            assert_eq!(metadata.row_state, RowState::Present);
            assert_eq!(
                metadata.gameplay_tag_names[0].value.as_deref(),
                Some(expected)
            );
            assert_eq!(metadata.evidence.unwrap().row, source_id);
        }
    }

    #[test]
    fn typed_baseline_tags_keep_explicit_overlay_absence_and_empty_values() {
        let mut tables = super::super::skill_levels::frozen::overlay_tables();
        let baseline = resolve_ability(
            "ntesim-current",
            &tables,
            "character:1019",
            "GA_Mint019_Skill",
        )
        .unwrap();
        assert_eq!(
            baseline.gameplay_tag_names[0].value.as_deref(),
            Some("Ability.Skill")
        );
        assert_eq!(
            baseline.evidence.unwrap().table,
            crate::ability_tags::EVIDENCE_TABLE
        );
        assert!(
            resolve_ability(
                "hypothesis:fixture",
                &tables,
                "character:1019",
                "GA_Mint019_Skill"
            )
            .is_err()
        );
        tables.insert("DT_CharacterAbilityEffectConfig".into(), json!({"schema_version":1,"source_id":"table:DT_CharacterAbilityEffectConfig","rows":{}}));
        let missing = resolve_ability(
            "ntesim-current",
            &tables,
            "character:1019",
            "GA_Mint019_Skill",
        )
        .unwrap();
        assert_eq!(missing.row_state, RowState::MissingNativeRow);
        assert!(missing.evidence.is_none());
        tables.get_mut("DT_CharacterAbilityEffectConfig").unwrap()["rows"] =
            json!({"GA_Mint019_Skill":{"AbilityGameplayTag":{"TagName":""}}});
        let empty = resolve_ability(
            "hypothesis:fixture",
            &tables,
            "character:1019",
            "GA_Mint019_Skill",
        )
        .unwrap();
        assert_eq!(empty.tag_state, FieldState::Empty);
        assert_eq!(empty.gameplay_tag_names[0].value.as_deref(), Some(""));
        assert_eq!(
            empty.evidence.unwrap().table,
            "DT_CharacterAbilityEffectConfig"
        );
    }

    #[test]
    fn frozen_native_metadata_preserves_exact_labels_and_independent_rows() {
        let tables = super::super::skill_levels::frozen::tables();
        let ability = resolve_ability(
            "ntesim-current",
            tables,
            "character:1019",
            "GA_Mint019_UltraSkill",
        )
        .unwrap();
        assert_eq!(ability.ability_id, "GA_Mint019_UltraSkill");
        assert_eq!(ability.native_ability_index, Some(3));
        assert_eq!(
            resolve_primary_role(ability.native_ability_index).unwrap(),
            NativePrimaryRole::Ultimate
        );
        assert_eq!(
            ability.raw_ability_config_entry["Key"],
            "GA_Mint019_UltraSkill"
        );
        assert_eq!(
            ability.config_evidence.table,
            crate::character_actions::TABLE_NAME
        );
        assert!(
            ability
                .config_evidence
                .pointer
                .contains("/CharacterAbilityList/")
        );
        assert_eq!(
            ability.gameplay_tag_names[0].value.as_deref(),
            Some("Ability.UltraSkill")
        );
        assert_eq!(
            ability.evidence.as_ref().unwrap().table,
            crate::ability_tags::EVIDENCE_TABLE
        );

        let damage = resolve_damage_effect(
            "ntesim-current",
            tables,
            "GE_Player_Mint_UtraSkill1_Damage_New",
        )
        .unwrap();
        assert_eq!(damage.ga_name.as_deref(), Some("GA_Mint019_UltraSkill"));
        assert_eq!(
            damage.damage_type_ex.as_deref(),
            Some("EExecutionDamageTypeEX::DAMAGE_TYPE_NATURE")
        );
        assert_eq!(
            damage.damage_source_category.as_deref(),
            Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q")
        );
        assert_eq!(
            damage.evidence.as_ref().unwrap().table,
            crate::native_damage::TABLE_NAME
        );
    }

    #[test]
    fn missing_native_rows_and_fields_are_not_silently_filled() {
        let mut tables = super::super::skill_levels::frozen::overlay_tables();
        tables.insert(
            "DT_CharacterAbilityEffectConfig".into(),
            crate::ability_tags::transport().clone(),
        );
        tables.get_mut("DT_CharacterAbilityEffectConfig").unwrap()["source_id"] =
            json!("table:DT_CharacterAbilityEffectConfig");
        let effect_table = tables.get_mut("DT_CharacterAbilityEffectConfig").unwrap();
        effect_table["rows"]
            .as_object_mut()
            .unwrap()
            .remove("GA_Mint019_Melee");
        let ability = resolve_ability(
            "hypothesis:test-fixture",
            &tables,
            "character:1019",
            "GA_Mint019_Melee",
        )
        .unwrap();
        assert_eq!(ability.row_state, RowState::MissingNativeRow);
        assert!(ability.evidence.is_none());

        let damage_table = tables.get_mut("DT_SkillDamageData").unwrap();
        let effect = &mut damage_table["rows"]["GE_Player_Mint_UtraSkill1_Damage_New"];
        effect.as_object_mut().unwrap().remove("DamageTypeEX");
        let damage = resolve_damage_effect(
            "hypothesis:test-fixture",
            &tables,
            "GE_Player_Mint_UtraSkill1_Damage_New",
        )
        .unwrap();
        assert_eq!(damage.row_state, RowState::Present);
        assert_eq!(damage.damage_type_ex, None);
        assert_eq!(damage.raw_damage_type_ex, None);
        assert!(damage.evidence.is_some());
        assert_eq!(
            damage.evidence.unwrap().content_id,
            "hypothesis:test-fixture"
        );
    }

    #[test]
    fn native_primary_role_mapping_is_explicit_and_rejects_unbound_indices() {
        assert_eq!(
            resolve_primary_role(Some(1)).unwrap(),
            NativePrimaryRole::Normal
        );
        assert_eq!(
            resolve_primary_role(Some(2)).unwrap(),
            NativePrimaryRole::RedirectSkill
        );
        assert_eq!(
            resolve_primary_role(Some(3)).unwrap(),
            NativePrimaryRole::Ultimate
        );
        assert_eq!(
            resolve_primary_role(Some(4)).unwrap(),
            NativePrimaryRole::Qte
        );
        assert!(resolve_primary_role(None).unwrap_err().contains("missing"));
        assert!(resolve_primary_role(Some(5)).unwrap_err().contains("5"));
    }
}
