use super::*;

const PREFIX: &str = "options duration=10 crit=fixed;
    baicang char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
    target hp=1000000 def=1080 resist=0; active baicang; ";

#[test]
fn chaos_authored_plunge_script_preserves_two_native_cycles_and_exact_channels() {
    let source = "options duration=3 crit=fixed;
        chaos char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
        target hp=1000000 def=1080 resist=0; active chaos;
        chaos plunge[airtime=600ms,height_bonus=1/2]:2;";
    let compiled = compiler::compile_scenario(source, Default::default()).unwrap();
    assert_eq!(compiled.request.actions[1].timestamp_us(), 759_289);
    let limits = crate::execution::ExecutionLimits::default();
    let report = crate::simulation::run_game_scenario(&compiled.request, &limits).unwrap();
    assert_eq!(report["execution_completed"], true);
    assert_eq!(report["hits_settled"], 2);
    assert_eq!(report["hit_receipts"][1]["time_us"], 1_406_883);
    let actor = &report["final_actor_states"][0];
    assert_eq!(actor["energy"], 4);
    assert_eq!(actor["charge_remainder"], serde_json::json!("3/5"));
    assert_eq!(
        actor["fractional_resources"]["chaos.crime"]["current"],
        serde_json::json!("60/1")
    );
    let replay = serde_json::from_slice(&serde_json::to_vec(&compiled.request).unwrap()).unwrap();
    assert_eq!(
        report,
        crate::simulation::run_game_scenario(&replay, &limits).unwrap()
    );
}

#[test]
fn typed_plunge_parameters_preserve_exact_values_source_ranges_and_continuing_time() {
    let source = format!("{PREFIX}baicang plunge[airtime=800ms,height_bonus=1/2]:2;");
    let compiled = compiler::compile_scenario(&source, Default::default()).unwrap();
    assert_eq!(compiled.request.actions.len(), 2);
    assert_eq!(compiled.request.actions[1].timestamp_us(), 998_548);
    for index in 0..2 {
        let input = compiled.request.plunge_inputs[&index];
        assert_eq!(input.landing_delay_us, 800_000);
        assert_eq!(
            input.height_bonus,
            crate::formula::Ratio {
                numerator: 1,
                denominator: 2
            }
        );
        assert_eq!(
            &source[compiled.action_locations[index].offset
                ..compiled.action_locations[index].offset + 6],
            "plunge"
        );
    }
    let bytes = serde_json::to_vec(&compiled.request).unwrap();
    let replay = serde_json::from_slice(&bytes).unwrap();
    let limits = crate::execution::ExecutionLimits::default();
    let report = crate::simulation::run_game_scenario(&compiled.request, &limits).unwrap();
    assert_eq!(report["execution_completed"], true);
    assert_eq!(report["hits_settled"], 2);
    assert_eq!(
        report,
        crate::simulation::run_game_scenario(&replay, &limits).unwrap()
    );
}

#[test]
fn unsupported_missing_repeated_and_mistyped_plunge_parameters_diagnose() {
    for action in [
        "plunge",
        "plunge[height_bonus=0]",
        "plunge[airtime=1]",
        "plunge[airtime=0,height_bonus=0]",
        "plunge[airtime=true,height_bonus=0]",
        "plunge[airtime=1,height_bonus=false]",
        "plunge[airtime=1,height_bonus=1.01]",
        "plunge[airtime=1,height_bonus=-0.1]",
        "plunge[airtime=1,height_bonus=0,airtime=2]",
        "plunge[airtime=1,height_bonus=0,target=0]",
    ] {
        assert!(
            compiler::compile_scenario(&format!("{PREFIX}baicang {action};"), Default::default())
                .is_err(),
            "{action}"
        );
    }
}
