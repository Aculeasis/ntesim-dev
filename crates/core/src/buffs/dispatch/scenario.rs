use super::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PoolInput {
    pub current: u64,
    pub capacity: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DispatchStep {
    ApplyBuff {
        buff: BuffDefinition,
        owner_actor: usize,
        timestamp_us: u64,
    },
    RegisterCallback {
        callback: CallbackRegistration,
    },
    EmitEvent {
        event: CombatEvent,
        timestamp_us: u64,
    },
    SwitchActor {
        target_index: usize,
        timestamp_us: u64,
    },
    AdvanceTime {
        timestamp_us: u64,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DispatchScenarioRequest {
    pub content_id: String,
    pub team_size: usize,
    pub initial_active: usize,
    pub initial_resources: BTreeMap<String, PoolInput>,
    pub steps: Vec<DispatchStep>,
}

pub fn evaluate_dispatch_scenario(
    request: &DispatchScenarioRequest,
) -> Result<serde_json::Value, String> {
    if request.content_id != "ntesim-current" {
        return Err(format!("unexpected package ID {}", request.content_id));
    }
    if request.team_size == 0 || request.team_size > 4 {
        return Err(format!("invalid team size {}", request.team_size));
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

    let mut engine = DispatchEngine::new(request.team_size, request.initial_active, pools)
        .map_err(|error| format!("{error:?}"))?;
    let mut step_receipts = Vec::new();

    for (step_idx, step) in request.steps.iter().enumerate() {
        let rec = match step {
            DispatchStep::ApplyBuff {
                buff,
                owner_actor,
                timestamp_us,
            } => {
                engine
                    .apply_buff(buff.clone(), *owner_actor, *timestamp_us)
                    .map_err(|e| format!("step {step_idx} apply buff failed: {e:?}"))?;
                serde_json::json!({
                    "step_index": step_idx,
                    "kind": "apply_buff",
                    "buff_id": buff.id,
                    "owner_actor": owner_actor,
                    "timestamp_us": timestamp_us,
                })
            }
            DispatchStep::RegisterCallback { callback } => {
                engine
                    .register_callback(callback.clone())
                    .map_err(|e| format!("step {step_idx} register callback failed: {e:?}"))?;
                serde_json::json!({
                    "step_index": step_idx,
                    "kind": "register_callback",
                    "callback_id": callback.id,
                })
            }
            DispatchStep::EmitEvent {
                event,
                timestamp_us,
            } => {
                let receipt = engine
                    .dispatch(event.clone(), *timestamp_us)
                    .map_err(|e| format!("step {step_idx} dispatch failed: {e:?}"))?;
                serde_json::json!({
                    "step_index": step_idx,
                    "kind": "emit_event",
                    "receipt": receipt,
                })
            }
            DispatchStep::SwitchActor {
                target_index,
                timestamp_us,
            } => {
                engine
                    .advance_time(*timestamp_us)
                    .map_err(|e| format!("step {step_idx} advance time failed: {e:?}"))?;
                if *target_index >= engine.team_size {
                    return Err(format!("target actor out of bounds: {target_index}"));
                }
                engine
                    .dispatch(
                        CombatEvent::ActorSwitched {
                            from_index: engine.active_actor,
                            to_index: *target_index,
                        },
                        *timestamp_us,
                    )
                    .map_err(|error| format!("step {step_idx}: {error:?}"))?;
                serde_json::json!({
                    "step_index": step_idx,
                    "kind": "switch_actor",
                    "new_active": target_index,
                    "timestamp_us": timestamp_us,
                })
            }
            DispatchStep::AdvanceTime { timestamp_us } => {
                engine
                    .advance_time(*timestamp_us)
                    .map_err(|e| format!("step {step_idx} advance time failed: {e:?}"))?;
                serde_json::json!({
                    "step_index": step_idx,
                    "kind": "advance_time",
                    "timestamp_us": timestamp_us,
                })
            }
        };
        step_receipts.push(rec);
    }

    let final_now = engine.last_time_us;
    let mut actor_buff_mods = Vec::new();
    for i in 0..engine.team_size {
        let mods = engine
            .buffs
            .get_stat_modifiers_for_actor(i, engine.active_actor, final_now)
            .map_err(|e| format!("stat modifiers failed: {e:?}"))?;
        actor_buff_mods.push(mods);
    }

    let mut final_resources = BTreeMap::new();
    for (name, pool) in &engine.resources {
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
            "final_active": engine.active_actor,
            "total_steps": request.steps.len(),
            "step_receipts": step_receipts,
            "active_buff_count": engine.buffs.active_buffs.len(),
            "actor_stat_modifiers": actor_buff_mods,
            "resources": final_resources,
            "last_time_us": engine.last_time_us,
        },
        "quality": {
            "dispatch_model_complete": true,
            "combat_adapter_complete": false,
            "game_dps_computed": false,
        }
    }))
}
