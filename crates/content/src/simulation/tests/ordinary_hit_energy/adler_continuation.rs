use super::owner_rotations::{assert_hit_oracle, push_packets};
use super::*;

fn adler_continuing_request(case: &Case) -> GameScenarioRequest {
    let mut req = request(case, false);
    req.actors[0].talent_levels = Some([8; 3]);
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def = 1000;
    req.actors[0].static_stats.def_percent = Ratio::zero();
    req.actors[0].static_stats.def_flat = Ratio::zero();
    req.actors[0].static_stats.damage_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonuses.clear();
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actors[0].initial_energy = 0;
    req.actors[0].energy_capacity = 120;
    req.targets[0].hp_max = 100_000_000;
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].defense = 1080;
    req.targets[0].resistance = Ratio::zero();
    req.relative_duration_us = 36_000_000;

    req.model_assumptions.insert(
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::adler::skill::SKILL_TRAIN_HYPOTHESIS_ID.into(),
        crate::characters::adler::skill::SKILL_TRAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::adler::skill::SKILL_ALL_IN_RANGE_HYPOTHESIS_ID.into(),
        crate::characters::adler::skill::SKILL_ALL_IN_RANGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::adler::skill::KARMA_HYPOTHESIS_ID.into(),
        crate::characters::adler::skill::KARMA_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::adler::skill::BURST_HYPOTHESIS_ID.into(),
        crate::characters::adler::skill::BURST_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::packet_semantics::ADLER_SKILL_DOT_PACKET_HYPOTHESIS_ID.into(),
        crate::characters::packet_semantics::ADLER_SKILL_DOT_PACKET_HYPOTHESIS_ID
            .description()
            .into(),
    );

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
    let (skill_ability, skill_effect, _) = case.skill.expect("Adler Skill");
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
        "GA_Adler_UltraSkill",
        "GE_Player_Adler_UltraSkill_Damage",
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
fn adler_zero_start_skill_burst_and_continuation_rotation() {
    const NORMAL: [(&str, u64, u64); 12] = [
        ("GE_Player_Adler_Melee1_Damage", 172, 1),
        ("GE_Player_Adler_Melee2_Damage", 989, 2),
        ("GE_Player_Adler_Melee3_Damage", 132, 1),
        ("GE_Player_Adler_Melee3_Damage", 132, 1),
        ("GE_Player_Adler_Melee3_Damage", 132, 1),
        ("GE_Player_Adler_Melee3_Damage", 132, 1),
        ("GE_Player_Adler_Melee3_Damage", 132, 1),
        ("GE_Player_Adler_Melee3_Damage", 132, 1),
        ("GE_Player_Adler_Melee3_Damage", 132, 1),
        ("GE_Player_Adler_Melee4_1_Damage", 375, 2),
        ("GE_Player_Adler_Melee4_2_Damage", 198, 1),
        ("GE_Player_Adler_Melee5_Damage", 750, 1),
    ];
    const SKILL_INITIAL: [(&str, u64, u64); 3] = [
        ("GE_Player_Adler_Skill2_Damage", 840, 1),
        ("GE_Player_Adler_Skill3_Damage", 840, 1),
        ("GE_Player_Adler_Skill3_Damage", 840, 1),
    ];
    const SKILL_TICK: [(&str, u64, u64); 10] = [
        ("GE_Player_Adler_Skill_Damage", 2401, 10),
        ("GE_Player_Adler_Skill_Damage", 2401, 10),
        ("GE_Player_Adler_Skill_Damage", 2401, 10),
        ("GE_Player_Adler_Skill_Damage", 2401, 10),
        ("GE_Player_Adler_Skill_Damage", 2401, 10),
        ("GE_Player_Adler_Skill_Damage", 2401, 10),
        ("GE_Player_Adler_Skill_Damage", 2401, 10),
        ("GE_Player_Adler_Skill_Damage", 2401, 10),
        ("GE_Player_Adler_Skill_Damage", 2401, 10),
        ("GE_Player_Adler_Skill_Damage", 2401, 10),
    ];
    const BURST: [(&str, u64, u64); 10] = [
        ("GE_Player_Adler_UltraSkill_Damage", 857, 1),
        ("GE_Player_Adler_UltraSkill_Damage", 857, 1),
        ("GE_Player_Adler_UltraSkill_Damage", 857, 1),
        ("GE_Player_Adler_UltraSkill_Damage", 857, 1),
        ("GE_Player_Adler_UltraSkill_Damage", 857, 1),
        ("GE_Player_Adler_UltraSkill_Damage", 857, 1),
        ("GE_Player_Adler_UltraSkill_Damage", 857, 1),
        ("GE_Player_Adler_UltraSkill_Damage", 857, 1),
        ("GE_Player_Adler_UltraSkill_Damage", 857, 1),
        ("GE_Player_Adler_UltraSkill_Damage", 857, 1),
    ];
    const NORMAL_CHARGE: [u64; 12] = [
        1254, 3599, 963, 963, 963, 963, 963, 963, 963, 1364, 1440, 5458,
    ];
    const SKILL_CHARGE: [u64; 1] = [4000];

    let case = CASES
        .iter()
        .find(|case| case.owner == "character:1033")
        .unwrap();
    let req = adler_continuing_request(case);

    let mut expected_hits = Vec::new();
    for _ in 0..6 {
        push_packets(&mut expected_hits, "GA_Adler_Melee", &NORMAL);
    }
    push_packets(&mut expected_hits, "GA_Adler_Skill", &SKILL_INITIAL);
    push_packets(&mut expected_hits, "GA_Adler_Skill", &SKILL_TICK[0..1]);
    push_packets(&mut expected_hits, "GA_Adler_UltraSkill", &BURST);
    push_packets(&mut expected_hits, "GA_Adler_Melee", &NORMAL[0..1]);
    push_packets(&mut expected_hits, "GA_Adler_Skill", &SKILL_TICK[1..2]);
    push_packets(&mut expected_hits, "GA_Adler_Melee", &NORMAL[1..2]);
    push_packets(&mut expected_hits, "GA_Adler_Skill", &SKILL_TICK[2..3]);
    push_packets(&mut expected_hits, "GA_Adler_Melee", &NORMAL[2..9]);
    push_packets(&mut expected_hits, "GA_Adler_Skill", &SKILL_TICK[3..4]);
    push_packets(&mut expected_hits, "GA_Adler_Melee", &NORMAL[9..11]);
    push_packets(&mut expected_hits, "GA_Adler_Skill", &SKILL_TICK[4..5]);
    push_packets(&mut expected_hits, "GA_Adler_Melee", &NORMAL[11..12]);
    push_packets(&mut expected_hits, "GA_Adler_Skill", &SKILL_TICK[5..10]);
    assert_eq!(expected_hits.len(), 107);

    let mut charge = Vec::new();
    for _ in 0..6 {
        charge.extend(NORMAL_CHARGE);
    }
    charge.extend(SKILL_CHARGE);
    charge.extend(NORMAL_CHARGE);
    assert_eq!(charge.len(), 85);

    let report = check_flow_with_spend(case, &req, &charge, &[(1, 1); 3], Some((73, 120, 100)));
    assert_eq!(report["hits_settled"], 107);
    assert_hit_oracle(&report, &expected_hits, (32_573, 1));

    let costs = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| {
            event["energy_change"]["source"] == "ability_cost"
                && event["energy_change"]["spent"] == 100
        })
        .collect::<Vec<_>>();
    assert_eq!(costs.len(), 1);
    assert_eq!(costs[0]["time_us"], 26_000_000);
    assert_eq!(costs[0]["energy_change"]["energy_before"], 120);
    assert_eq!(costs[0]["energy_change"]["spent"], 100);

    assert_eq!(report["final_actor_states"][0]["energy"], 39);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 142);
    assert_eq!(report["final_actor_states"][0]["overflow_energy"], 3);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        fraction(124, 125)
    );

    for recipient in 1..3 {
        assert_eq!(report["final_actor_states"][recipient]["energy"], 85);
        assert_eq!(
            report["final_actor_states"][recipient]["granted_energy"],
            85
        );
        assert_eq!(
            report["final_actor_states"][recipient]["overflow_energy"],
            0
        );
        assert_eq!(report["final_actor_states"][recipient]["spent_energy"], 0);
        assert_eq!(
            report["final_actor_states"][recipient]["charge_remainder"],
            fraction(497, 625)
        );
    }

    assert!(report["resolved_model_assumptions"]["adler.ordinary_hit_energy_v1"].is_string());
    assert!(report["resolved_model_assumptions"]["adler.normal_chain_v1"].is_string());
    assert!(report["resolved_model_assumptions"]["adler.skill_train_v1"].is_string());
    assert!(report["resolved_model_assumptions"]["adler.burst_sunyas_v1"].is_string());
}
