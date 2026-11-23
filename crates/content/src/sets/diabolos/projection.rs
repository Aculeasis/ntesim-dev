use crate::build::{ratio, receipt, weapon_seconds_to_us};
use crate::simulation::{GameActorConfig, GearProcConfig};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(crate) fn compile(tables: &BTreeMap<String, Value>) -> Result<(GearProcConfig, Value), String> {
    let native = ntesim_data::set_passive_values::diabolos(tables)?;
    let parameters = &native["parameters"];
    let ignore = ratio(receipt(&parameters["Chaos2_4_IgnoreChaos"]["selected"])?)?;
    let powered = ratio(receipt(&parameters["Chaos2_4_IgnoreChaos2"]["selected"])?)?;
    let duration_us = weapon_seconds_to_us(
        receipt(&parameters["Chaos2_4_CD"]["selected"])?,
        "Diabolos window",
    )?;
    if ignore.numerator == 0
        || ignore.numerator > ignore.denominator
        || powered.numerator == 0
        || powered.numerator > powered.denominator
        || duration_us == 0
    {
        return Err("Diabolos Chaos-ignore units are unsupported".into());
    }
    Ok((
        GearProcConfig::DiabolosChaosIgnore {
            chaos_res_ignore: ignore,
            powered_chaos_res_ignore: powered,
            duration_us,
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
            matches!(proc, GearProcConfig::DiabolosChaosIgnore { .. }).then_some(index)
        })
        .collect::<Vec<_>>();
    if actor.set_id.as_deref() != Some("set:Suit1") {
        return if existing.is_empty() {
            Ok(None)
        } else {
            Err("Diabolos proc conflicts with equipped set".into())
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
        || build["set_id"] != "set:Suit1"
        || !build["unresolved_set_buffs"]
            .as_array()
            .is_some_and(|effects| effects.iter().any(|effect| effect["threshold"] == 4))
    {
        return Err("Diabolos requires one verified equipped four-piece effect".into());
    }
    let (proc, native) = compile(tables)?;
    let replaced = existing
        .first()
        .map(|index| actor.gear_procs[*index].clone());
    let output = json!({"slot": actor.slot, "set_id": "set:Suit1", "compiled_proc": proc,
        "replaced_proc": replaced, "native_parameter_report": native});
    if let Some(index) = existing.first() {
        actor.gear_procs[*index] = proc;
    } else {
        actor.gear_procs.push(proc);
    }
    Ok(Some(output))
}
