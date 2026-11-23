use super::blackbird_secondary::{dash, stamina};
use super::daffodil_clock::{action, normal, run, skill};
use super::*;
use crate::characters::daffodil::secondary::{MODEL, PLUNGE, QTE, RIPOSTE};
use serde_json::json;
pub(super) fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::daffodil_cancel::request(targets);
    req.model_assumptions
        .insert(MODEL.into(), MODEL.description().into());
    stamina(&mut req, 100);
    req
}
fn assert_ready(req: &GameScenarioRequest, ready: u64) {
    let mut early = req.clone();
    early.actions.push(normal(ready - 1));
    let rejected = run_game_scenario(&early, &ExecutionLimits::default()).unwrap();
    assert_eq!(rejected["rejected_action"]["reason"]["until_us"], ready);
    *early.actions.last_mut().unwrap() = normal(ready);
    run(&early);
}
#[test]
fn daffodil_plunge_pays_airtime_and_height_only_scales_hp_not_charge_or_cycle() {
    for height in [0, 1] {
        let mut req = request(3);
        req.actions = vec![action(0, "GA_Daffodill_Melee", PLUNGE, 0)];
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
            assert_eq!(hit["time_us"], 653_784);
            assert_eq!(
                hit["non_crit_damage"],
                json!(format!("{}/{}", 727, if height == 0 { 2 } else { 1 }))
            );
        }
        let actor = &report["final_actor_states"][0];
        assert_eq!(actor["energy"], 2);
        assert_eq!(actor["charge_remainder"], json!("7/10"));
        assert_eq!(actor["cycle"]["current"], 4);
        assert_eq!(actor["cycle"]["carry"], json!("2/5"));
        assert_ready(&req, 920_849);
    }
}
#[test]
fn daffodil_paid_front_back_dodge_grants_exactly_one_next_normal_close_call() {
    for (back, recovery) in [(false, 1_014_541), (true, 1_007_068)] {
        let mut req = request(3);
        req.actions = vec![
            dash(0, true),
            normal(recovery),
            normal(recovery + 1_171_201),
        ];
        if back {
            req.dash_directions.insert(0, GameDashDirection::Back);
        }
        let report = run(&req);
        let hits = report["hit_receipts"].as_array().unwrap();
        assert_eq!(hits.len(), 6);
        for hit in &hits[..3] {
            assert_eq!(hit["damage_effect_id"], RIPOSTE);
            assert_eq!(hit["time_us"], recovery + 229_761);
            assert_eq!(hit["non_crit_damage"], json!("891/1"));
        }
        for hit in &hits[3..] {
            assert_eq!(hit["damage_effect_id"], "GE_Player_Daffodill_Melee1_Damage");
        }
        assert_eq!(report["final_actor_states"][0]["energy"], 3);
        assert_eq!(
            report["final_actor_states"][0]["charge_remainder"],
            json!("641/1000")
        );
        req.actions.truncate(1);
        assert_ready(&req, recovery);
        req.actions.push(normal(recovery));
        assert_ready(&req, recovery + 1_171_201);
    }
}
#[test]
fn daffodil_dodge_requires_the_declared_paid_profile_and_non_normal_action_consumes_authorization()
{
    let mut req = request(1);
    req.actions = vec![dash(0, true), skill(333_754), normal(1_591_746)];
    let report = run(&req);
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|h| h["damage_effect_id"] != RIPOSTE)
    );
    stamina(&mut req, 14);
    let rejected = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(rejected["execution_completed"], false);
    assert_eq!(rejected["hits_settled"], 0);
    req.combat_stamina = None;
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("stamina")
    );
}
fn switch(time: u64, actor: usize) -> GameActionCommand {
    GameActionCommand::SwitchActor {
        timestamp_us: time,
        target_actor_index: actor,
        cooldown_us: 0,
    }
}
#[test]
fn daffodil_actual_qualified_supports_capture_echoes_and_a1_replaces_each_stack_bonus() {
    for a1 in [false, true] {
        let mut req = request(2);
        req.actors[0].awakening =
            ntesim_data::AwakeningSelection::selected(if a1 { vec![1] } else { vec![] }).unwrap();
        for (slot, id) in [(1, "character:1042"), (2, "character:1039")] {
            let mut ally = req.actors[0].clone();
            ally.slot = slot;
            ally.character_id = id.into();
            ally.initial_cycle = 100;
            ally.initial_energy = 0;
            ally.awakening = Default::default();
            req.actors.push(ally);
        }
        req.active_actor_index = 1;
        req.actions = vec![
            switch(0, 0),
            switch(1_500_000, 2),
            switch(2_000_000, 0),
            skill(4_000_000),
        ];
        for target in &mut req.targets {
            target.break_profile = Some(GameTargetBreakProfileConfig {
                discord_fraction: None,
                capacity: Ratio {
                    numerator: 100,
                    denominator: 1,
                },
                current_gauge: Ratio {
                    numerator: 100,
                    denominator: 1,
                },
                broken_duration_us: 2_000_000,
            });
        }
        let report = run(&req);
        let own: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| h["actor_index"] == 0 && h["damage_effect_id"] == QTE)
            .collect();
        assert_eq!(own.len(), 4);
        assert_eq!(own[0]["time_us"], 745_033);
        assert_eq!(own[2]["time_us"], 2_745_033);
        let echoes: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| h["actor_index"] == 0 && h["command_index"] == 3 && h["target_index"] == 0)
            .collect();
        assert_eq!(echoes.len(), 7);
        assert_eq!(echoes[0]["time_us"], 4_118_959);
        assert_eq!(
            echoes[0]["non_crit_damage"],
            json!(format!("{}/{}", if a1 { 894 } else { 596 }, 1))
        );
        assert_eq!(
            report["final_actor_states"][0]["resources"]["daffodil.resonance"]["current"],
            0
        );
        assert_eq!(report["final_actor_states"][1]["cycle"]["spent"], 100);
        assert_eq!(report["final_actor_states"][2]["cycle"]["spent"], 100);
    }
}
#[test]
fn daffodil_raw_secondary_rows_and_undeclared_airtime_cannot_fabricate_authorization() {
    for (ability, effect) in [
        ("GA_Daffodill_Melee", RIPOSTE),
        ("GA_Daffodill_QTE", QTE),
        ("GA_Daffodill_Melee", PLUNGE),
    ] {
        let mut req = request(1);
        req.actions = vec![action(0, ability, effect, 0)];
        assert!(run_game_scenario(&req, &ExecutionLimits::default()).is_err());
    }
}
