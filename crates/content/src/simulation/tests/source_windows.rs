use crate::simulation::{PreparedSourceScenario, SourceActionWindow, SourceScenarioReport};

fn source(actions: &str) -> String {
    format!(
        "# 🐦\noptions duration=45 crit=random seed=42 stamina_capacity=100 initial_stamina=100
        stamina_use_efficiency=1 stamina_recovery_rate=0;
        blackbird char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=100;
        target hp=100000000 def=0 resist=0;
        target hp=100000000 def=0 resist=0; active blackbird; {actions}"
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
fn lacrimosa_swap_window_reaches_baicang_entry_with_qualified_support() {
    let result = run(
        "options duration=8 crit=fixed stamina_capacity=100 initial_stamina=100
        stamina_use_efficiency=1 stamina_recovery_rate=0;
        lacrimosa char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        baicang char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active lacrimosa;
        lacrimosa dash[window=swap]; wait(100ms); baicang swap;",
    );
    completed(&result);
    assert_eq!(
        result.occurrences[0].action_window,
        Some(SourceActionWindow::Swap)
    );
    assert_eq!(result.occurrences[0].completed_us, Some(0));
    let trace = &result.report.trace;
    assert!(trace.iter().any(
        |event| event.event_kind == "baicang_lacrimosa_entry_jump" && event.time_us == 100_000
    ));
    assert_eq!(
        result.report.final_actor_states[0]
            .cycle
            .as_ref()
            .unwrap()
            .spent,
        100
    );
    assert!(
        result
            .report
            .hit_receipts
            .iter()
            .any(|hit| hit.damage_effect_id == "GE_Player_Cang_QTE_Damage")
    );
}

#[test]
fn source_hold_window_preserves_contact_hitlag_and_two_paid_time_roostcalls() {
    let result = run(&source(
        "repeat 2 {blackbird attack[window=hold], attack[hold=200ms];}",
    ));
    completed(&result);
    assert_eq!(
        result.occurrences[0].action_window,
        Some(SourceActionWindow::Hold)
    );
    assert_eq!(result.occurrences[0].completed_us, Some(214_249));
    let hits: Vec<_> = result
        .report
        .hit_receipts
        .iter()
        .filter(|hit| {
            hit.damage_effect_id == crate::characters::blackbird::branches::ROOSTCALL_PACKET
        })
        .collect();
    assert_eq!(hits.len(), 4);
    assert_eq!(hits[0].time_us, 614_725);
    for hit in hits {
        assert_eq!(
            hit.time_us,
            result.request.actions[hit.command_index.unwrap()].timestamp_us() + 400_476
        );
    }
    assert_eq!(result.request.hold_inputs.len(), 2);
    let implicit = source("repeat 2 {blackbird attack[window=hold], attack[hold=200ms];}")
        .replacen(
            "target hp=",
            "mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=",
            1,
        )
        .replace("active blackbird;", "active mint;");
    let implicit = run(&implicit);
    completed(&implicit);
    assert_eq!(implicit.occurrences[0].command_indices.len(), 2);
    assert_eq!(
        implicit.occurrences[0].action_window,
        Some(SourceActionWindow::Hold)
    );
}

#[test]
fn source_evolved_hold_windows_keep_committed_extras_and_replace_grace_with_tidings() {
    let result = run(include_str!(
        "../../../../../examples/blackbird-input-windows.nte"
    ));
    completed(&result);
    let count = |effect: &str| {
        result
            .report
            .hit_receipts
            .iter()
            .filter(|hit| hit.damage_effect_id == effect)
            .count()
    };
    assert_eq!(
        count(crate::characters::blackbird::branches::BESTOWED_GRACE_PACKET),
        4
    );
    assert_eq!(
        count(crate::characters::blackbird::branches::TIDINGS_PACKET),
        20
    );
    assert_eq!(count("GE_Player_BlackBird_Evolve_Melee2_Extra_Damage"), 4);
    assert_eq!(result.report.final_actor_states[0].spent_energy, 100);
}

#[test]
fn source_jump_yields_after_only_caused_pauses_and_cancels_remaining_stage3_requests() {
    let result = run(&source(
        "blackbird attack:2;
        blackbird attack[window=jump]; blackbird jump; blackbird attack:5;",
    ));
    completed(&result);
    assert_eq!(result.occurrences[2].completed_us, Some(1_977_027));
    assert_eq!(result.occurrences[3].started_us, 1_977_027);
    assert_eq!(result.occurrences[3].completed_us, Some(1_977_027));
    assert_eq!(
        result
            .report
            .hit_receipts
            .iter()
            .filter(|hit| hit.command_index == Some(2))
            .count(),
        2
    );
}

#[test]
fn source_jump_keeps_an_already_spawned_train_after_montage_cancellation() {
    let source = source(
        "blackbird burst; blackbird attack:4;
        blackbird attack[window=jump]; blackbird jump;",
    )
    .replace("resist=0", "resist=1");
    let result = run(&source);
    completed(&result);
    let input = result.occurrences[5].command_indices[0];
    let start = result.request.actions[input].timestamp_us();
    let packets: Vec<_> = result
        .report
        .hit_receipts
        .iter()
        .filter(|hit| hit.damage_effect_id == "GE_Player_BlackBird_Evolve_Melee5_Damage")
        .collect();
    assert_eq!(packets.len(), 12);
    assert_eq!(result.occurrences[5].completed_us, Some(start + 524_134));
    assert_eq!(packets.last().unwrap().time_us, start + 937_118);
    assert!(packets.iter().all(|hit| hit.applied_hp_loss == 0));
}

#[test]
fn source_jump_controls_roundtrip_for_all_ten_implemented_owner_profiles() {
    for name in [
        "baicang",
        "blackbird",
        "chaos",
        "chiz",
        "daffodil",
        "edgar",
        "fadia",
        "haniel",
        "hathor",
        "nanally",
    ] {
        let actions = if name == "baicang" {
            "repeat 2 {baicang skill[window=jump]; baicang jump; wait(14);}".to_owned()
        } else {
            format!("repeat 2 {{{name} attack[window=jump]; {name} jump;}}")
        };
        let source = format!(
            "options duration=40 crit=random stamina_capacity=100 initial_stamina=100
            stamina_use_efficiency=1 stamina_recovery_rate=0;
            {name} char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
            target hp=100000000 def=0 resist=0; active {name};
            {actions}"
        );
        let result = run(&source);
        assert!(
            result.report.execution_completed,
            "{name}: {:?}",
            result.diagnostic
        );
        completed(&result);
        assert_eq!(
            result
                .request
                .actions
                .iter()
                .filter(|command| matches!(
                    command,
                    crate::simulation::GameActionCommand::PlayerInput {
                        input: crate::simulation::PlayerInput::Jump,
                        ..
                    }
                ))
                .count(),
            2,
            "{name}"
        );
    }
}

#[test]
fn source_skill_and_paid_dash_use_their_own_native_windows_without_polling() {
    let skill = run(&source(
        "repeat 2 {
        blackbird attack[window=skill]; blackbird skill; wait(16);
    }",
    ));
    completed(&skill);
    assert_eq!(skill.occurrences[0].completed_us, Some(362_777));
    assert_eq!(skill.occurrences[1].started_us, 362_777);
    let dash = run(&source(
        "repeat 2 {
        blackbird attack[window=dash]; blackbird dash[back=true,critical=true];
        blackbird attack; wait(1us);
    }",
    ));
    completed(&dash);
    assert_eq!(dash.occurrences[0].completed_us, Some(784_013));
    let stamina = serde_json::to_value(&dash.report).unwrap();
    assert_eq!(
        stamina["combat_stamina"]["total_spent"],
        serde_json::json!("30/1")
    );
    assert_eq!(
        dash.report
            .action_metadata_ledger
            .iter()
            .filter(|entry| {
                dash.report
                    .action_metadata(entry)
                    .unwrap()
                    .action
                    .input_role
                    == Some(crate::simulation::metadata::InputActionRole::CriticalRiposte)
            })
            .map(|entry| entry.command_indices.len())
            .sum::<usize>(),
        2
    );
}

#[test]
fn source_window_does_not_cancel_or_admit_a_following_tap_before_native_recovery() {
    let noop = run(&source(
        "blackbird attack[window=hold], swap, attack[hold=200ms];",
    ));
    completed(&noop);
    assert_eq!(noop.occurrences[1].started_us, 214_249);
    assert_eq!(noop.occurrences[1].completed_us, Some(214_249));
    assert_eq!(noop.occurrences[2].started_us, 214_249);
    let observed = run(&source("blackbird attack[window=hold];"));
    let full = run(&source(
        "blackbird attack; if 0 {blackbird attack[hold=200ms];}",
    ));
    completed(&observed);
    super::assert_same_combat(&observed.report, &full.report);
    let result = run(&source("blackbird attack[window=hold], attack[hold=0];"));
    assert!(!result.report.execution_completed);
    assert_eq!(result.occurrences[0].completed_us, Some(214_249));
    assert_eq!(result.occurrences[1].started_us, 214_249);
    assert_eq!(result.occurrences[1].completed_us, None);
    let rejected = serde_json::to_value(&result.report.rejected_action).unwrap();
    assert_eq!(rejected["reason"]["kind"], "recovery");
    assert_eq!(rejected["reason"]["until_us"], 785_620);
}

#[test]
fn source_window_identifiers_and_owners_validate_even_in_unvisited_branches() {
    for value in ["missing", "\"jump\"", "0", ".time"] {
        let source = source(&format!("if 0 {{blackbird attack[window={value}];}}"));
        let errors = PreparedSourceScenario::new(&source, Default::default(), &Default::default())
            .err()
            .unwrap();
        let error = &errors[0];
        assert_eq!(
            error.code,
            crate::dsl::validation::DiagnosticCode::Configuration
        );
        let span: Vec<_> = source
            .encode_utf16()
            .skip(error.loc.offset)
            .take(error.loc.length)
            .collect();
        assert_eq!(
            String::from_utf16(&span).unwrap(),
            if value == ".time" { "." } else { value }
        );
    }
    let source = source("if 0 {blackbird attack[window=hold];}").replace("blackbird", "mint");
    assert!(PreparedSourceScenario::new(&source, Default::default(), &Default::default()).is_err());
}
