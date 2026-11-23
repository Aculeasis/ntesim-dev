use super::owner_rotations::{assert_hit_oracle, push_packets};
use super::*;

fn fadia_continuing_request(case: &Case) -> GameScenarioRequest {
    let mut req = request(case, false);
    req.actors[0].talent_levels = Some([8; 3]);
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.hp = 10_000;
    req.actors[0].initial_hp = 10_000;
    req.actors[0].initial_hp_max = 10_000;
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
    req.relative_duration_us = 35_000_000;

    req.model_assumptions.insert(
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::fadia::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::fadia::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::fadia::skill::BURST_HYPOTHESIS_ID.into(),
        crate::characters::fadia::skill::BURST_HYPOTHESIS_ID
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
    let mut burst_action = mint_timed_action(
        24_000_000,
        "GA_Fadia_UltraSkill",
        "GE_Player_Fadia_UltraSkill1_Damage",
        120,
        2_000_000,
        20_000_000,
    );
    if let GameActionCommand::InvokeAbility {
        array_index,
        atk_rate,
        scaling_stat,
        ..
    } = &mut burst_action
    {
        *array_index = 7;
        *atk_rate = Ratio {
            numerator: 67,
            denominator: 1000,
        };
        *scaling_stat = NativeScalingStat::Hp;
    }
    actions.push(burst_action);
    for input in 0..5 {
        actions.push(mint_timed_action(
            34_417_366 + input * 800_000,
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
fn fadia_zero_start_burst_and_continuation_rotation() {
    const NORMAL: [(&str, u64, u64); 8] = [
        ("GE_Player_Fadia_Melee1_Damage", 371, 1),
        ("GE_Player_Fadia_Melee2_Damage", 509, 2),
        ("GE_Player_Fadia_Melee3_Damage", 617, 2),
        ("GE_Player_Fadia_Melee4_Damage", 1321, 2),
        ("GE_Player_Fadia_Melee5_Damage", 485, 2),
        ("GE_Player_Fadia_Melee6_Damage", 569, 2),
        ("GE_Player_Fadia_Melee7_Damage", 234, 1),
        ("GE_Player_Fadia_Melee8_Damage", 688, 1),
    ];
    const BURST: [(&str, u64, u64); 6] = [
        ("GE_Player_Fadia_UltraSkill1_Damage", 670, 1),
        ("GE_Player_Fadia_UltraSkill2_Damage", 210, 1),
        ("GE_Player_Fadia_UltraSkill2_Damage", 210, 1),
        ("GE_Player_Fadia_UltraSkill2_Damage", 210, 1),
        ("GE_Player_Fadia_UltraSkill2_Damage", 210, 1),
        ("GE_Player_Fadia_UltraSkill3_Damage", 1080, 1),
    ];
    const NORMAL_CHARGE: [u64; 8] = [2707, 1859, 2247, 4819, 1768, 2075, 1703, 5022];

    let case = CASES
        .iter()
        .find(|case| case.owner == "character:1039")
        .unwrap();
    let req = fadia_continuing_request(case);

    let mut expected_hits = Vec::new();
    for _ in 0..6 {
        push_packets(&mut expected_hits, "GA_Fadia_Melee", &NORMAL);
    }
    push_packets(&mut expected_hits, "GA_Fadia_UltraSkill", &BURST);
    push_packets(&mut expected_hits, "GA_Fadia_Melee", &NORMAL);
    assert_eq!(expected_hits.len(), 62);

    let mut charge = Vec::new();
    for _ in 0..6 {
        charge.extend(NORMAL_CHARGE);
    }
    charge.extend(NORMAL_CHARGE);
    assert_eq!(charge.len(), 56);

    let report = check_flow_with_spend(case, &req, &charge, &[(1, 1); 3], Some((48, 120, 120)));
    assert_hit_oracle(&report, &expected_hits, (47_789, 2));

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
    assert_eq!(costs[0]["time_us"], 24_000_000);
    assert_eq!(costs[0]["energy_change"]["energy_before"], 120);
    assert_eq!(costs[0]["energy_change"]["spent"], 120);

    assert_eq!(report["final_actor_states"][0]["energy"], 22);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 155);
    assert_eq!(report["final_actor_states"][0]["overflow_energy"], 13);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 120);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        fraction(400, 1000)
    );

    for recipient in 1..3 {
        assert_eq!(report["final_actor_states"][recipient]["energy"], 93);
        assert_eq!(
            report["final_actor_states"][recipient]["granted_energy"],
            93
        );
        assert_eq!(
            report["final_actor_states"][recipient]["overflow_energy"],
            0
        );
        assert_eq!(report["final_actor_states"][recipient]["spent_energy"], 0);
        assert_eq!(
            report["final_actor_states"][recipient]["charge_remainder"],
            fraction(1200, 5000)
        );
    }

    assert!(report["resolved_model_assumptions"]["fadia.ordinary_hit_energy_v1"].is_string());
    assert!(report["resolved_model_assumptions"]["fadia.normal_chain_v1"].is_string());
    assert!(report["resolved_model_assumptions"]["fadia.full_hp_initial_burst_v1"].is_string());
}
