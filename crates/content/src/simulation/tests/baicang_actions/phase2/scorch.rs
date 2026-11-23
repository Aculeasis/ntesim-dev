use super::*;

fn scenario() -> GameScenarioRequest {
    let mut req = phase2_request();
    let mut chaos = req.actors[0].clone();
    chaos.slot = 1;
    chaos.character_id = "character:1004".into();
    chaos.initial_cycle = 100;
    chaos.static_stats.cycle_intensity = Ratio {
        numerator: 600,
        denominator: 1,
    };
    chaos.static_stats.crit_damage_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };
    req.actors.push(chaos);
    req.active_actor_index = 1;
    req.relative_duration_us = 22_000_000;
    req.actions = vec![
        GameActionCommand::SwitchActor {
            timestamp_us: 0,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        normal(2_000_000),
        normal(3_000_000),
        normal(4_000_000),
    ];
    req
}

fn ticks(report: &serde_json::Value) -> Vec<&serde_json::Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "ntesim.reaction.scorch")
        .collect()
}

fn refreshes(report: &serde_json::Value) -> usize {
    report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "baicang_scorch_refresh")
        .count()
}

#[test]
fn word_refresh_preserves_original_snapshot_source_phase_and_ordinary_cap() {
    let report = run(&scenario());
    // Third Normal generates Bless4.166255; the switch creates no extra Word.
    // Ordinary .99s phase continues through18.81, not19.016255 after a reset.
    assert_eq!(refreshes(&report), 1);
    let ticks = ticks(&report);
    assert_eq!(ticks.len(), 19);
    for (index, tick) in ticks.iter().enumerate() {
        assert_eq!(tick["time_us"], (index as u64 + 1) * 990_000);
        assert_eq!(
            tick["actor_index"], 1,
            "Chaos keeps its original CI600 snapshot"
        );
        assert_eq!(tick["non_crit_damage"], json!("2700/1"));
        assert_eq!(tick["crit_damage"], json!("4050/1"));
        assert_eq!(tick["switch_origin"]["kind"], "esper_cycle");
    }
    assert_eq!(report["reaction_rng"]["streams"]["scorch:1"]["draws"], 19);
}

#[test]
fn word_refresh_retains_zankou_three_stack_policy_without_tick_recursion() {
    let mut req = scenario();
    let mut zankou = req.actors[0].clone();
    zankou.slot = 2;
    zankou.character_id = "character:1036".into();
    req.actors.push(zankou);
    req.relative_duration_us = 28_000_000;
    req.actions.extend([
        normal(5_300_000),
        normal(7_100_000),
        normal(8_100_000),
        normal(9_100_000),
        normal(10_100_000),
        normal(11_400_000),
    ]);
    let report = run(&req);
    assert_eq!(refreshes(&report), 3);
    assert_eq!(ticks(&report).len(), 26);
    for (index, tick) in ticks(&report).iter().enumerate() {
        // Zankou self CI does not modify the original Baicang capture.
        let expected = if index < 4 {
            2700
        } else if index < 8 {
            5400
        } else {
            8100
        };
        assert_eq!(
            tick["non_crit_damage"],
            json!(format!("{}/{}", expected, 1))
        );
        assert_eq!(tick["actor_index"], 1);
    }
}

#[test]
fn absent_expired_or_locked_passive_scorch_is_not_created() {
    let mut req = scenario();
    req.actors[1].initial_cycle = 0;
    let report = run(&req);
    assert_eq!(refreshes(&report), 0);
    assert!(ticks(&report).is_empty());

    let mut req = scenario();
    req.actions.truncate(1);
    req.actions
        .extend([normal(17_000_000), normal(18_000_000), normal(19_000_000)]);
    let report = run(&req);
    assert_eq!(refreshes(&report), 0);
    assert_eq!(ticks(&report).len(), 15);

    let mut req = scenario();
    req.actors[0].character_level = 20;
    req.actors[0].purchased_qte_level = Some(1);
    req.actors[0].character_max_level = Some(20);
    let report = run(&req);
    assert_eq!(refreshes(&report), 0);
    assert_eq!(ticks(&report).len(), 15);
}
