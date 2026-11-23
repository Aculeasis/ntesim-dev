use crate::build::{ratio, receipt, weapon_seconds_to_us};
use crate::simulation::{GameActorConfig, GearProcConfig};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(crate) fn compile(
    tables: &BTreeMap<String, Value>,
    baseline_in_stats: bool,
) -> Result<(GearProcConfig, Value), String> {
    let native = ntesim_data::set_passive_values::street_boxer(tables)?;
    let parameters = &native["parameters"];
    let crit = ratio(receipt(&parameters["Lakshana_4_1"]["selected"])?)?;
    let powered = ratio(receipt(&parameters["Lakshana_4_2"]["selected"])?)?;
    let duration_us = weapon_seconds_to_us(
        receipt(&parameters["Lakshana_4_CD"]["selected"])?,
        "Street Boxer window",
    )?;
    if crit.numerator == 0
        || crit.numerator > crit.denominator
        || powered.numerator == 0
        || powered.numerator > powered.denominator
        || duration_us == 0
    {
        return Err("Street Boxer crit-bonus units are unsupported".into());
    }
    if baseline_in_stats
        && receipt(&parameters["Lakshana_4_2"]["selected"])?
            .try_cmp(receipt(&parameters["Lakshana_4_1"]["selected"])?)?
            .is_lt()
    {
        return Err("Street Boxer compiled powered replacement must be >= baseline".into());
    }
    Ok((
        GearProcConfig::StreetBoxerCrit {
            crit_rate_bonus: crit,
            powered_crit_rate_bonus: powered,
            duration_us,
            baseline_in_stats,
        },
        native,
    ))
}

pub(crate) fn project_equipped(
    tables: &BTreeMap<String, Value>,
    actor: &mut GameActorConfig,
    build: &Value,
) -> Result<Option<Value>, String> {
    let existing = actor
        .gear_procs
        .iter()
        .enumerate()
        .filter_map(|(index, proc)| {
            matches!(proc, GearProcConfig::StreetBoxerCrit { .. }).then_some(index)
        })
        .collect::<Vec<_>>();
    if actor.set_id.as_deref() != Some("set:Suit5") {
        return if existing.is_empty() {
            Ok(None)
        } else {
            Err("Street Boxer proc conflicts with equipped set".into())
        };
    }
    if build["set_count"]
        .as_u64()
        .ok_or("declared set count missing")?
        < 4
    {
        if !existing.is_empty() {
            return Err("four-piece proc requires declared set count 4".into());
        }
        return Ok(None);
    }
    if existing.len() > 1
        || build["set_id"] != "set:Suit5"
        || !build["unresolved_set_buffs"]
            .as_array()
            .is_some_and(|effects| effects.iter().any(|effect| effect["threshold"] == 4))
    {
        return Err("Street Boxer requires one verified equipped four-piece effect".into());
    }
    let (proc, native) = compile(tables, true)?;
    let replaced = existing
        .first()
        .map(|index| actor.gear_procs[*index].clone());
    let output = json!({"slot": actor.slot, "set_id": "set:Suit5", "compiled_proc": proc,
        "replaced_proc": replaced, "native_parameter_report": native});
    if let Some(index) = existing.first() {
        actor.gear_procs[*index] = proc;
    } else {
        actor.gear_procs.push(proc);
    }
    Ok(Some(output))
}
