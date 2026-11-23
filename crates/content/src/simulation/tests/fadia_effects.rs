use super::*;
use crate::simulation::tests::support::formula_damage_total;
use serde_json::json;
pub(super) fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::fadia_clock::request(targets);
    req.actors[0].initial_hp = 10000;
    let model = crate::characters::fadia::effects::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    let model = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.relative_duration_us = 20000000;
    req
}
pub(super) fn skill(time: u64) -> GameActionCommand {
    super::edgar_actions::make_action(
        time,
        0,
        "GA_Fadia_Skill",
        "GE_Player_Fadia_Skill1_Damage",
        0,
        0,
    )
}
fn incoming(req: &mut GameScenarioRequest, time: u64, amount: u64) {
    req.incoming_damage_events.push(GameActorDamageEvent {
        timestamp_us: time,
        actor_index: 0,
        delivered_damage: amount,
    });
}
pub(super) fn burst(time: u64) -> GameActionCommand {
    super::edgar_actions::make_action(
        time,
        0,
        "GA_Fadia_UltraSkill",
        "GE_Player_Fadia_UltraSkill1_Damage",
        120,
        0,
    )
}
#[test]
fn fadia_burst_equality_is_ordinary_and_enhancement_is_captured_before_one_heal() {
    for (hp, a4, healed, damage) in [
        (2456, false, 4106, 28490),
        (2455, false, 5755, 37037),
        (2455, true, 9055, 37037),
    ] {
        let mut req = request(1);
        req.actors[0].initial_energy = 120;
        req.actors[0].initial_hp = hp;
        if a4 {
            req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([4]).unwrap();
        }
        req.actions = vec![burst(0)];
        let r = super::edgar_full::run(&req);
        assert_eq!(r["hits_settled"], 6);
        assert_eq!(
            serde_json::json!(formula_damage_total(&r)),
            serde_json::to_value(ExactFraction::new(damage, 10).unwrap()).unwrap()
        );
        assert_eq!(r["final_actor_states"][0]["hp"], healed);
        let healing: Vec<_> = r["autonomous_events"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| !e["recipient_healing"].is_null())
            .collect();
        assert_eq!(healing.len(), 1);
        assert_eq!(healing[0]["recipient_healing"]["hp_before"], hp);
        assert_eq!(healing[0]["recipient_healing"]["hp_after"], healed);
    }
}
#[test]
fn fadia_lilith_keeps_all_thirteen_packets_and_captured_final_section_past_form_deadline() {
    let mut req = request(1);
    req.actors[0].initial_energy = 120;
    req.actors[0].initial_hp = 2455;
    req.actions = vec![burst(0)];
    for time in [4415921, 5518132, 6173983, 7589082, 9177559, 10049476] {
        req.actions.push(super::edgar_actions::make_action(
            time,
            0,
            "GA_Fadia_Melee",
            "GE_Player_Fadia_Melee1_Damage",
            0,
            0,
        ));
    }
    let r = super::edgar_full::run(&req);
    assert_eq!(r["hits_settled"], 20);
    assert_eq!(
        serde_json::json!(formula_damage_total(&r)),
        json!("63202/5")
    );
    let hits = r["hit_receipts"].as_array().unwrap();
    assert_eq!(hits[18]["time_us"], 9507199);
    assert_eq!(
        hits[18]["damage_effect_id"],
        "GE_Player_Fadia_UltraSkillMelee8_Damage"
    );
    assert_eq!(hits[18]["damage"], json!("14443/10"));
    assert_eq!(
        hits[19]["damage_effect_id"],
        "GE_Player_Fadia_Melee1_Damage"
    );
}
#[test]
fn fadia_skill_notifies_mark_highest_max_hp_and_transfer_bypasses_ordinary_zones() {
    let mut req = request(3);
    req.targets[1].hp_max = 200000000;
    req.targets[2].hp_max = 200000000;
    req.actions = vec![skill(0)];
    incoming(&mut req, 2000000, 300);
    let r = super::edgar_full::run(&req);
    assert_eq!(r["hits_settled"], 10);
    assert_eq!(serde_json::json!(formula_damage_total(&r)), json!("5489/2"));
    let hits = r["hit_receipts"].as_array().unwrap();
    assert_eq!(hits[0]["damage_effect_id"], "GE_Player_Fadia_Skill3_Damage");
    assert_eq!(hits[0]["time_us"], 215663);
    assert_eq!(hits[3]["time_us"], 778126);
    assert_eq!(hits[6]["time_us"], 955868);
    let transfer = &hits[9];
    assert_eq!(transfer["target_index"], 1);
    assert_eq!(transfer["damage"], json!("880/1"));
    assert_eq!(transfer["crit_rate"], json!("0/1"));
    assert!(transfer["command_index"].is_null());
    assert!(transfer["autonomous_event_index"].is_number());
    let actor = &r["final_actor_states"][0];
    assert_eq!(actor["energy"], 4);
    assert_eq!(actor["charge_remainder"], json!("3/5"));
    assert_eq!(actor["cycle"]["current"], 7);
    assert_eq!(actor["cycle"]["carry"], json!("599/1000"));
    let marks: Vec<_> = r["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "fadia_mark_applied")
        .collect();
    assert_eq!(marks.len(), 1);
    assert_eq!(marks[0]["time_us"], 957926);
}
#[test]
fn fadia_a2_expiry_pays_remaining_budget_once_and_source_death_cancels_it() {
    let mut req = request(1);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(2);
    req.actions = vec![skill(0)];
    incoming(&mut req, 2000000, 300);
    let r = super::edgar_full::run(&req);
    let transfers: Vec<_> = r["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_Fadia_ZhouYin_Damage")
        .collect();
    assert_eq!(transfers.len(), 2);
    assert_eq!(transfers[0]["damage"], json!("1800/1"));
    assert_eq!(transfers[1]["damage"], json!("950/1"));
    assert_eq!(transfers[1]["time_us"], 15957926);
    incoming(&mut req, 3000000, 20000);
    let r = super::edgar_full::run(&req);
    assert_eq!(r["hits_settled"], 4);
    assert_eq!(r["final_actor_states"][0]["hp"], 0);
}
#[test]
fn fadia_exact_threshold_crossing_fills_pool_without_ce_and_preserves_fractional_carry() {
    let mut req = request(1);
    req.actors[0].initial_hp = 2457;
    req.actors[0].gear_procs = vec![GearProcConfig::ChargeEfficiency {
        multiplier: Ratio {
            numerator: 3,
            denominator: 2,
        },
    }];
    req.actions = vec![super::edgar_actions::make_action(
        0,
        0,
        "GA_Fadia_Melee",
        "GE_Player_Fadia_Melee1_Damage",
        0,
        0,
    )];
    incoming(&mut req, 2000000, 1);
    incoming(&mut req, 3000000, 1);
    let r = super::edgar_full::run(&req);
    let actor = &r["final_actor_states"][0];
    assert_eq!(actor["energy"], 120);
    assert_eq!(actor["charge_remainder"], json!("121/2000"));
    assert_eq!(actor["overflow_energy"], 0);
    let fills: Vec<_> = r["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "fadia_threshold_fill")
        .collect();
    assert_eq!(fills.len(), 1);
    assert_eq!(fills[0]["energy_change"]["gross"], 116);
}

#[test]
fn fadia_a6_independent_marks_snapshot_hp_and_combat_exit_purges_only_unspent_budget() {
    let mut req = request(12);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([2, 6]).unwrap();
    req.actions = vec![skill(0)];
    incoming(&mut req, 2000000, 100);
    req.combat_transition_events = vec![GameActorCombatTransitionEvent {
        timestamp_us: 3000000,
        actor_index: 0,
        leaving: true,
    }];
    let r = super::edgar_full::run(&req);
    let hits: Vec<_> = r["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_Fadia_ZhouYin_Damage")
        .collect();
    assert_eq!(hits.len(), 20);
    for index in 0..10 {
        assert_eq!(hits[index]["target_index"], index);
        assert_eq!(hits[index]["applied_hp_loss"], 600);
        assert_eq!(hits[index + 10]["target_index"], index);
        assert_eq!(hits[index + 10]["applied_hp_loss"], 2150);
    }
    assert_eq!(r["final_actor_states"][0]["hp_max"], 10000);
    assert_eq!(r["final_actor_states"][0]["hp"], 9900);
    assert_eq!(
        r["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["event_kind"] == "fadia_mark_applied")
            .count(),
        10
    );
}

#[test]
fn fadia_burst_snapshots_branch_before_hp_suit_but_heals_and_hits_after_the_grant() {
    let mut req = request(1);
    req.actors[0].set_id = Some("set:Suit10".into());
    req.relative_duration_us = 5000000;
    req.actors[0].initial_energy = 120;
    req.actors[0].initial_hp = 30000;
    req.actors[0].initial_hp_max = 30000;
    req.actors[0].static_stats.base_hp = Some(30000);
    req.actors[0].static_stats.hp = 30000;
    req.actors[0].gear_procs.push(GearProcConfig::HpSuitMaxHp {
        max_hp_bonus_per_stack: Ratio {
            numerator: 1,
            denominator: 25,
        },
        duration_us: 10000000,
        max_stacks: 10,
    });
    let model = crate::HypothesisId::SetsHpSuitV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.actions = vec![burst(0)];
    let r = super::edgar_full::run(&req);
    assert_eq!(
        serde_json::json!(formula_damage_total(&r)),
        json!("11655/1")
    );
    assert_eq!(r["final_actor_states"][0]["hp_max"], 45000);
    assert_eq!(r["final_actor_states"][0]["hp"], 36750);
}
