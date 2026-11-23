use super::*;
use crate::characters::hathor::power::MODEL;
use serde_json::{Value, json};
fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::hathor_clock::request(targets);
    req.model_assumptions
        .insert(MODEL.into(), MODEL.description().into());
    req
}
fn skill(time: u64) -> GameActionCommand {
    super::hathor_actions::make_action(
        time,
        0,
        "GA_Hathor_Skill",
        "GE_Player_Hathor_Skill1_Damage",
        0,
        0,
    )
}
fn run(req: &GameScenarioRequest) -> Value {
    super::edgar_full::run(req)
}
fn power(report: &Value) -> &Value {
    &report["final_actor_states"][0]["fractional_resources"]["hathor.express_delivery_power"]
}
#[test]
fn hathor_power_tap_has_three_causal_contacts_and_one_notify_not_per_recipient() {
    for targets in [1, 3] {
        let mut req = request(targets);
        req.actions = vec![skill(0)];
        let r = run(&req);
        assert_eq!(r["hits_settled"], 3 * targets);
        let times: Vec<_> = r["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| h["target_index"] == 0)
            .map(|h| h["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(times, [516245, 732420, 982202]);
        assert_eq!(
            r["total_damage_exact"],
            json!(format!("{}/{}", 2571 * targets, 1))
        );
        assert_eq!(power(&r)["current"], json!("2/1"));
        let actor = &r["final_actor_states"][0];
        assert_eq!(actor["energy"], 18);
        assert_eq!(actor["charge_remainder"], json!("499/1000"));
        assert_eq!(actor["cycle"]["current"], 14);
        assert_eq!(actor["cycle"]["carry"], json!("199/1000"));
        req.actions.push(super::hathor_clock::normal(1_577_960));
        assert_eq!(
            run_game_scenario(&req, &ExecutionLimits::default()).unwrap()["rejected_action"]["reason"]
                ["until_us"],
            1_577_961
        );
        *req.actions.last_mut().unwrap() = super::hathor_clock::normal(1_577_961);
        run(&req);
    }
}
#[test]
fn hathor_power_direct_delta_energy_bypasses_charge_and_does_not_share_to_peer() {
    let mut req = request(1);
    req.actions = vec![skill(0)];
    let mut peer = req.actors[0].clone();
    peer.slot = 1;
    peer.character_id = "character:1020".into();
    req.actors.push(peer);
    req.actors[0].gear_procs = vec![GearProcConfig::ChargeEfficiency {
        multiplier: Ratio {
            numerator: 2,
            denominator: 1,
        },
    }];
    let r = run(&req);
    assert_eq!(r["final_actor_states"][0]["energy"], 26);
    assert_eq!(
        r["final_actor_states"][0]["charge_remainder"],
        json!("499/500")
    );
    assert_eq!(r["final_actor_states"][1]["energy"], 5);
    assert_eq!(
        r["final_actor_states"][1]["charge_remainder"],
        json!("497/5000")
    );
    let grants: Vec<_> = r["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "hathor_power_gain")
        .collect();
    assert_eq!(grants.len(), 1);
    assert_eq!(grants[0]["time_us"], 283_333);
    assert_eq!(grants[0]["energy_change"]["gross"], 10);
    assert_eq!(grants[0]["energy_change"]["recipient_actor_index"], 0);
}
#[test]
fn hathor_power_cap_clips_delta_and_a5_counts_only_accepted_lifetime_gain() {
    let mut req = request(1);
    req.relative_duration_us = 70_000_000;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([5]).unwrap();
    req.actions = [0, 20_000_000, 40_000_000, 60_000_000]
        .into_iter()
        .map(skill)
        .collect();
    let r = run(&req);
    let p = power(&r);
    assert_eq!(p["current"], json!("6/1"));
    assert_eq!(p["granted"], json!("8/1"));
    assert_eq!(p["overflow"], json!("2/1"));
    assert_eq!(p["conserved"], true);
    assert_eq!(r["final_actor_states"][0]["energy"], 63);
    assert_eq!(
        r["final_actor_states"][0]["charge_remainder"],
        json!("249/250")
    );
    assert_eq!(
        r["final_actor_states"][0]["resources"]["hathor.a5_earned"]["current"],
        6
    );
    for (hit, expected) in r["hit_receipts"].as_array().unwrap().chunks(3).zip([
        json!("43707/50"),
        json!("22282/25"),
        json!("45421/50"),
        json!("45421/50"),
    ]) {
        for receipt in hit {
            assert_eq!(receipt["non_crit_damage"], expected);
        }
    }
}
#[test]
fn hathor_power_interrupt_preserves_actual_notify_and_cd_boundaries() {
    for (at, current, cooldown, hits) in [
        (283332, 0, 3_283_332, 0),
        (283333, 2, 3_283_333, 0),
        (300072, 2, 3_300_072, 0),
        (300073, 2, 16_300_073, 0),
        (516245, 2, 16_300_073, 1),
    ] {
        let mut req = request(1);
        req.actions = vec![skill(0)];
        req.interrupt_events = vec![GameActorInterruptEvent {
            timestamp_us: at,
            actor_index: 0,
            armor_damage: None,
        }];
        let r = run(&req);
        assert_eq!(power(&r)["current"], json!(format!("{}/{}", current, 1)));
        assert_eq!(r["hits_settled"], hits);
        assert_eq!(
            r["interrupt_receipts"][0]["skill_cooldown_after_us"],
            cooldown
        );
    }
}
#[test]
fn hathor_power_death_clears_current_pool_but_preserves_earned_count_and_ledger() {
    let mut req = request(1);
    let incoming = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(incoming.into(), incoming.description().into());
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([5]).unwrap();
    req.actions = vec![skill(0)];
    req.incoming_damage_events = vec![GameActorDamageEvent {
        timestamp_us: 2_000_000,
        actor_index: 0,
        delivered_damage: req.actors[0].initial_hp,
    }];
    let r = run(&req);
    assert_eq!(power(&r)["current"], json!("0/1"));
    assert_eq!(power(&r)["spent"], json!("2/1"));
    assert_eq!(power(&r)["conserved"], true);
    assert_eq!(
        r["final_actor_states"][0]["resources"]["hathor.a5_earned"]["current"],
        2
    );
}
#[test]
fn hathor_power_b4_own_kills_settle_after_the_causal_aoe_and_require_breakthrough_four() {
    for high in [false, true] {
        let mut req = request(3);
        req.actions = vec![super::hathor_clock::normal(0)];
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([5]).unwrap();
        if !high {
            req.actors[0].character_level = 20;
            req.actors[0].purchased_qte_level = Some(1);
            req.actors[0].character_max_level = Some(20);
            req.actors[0].talent_levels = Some([1; 3]);
        }
        for t in &mut req.targets {
            t.hp_current = 1;
            t.hp_max = 1;
        }
        let r = run(&req);
        assert_eq!(
            power(&r)["current"],
            json!(format!("{}/{}", if high { 3 } else { 0 }, 1))
        );
        if high {
            assert_eq!(r["final_actor_states"][0]["energy"], 16);
            for hit in r["hit_receipts"].as_array().unwrap() {
                assert_eq!(hit["non_crit_damage"], json!("208/1"));
            }
        }
    }
}

#[test]
fn hathor_power_survives_field_exit_and_skill_cooldown_keeps_real_time() {
    let mut req = request(1);
    let mut peer = req.actors[0].clone();
    peer.slot = 1;
    peer.character_id = "character:1021".into();
    req.actors.push(peer);
    req.actions = vec![
        skill(0),
        GameActionCommand::SwitchActor {
            timestamp_us: 2_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 4_000_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        skill(4_000_000),
    ];
    let early = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(early["execution_completed"], false);
    assert_eq!(early["rejected_action"]["reason"]["until_us"], 16_300_073);
    assert_eq!(power(&early)["current"], json!("2/1"));
    *req.actions.last_mut().unwrap() = skill(16_300_073);
    let r = run(&req);
    assert_eq!(power(&r)["current"], json!("4/1"));
}

#[test]
fn hathor_power_does_not_credit_another_owners_kill() {
    let mut req = request(1);
    let mut peer = req.actors[0].clone();
    peer.slot = 1;
    peer.character_id = "character:1021".into();
    req.actors.push(peer);
    req.active_actor_index = 1;
    req.targets[0].hp_current = 1;
    req.targets[0].hp_max = 1;
    req.actions = vec![super::edgar_actions::make_action(
        0,
        1,
        "GA_Edgar_Melee",
        "GE_Player_Edgar_Melee1_Damage",
        0,
        0,
    )];
    let r = run(&req);
    assert_eq!(power(&r)["current"], json!("0/1"));
    assert!(
        r["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|e| e["event_kind"] != "hathor_power_gain")
    );
}

#[test]
fn hathor_power_does_not_credit_another_owners_reaction_kill() {
    let mut req = request(1);
    for (slot, id) in [(1, "character:1019"), (2, "character:1021")] {
        let mut peer = req.actors[0].clone();
        peer.slot = slot;
        peer.character_id = id.into();
        peer.initial_cycle = if slot == 1 { 100 } else { 0 };
        req.actors.push(peer);
    }
    req.active_actor_index = 1;
    req.targets[0].hp_current = 1;
    req.targets[0].hp_max = 1;
    req.actions = vec![GameActionCommand::SwitchActor {
        timestamp_us: 0,
        target_actor_index: 2,
        cooldown_us: 0,
    }];
    let r = run(&req);
    assert_eq!(r["hit_receipts"][0]["actor_index"], 1);
    assert_eq!(power(&r)["current"], json!("0/1"));
}

#[test]
fn hathor_power_b4_credits_owned_break_damage_kills_on_and_off_field_once() {
    for off_field in [false, true] {
        let mut req = request(1);
        req.actions = vec![super::hathor_clock::normal(0)];
        let hp = if off_field { 500 } else { 300 };
        req.targets[0].hp_current = hp;
        req.targets[0].hp_max = hp;
        req.targets[0].break_profile = Some(GameTargetBreakProfileConfig {
            capacity: Ratio::one(),
            current_gauge: Ratio {
                numerator: if off_field { 3 } else { 1 },
                denominator: 10,
            },
            broken_duration_us: 2_000_000,
            discord_fraction: None,
        });
        if off_field {
            let mut peer = req.actors[0].clone();
            peer.slot = 1;
            peer.character_id = "character:1021".into();
            req.actors.push(peer);
            req.actions.push(GameActionCommand::SwitchActor {
                timestamp_us: 2_000_000,
                target_actor_index: 1,
                cooldown_us: 0,
            });
            req.actions.push(super::edgar_actions::make_action(
                2_000_000,
                1,
                "GA_Edgar_Melee",
                "GE_Player_Edgar_Melee1_Damage",
                0,
                0,
            ));
        }
        let r = run(&req);
        assert_eq!(r["hit_receipts"][0]["applied_hp_loss"], 208);
        assert!(r["hit_receipts"].as_array().unwrap().iter().any(|hit| {
            hit["break_contribution"].is_object()
                && hit["actor_index"] == 0
                && hit["applied_hp_loss"].as_u64().is_some_and(|lost| lost > 0)
        }));
        assert_eq!(r["final_target_states"][0]["hp_current"], 0);
        assert_eq!(power(&r)["current"], json!("1/1"));
        let gains: Vec<_> = r["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "hathor_power_gain")
            .collect();
        assert_eq!(gains.len(), 1);
        assert_eq!(gains[0]["active_actor"], if off_field { 1 } else { 0 });
    }
}
