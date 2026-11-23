use crate::build::{
    WeaponParameters, compile_weapon_values, finish_weapon_projection, ratio, weapon_seconds_to_us,
    weapon_wearer_item,
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
    weapon_wearer_item(tables, &actor.character_id, "fork_dustbin", "Decomposer")?;

    let native = compile_weapon_values(tables, schema_version, content_id, actor)?;
    let expected = [
        ("buff_dustbin_Unbal", false),
        ("buff_dustbin_CD", false),
        ("buff_dustbin_CD2", false),
    ];
    let parameters = WeaponParameters::new(&native, &expected, "Decomposer")?;
    let parameter = |name: &str| parameters.fraction(name);
    let proc = GearProcConfig::DustbinBreak {
        break_intensity_bonus: ratio(parameter("buff_dustbin_Unbal")?)?,
        duration_us: weapon_seconds_to_us(parameter("buff_dustbin_CD")?, "Decomposer duration")?,
        cooldown_us: weapon_seconds_to_us(
            parameter("buff_dustbin_CD2")?,
            "Decomposer trigger cooldown",
        )?,
    };
    let description = native["description"].clone();
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({
            "runtime_execution_validated": false,
            "native_description": {
                "key": description["Key"],
                "localized": description["LocalizedString"],
            },
            "scope": crate::weapons::dustbin::HYPOTHESIS_ID.description()
        }),
    )
}
