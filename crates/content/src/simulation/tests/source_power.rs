use crate::simulation::{PreparedSourceScenario, SourceScenarioReport};
use serde_json::json;

fn source(initial: &str, actions: &str) -> String {
    format!(
        "options duration=20 crit=random;
        hathor char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0 {initial};
        hathor add stats ce=1;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
        target hp=100000000 def=0 resist=0; active hathor; {actions}"
    )
}
fn run(source: &str) -> SourceScenarioReport {
    PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap()
}
fn replay(result: &SourceScenarioReport) {
    let native =
        crate::simulation::run_game_scenario_report(&result.request, &Default::default()).unwrap();
    let actual = serde_json::to_value(&result.report).unwrap();
    let native = serde_json::to_value(native).unwrap();
    for (key, value) in actual.as_object().unwrap() {
        assert_eq!(value, &native[key], "Power native replay {key}");
    }
}

#[test]
fn source_power_preload_is_not_generation_and_tap_pays_only_accepted_delta() {
    let mut values = vec![];
    for initial in [5, 6] {
        let guard = format!(
            "if .hathor.resources.power!={initial} || .hathor.energy!=0
            || .mint.energy!=0 || .hathor.emergency_delivery.active
            || .hathor.out_of_combat.active {{let assertion_failure=1/0;}}"
        );
        let actions = format!(
            "{guard} hathor skill;
            if .hathor.resources.power!=6 {{let assertion_failure=1/0;}}
            wait_until(17); hathor skill;
            if .hathor.resources.power!=6 {{let assertion_failure=1/0;}}"
        );
        let result = run(&source(&format!("start_power={initial}"), &actions));
        assert!(
            result.report.success && result.report.execution_completed,
            "{:?}",
            result.diagnostic
        );
        let baseline = run(&source(
            &format!("start_power={initial}"),
            "hathor skill; wait_until(17); hathor skill;",
        ));
        super::assert_same_combat(&result.report, &baseline.report);
        replay(&result);
        let value = serde_json::to_value(&result.report).unwrap();
        let pool = &value["final_actor_states"][0]["fractional_resources"]["hathor.express_delivery_power"];
        assert_eq!(pool["initial"], json!(format!("{}/{}", initial, 1)));
        assert_eq!(pool["granted"], json!("4/1"));
        assert_eq!(pool["overflow"], json!(format!("{}/{}", initial - 2, 1)));
        assert_eq!(pool["conserved"], true);
        values.push(value);
    }
    let actor = |i: usize| &values[i]["final_actor_states"][0];
    assert_eq!(
        actor(0)["energy"].as_u64().unwrap(),
        actor(1)["energy"].as_u64().unwrap() + 5
    );
    assert_eq!(actor(0)["charge_remainder"], actor(1)["charge_remainder"]);
    assert_eq!(
        values[0]["final_actor_states"][1],
        values[1]["final_actor_states"][1]
    );
    assert_eq!(values[0]["hit_receipts"], values[1]["hit_receipts"]);
}

#[test]
fn source_power_requires_owner_integral_capacity_and_native_models() {
    for initial in ["start_power=-1", "start_power=6.000001", "start_power=0.5"] {
        assert!(
            PreparedSourceScenario::new(
                &source(initial, ""),
                Default::default(),
                &Default::default()
            )
            .is_err()
        );
    }
    assert!(
        PreparedSourceScenario::new(
            &source("", "").replace("mint char", "mint char start_power=0"),
            Default::default(),
            &Default::default()
        )
        .is_err()
    );
    let result = run(&source(
        "start_power=0",
        "if .hathor.resources.power!=0 {let assertion_failure=1/0;}",
    ));
    assert!(result.report.success && result.report.execution_completed);
    let mut request = result.request;
    assert_eq!(
        serde_json::to_value(&request).unwrap()["actors"][0]["initial_resources"]["power"],
        json!("0/1")
    );
    request
        .model_assumptions
        .remove(crate::characters::hathor::power::MODEL.as_str());
    assert!(crate::simulation::PreparedGameScenario::new(&request, &Default::default()).is_err());
}

#[test]
fn source_power_emergency_flag_follows_native_application_and_expiry() {
    let text = source(
        "start_power=6",
        "
        if .hathor.emergency_delivery.active {let assertion_failure=1/0;}
        hathor burst;
        if !.hathor.emergency_delivery.active || .hathor.resources.power!=6 {let assertion_failure=1/0;}
        while .time<5.011964 {wait(5.011964-.time);}
        if .hathor.emergency_delivery.active {let assertion_failure=1/0;}",
    )
    .replacen("start_energy=0", "start_energy=100", 1);
    let result = run(&text);
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    replay(&result);
}
