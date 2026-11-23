use super::*;
use crate::simulation::GameActorInterruptEvent;
use crate::simulation::tests::support::formula_damage_total;

fn scenario(time: u64) -> GameScenarioRequest {
    let mut req = phase2_request();
    let id = crate::HypothesisId::BaicangForcedInterruptV1;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.interrupt_events = vec![GameActorInterruptEvent {
        timestamp_us: time,
        actor_index: 0,
        armor_damage: None,
    }];
    req
}

fn skill(time: u64) -> GameActionCommand {
    make_action(
        time,
        0,
        "GA_Cang_Skill",
        "GE_Player_Cang_Skill_Damage",
        0,
        14_000_000,
    )
}

#[test]
fn pre_actor_skill_interrupt_cancels_damage_and_uses_three_second_cancel_cooldown() {
    for (next, complete) in [(3_009_999, false), (3_010_000, true)] {
        let mut req = scenario(10_000);
        req.actions = vec![skill(0), skill(next)];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["execution_completed"], complete);
        assert_eq!(report["hits_settled"], usize::from(complete));
        let receipt = &report["interrupt_receipts"][0];
        assert_eq!(receipt["interrupted_command_index"], 0);
        assert_eq!(receipt["cancelled_hit_deliveries"], 1);
        assert_eq!(receipt["skill_cooldown_before_us"], 14_000_000);
        assert_eq!(receipt["skill_cooldown_after_us"], 3_010_000);
        assert_eq!(receipt["recovery_after_us"], 10_000);
        assert_eq!(report["final_actor_states"][0]["hp"], 10000);
    }
}

#[test]
fn an_emitted_skill_actor_survives_interrupt_and_keeps_its_full_cooldown() {
    for time in [18_070, 100_000] {
        let mut req = scenario(time);
        req.actions = vec![skill(0), normal(time + 1), skill(13_999_999)];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["execution_completed"], false);
        assert_eq!(report["hits_settled"], 2);
        assert_eq!(
            report["interrupt_receipts"][0]["cancelled_hit_deliveries"],
            0
        );
        assert_eq!(
            report["interrupt_receipts"][0]["skill_cooldown_after_us"],
            14_000_000
        );
        assert_eq!(
            report["rejected_action"]["reason"],
            json!({"kind":"cooldown","until_us":14_000_000})
        );
        assert!(
            report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .any(|hit| hit["time_us"] == 668_070)
        );
    }
}

#[test]
fn derived_skill_partial_emissions_keep_one_word_and_the_single_committed_explosion() {
    for (time, hits, removed, granted, numerator, denominator, hp) in [
        (600_000, 0, 8, 0, 0, 1, 10000),
        (716_575, 4, 4, 3, 87, 250, 9700),
        (716_581, 6, 2, 5, 11, 500, 9700),
        (716_683, 8, 0, 6, 87, 125, 9700),
    ] {
        let mut req = scenario(time);
        req.actions.clear();
        dodge(&mut req);
        req.actions.push(skill(500_000));
        let report = run(&req);
        assert_eq!(report["hits_settled"], hits);
        assert_eq!(
            report["interrupt_receipts"][0]["cancelled_hit_deliveries"],
            removed
        );
        assert_eq!(report["final_actor_states"][0]["hp"], hp);
        assert_eq!(report["final_actor_states"][0]["granted_energy"], granted);
        if numerator == 0 {
            assert!(report["final_actor_states"][0]["charge_remainder"].is_null());
        } else {
            assert_eq!(
                report["final_actor_states"][0]["charge_remainder"],
                json!(format!("{}/{}", numerator, denominator))
            );
        }
        assert_eq!(report["combat_stamina"]["total_spent"], json!("15/1"));
    }
}

#[test]
fn heavy_interruption_distinguishes_unemitted_actors_from_spawned_projectiles() {
    for (time, hits, removed, damage) in [(450_000, 1, 3, 396), (500_000, 4, 0, 792)] {
        let mut req = scenario(time);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Cang_Melee",
            "GE_Player_Cang_Branch1_Damage",
            0,
            0,
        )];
        let report = run(&req);
        assert_eq!(report["hits_settled"], hits);
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            json!(format!("{}/{}", damage, 1))
        );
        assert_eq!(
            report["interrupt_receipts"][0]["cancelled_hit_deliveries"],
            removed
        );
    }
    let mut req = scenario(2_160_000);
    req.actions = vec![
        normal(0),
        normal(1_000_000),
        make_action(
            2_000_000,
            0,
            "GA_Cang_Melee",
            "GE_Player_Cang_Branch2_Damage",
            0,
            0,
        ),
    ];
    let report = run(&req);
    assert_eq!(report["hits_settled"], 3); // two Normals, then only the already emitted first missile
    assert_eq!(
        report["interrupt_receipts"][0]["cancelled_hit_deliveries"],
        3
    );
    assert_eq!(report["hit_receipts"][2]["time_us"], 2_600_174);
    assert_eq!(report["final_actor_states"][0]["hp"], 9700); // its connected explosion is the third input
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 4);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        json!("133/500")
    );
}

#[test]
fn normal_train_interrupt_preserves_connected_input_and_cancels_only_future_slots() {
    let mut req = scenario(3_700_000);
    req.actions = vec![
        normal(0),
        normal(1_000_000),
        normal(2_000_000),
        normal(3_300_000),
    ];
    let report = run(&req);
    assert_eq!(report["hits_settled"], 6);
    assert_eq!(
        report["interrupt_receipts"][0]["cancelled_hit_deliveries"],
        4
    );
    assert_eq!(report["final_actor_states"][0]["hp"], 9700);
}

#[test]
fn burst_field_marker_and_equal_time_delivery_determine_what_survives_without_energy_refunds() {
    for (time, hits, field_cancelled, removed) in [
        (2_000_000, 0, true, 1),
        (2_664_744, 6, false, 1),
        (2_665_367, 7, false, 0),
    ] {
        let mut req = scenario(time);
        req.actors[0].initial_energy = 120;
        req.actors[0].energy_capacity = 120;
        req.actions = vec![burst(0)];
        let report = run(&req);
        assert_eq!(report["hits_settled"], hits);
        assert_eq!(
            report["interrupt_receipts"][0]["field_cancelled"],
            field_cancelled
        );
        assert_eq!(
            report["interrupt_receipts"][0]["cancelled_hit_deliveries"],
            removed
        );
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 120);
    }
}

#[test]
fn repeated_interrupt_is_idempotent_and_ordinary_same_time_readiness_still_precedes_it() {
    let mut req = scenario(10_000);
    req.interrupt_events.push(req.interrupt_events[0].clone());
    req.actions = vec![skill(0), normal(10_001)];
    let report = run(&req);
    assert_eq!(
        report["interrupt_receipts"][1]["interrupted_command_index"],
        serde_json::Value::Null
    );
    assert_eq!(
        report["interrupt_receipts"][1]["skill_cooldown_after_us"],
        serde_json::Value::Null
    );
    req.actions[1] = normal(10_000);
    let failed = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(failed["execution_completed"], false);
    assert!(failed["interrupt_receipts"].is_null());
}

#[test]
fn interruption_profiles_reject_unbound_models_owners_indices_and_work_limits() {
    let req = scenario(10_000);
    let mut invalid = req.clone();
    invalid
        .model_assumptions
        .remove(crate::HypothesisId::BaicangForcedInterruptV1.as_str());
    assert!(run_game_scenario(&invalid, &ExecutionLimits::default()).is_err());
    let mut invalid = req.clone();
    invalid.interrupt_events[0].actor_index = 1;
    assert!(run_game_scenario(&invalid, &ExecutionLimits::default()).is_err());
    let mut invalid = req.clone();
    invalid.interrupt_events[0].timestamp_us = req.absolute_limit_us + 1;
    assert!(run_game_scenario(&invalid, &ExecutionLimits::default()).is_err());
    for (owner, message) in [
        (
            "character:1019",
            "Mint interruption requires ordinary_clock_v1",
        ),
        ("character:1008", "unsupported owner interruption"),
    ] {
        let mut invalid = req.clone();
        invalid.actors[0].character_id = owner.into();
        invalid.actions.clear();
        let Err(error) = run_game_scenario(&invalid, &ExecutionLimits::default()) else {
            panic!("unbound interruption must reject");
        };
        assert!(error.contains(message), "{owner}: {error}");
    }
    let limits = ExecutionLimits {
        max_events: 1,
        ..Default::default()
    };
    assert!(
        run_game_scenario(&req, &limits)
            .unwrap_err()
            .contains("event count")
    );
}

#[test]
fn two_interrupted_cycles_preserve_cooldown_reentry_words_and_fractional_resources() {
    let mut req = scenario(10_000);
    req.actors[0].initial_energy = 0;
    req.interrupt_events.push(GameActorInterruptEvent {
        timestamp_us: 20_010_000,
        actor_index: 0,
        armor_damage: None,
    });
    req.actions.clear();
    for base in [0, 20_000_000] {
        req.actions.extend([
            skill(base),
            skill(base + 3_010_000),
            normal(base + 4_000_000),
            normal(base + 5_000_000),
        ]);
    }
    let report = run(&req);
    assert_eq!(report["hits_settled"], 6);
    assert_eq!(
        serde_json::json!(formula_damage_total(&report)),
        json!("12588/5")
    );
    let actor = &report["final_actor_states"][0];
    assert_eq!(actor["hp"], 9700);
    assert_eq!(actor["granted_energy"], 15);
    assert_eq!(actor["charge_remainder"], json!("83/250"));
    assert_eq!(actor["cycle"]["granted"], 25);
    assert_eq!(actor["cycle"]["carry"], json!("227/500"));
}

#[test]
fn emission_is_independent_of_damage_and_recipient_count_never_duplicates_resources() {
    let mut req = scenario(100_000);
    req.actions = vec![skill(0)];
    req.actors[0].static_stats.atk = Ratio::zero();
    req.actors[0].static_stats.base_atk = 0;
    let report = run(&req);
    assert_eq!(
        report["interrupt_receipts"][0]["skill_cooldown_after_us"],
        14_000_000
    );
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 0);
    let mut req = scenario(100_000);
    req.actions = vec![skill(0)];
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    let report = run(&req);
    assert_eq!(report["hits_settled"], 2);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 3);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        json!("2/5")
    );
    req.interrupt_events[0].timestamp_us = 10_000;
    assert_eq!(
        run(&req)["interrupt_receipts"][0]["cancelled_hit_deliveries"],
        2
    );
}

#[test]
fn activated_words_and_their_owner_healing_survive_interruption_of_the_burst_animation() {
    let mut req = scenario(6_664_744);
    req.actors[0].initial_energy = 120;
    req.actors[0].energy_capacity = 120;
    req.actions = vec![
        normal(0),
        normal(1_000_000),
        normal(2_000_000),
        burst(4_000_000),
    ];
    let report = run(&req);
    assert_eq!(report["hits_settled"], 12); // 3 Normals + 3 Bless packets + 6 domain ticks
    assert_eq!(
        report["interrupt_receipts"][0]["cancelled_hit_deliveries"],
        1
    );
    assert_eq!(report["final_actor_states"][0]["hp"], 10000);
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "baicang_word_heal")
            .count(),
        1
    );
}
