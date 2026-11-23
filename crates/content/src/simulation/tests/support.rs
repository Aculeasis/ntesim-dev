use super::*;

/// Preserve independently authored fractional formula oracles after changing
/// primary statistics. The extra raw sum exists only in tests; also reconcile
/// the public integer total against the settled receipt ledger.
pub(crate) fn formula_damage_total<T: serde::Serialize>(report: &T) -> ExactFraction {
    let report = serde_json::to_value(report).unwrap();
    let mut raw = ExactFraction::new(0, 1).unwrap();
    let mut settled = 0_u128;
    for hit in report["hit_receipts"].as_array().unwrap() {
        raw = raw
            .add(serde_json::from_value(hit["damage"].clone()).unwrap())
            .unwrap();
        settled = settled
            .checked_add(u128::from(hit["rounded_damage"].as_u64().unwrap()))
            .unwrap();
    }
    assert_eq!(
        report["total_damage_exact"],
        serde_json::json!(ExactFraction::new(settled, 1).unwrap())
    );
    assert_eq!(
        report["total_damage_dealt"].as_f64().unwrap(),
        settled as f64
    );
    raw
}

/// Keep authored fixtures on the live native clock before applying independent
/// numeric stat overrides. Positive contact remains the same in these fixtures.
pub(crate) fn live_source_request(source: &str) -> GameScenarioRequest {
    let result = crate::simulation::PreparedSourceScenario::new(
        source,
        Default::default(),
        &ExecutionLimits::default(),
    )
    .unwrap()
    .run(42)
    .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    result.request
}

pub(super) fn invoke_gear_cast(
    request: &GameScenarioRequest,
    state: &mut crate::simulation::state::GameSimulatorState,
    time_us: u64,
    action: &crate::simulation::metadata::ActionMetadata,
) {
    with_gear_context(request, state, time_us, action, None, false);
}

fn with_gear_context(
    request: &GameScenarioRequest,
    state: &mut crate::simulation::state::GameSimulatorState,
    time_us: u64,
    action: &crate::simulation::metadata::ActionMetadata,
    packet: Option<&crate::simulation::metadata::PacketMetadata>,
    after_hit: bool,
) {
    let mut context = crate::runtime_api::CastContext {
        origins: &mut state.origins,
        command_index: 0,
        native_array_index: None,
        execution_assumptions: &mut BTreeMap::new(),
        config: &request.actors[0],
        request,
        actor_index: 0,
        active_actor_index: state.active_actor_index,
        target_index: 0,
        ability_id: &action.ability_id,
        action_metadata: action,
        packet_metadata: packet,
        world_time_us: time_us,
        time_us,
        hit_time_us: time_us,
        buffs: &mut state.buffs,
        actor: &mut state.actors[0],
        stamina: state.combat_stamina.as_mut(),
        team_hp_snapshot: &[],
        targets: &state.targets,
        queue: &mut crate::simulation::timer_queue::EventQueue::new(),
        sequence: &mut 0,
    };
    if after_hit {
        crate::gear::on_hit(&mut context, false, 0).unwrap();
    } else {
        crate::gear::on_cast(&mut context).unwrap();
    }
}

pub(super) fn invoke_gear_hook(
    request: &GameScenarioRequest,
    state: &mut crate::simulation::state::GameSimulatorState,
    time_us: u64,
    role: crate::simulation::metadata::InputActionRole,
    packet: &crate::simulation::metadata::PacketMetadata,
    after_hit: bool,
) {
    let mut action = declared_test_action(
        "caller-test",
        crate::simulation::metadata::ActionCategory::Skill,
    );
    action.input_role = Some(role);
    with_gear_context(request, state, time_us, &action, Some(packet), after_hit);
}

pub(super) fn weapon_projection_tables() -> BTreeMap<String, serde_json::Value> {
    BTreeMap::from([
        (
            "DT_ForkUpgradeStarDataTable".into(),
            ntesim_data::weapon_refinements::overlay(),
        ),
        ("CT_ForkBuff".into(), ntesim_data::weapon_curves::overlay()),
    ])
}

pub(crate) fn sample_s1_request() -> GameScenarioRequest {
    let stats = GameActorStaticStats {
        base_atk: 1151,
        atk: Ratio {
            numerator: 46383,
            denominator: 20,
        },
        hp: 17720,
        base_hp: None,
        hp_percent: Ratio::zero(),
        hp_flat: Ratio::zero(),
        def: 727,
        base_def: Some(727),
        def_percent: Ratio::zero(),
        def_flat: Ratio::zero(),
        crit_rate: Ratio {
            numerator: 57,
            denominator: 100,
        },
        crit_damage_bonus: Ratio {
            numerator: 47,
            denominator: 50,
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
    };

    let actor = GameActorConfig {
        slot: 0,
        character_id: "character:1019".into(),
        character_level: 80,
        awakening: ntesim_data::AwakeningSelection::prefix(0),
        purchased_skill_level: 8,
        purchased_qte_level: Some(8),
        talent_levels: None,
        character_max_level: None,
        initial_hp: 17720,
        initial_hp_max: 17720,
        initial_energy: 100,
        energy_capacity: 100,
        initial_cycle: 0,
        initial_resources: Default::default(),
        cycle_gain_efficiency: ntesim_core::formula::Ratio::one(),
        weapon_id: "weapon:fork_lingganzhongjiezhe".into(),
        weapon_tier: 5,
        set_id: Some("set:Suit2".into()),
        static_stats: stats,
        gear_procs: vec![
            GearProcConfig::MindRoyaleEnergy {
                energy_grant: 18,
                cooldown_us: 20_000_000,
            },
            GearProcConfig::FirefliesCritDamage {
                crit_damage_per_stack: Ratio {
                    numerator: 2,
                    denominator: 25,
                },
                max_stacks: 7,
                duration_us: 10_000_000,
            },
        ],
    };

    let target = GameTargetConfig {
        original_atk: None,
        base_defense: None,
        // This normalized packet fixture isolates coefficients and resource flows.
        // Public source-based control cases explicitly select a nonimmune dummy.
        control_immune: true,
        target_index: 0,
        is_boss: false,
        level: 80,
        hp_max: 10_000_000,
        hp_current: 10_000_000,
        defense: 1080,
        resistance: Ratio {
            numerator: 1,
            denominator: 5,
        },
        elemental_resistances: Default::default(),
        airborne_until_us: 0,
        break_profile: None,
    };

    let actions = vec![
        GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Mint019_Melee".into(),
            array_index: 7,
            damage_effect_id: "GE_Player_Mint_Melee1_Damage_New".into(),
            atk_rate: Ratio {
                numerator: 89,
                denominator: 500,
            },
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 800_000,
            cooldown_us: 0,
            hit_delay_us: 0,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 1_000_000,
            actor_index: 0,
            ability_id: "GA_Mint019_Skill".into(),
            array_index: 7,
            damage_effect_id: "GE_Player_Mint_Skill1_Damage_New".into(),
            atk_rate: Ratio {
                numerator: 771,
                denominator: 500,
            },
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 1_200_000,
            cooldown_us: 6_000_000,
            hit_delay_us: 0,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 2_500_000,
            actor_index: 0,
            ability_id: "GA_Mint019_UltraSkill".into(),
            array_index: 7,
            damage_effect_id: "GE_Player_Mint_UtraSkill1_Damage_New".into(),
            atk_rate: Ratio {
                numerator: 2273,
                denominator: 1000,
            },
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 100,
            recovery_us: 2_000_000,
            cooldown_us: 20_000_000,
            hit_delay_us: 0,
        },
    ];

    GameScenarioRequest {
        clock_contract: Default::default(),
        timer_policy: Default::default(),
        absolute_limit_us: ExecutionLimits::default().max_horizon_us,
        schema_version: 1,
        stop: Default::default(),
        action_admission_order: Default::default(),
        source_exhausted: false,
        source_halt: None,
        input_time_contexts: Vec::new(),
        content_id: "ntesim-current".into(),
        start_fons: None,
        combat_stamina: None,
        model_assumptions: BTreeMap::from([(
            crate::simulation::metadata::NORMAL_PHYSICAL_HYPOTHESIS_ID
                .as_str()
                .into(),
            crate::simulation::metadata::NORMAL_PHYSICAL_ID
                .description()
                .into(),
        )]),
        relative_duration_us: 20_000_000,
        seed: 0,
        stochastic_crit: None,
        action_metadata: BTreeMap::new(),
        plunge_inputs: BTreeMap::new(),
        approach_inputs: Default::default(),
        hold_inputs: BTreeMap::new(),
        branch_inputs: BTreeMap::new(),
        dash_directions: BTreeMap::new(),
        aim_targets: BTreeMap::new(),
        actors: vec![actor],
        active_actor_index: 0,
        targets: vec![target],
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
    }
}

pub(crate) fn research_request() -> GameScenarioRequest {
    let mut req = sample_s1_request();
    req.seed = 42;
    req.actors[0].gear_procs.clear();
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].static_stats.crit_rate = Ratio {
        numerator: 1,
        denominator: 2,
    };
    req.actors[0].static_stats.crit_damage_bonus = Ratio {
        numerator: 1,
        denominator: 1,
    };
    req.action_metadata.clear();
    req.actions = (0..6)
        .map(|i| GameActionCommand::InvokeAbility {
            timestamp_us: i * 10,
            actor_index: 0,
            ability_id: "caller-test".into(),
            array_index: 0,
            damage_effect_id: "caller-effect".into(),
            atk_rate: Ratio {
                numerator: 1,
                denominator: 1,
            },
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 0,
        })
        .collect();
    let manual_description = "Test-only explicit metadata for synthetic actions.";
    req.model_assumptions
        .insert("test.synthetic_action".into(), manual_description.into());
    req.action_metadata = (0..6)
        .map(|index| {
            (
                index,
                crate::simulation::ResearchActionMetadata {
                    hypothesis_id: "test.synthetic_action".into(),
                    ability_id: "caller-test".into(),
                    damage_effect_id: "caller-effect".into(),
                    action_gameplay_tags: Some(vec!["Ability.Skill".into()]),
                    input_action_role: None,
                    damage_type_ex: Some("EExecutionDamageTypeEX::DAMAGE_TYPE_NATURE".into()),
                    damage_source_category: None,
                    fixed_crit_rate: Some(crate::simulation::ResearchFixedCritRate::NoOverride),
                    unbal_value: None,
                    packet_ga_name_binding: None,
                },
            )
        })
        .collect();
    req.stochastic_crit = Some(ResearchCritPolicy {
        roll_policy: ResearchCritRollPolicy::IndependentPerTargetHitV1,
        hypothesis: "Test independent per-target-hit crit rolls; game policy unknown".into(),
        eligible_effects: BTreeMap::from([(
            "character:1019".into(),
            BTreeSet::from(["caller-effect".into()]),
        )]),
        actor_sequences: BTreeMap::from([("character:1019".into(), 54)]),
    });
    req
}

pub(crate) fn assert_non_support_damage_prefix(report: &serde_json::Value, expected: &[u64]) {
    let damage: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["switch_origin"]["kind"] != "support")
        .map(|hit| hit["rounded_damage"].as_u64().unwrap())
        .collect();
    assert!(damage.len() >= expected.len());
    assert_eq!(&damage[..expected.len()], expected);
}

pub(crate) fn declared_test_action(
    ability_id: &str,
    category: crate::simulation::metadata::ActionCategory,
) -> crate::simulation::metadata::ActionMetadata {
    crate::simulation::metadata::ActionMetadata {
        categories: std::collections::BTreeSet::from([category]),
        gameplay_tags: vec![category.native_tag().to_owned()],
        ability_id: ability_id.to_owned(),
        native_ability_index: None,
        input_role: None,
        input_role_provenance: None,
        primary_slot: None,
        source_evidence: None,
        identity_evidence: None,
        provenance: crate::simulation::metadata::MetadataProvenance::AcceptedModelHypothesis {
            id: "test.action_metadata".into(),
            description: "Test-owned explicit action metadata.".into(),
        },
    }
}

/// Shared fixture for weapon passives triggered by Redirect Skill casts:
/// one synthetic action slot, cleared actions, caller-test metadata.
pub(crate) fn redirect_weapon_fixture(
    weapon_id: &str,
    procs: Vec<GearProcConfig>,
    atk: Ratio,
) -> GameScenarioRequest {
    let mut req = research_request();
    req.stochastic_crit = None;
    req.relative_duration_us = 40_000_000;
    req.actors[0].weapon_id = weapon_id.into();
    req.actors[0].gear_procs = procs;
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = atk;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.targets[0].resistance = Ratio::zero();
    let metadata = req.action_metadata[&0].clone();
    req.actions.clear();
    req.action_metadata.clear();
    req.action_metadata.insert(0, metadata);
    req
}

/// Append one synthetic single-packet ability with an explicit input role.
pub(crate) fn append_synthetic_cast(
    req: &mut GameScenarioRequest,
    time_us: u64,
    role: crate::simulation::metadata::InputActionRole,
    hit_delay_us: u64,
    damage_source_category: Option<&str>,
) {
    let index = req.actions.len();
    req.actions.push(GameActionCommand::InvokeAbility {
        timestamp_us: time_us,
        actor_index: 0,
        ability_id: "caller-test".into(),
        damage_effect_id: "caller-effect".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us,
    });
    let mut metadata = req.action_metadata[&0].clone();
    metadata.input_action_role = Some(role);
    metadata.action_gameplay_tags = Some(vec!["Ability.Melee".into()]);
    if let Some(category) = damage_source_category {
        metadata.damage_source_category = Some(category.into());
    }
    req.action_metadata.insert(index, metadata);
}

pub(super) fn gear_zone_modifiers(
    req: &crate::simulation::GameScenarioRequest,
    state: &crate::simulation::state::GameSimulatorState,
    time_us: u64,
    target_index: usize,
) -> crate::runtime_api::HitModifiers {
    gear_zone_modifiers_for_element(req, state, time_us, target_index, None)
}

pub(super) fn gear_zone_modifiers_for_element(
    req: &crate::simulation::GameScenarioRequest,
    state: &crate::simulation::state::GameSimulatorState,
    time_us: u64,
    target_index: usize,
    element: Option<crate::simulation::metadata::DamageElement>,
) -> crate::runtime_api::HitModifiers {
    let action_metadata = super::support::declared_test_action(
        "GA_Test",
        crate::simulation::metadata::ActionCategory::Melee,
    );
    let packet = crate::simulation::metadata::PacketMetadata {
        damage_effect_id: "GE_Test_Damage".into(),
        native_ga_name: None,
        native_damage_type_ex: format!(
            "EExecutionDamageTypeEX::DAMAGE_TYPE_{}",
            element.as_ref().map_or("NATURE", |element| element
                .actor_element_name()
                .unwrap_or("PHYSICAL"))
        ),
        native_damage_source_category: None,
        damage_kind: ntesim_core::combat::DamageKind::Elemental,
        element,
        fixed_crit_rate: None,
        unbal_value: None,
        fixed_crit_rate_provenance: None,
        unbal_value_provenance: None,
        semantic_tags: BTreeSet::new(),
        semantic_provenance: None,
        source_evidence: None,
        provenance: crate::simulation::metadata::MetadataProvenance::AcceptedModelHypothesis {
            id: "test".into(),
            description: "Test-owned packet metadata.".into(),
        },
        packet_kind: Default::default(),
    };
    let owned_status_counts = BTreeMap::new();
    let buff_stats = BTreeMap::new();
    let ctx = crate::runtime_api::DamageContext {
        foreign_dot_stacks: 0,
        packet_group_id: None,
        command_index: 0,
        config: &req.actors[0],
        start_fons: None,
        ability_id: "GA_Test",
        action_metadata: &action_metadata,
        target: &state.targets[target_index],
        actor: &state.actors[0],
        world_time_us: time_us,
        time_us,
        debuff_count: 0,
        damage_effect_id: "GE_Test_Damage",
        actor_index: 0,
        active_actor_index: 0,
        target_count: 1,
        non_boss_target_count: 1,
        target_index,
        buffs: &state.buffs,
        buff_stats: (&buff_stats).into(),
        owned_status_counts: &owned_status_counts,
        team_has_three_elements: false,
    };
    crate::gear::damage_bonus(
        &ctx,
        &packet,
        crate::runtime_api::HitModifiers::new(ExactFraction::new(0, 1).unwrap()).unwrap(),
    )
    .unwrap()
}

pub(crate) fn hit_metadata<'a>(
    report: &'a serde_json::Value,
    hit: &'a serde_json::Value,
) -> &'a serde_json::Value {
    let id = hit["metadata_id"]
        .as_u64()
        .expect("hit_receipt metadata_id") as usize;
    &report["metadata_catalog"][id]
}

pub(crate) fn action_metadata<'a>(
    report: &'a serde_json::Value,
    entry: &'a serde_json::Value,
) -> &'a serde_json::Value {
    let id = entry["metadata_id"]
        .as_u64()
        .expect("action_metadata_ledger metadata_id") as usize;
    &report["metadata_catalog"][id]
}

pub(crate) fn has_assumption(receipt: &serde_json::Value, hypothesis_id: &str) -> bool {
    receipt["execution_model_assumption_ids"]
        .as_array()
        .is_some_and(|items| items.iter().any(|item| item == hypothesis_id))
}

pub(crate) fn receipt_assumption<'a>(
    report: &'a serde_json::Value,
    receipt: &serde_json::Value,
    hypothesis_id: &str,
) -> Option<&'a str> {
    has_assumption(receipt, hypothesis_id)
        .then(|| report["resolved_model_assumptions"][hypothesis_id].as_str())
        .flatten()
}

#[allow(dead_code)]
pub(crate) fn entry_has_command(entry: &serde_json::Value, command_index: usize) -> bool {
    entry["command_indices"]
        .as_array()
        .is_some_and(|items| items.iter().any(|item| item == command_index))
}

#[allow(dead_code)]
pub(crate) fn final_buffs(report: &serde_json::Value) -> Vec<serde_json::Value> {
    let catalog = report["buff_catalog"].as_array();
    report["final_buff_state"]["instances"]
        .as_array()
        .expect("report final_buff_state.instances")
        .iter()
        .map(|entry| {
            let def_id = entry["buff_def_id"].as_u64().expect("buff_def_id") as usize;
            let mut merged = catalog
                .and_then(|entries| entries.get(def_id))
                .and_then(|value| value.as_object())
                .expect("buff_catalog entry")
                .clone();
            if let Some(obj) = entry.as_object() {
                for (key, value) in obj {
                    merged.insert(key.clone(), value.clone());
                }
            }
            serde_json::Value::Object(merged)
        })
        .collect()
}
impl crate::simulation::state::GameSimulatorState {
    /// Unit fixtures that bypass the scheduler must advance its clock and
    /// cached owner coordinates together. Paused execution uses PreparedExecution.
    pub(crate) fn advance_unpaused_test_clock(&mut self, now: u64) {
        assert!(!self.mode_clock.is_paused());
        let delta = now.saturating_sub(self.current_time_us);
        self.mode_clock.advance_to(now).unwrap();
        self.current_time_us = now;
        if let Some(actor) = self.actors.get_mut(self.active_actor_index) {
            actor.field_time_absolute_us += delta;
            actor.field_time_relative_us += delta;
        }
        let world = self.mode_clock.relative_us();
        for actor in &mut self.actors {
            actor.world_time_us = world;
            actor.ability_cooldowns.advance(now, world).unwrap();
        }
    }
}

/// Independent interval union for World-time gear-window arithmetic. Receipts
/// stay on A; this deliberately does not call the production clock converter.
pub(super) fn world_time_at(report: &serde_json::Value, time_us: u64) -> u64 {
    let mut spans: Vec<_> = report["timer_pauses"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            (
                p["start_absolute_us"].as_u64().unwrap().min(time_us),
                p["end_absolute_us"].as_u64().unwrap().min(time_us),
            )
        })
        .collect();
    spans.sort_unstable();
    let (mut end, mut paused) = (0, 0);
    for (start, stop) in spans {
        paused += stop.saturating_sub(start.max(end));
        end = end.max(stop);
    }
    time_us - paused
}
