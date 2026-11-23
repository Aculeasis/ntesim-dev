use super::owner_rotations::{assert_hit_oracle, assert_one_burst_spend, push_packets};
use super::*;

fn zero_continuing_request(case: &Case) -> GameScenarioRequest {
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
    req.relative_duration_us = 28_000_000;

    let mut actions = Vec::new();
    for input in 0..25 {
        actions.push(mint_timed_action(
            input * 800_000,
            case.melee.0,
            case.melee.1,
            0,
            800_000,
            0,
        ));
    }
    let (skill_ability, skill_effect, _) = case.skill.expect("Zero Skill");
    actions.push(mint_timed_action(
        20_000_000,
        skill_ability,
        skill_effect,
        0,
        1_200_000,
        6_000_000,
    ));
    actions.push(mint_timed_action(
        22_000_000,
        "GA_Female046_UltraSkill",
        "GE_Player_Female046_UltraSkill1_Damage",
        100,
        2_000_000,
        20_000_000,
    ));
    for input in 0..5 {
        actions.push(mint_timed_action(
            24_000_000 + input * 800_000,
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
fn zero_zero_start_skill_burst_and_continuation_rotation() {
    const NORMAL: [(&str, u64, u64); 10] = [
        ("GE_Player_Female046_Melee1_Damage", 197, 1),
        ("GE_Player_Female046_Melee2_1_Damage", 192, 1),
        ("GE_Player_Female046_Melee2_Damage", 222, 1),
        ("GE_Player_Female046_Melee3_Damage", 325, 1),
        ("GE_Player_Female046_Melee4_Damage", 55, 1),
        ("GE_Player_Female046_Melee5_Damage", 330, 1),
        ("GE_Player_Female046_Melee6_Damage", 132, 1),
        ("GE_Player_Female046_Melee7_Damage", 1709, 2),
        ("GE_Player_Female046_Melee8_Damage", 1171, 2),
        ("GE_Player_Female046_Melee8_Damage", 1171, 2),
    ];
    const SKILL: [(&str, u64, u64); 4] = [
        ("GE_Player_Female046_Skill1_Damage", 343, 2),
        ("GE_Player_Female046_Skill2_Damage", 343, 2),
        ("GE_Player_Female046_Skill3_Damage", 2153, 2),
        ("GE_Player_Female046_Skill4_Damage", 2303, 2),
    ];
    const BURST: [(&str, u64, u64); 7] = [
        ("GE_Player_Female046_UltraSkill1_Damage", 1137, 1),
        ("GE_Player_Female046_UltraSkill2_Damage", 91, 2),
        ("GE_Player_Female046_UltraSkill2_Damage", 91, 2),
        ("GE_Player_Female046_UltraSkill2_Damage", 91, 2),
        ("GE_Player_Female046_UltraSkill2_Damage", 91, 2),
        ("GE_Player_Female046_UltraSkill3_Damage", 1521, 1),
        ("GE_Player_Female046_UltraSkill4_Damage", 1445, 1),
    ];
    const NORMAL_CHARGE: [u64; 10] = [1151, 1122, 1297, 1900, 319, 1927, 771, 4993, 3420, 3420];
    const SKILL_CHARGE: [u64; 4] = [587, 587, 3683, 3944];

    let case = CASES
        .iter()
        .find(|case| case.owner == "character:1046")
        .unwrap();
    let req = zero_continuing_request(case);

    let mut expected_hits = Vec::new();
    for _ in 0..5 {
        push_packets(&mut expected_hits, "GA_Female046_Melee", &NORMAL);
    }
    push_packets(&mut expected_hits, "GA_Female046_Skill", &SKILL);
    push_packets(&mut expected_hits, "GA_Female046_UltraSkill", &BURST);
    push_packets(&mut expected_hits, "GA_Female046_Melee", &NORMAL);
    assert_eq!(expected_hits.len(), 71);

    let mut charge = Vec::new();
    for _ in 0..5 {
        charge.extend(NORMAL_CHARGE);
    }
    charge.extend(SKILL_CHARGE);
    charge.extend(NORMAL_CHARGE);
    assert_eq!(charge.len(), 64);

    let report = check_flow_with_spend(case, &req, &charge, &[(1, 1); 3], Some((54, 100, 100)));
    assert_hit_oracle(&report, &expected_hits, (27_727, 1));
    assert_one_burst_spend(&report, 22_000_000, 100);

    assert_eq!(report["final_actor_states"][0]["energy"], 20);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 130);
    assert_eq!(report["final_actor_states"][0]["overflow_energy"], 10);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        fraction(721, 1000)
    );

    for recipient in 1..3 {
        assert_eq!(report["final_actor_states"][recipient]["energy"], 78);
        assert_eq!(
            report["final_actor_states"][recipient]["granted_energy"],
            78
        );
        assert_eq!(
            report["final_actor_states"][recipient]["overflow_energy"],
            0
        );
        assert_eq!(report["final_actor_states"][recipient]["spent_energy"], 0);
        assert_eq!(
            report["final_actor_states"][recipient]["charge_remainder"],
            fraction(2163, 5000)
        );
    }

    assert!(report["resolved_model_assumptions"]["zero.ordinary_hit_energy_v1"].is_string());
    assert!(report["resolved_model_assumptions"]["zero.normal_chain_v1"].is_string());
    assert!(report["resolved_model_assumptions"]["zero.skill_packets_v1"].is_string());
    assert!(report["resolved_model_assumptions"]["zero.burst_packets_v1"].is_string());
    assert!(report["resolved_model_assumptions"]["zero.burst_all_in_range_v1"].is_string());
}
