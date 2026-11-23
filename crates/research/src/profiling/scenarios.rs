use super::*;

fn team_slots(team_slug: &str) -> Result<Vec<&str>, String> {
    let slots: Vec<&str> = team_slug.split('+').map(|s| s.trim()).collect();
    if slots.len() != 4 {
        return Err(format!(
            "team must have exactly 4 characters, found {}",
            slots.len()
        ));
    }
    if slots.iter().copied().collect::<BTreeSet<_>>().len() != 4 {
        return Err("team must contain 4 distinct characters".into());
    }
    Ok(slots)
}

/// Present Replay requires more than 60 Timepiece, whose pool starts empty and
/// accrues 6/s, so Hotori cannot cast before 10s of real combat time. Preview
/// fixtures containing her enter their fixed schedule this much later.
const HOTORI_PREVIEW_ENTRY_OFFSET_US: u64 = 10_000_000;

pub fn preview_entry_offset_us(team_slug: &str) -> u64 {
    if team_slug.split('+').any(|slot| slot.trim() == "hotori") {
        HOTORI_PREVIEW_ENTRY_OFFSET_US
    } else {
        0
    }
}

fn preview_schedule(first: &str, entry: u64) -> ([u64; 4], [u64; 3], u64) {
    // Opening Nanally E needs1.164219s before switching. The authored500ms
    // tail shift preserves entry-to-action gaps and source slot order; no polling.
    let tail = if first == "nanally" { 500_000 } else { 0 };
    const ENTRY_RECOVERY_US: u64 = 2_100_000;
    const SLOT_SPACING_US: u64 = ENTRY_RECOVERY_US + 2_790_000;
    let switches = [0, 1, 2].map(|slot| 1_000_000 + entry + tail + slot * SLOT_SPACING_US);
    let actions = [
        250_000 + entry,
        switches[0] + ENTRY_RECOVERY_US,
        switches[1] + ENTRY_RECOVERY_US,
        switches[2] + ENTRY_RECOVERY_US,
    ];
    (actions, switches, actions[3] + 590_000)
}

fn preview_actor(
    slot_idx: usize,
    slug: &str,
    overrides: Option<&BTreeMap<String, ActorOverride>>,
) -> Result<GameActorConfig, String> {
    let p = get_character_profile(slug)?;
    let o = overrides
        .map(|m| {
            m.get(slug)
                .ok_or_else(|| format!("missing actor override for {slug}"))
        })
        .transpose()?;

    let static_stats = match o {
        Some(ov) => ov.static_stats.clone(),
        None => GameActorStaticStats {
            base_atk: p.base_atk,
            atk: p.atk_ratio,
            hp: p.hp,
            base_hp: None,
            hp_percent: Ratio::zero(),
            hp_flat: Ratio::zero(),
            def: p.def,
            base_def: None,
            def_percent: Ratio::zero(),
            def_flat: Ratio::zero(),
            crit_rate: Ratio {
                numerator: 1,
                denominator: 2,
            },
            crit_damage_bonus: Ratio {
                numerator: 1,
                denominator: 1,
            },
            healing_bonus: Ratio::zero(),
            shield_bonus: Ratio::zero(),
            break_intensity: Ratio::zero(),
            cycle_intensity: Ratio::zero(),
            break_efficiency_multiplier: Ratio::one(),
            mental_damage_bonus: Ratio::zero(),
            damage_bonus: Ratio::zero(),
            elemental_bonuses: Default::default(),
            elemental_bonus: Ratio {
                numerator: 1,
                denominator: 10,
            },
        },
    };

    Ok(GameActorConfig {
        slot: slot_idx,
        character_id: p.character_id.to_string(),
        character_level: 80,
        awakening: p.awakening,
        purchased_skill_level: 8,
        purchased_qte_level: Some(8),
        talent_levels: None,
        character_max_level: None,
        initial_hp: static_stats.hp,
        initial_hp_max: static_stats.hp,
        initial_energy: 100,
        energy_capacity: 100,
        initial_cycle: 0,
        initial_resources: Default::default(),
        cycle_gain_efficiency: ntesim_core::formula::Ratio::one(),
        weapon_id: o
            .and_then(|ov| ov.weapon_id.as_deref())
            .unwrap_or(p.weapon_id)
            .to_string(),
        weapon_tier: o.and_then(|ov| ov.weapon_tier).unwrap_or(p.weapon_tier),
        set_id: o
            .and_then(|ov| ov.set_id.clone())
            .or_else(|| p.set_id.map(str::to_owned)),
        static_stats,
        gear_procs: p.gear_procs.clone(),
    })
}

pub fn build_pilot_scenario(
    team_slug: &str,
    scenario_kind: AcceptanceScenarioKind,
    timing_multiplier: Ratio,
    seed: u64,
    overrides: Option<&BTreeMap<String, ActorOverride>>,
) -> Result<GameScenarioRequest, String> {
    if scenario_kind.is_mode_clock() {
        return Err(format!(
            "unsupported scenario {}: game runner has no mode-clock horizon",
            scenario_kind.code()
        ));
    }
    let slots = team_slots(team_slug)?;
    let actors = slots
        .iter()
        .enumerate()
        .map(|(index, slug)| preview_actor(index, slug, overrides))
        .collect::<Result<Vec<_>, _>>()?;

    // The input slot order owns the opening rotation; no character gets a hidden priority.
    let initial_actor = 0;

    let mut sequence = vec![initial_actor];
    for slot_idx in 0..4 {
        if slot_idx != initial_actor {
            sequence.push(slot_idx);
        }
    }

    let mut actions = Vec::new();
    let mut preview_action_assumptions = BTreeMap::new();
    preview_action_assumptions.insert(
        ntesim_content::build::preview_projection::PREVIEW_SINGLE_PACKET_HYPOTHESIS_ID.into(),
        ntesim_content::build::preview_projection::PREVIEW_SINGLE_PACKET_HYPOTHESIS_DESCRIPTION
            .into(),
    );
    let switch_cooldown_us = 4_000_000u64;
    // The fixed preview allows 2.1s after each entry for connected Support
    // recovery and preserves the preceding cast's switch margin.
    // Teams with Hotori shift the whole schedule so her authored cast has
    // enough accrued Timepiece; relative spacing is unchanged.
    let entry_offset_us = preview_entry_offset_us(team_slug);
    let (action_times, switch_times, wait_at_us) = preview_schedule(slots[0], entry_offset_us);

    for step in 0..4 {
        let actor_idx = sequence[step];
        let slug = slots[actor_idx];
        let p = get_character_profile(slug)?;
        let action_projection = ntesim_content::build::preview_projection::project_preview_action(
            ntesim_data::skill_levels::frozen::tables(),
            &ntesim_content::build::preview_projection::PreviewActionRequest {
                content_id: "ntesim-current",
                character_id: p.character_id,
                ability_id: p.ability_id,
                damage_effect_id: admitted_packet(&p)?,
                purchased_skill_level: 8,
                awakening: p.awakening,
                model_assumptions: &preview_action_assumptions,
            },
        )
        .map_err(|error| format!("unsupported preview action: {error}"))?;
        let packet_assumptions: BTreeMap<String, String> =
            serde_json::from_value(action_projection["packet_model_assumptions"].clone())
                .map_err(|error| format!("projected packet assumptions are invalid: {error}"))?;
        preview_action_assumptions.extend(packet_assumptions);
        let ability_id = action_projection["ability_id"]
            .as_str()
            .ok_or("projected preview ability ID missing")?;
        let damage_effect_id = action_projection["damage_effect_id"]
            .as_str()
            .ok_or("projected preview damage effect ID missing")?;
        let array_index: usize = action_projection["array_index"]
            .as_u64()
            .ok_or("projected preview array coordinate missing")?
            .try_into()
            .map_err(|_| "projected preview array coordinate overflow")?;
        let atk_rate: Ratio = serde_json::from_value(action_projection["compiled_rate"].clone())
            .map_err(|error| format!("projected preview rate is invalid: {error}"))?;
        let scaling_stat =
            serde_json::from_value(action_projection["compiled_scaling_stat"].clone())
                .map_err(|error| format!("projected preview scaling stat is invalid: {error}"))?;

        if step > 0 {
            actions.push(GameActionCommand::SwitchActor {
                timestamp_us: switch_times[step - 1],
                target_actor_index: actor_idx,
                cooldown_us: switch_cooldown_us,
            });
        }

        actions.push(GameActionCommand::InvokeAbility {
            timestamp_us: action_times[step],
            actor_index: actor_idx,
            ability_id: ability_id.to_string(),
            array_index,
            damage_effect_id: damage_effect_id.to_string(),
            atk_rate,
            scaling_stat,
            energy_cost: p.energy_cost,
            recovery_us: 500_000,
            cooldown_us: 6_000_000,
            hit_delay_us: 100_000,
        });
    }

    let targets = scenario_kind.create_targets();
    let horizon_us = scenario_kind.horizon_us();

    actions.push(GameActionCommand::Wait {
        timestamp_us: wait_at_us,
        duration_us: horizon_us.saturating_sub(wait_at_us),
    });

    let mut research_crit = ResearchCritPolicy {
        roll_policy: ResearchCritRollPolicy::IndependentPerTargetHitV1,
        hypothesis: "Native profiling pilot execution".into(),
        eligible_effects: BTreeMap::new(),
        actor_sequences: BTreeMap::new(),
    };

    for (idx, slug) in slots.iter().enumerate() {
        let p = get_character_profile(slug)?;
        let effect = admitted_packet(&p)?.to_string();
        research_crit
            .eligible_effects
            .entry(p.character_id.to_string())
            .or_default()
            .insert(effect);
        research_crit
            .eligible_effects
            .get_mut(p.character_id)
            .expect("primary packet eligibility was just inserted")
            .extend(
                additional_critical_effects(&p)
                    .iter()
                    .map(|effect| (*effect).into()),
            );
        research_crit
            .actor_sequences
            .entry(p.character_id.to_string())
            .or_insert((idx as u64 + 1) * 101);
    }

    let scenario = GameScenarioRequest {
        clock_contract: Default::default(),
        timer_policy: Default::default(),
        absolute_limit_us: ntesim_core::execution::ExecutionLimits::default().max_horizon_us,
        schema_version: 1,
        stop: Default::default(),
        action_admission_order: Default::default(),
        source_exhausted: false,
        source_halt: None,
        input_time_contexts: Vec::new(),
        content_id: "ntesim-current".into(),
        model_assumptions: preview_action_assumptions,
        relative_duration_us: horizon_us,
        start_fons: None,
        combat_stamina: None,
        seed,
        stochastic_crit: Some(research_crit),
        // Profiling actions use frozen native ability/effect rows; only an
        // explicitly missing row may receive an action-indexed hypothesis.
        action_metadata: BTreeMap::new(),
        plunge_inputs: BTreeMap::new(),
        approach_inputs: Default::default(),
        hold_inputs: BTreeMap::new(),
        branch_inputs: BTreeMap::new(),
        dash_directions: BTreeMap::new(),
        aim_targets: BTreeMap::new(),
        actors,
        active_actor_index: initial_actor,
        targets,
        incoming_damage_events: Vec::new(),
        hp_cost_events: Vec::new(),
        interrupt_events: Vec::new(),
        tenacity_events: Vec::new(),
        parry_events: Vec::new(),
        combat_transition_events: Vec::new(),
        dummy_combat_events: Vec::new(),
        incoming_dummy_sources: Default::default(),
        incoming_damage_basis: Default::default(),
        actions,
    };

    transform_timing(&scenario, timing_multiplier)
}

fn admitted_packet(
    profile: &ntesim_content::registry::PreviewCharacterProfile,
) -> Result<&'static str, String> {
    ntesim_content::registry::primary_packet(profile.character_id, profile.ability_id).ok_or_else(
        || {
            format!(
                "no modeled input packet for {} ({})",
                profile.character_id, profile.ability_id
            )
        },
    )
}

fn additional_critical_effects(
    profile: &ntesim_content::registry::PreviewCharacterProfile,
) -> &'static [&'static str] {
    match (profile.character_id, profile.ability_id) {
        ("character:1076", "GA_Shinku_Skill") => &[
            "GE_Player_Shinku_Skill1_2_Damage",
            "GE_Player_Shinku_Skill1_3_Damage",
        ],
        ("character:1033", "GA_Adler_Skill") => &[
            "GE_Player_Adler_Skill3_Damage",
            "GE_Player_Adler_Skill_Damage",
        ],
        _ => &[],
    }
}
