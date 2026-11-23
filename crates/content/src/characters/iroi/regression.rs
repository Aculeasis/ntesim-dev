use crate::HypothesisId;
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::{
    ExactFraction,
    events::{GameEventData, ScheduledGameEvent},
};
use ntesim_core::execution::CausalPriority;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::IroiRegressionV1;

pub(crate) fn accepted(request: &crate::simulation::GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(HYPOTHESIS_ID.as_str())
        .map(String::as_str)
        == Some(HYPOTHESIS_ID.description())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum LambType {
    Gun,
    Syringe,
    Trumpet,
}

pub(crate) fn lamb_type_for_slot(other_ally_index: usize) -> LambType {
    match other_ally_index % 3 {
        0 => LambType::Gun,
        1 => LambType::Syringe,
        _ => LambType::Trumpet,
    }
}

pub(crate) fn push_lamb_ai(
    queue: &mut crate::simulation::timer_queue::EventQueue,
    sequence: &mut u64,
    time_us: u64,
    relative_duration_us: u64,
    owner: usize,
    ally_index: usize,
    lamb_type: LambType,
    skill_number: u8,
) -> Result<(), String> {
    if time_us > relative_duration_us {
        return Ok(());
    }
    super::clock::push(
        queue,
        sequence,
        time_us,
        super::clock::Event::LambAi {
            owner,
            ally_index,
            lamb_type,
            skill_number,
        },
    )
}

pub(crate) fn voluntary_regress_all_allies(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let iroi_idx = ctx.actor_index;
    let mut others: Vec<usize> = (0..ctx.request.actors.len())
        .filter(|&idx| idx != iroi_idx)
        .collect();
    others.sort_by_key(|&idx| ctx.request.actors[idx].slot);

    let now = ctx.time_us;
    let state = ctx.actor.character.iroi_mut()?;

    for (order, &ally_idx) in others.iter().enumerate() {
        if state.regressed_allies.contains_key(&ally_idx) {
            continue;
        }
        let lamb = lamb_type_for_slot(order);
        state.regressed_allies.insert(ally_idx, lamb);

        let (cd1, cd2) = match lamb {
            LambType::Gun => (1_500_000, 10_000_000),
            LambType::Syringe => (2_000_000, 9_000_000),
            LambType::Trumpet => (2_500_000, 8_500_000),
        };
        state
            .lamb_cooldowns
            .insert((ally_idx, 1), now.saturating_add(cd1));
        state
            .lamb_cooldowns
            .insert((ally_idx, 2), now.saturating_add(cd2));

        push_lamb_ai(
            ctx.queue,
            ctx.sequence,
            now.saturating_add(cd1),
            ctx.request.absolute_limit_us,
            iroi_idx,
            ally_idx,
            lamb,
            1,
        )?;
        push_lamb_ai(
            ctx.queue,
            ctx.sequence,
            now.saturating_add(cd2),
            ctx.request.absolute_limit_us,
            iroi_idx,
            ally_idx,
            lamb,
            2,
        )?;
    }

    ctx.record_hypothesis(HYPOTHESIS_ID)?;
    Ok(())
}

pub(crate) fn on_ally_fallen(
    ctx: &mut CallbackContext<'_>,
    iroi_actor_index: usize,
    fallen_actor_index: usize,
) -> Result<(), String> {
    if fallen_actor_index == iroi_actor_index || !accepted(ctx.request) {
        return Ok(());
    }

    let iroi_config = &ctx.request.actors[iroi_actor_index];
    if iroi_config.character_id != "character:1075" {
        return Ok(());
    }

    let state = ctx.state.actors[iroi_actor_index].character.iroi_mut()?;

    if state.regressed_allies.contains_key(&fallen_actor_index) {
        return Ok(());
    }

    let mut others: Vec<usize> = (0..ctx.request.actors.len())
        .filter(|&idx| idx != iroi_actor_index)
        .collect();
    others.sort_by_key(|&idx| ctx.request.actors[idx].slot);
    let order = others
        .iter()
        .position(|&idx| idx == fallen_actor_index)
        .unwrap_or(0);
    let lamb = lamb_type_for_slot(order);

    state.regressed_allies.insert(fallen_actor_index, lamb);

    let now = ctx.state.current_time_us;
    let (cd1, cd2) = match lamb {
        LambType::Gun => (1_500_000, 10_000_000),
        LambType::Syringe => (2_000_000, 9_000_000),
        LambType::Trumpet => (2_500_000, 8_500_000),
    };
    state
        .lamb_cooldowns
        .insert((fallen_actor_index, 1), now.saturating_add(cd1));
    state
        .lamb_cooldowns
        .insert((fallen_actor_index, 2), now.saturating_add(cd2));

    push_lamb_ai(
        ctx.queue,
        ctx.sequence,
        now.saturating_add(cd1),
        ctx.request.absolute_limit_us,
        iroi_actor_index,
        fallen_actor_index,
        lamb,
        1,
    )?;
    push_lamb_ai(
        ctx.queue,
        ctx.sequence,
        now.saturating_add(cd2),
        ctx.request.absolute_limit_us,
        iroi_actor_index,
        fallen_actor_index,
        lamb,
        2,
    )?;

    // Death grants 20 Moon once (if living -> fallen and not in Dream)
    if !state.dream_active {
        let base_moon = ExactFraction::new(20, 1)?;
        let gain = if iroi_config.awakening.has_effect(1) {
            base_moon.mul(ExactFraction::new(13, 10)?)?
        } else {
            base_moon
        };
        let cap = ExactFraction::new(100, 1)?;
        let new_moon = state.moon.add(gain)?;
        if new_moon.numerator > 100 * new_moon.denominator {
            let overflow = new_moon.sub(cap)?;
            state.moon_overflow = state.moon_overflow.add(overflow)?;
            state.moon = cap;
        } else {
            state.moon = new_moon;
        }
    }

    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        HYPOTHESIS_ID,
    )?;

    Ok(())
}

pub(crate) fn on_lamb_ai_tick(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    ally_index: usize,
    lamb_type: LambType,
    skill_number: u8,
) -> Result<(), String> {
    let state = ctx.state.actors[owner].character.iroi()?;
    if !state.regressed_allies.contains_key(&ally_index) {
        return Ok(());
    }
    if !ctx.state.targets.iter().any(|t| t.hp_current > 0) {
        return Ok(());
    }

    let now = ctx.state.current_time_us;
    let (effect, count, cooldown) = match (lamb_type, skill_number) {
        (LambType::Gun, 1) => ("GE_Player_Oneiroi_GunAI_Skill1_Damage", 1, 1_500_000),
        (LambType::Gun, 2) => ("GE_Player_Oneiroi_GunAI_Skill2_Damage", 5, 10_000_000),
        (LambType::Syringe, 1) => ("GE_Player_Oneiroi_SyringeAI_Skill1_Damage", 1, 2_000_000),
        (LambType::Syringe, 2) => ("GE_Player_Oneiroi_SyringeAI_Skill2_Damage", 1, 9_000_000),
        (LambType::Trumpet, 1) => ("GE_Player_Oneiroi_TrumpetAI_Skill1_Damage", 1, 2_500_000),
        (LambType::Trumpet, 2) => ("GE_Player_Oneiroi_TrumpetAI_Skill2_Damage", 1, 8_500_000),
        _ => return Ok(()),
    };

    let metadata = crate::simulation::metadata::resolve_requested_action_packet(
        &ctx.request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        &ctx.request.actors[owner].character_id,
        "GA_Oneiroi_Skill",
        effect,
        None,
        &ctx.state.execution_model_assumptions,
    )?;

    for _ in 0..count {
        for target_index in 0..ctx.request.targets.len() {
            if ctx.state.targets[target_index].hp_current == 0 {
                continue;
            }
            crate::runtime_api::increment_sequence(ctx.sequence)?;
            ctx.queue.push(ScheduledGameEvent {
                time_us: now,
                priority: CausalPriority::HitDelivery,
                seq: *ctx.sequence,
                data: GameEventData::HitDelivery {
                    command_index: 0,
                    actor_index: owner,
                    target_index,
                    ability_id: "GA_Oneiroi_Skill".into(),
                    damage_effect_id: effect.into(),
                    atk_rate: ntesim_core::formula::Ratio::one(),
                    scaling_stat: crate::simulation::NativeScalingStat::BaseAtk,
                    metadata: Box::new(metadata.clone()),
                },
            });
        }
    }

    if !ctx.state.actors[owner].character.iroi()?.dream_active {
        let moon_gain = match skill_number {
            1 => crate::simulation::ExactFraction::new(3, 1)?,
            2 => crate::simulation::ExactFraction::new(6, 1)?,
            _ => crate::simulation::ExactFraction::new(0, 1)?,
        };
        if !moon_gain.is_zero() {
            super::skill::grant_moon(
                &ctx.request.actors[owner],
                ctx.state.actors[owner].character.iroi_mut()?,
                moon_gain,
            )?;
        }
    }

    let next_tick = now
        .checked_add(cooldown)
        .ok_or("Lamb AI cooldown overflow")?;
    ctx.state.actors[owner]
        .character
        .iroi_mut()?
        .lamb_cooldowns
        .insert((ally_index, skill_number), next_tick);
    push_lamb_ai(
        ctx.queue,
        ctx.sequence,
        next_tick,
        ctx.request.absolute_limit_us,
        owner,
        ally_index,
        lamb_type,
        skill_number,
    )?;

    Ok(())
}

pub(crate) fn revive_regression(
    request: &crate::simulation::GameScenarioRequest,
    state: &mut crate::simulation::state::GameSimulatorState,
    iroi_actor_index: usize,
    target_ally_index: usize,
) -> Result<bool, String> {
    let a6 = request.actors[iroi_actor_index].awakening.has_effect(6);
    let iroi = state.actors[iroi_actor_index].character.iroi_mut()?;
    if !iroi.dream_active || !iroi.dream_is_ivory || iroi.revive_used_in_dream {
        return Ok(false);
    }

    if !iroi.regressed_allies.contains_key(&target_ally_index) {
        return Ok(false);
    }

    iroi.revive_used_in_dream = true;
    iroi.revived_ally = Some(target_ally_index);
    iroi.regressed_allies.remove(&target_ally_index);
    iroi.lamb_cooldowns.remove(&(target_ally_index, 1));
    iroi.lamb_cooldowns.remove(&(target_ally_index, 2));

    crate::runtime_api::record_hypothesis_id(
        request,
        &mut state.execution_model_assumptions,
        HYPOTHESIS_ID,
    )?;

    // Settle restored HP and resources
    // With A6: 100% own MaxHP + 50 Cycle + 50 conventional Ultimate Energy directly
    // Without A6: 10% own current MaxHP (rounded once, at least 1)
    let max_hp = state.actors[target_ally_index].hp_max;
    if a6 {
        let ally = &mut state.actors[target_ally_index];
        ally.hp = max_hp;
        ally.cycle_ledger.grant_direct(&mut ally.cycle, 50)?;
        ally.restore_energy(50)?;
    } else {
        let hp_10 = (max_hp as f64 * 0.1).round() as u64;
        state.actors[target_ally_index].hp = hp_10.max(1);
    }

    Ok(true)
}

pub(crate) fn re_regress_ally(
    ctx: &mut CallbackContext<'_>,
    iroi_actor_index: usize,
    ally_index: usize,
) -> Result<(), String> {
    let state = ctx.state.actors[iroi_actor_index].character.iroi_mut()?;
    let mut others: Vec<usize> = (0..ctx.request.actors.len())
        .filter(|&idx| idx != iroi_actor_index)
        .collect();
    others.sort_by_key(|&idx| ctx.request.actors[idx].slot);
    let order = others
        .iter()
        .position(|&idx| idx == ally_index)
        .unwrap_or(0);
    let lamb = lamb_type_for_slot(order);
    state.regressed_allies.insert(ally_index, lamb);

    let now = ctx.state.current_time_us;
    let (cd1, cd2) = match lamb {
        LambType::Gun => (1_500_000, 10_000_000),
        LambType::Syringe => (2_000_000, 9_000_000),
        LambType::Trumpet => (2_500_000, 8_500_000),
    };
    state
        .lamb_cooldowns
        .insert((ally_index, 1), now.saturating_add(cd1));
    state
        .lamb_cooldowns
        .insert((ally_index, 2), now.saturating_add(cd2));

    push_lamb_ai(
        ctx.queue,
        ctx.sequence,
        now.saturating_add(cd1),
        ctx.request.absolute_limit_us,
        iroi_actor_index,
        ally_index,
        lamb,
        1,
    )?;
    push_lamb_ai(
        ctx.queue,
        ctx.sequence,
        now.saturating_add(cd2),
        ctx.request.absolute_limit_us,
        iroi_actor_index,
        ally_index,
        lamb,
        2,
    )?;

    if ctx.state.active_actor_index == ally_index {
        ctx.state.active_actor_index = iroi_actor_index;
    }
    Ok(())
}
