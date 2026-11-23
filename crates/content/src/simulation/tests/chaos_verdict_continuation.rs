use super::chaos_actions::make_action;
use super::*;
use serde_json::{Value, json};

fn request(form: bool) -> GameScenarioRequest {
    let mut req = super::chaos_verdict::request(3, form);
    let model = crate::HypothesisId::ChaosVerdictContinuationV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req
}
fn normal(time: u64) -> GameActionCommand {
    make_action(
        time,
        0,
        "GA_Chaos071_Melee",
        "GE_Player_Chaos_Melee1_Damage",
        0,
        0,
    )
}
fn hold(req: &mut GameScenarioRequest, time: u64, duration: u64) {
    req.hold_inputs.insert(
        req.actions.len(),
        crate::simulation::GameHoldInput {
            reserved_stocks: None,
            duration_us: duration,
            moving: false,
        },
    );
    req.actions.push(normal(time));
}
fn ordinary() -> GameScenarioRequest {
    let mut req = request(false);
    let mut time = 0;
    for _ in 0..3 {
        super::chaos_crime::chain(&mut req, &mut time);
    }
    hold(&mut req, 17_000_000, 1_000_000);
    req
}
fn run(req: &GameScenarioRequest) -> Value {
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}
fn branch(report: &Value, command: usize) -> Vec<&Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["command_index"] == command && hit["target_index"] == 0)
        .collect()
}

#[test]
fn chaos_full_verdict_continuation_has_strict_lower_and_exclusive_end_after_general_recovery() {
    for (time, complete) in [
        (20_328_840, false),
        (20_328_841, true),
        (20_357_216, true),
        (20_357_217, false),
    ] {
        let mut req = ordinary();
        hold(&mut req, time, 200_000);
        let report = run(&req);
        assert_eq!(report["execution_completed"], complete);
        if time == 20_328_840 {
            assert_eq!(report["rejected_action"]["reason"]["until_us"], 20_328_841);
        }
        if complete {
            let hits = branch(&report, 16);
            assert_eq!(hits.len(), 2);
            assert_eq!(hits[0]["time_us"], time + 166_691);
            assert_eq!(hits[1]["time_us"], time + 883_658);
            assert_eq!(
                report["final_actor_states"][0]["fractional_resources"]["chaos.crime"]["spent"],
                json!("1000/1")
            );
            assert_eq!(
                report["final_actor_states"][0]["fractional_resources"]["chaos.crime"]["current"],
                json!("0/1")
            );
            let last_entry = report["action_metadata_ledger"]
                .as_array()
                .unwrap()
                .last()
                .unwrap();
            assert!(support::has_assumption(
                last_entry,
                crate::HypothesisId::ChaosVerdictContinuationV1.as_str()
            ));
            assert!(!support::has_assumption(
                last_entry,
                crate::HypothesisId::ChaosNormalComboVerdictV1.as_str()
            ));
        }
    }
    let mut req = ordinary();
    req.actions.push(normal(19_902_033)); // General next-action recovery is earlier than continuation.
    assert_eq!(run(&req)["execution_completed"], true);
}

#[test]
fn chaos_transformed_continuation_uses_point_one_hold_and_keeps_later_live_crime_read() {
    let mut req = request(true);
    hold(&mut req, 9_703_042, 1_000_000);
    hold(&mut req, 13_033_000, 100_000);
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = branch(&report, 2);
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0]["time_us"], 13_200_196);
    assert_eq!(
        hits[0]["damage_effect_id"],
        "GE_Player_Chaos_Branch1_Damage_Ultra"
    );
    assert_eq!(report["final_actor_states"][0]["energy"], 16);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        json!("3/10")
    );
    assert_eq!(report["final_actor_states"][0]["cycle"]["current"], 27);
    let crime = &report["final_actor_states"][0]["fractional_resources"]["chaos.crime"];
    assert_eq!(crime["spent"], json!("1000/1"));
    assert_eq!(crime["current"], json!("780/1"));
    req.hold_inputs.get_mut(&2).unwrap().duration_us = 99_999;
    assert!(run_game_scenario(&req, &ExecutionLimits::default()).is_err());
    let mut req = ordinary();
    hold(&mut req, 20_328_841, 100_000);
    assert_eq!(run(&req)["execution_completed"], false);
}

#[test]
fn chaos_continuation_keeps_captured_old_form_predicate_and_selects_new_live_branch_row() {
    let mut req = request(true);
    hold(&mut req, 15_583_605, 1_000_000);
    hold(&mut req, 18_913_563, 100_000);
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = branch(&report, 2);
    assert_eq!(hits[0]["time_us"], 19_080_254);
    assert_eq!(
        hits[0]["damage_effect_id"],
        "GE_Player_Chaos_Branch1_Damage"
    );
}

#[test]
fn chaos_interrupt_and_other_admission_end_the_live_continuation_after_recovery() {
    let mut req = ordinary();
    let model = crate::HypothesisId::ChaosForcedInterruptV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.interrupt_events = vec![crate::simulation::GameActorInterruptEvent {
        timestamp_us: 20_100_000,
        actor_index: 0,
        armor_damage: None,
    }];
    hold(&mut req, 20_328_841, 200_000);
    let report = run(&req);
    assert_eq!(report["execution_completed"], false);
    assert_eq!(
        report["interrupt_receipts"][0]["interrupted_command_index"],
        15
    );
    assert_eq!(
        report["interrupt_receipts"][0]["cancelled_hit_deliveries"],
        0
    );
    assert_eq!(
        report["interrupt_receipts"][0]["recovery_before_us"],
        19_902_033
    );
    assert_eq!(
        report["final_actor_states"][0]["fractional_resources"]["chaos.crime"]["spent"],
        json!("1000/1")
    );
    let mut req = ordinary();
    req.actions.push(normal(20_000_000));
    hold(&mut req, 20_328_841, 200_000);
    assert_eq!(run(&req)["execution_completed"], false); // No Normal-combo binding can replace the new Normal.
}

#[test]
fn chaos_full_stage_jump_retains_its_native_gate_during_live_continuation() {
    for time in [19_943_251, 19_943_252] {
        let mut req = ordinary();
        let model = crate::HypothesisId::ChaosJumpCancelV1;
        req.model_assumptions
            .insert(model.into(), model.description().into());
        req.actions.push(GameActionCommand::PlayerInput {
            timestamp_us: time,
            actor_index: 0,
            input: crate::simulation::PlayerInput::Jump,
            critical_success: false,
        });
        let report = run(&req);
        assert_eq!(report["execution_completed"], time == 19_943_252);
        if time == 19_943_251 {
            assert_eq!(report["rejected_action"]["reason"]["until_us"], 19_943_252);
        }
    }
}

#[test]
fn chaos_arc_commit_closes_the_old_full_verdict_continuation_source() {
    let mut req = ordinary();
    req.actors[0].weapon_id = "weapon:fork_jingmotingyuan".into();
    req.actors[0].gear_procs = vec![GearProcConfig::JingmotingyuanCrit {
        crit_damage_bonus: Ratio {
            numerator: 3,
            denominator: 25,
        },
        duration_us: 5_000_000,
    }];
    req.actions.push(GameActionCommand::PlayerInput {
        timestamp_us: 20_000_000,
        actor_index: 0,
        input: crate::simulation::PlayerInput::Arc,
        critical_success: false,
    });
    hold(&mut req, 20_328_841, 200_000);
    assert_eq!(run(&req)["execution_completed"], false);
}
