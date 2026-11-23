//! Charge-funded entry and finite parry finisher for Daffodil.
use super::clock::profile::Hit;
use crate::runtime_api::{
    ActorHitContext, CallbackContext, CastContext, DamageContext, OwnerEntryAttack,
};
use crate::simulation::{
    GameActorConfig, GameActorParryReceipt, GameScenarioRequest,
    state::{ActorRuntime, GameSimulatorState},
};

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::DaffodilPhantomStepV1;
pub(crate) const ENTRY1: &str = "GE_Player_Daffodill_EntryAttack1_Damage";
pub(crate) const ENTRY2: &str = "GE_Player_Daffodill_EntryAttack2_Damage";
pub(crate) const ENTRY2_1: &str = "GE_Player_Daffodill_EntryAttack2_1_Damage";
pub(crate) const ENTRY3: &str = "GE_Player_Daffodill_EntryAttack3_Damage";
pub(crate) const ENTRY3_1: &str = "GE_Player_Daffodill_EntryAttack3_1_Damage";
pub(crate) const ENTRY4: &str = "GE_Player_Daffodill_EntryAttack4_Damage";
pub(crate) const APPEAR_SUCCESS: &str = "GE_Player_Daffodill_AppearSuccess_Damage";
pub(crate) const QTE: &str = "GE_Player_Daffodill_QTE_Damage";

pub(crate) const CHARGE_CAP: u8 = 2;
const CHARGE_WINDOW_US: u64 = 30_000_000;
const ENTRY1_US: u64 = 253_952;
const ENTRY2_US: u64 = 610_256;
const ENTRY2_1_US: u64 = 700_212;
const ENTRY3_US: u64 = 789_783;
const ENTRY3_1_US: u64 = 877_311;
const ENTRY4_US: u64 = 1_317_330;
pub(crate) const FINISHER_US: u64 = 1_317_610;
pub(crate) const STEP_RECOVERY_US: u64 = 1_505_627;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Route {
    Step,
    QualifiedSupportParry,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) charges: u8,
    pub(crate) expires_at_us: Option<u64>,
    pub(crate) generation: u64,
    pub(crate) active: Option<Active>,
}

#[derive(Clone, Debug)]
pub(crate) struct Active {
    pub(crate) command_index: usize,
    pub(crate) route: Route,
    pub(crate) target_index: usize,
    pub(crate) a2_fallback: bool,
    pub(crate) finisher_scheduled: bool,
    pub(crate) parry_succeeded: bool,
    pub(crate) energy_rewarded: bool,
    pub(crate) charge_consumed: bool,
}

#[derive(Clone, Debug)]
pub(crate) enum Plan {
    Step(Vec<Hit>),
    QualifiedSupportParry(Vec<Hit>),
}

pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}

pub(crate) fn validate(request: &GameScenarioRequest) -> Result<(), String> {
    if accepted(request)
        && (!super::clock::accepted(request)
            || !super::clock::cancel::accepted(request)
            || !super::secondary::accepted(request))
    {
        return Err("Daffodil Phantom Step requires ordinary_clock_v1, action_cancels_v1, and secondary_actions_v1".into());
    }
    Ok(())
}

fn first_living_target(state: &GameSimulatorState) -> Option<usize> {
    state
        .targets
        .iter()
        .position(|target| target.hp_current > 0)
}

fn has_charge(state: &State, now_us: u64) -> bool {
    state.charges > 0 && state.expires_at_us.is_some_and(|until| until > now_us)
}

pub(crate) fn entry_parry_charge(actor: &ActorRuntime, now_us: u64) -> bool {
    matches!(&actor.character, crate::characters::RuntimeState::Daffodil(owner) if has_charge(&owner.phantom, now_us))
}

pub(crate) fn select(
    request: &GameScenarioRequest,
    state: &GameSimulatorState,
    old_actor: usize,
    new_actor: usize,
) -> Result<Option<OwnerEntryAttack>, String> {
    if !accepted(request)
        || old_actor == new_actor
        || request.actors[new_actor].character_id != "character:1054"
        || state.actors[new_actor].hp == 0
        || state.actors[new_actor].locked_until_us > state.current_time_us
        || first_living_target(state).is_none()
    {
        return Ok(None);
    }
    step_entry(request, state, new_actor)
}

pub(crate) fn step_entry(
    request: &GameScenarioRequest,
    state: &GameSimulatorState,
    new_actor: usize,
) -> Result<Option<OwnerEntryAttack>, String> {
    let crate::characters::RuntimeState::Daffodil(owner) = &state.actors[new_actor].character
    else {
        return Ok(None);
    };
    if !has_charge(&owner.phantom, state.current_time_us) || first_living_target(state).is_none() {
        return Ok(None);
    }
    let coordinate = crate::simulation::inputs::normal_coordinate(
        &request.actors[new_actor],
        "GA_Daffodill_Melee",
    )?;
    Ok(Some(OwnerEntryAttack {
        ability: "GA_Daffodill_Melee",
        effect: ENTRY1,
        coordinate,
        hit_delay_us: ENTRY1_US,
        recovery_us: STEP_RECOVERY_US,
        model: MODEL,
    }))
}

pub(crate) fn burst(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if !accepted(ctx.request) {
        return Ok(());
    }
    ctx.record_hypothesis(MODEL)?;
    let expires_at_us = ctx
        .time_us
        .checked_add(CHARGE_WINDOW_US)
        .ok_or("Daffodil Phantom Step handle overflow")?;
    let state = &mut ctx.actor.character.daffodil_mut()?.phantom;
    state.generation = state
        .generation
        .checked_add(1)
        .ok_or("Daffodil Phantom Step generation overflow")?;
    state.charges = CHARGE_CAP;
    state.expires_at_us = Some(expires_at_us);
    state.active = None;
    if expires_at_us <= ctx.request.absolute_limit_us {
        let generation = state.generation;
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue
            .push(crate::simulation::events::ScheduledGameEvent {
                time_us: expires_at_us,
                priority: crate::execution::CausalPriority::BuffTickOrExpiry,
                seq: *ctx.sequence,
                data: crate::simulation::events::GameEventData::Content(
                    crate::callbacks::ContentEvent::Daffodil(super::clock::Event::PhantomExpiry {
                        owner: ctx.actor_index,
                        generation,
                    }),
                ),
            });
    }
    Ok(())
}

fn action_is_switch_cast(ctx: &CastContext<'_>) -> bool {
    matches!(
        ctx.request.actions.get(ctx.command_index),
        Some(crate::simulation::GameActionCommand::SwitchActor { target_actor_index, .. })
            if *target_actor_index == ctx.actor_index
    ) && ctx.active_actor_index == ctx.actor_index
}

fn consume_charge(
    ctx: &mut CastContext<'_>,
    route: Route,
    target_index: usize,
    a2_fallback: bool,
    finisher_scheduled: bool,
) -> Result<(), String> {
    let phantom = &mut ctx.actor.character.daffodil_mut()?.phantom;
    if !has_charge(phantom, ctx.time_us) {
        return Err("Daffodil Phantom Step entry charge expired before admission".into());
    }
    phantom.charges -= 1;
    phantom.active = Some(Active {
        command_index: ctx.command_index,
        route,
        target_index,
        a2_fallback,
        finisher_scheduled,
        parry_succeeded: false,
        energy_rewarded: false,
        charge_consumed: true,
    });
    Ok(())
}

pub(crate) fn plan(
    ctx: &mut CastContext<'_>,
    qualified_support_selected: bool,
) -> Result<Option<Plan>, String> {
    if !accepted(ctx.request) {
        return Ok(None);
    }
    let effect = ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        .unwrap_or_default();
    if effect == ENTRY1 {
        if !action_is_switch_cast(ctx) || ctx.ability_id != "GA_Daffodill_Melee" {
            return Err("Daffodil Phantom Step requires its engine-selected switch entry".into());
        }
        let target_index = ctx
            .targets
            .iter()
            .position(|target| target.hp_current > 0)
            .ok_or("Daffodil Phantom Step requires a living target")?;
        let a2 = ctx.config.awakening.has_effect(2);
        // Schedule the conditional packet without inspecting future parry input.
        // The live marker callback removes it unless success or the A2 fallback qualifies.
        let finisher_scheduled = true;
        consume_charge(ctx, Route::Step, target_index, a2, finisher_scheduled)?;
        ctx.record_hypothesis(MODEL)?;
        let mut hits = vec![
            (ENTRY1, ENTRY1_US),
            (ENTRY2, ENTRY2_US),
            (ENTRY2_1, ENTRY2_1_US),
            (ENTRY3, ENTRY3_US),
        ];
        if a2 {
            hits.push((ENTRY3_1, ENTRY3_1_US));
        }
        hits.push((ENTRY4, ENTRY4_US));
        if finisher_scheduled {
            hits.push((APPEAR_SUCCESS, FINISHER_US));
        }
        return Ok(Some(Plan::Step(hits)));
    }
    if qualified_support_selected && effect == QTE && action_is_switch_cast(ctx) {
        let target_index = ctx
            .targets
            .iter()
            .position(|target| target.hp_current > 0)
            .ok_or("Daffodil qualified Support requires a living target")?;
        let charge_available = matches!(
            &ctx.actor.character,
            crate::characters::RuntimeState::Daffodil(owner)
                if has_charge(&owner.phantom, ctx.time_us)
        );
        if !charge_available {
            return Ok(None);
        }
        let phantom = &mut ctx.actor.character.daffodil_mut()?.phantom;
        phantom.active = Some(Active {
            command_index: ctx.command_index,
            route: Route::QualifiedSupportParry,
            target_index,
            a2_fallback: false,
            finisher_scheduled: true,
            parry_succeeded: false,
            energy_rewarded: false,
            charge_consumed: false,
        });
        ctx.record_hypothesis(MODEL)?;
        return Ok(Some(Plan::QualifiedSupportParry(vec![
            (QTE, 745_033),
            (APPEAR_SUCCESS, FINISHER_US),
        ])));
    }
    Ok(None)
}

pub(crate) fn marker_deadline_us(
    actor: &ActorRuntime,
    command: usize,
) -> Result<Option<u64>, String> {
    let crate::characters::RuntimeState::Daffodil(owner) = &actor.character else {
        return Ok(None);
    };
    let Some(_active) = owner
        .phantom
        .active
        .as_ref()
        .filter(|active| active.command_index == command && active.finisher_scheduled)
    else {
        return Ok(None);
    };
    super::clock::phantom_marker_deadline(actor, command)
}

pub(crate) fn finisher_scheduled(actor: &ActorRuntime, command: usize) -> bool {
    matches!(
        &actor.character,
        crate::characters::RuntimeState::Daffodil(owner)
            if owner.phantom.active.as_ref().is_some_and(|active|
                active.command_index == command && active.finisher_scheduled
            )
    )
}

pub(crate) fn validate_parry(
    request: &GameScenarioRequest,
    config: &GameActorConfig,
) -> Result<(), String> {
    if config.character_id != "character:1054" {
        return Ok(());
    }
    if !accepted(request) {
        return Err("Daffodil explicit parry requires phantom_step_v1".into());
    }
    validate(request)
}

pub(crate) fn parry(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    event_index: usize,
) -> Result<GameActorParryReceipt, String> {
    let now_us = ctx.state.current_time_us;
    let can_capture = if let Some(actor) = ctx.state.actors.get(actor_index)
        && let crate::characters::RuntimeState::Daffodil(owner) = &actor.character
        && let Some(active) = owner.phantom.active.as_ref()
    {
        let command = active.command_index;
        let marker = marker_deadline_us(actor, command)?;
        (active.target_index < ctx.state.targets.len()
            && ctx.state.targets[active.target_index].hp_current > 0
            && ctx.state.active_actor_index == actor_index
            && marker.is_some_and(|deadline| now_us < deadline))
        .then_some(command)
    } else {
        None
    };
    let mut extend_qte_recovery = false;
    if let Some(command) = can_capture {
        let actor = &mut ctx.state.actors[actor_index];
        if let crate::characters::RuntimeState::Daffodil(owner) = &mut actor.character {
            let route_ready = owner.phantom.active.as_ref().is_some_and(|active| {
                active.command_index == command
                    && !active.parry_succeeded
                    && (active.route == Route::Step
                        || active.charge_consumed
                        || has_charge(&owner.phantom, now_us))
            });
            if route_ready {
                extend_qte_recovery = owner
                    .phantom
                    .active
                    .as_ref()
                    .is_some_and(|active| active.route == Route::QualifiedSupportParry);
                if let Some(active) = owner.phantom.active.as_ref()
                    && active.route == Route::QualifiedSupportParry
                    && !active.charge_consumed
                {
                    owner.phantom.charges -= 1;
                }
                if let Some(active) = owner.phantom.active.as_mut() {
                    active.parry_succeeded = true;
                    active.charge_consumed = true;
                }
            }
        }
    }
    if extend_qte_recovery {
        super::clock::after_parry_success(&mut ctx.state.actors[actor_index], ctx.queue, now_us)?;
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        MODEL,
    )?;
    Ok(GameActorParryReceipt {
        event_index,
        actor_index,
        time_us: ctx.state.current_time_us,
        malice_granted: crate::simulation::ExactFraction::new(0, 1)?,
        skill_permission_until_us: None,
    })
}

pub(crate) fn parry_succeeded(actor: &ActorRuntime, command: usize) -> bool {
    matches!(
        &actor.character,
        crate::characters::RuntimeState::Daffodil(owner)
            if owner.phantom.active.as_ref().is_some_and(|active| {
                active.command_index == command && active.parry_succeeded
            })
    )
}

pub(crate) fn finish_check(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    command: usize,
) -> Result<(), String> {
    let actor = &mut ctx.state.actors[actor_index];
    let allow_finisher = if let crate::characters::RuntimeState::Daffodil(owner) = &actor.character
    {
        owner.phantom.active.as_ref().is_some_and(|active| {
            active.command_index == command
                && active.finisher_scheduled
                && (active.parry_succeeded || active.a2_fallback)
        })
    } else {
        false
    };
    if !allow_finisher {
        ctx.queue.retain(|event| {
            !matches!(
                &event.data,
                crate::simulation::events::GameEventData::HitDelivery {
                    command_index,
                    actor_index: owner,
                    damage_effect_id,
                    ..
                } if *command_index == command
                    && *owner == actor_index
                    && damage_effect_id == APPEAR_SUCCESS
            )
        });
    }
    Ok(())
}

pub(crate) fn cancel_active(actor: &mut ActorRuntime, command: usize) -> Result<(), String> {
    let phantom = &mut actor.character.daffodil_mut()?.phantom;
    if phantom
        .active
        .as_ref()
        .is_some_and(|active| active.command_index == command)
    {
        phantom.active = None;
    }
    Ok(())
}

pub(crate) fn damage_bonus_applies(ctx: &DamageContext<'_>) -> bool {
    if !matches!(
        ctx.damage_effect_id,
        ENTRY1 | ENTRY2 | ENTRY2_1 | ENTRY3 | ENTRY3_1 | ENTRY4 | APPEAR_SUCCESS
    ) {
        return false;
    }
    matches!(
        &ctx.actor.character,
        crate::characters::RuntimeState::Daffodil(owner)
            if owner.phantom.active.as_ref().is_some_and(|active| {
                if active.command_index != ctx.command_index {
                    return false;
                }
                match ctx.damage_effect_id {
                    ENTRY1 | ENTRY2 | ENTRY2_1 | ENTRY3 | ENTRY3_1 | ENTRY4 => {
                        active.route == Route::Step
                    }
                    APPEAR_SUCCESS => active.finisher_scheduled,
                    _ => false,
                }
            })
    )
}

pub(crate) fn on_hit(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    if !accepted(ctx.request) || ctx.damage_effect_id != APPEAR_SUCCESS || ctx.applied_hp_loss == 0
    {
        return Ok(());
    }
    let reward = if let crate::characters::RuntimeState::Daffodil(owner) = &mut ctx.actor.character
    {
        owner.phantom.active.as_mut().and_then(|active| {
            (active.command_index == ctx.command_index
                && active.finisher_scheduled
                && !active.energy_rewarded)
                .then(|| {
                    active.energy_rewarded = true;
                    true
                })
        })
    } else {
        None
    };
    if reward.is_none() {
        return Ok(());
    }
    crate::runtime_api::record_hypothesis_id(ctx.request, ctx.execution_assumptions, MODEL)?;
    let snapshot = crate::simulation::energy::EnergySnapshot::capture(ctx.actor);
    let missing = ctx
        .actor
        .energy
        .capacity()
        .saturating_sub(ctx.actor.energy.current());
    ctx.actor.restore_energy(missing)?;
    let energy_change = snapshot.receipt(
        ctx.actor,
        crate::simulation::GameEnergySource::DaffodilHit,
        ctx.actor_index,
        ctx.actor_index,
        None,
    )?;
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: ctx.time_us,
        phase: "ProcCallback".into(),
        event_kind: "daffodil_phantom_step_energy_fill".into(),
        active_actor: ctx.active_actor_index,
        hit_index: None,
        from_actor: Some(ctx.actor_index),
        to_actor: Some(ctx.actor_index),
        energy_change: Some(energy_change),
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            damage_effect_id: Some(APPEAR_SUCCESS.into()),
            ..Default::default()
        }),
    });
    Ok(())
}
