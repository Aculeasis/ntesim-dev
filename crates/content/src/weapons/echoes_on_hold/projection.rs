use crate::build::{
    compile_weapon_values, finish_weapon_projection, ratio, receipt, weapon_seconds_to_us,
    weapon_wearer_item,
};
use crate::simulation::{GameActorConfig, GearProcConfig};
use ntesim_data::weapon_refinements::ECHOES_ON_HOLD_PARAMETERS;
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(crate) fn project(
    tables: &BTreeMap<String, Value>,
    schema_version: u32,
    content_id: &str,
    actor: &mut GameActorConfig,
    positions: &[usize],
) -> Result<Value, String> {
    weapon_wearer_item(tables, &actor.character_id, "fork_elyms", "Echoes on Hold")?;
    let values = compile_weapon_values(tables, schema_version, content_id, actor)?;
    let parameters = values["parameters"]
        .as_array()
        .ok_or("Echoes on Hold parameters missing")?;
    if parameters.len() != ECHOES_ON_HOLD_PARAMETERS.len()
        || parameters
            .iter()
            .zip(ECHOES_ON_HOLD_PARAMETERS)
            .any(|(parameter, (name, percent))| {
                !parameter["name_id"]
                    .as_str()
                    .is_some_and(|id| id.eq_ignore_ascii_case(name))
                    || parameter["b_is_percent"] != percent
            })
    {
        return Err("unsupported Echoes on Hold parameters or unit hint".into());
    }
    let selected = |index: usize| receipt(&parameters[index]["selected"]["value"]);
    let proc = GearProcConfig::EchoesOnHold {
        atk_bonus: ratio(selected(0)?)?,
        team_def_ignore: ratio(selected(1)?)?,
        wearer_def_ignore: ratio(selected(2)?)?,
        cosmos_penetration: ratio(selected(3)?)?,
        duration_us: weapon_seconds_to_us(selected(4)?, "Echoes on Hold duration")?,
    };
    finish_weapon_projection(
        actor,
        positions,
        proc,
        values,
        json!({
            "runtime_execution_validated": true,
            "scope": super::HYPOTHESIS_ID.description(),
            "known_gaps": "Refinement parameter names are simulator identifiers. Native scaling rows and refresh ordering have not been recovered."
        }),
    )
}
