use super::*;
use crate::simulation::{GameActionCommand, GameTargetBreakProfileConfig, run_game_scenario};
use serde_json::json;

fn ordinary_hit(report: &serde_json::Value, index: usize) -> &serde_json::Value {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit.get("break_contribution").is_none())
        .nth(index)
        .unwrap()
}

fn unbal_request(value: Ratio) -> crate::simulation::GameScenarioRequest {
    let mut request = super::support::research_request();
    request.stochastic_crit = None;
    request.relative_duration_us = 2_000_200;
    request.targets[0].hp_max = 10_000_000;
    request.targets[0].hp_current = 10_000_000;
    request.targets[0].defense = 0;
    request.targets[0].resistance = Ratio::zero();
    request.targets[0].break_profile = Some(GameTargetBreakProfileConfig {
        capacity: Ratio::one(),
        current_gauge: Ratio {
            numerator: 1,
            denominator: 2,
        },
        broken_duration_us: 2_000_000,
        discord_fraction: None,
    });
    request.actions.truncate(4);
    for (action, timestamp_us) in request
        .actions
        .iter_mut()
        .zip([0, 100, 1_000_000, 2_000_110])
    {
        match action {
            GameActionCommand::InvokeAbility {
                timestamp_us: action_time,
                ..
            } => *action_time = timestamp_us,
            _ => unreachable!(),
        }
    }
    let mut metadata = request.action_metadata[&0].clone();
    metadata.unbal_value = Some(value);
    request.action_metadata = (0..4).map(|index| (index, metadata.clone())).collect();
    request
}

fn rose_unbal_request(relative_duration_us: u64) -> crate::simulation::GameScenarioRequest {
    let mut request = unbal_request(Ratio {
        numerator: 1,
        denominator: 3,
    });
    request.relative_duration_us = relative_duration_us;
    request.actors[0].weapon_id = "weapon:fork_Rose".into();
    request.actors[0].gear_procs = vec![crate::simulation::GearProcConfig::RoseThorn {
        atk_bonus: Ratio {
            numerator: 7,
            denominator: 50,
        },
        crit_damage_per_stack: Ratio {
            numerator: 3,
            denominator: 50,
        },
        duration_us: 3_000_000,
        broken_extension_us: 3_000_000,
    }];
    request
}

#[test]
fn incoming_break_efficiency_multiplies_outgoing_once_and_is_target_scoped() {
    use crate::simulation::{ExactFraction, state::GameSimulatorState};
    let mut request = unbal_request(Ratio::one());
    request.actors[0].static_stats.break_efficiency_multiplier = Ratio {
        numerator: 3,
        denominator: 2,
    };
    for (incoming, expected) in [((0, 1), (1, 2)), ((1, 4), (3, 8)), ((1, 1), (0, 1))] {
        let mut state = GameSimulatorState::new(&request);
        state.targets.push(state.targets[0].clone());
        state.targets[0]
            .break_state
            .as_mut()
            .unwrap()
            .incoming_efficiency_multiplier = ExactFraction::new(incoming.0, incoming.1).unwrap();
        let mut trace = Vec::new();
        crate::simulation::break_state::apply_unbalance(
            &request,
            &mut state,
            &mut Default::default(),
            &mut 0,
            &mut trace,
            0,
            0,
            ExactFraction::new(1, 3).unwrap(),
        )
        .unwrap();
        // 1/2 - (1/3 * 3/2 * 1/4) = 3/8. Zero is immunity,
        // not neutral; one removes exactly 1/2 and transitions once.
        assert_eq!(
            state.targets[0].break_state.as_ref().unwrap().gauge,
            ExactFraction::new(expected.0, expected.1).unwrap()
        );
        assert_eq!(
            state.targets[1].break_state.as_ref().unwrap().gauge,
            ExactFraction::new(1, 2).unwrap()
        );
        assert_eq!(
            trace
                .iter()
                .filter(|event| event.event_kind == "target_broken")
                .count(),
            usize::from(incoming == (1, 1))
        );
        assert_eq!(trace.is_empty(), incoming == (0, 1));
    }
}

#[test]
fn explicit_dummy_break_profile_uses_neutral_incoming_efficiency() {
    use crate::simulation::{ExactFraction, state::GameSimulatorState};
    let mut request = unbal_request(Ratio::one());
    let state = GameSimulatorState::new(&request);
    assert_eq!(
        state.targets[0]
            .break_state
            .as_ref()
            .unwrap()
            .incoming_efficiency_multiplier,
        ExactFraction::new(1, 1).unwrap()
    );
    request.targets[0].break_profile = None;
    assert!(
        GameSimulatorState::new(&request).targets[0]
            .break_state
            .is_none()
    );
}

#[test]
fn static_break_efficiency_scales_exact_gauge_loss_and_zero_is_inert() {
    for (multiplier, gauge) in [
        (
            Ratio {
                numerator: 3,
                denominator: 2,
            },
            json!("0/1"),
        ),
        (Ratio::zero(), json!("1/2")),
    ] {
        let mut request = unbal_request(Ratio {
            numerator: 1,
            denominator: 3,
        });
        request.actors[0].static_stats.break_efficiency_multiplier = multiplier;
        request.relative_duration_us = 1;
        request.actions.truncate(1);
        request.action_metadata.retain(|index, _| *index == 0);
        let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
        // (1/3) * (3/2) = 1/2: exactly the starting gauge, with no rounding.
        assert_eq!(
            report["final_target_states"][0]["break_state"]["gauge"],
            gauge
        );
        let broken = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_kind"] == "target_broken");
        assert_eq!(broken, multiplier.numerator != 0);
    }
}

#[test]
fn omitted_break_efficiency_defaults_to_one_but_explicit_invalid_ratio_rejects() {
    let request = unbal_request(Ratio::one());
    let mut serialized = serde_json::to_value(&request).unwrap();
    serialized["actors"][0]["static_stats"]
        .as_object_mut()
        .unwrap()
        .remove("break_efficiency_multiplier");
    let mut restored: crate::simulation::GameScenarioRequest =
        serde_json::from_value(serialized).unwrap();
    assert_eq!(
        restored.actors[0].static_stats.break_efficiency_multiplier,
        Ratio::one()
    );
    restored.actors[0].static_stats.break_efficiency_multiplier = Ratio {
        numerator: 1,
        denominator: 0,
    };
    assert!(run_game_scenario(&restored, &ExecutionLimits::default()).is_err());
}

#[test]
fn live_break_efficiency_honors_addition_recipients_expiry_and_source_lifetime() {
    use crate::simulation::{ExactFraction, state::GameSimulatorState};
    use ntesim_core::buffs::dispatch::*;
    let request = unbal_request(Ratio::one());
    let base_state = GameSimulatorState::new(&request);
    for (time, active, source_alive, lifetime, expected) in [
        (99, 0, true, BuffSourceLifetime::WhileAlive, (1, 12)),
        (100, 0, true, BuffSourceLifetime::WhileAlive, (1, 3)),
        (99, 1, true, BuffSourceLifetime::WhileAlive, (1, 3)),
        (99, 0, false, BuffSourceLifetime::WhileAlive, (1, 3)),
        (99, 0, false, BuffSourceLifetime::Snapshot, (1, 12)),
    ] {
        let mut state = base_state.clone();
        state.actors.push(state.actors[0].clone());
        state.advance_unpaused_test_clock(time);
        state.active_actor_index = active;
        if !source_alive {
            state.actors[1].hp = 0;
        }
        for (id, amount) in [("first", (1, 4)), ("first", (1, 4)), ("second", (1, 1))] {
            state
                .buffs
                .apply_buff(
                    BuffDefinition {
                        id: id.into(),
                        source: "test.break_efficiency".into(),
                        stacking_scope: BuffStackingScope::PerSource,
                        semantic_tags: Default::default(),
                        source_lifetime: lifetime,
                        recipient: BuffRecipient::ActiveActor,
                        duration_us: Some(100),
                        refresh_policy: RefreshPolicy::Reset,
                        max_stacks: 2,
                        decay_policy: DecayPolicy::AllAtOnce,
                        stat_modifiers: std::collections::BTreeMap::from([(
                            "break_efficiency".into(),
                            Ratio {
                                numerator: amount.0,
                                denominator: amount.1,
                            },
                        )]),
                    },
                    1,
                    0,
                )
                .unwrap();
        }
        crate::simulation::break_state::apply_unbalance(
            &request,
            &mut state,
            &mut Default::default(),
            &mut 0,
            &mut Vec::new(),
            0,
            0,
            ExactFraction::new(1, 6).unwrap(),
        )
        .unwrap();
        // 1/2 - (1/6)*(1 + 1/2 + 1) = 1/12; without bonuses: 1/3.
        assert_eq!(
            state.targets[0].break_state.as_ref().unwrap().gauge,
            ExactFraction::new(expected.0, expected.1).unwrap(),
            "time={time}, active={active}, alive={source_alive}, lifetime={lifetime:?}"
        );
    }
}

#[test]
fn positive_damage_unbal_transitions_target_and_restores_gauge_at_expiry() {
    let request = unbal_request(Ratio {
        numerator: 1,
        denominator: 3,
    });
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(
        report["resolved_model_assumptions"]
            [crate::simulation::break_state::HYPOTHESIS_ID.as_str()],
        crate::simulation::break_state::HYPOTHESIS_ID.description()
    );
    assert_eq!(
        report["final_target_states"][0]["break_state"]["gauge"],
        json!("2/3"),
        "{report}"
    );
    assert_eq!(
        report["final_target_states"][0]["break_state"]["broken_until_us"],
        serde_json::Value::Null
    );
    let trace = report["trace"].as_array().unwrap();
    let kinds = trace
        .iter()
        .map(|event| event["event_kind"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        kinds
            .iter()
            .filter(|kind| **kind == "target_break_gauge_reduced")
            .count(),
        3
    );
    assert!(kinds.contains(&"target_broken"));
    assert!(kinds.contains(&"target_break_recovered"));
    let broken = trace
        .iter()
        .find(|event| event["event_kind"] == "target_broken")
        .unwrap();
    assert_eq!(broken["time_us"], 100);
    let recovered = trace
        .iter()
        .find(|event| event["event_kind"] == "target_break_recovered")
        .unwrap();
    assert_eq!(recovered["time_us"], 2_000_100);
}

#[test]
fn break_transition_publishes_timed_broken_status_to_existing_consumers() {
    let mut request = unbal_request(Ratio {
        numerator: 1,
        denominator: 3,
    });
    request.relative_duration_us = 500;
    request.actions.truncate(2);
    request.action_metadata.retain(|index, _| *index < 2);
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["final_target_states"][0]["break_state"]["gauge"],
        json!("0/1")
    );
    assert_eq!(
        report["final_target_states"][0]["break_state"]["broken_until_us"],
        2_000_100
    );
    let tags = report["final_target_states"][0]["statuses"][0]["semantic_tags"]
        .as_array()
        .unwrap();
    assert!(
        tags.iter()
            .any(|tag| tag == crate::simulation::break_state::BROKEN_STATUS_TAG)
    );
}

#[test]
fn broken_status_activates_film_manich_damage_bonus_on_later_hits() {
    let mut request = unbal_request(Ratio {
        numerator: 1,
        denominator: 3,
    });
    request.relative_duration_us = 1_000_100;
    request.actions.truncate(3);
    request.action_metadata.retain(|index, _| *index < 3);
    request.actors[0].character_id = "character:1021".into();
    request.actors[0].weapon_id = "weapon:fork_jiaojuan".into();
    request.actors[0].weapon_tier = 1;
    request.actors[0].gear_procs = vec![crate::simulation::GearProcConfig::JiaojuanBreak {
        break_intensity_bonus: Ratio {
            numerator: 48,
            denominator: 1,
        },
        broken_damage_bonus: Ratio {
            numerator: 1,
            denominator: 10,
        },
    }];
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["hit_receipts"][0]["non_crit_damage"],
        json!("1000/1")
    );
    assert_eq!(
        report["hit_receipts"][1]["non_crit_damage"],
        json!("1000/1")
    );
    assert_eq!(ordinary_hit(&report, 2)["non_crit_damage"], json!("1100/1"));
}

#[test]
fn missing_profile_and_explicit_zero_unbal_do_not_change_gauge() {
    let mut no_profile = unbal_request(Ratio {
        numerator: 1,
        denominator: 3,
    });
    no_profile.relative_duration_us = 1;
    no_profile.actions.truncate(1);
    no_profile.action_metadata.retain(|index, _| *index == 0);
    no_profile.targets[0].break_profile = None;
    let no_profile_report = run_game_scenario(&no_profile, &ExecutionLimits::default()).unwrap();
    assert!(no_profile_report["final_target_states"][0]["break_state"].is_null());
    assert!(
        no_profile_report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| {
                event["event_kind"] != "target_break_gauge_reduced"
                    && event["event_kind"] != "target_broken"
            })
    );
    assert!(
        no_profile_report["resolved_model_assumptions"]
            .get(crate::simulation::break_state::HYPOTHESIS_ID.as_str())
            .is_none()
    );

    let mut zero = unbal_request(Ratio::zero());
    zero.relative_duration_us = 1;
    zero.actions.truncate(1);
    zero.action_metadata.retain(|index, _| *index == 0);
    let zero_report = run_game_scenario(&zero, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        zero_report["final_target_states"][0]["break_state"]["gauge"],
        json!("1/2")
    );
    assert!(
        zero_report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| {
                event["event_kind"] != "target_break_gauge_reduced"
                    && event["event_kind"] != "target_broken"
            })
    );
}

#[test]
fn break_gauge_hypothesis_records_and_validates_acceptance() {
    let mut request = unbal_request(Ratio {
        numerator: 1,
        denominator: 3,
    });
    let mut state = crate::simulation::state::GameSimulatorState::new(&request);
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    crate::simulation::break_state::apply_unbalance(
        &request,
        &mut state,
        &mut queue,
        &mut sequence,
        &mut trace,
        0,
        0,
        crate::simulation::ExactFraction::new(1, 3).unwrap(),
    )
    .unwrap();
    assert_eq!(
        state.execution_model_assumptions[crate::simulation::break_state::HYPOTHESIS_ID.as_str()],
        crate::simulation::break_state::HYPOTHESIS_ID.description(),
    );

    assert_eq!(
        state.targets[0].break_state.as_ref().unwrap().gauge,
        crate::simulation::ExactFraction::new(1, 6).unwrap()
    );

    request.model_assumptions.insert(
        crate::simulation::break_state::HYPOTHESIS_ID.into(),
        "conflicting description".into(),
    );
    let mut conflicting_state = crate::simulation::state::GameSimulatorState::new(&request);
    let error = crate::simulation::break_state::apply_unbalance(
        &request,
        &mut conflicting_state,
        &mut queue,
        &mut sequence,
        &mut trace,
        0,
        0,
        crate::simulation::ExactFraction::new(1, 3).unwrap(),
    )
    .unwrap_err();
    assert!(error.contains("conflicting reached model hypothesis"));
}

#[test]
fn rose_extends_an_existing_break_once_and_invalidates_the_old_recovery() {
    let request = rose_unbal_request(2_000_200);
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true, "{report}");
    assert_eq!(
        report["final_target_states"][0]["break_state"]["broken_until_us"], 5_000_100,
        "t=1_000_000 extension adds 3_000_000 to the original 2_000_100 deadline"
    );
    assert_eq!(
        report["resolved_model_assumptions"]
            [crate::weapons::rose::BREAK_EXTENSION_HYPOTHESIS_ID.as_str()],
        crate::weapons::rose::BREAK_EXTENSION_HYPOTHESIS_ID.description()
    );
    let trace = report["trace"].as_array().unwrap();
    let extensions = trace
        .iter()
        .filter(|event| event["event_kind"] == "target_broken_duration_extended")
        .collect::<Vec<_>>();
    assert_eq!(extensions.len(), 1, "{report}");
    assert_eq!(extensions[0]["time_us"], 1_000_000);
    assert_eq!(extensions[0]["data"]["cooldown_until_us"], 2_000_100);
    assert_eq!(extensions[0]["data"]["expires_at_us"], 5_000_100);
    let broken_status = report["final_target_states"][0]["statuses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|status| status["id"] == crate::simulation::break_state::BROKEN_STATUS_ID)
        .expect("the Broken target tag remains active through the extended deadline");
    assert_eq!(broken_status["expires_at_us"], 5_000_100);
    assert!(
        trace
            .iter()
            .all(|event| event["event_kind"] != "target_break_recovered"),
        "the queued recovery at the original deadline must be stale: {report}"
    );
    assert_eq!(
        support::receipt_assumption(
            &report,
            ordinary_hit(&report, 2),
            crate::weapons::rose::BREAK_EXTENSION_HYPOTHESIS_ID.as_str()
        ),
        Some(crate::weapons::rose::BREAK_EXTENSION_HYPOTHESIS_ID.description())
    );
}

#[test]
fn rose_recovery_runs_at_the_extended_deadline() {
    let request = rose_unbal_request(5_000_200);
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true, "{report}");
    assert_eq!(
        report["final_target_states"][0]["break_state"]["gauge"],
        json!("1/1")
    );
    assert_eq!(
        report["final_target_states"][0]["break_state"]["broken_until_us"],
        serde_json::Value::Null
    );
    let recoveries = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "target_break_recovered")
        .collect::<Vec<_>>();
    assert_eq!(recoveries.len(), 1, "{report}");
    assert_eq!(recoveries[0]["time_us"], 5_000_100);
}

#[test]
fn rose_extension_hypothesis_records_and_validates() {
    let mut request = rose_unbal_request(2_000_200);
    let mut assumptions = std::collections::BTreeMap::new();
    crate::runtime_api::record_hypothesis(
        &request,
        &mut assumptions,
        crate::weapons::rose::BREAK_EXTENSION_HYPOTHESIS_ID,
        crate::weapons::rose::BREAK_EXTENSION_HYPOTHESIS_ID.description(),
    )
    .unwrap();
    assert_eq!(
        assumptions[crate::weapons::rose::BREAK_EXTENSION_HYPOTHESIS_ID.as_str()],
        crate::weapons::rose::BREAK_EXTENSION_HYPOTHESIS_ID.description(),
    );
    request.model_assumptions.insert(
        crate::weapons::rose::BREAK_EXTENSION_HYPOTHESIS_ID.into(),
        "conflicting description".into(),
    );
    let rejected = crate::runtime_api::record_hypothesis(
        &request,
        &mut assumptions,
        crate::weapons::rose::BREAK_EXTENSION_HYPOTHESIS_ID,
        crate::weapons::rose::BREAK_EXTENSION_HYPOTHESIS_ID.description(),
    )
    .unwrap_err();
    assert!(rejected.contains("conflicting reached model hypothesis"));
}

#[test]
fn rose_extension_is_inert_without_an_explicit_break_profile() {
    let mut request = rose_unbal_request(1);
    request.targets[0].break_profile = None;
    request.actions.truncate(1);
    request.action_metadata.retain(|index, _| *index == 0);
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert!(report["final_target_states"][0]["break_state"].is_null());
    assert!(
        report["resolved_model_assumptions"]
            .get(crate::weapons::rose::BREAK_EXTENSION_HYPOTHESIS_ID.as_str())
            .is_none()
    );
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| { event["event_kind"] != "target_broken_duration_extended" })
    );
}
