use crate::simulation::{PreparedSourceScenario, SourceScenarioReport};

fn source(actions: &str) -> String {
    format!(
        "options duration=45 crit=random seed=42 stamina_capacity=60 initial_stamina=60
        stamina_use_efficiency=1 stamina_recovery_rate=0;
        daffodil char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0;
        target hp=100000000 def=1080 resist=0;
        target hp=100000000 def=1080 resist=0; active daffodil; {actions}"
    )
}

fn run(source: &str) -> SourceScenarioReport {
    let prepared =
        PreparedSourceScenario::new(source, Default::default(), &Default::default()).unwrap();
    let prepared = PreparedSourceScenario::from_export(
        &serde_json::from_slice(&serde_json::to_vec(&prepared.export()).unwrap()).unwrap(),
        &Default::default(),
    )
    .unwrap();
    prepared.run(42).unwrap()
}

fn completed(result: &SourceScenarioReport) {
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    super::assert_native_replay(result);
}

#[test]
fn source_daffodil_held_combo_uses_the_native_jump_gate_and_cancels_only_future_contacts() {
    let result = run(&source(
        "repeat 2 {daffodil attack, attack[window=hold], attack[hold=200ms]; wait(2.000001);}",
    ));
    completed(&result);
    assert_eq!(result.occurrences[1].completed_us, Some(1_164_546));
    assert_eq!(result.occurrences[2].started_us, 1_164_546);
    assert_eq!(result.occurrences[2].completed_us, Some(2_716_669));
    assert_eq!(result.report.hit_receipts.len(), 44);
    assert!(
        !result
            .report
            .hit_receipts
            .iter()
            .any(|hit| { hit.damage_effect_id == "GE_Player_Daffodill_Melee2_1_Damage" })
    );
    for occurrence in [&result.occurrences[2], &result.occurrences[6]] {
        let command = occurrence.command_indices[0];
        let hits: Vec<_> = result
            .report
            .hit_receipts
            .iter()
            .filter(|hit| hit.command_index == Some(command))
            .collect();
        assert_eq!(hits.len(), 18);
        assert_eq!(hits[0].time_us, occurrence.started_us + 104_054);
    }
    // An observer alone neither consumes the montage nor cancels its second packet.
    let observed = run(&source("daffodil attack, attack[window=hold];"));
    let ordinary = run(&source("daffodil attack:2;"));
    super::assert_same_combat(&observed.report, &ordinary.report);
}

#[test]
fn source_daffodil_front_dodge_preserves_normal_progress_without_an_unrelated_hold_input() {
    let result = run(&source(
        "repeat 2 {daffodil attack, attack[window=dash], dash, attack; wait(2.000001);}",
    ));
    completed(&result);
    assert!(result.request.hold_inputs.is_empty());
    assert_eq!(result.occurrences[1].completed_us, Some(1_164_546));
    assert_eq!(result.occurrences[2].completed_us, Some(2_732_900));
    assert_eq!(result.report.hit_receipts.len(), 36);
    for occurrence in [&result.occurrences[1], &result.occurrences[6]] {
        let command = occurrence.command_indices[0];
        let hits: Vec<_> = result
            .report
            .hit_receipts
            .iter()
            .filter(|hit| hit.command_index == Some(command) && hit.target_index == 0)
            .collect();
        assert_eq!(hits.len(), 2);
        assert_eq!(
            hits[1].damage_effect_id,
            "GE_Player_Daffodill_Melee2_1_Damage"
        );
        // The Dodge adds exactly its1.014541s pause to the already admitted packet.
        assert_eq!(hits[1].time_us, occurrence.started_us + 1_359_994);
    }
    assert_eq!(result.report.hit_receipts[6].time_us, 2_919_026);
    let value = serde_json::to_value(&result.report).unwrap();
    assert_eq!(
        value["combat_stamina"]["total_spent"],
        serde_json::json!("30/1")
    );
    // The observed Front gate cannot waive the later Back gate or pay its cost.
    let rejected = run(&source(
        "daffodil attack, attack[window=dash], dash[back=true];",
    ));
    assert!(!rejected.report.success && !rejected.report.execution_completed);
    assert_eq!(rejected.occurrences[2].started_us, 1_164_546);
    assert_eq!(rejected.occurrences[2].completed_us, None);
    let rejected_value = serde_json::to_value(&rejected.report).unwrap();
    assert_eq!(
        rejected_value["rejected_action"]["reason"]["until_us"],
        1_603_286
    );
    assert_eq!(
        rejected_value["combat_stamina"]["total_spent"],
        serde_json::json!("0/1")
    );
    super::assert_native_replay(&rejected);
}

#[test]
fn source_daffodil_jump_ends_suspension_without_delivering_the_cancelled_normal_packet() {
    let result = run(&source(
        "repeat 2 {
            daffodil attack, attack[window=dash], dash[window=jump], jump, attack;
            wait(2.000001);
        }",
    ));
    completed(&result);
    assert_eq!(result.occurrences[2].completed_us, Some(1_247_090));
    assert_eq!(result.occurrences[3].started_us, 1_247_090);
    assert_eq!(result.report.hit_receipts.len(), 12);
    assert!(
        !result
            .report
            .hit_receipts
            .iter()
            .any(|hit| { hit.damage_effect_id == "GE_Player_Daffodill_Melee2_1_Damage" })
    );
    assert_eq!(result.report.hit_receipts[4].time_us, 1_432_398);
    assert_eq!(
        result.report.hit_receipts[4].damage_effect_id,
        "GE_Player_Daffodill_Melee1_Damage"
    );
    completed(&run(include_str!(
        "../../../../../examples/daffodil-input-windows.nte"
    )));
}

#[test]
fn source_daffodil_skill_window_preserves_notify_cooldown_and_continuing_energy() {
    let result = run(&source(
        "let end=20;
        repeat 2 {
            daffodil attack, attack[window=skill], skill;
            if .daffodil.skill.cd!=end-20+17.622253-.time {let assertion_failure=1/0;}
            wait_until(end); end=end+20;
        }",
    ));
    completed(&result);
    assert_eq!(result.occurrences[1].completed_us, Some(1_439_963));
    assert_eq!(result.report.hit_receipts.len(), 44);
    let first = result
        .report
        .hit_receipts
        .iter()
        .find(|hit| hit.command_index == Some(2))
        .unwrap();
    assert_eq!(first.time_us, 1_636_638);
}
