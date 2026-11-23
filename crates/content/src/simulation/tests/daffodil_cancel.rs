use super::daffodil_clock::{chain, normal, run, skill};
use super::*;
use serde_json::json;
pub(super) fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::daffodil_clock::request(targets);
    let model = crate::characters::daffodil::clock::cancel::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req
}
fn jump(time: u64) -> GameActionCommand {
    GameActionCommand::PlayerInput {
        timestamp_us: time,
        actor_index: 0,
        input: PlayerInput::Jump,
        critical_success: false,
    }
}

#[test]
fn daffodil_jump_at_the_live_stage_gate_cancels_unreached_requests_and_resets_normal() {
    for targets in [1, 3] {
        let mut req = request(targets);
        req.actions = vec![
            normal(0),
            normal(1_003_012),
            jump(1_164_546),
            normal(1_200_000),
        ];
        let report = run(&req);
        assert_eq!(report["hits_settled"], 3 * targets);
        let hits: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| h["target_index"] == 0)
            .collect();
        assert_eq!(
            hits[1]["damage_effect_id"],
            "GE_Player_Daffodill_Melee2_Damage"
        );
        assert_eq!(
            hits[2]["damage_effect_id"],
            "GE_Player_Daffodill_Melee1_Damage"
        );
        assert_eq!(hits[2]["time_us"], 1_385_308);
        req.actions[2] = jump(1_164_545);
        let rejected = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(rejected["execution_completed"], false);
        assert_eq!(rejected["rejected_action"]["reason"]["until_us"], 1_164_546);
    }
}

#[test]
fn daffodil_every_normal_stage_obeys_its_distinct_jump_and_skill_boundary() {
    for (stage, jump_at, after_jump, skill_at, after_skill) in [
        (0, 216_490, 2, 283_178, 9),
        (1, 1_164_546, 3, 1_439_963, 11),
        (2, 2_620_050, 9, 2_954_898, 17),
        (3, 3_674_722, 11, 4_154_093, 20),
        (4, 5_732_709, 20, 5_825_436, 27),
    ] {
        let mut req = request(1);
        chain(&mut req, 0);
        req.actions.truncate(stage + 1);
        let mut jumped = req.clone();
        jumped.actions.extend([jump(jump_at), normal(jump_at + 1)]);
        assert_eq!(
            run(&jumped)["hits_settled"],
            after_jump,
            "stage{} Jump",
            stage + 1
        );
        req.actions.push(skill(skill_at));
        assert_eq!(
            run(&req)["hits_settled"],
            after_skill,
            "stage{} Skill",
            stage + 1
        );
        *req.actions.last_mut().unwrap() = skill(skill_at - 1);
        let rejected = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(rejected["rejected_action"]["reason"]["until_us"], skill_at);
    }
}
#[test]
fn daffodil_skill_cancel_gate_includes_only_already_connected_pause() {
    let mut req = request(1);
    req.actions = vec![normal(0), normal(1_003_012), skill(1_439_963)];
    let report = run(&req);
    assert_eq!(report["hits_settled"], 11);
    assert_eq!(report["hit_receipts"][3]["time_us"], 1_636_638);
    assert_eq!(report["final_actor_states"][0]["energy"], 9);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        json!("27/500")
    );
    *req.actions.last_mut().unwrap() = skill(1_439_962);
    let rejected = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(rejected["rejected_action"]["reason"]["until_us"], 1_439_963);
}
#[test]
fn daffodil_forced_skill_interruption_keeps_notify_owned_cooldowns_without_refunds() {
    for (time, next, cancelled, earlier) in
        [(100_000, 3_100_000, 8, 0), (200_000, 16_182_290, 7, 1)]
    {
        let mut req = request(2);
        req.actions = vec![skill(0), skill(next)];
        req.interrupt_events.push(GameActorInterruptEvent {
            timestamp_us: time,
            actor_index: 0,
            armor_damage: None,
        });
        let report = run(&req);
        assert_eq!(report["hits_settled"], 2 * (8 + earlier));
        let receipt = &report["interrupt_receipts"][0];
        assert_eq!(receipt["interrupted_command_index"], 0);
        assert_eq!(receipt["cancelled_hit_deliveries"], 2 * cancelled);
        assert_eq!(receipt["skill_cooldown_after_us"], next);
        assert_eq!(receipt["recovery_after_us"], time);
    }
}
#[test]
fn daffodil_new_action_inherits_only_the_live_remainder_of_a_connected_pause() {
    let mut req = request(1);
    req.actions = vec![skill(0), normal(1_120_000)];
    req.interrupt_events.push(GameActorInterruptEvent {
        timestamp_us: 1_110_000,
        actor_index: 0,
        armor_damage: None,
    });
    let report = run(&req);
    assert_eq!(report["hits_settled"], 9);
    assert_eq!(report["hit_receipts"][8]["time_us"], 1_338_397);
    assert_eq!(
        report["interrupt_receipts"][0]["cancelled_hit_deliveries"],
        0
    );
    assert_eq!(
        report["interrupt_receipts"][0]["skill_cooldown_after_us"],
        16_182_290
    );
    // Recovery and then the original2s continuity remain exact; no interruption
    // after a fully completed chain can erase its already settled packets.
    let mut continued = request(1);
    chain(&mut continued, 0);
    continued.interrupt_events.push(GameActorInterruptEvent {
        timestamp_us: 6_100_000,
        actor_index: 0,
        armor_damage: None,
    });
    assert_eq!(
        run(&continued)["interrupt_receipts"][0]["interrupted_command_index"],
        serde_json::Value::Null
    );
}
