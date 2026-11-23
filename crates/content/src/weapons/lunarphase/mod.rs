use crate::HypothesisId;
mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod projection;
use crate::callbacks::ContentEvent;
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::metadata::InputActionRole;
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GameEventPayload, GameEventReceipt, GearProcConfig};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope};
use ntesim_core::formula::Ratio;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsLunarphaseV1;

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    for proc in &ctx.config.gear_procs {
        if let GearProcConfig::LunarPhase {
            cosmos_bonus,
            def_ignore,
            duration_us,
            ..
        } = proc
        {
            ctx.record_hypothesis(HYPOTHESIS_ID)?;
            if ctx.action_metadata.input_role == Some(InputActionRole::Ultimate) {
                crate::gear::schedule_cast_buff(
                    ctx,
                    ContentEvent::LunarPhaseProcGrant {
                        actor_index: ctx.actor_index,
                        cosmos_bonus: *cosmos_bonus,
                        def_ignore: *def_ignore,
                        duration_us: *duration_us,
                    },
                )?;
            }
        }
    }
    Ok(())
}

pub(crate) fn apply(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    cosmos_bonus: Ratio,
    def_ignore: Ratio,
    duration_us: u64,
) -> Result<(), String> {
    let state = &mut *ctx.state;
    let trace = &mut *ctx.trace;
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);

    let cosmos_bonus_exact = ExactFraction::from_ratio(cosmos_bonus)?;
    let def_ignore_exact = ExactFraction::from_ratio(def_ignore)?;

    state.apply_buff(
        actor_index,
        TimedBuffDeclaration::reset(
            "lunarphase.cosmos_and_def_ignore",
            "weapon:fork_lunarphase",
            BuffRecipient::SelfActor,
            duration_us,
            BuffStackingScope::PerSource,
            std::collections::BTreeMap::from([
                (
                    "damage_bonus:ECharacterElementType::CHARACTER_ELEMENT_TYPE_COSMOS".into(),
                    cosmos_bonus_exact,
                ),
                ("def_ignore".into(), def_ignore_exact),
            ]),
        ),
    )?;

    trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "BuffTickOrExpiry".into(),
        event_kind: "lunarphase_proc".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: Some(actor_index),
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            damage_bonus: Some(cosmos_bonus_exact),
            def_ignore: Some(def_ignore_exact),
            duration_us: Some(duration_us),
            ..Default::default()
        }),
    });
    Ok(())
}
