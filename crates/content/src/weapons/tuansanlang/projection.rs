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
    weapon_wearer_item(
        tables,
        &actor.character_id,
        "fork_tuansanlang",
        "Danzaburou",
    )?;

    let native = compile_weapon_values(tables, schema_version, content_id, actor)?;
    let expected = [("buff_tuansanlang_Unbal", false)];
    let parameters = WeaponParameters::new(&native, &expected, "Danzaburou")?;
    let parameter = |name: &str| parameters.fraction(name);
    let proc = GearProcConfig::TuansanlangTeam {
        break_intensity_bonus: ratio(parameter("buff_tuansanlang_Unbal")?)?,
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
            "scope": crate::weapons::tuansanlang::HYPOTHESIS_ID.description()
        }),
    )
}
