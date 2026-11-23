use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReactionSnapshot {
    pub command_index: usize,
    pub switch_origin: GameSwitchOrigin,
    pub owner_index: usize,
    pub level: u32,
    pub cycle_intensity: ExactFraction,
    /// Winner element key (INCANTATION or CHAOS) selecting Scorch's RES channel.
    pub element_key: String,
    pub crit_damage: ExactFraction,
}

/// ExactFraction has no ordering: compare a/b > c/d through u128 products.
pub(in crate::simulation) fn fraction_greater(left: &ExactFraction, right: &ExactFraction) -> bool {
    left.numerator * right.denominator > right.numerator * left.denominator
}

pub(crate) fn element_from_key(key: &str) -> Option<ntesim_data::constants::character::Element> {
    use ntesim_data::constants::character::Element;
    Some(match key {
        "COSMOS" => Element::Cosmos,
        "NATURE" => Element::Nature,
        "INCANTATION" => Element::Incantation,
        "CHAOS" => Element::Chaos,
        "PSYCHE" => Element::Psyche,
        "LAKSHANA" => Element::Lakshana,
        _ => return None,
    })
}

pub(in crate::simulation) fn element_resistance(
    ctx: &EventContext<'_>,
    target_index: usize,
    element_key: &str,
) -> Result<ExactFraction, String> {
    let target = &ctx.request.targets[target_index];
    let resistance = target
        .elemental_resistances
        .get(element_key)
        .copied()
        .unwrap_or(target.resistance);
    let reduction = super::super::state::target_resistance_reduction(
        &ctx.state.buffs,
        target_index,
        &format!("ECharacterElementType::CHARACTER_ELEMENT_TYPE_{element_key}"),
    )?;
    resistance_factor(
        ExactFraction::from_ratio(resistance)?,
        reduction,
        NegativeResistanceModel::StandardRational,
    )
}

pub(in crate::simulation) fn intensity_factor(
    snapshot: &ReactionSnapshot,
) -> Result<ExactFraction, String> {
    // Q(CI) = 1 + CI/600 = (600 + CI) / 600.
    ExactFraction::new(600, 1)?
        .add(snapshot.cycle_intensity)?
        .try_div(ExactFraction::new(600, 1)?)
        .map_err(|error| format!("Cycle Intensity zone overflow: {error:?}"))
}

pub(in crate::simulation) fn reaction_metadata(
    effect_id: &str,
    damage_kind: DamageKind,
    element: Option<DamageElement>,
    damage_type_ex: &str,
) -> Result<ActionPacketMetadata, String> {
    let provenance = MetadataProvenance::AcceptedModelHypothesis {
        id: HYPOTHESIS_ID.as_str().into(),
        description: HYPOTHESIS_ID.description().into(),
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
            damage_effect_id: effect_id.into(),
            native_ga_name: None,
            native_damage_type_ex: damage_type_ex.into(),
            native_damage_source_category: None,
            damage_kind,
            element,
            fixed_crit_rate: Some(zero),
            fixed_crit_rate_provenance: Some(provenance.clone()),
            unbal_value: Some(zero),
            unbal_value_provenance: Some(provenance.clone()),
            semantic_tags: BTreeSet::from(["State.Damage.Reaction".into()]),
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

/// Shared trace identity for ordinary and triggered reaction damage.
pub(super) fn trace_damage(
    ctx: &mut EventContext<'_>,
    hit_index: usize,
    from_actor: usize,
    data: Option<GameEventPayload>,
) {
    if ctx.state.record_detail {
        ctx.trace.push(GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "ReactionOrBreak".into(),
            event_kind: "reaction_damage".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: Some(hit_index),
            from_actor: Some(from_actor),
            to_actor: None,
            energy_change: None,
            energy_fanout: None,
            data,
        });
    }
}

/// Exact reaction damage settled once against one living target, with
/// receipts and the shared death chain; never grants energy, Cycle or Break.
pub(in crate::simulation) fn settle_damage(
    ctx: &mut EventContext<'_>,
    target_index: usize,
    snapshot: &ReactionSnapshot,
    non_crit_damage: ExactFraction,
    metadata: ActionPacketMetadata,
    trace_data: GameEventPayload,
    critical: bool,
    crit_sample: Option<crate::rng::Sample>,
    crit_rate: crate::formula::Ratio,
) -> Result<(), String> {
    let target = &ctx.state.targets[target_index];
    if target.hp_current == 0 {
        return Ok(());
    }
    let crit_damage = if crit_rate.numerator == 0 {
        non_crit_damage
    } else {
        non_crit_damage.mul(ExactFraction::new(1, 1)?.add(snapshot.crit_damage)?)?
    };
    let damage = if critical {
        crit_damage
    } else {
        non_crit_damage
    };
    let effect_id = metadata.packet.damage_effect_id.clone();
    let ability_id = metadata.action.ability_id.clone();
    let mut assumptions = BTreeMap::from([(
        HYPOTHESIS_ID.as_str().into(),
        HYPOTHESIS_ID.description().into(),
    )]);
    if let MetadataProvenance::AcceptedModelHypothesis { id, description } =
        &metadata.packet.provenance
    {
        assumptions.insert(id.clone(), description.clone());
    }
    let mut vitals = ntesim_core::state::VitalState::new(
        target.hp_current,
        target.hp_max,
        ntesim_core::state::ShieldOrder::OldestFirst,
    )
    .map_err(|error| format!("reaction target vitals: {error:?}"))?;
    let rounded = damage.round_u64()?;
    let outcome = vitals
        .damage(rounded)
        .map_err(|error| format!("reaction HP settlement: {error:?}"))?;
    ctx.state.targets[target_index].hp_current = vitals.hp();
    crate::registry::observe_applied_hp(
        ctx.request,
        ctx.state,
        target_index,
        snapshot.command_index,
        &metadata,
        outcome.hp_lost,
    )?;
    ctx.state.hits_settled = ctx
        .state
        .hits_settled
        .checked_add(1)
        .ok_or("reaction hit count overflow")?;
    ctx.state.add_rounded_damage(rounded)?;
    let (report_command_index, autonomous_event_index) = ctx
        .state
        .origins
        .identify(ctx.request, snapshot.command_index)?;
    let execution_model_assumption_ids = ctx.state.record_receipt_assumptions(assumptions)?;
    let (metadata_id, energy_group_id) = ctx.state.intern_action_packet_metadata(&metadata)?;
    let parent_hit_index = ctx.hits.len();
    ctx.hits.push(GameHitReceipt {
        active_actor_index: ctx.state.active_actor_index,
        parent_hit_index: None,
        damage_base_override: None,
        autonomous_event_index,
        control_applications: Vec::new(),
        switch_origin: Some(snapshot.switch_origin.clone()),
        break_contribution: None,
        execution_model_assumption_ids,
        command_index: report_command_index,
        time_us: ctx.state.current_time_us,
        actor_index: snapshot.owner_index,
        target_index,
        ability_id: ability_id.clone(),
        damage_effect_id: effect_id.clone(),
        scaling_stat: super::super::NativeScalingStat::Atk,
        scaling_rate: crate::formula::Ratio::zero(),
        metadata_id,
        energy_group_id,
        static_general_damage_bonus: ExactFraction::new(0, 1)?,
        static_element_damage_bonus: ExactFraction::new(0, 1)?,
        static_mental_damage_bonus: ExactFraction::new(0, 1)?,
        non_crit_damage,
        crit_damage,
        crit_rate,
        critical,
        damage,
        rounded_damage: rounded,
        applied_hp_loss: outcome.hp_lost,
        crit_sample,
    });
    trace_damage(
        ctx,
        parent_hit_index,
        snapshot.owner_index,
        Some(trace_data),
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
            parent_hit_index,
            ctx.hits,
        )?;
    }
    if outcome.died {
        crate::registry::on_party_target_defeated(
            &mut crate::runtime_api::CallbackContext {
                state: ctx.state,
                request: ctx.request,
                queue: ctx.queue,
                sequence: ctx.sequence,
                trace: ctx.trace,
            },
            snapshot.owner_index,
            target_index,
        )?;
        crate::characters::team::on_target_defeated(
            ctx.request,
            ctx.state,
            target_index,
            ctx.trace,
        )?;
        for (config, actor) in ctx.request.actors.iter().zip(&mut ctx.state.actors) {
            crate::registry::on_target_defeated(config, actor, ctx.state.current_time_us)?;
        }
        crate::gear::on_target_defeated(
            &mut crate::runtime_api::CallbackContext {
                request: ctx.request,
                state: ctx.state,
                queue: ctx.queue,
                sequence: ctx.sequence,
                trace: ctx.trace,
            },
            snapshot.owner_index,
            target_index,
        )?;
        clear_target_statuses(&mut ctx.state.buffs, target_index);
    } else if outcome.hp_lost > 0 {
        super::hexed::on_hp_damage(
            ctx,
            target_index,
            snapshot.owner_index,
            snapshot.command_index,
            outcome.hp_lost,
            &metadata.packet,
            parent_hit_index,
        )?;
    }
    Ok(())
}
