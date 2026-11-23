use super::*;
use crate::characters::hathor::air::{MODEL, PACKET};
use serde_json::{Value, json};
fn request(initial: u64, landing: u64, height: Ratio) -> GameScenarioRequest {
    let mut r = super::hathor_secondary::request(1);
    r.relative_duration_us = 10_000_000;
    r.model_assumptions
        .insert(MODEL.into(), MODEL.description().into());
    super::blackbird_secondary::stamina(&mut r, initial);
    let mut action = super::hathor_clock::normal(0);
    if let GameActionCommand::InvokeAbility {
        damage_effect_id, ..
    } = &mut action
    {
        *damage_effect_id = PACKET.into();
    }
    r.actions = vec![action];
    r.plunge_inputs.insert(
        0,
        GamePlungeInput {
            landing_delay_us: landing,
            height_bonus: height,
        },
    );
    r
}
fn run(r: &GameScenarioRequest) -> Value {
    super::edgar_full::run(r)
}
#[test]
fn hathor_airattack_keeps_one_height_scaled_landing_packet_and_native_resources() {
    for (height, n, d) in [
        (Ratio::zero(), 1071, 1),
        (
            Ratio {
                numerator: 1,
                denominator: 2,
            },
            3213,
            2,
        ),
        (Ratio::one(), 2142, 1),
    ] {
        let r = request(100, 2_000_000, height);
        let report = run(&r);
        assert_eq!(report["hits_settled"], 1);
        assert_eq!(report["hit_receipts"][0]["time_us"], 2_116_417);
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            json!(format!("{}/{}", n, d))
        );
        let actor = &report["final_actor_states"][0];
        assert_eq!(actor["energy"], 6);
        assert_eq!(actor["charge_remainder"], json!("3/10"));
        assert_eq!(actor["cycle"]["current"], 10);
        assert_eq!(actor["cycle"]["carry"], json!("2/5"));
        assert_eq!(report["combat_stamina"]["total_spent"], json!("30/1"));
    }
}
#[test]
fn hathor_airattack_repeats_the_half_second_section_then_lands_on_actual_low_strength() {
    let r = request(35, 8_000_000, Ratio::zero());
    let report = run(&r);
    let spends = report["combat_stamina"]["spends"].as_array().unwrap();
    assert_eq!(spends.len(), 3);
    for (s, (time, after)) in spends
        .iter()
        .zip([(616_667, 25), (1_116_667, 15), (1_616_667, 5)])
    {
        assert_eq!(s["time_us"], time);
        assert_eq!(s["after"], json!(format!("{}/{}", after, 1)));
    }
    assert_eq!(report["hit_receipts"][0]["time_us"], 1_733_084);
    assert_eq!(report["combat_stamina"]["recovery_ready_at_us"], 3_616_667);
    assert_eq!(report["combat_stamina"]["current"], json!("5/1"));
}
#[test]
fn hathor_airattack_positive_strength_guard_is_not_a_flat_ten_point_admission_fee() {
    for initial in [1, 10] {
        let report = run(&request(initial, 2_000_000, Ratio::zero()));
        assert_eq!(report["hit_receipts"][0]["time_us"], 116_417);
        assert_eq!(report["combat_stamina"]["total_spent"], json!("0/1"));
    }
    let report = run_game_scenario(
        &request(0, 2_000_000, Ratio::zero()),
        &ExecutionLimits::default(),
    )
    .unwrap();
    assert_eq!(report["execution_completed"], false);
    assert_eq!(report["hits_settled"], 0);
    assert_eq!(report["combat_stamina"]["total_spent"], json!("0/1"));
    assert!(
        report["rejected_action"]["reason"]["message"]
            .as_str()
            .unwrap()
            .contains("positive Strength")
    );
}
#[test]
fn hathor_airattack_scaled_loop_cost_caps_at_available_strength_and_lands_once() {
    let mut r = request(15, 3_000_000, Ratio::zero());
    r.combat_stamina.as_mut().unwrap().use_efficiency = Ratio {
        numerator: 2,
        denominator: 1,
    };
    let report = run(&r);
    assert_eq!(report["hits_settled"], 1);
    assert_eq!(report["hit_receipts"][0]["time_us"], 733_084);
    assert_eq!(report["combat_stamina"]["total_spent"], json!("15/1"));
    assert_eq!(report["combat_stamina"]["current"], json!("0/1"));
}
#[test]
fn hathor_airattack_real_landing_wins_over_equal_time_cost_and_cancellation_keeps_actual_spends() {
    for (landing, spent) in [(600_000, 0), (616_667, 0), (616_668, 10)] {
        let report = run(&request(100, landing, Ratio::zero()));
        assert_eq!(report["hit_receipts"][0]["time_us"], landing + 116_417);
        assert_eq!(
            report["combat_stamina"]["total_spent"],
            json!(format!("{}/{}", spent, 1))
        );
    }
    for (time, spent) in [(500_000, 0), (700_000, 10)] {
        let mut r = request(100, 2_000_000, Ratio::zero());
        r.interrupt_events.push(GameActorInterruptEvent {
            timestamp_us: time,
            actor_index: 0,
            armor_damage: None,
        });
        let report = run(&r);
        assert_eq!(report["hits_settled"], 0);
        assert_eq!(
            report["combat_stamina"]["total_spent"],
            json!(format!("{}/{}", spent, 1))
        );
        assert_eq!(report["final_actor_states"][0]["energy"], 0);
    }
}
#[test]
fn hathor_airattack_inherits_source_dilation_without_extending_authored_real_airtime() {
    let mut r = request(100, 1_000_000, Ratio::zero());
    let mut old = r.actors[0].clone();
    old.slot = 1;
    old.character_id = "character:1020".into();
    old.initial_cycle = 100;
    r.actors.push(old);
    r.active_actor_index = 1;
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut r.actions[0] {
        *timestamp_us = 800_001;
    }
    r.actions.insert(
        0,
        GameActionCommand::SwitchActor {
            timestamp_us: 0,
            target_actor_index: 0,
            cooldown_us: 0,
        },
    );
    let plunge = r.plunge_inputs.remove(&0).unwrap();
    r.plunge_inputs.insert(1, plunge);
    r.interrupt_events.push(GameActorInterruptEvent {
        timestamp_us: 800_000,
        actor_index: 0,
        armor_damage: None,
    });
    let report = run(&r);
    assert_eq!(report["hits_settled"], 2);
    assert_eq!(report["hit_receipts"][1]["time_us"], 1_916_418);
    assert_eq!(report["combat_stamina"]["spends"][0]["time_us"], 1_603_437);
    assert_eq!(
        report["combat_stamina"]["spends"].as_array().unwrap().len(),
        1
    );
}

#[test]
fn hathor_airattack_aoe_deduplicates_resources_and_zero_damage_keeps_actual_stamina_costs() {
    let mut r = request(100, 2_000_000, Ratio::zero());
    for index in 1..4 {
        let mut t = r.targets[0].clone();
        t.target_index = index;
        r.targets.push(t);
    }
    let report = run(&r);
    assert_eq!(report["hits_settled"], 4);
    assert_eq!(report["final_actor_states"][0]["energy"], 6);
    assert_eq!(report["final_actor_states"][0]["cycle"]["current"], 10);
    r.actors[0].static_stats.atk = Ratio {
        numerator: 1,
        denominator: 10,
    };
    let report = run(&r);
    for h in report["hit_receipts"].as_array().unwrap() {
        assert_eq!(h["applied_hp_loss"], 0);
    }
    assert_eq!(report["final_actor_states"][0]["energy"], 0);
    assert!(report["final_actor_states"][0]["cycle"].is_null());
    assert_eq!(report["combat_stamina"]["total_spent"], json!("30/1"));
}

#[test]
fn hathor_airattack_early_landing_reprojects_recovery_and_preserves_normal_r3_talent() {
    let mut r = request(35, 8_000_000, Ratio::zero());
    r.actions.push(super::hathor_clock::normal(2_201_014));
    assert_eq!(
        run_game_scenario(&r, &ExecutionLimits::default()).unwrap()["rejected_action"]["reason"]["until_us"],
        2_201_015
    );
    r.actions[1] = super::hathor_clock::normal(2_201_015);
    assert_eq!(run(&r)["hits_settled"], 2);
    r.actions.truncate(1);
    r.actors[0].awakening = ntesim_data::AwakeningSelection::selected(vec![1, 3, 6]).unwrap();
    r.targets[0].is_boss = true;
    assert_eq!(
        run(&r)["hit_receipts"][0]["non_crit_damage"],
        json!("1157/1")
    );
}
