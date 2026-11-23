use super::*;
use serde_json::Value;

#[test]
fn castle_five_refinements_heal_two_continuing_wakes_and_a_switched_recipient() {
    for (tier, gross, skill_gross) in [
        (1, 638, 650),
        (2, 650, 661),
        (3, 661, 673),
        (4, 673, 684),
        (5, 684, 696),
    ] {
        let source = include_str!("../../../../../examples/castle-healing-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        let mut req = live_source_request(&source);
        for actor in &mut req.actors {
            actor.static_stats.hp = 20_000;
            actor.static_stats.base_hp = None;
            actor.static_stats.hp_percent = Ratio::zero();
            actor.static_stats.hp_flat = Ratio::zero();
            actor.initial_hp_max = 20_000;
            actor.initial_hp = 1_000;
        }
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        let trace = report["trace"].as_array().unwrap();
        let ticks: Vec<&Value> = trace
            .iter()
            .filter(|event| event["event_kind"] == "edgar_field_heal")
            .collect();
        assert_eq!(ticks.len(), 22); // Each Burst consumes its one newly granted Key.
        for (cycle, expected_ticks) in [(0, 11), (1, 11)] {
            // Tap recovery1.350100 + explicit wait.2 + field-spawn2.750956.
            let start = if cycle == 0 { 4_952_324 } else { 54_952_324 };
            let window: Vec<_> = ticks
                .iter()
                .filter(|tick| {
                    tick["time_us"].as_u64().unwrap() > start
                        && tick["time_us"].as_u64().unwrap() <= start + 12_000_000
                })
                .collect();
            assert_eq!(window.len(), expected_ticks);
            for (index, tick) in window.into_iter().enumerate() {
                assert_eq!(tick["time_us"], start + (index as u64 + 1) * 1_000_000);
                // .011 * 20000 + 350 = 570. Castle is source-owned and expires
                // at Skill+10s even while Wake continues healing another actor.
                let expected = if index < 8 { gross } else { 570 };
                assert_eq!(tick["data"]["gross"], expected, "{tick}");
            }
        }
        let ally_ticks = ticks
            .iter()
            .filter(|tick| tick["active_actor"] == 1)
            .count();
        assert_eq!(ally_ticks, 10);
        assert_eq!(
            report["final_actor_states"][1]["hp"],
            // Second tap selects Mint's lower absolute HP before the field.
            // (.014 * 20000 + 300) * Castle's outgoing bonus, rounded once.
            1_000 + skill_gross + 7 * gross + 3 * 570
        );
        assert_eq!(
            trace
                .iter()
                .filter(|event| event["energy_change"]["source"] == "ability_cost"
                    && event["energy_change"]["spent"] == 100)
                .count(),
            2
        );
    }
}
