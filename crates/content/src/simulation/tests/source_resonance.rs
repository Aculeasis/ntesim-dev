use super::source_causality::assert_native_replay;
use crate::simulation::tests::support::formula_damage_total;
use crate::simulation::{ExactFraction as F, PreparedSourceScenario, SourceScenarioReport};
use serde_json::{Value, json};

fn source(initial: &str, awakening: u8, actions: &str) -> String {
    format!(
        "options duration=35 crit=fixed;
        daffodil char lvl=80/80 awakening={awakening} talent=1,1,1 qte=1 {initial};
        target hp=100000000 def=0 resist=0; active daffodil; {actions}"
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
fn total(report: &Value) -> F {
    formula_damage_total(report)
}

#[test]
fn source_daffodil_native_cycles_pay_finale_and_keep_phantom_entry_without_internal_overrides() {
    let text = include_str!("../../../../../examples/daffodil-skill-burst-packets.nte");
    let result = run(text);
    assert_eq!(result.request.actors[0].energy_capacity, 120);
    assert_eq!(result.report.final_actor_states[0].spent_energy, 240);
    for ability in ["GA_Daffodill_Skill", "GA_Daffodill_UltraSkill"] {
        assert_eq!(
            result
                .report
                .action_metadata_ledger
                .iter()
                .filter(|entry| {
                    entry.actor_index == 0
                        && result
                            .report
                            .action_metadata(entry)
                            .unwrap()
                            .action
                            .ability_id
                            == ability
                })
                .map(|entry| entry.command_indices.len())
                .sum::<usize>(),
            2
        );
    }
    let entries: Vec<_> = result
        .report
        .hit_receipts
        .iter()
        .filter(|hit| hit.damage_effect_id == crate::characters::daffodil::phantom::ENTRY1)
        .collect();
    assert_eq!(entries.len(), 4);
    assert_eq!(
        entries.iter().filter(|hit| hit.target_index == 0).count(),
        2
    );
    assert_eq!(
        entries.iter().filter(|hit| hit.target_index == 1).count(),
        2
    );
    for hit in entries {
        assert_eq!(
            hit.time_us,
            result.request.actions[hit.command_index.unwrap()].timestamp_us() + 253_952
        );
    }
    let finale_openers: Vec<_> = result
        .report
        .hit_receipts
        .iter()
        .filter(|hit| hit.damage_effect_id == "GE_Player_Daffodill_UltraSkill1_Damage")
        .collect();
    assert_eq!(finale_openers.len(), 4);
    for hit in finale_openers {
        assert_eq!(
            hit.time_us,
            result.request.actions[hit.command_index.unwrap()].timestamp_us() + 2_849_375
        );
    }
    assert_native_replay(&result);
}

#[test]
fn source_resonance_preload_selects_and_captures_exact_skill_damage_once() {
    for awakening in [0, 1] {
        let mut reports = vec![];
        for stacks in [0, 1, 2] {
            let result = run(&source(
                &format!("start_resonance={stacks}"),
                awakening,
                &format!(
                    "if .daffodil.resources.resonance!={stacks} {{let assertion_failure=1/0;}}
                daffodil skill; if .daffodil.resources.resonance!=0 {{let assertion_failure=1/0;}}"
                ),
            ));
            assert_native_replay(&result);
            let report = serde_json::to_value(&result.report).unwrap();
            assert_eq!(report["hits_settled"], if stacks == 2 { 7 } else { 8 });
            reports.push(report);
        }
        // Ordinary coefficient sum3.001; Echoes sum2.999. Stack additions are
        // .5 (A0) or1 (A1) in the general damage zone, not coefficient additions.
        assert_eq!(
            total(&reports[1]),
            total(&reports[0])
                .mul(
                    F::new(
                        if awakening == 0 { 3 } else { 2 },
                        if awakening == 0 { 2 } else { 1 }
                    )
                    .unwrap()
                )
                .unwrap()
        );
        assert_eq!(
            total(&reports[2]),
            total(&reports[0])
                .mul(F::new(2999 * if awakening == 0 { 2 } else { 3 }, 3001).unwrap())
                .unwrap()
        );
    }
}

#[test]
fn source_resonance_two_skills_do_not_reapply_initial_stock_or_timed_handles() {
    let result = run(&source(
        "start_resonance=2",
        0,
        "
        if .daffodil.resources.phantom_steps!=0 || .daffodil.eyes.active
            || .daffodil.phantom_handle.active || .daffodil.mods.eyes!=0
            || .daffodil.mods.phantom_handle!=0 {let assertion_failure=1/0;}
        daffodil skill; if .daffodil.resources.resonance!=0 {let assertion_failure=1/0;}
        wait_until(20); daffodil skill;
        if .daffodil.resources.resonance!=0 {let assertion_failure=1/0;}",
    ));
    assert_eq!(result.report.hits_settled, 15);
    assert_native_replay(&result);
}

#[test]
fn source_resonance_actual_party_supports_refill_the_next_cast_after_field_exit() {
    let text = source(
        "start_resonance=2",
        0,
        "daffodil skill;
        if .daffodil.resources.resonance!=0 {let assertion_failure=1/0;}
        sakiri swap;
        if .daffodil.resources.resonance!=1 || .daffodil.onfield {let assertion_failure=1/0;}
        daffodil swap;
        wait_until(20); mint swap; zero swap;
        if .daffodil.resources.resonance!=2 {let assertion_failure=1/0;}
        daffodil skill;
        if .daffodil.resources.resonance!=0 {let assertion_failure=1/0;}",
    )
    .replace("start_resonance=2;", "start_resonance=2 start_cycle=100;")
    .replace(
        "active daffodil;",
        "sakiri char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
         mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
         zero char lvl=80/80 awakening=0 talent=1,1,1 qte=1; active daffodil;",
    );
    let result = run(&text);
    let value = serde_json::to_value(&result.report).unwrap();
    assert_eq!(
        value["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["event_kind"] == "daffodil_resonance_granted")
            .count(),
        2
    );
    assert_native_replay(&result);
}

#[test]
fn source_resonance_capacity_integrality_owner_and_zero_validate() {
    for value in ["-1", "0.5", "3"] {
        assert!(
            PreparedSourceScenario::new(
                &source(&format!("start_resonance={value}"), 0, ""),
                Default::default(),
                &Default::default()
            )
            .is_err()
        );
    }
    let wrong = source("start_resonance=0", 0, "").replace("daffodil", "mint");
    assert!(PreparedSourceScenario::new(&wrong, Default::default(), &Default::default()).is_err());
    let result = run(&source(
        "start_resonance=0",
        0,
        "if .daffodil.resources.resonance!=0 {let assertion_failure=1/0;}",
    ));
    assert_eq!(
        serde_json::to_value(&result.request).unwrap()["actors"][0]["initial_resources"]["resonance"],
        json!("0/1")
    );
    let mut request = result.request;
    request.actors[0].initial_resources.resonance = Some(crate::formula::Ratio {
        numerator: 3,
        denominator: 1,
    });
    assert!(crate::simulation::PreparedGameScenario::new(&request, &Default::default()).is_err());
}
