use crate::HypothesisId;
use crate::runtime_api::CallbackContext;
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use ntesim_core::execution::CausalPriority;
use ntesim_core::formula::Ratio;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::AureliaHarmonicsB2V1;
pub(crate) const PACKET: &str = "GE_Reaction_4_new_1070_Damage";

pub(crate) fn schedule_harmonics_if_eligible(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    target_index: usize,
    command_index: usize,
) -> Result<(), String> {
    if ctx.state.targets[target_index].hp_current == 0 || ctx.state.actors[actor_index].hp == 0 {
        return Ok(());
    }
    let request = ctx.request;
    if !request
        .model_assumptions
        .contains_key(HYPOTHESIS_ID.as_str())
    {
        return Ok(());
    }

    let config = &request.actors[actor_index];

    let progression = ntesim_data::skill_levels::progression(
        &config.character_id,
        config.character_level,
        config.character_max_level,
    )?;
    if progression["breakthrough"]
        .as_u64()
        .ok_or("Aurelia breakthrough missing")?
        < 2
    {
        return Ok(());
    }

    let now_us = ctx.state.current_time_us;
    let command_key = ctx.state.origins.register(
        request,
        crate::simulation::GameAutonomousEventReceipt {
            actor_index,
            active_actor_index: ctx.state.active_actor_index,
            ability_id: "GA_Mitsuki070_Passive_1".into(),
            time_us: now_us,
            trigger: crate::simulation::GameCausalTrigger::Action { command_index },
            recipient_healing: None,
        },
    )?;
    for offset_us in [300_000, 600_000, 900_000] {
        let hit_time_us = now_us
            .checked_add(offset_us)
            .ok_or("Harmonics hit time overflow")?;
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: hit_time_us,
            priority: CausalPriority::BuffTickOrExpiry,
            seq: *ctx.sequence,
            data: GameEventData::Content(crate::callbacks::ContentEvent::AureliaHarmonicsHit {
                actor_index,
                target_index,
                command_key,
            }),
        });
    }

    Ok(())
}

pub(crate) fn deliver_harmonics_hit(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    target_index: usize,
    command_key: usize,
) -> Result<(), String> {
    if ctx.state.actors[actor_index].hp == 0 {
        return Ok(());
    }
    if ctx.state.targets[target_index].hp_current == 0 {
        return Ok(());
    }

    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        HYPOTHESIS_ID,
    )?;

    let native = ntesim_data::native_action_metadata::resolve_damage_effect(
        &ctx.request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        PACKET,
    )?;
    let packet = crate::simulation::metadata::resolve_packet_metadata(
        &native,
        &ctx.request.model_assumptions,
    )?;
    let provenance = crate::simulation::metadata::MetadataProvenance::AcceptedModelHypothesis {
        id: HYPOTHESIS_ID.as_str().into(),
        description: HYPOTHESIS_ID.description().into(),
    };
    let action = crate::simulation::metadata::ActionMetadata {
        categories: std::collections::BTreeSet::from([
            crate::simulation::metadata::ActionCategory::Other,
        ]),
        gameplay_tags: Vec::new(),
        ability_id: "GA_Mitsuki070_Passive_1".into(),
        native_ability_index: None,
        input_role: Some(crate::simulation::metadata::InputActionRole::Other),
        input_role_provenance: Some(provenance.clone()),
        primary_slot: None,
        source_evidence: None,
        identity_evidence: None,
        provenance: provenance.clone(),
    };
    let metadata = crate::simulation::metadata::ActionPacketMetadata {
        action,
        packet,
        binding: crate::simulation::metadata::ActionPacketBinding {
            kind: crate::simulation::metadata::ActionPacketBindingKind::OwnerChild,
            initiating_ability_id: "GA_Mitsuki070_Passive_1".into(),
            packet_ga_name: native.ga_name.clone(),
            provenance: Some(provenance),
            energy_group_id: None,
        },
    };

    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: ctx.state.current_time_us,
        priority: CausalPriority::HitDelivery,
        seq: *ctx.sequence,
        data: GameEventData::HitDelivery {
            command_index: command_key,
            actor_index,
            target_index,
            ability_id: "GA_Mitsuki070_Passive_1".into(),
            damage_effect_id: PACKET.into(),
            atk_rate: Ratio {
                numerator: 1,
                denominator: 2,
            },
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            metadata: Box::new(metadata),
        },
    });

    Ok(())
}
