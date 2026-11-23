use super::*;
use serde_json::{Value, json};

const SOURCE: &str = include_str!("../../../../../examples/stamina-dodge-cycles.nte");

#[test]
fn stamina_nested_owner_suppression_preserves_prior_gate_and_failed_pair_payment() {
    let profile = CombatStaminaProfile {
        capacity: Ratio {
            numerator: 40,
            denominator: 1,
        },
        initial: Ratio {
            numerator: 5,
            denominator: 1,
        },
        use_efficiency: Ratio {
            numerator: 3,
            denominator: 2,
        },
        recovery_rate: Ratio {
            numerator: 10,
            denominator: 1,
        },
    };
    let mut pool = CombatStaminaReport::new(&profile);
    pool.suppress_recovery(0, 0, 0, true).unwrap();
    pool.suppress_recovery(1, 1, 0, true).unwrap();
    assert!(pool.try_spend(0, 0, 1_000_000, 2).unwrap());
    assert!(!pool.try_spend(0, 0, 2_000_000, 2).unwrap());
    assert_eq!(pool.recovery_ready_at_us, 3_000_000);
    assert_eq!(pool.spends.len(), 1);
    pool.suppress_recovery(0, 0, 4_000_000, false).unwrap();
    pool.advance(5_000_000).unwrap();
    assert_eq!(pool.current, ExactFraction::new(2, 1).unwrap());
    pool.suppress_recovery(1, 1, 5_000_000, false).unwrap();
    pool.advance(5_500_000).unwrap();
    assert_eq!(pool.current, ExactFraction::new(7, 1).unwrap());
    assert_eq!(pool.total_recovered, ExactFraction::new(5, 1).unwrap());
    pool.verify().unwrap();
}

fn run(source: &str) -> Value {
    let request = crate::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    run_game_scenario(&request, &ExecutionLimits::default()).unwrap()
}

#[test]
fn combat_stamina_recovers_exactly_after_refreshed_pause_and_follows_party_switches() {
    let report = run(SOURCE);
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    let stamina = &report["combat_stamina"];
    // Recovery at5/s spans [2.9,6] and [8,15]: (3.1+7)*5=50.5.
    // The continuing pool finishes at30-45+50.5=35.5 without a reset.
    assert_eq!(stamina["total_spent"], json!("45/1"));
    assert_eq!(stamina["total_recovered"], json!("101/2"));
    assert_eq!(stamina["current"], json!("71/2"));
    assert_eq!(stamina["recovery_ready_at_us"], 8_000_000);
    let spends = stamina["spends"].as_array().unwrap();
    assert_eq!(spends.len(), 3);
    assert_eq!(
        spends
            .iter()
            .map(|r| (
                r["time_us"].as_u64().unwrap(),
                r["actor_index"].as_u64().unwrap()
            ))
            .collect::<Vec<_>>(),
        [(0, 0), (900_000, 0), (6_000_000, 1)]
    );
    assert_eq!(report["player_input_receipts"].as_array().unwrap().len(), 3);
}

#[test]
fn combat_stamina_shortage_and_dash_cooldown_do_not_spend_or_refresh_pause() {
    for (source, kind) in [
        (
            SOURCE.replace("initial_stamina=30", "initial_stamina=20"),
            "stamina",
        ),
        (SOURCE.replace("wait(0.4);", ""), "cooldown"),
    ] {
        let report = run(&source);
        assert_eq!(report["execution_completed"], false);
        assert_eq!(report["rejected_action"]["reason"]["kind"], kind);
        assert_eq!(report["combat_stamina"]["total_spent"], json!("15/1"));
        assert_eq!(report["combat_stamina"]["recovery_ready_at_us"], 2_000_000);
        assert_eq!(
            report["combat_stamina"]["spends"].as_array().unwrap().len(),
            1
        );
    }
}

#[test]
fn combat_stamina_options_preserve_zero_and_reject_incomplete_or_invalid_profiles() {
    for source in [
        SOURCE.replace(" stamina_recovery_rate=5", ""),
        SOURCE.replace("stamina_capacity=40", "stamina_capacity=0"),
        SOURCE.replace("initial_stamina=30", "initial_stamina=41"),
        SOURCE.replace("stamina_use_efficiency=1", "stamina_use_efficiency=0"),
        SOURCE.replace("stamina_recovery_rate=5", "stamina_recovery_rate=-1"),
    ] {
        assert!(crate::dsl::compiler::compile_scenario(&source, Default::default()).is_err());
    }
    let report = run(&SOURCE
        .replace("stamina_recovery_rate=5", "stamina_recovery_rate=0")
        .replace("mint dash;", "mint attack;"));
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["combat_stamina"]["current"], json!("0/1"));
    assert_eq!(report["combat_stamina"]["total_recovered"], json!("0/1"));
}

#[test]
fn combat_stamina_fractional_use_efficiency_and_exact_pause_boundary() {
    let source = SOURCE.replace("stamina_use_efficiency=1", "stamina_use_efficiency=0.1+0.2");
    let mut req = crate::dsl::compiler::compile_scenario(&source, Default::default())
        .unwrap()
        .request;
    req.relative_duration_us = 2_000_000;
    req.actions.truncate(1);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["combat_stamina"]["current"], json!("51/2"));
    assert_eq!(report["combat_stamina"]["total_spent"], json!("9/2"));
    assert_eq!(report["combat_stamina"]["total_recovered"], json!("0/1"));
}

#[test]
fn combat_stamina_sparse_recovery_caps_without_polling_or_extra_events() {
    let short = run(SOURCE);
    let long = run(&SOURCE.replace("duration=15", "duration=600"));
    assert_eq!(long["execution_completed"], true);
    assert_eq!(long["combat_stamina"]["current"], json!("40/1"));
    assert_eq!(long["combat_stamina"]["total_recovered"], json!("55/1"));
    assert_eq!(
        long["total_events_processed"],
        short["total_events_processed"]
    );
    assert_eq!(long["invariants_passed"], true);
}
