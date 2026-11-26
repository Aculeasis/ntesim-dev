use super::*;
use serde_json::json;

#[test]
fn qte_projection_uses_its_own_purchase_and_owner_resonance() {
    use crate::simulation::talents::qte_coordinate;

    let mut actor = talent_request().actors.remove(0);
    actor.purchased_skill_level = 2;
    actor.purchased_qte_level = Some(8);
    actor.awakening = ntesim_data::AwakeningSelection::selected(vec![2, 4, 6]).unwrap();
    actor.character_id = "character:1021".into();
    assert_eq!(qte_coordinate(&actor).unwrap(), 8);
    actor.purchased_qte_level = Some(10);
    assert_eq!(qte_coordinate(&actor).unwrap(), 10);
    actor.purchased_qte_level = Some(11);
    assert!(qte_coordinate(&actor).unwrap_err().contains("range"));
    actor.purchased_qte_level = Some(8);
    actor.character_id = "character:1003".into();
    assert_eq!(qte_coordinate(&actor).unwrap(), 7);
    actor.character_level = 50;
    actor.character_max_level = Some(50);
    assert!(qte_coordinate(&actor).unwrap_err().contains("prerequisite"));
    actor.purchased_qte_level = None;
    assert!(qte_coordinate(&actor).unwrap_err().contains("explicit"));
}

fn talent_request() -> GameScenarioRequest {
    let mut request = sample_s1_request();
    let actor = &mut request.actors[0];
    actor.talent_levels = Some([1, 2, 3]);
    actor.character_max_level = Some(80);
    actor.gear_procs.clear();
    actor.static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    actor.static_stats.elemental_bonus = Ratio::zero();
    request.targets[0].defense = 0;
    request.targets[0].resistance = Ratio::zero();
    for command in &mut request.actions {
        if let GameActionCommand::InvokeAbility {
            array_index,
            atk_rate,
            ..
        } = command
        {
            *array_index = 999;
            *atk_rate = Ratio {
                numerator: 999,
                denominator: 1,
            };
        }
    }
    request
}

#[test]
fn prepared_talents_keep_receipts_and_event_limits_with_fresh_state() {
    let mut request = talent_request();
    request.stochastic_crit = research_request().stochastic_crit;
    let limits = ExecutionLimits::default();
    let prepared = PreparedGameScenario::new(&request, &limits).unwrap();
    for seed in [42, 43, 0, u64::MAX, 42] {
        request.seed = seed;
        let report = prepared.run(seed).unwrap();
        assert_eq!(
            serde_json::to_value(&report).unwrap(),
            run_game_scenario(&request, &limits).unwrap()
        );
        assert_eq!(report.seed, seed);
        assert_eq!(report.talent_projection_receipts.len(), 3);
    }
    let limits = ExecutionLimits {
        max_events: 3,
        ..limits
    };
    let limited = PreparedGameScenario::new(&request, &limits).unwrap();
    let report = limited.run(42).unwrap();
    assert!(!report.execution_completed);
    assert_eq!(report.total_events_processed, 3);
    assert_eq!(
        report.halt_diagnostic.as_deref(),
        Some(
            "Simulation event budget exhausted: limit 3 events processed. The run stopped before settling the next event."
        )
    );
}

#[test]
fn borrowed_reports_keep_full_talent_json_and_owned_round_trip() {
    for mut request in [talent_request(), research_request()] {
        for limit in [
            ExecutionLimits::default(),
            ExecutionLimits {
                max_events: request.actions.len() as u64,
                ..ExecutionLimits::default()
            },
        ] {
            request.seed = u64::MAX;
            let prepared = PreparedGameScenario::new(&request, &limit).unwrap();
            let borrowed = prepared.run_borrowed(request.seed).unwrap();
            let json = serde_json::to_vec(&borrowed).unwrap();
            let decoded: GameScenarioReport = serde_json::from_slice(&json).unwrap();
            let owned = prepared.run(request.seed).unwrap();
            assert_eq!(
                serde_json::to_value(&decoded).unwrap(),
                serde_json::to_value(&owned).unwrap()
            );
            assert_eq!(
                serde_json::to_value(&borrowed).unwrap(),
                serde_json::to_value(&owned).unwrap()
            );
            assert_eq!(
                serde_json::to_value(borrowed.into_owned()).unwrap(),
                serde_json::to_value(owned).unwrap()
            );
        }
    }
}

#[test]
fn independent_talents_select_native_coefficients_and_exact_damage() {
    let request = talent_request();
    let before = serde_json::to_value(&request).unwrap();
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    // Frozen Mint: Melee L1=.104+.231, two Skill L2=.972,
    // Burst L3=1.547+5*.686+4.357. ATK1000, no bonuses/mitigation/critical rolls.
    assert_eq!(report["hits_settled"], 11);
    for (hit, expected) in report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .zip([104, 231, 972, 972, 1547, 686, 686, 686, 686, 686, 4357])
    {
        assert_eq!(hit["damage"], json!(format!("{}/{}", expected, 1)));
    }
    assert_eq!(report["total_damage_exact"], json!("11613/1"));
    let receipts = report["talent_projection_receipts"].as_array().unwrap();
    for (index, receipt) in receipts.iter().enumerate() {
        assert_eq!(receipt["compiled_array_index"], index);
        assert_eq!(receipt["skill_level"]["effective_level"], index + 1);
        assert!(receipt["native_damage_report"]["evidence"].is_array());
    }
    assert_eq!(serde_json::to_value(&request).unwrap(), before);
    for index in 0..3 {
        let mut changed = request.clone();
        changed.actors[0].talent_levels.as_mut().unwrap()[index] = 10;
        let other = run_game_scenario(&changed, &ExecutionLimits::default()).unwrap();
        for hit in 0..11 {
            if report["hit_receipts"][hit]["command_index"] == index {
                assert_ne!(
                    other["hit_receipts"][hit]["damage"],
                    report["hit_receipts"][hit]["damage"]
                );
            } else {
                assert_eq!(
                    other["hit_receipts"][hit]["damage"],
                    report["hit_receipts"][hit]["damage"]
                );
            }
        }
    }
}

#[test]
fn talent_resonance_boundary_and_progression_are_checked() {
    for awakening in [0, 2, 3, 6] {
        let mut request = talent_request();
        request.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
        for (index, receipt) in report["talent_projection_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            assert_eq!(
                receipt["compiled_array_index"],
                index + usize::from(awakening >= 3)
            );
        }
    }
    for invalid in [[0, 2, 3], [1, 11, 3]] {
        let mut request = talent_request();
        request.actors[0].talent_levels = Some(invalid);
        request.actions.clear(); // Invalid unused talents must also fail.
        assert!(
            run_game_scenario(&request, &ExecutionLimits::default())
                .unwrap_err()
                .contains("purchased skill level")
        );
    }
    let mut request = talent_request();
    request.actors[0].character_level = 20;
    request.actors[0].character_max_level = Some(20);
    request.actors[0].talent_levels = Some([10; 3]);
    assert!(
        run_game_scenario(&request, &ExecutionLimits::default())
            .unwrap_err()
            .contains("prerequisites")
    );
    request = talent_request();
    request.model_assumptions.clear();
    assert_eq!(
        run_game_scenario(&request, &ExecutionLimits::default()).unwrap()["execution_completed"],
        true
    );
}

#[test]
fn frozen_talent_inputs_cover_every_released_character() {
    let released_characters = ntesim_data::character::released_ids();
    let mut count = 0;
    for id in released_characters {
        let mut request = talent_request();
        request.actions.clear();
        request.actors[0].character_id = id.clone();
        request.actors[0].talent_levels = Some([10; 3]);
        request.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
        super::super::talents::project(&request).unwrap_or_else(|error| panic!("{id}: {error}"));
        count += 1;
    }
    assert_eq!(count, 25);
}
