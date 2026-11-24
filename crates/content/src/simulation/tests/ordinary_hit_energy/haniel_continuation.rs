use super::owner_rotations::{assert_hit_oracle, push_packets};
use super::*;

fn haniel_continuing_request(case: &Case) -> GameScenarioRequest {
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
    req.targets[0].hp_max = 100_000_000;
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].defense = 1080;
    req.targets[0].resistance = Ratio::zero();
    req.relative_duration_us = 40_000_000;
    let hootie = crate::characters::haniel::hootie::HYPOTHESIS;
    req.model_assumptions
        .insert(hootie.into(), hootie.description().into());

    let mut actions = Vec::new();
    for input in 0..40 {
        actions.push(mint_timed_action(
            input * 800_000,
            case.melee.0,
            case.melee.1,
            0,
            800_000,
            0,
        ));
    }
    let (skill_ability, skill_effect, _) = case.skill.expect("Haniel Skill");
    actions.push(mint_timed_action(
        32_000_000,
        skill_ability,
        skill_effect,
        0,
        1_200_000,
        6_000_000,
    ));
    actions.push(mint_timed_action(
        34_000_000,
        "GA_Haniel_UltraSkill",
        "GE_Player_Haniel_Ultraskill1_Damage",
        100,
        2_000_000,
        20_000_000,
    ));
    for input in 0..5 {
        actions.push(mint_timed_action(
            36_000_000 + input * 800_000,
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
fn haniel_zero_start_skill_burst_and_continuation_rotation() {
    const NORMAL: [(&str, u64, u64); 9] = [
        ("GE_Player_Haniel_Melee1_Damage", 359, 1),
        ("GE_Player_Haniel_Melee2_Damage", 96, 1),
        ("GE_Player_Haniel_Melee2_Damage", 96, 1),
        ("GE_Player_Haniel_Melee3_Damage", 997, 2),
        ("GE_Player_Haniel_Melee4_Damage", 267, 2),
        ("GE_Player_Haniel_Melee4_Damage", 267, 2),
        ("GE_Player_Haniel_Melee4_Damage", 267, 2),
        ("GE_Player_Haniel_Melee4_Damage", 267, 2),
        ("GE_Player_Haniel_Melee5_Damage", 905, 2),
    ];
    const SKILL: [(&str, u64, u64); 2] = [
        ("GE_Player_Haniel_Skill1_Damage", 857, 1),
        ("GE_Player_Haniel_Skill1_Damage", 857, 1),
    ];
    const BURST: [(&str, u64, u64); 1] = [("GE_Player_Haniel_Ultraskill1_Damage", 344_447, 100)];
    const CONTINUING_NORMAL: [(&str, u64, u64); 9] = [
        ("GE_Player_Haniel_Melee1_Damage", 24_053, 50),
        ("GE_Player_Haniel_Melee2_Damage", 3_216, 25),
        ("GE_Player_Haniel_Melee2_Damage", 3_216, 25),
        ("GE_Player_Haniel_Melee3_Damage", 66_799, 100),
        ("GE_Player_Haniel_Melee4_Damage", 17_889, 100),
        ("GE_Player_Haniel_Melee4_Damage", 17_889, 100),
        ("GE_Player_Haniel_Melee4_Damage", 17_889, 100),
        ("GE_Player_Haniel_Melee4_Damage", 17_889, 100),
        ("GE_Player_Haniel_Melee5_Damage", 12_127, 20),
    ];
    const NORMAL_CHARGE: [u64; 9] = [2625, 701, 701, 3648, 979, 979, 979, 979, 3311];
    const SKILL_CHARGE: [u64; 2] = [1800, 1800];

    let case = CASES
        .iter()
        .find(|case| case.owner == "character:1020")
        .unwrap();
    let req = haniel_continuing_request(case);

    let mut expected_hits = Vec::new();
    for _ in 0..8 {
        push_packets(&mut expected_hits, "GA_Haniel_Melee", &NORMAL);
    }
    push_packets(&mut expected_hits, "GA_Haniel_Skill", &SKILL);
    push_packets(&mut expected_hits, "GA_Haniel_UltraSkill", &BURST);
    push_packets(
        &mut expected_hits,
        "GA_Haniel_Melee",
        &CONTINUING_NORMAL[..8],
    );
    push_packets(
        &mut expected_hits,
        "GA_Haniel_Skill",
        &[("GE_Player_Haniel_Skill2_Damage", 22981, 100)],
    );
    push_packets(
        &mut expected_hits,
        "GA_Haniel_UltraSkill",
        &[("GE_Player_Haniel_Ultraskill2_Damage", 17219, 50)],
    );
    push_packets(
        &mut expected_hits,
        "GA_Haniel_Melee",
        &CONTINUING_NORMAL[8..],
    );
    push_packets(
        &mut expected_hits,
        "GA_Haniel_UltraSkill",
        &[("GE_Player_Haniel_UltraSkill_Extra_Damage", 17219, 50)],
    );

    assert_eq!(expected_hits.len(), 87);

    let mut charge = Vec::new();
    for _ in 0..8 {
        charge.extend(NORMAL_CHARGE);
    }
    charge.extend(SKILL_CHARGE);
    charge.extend(NORMAL_CHARGE);
    assert_eq!(charge.len(), 83);

    let report = check_flow_with_spend(case, &req, &charge, &[(1, 1); 3], Some((74, 120, 100)));
    assert_hit_oracle(&report, &expected_hits, (627332, 25));

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
    assert_eq!(costs[0]["time_us"], 34_000_000);
    assert_eq!(costs[0]["energy_change"]["energy_before"], 120);
    assert_eq!(costs[0]["energy_change"]["spent"], 100);

    assert_eq!(report["final_actor_states"][0]["energy"], 35);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 137);
    assert_eq!(report["final_actor_states"][0]["overflow_energy"], 2);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        fraction(359, 500)
    );

    for recipient in 1..3 {
        assert_eq!(report["final_actor_states"][recipient]["energy"], 82);
        assert_eq!(
            report["final_actor_states"][recipient]["granted_energy"],
            82
        );
        assert_eq!(
            report["final_actor_states"][recipient]["overflow_energy"],
            0
        );
        assert_eq!(report["final_actor_states"][recipient]["spent_energy"], 0);
        assert_eq!(
            report["final_actor_states"][recipient]["charge_remainder"],
            fraction(1577, 2500)
        );
    }

    assert!(report["resolved_model_assumptions"]["haniel.ordinary_hit_energy_v1"].is_string());
    assert!(report["resolved_model_assumptions"]["haniel.normal_chain_v1"].is_string());
    assert!(report["resolved_model_assumptions"]["haniel.skill_burst_packets_v1"].is_string());
    assert!(report["resolved_model_assumptions"]["haniel.harmony_ensemble_v1"].is_string());
}
