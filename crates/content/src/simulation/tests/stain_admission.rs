use super::*;

#[test]
fn blackbird_nova_extension_preflights_model_and_both_generation_deadlines() {
    let source = "options duration=1;
        blackbird char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        lacrimosa char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=10000000 def=0 resist=0; active blackbird; wait(1);";
    let mut request = crate::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    let base = crate::HypothesisId::CombatEsperCycleReactionsV1;
    request
        .model_assumptions
        .insert(base.as_str().into(), base.description().into());
    let (mut execution, _) = PreparedExecution::new(&request, request.seed, &[], &[]).unwrap();
    execution
        .state
        .advance_unpaused_test_clock(u64::MAX - 8_000_000);
    let before = execution.state.execution_model_assumptions.clone();
    let error = execution.switch_actor(&request, &[], 0, 1, 0).unwrap_err();
    assert!(error.contains("deadline overflow"), "{error}");
    assert_eq!(execution.state.active_actor_index, 0);
    assert_eq!(execution.state.actors[0].cycle.current(), 100);
    assert_eq!(execution.state.execution_model_assumptions, before);
    assert!(execution.state.buffs.instances().is_empty());
    assert!(
        execution.state.actors[0]
            .character
            .blackbird_mut()
            .unwrap()
            .nova
            .is_empty()
    );
}

#[test]
fn stain_switch_model_or_deadline_failure_preserves_field_and_resources() {
    let source = "options duration=1; blackbird char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100; hathor char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=10000000 def=0 resist=0; active blackbird; wait(1);";
    let mut request = crate::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    for (character, _extension) in [
        ("character:1042", crate::HypothesisId::CombatStainSwitchV1),
        ("character:1046", crate::HypothesisId::CombatRemoraChargeV1),
    ] {
        request.actors[0].character_id = character.into();
        let (mut execution, _) = PreparedExecution::new(&request, request.seed, &[], &[]).unwrap();
        execution.state.advance_unpaused_test_clock(u64::MAX - 1);
        let before = execution.state.execution_model_assumptions.clone();
        let error = execution.switch_actor(&request, &[], 0, 1, 0).unwrap_err();
        assert!(error.contains("deadline overflow"), "{error}");
        assert_eq!(execution.state.active_actor_index, 0);
        assert_eq!(execution.state.actors[0].cycle.current(), 100);
        assert_eq!(execution.state.actors[0].switch_cooldown_until_us, 0);
        assert_eq!(execution.state.execution_model_assumptions, before);
        assert!(execution.state.buffs.instances().is_empty());
        assert!(
            execution
                .trace
                .iter()
                .all(|event| event.event_kind != "switch_actor")
        );
    }
}

#[test]
fn blackbird_b2_stain_preflight_rejects_missing_model_and_extended_deadline_without_mutation() {
    let source = "options duration=1;
        blackbird char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        hathor char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=10000000 def=0 resist=0; active blackbird; wait(1);";
    let mut request = crate::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    for id in [
        crate::HypothesisId::CombatEsperCycleReactionsV1,
        crate::HypothesisId::CombatStainSwitchV1,
    ] {
        request
            .model_assumptions
            .insert(id.as_str().into(), id.description().into());
    }
    let (mut execution, _) = PreparedExecution::new(&request, request.seed, &[], &[]).unwrap();
    execution
        .state
        .advance_unpaused_test_clock(u64::MAX - 15_000_000);
    let before = execution.state.execution_model_assumptions.clone();
    let error = execution.switch_actor(&request, &[], 0, 1, 0).unwrap_err();
    assert!(error.contains("deadline overflow"), "{error}");
    assert_eq!(execution.state.active_actor_index, 0);
    assert_eq!(execution.state.actors[0].cycle.current(), 100);
    assert_eq!(execution.state.execution_model_assumptions, before);
    assert!(execution.state.buffs.instances().is_empty());
}
