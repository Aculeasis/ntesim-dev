use super::*;

fn request() -> GameScenarioRequest {
    let mut req = research_request();
    req.actors[0].weapon_id = "weapon:fork_KnightCandy".into();
    req.actors[0].gear_procs = vec![GearProcConfig::KnightCandyCritDamage {
        crit_damage_per_stack: Ratio {
            numerator: 1,
            denominator: 25,
        },
        duration_us: 10_000_000,
    }];
    req.actors[0].static_stats.crit_rate = Ratio::one();
    req.targets[0].resistance = Ratio::zero();
    req
}

#[test]
fn knight_candy_stacks_only_after_critical_damage_and_never_buffs_the_trigger() {
    let mut req = request();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    // 1000 ATK * .5 DEF * (2 + prior stacks * .04).
    assert_eq!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|h| h["rounded_damage"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![1000, 1020, 1040, 1060, 1080, 1100]
    );
    assert_eq!(
        report["resolved_model_assumptions"][crate::weapons::knight_candy::HYPOTHESIS_ID.as_str()],
        crate::weapons::knight_candy::HYPOTHESIS_ID.description()
    );
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["event_kind"] == "knight_candy_crit_stack")
    );
    req.actors[0].static_stats.crit_rate = Ratio::one();
    for action in &mut req.actions {
        if let GameActionCommand::InvokeAbility { atk_rate, .. } = action {
            *atk_rate = Ratio::zero();
        }
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["event_kind"] == "knight_candy_crit_stack")
    );
}

#[test]
fn knight_candy_caps_refreshes_at_cap_and_expires_before_equal_time_hits() {
    let mut req = request();
    let template = req.actions[0].clone();
    let metadata = req.action_metadata[&0].clone();
    let times = (0..12)
        .map(|i| i * 1_000_000)
        .chain([20_500_000, 30_500_000])
        .collect::<Vec<_>>();
    req.relative_duration_us = 41_000_000;
    req.actions = times
        .iter()
        .map(|t| {
            let mut action = template.clone();
            if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut action {
                *timestamp_us = *t;
            }
            action
        })
        .collect();
    req.action_metadata = (0..times.len()).map(|i| (i, metadata.clone())).collect();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits[10]["rounded_damage"], 1200);
    assert_eq!(hits[11]["rounded_damage"], 1200);
    assert_eq!(hits[12]["rounded_damage"], 1200);
    assert_eq!(hits[13]["rounded_damage"], 1000);
    assert_eq!(
        report,
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap()
    );
}

#[test]
fn knight_candy_simultaneous_packets_use_prior_stacks_and_dead_targets_do_not_trigger() {
    let mut req = request();
    for action in &mut req.actions {
        if let GameActionCommand::InvokeAbility { timestamp_us, .. } = action {
            *timestamp_us = 0;
        }
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|h| h["rounded_damage"] == 1000)
    );
    req.targets[0].hp_current = 1;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["event_kind"] == "knight_candy_crit_stack")
            .count(),
        1
    );
}

#[test]
fn knight_candy_rejects_duplicate_wrong_weapon_zero_duration_and_overflow() {
    let req = request();
    let mut bad = req.clone();
    let duplicate = bad.actors[0].gear_procs[0].clone();
    bad.actors[0].gear_procs.push(duplicate);
    assert!(
        run_game_scenario(&bad, &ExecutionLimits::default())
            .unwrap_err()
            .contains("duplicates")
    );
    bad = req.clone();
    bad.actors[0].weapon_id.clear();
    assert!(
        run_game_scenario(&bad, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires equipped")
    );
    for duration_us in [0, u64::MAX] {
        bad = req.clone();
        bad.actors[0].gear_procs = vec![GearProcConfig::KnightCandyCritDamage {
            crit_damage_per_stack: Ratio::one(),
            duration_us,
        }];
        assert!(run_game_scenario(&bad, &ExecutionLimits::default()).is_err());
    }
}

#[test]
fn knight_candy_delayed_off_field_hits_keep_wearer_stacks_and_do_not_buff_allies() {
    let mut req = request();
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1003".into();
    ally.weapon_id.clear();
    ally.gear_procs.clear();
    req.actors.push(ally);
    req.stochastic_crit
        .as_mut()
        .unwrap()
        .eligible_effects
        .insert("character:1003".into(), BTreeSet::from(["*".into()]));
    req.stochastic_crit
        .as_mut()
        .unwrap()
        .actor_sequences
        .insert("character:1003".into(), 55);
    req.actions.truncate(2);
    if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut req.actions[1] {
        *hit_delay_us = 2_000_000;
    }
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 1_000_000,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    let mut ally_hit = req.actions[0].clone();
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index,
        ..
    } = &mut ally_hit
    {
        *timestamp_us = 1_600_000;
        *actor_index = 1;
    }
    req.actions.push(ally_hit);
    req.action_metadata.retain(|i, _| *i < 2);
    req.action_metadata
        .insert(3, req.action_metadata[&0].clone());
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let hits: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["ability_id"] != "GA_Sagiri_QTE")
        .collect();
    assert_eq!(hits[0]["rounded_damage"], 1000);
    assert_eq!(hits[1]["actor_index"], 1);
    assert_eq!(hits[1]["rounded_damage"], 1000);
    assert_eq!(hits[2]["actor_index"], 0);
    assert_eq!(hits[2]["rounded_damage"], 1020);
}
