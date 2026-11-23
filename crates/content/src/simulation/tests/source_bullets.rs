use crate::simulation::{PreparedSourceScenario, SourceScenarioReport};
use serde_json::json;

fn source(initial: &str, actions: &str) -> String {
    format!(
        "options duration=45 crit=random;
        jiuyuan char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0 {initial};
        target hp=100000000 def=0 resist=0; active jiuyuan; {actions}"
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
use super::source_causality::assert_native_replay as replay;

#[test]
fn source_bullets_keeps_two_normal_marker_cycles_and_cap_without_rng_draws() {
    for (initial, first, second) in [(0, 2, 4), (4, 6, 6)] {
        let text = source(
            &format!("start_bullets={initial}"),
            &format!(
                "
            if .jiuyuan.resources.bullets!={initial} || .jiuyuan.energy!=0 {{let assertion_failure=1/0;}}
            jiuyuan attack:5; if .jiuyuan.resources.bullets!={first} {{let assertion_failure=1/0;}}
            jiuyuan attack:5; if .jiuyuan.resources.bullets!={second} {{let assertion_failure=1/0;}}"
            ),
        );
        let result = run(&text);
        let baseline = run(&source(
            &format!("start_bullets={initial}"),
            "jiuyuan attack:5; jiuyuan attack:5;",
        ));
        super::assert_same_combat(&result.report, &baseline.report);
        let value = serde_json::to_value(&result.report).unwrap();
        assert_eq!(
            value["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|hit| hit["damage_effect_id"]
                    .as_str()
                    .is_some_and(|id| id.starts_with("GE_Player_Kuhara_Melee")))
                .count(),
            22
        );
        replay(&result);
    }
}

#[test]
fn source_bullets_preload_and_two_skills_remain_separate_from_pact_ticks() {
    let result = run(&source(
        "start_bullets=1",
        "jiuyuan skill;
        if .jiuyuan.resources.bullets!=5 {let assertion_failure=1/0;}
        wait_until(20); jiuyuan skill;
        if .jiuyuan.resources.bullets!=6 {let assertion_failure=1/0;}
        wait_until(40); if .jiuyuan.resources.bullets!=6 {let assertion_failure=1/0;}",
    ));
    replay(&result);
}

#[test]
fn source_bullets_field_exit_preserves_stock_and_off_field_death_clears_it() {
    let text = source("start_bullets=1", "jiuyuan attack:3;
        if .jiuyuan.resources.bullets!=2 {let assertion_failure=1/0;}
        mint attack;
        if .jiuyuan.onfield || .jiuyuan.resources.bullets!=2 {let assertion_failure=1/0;}
        wait_until(10);
        if .jiuyuan.hp!=0 || .jiuyuan.resources.bullets!=0 {let assertion_failure=1/0;}")
        .replace("active jiuyuan;", "mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; incoming jiuyuan at=10 damage=1000000; active jiuyuan;");
    let result = run(&text);
    replay(&result);
}

#[test]
fn source_bullets_capacity_integrality_owner_and_explicit_zero_validate() {
    for value in ["-1", "0.5", "7"] {
        assert!(
            PreparedSourceScenario::new(
                &source(&format!("start_bullets={value}"), ""),
                Default::default(),
                &Default::default()
            )
            .is_err()
        );
    }
    let wrong = source("start_bullets=0", "").replace("jiuyuan", "mint");
    assert!(PreparedSourceScenario::new(&wrong, Default::default(), &Default::default()).is_err());
    for value in [0, 6] {
        let result = run(&source(
            &format!("start_bullets={value}"),
            &format!("if .jiuyuan.resources.bullets!={value} {{let assertion_failure=1/0;}}"),
        ));
        assert_eq!(
            serde_json::to_value(&result.request).unwrap()["actors"][0]["initial_resources"]["bullets"],
            json!(format!("{}/{}", value, 1))
        );
        let mut request = result.request;
        request.actors[0].initial_resources.bullets = Some(crate::formula::Ratio {
            numerator: 7,
            denominator: 1,
        });
        assert!(
            crate::simulation::PreparedGameScenario::new(&request, &Default::default()).is_err()
        );
    }
    replay(&run(include_str!(
        "../../../../../presets/native/resource-control.nte"
    )));
}
