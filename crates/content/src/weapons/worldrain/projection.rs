use crate::build::{
    WeaponParameters, compile_weapon_values, finish_weapon_projection, ratio, weapon_seconds_to_us,
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
    crate::build::weapon_wearer_item(tables, &actor.character_id, "fork_Worldrain", "Worldrain")?;
    let native = compile_weapon_values(tables, schema_version, content_id, actor)?;
    let parameters = WeaponParameters::new(
        &native,
        &[
            ("buff_worldrain_CosmosUp", true),
            ("buff_worldrain_Mag", false),
            ("buff_worldrain_CD", false),
        ],
        "Worldrain",
    )?;
    let proc = GearProcConfig::WorldrainDamage {
        damage_bonus: ratio(parameters.fraction("buff_worldrain_CosmosUp")?)?,
        cycle_intensity_bonus: ratio(parameters.fraction("buff_worldrain_Mag")?)?,
        duration_us: weapon_seconds_to_us(
            parameters.fraction("buff_worldrain_CD")?,
            "Worldrain CI duration",
        )?,
    };
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({"runtime_execution_validated":true,
        "scope":crate::weapons::worldrain::HYPOTHESIS_ID.description()}),
    )
}
