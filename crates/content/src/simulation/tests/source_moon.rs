use crate::simulation::{ExactFraction as F, PreparedSourceScenario, SourceScenarioReport};
use serde_json::json;

fn source(initial: &str, awakening: u8, actions: &str, targets: usize) -> String {
    format!(
        "options duration=20 crit=random;
        iroi char lvl=80/80 awakening={awakening} talent=1,1,1 qte=1 {initial};
        {} active iroi; {actions}",
        "target hp=100000000 def=0 resist=0;".repeat(targets)
    )
}
fn run(text: &str) -> SourceScenarioReport {
    let result = PreparedSourceScenario::new(text, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    result
}
fn replay(result: &SourceScenarioReport) {
    let native =
        crate::simulation::run_game_scenario_report(&result.request, &Default::default()).unwrap();
    let actual = serde_json::to_value(&result.report).unwrap();
    let native = serde_json::to_value(native).unwrap();
    for (key, value) in actual.as_object().unwrap() {
        assert_eq!(value, &native[key], "Moon native replay {key}");
    }
}

#[test]
fn source_moon_keeps_exact_preload_and_two_chains_on_one_or_three_dummies() {
    for targets in [1, 3] {
        let actions =
            "if .iroi.resources.moon!=0.1 || .iroi.dream.active || .iroi.mods.dream!=0 {let assertion_failure=1/0;}
            iroi attack:5; if .iroi.resources.moon!=20.11 {let assertion_failure=1/0;}
            iroi attack:5; if .iroi.resources.moon!=40.12 {let assertion_failure=1/0;}";
        let result = run(&source("start_moon=0.1", 0, actions, targets));
        let baseline = run(&source(
            "start_moon=0.1",
            0,
            "iroi attack:5; iroi attack:5;",
            targets,
        ));
        super::assert_same_combat(&result.report, &baseline.report);
        // These native ordinary actions retain their selected recipient.
        assert_eq!(
            result.report.hits_settled,
            if targets == 1 { 32 } else { 80 }
        );
        replay(&result);
    }
}

#[test]
fn source_moon_dream_drains_continuously_and_retains_captured_ivory_gate() {
    let actions = "iroi burst;
        if !.iroi.dream.active || !.iroi.ivory_dream.active || .iroi.horn_dream.active
            || .iroi.resources.moon!=32-8*.time {let assertion_failure=1/0;}
        wait_until(7.004747);
        if .iroi.resources.moon!=8 || !.iroi.ivory_dream.active || .iroi.mods.dream!=1 {let assertion_failure=1/0;}
        wait_until(8.004746);
        if .iroi.resources.moon!=0.000008 || .iroi.mods.dream!=0.000001 {let assertion_failure=1/0;}
        wait(0.000001);
        if .iroi.resources.moon!=0 || .iroi.dream.active || .iroi.ivory_dream.active
            || .iroi.horn_dream.active || .iroi.mods.dream!=0 {let assertion_failure=1/0;}";
    let result = run(&source("start_moon=32", 0, actions, 1));
    let baseline = run(&source(
        "start_moon=32",
        0,
        "iroi burst; wait_until(7.004747); wait_until(8.004746); wait(0.000001);",
        1,
    ));
    super::assert_same_combat(&result.report, &baseline.report);
    let value = serde_json::to_value(&result.report).unwrap();
    assert_eq!(
        value["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "GE_Player_Oneiroi_UltraSkill1_Damage")
            .count(),
        3
    );
    replay(&result);
}

#[test]
fn source_moon_horn_threshold_and_a1_entry_initialization() {
    for (moon, ivory, end, pulses) in [
        ("29.999999", false, "7.754747", 0),
        ("30", true, "7.754747", 3),
        ("100", true, "16.504747", 12),
    ] {
        let ivory = u64::from(ivory);
        let result = run(&source(
            &format!("start_moon={moon}"),
            0,
            &format!(
                "iroi burst;
            if .iroi.ivory_dream.active!={ivory} || .iroi.horn_dream.active=={ivory} {{let assertion_failure=1/0;}}
            wait_until({end}); if .iroi.dream.active || .iroi.resources.moon!=0 {{let assertion_failure=1/0;}}"
            ),
            1,
        ));
        let value = serde_json::to_value(&result.report).unwrap();
        assert_eq!(
            value["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|hit| hit["damage_effect_id"] == "GE_Player_Oneiroi_UltraSkill1_Damage")
                .count(),
            pulses
        );
        replay(&result);
    }
    for initial in ["", "start_moon=0", "start_moon=12.5"] {
        run(&source(
            initial,
            1,
            "if .iroi.resources.moon!=100 {let assertion_failure=1/0;}",
            1,
        ));
    }
    for awakening in [0, 1] {
        let default = run(&source("", awakening, "iroi burst;", 1));
        let zero = run(&source("start_moon=0", awakening, "iroi burst;", 1));
        super::assert_same_combat(&default.report, &zero.report);
        replay(&default);
    }
}

#[test]
fn source_moon_capacity_owner_zero_and_model_validation() {
    for initial in ["start_moon=-1", "start_moon=100.000001"] {
        assert!(
            PreparedSourceScenario::new(
                &source(initial, 0, "", 1),
                Default::default(),
                &Default::default()
            )
            .is_err()
        );
    }
    let wrong = source("start_moon=0", 0, "", 1).replace("iroi", "mint");
    assert!(PreparedSourceScenario::new(&wrong, Default::default(), &Default::default()).is_err());
    let result = run(&source(
        "start_moon=0",
        0,
        "if .iroi.resources.moon!=0 {let assertion_failure=1/0;}",
        1,
    ));
    assert_eq!(
        serde_json::to_value(&result.request).unwrap()["actors"][0]["initial_resources"]["moon"],
        json!("0/1")
    );
    let mut request = result.request;
    request
        .model_assumptions
        .remove(crate::characters::iroi::dream::HYPOTHESIS_ID.as_str());
    assert!(crate::simulation::PreparedGameScenario::new(&request, &Default::default()).is_err());
}

#[test]
fn moon_projection_preserves_setup_and_exact_microsecond_rounding() {
    use crate::characters::iroi::{awakenings::State, resource};
    let state = State {
        moon: F::new(32, 1).unwrap(),
        dream_active: true,
        dream_release_us: 4_004_747,
        dream_end_us: 8_004_747,
        ..Default::default()
    };
    for (now, n, d) in [
        (0, 32, 1),
        (4_004_747, 32, 1),
        (4_004_748, 3_999_999, 125_000),
        (8_004_746, 1, 125_000),
        (8_004_747, 0, 1),
    ] {
        assert_eq!(
            resource::remaining(&state, now).unwrap(),
            F::new(n, d).unwrap()
        );
    }
    assert_eq!(state.moon, F::new(32, 1).unwrap());
    assert_eq!(
        resource::duration_us(F::new(8_000_001, 250_000).unwrap()).unwrap(),
        4_000_001
    );
    assert_eq!(
        resource::duration_us(F::new(7_999_999, 250_000).unwrap()).unwrap(),
        4_000_000
    );
}

#[test]
fn source_moon_owner_death_ends_dream_and_invalidates_later_pulses() {
    let text = source("start_moon=100", 0, "iroi burst; wait_until(10.004747);
        if .iroi.hp!=0 || .iroi.dream.active || .iroi.resources.moon!=0 || .iroi.mods.dream!=0 {let assertion_failure=1/0;}", 1)
        .replace("active iroi;", "mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; incoming iroi at=6 damage=1000000; active iroi;");
    let result = run(&text);
    let value = serde_json::to_value(&result.report).unwrap();
    assert_eq!(
        value["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "GE_Player_Oneiroi_UltraSkill1_Damage")
            .count(),
        5
    );
    replay(&result);
}

#[test]
fn source_moon_second_burst_uses_world_cooldown_and_never_restores_expired_budget() {
    let text = source(
        "start_moon=100",
        0,
        "iroi burst;
        iroi attack:5; iroi attack:5;
        if .iroi.resources.moon!=100-8*.time {let assertion_failure=1/0;}
        wait(.iroi.burst.cd); iroi burst;
        if .iroi.dream.active || .iroi.resources.moon!=0 || .iroi.mods.dream!=0 {let assertion_failure=1/0;}",
        1,
    )
    .replace("duration=20", "duration=24")
    .replace("active iroi;", "iroi add stats ce=1.5; active iroi;");
    let result = run(&text);
    let bursts: Vec<_> = result
        .request
        .actions
        .iter()
        .filter_map(|action| match action {
            crate::simulation::GameActionCommand::InvokeAbility {
                timestamp_us,
                ability_id,
                energy_cost,
                cooldown_us,
                ..
            } if ability_id == "GA_Oneiroi_UltraSkill" => {
                Some((*timestamp_us, *energy_cost, *cooldown_us))
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        bursts,
        [(0, 100, 16_000_000), (20_004_747, 100, 16_000_000)]
    );
    replay(&result);
}

#[test]
fn source_moon_initiated_dream_keeps_draining_off_field() {
    let text = source(
        "start_moon=100",
        0,
        "iroi burst; mint attack;
        if .iroi.onfield || !.iroi.ivory_dream.active
            || .iroi.resources.moon!=100-8*.time {let assertion_failure=1/0;}
        wait_until(16.504747);
        if .iroi.dream.active || .iroi.resources.moon!=0 {let assertion_failure=1/0;}",
        1,
    )
    .replace(
        "active iroi;",
        "mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; active iroi;",
    );
    let result = run(&text);
    replay(&result);
}
