use super::*;
use ntesim_content::simulation::{self, GameScenarioProgramRequest, GameScenarioRequest};
use ntesim_core::execution::ExecutionLimits;
use serde::Serialize;
use serde_json::Value;

#[test]
fn wide_resource_transport_preserves_native_decimal_components_in_json_and_binary() {
    let source = "options duration=150 crit=fixed start_fons=0;
        chiz char lvl=80/80 awakening=2 talent=8,8,8 qte=8 start_energy=0;
        chiz add weapon=fork_mamen refine=1;
        target hp=100000000 def=1080 resist=0; active chiz; chiz attack;";
    let mut request = ntesim_content::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    for model in [
        ntesim_contracts::hypotheses::HypothesisId::ChizOrdinaryClockV1,
        ntesim_contracts::hypotheses::HypothesisId::ChizExactFinanceV1,
    ] {
        request
            .model_assumptions
            .insert(model.into(), model.description().into());
    }
    let native = ntesim_content::simulation::run_game_scenario(
        &request,
        &ntesim_core::execution::ExecutionLimits::default(),
    )
    .unwrap();
    let grain = &native["final_actor_states"][0]["fractional_resources"]["chiz.grain"];
    let granted = grain["granted"].as_str().unwrap();
    let (gn, gd) = granted.split_once('/').unwrap();
    assert!(!gn.is_empty() && gn.bytes().all(|b| b.is_ascii_digit()));
    assert!(!gd.is_empty() && gd.bytes().all(|b| b.is_ascii_digit()));
    assert_game_transport(&request, &native);
}

#[test]
fn combat_resource_deposits_preserve_decimal_components_in_json_and_binary() {
    let mut request = ntesim_content::dsl::compiler::compile_scenario(
        "options duration=3; chiz char lvl=80/80 awakening=5 talent=8,8,8 qte=8;
         chiz add weapon=fork_mamen refine=1;
         target hp=100000000 def=1080 resist=0; active chiz;",
        Default::default(),
    )
    .unwrap()
    .request;
    let model = ntesim_contracts::hypotheses::HypothesisId::ChizBattleEntryV1;
    request
        .model_assumptions
        .insert(model.into(), model.description().into());
    for (timestamp_us, leaving) in [(1_000_000, true), (2_000_000, false)] {
        request
            .combat_transition_events
            .push(simulation::GameActorCombatTransitionEvent {
                timestamp_us,
                actor_index: 0,
                leaving,
            });
    }
    let native = simulation::run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(native["execution_completed"], true);
    let deposit = &native["combat_transition_receipts"][1]["resource_deposits"]["chiz.grain"];
    for component in ["input", "repaid", "credited", "overflow"] {
        let fraction = deposit[component].as_str().unwrap();
        let (n, d) = fraction.split_once('/').unwrap();
        assert!(!n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()));
        assert!(!d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()));
    }
    assert_game_transport(&request, &native);
}

fn assert_game_transport(request: &GameScenarioRequest, native: &Value) {
    let input = serde_json::to_vec(request).unwrap();
    let json = dispatch_transport("simulate_game", &input, true).unwrap();
    let binary = dispatch_transport("simulate_game", &input, false).unwrap();
    assert_eq!(&serde_json::from_slice::<Value>(&json).unwrap(), native);
    use std::io::Write;
    use std::process::{Command, Stdio};
    let decoder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("js/wire.js");
    let mut child=Command::new("node").args(["-e",r#"
const assert=require('node:assert/strict');
const {parseLosslessJson}=require(require('node:path').join(require('node:path').dirname(process.argv[1]),'lossless-json.js'));
const {decodeWire}=require(process.argv[1]);
const {native,json,binary}=parseLosslessJson(require('node:fs').readFileSync(0,'utf8'));
assert.deepEqual(parseLosslessJson(Buffer.from(json).toString('utf8')),native);
assert.deepEqual(decodeWire(Uint8Array.from(binary)),native);
"#]).arg(decoder).stdin(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(
            &serde_json::to_vec(&serde_json::json!({"native":native,"json":json,"binary":binary}))
                .unwrap(),
        )
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn hathor_power_energy_and_pool_receipts_preserve_json_and_binary_transport() {
    let source = "options duration=3; hathor char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0; hathor add weapon=fork_wushoutieyu refine=1; hathor add cartridge=Suit5 count=4; target hp=1000000 def=0 resist=0; active hathor; hathor skill;";
    let mut request = ntesim_content::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    for model in [
        ntesim_contracts::hypotheses::HypothesisId::HathorOrdinaryClockV1,
        ntesim_contracts::hypotheses::HypothesisId::HathorTapPowerV1,
    ] {
        request
            .model_assumptions
            .insert(model.into(), model.description().into());
    }
    let report = simulation::run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true);
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["energy_change"]["source"] == "hathor_power")
    );
    assert_game_transport(&request, &report);
}

#[test]
fn hathor_gaze_dummy_observations_preserve_native_json_binary_and_replay() {
    let source = "options duration=12; hathor char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0; hathor add weapon=fork_wushoutieyu refine=1; target hp=1000000 def=0 resist=0; active hathor; hathor skill;";
    let mut req = ntesim_content::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    for model in [
        ntesim_contracts::hypotheses::HypothesisId::HathorOrdinaryClockV1,
        ntesim_contracts::hypotheses::HypothesisId::HathorTapPowerV1,
        ntesim_contracts::hypotheses::HypothesisId::HathorGazeV1,
    ] {
        req.model_assumptions
            .insert(model.into(), model.description().into());
    }
    req.dummy_combat_events = vec![
        simulation::GameDummyCombatEvent::Proximity {
            timestamp_us: 0,
            target_index: 0,
            in_proximity: true,
        },
        simulation::GameDummyCombatEvent::AttackStart {
            timestamp_us: 9_000_000,
            target_index: 0,
            actor_index: 0,
        },
    ];
    let report = simulation::run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true);
    assert_eq!(
        report["final_actor_states"][0]["resources"]["hathor.express_delivery_power"]["current"],
        3
    );
    assert_game_transport(&req, &report);
}

#[test]
fn hathor_held_and_dummy_incoming_provenance_preserve_json_binary_transport() {
    let source = "options duration=8; hathor char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0; hathor add weapon=fork_wushoutieyu refine=1; target hp=1000000 def=0 resist=0; active hathor; hathor skill;";
    let mut req = ntesim_content::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    for model in [
        ntesim_contracts::hypotheses::HypothesisId::HathorOrdinaryClockV1,
        ntesim_contracts::hypotheses::HypothesisId::HathorTapPowerV1,
        ntesim_contracts::hypotheses::HypothesisId::HathorGazeV1,
        ntesim_contracts::hypotheses::HypothesisId::HathorHeldSkillV1,
        ntesim_contracts::hypotheses::HypothesisId::CombatIncomingDamageFixtureV1,
    ] {
        req.model_assumptions
            .insert(model.into(), model.description().into());
    }
    req.hold_inputs.insert(
        0,
        simulation::GameHoldInput {
            duration_us: 5_000_000,
            moving: false,
            reserved_stocks: None,
        },
    );
    req.incoming_damage_events = vec![simulation::GameActorDamageEvent {
        timestamp_us: 5_500_000,
        actor_index: 0,
        delivered_damage: 1,
    }];
    req.incoming_dummy_sources.insert(0, 0);
    req.dummy_combat_events = vec![simulation::GameDummyCombatEvent::AttackStart {
        timestamp_us: 5_500_000,
        target_index: 0,
        actor_index: 0,
    }];
    let report = simulation::run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true);
    assert_eq!(report["hits_settled"], 27);
    assert_eq!(
        report["final_actor_states"][0]["resources"]["hathor.express_delivery_power"]["current"],
        3
    );
    assert_game_transport(&req, &report);
}

#[test]
fn incoming_defense_basis_and_sampled_exact_def_preserve_json_binary_transport() {
    let source = "options duration=2; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=1000000 def=0 resist=0; incoming mint at=1 damage=1000; active mint; wait(2);";
    let mut req = ntesim_content::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def = 1000;
    req.actors[0].static_stats.def_percent = ntesim_core::formula::Ratio::zero();
    req.actors[0].static_stats.def_flat = ntesim_core::formula::Ratio::zero();
    req.targets[0].level = 80;
    let model = ntesim_contracts::hypotheses::HypothesisId::CombatIncomingDefenseV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.incoming_dummy_sources.insert(0, 0);
    req.incoming_damage_basis
        .insert(0, simulation::GameIncomingDamageBasis::BeforeDefense);
    let report = simulation::run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true);
    assert_eq!(
        report["incoming_damage_receipts"][0]["delivered_damage"],
        519
    );
    assert_eq!(
        report["incoming_damage_receipts"][0]["defense_mitigation"]["recipient_defense"],
        serde_json::json!("1000/1")
    );
    assert_game_transport(&req, &report);
}

#[test]
fn hathor_burst_snapshot_and_expiry_preserve_json_binary_transport() {
    let source = "options duration=12; hathor char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=100; hathor add weapon=fork_wushoutieyu refine=1; target hp=1000000 def=0 resist=0; active hathor; hathor burst;";
    let mut req = ntesim_content::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    for model in [
        ntesim_contracts::hypotheses::HypothesisId::HathorOrdinaryClockV1,
        ntesim_contracts::hypotheses::HypothesisId::HathorTapPowerV1,
        ntesim_contracts::hypotheses::HypothesisId::HathorBurstEmergencyV1,
    ] {
        req.model_assumptions
            .insert(model.into(), model.description().into());
    }
    let report = simulation::run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true);
    assert_eq!(report["hits_settled"], 3);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
    assert_game_transport(&req, &report);
}

#[test]
fn hathor_paid_cyclone_and_restored_cooldown_preserve_json_binary_transport() {
    use ntesim_content::simulation::GameActionCommand;
    let source = "options duration=12; hathor char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=100; hathor add weapon=fork_wushoutieyu refine=1; target hp=1000000 def=1080 resist=0; active hathor; hathor burst;";
    let mut req = ntesim_content::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    for model in [
        ntesim_contracts::hypotheses::HypothesisId::HathorOrdinaryClockV1,
        ntesim_contracts::hypotheses::HypothesisId::HathorTapPowerV1,
        ntesim_contracts::hypotheses::HypothesisId::HathorBurstEmergencyV1,
        ntesim_contracts::hypotheses::HypothesisId::HathorCycloneV1,
    ] {
        req.model_assumptions
            .insert(model.into(), model.description().into());
    }
    let mut tap = req.actions[0].clone();
    if let GameActionCommand::InvokeAbility {
        ability_id,
        damage_effect_id,
        energy_cost,
        ..
    } = &mut tap
    {
        *ability_id = "GA_Hathor_Skill".into();
        *damage_effect_id = "GE_Player_Hathor_Skill1_Damage".into();
        *energy_cost = 0;
    }
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut req.actions[0] {
        *timestamp_us = 2_000_000;
    }
    req.actions.insert(0, tap.clone());
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut tap {
        *timestamp_us = 7_000_000;
    }
    req.actions.push(tap);
    let report = simulation::run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true);
    assert_eq!(report["hits_settled"], 13);
    assert_eq!(
        report["final_actor_states"][0]["fractional_resources"]["hathor.express_delivery_power"]["spent"],
        serde_json::json!("1/1")
    );
    assert_game_transport(&req, &report);
}

#[test]
fn hathor_combat_reentry_and_future_gaze_deadline_preserve_json_binary_transport() {
    use ntesim_content::simulation::{GameActionCommand, GameActorCombatTransitionEvent};
    use ntesim_contracts::hypotheses::HypothesisId;
    let source = "options duration=12; hathor char lvl=80/80 awakening=0 talent=8,8,8 qte=8; hathor add weapon=fork_wushoutieyu refine=1; target hp=1000000 def=1080 resist=0; active hathor; hathor skill;";
    let mut req = ntesim_content::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    for m in [
        HypothesisId::HathorOrdinaryClockV1,
        HypothesisId::HathorTapPowerV1,
        HypothesisId::HathorGazeV1,
        HypothesisId::HathorBurstEmergencyV1,
        HypothesisId::HathorCombatLifecycleV1,
    ] {
        req.model_assumptions
            .insert(m.into(), m.description().into());
    }
    let mut next = req.actions[0].clone();
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut next {
        *timestamp_us = 3_100_000;
    }
    req.actions.push(next);
    req.combat_transition_events = vec![
        GameActorCombatTransitionEvent {
            timestamp_us: 100_000,
            actor_index: 0,
            leaving: true,
        },
        GameActorCombatTransitionEvent {
            timestamp_us: 1_000_000,
            actor_index: 0,
            leaving: false,
        },
    ];
    let report = simulation::run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true);
    assert_eq!(report["hits_settled"], 3);
    assert_eq!(
        report["combat_transition_receipts"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["event_kind"] == "hathor_gaze_deadline"
                && e["data"]["expires_at_us"] == 16_000_000)
    );
    assert_game_transport(&req, &report);
}

#[test]
fn hathor_paid_response_and_qualified_support_preserve_json_binary_transport() {
    use ntesim_content::simulation::{GameActionCommand, PlayerInput};
    use ntesim_contracts::hypotheses::HypothesisId;
    let source =
        "options duration=5; haniel char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_cycle=100;
        haniel add weapon=fork_BitGame refine=1;
        hathor char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0;
        hathor add weapon=fork_wushoutieyu refine=1;
        target hp=1000000 def=1080 resist=0; active hathor; hathor attack;";
    let mut req = ntesim_content::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    for m in [
        HypothesisId::HathorOrdinaryClockV1,
        HypothesisId::HathorTapPowerV1,
        HypothesisId::HathorGazeV1,
        HypothesisId::HathorBurstEmergencyV1,
        HypothesisId::HathorCycloneV1,
        HypothesisId::HathorCombatLifecycleV1,
        HypothesisId::HathorSecondaryActionsV1,
    ] {
        req.model_assumptions
            .insert(m.into(), m.description().into());
    }
    req.active_actor_index = 0;
    let mut normal = req.actions.remove(0);
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut normal {
        *timestamp_us = 3_000_000;
    }
    req.actions = vec![
        GameActionCommand::SwitchActor {
            timestamp_us: 0,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::PlayerInput {
            timestamp_us: 2_000_000,
            actor_index: 1,
            input: PlayerInput::Dash,
            critical_success: true,
        },
        normal,
    ];
    req.combat_stamina = Some(
        serde_json::from_value(serde_json::json!({
            "capacity": "100/1",
            "initial": "100/1",
            "recovery_rate": "0/1",
            "use_efficiency": "1/1"
        }))
        .unwrap(),
    );
    let report = simulation::run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true);
    assert_eq!(report["hits_settled"], 4);
    assert_eq!(
        report["final_actor_states"][1]["fractional_resources"]["hathor.express_delivery_power"]["current"],
        serde_json::json!("1/1")
    );
    assert_game_transport(&req, &report);
}

#[test]
fn hathor_airattack_exact_stamina_and_early_landing_preserve_json_binary_transport() {
    use ntesim_contracts::hypotheses::HypothesisId;
    let source="options duration=5 stamina_capacity=100 initial_stamina=35 stamina_recovery_rate=0 stamina_use_efficiency=1;
        hathor char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0;
        hathor add weapon=fork_wushoutieyu refine=1;
        target hp=1000000 def=1080 resist=0; active hathor; hathor attack;";
    let mut req = ntesim_content::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    for m in [
        HypothesisId::HathorOrdinaryClockV1,
        HypothesisId::HathorTapPowerV1,
        HypothesisId::HathorGazeV1,
        HypothesisId::HathorBurstEmergencyV1,
        HypothesisId::HathorCycloneV1,
        HypothesisId::HathorCombatLifecycleV1,
        HypothesisId::HathorSecondaryActionsV1,
        HypothesisId::HathorAirattackV1,
    ] {
        req.model_assumptions
            .insert(m.into(), m.description().into());
    }
    if let simulation::GameActionCommand::InvokeAbility {
        damage_effect_id, ..
    } = &mut req.actions[0]
    {
        *damage_effect_id = "GE_Player_Hathor_AirAttack_Damage".into();
    }
    req.plunge_inputs.insert(
        0,
        simulation::GamePlungeInput {
            landing_delay_us: 4_000_000,
            height_bonus: ntesim_core::formula::Ratio {
                numerator: 1,
                denominator: 2,
            },
        },
    );
    let report = simulation::run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true);
    assert_eq!(report["hits_settled"], 1);
    assert_eq!(report["hit_receipts"][0]["time_us"], 1_733_084);
    assert_eq!(
        report["combat_stamina"]["total_spent"],
        serde_json::json!("30/1")
    );
    assert_game_transport(&req, &report);
}

#[test]
fn hathor_branch_fallback_preserves_exact_stamina_and_packets_in_json_binary_transport() {
    use ntesim_contracts::hypotheses::HypothesisId;
    let source = "options duration=6 stamina_capacity=100 initial_stamina=100 stamina_recovery_rate=0 stamina_use_efficiency=1;
        hathor char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0;
        hathor add weapon=fork_wushoutieyu refine=1;
        target hp=1000000 def=1080 resist=0; active hathor; hathor attack;";
    let mut req = ntesim_content::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    for m in [
        HypothesisId::HathorOrdinaryClockV1,
        HypothesisId::HathorTapPowerV1,
        HypothesisId::HathorGazeV1,
        HypothesisId::HathorBurstEmergencyV1,
        HypothesisId::HathorCycloneV1,
        HypothesisId::HathorCombatLifecycleV1,
        HypothesisId::HathorSecondaryActionsV1,
        HypothesisId::HathorAirattackV1,
        HypothesisId::HathorBranchV1,
    ] {
        req.model_assumptions
            .insert(m.into(), m.description().into());
    }
    req.hold_inputs.insert(
        0,
        simulation::GameHoldInput {
            duration_us: 1_500_000,
            moving: false,
            reserved_stocks: None,
        },
    );
    req.branch_inputs.insert(
        0,
        simulation::GameBranchInput {
            can_be_launched: false,
            landing_delay_us: 4_000_000,
            height_bonus: ntesim_core::formula::Ratio {
                numerator: 1,
                denominator: 2,
            },
        },
    );
    let report = simulation::run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true);
    assert_eq!(report["hits_settled"], 2);
    assert_eq!(report["hit_receipts"][1]["time_us"], 4_116_417);
    assert_eq!(
        report["combat_stamina"]["total_spent"],
        serde_json::json!("40/1")
    );
    assert_eq!(
        report["combat_stamina"]["current"],
        serde_json::json!("60/1")
    );
    for kind in ["hathor_branch_fallback", "hathor_branch_fallback_landing"] {
        assert!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .any(|event| event["event_kind"] == kind)
        );
    }
    assert_game_transport(&req, &report);
}

#[test]
fn atk_transfer_receipts_and_optional_dummy_attack_preserve_json_and_binary_transport() {
    let source = "options duration=7;
        haniel char lvl=80/80 awakening=0 talent=8,8,8 qte=8;
        haniel add weapon=fork_BitGame refine=1;
        lacrimosa char lvl=80/80 awakening=0 talent=8,8,8 qte=8;
        lacrimosa add weapon=fork_Rose refine=1;
        target hp=1000000 def=0 resist=0 original_atk=1000; active lacrimosa;";
    let mut request = ntesim_content::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    let model = ntesim_contracts::hypotheses::HypothesisId::HanielNovaExtractionV1;
    request
        .model_assumptions
        .insert(model.into(), model.description().into());
    request.actors[1].initial_cycle = 100;
    request
        .actions
        .push(simulation::GameActionCommand::SwitchActor {
            timestamp_us: 0,
            target_actor_index: 0,
            cooldown_us: 0,
        });
    request
        .combat_transition_events
        .push(simulation::GameActorCombatTransitionEvent {
            timestamp_us: 6_000_000,
            actor_index: 0,
            leaving: true,
        });
    let native = simulation::run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert!(native["atk_transfers"].is_array());
    assert!(native["final_target_states"][0]["attack"].is_string());
    assert_game_transport(&request, &native);
}

#[test]
fn transport_errors_and_forced_fallback_decode_in_both_consumers() {
    struct Failing;
    impl Serialize for Failing {
        fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("deliberate test failure"))
        }
    }
    let expected = WasmErrorResponse {
        status: "error".into(),
        action: None,
        error: "internal transport error serialization failed".into(),
    };
    assert_eq!(
        serialize_transport(&expected, true).unwrap(),
        TRANSPORT_ERROR_JSON
    );
    assert_eq!(
        serialize_transport(&expected, false).unwrap(),
        TRANSPORT_ERROR_BINARY
    );
    let frames: Vec<_> = [true, false]
        .into_iter()
        .flat_map(|json| {
            [
                transport("missing", "{}", json),
                serialize_transport_error(&Failing, json),
            ]
            .map(|bytes| (json, bytes))
        })
        .collect();
    for (json, bytes) in &frames {
        if *json {
            let value: Value = serde_json::from_slice(bytes).unwrap();
            assert_eq!(value["status"], "error");
            assert!(value["error"].as_str().is_some_and(|s| !s.is_empty()));
        }
    }
    use std::io::Write;
    use std::process::{Command, Stdio};
    let decoder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("js/wire.js");
    let mut child = Command::new("node").args(["-e", r#"
const assert = require('node:assert/strict');
const { decodeWire } = require(process.argv[1]);
const frames = JSON.parse(require('node:fs').readFileSync(0, 'utf8'));
for (const [json, bytes] of frames) {
  const value = json ? JSON.parse(Buffer.from(bytes).toString()) : decodeWire(Uint8Array.from(bytes));
  assert.equal(value.status, 'error');
  assert.equal(typeof value.error, 'string');
  assert(value.error.length > 0);
}
assert.deepEqual(JSON.parse(Buffer.from(frames[1][1]).toString()), decodeWire(Uint8Array.from(frames[3][1])));
assert.deepEqual(JSON.parse(Buffer.from(frames[0][1]).toString()), decodeWire(Uint8Array.from(frames[2][1])));
assert.equal(decodeWire(Uint8Array.from(frames[2][1])).action, 'missing');
"#]).arg(decoder).stdin(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&frames).unwrap())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn source_api_returns_compiled_schema_and_structured_diagnostics() {
    let source = include_str!("../../../presets/studio/s1.nte");
    let invoke = |action: &str, payload: Value| {
        dispatch_invoke(
            &serde_json::to_vec(&serde_json::json!({"action":action,"payload":payload})).unwrap(),
        )
        .unwrap()
        .data
    };
    let compiled = invoke("compile_script", serde_json::json!({"source":source}));
    assert!(compiled["diagnostics"].as_array().unwrap().is_empty());
    let native =
        ntesim_content::dsl::compiler::compile_scenario(source, Default::default()).unwrap();
    assert_eq!(compiled["compiled"], serde_json::to_value(native).unwrap());
    let inspected = invoke("inspect_script", serde_json::json!({"source":source}));
    assert_eq!(inspected["party"], compiled["party"]);
    assert_eq!(
        inspected["party"]["actors"][0]["character_id"],
        "character:1019"
    );
    assert_eq!(inspected["diagnostics"], serde_json::json!([]));
    let diagnostics = invoke(
        "validate_script",
        serde_json::json!({"source":"options duration=1; wait(1 2);"}),
    );
    assert_eq!(diagnostics[0]["code"], "syntax");
    assert!(diagnostics[0]["loc"]["length"].is_number());
    let edit = invoke(
        "edit_script_options",
        serde_json::json!({"source":source,"changes":{"iteration":1}}),
    );
    assert!(edit["source"].as_str().unwrap().contains("iteration=1 "));
    assert!(edit["diagnostics"].as_array().unwrap().is_empty());
}

#[test]
fn street_boxer_static_baseline_source_transport_matches_native_compilation_and_replay() {
    let source = "options duration=2 crit=fixed; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; mint add cartridge=Suit5 count=4; mint add stats cr=0.30; target hp=1000000 def=0 resist=0; active mint; mint attack;";
    let native =
        ntesim_content::dsl::compiler::compile_scenario(source, Default::default()).unwrap();
    let compiled = dispatch_invoke(
        &serde_json::to_vec(
            &serde_json::json!({"action":"compile_script","payload":{"source":source}}),
        )
        .unwrap(),
    )
    .unwrap()
    .data;
    assert_eq!(compiled["compiled"], serde_json::to_value(&native).unwrap());
    let inspected = dispatch_invoke(
        &serde_json::to_vec(
            &serde_json::json!({"action":"inspect_script","payload":{"source":source}}),
        )
        .unwrap(),
    )
    .unwrap()
    .data;
    assert_eq!(inspected["party"], compiled["party"]);
    let roundtrip: GameScenarioRequest =
        serde_json::from_value(compiled["compiled"]["request"].clone()).unwrap();
    assert_eq!(
        simulation::run_game_scenario(&roundtrip, &ExecutionLimits::default()).unwrap(),
        simulation::run_game_scenario(&native.request, &ExecutionLimits::default()).unwrap()
    );
}

#[test]
fn version_reports_incomplete_and_cutoff() {
    let v = version_info();
    assert_eq!(v.content_id, "ntesim-current");
    assert_eq!(v.content_cutoff, "2026-10-04");
    assert_eq!(v.status, "INCOMPLETE");
    assert!(!v.game_dps_computed);
    assert_eq!(v.adapter, "ntesim-wasm");
    assert!(!v.build_started_at.is_empty());
    assert!(v.build_started_at.ends_with('Z'));
}

use serde_json::json;

#[test]
fn prepared_game_cache_preserves_seeded_reports_and_invalidates_all_input_changes() {
    let mut request: GameScenarioRequest = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/research-crit.json"
    )))
    .unwrap();
    request.actors[0].talent_levels = Some([1, 2, 3]);
    request.actors[0].character_max_level = Some(80);
    let limits = ExecutionLimits::default();
    let compare = |request: &GameScenarioRequest| {
        let cached = dispatch_simulate_game(&serde_json::to_vec(request).unwrap());
        let fresh = simulation::run_game_scenario_report(request, &limits);
        match (cached, fresh) {
            (Ok(cached), Ok(fresh)) => assert_eq!(
                serde_json::to_value(cached).unwrap(),
                serde_json::to_value(fresh).unwrap()
            ),
            (Err(cached), Err(fresh)) => assert_eq!(cached, fresh),
            _ => panic!("cached and fresh execution disagree"),
        }
    };
    for seed in [42, 43, 0, u64::MAX, 42] {
        request.seed = seed;
        compare(&request);
    }
    let original = request.clone();
    request.actors[0].talent_levels = Some([10, 10, 10]);
    compare(&request);
    request.actors[0].static_stats.atk.numerator += 123;
    compare(&request);
    request.targets[0].defense += 100;
    compare(&request);
    request.relative_duration_us = 1;
    compare(&request);
    request
        .model_assumptions
        .insert("test.cache_v1".into(), "Changed hypothesis".into());
    compare(&request);
    request.active_actor_index = usize::MAX;
    compare(&request);
    // Rejected preparation must not poison a subsequent valid replay.
    compare(&original);
    let mut unknown = serde_json::to_value(&original).unwrap();
    unknown["unknown_field"] = json!(true);
    assert!(dispatch_simulate_game(&serde_json::to_vec(&unknown).unwrap()).is_err());
    compare(&original);
}

#[test]
fn program_single_and_batch_caches_reuse_replace_and_recover_from_rejected_preparation() {
    let mut request: GameScenarioRequest =
        serde_json::from_slice(include_bytes!("../../../examples/research-crit.json")).unwrap();
    request.actors[0].talent_levels = Some([1, 2, 3]);
    request.actors[0].character_max_level = Some(80);
    let mut program = GameScenarioProgramRequest {
        request,
        branches: vec![],
        energy_waits: vec![],
    };
    let compare = |program: &GameScenarioProgramRequest| {
        let raw = serde_json::to_vec(program).unwrap();
        let cached = dispatch_simulate_game_program(&raw);
        let fresh = simulation::run_game_program_report(program, &ExecutionLimits::default());
        match (cached, fresh) {
            (Ok(cached), Ok(fresh)) => assert_eq!(
                serde_json::to_value(cached).unwrap(),
                serde_json::to_value(fresh).unwrap()
            ),
            (Err(cached), Err(fresh)) => {
                assert_eq!(program.request.active_actor_index, usize::MAX);
                assert_eq!(cached, fresh);
                return;
            }
            _ => panic!("program cache differs from fresh preparation"),
        }
        let batch = WasmGameProgramBatchRequest {
            program: program.clone(),
            start_seed: 42,
            count: 2,
        };
        let reports =
            dispatch_simulate_game_program_batch(&serde_json::to_vec(&batch).unwrap()).unwrap();
        for (i, report) in reports.into_iter().enumerate() {
            let mut seeded = program.clone();
            seeded.request.seed = 42 + i as u64;
            let fresh =
                simulation::run_game_program_report(&seeded, &ExecutionLimits::default()).unwrap();
            assert_eq!(
                serde_json::to_value(report).unwrap(),
                serde_json::to_value(fresh).unwrap()
            );
        }
    };
    compare(&program);
    program.request.seed = 43;
    compare(&program);
    program.request.targets[0].defense += 100;
    compare(&program);
    let valid = program.clone();
    program.request.active_actor_index = usize::MAX;
    compare(&program);
    let invalid_batch = WasmGameProgramBatchRequest {
        program,
        start_seed: 42,
        count: 2,
    };
    assert!(
        dispatch_simulate_game_program_batch(&serde_json::to_vec(&invalid_batch).unwrap()).is_err()
    );
    compare(&valid);
}

#[test]
fn tabular_verification_dispatch_passes_all_cases() {
    let req = WasmTabularRequest { cases: None };
    let input = serde_json::to_vec(&req).unwrap();
    let report = dispatch_verify_tabular(&input).unwrap();
    assert_eq!(report.total_cases, 6);
    assert_eq!(report.passed_cases, 6);
    assert_eq!(report.failed_cases, 0);
    assert_eq!(report.documented_discrepancies.len(), 3);
}

#[test]
fn composition_enumeration_dispatch_matches_exact_8855() {
    let req = WasmCompositionRequest { variants_csv: None };
    let input = serde_json::to_vec(&req).unwrap();
    let report = dispatch_enumerate_compositions(&input).unwrap();
    assert_eq!(report.total_characters, 23);
    assert_eq!(report.total_compositions, 8855);
    assert_eq!(report.published_variants_count, 45);
    assert_eq!(report.unconventional_compositions, 8810);
}

#[test]
fn wasm_dispatch_wrappers_reject_unknown_fields() {
    assert!(
        serde_json::from_value::<WasmRotationRequest>(
            json!({"fixture":{},"rotation":"","seed":1,"seed_typo":2})
        )
        .is_err()
    );
    assert!(
        serde_json::from_value::<WasmBatchRequest>(
            json!({"scenario":{},"request":{},"request_typo":{}})
        )
        .is_err()
    );
    assert!(
        serde_json::from_value::<WasmTabularRequest>(json!({"cases":[],"cases_typo":[]})).is_err()
    );
    assert!(
        serde_json::from_value::<WasmCompositionRequest>(
            json!({"variants_csv":"","variants_typo":""})
        )
        .is_err()
    );
    assert!(
        serde_json::from_value::<WasmGameBatchRequest>(
            json!({"scenario":{},"start_seed":0,"count":1,"unknown":1})
        )
        .is_err()
    );
    assert!(
        serde_json::from_value::<WasmGameProgramBatchRequest>(
            json!({"program":{"request":{},"branches":[],"energy_waits":[]},"start_seed":0,"count":1,"unknown":1})
        )
        .is_err()
    );
}

#[test]
fn universal_invoke_separates_environment_metadata_from_core_data() {
    let envelope = json!({
        "action": "version",
        "payload": {}
    });
    let input = serde_json::to_vec(&envelope).unwrap();
    let resp = dispatch_invoke(&input).unwrap();
    assert_eq!(resp.status, "ok");
    assert_eq!(resp.action, "version");
    assert_eq!(resp.environment.adapter, "ntesim-wasm");
    assert!(!resp.environment.parity_verified);
    assert_eq!(resp.data["status"], "INCOMPLETE");
}

#[test]
fn universal_invoke_rejects_unknown_action() {
    let envelope = json!({
        "action": "unknown_action_xyz",
        "payload": {}
    });
    let input = serde_json::to_vec(&envelope).unwrap();
    assert!(dispatch_invoke(&input).is_err());
}

#[test]
fn actor_projection_dispatch_uses_shared_stats_and_rejects_missing_native_rules() {
    let request = json!({"content_id":"ntesim-current","character_id":"character:1019","level":80,"max_level":80,
        "weapon_id":"weapon:fork_lingganzhongjiezhe","weapon_level":80,"weapon_max_level":80,
        "refinement":5,"set_id":"set:Suit2","set_count":4,"base_overrides":{},
        "additive_stats":{"atk":"100/1","atk_percent":"1/2"}});
    let response = dispatch_invoke(
        &serde_json::to_vec(&json!({"action":"actor_projection","payload":request})).unwrap(),
    )
    .unwrap();
    assert_eq!(response.data["static_stats"]["base_atk"], 1151);
    assert_eq!(response.data["static_stats"]["atk"], json!("3653/2"));
    let mut invalid = request;
    invalid["additive_stats"]["unknown"] = json!("1/1");
    assert!(
        dispatch_invoke(
            &serde_json::to_vec(&json!({"action":"actor_projection","payload":invalid})).unwrap()
        )
        .is_err()
    );
}

#[test]
fn game_batch_dispatch_matches_individual_runs() {
    let mut request: GameScenarioRequest = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/research-crit.json"
    )))
    .unwrap();
    request.actors[0].talent_levels = Some([1, 2, 3]);
    request.actors[0].character_max_level = Some(80);

    let batch_req = WasmGameBatchRequest {
        scenario: request.clone(),
        start_seed: 100,
        count: 5,
    };
    let batch_input = serde_json::to_vec(&batch_req).unwrap();
    let batch_reports = dispatch_simulate_game_batch(&batch_input).unwrap();
    assert_eq!(batch_reports.len(), 5);

    for (i, batch_report) in batch_reports.iter().enumerate() {
        let mut individual_req = request.clone();
        individual_req.seed = 100 + i as u64;
        let individual_report =
            dispatch_simulate_game(&serde_json::to_vec(&individual_req).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(batch_report).unwrap(),
            serde_json::to_value(individual_report).unwrap()
        );
    }
}

#[test]
fn transport_rejects_wrapped_seed_ranges_without_poisoning_cache() {
    assert!(validate_seed_range(u64::MAX, 2).is_err());
    assert!(validate_seed_range(u64::MAX, 1).is_ok());
    assert!(validate_seed_range(u64::MAX, 0).is_ok());
    let error: Value = serde_json::from_slice(&transport("missing", "{}", true)).unwrap();
    assert_eq!(error["status"], "error");
    let version: Value = serde_json::from_slice(&transport("version", "", true)).unwrap();
    assert_eq!(version, serde_json::to_value(version_info()).unwrap());
}

#[test]
fn wire_golden_bytes_preserve_dictionary_and_exact_integer_spelling() {
    // Independently written v1 frame: ["x", "x", 2^53].
    let encoded = wire::encode(&("x", "x", 9_007_199_254_740_992u64)).unwrap();
    let mut expected = b"NTE\x01\x06\x04\x01\0\0\0x\x05\0\0\0\0\x09\x04\x10\0\0\0".to_vec();
    expected.extend_from_slice(b"9007199254740992");
    expected.push(8);
    assert_eq!(encoded, expected);
    assert_eq!(wire::encode(&f64::NAN).unwrap(), b"NTE\x01\0");
}

#[test]
fn wire_preserves_raw_receipts_and_full_width_numeric_tokens() {
    for text in [i128::MIN.to_string(), u128::MAX.to_string()] {
        let value: Value = serde_json::from_str(&text).unwrap();
        let mut expected = b"NTE\x01\x09\x04".to_vec();
        expected.extend_from_slice(&(text.len() as u32).to_le_bytes());
        expected.extend_from_slice(text.as_bytes());
        assert_eq!(wire::encode(&value).unwrap(), expected);
    }
    let text = r#"{"receipt":340282366920938463463374607431768211455}"#;
    let raw = serde_json::value::RawValue::from_string(text.into()).unwrap();
    let mut expected = b"NTE\x01\x0a\x04".to_vec();
    expected.extend_from_slice(&(text.len() as u32).to_le_bytes());
    expected.extend_from_slice(text.as_bytes());
    assert_eq!(wire::encode(&raw).unwrap(), expected);
}

#[test]
fn wire_interns_canonical_rational_strings_without_precision_loss() {
    use ntesim_core::{combat::exact::ExactFraction, formula::Ratio, kernel::state::WideFraction};

    let exact = ExactFraction::new(u128::MAX, u128::MAX - 1).unwrap();
    let ratio = Ratio {
        numerator: 18,
        denominator: 100,
    };
    let wide = WideFraction {
        numerator: "340282366920938463463374607431768211457".into(),
        denominator: "9007199254740993".into(),
    };
    let payload = serde_json::json!([exact, exact, ratio, wide, wide]);
    let encoded = wire::encode(&payload).unwrap();
    // Repeated ExactFraction and WideFraction instances collapse to 5-byte references.
    let expected_exact = format!("{}/{}", u128::MAX, u128::MAX - 1);
    let expected_wide = "340282366920938463463374607431768211457/9007199254740993";
    assert_eq!(encoded[4], 6); // ARRAY
    assert_eq!(encoded[5], 4); // STRING (exact #1)
    assert!(encoded.len() < expected_exact.len() * 2 + expected_wide.len() * 2 + 30);

    use std::io::Write;
    use std::process::{Command, Stdio};
    let decoder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("js/wire.js");
    let mut child = Command::new("node")
        .args([
            "-e",
            r#"
const assert = require('node:assert/strict');
const { decodeWire } = require(process.argv[1]);
const { encoded, expected } = JSON.parse(require('node:fs').readFileSync(0, 'utf8'));
assert.deepEqual(decodeWire(Uint8Array.from(encoded)), expected);
"#,
        ])
        .arg(decoder)
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(
            &serde_json::to_vec(&serde_json::json!({
                "encoded": encoded,
                "expected": [expected_exact, expected_exact, "18/100", expected_wide, expected_wide],
            }))
            .unwrap(),
        )
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
#[test]
fn source_execution_transport_matches_native_and_keeps_dynamic_ranges() {
    for (source, count, windows) in [
        (
            include_str!("../../../presets/native/live-control.nte"),
            16,
            0,
        ),
        (
            include_str!("../../../examples/blackbird-input-windows.nte"),
            13,
            6,
        ),
        (
            include_str!("../../../examples/manual-break-cycles.nte"),
            9,
            0,
        ),
        (
            include_str!("../../../examples/lacrimosa-sleep-in.nte"),
            26,
            0,
        ),
        (
            include_str!("../../../examples/chaos-verdict-continuations.nte"),
            34,
            2,
        ),
        (
            include_str!("../../../examples/daffodil-parry-cycles.nte"),
            20,
            0,
        ),
    ] {
        let request = ntesim_content::dsl::compiler::SourceRequest {
            source: source.into(),
            limits: Default::default(),
        };
        let native = ntesim_content::simulation::run_source(&request, &Default::default()).unwrap();
        let envelope =
            serde_json::to_vec(&serde_json::json!({"action":"run_script", "payload":request}))
                .unwrap();
        let bytes = crate::transport::dispatch_transport("invoke", &envelope, true).unwrap();
        let transported: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(transported["data"] == serde_json::to_value(native).unwrap());
        let binary = crate::transport::dispatch_transport("invoke", &envelope, false).unwrap();
        assert!(binary.starts_with(b"NTE\x01"));
        let occurrences = transported["data"]["execution"]["occurrences"]
            .as_array()
            .unwrap();
        assert_eq!(occurrences.len(), count);
        assert_eq!(
            occurrences
                .iter()
                .filter(|occurrence| occurrence["action_window"] == "hold")
                .count(),
            windows
        );
        assert!(
            occurrences
                .iter()
                .all(|occurrence| occurrence["loc"]["length"].as_u64().unwrap() > 0)
        );
    }
}
