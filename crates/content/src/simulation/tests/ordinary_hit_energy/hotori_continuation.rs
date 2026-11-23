use super::owner_rotations::{assert_hit_oracle, push_packets};
use super::*;

fn hotori_continuing_request(case: &Case) -> GameScenarioRequest {
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
    req.relative_duration_us = 72_000_000;

    req.model_assumptions.insert(
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::hotori::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::hotori::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::hotori::skill::PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::hotori::skill::PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );

    let mut actions = Vec::new();
    for input in 0..20 {
        actions.push(mint_timed_action(
            input * 800_000,
            case.melee.0,
            case.melee.1,
            0,
            800_000,
            0,
        ));
    }
    let (skill_ability, skill_effect, _) = case.skill.expect("Hotori Skill");
    actions.push(mint_timed_action(
        16_000_000,
        skill_ability,
        skill_effect,
        0,
        1_200_000,
        6_000_000,
    ));
    // The second replay needs the passive to refill past 60 after the first
    // 5.167475s rewind, so it lands at 26s with 64.99515.
    actions.push(mint_timed_action(
        26_000_000,
        skill_ability,
        skill_effect,
        0,
        1_200_000,
        6_000_000,
    ));
    // World's Tide needs exactly 120 Timepiece (clamped from 123.9903 at
    // 51s) and spends no conventional energy.
    actions.push(mint_timed_action(
        51_000_000,
        "GA_Jin_UltraSkill",
        "GE_Player_Jin_UltraSkill1_Damage",
        0,
        2_000_000,
        20_000_000,
    ));
    // The 14.183333s mode ends at 65.183333s, after the normal chain resumes.
    for input in 0..5 {
        actions.push(mint_timed_action(
            66_000_000 + input * 800_000,
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
fn hotori_zero_start_skill_burst_and_continuation_rotation() {
    const NORMAL: [(&str, u64, u64); 15] = [
        ("GE_Player_Jin_Melee1_Damage", 343, 2),
        ("GE_Player_Jin_Melee2_Damage", 605, 2),
        ("GE_Player_Jin_Melee3_Damage", 75, 2),
        ("GE_Player_Jin_Melee3_Damage", 75, 2),
        ("GE_Player_Jin_Melee9_Damage", 1237, 2),
        ("GE_Player_Jin_Melee4_Damage", 97, 1),
        ("GE_Player_Jin_Melee4_Damage", 97, 1),
        ("GE_Player_Jin_Melee4_Damage", 97, 1),
        ("GE_Player_Jin_Melee4_Damage", 97, 1),
        ("GE_Player_Jin_Melee4_Damage", 97, 1),
        ("GE_Player_Jin_Melee5_Damage", 243, 2),
        ("GE_Player_Jin_Melee5_Damage", 243, 2),
        ("GE_Player_Jin_Melee6_Damage", 1141, 2),
        ("GE_Player_Jin_Melee7_Damage", 208, 1),
        ("GE_Player_Jin_Melee8_Damage", 773, 2),
    ];
    const SKILL: [(&str, u64, u64); 7] = [
        ("GE_Player_Jin_Skill3_Damage", 343, 1),
        ("GE_Player_Jin_Skill1_Damage", 343, 1),
        ("GE_Player_Jin_Skill3_Damage", 343, 1),
        ("GE_Player_Jin_Skill1_Damage", 343, 1),
        ("GE_Player_Jin_Skill1_Damage", 343, 1),
        ("GE_Player_Jin_Skill1_Damage", 343, 1),
        ("GE_Player_Jin_Skill2_Damage", 515, 1),
    ];
    const BURST: [(&str, u64, u64); 10] = [
        ("GE_Player_Jin_UltraSkill1_Damage", 291, 2),
        ("GE_Player_Jin_UltraSkill1_1_Damage", 291, 2),
        ("GE_Player_Jin_UltraSkill1_2_Damage", 291, 2),
        ("GE_Player_Jin_UltraSkill1_3_Damage", 291, 2),
        ("GE_Player_Jin_UltraSkill1_4_Damage", 291, 2),
        ("GE_Player_Jin_UltraSkill1_5_Damage", 291, 2),
        ("GE_Player_Jin_UltraSkill2_Damage", 2233, 2),
        ("GE_Player_Jin_UltraSkill4_Damage", 759, 2),
        ("GE_Player_Jin_UltraSkill3_Damage", 4483, 2),
        ("GE_Player_Jin_UltraSkill3_Damage", 4483, 2),
    ];
    const NORMAL_CHARGE: [u64; 15] = [
        1251, 2206, 274, 274, 4508, 703, 703, 703, 703, 703, 889, 889, 4160, 1519, 2814,
    ];
    const SKILL_CHARGE: [u64; 7] = [946, 947, 946, 947, 947, 947, 1421];

    let case = CASES
        .iter()
        .find(|case| case.owner == "character:1052")
        .unwrap();
    let req = hotori_continuing_request(case);

    let mut expected_hits = Vec::new();
    for _ in 0..4 {
        push_packets(&mut expected_hits, "GA_Jin_Melee", &NORMAL);
    }
    push_packets(&mut expected_hits, "GA_Jin_Skill", &SKILL);
    push_packets(&mut expected_hits, "GA_Jin_Skill", &SKILL);
    push_packets(&mut expected_hits, "GA_Jin_UltraSkill", &BURST);
    push_packets(&mut expected_hits, "GA_Jin_Melee", &NORMAL);
    assert_eq!(expected_hits.len(), 99);

    let mut charge = Vec::new();
    for _ in 0..4 {
        charge.extend(NORMAL_CHARGE);
    }
    charge.extend(SKILL_CHARGE);
    charge.extend(SKILL_CHARGE);
    charge.extend(NORMAL_CHARGE);
    assert_eq!(charge.len(), 89);

    let report = check_flow_with_spend(case, &req, &charge, &[(1, 1); 3], None);
    assert_eq!(report["hits_settled"], 99);
    assert_hit_oracle(&report, &expected_hits, (54_601, 2));

    // Burst packets carry no ordinary-hit charge and World's Tide spends
    // Timepiece rather than conventional energy.
    assert_eq!(report["final_actor_states"][0]["energy"], 100);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 125);
    assert_eq!(report["final_actor_states"][0]["overflow_energy"], 25);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 0);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        fraction(697, 1000)
    );

    for recipient in 1..3 {
        assert_eq!(report["final_actor_states"][recipient]["energy"], 75);
        assert_eq!(
            report["final_actor_states"][recipient]["granted_energy"],
            75
        );
        assert_eq!(
            report["final_actor_states"][recipient]["overflow_energy"],
            0
        );
        assert_eq!(report["final_actor_states"][recipient]["spent_energy"], 0);
        assert_eq!(
            report["final_actor_states"][recipient]["charge_remainder"],
            fraction(2091, 5000)
        );
    }

    assert!(report["resolved_model_assumptions"]["hotori.ordinary_hit_energy_v1"].is_string());
    assert!(report["resolved_model_assumptions"]["hotori.normal_chain_v1"].is_string());
    assert!(report["resolved_model_assumptions"]["hotori.skill_burst_packets_v1"].is_string());
}
