use super::{EventContext, ExactFraction, REACTION_SOURCE, ReactionSnapshot, trace};
use crate::simulation::metadata::{
    ActionCategory, ActionMetadata, ActionPacketBinding, ActionPacketBindingKind,
    ActionPacketMetadata, DamageElement, InputActionRole, MetadataProvenance, PacketMetadata,
};
use crate::simulation::{GameEventPayload, GameHitReceipt};
use ntesim_core::buffs::dispatch::{
    BuffDefinition, BuffManager, BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy,
};
use ntesim_core::combat::DamageKind;
use std::collections::BTreeSet;

pub(super) const DURATION_US: u64 = 12_000_000;
pub(crate) const STATUS_TAG: &str = "ntesim.status.hexed";
pub(crate) const STATUS_ID: &str = "ntesim.reaction.hexed";
pub(crate) const NATIVE_STATUS_TAG: &str = "Ability.Player.Buff.Reaction.LingZhou";
pub(crate) const FOLLOWUP_EFFECT_ID: &str = "ntesim.reaction.hexed_followup";

pub(super) fn apply(ctx: &mut EventContext<'_>, snapshot: ReactionSnapshot) -> Result<(), String> {
    let target = ctx.request.aim_target(snapshot.command_index);
    ctx.state
        .buffs
        .apply_buff(
            BuffDefinition {
                stacking_scope: BuffStackingScope::Shared,
                source_lifetime: Default::default(),
                source: REACTION_SOURCE.into(),
                recipient: BuffRecipient::SpecificTarget(target.into()),
                id: STATUS_ID.into(),
                duration_us: Some(DURATION_US),
                stat_modifiers: Default::default(),
                semantic_tags: BTreeSet::from([
                    STATUS_TAG.into(),
                    "ntesim.reaction.hexed".into(),
                    NATIVE_STATUS_TAG.into(),
                ]),
                max_stacks: 1,
                refresh_policy: RefreshPolicy::Reset,
                decay_policy: DecayPolicy::AllAtOnce,
            },
            snapshot.owner_index,
            ctx.state.mode_clock.relative_us(),
        )
        .map_err(|error| format!("Hexed status application failed: {error:?}"))?;
    trace(
        ctx,
        "reaction_hexed_applied",
        Some(snapshot.owner_index),
        Some(GameEventPayload {
            target_index: Some(target),
            duration_us: Some(DURATION_US),
            ..Default::default()
        }),
    );
    Ok(())
}

pub(crate) fn clear_target(buffs: &mut BuffManager, target_index: usize) {
    buffs.remove_target_buff(STATUS_ID, REACTION_SOURCE, &target_index.to_string());
}

pub(crate) fn is_hexed_active(buffs: &BuffManager, target_index: usize, now_us: u64) -> bool {
    buffs
        .active_tagged_stacks_for_target(&target_index.to_string(), STATUS_TAG, now_us)
        .unwrap_or(0)
        > 0
}

/// Calculate Hexed follow-up multiplier h(CI) = 1.2 * (1 + 0.2 * CI / (CI + 180)) - 1.
pub(crate) fn hexed_factor(ci: ExactFraction) -> Result<ExactFraction, String> {
    let one = ExactFraction::new(1, 1)?;
    let efficiency = one.add(
        ci.mul(ExactFraction::new(1, 5)?)?
            .div(ci.add(ExactFraction::new(180, 1)?)?)?,
    )?;
    ExactFraction::new(6, 5)?.mul(efficiency)?.sub(one)
}

pub(crate) fn hexed_metadata(
    element: Option<DamageElement>,
) -> Result<ActionPacketMetadata, String> {
    let provenance = MetadataProvenance::AcceptedModelHypothesis {
        id: super::HYPOTHESIS_ID.as_str().into(),
        description: super::HYPOTHESIS_ID.description().into(),
    };
    let zero = ExactFraction::new(0, 1)?;
    Ok(ActionPacketMetadata {
        action: ActionMetadata {
            categories: BTreeSet::from([ActionCategory::Other]),
            gameplay_tags: vec!["State.Damage.Reaction".into()],
            ability_id: REACTION_SOURCE.into(),
            native_ability_index: None,
            input_role: Some(InputActionRole::Other),
            input_role_provenance: Some(provenance.clone()),
            primary_slot: None,
            source_evidence: None,
            identity_evidence: None,
            provenance: provenance.clone(),
        },
        packet: PacketMetadata {
            damage_effect_id: FOLLOWUP_EFFECT_ID.into(),
            native_ga_name: None,
            native_damage_type_ex: "EExecutionDamageTypeEX::DAMAGE_TYPE_REACTION".into(),
            native_damage_source_category: None,
            damage_kind: DamageKind::Elemental,
            element,
            fixed_crit_rate: Some(zero),
            fixed_crit_rate_provenance: Some(provenance.clone()),
            unbal_value: Some(zero),
            unbal_value_provenance: Some(provenance.clone()),
            semantic_tags: BTreeSet::from([
                "State.Damage.Reaction".into(),
                "hexed_followup".into(),
            ]),
            semantic_provenance: Some(provenance.clone()),
            source_evidence: None,
            provenance: provenance.clone(),
            packet_kind: crate::simulation::metadata::PacketClassification::Standard,
        },
        binding: ActionPacketBinding {
            energy_group_id: None,
            kind: ActionPacketBindingKind::OwnerChild,
            initiating_ability_id: REACTION_SOURCE.into(),
            packet_ga_name: None,
            provenance: Some(provenance),
        },
    })
}

pub(in crate::simulation) fn on_hp_damage(
    ctx: &mut EventContext<'_>,
    target_index: usize,
    actor_index: usize,
    command_index: usize,
    applied_hp_loss: u64,
    packet: &PacketMetadata,
    parent_hit_index: usize,
) -> Result<(), String> {
    if applied_hp_loss == 0 {
        return Ok(());
    }
    if packet.damage_effect_id == FOLLOWUP_EFFECT_ID
        || packet.semantic_tags.contains("hexed_followup")
    {
        return Ok(());
    }
    if !matches!(
        packet.element,
        Some(DamageElement::Nature | DamageElement::Incantation)
    ) {
        return Ok(());
    }
    let now_us = ctx.state.current_time_us;
    if !is_hexed_active(&ctx.state.buffs, target_index, now_us) {
        return Ok(());
    }
    if target_index >= ctx.state.targets.len() || ctx.state.targets[target_index].hp_current == 0 {
        return Ok(());
    }
    let living_sources: BTreeSet<usize> = ctx
        .state
        .actors
        .iter()
        .enumerate()
        .filter_map(|(index, actor)| (actor.hp > 0).then_some(index))
        .collect();
    let live_ci = ctx
        .state
        .buffs
        .get_stat_modifiers_for_actor_with_living_sources(
            actor_index,
            ctx.state.active_actor_index,
            ctx.state.mode_clock.relative_us(),
            &living_sources,
        )
        .ok()
        .and_then(|m| m.get("cycle_intensity").copied())
        .unwrap_or(crate::formula::Ratio::zero());
    let ci =
        ExactFraction::from_ratio(ctx.request.actors[actor_index].static_stats.cycle_intensity)?
            .add(ExactFraction::from_ratio(live_ci)?)?;
    let h_ci = hexed_factor(ci)?;
    let damage_exact = ExactFraction::new(u128::from(applied_hp_loss), 1)?.mul(h_ci)?;
    let rounded = damage_exact.round_u64()?;
    if rounded == 0 {
        return Ok(());
    }
    let target = &ctx.state.targets[target_index];
    let mut vitals = ntesim_core::state::VitalState::new(
        target.hp_current,
        target.hp_max,
        ntesim_core::state::ShieldOrder::OldestFirst,
    )
    .map_err(|error| format!("Hexed follow-up target vitals: {error:?}"))?;
    let outcome = vitals
        .damage(rounded)
        .map_err(|error| format!("Hexed follow-up HP settlement: {error:?}"))?;
    ctx.state.targets[target_index].hp_current = vitals.hp();
    ctx.state.hits_settled = ctx
        .state
        .hits_settled
        .checked_add(1)
        .ok_or("hit count overflow")?;
    ctx.state.add_rounded_damage(rounded)?;
    let metadata = hexed_metadata(packet.element.clone())?;
    crate::registry::observe_applied_hp(
        ctx.request,
        ctx.state,
        target_index,
        command_index,
        &metadata,
        outcome.hp_lost,
    )?;
    let (report_command_index, autonomous_event_index) =
        ctx.state.origins.identify(ctx.request, command_index)?;
    let zero = ExactFraction::new(0, 1)?;
    let switch_origin = ctx
        .hits
        .get(parent_hit_index)
        .ok_or("Hexed damage parent receipt missing")?
        .switch_origin
        .clone();
    let execution_model_assumption_ids = ctx.state.record_receipt_assumptions([(
        super::HYPOTHESIS_ID.as_str().into(),
        super::HYPOTHESIS_ID.description().into(),
    )])?;
    let (metadata_id, energy_group_id) = ctx.state.intern_action_packet_metadata(&metadata)?;
    ctx.hits.push(GameHitReceipt {
        active_actor_index: ctx.state.active_actor_index,
        parent_hit_index: Some(parent_hit_index),
        damage_base_override: None,
        autonomous_event_index,
        control_applications: Vec::new(),
        // Follow the damage parent, including Support/Cycle participants;
        // the Hexed status applier does not replace this causal ownership.
        switch_origin,
        break_contribution: None,
        execution_model_assumption_ids,
        command_index: report_command_index,
        time_us: now_us,
        actor_index,
        target_index,
        ability_id: REACTION_SOURCE.into(),
        damage_effect_id: FOLLOWUP_EFFECT_ID.into(),
        scaling_stat: super::super::NativeScalingStat::Atk,
        scaling_rate: crate::formula::Ratio::zero(),
        metadata_id,
        energy_group_id,
        static_general_damage_bonus: zero,
        static_element_damage_bonus: zero,
        static_mental_damage_bonus: zero,
        non_crit_damage: damage_exact,
        crit_damage: damage_exact,
        crit_rate: crate::formula::Ratio::zero(),
        critical: false,
        damage: damage_exact,
        rounded_damage: rounded,
        applied_hp_loss: outcome.hp_lost,
        crit_sample: None,
    });
    super::damage::trace_damage(
        ctx,
        ctx.hits.len() - 1,
        actor_index,
        Some(GameEventPayload {
            target_index: Some(target_index),
            parent_hit_index: Some(parent_hit_index),
            ..Default::default()
        }),
    );
    if outcome.hp_lost > 0 {
        crate::registry::observe_cycle_hit(
            &mut crate::runtime_api::CallbackContext {
                request: ctx.request,
                state: ctx.state,
                queue: ctx.queue,
                sequence: ctx.sequence,
                trace: ctx.trace,
            },
            ctx.hits.len() - 1,
            ctx.hits,
        )?;
    }
    if vitals.hp() == 0 {
        crate::registry::on_finish_target_defeated(
            &mut crate::runtime_api::CallbackContext {
                state: ctx.state,
                request: ctx.request,
                queue: ctx.queue,
                sequence: ctx.sequence,
                trace: ctx.trace,
            },
            actor_index,
            target_index,
        )?;
        super::clear_target_statuses(&mut ctx.state.buffs, target_index);
    }
    Ok(())
}
