use super::execution::PreparedExecution;
use crate::simulation::{GameActorParryEvent, GameScenarioRequest};
use std::collections::BTreeMap;

pub(super) fn index(request: &GameScenarioRequest) -> BTreeMap<usize, usize> {
    request
        .parry_events
        .iter()
        .enumerate()
        .filter_map(|(index, event)| match event {
            GameActorParryEvent::AfterAction { command_index } => Some((*command_index, index)),
            _ => None,
        })
        .collect()
}

impl PreparedExecution {
    pub(super) fn register_action_parry(
        &mut self,
        command: usize,
        event: usize,
    ) -> Result<(), String> {
        match self.action_parries.entry(command) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(event);
                Ok(())
            }
            std::collections::btree_map::Entry::Occupied(_) => {
                Err("Duplicate action-bound parry.".into())
            }
        }
    }

    pub(super) fn queue_action_parry(&mut self, command: usize) -> Result<(), String> {
        if let Some(event_index) = self.action_parries.remove(&command) {
            self.seq = self
                .seq
                .checked_add(1)
                .ok_or("parry event sequence overflow")?;
            self.queue
                .push(crate::simulation::events::ScheduledGameEvent {
                    time_us: self.state.current_time_us,
                    priority: crate::execution::CausalPriority::ProcCallback,
                    seq: self.seq,
                    data: crate::simulation::events::GameEventData::ActorParry { event_index },
                });
        }
        Ok(())
    }

    pub(super) fn parry_switch_admission(
        &mut self,
        request: &GameScenarioRequest,
        command: usize,
        incoming: usize,
        entry: Option<&crate::runtime_api::OwnerEntryAction>,
    ) -> bool {
        if !self.action_parries.contains_key(&command) {
            return true;
        }
        let ready = self.state.active_actor_index != incoming
            && crate::characters::daffodil::phantom::entry_parry_charge(
                &self.state.actors[incoming],
                self.state.current_time_us,
            )
            && match entry {
                Some(crate::runtime_api::OwnerEntryAction::Attack(_)) => true,
                Some(crate::runtime_api::OwnerEntryAction::Support(entry)) => {
                    crate::simulation::availability::ability(
                        request,
                        &self.state,
                        incoming,
                        entry.ability_id,
                        entry.damage_effect_id,
                        0,
                    )
                    .ready
                }
                _ => false,
            };
        if ready {
            return true;
        }
        let message = "Assumed entry parry requires a live charge and an eligible distinct entry."
            .to_string();
        self.rejected_action = Some(crate::simulation::GameRejectedAction {
            command_index: command,
            time_us: self.state.current_time_us,
            actor_index: self.state.active_actor_index,
            reason: crate::simulation::GameAdmissionReason::Other {
                message: message.clone(),
            },
        });
        self.halt_diagnostic = Some(message);
        self.success = false;
        self.execution_completed = false;
        false
    }
}
