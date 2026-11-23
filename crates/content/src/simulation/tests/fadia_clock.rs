use super::*;
use crate::simulation::tests::support::formula_damage_total;
use serde_json::{Value, json};
pub(super) fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::edgar_full::request(targets);
    req.actors[0].character_id = "character:1039".into();
    req.actors[0].initial_energy = 0;
    req.actors[0].energy_capacity = 120;
    req.actors[0].talent_levels = Some([8; 3]);
    let model = crate::characters::fadia::clock::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req
}
fn normal(time: u64) -> GameActionCommand {
    super::edgar_actions::make_action(
        time,
        0,
        "GA_Fadia_Melee",
        "GE_Player_Fadia_Melee1_Damage",
        0,
        0,
    )
}
fn run(req: &GameScenarioRequest) -> Value {
    super::edgar_full::run(req)
}
#[test]
fn fadia_curved_normal_clock_preserves_two_continuing_chains_and_independent_resource_sums() {
    for targets in [1, 3] {
        let mut req = request(targets);
        let mut now = 0;
        for _ in 0..2 {
            for recovery in [1212790, 1871247, 1586548, 1712977, 1913355] {
                req.actions.push(normal(now));
                now += recovery;
            }
        }
        assert_eq!(now, 16593834);
        let r = run(&req);
        assert_eq!(r["hits_settled"], 16 * targets);
        assert_eq!(
            serde_json::json!(formula_damage_total(&r)),
            json!(format!("{}/{}", 6087 * targets, 1))
        );
        let actor = &r["final_actor_states"][0];
        assert_eq!(actor["energy"], 44);
        assert_eq!(actor["charge_remainder"], json!("2/5"));
        assert_eq!(actor["cycle"]["current"], 74);
        assert_eq!(actor["cycle"]["carry"], json!("1/500"));
        let hits: Vec<_> = r["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| h["target_index"] == 0)
            .collect();
        assert_eq!(hits[0]["time_us"], 316667);
        assert_eq!(hits[1]["time_us"], 1562790);
        assert_eq!(hits[2]["time_us"], 2062790);
        req.actions.push(normal(now - 1));
        let early = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(early["execution_completed"], false);
        assert_eq!(early["rejected_action"]["reason"]["until_us"], now);
    }
}
#[test]
fn fadia_interrupt_retains_the_actual_remaining_curve_instead_of_a_flat_loss_approximation() {
    let mut req = request(1);
    // Interrupt settlement follows ordinary admission at equal time.
    req.actions = vec![normal(0), normal(440001)];
    req.interrupt_events = vec![GameActorInterruptEvent {
        timestamp_us: 440000,
        actor_index: 0,
        armor_damage: None,
    }];
    let r = run(&req);
    assert_eq!(r["hits_settled"], 2);
    assert_eq!(r["hit_receipts"][0]["time_us"], 316667);
    assert_eq!(r["hit_receipts"][1]["time_us"], 762168);
    // The old curve contributes5499.395006625us remaining loss after440001.
    req.actions.push(normal(1658289));
    let early = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(early["execution_completed"], false);
    assert_eq!(early["rejected_action"]["reason"]["until_us"], 1658290);
}
