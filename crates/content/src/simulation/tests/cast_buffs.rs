use super::*;

#[test]
fn ultimate_cast_buffs_settle_before_hits_and_add_each_recipients_base_atk() {
    for delay in [0, 100_000] {
        let mut req = sample_s1_request();
        req.relative_duration_us = 30_000_000;
        let wearer = &mut req.actors[0];
        wearer.character_id = "character:1003".into();
        wearer.awakening = ntesim_data::AwakeningSelection::prefix(0);
        wearer.weapon_id = "weapon:fork_mofeikesi".into();
        wearer.set_id = Some("set:Suit11".into());
        wearer.static_stats.base_atk = 1000;
        wearer.static_stats.atk = Ratio {
            numerator: 2000,
            denominator: 1,
        };
        wearer.static_stats.elemental_bonus = Ratio::zero();
        wearer.static_stats.crit_rate = Ratio::zero();
        wearer.gear_procs = vec![
            GearProcConfig::GoodBoyTeamAtk {
                control_atk_bonus: Ratio::zero(),
                team_atk_bonus: Ratio {
                    numerator: 4,
                    denominator: 25,
                },
                duration_us: 20_000_000,
            },
            GearProcConfig::SpeedyHedgehogTeamAtk {
                team_atk_bonus: Ratio {
                    numerator: 3,
                    denominator: 20,
                },
                duration_us: 20_000_000,
            },
        ];
        let mut ally = wearer.clone();
        ally.slot = 1;
        ally.character_id = "character:1019".into();
        ally.weapon_id = String::new();
        ally.set_id = None;
        ally.static_stats.base_atk = 500;
        ally.gear_procs.clear();
        req.actors.push(ally);
        req.targets[0].defense = 1080;
        req.targets[0].resistance = Ratio::zero();
        let hit = |timestamp_us, actor_index, ultimate| GameActionCommand::InvokeAbility {
            timestamp_us,
            actor_index,
            ability_id: match (actor_index, ultimate) {
                (0, true) => "GA_Sagiri_UltraSkill",
                (0, false) => "GA_Sagiri_Melee",
                _ => "GA_Mint019_Melee",
            }
            .into(),
            damage_effect_id: match (actor_index, ultimate) {
                (0, true) => "GE_Player_Sagiri_UltraSkill1_Damage",
                (0, false) => "GE_Player_Sagiri_Melee1_Damage",
                _ => "GE_Player_Mint_Melee1_Damage_New",
            }
            .into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: delay,
        };
        req.actions = vec![
            hit(0, 0, false),
            hit(1_000_000, 0, true),
            hit(5_000_000, 0, true),
            GameActionCommand::SwitchActor {
                timestamp_us: 6_000_000,
                target_actor_index: 1,
                cooldown_us: 0,
            },
            hit(7_000_000, 1, false),
            hit(21_000_000, 1, false),
            hit(26_000_000 - delay, 1, false),
        ];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        // ATK bonus: 16% weapon + 15% set, additive against each recipient base.
        // Wearer: (2000 + 1000*.31)/2 = 1155. Ally: (2000 + 500*.31 + 1000*.30)/2 = 1227.5.
        // Ordinary Sakiri Burst adds source-base flat ATK to allies only.
        let values = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|h| h["rounded_damage"].as_u64().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            values,
            vec![
                1000, 1155, 1386, 1386, 1386, 1386, 3465, 1155, 1386, 1386, 1386, 1386, 3465, 1228,
                284, 1228, 284, 1000, 231,
            ]
        );
        assert_eq!(
            report["resolved_model_assumptions"][crate::gear::CAST_BUFF_HYPOTHESIS_ID.as_str()],
            crate::gear::CAST_BUFF_HYPOTHESIS_ID.description()
        );
        assert_eq!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["event_kind"] == "good_boy_team_atk")
                .count(),
            2
        );
    }
}

#[test]
fn speedy_hedgehog_ultimate_refreshes_nonstacking_team_atk_until_expiry() {
    let mut req = sample_s1_request();
    req.relative_duration_us = 30_000_000;
    let wearer = &mut req.actors[0];
    wearer.character_id = "character:1003".into();
    wearer.awakening = ntesim_data::AwakeningSelection::prefix(0);
    wearer.weapon_id = "weapon:fork_mofeikesi".into();
    wearer.set_id = Some("set:Suit11".into());
    wearer.static_stats.base_atk = 1000;
    wearer.static_stats.atk = Ratio {
        numerator: 2000,
        denominator: 1,
    };
    wearer.static_stats.elemental_bonus = Ratio::zero();
    wearer.static_stats.crit_rate = Ratio::zero();
    wearer.gear_procs = vec![GearProcConfig::SpeedyHedgehogTeamAtk {
        team_atk_bonus: Ratio {
            numerator: 3,
            denominator: 20,
        },
        duration_us: 20_000_000,
    }];
    let mut ally = wearer.clone();
    ally.slot = 1;
    ally.set_id = None;
    ally.static_stats.base_atk = 500;
    ally.gear_procs.clear();
    req.actors.push(ally);
    req.targets[0].defense = 1080;
    req.targets[0].resistance = Ratio::zero();
    let hit = |timestamp_us, actor_index, ultimate| GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index,
        ability_id: if ultimate {
            "GA_Sagiri_UltraSkill"
        } else {
            "GA_Sagiri_Melee"
        }
        .into(),
        damage_effect_id: if ultimate {
            "GE_Player_Sagiri_UltraSkill1_Damage"
        } else {
            "GE_Player_Sagiri_Melee1_Damage"
        }
        .into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 0,
    };
    req.actions = vec![
        hit(0, 0, false),
        hit(1_000_000, 0, true),
        hit(5_000_000, 0, true),
        GameActionCommand::SwitchActor {
            timestamp_us: 6_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        hit(7_000_000, 1, false),
        hit(22_000_000, 1, false),
        hit(26_000_000, 1, false),
        GameActionCommand::SwitchActor {
            timestamp_us: 27_000_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        hit(28_000_000, 0, false),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // Oracles (Sagiri packet structure established by
    // ultimate_cast_buffs_settle_before_hits_and_add_each_recipients_base_atk):
    // melee = ATK/2; Ultimate = ATK/2, four ATK*0.6, ATK*1.5. The 15% team buff
    // adds base_atk*0.15 to each recipient's ATK against that recipient's base.
    // Wearer melee@0 (no buff yet): 2000/2 = 1000.
    // Ultimate@1s, same-time damage with buff (bd9a1ca ordering):
    // (2000 + 1000*0.15)/2 = 1075; 2150*0.6 = 1290; 2150*1.5 = 3225.
    // Ultimate@5s refreshes instead of stacking: still 1075/1290/3225
    // (a second 15% stack would give 1150/1380/3450).
    // Ally melee@7s: (2000 + 500*0.15 + 1000*0.30)/2 = 1187.5 -> 1188 ties upward,
    // proving the team recipient and the per-recipient base.
    // Ally melee@22s: 1188, so the 5s cast refreshed the expiry to 25s
    // (without refresh the 1s cast would have expired at 21s).
    // Ally melee@26s: 1000, the 20s duration expired at 25s.
    // Wearer melee@28s: 1000, a non-Ultimate cast does not retrigger the buff.
    let values = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["ability_id"] != "GA_Sagiri_QTE")
        .map(|h| h["rounded_damage"].as_u64().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        values,
        vec![
            1000, 1075, 1290, 1290, 1290, 1290, 3225, 1075, 1290, 1290, 1290, 1290, 3225, 1188,
            1188, 1000, 1000,
        ]
    );
    assert_eq!(
        report["resolved_model_assumptions"][crate::gear::CAST_BUFF_HYPOTHESIS_ID.as_str()],
        crate::gear::CAST_BUFF_HYPOTHESIS_ID.description()
    );
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["event_kind"] == "speedy_hedgehog_team_atk")
            .count(),
        2
    );
}
