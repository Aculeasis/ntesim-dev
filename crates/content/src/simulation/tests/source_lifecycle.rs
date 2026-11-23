use super::*;

fn encode(value: &impl serde::Serialize) -> serde_json::Value {
    serde_json::to_value(value).unwrap()
}

#[test]
fn source_checkpoints_reconstruct_continuing_scheduler_rng_and_lexical_state() {
    let mint = include_str!("../../../../../examples/mint-native-cycles.nte");
    let hotori = "options duration=80 crit=random; hotori char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_timepiece=120;
        target hp=100000000 def=0 resist=0; active hotori;
        repeat 2 { let period=.time; hotori burst;
            while .hotori.time_stop.active {hotori attack[window=attack];}
            wait_until(period+40); }";
    let incoming = include_str!("../../../../../examples/incoming-shield-cycles.nte");
    for (source, chunk) in [(mint, 13), (hotori, 3), (incoming, 3)] {
        let program =
            PreparedSourceScenario::new(source, Default::default(), &Default::default()).unwrap();
        let full = program.run(42).unwrap();
        assert!(
            full.report.success && full.report.execution_completed,
            "{:?}",
            full.diagnostic
        );
        let mut saved = None;
        let mut start = 0;
        loop {
            let segment = program
                .run_controlled(42, saved.as_ref(), |boundary| {
                    Ok(boundary.completed_inputs >= start + chunk)
                })
                .unwrap();
            if let Some(checkpoint) = segment.checkpoint {
                assert!(!segment.execution.report.execution_completed);
                assert!(!segment.execution.report.game_dps_computed);
                assert!(segment.execution.report.invariants_passed);
                assert!(
                    segment
                        .execution
                        .occurrences
                        .iter()
                        .all(|input| input.completed_us.is_some())
                );
                start = checkpoint.completed_inputs;
                // Persist/reload the transport before each reconstruction.
                saved = Some(serde_json::from_value(encode(&checkpoint)).unwrap());
            } else {
                assert_eq!(encode(&segment.execution), encode(&full));
                break;
            }
        }
        assert!(start > 3);
    }
}

#[test]
fn source_checkpoint_rejects_changed_identity_prefix_and_unreached_boundaries() {
    let source = "options duration=10; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active mint; repeat 2 {mint attack; wait(1);}";
    let program =
        PreparedSourceScenario::new(source, Default::default(), &Default::default()).unwrap();
    let saved = program
        .run_controlled(42, None, |b| Ok(b.completed_inputs == 1))
        .unwrap()
        .checkpoint
        .unwrap();
    assert!(
        program
            .run_controlled(43, Some(&saved), |_| Ok(false))
            .unwrap_err()
            .contains("identity mismatch")
    );
    let mut limits = ExecutionLimits::default();
    limits.max_events -= 1;
    let changed = PreparedSourceScenario::new(source, Default::default(), &limits).unwrap();
    assert!(
        changed
            .run_controlled(42, Some(&saved), |_| Ok(false))
            .is_err()
    );
    let changed = PreparedSourceScenario::new(
        &source.replace("wait(1)", "wait(2)"),
        Default::default(),
        &Default::default(),
    )
    .unwrap();
    assert!(
        changed
            .run_controlled(42, Some(&saved), |_| Ok(false))
            .is_err()
    );
    let mut corrupted = saved.clone();
    corrupted.prefix_sha256 = "tampered".into();
    assert!(
        program
            .run_controlled(42, Some(&corrupted), |_| Ok(false))
            .unwrap_err()
            .contains("prefix mismatch")
    );
    corrupted = saved;
    corrupted.completed_inputs = 999;
    assert!(
        program
            .run_controlled(42, Some(&corrupted), |_| Ok(false))
            .unwrap_err()
            .contains("not reached")
    );
    // Stop before any input and before the tail drain, including a zero-length
    // script. Failures/event limits remain terminal and provide no checkpoint.
    let empty = PreparedSourceScenario::new(
        source.split("repeat").next().unwrap(),
        Default::default(),
        &Default::default(),
    )
    .unwrap();
    let zero = empty.run_controlled(42, None, |_| Ok(true)).unwrap();
    assert_eq!(zero.checkpoint.as_ref().unwrap().completed_inputs, 0);
    assert_eq!(
        encode(
            &empty
                .run_controlled(42, zero.checkpoint.as_ref(), |_| Ok(false))
                .unwrap()
                .execution
        ),
        encode(&empty.run(42).unwrap())
    );
    let failed = PreparedSourceScenario::new(
        &source.replace(
            "repeat 2 {mint attack; wait(1);}",
            "mint burst; mint burst;",
        ),
        Default::default(),
        &Default::default(),
    )
    .unwrap();
    let result = failed.run_controlled(42, None, |_| Ok(false)).unwrap();
    assert!(!result.execution.report.success);
    assert!(result.checkpoint.is_none());
    let limited = PreparedSourceScenario::new(
        source,
        Default::default(),
        &ExecutionLimits {
            max_events: 1,
            ..Default::default()
        },
    )
    .unwrap()
    .run_controlled(42, None, |_| Ok(false))
    .unwrap();
    assert!(!limited.execution.report.execution_completed);
    assert!(limited.execution.report.limit_reached.is_some());
    assert!(limited.checkpoint.is_none());
}
