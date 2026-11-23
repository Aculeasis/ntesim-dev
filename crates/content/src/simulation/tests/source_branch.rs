use crate::simulation::{PreparedSourceScenario, SourceScenarioReport};

const PREFIX: &str =
    "options duration=30 crit=random seed=42 stamina_capacity=100 initial_stamina=100
    stamina_use_efficiency=1 stamina_recovery_rate=0;
    hathor char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0;
    target hp=100000000 def=0 resist=0; target hp=100000000 def=0 resist=0; active hathor;";

fn run(source: &str) -> SourceScenarioReport {
    PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap()
}
fn completed(result: &SourceScenarioReport) {
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
}

#[test]
fn source_hathor_branch_keeps_launchable_and_fallback_routes_across_continuing_cycles() {
    let source = format!(
        "{PREFIX} repeat 2 {{
        hathor attack[hold=1.5,launchable=true,landing=2];
        hathor attack[landing=3,launchable=0,height_bonus=0.5,hold=1.5];
        wait(1);
    }} if .team.stamina!=60 {{let assertion_failure=1/0;}}"
    );
    let result = run(&source);
    completed(&result);
    assert_eq!(result.request.branch_inputs.len(), 4);
    let hits = &result.report.hit_receipts;
    assert_eq!(
        hits.iter()
            .filter(|hit| hit.damage_effect_id == "GE_Player_Hathor_Branch_Damage")
            .count(),
        4
    );
    assert_eq!(
        hits.iter()
            .filter(|hit| hit.damage_effect_id == "GE_Player_Hathor_AirAttack_Damage")
            .count(),
        4
    );
    assert_eq!(hits[0].time_us, 216_746);
    assert_eq!(hits[2].time_us, 2_000_000);
    assert_eq!(
        result.report.combat_stamina.as_ref().unwrap().total_spent,
        crate::simulation::ExactFraction::new(40, 1).unwrap()
    );
    assert_eq!(
        result.request.branch_inputs[&0].height_bonus,
        crate::formula::Ratio::zero()
    );
    assert_eq!(
        result.request.branch_inputs[&1].height_bonus,
        crate::formula::Ratio {
            numerator: 1,
            denominator: 2
        }
    );
    super::assert_native_replay(&result);
    let exported = crate::dsl::compiler::compile_scenario(
        &source.replace("if .team.stamina!=60 {let assertion_failure=1/0;}", ""),
        Default::default(),
    )
    .unwrap()
    .request;
    assert_eq!(
        exported.branch_inputs.values().collect::<Vec<_>>(),
        result.request.branch_inputs.values().collect::<Vec<_>>()
    );
}

#[test]
fn source_hathor_swap_starts_no_attack_and_authored_hold_binds_to_attack() {
    let source = PREFIX.replace(
        "active hathor;",
        "mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; active mint;",
    );
    let result = run(&format!(
        "{source} hathor swap[hold=0.6,launchable=true,landing=2];"
    ));
    completed(&result);
    assert!(matches!(
        result.request.actions[0],
        crate::simulation::GameActionCommand::SwitchActor { .. }
    ));
    assert!(result.request.hold_inputs.contains_key(&0));
    assert!(result.request.branch_inputs.contains_key(&0));
    assert!(result.report.hit_receipts.is_empty());
    super::assert_native_replay(&result);
    let source = format!("{source} hathor attack[hold=1.5,launchable=true,landing=2];");
    let implicit = run(&source);
    completed(&implicit);
    assert!(!implicit.request.branch_inputs.contains_key(&0));
    assert!(implicit.request.branch_inputs.contains_key(&1));
    assert!(implicit.request.hold_inputs.contains_key(&1));
    super::assert_native_replay(&implicit);
    let no_op = run(&format!(
        "{PREFIX} hathor swap[hold=0.6,launchable=true,landing=2];"
    ));
    completed(&no_op);
    assert!(no_op.report.hit_receipts.is_empty());
    super::assert_native_replay(&no_op);
}

#[test]
fn source_branch_shape_types_and_owner_gate_keep_rejected_commands_bound() {
    for (params, message) in [
        ("hold=1.5,launchable=true", "requires landing"),
        ("landing=2,launchable=true", "requires hold"),
        ("hold=1.5,landing=2", "requires launchable"),
        ("hold=1.5,landing=0,launchable=true", "must be positive"),
        ("hold=1.5,landing=2,launchable=0.5", "true or false"),
        (
            "hold=1.5,landing=2,launchable=true,height_bonus=2",
            "within0..1",
        ),
    ] {
        let result = crate::dsl::source::compile_response(&crate::dsl::compiler::SourceRequest {
            source: format!("{PREFIX} hathor attack[{params}];"),
            limits: Default::default(),
        });
        assert!(result.compiled.is_none());
        assert!(
            result.diagnostics[0].message.contains(message),
            "{:?}",
            result.diagnostics
        );
    }
    let result = run(&format!(
        "{PREFIX} hathor attack[hold=0.6,launchable=true,landing=2];"
    ));
    assert!(!result.report.execution_completed);
    assert_eq!(result.request.actions.len(), 1);
    assert_eq!(result.request.branch_inputs.len(), 1);
    assert!(result.request.branch_inputs.contains_key(&0));
    assert!(result.report.hit_receipts.is_empty());
    assert!(result.diagnostic.unwrap().message.contains("gate"));
    let result = run(&format!(
        "{PREFIX} hathor attack[hold=0.5,launchable=true,landing=2];"
    ));
    assert!(!result.report.execution_completed);
    assert!(result.request.actions.is_empty());
    assert!(result.request.branch_inputs.is_empty());
    assert!(
        result
            .diagnostic
            .unwrap()
            .message
            .contains("strictly beyond")
    );
}

#[test]
fn source_hathor_native_held_skill_preserves_both_complete_finite_cycles() {
    let result = run(&format!(
        "{PREFIX} for let i=0; i<2; i=i+1 {{
        wait_until(i*17); hathor skill[hold=5];
        if .hathor.resources.power {{let assertion_failure=1/0;}}
    }}"
    ));
    completed(&result);
    assert_eq!(result.report.hit_receipts.len(), 108);
    assert_eq!(result.report.hit_receipts[0].time_us, 936_667);
    assert_eq!(result.report.hit_receipts[54].time_us, 17_936_667);
    assert_eq!(result.request.hold_inputs.len(), 2);
    super::assert_native_replay(&result);
}
