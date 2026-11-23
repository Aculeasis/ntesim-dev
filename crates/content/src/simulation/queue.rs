use super::events::{GameEventData, ScheduledGameEvent};
use super::{GameActionCommand, GameScenarioRequest};
use crate::execution::CausalPriority;
use crate::simulation::timer_queue::EventQueue as BinaryHeap;

pub(super) fn initial_queue(
    request: &GameScenarioRequest,
    branches: &[super::GameLiveBranch],
    energy_waits: &[super::GameLiveEnergyWait],
) -> (BinaryHeap, u64, Vec<ScheduledGameEvent>) {
    let mut queue = BinaryHeap::new();
    let mut parked_actions = Vec::new();
    let mut seq = 0u64;
    let parked_after_action_index = energy_waits.first().map(|wait| wait.after_action_index);
    let action_priority = match request.action_admission_order {
        super::GameActionAdmissionOrder::Scheduled => CausalPriority::ActionAdmission,
        super::GameActionAdmissionOrder::AfterSettledEvents => CausalPriority::Observation,
    };

    for (cmd_idx, cmd) in request.actions.iter().enumerate() {
        seq += 1;
        match cmd {
            GameActionCommand::InvokeAbility { timestamp_us, .. }
            | GameActionCommand::TargetGauge { timestamp_us, .. }
            | GameActionCommand::PlayerInput { timestamp_us, .. } => {
                let event = ScheduledGameEvent {
                    time_us: *timestamp_us,
                    priority: action_priority,
                    seq,
                    data: GameEventData::ActionAdmission {
                        command_index: cmd_idx,
                    },
                };
                if parked_after_action_index.is_some_and(|index| cmd_idx >= index) {
                    parked_actions.push(event);
                } else {
                    queue.push(event);
                }
            }
            GameActionCommand::SwitchActor {
                timestamp_us,
                target_actor_index,
                cooldown_us,
            } => {
                seq += 1;
                let event = ScheduledGameEvent {
                    time_us: *timestamp_us,
                    priority: action_priority,
                    seq,
                    data: GameEventData::SwitchActor {
                        command_index: cmd_idx,
                        target_actor_index: *target_actor_index,
                        cooldown_us: *cooldown_us,
                    },
                };
                if parked_after_action_index.is_some_and(|index| cmd_idx >= index) {
                    parked_actions.push(event);
                } else {
                    queue.push(event);
                }
            }
            GameActionCommand::Wait { .. } => {}
        }
    }
    for (event_index, event) in request.dummy_combat_events.iter().enumerate() {
        seq += 1;
        queue.push(ScheduledGameEvent {
            time_us: event.timestamp_us(),
            priority: match event {
                super::GameDummyCombatEvent::Proximity { .. } => CausalPriority::ModePause,
                super::GameDummyCombatEvent::AttackStart { .. } => CausalPriority::ProcCallback,
            },
            seq,
            data: GameEventData::DummyCombat { event_index },
        });
    }
    for (event_index, event) in request.incoming_damage_events.iter().enumerate() {
        seq += 1;
        queue.push(ScheduledGameEvent {
            time_us: event.timestamp_us,
            priority: CausalPriority::HitDelivery,
            seq,
            data: GameEventData::IncomingDamage {
                event_index,
                actor_index: event.actor_index,
                delivered_damage: event.delivered_damage,
            },
        });
    }
    for (event_index, event) in request.hp_cost_events.iter().enumerate() {
        seq += 1;
        queue.push(ScheduledGameEvent {
            time_us: event.timestamp_us,
            priority: CausalPriority::HitDelivery,
            seq,
            data: GameEventData::HpCost { event_index },
        });
    }
    for (event_index, event) in request.interrupt_events.iter().enumerate() {
        seq += 1;
        queue.push(ScheduledGameEvent {
            time_us: event.timestamp_us,
            priority: CausalPriority::SettlementAndDeath,
            seq,
            data: GameEventData::ActorInterrupt { event_index },
        });
    }
    for (event_index, event) in request.parry_events.iter().enumerate() {
        let super::GameActorParryEvent::At { timestamp_us, .. } = event else {
            continue;
        };
        seq += 1;
        queue.push(ScheduledGameEvent {
            time_us: *timestamp_us,
            priority: CausalPriority::ProcCallback,
            seq,
            data: GameEventData::ActorParry { event_index },
        });
    }
    for (event_index, event) in request.combat_transition_events.iter().enumerate() {
        seq += 1;
        queue.push(ScheduledGameEvent {
            time_us: event.timestamp_us,
            priority: CausalPriority::ProcCallback,
            seq,
            data: GameEventData::CombatTransition { event_index },
        });
    }
    for (branch_index, branch) in branches.iter().enumerate() {
        seq += 1;
        let event = ScheduledGameEvent {
            time_us: branch.timestamp_us,
            priority: CausalPriority::Observation,
            seq,
            data: GameEventData::LiveBranch { branch_index },
        };
        if parked_after_action_index.is_some_and(|index| branch.after_action_index >= index) {
            parked_actions.push(event);
        } else {
            queue.push(event);
        }
    }
    for (wait_index, wait) in energy_waits.iter().take(1).enumerate() {
        seq += 1;
        queue.push(ScheduledGameEvent {
            time_us: wait.timestamp_us,
            priority: CausalPriority::Observation,
            seq,
            data: GameEventData::LiveEnergyWait { wait_index },
        });
    }
    (queue, seq, parked_actions)
}
