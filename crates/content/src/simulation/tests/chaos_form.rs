use super::chaos_actions::make_action;
use super::*;
use serde_json::{Value, json};

pub(super) fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::chaos_clock::request(targets);
    let model = crate::HypothesisId::ChaosPrimaryFormV1;
    req.model_assumptions
        .insert(model.as_str().into(), model.description().into());
    req.actors[0].initial_energy = 100;
    req.relative_duration_us = 20_000_000;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Chaos071_UltraSkill",
        "GE_Player_Chaos_UltraSkill1_Damage",
        100,
        0,
    )];
    req
}

fn normal(time: u64) -> GameActionCommand {
    make_action(
        time,
        0,
        "GA_Chaos071_Melee",
        "GE_Player_Chaos_Melee1_Damage",
        0,
        0,
    )
}

fn run(req: &GameScenarioRequest) -> Value {
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}
fn crime(report: &Value) -> &Value {
    &report["final_actor_states"][0]["fractional_resources"]["chaos.crime"]
}
fn times(report: &Value, kind: &str) -> Vec<u64> {
    report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == kind)
        .map(|event| event["time_us"].as_u64().unwrap())
        .collect()
}

#[test]
fn chaos_primary_form_has_sixty_world_ticks_and_final_endpoint_precedes_form_cleanup() {
    let req = request(1);
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(times(&report, "chaos_form_entry"), [2_718_888]);
    assert_eq!(times(&report, "chaos_form_end"), [16_802_493]);
    let ticks = times(&report, "chaos_auto_crime");
    assert_eq!(ticks.len(), 60);
    assert_eq!((ticks[0], ticks[59]), (5_002_493, 16_802_493));
    assert_eq!(crime(&report)["granted"], json!("1800/1"));
    assert_eq!(crime(&report)["current"], json!("1000/1"));
    assert_eq!(crime(&report)["overflow"], json!("800/1"));
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
    for time in [16_802_492, 16_802_493] {
        let mut req = req.clone();
        req.actions.push(normal(time));
        let report = run(&req);
        let hit = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|hit| hit["command_index"] == 1)
            .unwrap();
        assert_eq!(
            hit["damage_effect_id"],
            if time == 16_802_492 {
                "GE_Player_Chaos_Melee1_Damage_Ultra"
            } else {
                "GE_Player_Chaos_Melee1_Damage"
            }
        );
    }
}

#[test]
fn chaos_transformed_chain_swaps_stage2_rows_and_preserves_all_four_resource_channels() {
    for targets in [1, 3] {
        let mut req = request(targets);
        let mut time = 6_000_000;
        for recovery in [518_862, 1_174_976, 545_853, 2_018_381, 1_090_035] {
            req.actions.push(normal(time));
            time += recovery;
        }
        assert_eq!(time, 11_348_107);
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 17 * targets);
        let stage2: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["command_index"] == 2 && hit["target_index"] == 0)
            .collect();
        assert_eq!(
            stage2[0]["damage_effect_id"],
            "GE_Player_Chaos_Melee2_Damage_Ultra"
        );
        assert_eq!(stage2[0]["non_crit_damage"], json!("507/2"));
        assert_eq!(stage2[1]["non_crit_damage"], json!("1963/20"));
        let actor = &report["final_actor_states"][0];
        assert_eq!(actor["energy"], 18);
        assert_eq!(actor["charge_remainder"], json!("101/500"));
        assert_eq!(actor["cycle"]["current"], 30);
        assert_eq!(actor["cycle"]["carry"], json!("299/1000"));
        assert_eq!(crime(&report)["granted"], json!("2182/1"));
        assert_eq!(crime(&report)["overflow"], json!("1182/1"));
        assert_eq!(crime(&report)["conserved"], true);
        let last = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|hit| {
                hit["command_index"] == 5
                    && hit["target_index"] == 0
                    && hit["time_us"] == 11_010_191
            })
            .unwrap();
        assert_eq!(
            last["damage_effect_id"],
            "GE_Player_Chaos_Melee5_1_Damage_Ultra"
        );
    }
}

fn companion(req: &mut GameScenarioRequest) {
    let mut actor = sample_s1_request().actors.remove(0);
    actor.slot = 1;
    actor.weapon_id.clear();
    actor.gear_procs.clear();
    actor.set_id = None;
    actor.initial_cycle = 0;
    req.actors.push(actor);
}
fn switch(time: u64, actor: usize) -> GameActionCommand {
    GameActionCommand::SwitchActor {
        timestamp_us: time,
        target_actor_index: actor,
        cooldown_us: 0,
    }
}

#[test]
fn chaos_off_field_form_expires_without_banking_or_restarting_auto_tick_phase() {
    let mut req = request(1);
    companion(&mut req);
    req.actions
        .extend([switch(5_000_000, 1), switch(10_000_000, 0)]);
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let ticks = times(&report, "chaos_auto_crime");
    assert_eq!(ticks.len(), 35); // All35 occur after returning: the cinematic retained the initial phase.
    assert_eq!(ticks[10], 12_002_493);
    assert_eq!(ticks[11], 12_202_493);
    assert_eq!(crime(&report)["granted"], json!("1050/1"));
    assert_eq!(times(&report, "chaos_form_end"), [16_802_493]);
    req.actions.truncate(2);
    let report = run(&req);
    assert_eq!(crime(&report)["current"], json!("0/1"));
    assert_eq!(times(&report, "chaos_form_end"), [16_802_493]);
}

#[test]
fn chaos_burst_recovery_switch_lock_and_immunity_have_three_distinct_boundaries() {
    let mut req = request(1);
    companion(&mut req);
    req.actions.push(switch(4_944_577, 1));
    assert_eq!(
        run(&req)["rejected_action"]["reason"]["until_us"],
        4_944_578
    );
    req.actions[1] = switch(4_944_578, 1);
    assert_eq!(run(&req)["execution_completed"], true);
    req.actions[1] = normal(4_915_030);
    assert_eq!(
        run(&req)["rejected_action"]["reason"]["until_us"],
        4_915_031
    );
    req.actions[1] = normal(4_915_031);
    assert_eq!(run(&req)["execution_completed"], true);
    req.actions.truncate(1);
    let id = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(id.as_str().into(), id.description().into());
    // Equal-time due damage sees the finite immunity; positive future World
    // delay is retained until cinematic release, after that immunity ends.
    req.incoming_damage_events = [0, 1]
        .map(|timestamp_us| GameActorDamageEvent {
            timestamp_us,
            actor_index: 0,
            delivered_damage: 100,
        })
        .to_vec();
    let report = run(&req);
    assert_eq!(report["incoming_damage_receipts"][0]["actual_hp_loss"], 0);
    assert_eq!(report["incoming_damage_receipts"][1]["actual_hp_loss"], 100);
}

#[test]
fn chaos_a6_fills_once_at_commit_and_failed_burst_cannot_fill_or_enter_form() {
    let mut req = request(1);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([6]).unwrap();
    req.targets[0].resistance = Ratio::one();
    let report = run(&req);
    assert_eq!(crime(&report)["granted"], json!("2800/1"));
    assert_eq!(crime(&report)["overflow"], json!("1800/1"));
    req.actors[0].initial_energy = 99;
    let report = run(&req);
    assert_eq!(report["execution_completed"], false);
    assert_eq!(crime(&report)["granted"], json!("0/1"));
    assert!(times(&report, "chaos_form_entry").is_empty());
    req.actors[0].energy_capacity = 200;
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("capacity")
    );
}
