//! Observe actual HP/MaxHP settlements at existing causal events, never by polling.
use super::{GameCausalTrigger, events::GameEventData};
use crate::runtime_api::CallbackContext;

#[derive(Clone, Copy, Debug)]
pub(crate) struct CombatHpLoss {
    pub(crate) hp_before: u64,
    pub(crate) hp_after: u64,
    pub(crate) max_hp: u64,
}

#[derive(Clone)]
pub(crate) struct HpChange {
    pub(crate) actor_index: usize,
    pub(crate) before: u64,
    /// Explicit damage/cost settlements; structural clamps and healing add none.
    pub(crate) combat_losses: Vec<CombatHpLoss>,
    pub(crate) after: u64,
}

pub(crate) struct Snapshot(Vec<(u64, u64)>);

impl Snapshot {
    pub(crate) fn capture(state: &super::state::GameSimulatorState) -> Result<Self, String> {
        state
            .actors
            .iter()
            .enumerate()
            .map(|(index, actor)| {
                Ok((
                    actor.hp,
                    super::state::effective_hp_max(
                        &state.buffs,
                        actor.hp_max_base,
                        &actor.hp_max_static_bonus,
                        index,
                        actor.hp_max,
                    )?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()
            .map(Self)
    }

    pub(crate) fn settle(
        self,
        ctx: &mut CallbackContext<'_>,
        trigger: GameCausalTrigger,
    ) -> Result<(), String> {
        let equality_actor = match trigger {
            GameCausalTrigger::IncomingDamage { event_index } => {
                let actor = ctx.request.incoming_damage_events[event_index].actor_index;
                if actor == usize::MAX {
                    Some(ctx.state.active_actor_index)
                } else {
                    Some(actor)
                }
            }
            GameCausalTrigger::HpCost { event_index } => {
                Some(ctx.request.hp_cost_events[event_index].actor_index)
            }
            _ => None,
        };
        let mut changes = Vec::new();
        for (index, (before, old_max)) in self.0.into_iter().enumerate() {
            let actor = &mut ctx.state.actors[index];
            let combat_losses = actor
                .combat_hp_losses
                .as_mut()
                .map(std::mem::take)
                .unwrap_or_default();
            let max = super::state::effective_hp_max(
                &ctx.state.buffs,
                actor.hp_max_base,
                &actor.hp_max_static_bonus,
                index,
                actor.hp_max,
            )?;
            if before != actor.hp
                || old_max != max
                || equality_actor == Some(index)
                || !combat_losses.is_empty()
            {
                changes.push(HpChange {
                    actor_index: index,
                    before,
                    combat_losses,
                    after: actor.hp,
                });
            }
        }
        if !changes.is_empty() {
            crate::registry::on_vitals_changed(ctx, &changes, trigger)?;
        }
        Ok(())
    }
}

pub(crate) fn trigger(event: &GameEventData, sequence: u64) -> GameCausalTrigger {
    match event {
        GameEventData::ActionAdmission { command_index } => GameCausalTrigger::Action {
            command_index: *command_index,
        },
        GameEventData::SwitchActor { command_index, .. } => GameCausalTrigger::FieldEntry {
            command_index: *command_index,
        },
        GameEventData::IncomingDamage { event_index, .. } => GameCausalTrigger::IncomingDamage {
            event_index: *event_index,
        },
        GameEventData::HpCost { event_index } => GameCausalTrigger::HpCost {
            event_index: *event_index,
        },
        _ => GameCausalTrigger::VitalChange {
            event_sequence: sequence,
        },
    }
}
