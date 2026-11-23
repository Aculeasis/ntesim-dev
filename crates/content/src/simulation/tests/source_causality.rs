use crate::simulation::{GameActionAdmissionOrder, PreparedSourceScenario, SourceScenarioReport};
use serde_json::Value;

pub(crate) fn assert_native_replay(result: &SourceScenarioReport) {
    let request = serde_json::from_value(serde_json::to_value(&result.request).unwrap()).unwrap();
    let native =
        crate::simulation::run_game_scenario_report(&request, &Default::default()).unwrap();
    let actual = serde_json::to_value(&result.report).unwrap();
    let native = serde_json::to_value(native).unwrap();
    assert!(
        actual == native,
        "source/native report{}",
        difference(&actual, &native)
    );
}

/// Different authored probes/windows/waits can preserve all combat outcomes
/// while intentionally changing source ranges or the input-time partition.
pub(crate) fn assert_same_combat(
    a: &crate::simulation::GameScenarioReport,
    b: &crate::simulation::GameScenarioReport,
) {
    let a = serde_json::to_value(a).unwrap();
    let b = serde_json::to_value(b).unwrap();
    for (key, value) in a.as_object().unwrap() {
        if key != "input_time" {
            assert!(
                value == &b[key],
                "combat report.{key}{}",
                difference(value, &b[key])
            );
        }
    }
}

fn difference(a: &Value, b: &Value) -> String {
    if let (Some(a), Some(b)) = (a.as_object(), b.as_object()) {
        for (key, value) in a {
            if Some(value) != b.get(key) {
                return format!(".{key}{}", difference(value, &b[key]));
            }
        }
        return format!(" object lengths {} / {}", a.len(), b.len());
    }
    if let (Some(a), Some(b)) = (a.as_array(), b.as_array()) {
        for (index, (a, b)) in a.iter().zip(b).enumerate() {
            if a != b {
                return format!("[{index}]{}", difference(a, b));
            }
        }
        return format!(" array lengths {} / {}", a.len(), b.len());
    }
    format!(
        " {} / {}",
        a.to_string().chars().take(120).collect::<String>(),
        b.to_string().chars().take(120).collect::<String>()
    )
}

#[test]
fn source_causality_replay_keeps_due_pact_tick_before_same_time_skill_and_rng() {
    let text = "options duration=33 crit=random;
        jiuyuan char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active jiuyuan;
        jiuyuan skill; wait_until(20); jiuyuan skill;";
    let result = PreparedSourceScenario::new(text, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap();
    assert!(result.report.success && result.report.execution_completed);
    assert_eq!(
        result.request.action_admission_order,
        GameActionAdmissionOrder::AfterSettledEvents
    );
    let value = serde_json::to_value(&result.report).unwrap();
    let first = value["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["time_us"] == 20_000_000)
        .unwrap();
    assert_eq!(first["damage_effect_id"], "GE_Player_Kuhara_Seed_Damage");
    assert_native_replay(&result);
    let mut scheduled = result.request;
    scheduled.action_admission_order = GameActionAdmissionOrder::Scheduled;
    scheduled.source_exhausted = false;
    let report = crate::simulation::run_game_scenario(&scheduled, &Default::default()).unwrap();
    let first = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["time_us"] == 20_000_000)
        .unwrap();
    assert_eq!(first["damage_effect_id"], "GE_Player_Kuhara_Skill1_Damage");
}

#[test]
fn source_causality_replay_retains_equal_time_defeat_before_ordinary_admission() {
    let text = "options duration=2 crit=fixed;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_hp=10;
        zero char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0;
        incoming mint at=1 damage=10; active mint; wait_until(1); mint attack;";
    let result = PreparedSourceScenario::new(text, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap();
    assert!(!result.report.success && !result.report.execution_completed);
    assert_native_replay(&result);
    let mut scheduled = result.request;
    scheduled.action_admission_order = GameActionAdmissionOrder::Scheduled;
    scheduled.source_exhausted = false;
    let report = crate::simulation::run_game_scenario(&scheduled, &Default::default()).unwrap();
    assert_eq!(report["success"], true);
    assert_eq!(report["hits_settled"], 0);
}
