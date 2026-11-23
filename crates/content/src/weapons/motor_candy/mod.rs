use crate::HypothesisId;
mod config;
pub(crate) use config::validate_config;
pub(crate) mod projection;
use crate::callbacks::ContentEvent;
use crate::runtime_api::{CallbackContext, CastContext, increment_sequence, record_hypothesis_id};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::{GameActorConfig, GameEventPayload, GameEventReceipt, GearProcConfig};
use ntesim_core::buffs::dispatch::{
    BuffDefinition, BuffRecipient, BuffSourceLifetime, BuffStackingScope, DecayPolicy,
    RefreshPolicy,
};
use ntesim_core::execution::CausalPriority;
use ntesim_core::formula::Ratio;
use std::collections::BTreeMap;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsMotorCandyV1;
const BUFF_ID: &str = "buff_fork_MotorCandy_atk";
const SOURCE: &str = "weapon:fork_MotorCandy";
const MAX_STACKS: u32 = 5;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    generation: u64,
}

fn parameters(config: &GameActorConfig) -> Option<(Ratio, u64)> {
    config.gear_procs.iter().find_map(|proc| match proc {
        GearProcConfig::MotorCandyAtk {
            atk_per_stack,
            interval_us,
        } => Some((*atk_per_stack, *interval_us)),
        _ => None,
    })
}

mod field;
mod timer;
pub(crate) use field::{initialize, on_cast, on_switch};
use timer::schedule;
pub(crate) use timer::tick;

fn receipt(
    ctx: &mut CallbackContext<'_>,
    kind: &str,
    from_actor: Option<usize>,
    to_actor: Option<usize>,
    data: Option<GameEventPayload>,
) {
    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "BuffTickOrExpiry".into(),
        event_kind: kind.into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor,
        to_actor,
        energy_change: None,
        energy_fanout: None,
        data,
    });
}
