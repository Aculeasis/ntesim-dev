use super::*;
use serde_json::{Value, json};

pub(super) fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::chiz_actions::request();
    let model = crate::characters::chiz::clock::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.actors[0].initial_energy = 0;
    req.actions.clear();
    req.relative_duration_us = 15_000_000;
    for index in 1..targets {
        let mut target = req.targets[0].clone();
        target.target_index = index;
        req.targets.push(target);
    }
    for target in &mut req.targets {
        target.hp_current = 100_000_000;
        target.hp_max = 100_000_000;
    }
    req
}

pub(super) fn normal(time: u64) -> GameActionCommand {
    serde_json::from_value(json!({"kind":"invoke_ability","timestamp_us":time,"actor_index":0,
        "ability_id":"GA_Chiichan073_Melee","damage_effect_id":"GE_Player_Chiichan073_Melee1_Damage",
        "array_index":0,"atk_rate":"1/1","scaling_stat":"atk",
        "energy_cost":0,"recovery_us":1,"cooldown_us":0,"hit_delay_us":1})).unwrap()
}

pub(super) fn chain(req: &mut GameScenarioRequest, mut time: u64) -> u64 {
    for recovery in [590_609, 783_916, 1_042_182, 1_099_458, 1_395_803] {
        req.actions.push(normal(time));
        time += recovery;
    }
    time
}

fn run(req: &GameScenarioRequest) -> Value {
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}

#[test]
fn chiz_normal_clock_keeps_twelve_independent_source_contacts_and_delayed_pause_onsets() {
    for targets in [1, 3] {
        let mut req = request(targets);
        assert_eq!(chain(&mut req, 0), 4_911_968);
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let times: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["target_index"] == 0)
            .map(|hit| hit["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(
            times,
            [
                200_711, 789_354, 874_313, 988_815, 1_892_078, 2_652_813, 2_802_813, 2_952_813,
                3_102_813, 3_252_813, 3_849_251, 4_281_702
            ]
        );
        let pauses: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "chiz_normal_hit_pause")
            .map(|event| event["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(pauses, [217_411, 1_908_778, 3_865_951, 4_298_402]);
        req.actions.push(normal(4_911_967));
        assert_eq!(
            run(&req)["rejected_action"]["reason"]["until_us"],
            4_911_968
        );
        *req.actions.last_mut().unwrap() = normal(4_911_968);
        assert_eq!(run(&req)["execution_completed"], true);
    }
}

#[test]
fn chiz_clock_two_continuing_chains_do_not_multiply_channels_by_dummies_or_hit_pulses() {
    for targets in [1, 3] {
        let mut req = request(targets);
        let second = chain(&mut req, 0);
        assert_eq!(chain(&mut req, second), 9_823_936);
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 24 * targets);
        let actor = &report["final_actor_states"][0];
        assert_eq!(actor["energy"], 39); //2*16.3*1.2 =39.12, distinct from Cycle.
        assert_eq!(actor["charge_remainder"], json!("3/25"));
        assert_eq!(actor["cycle"]["current"], 54);
        assert_eq!(actor["cycle"]["carry"], json!("49/250"));
        assert_eq!(actor["cycle"]["conserved"], true);
        assert_eq!(
            actor["fractional_resources"]["chiz.grain"]["current"],
            json!("4500/1")
        );
    }
}

#[test]
fn chiz_miss_and_dead_primary_recipient_keep_recovery_and_first_living_packet_ownership() {
    let mut req = request(3);
    req.targets[0].hp_current = 0;
    chain(&mut req, 0);
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 24);
    assert_eq!(report["final_actor_states"][0]["cycle"]["current"], 27);
    assert_eq!(
        report["final_actor_states"][0]["fractional_resources"]["chiz.grain"]["current"],
        json!("2250/1")
    );
    let mut req = request(1);
    req.targets[0].resistance = Ratio::one();
    let mut time = 0;
    for recovery in [531_209, 783_916, 982_782, 1_099_458, 1_197_803] {
        req.actions.push(normal(time));
        time += recovery;
    }
    assert_eq!(time, 4_595_168);
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 12);
    assert_eq!(report["final_actor_states"][0]["energy"], 0);
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| event["event_kind"] != "chiz_normal_hit_pause")
    );
    req.targets[0].hp_current = 0;
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 0);
}

#[test]
fn chiz_normal4_producer_stops_on_death_without_postmortem_grain_or_pending_pulses() {
    let mut req = request(3);
    chain(&mut req, 0);
    req.actions.truncate(4);
    let model = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.incoming_damage_events
        .push(crate::simulation::GameActorDamageEvent {
            timestamp_us: 2_900_000,
            actor_index: 0,
            delivered_damage: req.actors[0].initial_hp,
        });
    req.relative_duration_us = 3_000_000;
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 21); //Five prior contacts plus two released pulses, three recipients.
    assert_eq!(
        report["final_actor_states"][0]["fractional_resources"]["chiz.grain"]["current"],
        json!("1800/1")
    );
}

#[test]
fn chiz_clock_continuity_uses_full_recovery_plus_two_seconds_and_same_actor_switch() {
    for (time, effect) in [
        (2_590_609, "GE_Player_Chiichan073_Melee2_Damage"),
        (2_590_610, "GE_Player_Chiichan073_Melee1_Damage"),
    ] {
        let mut req = request(1);
        req.actions = vec![
            normal(0),
            GameActionCommand::SwitchActor {
                timestamp_us: 590_609,
                target_actor_index: 0,
                cooldown_us: 0,
            },
            normal(time),
        ];
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hit_receipts"][1]["damage_effect_id"], effect);
    }
}
