use crate::build::{
    WeaponParameters, compile_weapon_values, finish_weapon_projection, integer, ratio,
    weapon_seconds_to_us, weapon_wearer_item,
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
    let item = weapon_wearer_item(tables, &actor.character_id, "fork_rishi", "Eclipse")?;
    let exclusive = item["ElementData"]["ExclusiveCharacterIDArray"]
        .as_array()
        .ok_or("Eclipse frozen item has unsupported exclusive-character data")?;
    if !exclusive.is_empty()
        || item["ElementData"]["ExclusiveDesc"]["CultureInvariantString"].is_string()
    {
        return Err("Eclipse frozen item has unsupported exclusive-character data".into());
    }

    let native = compile_weapon_values(tables, schema_version, content_id, actor)?;
    let expected = [
        ("buff_rishi_AtkUp", true),
        ("buff_rishi_CD2", false),
        ("buff_rishi_energy", false),
        ("buff_rishi_stack", false),
        ("buff_rishi_CD", false),
    ];
    let parameters = WeaponParameters::new(&native, &expected, "Eclipse")?;
    let parameter = |name: &str| parameters.fraction(name);
    let max_defeats = integer(
        parameters.selected("buff_rishi_stack")?,
        "Eclipse defeat cap",
    )?;
    let proc = GearProcConfig::RishiAtk {
        atk_bonus: ratio(parameter("buff_rishi_AtkUp")?)?,
        eclipse_duration_us: weapon_seconds_to_us(
            parameter("buff_rishi_CD2")?,
            "Eclipse field duration",
        )?,
        defeat_energy: integer(
            parameters.selected("buff_rishi_energy")?,
            "Eclipse defeat energy",
        )?,
        max_defeats: u32::try_from(max_defeats).map_err(|_| "Eclipse defeat cap exceeds u32")?,
        arc_cooldown_us: weapon_seconds_to_us(parameter("buff_rishi_CD")?, "Eclipse arc cooldown")?,
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
            "arc_scope": "buff_rishi_CD2 binds eclipse_duration_us (40s field), buff_rishi_energy binds defeat_energy (flat whole energy per defeat), buff_rishi_stack binds max_defeats, buff_rishi_CD binds arc_cooldown_us (300s). Field creation runs on explicit wearer Arc admission; Ultimate alone does not activate it. Each wearer-credited defeat inside the field window grants defeat_energy through the live grant path up to max_defeats per field.",
            "scope": crate::weapons::rishi::HYPOTHESIS_ID.description()
        }),
    )
}
