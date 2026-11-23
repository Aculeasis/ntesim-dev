use super::*;
use crate::simulation::metadata::InputActionRole;
use crate::simulation::state::GameSimulatorState;
use ntesim_core::buffs::dispatch::{BuffDuration, BuffManager};
use serde_json::json;
use std::collections::BTreeMap;

fn tables() -> BTreeMap<String, serde_json::Value> {
    super::support::weapon_projection_tables()
}

fn projection_actor(character_id: &str, tier: u32) -> GameActorConfig {
    serde_json::from_value(json!({
        "slot": 0, "character_id": character_id, "character_level": 80, "awakening": [],
        "purchased_skill_level": 8, "initial_hp": 1, "initial_hp_max": 1, "initial_energy": 0,
        "energy_capacity": 100, "weapon_id": "weapon:fork_Rose", "weapon_tier": tier,
        "static_stats": {
            "base_atk": 1, "atk": "1/1", "hp": 1, "def": 1,
            "crit_rate": "0/1", "crit_damage_bonus": "0/1"
        }
    }))
    .unwrap()
}

fn t1_proc() -> GearProcConfig {
    GearProcConfig::RoseThorn {
        atk_bonus: Ratio {
            numerator: 7,
            denominator: 50,
        },
        crit_damage_per_stack: Ratio {
            numerator: 3,
            denominator: 50,
        },
        duration_us: 3_000_000,
        broken_extension_us: 3_000_000,
    }
}

fn thorn_count(buffs: &BuffManager, actor_index: usize, now_us: u64) -> u32 {
    buffs
        .instances()
        .iter()
        .filter(|buff| {
            buff.id == crate::weapons::rose::THORN_BUFF_ID
                && buff.owner_actor == actor_index
                && matches!(
                    buff.duration,
                    BuffDuration::Timed { expires_at_us, .. } if expires_at_us > now_us
                )
        })
        .map(|buff| buff.stacks)
        .sum()
}

#[test]
fn rose_projection_binds_five_refinements_with_independent_curve_oracles() {
    let tables = tables();
    let expected_atk = [(7u64, 50u64), (7, 40), (21, 100), (49, 200), (7, 25)];
    let expected_crit = [(3u64, 50u64), (3, 40), (9, 100), (21, 200), (3, 25)];
    for (tier, ((atk_n, atk_d), (crit_n, crit_d))) in
        (1..=5u32).zip(expected_atk.iter().zip(expected_crit.iter()))
    {
        let mut actor = projection_actor("character:1004", tier);
        let report = crate::weapons::rose::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap();
        assert_eq!(
            report["compiled_proc"]["atk_bonus"],
            json!(format!("{}/{}", atk_n, atk_d)),
            "tier {tier} atk"
        );
        assert_eq!(
            report["compiled_proc"]["crit_damage_per_stack"],
            json!(format!("{}/{}", crit_n, crit_d)),
            "tier {tier} crit"
        );
        assert_eq!(report["compiled_proc"]["duration_us"], 3_000_000);
        assert_eq!(report["compiled_proc"]["broken_extension_us"], 3_000_000);
        let proc: GearProcConfig = serde_json::from_value(report["compiled_proc"].clone()).unwrap();
        assert!(matches!(proc, GearProcConfig::RoseThorn { .. }));
        assert_eq!(actor.gear_procs.len(), 1);
    }
    assert_eq!(
        ntesim_data::weapon_definitions::lookup(&tables, "fork_Rose")
            .unwrap()
            .1["ElementData"]["ApplyGroupType"],
        "ECharacterGroupType::CHARACTER_GROUP_TYPE_TWO"
    );
}

#[test]
fn rose_projection_rejects_group_four_character() {
    let tables = tables();
    let mut actor = projection_actor("character:1036", 1);
    assert!(
        crate::weapons::rose::projection::project(&tables, 1, "ntesim-current", &mut actor, &[],)
            .unwrap_err()
            .contains("incompatible")
    );
    assert!(actor.gear_procs.is_empty());
}

#[test]
fn rose_validate_config_rejects_bad_proc() {
    let request = research_request();
    let limits = ExecutionLimits::default();
    let mut bad_actor = request.actors[0].clone();
    bad_actor.weapon_id = "weapon:fork_Kite".into();
    let bad_ctx = crate::gear::validation::ConfigContext {
        actor: &bad_actor,
        actor_index: 0,
        limits: &limits,
    };
    match crate::weapons::rose::validate_config(&t1_proc(), &bad_ctx) {
        Err(err) => assert!(err.contains("fork_Rose"), "unexpected error: {err}"),
        Ok(_) => panic!("wrong weapon id should reject"),
    }
    let mut good_actor = request.actors[0].clone();
    good_actor.weapon_id = "weapon:fork_Rose".into();
    let good_ctx = crate::gear::validation::ConfigContext {
        actor: &good_actor,
        actor_index: 0,
        limits: &limits,
    };
    let zero_duration = GearProcConfig::RoseThorn {
        atk_bonus: Ratio {
            numerator: 7,
            denominator: 50,
        },
        crit_damage_per_stack: Ratio {
            numerator: 3,
            denominator: 50,
        },
        duration_us: 0,
        broken_extension_us: 3_000_000,
    };
    match crate::weapons::rose::validate_config(&zero_duration, &good_ctx) {
        Err(err) => assert!(err.contains("positive"), "unexpected error: {err}"),
        Ok(_) => panic!("zero duration should reject"),
    }
    let zero_extension = GearProcConfig::RoseThorn {
        atk_bonus: Ratio {
            numerator: 7,
            denominator: 50,
        },
        crit_damage_per_stack: Ratio {
            numerator: 3,
            denominator: 50,
        },
        duration_us: 3_000_000,
        broken_extension_us: 0,
    };
    match crate::weapons::rose::validate_config(&zero_extension, &good_ctx) {
        Err(err) => assert!(err.contains("positive"), "unexpected error: {err}"),
        Ok(_) => panic!("zero Broken extension should reject"),
    }
    assert!(crate::weapons::rose::validate_config(&t1_proc(), &good_ctx).is_ok());
}

fn scenario_request() -> GameScenarioRequest {
    let mut req = research_request();
    req.stochastic_crit = None;
    req.relative_duration_us = 20_000_000;
    req.actors[0].character_id = "character:1004".into();
    req.actors[0].weapon_id = "weapon:fork_Rose".into();
    req.actors[0].weapon_tier = 1;
    req.actors[0].gear_procs = vec![t1_proc()];
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    let metadata = req.action_metadata[&0].clone();
    req.actions.clear();
    req.action_metadata.clear();
    req.action_metadata.insert(0, metadata);
    req
}

#[test]
fn rose_static_atk_bonus_joins_base_scaled_sum() {
    let mut req = scenario_request();
    super::append_synthetic_cast(&mut req, 0, InputActionRole::Normal, 0, None);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(
        report["hit_receipts"][0]["non_crit_damage"],
        json!("1140/1")
    );
}

#[test]
fn rose_thorn_grant_respects_icd() {
    let mut buffs = BuffManager::default();
    assert!(
        crate::weapons::rose::grant_thorn_stacks(&mut buffs, 0, 0, 1, 3_000_000, true).unwrap()
    );
    assert_eq!(thorn_count(&buffs, 0, 100_000), 1);
    assert!(
        !crate::weapons::rose::grant_thorn_stacks(&mut buffs, 0, 100_000, 1, 3_000_000, true)
            .unwrap()
    );
    assert_eq!(thorn_count(&buffs, 0, 200_000), 1);
    assert!(
        crate::weapons::rose::grant_thorn_stacks(&mut buffs, 0, 300_000, 1, 3_000_000, true)
            .unwrap()
    );
    assert_eq!(thorn_count(&buffs, 0, 400_000), 2);
}

#[test]
fn rose_thorn_grant_caps_at_ten_stacks_and_bypasses_icd_for_redirect() {
    let mut buffs = BuffManager::default();
    for _ in 0..20 {
        let _ = crate::weapons::rose::grant_thorn_stacks(
            &mut buffs, 0, 10_000_000, 1, 3_000_000, false,
        );
    }
    assert_eq!(thorn_count(&buffs, 0, 10_000_000), 10);
    assert!(
        crate::weapons::rose::grant_thorn_stacks(&mut buffs, 0, 10_000_100, 10, 3_000_000, false)
            .unwrap()
    );
    assert_eq!(thorn_count(&buffs, 0, 10_000_200), 10);
}

#[test]
fn rose_thorn_stacks_expire_after_duration() {
    let mut buffs = BuffManager::default();
    crate::weapons::rose::grant_thorn_stacks(&mut buffs, 0, 0, 5, 3_000_000, false).unwrap();
    assert_eq!(thorn_count(&buffs, 0, 2_999_999), 5);
    assert_eq!(thorn_count(&buffs, 0, 3_000_000), 0);
}

// COMBAT_FORMULA_CONTRACT §2: live ATK = ATK_static + base_atk × p (7/50 at T1),
// never ATK_static × (1+p). Fixture: static 1200, base 1000 → 1340 additive,
// 1368 under the multiplicative misreading.
#[test]
fn rose_t1_live_atk_additive_rejects_multiplicative() {
    let mut req = scenario_request();
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1200,
        denominator: 1,
    };
    super::append_synthetic_cast(&mut req, 0, InputActionRole::Normal, 0, None);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let damage = &report["hit_receipts"][0]["non_crit_damage"];
    assert_eq!(damage, &json!("1340/1"));
    assert_ne!(damage, &json!("1368/1"));
}

// Retrigger refresh is a shared Reset, not independent stack expiry: the 5 stacks
// granted at t=0 would expire at 3s without the t=2s retrigger moving the expiry
// to 5s, and independent expiry would leave only 1 stack at 4_999_999 us.
#[test]
fn rose_thorn_retrigger_resets_shared_expiry_all_at_once() {
    let mut buffs = BuffManager::default();
    crate::weapons::rose::grant_thorn_stacks(&mut buffs, 0, 0, 5, 3_000_000, true).unwrap();
    crate::weapons::rose::grant_thorn_stacks(&mut buffs, 0, 2_000_000, 1, 3_000_000, true).unwrap();
    assert_eq!(thorn_count(&buffs, 0, 3_000_000), 6);
    assert_eq!(thorn_count(&buffs, 0, 4_999_999), 6);
    assert_eq!(thorn_count(&buffs, 0, 5_000_000), 0);
}

#[test]
fn rose_redirect_grant_affects_its_hit_and_expires_before_equal_time_damage() {
    let mut request = scenario_request();
    for (time_us, role) in [
        (0, InputActionRole::RedirectSkill),
        (2_000_000, InputActionRole::Normal),
        (3_000_000, InputActionRole::Normal),
    ] {
        append_synthetic_cast(&mut request, time_us, role, 0, None);
    }
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    for (hit, expected) in report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .zip([2964, 2964, 2280])
    {
        assert_eq!(hit["crit_damage"], json!(format!("{}/{}", expected, 1)));
    }
}

#[test]
fn rose_dot_delivery_grants_one_stack_with_a_shared_icd() {
    let request = scenario_request();
    let mut state = GameSimulatorState::new(&request);
    let packet = crate::simulation::metadata::resolve_packet_metadata(
        &ntesim_data::native_action_metadata::resolve_damage_effect(
            "ntesim-current",
            ntesim_data::skill_levels::frozen::tables(),
            "GE_Player_Zankou_DotDamage",
        )
        .unwrap(),
        &BTreeMap::from([
            (
                crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID
                    .as_str()
                    .into(),
                crate::simulation::metadata::FIXED_CRIT_PROBABILITY_ID
                    .description()
                    .into(),
            ),
            (
                crate::characters::packet_semantics::ZANKOU_DOT_PACKET_HYPOTHESIS_ID.into(),
                crate::characters::packet_semantics::ZANKOU_DOT_PACKET_HYPOTHESIS_ID
                    .description()
                    .into(),
            ),
        ]),
    )
    .unwrap();
    for (time_us, expected) in [(0, 1), (100_000, 1), (300_000, 2)] {
        super::support::invoke_gear_hook(
            &request,
            &mut state,
            time_us,
            InputActionRole::Normal,
            &packet,
            true,
        );
        assert_eq!(thorn_count(&state.buffs, 0, time_us), expected);
    }
    assert_eq!(
        super::support::gear_zone_modifiers(&request, &state, 300_000, 0).crit_damage_bonus,
        ExactFraction::new(3, 25).unwrap()
    );
}

#[test]
fn rose_recognizes_adler_skill_train_dot_packet() {
    let request = scenario_request();
    let mut state = GameSimulatorState::new(&request);
    let assumptions = BTreeMap::from([
        (
            crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID
                .as_str()
                .into(),
            crate::simulation::metadata::FIXED_CRIT_PROBABILITY_ID
                .description()
                .into(),
        ),
        (
            crate::characters::packet_semantics::ADLER_SKILL_DOT_PACKET_HYPOTHESIS_ID.into(),
            crate::characters::packet_semantics::ADLER_SKILL_DOT_PACKET_HYPOTHESIS_ID
                .description()
                .into(),
        ),
    ]);
    let native = ntesim_data::native_action_metadata::resolve_damage_effect(
        "ntesim-current",
        ntesim_data::skill_levels::frozen::tables(),
        "GE_Player_Adler_Skill_Damage",
    )
    .unwrap();
    let packet =
        crate::simulation::metadata::resolve_packet_metadata(&native, &assumptions).unwrap();
    assert!(
        packet
            .semantic_tags
            .contains(crate::characters::packet_semantics::DOT_DAMAGE_TAG)
    );
    super::support::invoke_gear_hook(
        &request,
        &mut state,
        0,
        InputActionRole::Normal,
        &packet,
        true,
    );
    assert_eq!(thorn_count(&state.buffs, 0, 0), 1);
}

#[test]
fn rose_thorn_crit_damage_scales_with_stacks() {
    let req = scenario_request();
    let mut state = GameSimulatorState::new(&req);
    crate::weapons::rose::grant_thorn_stacks(&mut state.buffs, 0, 0, 4, 3_000_000, false).unwrap();
    let action_metadata = super::support::declared_test_action(
        "caller-test",
        crate::simulation::metadata::ActionCategory::Skill,
    );
    let packet = crate::simulation::metadata::PacketMetadata {
        damage_effect_id: "caller-effect".into(),
        native_ga_name: None,
        native_damage_type_ex: "EExecutionDamageTypeEX::DAMAGE_TYPE_PHYSICAL".into(),
        native_damage_source_category: None,
        damage_kind: ntesim_core::combat::DamageKind::Elemental,
        element: None,
        fixed_crit_rate: None,
        unbal_value: None,
        fixed_crit_rate_provenance: None,
        unbal_value_provenance: None,
        semantic_tags: Default::default(),
        semantic_provenance: None,
        source_evidence: None,
        provenance: crate::simulation::metadata::MetadataProvenance::AcceptedModelHypothesis {
            id: "test.rose_packet".into(),
            description: "Test-owned synthetic packet metadata.".into(),
        },
        packet_kind: Default::default(),
    };
    let buff_stats = BTreeMap::new();
    let owned_status_counts = BTreeMap::new();
    let ctx = crate::runtime_api::DamageContext {
        foreign_dot_stacks: 0,
        packet_group_id: None,
        command_index: 0,
        config: &req.actors[0],
        start_fons: None,
        ability_id: "caller-test",
        action_metadata: &action_metadata,
        target: &state.targets[0],
        actor: &state.actors[0],
        world_time_us: 1_000_000,
        time_us: 1_000_000,
        debuff_count: 0,
        damage_effect_id: "caller-effect",
        actor_index: 0,
        active_actor_index: 0,
        target_count: 1,
        non_boss_target_count: 1,
        target_index: 0,
        buffs: &state.buffs,
        buff_stats: (&buff_stats).into(),
        owned_status_counts: &owned_status_counts,
        team_has_three_elements: false,
    };
    let modifiers = crate::gear::damage_bonus(
        &ctx,
        &packet,
        crate::runtime_api::HitModifiers::new(crate::simulation::ExactFraction::new(0, 1).unwrap())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        modifiers.crit_damage_bonus,
        crate::simulation::ExactFraction::new(24, 100).unwrap()
    );
    assert_eq!(
        modifiers.atk_percent,
        crate::simulation::ExactFraction::new(14, 100).unwrap()
    );
}

fn live_dot_request() -> GameScenarioRequest {
    // Lacrimosa wearing Bohemian Rose; her own engine-scheduled Nightmare
    // ticks (not invoke_gear_hook) drive the thorn trigger below.
    let mut req = super::support::sample_s1_request();
    req.relative_duration_us = 16_000_000;
    req.actors[0].character_id = "character:1004".into();
    req.actors[0].weapon_id = "weapon:fork_Rose".into();
    req.actors[0].weapon_tier = 1;
    req.actors[0].gear_procs = vec![t1_proc()];
    req.actors[0].set_id = None;
    req.actors[0].talent_levels = Some([1, 8, 1]);
    req.actors[0].character_max_level = Some(80);
    req.actors[0].character_level = 80;
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].initial_energy = 120;
    req.actors[0].energy_capacity = 120;
    req.targets[0].resistance = Ratio::zero();
    req.targets[0].level = 80;
    req.model_assumptions.insert(
        crate::characters::lacrimosa::dot::DOT_SCHEDULING_HYPOTHESIS_ID.into(),
        crate::characters::lacrimosa::dot::DOT_SCHEDULING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_ID
            .description()
            .into(),
    );
    let mut ally = req.actors[0].clone();
    ally.character_id = "character:1019".into();
    ally.slot = 1;
    ally.weapon_id.clear();
    ally.gear_procs.clear();
    req.actors.push(ally);
    req.actions = vec![
        GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Lacrimosa_Skill".into(),
            damage_effect_id: "GE_Player_Lacrimosa_Skill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 2_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    req
}

#[test]
fn rose_live_dot_ticks_grant_thorn_stacks_end_to_end() {
    // Q1-V8: the wearer's own engine-scheduled Nightmare DoT ticks reach the
    // Rose gear hook with no synthetic invocation. One cast applies 5 stacks
    // (3s aggregate duration, decay tail): 14 ticks at t=1..14s, each 1s
    // apart so the shared 0.3s ICD never blocks and each grants exactly one
    // thorn stack (3s each). The wearer is off-field from t=2s; every tick
    // still attributes to actor 0 and grants to the off-field wearer.
    // Hand-derived: 14 ticks grant one thorn stack each (the 0.3s ICD never
    // blocks 1s-spaced ticks). Stacks accumulate to the cap of 10 and every
    // grant refreshes the shared 3s expiry (RefreshPolicy::Reset captures the
    // new application's values), so at the final t=16s all 10 stacks are live
    // with expiry t=17s. This exercises cap and refresh on the live path.
    let req = live_dot_request();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let ticks: Vec<&serde_json::Value> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Lacrimosa_Blood_Damage")
        .collect();
    assert_eq!(
        ticks.len(),
        14,
        "engine-scheduled blood ticks, no synthetic hooks"
    );
    for tick in &ticks {
        assert_eq!(tick["actor_index"], 0, "wearer-owned DoT attribution");
    }
    let final_us = report["final_buff_state"]["time_us"].as_u64().unwrap();
    assert_eq!(final_us, 16_000_000);
    let thorn_stacks: u32 = crate::simulation::tests::support::final_buffs(&report)
        .iter()
        .filter(|buff| {
            buff["id"] == crate::weapons::rose::THORN_BUFF_ID
                && buff["owner_actor"] == 0
                && buff["duration"]["expires_at_us"].as_u64().unwrap() > final_us
        })
        .map(|buff| buff["stacks"].as_u64().unwrap() as u32)
        .sum();
    assert_eq!(thorn_stacks, 10, "cap reached, shared expiry refreshed");
}

fn all_target_burst_request(tier: u32) -> GameScenarioRequest {
    let mut req = live_dot_request();
    let atk = [(7, 50), (7, 40), (21, 100), (49, 200), (7, 25)][tier as usize - 1];
    let crit = [(3, 50), (3, 40), (9, 100), (21, 200), (3, 25)][tier as usize - 1];
    req.actors[0].weapon_tier = tier;
    req.actors[0].gear_procs = vec![GearProcConfig::RoseThorn {
        atk_bonus: Ratio {
            numerator: atk.0,
            denominator: atk.1,
        },
        crit_damage_per_stack: Ratio {
            numerator: crit.0,
            denominator: crit.1,
        },
        duration_us: 3_000_000,
        broken_extension_us: 3_000_000,
    }];
    // Normal 8, Skill/Burst 1: Blood must not inherit its producer's talent.
    req.actors[0].talent_levels = Some([8, 1, 1]);
    req.actors[0].static_stats.crit_damage_bonus = Ratio::zero();
    req.actors[0].static_stats.damage_bonus = Ratio::zero();
    req.targets[0].defense = 0;
    req.targets[0].hp_max = 10_000_000;
    req.targets[0].hp_current = 10_000_000;
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    let id = crate::characters::lacrimosa::dot::ALL_TARGETS_HYPOTHESIS_ID;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    if let GameActionCommand::InvokeAbility {
        ability_id,
        damage_effect_id,
        energy_cost,
        ..
    } = &mut req.actions[0]
    {
        *ability_id = "GA_Lacrimosa_UltraSkill".into();
        *damage_effect_id = "GE_Player_Lacrimosa_UltraSkill1_Damage".into();
        *energy_cost = 120;
    }
    req.relative_duration_us = 16_999_999;
    req
}

fn blood_ticks(report: &serde_json::Value) -> Vec<&serde_json::Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Lacrimosa_Blood_Damage")
        .collect()
}

#[test]
fn rose_multi_target_nightmare_refinements_share_icd_and_apply_stacks_after_each_hit() {
    // Five stacks at Normal 8 give .515 * live ATK. No Redirect was cast,
    // so the first target has no Thorn and only the next hit sees its grant.
    let first = [(5871, 10), (4841, 8), (12463, 20), (25647, 40), (3296, 5)];
    let second = [
        (311163, 500),
        (208163, 320),
        (1358467, 2000),
        (5667987, 8000),
        (92288, 125),
    ];
    for tier in 1..=5 {
        let req = all_target_burst_request(tier);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let ticks = blood_ticks(&report);
        assert_eq!(ticks.len(), 28);
        let (n, d) = first[tier as usize - 1];
        assert_eq!(ticks[0]["crit_damage"], json!(format!("{}/{}", n, d)));
        let (n, d) = second[tier as usize - 1];
        assert_eq!(ticks[1]["crit_damage"], json!(format!("{}/{}", n, d)));
        assert_eq!(
            ticks[2]["crit_damage"], ticks[1]["crit_damage"],
            "one shared grant per timestamp"
        );
        let buffs = crate::simulation::tests::support::final_buffs(&report);
        let thorn = buffs
            .iter()
            .find(|buff| buff["id"] == crate::weapons::rose::THORN_BUFF_ID)
            .unwrap();
        assert_eq!(thorn["stacks"], 10);
        assert_eq!(thorn["owner_actor"], 0);
        assert_eq!(thorn["duration"]["expires_at_us"], 17_000_000);
        assert_eq!(report["final_buff_state"]["active_actor_index"], 1);
        assert!(ticks.iter().all(|hit| hit["actor_index"] == 0));
    }
}

#[test]
fn rose_multi_target_nightmare_full_horizon_and_exact_thorn_expiry() {
    let mut req = all_target_burst_request(1);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let total = blood_ticks(&report)
        .iter()
        .fold(ExactFraction::new(0, 1).unwrap(), |sum, hit| {
            sum.add(serde_json::from_value(hit["non_crit_damage"].clone()).unwrap())
                .unwrap()
        });
    // Per target the decay train has 40 stack-ticks: 2*5+3*4+3*3+3*2+3*1.
    // Two targets * 40 * .103 * 1140 = 9393.6, with no crit contribution.
    assert_eq!(total, ExactFraction::new(46968, 5).unwrap());
    req.relative_duration_us = 17_000_000;
    let expired = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(expired["execution_completed"], true);
    assert_eq!(expired["total_damage_dealt"], report["total_damage_dealt"]);
    assert!(
        !crate::simulation::tests::support::final_buffs(&expired)
            .iter()
            .any(|buff| buff["id"] == crate::weapons::rose::THORN_BUFF_ID)
    );
}

#[test]
fn rose_multi_target_nightmare_runs_two_legal_continuing_energy_cycles() {
    let source = "options iteration=1 duration=140 crit=fixed;
        lacrimosa char lvl=80/80 awakening=0 talent=8,1,1 qte=1 start_energy=120;
        lacrimosa add weapon=fork_Rose refine=1;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; target hp=100000000 def=0 resist=0;
        active lacrimosa; lacrimosa burst, attack:40; wait(20); lacrimosa burst; mint swap;";
    let req = crate::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    for id in [
        crate::characters::lacrimosa::dot::DOT_SCHEDULING_HYPOTHESIS_ID,
        crate::characters::lacrimosa::dot::ALL_TARGETS_HYPOTHESIS_ID,
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID,
    ] {
        assert_eq!(req.model_assumptions[id.as_str()], id.description());
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // Lacrimosa's native Burst costs 120 energy; two funded Bursts spend 240.
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 240);
    assert_eq!(report["final_actor_states"][0]["energy_conserved"], true);
    assert_eq!(blood_ticks(&report).len(), 56);
    assert_eq!(report["final_buff_state"]["active_actor_index"], 1);
}
