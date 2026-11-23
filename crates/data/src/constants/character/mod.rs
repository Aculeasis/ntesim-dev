mod definitions;
pub use definitions::{
    Definition, Element, WeaponGroup, definition, definitions, released_ids, released_inventory,
};
pub mod progression;
mod values;
use crate::native::Scalar;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub const TABLES: [&str; 0] = [];

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema_version: u32,
    pub content_id: String,
    pub character_id: String,
    pub level: u32,
    pub breakthrough: u32,
}

/// Compile typed primary constants and independently rounded growth terms.
/// Character identity, element, group and console selection are also typed.
pub fn compile(request: &Request) -> Result<Value, String> {
    if request.schema_version != 1 || request.content_id != "ntesim-current" {
        return Err("unsupported character request schema/package".into());
    }
    let character = definition(&request.character_id)?;
    let character_id = character.native_id;
    let base = &character.base;
    let stats = base.compile(request.level, request.breakthrough)?;
    let mut attributes = BTreeMap::new();
    for (name, value) in [
        ("HPMaxBase", stats.total.hp),
        ("AtkBase", stats.total.atk),
        ("DefBase", stats.total.def),
        ("ChargeMax", base.energy),
    ] {
        attributes.insert(name, Scalar::new(value.into(), 1));
    }
    for (name, n, d) in [
        ("CritBase", 1, 20),
        ("CritDamageBase", 1, 2),
        ("ChargeGetEfficiencyBase", 1, 1),
        ("UnbalAccrueEfficiencyBase", 1, 1),
    ] {
        attributes.insert(name, Scalar::new(n, d));
    }
    // Only zero stat domains read by the current build/combat projection.
    for name in [
        "HPMaxUp",
        "HPMaxAdd",
        "AtkUp",
        "AtkAdd",
        "DefUp",
        "DefAdd",
        "UnbalIntensityBase",
        "DamageUpNormalBase",
        "DamageUpGeneralBase",
        "DamageUpPsychicallyBase",
        "DamageUpChaosBase",
        "DamageUpCosmosBase",
        "DamageUpIncantationBase",
        "DamageUpLakshanaBase",
        "DamageUpNatureBase",
        "DamageUpPsycheBase",
    ] {
        attributes.insert(name, Scalar::zero());
    }
    let mut terminal_step = json!({"kind":"terminal_breakthrough","values":stats.terminal});
    if request.breakthrough == 0 {
        terminal_step["model_assumption"] = json!(
            "An absent unascended stage-zero pack contributes no breakthrough bonus; base and level growth still apply."
        );
    }
    let steps = json!([
        {"kind":"base","values":base.primary},
        {"kind":"level","values":stats.growth},
        terminal_step
    ]);
    let evidence = json!([
        character.evidence(),
        {"kind":"typed_character_stats","character_id":character_id,"level":request.level,"breakthrough":request.breakthrough,
         "source":"crates/data/src/constants/character/values.rs","base":base.primary,"energy_capacity":base.energy,
         "growth":stats.growth,"terminal_bonus":stats.terminal}
    ]);
    let attributes: BTreeMap<_, _> = attributes
        .into_iter()
        .map(|(key, value)| (key, value.receipt()))
        .collect();
    Ok(json!({
        "schema_version":1,"kind":"character-constants-v1","content_id":request.content_id,
        "character_id":format!("character:{character_id}"),"compatibility_group":character.weapon_group.native_key(),
        "element":character.element.native_key(),"level":request.level,"breakthrough":request.breakthrough,"max_level":stats.max_level,
        "attributes":attributes,"attribute_units":"native property units; growth and terminal terms individually rounded before addition",
        "arithmetic":"checked integer growth; nearest integer with positive halves rounded up; exact rational stat defaults","steps":steps,"evidence":evidence,
        "quality":"incomplete","numeric_status":"typed_character_constant_stats","release_status":"unverified",
        "full_build_legality_checked":false,"combat_adapter_complete":false,
        "gaps":["release availability requires independent roster evidence","awakening, skill, passive and initial-resource execution remain unbound","complete progression, equipment compatibility and build legality require the build contract"]
    }))
}

#[cfg(test)]
mod tests;
