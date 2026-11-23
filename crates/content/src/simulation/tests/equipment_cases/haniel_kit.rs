use super::*;

#[test]
fn haniel_a3_a5_and_resonance_6() {
    let limits = ExecutionLimits::default();
    let mut req = research_request();
    req.stochastic_crit = None;

    req.actors[0].character_id = "character:1020".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actors[0].static_stats.base_atk = 0;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio {
        numerator: 0,
        denominator: 1,
    };
    req.actors[0].static_stats.crit_rate = Ratio {
        numerator: 0,
        denominator: 1,
    };
    req.actors[0].static_stats.crit_damage_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };

    req.action_metadata.clear();
    req.actions = vec![
        GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Haniel_UltraSkill".into(),
            damage_effect_id: "GE_Player_Haniel_Ultraskill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 20_000_000,
            hit_delay_us: 100_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 1_000_000,
            actor_index: 0,
            ability_id: "GA_Haniel_Skill".into(),
            damage_effect_id: "GE_Player_Haniel_Skill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 10_000_000,
            hit_delay_us: 100_000,
        },
    ];
    req.relative_duration_us = 2_000_000;

    // A6 test:
    // Paranormal Ace active: A5 gives +30% dmg bonus -> dmg_mult = 13/10.
    // Gross = 1000 * 13/10 = 1300. Def/res factor = 2/5 -> Non-crit = 1300 * 2/5 = 520.
    let report_a6 = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report_a6["success"], true);
    assert_eq!(report_a6["hits_settled"], 3);
    let hit0_a6 = &report_a6["hit_receipts"][0];
    assert_eq!(hit0_a6["non_crit_damage"], "520/1");
    let hit1_a6 = &report_a6["hit_receipts"][1];
    assert_eq!(hit1_a6["non_crit_damage"], "520/1");
    let hit2_a6 = &report_a6["hit_receipts"][2];
    assert_eq!(hit2_a6["non_crit_damage"], "520/1");

    // Also check crit_rate with A3 (+20% = 1/5) and crit_damage with resonance_6 (+12% = 3/25):
    // base crit rate = 0 -> total crit rate = 1/5.
    // base crit dmg = 1/2 (50%) + res_6 3/25 (12%) = 31/50 (62%).
    // crit_mult = 1 + 31/50 = 81/50.
    // Crit damage = 520 * 81/50 = 4212/5.
    assert_eq!(hit0_a6["crit_rate"], "1/5");
    assert_eq!(hit0_a6["crit_damage"], "4212/5");

    let mut req_a2 = req.clone();
    req_a2.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(2);
    let report_a2 = run_game_scenario(&req_a2, &limits).unwrap();
    assert_eq!(report_a2["hits_settled"], 3);
    let hit0_a2 = &report_a2["hit_receipts"][0];
    assert_eq!(hit0_a2["non_crit_damage"], "400/1");
    let hit1_a2 = &report_a2["hit_receipts"][1];
    assert_eq!(hit1_a2["non_crit_damage"], "400/1");
    let hit2_a2 = &report_a2["hit_receipts"][2];
    assert_eq!(hit2_a2["non_crit_damage"], "400/1");
    assert_eq!(hit0_a2["crit_rate"], "0/1");
    assert_eq!(hit0_a2["crit_damage"], "600/1"); // 400 * (1 + 0.5) = 600
}
