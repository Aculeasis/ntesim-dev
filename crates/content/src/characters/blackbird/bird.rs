//! One source/cast controller, with target marks and a shared count/deadline.
use crate::runtime_api::{ActorHitContext, CallbackContext, CastContext};
use crate::simulation::{
    GameCausalTrigger, GameHitReceipt, GameScenarioRequest,
    events::{GameEventData, ScheduledGameEvent},
};
use std::collections::{BTreeMap, BTreeSet};

mod packet;
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::BlackbirdCoordinatedBirdV1;
pub(crate) const ABILITY: &str = "GA_BlackBird_Skill";
pub(crate) const PACKET: &str = "GE_Player_BlackBird_SkillAssist_Damage";

#[derive(Clone, Debug, Default)]
pub(crate) struct Controller {
    generation: u64,
    command: Option<usize>,
    coordinate: usize,
    marks: BTreeMap<usize, u64>,
    accepted: u8,
    ready_us: u64,
    packets: BTreeSet<String>,
}

pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}

pub(crate) fn on_skill(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if !accepted(ctx.request) {
        return Ok(());
    }
    if !super::malice::accepted(ctx.request) {
        return Err("Blackbird bird admission requires blackbird.ordinary_malice_v1".into());
    }
    ctx.record_hypothesis(MODEL)?;
    let bird = &mut ctx.actor.character.blackbird_mut()?.bird;
    if bird.command == Some(ctx.command_index) {
        return Ok(());
    }
    *bird = Controller {
        generation: bird
            .generation
            .checked_add(1)
            .ok_or("Blackbird bird generation overflow")?,
        command: Some(ctx.command_index),
        coordinate: ctx
            .native_array_index
            .ok_or("Blackbird bird requires invested Skill coordinate")?,
        ready_us: ctx.time_us,
        ..Default::default()
    };
    Ok(())
}

pub(crate) fn on_hit(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    if !accepted(ctx.request)
        || ctx.applied_hp_loss == 0
        || !ctx.target_alive
        || ctx.damage_effect_id != "GE_Player_BlackBird_Skill_Damage"
    {
        return Ok(());
    }
    let bird = &mut ctx.actor.character.blackbird_mut()?.bird;
    if bird.command == Some(ctx.command_index) {
        bird.marks.insert(
            ctx.target_index,
            ctx.time_us
                .checked_add(15_000_000)
                .ok_or("Blackbird mark expiry overflow")?,
        );
    }
    Ok(())
}

pub(crate) fn observe(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    index: usize,
    hits: &[GameHitReceipt],
) -> Result<(), String> {
    if !accepted(ctx.request) || ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }
    let hit = hits
        .get(index)
        .ok_or("Blackbird bird trigger has no settled hit")?;
    let now = ctx.state.current_time_us;
    if hit.actor_index == owner
        || hit.actor_index != ctx.state.active_actor_index
        || ctx.state.actors[hit.actor_index].hp == 0
        || hit.applied_hp_loss == 0
        || ctx.state.targets[hit.target_index].hp_current == 0
    {
        return Ok(());
    }
    let identity = hit
        .energy_group_id
        .clone()
        .unwrap_or_else(|| format!("hit:{index}"));
    let bird = &mut ctx.state.actors[owner].character.blackbird_mut()?.bird;
    let Some(command) = bird.command else {
        return Ok(());
    };
    if bird.accepted >= 5
        || bird.ready_us > now
        || bird.packets.contains(&identity)
        || bird
            .marks
            .get(&hit.target_index)
            .is_none_or(|&expiry| expiry <= now)
    {
        return Ok(());
    }
    let deadline = now
        .checked_add(2_000_000)
        .ok_or("Blackbird bird deadline overflow")?;
    let generation = bird.generation;
    // Validate both real parents before consuming the controller opportunity.
    let key = ctx.state.origins.register_from_hit(
        ctx.request,
        crate::simulation::GameAutonomousEventReceipt {
            actor_index: owner,
            active_actor_index: hit.actor_index,
            ability_id: ABILITY.into(),
            time_us: now,
            trigger: GameCausalTrigger::CoordinatedHit {
                cast_command_index: command,
                hit_index: index,
            },
            recipient_healing: None,
        },
        hits,
    )?;
    let bird = &mut ctx.state.actors[owner].character.blackbird_mut()?.bird;
    bird.accepted += 1;
    bird.ready_us = deadline;
    bird.packets.insert(identity);
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: now,
        priority: ntesim_core::execution::CausalPriority::ProcCallback,
        seq: *ctx.sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::Blackbird(
            super::events::Event::Bird {
                actor_index: owner,
                generation,
                key,
                target_index: hit.target_index,
            },
        )),
    });
    Ok(())
}

pub(crate) fn deliver(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    generation: u64,
    key: usize,
    target: usize,
) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 || ctx.state.targets[target].hp_current == 0 {
        return Ok(());
    }
    let bird = &ctx.state.actors[owner].character.blackbird_mut()?.bird;
    if bird.generation != generation {
        return Ok(());
    }
    let coordinate = bird.coordinate;
    packet::enqueue(ctx, owner, key, coordinate)
}

impl Controller {
    pub(crate) fn pause_world(&mut self, from: u64, delta: u64) -> Result<(), String> {
        crate::simulation::timer_state::future(&mut self.ready_us, from, delta)?;
        for expiry in self.marks.values_mut() {
            crate::simulation::timer_state::future(expiry, from, delta)?;
        }
        Ok(())
    }
}
