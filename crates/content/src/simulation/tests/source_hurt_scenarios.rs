//! Tests for periodic seeded hurt, explicit dummy declarations, incoming source/basis,
//! actor interruptions, combat transitions, and time stop integration.

use crate::dsl::compiler::{CompileLimits, compile_scenario};
use crate::dsl::validation::DiagnosticCode;
use crate::execution::ExecutionLimits;
use crate::simulation::runtime::PreparedSourceScenario;
use crate::simulation::timer_queue::EventQueue as BinaryHeap;

const BASE_SCENARIO: &str = "\
options duration=1 seed=42;
mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
mint add weapon=fork_lingganzhongjiezhe refine=1;
target hp=10000000 def=0 resist=0;
active mint;
";

#[test]
fn fixed_hurt_and_explicit_incoming_share_the_event_budget() {
    let source = format!(
        "{BASE_SCENARIO}\
incoming mint at=500ms damage=1;
hurt every interval=1s amount=1;
hurt every interval=1s amount=2;
wait(1);"
    );
    let mut limits = ExecutionLimits {
        max_events: 3,
        ..Default::default()
    };
    let program = PreparedSourceScenario::new(&source, CompileLimits::default(), &limits).unwrap();
    assert_eq!(program.request().incoming_damage_events.len(), 3);

    limits.max_events = 2;
    let diagnostics = PreparedSourceScenario::new(&source, CompileLimits::default(), &limits)
        .err()
        .unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, DiagnosticCode::EventLimit);
    assert_eq!(diagnostics[0].loc.line, 8);
}

#[test]
fn reseeded_hurt_checks_the_incoming_event_budget() {
    let source = format!(
        "{BASE_SCENARIO}\
incoming mint at=500ms damage=1;
hurt every interval=400ms,800ms amount=1;
wait(1);"
    );
    let limits = ExecutionLimits {
        max_events: 2,
        ..Default::default()
    };
    let program = PreparedSourceScenario::new(&source, CompileLimits::default(), &limits).unwrap();
    assert_eq!(program.request().incoming_damage_events.len(), 2);

    let error = program.run(9999).err().unwrap();
    assert_eq!(
        error,
        "Incoming damage exceeds the limit of 2 events. (line 7, column 1)"
    );
}

#[test]
fn periodic_hurt_with_fixed_interval_and_amount_delivers_to_active_actor() {
    let source = format!("{BASE_SCENARIO}hurt every interval=200ms amount=500; wait(1);");
    let compiled = compile_scenario(&source, CompileLimits::default()).unwrap();
    let report =
        crate::simulation::run_game_scenario(&compiled.request, &Default::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );

    let receipts = report["incoming_damage_receipts"].as_array().unwrap();
    assert_eq!(receipts.len(), 5);
    for (i, receipt) in receipts.iter().enumerate() {
        assert_eq!(receipt["time_us"], (i as u64 + 1) * 200_000);
        assert_eq!(receipt["actor_index"], 0);
        assert_eq!(receipt["delivered_damage"], 500);
        assert_eq!(receipt["actual_hp_loss"], 500);
    }
}

#[test]
fn periodic_hurt_delivery_time_active_actor_resolves_across_switches() {
    let source = "\
options duration=2 seed=42;
mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
mint add weapon=fork_lingganzhongjiezhe refine=1;
sakiri char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
sakiri add weapon=fork_mofeikesi refine=1;
target hp=10000000 def=0 resist=0;
active mint;
hurt every interval=500ms amount=1000 actor=active;
wait(0.7);
sakiri swap;
wait_until(2);
";
    let compiled = compile_scenario(source, CompileLimits::default()).unwrap();
    let report =
        crate::simulation::run_game_scenario(&compiled.request, &Default::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );

    let receipts = report["incoming_damage_receipts"].as_array().unwrap();
    assert_eq!(receipts.len(), 4);
    // At 500ms: Mint was active
    assert_eq!(receipts[0]["time_us"], 500_000);
    assert_eq!(receipts[0]["actor_index"], 0);

    // At 1000ms, 1500ms, 2000ms: Sakiri was active
    assert_eq!(receipts[1]["time_us"], 1_000_000);
    assert_eq!(receipts[1]["actor_index"], 1);

    assert_eq!(receipts[2]["time_us"], 1_500_000);
    assert_eq!(receipts[2]["actor_index"], 1);

    assert_eq!(receipts[3]["time_us"], 2_000_000);
    assert_eq!(receipts[3]["actor_index"], 1);
}

#[test]
fn periodic_hurt_with_ranges_draws_seeded_uniform_and_replays_consistently() {
    let source = "\
options duration=2 seed=42;
mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
mint add weapon=fork_lingganzhongjiezhe refine=1;
target hp=10000000 def=0 resist=0;
active mint;
hurt every interval=100ms,300ms amount=100,500;
wait(2);
";
    let limits = crate::execution::ExecutionLimits::default();
    let program = PreparedSourceScenario::new(source, CompileLimits::default(), &limits).unwrap();

    let report1 = program.run(42).unwrap();
    let report2 = program.run(42).unwrap();
    let report_diff = program.run(9999).unwrap();

    let r1_receipts = &report1.report.incoming_damage_receipts;
    let r2_receipts = &report2.report.incoming_damage_receipts;
    let r_diff_receipts = &report_diff.report.incoming_damage_receipts;

    assert!(!r1_receipts.is_empty());
    assert_eq!(
        r1_receipts, r2_receipts,
        "Same seed must produce identical hurt receipts"
    );

    // Different seed must produce different sampled intervals/amounts
    let r1_times: Vec<_> = r1_receipts.iter().map(|r| r.time_us).collect();
    let diff_times: Vec<_> = r_diff_receipts.iter().map(|r| r.time_us).collect();
    assert_ne!(
        r1_times, diff_times,
        "Different seed must produce different event times"
    );
}

#[test]
fn dummy_attack_start_and_proximity_trigger_hathor_gaze() {
    let source = "\
options duration=2 seed=42;
hathor char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
hathor add weapon=fork_TigerTally refine=1;
target hp=10000000 def=0 resist=0;
active hathor;
dummy t0 proximity at=100ms in_proximity=true;
dummy t0 attack_start at=500ms actor=hathor;
wait(1);
";
    let compiled = compile_scenario(source, CompileLimits::default()).unwrap();
    let report =
        crate::simulation::run_game_scenario(&compiled.request, &Default::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );

    let trace = report["trace"].as_array().unwrap();
    let dummy_events: Vec<_> = trace
        .iter()
        .filter(|entry| entry["event_kind"] == "dummy_combat")
        .collect();
    assert_eq!(dummy_events.len(), 2);
    assert_eq!(dummy_events[0]["time_us"], 100_000);
    assert_eq!(dummy_events[0]["phase"], "ModePause");
    assert_eq!(dummy_events[1]["time_us"], 500_000);
    assert_eq!(dummy_events[1]["phase"], "ProcCallback");
}

#[test]
fn incoming_with_dummy_source_and_before_defense_basis() {
    let source = "\
options duration=2 seed=42;
mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
mint add weapon=fork_lingganzhongjiezhe refine=1;
mint add stats def=1000;
target lvl=80 hp=10000000 def=0 resist=0;
active mint;
incoming mint at=500ms damage=1000 source=t0 basis=before_defense;
wait(1);
";
    let compiled = compile_scenario(source, CompileLimits::default()).unwrap();
    assert_eq!(
        compiled.request.incoming_damage_basis.get(&0),
        Some(&crate::simulation::GameIncomingDamageBasis::BeforeDefense)
    );
    assert_eq!(compiled.request.incoming_dummy_sources.get(&0), Some(&0));

    let report =
        crate::simulation::run_game_scenario(&compiled.request, &Default::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );

    let receipts = report["incoming_damage_receipts"].as_array().unwrap();
    assert_eq!(receipts.len(), 1);
    assert!(
        receipts[0]["defense_mitigation"].is_object(),
        "Expected defense mitigation receipt"
    );
    // 1000 delivered damage is mitigated by DEF
    assert!(receipts[0]["delivered_damage"].as_u64().unwrap() < 1000);
}

#[test]
fn explicit_actor_interrupt_is_lowered_and_settled() {
    let source = "\
options duration=2 seed=42;
chaos char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
chaos add weapon=fork_BitterCake refine=1;
target hp=10000000 def=0 resist=0;
active chaos;
interrupt chaos at=500ms;
wait(1);
";
    let compiled = compile_scenario(source, CompileLimits::default()).unwrap();
    assert_eq!(compiled.request.interrupt_events.len(), 1);
    assert_eq!(compiled.request.interrupt_events[0].timestamp_us, 500_000);
    assert_eq!(compiled.request.interrupt_events[0].actor_index, 0);

    let report =
        crate::simulation::run_game_scenario(&compiled.request, &Default::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["interrupt_receipts"].as_array().unwrap().len(), 1);
}

#[test]
fn explicit_combat_transitions_lower_and_drive_state() {
    let source = "\
options duration=5 seed=42;
blackbird char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
blackbird add weapon=fork_twinbirds refine=1;
target hp=10000000 def=0 resist=0;
active blackbird;
combat blackbird at=1s leaving=true;
combat blackbird at=3s leaving=false;
wait(4);
";
    let compiled = compile_scenario(source, CompileLimits::default()).unwrap();
    assert_eq!(compiled.request.combat_transition_events.len(), 2);
    assert_eq!(
        compiled.request.combat_transition_events[0].timestamp_us,
        1_000_000
    );
    assert!(compiled.request.combat_transition_events[0].leaving);
    assert_eq!(
        compiled.request.combat_transition_events[1].timestamp_us,
        3_000_000
    );
    assert!(!compiled.request.combat_transition_events[1].leaving);

    let report =
        crate::simulation::run_game_scenario(&compiled.request, &Default::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(
        report["combat_transition_receipts"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn time_stop_policies_classify_and_shift_incoming_and_dummy_schedules() {
    use crate::execution::CausalPriority;
    use crate::simulation::events::{GameEventData, ScheduledGameEvent};
    use ntesim_core::kernel::timing::{ModePauseKind, PauseOwner, RuntimeModeClock};

    let incoming = GameEventData::IncomingDamage {
        event_index: 0,
        actor_index: 0,
        delivered_damage: 100,
    };
    let dummy = GameEventData::DummyCombat { event_index: 0 };

    let mut clock = RuntimeModeClock::default();
    let mut queue = BinaryHeap::new();
    queue.push(ScheduledGameEvent {
        time_us: 3_000_000,
        priority: CausalPriority::HitDelivery,
        seq: 1,
        data: incoming,
    });
    queue.push(ScheduledGameEvent {
        time_us: 3_500_000,
        priority: CausalPriority::ProcCallback,
        seq: 2,
        data: dummy,
    });
    let owner = PauseOwner {
        owner: 0,
        generation: 1,
    };
    clock
        .start_pause(owner, ModePauseKind::ActiveStop, 1_000_000)
        .unwrap();
    queue.sync(&clock).unwrap();
    assert!(queue.peek().is_none());
    clock.end_pause(owner, 2_000_000).unwrap();
    queue.sync(&clock).unwrap();

    assert_eq!(queue.pop().unwrap().time_us, 4_000_000); // 3s + 1s shifted
    assert_eq!(queue.pop().unwrap().time_us, 4_500_000); // 3.5s + 1s shifted
}
