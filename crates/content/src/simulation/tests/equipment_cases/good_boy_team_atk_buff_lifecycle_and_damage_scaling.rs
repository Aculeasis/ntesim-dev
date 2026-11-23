use super::*;

#[test]
fn good_boy_team_atk_buff_lifecycle_and_damage_scaling() {
    let limits = ExecutionLimits::default();
    let mut req = sample_s1_request();
    req.actors[0].static_stats = GameActorStaticStats {
        base_atk: 1000,
        atk: Ratio {
            numerator: 2000,
            denominator: 1,
        },
        hp: 10000,
        base_hp: None,
        hp_percent: Ratio::zero(),
        hp_flat: Ratio::zero(),
        def: 500,
        base_def: Some(500),
        def_percent: Ratio::zero(),
        def_flat: Ratio::zero(),
        crit_rate: Ratio::zero(),
        crit_damage_bonus: Ratio {
            numerator: 1,
            denominator: 2,
        },
        healing_bonus: Ratio::zero(),
        shield_bonus: Ratio::zero(),
        break_intensity: Ratio::zero(),
        cycle_intensity: Ratio::zero(),
        break_efficiency_multiplier: Ratio::one(),
        mental_damage_bonus: Ratio::zero(),
        damage_bonus: Ratio::zero(),
        elemental_bonuses: Default::default(),
        elemental_bonus: Ratio::zero(),
    };
    req.actors[0].gear_procs.clear();

    let mut sakiri = req.actors[0].clone();
    sakiri.slot = 1;
    sakiri.character_id = "character:1003".into();
    sakiri.weapon_id = "weapon:fork_mofeikesi".into();
    sakiri.static_stats.base_atk = 1200;
    sakiri.static_stats.atk = Ratio {
        numerator: 2400,
        denominator: 1,
    };
    sakiri.gear_procs = vec![GearProcConfig::GoodBoyTeamAtk {
        control_atk_bonus: Ratio::zero(),
        team_atk_bonus: Ratio {
            numerator: 1,
            denominator: 10,
        },
        duration_us: 20_000_000,
    }];
    req.actors.push(sakiri);

    req.targets[0].defense = 1080;
    req.targets[0].resistance = Ratio::zero();

    req.action_metadata.clear();
    req.actions = vec![
        GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Mint019_Melee".into(),
            damage_effect_id: "GE_Player_Mint_Melee1_Damage_New".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 0,
            hit_delay_us: 100_000,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 1_000_000,
            target_actor_index: 1,
            cooldown_us: 1_000_000,
        },
        // 3. Sakiri casts UltraSkill at t = 2.0s:
        // Good Boy triggers: grants +10% Team ATK for 20s (expires at 22.0s).
        // Sakiri base_atk 1200 * 10% = +120 ATK -> total 2400 + 120 = 2520 ATK.
        // Hit at 2.1s: 2520 * 0.5 = 1260 dmg.
        GameActionCommand::InvokeAbility {
            timestamp_us: 2_000_000,
            actor_index: 1,
            ability_id: "GA_Sagiri_UltraSkill".into(),
            damage_effect_id: "GE_Player_Sagiri_UltraSkill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 100,
            recovery_us: 1_000_000,
            cooldown_us: 20_000_000,
            hit_delay_us: 100_000,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 4_000_000,
            target_actor_index: 0,
            cooldown_us: 1_000_000,
        },
        // 5. Mint attacks during active buff at t = 5.0s:
        // Mint gets 1000 * 10% from Good Boy and 1200 * 30% from Sakiri.
        // Hit at 5.1s: (2000 + 100 + 360) * 0.5 = 1230 dmg.
        GameActionCommand::InvokeAbility {
            timestamp_us: 5_000_000,
            actor_index: 0,
            ability_id: "GA_Mint019_Melee".into(),
            damage_effect_id: "GE_Player_Mint_Melee1_Damage_New".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 0,
            hit_delay_us: 100_000,
        },
        // 6. Mint attacks after buff expired at t = 25.0s (buff expired at 22.0s):
        // Mint ATK back to 2000 -> 2000 * 0.5 = 1000 dmg.
        GameActionCommand::InvokeAbility {
            timestamp_us: 25_000_000,
            actor_index: 0,
            ability_id: "GA_Mint019_Melee".into(),
            damage_effect_id: "GE_Player_Mint_Melee1_Damage_New".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 0,
            hit_delay_us: 100_000,
        },
    ];
    req.relative_duration_us = 30_000_000;

    let report = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report["success"], true);
    assert_eq!(report["invariants_passed"], true);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );

    let damages: Vec<u64> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["ability_id"] != "GA_Sagiri_QTE")
        .map(|h| h["rounded_damage"].as_u64().unwrap())
        .collect();
    assert_eq!(
        damages,
        vec![
            1000, 231, 1260, 1512, 1512, 1512, 1512, 3780, 1230, 284, 1000, 231
        ]
    );

    let good_boy_event = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["event_kind"] == "good_boy_team_atk")
        .expect("good_boy_team_atk event should be present");
    assert_eq!(good_boy_event["data"]["atk_percent"], "1/10");
    assert_eq!(good_boy_event["data"]["duration_us"], 20_000_000);
}
