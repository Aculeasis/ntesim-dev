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
    relative_duration_us: u64,
    actor: &mut GameActorConfig,
    positions: &[usize],
) -> Result<Value, String> {
    let native = compile_weapon_values(tables, schema_version, content_id, actor)?;
    let parameters = parameters(&native)?;
    let atk_ratio = ratio(parameters.fraction("buff_mofeikesi_Atk")?)?;
    let duration_us = weapon_seconds_to_us(
        parameters.fraction("buff_mofeikesi_CD")?,
        "Good Boy duration",
    )?;
    relative_duration_us
        .checked_add(duration_us)
        .ok_or("Good Boy projected deadline overflow")?;
    let proc = GearProcConfig::GoodBoyTeamAtk {
        team_atk_bonus: atk_ratio,
        duration_us,
        control_atk_bonus: ratio(parameters.fraction("buff_mofeikesi_Up")?)?,
    };
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({
            "runtime_execution_validated": true,
            "scope": super::HYPOTHESIS_ID.description()
        }),
    )
}

fn parameters(native: &Value) -> Result<WeaponParameters<'_>, String> {
    WeaponParameters::new(
        native,
        &[
            ("buff_mofeikesi_ChargeGetEfficiency", true),
            ("buff_mofeikesi_CD", false),
            ("buff_mofeikesi_Atk", true),
            ("buff_mofeikesi_Up", true),
        ],
        "Good Boy",
    )
}

pub(crate) fn static_charge(
    tables: &BTreeMap<String, Value>,
    content_id: &str,
    weapon_id: Option<&str>,
    tier: u32,
) -> Result<Option<Value>, String> {
    let Some(weapon_id) = weapon_id.filter(|id| id.eq_ignore_ascii_case("weapon:fork_mofeikesi"))
    else {
        return Ok(None);
    };
    let native = ntesim_data::weapon_passive_values::compile(
        tables,
        &ntesim_data::weapon_passive_values::Request {
            schema_version: 1,
            content_id: content_id.into(),
            weapon_id: weapon_id.into(),
            refinement: tier,
        },
    )?;
    let params = parameters(&native)?;
    let bonus = params.fraction("buff_mofeikesi_ChargeGetEfficiency")?;
    Ok(Some(
        json!({"weapon_id":weapon_id,"refinement":tier,"charge_efficiency_bonus":ratio(bonus)?,
        "hypothesis_id":super::HYPOTHESIS_ID,"hypothesis":super::HYPOTHESIS_ID.description(),
        "native_parameter_report":native}),
    ))
}
