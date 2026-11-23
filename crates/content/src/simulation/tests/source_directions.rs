use crate::simulation::{
    GameActionCommand, GameDashDirection, PreparedSourceScenario, SourceScenarioReport,
};

const SOURCE: &str =
    "options duration=14 crit=random seed=42 stamina_capacity=100 initial_stamina=100
    stamina_use_efficiency=1 stamina_recovery_rate=0;
    mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
    edgar char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
    target hp=100000000 def=0 resist=0; active mint;
    let back=1;
    repeat 2 {
        edgar dash[back=back,critical=1];
        edgar attack;
        back=1-back;
        wait(2);
    }
    if .team.stamina!=70 {let assertion_failure=1/0;}";

fn run(source: &str) -> SourceScenarioReport {
    PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap()
}

#[test]
fn source_hathor_party_projection_binds_native_paid_dodges_ripostes_and_default_power() {
    let source = SOURCE
        .replace("edgar", "hathor")
        .replace("duration=14", "duration=20")
        .replace(
            "let back=1;",
            "if .hathor.resources.power {let assertion_failure=1/0;} let back=1;",
        );
    for (wait, power) in [(2, 1), (4, 2)] {
        // Native Riposte's5s reward carrier survives the second short-spaced input.
        let source = source.replace("wait(2);", &format!("wait({wait});"));
        let result = run(&format!(
            "{source} if .hathor.resources.power!={power} {{let assertion_failure=1/0;}}"
        ));
        assert!(
            result.report.success && result.report.execution_completed,
            "{:?}",
            result.diagnostic
        );
        assert_eq!(
            result
                .report
                .hit_receipts
                .iter()
                .filter(|hit| matches!(
                    hit.damage_effect_id.as_str(),
                    "GE_Player_Hathor_PerfectEvadeAttack_Damage"
                        | "GE_Player_Hathor_PerfectEvadeAttack1_Damage"
                ))
                .count(),
            4
        );
        assert_eq!(result.request.dash_directions[&1], GameDashDirection::Back);
        assert_eq!(result.request.dash_directions[&4], GameDashDirection::Front);
        super::assert_native_replay(&result);
    }
}

#[test]
fn source_hathor_ordinary_inputs_use_two_native_continuing_five_stage_clocks() {
    let result = run("options duration=20 crit=random seed=42;
        hathor char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0;
        target hp=100000000 def=0 resist=0; active hathor;
        repeat 2 {hathor attack:5;}");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    assert_eq!(result.report.hit_receipts.len(), 26);
    assert_eq!(result.report.hit_receipts[0].time_us, 216_746);
    assert_eq!(result.report.hit_receipts[13].time_us, 8_441_361);
    let report = serde_json::to_value(&result.report).unwrap();
    assert_eq!(report["final_actor_states"][0]["energy"], 44);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        serde_json::json!("301/500")
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_dash_direction_uses_live_booleans_actual_command_indices_and_paid_ripostes() {
    let result = run(SOURCE);
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    assert_eq!(
        result
            .request
            .dash_directions
            .iter()
            .map(|(index, direction)| (*index, *direction))
            .collect::<Vec<_>>(),
        vec![(1, GameDashDirection::Back), (4, GameDashDirection::Front)]
    );
    for &index in result.request.dash_directions.keys() {
        assert!(matches!(
            result.request.actions[index],
            GameActionCommand::PlayerInput {
                actor_index: 1,
                input: crate::simulation::PlayerInput::Dash,
                critical_success: true,
                ..
            }
        ));
    }
    assert_eq!(
        result
            .report
            .hit_receipts
            .iter()
            .filter(|hit| hit.damage_effect_id == "GE_Player_Edgar_PerfectEvadeAttack_Damage")
            .count(),
        2
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_directions_declare_remaining_owner_clocks_before_implicit_entry() {
    for (owner, packet, count) in [
        ("blackbird", "GE_Player_BlackBird_PerfectEvade_Damage", 2),
        ("chaos", "GE_Player_Chaos_PerfectEvadeAttack1_Damage", 2),
        (
            "chiz",
            "GE_Player_Chiichan073_PerfectEvadeAttack1_Damage",
            2,
        ),
        (
            "daffodil",
            "GE_Player_Daffodill_PerfectEvadeAttack_Damage",
            2,
        ),
        ("haniel", "GE_Player_Haniel020_PerfectEvadeAttack_Damage", 2),
        ("fadia", "GE_Player_Fadia_PerfectEvadeAttack_Damage", 2),
    ] {
        let source = SOURCE
            .replace("edgar", owner)
            .replace("duration=14", "duration=25");
        let result = run(&source);
        assert!(
            result.report.success && result.report.execution_completed,
            "{owner}: {:?}; {:?}",
            result.diagnostic,
            result.report.halt_diagnostic
        );
        assert_eq!(
            result
                .report
                .hit_receipts
                .iter()
                .filter(|hit| hit.damage_effect_id == packet)
                .count(),
            count,
            "{owner}"
        );
        assert_eq!(
            result
                .request
                .dash_directions
                .values()
                .copied()
                .collect::<Vec<_>>(),
            vec![GameDashDirection::Back, GameDashDirection::Front]
        );
        super::assert_native_replay(&result);
        let front = run(&source.replace("back=back,", ""));
        assert!(
            front.report.execution_completed,
            "{owner}: {:?}",
            front.diagnostic
        );
        assert!(front.request.dash_directions.is_empty());
        assert_eq!(front.report.player_input_receipts.len(), 2);
    }
}

#[test]
fn source_direction_roundtrip_and_scheduled_export_preserve_explicit_front_and_back() {
    for owner in ["edgar", "blackbird", "chaos", "chiz", "daffodil", "haniel"] {
        let source = SOURCE
            .replace("edgar", owner)
            .replace("duration=14", "duration=25");
        let artifact = crate::dsl::source::compile_response(&crate::dsl::compiler::SourceRequest {
            source: source.clone(),
            limits: Default::default(),
        });
        assert!(
            artifact.diagnostics.is_empty(),
            "{:?}",
            artifact.diagnostics
        );
        let roundtrip = serde_json::from_slice(&serde_json::to_vec(&artifact).unwrap()).unwrap();
        let executed = crate::simulation::run_compiled_source(&roundtrip, &Default::default())
            .unwrap()
            .execution
            .unwrap();
        assert!(
            executed.report.execution_completed,
            "{:?}",
            executed.diagnostic
        );
        assert_eq!(
            serde_json::to_value(executed.report).unwrap(),
            serde_json::to_value(run(&source).report).unwrap()
        );
        let scheduled = crate::dsl::compiler::compile_scenario(
            &source.replace("if .team.stamina!=70 {let assertion_failure=1/0;}", ""),
            Default::default(),
        )
        .unwrap()
        .request;
        assert_eq!(
            scheduled
                .dash_directions
                .values()
                .copied()
                .collect::<Vec<_>>(),
            vec![GameDashDirection::Back, GameDashDirection::Front]
        );
        for (id, value) in crate::characters::source_choices::assumptions(
            &scheduled.actors[1].character_id,
            "dash",
            false,
        ) {
            assert_eq!(
                scheduled.model_assumptions.get(&id),
                Some(&value),
                "{owner}: {id}"
            );
        }
    }
}

#[test]
fn source_invalid_direction_values_and_unsupported_owners_do_not_leave_orphaned_inputs() {
    for (value, message) in [
        ("2", "must be true or false"),
        ("1/2", "must be true or false"),
        ("1", "no owner Back Dodge binding"),
    ] {
        let result = run(&format!(
            "options duration=1 crit=random;
            sakiri char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
            target hp=100000000 def=0 resist=0; active sakiri;
            let flag={value}; sakiri dash[back=flag];"
        ));
        assert!(!result.report.execution_completed);
        assert!(result.request.actions.is_empty());
        assert!(result.request.dash_directions.is_empty());
        let diagnostic = result.diagnostic.unwrap();
        assert!(diagnostic.message.contains(message), "{diagnostic:?}");
        assert_eq!(diagnostic.loc.line, 4);
        assert!(diagnostic.loc.length > 0);
    }
}

#[test]
fn source_closed_direction_flags_diagnose_exact_utf16_literal_and_operator_ranges() {
    for (value, highlighted) in [("2", "2"), ("1/2", "/"), ("\"back\"", "\"back\"")] {
        let source = format!(
            "options duration=1 crit=random; // 😀
            mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
            target hp=100000000 def=0 resist=0; active mint;
            mint dash[back={value}];"
        );
        let artifact = crate::dsl::source::compile_response(&crate::dsl::compiler::SourceRequest {
            source: source.clone(),
            limits: Default::default(),
        });
        assert!(artifact.compiled.is_none());
        let diagnostic = &artifact.diagnostics[0];
        assert_eq!(
            diagnostic.code,
            crate::dsl::validation::DiagnosticCode::Configuration
        );
        assert!(
            diagnostic
                .message
                .contains("dash back must be true or false"),
            "{diagnostic:?}"
        );
        assert_eq!(diagnostic.loc.line, 4);
        let span = source
            .encode_utf16()
            .skip(diagnostic.loc.offset)
            .take(diagnostic.loc.length)
            .collect::<Vec<_>>();
        assert_eq!(String::from_utf16(&span).unwrap(), highlighted);
    }
}
