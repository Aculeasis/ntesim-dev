use super::*;
use crate::simulation::tests::support::formula_damage_total;

fn mint_zero_start_request(case: &Case) -> GameScenarioRequest {
    let mut req = request(case, false);
    req.actors[0].talent_levels = Some([8, 8, 8]);
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.damage_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonuses.clear();
    req.targets[0].hp_max = 100_000_000;
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].defense = 1080;
    req.targets[0].resistance = Ratio::zero();
    req.relative_duration_us = 55_000_000;

    let mut actions = Vec::new();
    for input in 0..40 {
        actions.push(mint_timed_action(
            input * 800_000,
            "GA_Mint019_Melee",
            "GE_Player_Mint_Melee1_Damage_New",
            0,
            800_000,
            0,
        ));
    }
    for timestamp in [32_000_000, 38_500_000] {
        actions.push(mint_timed_action(
            timestamp,
            "GA_Mint019_Skill",
            "GE_Player_Mint_Skill1_Damage_New",
            0,
            1_200_000,
            6_000_000,
        ));
    }
    for input in 0..5 {
        actions.push(mint_timed_action(
            39_700_000 + input * 800_000,
            "GA_Mint019_Melee",
            "GE_Player_Mint_Melee1_Damage_New",
            0,
            800_000,
            0,
        ));
    }
    actions.push(mint_timed_action(
        43_700_000,
        "GA_Mint019_UltraSkill",
        "GE_Player_Mint_UtraSkill1_Damage_New",
        100,
        2_000_000,
        20_000_000,
    ));
    for input in 0..5 {
        actions.push(mint_timed_action(
            48_700_000 + input * 800_000,
            "GA_Mint019_Melee",
            "GE_Player_Mint_Melee1_Damage_New",
            0,
            800_000,
            0,
        ));
    }
    let expected_timestamps = (0_u64..40)
        .map(|input| input * 800_000)
        .chain([32_000_000, 38_500_000])
        .chain((0..5).map(|input| 39_700_000 + input * 800_000))
        .chain([43_700_000])
        .chain((0..5).map(|input| 48_700_000 + input * 800_000))
        .collect::<Vec<_>>();
    let actual_timestamps = actions
        .iter()
        .map(|action| match action {
            GameActionCommand::InvokeAbility { timestamp_us, .. } => *timestamp_us,
            _ => unreachable!("Mint rotation only contains ability inputs"),
        })
        .collect::<Vec<_>>();
    assert_eq!(actual_timestamps, expected_timestamps);
    let timing = |index: usize| match &actions[index] {
        GameActionCommand::InvokeAbility {
            recovery_us,
            cooldown_us,
            ..
        } => (*recovery_us, *cooldown_us),
        _ => unreachable!("Mint rotation only contains ability inputs"),
    };
    assert_eq!(timing(0), (800_000, 0));
    assert_eq!(timing(40), (1_200_000, 6_000_000));
    assert_eq!(timing(41), (1_200_000, 6_000_000));
    assert_eq!(timing(47), (2_000_000, 20_000_000));
    req.actions = actions;
    req
}

fn mint_expected_hit_sequence() -> Vec<(&'static str, &'static str)> {
    const NORMAL_DAMAGE_EFFECTS: [&str; 8] = [
        "GE_Player_Mint_Melee1_Damage_New",
        "GE_Player_Mint_Melee2_Damage_New",
        "GE_Player_Mint_Melee3_Damage_New",
        "GE_Player_Mint_Melee4_Damage_New",
        "GE_Player_Mint_Melee5_Damage_New",
        "GE_Player_Mint_Melee6_Damage_New",
        "GE_Player_Mint_Melee7_Damage_New",
        "GE_Player_Mint_Melee8_Damage_New",
    ];
    let mut expected = Vec::new();
    for _ in 0..8 {
        expected.extend(
            NORMAL_DAMAGE_EFFECTS
                .iter()
                .map(|effect| ("GA_Mint019_Melee", *effect)),
        );
    }
    expected.extend((0..4).map(|_| ("GA_Mint019_Skill", "GE_Player_Mint_Skill1_Damage_New")));
    expected.extend(
        NORMAL_DAMAGE_EFFECTS
            .iter()
            .map(|effect| ("GA_Mint019_Melee", *effect)),
    );
    expected.extend([
        (
            "GA_Mint019_UltraSkill",
            "GE_Player_Mint_UtraSkill1_Damage_New",
        ),
        (
            "GA_Mint019_UltraSkill",
            "GE_Player_Mint_UtraSkill2_Damage_New",
        ),
        (
            "GA_Mint019_UltraSkill",
            "GE_Player_Mint_UtraSkill3_Damage_New",
        ),
        (
            "GA_Mint019_UltraSkill",
            "GE_Player_Mint_UtraSkill4_Damage_New",
        ),
        (
            "GA_Mint019_UltraSkill",
            "GE_Player_Mint_UtraSkill5_Damage_New",
        ),
        (
            "GA_Mint019_UltraSkill",
            "GE_Player_Mint_UtraSkill6_Damage_New",
        ),
        (
            "GA_Mint019_UltraSkill",
            "GE_Player_Mint_UtraSkill7_Damage_New",
        ),
    ]);
    expected.extend(
        NORMAL_DAMAGE_EFFECTS
            .iter()
            .map(|effect| ("GA_Mint019_Melee", *effect)),
    );
    expected
}

fn assert_mint_spend_and_remainder(report: &Value) {
    assert_eq!(report["final_actor_states"][0]["energy"], 11);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 124);
    assert_eq!(report["final_actor_states"][0]["overflow_energy"], 13);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
    let burst_costs = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| {
            event["energy_change"]["source"] == "ability_cost"
                && event["energy_change"]["spent"] == 100
        })
        .collect::<Vec<_>>();
    assert_eq!(burst_costs.len(), 1);
    assert_eq!(burst_costs[0]["time_us"], 43_700_000);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        fraction(79, 100)
    );
    for recipient in 1..3 {
        assert_eq!(report["final_actor_states"][recipient]["energy"], 74);
        assert_eq!(
            report["final_actor_states"][recipient]["charge_remainder"],
            fraction(437, 500)
        );
    }
}

#[test]
fn mint_empty_energy_normal_skill_burst_rotation_spends_and_continues() {
    const NORMAL_CHARGE_MILLI: [u64; 8] = [578, 1283, 943, 1455, 1893, 1023, 1417, 2907];
    const SKILL_CHARGE_MILLI: [u64; 2] = [2450, 2450];
    let case = CASES
        .iter()
        .find(|case| case.owner == "character:1019")
        .unwrap();
    let req = mint_zero_start_request(case);
    let mut packet_charge_milli = Vec::new();
    for _ in 0..8 {
        packet_charge_milli.extend(NORMAL_CHARGE_MILLI);
    }
    packet_charge_milli.extend(SKILL_CHARGE_MILLI);
    packet_charge_milli.extend(SKILL_CHARGE_MILLI);
    packet_charge_milli.extend(NORMAL_CHARGE_MILLI);
    packet_charge_milli.extend(NORMAL_CHARGE_MILLI);
    assert_eq!(packet_charge_milli.len(), 84);

    let report = check_flow_with_spend(
        case,
        &req,
        &packet_charge_milli,
        &[(1, 1); 3],
        Some((76, 100, 100)),
    );
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hit_receipts"].as_array().unwrap().len(), 91);
    assert_eq!(
        serde_json::json!(formula_damage_total(&report)),
        json!("27671/1")
    );
    let receipts = report["hit_receipts"].as_array().unwrap();
    let expected = mint_expected_hit_sequence();
    assert_eq!(expected.len(), receipts.len());
    for (receipt, (ability, effect)) in receipts.iter().zip(expected) {
        assert_eq!(receipt["ability_id"], ability);
        assert_eq!(receipt["damage_effect_id"], effect);
    }
    for receipt in &receipts[64..68] {
        assert_eq!(receipt["ability_id"], "GA_Mint019_Skill");
    }
    assert_mint_spend_and_remainder(&report);
}

fn sakiri_zero_start_request(case: &Case) -> GameScenarioRequest {
    let mut req = request(case, false);
    req.actors[0].talent_levels = Some([8; 3]);
    req.actors[0].energy_capacity = 120;
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.damage_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonuses.clear();
    req.targets[0].hp_max = 100_000_000;
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].defense = 1080;
    req.targets[0].resistance = Ratio::zero();
    req.relative_duration_us = 35_000_000;

    let mut actions = Vec::new();
    for input in 0..20 {
        actions.push(sakiri_timed_action(
            input * 800_000,
            "GA_Sagiri_Melee",
            "GE_Player_Sagiri_Melee1_Damage",
            0,
            800_000,
            0,
        ));
    }
    for timestamp in [16_000_000, 22_000_000] {
        actions.push(sakiri_timed_action(
            timestamp,
            "GA_Sagiri_Skill",
            "GE_Player_Sagiri_Skill1_Damage",
            0,
            1_200_000,
            6_000_000,
        ));
    }
    actions.push(sakiri_timed_action(
        23_200_000,
        "GA_Sagiri_UltraSkill",
        "GE_Player_Sagiri_UltraSkill1_Damage",
        100,
        2_000_000,
        20_000_000,
    ));
    for input in 0..5 {
        actions.push(sakiri_timed_action(
            25_200_000 + input * 800_000,
            "GA_Sagiri_Melee",
            "GE_Player_Sagiri_Melee1_Damage",
            0,
            800_000,
            0,
        ));
    }

    let expected_timestamps = (0_u64..20)
        .map(|input| input * 800_000)
        .chain([16_000_000, 22_000_000, 23_200_000])
        .chain((0..5).map(|input| 25_200_000 + input * 800_000))
        .collect::<Vec<_>>();
    let actual_timestamps = actions
        .iter()
        .map(|action| match action {
            GameActionCommand::InvokeAbility { timestamp_us, .. } => *timestamp_us,
            _ => unreachable!("Sakiri rotation only contains ability inputs"),
        })
        .collect::<Vec<_>>();
    assert_eq!(actual_timestamps, expected_timestamps);
    let timing = |index: usize| match &actions[index] {
        GameActionCommand::InvokeAbility {
            recovery_us,
            cooldown_us,
            ..
        } => (*recovery_us, *cooldown_us),
        _ => unreachable!("Sakiri rotation only contains ability inputs"),
    };
    assert_eq!(timing(0), (800_000, 0));
    assert_eq!(timing(20), (1_200_000, 6_000_000));
    assert_eq!(timing(21), (1_200_000, 6_000_000));
    assert_eq!(timing(22), (2_000_000, 20_000_000));
    req.actions = actions;
    req
}

fn sakiri_expected_hits() -> Vec<(&'static str, &'static str, u64, u64)> {
    const NORMAL_DAMAGE: [(&str, u64, u64); 10] = [
        ("GE_Player_Sagiri_Melee1_Damage", 691, 2),
        ("GE_Player_Sagiri_Melee2_Damage", 237, 2),
        ("GE_Player_Sagiri_Melee3_Damage", 1297, 2),
        ("GE_Player_Sagiri_Melee8_Damage", 27, 2),
        ("GE_Player_Sagiri_Melee4_Damage", 1227, 2),
        ("GE_Player_Sagiri_Melee5_Damage", 567, 2),
        ("GE_Player_Sagiri_Melee5_Damage", 567, 2),
        ("GE_Player_Sagiri_Melee5_Damage", 567, 2),
        ("GE_Player_Sagiri_Melee6_Damage", 30, 1),
        ("GE_Player_Sagiri_Melee7_Damage", 910, 1),
    ];
    let mut expected = Vec::new();
    for _ in 0..4 {
        expected.extend(
            NORMAL_DAMAGE
                .iter()
                .map(|(effect, n, d)| ("GA_Sagiri_Melee", *effect, *n, *d)),
        );
    }
    for _ in 0..2 {
        expected.extend([
            ("GA_Sagiri_Skill", "GE_Player_Sagiri_Skill1_Damage", 1438, 1),
            ("GA_Sagiri_Skill", "GE_Player_Sagiri_Skill2_Damage", 3979, 2),
        ]);
    }
    expected.extend((0..5).map(|_| {
        (
            "GA_Sagiri_UltraSkill",
            "GE_Player_Sagiri_UltraSkill1_Damage",
            2057,
            2,
        )
    }));
    expected.push((
        "GA_Sagiri_UltraSkill",
        "GE_Player_Sagiri_UltraSkill2_Damage",
        5141,
        2,
    ));
    expected.extend(
        NORMAL_DAMAGE
            .iter()
            .map(|(effect, n, d)| ("GA_Sagiri_Melee", *effect, *n, *d)),
    );
    expected
}

fn assert_sakiri_damage_receipts(report: &Value) {
    let expected = sakiri_expected_hits();
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(expected.len(), hits.len());
    for (hit, (ability, effect, numerator, denominator)) in hits.iter().zip(expected) {
        assert_eq!(hit["ability_id"], ability);
        assert_eq!(hit["damage_effect_id"], effect);
        assert_eq!(
            hit["non_crit_damage"],
            json!(format!("{}/{}", numerator, denominator))
        );
    }
}

fn assert_sakiri_continuing_energy_flow(
    report: &Value,
    req: &GameScenarioRequest,
    packets: &[u64],
) {
    let case = CASES
        .iter()
        .find(|case| case.owner == "character:1003")
        .unwrap();
    assert_flow_with_spend(
        case,
        req,
        report,
        packets,
        &[(1, 1); 3],
        Some((44, 101, 100)),
    );
}

#[test]
fn sakiri_zero_start_normal_skill_burst_and_continuation_rotation() {
    const NORMAL_CHARGE_MILLI: [u64; 10] = [2014, 690, 3785, 81, 3583, 1654, 1654, 1654, 176, 5311];
    const SKILL_CHARGE_MILLI: [u64; 2] = [3902, 5398];
    let case = CASES
        .iter()
        .find(|case| case.owner == "character:1003")
        .unwrap();
    let req = sakiri_zero_start_request(case);
    let mut charge_packets = Vec::new();
    for _ in 0..4 {
        charge_packets.extend(NORMAL_CHARGE_MILLI);
    }
    charge_packets.extend(SKILL_CHARGE_MILLI);
    charge_packets.extend(SKILL_CHARGE_MILLI);
    charge_packets.extend(NORMAL_CHARGE_MILLI);
    assert_eq!(charge_packets.len(), 54);

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hit_receipts"].as_array().unwrap().len(), 60);
    assert_eq!(
        serde_json::json!(formula_damage_total(&report)),
        json!("32218/1")
    );
    assert_sakiri_damage_receipts(&report);
    assert_sakiri_continuing_energy_flow(&report, &req, &charge_packets);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 121);
    assert_eq!(report["final_actor_states"][0]["overflow_energy"], 0);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
    assert_eq!(report["final_actor_states"][0]["energy"], 21);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        fraction(61, 100)
    );
    for recipient in 1..3 {
        assert_eq!(
            report["final_actor_states"][recipient]["granted_energy"],
            72
        );
        assert_eq!(report["final_actor_states"][recipient]["energy"], 72);
        assert_eq!(
            report["final_actor_states"][recipient]["overflow_energy"],
            0
        );
        assert_eq!(report["final_actor_states"][recipient]["spent_energy"], 0);
        assert_eq!(
            report["final_actor_states"][recipient]["charge_remainder"],
            fraction(483, 500)
        );
    }
    assert!(report["resolved_model_assumptions"]["sakiri.ordinary_hit_energy_v1"].is_string());
    assert!(report["resolved_model_assumptions"]["sakiri.normal_chain_v1"].is_string());
    assert!(report["resolved_model_assumptions"]["sakiri.skill_burst_packets_v1"].is_string());
}
