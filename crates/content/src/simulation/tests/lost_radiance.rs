use super::*;

fn cast(t: u64, actor: usize, ultimate: bool, delay: u64) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us: t,
        actor_index: actor,
        ability_id: if ultimate {
            "GA_Mint019_UltraSkill"
        } else {
            "GA_Mint019_Melee"
        }
        .into(),
        damage_effect_id: if ultimate {
            "GE_Player_Mint_UtraSkill1_Damage_New"
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
    req.relative_duration_us = 40_000_000;
    req.actors[0].character_id = "character:1019".into();
    req.actors[0].character_level = 80;
    req.actors[0].weapon_id = "weapon:fork_lingganzhongjiezhe".into();
    req.actors[0].set_id = Some("set:Suit6".into());
    req.actors[0].gear_procs = vec![GearProcConfig::LostRadianceDefIgnore {
        def_ignore: Ratio {
            numerator: 1,
            denominator: 4,
        },
        duration_us: 20_000_000,
    }];
    req.actors[0].static_stats.base_atk = 1400;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1400,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.targets[0].resistance = Ratio::zero();
    req.targets[0].defense = 1080;
    req.actions.clear();
    req
}

fn packets_for_command(
    receipts: &[serde_json::Value],
    command_index: u64,
) -> Vec<&serde_json::Value> {
    receipts
        .iter()
        .filter(|hit| hit["command_index"].as_u64() == Some(command_index))
        .collect()
}

#[test]
fn lost_radiance_applies_def_ignore_on_ultimate_and_resets_duration() {
    let mut req = fixture();
    req.actions = vec![
        cast(0, 0, false, 0),
        cast(1_000_000, 0, true, 0),
        cast(5_000_000, 0, false, 0),
        cast(15_000_000, 0, true, 0),
        cast(30_000_000, 0, false, 0),
        cast(38_903_489, 0, false, 0),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // Base 1080 defense / (1080 + 1080) = 1/2 mitigation without ignore -> 1400 * 1/2 = 700.
    // 25% DEF ignore reduces effective DEF to 810; 1080 / (1080 + 810) = 4/7 -> 1400 * 4/7 = 800.
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(
        hits.iter()
            .map(|hit| hit["rounded_damage"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![
            700, 162, 800, 470, 470, 470, 470, 470, 2988, 800, 185, 800, 470, 470, 470, 470, 470,
            2988, 800, 185, 700, 162,
        ]
    );
    assert_eq!(
        report["resolved_model_assumptions"][crate::sets::lost_radiance::HYPOTHESIS_ID.as_str()],
        crate::sets::lost_radiance::HYPOTHESIS_ID.description()
    );
    assert_eq!(
        report["resolved_model_assumptions"][crate::gear::CAST_BUFF_HYPOTHESIS_ID.as_str()],
        crate::gear::CAST_BUFF_HYPOTHESIS_ID.description()
    );
}

#[test]
fn lost_radiance_is_self_actor_and_does_not_buff_active_ally() {
    let mut req = fixture();
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1003".into();
    ally.weapon_id = String::new();
    ally.set_id = None;
    ally.gear_procs.clear();
    req.actors.push(ally);

    let mut ally_hit = cast(3_000_000, 1, false, 0);
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
        cast(1_000_000, 0, true, 0),
        GameActionCommand::SwitchActor {
            timestamp_us: 2_000_000,
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
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits[0]["rounded_damage"], 800);
    assert_eq!(hits.last().unwrap()["rounded_damage"], 700);
}

#[test]
fn lost_radiance_def_ignore_is_a_defense_factor_not_a_damage_multiplier() {
    // Independent numeric counterexample per COMBAT_FORMULA_CONTRACT section 3:
    // the 25% DEF-ignore is subtracted inside the defense factor, not added
    // to a damage zone. Hand-derived oracles below; every value is exact.
    //
    // Fixture: attacker level 80, B = 1400 * 1 = 1400, target DEF 1080,
    // no resistance, no crit.  A = 600 + 6 * 80 = 1080.
    //
    // No ignore:   DEF_effective = 1080; F_def = 1080 / (1080 + 1080) = 1/2;
    //              D = 1400 * 1/2 = 700.
    // 25% ignore:  DEF_effective = 1080 * 3/4 = 810;
    //              F_def = 1080 / (1080 + 810) = 1080/1890 = 4/7;
    //              D = 1400 * 4/7 = 800.
    // Wrong-zone model ("+25% damage"): D = 700 * (1 + 1/4) = 875 != 800.
    // The ignore adds 14.2857% (4/7 vs 1/2), not 25%.
    let mut req = fixture();
    req.actions = vec![
        cast(0, 0, false, 0),
        cast(1_000_000, 0, true, 0),
        cast(5_000_000, 0, false, 0),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let damages = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|hit| hit["rounded_damage"].as_u64().unwrap())
        .collect::<Vec<_>>();
    // Hit layout: melee = [700, 162], Ultimate = [800, 470 x5, 2988],
    // melee = [800, 185]. Index 9 is the first packet of the post-Ultimate
    // melee, fired while the 20s ignore window is live.
    assert_eq!(damages[0], 700);
    assert_eq!(damages[9], 800);
    assert_ne!(damages[9], 875);
}

#[test]
fn lost_radiance_expires_on_the_ultimate_window_boundary() {
    let mut req = fixture();
    req.actions = vec![
        cast(0, 0, false, 0),
        cast(1_000_000, 0, true, 0),
        cast(20_500_000 + 3_903_489, 0, false, 0),
        cast(21_000_000 + 3_903_489, 0, false, 0),
        cast(21_000_001 + 3_903_489, 0, false, 0),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );

    // The Ultimate is admitted at 1s, so the 20s effect expires at 21s in
    // world combat time. Compare matching native packets to the same rotation
    // without the proc: at 20.5s each packet gains 8/7 from the defense zone;
    // at and after 21s each returns exactly to its no-proc value. Later normal
    // inputs are different chain stages, so their absolute damage is not 700.
    let mut baseline_request = req.clone();
    baseline_request.actors[0].gear_procs.clear();
    let baseline = run_game_scenario(&baseline_request, &ExecutionLimits::default()).unwrap();
    assert_eq!(baseline["execution_completed"], true);
    let hits = report["hit_receipts"].as_array().unwrap();
    let baseline_hits = baseline["hit_receipts"].as_array().unwrap();
    let before = packets_for_command(hits, 0);
    let before_baseline = packets_for_command(baseline_hits, 0);
    assert_eq!(before.len(), before_baseline.len());
    for (actual, expected) in before.iter().zip(before_baseline) {
        assert_eq!(actual["non_crit_damage"], expected["non_crit_damage"]);
    }
    let during = packets_for_command(hits, 2);
    let during_baseline = packets_for_command(baseline_hits, 2);
    assert_eq!(during.len(), during_baseline.len());
    assert!(!during.is_empty());
    for (actual, expected) in during.iter().zip(during_baseline) {
        let actual_f: ExactFraction =
            serde_json::from_value(actual["non_crit_damage"].clone()).unwrap();
        let base_f: ExactFraction =
            serde_json::from_value(expected["non_crit_damage"].clone()).unwrap();
        assert_eq!(
            actual_f.numerator * 7 * base_f.denominator,
            base_f.numerator * 8 * actual_f.denominator
        );
    }
    for command_index in [3, 4] {
        let after = packets_for_command(hits, command_index);
        let after_baseline = packets_for_command(baseline_hits, command_index);
        assert_eq!(after.len(), after_baseline.len());
        for (actual, expected) in after.iter().zip(after_baseline) {
            assert_eq!(actual["non_crit_damage"], expected["non_crit_damage"]);
        }
    }
    assert!(
        hits.iter()
            .any(|hit| hit["time_us"] == (20_500_000 + 3_903_489))
    );
    assert!(
        hits.iter()
            .any(|hit| hit["time_us"] == (21_000_000 + 3_903_489))
    );
    assert!(
        hits.iter()
            .any(|hit| hit["time_us"] == (21_000_001 + 3_903_489))
    );
}

#[test]
fn lost_radiance_rejects_bad_equipment_and_does_not_trigger_on_failed_admission() {
    let mut req = fixture();
    let limits = ExecutionLimits::default();
    let mut bad = req.clone();
    bad.actors[0].set_id = None;
    assert!(
        run_game_scenario(&bad, &limits)
            .unwrap_err()
            .contains("requires equipped set:Suit6")
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
            .any(|e| e["event_kind"] == "lost_radiance_def_ignore")
    );
}
