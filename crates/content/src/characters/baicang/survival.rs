use crate::simulation::GameScenarioRequest;
use crate::simulation::state::ActorRuntime;

pub(crate) fn execute_owner(
    request: &GameScenarioRequest,
    state: &crate::simulation::state::GameSimulatorState,
    parent_actor: usize,
    target_index: usize,
    hp_loss: u64,
) -> Option<usize> {
    if hp_loss == 0 || state.targets[target_index].hp_current == 0 {
        return None;
    }
    let target = &state.targets[target_index];
    let threshold = if target.is_boss { 5 } else { 10 };
    if u128::from(target.hp_current) * 100 >= u128::from(target.hp_max) * threshold {
        return None;
    }
    state.actors.iter().enumerate().find_map(|(index, actor)| {
        let crate::characters::RuntimeState::Baicang(field) = &actor.character else {
            return None;
        };
        (actor.hp > 0
            && field.domain_start_us <= state.current_time_us
            && field.domain_until_us > state.current_time_us
            && ((index == parent_actor && state.active_actor_index == index)
                || request.actors[index].awakening.count() >= 6)
            && request
                .model_assumptions
                .get(super::skill::EXECUTE_HYPOTHESIS_ID.as_str())
                .map(String::as_str)
                == Some(super::skill::EXECUTE_HYPOTHESIS_ID.description()))
        .then_some(index)
    })
}

pub(crate) fn is_actor_fatal_hit_immune(actor: &ActorRuntime, now_us: u64) -> bool {
    matches!(&actor.character, crate::characters::RuntimeState::Baicang(field) if field.fatal_hit_immune_until_us > now_us || field.dodge_immune_until_us > now_us)
}

pub(crate) fn intercept_fatal_hit(
    ctx: &mut crate::runtime_api::IncomingHpContext<'_>,
    vitals: &mut ntesim_core::state::VitalState,
) -> Result<bool, String> {
    if !matches!(&ctx.state.actors[ctx.actor_index].character, crate::characters::RuntimeState::Baicang(field) if field.domain_start_us <= ctx.state.current_time_us && field.domain_until_us > ctx.state.current_time_us)
    {
        return Ok(false);
    }
    if !ctx
        .request
        .model_assumptions
        .contains_key("baicang.execute_and_fatal_hit_debt_v1")
    {
        return Err("UNSUPPORTED_BAICANG_FATAL_IMMUNITY: residual HP damage reached the unmodeled fatal-immunity branch during Judgment of Autumn".into());
    }
    let now_us = ctx.state.current_time_us;
    let actor_index = ctx.actor_index;
    let hp_before = ctx.hp_before;
    let residual = ctx.residual;
    let (generation, old_end, was_armed) = {
        let baicang = ctx.state.actors[actor_index].character.baicang_mut()?;
        if baicang.fatal_hit_armed {
            baicang.fatal_hit_armed = false;
            baicang.fatal_hit_debt_total = baicang.fatal_hit_debt_total.saturating_add(residual);
            baicang.fatal_hit_immune_until_us = now_us.saturating_add(3_000_000);
            let old_end = baicang.domain_until_us;
            let shortened_end = now_us.saturating_add(3_000_000);
            baicang.domain_until_us = old_end.min(shortened_end);
            (baicang.domain_generation, old_end, true)
        } else {
            (0, 0, false)
        }
    };
    if was_armed {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            crate::characters::baicang::skill::EXECUTE_HYPOTHESIS_ID,
        )?;
        let shortened_end = now_us.saturating_add(3_000_000);
        vitals
            .settle_hp_damage(hp_before.saturating_sub(1))
            .map_err(|e| format!("settle HP failed: {e:?}"))?;

        if shortened_end < old_end {
            crate::runtime_api::increment_sequence(ctx.sequence)?;
            ctx.queue
                .push(crate::simulation::events::ScheduledGameEvent {
                    time_us: shortened_end,
                    priority: ntesim_core::execution::CausalPriority::BuffTickOrExpiry,
                    seq: *ctx.sequence,
                    data: crate::simulation::events::GameEventData::Content(
                        crate::callbacks::ContentEvent::BaicangDomainEnd {
                            actor_index,
                            generation,
                        },
                    ),
                });
        }

        let end = old_end.min(shortened_end);
        let each = residual / 10;
        for slot in 1..=10 {
            crate::runtime_api::increment_sequence(ctx.sequence)?;
            ctx.queue
                .push(crate::simulation::events::ScheduledGameEvent {
                    time_us: end
                        .checked_add(slot * 1_000_000)
                        .ok_or("fatal debt deadline overflow")?,
                    priority: ntesim_core::execution::CausalPriority::SettlementAndDeath,
                    seq: *ctx.sequence,
                    data: crate::simulation::events::GameEventData::Content(
                        crate::callbacks::ContentEvent::BaicangFatalDebtInstallment {
                            actor_index,
                            amount: if slot == 10 {
                                residual - 9 * each
                            } else {
                                each
                            },
                        },
                    ),
                });
        }
        ctx.trace.push(crate::simulation::GameEventReceipt {
            time_us: now_us,
            phase: "HitDelivery".into(),
            event_kind: "baicang_fatal_hit_intercepted".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: None,
            to_actor: Some(actor_index),
            energy_change: None,
            energy_fanout: None,
            data: Some(crate::simulation::GameEventPayload {
                amount: Some(residual),
                hp_after: Some(1),
                duration_us: Some(3_000_000),
                ..Default::default()
            }),
        });

        Ok(true)
    } else {
        Ok(false)
    }
}

pub(crate) fn finishing_damage(
    ctx: &crate::runtime_api::FinishHitContext<'_>,
    owner: usize,
) -> Result<Option<crate::runtime_api::FinishDamage>, String> {
    if execute_owner(
        ctx.request,
        ctx.state,
        ctx.parent_actor,
        ctx.target_index,
        ctx.applied_hp_loss,
    ) != Some(owner)
    {
        return Ok(None);
    }
    let id = super::skill::EXECUTE_HYPOTHESIS_ID;
    let zero = crate::simulation::ExactFraction::new(0, 1)?;
    let mut metadata = ctx.metadata.clone();
    metadata.action.ability_id = "GA_Cang_UltraSkill".into();
    metadata.action.categories =
        std::collections::BTreeSet::from([crate::simulation::metadata::ActionCategory::Other]);
    metadata.action.gameplay_tags.clear();
    metadata.action.primary_slot = None;
    metadata.action.input_role = Some(crate::simulation::metadata::InputActionRole::Other);
    metadata.action.native_ability_index = None;
    metadata.action.source_evidence = None;
    metadata.action.identity_evidence = None;
    metadata.packet.damage_effect_id = "ntesim.baicang.execute".into();
    metadata.packet.damage_kind = ntesim_core::combat::DamageKind::True;
    metadata.packet.element = None;
    metadata.packet.native_ga_name = None;
    metadata.packet.native_damage_source_category = None;
    metadata.packet.native_damage_type_ex = "EExecutionDamageTypeEX::DAMAGE_TYPE_TRUE".into();
    metadata.packet.fixed_crit_rate = Some(zero);
    metadata.packet.unbal_value = Some(zero);
    metadata.packet.semantic_tags.clear();
    metadata.packet.source_evidence = None;
    let provenance = crate::simulation::metadata::MetadataProvenance::AcceptedModelHypothesis {
        id: id.as_str().into(),
        description: id.description().into(),
    };
    metadata.action.provenance = provenance.clone();
    metadata.action.input_role_provenance = Some(provenance.clone());
    metadata.packet.provenance = provenance.clone();
    metadata.packet.fixed_crit_rate_provenance = Some(provenance.clone());
    metadata.packet.unbal_value_provenance = Some(provenance.clone());
    metadata.packet.semantic_provenance = Some(provenance.clone());
    metadata.binding = crate::simulation::metadata::ActionPacketBinding {
        energy_group_id: None,
        kind: crate::simulation::metadata::ActionPacketBindingKind::OwnerChild,
        initiating_ability_id: "GA_Cang_UltraSkill".into(),
        packet_ga_name: None,
        provenance: Some(provenance),
    };

    Ok(Some(crate::runtime_api::FinishDamage {
        owner,
        hypothesis: id,
        metadata,
        ability_id: "GA_Cang_UltraSkill",
        effect_id: "ntesim.baicang.execute",
    }))
}
