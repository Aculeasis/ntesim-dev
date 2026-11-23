//! Reached Skill.3 applies real Warrant before the first bite, with its own lifetime.
use crate::runtime_api::{CallbackContext, CastContext};
pub(crate) const TRANSFER: crate::HypothesisId = crate::HypothesisId::ChaosWarrantTransferV1;
pub(super) const MARKER_US: u64 = 283_733;

fn transfer_accepted(request: &crate::simulation::GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(TRANSFER.as_str())
        .map(String::as_str)
        == Some(TRANSFER.description())
}

pub(crate) fn validate(request: &crate::simulation::GameScenarioRequest) -> Result<(), String> {
    if transfer_accepted(request) && !super::clock::accepted(request) {
        return Err("Chaos Warrant transfer requires ordinary_clock_v1".into());
    }
    Ok(())
}

pub(super) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let all = match ctx.ability_id {
        "GA_Chaos071_Skill" if super::clock::accepted(ctx.request) => false,
        "GA_Chaos071_UltraSkill" if super::form::accepted(ctx.request) => true,
        _ => return Ok(()),
    };
    let progress = if all { 2_700_000 } else { MARKER_US };
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue
        .push(crate::simulation::events::ScheduledGameEvent {
            time_us: ctx
                .time_us
                .checked_add(progress)
                .ok_or("Chaos Warrant marker overflow")?,
            priority: ntesim_core::execution::CausalPriority::BuffTickOrExpiry,
            seq: *ctx.sequence,
            data: crate::simulation::events::GameEventData::Content(
                crate::callbacks::ContentEvent::Chaos(super::events::Event::Warrant {
                    owner: ctx.actor_index,
                    command: ctx.command_index,
                    all,
                    progress,
                }),
            ),
        });
    Ok(())
}

pub(crate) fn apply(ctx: &mut CallbackContext<'_>, owner: usize, all: bool) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }
    let recipients: Vec<_> = ctx
        .state
        .targets
        .iter()
        .enumerate()
        .filter(|(_, target)| target.hp_current > 0)
        .take(if all { usize::MAX } else { 2 })
        .map(|(index, _)| index)
        .collect();
    for target_index in &recipients {
        apply_one(ctx, owner, *target_index)?;
    }
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "BuffTickOrExpiry".into(),
        event_kind: if all {
            "chaos_burst_warrant"
        } else {
            "chaos_skill_warrant"
        }
        .into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            all: Some(all),
            recipients: Some(recipients),
            ..Default::default()
        }),
    });
    Ok(())
}

fn apply_one(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    target_index: usize,
) -> Result<(), String> {
    let config = &ctx.request.actors[owner];
    let reductions = if config.awakening.count() >= 6 {
        std::collections::BTreeMap::from([(
            "ECharacterElementType::CHARACTER_ELEMENT_TYPE_LAKSHANA".into(),
            super::parameters::PARAMETERS.resonance_resistance_reduction,
        )])
    } else {
        Default::default()
    };
    crate::simulation::state::apply_target_status(
        &mut ctx.state.buffs,
        crate::simulation::state::TargetStatusDeclaration {
            source_actor: owner,
            source: config.character_id.clone(),
            target_index,
            id: "chaos.warrant".into(),
            now_us: ctx.state.mode_clock.relative_us(),
            duration_us: super::parameters::PARAMETERS.warrant_duration_us,
            reductions,
        },
    )?;
    Ok(())
}

pub(crate) fn on_party_defeat(
    ctx: &mut CallbackContext<'_>,
    killer: usize,
    target: usize,
) -> Result<(), String> {
    if !super::clock::accepted(ctx.request) {
        return Ok(());
    }
    let marked =
        |owner: usize, recipient: usize, buffs: &ntesim_core::buffs::dispatch::BuffManager| {
            crate::simulation::state::target_has_status(
                buffs,
                owner,
                "character:1071",
                recipient,
                "chaos.warrant",
            )
        };
    let owners: Vec<_> = ctx
        .request
        .actors
        .iter()
        .enumerate()
        .filter(|(owner, config)| {
            config.character_id == "character:1071"
                && config.awakening.has_effect(4)
                && marked(*owner, target, &ctx.state.buffs)
        })
        .map(|(owner, _)| owner)
        .collect();
    if !owners.is_empty() && !transfer_accepted(ctx.request) {
        return Err("Chaos marked party kill with A4 requires warrant_transfer_v1".into());
    }
    for owner in owners {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            TRANSFER,
        )?;
        let next = ctx
            .state
            .targets
            .iter()
            .enumerate()
            .find(|(index, victim)| {
                *index != target
                    && victim.hp_current > 0
                    && !marked(owner, *index, &ctx.state.buffs)
            })
            .map(|(index, _)| index);
        if let Some(next) = next {
            apply_one(ctx, owner, next)?;
        }
        ctx.trace.push(crate::simulation::GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "SettlementAndDeath".into(),
            event_kind: "chaos_warrant_transfer".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: Some(owner),
            to_actor: Some(killer),
            energy_change: None,
            energy_fanout: None,
            data: Some(crate::simulation::GameEventPayload {
                target_index: Some(target),
                next_target_index: next,
                ..Default::default()
            }),
        });
    }
    // Every source's mark ends with its recipient, including A0 and no-candidate cases.
    // Removal after all source decisions makes repeated death callbacks harmless.
    ctx.state
        .buffs
        .remove_target_buff("chaos.warrant", "character:1071", &target.to_string());
    Ok(())
}
