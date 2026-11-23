use super::*;
use crate::characters::fadia::secondary::{PLUNGE, QTE, RIPOSTE};
use serde_json::json;
fn request(targets: usize) -> GameScenarioRequest {
    super::fadia_effects::request(targets)
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
fn jump(time: u64) -> GameActionCommand {
    GameActionCommand::PlayerInput {
        timestamp_us: time,
        actor_index: 0,
        input: PlayerInput::Jump,
        critical_success: false,
    }
}
#[test]
fn fadia_paid_critical_dodge_consumes_one_token_and_keeps_two_cycle_packet_scoped_resources() {
    for targets in [1, 3] {
        let mut req = request(targets);
        super::blackbird_secondary::stamina(&mut req, 100);
        for base in [0, 5000000] {
            req.actions.extend([
                super::blackbird_secondary::dash(base, true),
                normal(base + 1006571),
                normal(base + 2177772),
            ]);
        }
        let r = super::edgar_full::run(&req);
        let hits = r["hit_receipts"].as_array().unwrap();
        assert_eq!(hits.len(), 4 * targets);
        for (index, packet) in [
            (0, RIPOSTE),
            (targets, "GE_Player_Fadia_Melee1_Damage"),
            (2 * targets, RIPOSTE),
            (3 * targets, "GE_Player_Fadia_Melee1_Damage"),
        ] {
            assert_eq!(hits[index]["damage_effect_id"], packet);
        }
        assert_eq!(hits[0]["time_us"], 1207280);
        assert_eq!(hits[targets]["time_us"], 2494439);
        assert_eq!(hits[0]["non_crit_damage"], json!("3051/2"));
        let actor = &r["final_actor_states"][0];
        assert_eq!(actor["energy"], 14);
        assert_eq!(actor["charge_remainder"], json!("207/500"));
        assert_eq!(actor["cycle"]["current"], 23);
        assert_eq!(actor["cycle"]["carry"], json!("411/500"));
        assert_eq!(r["combat_stamina"]["spends"].as_array().unwrap().len(), 2);
    }
}
#[test]
fn fadia_unpaid_critical_dodge_is_atomic() {
    let mut req = request(1);
    super::blackbird_secondary::stamina(&mut req, 14);
    req.actions = vec![super::blackbird_secondary::dash(0, true)];
    let r = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(r["execution_completed"], false);
    assert_eq!(r["hits_settled"], 0);
    assert!(r["combat_stamina"]["spends"].as_array().unwrap().is_empty());
}
#[test]
fn fadia_plunge_uses_one_landing_and_inherits_its_delayed_pause_after_jump_cancel() {
    let mut req = request(3);
    super::blackbird_secondary::stamina(&mut req, 100);
    req.actions = vec![super::edgar_actions::make_action(
        0,
        0,
        "GA_Fadia_Melee",
        PLUNGE,
        0,
        0,
    )];
    req.plunge_inputs.insert(
        0,
        GamePlungeInput {
            landing_delay_us: 600000,
            height_bonus: Ratio {
                numerator: 1,
                denominator: 2,
            },
        },
    );
    let r = super::edgar_full::run(&req);
    assert_eq!(r["hits_settled"], 3);
    assert_eq!(r["hit_receipts"][0]["time_us"], 632851);
    assert_eq!(r["hit_receipts"][0]["non_crit_damage"], json!("771/1"));
    assert_eq!(r["final_actor_states"][0]["energy"], 3);
    assert_eq!(r["final_actor_states"][0]["cycle"]["current"], 5);
    req.actions.extend([jump(715245), normal(715246)]);
    let r = super::edgar_full::run(&req);
    assert_eq!(r["hits_settled"], 6);
    assert_eq!(r["hit_receipts"][3]["time_us"], 1130913);
}
#[test]
fn fadia_jump_cancel_forfeits_unissued_contact_and_normal_restarts_at_one() {
    let mut req = request(1);
    super::blackbird_secondary::stamina(&mut req, 100);
    req.actions = vec![normal(0), normal(1212790), jump(1692802), normal(1692803)];
    let r = super::edgar_full::run(&req);
    let hits = r["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 3);
    assert_eq!(hits[1]["damage_effect_id"], "GE_Player_Fadia_Melee2_Damage");
    assert_eq!(hits[2]["damage_effect_id"], "GE_Player_Fadia_Melee1_Damage");
    assert_eq!(hits[2]["time_us"], 2009470);
    req.actions[2] = jump(1692801);
    let r = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(r["execution_completed"], false);
    assert_eq!(r["rejected_action"]["reason"]["until_us"], 1692802);
}
#[test]
fn fadia_skill_cancel_uses_the_integrated_source_gate() {
    let mut req = request(1);
    req.actions = vec![normal(0), super::fadia_effects::skill(730152)];
    let r = super::edgar_full::run(&req);
    assert_eq!(r["hits_settled"], 4);
    assert_eq!(r["hit_receipts"][1]["time_us"], 945815);
    req.actions[1] = super::fadia_effects::skill(730151);
    let r = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(r["execution_completed"], false);
    assert_eq!(r["rejected_action"]["reason"]["until_us"], 730152);
}
#[test]
fn fadia_qualified_support_is_one_native_aoe_packet_and_unqualified_entry_is_nondamaging() {
    let mut req = request(3);
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1004".into();
    ally.initial_cycle = 100;
    req.actors.push(ally);
    req.active_actor_index = 1;
    req.actions = vec![GameActionCommand::SwitchActor {
        timestamp_us: 0,
        target_actor_index: 0,
        cooldown_us: 0,
    }];
    let r = super::edgar_full::run(&req);
    let hits: Vec<_> = r["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["damage_effect_id"] == QTE)
        .collect();
    assert_eq!(hits.len(), 3);
    assert_eq!(hits[0]["time_us"], 685983);
    assert_eq!(hits[0]["non_crit_damage"], json!("1714/1"));
    assert_eq!(r["final_actor_states"][0]["energy"], 7);
    assert_eq!(r["final_actor_states"][0]["charge_remainder"], json!("3/5"));
    req.actors[1].initial_cycle = 99;
    let r = super::edgar_full::run(&req);
    assert_eq!(r["hits_settled"], 0);
}

#[test]
fn fadia_riposte_preserves_form_permission_and_only_mental_followups_keep_high_q() {
    let mut req = request(1);
    req.actors[0].initial_energy = 120;
    req.actors[0].initial_hp = 2455;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([5]).unwrap();
    super::blackbird_secondary::stamina(&mut req, 100);
    req.actions = vec![
        super::fadia_effects::burst(0),
        super::blackbird_secondary::dash(4415921, true),
        normal(5422492),
        normal(6593693),
    ];
    let r = super::edgar_full::run(&req);
    let hits = r["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 8);
    assert_eq!(hits[6]["damage_effect_id"], RIPOSTE);
    assert_eq!(hits[6]["crit_rate"], json!("1/2"));
    assert_eq!(hits[6]["non_crit_damage"], json!("3051/2"));
    assert_eq!(
        hits[7]["damage_effect_id"],
        "GE_Player_Fadia_UltraSkillMelee1_Damage"
    );
    assert_eq!(hits[7]["crit_rate"], json!("1/2"));
    assert_eq!(hits[7]["non_crit_damage"], json!("2431/5"));
}

#[test]
fn fadia_back_dodge_has_its_own_skill_gate_and_does_not_infer_critical_success() {
    let mut req = request(1);
    super::blackbird_secondary::stamina(&mut req, 100);
    let model = crate::HypothesisId::InputsDashDirectionsV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.dash_directions.insert(0, GameDashDirection::Back);
    req.actions = vec![
        super::blackbird_secondary::dash(0, false),
        super::fadia_effects::skill(494283),
    ];
    let r = super::edgar_full::run(&req);
    assert_eq!(r["hits_settled"], 3);
    assert_eq!(r["hit_receipts"][0]["time_us"], 709946);
    req.actions[1] = super::fadia_effects::skill(494282);
    let r = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(r["execution_completed"], false);
    assert_eq!(r["rejected_action"]["reason"]["until_us"], 494283);
}
