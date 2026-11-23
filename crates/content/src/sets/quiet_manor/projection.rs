use crate::build::{integer, ratio, receipt};
use crate::simulation::{GameActorConfig, GearProcConfig};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(crate) fn compile(tables: &BTreeMap<String, Value>) -> Result<(GearProcConfig, Value), String> {
    let native = ntesim_data::set_passive_values::quiet_manor(tables)?;
    let parameters = &native["parameters"];
    let bonus = ratio(receipt(&parameters["Psychically_4"]["selected"])?)?;
    let duration = integer(
        &parameters["Psychically_4_CD"]["selected"],
        "Quiet Manor duration",
    )?;
    let cap = integer(
        &parameters["Psychically_4_stack"]["selected"],
        "Quiet Manor cap",
    )?;
    if bonus.numerator == 0
        || bonus.numerator > bonus.denominator
        || duration == 0
        || !(1..=64).contains(&cap)
    {
        return Err("Quiet Manor native ratio, seconds or stack units are unsupported".into());
    }
    Ok((
        GearProcConfig::QuietManorMental {
            mental_bonus_per_stack: bonus,
            duration_us: duration
                .checked_mul(1_000_000)
                .ok_or("Quiet Manor duration overflow")?,
            max_stacks: cap as u32,
        },
        native,
    ))
}

pub(crate) fn project_equipped(
    tables: &BTreeMap<String, Value>,
    actor: &mut GameActorConfig,
    build_report: &Value,
) -> Result<Option<Value>, String> {
    let positions = actor
        .gear_procs
        .iter()
        .enumerate()
        .filter_map(|(index, proc)| {
            matches!(proc, GearProcConfig::QuietManorMental { .. }).then_some(index)
        })
        .collect::<Vec<_>>();
    if actor.set_id.as_deref() != Some("set:Suit12") {
        if !positions.is_empty() {
            return Err("Quiet Manor proc disagrees with equipped native set".into());
        }
        return Ok(None);
    }
    if positions.len() > 1 {
        return Err("duplicate Quiet Manor proc in native build".into());
    }
    if build_report["set_count"]
        .as_u64()
        .ok_or("declared set count missing")?
        < 4
    {
        if !positions.is_empty() {
            return Err("four-piece proc requires declared set count 4".into());
        }
        return Ok(None);
    }
    if build_report["set_id"] != "set:Suit12"
        || !build_report["unresolved_set_buffs"]
            .as_array()
            .ok_or("Quiet Manor equipped effects missing")?
            .iter()
            .any(|effect| effect["threshold"] == 4)
    {
        return Err("Quiet Manor four-piece effect is not active in the native build".into());
    }
    let (proc, native) = compile(tables)?;
    let replaced = positions
        .first()
        .map(|index| serde_json::to_value(&actor.gear_procs[*index]))
        .transpose()
        .map_err(|error| error.to_string())?;
    let report = json!({"slot":actor.slot,"set_id":"set:Suit12","replaced_proc":replaced,"compiled_proc":proc,
        "native_parameter_report":native,"activation":"Declared set count 4"});
    if let Some(index) = positions.first() {
        actor.gear_procs[*index] = proc;
    } else {
        actor.gear_procs.push(proc);
    }
    Ok(Some(report))
}
