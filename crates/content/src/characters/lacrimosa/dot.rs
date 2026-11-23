//! Lacrimosa Nightmare (blood) DoT scheduling model.
//!
//! Evidence: the local `Cau_LacrimosaBloodDam` probe table (additional
//! coefficient `clamp(count - 1, 0, 20)`, positive expansion bypasses the
//! addition), the serialized `GE_Player_Lacrimosa_Blood_Damage` (period 1.0s,
//! `NeverReset`, `RemoveSingleStackAndRefreshDuration`, 3s A0 duration,
//! `ratio_per_stack` 0.103 at skill 8, fixed crit 0.5), and
//! `DT_LacrimosaEffectFigure` (`BloodDur` 3.0, `BloodDurPas1` 6.0,
//! `BloodEStack`/`BloodQStack` 5, A4 extends to 6s).
//! See `docs/mechanics/characters/lacrimosa/nightmare-damage-and-duration.md`
//! and `reviews/q10-game-action-binding/lacrimosa-dot-scheduling-model.json`.
//!
//! The stack multiplier is applied in the coefficient-addition zone
//! (`k = k_native + sum(delta_k)`, per
//! `docs/mechanics/shared/damage-and-stat-zones.md`): each tick carries
//! the native per-stack ATK ratio, and `awakenings::damage_bonus` adds
//! `ratio * clamp(live_count - 1, 0, 20)` from the live count at tick time.
//! Ticks are pre-scheduled `HitDelivery` events; every reapply rebuilds the
//! pending train from the current aggregate state.

use crate::HypothesisId;
use crate::runtime_api::CastContext;
use crate::simulation::ExactFraction;
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use ntesim_core::execution::CausalPriority;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const BLOOD_DAMAGE_PACKET: &str = "GE_Player_Lacrimosa_Blood_Damage";
pub(crate) const BLOOD_STACKS_PER_CAST: u32 = 5;
pub(crate) const BLOOD_STACK_CAP: u32 = 20;
pub(crate) const BLOOD_TICK_PERIOD_US: u64 = 1_000_000;
pub(crate) const BLOOD_BASE_DURATION_US: u64 = 3_000_000;
pub(crate) const BLOOD_A4_DURATION_US: u64 = 6_000_000;

pub(crate) const DOT_SCHEDULING_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::LacrimosaNightmareDotSchedulingV1;
pub(crate) const ALL_TARGETS_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::LacrimosaNightmareAllTargetsV1;

/// Per-target Nightmare aggregate state, keyed by target index in `skill::State`.
#[derive(Clone, Debug)]
pub(crate) struct NightmareTargetState {
    carrier: Option<Box<NightmareApplication>>,
    pub stacks: u32,
    pub deadline_us: u64,
    pub duration_us: u64,
    pub grid_origin_us: u64,
    pub per_stack_ratio: ntesim_core::formula::Ratio,
    pub pending_tick_seqs: Vec<u64>,
}

/// Probe-table additional coefficient: `clamp(count - 1, 0, 20)`, bypassed
/// (zero) when the expansion term is positive.
pub(crate) fn additional_coefficient(stack_count: u32, expand_positive: bool) -> u32 {
    if expand_positive || stack_count == 0 {
        return 0;
    }
    (stack_count - 1).min(BLOOD_STACK_CAP)
}

/// Lazy `RemoveSingleStackAndRefreshDuration`: collapse every aggregate expiry
/// at or before `time_us`, removing one stack per elapsed duration. Expiry is
/// processed before a tick on coinciding timestamps.
pub(crate) fn effective_stacks(entry: &NightmareTargetState, time_us: u64) -> u32 {
    if entry.stacks == 0 || entry.duration_us == 0 || time_us < entry.grid_origin_us {
        return 0;
    }
    let mut stacks = entry.stacks;
    let mut deadline = entry.deadline_us;
    while stacks > 0 && deadline <= time_us {
        stacks -= 1;
        match deadline.checked_add(entry.duration_us) {
            Some(next) => deadline = next,
            None => break,
        }
    }
    stacks
}

/// Coefficient-addition term for a Blood tick settling at `time_us`:
/// `per_stack_ratio * clamp(live_count - 1, 0, 20)`. Zero when the target
/// holds no live stacks, so unmodeled packets are unaffected.
pub(crate) fn stack_ratio_add(
    nightmare: &BTreeMap<usize, NightmareTargetState>,
    target_index: usize,
    time_us: u64,
) -> Result<ExactFraction, String> {
    let Some(entry) = nightmare.get(&target_index) else {
        return ExactFraction::new(0, 1);
    };
    let additional = additional_coefficient(effective_stacks(entry, time_us), false);
    if additional == 0 {
        return ExactFraction::new(0, 1);
    }
    ExactFraction::from_ratio(entry.per_stack_ratio)?
        .mul(ExactFraction::new(u128::from(additional), 1)?)
}

fn accepted(request: &crate::simulation::GameScenarioRequest, id: HypothesisId) -> bool {
    request
        .model_assumptions
        .get(id.as_str())
        .map(String::as_str)
        == Some(id.description())
}

/// Apply Nightmare from an admitted Skill/Ult cast. Opt-in: without the exact
/// accepted scheduling declaration the legacy optional model is inactive.
/// The all-target extension binds recipients and delivery-time application.
/// Public Skill/Burst descriptors attach both accepted models automatically.
pub(crate) fn apply_nightmare(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let all_targets = accepted(ctx.request, ALL_TARGETS_HYPOTHESIS_ID);
    if !accepted(ctx.request, DOT_SCHEDULING_HYPOTHESIS_ID) {
        if all_targets {
            return Err("multi-target Nightmare requires its accepted DoT scheduling model".into());
        }
        return Ok(());
    }
    ctx.record_hypothesis(DOT_SCHEDULING_HYPOTHESIS_ID)?;
    if all_targets {
        ctx.record_hypothesis(ALL_TARGETS_HYPOTHESIS_ID)?;
    }
    let duration_us = if ctx.config.awakening.has_effect(4) {
        BLOOD_A4_DURATION_US
    } else {
        BLOOD_BASE_DURATION_US
    };

    let coordinate = if all_targets {
        normal_coordinate(ctx.config)?
    } else {
        match ctx.request.actions.get(ctx.command_index) {
            Some(crate::simulation::GameActionCommand::InvokeAbility { array_index, .. }) => {
                *array_index
            }
            _ => return Err("Lacrimosa Nightmare DoT requires an admitted ability command".into()),
        }
    };
    let (per_stack_ratio, scaling_stat) = project_blood(ctx, coordinate)?;
    let application = NightmareApplication {
        actor_index: ctx.actor_index,
        command_index: ctx.command_index,
        duration_us,
        per_stack_ratio,
        scaling_stat,
        metadata: Box::new(blood_metadata(ctx)?),
    };
    if all_targets {
        if ctx.ability_id == "GA_Lacrimosa_Skill" {
            if ctx
                .packet_metadata
                .map(|packet| packet.damage_effect_id.as_str())
                != Some("GE_Player_Lacrimosa_Skill1_Damage")
            {
                return Err("all-target Nightmare requires the native Skill primary packet".into());
            }
            for target_index in 0..ctx.targets.len() {
                if target_index == ctx.target_index {
                    continue;
                }
                if ctx.targets[target_index].hp_current > 0 {
                    ctx.schedule_native_child_packet(
                        "GE_Player_Lacrimosa_Skill1_Damage",
                        target_index,
                        ctx.hit_time_us,
                    )?;
                }
            }
        }
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: ctx.hit_time_us,
            priority: CausalPriority::ProcCallback,
            seq: *ctx.sequence,
            data: GameEventData::Content(crate::callbacks::ContentEvent::LacrimosaNightmareApply(
                application,
            )),
        });
        Ok(())
    } else {
        application.apply_target(
            ctx.actor.character.lacrimosa_mut()?,
            ctx.queue,
            ctx.sequence,
            0,
            ctx.hit_time_us,
            BLOOD_STACKS_PER_CAST,
        )
    }
}

fn normal_coordinate(config: &crate::simulation::GameActorConfig) -> Result<usize, String> {
    let progression = ntesim_data::skill_levels::progression(
        &config.character_id,
        config.character_level,
        config.character_max_level,
    )?;
    let breakthrough = progression["breakthrough"]
        .as_u64()
        .ok_or("Nightmare talent breakthrough missing")?;
    let purchased = config
        .talent_levels
        .map_or(config.purchased_skill_level, |levels| levels[0]);
    let level = crate::build::preview_projection::effective_skill_level(
        ntesim_data::skill_levels::frozen::tables(),
        &config.character_id,
        "GA_Lacrimosa_Melee",
        purchased,
        breakthrough
            .try_into()
            .map_err(|_| "Nightmare breakthrough exceeds u32")?,
        &config.awakening,
    )?;
    level["array_index"]
        .as_u64()
        .ok_or("Nightmare Normal coordinate missing")?
        .try_into()
        .map_err(|_| "Nightmare Normal coordinate exceeds usize".into())
}

fn project_blood(
    ctx: &CastContext<'_>,
    coordinate: usize,
) -> Result<
    (
        ntesim_core::formula::Ratio,
        crate::simulation::NativeScalingStat,
    ),
    String,
> {
    // The Blood row is filed under GA_Lacrimosa_Melee in DT_SkillDamageData;
    // the all-target model resolves its independent Normal coordinate, while
    // the legacy target-zero declaration retains its applying-cast coordinate.
    let projection = crate::build::project_native_packet_coefficient(
        ntesim_data::skill_levels::frozen::tables(),
        &crate::build::PacketCoefficientRequest {
            content_id: &ctx.request.content_id,
            character_id: &ctx.config.character_id,
            ability_id: "GA_Lacrimosa_Melee",
            damage_effect_id: BLOOD_DAMAGE_PACKET,
            array_index: coordinate
                .try_into()
                .map_err(|_| "Lacrimosa Nightmare DoT coordinate exceeds u32")?,
            research_metadata: None,
            model_assumptions: &ctx.request.model_assumptions,
        },
    )?;
    let per_stack_ratio: ntesim_core::formula::Ratio =
        serde_json::from_value(projection["compiled_rate"].clone())
            .map_err(|error| format!("Lacrimosa Nightmare DoT coefficient invalid: {error}"))?;
    let scaling_stat: crate::simulation::NativeScalingStat =
        serde_json::from_value(projection["compiled_scaling_stat"].clone())
            .map_err(|error| format!("Lacrimosa Nightmare DoT scaling stat invalid: {error}"))?;
    Ok((per_stack_ratio, scaling_stat))
}

fn blood_metadata(
    ctx: &CastContext<'_>,
) -> Result<crate::simulation::metadata::ActionPacketMetadata, String> {
    let native = ntesim_data::native_action_metadata::resolve_damage_effect(
        &ctx.request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        BLOOD_DAMAGE_PACKET,
    )?;
    let mut packet = crate::simulation::metadata::resolve_packet_metadata(
        &native,
        &ctx.request.model_assumptions,
    )?;
    packet
        .semantic_tags
        .insert(crate::characters::packet_semantics::DOT_DAMAGE_TAG.to_owned());
    let provenance = Some(
        crate::simulation::metadata::MetadataProvenance::AcceptedModelHypothesis {
            id: DOT_SCHEDULING_HYPOTHESIS_ID.as_str().into(),
            description: DOT_SCHEDULING_HYPOTHESIS_ID.description().into(),
        },
    );
    packet.semantic_provenance = provenance.clone();
    Ok(crate::simulation::metadata::ActionPacketMetadata {
        action: ctx.action_metadata.clone(),
        packet,
        binding: crate::simulation::metadata::ActionPacketBinding {
            energy_group_id: None,
            kind: crate::simulation::metadata::ActionPacketBindingKind::OwnerChild,
            initiating_ability_id: ctx.ability_id.to_owned(),
            packet_ga_name: native.ga_name.clone(),
            provenance,
        },
    })
}

fn replace_target(
    character: &mut super::skill::State,
    target_index: usize,
    apply_us: u64,
    duration_us: u64,
    per_stack_ratio: ntesim_core::formula::Ratio,
    added_stacks: u32,
) -> Result<(BTreeSet<u64>, Vec<u64>), String> {
    let stale: BTreeSet<u64> = character
        .nightmare
        .get(&target_index)
        .map(|entry| entry.pending_tick_seqs.iter().copied().collect())
        .unwrap_or_default();
    let live = character
        .nightmare
        .get(&target_index)
        .map(|entry| effective_stacks(entry, apply_us))
        .unwrap_or(0);
    let stacks = live.saturating_add(added_stacks).min(BLOOD_STACK_CAP);
    let grid_origin_us = match character.nightmare.get(&target_index) {
        Some(entry) if live > 0 => entry.grid_origin_us,
        _ => apply_us,
    };
    let entry = NightmareTargetState {
        carrier: None,
        stacks,
        deadline_us: apply_us
            .checked_add(duration_us)
            .ok_or("Lacrimosa Nightmare DoT deadline overflow")?,
        duration_us,
        grid_origin_us,
        per_stack_ratio,
        pending_tick_seqs: Vec::new(),
    };
    let mut plan = Vec::new();
    let mut tick_us = grid_origin_us
        .checked_add(BLOOD_TICK_PERIOD_US)
        .ok_or("Lacrimosa Nightmare DoT first tick overflow")?;
    loop {
        if tick_us > apply_us {
            if effective_stacks(&entry, tick_us) == 0 {
                break;
            }
            plan.push(tick_us);
        }
        tick_us = match tick_us.checked_add(BLOOD_TICK_PERIOD_US) {
            Some(next) => next,
            None => break,
        };
    }
    character.nightmare.insert(target_index, entry);
    Ok((stale, plan))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NightmareApplication {
    pub(crate) actor_index: usize,
    pub(crate) command_index: usize,
    duration_us: u64,
    per_stack_ratio: ntesim_core::formula::Ratio,
    scaling_stat: crate::simulation::NativeScalingStat,
    metadata: Box<crate::simulation::metadata::ActionPacketMetadata>,
}

impl NightmareApplication {
    fn apply_target(
        &self,
        character: &mut super::skill::State,
        queue: &mut BinaryHeap,
        sequence: &mut u64,
        target_index: usize,
        apply_us: u64,
        added_stacks: u32,
    ) -> Result<(), String> {
        let (stale_seqs, tick_plan) = replace_target(
            character,
            target_index,
            apply_us,
            self.duration_us,
            self.per_stack_ratio,
            added_stacks,
        )?;
        queue.retain(|event| !stale_seqs.contains(&event.seq));

        let mut pending_tick_seqs = Vec::with_capacity(tick_plan.len());
        for tick_us in tick_plan {
            crate::runtime_api::increment_sequence(sequence)?;
            queue.push(ScheduledGameEvent {
                time_us: tick_us,
                priority: CausalPriority::HitDelivery,
                seq: *sequence,
                data: GameEventData::HitDelivery {
                    command_index: self.command_index,
                    actor_index: self.actor_index,
                    target_index,
                    ability_id: self.metadata.action.ability_id.clone(),
                    damage_effect_id: BLOOD_DAMAGE_PACKET.to_owned(),
                    atk_rate: self.per_stack_ratio,
                    scaling_stat: self.scaling_stat,
                    metadata: self.metadata.clone(),
                },
            });
            pending_tick_seqs.push(*sequence);
        }
        character
            .nightmare
            .get_mut(&target_index)
            .ok_or("Lacrimosa Nightmare DoT state missing after apply")?
            .pending_tick_seqs = pending_tick_seqs;
        character
            .nightmare
            .get_mut(&target_index)
            .ok_or("Nightmare carrier missing")?
            .carrier = Some(Box::new(self.clone()));
        Ok(())
    }
}

pub(crate) fn apply(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    application: NightmareApplication,
) -> Result<(), String> {
    if ctx.state.actors[application.actor_index].hp == 0 {
        return Ok(());
    }
    for (target_index, target) in ctx.state.targets.iter().enumerate() {
        if target.hp_current == 0 {
            continue;
        }
        application.apply_target(
            ctx.state.actors[application.actor_index]
                .character
                .lacrimosa_mut()?,
            ctx.queue,
            ctx.sequence,
            target_index,
            ctx.state.current_time_us,
            BLOOD_STACKS_PER_CAST,
        )?;
    }
    Ok(())
}

pub(crate) fn spread_carrier(
    actor: &crate::simulation::state::ActorRuntime,
    target: usize,
    now: u64,
) -> Option<(NightmareApplication, u32)> {
    let crate::characters::RuntimeState::Lacrimosa(state) = &actor.character else {
        return None;
    };
    if actor.hp == 0 {
        return None;
    }
    let entry = state.nightmare.get(&target)?;
    let stacks = effective_stacks(entry, now);
    if stacks == 0 {
        return None;
    }
    Some(((**entry.carrier.as_ref()?).clone(), stacks))
}
pub(crate) fn spread(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    application: NightmareApplication,
    stacks: u32,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    for (target, actor) in ctx.state.targets.iter().enumerate() {
        if actor.hp_current == 0 {
            continue;
        }
        application.apply_target(
            ctx.state.actors[application.actor_index]
                .character
                .lacrimosa_mut()?,
            ctx.queue,
            ctx.sequence,
            target,
            now,
            stacks,
        )?;
    }
    Ok(())
}

pub(crate) fn extend_nightmare(
    character: &mut super::skill::State,
    target_index: usize,
    _queue: &mut BinaryHeap,
    _sequence: &mut u64,
    extend_us: u64,
    now_us: u64,
) -> Result<(), String> {
    if let Some(entry) = character.nightmare.get_mut(&target_index)
        && effective_stacks(entry, now_us) > 0
    {
        entry.deadline_us = entry.deadline_us.saturating_add(extend_us);
    }
    Ok(())
}

pub(crate) fn attached_dot_stacks(
    actor: &crate::simulation::state::ActorRuntime,
    target: usize,
    now: u64,
) -> u32 {
    match &actor.character {
        crate::characters::RuntimeState::Lacrimosa(state) => state
            .nightmare
            .get(&target)
            .map(|entry| effective_stacks(entry, now))
            .unwrap_or(0),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(
        stacks: u32,
        deadline_us: u64,
        duration_us: u64,
        grid_origin_us: u64,
    ) -> NightmareTargetState {
        NightmareTargetState {
            carrier: None,
            stacks,
            deadline_us,
            duration_us,
            grid_origin_us,
            per_stack_ratio: ntesim_core::formula::Ratio {
                numerator: 103,
                denominator: 1000,
            },
            pending_tick_seqs: Vec::new(),
        }
    }

    #[test]
    fn additional_coefficient_matches_probe_table() {
        // Cau_LacrimosaBloodDam probes: 0->0, 1->0, 3->2, 10->9, 30->20.
        for (count, expected) in [(0, 0), (1, 0), (3, 2), (10, 9), (30, 20), (21, 20)] {
            assert_eq!(
                additional_coefficient(count, false),
                expected,
                "count {count}"
            );
        }
        // Positive expansion bypasses the added coefficient.
        assert_eq!(additional_coefficient(10, true), 0);
    }

    #[test]
    fn effective_stacks_removes_one_per_duration_with_expiry_before_tick() {
        let state = entry(5, 3_000_000, 3_000_000, 0);
        // Before the first expiry the full 5 stacks are live.
        assert_eq!(effective_stacks(&state, 2_999_999), 5);
        // At the coinciding timestamp the expiry wins: 4 stacks at t=3.
        assert_eq!(effective_stacks(&state, 3_000_000), 4);
        assert_eq!(effective_stacks(&state, 5_999_999), 4);
        assert_eq!(effective_stacks(&state, 6_000_000), 3);
        assert_eq!(effective_stacks(&state, 12_000_000), 1);
        assert_eq!(effective_stacks(&state, 14_999_999), 1);
        // The last stack is removed at t=15; nothing ticks afterwards.
        assert_eq!(effective_stacks(&state, 15_000_000), 0);
        assert_eq!(effective_stacks(&state, 60_000_000), 0);
    }

    #[test]
    fn effective_stacks_collapses_expiries_before_reapply() {
        // 10 stacks applied at t=0 with deadline t=3; reapply lands at t=4.
        let state = entry(10, 3_000_000, 3_000_000, 0);
        assert_eq!(effective_stacks(&state, 4_000_000), 9);
    }
}
