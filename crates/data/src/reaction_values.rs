//! Frozen native Esper Cycle reaction values for the shared simulator.
//! Values come from the global DT_ReactionData/DT_ReactionDamageData exports
//! (global-2026-10-04). Consumed fields only: unordered element-pair result
//! bindings, damage-result base arrays and the selected DefaultDamageGE names.
//! Owner-specific variants (GE_ActorReaction_1_1019_Damage, Buff_Reaction_5_new_1036)
//! and unbound application/aura metadata stay cache-only.
use crate::constants::character::Element;
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::OnceLock};

pub const TABLE_NAME: &str = "typed_reaction_values";
pub const SOURCE_ID: &str = "table:typed_reaction_values";

/// Native `EReactionResultType` outcomes bound by DT_ReactionData result rows 1..=6.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReactionResult {
    Blossom,
    Hexed,
    Scorch,
    Nova,
    Stain,
    Remora,
}

impl ReactionResult {
    /// Serialized DT_ReactionData row ordinal (`/0/Rows/REACTION_RESULT_TYPE_<n>`).
    pub const fn native_ordinal(self) -> u8 {
        match self {
            Self::Blossom => 1,
            Self::Hexed => 2,
            Self::Scorch => 3,
            Self::Nova => 4,
            Self::Stain => 5,
            Self::Remora => 6,
        }
    }

    /// Configured `DefaultDamageGE`; `None` rows (Hexed, Stain, Remora) have
    /// separate native consumers and no damage gameplay effect.
    pub const fn damage_effect(self) -> Option<&'static str> {
        match self {
            Self::Blossom => Some("GE_ActorReaction_1_Damage"),
            Self::Hexed => None,
            Self::Scorch => Some("Buff_Reaction_5_new"),
            Self::Nova => Some("Buff_Reaction_4_new"),
            Self::Stain => None,
            Self::Remora => None,
        }
    }

    pub const fn key(self) -> &'static str {
        match self {
            Self::Blossom => "Blossom",
            Self::Hexed => "Hexed",
            Self::Scorch => "Scorch",
            Self::Nova => "Nova",
            Self::Stain => "Stain",
            Self::Remora => "Remora",
        }
    }
}

/// Adjacent-element pairs bound by DT_ReactionData, first element as serialized.
/// Only these six unordered pairs resolve; same-element, Physical (not an
/// `Element`) and nonadjacent pairs produce no native reaction.
const REACTION_PAIRS: &[(Element, Element, ReactionResult)] = &[
    (Element::Cosmos, Element::Nature, ReactionResult::Blossom),
    (Element::Nature, Element::Incantation, ReactionResult::Hexed),
    (Element::Incantation, Element::Chaos, ReactionResult::Scorch),
    (Element::Chaos, Element::Psyche, ReactionResult::Nova),
    (Element::Psyche, Element::Lakshana, ReactionResult::Stain),
    (Element::Lakshana, Element::Cosmos, ReactionResult::Remora),
];

/// Unordered adjacency lookup; row order never selects the result.
pub fn pair_result(first: Element, second: Element) -> Option<ReactionResult> {
    REACTION_PAIRS
        .iter()
        .find(|(a, b, _)| (*a == first && *b == second) || (*a == second && *b == first))
        .map(|(_, _, result)| *result)
}

/// Damage reactions with a frozen per-level `ReactionDamageArray`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReactionDamage {
    Blossom,
    Scorch,
    Nova,
}

impl ReactionDamage {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Blossom => "Blossom",
            Self::Scorch => "Scorch",
            Self::Nova => "Nova",
        }
    }
}

const BLOSSOM_ARRAY: [u64; 16] = [
    80, 120, 200, 300, 400, 600, 800, 1000, 1700, 2200, 3600, 5000, 6000, 7000, 8000, 9000,
];
const SCORCH_ARRAY: [u64; 16] = [
    20, 35, 60, 90, 120, 180, 240, 300, 510, 660, 1080, 1500, 1800, 2100, 2400, 2700,
];
const NOVA_ARRAY: [u64; 16] = [
    400, 600, 1000, 1500, 2000, 3000, 4000, 5000, 8500, 11000, 18000, 25000, 30000, 35000, 40000,
    45000,
];

/// Frozen DT_ReactionDamageData base value for `ceil(level/5)-1`.
/// Supported levels are 1..=80, so the index lands in 0..=15 without clamping.
pub fn base_at_level(kind: ReactionDamage, level: u32) -> Result<u64, String> {
    if level == 0 || level > 80 {
        return Err(format!(
            "unsupported reaction damage level {level}: DT_ReactionDamageArray supports levels 1..=80"
        ));
    }
    let index = level.div_ceil(5) - 1;
    let array = match kind {
        ReactionDamage::Blossom => &BLOSSOM_ARRAY,
        ReactionDamage::Scorch => &SCORCH_ARRAY,
        ReactionDamage::Nova => &NOVA_ARRAY,
    };
    Ok(array[index as usize])
}

/// JSON transport mirrors the typed tables; rows are the six pair bindings and
/// the three damage arrays. Deterministic BTreeMap ordering.
pub fn transport() -> &'static Value {
    static VALUE: OnceLock<Value> = OnceLock::new();
    VALUE.get_or_init(|| {
        let mut rows = BTreeMap::new();
        for (first, second, result) in REACTION_PAIRS {
            rows.insert(
                format!("{}+{}", first.key(), second.key()),
                json!({
                    "result": result.key(),
                    "native_ordinal": result.native_ordinal(),
                    "damage_effect": result.damage_effect(),
                }),
            );
        }
        for (key, array) in [
            (ReactionDamage::Blossom, &BLOSSOM_ARRAY),
            (ReactionDamage::Scorch, &SCORCH_ARRAY),
            (ReactionDamage::Nova, &NOVA_ARRAY),
        ] {
            rows.insert(
                format!("DamageArray:{}", key.key()),
                json!({"ReactionDamageArray": array}),
            );
        }
        json!({"schema_version":1,"source_id":SOURCE_ID,"rows":rows})
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjacent_pairs_resolve_in_both_orders() {
        for (first, second, result) in REACTION_PAIRS {
            assert_eq!(pair_result(*first, *second), Some(*result));
            assert_eq!(pair_result(*second, *first), Some(*result));
        }
        assert_eq!(
            pair_result(Element::Lakshana, Element::Cosmos),
            Some(ReactionResult::Remora)
        );
        assert_eq!(ReactionResult::Blossom.native_ordinal(), 1);
        assert_eq!(
            ReactionResult::Blossom.damage_effect(),
            Some("GE_ActorReaction_1_Damage")
        );
        assert_eq!(ReactionResult::Hexed.damage_effect(), None);
        assert_eq!(ReactionResult::Scorch.native_ordinal(), 3);
        assert_eq!(
            ReactionResult::Scorch.damage_effect(),
            Some("Buff_Reaction_5_new")
        );
        assert_eq!(
            ReactionResult::Nova.damage_effect(),
            Some("Buff_Reaction_4_new")
        );
        assert_eq!(ReactionResult::Stain.damage_effect(), None);
        assert_eq!(ReactionResult::Remora.damage_effect(), None);
    }

    #[test]
    fn same_element_and_nonadjacent_pairs_return_none() {
        // Physical is not an `Element`, so it can never enter the lookup;
        // every pair outside the six adjacent rows returns None.
        let elements = [
            Element::Chaos,
            Element::Cosmos,
            Element::Incantation,
            Element::Lakshana,
            Element::Nature,
            Element::Psyche,
        ];
        for element in elements {
            assert_eq!(pair_result(element, element), None);
        }
        // All nine nonadjacent unordered element pairs, both orders.
        let nonadjacent = [
            (Element::Chaos, Element::Cosmos),
            (Element::Chaos, Element::Lakshana),
            (Element::Chaos, Element::Nature),
            (Element::Cosmos, Element::Incantation),
            (Element::Cosmos, Element::Psyche),
            (Element::Incantation, Element::Lakshana),
            (Element::Incantation, Element::Psyche),
            (Element::Lakshana, Element::Nature),
            (Element::Nature, Element::Psyche),
        ];
        for (first, second) in nonadjacent {
            assert_eq!(pair_result(first, second), None, "{first:?}+{second:?}");
            assert_eq!(pair_result(second, first), None, "{second:?}+{first:?}");
        }
    }

    #[test]
    fn base_at_level_matches_frozen_reaction_damage_arrays() {
        // Level 1 -> index 0.
        assert_eq!(base_at_level(ReactionDamage::Blossom, 1), Ok(80));
        assert_eq!(base_at_level(ReactionDamage::Scorch, 1), Ok(20));
        assert_eq!(base_at_level(ReactionDamage::Nova, 1), Ok(400));
        // Level 75 -> ceil(75/5)-1 = 14.
        assert_eq!(base_at_level(ReactionDamage::Blossom, 75), Ok(8000));
        assert_eq!(base_at_level(ReactionDamage::Scorch, 75), Ok(2400));
        assert_eq!(base_at_level(ReactionDamage::Nova, 75), Ok(40000));
        // Level 80 -> index 15.
        assert_eq!(base_at_level(ReactionDamage::Blossom, 80), Ok(9000));
        assert_eq!(base_at_level(ReactionDamage::Scorch, 80), Ok(2700));
        assert_eq!(base_at_level(ReactionDamage::Nova, 80), Ok(45000));
        // Band boundaries share the frozen value.
        assert_eq!(base_at_level(ReactionDamage::Blossom, 5), Ok(80));
        assert_eq!(base_at_level(ReactionDamage::Blossom, 6), Ok(120));
    }

    #[test]
    fn base_at_level_rejects_unsupported_levels() {
        for level in [0u32, 81, 200] {
            for kind in [
                ReactionDamage::Blossom,
                ReactionDamage::Scorch,
                ReactionDamage::Nova,
            ] {
                let error = base_at_level(kind, level).unwrap_err();
                assert!(error.contains("1..=80"), "level {level}: {error}");
            }
        }
    }

    #[test]
    fn transport_keeps_nine_consumed_rows_without_owner_variants() {
        let value = transport();
        assert_eq!(value["schema_version"], 1);
        assert_eq!(value["source_id"], SOURCE_ID);
        let rows = value["rows"].as_object().unwrap();
        assert_eq!(rows.len(), 9);
        // Six pair rows with the serialized ordinals and DefaultDamageGE names.
        assert_eq!(
            rows["COSMOS+NATURE"],
            json!({"result":"Blossom","native_ordinal":1,"damage_effect":"GE_ActorReaction_1_Damage"})
        );
        assert_eq!(rows["NATURE+INCANTATION"]["damage_effect"], Value::Null);
        assert_eq!(rows["INCANTATION+CHAOS"]["native_ordinal"], 3);
        assert_eq!(rows["CHAOS+PSYCHE"]["damage_effect"], "Buff_Reaction_4_new");
        assert_eq!(rows["PSYCHE+LAKSHANA"]["result"], "Stain");
        assert_eq!(rows["LAKSHANA+COSMOS"]["result"], "Remora");
        // Three frozen base arrays.
        assert_eq!(rows["DamageArray:Blossom"]["ReactionDamageArray"][15], 9000);
        assert_eq!(rows["DamageArray:Scorch"]["ReactionDamageArray"][0], 20);
        assert_eq!(rows["DamageArray:Nova"]["ReactionDamageArray"][8], 8500);
        // Owner-specific variants remain excluded.
        let serialized = value.to_string();
        assert!(!serialized.contains("1019"));
        assert!(!serialized.contains("1036"));
    }
}
