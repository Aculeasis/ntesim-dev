use super::*;
use crate::characters::haniel::cannon::{MODEL, PACKET};
use serde_json::{Value, json};

fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::haniel_clock::request();
    req.model_assumptions
        .insert(MODEL.into(), MODEL.description().into());
    let full = crate::characters::haniel::secondary::MODEL;
    req.model_assumptions
        .insert(full.into(), full.description().into());
    for index in 1..targets {
        let mut target = req.targets[0].clone();
        target.target_index = index;
        req.targets.push(target);
    }
    req
}
fn normal(time: u64) -> GameActionCommand {
    super::haniel_actions::make_action(
        time,
        0,
        "GA_Haniel_Melee",
        "GE_Player_Haniel_Melee1_Damage",
        0,
        0,
    )
}
fn skill(time: u64) -> GameActionCommand {
    super::haniel_actions::make_action(
        time,
        0,
        "GA_Haniel_Skill",
        "GE_Player_Haniel_Skill1_Damage",
        0,
        0,
    )
}
fn hold(req: &mut GameScenarioRequest, time: u64, duration_us: u64) {
    let command = req.actions.len();
    req.actions.push(normal(time));
    req.hold_inputs.insert(
        command,
        GameHoldInput {
            duration_us,
            moving: false,
            reserved_stocks: None,
        },
    );
}
fn chain(req: &mut GameScenarioRequest, mut time: u64) {
    for recovery in [887510, 722267, 417606, 655768, 950032] {
        req.actions.push(normal(time));
        time += recovery;
    }
}
pub(super) fn plot_two(targets: usize) -> GameScenarioRequest {
    let mut req = request(targets);
    req.actions.push(skill(0));
    chain(&mut req, 1_309_354);
    chain(&mut req, 4_942_537);
    hold(&mut req, 8_575_720, 200_000);
    req
}
fn run(req: &GameScenarioRequest) -> Value {
    super::edgar_full::run(req)
}
fn shots(r: &Value) -> Vec<&Value> {
    r["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["damage_effect_id"] == PACKET)
        .collect()
}

#[test]
fn haniel_cannon_preloaded_plot_is_captured_twice_without_spending_or_ensemble() {
    for (awakening, plot) in [(0, 4), (1, 6)] {
        let mut req = request(3);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actors[0].initial_resources.plot = Some(Ratio {
            numerator: plot,
            denominator: 1,
        });
        hold(&mut req, 0, 200_000);
        hold(&mut req, 3_000_000, 200_000);
        let result = run(&req);
        assert_eq!(shots(&result).len(), 2 * (1 + 3 * plot as usize));
        assert_eq!(
            result["final_actor_states"][0]["resources"]["haniel.plot_armor"]["current"],
            plot
        );
        let legs: Vec<_> = result["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "haniel_cannon_leg")
            .collect();
        assert_eq!(legs.len(), 2 * (1 + plot as usize));
        assert!(
            result["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .all(|hit| hit["damage_effect_id"] != crate::characters::haniel::hootie::ENSEMBLE)
        );
    }
}

#[test]
fn haniel_cannon_short_hold_is_tap_and_long_hold_cannot_bank_another_projectile() {
    for (duration, count) in [
        (0, 0),
        (199999, 0),
        (200000, 1),
        (300000, 1),
        (2_000_000, 1),
    ] {
        let mut req = request(1);
        hold(&mut req, 0, duration);
        let r = run(&req);
        assert_eq!(r["hit_receipts"][0]["time_us"], 179633);
        let cannon = shots(&r);
        assert_eq!(cannon.len(), count);
        if count == 1 {
            assert_eq!(cannon[0]["time_us"], 1_061_263);
            assert_eq!(cannon[0]["non_crit_damage"], json!("617/2"));
            assert_eq!(
                r["final_actor_states"][0]["charge_remainder"],
                json!("101/200")
            );
        }
    }
}

#[test]
fn haniel_cannon_plot_two_routes_distinct_anchors_and_resources_are_per_packet_not_recipient() {
    for (targets, deliveries, energy, carry_n, cycle, cycle_n) in [
        (1, 1, 36, 909, 61, 427),
        (2, 5, 38, 669, 64, 347),
        (3, 7, 38, 669, 64, 347),
    ] {
        let req = plot_two(targets);
        let r = run(&req);
        let cannon = shots(&r);
        assert_eq!(cannon.len(), deliveries);
        for hit in &cannon {
            assert_eq!(hit["time_us"], 9_636_983);
            assert_eq!(hit["non_crit_damage"], json!("617/2"));
        }
        let actor = &r["final_actor_states"][0];
        assert_eq!(actor["energy"], energy);
        assert_eq!(
            actor["charge_remainder"],
            json!(format!("{}/{}", carry_n, 1000))
        );
        assert_eq!(actor["cycle"]["current"], cycle);
        assert_eq!(
            actor["cycle"]["carry"],
            json!(format!("{}/{}", cycle_n, 1000))
        );
        assert_eq!(actor["resources"]["haniel.plot_armor"]["current"], 2);
        let legs: Vec<_> = r["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["event_kind"] == "haniel_cannon_leg")
            .collect();
        assert_eq!(legs.len(), if targets == 1 { 1 } else { 3 });
        if targets > 1 {
            assert_eq!(legs[1]["data"]["ordinal"], 1);
            assert_eq!(legs[1]["data"]["target_index"], 1);
            assert_eq!(legs[2]["data"]["ordinal"], 2);
            assert_eq!(legs[2]["data"]["target_index"], 0);
        }
    }
}

#[test]
fn haniel_cannon_connected_pause_extends_recovery_once_per_leg_and_next_normal_resets() {
    let mut req = plot_two(2);
    req.actions.push(normal(9_847_462));
    let early = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(early["execution_completed"], false);
    assert_eq!(early["rejected_action"]["reason"]["until_us"], 9_847_463);
    *req.actions.last_mut().unwrap() = normal(9_847_463);
    let r = run(&req);
    assert_eq!(shots(&r).len(), 5);
    assert_eq!(
        r["hit_receipts"].as_array().unwrap().last().unwrap()["damage_effect_id"],
        "GE_Player_Haniel_Melee1_Damage"
    );
}

#[test]
fn haniel_cannon_interruption_cancels_unissued_shot_but_not_launched_child_lineage() {
    for (time, count) in [(9_636_982, 0), (9_636_983, 5)] {
        let mut req = plot_two(2);
        req.interrupt_events = vec![GameActorInterruptEvent {
            timestamp_us: time,
            actor_index: 0,
            armor_damage: None,
        }];
        let r = run(&req);
        assert_eq!(shots(&r).len(), count);
        assert_eq!(
            r["final_actor_states"][0]["resources"]["haniel.plot_armor"]["current"],
            2
        );
    }
}

#[test]
fn haniel_cannon_ace_release_uses_live_strict_window_and_ignores_excess_hold() {
    for (duration, shot, recovery) in [
        (200000, 4_944_597, 5_115_078),
        (650000, 4_994_596, 5_165_077),
        (700000, 5_361_263, 5_531_743),
        (2_000_000, 5_361_263, 5_531_743),
    ] {
        let mut req = request(1);
        req.actors[0].initial_energy = 120;
        req.actions.push(super::haniel_actions::make_action(
            0,
            0,
            "GA_Haniel_UltraSkill",
            "GE_Player_Haniel_Ultraskill1_Damage",
            120,
            0,
        ));
        hold(&mut req, 4_300_000, duration);
        req.actions.push(normal(recovery - 1));
        let r = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(r["execution_completed"], false);
        assert_eq!(shots(&r)[0]["time_us"], shot);
        assert_eq!(r["rejected_action"]["reason"]["until_us"], recovery);
        *req.actions.last_mut().unwrap() = normal(recovery);
        run(&req);
    }
    // Ace expires exactly at the first strict fast gate: it cannot admit early End.
    let mut req = request(1);
    req.actors[0].initial_energy = 120;
    req.actions.push(super::haniel_actions::make_action(
        0,
        0,
        "GA_Haniel_UltraSkill",
        "GE_Player_Haniel_Ultraskill1_Damage",
        120,
        0,
    ));
    hold(&mut req, 13_637_613, 200000);
    let r = run(&req);
    assert_eq!(shots(&r)[0]["time_us"], 14_698_876);
}

#[test]
fn haniel_cannon_a1_cap_six_persists_across_speakers_and_is_not_spent_by_shots() {
    let mut req = request(2);
    req.relative_duration_us = 60_000_000;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([1]).unwrap();
    for base in [0, 20_684_620, 41_369_240] {
        req.actions.push(skill(base));
        chain(&mut req, base + 1_309_354);
        chain(&mut req, base + 4_942_537);
    }
    hold(&mut req, 49_944_960, 200000);
    hold(&mut req, 51_944_960, 200000);
    let r = run(&req);
    assert_eq!(shots(&r).len(), 26);
    assert_eq!(
        r["final_actor_states"][0]["resources"]["haniel.plot_armor"]["current"],
        6
    );
}
