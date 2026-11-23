use serde_json::Value;
use std::{collections::BTreeMap, sync::OnceLock};

static TABLES: OnceLock<BTreeMap<String, Value>> = OnceLock::new();

/// Identity check for caches that may only reuse the immutable embedded tables.
/// Caller-supplied or cloned tables must retain their own validation and indexes.
pub(crate) fn table(name: &str) -> Option<&'static Value> {
    TABLES.get()?.get(name)
}

pub fn tables() -> &'static BTreeMap<String, Value> {
    TABLES.get_or_init(|| {
        let mut tables = BTreeMap::from([(
            crate::native_damage::TABLE_NAME.into(),
            crate::native_damage::transport().clone(),
        )]);
        tables.insert(
            crate::character_actions::TABLE_NAME.into(),
            crate::character_actions::build_transport(),
        );
        tables.insert(
            crate::resonance_levels::TABLE_NAME.into(),
            crate::resonance_levels::transport().clone(),
        );
        tables
    })
}

/// Mutable test/research tables with an explicit damage override.
pub fn overlay_tables() -> BTreeMap<String, Value> {
    let mut values = tables().clone();
    values.remove(crate::native_damage::TABLE_NAME);
    values.insert("DT_SkillDamageData".into(), crate::native_damage::overlay());
    values
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    #[test]
    fn effect_figure_source_exports_are_not_embedded_runtime_tables() {
        assert!(
            super::tables()
                .keys()
                .all(|name| !name.ends_with("EffectFigure"))
        );
    }

    #[test]
    fn source_only_costs_and_tips_are_not_embedded_runtime_tables() {
        for name in [
            "DT_GameplayAbilityCostData",
            "DT_GameplayAbilityTipsData",
            "DT_CharacterAbilityEffectConfig",
            "DT_GameplayEffectMappingData",
            "DT_CharacterAbilityConfig",
            "DT_SkillTeamChargeRelationData",
        ] {
            assert!(super::table(name).is_none(), "retired source table: {name}");
        }
    }

    #[test]
    fn awakening_tables_cover_only_supported_simulator_characters() {
        let expected: BTreeSet<_> = crate::character::definitions()
            .iter()
            .map(|character| character.native_id)
            .collect();
        let tables = super::tables();
        assert!(tables.keys().all(|name| !name.contains("AwakenEffect")));
        let actual: BTreeSet<_> = tables[crate::resonance_levels::TABLE_NAME]["rows"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(actual, expected);
    }
}
