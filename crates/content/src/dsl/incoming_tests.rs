use super::compiler::{CompileLimits, compile_scenario};

const PREFIX: &str = "options duration=5; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=100000000 def=0 resist=0;";

#[test]
fn incoming_source_keeps_exact_constants_stable_event_order_and_locations() {
    let source = format!(
        "{PREFIX}\nconst delay=0.1+0.2;\nincoming mint at=1 damage=0;\nincoming mint at=delay damage=10+20;\nincoming mint at=300ms damage=40;\nactive mint; wait(2);"
    );
    let compiled = compile_scenario(&source, CompileLimits::default()).unwrap();
    let events = &compiled.request.incoming_damage_events;
    assert_eq!(
        events
            .iter()
            .map(|e| (e.timestamp_us, e.delivered_damage))
            .collect::<Vec<_>>(),
        [(300000, 30), (300000, 40), (1000000, 0)]
    );
    assert_eq!(
        compiled
            .incoming_damage_locations
            .iter()
            .map(|loc| loc.line)
            .collect::<Vec<_>>(),
        [4, 5, 3]
    );
    let id = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    assert_eq!(
        compiled.request.model_assumptions[id.as_str()],
        id.description()
    );
}

#[test]
fn incoming_source_rejects_invalid_fields_values_recipients_and_placement() {
    for declaration in [
        "incoming mint at=0;",
        "incoming mint damage=0;",
        "incoming mint at=-1 damage=1;",
        "incoming mint at=0.0000001 damage=1;",
        "incoming mint at=3601 damage=1;",
        "incoming mint at=0 damage=-1;",
        "incoming mint at=0 damage=1/2;",
        "incoming mint at=0 damage=true;",
        "incoming mint at=0 damage=9007199254740992;",
        "incoming mint at=0 damage=1 amount=1;",
        "incoming mint at=0 at=1 damage=1;",
        "incoming unknown at=0 damage=1;",
    ] {
        assert!(
            compile_scenario(
                &format!("{PREFIX}{declaration} active mint; wait(1);"),
                CompileLimits::default()
            )
            .is_err(),
            "{declaration}"
        );
    }
    let errors = compile_scenario(
        &format!("{PREFIX} active mint; wait(1); incoming mint at=2 damage=1;"),
        CompileLimits::default(),
    )
    .unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message == "configuration must precede rotation statements")
    );
}

#[test]
fn incoming_source_uses_native_shield_absorption_and_off_field_named_recipients_across_cycles() {
    let mut compiled = compile_scenario(
        include_str!("../../../../examples/incoming-shield-cycles.nte"),
        CompileLimits::default(),
    )
    .unwrap();
    for actor in &mut compiled.request.actors {
        actor.static_stats.base_hp = Some(10000);
        actor.static_stats.hp = 10000;
        actor.initial_hp = 10000;
        actor.initial_hp_max = 10000;
    }
    compiled.request.actors[0].static_stats.base_def = Some(1000);
    compiled.request.actors[0].static_stats.def = 1000;
    let report =
        crate::simulation::run_game_scenario(&compiled.request, &Default::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    let receipts = report["incoming_damage_receipts"].as_array().unwrap();
    assert_eq!(receipts.len(), 5);
    // Lv1 Blessing is 1000 + .8 * (1000 * 1.2 with Righteous Heart) = 1960.
    // The first two events absorb 1000 then 960. Renewal absorbs 1000 again.
    assert_eq!(receipts[0]["actual_hp_loss"], 0);
    assert_eq!(receipts[1]["actual_hp_loss"], 4040);
    assert_eq!(receipts[2]["actor_index"], 0);
    assert_eq!(receipts[2]["delivered_damage"], 0);
    assert_eq!(receipts[3]["actor_index"], 0);
    assert_eq!(receipts[3]["actual_hp_loss"], 1000);
    assert_eq!(receipts[4]["actual_hp_loss"], 0);
    assert_eq!(report["final_actor_states"][0]["hp"], 9000);
    assert_eq!(report["final_actor_states"][1]["hp"], 5960);
    assert_eq!(report["final_buff_state"]["active_actor_index"], 1);
}

#[test]
fn incoming_source_retains_zero_horizon_endpoint_and_death_without_repeated_hp_loss() {
    let compiled = compile_scenario(&format!("{PREFIX} incoming mint at=0 damage=0; incoming mint at=1 damage=9007199254740991; incoming mint at=5 damage=10; active mint; wait(5);"), CompileLimits::default()).unwrap();
    let report =
        crate::simulation::run_game_scenario(&compiled.request, &Default::default()).unwrap();
    assert_eq!(report["execution_completed"], true);
    let receipts = report["incoming_damage_receipts"].as_array().unwrap();
    assert_eq!(receipts.len(), 3);
    assert_eq!(receipts[0]["actual_hp_loss"], 0);
    assert_eq!(
        receipts[1]["actual_hp_loss"],
        compiled.request.actors[0].initial_hp
    );
    assert_eq!(receipts[1]["defeated"], true);
    assert_eq!(receipts[2]["time_us"], 5_000_000);
    assert_eq!(receipts[2]["actual_hp_loss"], 0);
    assert_eq!(receipts[2]["overflow"], 10);
}
