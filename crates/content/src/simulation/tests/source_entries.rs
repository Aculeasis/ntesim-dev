use crate::simulation::{PreparedSourceScenario, SourceScenarioReport};

fn run(source: &str) -> SourceScenarioReport {
    PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap()
}

#[test]
fn source_skia_to_baicang_uses_actual_control_and_continuing_native_entry_state() {
    let result = run("options duration=60 crit=random;
        skia char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        baicang char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_hp=1000;
        target hp=100000000 def=0 resist=0;
        target hp=100000000 def=0 resist=0; active skia;
        for let i=0; i<2; i=i+1 {
            wait_until(i*33);
            skia skill;
            if !.skia.gnaw.active || .baicang.support.ready || .baicang.resources.words {let assertion_failure=1/0;}
            baicang swap;
            if .skia.gnaw.active || !.baicang.onfield || .baicang.resources.words {let assertion_failure=1/0;}
            wait(1);
            if .baicang.resources.words!=1 || .baicang.normal!=1 {let assertion_failure=1/0;}
        }");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    let pulses = result
        .report
        .hit_receipts
        .iter()
        .filter(|hit| hit.damage_effect_id == "GE_Player_Cang_Skia_Damage")
        .collect::<Vec<_>>();
    assert_eq!(pulses.len(), 12);
    assert_eq!(
        pulses
            .iter()
            .take(6)
            .map(|hit| hit.time_us)
            .collect::<Vec<_>>(),
        vec![
            1_913_903, 1_913_903, 1_993_903, 1_993_903, 2_073_903, 2_073_903
        ]
    );
    // Below half MaxHP the native Word cost is1.5% current HP, once per entry
    // independent of the six AoE receipts:1000-15=985, then985-round(14.775)=970.
    assert!(result.request.actors[1].initial_hp_max >= 2000);
    assert_eq!(result.report.final_actor_states[1].hp, 970);
    super::assert_native_replay(&result);
}

#[test]
fn source_baicang_implicit_entry_selects_the_same_native_three_pulses() {
    let prefix = "options duration=20 crit=random;
        skia char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        baicang char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active skia;
        skia skill;";
    let implicit = run(&format!("{prefix} baicang attack;"));
    let explicit = run(&format!("{prefix} baicang swap; baicang attack;"));
    assert!(
        implicit.report.execution_completed,
        "{:?}",
        implicit.diagnostic
    );
    assert!(
        explicit.report.execution_completed,
        "{:?}",
        explicit.diagnostic
    );
    super::assert_same_combat(&implicit.report, &explicit.report);
    assert_eq!(
        implicit
            .report
            .hit_receipts
            .iter()
            .filter(|hit| hit.damage_effect_id == "GE_Player_Cang_Skia_Damage")
            .count(),
        3
    );
    super::assert_native_replay(&implicit);
}
