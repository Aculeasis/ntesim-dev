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
    weapon_wearer_item(
        tables,
        &actor.character_id,
        "fork_TigerTally",
        "Commander Tiger Talisman",
    )?;

    let native = compile_weapon_values(tables, schema_version, content_id, actor)?;
    let expected = [
        ("buff_TigerTally_AtkUp", true),
        ("buff_TigerTally_NormalUp", true),
        ("buff_TigerTally_CD", false),
        ("buff_TigerTally_CD4", false),
        ("buff_TigerTally_Qup", true),
        ("buff_TigerTally_CD3", false),
    ];
    let parameters = WeaponParameters::new(&native, &expected, "Commander Tiger Talisman")?;
    let parameter = |name: &str| parameters.fraction(name);
    let duration_us = weapon_seconds_to_us(
        parameter("buff_TigerTally_CD")?,
        "Commander Tiger Talisman duration",
    )?;
    let talisman_window_us = weapon_seconds_to_us(
        parameter("buff_TigerTally_CD4")?,
        "Commander Tiger Talisman talisman window",
    )?;
    let boss_damage_duration_us = weapon_seconds_to_us(
        parameter("buff_TigerTally_CD3")?,
        "Arc Commander Tiger Talisman duration",
    )?;
    let proc = GearProcConfig::TigerTallyAtk {
        atk_bonus: ratio(parameter("buff_TigerTally_AtkUp")?)?,
        normal_damage_bonus: ratio(parameter("buff_TigerTally_NormalUp")?)?,
        duration_us,
        talisman_window_us,
        boss_damage_bonus: ratio(parameter("buff_TigerTally_Qup")?)?,
        boss_damage_duration_us,
    };
    let description = native["description"].clone();
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({
            "runtime_execution_validated": true,
            "excluded_native_clauses": [],
            "native_description": {
                "key": description["Key"],
                "localized": description["LocalizedString"],
            },
            "scope": crate::weapons::tiger_tally::HYPOTHESIS_ID.description()
        }),
    )
}
