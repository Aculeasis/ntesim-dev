use super::model::{
    ActionPhaseKind, ActionPhaseTimings, ActorVitality, CancelType, FieldStatus, InterruptionCause,
    RefundPolicy,
};
use super::team::TeamLifecycle;
use crate::state::Pool;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PoolInput {
    pub current: u64,
    pub capacity: u64,
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum LifecycleStep {
    StartAction {
        actor_index: usize,
        action_id: String,
        timings: ActionPhaseTimings,
        #[serde(default)]
        costs: BTreeMap<String, u64>,
        timestamp_us: u64,
        #[serde(default = "default_true")]
        require_on_field: bool,
    },
    SwitchActor {
        target_index: usize,
        timestamp_us: u64,
    },
    CancelAction {
        actor_index: usize,
        cancel_type: CancelType,
        timestamp_us: u64,
    },
    InterruptAction {
        actor_index: usize,
        cause: InterruptionCause,
        refund_policy: RefundPolicy,
        timestamp_us: u64,
    },
    KillActor {
        actor_index: usize,
        timestamp_us: u64,
    },
    EnterForm {
        actor_index: usize,
        form_id: String,
        duration_us: Option<u64>,
        timestamp_us: u64,
    },
    OpenRecast {
        actor_index: usize,
        base_action: String,
        recast_action: String,
        duration_us: u64,
        max_uses: u32,
        timestamp_us: u64,
    },
    ConsumeRecast {
        actor_index: usize,
        recast_action: String,
        timestamp_us: u64,
    },
    AdvanceTime {
        timestamp_us: u64,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LifecycleScenarioRequest {
    pub content_id: String,
    pub team_size: usize,
    pub characters: Vec<String>,
    pub initial_active: usize,
    pub switch_cooldown_us: u64,
    pub initial_resources: BTreeMap<String, PoolInput>,
    pub steps: Vec<LifecycleStep>,
}

#[derive(Clone, Debug, Serialize)]
pub struct StepExecutionReceipt {
    pub step_index: usize,
    pub timestamp_us: u64,
    pub kind: String,
    pub details: serde_json::Value,
}

#[derive(Clone, Debug, Serialize)]
pub struct ActorFinalState {
    pub actor_index: usize,
    pub character_id: String,
    pub vitality: ActorVitality,
    pub field_status: FieldStatus,
    pub phase: ActionPhaseKind,
    pub active_form: Option<String>,
    pub active_recast: Option<String>,
}

pub fn evaluate_lifecycle_scenario(
    request: &LifecycleScenarioRequest,
) -> Result<serde_json::Value, String> {
    if request.content_id != "ntesim-current" {
        return Err(format!("unexpected package ID {}", request.content_id));
    }
    if request.team_size == 0 || request.team_size > 4 {
        return Err(format!("invalid team size {}", request.team_size));
    }
    if request.characters.len() != request.team_size {
        return Err(format!(
            "character count mismatch: expected {}, got {}",
            request.team_size,
            request.characters.len()
        ));
    }
    if request.initial_active >= request.team_size {
        return Err(format!(
            "initial active actor index out of bounds: {}",
            request.initial_active
        ));
    }

    let mut pools = BTreeMap::new();
    for (name, p) in &request.initial_resources {
        let pool = Pool::new(p.current, p.capacity)
            .map_err(|e| format!("pool error for {name}: {e:?}"))?;
        pools.insert(name.clone(), pool);
    }

    let mut team = TeamLifecycle::new(
        request.team_size,
        request.characters.clone(),
        request.initial_active,
        request.switch_cooldown_us,
        pools,
    )
    .map_err(|e| format!("{e:?}"))?;

    let mut step_receipts = Vec::new();

    for (step_idx, step) in request.steps.iter().enumerate() {
        let receipt = match step {
            LifecycleStep::StartAction {
                actor_index,
                action_id,
                timings,
                costs,
                timestamp_us,
                require_on_field,
            } => {
                team.start_action(
                    *actor_index,
                    action_id.clone(),
                    timings.clone(),
                    costs.clone(),
                    *timestamp_us,
                    *require_on_field,
                )
                .map_err(|e| format!("step {step_idx} (start_action) failed: {e:?}"))?;
                let phase = team.actors[*actor_index].action_state.phase.kind();
                StepExecutionReceipt {
                    step_index: step_idx,
                    timestamp_us: *timestamp_us,
                    kind: "start_action".to_string(),
                    details: serde_json::json!({
                        "actor_index": actor_index,
                        "action_id": action_id,
                        "phase": phase,
                    }),
                }
            }
            LifecycleStep::SwitchActor {
                target_index,
                timestamp_us,
            } => {
                let rec = team
                    .switch_to(*target_index, *timestamp_us)
                    .map_err(|e| format!("step {step_idx} (switch_actor) failed: {e:?}"))?;
                StepExecutionReceipt {
                    step_index: step_idx,
                    timestamp_us: *timestamp_us,
                    kind: "switch_actor".to_string(),
                    details: serde_json::to_value(&rec).unwrap(),
                }
            }
            LifecycleStep::CancelAction {
                actor_index,
                cancel_type,
                timestamp_us,
            } => {
                let rec = team
                    .cancel_action(*actor_index, *cancel_type, *timestamp_us)
                    .map_err(|e| format!("step {step_idx} (cancel_action) failed: {e:?}"))?;
                StepExecutionReceipt {
                    step_index: step_idx,
                    timestamp_us: *timestamp_us,
                    kind: "cancel_action".to_string(),
                    details: serde_json::to_value(&rec).unwrap(),
                }
            }
            LifecycleStep::InterruptAction {
                actor_index,
                cause,
                refund_policy,
                timestamp_us,
            } => {
                let rec = team
                    .interrupt_action(*actor_index, *cause, refund_policy, *timestamp_us)
                    .map_err(|e| format!("step {step_idx} (interrupt_action) failed: {e:?}"))?;
                StepExecutionReceipt {
                    step_index: step_idx,
                    timestamp_us: *timestamp_us,
                    kind: "interrupt_action".to_string(),
                    details: serde_json::to_value(&rec).unwrap(),
                }
            }
            LifecycleStep::KillActor {
                actor_index,
                timestamp_us,
            } => {
                let rec = team
                    .kill_actor(*actor_index, *timestamp_us)
                    .map_err(|e| format!("step {step_idx} (kill_actor) failed: {e:?}"))?;
                StepExecutionReceipt {
                    step_index: step_idx,
                    timestamp_us: *timestamp_us,
                    kind: "kill_actor".to_string(),
                    details: serde_json::json!({
                        "actor_index": actor_index,
                        "switch_record": rec,
                    }),
                }
            }
            LifecycleStep::EnterForm {
                actor_index,
                form_id,
                duration_us,
                timestamp_us,
            } => {
                team.enter_form(*actor_index, form_id.clone(), *duration_us, *timestamp_us)
                    .map_err(|error| format!("{error:?}"))?;
                StepExecutionReceipt {
                    step_index: step_idx,
                    timestamp_us: *timestamp_us,
                    kind: "enter_form".to_string(),
                    details: serde_json::json!({
                        "actor_index": actor_index,
                        "form_id": form_id,
                        "duration_us": duration_us,
                    }),
                }
            }
            LifecycleStep::OpenRecast {
                actor_index,
                base_action,
                recast_action,
                duration_us,
                max_uses,
                timestamp_us,
            } => {
                team.open_recast(
                    *actor_index,
                    base_action.clone(),
                    recast_action.clone(),
                    *duration_us,
                    *max_uses,
                    *timestamp_us,
                )
                .map_err(|error| format!("{error:?}"))?;
                StepExecutionReceipt {
                    step_index: step_idx,
                    timestamp_us: *timestamp_us,
                    kind: "open_recast".to_string(),
                    details: serde_json::json!({
                        "actor_index": actor_index,
                        "base_action": base_action,
                        "recast_action": recast_action,
                        "duration_us": duration_us,
                        "max_uses": max_uses,
                    }),
                }
            }
            LifecycleStep::ConsumeRecast {
                actor_index,
                recast_action,
                timestamp_us,
            } => {
                let rem = team
                    .consume_recast(*actor_index, recast_action, *timestamp_us)
                    .map_err(|e| format!("step {step_idx} (consume_recast) failed: {e:?}"))?;
                StepExecutionReceipt {
                    step_index: step_idx,
                    timestamp_us: *timestamp_us,
                    kind: "consume_recast".to_string(),
                    details: serde_json::json!({
                        "actor_index": actor_index,
                        "recast_action": recast_action,
                        "remaining_uses": rem,
                    }),
                }
            }
            LifecycleStep::AdvanceTime { timestamp_us } => {
                let transitions = team
                    .advance_time(*timestamp_us)
                    .map_err(|e| format!("step {step_idx} (advance_time) failed: {e:?}"))?;
                StepExecutionReceipt {
                    step_index: step_idx,
                    timestamp_us: *timestamp_us,
                    kind: "advance_time".to_string(),
                    details: serde_json::json!({
                        "transitions": transitions,
                    }),
                }
            }
        };
        step_receipts.push(receipt);
    }

    let final_now = team.last_time_us;
    let mut actor_states = Vec::new();
    for actor in &team.actors {
        let field_status = team
            .field_status(actor.actor_index)
            .map_err(|e| format!("{e:?}"))?;
        let active_form = actor.form_state.active_form(final_now).map(String::from);
        let active_recast = actor
            .recast_state
            .active_recast()
            .map(|r| r.recast_action.clone());
        actor_states.push(ActorFinalState {
            actor_index: actor.actor_index,
            character_id: actor.character_id.clone(),
            vitality: team.field.vitality[actor.actor_index],
            field_status,
            phase: actor.action_state.phase.kind(),
            active_form,
            active_recast,
        });
    }

    let mut final_resources = BTreeMap::new();
    for (name, pool) in &team.resources {
        final_resources.insert(
            name.clone(),
            PoolInput {
                current: pool.current(),
                capacity: pool.capacity(),
            },
        );
    }

    Ok(serde_json::json!({
        "status": "ok",
        "engine": ntesim_contracts::ENGINE_VERSION,
        "engine_source_sha256": ntesim_contracts::ENGINE_SOURCE_SHA256,
        "content_id": request.content_id,
        "receipt": {
            "initial_active": request.initial_active,
            "switch_cooldown_us": request.switch_cooldown_us,
            "total_steps": request.steps.len(),
            "step_receipts": step_receipts,
            "final_active": team.field.active_actor(),
            "actors": actor_states,
            "resources": final_resources,
            "last_time_us": team.last_time_us,
        },
        "quality": {
            "lifecycle_model_complete": true,
            "combat_adapter_complete": false,
            "game_dps_computed": false,
        }
    }))
}
