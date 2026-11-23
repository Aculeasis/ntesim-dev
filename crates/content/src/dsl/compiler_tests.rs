use super::compiler::*;
use crate::simulation::GameActionCommand;

const PREFIX: &str = "options duration=20; mint char lvl=80/80 awakening=0 talent=8,8,8 qte=8; target hp=10000000 def=1080 resist=0.2; active mint;";

#[test]
fn authored_flat_intensities_preserve_exact_independent_stat_zones() {
    use ntesim_core::formula::Ratio;
    let compile = |additions: &str| {
        compile_scenario(
            &format!("{PREFIX} {additions} mint attack;"),
            CompileLimits::default(),
        )
        .unwrap()
        .request
        .actors[0]
            .static_stats
            .clone()
    };
    let baseline = compile("");
    let stats = compile("mint add stats ci=100.25 bi=100.25; mint add stats ci=67.75 bi=68.25;");
    assert_eq!(
        stats.cycle_intensity,
        Ratio {
            numerator: 168,
            denominator: 1
        }
    );
    let bi = stats.break_intensity;
    let base = baseline.break_intensity;
    assert_eq!(
        2 * (bi.numerator * base.denominator - base.numerator * bi.denominator),
        337 * bi.denominator * base.denominator
    );
    assert_eq!(stats.atk, baseline.atk);
    assert_eq!(stats.damage_bonus, baseline.damage_bonus);
    assert!(
        compile_scenario(
            &format!("{PREFIX} mint add stats ci%=1;"),
            CompileLimits::default()
        )
        .is_err()
    );
}

#[test]
fn fadia_public_burst_spends_full_native_pool_and_shortage_is_atomic() {
    for energy in [100, 119, 120] {
        let source = format!(
            "options duration=12 iteration=1 crit=fixed;
            fadia char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy={energy};
            fadia add weapon=fork_Arachne refine=1;
            target hp=100000000 def=0 resist=0; active fadia; fadia burst;"
        );
        let compiled = compile_scenario(&source, CompileLimits::default()).unwrap();
        let GameActionCommand::InvokeAbility { energy_cost, .. } = compiled.request.actions[0]
        else {
            panic!("public Burst must compile to owner ability");
        };
        assert_eq!(energy_cost, 120);
        let report = crate::simulation::run_game_scenario(
            &compiled.request,
            &ntesim_core::execution::ExecutionLimits::default(),
        )
        .unwrap();
        assert_eq!(report["execution_completed"], energy == 120);
        assert_eq!(
            report["final_actor_states"][0]["energy"],
            if energy == 120 { 0 } else { energy }
        );
        assert_eq!(
            report["final_actor_states"][0]["spent_energy"],
            if energy == 120 { 120 } else { 0 }
        );
        assert_eq!(report["hits_settled"], if energy == 120 { 6 } else { 0 });
        assert_eq!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["event_kind"] == "eternal_waltz_mental")
                .count(),
            usize::from(energy == 120)
        );
    }
}

#[test]
fn authored_break_profile_runs_continuing_break_and_recovery_cycles() {
    let compiled = compile_scenario(
        "options iteration=1 duration=8 crit=fixed;
         mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
         target hp=100000000 def=0 resist=0 break_gauge=1/1000 max_break_gauge=1/1000 broken_duration=0.1;
         active mint; mint attack; wait(2); mint attack; wait(2); mint attack;",
        CompileLimits::default(),
    ).unwrap();
    let report = crate::simulation::run_game_scenario(
        &compiled.request,
        &ntesim_core::execution::ExecutionLimits::default(),
    )
    .unwrap();
    assert_eq!(report["execution_completed"], true);
    let trace = report["trace"].as_array().unwrap();
    let times = |kind: &str| {
        trace
            .iter()
            .filter(|event| event["event_kind"] == kind)
            .map(|event| event["time_us"].as_u64().unwrap())
            .collect::<Vec<_>>()
    };
    let broken = times("target_broken");
    let recovered = times("target_break_recovered");
    assert_eq!(broken.len(), 3, "{report}");
    assert_eq!(
        recovered,
        broken.iter().map(|time| time + 100_000).collect::<Vec<_>>()
    );
    assert_eq!(
        report["final_target_states"][0]["break_state"]["gauge"],
        serde_json::json!("1/1000")
    );
}

#[test]
fn break_efficiency_is_not_a_user_build_stat() {
    for stat in ["be", "break_efficiency", "UnbalAccrueEfficiencyBase"] {
        let error = compile_scenario(
            &format!("{PREFIX}mint add stats {stat}=0.25; mint attack;"),
            CompileLimits::default(),
        )
        .unwrap_err();
        assert!(
            error
                .iter()
                .any(|diagnostic| diagnostic.message.contains("Unsupported combat stat")),
            "{error:?}"
        );
    }
}

fn compile(rotation: &str) -> CompiledScript {
    compile_scenario(&format!("{PREFIX}{rotation}"), CompileLimits::default()).unwrap()
}

#[test]
fn rust_source_compilation_expands_exact_control_flow_and_keeps_source_locations() {
    let compiled = compile(
        "let n=0; for let i=0; i<4; i=i+1 { if i==1 {continue;} n=n+1; wait(0.1+0.2); } repeat n { mint attack; }",
    );
    assert_eq!(compiled.request.actions.len(), 6);
    assert_eq!(compiled.action_locations.len(), 6);
    assert!(matches!(
        compiled.request.actions[2],
        GameActionCommand::Wait {
            timestamp_us: 600_000,
            duration_us: 300_000
        }
    ));
    assert!(matches!(
        compiled.request.actions[3],
        GameActionCommand::InvokeAbility {
            timestamp_us: 900_000,
            ..
        }
    ));
    assert!(matches!(
        compiled.request.actions[5],
        GameActionCommand::InvokeAbility {
            timestamp_us: 2_500_000,
            ..
        }
    ));
}

#[test]
fn nonprogress_loops_and_invalid_assignments_report_source_diagnostics() {
    for rotation in [
        "while 1 {}",
        "let x=true;",
        "wait(0.0000001);",
        "mint dash[critical=2];",
        "mint dash[hold=1];",
        "if .actor.skill.ready { mint skill; }",
    ] {
        assert!(
            compile_scenario(
                &format!("{PREFIX}{rotation}"),
                CompileLimits {
                    max_steps: 100,
                    max_actions: 100
                }
            )
            .is_err(),
            "{rotation}"
        );
    }
    assert!(validate_script(&format!("{PREFIX}mint attack;")).is_empty());
    assert!(!validate_script(&PREFIX.replace("talent=8,8,8", "talent=0,8,8")).is_empty());
}

#[test]
fn authored_hold_is_an_exact_duration_bound_to_each_ability_command() {
    let source = "options duration=8; nanally char lvl=80/80 awakening=0 talent=8,8,8 qte=8;
        target hp=100000000 def=1080 resist=0; active nanally;
        let held=3000ms; nanally attack[hold=held]:2;";
    let compiled = compile_scenario(source, Default::default()).unwrap();
    assert_eq!(compiled.request.hold_inputs.len(), 2);
    assert_eq!(compiled.request.actions[1].timestamp_us(), 3_000_000);
    assert!(
        compiled
            .request
            .hold_inputs
            .values()
            .all(|input| input.duration_us == 3_000_000
                && input.reserved_stocks.is_none()
                && !input.moving)
    );
    for value in ["true", "-1", "0.0000001s"] {
        let invalid = source.replace("hold=held", &format!("hold={value}"));
        let diagnostics = compile_scenario(&invalid, Default::default()).unwrap_err();
        assert!(diagnostics[0].loc.line > 0);
        assert!(diagnostics[0].loc.column > 0);
    }
}

#[test]
fn source_energy_gates_and_live_branch_keep_native_program_boundaries() {
    let compiled = compile("wait_for_energy(10); mint attack; wait_for_energy(20); mint attack;");
    let program = compiled.live_program.unwrap();
    assert_eq!(program.energy_waits.len(), 2);
    assert_eq!(program.energy_waits[0].after_action_index, 0);
    assert_eq!(program.energy_waits[1].after_action_index, 2);
    assert_eq!(program.energy_waits[1].timestamp_us, 800_000);
    let compiled = compile("if .target.hp < 999999 {wait(0.1);} else {wait(0.3);} mint attack;");
    let program = compiled.live_program.unwrap();
    assert_eq!(program.branches[0].after_action_index, 1);
    assert_eq!(program.branches[0].wait_if_below_us, 100_000);
    assert_eq!(program.branches[0].wait_otherwise_us, 300_000);
}

#[test]
fn public_studio_sources_compile_entirely_in_rust() {
    for source in [
        include_str!("../../../../presets/studio/s1.nte"),
        include_str!("../../../../presets/studio/s2.nte"),
        include_str!("../../../../presets/studio/s3.nte"),
        include_str!("../../../../presets/studio/s4.nte"),
        include_str!("../../../../presets/studio/s5.nte"),
        include_str!("../../../../presets/studio/language-reference.nte"),
    ] {
        let compiled = compile_scenario(
            source,
            CompileLimits {
                max_steps: 100_000,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(!compiled.request.actions.is_empty());
        assert_eq!(
            compiled.request.actions.len(),
            compiled.action_locations.len()
        );
    }
}
