use super::*;
use crate::characters::blackbird::bird::PACKET;
use serde_json::{Value, json};

pub(super) fn request(times: &[u64]) -> GameScenarioRequest {
    let mut req = super::blackbird_actions::request();
    req.relative_duration_us = 20_000_000;
    req.actors[0].initial_energy = 0;
    req.actors[0].initial_cycle = 0;
    for id in [
        crate::HypothesisId::BlackbirdOrdinaryMaliceV1,
        crate::HypothesisId::BlackbirdCoordinatedBirdV1,
    ] {
        req.model_assumptions
            .insert(id.as_str().into(), id.description().into());
    }
    let mut ally = req.actors[0].clone();
    ally.character_id = "character:1004".into();
    ally.slot = 1;
    req.actors.push(ally);
    for target_index in 1..3 {
        let mut target = req.targets[0].clone();
        target.target_index = target_index;
        req.targets.push(target);
    }
    for target in &mut req.targets {
        target.hp_current = 100_000_000;
        target.hp_max = 100_000_000;
    }
    let mut skill = req.actions[0].clone();
    if let GameActionCommand::InvokeAbility {
        ability_id,
        damage_effect_id,
        hit_delay_us,
        ..
    } = &mut skill
    {
        *ability_id = "GA_BlackBird_Skill".into();
        *damage_effect_id = "GE_Player_BlackBird_Skill_Damage".into();
        *hit_delay_us = 0;
    }
    req.actions = vec![skill, super::reactions::switch(500_000, 1)];
    for &time in times {
        req.actions.push(GameActionCommand::InvokeAbility {
            timestamp_us: time,
            actor_index: 1,
            ability_id: "GA_Lacrimosa_Melee".into(),
            array_index: 0,
            damage_effect_id: "GE_Player_Lacrimosa_Melee1_Damage".into(),
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 0,
        });
    }
    req
}

fn birds(report: &Value) -> Vec<&Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["damage_effect_id"] == PACKET)
        .collect()
}
fn run(req: &GameScenarioRequest) -> Value {
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}

#[test]
fn blackbird_bird_has_one_shared_five_count_inclusive_icd_full_aoe_and_real_causes() {
    let req = request(&[
        1_100_000, 2_000_000, 3_000_000, 5_000_000, 7_000_000, 9_000_000, 11_000_000,
    ]);
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = birds(&report);
    assert_eq!(hits.len(), 15);
    for (index, (group, time)) in hits
        .chunks(3)
        .zip([600_000, 3_000_000, 5_000_000, 7_000_000, 9_000_000])
        .enumerate()
    {
        for (target, hit) in group.iter().enumerate() {
            assert_eq!(hit["time_us"], time);
            assert_eq!(hit["target_index"], target);
            assert_eq!(hit["non_crit_damage"], json!("1000/1"));
            assert!(hit["command_index"].is_null());
            assert_eq!(hit["autonomous_event_index"], index);
        }
    }
    for event in report["autonomous_events"].as_array().unwrap() {
        assert_eq!(event["actor_index"], 0);
        assert_eq!(event["active_actor_index"], 1);
        assert_eq!(event["trigger"]["kind"], "coordinated_hit");
        assert_eq!(event["trigger"]["cast_command_index"], 0);
        let parent =
            &report["hit_receipts"][event["trigger"]["hit_index"].as_u64().unwrap() as usize];
        assert_eq!(parent["actor_index"], 1);
        assert_eq!(parent["time_us"], event["time_us"]);
        assert_ne!(parent["applied_hp_loss"], 0);
    }
    // The bird contributes exactly five grants of2 to its source. Peer grants
    // from Lacrimosa are independent and remain present in both executions.
    let mut without = req.clone();
    without
        .model_assumptions
        .remove(crate::HypothesisId::BlackbirdCoordinatedBirdV1.as_str());
    let baseline = run(&without);
    assert_eq!(
        report["final_actor_states"][0]["granted_energy"]
            .as_u64()
            .unwrap()
            - baseline["final_actor_states"][0]["granted_energy"]
                .as_u64()
                .unwrap(),
        10
    );
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        baseline["final_actor_states"][0]["charge_remainder"]
    );
    assert_eq!(report["final_actor_states"][0]["cycle"]["current"], 17);
    assert_eq!(
        report["final_actor_states"][0]["cycle"]["carry"],
        json!("1/10")
    );
}

#[test]
fn blackbird_bird_mark_expiry_and_zero_hp_filter_prevent_invented_requests() {
    let mut req = request(&[15_000_000]);
    assert!(
        birds(&run(&req))
            .iter()
            .all(|hit| hit["time_us"] == 600_000)
    );
    req = request(&[1_100_000]);
    for target in &mut req.targets {
        target.resistance = Ratio::one();
    }
    assert!(birds(&run(&req)).is_empty());
    req = request(&[]);
    // Retreating Blackbird's Skill and the shared Nova tick cannot grant requests.
    let report = run(&req);
    assert_eq!(birds(&report).len(), 3);
    assert!(birds(&report).iter().all(|hit| hit["time_us"] == 600_000));
}

#[test]
fn blackbird_a6_bird_malice_is_once_per_packet_and_stays_separate_from_charge() {
    let mut req = request(&[1_100_000, 3_000_000]);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    let report = run(&req);
    assert_eq!(birds(&report).len(), 6);
    let pool = &report["final_actor_states"][0]["fractional_resources"]["blackbird.malice"];
    assert_eq!(pool["granted"], json!("68/1"));
    assert_eq!(pool["overflow"], json!("8/1"));
    assert_eq!(pool["current"], json!("60/1"));
    assert_eq!(pool["conserved"], true);
}

#[test]
fn blackbird_bird_recast_rearms_once_and_retains_new_cast_causality() {
    let mut req = request(&[
        1_100_000, 3_000_000, 5_000_000, 7_000_000, 9_000_000, 11_000_000,
    ]);
    req.relative_duration_us = 30_000_000;
    req.actions.push(super::reactions::switch(16_000_000, 0));
    let mut skill = req.actions[0].clone();
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut skill {
        *timestamp_us = 16_000_000;
    }
    let cast = req.actions.len();
    req.actions.push(skill);
    req.actions.push(super::reactions::switch(16_500_000, 1));
    let mut ally = req.actions[2].clone();
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut ally {
        *timestamp_us = 17_100_000;
    }
    req.actions.push(ally);
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(birds(&report).len(), 18);
    assert_eq!(
        report["autonomous_events"][5]["trigger"]["cast_command_index"],
        cast
    );
}

#[test]
fn blackbird_bird_ignores_an_ally_packet_delivered_after_that_ally_leaves() {
    let mut req = request(&[1_100_000]);
    if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut req.actions[2] {
        *hit_delay_us = 2_000_000;
    }
    req.actions.push(super::reactions::switch(2_000_000, 0));
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|h| h["actor_index"] == 1 && h["time_us"] == 3_100_000)
    );
    assert_eq!(birds(&report).len(), 3);
    assert!(birds(&report).iter().all(|hit| hit["time_us"] == 600_000));
}

#[test]
fn blackbird_bird_stops_new_requests_when_the_source_dies() {
    let mut req = request(&[1_100_000, 3_000_000]);
    let id = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(id.as_str().into(), id.description().into());
    req.incoming_damage_events
        .push(crate::simulation::GameActorDamageEvent {
            timestamp_us: 2_000_000,
            actor_index: 0,
            delivered_damage: u64::MAX / 2,
        });
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(birds(&report).len(), 3);
    assert_eq!(report["final_actor_states"][0]["hp"], 0);
}

#[test]
fn blackbird_bird_uses_skill_investment_without_the_ordinary_a2_main_multiplier() {
    for (level, expected) in [(1, 1000), (8, 1714), (9, 1851)] {
        let mut req = request(&[1_000_000]);
        req.actors[0].talent_levels = Some([1, level, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(2);
        for hit in birds(&run(&req)) {
            assert_eq!(hit["non_crit_damage"], json!(format!("{}/{}", expected, 1)));
        }
    }
}

#[test]
fn blackbird_bird_accepts_only_one_request_from_an_ally_aoe_packet() {
    let mut req = request(&[1_000_000]);
    let id = crate::HypothesisId::BaicangPowerWordsV1;
    req.model_assumptions
        .insert(id.as_str().into(), id.description().into());
    req.actors[1].character_id = "character:1023".into();
    if let GameActionCommand::InvokeAbility {
        ability_id,
        damage_effect_id,
        ..
    } = &mut req.actions[2]
    {
        *ability_id = "GA_Cang_Melee".into();
        *damage_effect_id = "GE_Player_Cang_Melee1_Damage".into();
    }
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| h["actor_index"] == 1)
            .count(),
        3
    );
    assert_eq!(report["autonomous_events"].as_array().unwrap().len(), 1);
    assert_eq!(birds(&report).len(), 3);
    let index = report["autonomous_events"][0]["trigger"]["hit_index"]
        .as_u64()
        .unwrap() as usize;
    assert_eq!(report["hit_receipts"][index]["target_index"], 0);
}

#[test]
fn blackbird_coordinated_origin_requires_both_actual_parents_and_positive_settlement() {
    let req = request(&[1_000_000]);
    let report = run(&req);
    let event: crate::simulation::GameAutonomousEventReceipt =
        serde_json::from_value(report["autonomous_events"][0].clone()).unwrap();
    let hits: Vec<crate::simulation::GameHitReceipt> =
        serde_json::from_value(report["hit_receipts"].clone()).unwrap();
    let mut origins = crate::simulation::origin::Origins::default();
    assert!(origins.register(&req, event.clone()).is_err());
    assert!(origins.register_from_hit(&req, event.clone(), &[]).is_err());
    let mut wrong = event.clone();
    if let crate::simulation::GameCausalTrigger::CoordinatedHit {
        cast_command_index, ..
    } = &mut wrong.trigger
    {
        *cast_command_index = 2;
    }
    assert!(origins.register_from_hit(&req, wrong, &hits).is_err());
    let mut zero = hits.clone();
    if let crate::simulation::GameCausalTrigger::CoordinatedHit { hit_index, .. } = event.trigger {
        zero[hit_index].applied_hp_loss = 0;
    }
    assert!(
        origins
            .register_from_hit(&req, event.clone(), &zero)
            .is_err()
    );
    assert!(origins.register_from_hit(&req, event, &hits).is_ok());
    assert_eq!(origins.receipts().len(), 1);
}
