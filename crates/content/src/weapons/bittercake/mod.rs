use crate::HypothesisId;
mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod projection;
use crate::runtime_api::CallbackContext;
use crate::simulation::{ExactFraction, GameEventPayload, GameEventReceipt, GearProcConfig};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope};

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsBittercakeDamageEventV1;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) next_trigger_us: Option<u64>,
}

pub(crate) fn on_damage_received(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    actual_hp_loss: u64,
) -> Result<(), String> {
    if actual_hp_loss == 0 {
        return Ok(());
    }
    let Some((def_bonus, duration_us, cooldown_us)) = ctx.request.actors[actor_index]
        .gear_procs
        .iter()
        .find_map(|proc| match proc {
            GearProcConfig::BitterCake {
                def_bonus,
                duration_us,
                cooldown_us,
            } => Some((*def_bonus, *duration_us, *cooldown_us)),
            _ => None,
        })
    else {
        return Ok(());
    };
    if ctx.state.actors[actor_index]
        .equipment
        .bittercake
        .next_trigger_us
        .is_some_and(|ready_at| ctx.state.current_time_us < ready_at)
    {
        return Ok(());
    }

    let ready_at = ctx
        .state
        .current_time_us
        .checked_add(cooldown_us)
        .ok_or("Bittercake cooldown deadline overflow")?;
    ctx.state.apply_buff(
        actor_index,
        crate::simulation::state::TimedBuffDeclaration::reset(
            "bittercake.defense",
            "weapon:fork_BitterCake",
            BuffRecipient::SelfActor,
            duration_us,
            BuffStackingScope::PerSource,
            std::collections::BTreeMap::from([(
                "def_percent".into(),
                ExactFraction::from_ratio(def_bonus)?,
            )]),
        ),
    )?;
    ctx.state.actors[actor_index]
        .equipment
        .bittercake
        .next_trigger_us = Some(ready_at);
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        HYPOTHESIS_ID,
    )?;
    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "HitDelivery".into(),
        event_kind: "bittercake_defense".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: Some(actor_index),
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            def_percent: Some(ExactFraction::from_ratio(def_bonus)?),
            duration_us: Some(duration_us),
            ..Default::default()
        }),
    });
    Ok(())
}
