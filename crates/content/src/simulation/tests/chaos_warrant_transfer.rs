use super::chaos_actions::make_action;
use super::*;
use serde_json::{Value, json};

fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::chaos_clock::request(targets);
    let model = crate::HypothesisId::ChaosWarrantTransferV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([4]).unwrap();
    req.actions = vec![make_action(
        0,
        0,
        "GA_Chaos071_Skill",
        "GE_Player_Chaos_Skill1_Damage",
        0,
        0,
    )];
    req.relative_duration_us = 2_000_000;
    req
}
fn run(req: &GameScenarioRequest) -> Value {
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}
fn mark(report: &Value, target: usize) -> Option<&Value> {
    report["final_target_states"][target]["statuses"]
        .as_array()?
        .iter()
        .find(|status| status["id"] == "chaos.warrant")
}
fn transfers(report: &Value) -> Vec<&Value> {
    report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "chaos_warrant_transfer")
        .collect()
}

#[test]
fn chaos_a4_transfers_before_later_aoe_recipient_and_uses_fresh_lifetime() {
    let mut req = request(4);
    req.targets[0].hp_current = 500;
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(transfers(&report).len(), 1);
    assert_eq!(transfers(&report)[0]["time_us"], 298_280);
    assert!(mark(&report, 0).is_none());
    assert_eq!(mark(&report, 1).unwrap()["expires_at_us"], 15_283_733);
    assert_eq!(mark(&report, 2).unwrap()["expires_at_us"], 15_298_280);
    assert!(mark(&report, 3).is_none());
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits[2]["target_index"], 2);
    assert_eq!(hits[2]["non_crit_damage"], json!("2145/4"));
    assert_eq!(hits[3]["non_crit_damage"], json!("825/2"));
    assert_eq!(
        report["final_actor_states"][0]["fractional_resources"]["chaos.crime"]["current"],
        json!("100/1")
    );
    req.relative_duration_us = 15_283_734;
    let report = run(&req);
    assert!(mark(&report, 1).is_none());
    assert!(mark(&report, 2).is_some());
    req.relative_duration_us = 15_298_280;
    assert!(mark(&run(&req), 2).is_none());
}

#[test]
fn chaos_a4_excludes_dead_and_marked_candidates_and_does_not_refresh_sole_existing_mark() {
    let mut req = request(4);
    req.targets[0].hp_current = 500;
    req.targets[2].hp_current = 0;
    let report = run(&req);
    assert!(mark(&report, 3).is_some());
    assert!(mark(&report, 2).is_none());
    let mut req = request(2);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([4, 5]).unwrap();
    req.targets[0].hp_current = 500;
    let report = run(&req);
    assert_eq!(transfers(&report).len(), 1);
    assert!(mark(&report, 0).is_none());
    assert_eq!(mark(&report, 1).unwrap()["expires_at_us"], 15_283_733);
    // Target0 dies first: only target1 remains marked, so its same-group A5 is live.
    assert_eq!(
        report["hit_receipts"][1]["non_crit_damage"],
        json!("2475/4")
    );
}

#[test]
fn chaos_a4_observes_teammate_kills_while_source_is_off_field() {
    let mut req = request(3);
    req.targets[0].hp_current = 2500;
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1042".into();
    ally.awakening = ntesim_data::AwakeningSelection::prefix(0);
    ally.static_stats.atk = Ratio {
        numerator: 100_000,
        denominator: 1,
    };
    ally.static_stats.base_atk = 100_000;
    req.actors.push(ally);
    req.relative_duration_us = 4_000_000;
    req.actions.extend([
        super::reactions::switch(2_000_000, 1),
        make_action(
            2_000_000,
            1,
            "GA_BlackBird_Melee",
            "GE_Player_BlackBird_Melee1_Damage",
            0,
            0,
        ),
    ]);
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(transfers(&report).len(), 1);
    assert!(mark(&report, 0).is_none());
    assert_eq!(mark(&report, 2).unwrap()["expires_at_us"], 17_000_100);
    assert_eq!(
        report["final_actor_states"][0]["fractional_resources"]["chaos.crime"]["current"],
        json!("100/1")
    );
    let model = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
        timestamp_us: 2_100_000,
        actor_index: 0,
        delivered_damage: 100_000,
    }];
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut req.actions[2] {
        *timestamp_us = 3_000_000;
    }
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["final_actor_states"][0]["hp"], 0);
    assert_eq!(mark(&report, 2).unwrap()["expires_at_us"], 18_000_100);
}

#[test]
fn chaos_a4_requires_real_marked_death_and_individual_selection_and_explicit_binding() {
    let mut req = request(4);
    req.targets[2].hp_current = 300;
    let report = run(&req);
    assert!(transfers(&report).is_empty()); // Unmarked target2 dies; target3 stays unmarked.
    assert!(mark(&report, 3).is_none());
    req.targets[0].hp_current = 500;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([1, 2, 3]).unwrap();
    let report = run(&req);
    assert!(transfers(&report).is_empty()); // R3 count is not individual A4.
    assert!(mark(&report, 0).is_none());
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([4]).unwrap();
    req.model_assumptions
        .remove(crate::HypothesisId::ChaosWarrantTransferV1.as_str());
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("warrant_transfer_v1")
    );
}

#[test]
fn chaos_transferred_warrant_retains_source_r6_resistance_modifier() {
    let mut req = request(3);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.targets[0].hp_current = 1;
    let report = run(&req);
    let status = mark(&report, 2).unwrap();
    assert_eq!(status["source_actor_index"], 0);
    assert_eq!(
        status["resistance_reductions"]["ECharacterElementType::CHARACTER_ELEMENT_TYPE_LAKSHANA"],
        json!("1/10")
    );
}

#[test]
fn chaos_warrant_observes_party_reaction_kill_before_shared_target_cleanup() {
    let mut req = request(3);
    for (slot, character, cycle) in [(1, "character:1033", 100), (2, "character:1004", 0)] {
        let mut ally = req.actors[0].clone();
        ally.slot = slot;
        ally.character_id = character.into();
        ally.awakening = ntesim_data::AwakeningSelection::prefix(0);
        ally.initial_cycle = cycle;
        req.actors.push(ally);
    }
    req.actions.extend([
        super::reactions::switch(2_000_000, 1),
        super::reactions::switch(3_000_000, 2),
    ]);
    req.relative_duration_us = 18_980_000;
    let probe = run(&req);
    let before_first_scorch: u64 = probe["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["target_index"] == 0 && hit["time_us"].as_u64().unwrap() < 3_990_000)
        .map(|hit| hit["applied_hp_loss"].as_u64().unwrap())
        .sum();
    req.targets[0].hp_current = before_first_scorch + 1;
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(transfers(&report).len(), 1);
    assert_eq!(transfers(&report)[0]["time_us"], 3_990_000);
    assert_eq!(mark(&report, 2).unwrap()["expires_at_us"], 18_990_000);
    assert!(mark(&report, 0).is_none());
}
