use super::daffodil_clock::{burst, normal, run};
use super::*;
use crate::characters::daffodil::phantom::{APPEAR_SUCCESS, ENTRY1, ENTRY3_1, MODEL, QTE};
use serde_json::json;

fn switch(time: u64, actor: usize) -> GameActionCommand {
    GameActionCommand::SwitchActor {
        timestamp_us: time,
        target_actor_index: actor,
        cooldown_us: 0,
    }
}

fn request(targets: usize, partner_cycle: u64) -> GameScenarioRequest {
    let mut req = super::daffodil_secondary::request(targets);
    req.relative_duration_us = 10_000_000;
    req.model_assumptions
        .insert(MODEL.into(), MODEL.description().into());
    req.actors[0].initial_energy = 120;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    let mut partner = req.actors[0].clone();
    partner.slot = req.actors.len();
    partner.character_id = "character:1042".into();
    partner.initial_energy = 0;
    partner.initial_cycle = partner_cycle;
    partner.awakening = Default::default();
    req.actors.push(partner);
    req.active_actor_index = 0;
    req.actions = vec![burst(0), switch(5_100_000, 1), switch(5_200_000, 0)];
    req
}

fn qualified_qte_request(targets: usize) -> GameScenarioRequest {
    let mut req = request(targets, 0);
    req.relative_duration_us = 20_000_000;
    let mut partner = req.actors[0].clone();
    partner.slot = req.actors.len();
    partner.character_id = "character:1039".into();
    partner.initial_energy = 0;
    partner.initial_cycle = 100;
    partner.awakening = Default::default();
    req.actors.push(partner);
    req.active_actor_index = 1;
    req.actions = vec![
        switch(0, 0),
        burst(1_300_000),
        switch(6_400_000, 2),
        switch(6_500_000, 0),
    ];
    req
}

fn effects(report: &serde_json::Value, command: u64, effect: &str) -> Vec<serde_json::Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["command_index"] == command && hit["damage_effect_id"] == effect)
        .cloned()
        .collect()
}

#[test]
fn phantom_step_uses_the_first_living_target_and_live_parry_for_one_finisher_reward() {
    let mut req = request(2, 0);
    req.targets[0].hp_current = 0;
    req.parry_events = vec![GameActorParryEvent::At {
        timestamp_us: 5_300_000,
        actor_index: 0,
    }];
    let report = run(&req);
    let entry = effects(&report, 2, ENTRY1);
    let finisher = effects(&report, 2, APPEAR_SUCCESS);
    assert_eq!(entry.len(), 1);
    assert_eq!(entry[0]["target_index"], 1);
    assert_eq!(entry[0]["time_us"], 5_453_952);
    assert_eq!(entry[0]["non_crit_damage"], json!("22173/20"));
    assert_eq!(finisher.len(), 1);
    assert_eq!(finisher[0]["target_index"], 1);
    assert_eq!(finisher[0]["time_us"], 6_657_010);
    assert_eq!(report["final_actor_states"][0]["energy"], 120);
    assert_eq!(
        report["final_actor_states"][0]["resources"]["daffodil.phantom_step_charges"]["current"],
        1
    );
}

#[test]
fn phantom_parry_cannot_retarget_after_its_captured_dummy_dies() {
    let mut req = request(2, 0);
    req.targets[0].hp_current = 7_600;
    req.parry_events = vec![GameActorParryEvent::At {
        timestamp_us: 5_500_000,
        actor_index: 0,
    }];
    let report = run(&req);
    assert_eq!(effects(&report, 2, APPEAR_SUCCESS).len(), 0);
    assert_eq!(report["final_target_states"][0]["hp_current"], 0);
    assert!(report["final_actor_states"][0]["energy"].as_u64().unwrap() < 120);
    assert_eq!(
        report["final_actor_states"][0]["resources"]["daffodil.phantom_step_charges"]["current"],
        1
    );
}

#[test]
fn awakening_two_admits_only_one_failed_counter_finisher_and_fills_energy() {
    let mut req = request(1, 0);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(vec![2]).unwrap();
    let report = run(&req);
    assert_eq!(effects(&report, 2, ENTRY3_1).len(), 1);
    assert_eq!(effects(&report, 2, APPEAR_SUCCESS).len(), 1);
    assert_eq!(report["final_actor_states"][0]["energy"], 120);
    assert_eq!(
        report["final_actor_states"][0]["resources"]["daffodil.phantom_step_charges"]["current"],
        1
    );
}

#[test]
fn qualified_support_parry_spends_only_on_an_eligible_pre_marker_callback() {
    let mut no_parry = qualified_qte_request(2);
    no_parry.actions.push(normal(9_644_677));
    let report = run(&no_parry);
    assert_eq!(effects(&report, 3, APPEAR_SUCCESS).len(), 0);
    assert_eq!(effects(&report, 3, QTE)[0]["time_us"], 7_245_033);
    assert_eq!(effects(&report, 3, ENTRY1)[0]["time_us"], 8_253_602);
    assert_eq!(
        effects(&report, 4, "GE_Player_Daffodill_Melee1_Damage")[0]["time_us"],
        9_829_985
    );
    assert_eq!(
        report["final_actor_states"][0]["resources"]["daffodil.phantom_step_charges"]["current"],
        1
    );

    let mut timely = qualified_qte_request(2);
    timely.parry_events = vec![GameActorParryEvent::At {
        timestamp_us: 6_600_000,
        actor_index: 0,
    }];
    let report = run(&timely);
    assert_eq!(effects(&report, 3, QTE).len(), 2);
    assert_eq!(effects(&report, 3, ENTRY1).len(), 2);
    assert_eq!(effects(&report, 3, ENTRY1)[0]["time_us"], 8_421_212);
    let finisher = effects(&report, 3, APPEAR_SUCCESS);
    assert_eq!(finisher.len(), 2);
    assert_eq!(finisher[0]["time_us"], 8_167_260);
    assert_eq!(finisher[0]["non_crit_damage"], json!("97679/20"));
    assert_eq!(
        super::support::hit_metadata(&report, &finisher[0])["packet"]["native_damage_source_category"],
        "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"
    );
    assert_eq!(
        report["final_actor_states"][0]["resources"]["daffodil.phantom_step_charges"]["current"],
        0
    );

    let mut exact_ready = timely.clone();
    exact_ready.actions.push(normal(9_812_287));
    let report = run(&exact_ready);
    assert_eq!(effects(&report, 3, APPEAR_SUCCESS).len(), 2);
    assert_eq!(
        effects(&report, 4, "GE_Player_Daffodill_Melee1_Damage")[0]["time_us"],
        9_997_595
    );

    let mut late = qualified_qte_request(2);
    late.parry_events = vec![GameActorParryEvent::At {
        timestamp_us: 10_000_000,
        actor_index: 0,
    }];
    let report = run(&late);
    assert_eq!(effects(&report, 3, APPEAR_SUCCESS).len(), 0);
    assert_eq!(
        report["final_actor_states"][0]["resources"]["daffodil.phantom_step_charges"]["current"],
        1
    );

    let mut cancelled = qualified_qte_request(2);
    cancelled.interrupt_events.push(GameActorInterruptEvent {
        timestamp_us: 6_600_000,
        actor_index: 0,
        armor_damage: None,
    });
    cancelled.parry_events = vec![GameActorParryEvent::At {
        timestamp_us: 6_700_000,
        actor_index: 0,
    }];
    let report = run(&cancelled);
    assert_eq!(effects(&report, 3, QTE).len(), 0);
    assert_eq!(effects(&report, 3, APPEAR_SUCCESS).len(), 0);
    assert_eq!(effects(&report, 3, ENTRY1).len(), 0);
    assert_eq!(
        report["final_actor_states"][0]["resources"]["daffodil.phantom_step_charges"]["current"],
        2
    );
}

#[test]
fn burst_entry_charges_expire_on_real_time_even_when_unused() {
    let mut req = request(1, 0);
    req.relative_duration_us = 40_000_000;
    req.actions.truncate(1);
    let report = run(&req);
    assert_eq!(
        report["final_actor_states"][0]["resources"]["daffodil.phantom_step_charges"]["current"],
        0
    );
    assert_eq!(effects(&report, 0, ENTRY1).len(), 0);
}
