use super::*;
use crate::simulation::tests::support::formula_damage_total;

fn continuing_request(case: &Case, owner: &str) -> GameScenarioRequest {
    let mut req = request(case, false);
    req.actors[0].talent_levels = Some([8; 3]);
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.damage_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonuses.clear();
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.targets[0].hp_max = 100_000_000;
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].defense = 1080;
    req.targets[0].resistance = Ratio::zero();
    let skill = case.skill.expect("continuing owner has a modeled Skill");
    let (
        pre_normal_inputs,
        skill_times,
        burst_time,
        continuation_time,
        horizon,
        burst,
        burst_effect,
    ) = match owner {
        "shinku" => (
            30,
            [24_000_000, 30_000_000],
            31_200_000,
            44_450_242,
            49_250_242,
            "GA_Shinku_UltraSkill",
            "GE_Player_Shinku_UltraSkill1_Damage",
        ),
        "linko" => (
            20,
            [16_000_000, 22_000_000],
            23_200_000,
            31_200_000,
            36_000_000,
            "GA_Radio072_UltraSkill",
            "GE_Player_Radio072_UltralSkillLTE_AOE_Level4_Damage",
        ),
        "hathor" => (
            20,
            [16_000_000, 22_000_000],
            23_200_000,
            35_718_597,
            36_000_000,
            "GA_Hathor_UltraSkill",
            "GE_Player_Hathor_UltraSkill1_Damage",
        ),
        "skia" => (
            25,
            [20_000_000, 26_000_000],
            27_200_000,
            35_200_000,
            40_000_000,
            "GA_Skia_UltraSkill",
            "GE_Player_Skia_UltraSkill1_Damage",
        ),
        _ => unreachable!("owner fixture is Shinku, Linko, Hathor, or Skia"),
    };
    req.relative_duration_us = horizon;

    let mut actions = Vec::new();
    for input in 0..pre_normal_inputs {
        actions.push(mint_timed_action(
            input * 800_000,
            case.melee.0,
            case.melee.1,
            0,
            800_000,
            0,
        ));
    }
    for timestamp in skill_times {
        actions.push(mint_timed_action(
            timestamp, skill.0, skill.1, 0, 1_200_000, 6_000_000,
        ));
    }
    actions.push(mint_timed_action(
        burst_time,
        burst,
        burst_effect,
        if owner == "shinku" { 60 } else { 100 },
        if owner == "shinku" {
            7_253_918
        } else {
            2_000_000
        },
        20_000_000,
    ));
    for input in 0..5 {
        actions.push(mint_timed_action(
            continuation_time + input * 800_000,
            case.melee.0,
            case.melee.1,
            0,
            800_000,
            0,
        ));
    }
    let timestamps = actions
        .iter()
        .map(|action| match action {
            GameActionCommand::InvokeAbility { timestamp_us, .. } => *timestamp_us,
            _ => unreachable!("owner rotation only contains ability inputs"),
        })
        .collect::<Vec<_>>();
    assert_eq!(timestamps.len(), pre_normal_inputs as usize + 8);
    assert_eq!(
        &timestamps[..pre_normal_inputs as usize],
        &(0_u64..pre_normal_inputs)
            .map(|i| i * 800_000)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        &timestamps[pre_normal_inputs as usize..],
        &[
            skill_times[0],
            skill_times[1],
            burst_time,
            continuation_time,
            continuation_time + 800_000,
            continuation_time + 1_600_000,
            continuation_time + 2_400_000,
            continuation_time + 3_200_000,
        ]
    );
    req.actions = actions;
    req
}

pub(super) fn push_packets(
    expected: &mut Vec<(&'static str, &'static str, u64, u64)>,
    ability: &'static str,
    packets: &[(&'static str, u64, u64)],
) {
    expected.extend(
        packets
            .iter()
            .map(|(effect, numerator, denominator)| (ability, *effect, *numerator, *denominator)),
    );
}

pub(super) fn assert_hit_oracle(
    report: &Value,
    expected: &[(&'static str, &'static str, u64, u64)],
    total: (u64, u64),
) {
    let receipts = report["hit_receipts"].as_array().unwrap();
    assert_eq!(receipts.len(), expected.len());
    for (receipt, (ability, effect, numerator, denominator)) in receipts.iter().zip(expected) {
        assert_eq!(
            receipt["ability_id"], *ability,
            "command {} at {} expected {effect}",
            receipt["command_index"], receipt["time_us"]
        );
        assert_eq!(receipt["damage_effect_id"], *effect);
        assert_eq!(
            receipt["non_crit_damage"],
            fraction(*numerator, *denominator),
            "{ability} / {effect}"
        );
    }
    assert_eq!(
        serde_json::json!(formula_damage_total(&report)),
        fraction(total.0, total.1)
    );
}

pub(super) fn assert_one_burst_spend(report: &Value, burst_time_us: u64, cost: u64) {
    let costs = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| {
            event["energy_change"]["source"] == "ability_cost"
                && event["energy_change"]["spent"] == cost
        })
        .collect::<Vec<_>>();
    assert_eq!(costs.len(), 1);
    assert_eq!(costs[0]["time_us"], burst_time_us);
    assert_eq!(costs[0]["energy_change"]["energy_before"], cost);
    assert_eq!(costs[0]["energy_change"]["spent"], cost);
}

#[test]
fn linko_empty_energy_rotation_spends_burst_and_continues() {
    const NORMAL: [(&str, u64, u64); 13] = [
        ("GE_Player_Radio072_Melee1_Damage", 425, 1),
        ("GE_Player_Radio072_Melee2_Damage", 291, 2),
        ("GE_Player_Radio072_Melee2_Damage", 291, 2),
        ("GE_Player_Radio072_Melee3_Damage", 346, 1),
        ("GE_Player_Radio072_Melee4_1_Damage", 114, 1),
        ("GE_Player_Radio072_Melee4_2_Damage", 511, 2),
        ("GE_Player_Radio072_Melee4_3_Damage", 119, 1),
        ("GE_Player_Radio072_Melee4_3_Damage", 119, 1),
        ("GE_Player_Radio072_Melee4_3_Damage", 119, 1),
        ("GE_Player_Radio072_Melee4_3_Damage", 119, 1),
        ("GE_Player_Radio072_Melee4_3_Damage", 119, 1),
        ("GE_Player_Radio072_Melee4_Damage", 295, 1),
        ("GE_Player_Radio072_Melee5_Damage", 634, 1),
    ];
    const SKILL: [(&str, u64, u64); 4] = [
        ("GE_Player_Radio072_Skill1_Damage", 305, 2),
        ("GE_Player_Radio072_Skill2_Damage", 237, 2),
        ("GE_Player_Radio072_Skill3_Damage", 204, 1),
        ("GE_Player_Radio072_Skill4_Damage", 382, 1),
    ];
    const NORMAL_CHARGE: [u64; 13] = [
        3105, 1065, 1065, 2531, 835, 1867, 869, 869, 869, 869, 869, 2154, 4635,
    ];
    const SKILL_CHARGE: [u64; 4] = [1493, 1162, 1996, 3749];
    let case = CASES
        .iter()
        .find(|case| case.owner == "character:1072")
        .unwrap();
    let req = continuing_request(case, "linko");
    let mut expected_hits = Vec::new();
    for _ in 0..4 {
        push_packets(&mut expected_hits, "GA_Radio072_Melee", &NORMAL);
    }
    for _ in 0..2 {
        push_packets(&mut expected_hits, "GA_Radio072_Skill", &SKILL);
    }
    push_packets(
        &mut expected_hits,
        "GA_Radio072_UltraSkill",
        &[(
            "GE_Player_Radio072_UltralSkillLTE_AOE_Level4_Damage",
            857,
            1,
        )],
    );
    push_packets(&mut expected_hits, "GA_Radio072_Melee", &NORMAL);
    assert_eq!(expected_hits.len(), 74);

    let mut charge = Vec::new();
    for _ in 0..4 {
        charge.extend(NORMAL_CHARGE);
    }
    charge.extend(SKILL_CHARGE);
    charge.extend(SKILL_CHARGE);
    charge.extend(NORMAL_CHARGE);
    assert_eq!(charge.len(), 73);
    let report = check_flow_with_spend(case, &req, &charge, &[(1, 1); 3], Some((60, 100, 100)));
    assert_hit_oracle(&report, &expected_hits, (34_697, 2));
    assert_one_burst_spend(&report, 23_200_000, 100);
    assert_eq!(report["final_actor_states"][0]["energy"], 21);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 124);
    assert_eq!(report["final_actor_states"][0]["overflow_energy"], 3);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        fraction(81, 100)
    );
    for recipient in 1..3 {
        assert_eq!(report["final_actor_states"][recipient]["energy"], 74);
        assert_eq!(
            report["final_actor_states"][recipient]["charge_remainder"],
            fraction(443, 500)
        );
    }
}

#[test]
fn hathor_empty_energy_rotation_spends_burst_and_continues() {
    const NORMAL: [(&str, u64, u64); 13] = [
        ("GE_Player_Hathor_Melee1_Damage", 208, 1),
        ("GE_Player_Hathor_Melee2_Damage", 138, 1),
        ("GE_Player_Hathor_Melee2_1_Damage", 233, 2),
        ("GE_Player_Hathor_Melee2_1_Damage", 233, 2),
        ("GE_Player_Hathor_Melee2_1_Damage", 233, 2),
        ("GE_Player_Hathor_Melee2_1_Damage", 233, 2),
        ("GE_Player_Hathor_Melee3_Damage", 114, 1),
        ("GE_Player_Hathor_Melee3_Damage", 114, 1),
        ("GE_Player_Hathor_Melee4_Damage", 192, 1),
        ("GE_Player_Hathor_Melee4_1_Damage", 821, 2),
        ("GE_Player_Hathor_Melee4_2_Damage", 852, 1),
        ("GE_Player_Hathor_Melee5_Damage", 378, 1),
        ("GE_Player_Hathor_Melee5_1_Damage", 1883, 2),
    ];
    const SKILL: [(&str, u64, u64); 3] = [
        ("GE_Player_Hathor_Skill1_Damage", 857, 1),
        ("GE_Player_Hathor_Skill1_Damage", 857, 1),
        ("GE_Player_Hathor_Skill11_Damage", 857, 1),
    ];
    const NORMAL_CHARGE: [u64; 13] = [
        1219, 807, 681, 681, 681, 681, 664, 664, 1121, 2403, 4982, 2212, 5505,
    ];
    const SKILL_CHARGE: [u64; 3] = [2833; 3];
    let case = CASES
        .iter()
        .find(|case| case.owner == "character:1025")
        .unwrap();
    let req = continuing_request(case, "hathor");
    let mut expected_hits = Vec::new();
    for _ in 0..4 {
        push_packets(&mut expected_hits, "GA_Hathor_Melee", &NORMAL);
    }
    for _ in 0..2 {
        push_packets(&mut expected_hits, "GA_Hathor_Skill", &SKILL);
    }
    push_packets(
        &mut expected_hits,
        "GA_Hathor_UltraSkill",
        &[
            ("GE_Player_Hathor_UltraSkill1_Damage", 37_927, 40),
            ("GE_Player_Hathor_UltraSkill1_Damage", 37_927, 40),
            ("GE_Player_Hathor_UltraSkill2_Damage", 200_077, 40),
        ],
    );
    push_packets(&mut expected_hits, "GA_Hathor_Melee", &NORMAL);
    assert_eq!(expected_hits.len(), 74);

    let mut charge = Vec::new();
    for _ in 0..4 {
        charge.extend(NORMAL_CHARGE);
    }
    charge.extend(SKILL_CHARGE);
    charge.extend(SKILL_CHARGE);
    charge.extend(NORMAL_CHARGE);
    assert_eq!(charge.len(), 71);
    let report = check_flow_with_spend(case, &req, &charge, &[(1, 1); 3], Some((58, 100, 100)));
    assert_hit_oracle(&report, &expected_hits, (1_244_411, 40));
    assert_one_burst_spend(&report, 23_200_000, 100);
    assert_eq!(report["final_actor_states"][0]["energy"], 22);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 128);
    assert_eq!(report["final_actor_states"][0]["overflow_energy"], 6);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        fraction(503, 1000)
    );
    for recipient in 1..3 {
        assert_eq!(report["final_actor_states"][recipient]["energy"], 77);
        assert_eq!(
            report["final_actor_states"][recipient]["charge_remainder"],
            fraction(509, 5000)
        );
    }
}

#[test]
fn skia_empty_energy_rotation_spends_burst_and_continues() {
    const NORMAL: [(&str, u64, u64); 9] = [
        ("GE_Player_Skia_Melee1_Damage", 521, 2),
        ("GE_Player_Skia_Melee2_Damage", 63, 2),
        ("GE_Player_Skia_Melee2_Damage", 63, 2),
        ("GE_Player_Skia_Melee4_Damage", 162, 1),
        ("GE_Player_Skia_Melee5_Damage", 275, 1),
        ("GE_Player_Skia_Melee6_Damage", 787, 2),
        ("GE_Player_Skia_Melee7_Damage", 335, 1),
        ("GE_Player_Skia_Melee8_Damage", 252, 1),
        ("GE_Player_Skia_Melee9_Damage", 949, 2),
    ];
    const SKILL: [(&str, u64, u64); 6] = [
        ("GE_Player_Skia_Skill1_Damage", 257, 1),
        ("GE_Player_Skia_Skill1_Damage", 257, 1),
        ("GE_Player_Skia_Skill1_Damage", 257, 1),
        ("GE_Player_Skia_Skill1_Damage", 257, 1),
        ("GE_Player_Skia_Skill2_Damage", 79, 2),
        ("GE_Player_Skia_Skill3_Damage", 435, 2),
    ];
    const NORMAL_CHARGE: [u64; 9] = [2525, 306, 306, 1572, 2667, 3820, 3250, 2448, 4606];
    const SKILL_CHARGE: [u64; 6] = [1080, 1080, 1080, 1080, 166, 914];
    let case = CASES
        .iter()
        .find(|case| case.owner == "character:1008")
        .unwrap();
    let req = continuing_request(case, "skia");
    let mut expected_hits = Vec::new();
    for _ in 0..5 {
        push_packets(&mut expected_hits, "GA_Skia_Melee", &NORMAL);
    }
    for _ in 0..2 {
        push_packets(&mut expected_hits, "GA_Skia_Skill", &SKILL);
    }
    push_packets(
        &mut expected_hits,
        "GA_Skia_UltraSkill",
        &[
            ("GE_Player_Skia_UltraSkill1_Damage", 2487, 1),
            ("GE_Player_Skia_UltraSkill2_Damage", 599, 1),
            ("GE_Player_Skia_UltraSkill2_Damage", 599, 1),
            ("GE_Player_Skia_UltraSkill2_Damage", 599, 1),
        ],
    );
    push_packets(&mut expected_hits, "GA_Skia_Melee", &NORMAL);
    assert_eq!(expected_hits.len(), 70);

    let mut charge = Vec::new();
    for _ in 0..5 {
        charge.extend(NORMAL_CHARGE);
    }
    charge.extend(SKILL_CHARGE);
    charge.extend(SKILL_CHARGE);
    charge.extend(NORMAL_CHARGE);
    assert_eq!(charge.len(), 66);
    let report = check_flow_with_spend(case, &req, &charge, &[(1, 1); 3], Some((57, 100, 100)));
    assert_hit_oracle(&report, &expected_hits, (20_147, 1));
    assert_one_burst_spend(&report, 27_200_000, 100);
    assert_eq!(report["final_actor_states"][0]["energy"], 21);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 139);
    assert_eq!(report["final_actor_states"][0]["overflow_energy"], 18);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        fraction(4, 5)
    );
    for recipient in 1..3 {
        assert_eq!(report["final_actor_states"][recipient]["energy"], 83);
        assert_eq!(
            report["final_actor_states"][recipient]["charge_remainder"],
            fraction(22, 25)
        );
    }
}

#[test]
fn shinku_empty_energy_rotation_spends_burst_and_continues() {
    const NORMAL: [(&str, u64); 10] = [
        ("GE_Player_Shinku_Melee1_1_Damage", 312),
        ("GE_Player_Shinku_Melee1_2_Damage", 528),
        ("GE_Player_Shinku_Melee1_2_1_Damage", 351),
        ("GE_Player_Shinku_Melee1_3_Damage", 504),
        ("GE_Player_Shinku_Melee1_3_1_Damage", 298),
        ("GE_Player_Shinku_Melee1_3_2_Damage", 338),
        ("GE_Player_Shinku_Melee1_3_3_Damage", 722),
        ("GE_Player_Shinku_Melee1_4_Damage", 1397),
        ("GE_Player_Shinku_Melee1_4_1_Damage", 853),
        ("GE_Player_Shinku_Melee1_5_Damage", 1935),
    ];
    const TIDAL_ASSAULT: [(&str, u64); 12] = [
        ("GE_Player_Shinku_Melee1_Rage_Damage", 600),
        ("GE_Player_Shinku_Melee1_1_Rage_Damage", 473),
        ("GE_Player_Shinku_Melee2_Rage_Damage", 699),
        ("GE_Player_Shinku_Melee3_Rage_Damage", 435),
        ("GE_Player_Shinku_Melee3_1_Rage_Damage", 134),
        ("GE_Player_Shinku_Melee3_1_Rage_Damage", 134),
        ("GE_Player_Shinku_Melee3_1_Rage_Damage", 134),
        ("GE_Player_Shinku_Melee3_1_Rage_Damage", 134),
        ("GE_Player_Shinku_Melee3_1_Rage_Damage", 134),
        ("GE_Player_Shinku_Melee4_Rage_Damage", 919),
        ("GE_Player_Shinku_Melee4_1_Rage_Damage", 1177),
        ("GE_Player_Shinku_Melee5_Rage_Damage", 797),
    ];
    const SKILL: [(&str, u64); 3] = [
        ("GE_Player_Shinku_Skill1_1_Damage", 296),
        ("GE_Player_Shinku_Skill1_2_Damage", 396),
        ("GE_Player_Shinku_Skill1_3_Damage", 1364),
    ];
    const BURST: [(&str, u64); 9] = [
        ("GE_Player_Shinku_UltraSkill1_Damage", 317),
        ("GE_Player_Shinku_UltraSkill2_Damage", 2057),
        ("GE_Player_Shinku_UltraSkill3_Damage", 943),
        ("GE_Player_Shinku_UltraSkill3_Damage", 943),
        ("GE_Player_Shinku_UltraSkill3_Damage", 943),
        ("GE_Player_Shinku_UltraSkill3_Damage", 943),
        ("GE_Player_Shinku_UltraSkill3_Damage", 943),
        ("GE_Player_Shinku_UltraSkill4_Damage", 2468),
        ("GE_Player_Shinku_UltraSkill5_Damage", 728),
    ];
    const NORMAL_CHARGE: [u64; 10] = [760, 1285, 853, 1225, 724, 820, 1753, 3399, 2075, 4706];
    const TIDAL_CHARGE: [u64; 12] = [
        1349, 1066, 1573, 981, 302, 302, 302, 302, 302, 2069, 2654, 1796,
    ];
    const SKILL_CHARGE: [u64; 3] = [750, 1000, 3450];
    let case = &SHINKU_CASE;
    let mut req = continuing_request(case, "shinku");
    req.actors[0].energy_capacity = 60;
    let mut expected_hits = Vec::new();
    for _ in 0..6 {
        for (effect, rate_milli) in NORMAL {
            push_packets(
                &mut expected_hits,
                "GA_Shinku_Melee",
                &[(effect, rate_milli, 2)],
            );
        }
    }
    // Skill 1 at 24s
    for (effect, rate_milli) in SKILL {
        push_packets(
            &mut expected_hits,
            "GA_Shinku_Skill",
            &[(effect, rate_milli, 2)],
        );
    }
    // Skill 2 at 30s (after 6s idle -> 3 Menacing Gaze Watch ticks [27,28,29s] fire as FollowupHit)
    for (effect, rate_milli) in SKILL {
        push_packets(
            &mut expected_hits,
            "GA_Shinku_Skill",
            &[(effect, rate_milli, 2)],
        );
    }
    push_packets(
        &mut expected_hits,
        "GA_Shinku_Skill",
        &[("GE_Player_Shinku_Watch_Damage", 1714 * 3, 2)],
    );
    // Burst at 31.2s (+30% Surging Crimson + +50% Passive 2 non-boss = 9/5, over 1/2 DEF = 18/20)
    push_packets(
        &mut expected_hits,
        "GA_Shinku_UltraSkill",
        &BURST.map(|(effect, rate_milli)| (effect, rate_milli * 18, 20)),
    );
    // Post-burst Tidal Assault at 39.2s (8s elapsed minus 2s Burst pause = 6s world idle -> 3 Watch ticks)
    for (idx, (effect, rate_milli)) in TIDAL_ASSAULT.into_iter().enumerate() {
        push_packets(
            &mut expected_hits,
            "GA_Shinku_Melee",
            &[(effect, rate_milli * 13, 20)],
        );
        if idx == 1 {
            push_packets(
                &mut expected_hits,
                "GA_Shinku_Melee",
                &[("GE_Player_Shinku_Watch_Damage", 1714 * 3, 2)],
            );
        }
    }
    assert_eq!(expected_hits.len(), 89);

    let mut charge = Vec::new();
    for _ in 0..6 {
        charge.extend(NORMAL_CHARGE);
    }
    charge.extend(SKILL_CHARGE);
    charge.extend(SKILL_CHARGE);
    charge.push(3750);
    for (idx, val) in TIDAL_CHARGE.into_iter().enumerate() {
        charge.push(val);
        if idx == 1 {
            charge.push(3750);
        }
    }
    assert_eq!(charge.len(), 80);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let fanout_events: Vec<&Value> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["energy_fanout"]["source"] == case.source)
        .collect();
    assert_eq!(fanout_events.len(), charge.len());
    for (packet, base) in charge.iter().enumerate() {
        let fanout = &fanout_events[packet]["energy_fanout"];
        let recipients = fanout["recipients"].as_array().unwrap();
        assert_eq!(recipients.len(), 2);
        assert_eq!(fanout["base"], fraction(*base, 1000));
    }
    let shinku_grants: Vec<&Value> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| {
            event["event_kind"] == "shinku_ordinary_hit_energy_grant"
                && event["energy_change"].is_object()
        })
        .collect();
    assert_eq!(shinku_grants.len(), 67);
    // Total non-crit damage (/20):
    //   Pre-burst Normal (6*7238*10) + Skill (2*2056*10) + Watch#1 (5142*10)
    //   + Burst (10285*18) + Tidal (5770*13) + Watch#2 (5142*10)
    // = 434280 + 41120 + 51420 + 185130 + 75010 + 51420 = 838380 / 20 = 41919/1
    assert_hit_oracle(&report, &expected_hits, (41_919, 1));
    // Crimson Fury spends 60 Defiant Spirit directly from Shinku's 60-capacity Ultimate Energy bar.
    assert_eq!(report["final_actor_states"][0]["energy"], 0);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 121);
    assert_eq!(report["final_actor_states"][0]["overflow_energy"], 61);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 60);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        fraction(3, 4)
    );
    for recipient in 1..3 {
        assert_eq!(report["final_actor_states"][recipient]["energy"], 81);
        assert_eq!(
            report["final_actor_states"][recipient]["charge_remainder"],
            fraction(2247, 2500)
        );
    }
}

fn lacrimosa_timed_action(
    timestamp_us: u64,
    ability: &str,
    effect: &str,
    energy_cost: u64,
    recovery_us: u64,
    cooldown_us: u64,
) -> GameActionCommand {
    let mut command = mint_timed_action(
        timestamp_us,
        ability,
        effect,
        energy_cost,
        recovery_us,
        cooldown_us,
    );
    if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut command {
        *hit_delay_us = 0;
    }
    command
}

fn lacrimosa_rose_continuing_request(case: &Case) -> GameScenarioRequest {
    let mut req = request(case, false);
    req.actors[0].talent_levels = Some([8; 3]);
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.damage_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonuses.clear();
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actors[0].initial_energy = 0;
    req.actors[0].energy_capacity = 120;
    req.actors[0].weapon_id = "weapon:fork_Rose".into();
    req.actors[0].weapon_tier = 1;
    req.actors[0].gear_procs = vec![GearProcConfig::RoseThorn {
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
    }];
    req.targets[0].hp_max = 100_000_000;
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].defense = 1080;
    req.targets[0].resistance = Ratio::zero();
    req.relative_duration_us = 56_000_000;
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

    let (normal_ability, normal_effect, _) = case.melee;
    let (skill_ability, skill_effect, _) = case.skill.unwrap();
    let mut actions = Vec::new();
    for input in 0..30 {
        actions.push(lacrimosa_timed_action(
            input * 800_000,
            normal_ability,
            normal_effect,
            0,
            800_000,
            0,
        ));
    }
    actions.push(lacrimosa_timed_action(
        24_000_000,
        skill_ability,
        skill_effect,
        0,
        1_200_000,
        6_000_000,
    ));
    actions.push(lacrimosa_timed_action(
        25_200_000,
        "GA_Lacrimosa_UltraSkill",
        "GE_Player_Lacrimosa_UltraSkill1_Damage",
        120,
        2_000_000,
        20_000_000,
    ));
    for input in 0..5 {
        actions.push(lacrimosa_timed_action(
            27_200_000 + input * 800_000,
            normal_ability,
            normal_effect,
            0,
            800_000,
            0,
        ));
    }
    actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 31_200_000,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    req.actions = actions;
    req
}

#[test]
fn lacrimosa_rose_rotation_builds_energy_spends_burst_and_ticks_off_field() {
    const NORMAL: [(&str, u64, u64); 10] = [
        ("GE_Player_Lacrimosa_Melee1_Damage", 9063, 25),
        ("GE_Player_Lacrimosa_Melee2_Damage", 7809, 50),
        ("GE_Player_Lacrimosa_Melee2_Damage", 7809, 50),
        ("GE_Player_Lacrimosa_Melee2_Damage", 7809, 50),
        ("GE_Player_Lacrimosa_Melee3_Damage", 27816, 25),
        ("GE_Player_Lacrimosa_Melee4_Damage", 15447, 100),
        ("GE_Player_Lacrimosa_Melee4_Damage", 15447, 100),
        ("GE_Player_Lacrimosa_Melee4_Damage", 15447, 100),
        ("GE_Player_Lacrimosa_Melee6_Damage", 9462, 5),
        ("GE_Player_Lacrimosa_Melee5_Damage", 121011, 100),
    ];
    const BURST: [(&str, u64, u64); 5] = [
        ("GE_Player_Lacrimosa_UltraSkill1_Damage", 45999, 50),
        ("GE_Player_Lacrimosa_UltraSkill1_Damage", 45999, 50),
        ("GE_Player_Lacrimosa_UltraSkill1_Damage", 45999, 50),
        ("GE_Player_Lacrimosa_UltraSkill1_Damage", 45999, 50),
        ("GE_Player_Lacrimosa_UltraSkill2_Damage", 304437, 50),
    ];
    const NORMAL_CHARGE: [u64; 10] = [1428, 614, 614, 614, 4386, 607, 607, 607, 7454, 4767];
    let case = CASES
        .iter()
        .find(|case| case.owner == "character:1004")
        .unwrap();
    let req = lacrimosa_rose_continuing_request(case);

    let mut charge = Vec::new();
    for _ in 0..6 {
        charge.extend(NORMAL_CHARGE);
    }
    charge.push(6400);
    charge.extend(NORMAL_CHARGE);
    assert_eq!(charge.len(), 71);
    let report = check_flow_with_spend(case, &req, &charge, &[(1, 1); 3], Some((61, 120, 120)));

    let receipts = report["hit_receipts"].as_array().unwrap();
    assert_eq!(receipts.len(), 107);
    let direct = receipts
        .iter()
        .filter(|hit| hit["damage_effect_id"] != "GE_Player_Lacrimosa_Blood_Damage")
        .collect::<Vec<_>>();
    assert_eq!(direct.len(), 76);
    let mut expected_direct = Vec::new();
    for _ in 0..6 {
        push_packets(&mut expected_direct, "GA_Lacrimosa_Melee", &NORMAL);
    }
    push_packets(
        &mut expected_direct,
        "GA_Lacrimosa_Skill",
        &[("GE_Player_Lacrimosa_Skill1_Damage", 293037, 100)],
    );
    push_packets(&mut expected_direct, "GA_Lacrimosa_UltraSkill", &BURST);
    push_packets(&mut expected_direct, "GA_Lacrimosa_Melee", &NORMAL);
    for (hit, (ability, effect, numerator, denominator)) in direct.iter().zip(&expected_direct) {
        assert_eq!(hit["ability_id"], *ability);
        assert_eq!(hit["damage_effect_id"], *effect);
        assert_eq!(
            hit["non_crit_damage"],
            fraction(*numerator, *denominator),
            "{ability} / {effect}"
        );
    }
    assert_eq!(expected_direct.len(), 76);

    let ticks = receipts
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Lacrimosa_Blood_Damage")
        .collect::<Vec<_>>();
    assert_eq!(ticks.len(), 31);
    for (index, tick) in ticks.iter().enumerate() {
        let time_us =
            25_000_000 + index as u64 * 1_000_000 + if index == 0 { 0 } else { 2_000_000 };
        let stacks = if index == 0 {
            5
        } else if index <= 3 {
            10
        } else {
            9 - ((index - 4) / 3) as u64
        };
        assert_eq!(tick["actor_index"], 0);
        assert_eq!(tick["time_us"], time_us);
        assert_eq!(
            tick["non_crit_damage"],
            fraction(5871 * stacks, 100),
            "Nightmare tick at {}s with {stacks} live stacks",
            time_us / 1_000_000
        );
        assert!(
            crate::simulation::tests::support::hit_metadata(&report, tick)["packet"]
                ["semantic_tags"]
                .as_array()
                .unwrap()
                .iter()
                .any(|tag| tag == "State.Damage.Dot")
        );
    }
    assert_eq!(
        serde_json::json!(formula_damage_total(&report)),
        fraction(6_124_707, 100)
    );

    let costs = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| {
            event["energy_change"]["source"] == "ability_cost"
                && event["energy_change"]["spent"] == 120
        })
        .collect::<Vec<_>>();
    assert_eq!(costs.len(), 1);
    assert_eq!(costs[0]["time_us"], 25_200_000);
    assert_eq!(costs[0]["energy_change"]["energy_before"], 120);
    assert_eq!(costs[0]["energy_change"]["spent"], 120);
    assert_one_burst_spend(&report, 25_200_000, 120);
    assert_eq!(report["final_actor_states"][0]["energy"], 22);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 158);
    assert_eq!(report["final_actor_states"][0]["overflow_energy"], 16);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 120);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        fraction(143, 500)
    );
    for recipient in 1..3 {
        assert_eq!(report["final_actor_states"][recipient]["energy"], 94);
        assert_eq!(
            report["final_actor_states"][recipient]["charge_remainder"],
            fraction(2429, 2500)
        );
    }
    let all_buffs = crate::simulation::tests::support::final_buffs(&report);
    let live_thorns = all_buffs
        .iter()
        .filter(|buff| {
            buff["id"] == crate::weapons::rose::THORN_BUFF_ID
                && buff["owner_actor"] == 0
                && buff["duration"]["expires_at_us"] == 58_000_000
        })
        .collect::<Vec<_>>();
    assert_eq!(
        live_thorns
            .iter()
            .map(|buff| buff["stacks"].as_u64().unwrap())
            .sum::<u64>(),
        10
    );
}
