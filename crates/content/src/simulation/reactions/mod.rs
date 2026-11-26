//! Esper Cycle reactions admitted by qualified adjacent-element switches
//! (B9/Q3). Damage reactions settle through the ordinary HP/death path with
//! exact arithmetic; statuses ride the shared BuffManager tags.

use super::context::EventContext;
use super::events::{GameEventData, ScheduledGameEvent};
use super::metadata::{
    ActionCategory, ActionMetadata, ActionPacketBinding, ActionPacketBindingKind,
    ActionPacketMetadata, DamageElement, InputActionRole, MetadataProvenance, PacketMetadata,
};
use super::{GameEventPayload, GameEventReceipt, GameHitReceipt, GameSwitchOrigin};
use crate::HypothesisId;
use crate::execution::CausalPriority;
use ntesim_core::buffs::dispatch::{
    BuffDefinition, BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy,
};
use ntesim_core::combat::exact::{ExactFraction, defense_factor, resistance_factor};
use ntesim_core::combat::{DamageKind, NegativeResistanceModel};
use std::collections::{BTreeMap, BTreeSet};

pub(super) const HYPOTHESIS_ID: HypothesisId = HypothesisId::CombatEsperCycleReactionsV1;
pub(crate) const STAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::CombatStainSwitchV1;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum ReactionKind {
    Blossom = 0,
    Hexed = 1,
    Scorch = 2,
    Nova = 3,
    Stain = 4,
    Remora = 5,
    Charge = 6,
    Discord = 7,
}

impl ReactionKind {
    pub const ALL: [ReactionKind; 8] = [
        ReactionKind::Blossom,
        ReactionKind::Hexed,
        ReactionKind::Scorch,
        ReactionKind::Nova,
        ReactionKind::Stain,
        ReactionKind::Remora,
        ReactionKind::Charge,
        ReactionKind::Discord,
    ];

    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn name(self) -> &'static str {
        match self {
            ReactionKind::Blossom => "Blossom",
            ReactionKind::Hexed => "Hexed",
            ReactionKind::Scorch => "Scorch",
            ReactionKind::Nova => "Nova",
            ReactionKind::Stain => "Stain",
            ReactionKind::Remora => "Remora",
            ReactionKind::Charge => "Charge",
            ReactionKind::Discord => "Discord",
        }
    }
}

impl From<ntesim_data::reaction_values::ReactionResult> for ReactionKind {
    fn from(result: ntesim_data::reaction_values::ReactionResult) -> Self {
        match result {
            ntesim_data::reaction_values::ReactionResult::Blossom => Self::Blossom,
            ntesim_data::reaction_values::ReactionResult::Hexed => Self::Hexed,
            ntesim_data::reaction_values::ReactionResult::Scorch => Self::Scorch,
            ntesim_data::reaction_values::ReactionResult::Nova => Self::Nova,
            ntesim_data::reaction_values::ReactionResult::Stain => Self::Stain,
            ntesim_data::reaction_values::ReactionResult::Remora => Self::Remora,
        }
    }
}

mod blossom;
pub(crate) use blossom::expire_bud;
mod damage;
pub(crate) mod hexed;
mod nova;
mod remora;
mod replacement;
mod scorch;
mod stain;

pub(crate) use blossom::{BLOSSOM_PARTY_BUD_CAP, BudGeneration, PistilEvent};
pub(super) use blossom::{create_blossom, settle_pistil};
pub(crate) use damage::{ReactionSnapshot, element_from_key};
pub(super) use damage::{
    element_resistance, fraction_greater, intensity_factor, reaction_metadata, settle_damage,
};
#[allow(unused_imports)]
pub(crate) use hexed::STATUS_TAG as HEXED_STATUS_TAG;
pub(super) use hexed::on_hp_damage as hexed_on_hp_damage;
pub(super) use nova::{apply as apply_nova, end as end_nova, settle as settle_nova};
pub(crate) use remora::STATUS_TAG as REMORA_STATUS_TAG;
pub(crate) use remora::apply_from_owner as apply_remora_from_owner;
pub(crate) use remora::apply_from_owner_with_retrigger_control as apply_remora_from_owner_with_retrigger_control;
pub(crate) use remora::cancel_target as cancel_target_remora;
pub(crate) use remora::command_for_target as remora_command;
pub(crate) use remora::crit_rate_bonus as remora_crit_rate_bonus;
pub(crate) use remora::discard_unobserved as discard_unobserved_remora_deadlines;
pub(in crate::simulation) use remora::end as end_remora;
#[cfg(test)]
pub(in crate::simulation) use remora::remove as remove_remora;
pub(super) use replacement::{on_dot_delivery, validate_dot_replacement};
pub(super) use scorch::{
    SCORCH_DURATION_US, apply_scorch, apply_scorch_with_application, refresh_scorch_from_word,
    settle_scorch_tick,
};
pub(crate) use scorch::{
    SCORCH_STATUS_ID, ScorchGeneration, ScorchTickEvent, spread_scorch_to_target,
};
pub(crate) use stain::STATUS_TAG as STAIN_STATUS_TAG;
pub(crate) use stain::damage_multiplier as stain_multiplier;

pub(crate) fn clear_target_statuses(
    buffs: &mut ntesim_core::buffs::dispatch::BuffManager,
    target_index: usize,
) {
    hexed::clear_target(buffs, target_index);
    stain::clear_target(buffs, target_index);
    remora::clear_target(buffs, target_index);
    buffs.remove_target_buff(SCORCH_STATUS_ID, REACTION_SOURCE, &target_index.to_string());
    crate::characters::adler::temperance::clear_target(buffs, target_index);
}

/// Nominal per-owner Esper Cycle pool capacity from the public mechanics
/// calculation; only registry-bound native packets accrue additional Cycle.
pub(crate) const CYCLE_CAPACITY: u64 = 100;

const SWITCH_CYCLE_COST: u64 = 100;
const NOVA_PACKET_DELAY_US: u64 = 5_000_000;
const NOVA_STATUS_US: u64 = 5_100_000;

const NOVA_STATUS_ID: &str = "ntesim.reaction.nova";
const REACTION_SOURCE: &str = "ntesim.reaction";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NovaEvent {
    pub target_index: usize,
    pub generation: u64,
    pub snapshot: ReactionSnapshot,
}

#[derive(Clone, Debug)]
pub(crate) struct NovaGeneration {
    pub generation: u64,
    pub snapshot: ReactionSnapshot,
    pub expires_at_us: u64,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ReactionRuntime {
    remora: BTreeMap<usize, remora::Generation>,
    seed: u64,
    rng: Option<crate::rng::Rng>,
    next_id: u64,
    buds: Vec<BudGeneration>,
    pub(crate) scorch: BTreeMap<usize, ScorchGeneration>,
    nova: BTreeMap<usize, NovaGeneration>,
    discord_pairs: BTreeSet<(usize, u64, u64)>,
    fired_novas: BTreeSet<u64>,
    ended_novas: BTreeSet<u64>,
    pub(crate) reaction_counts: [u32; 8],
}

impl ReactionRuntime {
    pub(crate) fn record_reaction(&mut self, kind: ReactionKind) {
        self.reaction_counts[kind as usize] = self.reaction_counts[kind as usize].saturating_add(1);
    }

    pub(crate) fn pause_world(&mut self, from: u64, delta: u64) -> Result<(), String> {
        use super::timer_state::{anchor, future};
        for scorch in self
            .scorch
            .values_mut()
            .filter(|state| state.expires_at_us > from)
        {
            anchor(&mut scorch.phase_origin_us, delta)?;
            anchor(&mut scorch.scheduled_until_us, delta)?;
            anchor(&mut scorch.expires_at_us, delta)?;
        }
        for nova in self.nova.values_mut() {
            future(&mut nova.expires_at_us, from, delta)?;
        }
        for remora in self.remora.values_mut() {
            future(&mut remora.expires_us, from, delta)?;
        }
        Ok(())
    }
}

/// Emit the single Discord callback for a live same-target Scorch/Nova pair.
/// The pair key uses both application generations, so refresh/reuse cannot
/// cause ticks or readiness checks to refire it.
pub(super) fn try_discord_pair(
    ctx: &mut EventContext<'_>,
    target_index: usize,
    source_actor: usize,
    command_index: usize,
) -> Result<(), String> {
    if ctx.state.targets[target_index].hp_current == 0 {
        return Ok(());
    }
    let now_us = ctx.state.current_time_us;
    let Some(scorch) = ctx
        .state
        .reactions
        .scorch
        .get(&target_index)
        .filter(|entry| entry.expires_at_us > now_us)
        .map(|entry| entry.discord_generation)
    else {
        return Ok(());
    };
    let Some(nova) = ctx
        .state
        .reactions
        .nova
        .get(&target_index)
        .filter(|entry| entry.expires_at_us > now_us)
        .map(|entry| entry.generation)
    else {
        return Ok(());
    };
    if !ctx
        .state
        .reactions
        .discord_pairs
        .insert((target_index, scorch, nova))
    {
        return Ok(());
    }

    ctx.state.reactions.record_reaction(ReactionKind::Discord);

    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        crate::HypothesisId::CombatDiscordBreakGaugeV1,
    )?;
    ctx.trace.push(GameEventReceipt {
        time_us: now_us,
        phase: "ReactionOrBreak".into(),
        event_kind: "reaction_discord_triggered".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(source_actor),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            target_index: Some(target_index),
            scorch_generation: Some(scorch),
            nova_generation: Some(nova),
            ..Default::default()
        }),
    });

    if ctx.state.targets[target_index].break_state.is_some() {
        let fraction = ctx.request.targets[target_index]
            .break_profile
            .as_ref()
            .and_then(|profile| profile.discord_fraction)
            .ok_or_else(|| {
                format!(
                    "Discord on breakable target {target_index} requires explicit discord_fraction in its break_profile"
                )
            })?;
        let fraction = ExactFraction::from_ratio(fraction)?;
        let cap_break = crate::registry::on_discord(
            &mut crate::runtime_api::CallbackContext {
                request: ctx.request,
                state: ctx.state,
                queue: ctx.queue,
                sequence: ctx.sequence,
                trace: ctx.trace,
            },
            source_actor,
            target_index,
            command_index,
        )?;
        let broke = crate::simulation::break_state::apply_discord_gauge_loss(
            ctx.request,
            ctx.state,
            ctx.queue,
            ctx.sequence,
            ctx.trace,
            source_actor,
            target_index,
            fraction,
        )?;
        if cap_break || broke {
            crate::simulation::break_damage::schedule(
                ctx.request,
                ctx.state,
                ctx.queue,
                ctx.sequence,
                command_index,
                target_index,
            )?;
        }
    }
    Ok(())
}

impl ReactionRuntime {
    pub(crate) fn buds(&self) -> &[BudGeneration] {
        &self.buds
    }
    pub(crate) fn supports_dot_replacement(&self, target_index: usize, now_us: u64) -> bool {
        self.scorch
            .get(&target_index)
            .is_some_and(|entry| entry.expires_at_us > now_us && entry.enhancement.is_some())
    }
    pub(super) fn new(seed: u64) -> Self {
        Self {
            seed,
            ..Self::default()
        }
    }

    fn reaction_rng(&mut self) -> Result<&mut crate::rng::Rng, String> {
        if self.rng.is_none() {
            self.rng = Some(crate::rng::Rng::new(
                self.seed,
                (0..4)
                    .map(|slot| (format!("scorch:{slot}"), 54 + slot))
                    .chain([(crate::characters::adler::temperance::RNG_STREAM.into(), 58)])
                    .collect(),
            )?);
        }
        Ok(self.rng.as_mut().expect("reaction RNG initialized"))
    }

    fn sample_scorch(&mut self, owner_index: usize) -> Result<crate::rng::Sample, String> {
        self.reaction_rng()?
            .uniform(&format!("scorch:{owner_index}"), 2)
    }

    pub(super) fn rng_audit(&self) -> Result<Option<serde_json::Value>, String> {
        self.rng
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(|error| error.to_string())
    }
}

fn schedule(ctx: &mut EventContext<'_>, time_us: u64, data: GameEventData) -> Result<(), String> {
    *ctx.sequence += 1;
    ctx.queue.push(ScheduledGameEvent {
        time_us,
        priority: CausalPriority::ReactionOrBreak,
        seq: *ctx.sequence,
        data,
    });
    Ok(())
}

fn buff_stat(
    ctx: &EventContext<'_>,
    actor_index: usize,
    key: &str,
) -> Result<ExactFraction, String> {
    let living_sources: BTreeSet<usize> = ctx
        .state
        .actors
        .iter()
        .enumerate()
        .filter_map(|(index, actor)| (actor.hp > 0).then_some(index))
        .collect();
    let buffs = ctx
        .state
        .buffs
        .get_stat_modifiers_for_actor_with_living_sources(
            actor_index,
            ctx.state.active_actor_index,
            ctx.state.mode_clock.relative_us(),
            &living_sources,
        )
        .map_err(|error| format!("reaction source modifiers failed: {error:?}"))?;
    ExactFraction::from_ratio(
        buffs
            .get(key)
            .copied()
            .unwrap_or(crate::formula::Ratio::zero()),
    )
}

fn cycle_intensity(ctx: &EventContext<'_>, actor_index: usize) -> Result<ExactFraction, String> {
    let config = &ctx.request.actors[actor_index];
    ExactFraction::from_ratio(config.static_stats.cycle_intensity)?.add(buff_stat(
        ctx,
        actor_index,
        "cycle_intensity",
    )?)
}

fn trace(
    ctx: &mut EventContext<'_>,
    event_kind: &str,
    from_actor: Option<usize>,
    data: Option<GameEventPayload>,
) {
    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ReactionOrBreak".into(),
        event_kind: event_kind.into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor,
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data,
    });
}

#[derive(Clone, Copy)]
pub(super) struct QualifiedSwitchReaction {
    kind: ntesim_data::reaction_values::ReactionResult,
    free_lacrimosa_entry: bool,
}

fn qualifying_switch_reaction(
    request: &super::GameScenarioRequest,
    state: &super::state::GameSimulatorState,
    old_actor: usize,
    new_actor: usize,
    target_index: usize,
) -> Result<Option<QualifiedSwitchReaction>, String> {
    if old_actor == new_actor || state.actors[old_actor].hp == 0 || state.actors[new_actor].hp == 0
    {
        return Ok(None);
    }
    let old_element =
        crate::registry::get_character_descriptor_by_id(&request.actors[old_actor].character_id)?
            .element;
    let new_element =
        crate::registry::get_character_descriptor_by_id(&request.actors[new_actor].character_id)?
            .element;
    let (Some(old_key), Some(new_key)) = (
        element_from_key(&old_element),
        element_from_key(&new_element),
    ) else {
        return Ok(None);
    };
    let result = match ntesim_data::reaction_values::pair_result(old_key, new_key) {
        Some(
            result @ (ntesim_data::reaction_values::ReactionResult::Blossom
            | ntesim_data::reaction_values::ReactionResult::Hexed
            | ntesim_data::reaction_values::ReactionResult::Scorch
            | ntesim_data::reaction_values::ReactionResult::Nova
            | ntesim_data::reaction_values::ReactionResult::Stain
            | ntesim_data::reaction_values::ReactionResult::Remora),
        ) => result,
        _ => return Ok(None),
    };
    if state
        .targets
        .get(target_index)
        .is_none_or(|target| target.hp_current == 0)
    {
        return Ok(None);
    }
    let free_lacrimosa_entry =
        matches!(
            result,
            ntesim_data::reaction_values::ReactionResult::Nova
                | ntesim_data::reaction_values::ReactionResult::Scorch
        ) && crate::characters::lacrimosa::support::ready(request, state, new_actor);
    if !free_lacrimosa_entry && state.actors[old_actor].cycle.current() < SWITCH_CYCLE_COST {
        return Ok(None);
    }
    Ok(Some(QualifiedSwitchReaction {
        kind: result,
        free_lacrimosa_entry,
    }))
}

fn record_switch_models(
    request: &super::GameScenarioRequest,
    state: &super::state::GameSimulatorState,
    reaction: ntesim_data::reaction_values::ReactionResult,
    target_index: usize,
    assumptions: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    crate::runtime_api::record_hypothesis_id(request, assumptions, HYPOTHESIS_ID)?;
    if reaction == ntesim_data::reaction_values::ReactionResult::Scorch {
        crate::registry::scorch_enhancement(request, state, assumptions)?;
    }
    if reaction == ntesim_data::reaction_values::ReactionResult::Nova {
        crate::registry::validate_nova_extensions(request, state, target_index, assumptions)?;
    }
    if reaction == ntesim_data::reaction_values::ReactionResult::Stain {
        crate::runtime_api::record_hypothesis_id(request, assumptions, STAIN_HYPOTHESIS_ID)?;
        crate::registry::stain_enhancement(request, state, assumptions)?;
    }
    if reaction == ntesim_data::reaction_values::ReactionResult::Remora {
        crate::runtime_api::record_hypothesis_id(
            request,
            assumptions,
            HypothesisId::CombatRemoraChargeV1,
        )?;
        crate::registry::remora_enhancement(request, state, assumptions)?;
        crate::registry::validate_remora_extensions(request, state, assumptions)?;
    }
    Ok(())
}

pub(super) fn validate_on_switch(
    request: &super::GameScenarioRequest,
    state: &super::state::GameSimulatorState,
    old_actor: usize,
    new_actor: usize,
    target_index: usize,
) -> Result<Option<QualifiedSwitchReaction>, String> {
    let reaction = qualifying_switch_reaction(request, state, old_actor, new_actor, target_index)?;
    if let Some(qualified) = reaction {
        let result = qualified.kind;
        record_switch_models(request, state, result, target_index, &mut BTreeMap::new())?;
        validate_discord_pair_admission(request, state, result, target_index)?;
        let duration = match result {
            ntesim_data::reaction_values::ReactionResult::Blossom => {
                u64::from(blossom::BLOSSOM_PISTIL_COUNT) * blossom::BLOSSOM_PISTIL_INTERVAL_US
            }
            ntesim_data::reaction_values::ReactionResult::Hexed => hexed::DURATION_US,
            ntesim_data::reaction_values::ReactionResult::Scorch => scorch::SCORCH_DURATION_US,
            ntesim_data::reaction_values::ReactionResult::Nova => {
                if crate::registry::validate_nova_extensions(
                    request,
                    state,
                    target_index,
                    &mut BTreeMap::new(),
                )? {
                    2 * NOVA_STATUS_US
                } else {
                    NOVA_STATUS_US
                }
            }
            ntesim_data::reaction_values::ReactionResult::Stain => {
                crate::registry::stain_enhancement(request, state, &mut BTreeMap::new())?
                    .map_or(stain::DURATION_US, |policy| policy.duration_us)
            }
            ntesim_data::reaction_values::ReactionResult::Remora => {
                crate::registry::remora_enhancement(request, state, &mut BTreeMap::new())?
                    .map_or(remora::DURATION_US, |policy| policy.duration_us)
            }
        };
        state
            .current_time_us
            .checked_add(duration)
            .ok_or("reaction deadline overflow")?;
    }
    Ok(reaction)
}

/// Validate every model/input that Discord would reach before switch admission
/// mutates field ownership, Cycle, statuses or reaction generations.
fn validate_discord_pair_admission(
    request: &super::GameScenarioRequest,
    state: &super::state::GameSimulatorState,
    reaction: ntesim_data::reaction_values::ReactionResult,
    target_index: usize,
) -> Result<(), String> {
    let Some(target) = state.targets.get(target_index) else {
        return Ok(());
    };
    if target.hp_current == 0 {
        return Ok(());
    }
    let now_us = state.current_time_us;
    let live_scorch = state
        .reactions
        .scorch
        .get(&target_index)
        .filter(|entry| entry.expires_at_us > now_us);
    let live_nova = state
        .reactions
        .nova
        .get(&target_index)
        .filter(|entry| entry.expires_at_us > now_us);
    let creates_new_pair = match reaction {
        // A qualified Scorch application receives a fresh pair identity even
        // when its existing generation keeps the prior tick phase.
        ntesim_data::reaction_values::ReactionResult::Scorch => live_nova.is_some(),
        // Nova always receives a fresh generation when admitted.
        ntesim_data::reaction_values::ReactionResult::Nova => live_scorch.is_some(),
        _ => false,
    };
    if !creates_new_pair {
        return Ok(());
    }

    let mut assumptions = BTreeMap::new();
    crate::runtime_api::record_hypothesis_id(
        request,
        &mut assumptions,
        crate::HypothesisId::CombatDiscordBreakGaugeV1,
    )?;
    if target.break_state.is_some() {
        if request.targets[target_index]
            .break_profile
            .as_ref()
            .and_then(|profile| profile.discord_fraction)
            .is_none()
        {
            return Err(format!(
                "Discord on breakable target {target_index} requires explicit discord_fraction in its break_profile"
            ));
        }
        crate::registry::validate_discord_extensions(request, state, &mut assumptions)?;
    }
    Ok(())
}

/// Qualified switches snapshot higher-CI ownership, admit equipment windows,
/// spend outgoing Cycle and create the reaction on the switch's locked aim.
pub(super) fn admit_on_switch(
    ctx: &mut EventContext<'_>,
    command_index: usize,
    old_actor: usize,
    new_actor: usize,
    reaction: Option<QualifiedSwitchReaction>,
) -> Result<(), String> {
    let Some(qualified) = reaction else {
        return Ok(());
    };
    let result = qualified.kind;
    let request = ctx.request;
    let mut assumptions = BTreeMap::new();
    record_switch_models(
        request,
        ctx.state,
        result,
        request.aim_target(command_index),
        &mut assumptions,
    )?;
    let old_ci = cycle_intensity(ctx, old_actor)?;
    let new_ci = cycle_intensity(ctx, new_actor)?;
    let owner_index = if fraction_greater(&new_ci, &old_ci) {
        new_actor
    } else {
        old_actor
    };
    let owner_config = &request.actors[owner_index];
    let mut snapshot = ReactionSnapshot {
        command_index,
        switch_origin: GameSwitchOrigin::EsperCycle {
            outgoing_actor_index: old_actor,
            incoming_actor_index: new_actor,
        },
        owner_index,
        level: owner_config.character_level,
        cycle_intensity: if owner_index == old_actor {
            old_ci
        } else {
            new_ci
        },
        element_key: crate::registry::get_character_descriptor_by_id(&owner_config.character_id)?
            .element,
        crit_damage: ExactFraction::from_ratio(owner_config.static_stats.crit_damage_bonus)?
            .add(buff_stat(ctx, owner_index, "crit_damage_bonus")?)?,
    };
    if result == ntesim_data::reaction_values::ReactionResult::Scorch {
        snapshot.crit_damage =
            snapshot
                .crit_damage
                .add(buff_stat(ctx, owner_index, "crit_damage_bonus_dot")?)?;
    }
    // Participation windows do not change the already captured CI owner.
    crate::gear::on_reaction_participation(
        &mut crate::runtime_api::CallbackContext {
            state: &mut *ctx.state,
            request,
            queue: &mut *ctx.queue,
            sequence: &mut *ctx.sequence,
            trace: &mut *ctx.trace,
        },
        [old_actor, new_actor],
        &result,
    )?;
    crate::characters::zankou::skill::on_reaction_participation(
        &mut crate::runtime_api::CallbackContext {
            state: &mut *ctx.state,
            request,
            queue: &mut *ctx.queue,
            sequence: &mut *ctx.sequence,
            trace: &mut *ctx.trace,
        },
        [old_actor, new_actor],
        &result,
    )?;
    for &actor_idx in &[old_actor, new_actor] {
        if actor_idx < request.actors.len() {
            let config = &request.actors[actor_idx];
            if config.character_id == "character:1019"
                && config.awakening.has_resonance_3()
                && ctx.state.actors[actor_idx].hp > 0
                && matches!(
                    result,
                    ntesim_data::reaction_values::ReactionResult::Blossom
                        | ntesim_data::reaction_values::ReactionResult::Hexed
                )
            {
                crate::characters::mint::awakenings::apply_a3_attendance_bonus(
                    &mut ctx.state.buffs,
                    actor_idx,
                    ctx.state.mode_clock.relative_us(),
                )?;
                ctx.trace.push(GameEventReceipt {
                    time_us: ctx.state.current_time_us,
                    phase: "ReactionOrBreak".into(),
                    event_kind: "mint_a3_attendance_bonus_applied".into(),
                    active_actor: ctx.state.active_actor_index,
                    hit_index: None,
                    from_actor: Some(actor_idx),
                    to_actor: Some(actor_idx),
                    energy_change: None,
                    energy_fanout: None,
                    data: Some(GameEventPayload {
                        reaction: Some(format!("{result:?}")),
                        atk_percent: Some(ExactFraction::new(3, 20)?),
                        duration_us: Some(15_000_000),
                        ..Default::default()
                    }),
                });
            }
        }
    }
    if qualified.free_lacrimosa_entry {
        crate::characters::lacrimosa::support::consume(
            &mut crate::runtime_api::CallbackContext {
                state: ctx.state,
                request,
                queue: ctx.queue,
                sequence: ctx.sequence,
                trace: ctx.trace,
            },
            new_actor,
        )?;
    } else {
        ctx.state.actors[old_actor]
            .cycle
            .spend(SWITCH_CYCLE_COST)
            .map_err(|error| format!("Cycle spend failed: {error:?}"))?;
        ctx.state.actors[old_actor]
            .cycle_ledger
            .spend(SWITCH_CYCLE_COST)?;
    }
    if matches!(
        result,
        ntesim_data::reaction_values::ReactionResult::Blossom
            | ntesim_data::reaction_values::ReactionResult::Remora
    ) {
        crate::characters::zero::callback::on_paid_cosmos_cycle(
            &mut crate::runtime_api::CallbackContext {
                state: &mut *ctx.state,
                request,
                queue: &mut *ctx.queue,
                sequence: &mut *ctx.sequence,
                trace: &mut *ctx.trace,
            },
            old_actor,
        )?;
    }
    ctx.state.execution_model_assumptions.extend(assumptions);
    let now_us = ctx.state.current_time_us;
    ctx.state.reactions.record_reaction(result.into());
    match result {
        ntesim_data::reaction_values::ReactionResult::Blossom => {
            create_blossom(ctx, now_us, snapshot)
        }
        ntesim_data::reaction_values::ReactionResult::Scorch => {
            apply_scorch(ctx, now_us, request.aim_target(command_index), snapshot)
        }
        ntesim_data::reaction_values::ReactionResult::Nova => {
            apply_nova(ctx, now_us, request.aim_target(command_index), snapshot)
        }
        ntesim_data::reaction_values::ReactionResult::Stain => stain::apply(ctx, snapshot),
        ntesim_data::reaction_values::ReactionResult::Remora => remora::apply(ctx, snapshot),
        ntesim_data::reaction_values::ReactionResult::Hexed => hexed::apply(ctx, snapshot),
    }
}

pub(in crate::simulation) fn execute_sheep_switch_reaction(
    ctx: &mut EventContext<'_>,
    command_index: usize,
    old_actor: usize,
    target_actor: usize,
    result: ntesim_data::reaction_values::ReactionResult,
) -> Result<(), String> {
    let request = ctx.request;
    let mut assumptions = BTreeMap::new();
    record_switch_models(
        request,
        ctx.state,
        result,
        request.aim_target(command_index),
        &mut assumptions,
    )?;
    let old_ci = cycle_intensity(ctx, old_actor)?;
    let snapshot = ReactionSnapshot {
        command_index,
        switch_origin: GameSwitchOrigin::EsperCycle {
            outgoing_actor_index: old_actor,
            incoming_actor_index: target_actor,
        },
        owner_index: old_actor,
        level: request.actors[old_actor].character_level,
        cycle_intensity: old_ci,
        element_key: "NATURE".into(),
        crit_damage: ExactFraction::from_ratio(
            request.actors[old_actor].static_stats.crit_damage_bonus,
        )?
        .add(buff_stat(ctx, old_actor, "crit_damage_bonus")?)?,
    };
    ctx.state.execution_model_assumptions.extend(assumptions);
    let now_us = ctx.state.current_time_us;
    ctx.state.reactions.record_reaction(result.into());
    match result {
        ntesim_data::reaction_values::ReactionResult::Blossom => {
            create_blossom(ctx, now_us, snapshot)
        }
        ntesim_data::reaction_values::ReactionResult::Hexed => hexed::apply(ctx, snapshot),
        _ => Ok(()),
    }
}

pub(super) fn apply_status(
    ctx: &mut EventContext<'_>,
    target_index: usize,
    owner_index: usize,
    id: &str,
    tags: BTreeSet<String>,
    duration_us: u64,
) -> Result<(), String> {
    apply_status_with_cap(ctx, target_index, owner_index, id, tags, duration_us, 1)
}

pub(super) fn apply_status_with_cap(
    ctx: &mut EventContext<'_>,
    target_index: usize,
    owner_index: usize,
    id: &str,
    tags: BTreeSet<String>,
    duration_us: u64,
    stack_cap: u32,
) -> Result<(), String> {
    ctx.state
        .buffs
        .apply_buff(
            BuffDefinition {
                stacking_scope: BuffStackingScope::Shared,
                source_lifetime: Default::default(),
                source: REACTION_SOURCE.into(),
                recipient: BuffRecipient::SpecificTarget(target_index.into()),
                id: id.into(),
                duration_us: Some(duration_us),
                stat_modifiers: BTreeMap::new(),
                semantic_tags: tags,
                max_stacks: stack_cap,
                refresh_policy: RefreshPolicy::Reset,
                decay_policy: DecayPolicy::AllAtOnce,
            },
            owner_index,
            ctx.state.mode_clock.relative_us(),
        )
        .map_err(|error| format!("reaction status application failed: {error:?}"))
}
