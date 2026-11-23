use super::*;

#[test]
pub(crate) fn configured_fireflies_amount_cap_and_exact_expiry_are_honored() {
    let mut req = research_request();
    req.actors[0].static_stats.crit_rate = Ratio {
        numerator: 1,
        denominator: 1,
    };
    req.actors[0].gear_procs = vec![GearProcConfig::FirefliesCritDamage {
        crit_damage_per_stack: Ratio {
            numerator: 1,
            denominator: 4,
        },
        max_stacks: 2,
        duration_us: 25,
    }];
    req.actions.truncate(5); // hits at 0,10,20,30,40; per-stack deadlines 25,35,...
    req.action_metadata
        .retain(|index, _| *index < req.actions.len());
    req.relative_duration_us = 65;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let damages: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|hit| hit["rounded_damage"].as_u64().unwrap())
        .collect();
    assert_eq!(damages, vec![800, 900, 1000, 900, 900]);
    assert_eq!(report["final_actor_states"][0]["fireflies_stacks"], 0);
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["event_kind"] == "fireflies_stack_capped")
    );
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let mut req = research_request();
    req.actors[0].static_stats.crit_rate = Ratio {
        numerator: 1,
        denominator: 1,
    };
    req.actors[0].gear_procs = vec![GearProcConfig::FirefliesCritDamage {
        crit_damage_per_stack: Ratio {
            numerator: 1,
            denominator: 4,
        },
        max_stacks: 1,
        duration_us: 10,
    }];
    req.actions.truncate(2);
    req.action_metadata
        .retain(|index, _| *index < req.actions.len());
    req.relative_duration_us = 20;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hit_receipts"][1]["rounded_damage"], 800); // first stack expires at exact hit timestamp
}

fn team_hit_fixture() -> (
    crate::simulation::GameScenarioRequest,
    crate::simulation::metadata::PacketMetadata,
) {
    let mut req = research_request();
    // research_request clears gear procs; restore the sample Fireflies proc:
    // 2/25 per stack, cap 7, 10s.
    req.actors[0].gear_procs = vec![GearProcConfig::FirefliesCritDamage {
        crit_damage_per_stack: Ratio {
            numerator: 2,
            denominator: 25,
        },
        max_stacks: 7,
        duration_us: 10_000_000,
    }];
    let mut teammate = req.actors[0].clone();
    teammate.slot = 1;
    teammate.gear_procs.clear();
    req.actors.push(teammate);
    // the Fireflies wearer (actor 0) stays off-field while the teammate acts
    req.active_actor_index = 1;
    req.relative_duration_us = 30_000_000;
    let packet = crate::simulation::metadata::PacketMetadata {
        damage_effect_id: "test-anima-packet".into(),
        native_ga_name: None,
        native_damage_type_ex: "EExecutionDamageTypeEX::DAMAGE_TYPE_NATURE".into(),
        native_damage_source_category: None,
        damage_kind: ntesim_core::combat::DamageKind::Elemental,
        element: Some(crate::simulation::metadata::DamageElement::Nature),
        fixed_crit_rate: None,
        unbal_value: None,
        fixed_crit_rate_provenance: None,
        unbal_value_provenance: None,
        semantic_tags: BTreeSet::new(),
        semantic_provenance: None,
        source_evidence: None,
        provenance: crate::simulation::metadata::MetadataProvenance::AcceptedModelHypothesis {
            id: "test.synthetic_packet".into(),
            description: "Test-only synthetic Anima packet.".into(),
        },
        packet_kind: Default::default(),
    };
    (req, packet)
}

fn drive_team_hit(
    req: &crate::simulation::GameScenarioRequest,
    packet: &crate::simulation::metadata::PacketMetadata,
    source_actor_index: usize,
    time_us: u64,
    prepare: impl FnOnce(&mut crate::simulation::state::GameSimulatorState),
) -> (
    crate::simulation::state::GameSimulatorState,
    Vec<crate::simulation::GameEventReceipt>,
    BTreeMap<String, String>,
) {
    let mut state = crate::simulation::state::GameSimulatorState::new(req);
    prepare(&mut state);
    state.advance_unpaused_test_clock(time_us);
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0u64;
    let mut trace = Vec::new();
    let mut execution_assumptions = BTreeMap::new();
    // The runner only reaches on_team_hit for connected (nonzero-damage) hits;
    // see hit/effects.rs. The synthetic packet stands in for such a hit here.
    crate::sets::fireflies::hit::on_team_hit(
        &mut crate::runtime_api::CallbackContext {
            state: &mut state,
            request: req,
            queue: &mut queue,
            sequence: &mut sequence,
            trace: &mut trace,
        },
        source_actor_index,
        packet,
        time_us,
        &mut execution_assumptions,
    )
    .unwrap();
    while let Some(scheduled) = queue.pop() {
        let crate::simulation::events::GameEventData::Content(event) = scheduled.data else {
            panic!("expected a content event");
        };
        crate::callbacks::apply(
            event,
            &mut crate::runtime_api::CallbackContext {
                state: &mut state,
                request: req,
                queue: &mut queue,
                sequence: &mut sequence,
                trace: &mut trace,
            },
        )
        .unwrap();
    }
    (state, trace, execution_assumptions)
}

#[test]
pub(crate) fn team_anima_hit_grants_one_stack_to_each_living_wearer_including_off_field() {
    let (req, packet) = team_hit_fixture();
    // source is the teammate (actor 1), not the wearer
    let (state, trace, execution_assumptions) = drive_team_hit(&req, &packet, 1, 5_000_000, |_| {});
    // the off-field wearer admits exactly one stack; the teammate admits none
    assert_eq!(crate::sets::fireflies::stack_count(&state.buffs, 0), 1);
    assert_eq!(crate::sets::fireflies::stack_count(&state.buffs, 1), 0);
    let stacks = crate::sets::fireflies::stacks(&state.buffs, 0).unwrap();
    assert_eq!(stacks.len(), 1);
    assert_eq!(stacks[0].expires_at_us, 15_000_000);
    assert_eq!(
        stacks[0].crit_damage_bonus,
        crate::simulation::ExactFraction::new(2, 25).unwrap()
    );
    // the stack feeds the crit_damage_bonus modifier key summed into the CRIT DMG zone
    // even while the wearer is not the active actor
    let living: BTreeSet<usize> = [0, 1].into_iter().collect();
    let modifiers = state
        .buffs
        .get_stat_modifiers_for_actor_with_living_sources(0, 1, 5_000_000, &living)
        .unwrap();
    assert_eq!(
        modifiers.get("crit_damage_bonus").copied(),
        Some(Ratio {
            numerator: 2,
            denominator: 25
        })
    );
    assert!(trace.iter().any(|event| {
        event.event_kind == "fireflies_crit_stack"
            && event.to_actor == Some(0)
            && event
                .data
                .as_ref()
                .is_some_and(|d| d.admitted == Some(true))
    }));
    assert_eq!(
        execution_assumptions[crate::sets::fireflies::hit::TEAM_HIT_HYPOTHESIS_ID.as_str()],
        crate::sets::fireflies::hit::TEAM_HIT_HYPOTHESIS_ID.description()
    );
    assert_eq!(
        execution_assumptions[crate::sets::fireflies::hit::ALL_IN_RANGE_HYPOTHESIS_ID.as_str()],
        crate::sets::fireflies::hit::ALL_IN_RANGE_HYPOTHESIS_ID.description()
    );
}

#[test]
pub(crate) fn team_trigger_filters_off_element_dead_source_and_never_double_grants() {
    let (req, packet) = team_hit_fixture();
    // the wearer dealing the hit grants itself exactly one stack, not two
    let (state, _, _) = drive_team_hit(&req, &packet, 0, 5_000_000, |_| {});
    assert_eq!(crate::sets::fireflies::stack_count(&state.buffs, 0), 1);
    // a defeated source grants nothing
    let (state, trace, _) = drive_team_hit(&req, &packet, 1, 5_000_000, |state| {
        state.actors[1].hp = 0;
    });
    assert_eq!(crate::sets::fireflies::stack_count(&state.buffs, 0), 0);
    assert!(
        !trace
            .iter()
            .any(|event| event.event_kind == "fireflies_crit_stack")
    );
    // an off-element packet grants nothing
    let mut chaos_packet = packet;
    chaos_packet.element = Some(crate::simulation::metadata::DamageElement::Chaos);
    let (state, trace, _) = drive_team_hit(&req, &chaos_packet, 1, 5_000_000, |_| {});
    assert_eq!(crate::sets::fireflies::stack_count(&state.buffs, 0), 0);
    assert!(
        !trace
            .iter()
            .any(|event| event.event_kind == "fireflies_crit_stack")
    );
}

fn multi_dummy_fixture(
    count: usize,
) -> (
    crate::simulation::GameScenarioRequest,
    crate::simulation::metadata::PacketMetadata,
) {
    let (mut req, packet) = team_hit_fixture();
    let template = req.targets.remove(0);
    for index in 0..count {
        let mut target = template.clone();
        target.target_index = index;
        req.targets.push(target);
    }
    (req, packet)
}

#[test]
pub(crate) fn team_anima_hit_admits_connected_hits_on_each_dummy() {
    let (req, packet) = multi_dummy_fixture(3);
    let (state, _, execution_assumptions) = drive_team_hit(&req, &packet, 1, 5_000_000, |_| {});
    // the off-field wearer admits exactly one stack; the teammate admits none
    assert_eq!(crate::sets::fireflies::stack_count(&state.buffs, 0), 1);
    assert_eq!(crate::sets::fireflies::stack_count(&state.buffs, 1), 0);
    let stacks = crate::sets::fireflies::stacks(&state.buffs, 0).unwrap();
    assert_eq!(stacks.len(), 1);
    assert_eq!(stacks[0].expires_at_us, 15_000_000);
    assert_eq!(
        stacks[0].crit_damage_bonus,
        crate::simulation::ExactFraction::new(2, 25).unwrap()
    );
    assert_eq!(
        execution_assumptions[crate::sets::fireflies::hit::TEAM_HIT_HYPOTHESIS_ID.as_str()],
        crate::sets::fireflies::hit::TEAM_HIT_HYPOTHESIS_ID.description()
    );
    assert_eq!(
        execution_assumptions[crate::sets::fireflies::hit::ALL_IN_RANGE_HYPOTHESIS_ID.as_str()],
        crate::sets::fireflies::hit::ALL_IN_RANGE_HYPOTHESIS_ID.description()
    );
}
