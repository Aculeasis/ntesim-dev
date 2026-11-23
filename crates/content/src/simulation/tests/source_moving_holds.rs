use crate::simulation::{PreparedSourceScenario, SourceScenarioReport};

const SOURCE: &str =
    "options duration=25 crit=random seed=42 stamina_capacity=100 initial_stamina=100
    stamina_use_efficiency=1 stamina_recovery_rate=0;
    mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
    baicang char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_hp=1000;
    target hp=100000000 def=0 resist=0;
    target hp=100000000 def=0 resist=0; active mint;
    let moving=1;
    repeat 2 {
        baicang dash;
        baicang attack[moving=moving,hold=0.2];
        moving=1-moving;
        wait(2);
    }
    baicang attack;
    if .baicang.resources.words!=1 || .baicang.hp!=985 {let assertion_failure=1/0;}
    if .team.stamina!=70 {let assertion_failure=1/0;}";

fn run(source: &str) -> SourceScenarioReport {
    PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap()
}

#[test]
fn source_moving_heavy_inputs_select_native_branches_and_keep_two_paid_cycles() {
    let result = run(SOURCE);
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    let holds = result
        .request
        .hold_inputs
        .iter()
        .map(|(index, input)| (*index, input.duration_us, input.moving))
        .collect::<Vec<_>>();
    assert_eq!(holds, vec![(2, 200_000, true), (5, 200_000, false)]);
    for (packet, count) in [
        ("GE_Player_Cang_Branch2_Damage", 2),
        ("GE_Player_Cang_Branch2_1_Damage", 6),
        ("GE_Player_Cang_Branch1_Damage", 2),
        ("GE_Player_Cang_Branch1_1_Damage", 6),
    ] {
        assert_eq!(
            result
                .report
                .hit_receipts
                .iter()
                .filter(|hit| hit.damage_effect_id == packet)
                .count(),
            count,
            "{packet}"
        );
    }
    // The two heavies retain two connected ordinary inputs, independently of
    // their sixteen AoE deliveries. Only the following third input grants a
    // Word and pays1.5% of the low current1000 HP once.
    assert_eq!(result.report.final_actor_states[1].hp, 985);
    super::assert_native_replay(&result);
    let scheduled = crate::dsl::compiler::compile_scenario(
        &SOURCE
            .replace("if .team.stamina!=70 {let assertion_failure=1/0;}", "")
            .replace(
                "if .baicang.resources.words!=1 || .baicang.hp!=985 {let assertion_failure=1/0;}",
                "",
            ),
        Default::default(),
    )
    .unwrap()
    .request;
    assert_eq!(
        scheduled
            .hold_inputs
            .values()
            .map(|input| (input.duration_us, input.moving))
            .collect::<Vec<_>>(),
        vec![(200_000, true), (200_000, false)]
    );
}

#[test]
fn source_moving_flags_require_hold_and_native_dodge_permission() {
    let prefix = "options duration=5 crit=random;
        baicang char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active baicang;";
    for (params, message) in [
        ("moving=true", "moving requires an explicit hold duration"),
        ("hold=0.2,moving=2", "moving must be true or false"),
    ] {
        let artifact = crate::dsl::source::compile_response(&crate::dsl::compiler::SourceRequest {
            source: format!("{prefix} baicang attack[{params}];"),
            limits: Default::default(),
        });
        assert!(artifact.compiled.is_none());
        assert!(
            artifact.diagnostics[0].message.contains(message),
            "{:?}",
            artifact.diagnostics
        );
    }
    let rejected = run(&format!("{prefix} baicang attack[hold=0.2,moving=true];"));
    assert!(!rejected.report.execution_completed);
    assert!(
        rejected
            .diagnostic
            .unwrap()
            .message
            .contains("moving heavy requires a preceding admitted dash")
    );
}

#[test]
fn source_remaining_neutral_holds_use_owner_packets_and_continuing_native_state() {
    for (owner, duration, packet, count, first_delay) in [
        ("chaos", "1", "GE_Player_Chaos_Branch1_Damage", 4, 1_166_691),
        (
            "daffodil",
            "0.000001",
            "GE_Player_Daffodill_Branch5_Damage",
            2,
            104_054,
        ),
        (
            "haniel",
            "0.2",
            "GE_Player_Haniel_Branch_Damage1",
            2,
            1_061_263,
        ),
    ] {
        // Haniel's native appear is a queued Normal1. An authored idle ends
        // that queue before a new neutral Cannon, rather than charging Normal2.
        let entry = if owner == "haniel" {
            "haniel swap; wait(1us);"
        } else {
            ""
        };
        let source = format!(
            "options duration=30 crit=random;
            mint char lvl=80/80 awakening=0 talent=8,8,8 qte=8;
            {owner} char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0;
            target hp=100000000 def=0 resist=0; active mint;
            {entry} repeat 2 {{{owner} attack[hold={duration}]; wait(3);}}"
        );
        let result = run(&source);
        assert!(
            result.report.success && result.report.execution_completed,
            "{owner}: {:?}; {:?}",
            result.diagnostic,
            result.report.halt_diagnostic
        );
        assert_eq!(result.request.hold_inputs.len(), 2);
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
        let first = result
            .report
            .hit_receipts
            .iter()
            .find(|hit| hit.damage_effect_id == packet)
            .unwrap();
        assert_eq!(
            first.time_us,
            result.request.actions[first.command_index.unwrap()].timestamp_us() + first_delay,
            "{owner}"
        );
        super::assert_native_replay(&result);
    }
}
