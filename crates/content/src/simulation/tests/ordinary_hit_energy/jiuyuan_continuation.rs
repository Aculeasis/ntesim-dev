use super::owner_rotations::{assert_hit_oracle, assert_one_burst_spend, push_packets};
use super::*;

fn jiuyuan_continuing_request(case: &Case) -> GameScenarioRequest {
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
    req.actors[0].energy_capacity = 100;
    req.targets[0].hp_max = 100_000_000;
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].defense = 1080;
    req.targets[0].resistance = Ratio::zero();
    req.relative_duration_us = 32_000_000;

    let mut actions = Vec::new();
    for input in 0..30 {
        actions.push(mint_timed_action(
            input * 800_000,
            case.melee.0,
            case.melee.1,
            0,
            800_000,
            0,
        ));
    }
    let (skill_ability, skill_effect, _) = case.skill.expect("Jiuyuan Skill");
    actions.push(mint_timed_action(
        24_000_000,
        skill_ability,
        skill_effect,
        0,
        1_200_000,
        6_000_000,
    ));
    actions.push(mint_timed_action(
        26_000_000,
        "GA_Kuhara_UltraSkill",
        "GE_Player_Kuhara_UltraSkill1_Damage",
        100,
        2_000_000,
        20_000_000,
    ));
    for input in 0..5 {
        actions.push(mint_timed_action(
            28_000_000 + input * 800_000,
            case.melee.0,
            case.melee.1,
            0,
            800_000,
            0,
        ));
    }
    req.actions = actions;
    req
}

#[test]
fn jiuyuan_zero_start_skill_burst_and_continuation_rotation() {
    const NORMAL: [(&str, u64, u64); 11] = [
        ("GE_Player_Kuhara_Melee1_Damage", 187, 1),
        ("GE_Player_Kuhara_Melee2_Damage", 413, 2),
        ("GE_Player_Kuhara_Melee2_1_Damage", 159, 2),
        ("GE_Player_Kuhara_Melee3_Damage", 287, 1),
        ("GE_Player_Kuhara_Melee3_1_Damage", 87, 2),
        ("GE_Player_Kuhara_Melee3_2_Damage", 222, 1),
        ("GE_Player_Kuhara_Melee4_Damage", 108, 1),
        ("GE_Player_Kuhara_Melee4_1_Damage", 210, 1),
        ("GE_Player_Kuhara_Melee4_2_Damage", 150, 1),
        ("GE_Player_Kuhara_Melee4_3_Damage", 175, 2),
        ("GE_Player_Kuhara_Melee5_Damage", 1303, 2),
    ];
    const SKILL: [(&str, u64, u64); 6] = [
        ("GE_Player_Kuhara_Skill1_Damage", 617, 2),
        ("GE_Player_Kuhara_Skill2_Damage", 655, 2),
        ("GE_Player_Kuhara_Skill2_Damage", 655, 2),
        ("GE_Player_Kuhara_Skill2_Damage", 655, 2),
        ("GE_Player_Kuhara_Skill2_Damage", 655, 2),
        ("GE_Player_Kuhara_Skill3_Damage", 1909, 2),
    ];
    const BURST: [(&str, u64, u64); 12] = [
        ("GE_Player_Kuhara_UltraSkill1_Damage", 1059, 2),
        ("GE_Player_Kuhara_UltraSkill2_Damage", 672, 1),
        ("GE_Player_Kuhara_UltraSkill2_Damage", 672, 1),
        ("GE_Player_Kuhara_UltraSkill2_Damage", 672, 1),
        ("GE_Player_Kuhara_UltraSkill3_Damage", 209, 1),
        ("GE_Player_Kuhara_UltraSkill3_Damage", 209, 1),
        ("GE_Player_Kuhara_UltraSkill3_Damage", 209, 1),
        ("GE_Player_Kuhara_UltraSkill3_Damage", 209, 1),
        ("GE_Player_Kuhara_UltraSkill3_Damage", 209, 1),
        ("GE_Player_Kuhara_UltraSkill3_Damage", 209, 1),
        ("GE_Player_Kuhara_UltraSkill3_Damage", 209, 1),
        ("GE_Player_Kuhara_UltraSkill4_Damage", 1132, 1),
    ];
    const NORMAL_CHARGE: [u64; 11] = [1365, 1511, 581, 2099, 320, 1619, 788, 1532, 1094, 636, 4754];
    const SKILL_CHARGE: [u64; 6] = [852, 903, 903, 903, 903, 2636];

    let case = CASES
        .iter()
        .find(|case| case.owner == "character:1055")
        .unwrap();
    let req = jiuyuan_continuing_request(case);

    let mut expected_hits = Vec::new();
    for _ in 0..6 {
        push_packets(&mut expected_hits, "GA_Kuhara_Melee", &NORMAL);
    }
    push_packets(&mut expected_hits, "GA_Kuhara_Skill", &SKILL);
    push_packets(&mut expected_hits, "GA_Kuhara_UltraSkill", &BURST);
    push_packets(&mut expected_hits, "GA_Kuhara_Melee", &NORMAL);
    assert_eq!(expected_hits.len(), 95);

    let mut charge = Vec::new();
    for _ in 0..6 {
        charge.extend(NORMAL_CHARGE);
    }
    charge.extend(SKILL_CHARGE);
    charge.extend(NORMAL_CHARGE);
    assert_eq!(charge.len(), 83);

    let report = check_flow_with_spend(case, &req, &charge, &[(1, 1); 3], Some((72, 100, 100)));
    assert_hit_oracle(&report, &expected_hits, (23_341, 1));
    assert_one_burst_spend(&report, 26_000_000, 100);

    assert_eq!(report["final_actor_states"][0]["energy"], 17);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 121);
    assert_eq!(report["final_actor_states"][0]["overflow_energy"], 4);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        fraction(193, 1000)
    );

    for recipient in 1..3 {
        assert_eq!(report["final_actor_states"][recipient]["energy"], 72);
        assert_eq!(
            report["final_actor_states"][recipient]["granted_energy"],
            72
        );
        assert_eq!(
            report["final_actor_states"][recipient]["overflow_energy"],
            0
        );
        assert_eq!(report["final_actor_states"][recipient]["spent_energy"], 0);
        assert_eq!(
            report["final_actor_states"][recipient]["charge_remainder"],
            fraction(3579, 5000)
        );
    }

    assert!(report["resolved_model_assumptions"]["jiuyuan.ordinary_hit_energy_v1"].is_string());
    assert!(report["resolved_model_assumptions"]["jiuyuan.normal_chain_v1"].is_string());
    assert!(report["resolved_model_assumptions"]["jiuyuan.skill_burst_packets_v1"].is_string());
}
