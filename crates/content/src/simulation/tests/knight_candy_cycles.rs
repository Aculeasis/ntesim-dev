use super::*;

#[test]
fn knight_candy_native_off_field_criticals_cap_refresh_and_expire_between_skill_trains() {
    for (tier, per_stack) in [(1, 40), (2, 48), (3, 56), (4, 64), (5, 72)] {
        let source = include_str!("../../../../../examples/knight-candy-critical-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        let mut req = crate::dsl::compiler::compile_scenario(&source, Default::default())
            .unwrap()
            .request;
        let stats = &mut req.actors[0].static_stats;
        stats.crit_rate = Ratio::one();
        stats.crit_damage_bonus = Ratio {
            numerator: 1,
            denominator: 2,
        };
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        let hits = report["hit_receipts"].as_array().unwrap();
        let mut index = 0;
        let mut stacks = 0;
        let mut previous: Option<u64> = None;
        let mut resets = 0;
        let mut capped = 0;
        let mut critical_count = 0;
        while index < hits.len() {
            let time = hits[index]["time_us"].as_u64().unwrap();
            if previous.is_none_or(|last| time >= last + 10_000_000) {
                stacks = 0;
                resets += 1;
            }
            let group: Vec<_> = hits[index..]
                .iter()
                .take_while(|hit| hit["time_us"].as_u64() == Some(time))
                .collect();
            // Initial Skill has three native packets per dummy; each tick one.
            assert_eq!(group.len(), if stacks == 0 { 6 } else { 2 });
            if stacks == 10 {
                capped += 1;
            }
            for hit in &group {
                assert_eq!(hit["actor_index"], 0);
                let value = |name: &str| -> ExactFraction {
                    let ratio: Ratio = serde_json::from_value(hit[name].clone()).unwrap();
                    ExactFraction::from_ratio(ratio).unwrap()
                };
                // Every simultaneous packet settles before its callbacks;
                // no recipient sees a stack earned by this timestamp's hits.
                let expected = value("non_crit_damage")
                    .mul(ExactFraction::new(1500 + stacks * per_stack, 1000).unwrap())
                    .unwrap();
                assert_eq!(value("crit_damage"), expected, "tier {tier}, time {time}");
                let critical = hit["critical"].as_bool().unwrap();
                let damage = if critical {
                    expected
                } else {
                    value("non_crit_damage")
                };
                assert_eq!(hit["rounded_damage"], damage.round_u64().unwrap());
            }
            let earned = group.iter().filter(|hit| hit["critical"] == true).count();
            critical_count += earned;
            stacks = (stacks + earned as u128).min(10);
            index += group.len();
            previous = Some(time);
        }
        assert_eq!(resets, 2);
        assert!(capped > 0);
        assert_eq!(hits.len(), 52);
        // Native Adler periodic packets replace the wearer's 100% rate with
        // 50%; seeded noncritical hits cannot create or refresh stacks.
        assert!(critical_count > 20 && critical_count < 52);
        assert_eq!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["event_kind"] == "knight_candy_crit_stack")
                .count(),
            critical_count
        );
        assert_eq!(report["final_buff_state"]["active_actor_index"], 1);
    }
}
