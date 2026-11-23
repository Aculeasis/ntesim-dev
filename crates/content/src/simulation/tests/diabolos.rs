use super::*;
use crate::simulation::GameActionCommand;
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use serde_json::json;

fn chaos_fixture() -> GameScenarioRequest {
    let mut req = crate::simulation::tests::research_request();
    req.stochastic_crit = None;
    req.relative_duration_us = 100;
    req.actors[0].set_id = Some("set:Suit1".into());
    let (proc, _) =
        crate::sets::diabolos::projection::compile(ntesim_data::stat_projection::frozen_tables())
            .unwrap();
    req.actors[0].gear_procs = vec![proc];
    for metadata in req.action_metadata.values_mut() {
        metadata.damage_type_ex = Some("EExecutionDamageTypeEX::DAMAGE_TYPE_CHAOS".into());
    }
    req
}

fn run(req: &GameScenarioRequest) -> Value {
    let limits = ExecutionLimits::default();
    let packets = crate::simulation::validation::validate_and_resolve(req, &limits).unwrap();
    let (mut execution, assumptions) = PreparedExecution::new(req, req.seed, &[], &[]).unwrap();
    execution
        .process_events(req, &limits, &packets, &[], &[])
        .unwrap();
    serde_json::to_value(execution.into_report(req, req.seed, assumptions).unwrap()).unwrap()
}

fn participation_fixture(
    other: &str,
    hit_times: &[u64],
    refresh_at: Option<u64>,
) -> GameScenarioRequest {
    let mut req = chaos_fixture();
    req.relative_duration_us = 31_000_000;
    req.actors[0].character_id = "character:1004".into();
    req.actors[0].initial_cycle = 100;
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = other.into();
    ally.initial_cycle = if refresh_at.is_some() { 100 } else { 0 };
    ally.set_id = None;
    ally.gear_procs.clear();
    req.actors.push(ally);
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].hp_max = 100_000_000;
    let action = req.actions[0].clone();
    let metadata = req.action_metadata[&0].clone();
    req.actions = vec![
        GameActionCommand::SwitchActor {
            timestamp_us: 0,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: refresh_at.unwrap_or(1_400_000),
            target_actor_index: 0,
            cooldown_us: 0,
        },
    ];
    req.action_metadata.clear();
    for time in hit_times {
        let mut hit = action.clone();
        if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut hit {
            *timestamp_us = *time;
        }
        req.action_metadata
            .insert(req.actions.len(), metadata.clone());
        req.actions.push(hit);
    }
    req
}

fn direct_damages(report: &Value) -> Vec<Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "caller-effect")
        .map(|hit| hit["non_crit_damage"].clone())
        .collect()
}

#[test]
fn diabolos_nova_and_scorch_participation_replace_ignore_and_expire_exactly() {
    for other in ["character:1042", "character:1033"] {
        let req = participation_fixture(other, &[19_999_999, 20_000_000, 20_000_001], None);
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        // R=-1/25 gives F_res=1+1/26=27/26 under standard_rational:
        // 500*27/26=6750/13, then 500*(1-.08)=460 after expiry.
        // Adding .12+.24 would instead yield 16500/29.
        assert_eq!(
            direct_damages(&report),
            [(6750, 13), (460, 1), (460, 1)].map(|(n, d)| json!(format!("{}/{}", n, d)))
        );
        assert_eq!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["event_kind"] == "reaction_equipment_window")
                .count(),
            1
        );
    }
}

#[test]
fn diabolos_incoming_participant_does_not_need_reaction_ownership() {
    let mut req = participation_fixture("character:1042", &[2_000_000], None);
    req.actors.swap(0, 1);
    for (index, actor) in req.actors.iter_mut().enumerate() {
        actor.slot = index;
        actor.initial_cycle = if index == 0 { 100 } else { 0 };
    }
    if let GameActionCommand::SwitchActor {
        target_actor_index, ..
    } = &mut req.actions[1]
    {
        *target_actor_index = 1; // same-actor switch must not retrigger
    }
    if let GameActionCommand::InvokeAbility { actor_index, .. } = &mut req.actions[2] {
        *actor_index = 1;
    }
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(direct_damages(&report), [json!("6750/13")]);
    let nova = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["damage_effect_id"] == "ntesim.reaction.nova")
        .unwrap();
    assert_eq!(
        nova["actor_index"], 0,
        "outgoing CI tie winner does not wear Diabolos"
    );
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "reaction_equipment_window")
            .count(),
        1
    );
}

#[test]
fn diabolos_second_cycle_refreshes_one_window_and_other_elements_stay_unchanged() {
    let mut req = participation_fixture(
        "character:1042",
        &[11_000_000, 29_999_999, 30_000_000],
        Some(10_000_000),
    );
    req.action_metadata.get_mut(&2).unwrap().damage_type_ex =
        Some("EExecutionDamageTypeEX::DAMAGE_TYPE_NATURE".into());
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(
        direct_damages(&report),
        [(400, 1), (6750, 13), (460, 1)].map(|(n, d)| json!(format!("{}/{}", n, d)))
    );
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "reaction_equipment_window")
            .count(),
        2
    );
}

#[test]
fn diabolos_unqualified_switch_does_not_create_a_window() {
    let mut req = participation_fixture("character:1042", &[2_000_000], None);
    req.actors[0].initial_cycle = 99;
    let report = run(&req);
    assert_eq!(direct_damages(&report), [json!("460/1")]);
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_kind"] == "reaction_equipment_window")
    );
}

#[test]
fn diabolos_grants_both_participants_but_not_a_third_equipped_actor_or_blossom() {
    let mut req = participation_fixture("character:1042", &[2_000_000], None);
    // Include Blackbird's child end at10.2s while the20s equipment windows live.
    req.relative_duration_us = 11_000_000;
    req.actors[1].set_id = req.actors[0].set_id.clone();
    req.actors[1].gear_procs = req.actors[0].gear_procs.clone();
    let mut third = req.actors[0].clone();
    third.slot = 2;
    third.character_id = "character:1033".into();
    third.initial_cycle = 0;
    req.actors.push(third);
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let buffs = crate::simulation::tests::support::final_buffs(&report);
    let windows: Vec<_> = buffs
        .iter()
        .filter(|buff| buff["id"] == "diabolos.reaction_window")
        .collect();
    assert_eq!(windows.len(), 2);
    for (owner, window) in windows.iter().enumerate() {
        assert_eq!(window["owner_actor"], owner);
        assert_eq!(window["stacks"], 1);
        assert_eq!(window["duration"]["expires_at_us"], 20_000_000);
    }
    let mut blossom = participation_fixture("character:1019", &[2_000_000], None);
    blossom.actors[0].character_id = "character:1046".into();
    let report = run(&blossom);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_kind"] == "reaction_equipment_window")
    );
}

#[test]
fn diabolos_participation_failure_publishes_no_partial_buff_or_receipt() {
    let mut req = participation_fixture("character:1042", &[], None);
    req.actors[1].set_id = req.actors[0].set_id.clone();
    req.actors[1].gear_procs = req.actors[0].gear_procs.clone();
    if let crate::simulation::GearProcConfig::DiabolosChaosIgnore { duration_us, .. } =
        &mut req.actors[1].gear_procs[0]
    {
        *duration_us = u64::MAX;
    }
    // The public validation boundary rejects this before any switch admission.
    let error =
        crate::simulation::validate_game_request(&req, &ExecutionLimits::default()).unwrap_err();
    assert!(error.contains("deadline overflow"));
    // The private dispatcher also prepares both recipients transactionally.
    let mut state = GameSimulatorState::new(&req);
    state.advance_unpaused_test_clock(1);
    let before = state.execution_model_assumptions.clone();
    let mut queue = BinaryHeap::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    let result = crate::gear::on_reaction_participation(
        &mut crate::runtime_api::CallbackContext {
            state: &mut state,
            request: &req,
            queue: &mut queue,
            sequence: &mut sequence,
            trace: &mut trace,
        },
        [0, 1],
        &ntesim_data::reaction_values::ReactionResult::Nova,
    );
    assert!(result.is_err());
    assert!(state.buffs.instances().is_empty());
    assert_eq!(state.execution_model_assumptions, before);
    assert!(trace.is_empty());
    assert!(queue.is_empty());
}

#[test]
fn diabolos_reaction_parameters_require_present_ratios_and_positive_duration() {
    for (key, value) in [
        ("Chaos2_4_IgnoreChaos2", json!(0)),
        ("Chaos2_4_IgnoreChaos2", json!(24)),
        ("Chaos2_4_CD", json!(0)),
    ] {
        let mut tables = ntesim_data::stat_projection::frozen_tables().clone();
        tables.insert(
            "CT_Equipmentbuff".into(),
            ntesim_data::set_parameters::overlay(),
        );
        tables.get_mut("CT_Equipmentbuff").unwrap()["rows"][key]["Keys"][0]["Value"] = value;
        assert!(crate::sets::diabolos::projection::compile(&tables).is_err());
    }
    let mut tables = ntesim_data::stat_projection::frozen_tables().clone();
    let mut overlay = ntesim_data::set_parameters::overlay();
    overlay["rows"]
        .as_object_mut()
        .unwrap()
        .remove("Chaos2_4_IgnoreChaos2");
    tables.insert("CT_Equipmentbuff".into(), overlay);
    assert!(crate::sets::diabolos::projection::compile(&tables).is_err());
}

#[test]
fn diabolos_four_piece_ignores_chaos_resistance_only_for_chaos_packets() {
    let report = run(&chaos_fixture());
    // 1000 * 1/2 * F_res: 0.20 Chaos RES minus the frozen 0.12 ignore gives
    // 0.08, so each hit is 500 * 23/25 = 460 instead of 500 * 4/5 = 400.
    assert_eq!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|hit| hit["non_crit_damage"].clone())
            .collect::<Vec<_>>(),
        [460, 460, 460, 460, 460, 460].map(|n| json!(format!("{}/{}", n, 1)))
    );
    assert_eq!(
        report["resolved_model_assumptions"][crate::sets::diabolos::HYPOTHESIS_ID.as_str()],
        crate::sets::diabolos::HYPOTHESIS_ID.description()
    );
    // A non-Chaos packet from the same wearer keeps the untouched resistance.
    let mut req = chaos_fixture();
    req.action_metadata.get_mut(&3).unwrap().damage_type_ex =
        Some("EExecutionDamageTypeEX::DAMAGE_TYPE_NATURE".into());
    let report = run(&req);
    assert_eq!(report["hit_receipts"][3]["non_crit_damage"], json!("400/1"));
    assert_eq!(report["hit_receipts"][2]["non_crit_damage"], json!("460/1"));
}

#[test]
fn diabolos_ignore_joins_target_resistance_statuses_additively_until_expiry() {
    let req = chaos_fixture();
    let limits = ExecutionLimits::default();
    let packets = crate::simulation::validation::validate_and_resolve(&req, &limits).unwrap();
    let (mut execution, assumptions) = PreparedExecution::new(&req, req.seed, &[], &[]).unwrap();
    crate::simulation::state::apply_target_status(
        &mut execution.state.buffs,
        crate::simulation::state::TargetStatusDeclaration {
            source_actor: 0,
            source: "test.resistance_shred".into(),
            target_index: 0,
            id: "test.chaos_res_shred".into(),
            now_us: 0,
            duration_us: 30,
            reductions: BTreeMap::from([(
                "ECharacterElementType::CHARACTER_ELEMENT_TYPE_CHAOS".into(),
                crate::simulation::ExactFraction::new(1, 20).unwrap(),
            )]),
        },
    )
    .unwrap();
    execution
        .process_events(&req, &limits, &packets, &[], &[])
        .unwrap();
    let report =
        serde_json::to_value(execution.into_report(&req, req.seed, assumptions).unwrap()).unwrap();
    // Shared reduction zone: 0.20 - 0.12 (wearer ignore) - 0.05 (status) = 0.03,
    // so 500 * 97/100 = 485 until the status expires at t=30, then 460 again.
    assert_eq!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|hit| hit["non_crit_damage"].clone())
            .collect::<Vec<_>>(),
        [485, 485, 485, 460, 460, 460].map(|n| json!(format!("{}/{}", n, 1)))
    );
}

#[test]
fn diabolos_pending_hit_keeps_off_field_wearer_ignore() {
    let mut req = chaos_fixture();
    req.actions.truncate(1);
    req.action_metadata.retain(|index, _| *index == 0);
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.set_id = None;
    ally.gear_procs.clear();
    req.actors.push(ally);
    req.actions
        .push(crate::simulation::GameActionCommand::SwitchActor {
            timestamp_us: 10,
            target_actor_index: 1,
            cooldown_us: 0,
        });
    if let crate::simulation::GameActionCommand::InvokeAbility { hit_delay_us, .. } =
        &mut req.actions[0]
    {
        *hit_delay_us = 20;
    }
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hit_receipts"][0]["non_crit_damage"], json!("460/1"));
    assert_eq!(report["hit_receipts"][0]["actor_index"], 0);
    assert_eq!(report["final_buff_state"]["active_actor_index"], 1);
}

#[test]
fn diabolos_projection_binds_the_frozen_curve_and_validates_native_equipment() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    let (proc, native) = crate::sets::diabolos::projection::compile(tables).unwrap();
    assert_eq!(
        serde_json::to_value(&proc).unwrap(),
        json!({"kind":"diabolos_chaos_ignore","chaos_res_ignore":"3/25",
            "powered_chaos_res_ignore":"6/25", "duration_us":20000000})
    );
    assert_eq!(
        native["parameters"]["Chaos2_4_IgnoreChaos"]["selected"],
        json!({"numerator":"3","denominator":"25"})
    );
    let mut actor = chaos_fixture().actors.remove(0);
    actor.gear_procs.clear();
    let equipped =
        json!({"set_id":"set:Suit1","set_count":4,"unresolved_set_buffs":[{"threshold":4}]});
    let project = crate::sets::diabolos::projection::project_equipped;
    assert!(project(tables, &mut actor, &equipped).unwrap().is_some());
    assert!(project(tables, &mut actor, &equipped).unwrap().is_some());
    assert_eq!(actor.gear_procs.len(), 1);
    actor.gear_procs.push(proc);
    assert!(project(tables, &mut actor, &equipped).is_err());
    actor.gear_procs.clear();
    let two_piece = json!({"set_id":"set:Suit1","set_count":2,"unresolved_set_buffs":[]});
    assert!(project(tables, &mut actor, &two_piece).unwrap().is_none());
    assert!(
        project(
            tables,
            &mut actor,
            &json!({"set_id":"set:Suit1","set_count":4,"unresolved_set_buffs":[]})
        )
        .is_err()
    );
    for value in [json!(-0.12), json!(12)] {
        let mut changed = tables.clone();
        changed.insert(
            "CT_Equipmentbuff".into(),
            ntesim_data::set_parameters::overlay(),
        );
        changed.get_mut("CT_Equipmentbuff").unwrap()["rows"]["Chaos2_4_IgnoreChaos"]["Keys"][0]["Value"] =
            value;
        assert!(crate::sets::diabolos::projection::compile(&changed).is_err());
    }
    let mut invalid = chaos_fixture();
    invalid.actors[0].set_id = None;
    assert!(
        crate::simulation::validate_game_request(&invalid, &ExecutionLimits::default())
            .unwrap_err()
            .contains("set:Suit1")
    );
}
