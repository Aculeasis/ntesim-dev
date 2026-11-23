use super::hathor_burst::{burst, skill};
use super::*;
use serde_json::{Value, json};
fn request() -> GameScenarioRequest {
    let mut r = super::hathor_burst::request(1);
    let m = crate::HypothesisId::HathorCycloneV1;
    r.model_assumptions.insert(m.into(), m.description().into());
    r.relative_duration_us = 75_000_000;
    r.actions = vec![
        skill(0),
        skill(20_000_000),
        skill(40_000_000),
        burst(50_000_000),
    ];
    r
}
fn run(r: &GameScenarioRequest) -> Value {
    super::edgar_full::run(r)
}
fn power(r: &Value) -> &Value {
    &r["final_actor_states"][0]["fractional_resources"]["hathor.express_delivery_power"]
}
#[test]
fn hathor_cyclone_spends_one_two_three_and_preserves_real_connected_chain() {
    let mut r = request();
    r.actions
        .extend([skill(55_000_000), skill(56_343_016), skill(57_793_023)]);
    r.actions.push(super::hathor_clock::normal(59_420_539));
    let report = run(&r);
    assert_eq!(power(&report)["spent"], json!("6/1"));
    assert_eq!(power(&report)["current"], json!("0/1"));
    let hits = report["hit_receipts"].as_array().unwrap();
    let cyclone: Vec<_> = hits
        .iter()
        .filter(|h| {
            crate::characters::hathor::cyclone::packet(h["damage_effect_id"].as_str().unwrap())
        })
        .collect();
    assert_eq!(cyclone.len(), 12);
    assert_eq!(cyclone[0]["time_us"], 55_749_916);
    assert_eq!(cyclone[6]["time_us"], 56_244_033);
    assert_eq!(cyclone[7]["time_us"], 56_859_640);
    assert_eq!(cyclone[10]["time_us"], 58_076_085);
    // Actual Power6 at state application gives ATK1900, Skill5 level8=.735, DEF=.5.
    assert_eq!(cyclone[0]["non_crit_damage"], json!("2793/4"));
    if let Some(GameActionCommand::InvokeAbility { timestamp_us, .. }) = r.actions.last_mut() {
        *timestamp_us -= 1;
    }
    let rejected = run_game_scenario(&r, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        rejected["rejected_action"]["reason"]["until_us"],
        59_420_539
    );
}
#[test]
fn hathor_cyclone_insufficient_next_stage_does_not_refund_or_fall_back_to_tap() {
    let mut r = request();
    r.actions = vec![
        skill(0),
        burst(2_000_000),
        skill(7_000_000),
        skill(8_343_016),
    ];
    let report = run_game_scenario(&r, &ExecutionLimits::default()).unwrap();
    assert!(
        report["rejected_action"]["reason"]["message"]
            .as_str()
            .unwrap_or("")
            .contains("requires2 Power"),
        "{}",
        report["rejected_action"]
    );
    assert_eq!(power(&report)["spent"], json!("1/1"));
    assert_eq!(power(&report)["current"], json!("1/1"));
    r.actions = vec![burst(0), skill(5_000_000)];
    let report = run_game_scenario(&r, &ExecutionLimits::default()).unwrap();
    assert!(
        report["rejected_action"]["reason"]["message"]
            .as_str()
            .unwrap_or("")
            .contains("current0")
    );
    assert_eq!(report["hits_settled"], 3);
}
#[test]
fn hathor_cyclone_hidden_ordinary_cooldown_restores_before_equal_time_state_end_input() {
    let mut r = request();
    r.actions = vec![
        skill(0),
        burst(2_000_000),
        skill(7_000_000),
        skill(14_518_597),
    ];
    let report = run_game_scenario(&r, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["rejected_action"]["reason"]["until_us"], 20_818_670);
    *r.actions.last_mut().unwrap() = skill(20_818_670);
    let report = run(&r);
    assert_eq!(power(&report)["spent"], json!("1/1"));
    assert_eq!(power(&report)["current"], json!("3/1"));
}
#[test]
fn hathor_cyclone_a4_counts_completed_positive_stages_once_and_snapshots_previous_count() {
    let mut r = request();
    r.actors[0].awakening = ntesim_data::AwakeningSelection::selected(vec![4]).unwrap();
    r.actions
        .extend([skill(55_000_000), skill(56_343_016), skill(57_793_023)]);
    let report = run(&r);
    let hits: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| {
            crate::characters::hathor::cyclone::packet(h["damage_effect_id"].as_str().unwrap())
        })
        .collect();
    for (index, hit) in hits.iter().enumerate() {
        let expected = if index < 7 {
            json!("0/1")
        } else if index < 10 {
            json!("1/4")
        } else {
            json!("1/2")
        };
        assert_eq!(hit["crit_rate"], expected);
    }
    // A marker reached after positive contacts succeeds even when its recovery is interrupted.
    for (at, expected) in [(56_292_981, 0), (56_292_982, 1)] {
        let mut r = request();
        r.actors[0].awakening = ntesim_data::AwakeningSelection::selected(vec![4]).unwrap();
        r.actions.extend([skill(55_000_000), skill(57_000_000)]);
        r.interrupt_events = vec![GameActorInterruptEvent {
            timestamp_us: at,
            actor_index: 0,
            armor_damage: None,
        }];
        let report = run(&r);
        let last = report["hit_receipts"].as_array().unwrap().last().unwrap();
        assert_eq!(
            last["crit_rate"],
            json!(format!(
                "{}/{}",
                expected,
                if expected == 0 { 1 } else { 4 }
            ))
        );
        assert_eq!(power(&report)["spent"], json!("2/1"));
    }
}
fn incoming(r: &mut GameScenarioRequest, times: &[u64]) {
    for m in [
        crate::HypothesisId::CombatIncomingDamageFixtureV1,
        crate::HypothesisId::CombatIncomingDefenseV1,
    ] {
        r.model_assumptions.insert(m.into(), m.description().into());
    }
    r.actors[0].static_stats.base_def = Some(1000);
    r.actors[0].static_stats.def = 1600;
    r.actors[0].static_stats.def_percent = Ratio {
        numerator: 1,
        denominator: 2,
    };
    r.actors[0].static_stats.def_flat = Ratio {
        numerator: 100,
        denominator: 1,
    };
    r.actors[0].static_stats.hp = 10000;
    r.actors[0].initial_hp = 10000;
    r.actors[0].initial_hp_max = 10000;
    for (i, &t) in times.iter().enumerate() {
        r.incoming_damage_events.push(GameActorDamageEvent {
            timestamp_us: t - 4_518_597,
            actor_index: 0,
            delivered_damage: 1000,
        });
        r.incoming_dummy_sources.insert(i, 0);
        r.incoming_damage_basis
            .insert(i, GameIncomingDamageBasis::BeforeDefense);
    }
}
#[test]
fn hathor_cyclone_def_adds_own_base_and_cleans_at_real_finish_or_interruption() {
    for interrupt in [false, true] {
        let mut r = request();
        r.actions.push(skill(55_000_000));
        let end = if interrupt { 56_300_000 } else { 57_140_666 };
        if interrupt {
            r.interrupt_events = vec![GameActorInterruptEvent {
                timestamp_us: end,
                actor_index: 0,
                armor_damage: None,
            }];
        }
        // Incoming delivery precedes an equal-time explicit interruption; recovery is ModePause.
        let times = if interrupt {
            [54_999_999, 55_000_000, end, end + 1]
        } else {
            [54_999_999, 55_000_000, end - 1, end]
        };
        incoming(&mut r, &times);
        let report = run(&r);
        let hits = report["incoming_damage_receipts"].as_array().unwrap();
        for (h, def, damage) in hits
            .iter()
            .zip([1600, 2600, 2600, 1600])
            .zip([403, 293, 293, 403])
            .map(|((h, d), v)| (h, d, v))
        {
            assert_eq!(
                h["defense_mitigation"]["recipient_defense"],
                json!(format!("{}/{}", def, 1))
            );
            assert_eq!(h["actual_hp_loss"], damage);
        }
    }
}
#[test]
fn hathor_cyclone_state_expiry_retains_only_atk_until_pending_stage_finishes() {
    let mut r = request();
    r.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    r.targets[0].is_boss = true;
    r.actions.extend([
        skill(55_000_000),
        skill(61_518_597),
        super::hathor_clock::normal(63_659_263),
    ]);
    incoming(&mut r, &[62_518_597, 63_659_263]);
    let report = run(&r);
    let hits: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["command_index"] == 5)
        .collect();
    assert_eq!(hits.len(), 7);
    for (index, hit) in hits.iter().enumerate() {
        if index < 4 {
            assert_eq!(hit["crit_rate"], json!("1/4"));
            assert_eq!(hit["non_crit_damage"], json!("21041/25"));
        } else {
            assert_eq!(hit["crit_rate"], json!("0/1"));
            assert_eq!(hit["non_crit_damage"], json!("399779/500"));
            assert_eq!(hit["crit_damage"], hit["non_crit_damage"]);
        }
    }
    let damage = report["incoming_damage_receipts"].as_array().unwrap();
    assert_eq!(damage[0]["actual_hp_loss"], 293);
    assert_eq!(damage[1]["actual_hp_loss"], 403);
    // Gaze expiry60 grants one actual Power/A5 after Emergency; Normal has ATK1000 and A5+.07.
    assert_eq!(
        report["hit_receipts"].as_array().unwrap().last().unwrap()["non_crit_damage"],
        json!("477/2")
    );
}

#[test]
fn hathor_cyclone_pending_atk_has_two_second_safety_and_interrupt_cleans_earlier() {
    for interrupt in [None, Some(62_818_606)] {
        let mut r = request();
        r.actions.push(skill(62_518_596));
        if let Some(t) = interrupt {
            r.interrupt_events = vec![GameActorInterruptEvent {
                timestamp_us: t,
                actor_index: 0,
                armor_damage: None,
            }];
        }
        let report = run(&r);
        let ends: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["event_kind"] == "hathor_emergency_end")
            .map(|e| e["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(ends, [interrupt.unwrap_or(64_518_597)]);
        assert_eq!(power(&report)["spent"], json!("1/1"));
        let state_ends: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["event_kind"] == "hathor_emergency_state_end")
            .map(|e| e["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(state_ends, [62_518_597]);
    }
}
#[test]
fn hathor_cyclone_rounded_zero_contacts_still_pay_but_never_award_success_or_pause() {
    let mut r = request();
    r.actors[0].static_stats.base_atk = 1;
    r.actors[0].static_stats.atk = Ratio::one();
    r.actors[0].awakening = ntesim_data::AwakeningSelection::selected(vec![4]).unwrap();
    r.targets[0].defense = 1_000_000_000;
    r.actions.extend([
        skill(55_000_000),
        skill(56_116_684),
        skill(57_550_025),
        super::hathor_clock::normal(59_049_707),
    ]);
    let report = run(&r);
    assert_eq!(power(&report)["spent"], json!("6/1"));
    for h in report["hit_receipts"].as_array().unwrap() {
        assert_eq!(h["applied_hp_loss"], 0);
        assert_eq!(h["crit_rate"], json!("0/1"));
    }
}
#[test]
fn hathor_cyclone_aoe_success_is_one_increment_per_stage_and_total_cr_clamps() {
    let mut r = request();
    r.actors[0].awakening = ntesim_data::AwakeningSelection::selected(vec![4]).unwrap();
    r.actors[0].static_stats.crit_rate = Ratio {
        numerator: 3,
        denominator: 4,
    };
    r.stochastic_crit=Some(ResearchCritPolicy {
        roll_policy:ResearchCritRollPolicy::IndependentPerTargetHitV1,
        hypothesis:"Check that nominal A4 plus base CR saturates sampled probability without extra RNG draws.".into(),
        eligible_effects:std::collections::BTreeMap::from([("character:1025".into(),
            (5..=10).map(|i|format!("GE_Player_Hathor_Skill{i}_Damage")).collect())]),
        actor_sequences:std::collections::BTreeMap::from([("character:1025".into(),13)]),
    });
    for index in 1..3 {
        let mut target = r.targets[0].clone();
        target.target_index = index;
        r.targets.push(target);
    }
    r.actions
        .extend([skill(55_000_000), skill(56_343_016), skill(57_793_023)]);
    let report = run(&r);
    let hits: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| {
            crate::characters::hathor::cyclone::packet(h["damage_effect_id"].as_str().unwrap())
        })
        .collect();
    assert_eq!(hits.len(), 36);
    for (i, h) in hits.iter().enumerate() {
        assert_eq!(
            h["crit_rate"],
            if i < 21 {
                json!("3/4")
            } else if i < 30 {
                json!("1/1")
            } else {
                json!("5/4")
            }
        );
        if i >= 21 {
            assert_eq!(h["critical"], true);
            assert!(h.get("crit_sample").is_none());
        }
    }
    assert_eq!(report["rng"]["streams"]["character:1025"]["draws"], 21);
}
