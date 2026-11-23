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
    let item = weapon_wearer_item(tables, &actor.character_id, "fork_Rose", "Bohemian Rose")?;
    let exclusive = item["ElementData"]["ExclusiveCharacterIDArray"]
        .as_array()
        .ok_or("Bohemian Rose exclusive character list missing")?;
    let exclusive_ids: Vec<&str> = exclusive
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .ok_or("Bohemian Rose invalid exclusive-character ID")
        })
        .collect::<Result<_, _>>()?;
    let exclusive_desc = item["ElementData"]["ExclusiveDesc"]["LocalizedString"]
        .as_str()
        .unwrap_or("");
    if exclusive_ids != ["1004"] || !exclusive_desc.contains("skin") {
        return Err("Bohemian Rose frozen item has unsupported exclusive-character data".into());
    }

    let native = compile_weapon_values(tables, schema_version, content_id, actor)?;
    let expected = [
        ("buff_Rose_AtkUp", true),
        ("buff_Rose_CritDamageUp", true),
        ("buff_Rose_CD", false),
        ("buff_Rose_UnbalTime", false),
    ];
    let parameters = WeaponParameters::new(&native, &expected, "Bohemian Rose")?;
    let parameter = |name: &str| parameters.fraction(name);
    let proc = GearProcConfig::RoseThorn {
        atk_bonus: ratio(parameter("buff_Rose_AtkUp")?)?,
        crit_damage_per_stack: ratio(parameter("buff_Rose_CritDamageUp")?)?,
        duration_us: weapon_seconds_to_us(
            parameter("buff_Rose_CD")?,
            "Bohemian Rose thorn duration",
        )?,
        broken_extension_us: weapon_seconds_to_us(
            parameter("buff_Rose_UnbalTime")?,
            "Bohemian Rose Broken extension",
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
            "scope": crate::weapons::rose::HYPOTHESIS_ID.description()
        }),
    )
}
