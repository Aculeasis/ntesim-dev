//! Passive 1 (`Relay Node`): Charge Enhancement & Remora Enhancement + Passive 2 (`Pity System`).
use super::parameters::PARAMETERS;
use crate::callbacks::ContentEvent;
use crate::execution::CausalPriority;
use crate::runtime_api::CallbackContext;
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GameCausalTrigger, GameEventPayload, GameEventReceipt};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy};
use std::collections::BTreeMap;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ElymsRelayNodeV1;
pub(crate) const CHARGE_ENHANCEMENT_BUFF_ID: &str = "elyms.passive1.charge_enhancement";
const COSMOS_DMG_KEY: &str = "damage_bonus:ECharacterElementType::CHARACTER_ELEMENT_TYPE_COSMOS";

pub(crate) fn on_active_actor_energy_gain(
    ctx: &mut CallbackContext<'_>,
    recipient: usize,
) -> Result<(), String> {
    if recipient != ctx.state.active_actor_index || ctx.state.actors[recipient].hp == 0 {
        return Ok(());
    }
    let owners: Vec<usize> = (0..ctx.request.actors.len())
        .filter(|&i| {
            ctx.request.actors[i].character_id == "character:1090" && ctx.state.actors[i].hp > 0
        })
        .collect();
    for owner in owners {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            MODEL,
        )?;
        let mut decl = TimedBuffDeclaration::reset(
            CHARGE_ENHANCEMENT_BUFF_ID,
            "character:1090",
            BuffRecipient::TeamAll,
            PARAMETERS.passive1_cosmos_duration_us,
            BuffStackingScope::PerSource,
            BTreeMap::from([(
                COSMOS_DMG_KEY.into(),
                PARAMETERS.passive1_cosmos_dmg_per_stack,
            )]),
        );
        decl.refresh_policy = RefreshPolicy::Reset;
        decl.max_stacks = PARAMETERS.passive1_cosmos_max_stacks;
        decl.decay_policy = DecayPolicy::AllAtOnce;
        ctx.state.apply_buff(owner, decl)?;
        ctx.trace.push(GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "ProcCallback".into(),
            event_kind: "elyms_charge_enhancement".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: Some(owner),
            to_actor: Some(recipient),
            energy_change: None,
            energy_fanout: None,
            data: Some(GameEventPayload {
                buff_id: Some(CHARGE_ENHANCEMENT_BUFF_ID.into()),
                duration_us: Some(PARAMETERS.passive1_cosmos_duration_us),
                ..Default::default()
            }),
        });
    }
    Ok(())
}

pub(crate) fn on_remora_created(
    ctx: &mut CallbackContext<'_>,
    _applier: usize,
    target: usize,
    allow_elyms_retrigger: bool,
) -> Result<(), String> {
    if !allow_elyms_retrigger || ctx.state.targets[target].hp_current == 0 {
        return Ok(());
    }
    let rel_now = ctx.state.mode_clock.relative_us();
    let abs_now = ctx.state.current_time_us;
    let command = crate::simulation::reactions::remora_command(ctx.state, target).unwrap_or(0);
    let owners: Vec<usize> = (0..ctx.request.actors.len())
        .filter(|&i| {
            ctx.request.actors[i].character_id == "character:1090" && ctx.state.actors[i].hp > 0
        })
        .collect();
    for owner in owners {
        let should_schedule = {
            let state = ctx.state.actors[owner].character.elyms_mut()?;
            if rel_now < state.passive1_remora_icd_until_rel_us {
                false
            } else {
                state.passive1_remora_icd_until_rel_us = rel_now
                    .checked_add(PARAMETERS.passive1_remora_icd_us)
                    .ok_or("Elyms Remora Enhancement ICD overflow")?;
                state.passive1_remora_generation =
                    state.passive1_remora_generation.saturating_add(1);
                true
            }
        };
        if !should_schedule {
            continue;
        }
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            MODEL,
        )?;
        let generation = ctx.state.actors[owner]
            .character
            .elyms()?
            .passive1_remora_generation;
        let fire_at = abs_now
            .checked_add(PARAMETERS.passive1_remora_delay_us)
            .ok_or("Elyms Remora Enhancement deadline overflow")?;
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: fire_at,
            priority: CausalPriority::ProcCallback,
            seq: *ctx.sequence,
            data: GameEventData::Content(ContentEvent::Elyms(
                super::events::Event::RemoraEnhancement {
                    owner,
                    target,
                    command,
                    generation,
                },
            )),
        });
    }
    Ok(())
}

pub(crate) fn on_delayed_remora(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    target: usize,
    command: usize,
    _generation: u64,
) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 || ctx.state.targets[target].hp_current == 0 {
        return Ok(());
    }
    let trigger = match ctx.state.origins.identify(ctx.request, command)? {
        (None, Some(autonomous_event_index)) => GameCausalTrigger::OwnerDeadline {
            autonomous_event_index,
        },
        (Some(cmd), None) => {
            if matches!(
                ctx.request.actions.get(cmd),
                Some(crate::simulation::GameActionCommand::SwitchActor { .. })
            ) {
                GameCausalTrigger::FieldEntry { command_index: cmd }
            } else {
                GameCausalTrigger::Action { command_index: cmd }
            }
        }
        _ => return Err("unexpected causal identity for Elyms Remora Enhancement".into()),
    };
    let key = ctx.state.origins.register(
        ctx.request,
        crate::simulation::GameAutonomousEventReceipt {
            actor_index: owner,
            active_actor_index: ctx.state.active_actor_index,
            ability_id: "GA_Elyms_Passive1".into(),
            time_us: ctx.state.current_time_us,
            trigger,
            recipient_healing: None,
        },
    )?;
    crate::simulation::reactions::apply_remora_from_owner_with_retrigger_control(
        ctx, owner, target, key, false,
    )?;
    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "elyms_remora_enhancement".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: Some(target),
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            ability_id: Some("GA_Elyms_Passive1".into()),
            ..Default::default()
        }),
    });
    Ok(())
}

pub(crate) fn crit_damage_bonus(actor: &crate::simulation::state::ActorRuntime) -> ExactFraction {
    if actor
        .character
        .elyms()
        .is_ok_and(|state| state.passive2_unlocked)
    {
        PARAMETERS.passive2_crit_dmg_bonus
    } else {
        ExactFraction {
            numerator: 0,
            denominator: 1,
        }
    }
}
