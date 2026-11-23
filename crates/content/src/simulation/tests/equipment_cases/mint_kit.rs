use super::*;

#[test]
fn mint_a4_crit_rate_is_connected_to_the_shared_hit_calculation() {
    let mut req = research_request();
    req.actors[0].character_id = "character:1019".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(4);
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actors[0].gear_procs.clear();
    req.stochastic_crit = None;
    req.action_metadata.clear();
    req.actions = vec![GameActionCommand::InvokeAbility {
        timestamp_us: 0,
        actor_index: 0,
        ability_id: "GA_Mint019_Melee".into(),
        damage_effect_id: "GE_Player_Mint_Melee1_Damage_New".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 0,
    }];
    req.relative_duration_us = 1;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hit_receipts"][0]["crit_rate"], "3/25");
}

#[test]
fn mint_a4_crit_rate_bonus_and_a5_crit_damage_bonus() {
    let limits = ExecutionLimits::default();
    let mut req = research_request();
    req.stochastic_crit = None;

    req.actors[0].character_id = "character:1019".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(5);
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.crit_rate = Ratio {
        numerator: 1,
        denominator: 10,
    };
    req.actors[0].static_stats.crit_damage_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };

    req.action_metadata.clear();
    req.actions = vec![GameActionCommand::InvokeAbility {
        timestamp_us: 0,
        actor_index: 0,
        ability_id: "GA_Mint019_Skill".into(),
        damage_effect_id: "GE_Player_Mint_Skill1_Damage_New".into(),
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

    let hit = &report["hit_receipts"][0];
    // Base crit rate: 10% (1/10) + A4 12% (3/25) = 22% (11/50)
    assert_eq!(hit["crit_rate"], "11/50");

    // Non-crit damage:
    // ATK 1000, A2 +40% DMG -> 1000 * 1.4 = 1400. Mitigation: 0.4.
    // non_crit = 1400 * 0.4 = 560
    assert_eq!(hit["non_crit_damage"], "560/1");

    // Crit damage multiplier:
    // 1 + base 50% (1/2) + A5 25% (1/4) = 1 + 75% = 1.75 (7/4)
    // crit_damage = 560 * 1.75 = 980
    assert_eq!(hit["crit_damage"], "980/1");
}
