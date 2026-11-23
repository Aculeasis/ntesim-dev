use super::adler_actions::{def_action, make_action, request};
use super::*;
use serde_json::json;

fn burst() -> GameActionCommand {
    make_action(
        0,
        0,
        "GA_Adler_UltraSkill",
        "GE_Player_Adler_UltraSkill_Damage",
        100,
        0,
    )
}

#[test]
fn adler_a5_burst_refunds_flat_energy_once_to_its_living_captured_owner() {
    for (awakening, expected_energy) in [(4, 0), (5, 20), (6, 20)] {
        let mut req = request();
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actors[0].initial_energy = 100;
        req.actions = vec![burst()];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["final_actor_states"][0]["energy"], expected_energy);
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
        let refunds = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["energy_change"]["source"] == "adler_a5")
            .collect::<Vec<_>>();
        assert_eq!(refunds.len(), usize::from(awakening >= 5));
        if let Some(refund) = refunds.first() {
            assert_eq!(refund["energy_change"]["gross"], 20);
            assert_eq!(refund["energy_change"]["source_actor_index"], 0);
            assert_eq!(refund["energy_change"]["recipient_actor_index"], 0);
            assert_eq!(
                report["resolved_model_assumptions"]
                    [crate::characters::adler::skill::A5_ENERGY_HYPOTHESIS_ID.as_str()],
                crate::characters::adler::skill::A5_ENERGY_HYPOTHESIS_ID.description()
            );
        }
    }

    let mut capped = request();
    capped.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(5);
    capped.actors[0].initial_energy = 95;
    capped.actions = vec![make_action(
        0,
        0,
        "GA_Adler_UltraSkill",
        "GE_Player_Adler_UltraSkill_Damage",
        0,
        0,
    )];
    let capped = run_game_scenario(&capped, &ExecutionLimits::default()).unwrap();
    assert_eq!(capped["final_actor_states"][0]["energy"], 100);
    let refund = capped["trace"]
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["energy_change"]["source"] == "adler_a5")
        .unwrap();
    assert_eq!(refund["energy_change"]["gross"], 20);
    assert_eq!(refund["energy_change"]["retained"], 5);
    assert_eq!(refund["energy_change"]["overflow"], 15);

    let mut carried = request();
    carried.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(5);
    carried.actors[0].initial_energy = 95;
    carried.actors[0].gear_procs = vec![crate::simulation::GearProcConfig::ChargeEfficiency {
        multiplier: Ratio {
            numerator: 7,
            denominator: 5,
        },
    }];
    let mut skill = def_action(
        0,
        "GA_Adler_Skill",
        "GE_Player_Adler_Skill2_Damage",
        Ratio::one(),
    );
    if let GameActionCommand::InvokeAbility { energy_cost, .. } = &mut skill {
        *energy_cost = 0;
    }
    let mut second_burst = burst();
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        energy_cost,
        ..
    } = &mut second_burst
    {
        *timestamp_us = 101;
        *energy_cost = 100;
    }
    carried.actions = vec![skill, second_burst];
    let carried = run_game_scenario(&carried, &ExecutionLimits::default()).unwrap();
    assert_eq!(carried["final_actor_states"][0]["energy"], 20);
    assert_eq!(
        carried["final_actor_states"][0]["charge_remainder"],
        json!("3/5")
    );
    let refunds = carried["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["energy_change"]["source"] == "adler_a5")
        .collect::<Vec<_>>();
    assert_eq!(refunds.len(), 1);
    assert_eq!(refunds[0]["energy_change"]["gross"], 20);
}

#[test]
fn adler_a5_refund_requires_admission_and_a_living_captured_owner() {
    let mut multi = request();
    multi.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(5);
    multi.actors[0].initial_energy = 100;
    let mut second_target = multi.targets[0].clone();
    second_target.target_index = 1;
    multi.targets.push(second_target);
    multi.actions = vec![burst()];
    let multi = run_game_scenario(&multi, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        multi["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["energy_change"]["source"] == "adler_a5")
            .count(),
        1
    );

    let mut failed = request();
    failed.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(5);
    failed.actors[0].initial_energy = 100;
    let mut too_expensive = burst();
    if let GameActionCommand::InvokeAbility { energy_cost, .. } = &mut too_expensive {
        *energy_cost = 101;
    }
    failed.actions = vec![too_expensive];
    let failed = run_game_scenario(&failed, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        failed["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| { event["energy_change"]["source"] == "adler_a5" })
            .count(),
        0
    );

    let mut switched = request();
    switched.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(5);
    switched.actors[0].initial_energy = 100;
    let mut ally = switched.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    switched.actors.push(ally);
    switched.actions = vec![
        burst(),
        GameActionCommand::SwitchActor {
            timestamp_us: 0,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    let switched = run_game_scenario(&switched, &ExecutionLimits::default()).unwrap();
    let refund = switched["trace"]
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["energy_change"]["source"] == "adler_a5")
        .unwrap();
    assert_eq!(refund["active_actor"], 1);
    assert_eq!(refund["energy_change"]["recipient_actor_index"], 0);
    assert_eq!(switched["final_actor_states"][1]["granted_energy"], 0);

    let mut defeated = request();
    defeated.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(5);
    defeated.actors[0].initial_energy = 100;
    defeated.actions = vec![burst()];
    defeated.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
        timestamp_us: 0,
        actor_index: 0,
        delivered_damage: defeated.actors[0].initial_hp,
    }];
    defeated.model_assumptions.insert(
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let defeated = run_game_scenario(&defeated, &ExecutionLimits::default()).unwrap();
    assert_eq!(defeated["final_actor_states"][0]["hp"], 0);
    assert_eq!(defeated["final_actor_states"][0]["energy"], 0);
    assert_eq!(
        defeated["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| { event["energy_change"]["source"] == "adler_a5" })
            .count(),
        0
    );
}
