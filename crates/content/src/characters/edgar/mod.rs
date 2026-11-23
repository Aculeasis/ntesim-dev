pub(crate) mod awakenings;
pub(crate) mod clock;
pub(crate) mod effects;
mod parameters;
pub(crate) mod preview;
pub(crate) mod secondary;
pub(crate) mod skill;

pub(crate) fn buff_duration(_awakening: u32, ability: &str) -> Option<u64> {
    if ability == "GA_Edgar_UltraSkill" {
        Some(13_000_000)
    } else {
        None
    }
}

pub(crate) fn primary_packet(ability: &str) -> Option<&'static str> {
    match ability {
        "GA_Edgar_QTE" => Some("GE_Player_Edgar_QTE_Damage"),
        "GA_Edgar_Melee" => Some("GE_Player_Edgar_Melee1_Damage"),
        "GA_Edgar_Skill" => Some("GE_Player_Edgar_Skill_Damage"),
        "GA_Edgar_UltraSkill" => Some("GE_Player_Edgar_UltraSkill_Damage"),
        _ => None,
    }
}

pub(crate) fn observe_admission(
    request: &crate::simulation::GameScenarioRequest,
    state: &mut crate::simulation::state::GameSimulatorState,
    actor_index: usize,
    ability: &str,
    metadata: &crate::simulation::metadata::ActionMetadata,
    assumptions: &mut std::collections::BTreeMap<String, String>,
    trace: &mut Vec<crate::simulation::GameEventReceipt>,
) -> Result<(), String> {
    if clock::accepted(request) {
        return effects::observe_support(request, state, actor_index, metadata, assumptions, trace);
    }
    let role = metadata.input_role;
    let own_key = request.actors[actor_index].character_id == "character:1021"
        && matches!(ability, "GA_Edgar_Skill" | "GA_Edgar_QTE");
    let teammate_qte = role == Some(crate::simulation::metadata::InputActionRole::Qte);
    if own_key {
        if state.actors[actor_index].hp == 0 {
            return Ok(());
        }
        crate::runtime_api::record_hypothesis_id(
            request,
            assumptions,
            crate::characters::edgar::skill::KEY_HYPOTHESIS_ID,
        )?;
        let keys = &mut state.actors[actor_index]
            .character
            .edgar_mut()?
            .keys_of_truth;
        let before = *keys;
        *keys = before.saturating_add(1).min(3);
        trace.push(crate::simulation::GameEventReceipt {
            time_us: state.current_time_us,
            phase: "ActionAdmission".into(),
            event_kind: "edgar_key_of_truth_granted".into(),
            active_actor: state.active_actor_index,
            hit_index: None,
            from_actor: Some(actor_index),
            to_actor: Some(actor_index),
            energy_change: None,
            energy_fanout: None,
            data: Some(crate::simulation::GameEventPayload {
                ability_id: Some(ability.into()),
                stacks_before: Some(before),
                stacks: Some(*keys),
                max_stacks: Some(3),
                ..Default::default()
            }),
        });
    }
    if ability == "GA_Edgar_UltraSkill"
        && request.actors[actor_index].character_id == "character:1021"
    {
        let (keys, wake_until) = match &state.actors[actor_index].character {
            crate::characters::RuntimeState::Edgar(edgar) => {
                (edgar.keys_of_truth, edgar.finnegan_wake_until_us)
            }
            _ => return Err("Edgar runtime extension unavailable".into()),
        };
        if keys > 0 {
            crate::runtime_api::record_hypothesis_id(
                request,
                assumptions,
                crate::characters::edgar::skill::KEY_HYPOTHESIS_ID,
            )?;
        }
        trace.push(crate::simulation::GameEventReceipt {
            time_us: state.current_time_us,
            phase: "ActionAdmission".into(),
            event_kind: "edgar_key_of_truth_sampled".into(),
            active_actor: state.active_actor_index,
            hit_index: None,
            from_actor: Some(actor_index),
            to_actor: Some(actor_index),
            energy_change: None,
            energy_fanout: None,
            data: Some(crate::simulation::GameEventPayload {
                stacks: Some(keys),
                max_stacks: Some(3),
                consumed: Some(0),
                expires_at_us: Some(wake_until),
                ..Default::default()
            }),
        });
        return Ok(());
    }
    if !teammate_qte || state.actors[actor_index].hp == 0 {
        return Ok(());
    }
    for (owner_index, owner) in request.actors.iter().enumerate() {
        if owner.character_id != "character:1021"
            || owner_index == actor_index
            || !owner.awakening.has_effect(1)
            || state.actors[owner_index].hp == 0
        {
            continue;
        }
        crate::runtime_api::record_hypothesis_id(
            request,
            assumptions,
            crate::characters::edgar::skill::KEY_HYPOTHESIS_ID,
        )?;
        let keys = &mut state.actors[owner_index]
            .character
            .edgar_mut()?
            .keys_of_truth;
        let before = *keys;
        *keys = before.saturating_add(1).min(3);
        trace.push(crate::simulation::GameEventReceipt {
            time_us: state.current_time_us,
            phase: "ActionAdmission".into(),
            event_kind: "edgar_key_of_truth_granted".into(),
            active_actor: state.active_actor_index,
            hit_index: None,
            from_actor: Some(actor_index),
            to_actor: Some(owner_index),
            energy_change: None,
            energy_fanout: None,
            data: Some(crate::simulation::GameEventPayload {
                reason: Some("teammate_qte".into()),
                stacks_before: Some(before),
                stacks: Some(*keys),
                max_stacks: Some(3),
                ..Default::default()
            }),
        });
    }
    Ok(())
}

// Common.EndAbilitySkill; retrieved configuration, selected timer model.
pub(crate) const CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::Notify {
        progress_us: 3402224,
        montage: "Edgar_UltraSkill",
    };
