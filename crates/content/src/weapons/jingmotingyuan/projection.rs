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
        "fork_jingmotingyuan",
        "Silent Garden",
    )?;

    let native = compile_weapon_values(tables, schema_version, content_id, actor)?;
    let expected = [
        ("buff_tingyuan_CD", false),
        ("buff_tingyuan_CritDamageUp", true),
        ("buff_tingyuan_HPreduce", true),
        ("buff_tingyuan_CD2", false),
        ("GE_Fork_serenity_Skill1_Damage1", true),
        ("GE_Fork_serenity_Skill2_Damage", true),
    ];
    let parameters = WeaponParameters::new(&native, &expected, "Silent Garden")?;
    let parameter = |name: &str| parameters.fraction(name);
    let proc = GearProcConfig::JingmotingyuanCrit {
        crit_damage_bonus: ratio(parameter("buff_tingyuan_CritDamageUp")?)?,
        duration_us: weapon_seconds_to_us(
            parameter("buff_tingyuan_CD")?,
            "Silent Garden duration",
        )?,
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
            "scope": crate::weapons::jingmotingyuan::HYPOTHESIS_ID.description()
        }),
    )
}
