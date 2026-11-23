use crate::build::{
    WeaponParameters, compile_weapon_values, finish_weapon_projection, integer, ratio,
    weapon_wearer_item,
};
use crate::simulation::{GameActorConfig, GearProcConfig};
use serde_json::{Value, json};
use std::collections::BTreeMap;

const EXPECTED_DESCRIPTION: &str = "Increases Cosmos DMG by <lv>{1}</> for every <lv>{0}</> Fons the wearer holds, up to <lv>10</> stacks.";

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
        "fork_mamen",
        "Contemplative Cat",
    )?;
    let exclusive = item["ElementData"]["ExclusiveCharacterIDArray"]
        .as_array()
        .ok_or("Contemplative Cat exclusive character list missing")?;
    if !exclusive.is_empty()
        || item["ElementData"]["ExclusiveDesc"]["CultureInvariantString"].is_string()
    {
        return Err(
            "Contemplative Cat frozen item has unsupported exclusive-character data".into(),
        );
    }

    let native = compile_weapon_values(tables, schema_version, content_id, actor)?;
    let expected = [("buff_mamen_fons", false), ("buff_mamen_CosmosUp", true)];
    let parameters = WeaponParameters::new(&native, &expected, "Contemplative Cat")?;
    let selected = |name: &str| parameters.selected(name);
    let description = native["description"]["LocalizedString"]
        .as_str()
        .ok_or("Contemplative Cat native description missing")?;
    if description != EXPECTED_DESCRIPTION {
        return Err(
            "Contemplative Cat native description changed; the 10-stack cap evidence must be rechecked"
                .into(),
        );
    }
    let fons_per_stack = integer(
        selected("buff_mamen_fons")?,
        "Contemplative Cat fons_per_stack",
    )?;
    if fons_per_stack == 0 {
        return Err("Contemplative Cat requires a positive fons_per_stack".into());
    }
    let proc = GearProcConfig::MamenFons {
        cosmos_bonus_per_stack: ratio(parameters.fraction("buff_mamen_CosmosUp")?)?,
        fons_per_stack,
        max_stacks: 10,
    };
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({"runtime_execution_validated":true,
        "hypothesis_id":crate::weapons::mamen::HYPOTHESIS_ID,
        "scope":crate::weapons::mamen::HYPOTHESIS_ID.description(),
        "fons_input":"Explicit party start_fons is read at combat time; projection does not create a weapon-local balance.",
        "passive_description":{"key":"buff_fork_mamen_effect","localized":EXPECTED_DESCRIPTION}}),
    )
}
