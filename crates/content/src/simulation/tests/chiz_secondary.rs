use super::blackbird_secondary::{dash, stamina};
use super::*;
use crate::characters::chiz::secondary::{MODEL, PLUNGE, RIPOSTE};
use serde_json::{Value, json};

pub(super) fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::chiz_cancel::request(targets);
    req.model_assumptions
        .insert(MODEL.into(), MODEL.description().into());
    stamina(&mut req, 100);
    req
}
fn run(req: &GameScenarioRequest) -> Value {
    let report = run_game_scenario(req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    report
}
fn assert_ready(req: &GameScenarioRequest, ready: u64) {
    let mut check = req.clone();
    check.actions.push(super::chiz_clock::normal(ready - 1));
    let rejected = run_game_scenario(&check, &ExecutionLimits::default()).unwrap();
    assert_eq!(rejected["rejected_action"]["reason"]["until_us"], ready);
    *check.actions.last_mut().unwrap() = super::chiz_clock::normal(ready);
    run(&check);
}
#[test]
fn chiz_plunge_pays_real_airtime_and_height_scales_damage_without_scaling_grain_or_channels() {
    for height in [0, 1] {
        let mut req = request(3);
        let mut action = super::chiz_clock::normal(0);
        if let GameActionCommand::InvokeAbility {
            damage_effect_id, ..
        } = &mut action
        {
            *damage_effect_id = PLUNGE.into();
        }
        req.actions = vec![action];
        req.plunge_inputs.insert(
            0,
            GamePlungeInput {
                landing_delay_us: 600_000,
                height_bonus: Ratio {
                    numerator: height,
                    denominator: 1,
                },
            },
        );
        let report = run(&req);
        assert_eq!(report["hits_settled"], 3);
        for hit in report["hit_receipts"].as_array().unwrap() {
            assert_eq!(hit["time_us"], 799_456);
            assert_eq!(
                hit["non_crit_damage"],
                json!(format!("{}/{}", 771 * (1 + height), 1))
            );
        }
        let actor = &report["final_actor_states"][0];
        assert_ready(&req, 1_248_515);
        assert_eq!(actor["energy"], 5);
        assert_eq!(actor["cycle"]["current"], 7);
        assert_eq!(
            actor["fractional_resources"]["chiz.grain"]["current"],
            json!("450/1")
        );
    }
}

#[test]
fn chiz_paid_critical_dodge_has_one_two_request_riposte_and_missing_cost_never_authorizes_it() {
    for back in [false, true] {
        let mut req = request(3);
        let ready = if back { 1_012_579 } else { 998_284 };
        req.actions = vec![dash(0, true), super::chiz_clock::normal(ready)];
        if back {
            req.dash_directions.insert(0, GameDashDirection::Back);
        }
        let report = run(&req);
        let hits = report["hit_receipts"].as_array().unwrap();
        assert_eq!(hits.len(), 6);
        assert_eq!(hits[0]["damage_effect_id"], RIPOSTE);
        assert_eq!(hits[0]["time_us"], ready + 150_950);
        assert_eq!(hits[3]["time_us"], ready + 333_387);
        let actor = &report["final_actor_states"][0];
        assert_ready(&req, ready + 1_210_147);
        assert_eq!(
            actor["fractional_resources"]["chiz.grain"]["current"],
            json!("900/1")
        );
        assert_eq!(report["combat_stamina"]["current"], json!("85/1"));
        let next = ready + 1_210_147;
        req.actions.push(super::chiz_clock::normal(next));
        assert_eq!(
            run(&req)["hit_receipts"][6]["damage_effect_id"],
            "GE_Player_Chiichan073_Melee1_Damage"
        );
        stamina(&mut req, 14);
        let failed = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(failed["execution_completed"], false);
        assert_eq!(failed["hits_settled"], 0);
        req.combat_stamina = None;
        assert!(
            run_game_scenario(&req, &ExecutionLimits::default())
                .unwrap_err()
                .contains("stamina")
        );
    }
}

#[test]
fn chiz_qualified_support_spends_outgoing_cycle_and_preserves_hermitian_pause_and_no_grain() {
    let mut req = request(3);
    let mut ally=crate::dsl::compiler::compile_scenario("options duration=4; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=100000000 def=1080 resist=0; active mint;",Default::default()).unwrap().request.actors.remove(0);
    ally.slot = 1;
    ally.initial_cycle = 100;
    req.actors.push(ally);
    req.active_actor_index = 1;
    req.actions = vec![GameActionCommand::SwitchActor {
        timestamp_us: 0,
        target_actor_index: 0,
        cooldown_us: 0,
    }];
    let report = run(&req);
    assert_eq!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "GE_Player_Chiichan073_QTE_Damage")
            .count(),
        3
    );
    assert_eq!(
        report["hit_receipts"][0]["damage_effect_id"],
        "GE_Player_Chiichan073_QTE_Damage"
    );
    assert_eq!(report["hit_receipts"][0]["time_us"], 800_160);
    assert_ready(&req, 1_897_278);
    assert_eq!(report["final_actor_states"][1]["cycle"]["spent"], 100);
    assert_eq!(report["final_actor_states"][0]["energy"], 10);
    assert_eq!(
        report["final_actor_states"][0]["fractional_resources"]["chiz.grain"]["current"],
        json!("0/1")
    );
    req.actors[1].initial_cycle = 99;
    assert_eq!(run(&req)["hits_settled"], 0);
}

#[test]
fn chiz_cancelled_riposte_retains_already_connected_pause_in_new_skill_avatar() {
    let mut req = request(1);
    req.actions = vec![
        dash(0, true),
        super::chiz_clock::normal(998_284),
        super::chiz_pressed::press(1_338_284),
    ];
    let report = run(&req);
    // The Riposte's second contact at1.331671 starts its real pause at1.348371,
    // after the new Skill input. Its184000us lost progress still delays Skill.
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits[2]["time_us"], 1_705_582);
    assert_eq!(hits[3]["time_us"], 2_301_842);
}
