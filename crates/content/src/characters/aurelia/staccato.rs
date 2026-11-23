use crate::HypothesisId;
use crate::runtime_api::CastContext;
use crate::simulation::ExactFraction;
use ntesim_core::buffs::dispatch::BuffRecipient;
use std::collections::BTreeMap;

pub(crate) const STACCATO_HYPOTHESIS_ID: HypothesisId = HypothesisId::AureliaStaccatoV1;
pub(crate) const PERFECT_DODGE_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::AureliaStaccatoPerfectDodgeV1;

pub(crate) const EVADE_BRANCH_PACKET: &str = "GE_Player_Mitsuki_EvadeBranch_Damage";
pub(crate) const EVADE_BRANCH_POWER_PACKET: &str = "GE_Player_Mitsuki_EvadeBranch_Power_Damage";
pub(crate) const PERFECT_BULLET_PACKET: &str = "GE_Player_Mitsuki_PerfectAtkBullet_Damage";
pub(crate) const PERFECT_BULLET_POWER_PACKET: &str =
    "GE_Player_Mitsuki_PerfectAtkBullet_Power_Damage";

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<bool, String> {
    let packet_id = ctx
        .packet_metadata
        .map(|p| p.damage_effect_id.as_str())
        .unwrap_or_default();

    if packet_id != EVADE_BRANCH_PACKET && packet_id != EVADE_BRANCH_POWER_PACKET {
        return Ok(false);
    }

    if ctx
        .request
        .model_assumptions
        .contains_key(STACCATO_HYPOTHESIS_ID.as_str())
    {
        ctx.record_hypothesis(STACCATO_HYPOTHESIS_ID)?;
        ctx.cancel_primary_delivery()?;

        let cadenza_until = {
            let state = ctx.actor.character.aurelia_mut()?;
            state.cadenza_until_us
        };

        // 4 loop blasts in the 2-second Staccato ride:
        // Progress coordinates: 1.197229, 1.3372288, 1.7264841, 1.8617061
        for offset_us in [1_197_229, 1_337_229, 1_726_484, 1_861_706] {
            let blast_time_us = ctx
                .time_us
                .checked_add(offset_us)
                .ok_or("Staccato blast time overflow")?;
            let is_powered = cadenza_until > blast_time_us;
            let packet = if is_powered {
                EVADE_BRANCH_POWER_PACKET
            } else {
                EVADE_BRANCH_PACKET
            };
            if is_powered {
                ctx.schedule_native_aoe_packet(packet, blast_time_us)?;
            } else {
                ctx.schedule_native_child_packet(packet, ctx.target_index, blast_time_us)?;
            }
        }

        // A4: +20% base DEF while Evading (in Staccato)
        if ctx.config.awakening.has_effect(4) {
            ctx.refresh_buff(
                "aurelia.a4_def",
                BuffRecipient::SelfActor,
                2_000_000,
                BTreeMap::from([("def_percent".into(), ExactFraction::new(20, 100)?)]),
            )?;
        }

        let state = ctx.actor.character.aurelia_mut()?;
        state.in_staccato = true;
        state.staccato_entered_us = ctx.time_us;

        return Ok(true);
    }

    Ok(false)
}

pub(crate) fn handle_staccato_critical_dodge(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if !ctx
        .request
        .model_assumptions
        .contains_key(PERFECT_DODGE_HYPOTHESIS_ID.as_str())
    {
        return Ok(());
    }

    let in_staccato = {
        let state = ctx.actor.character.aurelia_mut()?;
        state.is_in_staccato(ctx.time_us)
    };
    if !in_staccato {
        return Ok(());
    }

    ctx.record_hypothesis(PERFECT_DODGE_HYPOTHESIS_ID)?;

    let has_cadenza = {
        let state = ctx.actor.character.aurelia_mut()?;
        state.cadenza_until_us > ctx.time_us
    };

    let bullet_packet = if has_cadenza {
        PERFECT_BULLET_POWER_PACKET
    } else {
        PERFECT_BULLET_PACKET
    };

    let targets = if has_cadenza {
        ctx.targets
            .iter()
            .enumerate()
            .filter_map(|(idx, t)| (t.hp_current > 0).then_some(idx))
            .collect::<Vec<_>>()
    } else {
        vec![0]
    };

    let array_index: usize = if let Some(idx) = ctx.native_array_index {
        idx
    } else {
        let progression = ntesim_data::skill_levels::progression(
            &ctx.config.character_id,
            ctx.config.character_level,
            ctx.config.character_max_level,
        )?;
        let breakthrough: u32 = progression["breakthrough"]
            .as_u64()
            .ok_or("breakthrough missing")?
            .try_into()
            .map_err(|_| "breakthrough exceeds u32")?;
        let level = crate::build::preview_projection::effective_skill_level(
            ntesim_data::skill_levels::frozen::tables(),
            &ctx.config.character_id,
            "GA_Mitsuki_Melee",
            ctx.config
                .talent_levels
                .map_or(ctx.config.purchased_skill_level, |l| l[0]),
            breakthrough,
            &ctx.config.awakening,
        )?;
        level["array_index"]
            .as_u64()
            .ok_or("normal coordinate missing")?
            .try_into()
            .map_err(|_| "coordinate exceeds usize")?
    };

    let projection = crate::build::project_native_packet_coefficient(
        ntesim_data::skill_levels::frozen::tables(),
        &crate::build::PacketCoefficientRequest {
            content_id: &ctx.request.content_id,
            character_id: &ctx.config.character_id,
            ability_id: "GA_Mitsuki_Melee",
            damage_effect_id: bullet_packet,
            array_index: array_index
                .try_into()
                .map_err(|_| "array index exceeds u32")?,
            research_metadata: None,
            model_assumptions: &ctx.request.model_assumptions,
        },
    )?;
    let atk_rate: ntesim_core::formula::Ratio =
        serde_json::from_value(projection["compiled_rate"].clone())
            .map_err(|e| format!("rate error: {e}"))?;
    let scaling_stat = serde_json::from_value(projection["compiled_scaling_stat"].clone())
        .map_err(|e| format!("stat error: {e}"))?;

    let native = ntesim_data::native_action_metadata::resolve_damage_effect(
        &ctx.request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        bullet_packet,
    )?;
    let packet = crate::simulation::metadata::resolve_packet_metadata(
        &native,
        &ctx.request.model_assumptions,
    )?;
    let provenance = crate::simulation::metadata::MetadataProvenance::AcceptedModelHypothesis {
        id: PERFECT_DODGE_HYPOTHESIS_ID.as_str().into(),
        description: PERFECT_DODGE_HYPOTHESIS_ID.description().into(),
    };
    let action = crate::simulation::metadata::ActionMetadata {
        categories: std::collections::BTreeSet::from([
            crate::simulation::metadata::ActionCategory::Melee,
        ]),
        gameplay_tags: Vec::new(),
        ability_id: "GA_Mitsuki_Melee".into(),
        native_ability_index: Some(array_index as u64),
        input_role: Some(crate::simulation::metadata::InputActionRole::Normal),
        input_role_provenance: Some(provenance.clone()),
        primary_slot: Some(crate::simulation::metadata::PrimaryActionSlot::Normal),
        source_evidence: None,
        identity_evidence: None,
        provenance: provenance.clone(),
    };
    let metadata = crate::simulation::metadata::ActionPacketMetadata {
        action,
        packet,
        binding: crate::simulation::metadata::ActionPacketBinding {
            kind: crate::simulation::metadata::ActionPacketBindingKind::OwnerChild,
            initiating_ability_id: "GA_Mitsuki_Melee".into(),
            packet_ga_name: native.ga_name.clone(),
            provenance: Some(provenance),
            energy_group_id: None,
        },
    };

    for &target_index in &targets {
        for offset_us in [200_000, 400_000, 600_000, 800_000] {
            let hit_time_us = ctx
                .time_us
                .checked_add(offset_us)
                .ok_or("Staccato perfect bullet time overflow")?;
            crate::runtime_api::increment_sequence(ctx.sequence)?;
            ctx.queue
                .push(crate::simulation::events::ScheduledGameEvent {
                    time_us: hit_time_us,
                    priority: ntesim_core::execution::CausalPriority::HitDelivery,
                    seq: *ctx.sequence,
                    data: crate::simulation::events::GameEventData::HitDelivery {
                        command_index: ctx.command_index,
                        actor_index: ctx.actor_index,
                        target_index,
                        ability_id: "GA_Mitsuki_Melee".into(),
                        damage_effect_id: bullet_packet.into(),
                        atk_rate,
                        scaling_stat,
                        metadata: Box::new(metadata.clone()),
                    },
                });
        }
    }

    Ok(())
}
