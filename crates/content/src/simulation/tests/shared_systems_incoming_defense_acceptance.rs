use super::*;
use serde_json::json;

#[test]
fn shared_systems_incoming_damage_def_and_shares_acceptance() {
    let mut req = sample_s1_request();
    req.relative_duration_us = 5_000_000;
    req.actions.clear();
    req.action_metadata.clear();

    // Actor 0: Adler with DEF 1000 base, +50% percent, +100 flat -> DEF 1600
    let actor = &mut req.actors[0];
    actor.character_id = "character:1033".into();
    actor.slot = 0;
    actor.weapon_id.clear();
    actor.set_id = None;
    actor.gear_procs.clear();
    actor.initial_hp = 10000;
    actor.initial_hp_max = 10000;
    actor.static_stats.hp = 10000;
    actor.static_stats.base_def = Some(1000);
    actor.static_stats.def = 1600;
    actor.static_stats.def_percent = Ratio {
        numerator: 1,
        denominator: 2,
    };
    actor.static_stats.def_flat = Ratio {
        numerator: 100,
        denominator: 1,
    };

    req.targets[0].level = 80;

    for model in [
        crate::HypothesisId::CombatIncomingDamageFixtureV1,
        crate::HypothesisId::CombatIncomingDefenseV1,
    ] {
        req.model_assumptions
            .insert(model.into(), model.description().into());
    }

    // Two incoming events:
    // Event 0: BeforeDefense 1000 raw at t=500ms from dummy level 80
    // Factor: A = 600 + 6 * 80 = 1080. A / (A + DEF) = 1080 / (1080 + 1600) = 1080 / 2680 = 27 / 67.
    // 1000 * 27 / 67 = 402.985... -> rounds to 403.
    // Event 1: ExternallyMitigated 500 raw at t=1000ms (no DEF factor applied) -> delivered = 500.
    req.incoming_damage_events = vec![
        GameActorDamageEvent {
            timestamp_us: 500_000,
            actor_index: 0,
            delivered_damage: 1000,
        },
        GameActorDamageEvent {
            timestamp_us: 1_000_000,
            actor_index: 0,
            delivered_damage: 500,
        },
    ];

    req.incoming_dummy_sources.insert(0, 0);
    req.incoming_damage_basis
        .insert(0, GameIncomingDamageBasis::BeforeDefense);
    // Event 1 has no basis override, defaulting to ExternallyMitigated

    let limits = ExecutionLimits::default();
    let report = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report["status"], "ok");

    let receipts = report["incoming_damage_receipts"].as_array().unwrap();
    assert_eq!(receipts.len(), 2);

    // Event 0 check: BeforeDefense
    assert_eq!(receipts[0]["event_index"], 0);
    assert_eq!(receipts[0]["delivered_damage"], 403);
    assert_eq!(receipts[0]["actual_hp_loss"], 403);
    assert_eq!(
        receipts[0]["defense_mitigation"],
        json!({
            "damage_before_defense": 1000,
            "source_target_index": 0,
            "source_level": 80,
            "recipient_defense": "1600/1"
        })
    );

    // Event 1 check: ExternallyMitigated (no defense mitigation receipt, exact 500 delivered)
    assert_eq!(receipts[1]["event_index"], 1);
    assert_eq!(receipts[1]["delivered_damage"], 500);
    assert_eq!(receipts[1]["actual_hp_loss"], 500);
    assert!(receipts[1].get("defense_mitigation").is_none());

    // Total HP loss: 403 + 500 = 903 -> HP after: 10000 - 903 = 9097
    assert_eq!(report["final_actor_states"][0]["hp"], 9097);

    // Replay determinism check
    let replay = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(
        report["incoming_damage_receipts"],
        replay["incoming_damage_receipts"]
    );
    assert_eq!(report["final_actor_states"], replay["final_actor_states"]);

    // Atomic failure check: invalid dummy source index rejects before execution
    let mut invalid_req = req.clone();
    invalid_req.incoming_dummy_sources.insert(0, 999);
    assert!(run_game_scenario(&invalid_req, &limits).is_err());
}
