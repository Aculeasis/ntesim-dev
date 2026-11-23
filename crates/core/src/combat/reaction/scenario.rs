use super::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TargetProfileInput {
    pub id: String,
    pub position: Point3D,
    pub defense: u64,
    pub resistances: BTreeMap<String, Ratio>,
    pub break_cap: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ActorProfileInput {
    pub actor_index: usize,
    pub character_id: String,
    pub level: u32,
    pub base_break: u64,
    pub break_intensity: Ratio,
    pub energy: u64,
    pub max_energy: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReactionScenarioStep {
    DealAttack {
        source_actor: usize,
        targeting: TargetingRule,
        damage: u64,
        break_shred: u64,
        energy_gain: u64,
        timestamp_us: u64,
    },
    TriggerReaction {
        source_actor: usize,
        target_id: String,
        reaction: ReactionKind,
        timestamp_us: u64,
    },
    AdvanceTime {
        timestamp_us: u64,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReactionScenarioRequest {
    pub content_id: String,
    pub team_size: usize,
    pub actors: Vec<ActorProfileInput>,
    pub targets: Vec<TargetProfileInput>,
    pub break_enabled: bool,
    pub steps: Vec<ReactionScenarioStep>,
}

pub fn evaluate_reaction_scenario(
    request: &ReactionScenarioRequest,
) -> Result<serde_json::Value, String> {
    if request.content_id != "ntesim-current" {
        return Err(format!("unexpected package ID {}", request.content_id));
    }
    if request.team_size == 0 || request.team_size > 4 {
        return Err(format!("invalid team size {}", request.team_size));
    }
    if request.actors.len() != request.team_size {
        return Err(format!(
            "actors count {} does not match team size {}",
            request.actors.len(),
            request.team_size
        ));
    }
    if request.targets.is_empty() {
        return Err("targets cannot be empty".to_string());
    }

    let mut target_profiles = Vec::new();
    for t in &request.targets {
        target_profiles.push(TargetProfile {
            id: t.id.clone(),
            position: t.position,
            defense: t.defense,
            resistances: t.resistances.clone(),
            break_cap: t.break_cap,
            current_break_gauge: t.break_cap,
            break_state: BreakState::Unbroken,
            auras: BTreeMap::new(),
        });
    }

    let mut actor_profiles = Vec::new();
    for a in &request.actors {
        let pool = Pool::new(a.energy, a.max_energy)
            .map_err(|e| format!("energy pool error for actor {}: {e:?}", a.actor_index))?;
        actor_profiles.push(ActorCombatProfile {
            actor_index: a.actor_index,
            character_id: a.character_id.clone(),
            level: a.level,
            base_break: a.base_break,
            break_intensity: a.break_intensity,
            energy_pool: pool,
        });
    }

    let mut engine = ReactionEngine::new(target_profiles, actor_profiles, request.break_enabled)?;

    let mut all_hits = Vec::new();
    let mut all_breaks = Vec::new();
    let mut all_reactions = Vec::new();

    for step in &request.steps {
        match step {
            ReactionScenarioStep::DealAttack {
                source_actor,
                targeting,
                damage,
                break_shred,
                energy_gain,
                timestamp_us,
            } => {
                let (hits, breaks, reactions) = engine.process_attack(
                    *source_actor,
                    targeting,
                    *damage,
                    *break_shred,
                    *energy_gain,
                    *timestamp_us,
                )?;
                all_hits.extend(hits);
                all_breaks.extend(breaks);
                all_reactions.extend(reactions);
            }
            ReactionScenarioStep::TriggerReaction {
                source_actor,
                target_id,
                reaction,
                timestamp_us,
            } => {
                let rec = engine.trigger_reaction(
                    *source_actor,
                    target_id,
                    reaction.clone(),
                    *timestamp_us,
                )?;
                all_reactions.extend(rec);
            }
            ReactionScenarioStep::AdvanceTime { timestamp_us } => {
                let ticks = engine.advance_time(*timestamp_us)?;
                all_reactions.extend(ticks);
            }
        }
    }

    let mut final_energies = Vec::new();
    for a in engine.actors() {
        final_energies.push(a.energy_pool.current());
    }

    let mut final_targets = Vec::new();
    for t in engine.targets().values() {
        final_targets.push(serde_json::json!({
            "id": t.id,
            "break_gauge": t.current_break_gauge,
            "break_state": t.break_state,
        }));
    }

    Ok(serde_json::json!({
        "status": "ok",
        "engine": ntesim_contracts::ENGINE_VERSION,
        "engine_source_sha256": ntesim_contracts::ENGINE_SOURCE_SHA256,
        "content_id": request.content_id,
        "receipt": {
            "break_enabled": request.break_enabled,
            "total_hits": all_hits.len(),
            "hits": all_hits,
            "break_events": all_breaks,
            "reactions": all_reactions,
            "final_energies": final_energies,
            "final_targets": final_targets,
            "last_time_us": engine.last_time_us(),
        },
        "quality": {
            "reaction_model_complete": false,
            "reaction_model_scope": "Explicit trigger research harness; fixed base quantities, latest-applier Scorch stacks; no automatic aura/cycle or intensity-dependent reaction model",
            "combat_adapter_complete": false,
            "game_dps_computed": false,
        }
    }))
}
