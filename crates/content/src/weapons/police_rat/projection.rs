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
    let item = weapon_wearer_item(
        tables,
        &actor.character_id,
        "fork_PoliceRat",
        "Justice Executioner",
    )?;
    let exclusive = item["ElementData"]["ExclusiveCharacterIDArray"]
        .as_array()
        .ok_or("Justice Executioner exclusive character list missing")?;
    if !exclusive.is_empty()
        || item["ElementData"]["ExclusiveDesc"]["CultureInvariantString"].is_string()
    {
        return Err(
            "Justice Executioner frozen item has unsupported exclusive-character data".into(),
        );
    }

    let native = compile_weapon_values(tables, schema_version, content_id, actor)?;
    let expected = [
        ("buff_PoliceRat_AtkUp", true),
        ("buff_PoliceRat_Up", true),
        ("buff_PoliceRat_SkillDamage", true),
        ("buff_Rat_CD", false),
    ];
    let parameters = WeaponParameters::new(&native, &expected, "Justice Executioner")?;
    let parameter = |name: &str| parameters.fraction(name);
    let proc = GearProcConfig::PoliceRatBoss {
        atk_bonus: ratio(parameter("buff_PoliceRat_AtkUp")?)?,
        boss_damage_bonus: ratio(parameter("buff_PoliceRat_Up")?)?,
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
            "summon_arc_scope": "buff_PoliceRat_SkillDamage and buff_Rat_CD drive the Arc: Officer Whisker summon; executed via Arc admission and periodic ticks",
            "scope": crate::weapons::police_rat::HYPOTHESIS_ID.description()
        }),
    )
}
