use crate::simulation::{
    GameActionCommand, GameActorParryEvent, PreparedSourceScenario, SourceScenarioReport,
};

fn run(source: &str) -> SourceScenarioReport {
    let program =
        PreparedSourceScenario::new(source, Default::default(), &Default::default()).unwrap();
    let program = PreparedSourceScenario::from_export(
        &serde_json::from_slice(&serde_json::to_vec(&program.export()).unwrap()).unwrap(),
        &Default::default(),
    )
    .unwrap();
    program.run(42).unwrap()
}

fn complete(result: &SourceScenarioReport) {
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    super::assert_native_replay(result);
}

#[test]
fn source_parry_dodges_pay_stamina_and_bind_actual_inputs_without_critical_dodge() {
    for owner in ["blackbird", "hathor", "haniel"] {
        let source = format!(
            "options duration=15 crit=random seed=42 stamina_capacity=60 initial_stamina=60
            stamina_use_efficiency=1 stamina_recovery_rate=0;
            {owner} char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
            mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
            target hp=100000000 def=1080 resist=0; active mint;
            repeat 2 {{{owner} dash[parry=true]; {owner} attack; wait(1);}}"
        );
        let result = run(&source);
        complete(&result);
        assert_eq!(result.occurrences[0].command_indices.len(), 2);
        assert_eq!(result.report.parry_receipts.len(), 2);
        for receipt in &result.report.parry_receipts {
            let GameActorParryEvent::AfterAction { command_index } =
                result.request.parry_events[receipt.event_index]
            else {
                panic!("unbound parry")
            };
            assert_eq!(
                receipt.time_us,
                result.request.actions[command_index].timestamp_us()
            );
            assert!(matches!(
                result.request.actions[command_index],
                GameActionCommand::PlayerInput {
                    input: crate::simulation::PlayerInput::Dash,
                    critical_success: false,
                    ..
                }
            ));
        }
        let value = serde_json::to_value(&result.report).unwrap();
        assert_eq!(
            value["combat_stamina"]["total_spent"],
            serde_json::json!("30/1")
        );
        let ripostes: usize = result
            .report
            .action_metadata_ledger
            .iter()
            .filter(|entry| {
                result
                    .report
                    .action_metadata(entry)
                    .unwrap()
                    .action
                    .input_role
                    == Some(crate::simulation::metadata::InputActionRole::CriticalRiposte)
            })
            .map(|entry| entry.command_indices.len())
            .sum();
        assert_eq!(
            ripostes,
            if owner == "blackbird" { 0 } else { 2 },
            "{owner}"
        );
        let plain = run(&source.replace("parry=true", "parry=false"));
        complete(&plain);
        assert!(plain.report.parry_receipts.is_empty());
        if owner == "haniel" {
            for command_index in [0, usize::MAX] {
                let mut invalid = result.request.clone();
                invalid.parry_events[0] = GameActorParryEvent::AfterAction { command_index };
                assert!(
                    crate::simulation::PreparedGameScenario::new(&invalid, &Default::default())
                        .is_err()
                );
            }
            let mut duplicate = result.request.clone();
            duplicate
                .parry_events
                .push(duplicate.parry_events[0].clone());
            assert!(
                crate::simulation::PreparedGameScenario::new(&duplicate, &Default::default())
                    .is_err()
            );
        }
    }
}

#[test]
fn source_parry_qualified_support_settles_the_finisher_then_phantom_step() {
    for (hp, recipients) in [(100_000_000, 2), (1, 1)] {
        let source = format!(
            "options duration=20 crit=random seed=42;
            daffodil char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=120;
            fadia char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
            target max_hp=100000000 hp={hp} def=1080 resist=0;
            target hp=100000000 def=1080 resist=0; active daffodil;
            daffodil burst[target=t1]; fadia swap; wait(1); daffodil swap[target=t1,parry=true];"
        );
        let result = run(&source);
        complete(&result);
        let occurrence = &result.occurrences[3];
        assert_eq!(
            occurrence.completed_us,
            Some(occurrence.started_us + 1_667_260 + 1_645_027)
        );
        let command = occurrence.command_indices[0];
        let hits: Vec<_> = result
            .report
            .hit_receipts
            .iter()
            .filter(|hit| hit.command_index == Some(command))
            .collect();
        assert_eq!(
            hits.iter()
                .filter(|hit| hit.damage_effect_id == crate::characters::daffodil::phantom::QTE)
                .count(),
            recipients
        );
        assert_eq!(
            hits.iter()
                .filter(|hit| hit.damage_effect_id
                    == crate::characters::daffodil::phantom::APPEAR_SUCCESS)
                .count(),
            recipients
        );
        assert_eq!(
            hits.iter()
                .filter(|hit| hit.damage_effect_id == crate::characters::daffodil::phantom::ENTRY1)
                .count(),
            recipients
        );
        let value = serde_json::to_value(&result.report).unwrap();
        assert_eq!(
            value["final_actor_states"][0]["resources"]["daffodil.phantom_step_charges"]["current"],
            0
        );
    }
}

#[test]
fn source_parry_blackbird_form_grants_one_real_skill_permission_without_an_extra_attack() {
    let source = "options duration=40 crit=random seed=42 stamina_capacity=30 initial_stamina=30
        stamina_use_efficiency=1 stamina_recovery_rate=0;
        blackbird char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=100;
        target hp=100000000 def=1080 resist=0; active blackbird;
        blackbird burst; blackbird dash[parry=true];
        if !.blackbird.skill.ready {let assertion_failure=1/0;}
        blackbird skill;
        if .blackbird.skill.ready {let assertion_failure=1/0;}";
    let result = run(source);
    complete(&result);
    let parry = &result.report.parry_receipts[0];
    assert_eq!(
        parry.malice_granted,
        crate::simulation::ExactFraction::new(0, 1).unwrap()
    );
    assert_eq!(
        parry.skill_permission_until_us,
        Some(parry.time_us + 30_000_000)
    );
    assert_eq!(result.request.actions.iter().filter(|command| matches!(command,
        GameActionCommand::InvokeAbility {ability_id, ..} if ability_id=="GA_BlackBird_Evolve_Skill")).count(), 1);
    let rejected = run(&format!("{source} blackbird skill;"));
    assert!(!rejected.report.success && !rejected.report.execution_completed);
    super::assert_native_replay(&rejected);
}

#[test]
fn source_parry_parameters_diagnose_unvisited_types_and_owner_families_with_utf16_ranges() {
    for (action, token) in [
        ("mint swap[parry=false]", "false"),
        ("haniel skill[parry=true]", "true"),
        ("haniel dash[parry=\"true\"]", "\"true\""),
        ("haniel dash[parry=2]", "2"),
    ] {
        let source = format!(
            "# 🛡️\noptions duration=10 crit=fixed;
            mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
            haniel char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
            target hp=100000000 def=1080 resist=0; active mint;
            if 0 {{{action};}}"
        );
        let errors = PreparedSourceScenario::new(&source, Default::default(), &Default::default())
            .err()
            .unwrap();
        assert_eq!(
            errors[0].code,
            crate::dsl::validation::DiagnosticCode::Configuration
        );
        let loc = &errors[0].loc;
        assert_eq!(
            String::from_utf16(
                &source
                    .encode_utf16()
                    .skip(loc.offset)
                    .take(loc.length)
                    .collect::<Vec<_>>()
            )
            .unwrap(),
            token
        );
    }
}

#[test]
fn source_parry_daffodil_entry_retains_two_paid_bursts_and_four_charge_backed_finishers() {
    let result = run(include_str!(
        "../../../../../examples/daffodil-parry-cycles.nte"
    ));
    complete(&result);
    assert_eq!(result.report.parry_receipts.len(), 4);
    assert_eq!(result.report.final_actor_states[0].spent_energy, 240);
    let finishers: Vec<_> = result
        .report
        .hit_receipts
        .iter()
        .filter(|hit| hit.damage_effect_id == crate::characters::daffodil::phantom::APPEAR_SUCCESS)
        .collect();
    assert_eq!(finishers.len(), 8);
    let value = serde_json::to_value(&result.report).unwrap();
    assert_eq!(
        value["final_actor_states"][0]["resources"]["daffodil.phantom_step_charges"]["current"],
        0
    );
    for receipt in &result.report.parry_receipts {
        let GameActorParryEvent::AfterAction { command_index } =
            result.request.parry_events[receipt.event_index]
        else {
            panic!("unbound parry")
        };
        assert!(matches!(
            result.request.actions[command_index],
            GameActionCommand::SwitchActor {
                target_actor_index: 0,
                ..
            }
        ));
        assert_eq!(
            finishers
                .iter()
                .filter(|hit| hit.command_index == Some(command_index))
                .count(),
            2
        );
    }
}

#[test]
fn source_parry_invalid_entry_fails_without_switch_or_grant_and_replays_its_failure() {
    for active in ["mint", "daffodil"] {
        let source = format!(
            "options duration=10 crit=fixed;
            daffodil char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
            mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
            target hp=100000000 def=1080 resist=0; active {active};
            daffodil swap[parry=true];"
        );
        let result = run(&source);
        assert!(!result.report.success && !result.report.execution_completed);
        assert!(result.report.parry_receipts.is_empty());
        assert!(result.report.hit_receipts.is_empty());
        assert_eq!(result.occurrences[0].completed_us, None);
        super::assert_native_replay(&result);
    }
}
