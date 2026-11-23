use super::chaos_actions::make_action;
use super::*;
use crate::simulation::{GameHoldInput, PlayerInput};
use serde_json::{Value, json};

fn request(form: bool) -> GameScenarioRequest {
    let mut req = super::chaos_verdict::request(3, form);
    for model in [
        crate::HypothesisId::ChaosJumpCancelV1,
        crate::HypothesisId::ChaosNormalComboVerdictV1,
    ] {
        req.model_assumptions
            .insert(model.into(), model.description().into());
    }
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
fn jump(time: u64) -> GameActionCommand {
    GameActionCommand::PlayerInput {
        timestamp_us: time,
        actor_index: 0,
        input: PlayerInput::Jump,
        critical_success: false,
    }
}
fn held(req: &mut GameScenarioRequest, time: u64, duration: u64) {
    req.hold_inputs.insert(
        req.actions.len(),
        GameHoldInput {
            reserved_stocks: None,
            duration_us: duration,
            moving: false,
        },
    );
    req.actions.push(normal(time));
}
fn run(req: &GameScenarioRequest) -> Value {
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}
fn crime(report: &Value) -> &Value {
    &report["final_actor_states"][0]["fractional_resources"]["chaos.crime"]
}

#[test]
fn chaos_jump_normal_gate_includes_only_reached_pause_and_resets_chain_without_free_hit() {
    for time in [283_526, 283_527] {
        let mut req = request(false);
        req.actions = vec![normal(0), jump(time), normal(time)];
        let report = run(&req);
        assert_eq!(report["execution_completed"], time == 283_527);
        if time == 283_526 {
            assert_eq!(report["rejected_action"]["reason"]["until_us"], 283_527);
        } else {
            assert_eq!(report["hits_settled"], 6);
            assert_eq!(crime(&report)["current"], json!("44/1"));
            assert_eq!(
                report["hit_receipts"][3]["damage_effect_id"],
                "GE_Player_Chaos_Melee1_Damage"
            );
            assert_eq!(report["hit_receipts"][3]["time_us"], 498_100);
        }
    }
}

#[test]
fn chaos_jump_plain_verdict_discards_second_contact_and_pending_spend_but_preserves_stock() {
    let mut req = request(false);
    let mut time = 0;
    for _ in 0..3 {
        super::chaos_crime::chain(&mut req, &mut time);
    }
    held(&mut req, 17_000_000, 1_000_000);
    req.actions.push(jump(18_183_883));
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 111); //36 primary groups + one plain branch, three dummies.
    assert_eq!(crime(&report)["spent"], json!("0/1"));
    assert_eq!(crime(&report)["current"], json!("1000/1"));
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| event["event_kind"] != "chaos_verdict_read")
    );
}

#[test]
fn chaos_normal_three_combo_uses_strict_progress_gate_and_cancels_only_future_packet() {
    for time in [2_113_638, 2_113_639] {
        let mut req = request(false);
        req.actions = vec![normal(0), normal(518_862), normal(1_693_838)];
        held(&mut req, time, 200_000);
        let report = run(&req);
        assert_eq!(report["execution_completed"], time == 2_113_639);
        if time == 2_113_638 {
            assert_eq!(report["rejected_action"]["reason"]["until_us"], 2_113_639);
        } else {
            assert_eq!(report["hits_settled"], 18);
            assert_eq!(crime(&report)["current"], json!("98/1"));
            let own: Vec<_> = report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|hit| hit["target_index"] == 0)
                .map(|hit| hit["time_us"].as_u64().unwrap())
                .collect();
            assert_eq!(
                own,
                [214_573, 756_182, 918_306, 2_035_883, 2_280_330, 2_997_297]
            );
        }
    }
}

#[test]
fn chaos_transformed_normal_two_combo_uses_its_own_point_sixty_four_predicate() {
    for time in [6_139_863, 6_179_862, 6_179_863] {
        let mut req = request(true);
        req.actions.extend([normal(5_000_000), normal(5_518_862)]);
        held(&mut req, time, 200_000);
        let report = run(&req);
        assert_eq!(report["execution_completed"], time == 6_179_863);
        if time < 6_179_863 {
            assert_eq!(report["rejected_action"]["reason"]["until_us"], 6_179_863);
        } else {
            let branch = report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .find(|hit| hit["damage_effect_id"] == "GE_Player_Chaos_Branch1_Damage_Ultra")
                .unwrap();
            assert_eq!(branch["time_us"], 6_347_059);
        }
    }
}

#[test]
fn chaos_combo_requires_real_hold_overlap_and_has_no_idle_or_unsupported_short_fallback() {
    let mut req = request(false);
    held(&mut req, 0, 200_000);
    assert_eq!(run(&req)["execution_completed"], false);
    req.actions.clear();
    req.hold_inputs.clear();
    req.actions.push(normal(0));
    held(&mut req, 319_801, 400_000);
    let report = run(&req);
    assert_eq!(report["execution_completed"], false);
    assert_eq!(report["rejected_action"]["reason"]["until_us"], 400_000);
    // Even after recovery, an authored .2 hold cannot become a free idle Verdict.
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut req.actions[1] {
        *timestamp_us = 518_862;
    }
    req.hold_inputs.get_mut(&1).unwrap().duration_us = 200_000;
    assert_eq!(run(&req)["execution_completed"], false);
}

#[test]
fn chaos_transformed_full_verdict_jump_clears_immunity_without_refunding_spent_crime() {
    let mut req = request(true);
    held(&mut req, 9_703_042, 1_000_000);
    req.actions.push(jump(12_605_401)); // .632402 progress + two .185253 pauses.
    let model = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
        timestamp_us: 7_802_909,
        actor_index: 0,
        delivered_damage: 100,
    }];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["incoming_damage_receipts"][0]["actual_hp_loss"], 100);
    assert_eq!(crime(&report)["spent"], json!("1000/1"));
    *req.actions.last_mut().unwrap() = jump(12_605_400);
    assert_eq!(
        run(&req)["rejected_action"]["reason"]["until_us"],
        12_605_401
    );
}

#[test]
fn chaos_each_normal_stage_jump_uses_captured_form_and_exact_reached_pauses() {
    for form in [false, true] {
        for stage in 0..5 {
            let start = if form { 5_000_000 } else { 0 };
            let gates = [
                283_527,
                1_004_782,
                if form { 2_216_682 } else { 2_235_850 },
                3_498_812,
                5_319_894,
            ];
            for early in [true, false] {
                let mut req = request(form);
                let mut time = start;
                for recovery in [518_862, 1_174_976, 545_853, 2_018_381, 1_072_842]
                    .iter()
                    .take(stage + 1)
                {
                    req.actions.push(normal(time));
                    time += recovery;
                }
                req.actions
                    .push(jump(start + gates[stage] - u64::from(early)));
                let report = run(&req);
                assert_eq!(
                    report["execution_completed"], !early,
                    "form {form} stage {stage}"
                );
                if early {
                    assert_eq!(
                        report["rejected_action"]["reason"]["until_us"],
                        start + gates[stage]
                    );
                }
            }
        }
    }
}

#[test]
fn chaos_combo_predicate_keeps_old_montage_form_but_new_branch_uses_live_form_after_expiry() {
    let mut req = request(true);
    req.actions
        .extend([normal(14_889_767), normal(15_408_629), normal(16_583_605)]);
    held(&mut req, 17_003_406, 200_000);
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    assert!(hits.iter().any(|hit| hit["damage_effect_id"]
        == "GE_Player_Chaos_Melee3_Damage_Ultra"
        && hit["time_us"] == 16_925_650));
    let branch = hits
        .iter()
        .find(|hit| hit["damage_effect_id"] == "GE_Player_Chaos_Branch1_Damage")
        .unwrap();
    assert_eq!(branch["time_us"], 17_170_097);
    assert!(
        !hits
            .iter()
            .any(|hit| hit["damage_effect_id"] == "GE_Player_Chaos_Branch1_Damage_Ultra")
    );
}
