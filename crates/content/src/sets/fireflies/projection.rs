//! Source-bound Fireflies values shared by equipped builds, Studio and solo research.
use crate::build::{ActorBuild, integer, ratio, receipt};
use crate::simulation::{
    ExactFraction, GameActionCommand, GameActorConfig, GameScenarioRequest, GearProcConfig,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(crate) fn project(
    tables: &BTreeMap<String, Value>,
    scenario: &mut GameScenarioRequest,
    builds: &[ActorBuild],
) -> Result<Value, String> {
    if scenario.actors.len() != 1 {
        return Err("Fireflies native solo projection requires one actor".into());
    }
    if scenario.actors[0].set_id.as_deref() != Some("set:Suit2") {
        return Err("Fireflies native projection requires equipped set:Suit2".into());
    }
    for command in &scenario.actions {
        if let GameActionCommand::InvokeAbility {
            damage_effect_id, ..
        } = command
            && ntesim_data::native_damage::lookup(tables, damage_effect_id)?.1["DamageTypeEX"]
                != "EExecutionDamageTypeEX::DAMAGE_TYPE_NATURE"
        {
            return Err(
                "Fireflies solo projection supports native Anima damage invocations only".into(),
            );
        }
    }
    let (proc, native) = compile(tables)?;
    let actor = &mut scenario.actors[0];
    let build = builds
        .iter()
        .find(|b| b.slot == actor.slot)
        .ok_or("Fireflies equipped build missing")?;
    let matching = build.build.set_count;
    if matching < 4 {
        return Err("Fireflies requires declared set count 4".into());
    }
    validate_horizon(&proc, scenario.absolute_limit_us)?;
    let mut report = install(actor, proc, native)?;
    report["declared_set_count"] = json!(matching);
    Ok(report)
}

pub(crate) fn compile(tables: &BTreeMap<String, Value>) -> Result<(GearProcConfig, Value), String> {
    let native = ntesim_data::set_passive_values::fireflies(tables)?;
    let parameters = &native["parameters"];
    let amount = ratio(receipt(&parameters["Nature2_4_CritDamage"]["selected"])?)?;
    let cap = integer(
        &parameters["Nature2_4_Stack"]["selected"],
        "Fireflies stack cap",
    )?;
    if !(1..=64).contains(&cap) {
        return Err("Fireflies native stack cap out of runtime range".into());
    }
    let duration =
        receipt(&parameters["Nature2_4_CD"]["selected"])?.mul(ExactFraction::new(1_000_000, 1)?)?;
    if duration.denominator != 1 || duration.is_zero() {
        return Err("Fireflies native duration requires positive integral microseconds".into());
    }
    let duration_us =
        u64::try_from(duration.numerator).map_err(|_| "Fireflies native duration overflow")?;
    Ok((
        GearProcConfig::FirefliesCritDamage {
            crit_damage_per_stack: amount,
            max_stacks: cap as u32,
            duration_us,
        },
        native,
    ))
}

fn validate_horizon(proc: &GearProcConfig, relative_duration_us: u64) -> Result<(), String> {
    if let GearProcConfig::FirefliesCritDamage { duration_us, .. } = proc {
        relative_duration_us
            .checked_add(*duration_us)
            .ok_or("Fireflies native deadline overflow")?;
    }
    Ok(())
}

pub(crate) fn project_equipped(
    tables: &BTreeMap<String, Value>,
    actor: &mut GameActorConfig,
    build_report: &Value,
    relative_duration_us: u64,
    gear_projection: Option<&crate::build::GearProjection>,
) -> Result<Option<Value>, String> {
    // The separate solo research policy produces its receipt after action
    // projection; do not install the same proc twice or erase its input receipt.
    if gear_projection.is_some_and(|projection| {
        matches!(
            projection.policy,
            crate::build::GearProjectionPolicy::NativeMindRoyaleFirefliesSoloV1
        )
    }) {
        return Ok(None);
    }
    if actor.set_id.as_deref() != Some("set:Suit2") {
        if actor
            .gear_procs
            .iter()
            .any(|proc| matches!(proc, GearProcConfig::FirefliesCritDamage { .. }))
        {
            return Err("Fireflies proc disagrees with equipped native set".into());
        }
        return Ok(None);
    }
    if build_report["set_count"]
        .as_u64()
        .ok_or("declared set count missing")?
        < 4
    {
        if actor
            .gear_procs
            .iter()
            .any(|p| matches!(p, GearProcConfig::FirefliesCritDamage { .. }))
        {
            return Err("four-piece proc requires declared set count 4".into());
        }
        return Ok(None);
    }
    if build_report["set_id"] != "set:Suit2"
        || !build_report["unresolved_set_buffs"]
            .as_array()
            .ok_or("Fireflies equipped effects missing")?
            .iter()
            .any(|effect| effect["threshold"] == 4)
    {
        return Err("Fireflies four-piece effect is not active in the native build".into());
    }
    let (proc, native) = compile(tables)?;
    validate_horizon(&proc, relative_duration_us)?;
    Ok(Some(install(actor, proc, native)?))
}

fn install(
    actor: &mut GameActorConfig,
    proc: GearProcConfig,
    native: Value,
) -> Result<Value, String> {
    let positions = actor
        .gear_procs
        .iter()
        .enumerate()
        .filter_map(|(i, p)| matches!(p, GearProcConfig::FirefliesCritDamage { .. }).then_some(i))
        .collect::<Vec<_>>();
    if positions.len() > 1 {
        return Err("duplicate Fireflies proc in native projection".into());
    }
    let replaced = positions
        .first()
        .map(|i| serde_json::to_value(&actor.gear_procs[*i]).map_err(|e| e.to_string()))
        .transpose()?;
    let receipt = json!({"slot":actor.slot,"set_id":"set:Suit2","replaced_proc":replaced,"compiled_proc":proc,"native_parameter_report":native,"runtime_execution_validated":false,
        "scope":"Connected Anima deliveries to all-in-range dummies grant each living equipped team wearer one stack per delivery, including off-field wearers. Independent stack deadlines, no refresh at cap, expiry-before-hit and per-delivery grants remain tested model hypotheses; native trigger and refresh semantics remain unresolved."});
    if let Some(i) = positions.first() {
        actor.gear_procs[*i] = proc;
    } else {
        actor.gear_procs.push(proc);
    }
    Ok(receipt)
}
