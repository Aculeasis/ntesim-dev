use super::*;
use serde_json::json;

pub(super) fn make_action(
    timestamp_us: u64,
    actor_index: usize,
    ability_id: &str,
    damage_effect_id: &str,
    energy_cost: u64,
    cooldown_us: u64,
) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index,
        ability_id: ability_id.into(),
        damage_effect_id: damage_effect_id.into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost,
        recovery_us: 0,
        cooldown_us,
        hit_delay_us: 100,
    }
}

pub(super) fn request() -> GameScenarioRequest {
    let mut req = sample_s1_request();
    req.actors[0].character_id = "character:1010".into();
    req.actors[0].weapon_id.clear();
    req.actors[0].gear_procs.clear();
    req.actors[0].set_id = None;
    req.actors[0].talent_levels = Some([1; 3]);
    req.actors[0].character_max_level = Some(80);
    req.actors[0].character_level = 80;
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.targets[0].resistance = Ratio::zero();
    req.targets[0].level = 80;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Nanally_Melee",
        "GE_Player_Nanally_Melee1_Damage",
        0,
        0,
    )];
    req
}

#[test]
fn nanally_normal_skill_and_burst_use_native_talent_coefficients_and_resonance_bonus() {
    // Test Normal Attack (GA_Nanally_Melee / GE_Player_Nanally_Melee1_Damage)
    // Coeffs: Lv1 = 0.296, Lv8 = 0.507, Lv9 (A3 resonance) = 0.548
    // At 1000 ATK, 0 RES, 0.5 DEF:
    // Lv1: 1000 * 0.296 * 0.5 = 148 / 1
    // Lv8: 1000 * 0.507 * 0.5 = 507 / 2
    // Lv9 (A3): 1000 * 0.548 * 0.5 = 274 / 1
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 148, 1),
        (8, 0, 507, 2),
        (8, 1, 507, 2),
        (8, 3, 274, 1),
        (8, 4, 274, 1),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([level, 1, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 1);
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            json!(format!("{}/{}", expected_num, expected_den))
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"
        );
    }

    // Test Skill (GA_Nanally_Skill / GE_Player_Nanally_Skill1_Damage)
    // Coeffs: Lv1 = 1.0, Lv8 = 1.714, Lv9 (A3 resonance) = 1.851
    // At 1000 ATK, 0 RES, 0.5 DEF:
    // Lv1: 1000 * 1.0 * 0.5 = 500 / 1 (five hits)
    // Lv8, A0: 1000 * 1.714 * 0.5 = 857 / 1 (five hits)
    // Lv8, A3: five ordinary hits plus one A3 follow-up.
    //   Hit 0: Primary hit (Lv9 = 1.851) -> 1000 * 1.851 * 0.5 = 1851 / 2
    //   Hit 5: A3 follow-up child hit (GE_Nanally010_Lv3_Damage, 50% ATK) -> 1000 * 0.5 * 0.5 = 250 / 1
    for (level, awakening, expected_hits, expected_primary_num, expected_primary_den) in [
        (1, 0, 5, 500, 1),
        (8, 0, 5, 857, 1),
        (8, 1, 5, 857, 1),
        (8, 3, 6, 1851, 2),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, level, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Nanally_Skill",
            "GE_Player_Nanally_Skill1_Damage",
            0,
            6_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], expected_hits);
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            json!(format!("{}/{}", expected_primary_num, expected_primary_den))
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
        );
        for hit in report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "GE_Player_Nanally_Skill1_Damage")
        {
            assert_eq!(
                hit["non_crit_damage"],
                json!(format!("{}/{}", expected_primary_num, expected_primary_den))
            );
            assert_eq!(hit["damage_effect_id"], "GE_Player_Nanally_Skill1_Damage");
        }
        if expected_hits == 6 {
            assert_eq!(
                report["hit_receipts"][2]["damage_effect_id"],
                "GE_Nanally010_Lv3_Damage"
            );
            assert_eq!(report["hit_receipts"][2]["non_crit_damage"], json!("270/1"));
        }
    }

    for (level, awakening, expected) in [
        (
            1,
            0,
            vec![
                ("GE_Player_Nanally_UltraSkill1_Damage", 2931, 2),
                ("GE_Player_Nanally_UltraSkill2_Damage", 423, 2),
                ("GE_Player_Nanally_UltraSkill2_Damage", 423, 2),
                ("GE_Player_Nanally_UltraSkill2_Damage", 423, 2),
                ("GE_Player_Nanally_UltraSkill2_Damage", 423, 2),
                ("GE_Player_Nanally_UltraSkill2_Damage", 423, 2),
                ("GE_Player_Nanally_UltraSkill3_Damage", 2477, 1),
            ],
        ),
        (
            8,
            0,
            vec![
                ("GE_Player_Nanally_UltraSkill1_Damage", 5023, 2),
                ("GE_Player_Nanally_UltraSkill2_Damage", 725, 2),
                ("GE_Player_Nanally_UltraSkill2_Damage", 725, 2),
                ("GE_Player_Nanally_UltraSkill2_Damage", 725, 2),
                ("GE_Player_Nanally_UltraSkill2_Damage", 725, 2),
                ("GE_Player_Nanally_UltraSkill2_Damage", 725, 2),
                ("GE_Player_Nanally_UltraSkill3_Damage", 4245, 1),
            ],
        ),
        (
            8,
            3,
            vec![
                ("GE_Player_Nanally_UltraSkill1_Damage", 5425, 2),
                ("GE_Player_Nanally_UltraSkill2_Damage", 783, 2),
                ("GE_Player_Nanally_UltraSkill2_Damage", 783, 2),
                ("GE_Player_Nanally_UltraSkill2_Damage", 783, 2),
                ("GE_Player_Nanally_UltraSkill2_Damage", 783, 2),
                ("GE_Player_Nanally_UltraSkill2_Damage", 783, 2),
                ("GE_Player_Nanally_UltraSkill3_Damage", 4585, 1),
            ],
        ),
        (
            8,
            4,
            vec![
                ("GE_Player_Nanally_UltraSkill1_Damage", 5425, 2),
                ("GE_Player_Nanally_UltraSkill2_Damage", 783, 2),
                ("GE_Player_Nanally_UltraSkill2_Damage", 783, 2),
                ("GE_Player_Nanally_UltraSkill2_Damage", 783, 2),
                ("GE_Player_Nanally_UltraSkill2_Damage", 783, 2),
                ("GE_Player_Nanally_UltraSkill2_Damage", 783, 2),
                ("GE_Player_Nanally_UltraSkill3_Damage", 4585, 1),
            ],
        ),
        (
            8,
            6,
            vec![
                ("GE_Player_Nanally_UltraSkill1_Damage", 24955, 8),
                ("GE_Player_Nanally_UltraSkill2_Damage", 18009, 40),
                ("GE_Player_Nanally_UltraSkill2_Damage", 18009, 40),
                ("GE_Player_Nanally_UltraSkill2_Damage", 18009, 40),
                ("GE_Player_Nanally_UltraSkill2_Damage", 18009, 40),
                ("GE_Player_Nanally_UltraSkill2_Damage", 18009, 40),
                ("GE_Player_Nanally_UltraSkill3_Damage", 21091, 4),
            ],
        ),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, 1, level]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actors[0].initial_energy = 100;
        req.actors[0].energy_capacity = 100;
        req.actions = vec![make_action(
            0,
            0,
            "GA_Nanally_UltraSkill",
            "GE_Player_Nanally_UltraSkill1_Damage",
            100,
            20_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 7);
        assert_eq!(report["pending_events"], 0);
        for (index, (effect, numerator, denominator)) in expected.iter().enumerate() {
            let hit = &report["hit_receipts"][index];
            assert_eq!(hit["damage_effect_id"], *effect, "packet {index}");
            assert_eq!(hit["actor_index"], 0, "packet {index}");
            assert_eq!(
                hit["time_us"],
                [
                    2_300_192, 2_382_032, 2_468_449, 2_547_640, 2_634_465, 2_716_783, 3_015_465
                ][index],
                "packet {index}"
            );
            assert_eq!(
                hit["non_crit_damage"],
                json!(format!("{}/{}", numerator, denominator)),
                "packet {index}"
            );
            assert_eq!(
                super::support::hit_metadata(&report, hit)["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q",
                "packet {index} category"
            );
        }
    }
}

#[test]
fn nanally_actions_respect_off_field_delivery_and_dead_target_boundaries() {
    let mut req = request();
    let mut actor_mint = req.actors[0].clone();
    actor_mint.character_id = "character:1019".into();
    actor_mint.slot = 1;
    req.actors.push(actor_mint);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Nanally_Skill",
            "GE_Player_Nanally_Skill1_Damage",
            0,
            6_000_000,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 1_200_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 5);
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|hit| hit["actor_index"] == 0)
    );
    req.actions.truncate(1);
    req.relative_duration_us = 429_636;
    let pending = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(pending["pending_events"], 6); // Five hits and unborn Authority; cooldown already reached.
    assert_eq!(pending["hits_settled"], 0);
    req.relative_duration_us = 429_637;
    req.targets[0].hp_current = 1;
    assert_eq!(
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap()["hits_settled"],
        1
    );

    let mut req_dead = request();
    req_dead.targets[0].hp_current = 0;
    let report_dead = run_game_scenario(&req_dead, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_dead["execution_completed"], true);
    assert_eq!(report_dead["hits_settled"], 0);

    let mut req_horizon = request();
    req_horizon.relative_duration_us = 50;
    let report_horizon = run_game_scenario(&req_horizon, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_horizon["execution_completed"], true);
    assert_eq!(report_horizon["hits_settled"], 0);
}

#[test]
fn nanally_burst_train_keeps_off_field_owner_and_respects_horizon_and_lethal_target() {
    let mut req = request();
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    req.actors.push(ally);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Nanally_UltraSkill",
            "GE_Player_Nanally_UltraSkill1_Damage",
            0,
            20_000_000,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 3_583_025,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 7);
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|hit| hit["actor_index"] == 0)
    );

    req.actions.truncate(1);
    req.absolute_limit_us = 2_300_191;
    let pending = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(pending["pending_events"], 9);
    assert_eq!(pending["hits_settled"], 0);
    req.absolute_limit_us = 3_600_000_000;
    req.relative_duration_us = 2_300_192;
    req.targets[0].hp_current = 1;
    let lethal = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(lethal["hits_settled"], 1);
    assert_eq!(
        lethal["hit_receipts"][0]["damage_effect_id"],
        "GE_Player_Nanally_UltraSkill1_Damage"
    );

    req.targets[0].hp_current = 0;
    req.relative_duration_us = 5_000_000;
    let dead = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(dead["execution_completed"], true);
    assert_eq!(dead["hits_settled"], 0);
}

#[test]
fn nanally_burst_creates_no_free_followup_callbacks_and_strict_timing_is_explicit() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(5);
    req.actors[0].initial_energy = 0;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Nanally_UltraSkill",
        "GE_Player_Nanally_UltraSkill1_Damage",
        0,
        20_000_000,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 7);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 0);
    for kind in ["nanally_a1_energy_grant", "nanally_a5_atk_buff"] {
        assert_eq!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["event_kind"] == kind)
                .count(),
            0,
            "{kind} must not be granted by summon creation"
        );
    }
    req.actors[0].talent_levels = None;
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::nanally::skill::BURST_PACKETS_HYPOTHESIS_ID.into(),
        "incorrect timing assumption".into(),
    );
    let mismatch = run_game_scenario(&req, &ExecutionLimits::default()).unwrap_err();
    assert!(
        mismatch.contains("conflicting reached model hypothesis"),
        "{mismatch}"
    );
    req.model_assumptions
        .remove(crate::characters::nanally::skill::BURST_PACKETS_HYPOTHESIS_ID.as_str());
    req.model_assumptions.insert(
        crate::characters::nanally::skill::BURST_PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::nanally::skill::BURST_PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::nanally::underboss::SUMMON_HYPOTHESIS_ID.into(),
        crate::characters::nanally::underboss::SUMMON_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let clock = crate::characters::nanally::clock::MODEL;
    req.model_assumptions
        .insert(clock.into(), clock.description().into());
    let accepted = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(accepted["hits_settled"], 7);
}

#[test]
fn nanally_five_skill_hits_keep_one_awakening_follow_up_and_one_callback_pair() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(5);
    req.actors[0].initial_energy = 0;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Nanally_Skill",
        "GE_Player_Nanally_Skill1_Damage",
        0,
        0,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 6);
    let direct: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Nanally_Skill1_Damage")
        .collect();
    for (index, hit) in direct.into_iter().enumerate() {
        // First two L2 E hits precede the A3/A5 callback; later E hits sample +2% base ATK.
        assert_eq!(
            hit["non_crit_damage"],
            if index < 2 {
                json!("540/1")
            } else {
                json!("2754/5")
            }
        );
    }
    assert_eq!(report["hit_receipts"][2]["non_crit_damage"], json!("270/1"));
    // A1 callback grants 2 whole units; the five Skill1 packets (ChargeAdd 1.26
    // = 63/50 each) grant 6 more whole units under nanally.ordinary_hit_energy_v1.
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 8);
    for kind in ["nanally_a1_energy_grant", "nanally_a5_atk_buff"] {
        assert_eq!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["event_kind"] == kind)
                .count(),
            1
        );
    }
    if let GameActionCommand::InvokeAbility { energy_cost, .. } = &mut req.actions[0] {
        *energy_cost = 1;
    }
    let rejected = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(rejected["hits_settled"], 0);
    assert_eq!(rejected["final_actor_states"][0]["granted_energy"], 0);
    assert_eq!(rejected["pending_events"], 0);
}

#[test]
fn nanally_skill_timing_requires_explicit_strict_acceptance() {
    let mut req = request();
    req.actions = vec![make_action(
        0,
        0,
        "GA_Nanally_Skill",
        "GE_Player_Nanally_Skill1_Damage",
        0,
        0,
    )];
    req.actors[0].talent_levels = None;
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::nanally::skill::SKILL_PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::nanally::skill::SKILL_PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::simulation::energy::NANALLY_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::NANALLY_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let clock = crate::characters::nanally::clock::MODEL;
    req.model_assumptions
        .insert(clock.into(), clock.description().into());
    let cycle = crate::HypothesisId::CombatPacketCycleAccrualV1;
    req.model_assumptions
        .insert(cycle.into(), cycle.description().into());
    assert_eq!(
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap()["hits_settled"],
        5
    );
}

#[test]
fn nanally_authority_crit_ends_on_field_exit_and_expiry_and_recast_rejects() {
    let mut req = request();
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    req.actors.push(ally);
    req.actions = vec![make_action(
        0,
        0,
        "GA_Nanally_Skill",
        "GE_Player_Nanally_Skill1_Damage",
        0,
        16_000_000,
    )];
    let initial = req.clone();
    let on_field = run_game_scenario(&req, &Default::default()).unwrap();
    for (index, hit) in on_field["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        // First E precedes Authority:500*1.94=970; later E500*2.24=1120.
        assert_eq!(
            hit["crit_damage"],
            json!(format!("{}/{}", if index == 0 { 970 } else { 1120 }, 1))
        );
    }
    req.actions.extend([
        GameActionCommand::SwitchActor {
            timestamp_us: 1_200_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 1_300_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        make_action(
            3_000_000,
            0,
            "GA_Nanally_Melee",
            "GE_Player_Nanally_Melee1_Damage",
            0,
            0,
        ),
    ]);
    let off_field = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(
        off_field["execution_completed"], true,
        "{}",
        off_field["halt_diagnostic"]
    );
    let normal = off_field["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["damage_effect_id"] == "GE_Player_Nanally_Melee1_Damage")
        .unwrap();
    assert_eq!(normal["crit_damage"], json!("7178/25"));
    assert!(
        off_field["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_kind"] == "nanally_authority_end")
    );
    let mut same = initial.clone();
    same.actions.extend([
        GameActionCommand::SwitchActor {
            timestamp_us: 1_200_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        make_action(
            2_000_000,
            0,
            "GA_Nanally_Melee",
            "GE_Player_Nanally_Melee1_Damage",
            0,
            0,
        ),
    ]);
    let report = run_game_scenario(&same, &Default::default()).unwrap();
    let normal = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["damage_effect_id"] == "GE_Player_Nanally_Melee1_Damage")
        .unwrap();
    assert_eq!(normal["crit_damage"], json!("8288/25"));
    let mut expired = initial.clone();
    expired.actions.push(make_action(
        12_238_863,
        0,
        "GA_Nanally_Melee",
        "GE_Player_Nanally_Melee1_Damage",
        0,
        0,
    ));
    let report = run_game_scenario(&expired, &Default::default()).unwrap();
    assert_eq!(report["hit_receipts"][5]["time_us"], 12_429_917);
    assert_eq!(report["hit_receipts"][5]["crit_damage"], json!("7178/25"));
    let mut denied = initial;
    denied.actions.push(make_action(
        1_500_000,
        0,
        "GA_Nanally_Skill",
        "GE_Player_Nanally_Skill1_Damage",
        0,
        0,
    ));
    let report = run_game_scenario(&denied, &Default::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert!(
        report["halt_diagnostic"]
            .as_str()
            .unwrap()
            .contains("on cooldown until 2429917")
    );
}

#[test]
fn nanally_normal_chain_damage_multiplicity_loop_and_resonance() {
    for (level, awakening, expected_hits, total_num, total_den) in [
        (
            1,
            0,
            vec![
                ("GE_Player_Nanally_Melee1_Damage", 148, 1),
                ("GE_Player_Nanally_Melee2_Damage", 194, 1),
                ("GE_Player_Nanally_Melee3_Damage", 61, 1),
                ("GE_Player_Nanally_Melee4_Damage", 105, 1),
                ("GE_Player_Nanally_Melee5_Damage", 635, 1),
                ("GE_Player_Nanally_Melee6_Damage", 69, 2),
                ("GE_Player_Nanally_Melee6_Damage", 69, 2),
                ("GE_Player_Nanally_Melee6_Damage", 69, 2),
                ("GE_Player_Nanally_Melee6_Damage", 69, 2),
                ("GE_Player_Nanally_Melee6_Damage", 69, 2),
                ("GE_Player_Nanally_Melee6_Damage", 69, 2),
                ("GE_Player_Nanally_Melee6_Damage", 69, 2),
                ("GE_Player_Nanally_Melee9_Damage", 771, 2),
                ("GE_Player_Nanally_Melee10_Damage", 773, 2),
                ("GE_Player_Nanally_Melee1_Damage", 148, 1),
            ],
            4609,
            2,
        ),
        (
            8,
            0,
            vec![
                ("GE_Player_Nanally_Melee1_Damage", 507, 2),
                ("GE_Player_Nanally_Melee2_Damage", 665, 2),
                ("GE_Player_Nanally_Melee3_Damage", 209, 2),
                ("GE_Player_Nanally_Melee4_Damage", 180, 1),
                ("GE_Player_Nanally_Melee5_Damage", 2177, 2),
                ("GE_Player_Nanally_Melee6_Damage", 59, 1),
                ("GE_Player_Nanally_Melee6_Damage", 59, 1),
                ("GE_Player_Nanally_Melee6_Damage", 59, 1),
                ("GE_Player_Nanally_Melee6_Damage", 59, 1),
                ("GE_Player_Nanally_Melee6_Damage", 59, 1),
                ("GE_Player_Nanally_Melee6_Damage", 59, 1),
                ("GE_Player_Nanally_Melee6_Damage", 59, 1),
                ("GE_Player_Nanally_Melee9_Damage", 1321, 2),
                ("GE_Player_Nanally_Melee10_Damage", 1325, 2),
                ("GE_Player_Nanally_Melee1_Damage", 507, 2),
            ],
            7897,
            2,
        ),
        (
            8,
            3,
            vec![
                ("GE_Player_Nanally_Melee1_Damage", 274, 1),
                ("GE_Player_Nanally_Melee2_Damage", 359, 1),
                ("GE_Player_Nanally_Melee3_Damage", 113, 1),
                ("GE_Player_Nanally_Melee4_Damage", 389, 2),
                ("GE_Player_Nanally_Melee5_Damage", 2351, 2),
                ("GE_Player_Nanally_Melee6_Damage", 64, 1),
                ("GE_Player_Nanally_Melee6_Damage", 64, 1),
                ("GE_Player_Nanally_Melee6_Damage", 64, 1),
                ("GE_Player_Nanally_Melee6_Damage", 64, 1),
                ("GE_Player_Nanally_Melee6_Damage", 64, 1),
                ("GE_Player_Nanally_Melee6_Damage", 64, 1),
                ("GE_Player_Nanally_Melee6_Damage", 64, 1),
                ("GE_Player_Nanally_Melee9_Damage", 1427, 2),
                ("GE_Player_Nanally_Melee10_Damage", 1431, 2),
                ("GE_Player_Nanally_Melee1_Damage", 274, 1),
            ],
            4267,
            1,
        ),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([level, 1, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = (0..6)
            .map(|index| {
                make_action(
                    super::nanally_clock::STARTS[index as usize],
                    0,
                    "GA_Nanally_Melee",
                    "GE_Player_Nanally_Melee1_Damage",
                    0,
                    0,
                )
            })
            .collect();
        req.relative_duration_us = 6_500_000;
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 15);
        let receipts = report["hit_receipts"].as_array().unwrap();
        for (i, (expected_effect, exp_num, exp_den)) in expected_hits.iter().enumerate() {
            assert_eq!(
                receipts[i]["damage_effect_id"], *expected_effect,
                "hit {i} effect mismatch"
            );
            assert_eq!(
                receipts[i]["non_crit_damage"],
                json!(format!("{}/{}", exp_num, exp_den)),
                "hit {i} damage mismatch"
            );
            assert_eq!(
                super::support::hit_metadata(&report, &receipts[i])["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"
            );
        }
        let sum: ExactFraction = receipts
            .iter()
            .try_fold(ExactFraction::new(0, 1).unwrap(), |acc, r| {
                acc.add(
                    serde_json::from_value::<ExactFraction>(r["non_crit_damage"].clone())
                        .map_err(|err| err.to_string())?,
                )
            })
            .unwrap();
        assert_eq!(
            sum,
            ExactFraction::new(total_num as u128, total_den as u128).unwrap()
        );
    }
}

#[test]
fn nanally_normal_chain_resets_and_strict() {
    let mut req = request();
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Nanally_Melee",
            "GE_Player_Nanally_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            2_000_001,
            0,
            "GA_Nanally_Melee",
            "GE_Player_Nanally_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 2);
    let receipts = report["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts[0]["damage_effect_id"],
        "GE_Player_Nanally_Melee1_Damage"
    );
    assert_eq!(
        receipts[1]["damage_effect_id"],
        "GE_Player_Nanally_Melee1_Damage"
    );

    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    let mut swap_req = request();
    swap_req.actors.push(ally);
    swap_req.actions = vec![
        make_action(
            0,
            0,
            "GA_Nanally_Melee",
            "GE_Player_Nanally_Melee1_Damage",
            0,
            0,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 700_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 1_000_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        make_action(
            2_400_000,
            0,
            "GA_Nanally_Melee",
            "GE_Player_Nanally_Melee1_Damage",
            0,
            0,
        ),
    ];
    let swap_report = run_game_scenario(&swap_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(swap_report["execution_completed"], true);
    let swap_receipts: Vec<_> = swap_report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Nanally_Melee1_Damage")
        .collect();
    assert_eq!(
        swap_receipts[0]["damage_effect_id"],
        "GE_Player_Nanally_Melee1_Damage"
    );
    assert_eq!(
        swap_receipts[1]["damage_effect_id"],
        "GE_Player_Nanally_Melee1_Damage"
    );

    let mut strict_req = request();
    strict_req.actors[0].talent_levels = None;
    strict_req.actors[0].purchased_skill_level = 1;
    strict_req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    strict_req.model_assumptions.insert(
        crate::characters::nanally::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::nanally::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    strict_req.model_assumptions.insert(
        crate::simulation::energy::NANALLY_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::NANALLY_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let clock = crate::characters::nanally::clock::MODEL;
    strict_req
        .model_assumptions
        .insert(clock.into(), clock.description().into());
    let cycle = crate::HypothesisId::CombatPacketCycleAccrualV1;
    strict_req
        .model_assumptions
        .insert(cycle.into(), cycle.description().into());
    let report = run_game_scenario(&strict_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 1);
}

#[test]
fn nanally_authority_early_end_recast() {
    let mut req = request();
    req.relative_duration_us = 10_000_000;

    // Test 1: Recast before 2.0s lock fails
    let mut locked_req = req.clone();
    locked_req.actions = vec![
        make_action(
            0,
            0,
            "GA_Nanally_Skill",
            "GE_Player_Nanally_Skill1_Damage",
            0,
            0,
        ),
        make_action(
            1_500_000,
            0,
            "GA_Nanally_Skill",
            "GE_Player_Nanally_Skill1_Damage",
            0,
            0,
        ),
    ];
    let rejected = run_game_scenario(&locked_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(rejected["execution_completed"], false);
    assert!(
        rejected["halt_diagnostic"]
            .as_str()
            .unwrap()
            .contains("on cooldown until 2429917")
    );

    // Test 2: Recast at 3.0s (after 2.0s lock) successfully terminates Authority
    let mut recast_req = req.clone();
    recast_req.actions = vec![
        // 1. Initial Skill cast (starts Authority for 12.0s, locks recast until2.429917s)
        make_action(
            0,
            0,
            "GA_Nanally_Skill",
            "GE_Player_Nanally_Skill1_Damage",
            0,
            0,
        ),
        // 2. Early-end recast at 3.0s (terminates Authority stance)
        make_action(
            3_000_000,
            0,
            "GA_Nanally_Skill",
            "GE_Player_Nanally_Skill1_Damage",
            0,
            13_000_000,
        ),
        // 3. Normal attack at 4.0s (Authority should be terminated)
        make_action(
            4_000_000,
            0,
            "GA_Nanally_Melee",
            "GE_Player_Nanally_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&recast_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );

    // 5 hits from initial Skill + 0 hits from recast + 1 hit from Melee = 6 hits settled
    assert_eq!(report["hits_settled"], 6);
}

#[test]
fn nanally_summons_and_stances_deactivate_on_defeat() {
    let mut req = request();
    req.relative_duration_us = 7_000_000;
    req.actors[0].initial_energy = 100;
    req.actors[0].energy_capacity = 100;
    req.model_assumptions.insert(
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
        timestamp_us: 5_000_000,
        actor_index: 0,
        delivered_damage: 100_000_000,
    }];

    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Nanally_UltraSkill",
            "GE_Player_Nanally_UltraSkill1_Damage",
            100,
            15_000_000,
        ),
        make_action(
            4_000_000,
            0,
            "GA_Nanally_Skill",
            "GE_Player_Nanally_Skill1_Damage",
            0,
            16_000_000,
        ),
    ];

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["final_actor_states"][0]["hp"], 0);
}

#[test]
fn nanally_two_continuing_cycles_preserve_resources_and_state() {
    let mut req = request();
    req.relative_duration_us = 130_000_000;
    req.actors[0].talent_levels = Some([8, 8, 8]);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actors[0].initial_energy = 100;
    req.actors[0].energy_capacity = 100;

    // Add teammate Actor 1 to participate in team rotations
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    req.actors.push(ally);

    req.model_assumptions.insert(
        crate::simulation::energy::NANALLY_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::NANALLY_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::nanally::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::nanally::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::nanally::skill::SKILL_PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::nanally::skill::SKILL_PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::nanally::skill::BURST_PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::nanally::skill::BURST_PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::nanally::underboss::HYPOTHESIS_ID.into(),
        crate::characters::nanally::underboss::HYPOTHESIS_ID
            .description()
            .into(),
    );

    let mut actions = Vec::new();
    for cycle in 0..2 {
        let base_t = cycle * 60_000_000;

        // 1. Skill cast at base_t (5 hits, starts Authority for 15s with A6, cooldown 16s)
        actions.push(make_action(
            base_t,
            0,
            "GA_Nanally_Skill",
            "GE_Player_Nanally_Skill1_Damage",
            0,
            16_000_000,
        ));

        // 2. Burst cast at base_t + 2s (100 energy, 7 direct hits + Underboss summon for 9s with A2, cooldown 15s)
        actions.push(make_action(
            base_t + 2_000_000,
            0,
            "GA_Nanally_UltraSkill",
            "GE_Player_Nanally_UltraSkill1_Damage",
            100,
            15_000_000,
        ));

        // 3. Normal attack combos while Underboss is active (coordinates Cat attacks)
        for stage in 0..5 {
            actions.push(make_action(
                base_t + 6_000_000 + super::nanally_clock::STARTS[stage as usize],
                0,
                "GA_Nanally_Melee",
                "GE_Player_Nanally_Melee1_Damage",
                0,
                0,
            ));
        }

        // 4. Switch to teammate at base_t + 10s
        actions.push(GameActionCommand::SwitchActor {
            timestamp_us: base_t + 12_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        });

        // 5. Teammate attacks
        actions.push(GameActionCommand::InvokeAbility {
            timestamp_us: base_t + 13_000_000,
            actor_index: 1,
            ability_id: "GA_Mint019_Melee".into(),
            damage_effect_id: "GE_Player_Mint_Melee1_Damage_New".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 100,
        });

        // 6. Switch back to Nanally at base_t + 14s
        actions.push(GameActionCommand::SwitchActor {
            timestamp_us: base_t + 16_000_000,
            target_actor_index: 0,
            cooldown_us: 0,
        });

        // 7. Normal attack loops to rebuild energy to 100 for the next cycle
        for loop_idx in 0..6 {
            let loop_base = base_t + 18_000_000 + loop_idx * 6_000_000;
            for stage in 0..5 {
                actions.push(make_action(
                    loop_base + super::nanally_clock::STARTS[stage as usize],
                    0,
                    "GA_Nanally_Melee",
                    "GE_Player_Nanally_Melee1_Damage",
                    0,
                    0,
                ));
            }
        }
    }

    req.actions = actions;

    let report1 = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report1["execution_completed"], true);
    assert_eq!(report1["invariants_passed"], true);
    assert_eq!(report1["success"], true);

    // Verify deterministic replay
    let report2 = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report1["hits_settled"], report2["hits_settled"]);
    assert_eq!(report1["hit_receipts"], report2["hit_receipts"]);
    assert_eq!(report1["final_actor_states"], report2["final_actor_states"]);
}
