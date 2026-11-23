use super::blackbird_secondary::dash;
use super::hathor_burst::skill;
use super::hathor_clock::normal;
use super::*;
use crate::characters::hathor::secondary::{MODEL, QTE, QTE2, RIPOSTE, RIPOSTE2};
use serde_json::{Value, json};
pub(super) fn request(targets: usize) -> GameScenarioRequest {
    let mut r = super::hathor_burst::request(targets);
    r.actors[0].initial_energy = 0;
    r.relative_duration_us = 15_000_000;
    for m in [
        crate::HypothesisId::HathorCombatLifecycleV1,
        crate::HypothesisId::HathorCycloneV1,
        crate::HypothesisId::HathorB2RemoraV1,
        MODEL,
    ] {
        r.model_assumptions.insert(m.into(), m.description().into());
    }
    super::blackbird_secondary::stamina(&mut r, 100);
    r
}
fn run(r: &GameScenarioRequest) -> Value {
    super::edgar_full::run(r)
}
fn power(r: &Value) -> &Value {
    &r["final_actor_states"][0]["fractional_resources"]["hathor.express_delivery_power"]
}
fn jump(t: u64) -> GameActionCommand {
    GameActionCommand::PlayerInput {
        timestamp_us: t,
        actor_index: 0,
        input: PlayerInput::Jump,
        critical_success: false,
    }
}
#[test]
fn hathor_secondary_paid_dodge_arms_one_two_contact_riposte_and_preserves_exact_resources() {
    let mut r = request(1);
    r.actions = vec![dash(0, true), normal(996_277), normal(2_179_825)];
    let report = run(&r);
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 3);
    assert_eq!(hits[0]["damage_effect_id"], RIPOSTE);
    assert_eq!(hits[1]["damage_effect_id"], RIPOSTE2);
    assert_eq!(
        hits[2]["damage_effect_id"],
        "GE_Player_Hathor_Melee1_Damage"
    );
    assert_eq!(hits[0]["time_us"], 1_337_680);
    assert_eq!(hits[1]["time_us"], 1_646_151);
    assert_eq!(hits[0]["non_crit_damage"], json!("1165/2"));
    assert_eq!(hits[1]["non_crit_damage"], json!("1114/1"));
    assert_eq!(power(&report)["granted"], json!("1/1"));
    assert_eq!(report["final_actor_states"][0]["energy"], 11);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        json!("219/1000")
    );
    assert_eq!(report["combat_stamina"]["total_spent"], json!("15/1"));
    r.actions[1] = normal(996_276);
    assert_eq!(
        run_game_scenario(&r, &ExecutionLimits::default()).unwrap()["rejected_action"]["reason"]["until_us"],
        996_277
    );
}
#[test]
fn hathor_secondary_riposte_gate_deduplicates_aoe_and_second_contact_and_a5_settles_after_packet() {
    let mut r = request(3);
    r.actors[0].awakening = ntesim_data::AwakeningSelection::selected(vec![5]).unwrap();
    r.actions = vec![
        dash(0, true),
        normal(1_000_000),
        dash(2_500_000, true),
        normal(3_500_000),
        dash(5_000_000, true),
        normal(6_000_000),
    ];
    let report = run(&r);
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 18);
    for hit in &hits[..3] {
        assert_eq!(hit["non_crit_damage"], json!("1165/2"));
    }
    for hit in &hits[3..6] {
        assert_eq!(hit["non_crit_damage"], json!("56257/50"));
    }
    for hit in &hits[6..9] {
        assert_eq!(hit["non_crit_damage"], json!("23533/40"));
    }
    assert_eq!(power(&report)["granted"], json!("2/1"));
    let awards: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "hathor_power_gain")
        .map(|e| e["time_us"].as_u64().unwrap())
        .collect();
    assert_eq!(awards, [1_341_403, 6_341_403]);
}
#[test]
fn hathor_secondary_jump_cancels_unissued_riposte_without_refund_or_reward_and_skill_uses_dodge_gate()
 {
    let mut r = request(1);
    r.actions = vec![
        dash(0, true),
        normal(1_000_000),
        jump(1_000_000),
        normal(2_000_000),
    ];
    let report = run(&r);
    assert_eq!(report["hits_settled"], 1);
    assert_eq!(
        report["hit_receipts"][0]["damage_effect_id"],
        "GE_Player_Hathor_Melee1_Damage"
    );
    assert_eq!(power(&report)["granted"], json!("0/1"));
    assert_eq!(report["combat_stamina"]["total_spent"], json!("15/1"));
    for (back, gate) in [(false, 267_501), (true, 331_208)] {
        let mut r = request(1);
        if back {
            r.dash_directions.insert(0, GameDashDirection::Back);
        }
        r.actions = vec![dash(0, true), skill(gate - 1)];
        assert_eq!(
            run_game_scenario(&r, &ExecutionLimits::default()).unwrap()["rejected_action"]["reason"]
                ["until_us"],
            gate
        );
        r.actions[1] = skill(gate);
        let report = run(&r);
        assert_eq!(report["hits_settled"], 3);
        assert_eq!(power(&report)["granted"], json!("2/1"));
    }
}
#[test]
fn hathor_secondary_explicit_parry_requires_a_paid_live_dodge_window() {
    let mut r = request(1);
    r.actions = vec![dash(0, false), normal(1_000_000)];
    r.parry_events = vec![GameActorParryEvent::At {
        timestamp_us: 100_000,
        actor_index: 0,
    }];
    let report = run(&r);
    assert_eq!(report["hits_settled"], 2);
    assert_eq!(report["parry_receipts"].as_array().unwrap().len(), 1);
    assert_eq!(power(&report)["granted"], json!("1/1"));
    r.actions = vec![normal(1_000_000)];
    assert!(run_game_scenario(&r, &ExecutionLimits::default()).is_err());
}
#[test]
fn hathor_secondary_support_is_qualified_and_keeps_both_packets_and_native_qte_talent() {
    for cycle in [0, 100] {
        for rank in [0, 3] {
            let mut r = request(1);
            r.targets[0].is_boss = true;
            r.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(rank);
            let mut peer = r.actors[0].clone();
            peer.slot = 1;
            peer.character_id = "character:1020".into();
            peer.initial_cycle = cycle;
            r.actors.push(peer);
            r.active_actor_index = 1;
            r.actions = vec![GameActionCommand::SwitchActor {
                timestamp_us: 0,
                target_actor_index: 0,
                cooldown_us: 0,
            }];
            let report = run(&r);
            let hits = report["hit_receipts"].as_array().unwrap();
            if cycle == 100 {
                assert_eq!(hits.len(), 2);
                assert_eq!(hits[0]["damage_effect_id"], QTE);
                assert_eq!(hits[1]["damage_effect_id"], QTE2);
                assert_eq!(hits[0]["time_us"], 633_462);
                assert_eq!(hits[1]["time_us"], 1_144_235);
                assert_eq!(
                    hits[0]["non_crit_damage"],
                    // Qualified Psyche -> Lakshana Stain adds one20% damage zone.
                    json!("657/1")
                );
                assert_eq!(hits[1]["non_crit_damage"], json!("6999/5"));
                r.actions.push(normal(1_703_066));
                assert_eq!(
                    run_game_scenario(&r, &ExecutionLimits::default()).unwrap()["rejected_action"]
                        ["reason"]["until_us"],
                    1_703_067
                );
                *r.actions.last_mut().unwrap() = normal(1_703_067);
                run(&r);
            } else {
                assert!(hits.is_empty());
                r.actions.push(normal(0));
                let report = run(&r);
                assert_eq!(
                    report["hit_receipts"][0]["damage_effect_id"],
                    "GE_Player_Hathor_Melee1_Damage"
                );
            }
        }
    }
}
#[test]
fn hathor_secondary_qte_interruption_preserves_real_point_two_dilation_until_its_interval_ends() {
    let mut r = request(1);
    let mut old = r.actors[0].clone();
    old.slot = 1;
    old.character_id = "character:1020".into();
    old.initial_cycle = 100;
    r.actors.push(old);
    r.active_actor_index = 1;
    r.actions = vec![
        GameActionCommand::SwitchActor {
            timestamp_us: 0,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        normal(800_001),
    ];
    r.interrupt_events = vec![GameActorInterruptEvent {
        timestamp_us: 800_000,
        actor_index: 0,
        armor_damage: None,
    }];
    let report = run(&r);
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0]["damage_effect_id"], QTE);
    assert_eq!(
        hits[1]["damage_effect_id"],
        "GE_Player_Hathor_Melee1_Damage"
    );
    // Remaining233461us at rate.2 gives46692.2 actor us, then170053.8us at1.
    assert_eq!(hits[1]["time_us"], 1_203_516);
    assert_eq!(
        report["interrupt_receipts"][0]["cancelled_hit_deliveries"],
        1
    );
}
#[test]
fn hathor_secondary_dodge_recovery_and_parry_window_share_continuing_owner_dilation() {
    let mut r = request(1);
    let mut old = r.actors[0].clone();
    old.slot = 1;
    old.character_id = "character:1020".into();
    old.initial_cycle = 100;
    r.actors.push(old);
    r.active_actor_index = 1;
    r.actions = vec![
        GameActionCommand::SwitchActor {
            timestamp_us: 0,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        dash(800_001, false),
        normal(2_000_000),
    ];
    r.interrupt_events = vec![GameActorInterruptEvent {
        timestamp_us: 800_000,
        actor_index: 0,
        armor_damage: None,
    }];
    r.parry_events = vec![GameActorParryEvent::At {
        timestamp_us: 1_900_000,
        actor_index: 0,
    }];
    let report = run(&r);
    assert_eq!(report["hits_settled"], 3);
    assert_eq!(report["hit_receipts"][1]["damage_effect_id"], RIPOSTE);
    assert_eq!(power(&report)["granted"], json!("1/1"));
    r.actions[2] = normal(1_983_046);
    assert_eq!(
        run_game_scenario(&r, &ExecutionLimits::default()).unwrap()["rejected_action"]["reason"]["until_us"],
        1_983_047
    );
}
#[test]
fn hathor_secondary_old_riposte_reward_cannot_cross_equal_time_combat_restart() {
    let mut r = request(1);
    r.actions = vec![
        dash(0, true),
        normal(1_000_000),
        dash(2_500_000, true),
        normal(3_500_000),
    ];
    r.combat_transition_events = vec![
        GameActorCombatTransitionEvent {
            timestamp_us: 1_341_403,
            actor_index: 0,
            leaving: true,
        },
        GameActorCombatTransitionEvent {
            timestamp_us: 1_341_403,
            actor_index: 0,
            leaving: false,
        },
    ];
    let report = run(&r);
    assert_eq!(power(&report)["granted"], json!("1/1"));
    let awards: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "hathor_power_gain")
        .map(|e| e["time_us"].as_u64().unwrap())
        .collect();
    assert_eq!(awards, [3_841_403]);
    assert_eq!(report["hits_settled"], 3);
}

#[test]
fn hathor_secondary_unpaid_or_zero_damage_response_cannot_generate_power() {
    let mut r = request(1);
    r.actions = vec![dash(0, false), normal(1_000_000)];
    let report = run(&r);
    assert_eq!(report["hits_settled"], 1);
    assert_eq!(power(&report)["granted"], json!("0/1"));
    if let GameActionCommand::InvokeAbility {
        damage_effect_id, ..
    } = &mut r.actions[1]
    {
        *damage_effect_id = RIPOSTE.into();
    }
    assert!(
        run_game_scenario(&r, &ExecutionLimits::default())
            .unwrap_err()
            .contains("Riposte response is not ready")
    );
    r.actions = vec![dash(0, true), normal(1_000_000)];
    r.actors[0].static_stats.atk = Ratio {
        numerator: 1,
        denominator: 10,
    };
    let report = run(&r);
    assert_eq!(report["hits_settled"], 2);
    for hit in report["hit_receipts"].as_array().unwrap() {
        assert_eq!(hit["applied_hp_loss"], 0);
    }
    assert_eq!(power(&report)["granted"], json!("0/1"));
}

#[test]
fn hathor_secondary_emergency_rejects_reward_but_preserves_its_five_second_gate() {
    let mut r = request(1);
    r.actors[0].initial_energy = 100;
    r.actions = vec![
        super::hathor_burst::burst(0),
        dash(5_000_000, true),
        normal(6_000_000),
        dash(9_000_000, true),
        normal(10_000_000),
        dash(12_000_000, true),
        normal(13_000_000),
    ];
    let report = run(&r);
    assert_eq!(report["hits_settled"], 9);
    assert_eq!(power(&report)["granted"], json!("1/1"));
    let awards: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "hathor_power_gain")
        .map(|e| e["time_us"].as_u64().unwrap())
        .collect();
    assert_eq!(awards, [13_341_403]);
}

#[test]
fn hathor_secondary_normal_jump_and_burst_switch_use_their_own_source_gates() {
    let mut r = request(1);
    r.actions = vec![normal(0), jump(350_248)];
    assert_eq!(
        run_game_scenario(&r, &ExecutionLimits::default()).unwrap()["rejected_action"]["reason"]["until_us"],
        350_249
    );
    r.actions[1] = jump(350_249);
    assert_eq!(run(&r)["hits_settled"], 1);
    let mut peer = r.actors[0].clone();
    peer.slot = 1;
    peer.character_id = "character:1020".into();
    r.actors.push(peer);
    r.actors[0].initial_energy = 100;
    r.actions = vec![
        super::hathor_burst::burst(0),
        GameActionCommand::SwitchActor {
            timestamp_us: 4_522_840,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    assert_eq!(
        run_game_scenario(&r, &ExecutionLimits::default()).unwrap()["rejected_action"]["reason"]["until_us"],
        4_522_841
    );
    if let GameActionCommand::SwitchActor { timestamp_us, .. } = &mut r.actions[1] {
        *timestamp_us = 4_522_841;
    }
    assert_eq!(run(&r)["hits_settled"], 3);
}

#[test]
fn hathor_secondary_r3_raises_riposte_normal_talent_and_field_exit_keeps_reward_gate() {
    let mut r = request(1);
    r.targets[0].is_boss = true;
    r.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(3);
    r.actions = vec![dash(0, true), normal(1_000_000)];
    let report = run(&r);
    assert_eq!(
        report["hit_receipts"][0]["non_crit_damage"],
        json!("1259/2")
    );
    assert_eq!(
        report["hit_receipts"][1]["non_crit_damage"],
        json!("1203/1")
    );
    // Isolate field-exit reward gating from the independent A2 Gaze expiry.
    r.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
    let mut peer = r.actors[0].clone();
    peer.slot = 1;
    peer.character_id = "character:1020".into();
    r.actors.push(peer);
    r.actions.extend([
        GameActionCommand::SwitchActor {
            timestamp_us: 2_200_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 2_500_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        dash(4_000_000, true),
        normal(5_000_000),
    ]);
    assert_eq!(power(&run(&r))["granted"], json!("1/1"));
}

#[test]
fn hathor_secondary_support_switch_gate_is_distinct_from_ordinary_recovery() {
    let mut r = request(1);
    let mut old = r.actors[0].clone();
    old.slot = 1;
    old.character_id = "character:1020".into();
    old.initial_cycle = 100;
    r.actors.push(old);
    r.active_actor_index = 1;
    r.actions = vec![
        GameActionCommand::SwitchActor {
            timestamp_us: 0,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 1_563_462,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    assert_eq!(
        run_game_scenario(&r, &ExecutionLimits::default()).unwrap()["rejected_action"]["reason"]["until_us"],
        1_563_463
    );
    if let GameActionCommand::SwitchActor { timestamp_us, .. } = &mut r.actions[1] {
        *timestamp_us = 1_563_463;
    }
    assert_eq!(run(&r)["hits_settled"], 2);
}
