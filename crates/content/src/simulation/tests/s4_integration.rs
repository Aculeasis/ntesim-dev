use super::*;
use serde_json::json;

fn request() -> GameScenarioRequest {
    serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/s4-game-scenario.json"
    )))
    .unwrap()
}

#[test]
fn s4_four_actor_packets_shared_stats_resources_and_replay() {
    let request = request();
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["success"], true, "{}", report["halt_diagnostic"]);
    assert_eq!(report["invariants_passed"], true);
    assert_eq!(report["quality"], "incomplete");
    assert_eq!(report["hits_settled"], 37);
    let hits = report["hit_receipts"].as_array().unwrap();
    // Full owner trains: Sakiri 5+1, Zero Skill4+A1, Iroi Support1+Skill(1+7+1), Mint 2+2+7.
    // Zero additionally owns five Blossom pistils after the Skill handoff.
    for (actor, count) in [(0, 11), (1, 6), (2, 10), (3, 10)] {
        assert_eq!(
            hits.iter()
                .filter(|hit| hit["actor_index"] == actor)
                .count(),
            count
        );
    }
    // Iroi grants .20 * 1166 = 233.2 to Mint, excludes herself, and grants
    // 5% DEF ignore. Ordinary Sakiri Burst separately adds .30 * 1094 = 328.2.
    let mint = hits.iter().find(|hit| hit["actor_index"] == 0).unwrap();
    assert_eq!(mint["non_crit_damage"], json!("1064173/4375"));
    // Sakiri leaves DEF972; Iroi ignore5% leaves923.4, hence Fdef=200/371.
    // (46383/20 + 1166/5 + 1641/5) * .178 * 1.1 * (200/371) * .8.
    let iroi = hits
        .iter()
        .find(|hit| hit["damage_effect_id"] == "GE_Player_Oneiroi_Skill_Gun_Damage")
        .unwrap();
    let expected = ExactFraction::new(
        (2332 * 5 + 1641) * 2571 * 11 * 200 * 4,
        5 * 1000 * 10 * 371 * 5,
    )
    .unwrap();
    let actual: ExactFraction = serde_json::from_value(iroi["non_crit_damage"].clone()).unwrap();
    assert_eq!(actual, expected);

    let actors = report["final_actor_states"].as_array().unwrap();
    assert_eq!(actors.len(), 4);
    for actor in actors {
        assert_eq!(actor["energy_conserved"], true);
    }
    assert_eq!(actors[0]["energy"], 0);
    assert_eq!(actors[0]["spent_energy"], 100);
    assert_eq!(actors[0]["granted_energy"], actors[0]["overflow_energy"]);
    assert!(actors[0]["granted_energy"].as_u64().unwrap() >= 18);
    assert_eq!(actors[0]["fireflies_stacks"], 0);
    assert_eq!(actors[1]["spent_energy"], 100);
    assert_eq!(actors[1]["energy"], actors[1]["granted_energy"]);
    assert_eq!(actors[1]["overflow_energy"], 0);
    for actor in &actors[2..] {
        assert_eq!(actor["energy"], 100);
        assert_eq!(actor["spent_energy"], 0);
        assert_eq!(actor["granted_energy"], actor["overflow_energy"]);
    }
    let switches: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "switch_actor")
        .map(|event| event["time_us"].as_u64().unwrap())
        .collect();
    assert_eq!(switches, [500_000, 2_000_000, 4_700_000]);
    assert_eq!(
        run_game_scenario(&request, &ExecutionLimits::default()).unwrap(),
        report
    );
}

#[test]
fn s4_four_actor_illegal_cast_and_unrecovered_switch_halt() {
    let mut inactive = request();
    let mut cast = inactive.actions[4].clone();
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut cast {
        *timestamp_us = 5_400_000;
    } else {
        unreachable!();
    }
    inactive.actions.insert(7, cast);
    let mut locked = request();
    if let GameActionCommand::InvokeAbility { recovery_us, .. } = &mut locked.actions[4] {
        *recovery_us = 1_500_000;
    } else {
        unreachable!();
    }
    for (request, diagnostic) in [(inactive, "ACTOR_NOT_ACTIVE"), (locked, "ACTOR_LOCKED")] {
        let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["success"], false);
        assert_eq!(report["execution_completed"], false);
        assert_eq!(report["invariants_passed"], true);
        assert!(
            report["halt_diagnostic"]
                .as_str()
                .unwrap()
                .starts_with(diagnostic),
            "expected {diagnostic}, got {}",
            report["halt_diagnostic"]
        );
    }
}
