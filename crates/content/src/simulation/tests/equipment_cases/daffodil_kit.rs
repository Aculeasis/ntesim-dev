use super::*;

#[test]
fn daffodil_zero_support_has_no_a1_skill_bonus_and_retains_a6_def_ignore() {
    let limits = ExecutionLimits::default();
    let mut req = research_request();
    req.stochastic_crit = None;

    req.actors[0].character_id = "character:1054".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio {
        numerator: 0,
        denominator: 1,
    };

    req.action_metadata.clear();
    req.actions = vec![GameActionCommand::InvokeAbility {
        timestamp_us: 0,
        actor_index: 0,
        ability_id: "GA_Daffodill_Skill".into(),
        damage_effect_id: "GE_Player_Daffodill_Skill1_Damage".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 500_000,
        cooldown_us: 6_000_000,
        hit_delay_us: 100_000,
    }];
    req.relative_duration_us = 2_000_000;

    let report = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report["success"], true);
    assert_eq!(report["hits_settled"], 8);

    let hit = &report["hit_receipts"][0];
    // ATK 1000, rate 1.0; zero Support stacks => no A1 addition, 1000 gross.
    // DEF ignore A6 = 15% (3/20).
    // Target DEF 1080 -> effective DEF = 1080 * 0.85 = 918.
    // def_factor = 1080 / (1080 + 918) = 1080 / 1998 = 20 / 37.
    // Resistance = 1/5 (20%) -> res_factor = 4/5.
    // Mitigation = (20 / 37) * (4 / 5) = 16 / 37.
    // non_crit = 1000 * (16 / 37) = 16000 / 37.
    assert_eq!(hit["non_crit_damage"], "16000/37");

    let mut req_a0 = req.clone();
    req_a0.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
    let report_a0 = run_game_scenario(&req_a0, &limits).unwrap();
    assert_eq!(report_a0["hits_settled"], 8);
    let hit_a0 = &report_a0["hit_receipts"][0];
    // ATK 1000, rate 1.0, dmg_mult = 1 -> 1000 gross.
    // No DEF ignore: def_factor = 1080 / 2160 = 1/2.
    // Mitigation = (1/2) * (4/5) = 2/5 = 0.4.
    // non_crit = 1000 * 0.4 = 400.
    assert_eq!(hit_a0["non_crit_damage"], "400/1");
}
