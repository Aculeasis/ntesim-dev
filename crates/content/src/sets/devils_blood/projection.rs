use crate::build::{ratio, receipt};
use crate::simulation::{GameActorConfig, GearProcConfig};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(crate) fn compile(tables: &BTreeMap<String, Value>) -> Result<(GearProcConfig, Value), String> {
    let native = ntesim_data::set_passive_values::devils_blood(tables)?;
    let parameters = &native["parameters"];
    let base = ratio(receipt(&parameters["Psyche_4_1"]["selected"])?)?;
    let affected = ratio(receipt(&parameters["Psyche_4_2"]["selected"])?)?;
    if [base, affected]
        .iter()
        .any(|value| value.numerator == 0 || value.numerator > value.denominator)
    {
        return Err("Devil's Blood damage-bonus units are unsupported".into());
    }
    Ok((
        GearProcConfig::DevilsBloodDamage {
            damage_bonus: base,
            affected_bonus: affected,
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
            matches!(proc, GearProcConfig::DevilsBloodDamage { .. }).then_some(index)
        })
        .collect::<Vec<_>>();
    if actor.set_id.as_deref() != Some("set:Suit3") {
        return if existing.is_empty() {
            Ok(None)
        } else {
            Err("Devil's Blood proc conflicts with equipped set".into())
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
        || build["set_id"] != "set:Suit3"
        || !build["unresolved_set_buffs"]
            .as_array()
            .is_some_and(|effects| effects.iter().any(|effect| effect["threshold"] == 4))
    {
        return Err("Devil's Blood requires one verified equipped four-piece effect".into());
    }
    let (proc, native) = compile(tables)?;
    let replaced = existing
        .first()
        .map(|index| actor.gear_procs[*index].clone());
    let output = json!({"slot": actor.slot, "set_id": "set:Suit3", "compiled_proc": proc,
        "replaced_proc": replaced, "native_parameter_report": native});
    if let Some(index) = existing.first() {
        actor.gear_procs[*index] = proc;
    } else {
        actor.gear_procs.push(proc);
    }
    Ok(Some(output))
}
