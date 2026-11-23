use super::*;

fn cast(t: u64, actor: usize, skill: bool, delay: u64) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us: t,
        actor_index: actor,
        ability_id: if skill {
            "GA_Mint019_Skill"
        } else {
            "GA_Mint019_Melee"
        }
        .into(),
        damage_effect_id: if skill {
            "GE_Player_Mint_Skill1_Damage_New"
        } else {
            "GE_Player_Mint_Melee1_Damage_New"
        }
        .into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: delay,
    }
}

fn fixture() -> GameScenarioRequest {
    let mut req = sample_s1_request();
    req.relative_duration_us = 20_000_000;
    req.actors[0].weapon_id = "weapon:fork_lingganzhongjiezhe".into();
    req.actors[0].set_id = Some("set:Suit8".into());
    req.actors[0].gear_procs = vec![GearProcConfig::ShadowCreedAtk {
        atk_bonus: Ratio {
            numerator: 1,
            denominator: 4,
        },
        duration_us: 10_000_000,
    }];
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 2000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.targets[0].resistance = Ratio::zero();
    req.targets[0].defense = 1080;
    req.actions.clear();
    req
}

#[test]
fn shadow_creed_uses_self_base_atk_refresh_and_exact_expiry_before_hits() {
    let mut req = fixture();
    req.actions = vec![
        cast(0, 0, false, 0),
        cast(1_000_000, 0, true, 0),
        cast(5_000_000, 0, true, 0),
        cast(11_000_000, 0, false, 0),
        cast(14_999_999, 0, false, 0),
        cast(15_000_000, 0, false, 0),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // 2000 * 1/2 mitigation; active: (2000 + 1000 * 1/4) * 1/2.
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(
        hits.iter()
            .map(|hit| hit["rounded_damage"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        // Child uses native .9; manually configured primary uses1.0.
        vec![1000, 231, 1125, 1013, 1125, 1013, 1125, 260, 1125, 260, 170]
    );
    assert_eq!(
        report["resolved_model_assumptions"][crate::sets::shadow_creed::HYPOTHESIS_ID.as_str()],
        crate::sets::shadow_creed::HYPOTHESIS_ID.description()
    );
}

#[test]
fn shadow_creed_preserves_off_field_delivery_without_buffing_the_active_ally() {
    let mut req = fixture();
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1003".into();
    ally.weapon_id = String::new();
    ally.gear_procs.clear();
    req.actors.push(ally);
    let mut ally_hit = cast(2_000_000, 1, false, 0);
    if let GameActionCommand::InvokeAbility {
        ability_id,
        damage_effect_id,
        ..
    } = &mut ally_hit
    {
        *ability_id = "GA_Sagiri_Melee".into();
        *damage_effect_id = "GE_Player_Sagiri_Melee1_Damage".into();
    }
    req.actions = vec![
        cast(0, 0, true, 3_000_000),
        GameActionCommand::SwitchActor {
            timestamp_us: 1_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        ally_hit,
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    super::support::assert_non_support_damage_prefix(&report, &[1000, 1125]);
}

#[test]
fn shadow_creed_does_not_trigger_on_ultimate_cast() {
    let mut req = fixture();
    req.actions = vec![GameActionCommand::InvokeAbility {
        timestamp_us: 0,
        actor_index: 0,
        ability_id: "GA_Mint019_UltraSkill".into(),
        damage_effect_id: "GE_Player_Mint_UtraSkill1_Damage_New".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 0,
    }];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // 2000 * 1/2; an Ultimate cast must not grant the +25% self ATK buff.
    assert_eq!(report["hit_receipts"][0]["rounded_damage"], 1000);
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["event_kind"] == "shadow_creed_atk")
    );
}

#[test]
fn shadow_creed_rejects_bad_equipment_and_does_not_trigger_on_failed_admission() {
    let mut req = fixture();
    let limits = ExecutionLimits::default();
    let mut bad = req.clone();
    bad.actors[0].set_id = None;
    assert!(
        run_game_scenario(&bad, &limits)
            .unwrap_err()
            .contains("requires equipped")
    );
    let duplicate = req.actors[0].gear_procs[0].clone();
    bad = req.clone();
    bad.actors[0].gear_procs.push(duplicate);
    assert!(
        run_game_scenario(&bad, &limits)
            .unwrap_err()
            .contains("duplicates")
    );
    req.actors[0].initial_energy = 0;
    let mut action = cast(0, 0, true, 0);
    if let GameActionCommand::InvokeAbility { energy_cost, .. } = &mut action {
        *energy_cost = 100;
    }
    req.actions = vec![action];
    let report = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["event_kind"] == "shadow_creed_atk")
    );
}
