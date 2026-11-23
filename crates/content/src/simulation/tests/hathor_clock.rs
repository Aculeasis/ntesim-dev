use super::*;
use crate::characters::hathor::clock::MODEL;
use crate::simulation::tests::support::formula_damage_total;
use serde_json::{Value, json};
pub(super) const CONNECTED: [u64; 5] = [1_134_310, 1_517_336, 1_317_210, 1_867_414, 2_388_345];
pub(super) fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::hathor_actions::request();
    req.actions.clear();
    req.actors[0].initial_energy = 0;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actors[0].talent_levels = Some([8; 3]);
    req.relative_duration_us = 30_000_000;
    req.model_assumptions
        .insert(MODEL.into(), MODEL.description().into());
    for target_index in 1..targets {
        let mut t = req.targets[0].clone();
        t.target_index = target_index;
        req.targets.push(t);
    }
    req
}
pub(super) fn normal(time: u64) -> GameActionCommand {
    super::hathor_actions::make_action(
        time,
        0,
        "GA_Hathor_Melee",
        "GE_Player_Hathor_Melee1_Damage",
        0,
        0,
    )
}
fn run(req: &GameScenarioRequest) -> Value {
    super::edgar_full::run(req)
}
#[test]
fn hathor_clock_two_continuing_chains_keep_independent_counts_damage_and_packet_resources() {
    for targets in [1, 3] {
        let mut req = request(targets);
        let mut now = 0;
        for _ in 0..2 {
            for recovery in CONNECTED {
                req.actions.push(normal(now));
                now += recovery;
            }
        }
        assert_eq!(now, 16_449_230);
        let r = run(&req);
        assert_eq!(r["hits_settled"], 26 * targets);
        assert_eq!(
            serde_json::json!(formula_damage_total(&r)),
            json!(format!("{}/{}", 7628 * targets, 1))
        );
        let actor = &r["final_actor_states"][0];
        assert_eq!(actor["energy"], 44);
        assert_eq!(actor["charge_remainder"], json!("301/500"));
        assert_eq!(actor["cycle"]["current"], 74);
        assert_eq!(actor["cycle"]["carry"], json!("99/500"));
        let first: Vec<_> = r["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| h["target_index"] == 0)
            .collect();
        assert_eq!(first[0]["time_us"], 216_746);
        assert_eq!(first[1]["time_us"], 1_334_274);
        assert_eq!(first[2]["time_us"], 1_550_838);
        assert_eq!(first[13]["time_us"], 8_441_361);
    }
}
#[test]
fn hathor_clock_early_input_diagnoses_and_explicit_idle_resets_chain() {
    let mut req = request(1);
    req.actions = vec![normal(0), normal(1_134_309)];
    let r = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(r["execution_completed"], false);
    assert_eq!(r["rejected_action"]["reason"]["until_us"], 1_134_310);
    req.actions = vec![normal(0), normal(1_134_310), normal(2_651_647)];
    let r = run(&req);
    assert_eq!(r["hits_settled"], 7);
    assert_eq!(
        r["hit_receipts"].as_array().unwrap().last().unwrap()["damage_effect_id"],
        "GE_Player_Hathor_Melee1_Damage"
    );
}
#[test]
fn hathor_clock_interruption_cancels_future_contacts_and_does_not_invent_their_resources() {
    let mut req = request(1);
    req.actions = vec![normal(0), normal(1_134_310), normal(1_600_001)];
    req.interrupt_events = vec![GameActorInterruptEvent {
        timestamp_us: 1_600_000,
        actor_index: 0,
        armor_damage: None,
    }];
    let r = run(&req);
    assert_eq!(r["hits_settled"], 4);
    assert_eq!(r["interrupt_receipts"][0]["cancelled_hit_deliveries"], 3);
    assert_eq!(
        r["hit_receipts"][3]["damage_effect_id"],
        "GE_Player_Hathor_Melee1_Damage"
    );
    assert_eq!(r["hit_receipts"][3]["time_us"], 1_816_747);
    //1.219+.807+.681+1.219=3.926; no phantom later-stage grants.
    assert_eq!(r["final_actor_states"][0]["energy"], 3);
    assert_eq!(
        r["final_actor_states"][0]["charge_remainder"],
        json!("463/500")
    );
}
