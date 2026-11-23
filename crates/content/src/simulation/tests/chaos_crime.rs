use super::chaos_actions::{make_action, request as base_request};
use super::*;
use serde_json::{Value, json};

pub(super) fn request(targets: usize) -> GameScenarioRequest {
    let mut req = base_request();
    let model = crate::HypothesisId::ChaosOrdinaryCrimeV1;
    req.model_assumptions
        .insert(model.as_str().into(), model.description().into());
    req.actors[0].initial_energy = 0;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    for index in 1..targets {
        let mut target = req.targets[0].clone();
        target.target_index = index;
        req.targets.push(target);
    }
    for target in &mut req.targets {
        target.hp_current = 100_000_000;
        target.hp_max = 100_000_000;
    }
    req
}

pub(super) fn chain(req: &mut GameScenarioRequest, time: &mut u64) {
    // Authored legal full recoveries, rounded to microseconds. This resource
    // prerequisite still uses the declared caller clock for packet delivery.
    for recovery in [518_862, 1_174_976, 545_853, 2_018_381, 1_072_842] {
        let mut command = make_action(
            *time,
            0,
            "GA_Chaos071_Melee",
            "GE_Player_Chaos_Melee1_Damage",
            0,
            0,
        );
        if let GameActionCommand::InvokeAbility {
            recovery_us,
            hit_delay_us,
            ..
        } = &mut command
        {
            *recovery_us = recovery;
            *hit_delay_us = 0;
        }
        req.actions.push(command);
        *time += recovery;
    }
}

fn run(req: &GameScenarioRequest) -> Value {
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}

fn actor(report: &Value) -> &Value {
    &report["final_actor_states"][0]
}
fn crime(report: &Value) -> &Value {
    &actor(report)["fractional_resources"]["chaos.crime"]
}

#[test]
fn chaos_crime_and_cycle_preserve_four_continuing_twelve_packet_chains_with_aoe() {
    for targets in [1, 3] {
        let mut req = request(targets);
        req.actions.clear();
        req.relative_duration_us = 25_000_000;
        let mut time = 0;
        for _ in 0..4 {
            chain(&mut req, &mut time);
        }
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 48 * targets);
        // Per chain: 22+33+15+2*28+35+2*13+19+2*41+94 =382.
        assert_eq!(crime(&report)["current"], json!("1000/1"));
        assert_eq!(crime(&report)["granted"], json!("1528/1"));
        assert_eq!(crime(&report)["overflow"], json!("528/1"));
        assert_eq!(crime(&report)["conserved"], true);
        // Independent native channels: 4*18.202 charge versus 4*30.299 Cycle.
        assert_eq!(actor(&report)["energy"], 72);
        assert_eq!(actor(&report)["charge_remainder"], json!("101/125"));
        assert_eq!(actor(&report)["cycle"]["current"], 100);
        assert_eq!(actor(&report)["cycle"]["overflow"], 21);
        assert_eq!(actor(&report)["cycle"]["carry"], json!("49/250"));
        assert_eq!(actor(&report)["cycle"]["conserved"], true);
    }
}

#[test]
fn chaos_crime_a1_is_exact_and_uses_warrant_without_multiplying_by_recipients() {
    let mut req = request(3);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([1]).unwrap();
    req.actions = vec![make_action(
        0,
        0,
        "GA_Chaos071_Skill",
        "GE_Player_Chaos_Skill1_Damage",
        0,
        0,
    )];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 6);
    assert_eq!(crime(&report)["current"], json!("140/1"));
    assert_eq!(actor(&report)["cycle"]["current"], 12);
    assert_eq!(actor(&report)["cycle"]["carry"], json!("1/5"));
    assert_eq!(actor(&report)["energy"], 7);
    assert_eq!(actor(&report)["charge_remainder"], json!("3/10"));
    req.actions.push(make_action(
        1_100_000,
        0,
        "GA_Chaos071_Melee",
        "GE_Player_Chaos_Melee1_Damage",
        0,
        0,
    ));
    assert_eq!(crime(&run(&req))["current"], json!("854/5"));
}

#[test]
fn chaos_resources_fall_through_dead_target_zero_and_require_positive_hp() {
    let mut req = request(2);
    req.actions.clear();
    req.targets[0].hp_current = 1;
    let mut time = 0;
    chain(&mut req, &mut time);
    let report = run(&req);
    assert_eq!(crime(&report)["current"], json!("382/1"));
    assert_eq!(actor(&report)["energy"], 18);
    assert_eq!(actor(&report)["cycle"]["current"], 30);
    for target in &mut req.targets {
        target.resistance = Ratio::one();
    }
    let report = run(&req);
    assert_eq!(crime(&report)["granted"], json!("0/1"));
    assert_eq!(actor(&report)["energy"], 0);
}

#[test]
fn chaos_cycle_efficiency_does_not_scale_crime_or_ultimate_charge() {
    let mut req = request(1);
    req.actions.clear();
    req.actors[0].cycle_gain_efficiency = Ratio {
        numerator: 3,
        denominator: 2,
    };
    let mut time = 0;
    chain(&mut req, &mut time);
    let report = run(&req);
    assert_eq!(crime(&report)["current"], json!("382/1"));
    assert_eq!(actor(&report)["energy"], 18);
    assert_eq!(actor(&report)["cycle"]["current"], 45);
    assert_eq!(actor(&report)["cycle"]["carry"], json!("897/2000"));
}

fn one_packet_receipt(
    effect: &str,
    order: &[(usize, u64)],
    initial: u128,
) -> ntesim_core::kernel::state::ExactPoolSummary {
    let mut req = request(2);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([1]).unwrap();
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    state.actors[0].character.chaos_mut().unwrap().crime = Some(
        ntesim_core::kernel::state::ExactPool::new(
            ExactFraction::new(1000, 1).unwrap(),
            ExactFraction::new(initial, 1).unwrap(),
        )
        .unwrap(),
    );
    crate::simulation::state::apply_target_status(
        &mut state.buffs,
        crate::simulation::state::TargetStatusDeclaration {
            source_actor: 0,
            source: "character:1071".into(),
            target_index: 1,
            id: "chaos.warrant".into(),
            now_us: 0,
            duration_us: 15_000_000,
            reductions: BTreeMap::new(),
        },
    )
    .unwrap();
    let packet = crate::simulation::metadata::resolve_requested_action_packet(
        &req.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        "character:1071",
        "GA_Chaos071_Melee",
        effect,
        None,
        &req.model_assumptions,
    )
    .unwrap();
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    let mut assumptions = BTreeMap::new();
    let mut control_applications = Vec::new();
    for &(target_index, applied_hp_loss) in order {
        crate::characters::chaos::resources::on_hit(&mut crate::runtime_api::ActorHitContext {
            hit_index: 0,
            packet_group_id: Some("test:ordinary:1"),
            authored_command_index: Some(0),
            autonomous_event_index: None,
            target_alive: true,
            control_applications: &mut control_applications,
            applied_hp_loss,
            active_actor_index: 0,
            command_index: 0,
            request: &req,
            execution_assumptions: &mut assumptions,
            action_metadata: &packet.action,
            target_level: 80,
            queue: &mut queue,
            sequence: &mut sequence,
            config: &req.actors[0],
            actor_index: 0,
            ability_id: "GA_Chaos071_Melee",
            damage_effect_id: effect,
            world_time_us: 0,
            time_us: 0,
            target_index,
            target_airborne: false,
            buffs: &mut state.buffs,
            actor: &mut state.actors[0],
            trace: &mut trace,
        })
        .unwrap();
    }
    state.actors[0]
        .character
        .chaos_mut()
        .unwrap()
        .crime
        .as_ref()
        .unwrap()
        .summary()
        .unwrap()
}

#[test]
fn chaos_a1_uses_any_connected_marked_recipient_once_with_order_independent_overflow() {
    for initial in [0, 990] {
        let unmarked_first = one_packet_receipt(
            "GE_Player_Chaos_Melee1_Damage",
            &[(0, 1), (1, 1), (1, 1)],
            initial,
        );
        let marked_first = one_packet_receipt(
            "GE_Player_Chaos_Melee1_Damage",
            &[(1, 1), (0, 1), (0, 1)],
            initial,
        );
        assert_eq!(unmarked_first, marked_first);
        assert_eq!(unmarked_first.granted, ExactFraction::new(154, 5).unwrap());
        assert!(unmarked_first.conserved);
        if initial == 990 {
            assert_eq!(unmarked_first.current, ExactFraction::new(1000, 1).unwrap());
            assert_eq!(unmarked_first.overflow, ExactFraction::new(104, 5).unwrap());
        }
    }
    // A marked recipient that applies zero HP cannot upgrade an unmarked hit.
    assert_eq!(
        one_packet_receipt("GE_Player_Chaos_Melee1_Damage", &[(0, 1), (1, 0)], 0).current,
        ExactFraction::new(22, 1).unwrap()
    );
}

#[test]
fn chaos_transformed_stage2_crime_follows_native_ge_names_instead_of_coefficient_size() {
    assert_eq!(
        one_packet_receipt("GE_Player_Chaos_Melee2_Damage_Ultra", &[(0, 1)], 0).current,
        ExactFraction::new(15, 1).unwrap()
    );
    assert_eq!(
        one_packet_receipt("GE_Player_Chaos_Melee2_1_Damage_Ultra", &[(0, 1)], 0).current,
        ExactFraction::new(33, 1).unwrap()
    );
}
