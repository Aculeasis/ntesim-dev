use super::*;
use serde_json::Value;

fn request_at(times: &[u64]) -> GameScenarioRequest {
    let mut req = research_request();
    req.stochastic_crit = None;
    req.actors[0].weapon_id = "weapon:fork_MotorCandy".into();
    req.actors[0].gear_procs = vec![GearProcConfig::MotorCandyAtk {
        atk_per_stack: Ratio {
            numerator: 1,
            denominator: 20,
        },
        interval_us: 1_000_000,
    }];
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.targets[0].resistance = Ratio::zero();
    let action = req.actions[0].clone();
    let metadata = req.action_metadata[&0].clone();
    req.actions.clear();
    req.action_metadata.clear();
    for (index, time) in times.iter().enumerate() {
        let mut action = action.clone();
        if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut action {
            *timestamp_us = *time;
        }
        req.actions.push(action);
        req.action_metadata.insert(index, metadata.clone());
    }
    req
}

fn add_ally(req: &mut GameScenarioRequest) {
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    req.actors.push(ally);
}

fn prepend_switches(req: &mut GameScenarioRequest, switches: &[(u64, usize)]) {
    let old = std::mem::take(&mut req.actions);
    let metadata = std::mem::take(&mut req.action_metadata);
    req.actions = switches
        .iter()
        .map(
            |(timestamp_us, target_actor_index)| GameActionCommand::SwitchActor {
                timestamp_us: *timestamp_us,
                target_actor_index: *target_actor_index,
                cooldown_us: 0,
            },
        )
        .collect();
    let offset = req.actions.len();
    req.actions.extend(old);
    req.action_metadata = metadata
        .into_iter()
        .map(|(index, value)| (index + offset, value))
        .collect();
    let metadata = std::mem::take(&mut req.action_metadata);
    let mut entries = std::mem::take(&mut req.actions)
        .into_iter()
        .enumerate()
        .map(|(index, command)| (command, metadata.get(&index).cloned()))
        .collect::<Vec<_>>();
    entries.sort_by_key(|(command, _)| command.timestamp_us());
    for (index, (command, metadata)) in entries.into_iter().enumerate() {
        req.actions.push(command);
        if let Some(metadata) = metadata {
            req.action_metadata.insert(index, metadata);
        }
    }
}

fn damages(report: &Value) -> Vec<u64> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|hit| hit["rounded_damage"].as_u64().unwrap())
        .collect()
}

#[test]
fn motor_ticks_at_full_intervals_before_hits_and_stops_at_cap() {
    let req = request_at(&[0, 999_999, 1_000_000, 4_999_999, 5_000_000, 12_000_000]);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // (1000 total ATK + 1000 base ATK * .05 * stacks) * .5 DEF.
    assert_eq!(damages(&report), vec![500, 500, 525, 600, 625, 625]);
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["event_kind"] == "motor_candy_atk_stack")
            .count(),
        5
    );
    let instances = crate::simulation::tests::support::final_buffs(&report);
    let instance = &instances[0];
    assert_eq!(instance["stacks"], 5);
    assert_eq!(instance["max_stacks"], 5);
    assert_eq!(instance["duration"]["kind"], "permanent");
    assert_eq!(instance["source_lifetime"], "while_alive");
}

#[test]
fn motor_passive_only_and_short_runs_keep_horizon_state_and_assumptions() {
    for (horizon, stacks) in [(500_000, 0), (2_500_000, 2), (20_000_000, 5)] {
        let mut req = request_at(&[]);
        req.relative_duration_us = horizon;
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["final_buff_state"]["time_us"], horizon);
        assert!(
            report["action_metadata_ledger"]
                .as_array()
                .is_none_or(|entries| entries.is_empty())
        );
        assert_eq!(
            report["resolved_model_assumptions"]
                [crate::weapons::motor_candy::HYPOTHESIS_ID.as_str()],
            crate::weapons::motor_candy::HYPOTHESIS_ID.description()
        );
        let instances = report["final_buff_state"]["instances"].as_array().unwrap();
        if stacks == 0 {
            assert!(instances.is_empty());
        } else {
            assert_eq!(instances[0]["stacks"], stacks);
        }
    }
}

#[test]
fn motor_real_switch_resets_and_stale_timers_cannot_buff_a_reentered_wearer() {
    let mut req = request_at(&[1_100_000, 1_900_000, 2_900_000]);
    add_ally(&mut req);
    prepend_switches(&mut req, &[(600_000, 1), (900_000, 0)]);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(damages(&report), vec![500, 525, 550]);
    let instances = crate::simulation::tests::support::final_buffs(&report);
    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0]["owner_actor"], 0);
}

#[test]
fn motor_departure_removes_earned_stacks_and_a_second_cycle_starts_from_zero() {
    let mut req = request_at(&[1_100_000, 4_500_000, 5_000_000]);
    add_ally(&mut req);
    if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut req.actions[0] {
        *hit_delay_us = 2_000_000;
    }
    prepend_switches(&mut req, &[(1_500_000, 1), (4_000_000, 0)]);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(damages(&report), vec![500, 500, 525]);
    let instances = crate::simulation::tests::support::final_buffs(&report);
    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0]["owner_actor"], 0);
}

#[test]
fn motor_initially_off_field_wearer_starts_its_clock_only_on_entry() {
    let mut req = request_at(&[2_999_999, 3_000_000]);
    add_ally(&mut req);
    req.active_actor_index = 1;
    req.actors[1].gear_procs.clear();
    req.actors[1].weapon_id.clear();
    prepend_switches(&mut req, &[(2_000_000, 0)]);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(damages(&report), vec![500, 525]);
}

#[test]
fn motor_noop_and_rejected_switches_preserve_the_field_clock_and_stacks() {
    let mut req = request_at(&[0, 2_000_000]);
    prepend_switches(&mut req, &[(1_500_000, 0)]);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(damages(&report), vec![500, 550]);
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["event_kind"] == "motor_candy_reset")
    );
    let mut req = request_at(&[1_100_000]);
    add_ally(&mut req);
    if let GameActionCommand::InvokeAbility { recovery_us, .. } = &mut req.actions[0] {
        *recovery_us = 2_000_000;
    }
    prepend_switches(&mut req, &[(1_500_000, 1)]);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert_eq!(report["final_buff_state"]["time_us"], 1_500_000);
    assert_eq!(report["final_buff_state"]["instances"][0]["stacks"], 1);
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["event_kind"] == "motor_candy_reset")
    );
}

#[test]
fn motor_departure_precedes_equal_time_ticks_and_delayed_hits_use_current_stacks() {
    let mut req = request_at(&[0]);
    add_ally(&mut req);
    if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut req.actions[0] {
        *hit_delay_us = 2_000_000;
    }
    prepend_switches(&mut req, &[(1_000_000, 1)]);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(damages(&report), vec![500]);
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["event_kind"] == "motor_candy_atk_stack" && e["time_us"] == 1_000_000)
    );
    assert_eq!(
        crate::simulation::tests::support::final_buffs(&report)[0]["owner_actor"],
        1
    );
}

#[test]
fn motor_rejects_invalid_parameters_conflicts_and_preserves_event_limit_state() {
    let mut req = request_at(&[]);
    let mut invalid = req.clone();
    invalid.actors[0].gear_procs = vec![GearProcConfig::MotorCandyAtk {
        atk_per_stack: Ratio::one(),
        interval_us: 0,
    }];
    assert!(
        run_game_scenario(&invalid, &ExecutionLimits::default())
            .unwrap_err()
            .contains("positive interval")
    );
    invalid = req.clone();
    let extra = invalid.actors[0].gear_procs[0].clone();
    invalid.actors[0].gear_procs.push(extra);
    assert!(
        run_game_scenario(&invalid, &ExecutionLimits::default())
            .unwrap_err()
            .contains("duplicates")
    );
    invalid = req.clone();
    invalid.actors[0].weapon_id.clear();
    assert!(
        run_game_scenario(&invalid, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires equipped")
    );
    invalid = req.clone();
    invalid.model_assumptions.insert(
        crate::weapons::motor_candy::HYPOTHESIS_ID.into(),
        "different".into(),
    );
    assert!(
        run_game_scenario(&invalid, &ExecutionLimits::default())
            .unwrap_err()
            .contains("conflicting reached")
    );
    req.relative_duration_us = 10_000_000;
    let limits = ExecutionLimits {
        max_events: 2,
        ..ExecutionLimits::default()
    };
    let report = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report["success"], false);
    assert_eq!(report["execution_completed"], false);
    assert_eq!(report["final_buff_state"]["instances"][0]["stacks"], 2);
    assert_eq!(report["final_buff_state"]["time_us"], 2_000_000);
}
