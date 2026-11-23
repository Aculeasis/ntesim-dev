use super::super::reactions::stain_multiplier;
use super::*;

pub(super) struct CalculatedHit {
    pub damage_base_override: Option<ExactFraction>,
    pub non_crit: ExactFraction,
    pub crit_damage: ExactFraction,
    pub crit_rate: crate::formula::Ratio,
    pub crit_eligible: bool,
    pub static_element_bonus: ExactFraction,
    pub static_mental_bonus: ExactFraction,
    pub assumptions: std::collections::BTreeMap<String, String>,
}

pub(crate) use super::scaling::effective_def;

fn critical_parameters(
    ctx: &crate::runtime_api::DamageContext<'_>,
    packet: &crate::simulation::metadata::PacketMetadata,
    modifiers: &crate::runtime_api::HitModifiers,
) -> Result<(ExactFraction, ExactFraction), String> {
    let actor_config = ctx.config;
    let buff_stat = |key: &str| -> Result<ExactFraction, String> {
        ctx.buff_stats
            .get(key)
            .copied()
            .map(ExactFraction::from_ratio)
            .unwrap_or_else(|| ExactFraction::new(0, 1))
    };
    let base_crit_bonus = ExactFraction::from_ratio(actor_config.static_stats.crit_damage_bonus)?
        .add(buff_stat("crit_damage_bonus")?)?;
    let kit_crit_bonus = crate::registry::crit_bonus(ctx)?;
    let packet_dot_crit_bonus = if packet
        .semantic_tags
        .contains(crate::characters::packet_semantics::DOT_DAMAGE_TAG)
    {
        buff_stat("crit_damage_bonus_dot")?
    } else {
        ExactFraction::new(0, 1)?
    };
    let total_crit_bonus = base_crit_bonus
        .add(kit_crit_bonus)?
        .add(packet_dot_crit_bonus)?
        .add(modifiers.crit_damage_bonus)?;
    let base_cr = ExactFraction::from_ratio(actor_config.static_stats.crit_rate)?
        .add(buff_stat("crit_rate_bonus")?)?;
    let kit_cr = crate::registry::crit_rate_bonus(ctx)?;
    let gear_cr = crate::gear::crit_rate_bonus(ctx)?;
    let proposed_cr = match packet.fixed_crit_rate {
        Some(rate) => rate,
        None => base_cr
            .add(kit_cr)?
            .add(gear_cr)?
            .add(super::super::reactions::remora_crit_rate_bonus(ctx)?)?,
    };
    Ok((total_crit_bonus, proposed_cr))
}

fn live_buff_stats<'a>(
    state: &crate::simulation::state::GameSimulatorState,
    actor_index: usize,
    buffs: &'a ntesim_core::buffs::dispatch::BuffManager,
) -> Result<Vec<(&'a str, crate::formula::Ratio)>, String> {
    let all_alive = state.actors.iter().all(|source| source.hp > 0);
    let n_actors = state.actors.len();
    if all_alive && n_actors <= 4 {
        static ALL_LIVING_4: std::sync::LazyLock<[std::collections::BTreeSet<usize>; 5]> =
            std::sync::LazyLock::new(|| {
                [
                    std::collections::BTreeSet::new(),
                    std::collections::BTreeSet::from([0]),
                    std::collections::BTreeSet::from([0, 1]),
                    std::collections::BTreeSet::from([0, 1, 2]),
                    std::collections::BTreeSet::from([0, 1, 2, 3]),
                ]
            });
        buffs
            .get_stat_modifiers_for_actor_with_living_sources_borrowed(
                actor_index,
                state.active_actor_index,
                state.mode_clock.relative_us(),
                &ALL_LIVING_4[n_actors],
            )
            .map_err(|error| format!("buff query failed: {error:?}"))
    } else {
        let living_sources = state
            .actors
            .iter()
            .enumerate()
            .filter_map(|(index, source)| (source.hp > 0).then_some(index))
            .collect::<std::collections::BTreeSet<_>>();
        buffs
            .get_stat_modifiers_for_actor_with_living_sources_borrowed(
                actor_index,
                state.active_actor_index,
                state.mode_clock.relative_us(),
                &living_sources,
            )
            .map_err(|error| format!("buff query failed: {error:?}"))
    }
}

use crate::simulation::metadata::DamageElement;

type DamageBonuses = (ExactFraction, ExactFraction, ExactFraction);

#[derive(Debug)]
pub(crate) struct PreparedDamageBonuses([Result<DamageBonuses, String>; 14]);

impl PreparedDamageBonuses {
    pub(crate) fn new(actor: &crate::simulation::GameActorConfig) -> Self {
        let descriptor = crate::registry::character_descriptor_by_id(&actor.character_id);
        Self(std::array::from_fn(|index| {
            let descriptor = descriptor.as_ref().map_err(|error| (*error).clone())?;
            static_damage_bonuses(
                &actor.static_stats,
                DamageElement::ACTOR_ELEMENTS[index / 2],
                &descriptor.element,
                index % 2 == 1,
            )
        }))
    }

    fn get(&self, element: Option<&DamageElement>, mental: bool) -> Result<DamageBonuses, String> {
        let index = element.map_or(0, DamageElement::actor_element_index) * 2 + usize::from(mental);
        self.0[index].clone()
    }
}

fn static_damage_bonuses(
    stats: &crate::simulation::GameActorStaticStats,
    element: Option<&str>,
    actor_element: &str,
    mental: bool,
) -> Result<DamageBonuses, String> {
    let own_element_bonus = if element == Some(actor_element) {
        ExactFraction::from_ratio(stats.elemental_bonus)?
    } else {
        ExactFraction::new(0, 1)?
    };
    let element_bonus = element
        .and_then(|element| stats.elemental_bonuses.get(element))
        .copied()
        .map(ExactFraction::from_ratio)
        .transpose()?
        .unwrap_or(ExactFraction::new(0, 1)?)
        .add(own_element_bonus)?;
    let mental_bonus = if mental {
        ExactFraction::from_ratio(stats.mental_damage_bonus)?
    } else {
        ExactFraction::new(0, 1)?
    };
    let total = ExactFraction::from_ratio(stats.damage_bonus)?
        .add(element_bonus)?
        .add(mental_bonus)?;
    Ok((total, element_bonus, mental_bonus))
}

fn team_damage_modifiers(
    request: &crate::simulation::GameScenarioRequest,
    state: &crate::simulation::state::GameSimulatorState,
    target_count: usize,
    packet: &crate::simulation::metadata::PacketMetadata,
    mut modifiers: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    for (source_index, (source_config, source_actor)) in
        request.actors.iter().zip(&state.actors).enumerate()
    {
        modifiers = crate::characters::team::modifiers(
            &crate::runtime_api::RecipientContext {
                source_index,
                source_config,
                source_actor,
                team_actors: &state.actors,

                time_us: state.current_time_us,
                target_count,
                packet: Some(packet),
                request: Some(request),
            },
            modifiers,
        )?;
    }
    Ok(modifiers)
}

fn resistance_layers(
    ctx: &crate::runtime_api::DamageContext<'_>,
    key: Option<&str>,
    modifiers: &crate::runtime_api::HitModifiers,
) -> Result<ExactFraction, String> {
    match key {
        Some(element) => crate::simulation::state::target_resistance_reduction(
            ctx.buffs,
            ctx.target_index,
            element,
        )?
        .add(modifiers.resistance_reduction),
        None => Ok(modifiers.resistance_reduction),
    }
}

fn living_target_counts(state: &crate::simulation::state::GameSimulatorState) -> (usize, usize) {
    state
        .targets
        .iter()
        .filter(|t| t.hp_current > 0)
        .fold((0, 0), |(all, non_boss), t| {
            (all + 1, non_boss + usize::from(!t.is_boss))
        })
}

fn element_damage_keys(element: Option<&str>) -> (Option<&'static str>, Option<&'static str>) {
    match element {
        Some("COSMOS") => (
            Some("ECharacterElementType::CHARACTER_ELEMENT_TYPE_COSMOS"),
            Some("damage_bonus:ECharacterElementType::CHARACTER_ELEMENT_TYPE_COSMOS"),
        ),
        Some("NATURE") => (
            Some("ECharacterElementType::CHARACTER_ELEMENT_TYPE_NATURE"),
            Some("damage_bonus:ECharacterElementType::CHARACTER_ELEMENT_TYPE_NATURE"),
        ),
        Some("CHAOS") => (
            Some("ECharacterElementType::CHARACTER_ELEMENT_TYPE_CHAOS"),
            Some("damage_bonus:ECharacterElementType::CHARACTER_ELEMENT_TYPE_CHAOS"),
        ),
        Some("INCANTATION") => (
            Some("ECharacterElementType::CHARACTER_ELEMENT_TYPE_INCANTATION"),
            Some("damage_bonus:ECharacterElementType::CHARACTER_ELEMENT_TYPE_INCANTATION"),
        ),
        Some("LAKSHANA") => (
            Some("ECharacterElementType::CHARACTER_ELEMENT_TYPE_LAKSHANA"),
            Some("damage_bonus:ECharacterElementType::CHARACTER_ELEMENT_TYPE_LAKSHANA"),
        ),
        Some("PSYCHE") => (
            Some("ECharacterElementType::CHARACTER_ELEMENT_TYPE_PSYCHE"),
            Some("damage_bonus:ECharacterElementType::CHARACTER_ELEMENT_TYPE_PSYCHE"),
        ),
        _ => (None, None),
    }
}

pub(super) fn evaluate(
    request: &crate::simulation::GameScenarioRequest,
    state: &crate::simulation::state::GameSimulatorState,
    delivery: &HitDelivery,
    speculative_buffs: &ntesim_core::buffs::dispatch::BuffManager,
) -> Result<CalculatedHit, String> {
    let HitIdentity {
        command_index,
        actor_index,
        target_index,
        ..
    } = delivery.identity;
    let ability_id = &delivery.ability_id;
    let damage_effect_id = &delivery.damage_effect_id;
    let atk_rate = delivery.atk_rate;
    let scaling_stat = delivery.scaling_stat;
    let metadata = &delivery.metadata;
    let actor_config = &request.actors[actor_index];
    let actor = &state.actors[actor_index];
    let (target_count, non_boss_target_count) = living_target_counts(state);
    let owned_status_counts = crate::simulation::state::owned_status_counts(
        speculative_buffs,
        &state.targets,
        actor_index,
    );
    let target = &state.targets[target_index];

    let buff_stats = live_buff_stats(state, actor_index, speculative_buffs)?;
    let buff_stat = |key: &str| -> Result<ExactFraction, String> {
        for (stat, ratio) in &buff_stats {
            if *stat == key {
                return ExactFraction::from_ratio(*ratio);
            }
        }
        ExactFraction::new(0, 1)
    };
    let team_def_ignore = buff_stat("def_ignore")?;
    let mut rate_exact = ExactFraction::from_ratio(atk_rate)?;
    let packet_element_name = metadata
        .packet
        .element
        .as_ref()
        .and_then(crate::simulation::metadata::DamageElement::actor_element_name);
    let mental = metadata.packet.damage_kind == ntesim_core::combat::DamageKind::Mental;
    let (static_bonus, static_element_bonus, static_mental_bonus) = match &state.damage_bonuses {
        Some(bonuses) => bonuses[actor_index].get(metadata.packet.element.as_ref(), mental)?,
        None => {
            let descriptor =
                crate::registry::character_descriptor_by_id(&actor_config.character_id)?;
            static_damage_bonuses(
                &actor_config.static_stats,
                packet_element_name,
                &descriptor.element,
                mental,
            )?
        }
    };
    let mut modifiers = crate::runtime_api::HitModifiers::new(static_bonus)?;
    if mental {
        modifiers = modifiers.add_damage(buff_stat("mental_damage_bonus")?)?;
    }
    let (res_key, dmg_elem_key) = element_damage_keys(packet_element_name);
    let res_key = match metadata.packet.damage_kind {
        ntesim_core::combat::DamageKind::Mental => Some("MENTAL"),
        _ if metadata.packet.element
            == Some(crate::simulation::metadata::DamageElement::Physical) =>
        {
            Some("PHYSICAL")
        }
        _ => res_key,
    };
    let target_resistance = packet_element_name
        .and_then(|element| {
            request.targets[target_index]
                .elemental_resistances
                .get(element)
        })
        .copied()
        .unwrap_or(target.resistance);
    modifiers = team_damage_modifiers(request, state, target_count, &metadata.packet, modifiers)?;

    let debuff_count =
        crate::simulation::state::target_status_count(speculative_buffs, target_index);

    let team_has_three_elements = state.team_has_three_elements()?;

    let damage_ctx = crate::runtime_api::DamageContext {
        foreign_dot_stacks: crate::registry::attached_dot_stacks(
            request,
            state,
            target_index,
            state.current_time_us,
        )?,
        packet_group_id: metadata.binding.energy_group_id.as_deref(),
        command_index,
        config: actor_config,
        start_fons: request.start_fons,
        ability_id,
        action_metadata: &metadata.action,
        target,
        actor,
        world_time_us: state.mode_clock.relative_us(),
        time_us: state.current_time_us,
        debuff_count,
        damage_effect_id,
        actor_index,
        active_actor_index: state.active_actor_index,
        target_count,
        non_boss_target_count,
        target_index,
        buffs: speculative_buffs,
        buff_stats: crate::runtime_api::BuffStats::new(&buff_stats),
        owned_status_counts: &owned_status_counts,
        team_has_three_elements,
    };
    modifiers = crate::registry::damage_bonus(&damage_ctx, modifiers)?;
    if let Some(damage) = modifiers.resolved_damage {
        return Ok(CalculatedHit {
            damage_base_override: None,
            non_crit: damage,
            crit_damage: damage,
            crit_rate: crate::formula::Ratio::zero(),
            crit_eligible: false,
            static_element_bonus,
            static_mental_bonus,
            assumptions: modifiers.assumptions,
        });
    }
    modifiers = crate::gear::damage_bonus(&damage_ctx, &metadata.packet, modifiers)?;
    modifiers = modifiers.add_damage(buff_stat("damage_bonus")?)?;
    if let Some(key) = dmg_elem_key {
        modifiers = modifiers.add_damage(buff_stat(key)?)?;
    }

    if matches!(
        scaling_stat,
        crate::simulation::NativeScalingStat::Mixed { .. }
    ) && (!modifiers.ratio_add.is_zero() || !modifiers.ratio_modify.is_zero())
    {
        return Err(
            "mixed packet coefficient changes require an owner-selected scaling term".into(),
        );
    }
    rate_exact = rate_exact
        .mul(ExactFraction::new(1, 1)?.add(modifiers.ratio_modify)?)?
        .add(modifiers.ratio_add)?;
    let packet_stat = if let Some(base) = modifiers.damage_base_override {
        // The native coefficient remains provenance, never a second scaling
        // term for a resource-derived replacement base.
        rate_exact = ExactFraction::new(1, 1)?;
        base
    } else {
        super::scaling::packet_stat(&damage_ctx, scaling_stat, &modifiers)?
    };

    let ability_def_ignore = crate::registry::defense_ignore(&damage_ctx)?;
    let total_def_ignore =
        crate::runtime_api::aggregate_defense_ignore(ability_def_ignore, team_def_ignore)?;
    let (total_crit_bonus, proposed_cr) =
        critical_parameters(&damage_ctx, &metadata.packet, &modifiers)?;
    let breakdown = ntesim_core::combat::exact::evaluate_direct_damage(
        &ntesim_core::combat::exact::ExactDirectDamageRequest {
            kind: metadata.packet.damage_kind,
            attacker_level: actor_config.character_level,
            stat: packet_stat,
            ratio: rate_exact,
            damage_bonus: modifiers.damage_bonus,
            target_defense: crate::runtime_api::target_defense(
                speculative_buffs,
                target,
                target_index,
                state.mode_clock.relative_us(),
            )?,
            defense_shred: ExactFraction::new(0, 1)?,
            defense_ignore: total_def_ignore,
            target_resistance: ExactFraction::from_ratio(target_resistance)?,
            resistance_reduction: resistance_layers(&damage_ctx, res_key, &modifiers)?,
            negative_resistance_model:
                ntesim_core::combat::NegativeResistanceModel::StandardRational,
            final_multiplier: ExactFraction::new(1, 1)?.add(modifiers.final_bonus)?,
            special_multiplier: stain_multiplier(&damage_ctx, &metadata.packet)?,
            crit_bonus: total_crit_bonus,
        },
    )?;
    let non_crit = breakdown.non_crit;
    let crit_damage = breakdown.critical;
    let total_cr = if breakdown.crit_eligible {
        proposed_cr
    } else {
        ExactFraction::new(0, 1)?
    };
    let rate = ntesim_core::formula::Ratio {
        numerator: u64::try_from(total_cr.numerator)
            .map_err(|_| "crit-rate numerator exceeds supported u64 range")?,
        denominator: u64::try_from(total_cr.denominator)
            .map_err(|_| "crit-rate denominator exceeds supported u64 range")?,
    };
    Ok(CalculatedHit {
        damage_base_override: modifiers.damage_base_override,
        non_crit,
        crit_damage,
        crit_rate: rate,
        crit_eligible: breakdown.crit_eligible,
        static_element_bonus,
        static_mental_bonus,
        assumptions: modifiers.assumptions,
    })
}

pub(super) struct RolledHit {
    pub critical: bool,
    pub crit_sample: Option<crate::rng::Sample>,
    pub damage: ExactFraction,
}

pub(super) fn roll(
    ctx: &mut EventContext<'_>,
    delivery: &HitDelivery,
    calculated: &CalculatedHit,
) -> Result<RolledHit, String> {
    let request = ctx.request;
    let actor_config = &request.actors[delivery.identity.actor_index];
    let damage_effect_id = &delivery.damage_effect_id;
    let rng = &mut *ctx.rng;
    let rate = calculated.crit_rate;
    let eligible = calculated.crit_eligible
        && request.stochastic_crit.as_ref().is_some_and(|policy| {
            policy
                .eligible_effects
                .get(&actor_config.character_id)
                .is_some_and(|effects| effects.contains("*") || effects.contains(damage_effect_id))
        });
    let mut crit_sample = None;
    let critical = if !eligible || rate.numerator == 0 {
        false
    } else if rate.numerator >= rate.denominator {
        true
    } else {
        let sample = rng
            .as_mut()
            .ok_or("research RNG missing")?
            .uniform(&actor_config.character_id, rate.denominator)?;
        let rolled = sample.value < rate.numerator;
        crit_sample = Some(sample);
        rolled
    };
    let damage_exact = if critical {
        calculated.crit_damage
    } else {
        calculated.non_crit
    };
    Ok(RolledHit {
        critical,
        crit_sample,
        damage: damage_exact,
    })
}
