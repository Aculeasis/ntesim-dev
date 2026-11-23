use super::*;

#[test]
fn edgar_a1_a4_a6_and_resonance_6() {
    let limits = ExecutionLimits::default();
    let mut req = research_request();
    req.stochastic_crit = None;

    req.actors[0].character_id = "character:1021".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actors[0].static_stats.base_hp = Some(13_000);
    req.actors[0].static_stats.hp_percent = Ratio {
        numerator: 1,
        denominator: 5,
    };
    req.actors[0].static_stats.hp = 15_600;
    req.actors[0].initial_hp = 15_600;
    req.actors[0].initial_hp_max = 15_600;
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].initial_energy = 100;
    req.actors[0].energy_capacity = 100;
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();

    req.targets[0].defense = 1080;
    req.targets[0].resistance = Ratio::zero();

    req.action_metadata.clear();
    req.actions = vec![
        GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Edgar_Melee".into(),
            damage_effect_id: "GE_Player_Edgar_Melee1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 0,
            hit_delay_us: 100_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 1_000_000,
            actor_index: 0,
            ability_id: "GA_Edgar_Skill".into(),
            damage_effect_id: "GE_Player_Edgar_Skill_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 1_000_000,
            hit_delay_us: 100_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 2_000_000,
            actor_index: 0,
            ability_id: "GA_Edgar_UltraSkill".into(),
            damage_effect_id: "GE_Player_Edgar_UltraSkill_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 100,
            recovery_us: 500_000,
            cooldown_us: 20_000_000,
            hit_delay_us: 100_000,
        },
        // 3. Skill at t = 3.0s: in Finnegan's Wake.
        GameActionCommand::InvokeAbility {
            timestamp_us: 3_000_000,
            actor_index: 0,
            ability_id: "GA_Edgar_Skill".into(),
            damage_effect_id: "GE_Player_Edgar_Skill_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 0,
            hit_delay_us: 100_000,
        },
    ];
    req.relative_duration_us = 13_000_000;

    let report_a6 = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report_a6["execution_completed"], true);
    assert_eq!(
        report_a6["resolved_model_assumptions"]
            [crate::characters::edgar::skill::A6_DEF_HYPOTHESIS_ID.as_str()],
        crate::characters::edgar::skill::A6_DEF_HYPOTHESIS_ID.description()
    );
    let damages_a6: Vec<u64> = report_a6["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|hit| hit["rounded_damage"].as_u64().unwrap())
        .collect();
    let expected_a6 = std::iter::once(500)
        .chain(std::iter::once(750))
        .chain(std::iter::repeat_n(600, 6))
        .chain(std::iter::once(500))
        .chain(std::iter::once(750))
        .chain(std::iter::repeat_n(600, 6))
        .collect::<Vec<_>>();
    assert_eq!(damages_a6, expected_a6);

    let mut req_a4 = req.clone();
    req_a4.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(4);
    let report_a4 = run_game_scenario(&req_a4, &limits).unwrap();
    let damages_a4: Vec<u64> = report_a4["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["rounded_damage"].as_u64().unwrap())
        .collect();
    let expected_a4 = std::iter::once(500)
        .chain(std::iter::once(750))
        .chain(std::iter::repeat_n(600, 6))
        .chain(std::iter::once(500))
        .chain(std::iter::once(750))
        .chain(std::iter::repeat_n(600, 6))
        .collect::<Vec<_>>();
    assert_eq!(damages_a4, expected_a4);

    let mut req_a1 = req.clone();
    req_a1.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
    let report_a1 = run_game_scenario(&req_a1, &limits).unwrap();
    let damages_a1: Vec<u64> = report_a1["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["rounded_damage"].as_u64().unwrap())
        .collect();
    let expected_a1 = std::iter::once(500)
        .chain(std::iter::once(500))
        .chain(std::iter::repeat_n(400, 6))
        .chain(std::iter::once(500))
        .chain(std::iter::once(500))
        .chain(std::iter::repeat_n(400, 6))
        .collect::<Vec<_>>();
    assert_eq!(damages_a1, expected_a1);

    let mut req_a0 = req.clone();
    req_a0.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
    let report_a0 = run_game_scenario(&req_a0, &limits).unwrap();
    let damages_a0: Vec<u64> = report_a0["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["rounded_damage"].as_u64().unwrap())
        .collect();
    let expected_a0 = std::iter::once(500)
        .chain(std::iter::once(500))
        .chain(std::iter::repeat_n(400, 6))
        .chain(std::iter::once(500))
        .chain(std::iter::once(500))
        .chain(std::iter::repeat_n(400, 6))
        .collect::<Vec<_>>();
    assert_eq!(damages_a0, expected_a0);
}
