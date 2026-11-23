use super::*;

#[test]
fn nanally_a4_underboss_and_resonance_6_damage_bonus() {
    let limits = ExecutionLimits::default();
    let mut req = research_request();
    req.stochastic_crit = None;

    req.actors[0].character_id = "character:1010".into();
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
    req.actors[0].static_stats.crit_damage_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };

    req.action_metadata.clear();
    req.actions = vec![GameActionCommand::InvokeAbility {
        timestamp_us: 0,
        actor_index: 0,
        ability_id: "GA_Nanally_UltraSkill".into(),
        damage_effect_id: "GE_Player_Nanally_Cat_AirAttack_Damage".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 500_000,
        cooldown_us: 6_000_000,
        hit_delay_us: 100_000,
    }];
    req.relative_duration_us = 2_000_000;

    // A6 test: dmg_mult = 1 + 1 (A4) + 3/20 (res_6) = 43/20.
    // A manually supplied packet creates no free A5 before damage; ATK stays1000.
    // Mitigation2/5:1000*43/20*2/5=860.
    let report_a6 = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report_a6["success"], true);
    let hit_a6 = &report_a6["hit_receipts"][0];
    assert_eq!(hit_a6["non_crit_damage"], "860/1");

    // A4 test (awakening = 4, no res_6): dmg_mult = 1 + 1 = 2.
    // Non-crit = 1000 * 2 * 2/5 = 800.
    let mut req_a4 = req.clone();
    req_a4.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(4);
    let report_a4 = run_game_scenario(&req_a4, &limits).unwrap();
    let hit_a4 = &report_a4["hit_receipts"][0];
    assert_eq!(hit_a4["non_crit_damage"], "800/1");

    // A0 test (awakening = 0, no A4, no res_6): dmg_mult = 1.
    // Non-crit = 1000 * 1 * 2/5 = 400.
    let mut req_a0 = req.clone();
    req_a0.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
    let report_a0 = run_game_scenario(&req_a0, &limits).unwrap();
    let hit_a0 = &report_a0["hit_receipts"][0];
    assert_eq!(hit_a0["non_crit_damage"], "400/1");
}
