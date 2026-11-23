use super::*;
use serde_json::json;

fn attack(timestamp_us: u64, packet: &str) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index: 0,
        ability_id: "GA_Kuhara_Melee".into(),
        damage_effect_id: packet.into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 100,
    }
}

fn request() -> GameScenarioRequest {
    let mut req = sample_s1_request();
    req.actors[0].character_id = "character:1055".into();
    req.actors[0].weapon_id.clear();
    req.actors[0].gear_procs.clear();
    req.actors[0].talent_levels = Some([1; 3]);
    req.actors[0].character_max_level = Some(80);
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.targets[0].resistance = Ratio::zero();
    req.actions = vec![attack(0, "GE_Player_Kuhara_Melee1_Damage")];
    req
}

#[test]
fn jiuyuan_first_attack_uses_normal_talent_without_applying_pact_or_awakening_bonuses() {
    for (level, awakening, expected) in [
        (1, 0, 109),
        (8, 0, 187),
        (8, 1, 187),
        (8, 5, 202),
        (8, 6, 202),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([level, 1, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 1);
        // 1000 ATK * native Normal coefficient (.218/.374/.404) * .5 DEF.
        // A3 resonance grants level9 at A5/A6. No Pact means no A1 ATK,
        // A5 +8% damage or resonance6 Anima RES reduction.
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            json!(format!("{}/{}", expected, 1))
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"
        );
        assert!(crate::simulation::tests::support::final_buffs(&report).is_empty());
    }
}

#[test]
fn jiuyuan_earlier_and_other_melee_packets_do_not_create_or_refresh_pact() {
    let excluded = [
        "GE_Player_Kuhara_Melee1_Damage",
        "GE_Player_Kuhara_Melee2_Damage",
        "GE_Player_Kuhara_Melee2_1_Damage",
        "GE_Player_Kuhara_Melee3_Damage",
        "GE_Player_Kuhara_Melee3_1_Damage",
        "GE_Player_Kuhara_Melee3_2_Damage",
        "GE_Player_Kuhara_Melee4_Damage",
        "GE_Player_Kuhara_Melee4_1_Damage",
        "GE_Player_Kuhara_Melee4_2_Damage",
        "GE_Player_Kuhara_Melee4_3_Damage",
        "GE_Player_Kuhara_Branch_Damage",
        "GE_Player_Kuhara_BranchPower_Damage",
        "GE_Player_Kuhara_AimAttack_Damage",
        "GE_Player_Kuhara_AirAttack_Damage",
        "GE_Player_Kuhara_SeedReaction_Damage",
        "GE_Player_Kuhara_BudBoom_Damage",
        "GE_Player_Kuhara_TargetActivateSkill_Damage",
    ];
    for packet in excluded {
        let mut req = request();
        req.actors[0].talent_levels = None;
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
        req.relative_duration_us = 100;
        req.actions = vec![attack(0, packet)];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert!(
            crate::simulation::tests::support::final_buffs(&report).is_empty(),
            "{packet}"
        );
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            json!("500/1"),
            "{packet}"
        );

        // Admit the existing fifth-hit Pact model at100us, then this packet at
        //10s. It may benefit from the existing Pact but cannot extend its25s.
        req.actions
            .insert(0, attack(0, "GE_Player_Kuhara_Melee5_Damage"));
        req.actions[1] = attack(10_000_000, packet);
        req.relative_duration_us = 25_000_100;
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["hits_settled"], 2);
        assert!(
            crate::simulation::tests::support::final_buffs(&report).is_empty(),
            "{packet} refreshed Pact"
        );
    }
}

#[test]
fn jiuyuan_fifth_packet_pact_requires_connected_living_target_and_off_field_first_attack_keeps_owner()
 {
    let mut req = request();
    req.actors[0].talent_levels = None;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actions = vec![attack(0, "GE_Player_Kuhara_Melee5_Damage")];
    req.relative_duration_us = 100;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    // Existing Pact bonuses:1050 ATK * .5 DEF *1.08 damage. The shared
    // negative-RES model at-.1 is1 + .1/1.1 =12/11 (not linear1.1).
    assert_eq!(
        report["hit_receipts"][0]["non_crit_damage"],
        json!("6804/11")
    );
    assert_eq!(
        crate::simulation::tests::support::final_buffs(&report).len(),
        1
    );
    if let GameActionCommand::InvokeAbility { atk_rate, .. } = &mut req.actions[0] {
        *atk_rate = Ratio::zero();
    }
    let zero = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert!(crate::simulation::tests::support::final_buffs(&zero).is_empty());
    req.targets[0].hp_current = 0;
    let dead = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(dead["hits_settled"], 0);
    assert!(crate::simulation::tests::support::final_buffs(&dead).is_empty());

    req = request();
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    req.actors.push(ally);
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 50,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    let off_field = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(off_field["hit_receipts"][0]["actor_index"], 0);
    assert_eq!(
        off_field["hit_receipts"][0]["non_crit_damage"],
        json!("109/1")
    );
    req.relative_duration_us = 99;
    let pending = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(pending["execution_completed"], true);
    assert_eq!(pending["hits_settled"], 0);
    assert_eq!(pending["pending_events"], 1);
}
