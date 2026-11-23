use crate::build::{
    WeaponParameters, compile_weapon_values, finish_weapon_projection, ratio, weapon_wearer_item,
};
use crate::simulation::{GameActorConfig, GearProcConfig};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(crate) fn project(
    tables: &BTreeMap<String, Value>,
    schema_version: u32,
    content_id: &str,
    actor: &mut GameActorConfig,
    positions: &[usize],
) -> Result<Value, String> {
    weapon_wearer_item(tables, &actor.character_id, "fork_BlackBook", "Black Tome")?;

    let native = compile_weapon_values(tables, schema_version, content_id, actor)?;
    let expected = [
        ("buff_BlackBook2_Unbal", false),
        ("buff_BlackBook2_CD", false),
        ("buff_BlackBook2_CD2", false),
        ("buff_BlackBook2_DamageUpChaosBase", true),
        ("buff_BlackBook2_SkillDamage", true),
    ];
    let parameters = WeaponParameters::new(&native, &expected, "Black Tome")?;
    let parameter = |name: &str| parameters.fraction(name);
    let proc = GearProcConfig::BlackBookBreak {
        break_intensity_bonus: ratio(parameter("buff_BlackBook2_Unbal")?)?,
    };
    let description = native["description"].clone();
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({
            "runtime_execution_validated": true,
            "native_description": {
                "key": description["Key"],
                "localized": description["LocalizedString"],
            },
            "scope": crate::weapons::black_book::HYPOTHESIS_ID.description()
        }),
    )
}
