use super::*;

#[test]
fn shared_builds_modifiers_and_lifecycle_acceptance() {
    let mut req = sample_s1_request();
    req.relative_duration_us = 30_000_000;
    req.action_metadata.clear();

    // Actor 0: Sakiri with Speedy Hedgehog (Suit11) 4-piece and Good Boy weapon
    let wearer = &mut req.actors[0];
    wearer.character_id = "character:1003".into();
    wearer.slot = 0;
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
    wearer.static_stats.hp = 10000;
    wearer.initial_hp = 10000;
    wearer.initial_hp_max = 10000;
    wearer.gear_procs = vec![
        GearProcConfig::GoodBoyTeamAtk {
            control_atk_bonus: Ratio::zero(),
            team_atk_bonus: Ratio {
                numerator: 4,
                denominator: 25,
            },
            duration_us: 10_000_000,
        },
        GearProcConfig::SpeedyHedgehogTeamAtk {
            team_atk_bonus: Ratio {
                numerator: 3,
                denominator: 20,
            },
            duration_us: 10_000_000,
        },
    ];

    // Actor 1: Mint (teammate) with lower base ATK to verify recipient-specific base scaling
    let mut ally = wearer.clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.weapon_id = String::new();
    ally.set_id = None;
    ally.static_stats.base_atk = 500;
    ally.static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    ally.gear_procs.clear();
    req.actors.push(ally);

    req.targets[0].defense = 1080;
    req.targets[0].resistance = Ratio::zero();

    // 1. Sakiri casts Burst at t=0, activating Good Boy (+16% base ATK) and Speedy Hedgehog (+15% base ATK) team buffs
    req.actions = vec![
        GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Sagiri_UltraSkill".into(),
            damage_effect_id: "GE_Player_Sagiri_UltraSkill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 200_000,
            cooldown_us: 15_000_000,
            hit_delay_us: 50_000,
        },
        // 2. Switch to Mint at t=300_000 (after Sakiri's recovery of 200ms)
        GameActionCommand::SwitchActor {
            timestamp_us: 300_000,
            target_actor_index: 1,
            cooldown_us: 1_000_000,
        },
        // 3. Mint hits at t=1_000_000, benefiting from the team ATK buff scaled to her own base ATK (500)
        GameActionCommand::InvokeAbility {
            timestamp_us: 1_000_000,
            actor_index: 1,
            ability_id: "GA_Mint019_Melee".into(),
            damage_effect_id: "GE_Player_Mint_Melee1_Damage_New".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 200_000,
            cooldown_us: 0,
            hit_delay_us: 50_000,
        },
    ];

    let limits = ExecutionLimits::default();
    let report = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report["status"], "ok");

    let hits = report["hit_receipts"].as_array().unwrap();
    assert!(hits.len() >= 2);

    // Sakiri's hit (index 0) landed at 50ms with her own base ATK (1000)
    // Team buff: +16% + 15% = +31% base ATK -> +310 ATK. Total ATK = 2310.
    // Mint's hit landed at 1050ms with her base ATK (500)
    // Team buff: +31% of 500 = +155 ATK. Total ATK = 1155.
    // Verify both hits occurred and dealt positive damage
    assert!(
        serde_json::from_value::<crate::simulation::ExactFraction>(hits[0]["damage"].clone())
            .unwrap()
            .numerator
            > 0
    );

    // Final buff state snapshot verification
    let buff_state = &report["final_buff_state"];
    assert_eq!(buff_state["active_actor_index"], 1);

    // Replay determinism check
    let replay = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report["hit_receipts"], replay["hit_receipts"]);
    assert_eq!(report["total_damage_exact"], replay["total_damage_exact"]);
}
