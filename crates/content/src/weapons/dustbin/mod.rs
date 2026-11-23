use crate::HypothesisId;
mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod projection;
use crate::simulation::state::{TimedBuffDeclaration, apply_timed_buff};
use crate::simulation::{ExactFraction, GearProcConfig};
use ntesim_core::buffs::dispatch::{
    BuffDuration, BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy,
};
use std::collections::BTreeMap;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsDustbinV1;
pub(crate) const BUFF_ID: &str = "dustbin.break_intensity";
pub(crate) const ICD_BUFF_ID: &str = "dustbin.break_cooldown";
pub(crate) const SOURCE: &str = "weapon:fork_dustbin";

pub(crate) fn icd_active(
    buffs: &ntesim_core::buffs::dispatch::BuffManager,
    actor_index: usize,
    time_us: u64,
) -> bool {
    buffs.instances().iter().any(|instance| {
        instance.id == ICD_BUFF_ID
            && instance.owner_actor == actor_index
            && match &instance.duration {
                BuffDuration::Timed { expires_at_us, .. } => *expires_at_us > time_us,
                BuffDuration::Permanent => true,
            }
    })
}

pub(crate) fn try_grant_break_buff(
    buffs: &mut ntesim_core::buffs::dispatch::BuffManager,
    actor_index: usize,
    time_us: u64,
    proc: &GearProcConfig,
) -> Result<bool, String> {
    let GearProcConfig::DustbinBreak {
        break_intensity_bonus,
        duration_us,
        cooldown_us,
    } = proc
    else {
        return Err("Decomposer grant received the wrong proc kind".into());
    };
    if icd_active(buffs, actor_index, time_us) {
        return Ok(false);
    }
    let bonus = ExactFraction::from_ratio(*break_intensity_bonus)?;
    let buff = TimedBuffDeclaration {
        id: BUFF_ID.into(),
        source: SOURCE.into(),
        recipient: BuffRecipient::SpecificActor(actor_index),
        duration_us: *duration_us,
        stacking_scope: BuffStackingScope::PerSource,
        refresh_policy: RefreshPolicy::Disallow,
        max_stacks: 1,
        decay_policy: DecayPolicy::AllAtOnce,
        stat_modifiers: BTreeMap::from([("break_intensity".into(), bonus)]),
    };
    apply_timed_buff(buffs, buff, actor_index, time_us)
        .map_err(|error| format!("Decomposer buff failed: {error:?}"))?;
    let cooldown = TimedBuffDeclaration {
        id: ICD_BUFF_ID.into(),
        source: SOURCE.into(),
        recipient: BuffRecipient::SpecificActor(actor_index),
        duration_us: *cooldown_us,
        stacking_scope: BuffStackingScope::PerSource,
        refresh_policy: RefreshPolicy::Disallow,
        max_stacks: 1,
        decay_policy: DecayPolicy::AllAtOnce,
        stat_modifiers: BTreeMap::new(),
    };
    apply_timed_buff(buffs, cooldown, actor_index, time_us)
        .map_err(|error| format!("Decomposer cooldown marker failed: {error:?}"))?;
    Ok(true)
}

pub(crate) fn on_break_reduced(
    request: &crate::simulation::GameScenarioRequest,
    state: &mut crate::simulation::state::GameSimulatorState,
    actor_index: usize,
    trace: &mut Vec<crate::simulation::GameEventReceipt>,
) -> Result<(), String> {
    for proc in &request.actors[actor_index].gear_procs {
        if !matches!(proc, GearProcConfig::DustbinBreak { .. }) {
            continue;
        }
        crate::runtime_api::record_hypothesis_id(
            request,
            &mut state.execution_model_assumptions,
            HYPOTHESIS_ID,
        )?;
        if try_grant_break_buff(
            &mut state.buffs,
            actor_index,
            state.mode_clock.relative_us(),
            proc,
        )? {
            trace.push(crate::simulation::GameEventReceipt {
                time_us: state.current_time_us,
                phase: "ReactionOrBreak".into(),
                event_kind: "dustbin_break_intensity_granted".into(),
                active_actor: state.active_actor_index,
                hit_index: None,
                from_actor: None,
                to_actor: Some(actor_index),
                energy_change: None,
                energy_fanout: None,
                data: None,
            });
        }
    }
    Ok(())
}
