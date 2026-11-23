use super::*;
use serde_json::json;

#[test]
fn film_three_equipped_break_cycles_keep_owner_intensity_and_additive_broken_bonus() {
    for (tier, intensity, break_n, break_d) in [
        (1, 48, 34829, 25),
        (2, 57, 428757, 300),
        (3, 66, 439566, 300),
        (4, 75, 6005, 4),
        (5, 84, 461184, 300),
    ] {
        let source = include_str!("../../../../../examples/film-break-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        let mut req = live_source_request(&source);
        for actor in &mut req.actors {
            actor.static_stats.base_atk = 1000;
            actor.static_stats.atk = Ratio {
                numerator: 1000,
                denominator: 1,
            };
            actor.static_stats.crit_rate = Ratio::zero();
            actor.static_stats.damage_bonus = Ratio {
                numerator: 1,
                denominator: 2,
            };
            actor.static_stats.elemental_bonus = Ratio::zero();
            actor.static_stats.elemental_bonuses.clear();
        }
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        let breaks: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit.get("break_contribution").is_some())
            .collect();
        assert_eq!(breaks.len(), 6);
        for hit in &breaks {
            // 3603 * (1 + BI/300) * original capacity1/3. Ordinary DMG+.5
            // and the Broken-only weapon damage zone do not multiply Break.
            let owner = hit["actor_index"] == 0;
            assert_eq!(
                hit["break_contribution"]["intensity"],
                json!(format!("{}/{}", if owner { intensity } else { 0 }, 1))
            );
            let expected = if owner {
                ExactFraction::new(break_n, break_d).unwrap()
            } else {
                ExactFraction::new(1201, 1).unwrap()
            };
            assert_eq!(hit["damage"], serde_json::to_value(expected).unwrap());
        }
        let direct: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| {
                hit["damage_effect_id"]
                    .as_str()
                    .unwrap()
                    .starts_with("GE_Player_Edgar_")
            })
            .collect();
        assert_eq!(direct.len(), 18); // Three authored six-packet chains.
        let ordinary: Vec<_> = direct
            .iter()
            .filter(|hit| {
                matches!(
                    req.actions[hit["command_index"].as_u64().unwrap() as usize],
                    GameActionCommand::InvokeAbility { .. }
                )
            })
            .collect();
        assert_eq!(ordinary.len(), 18);
        for cycle in ordinary.as_chunks::<6>().0 {
            for (index, coefficient) in [309, 627, 390, 390, 833, 698].into_iter().enumerate() {
                // Two packets deplete the bar; only later packets see Broken.
                let zone = if index < 2 { 150 } else { 158 + 2 * tier };
                let expected = ExactFraction::new(coefficient * zone as u128, 100).unwrap();
                assert_eq!(
                    cycle[index]["non_crit_damage"],
                    serde_json::to_value(expected).unwrap()
                );
            }
        }
        let entry: Vec<_> = direct
            .iter()
            .filter(|hit| {
                matches!(
                    req.actions[hit["command_index"].as_u64().unwrap() as usize],
                    GameActionCommand::SwitchActor { .. }
                )
            })
            .collect();
        assert!(entry.is_empty());
        let blossoms: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "ntesim.reaction.blossom")
            .collect();
        assert_eq!(blossoms.len(), 5);
        assert!(
            blossoms
                .iter()
                .all(|hit| hit["non_crit_damage"] == json!("9000/1"))
        );
        assert_eq!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["event_kind"] == "target_break_recovered")
                .count(),
            3
        );
        // Three chains supply 60.9 raw charge. Break and the five Blossoms
        // add no energy. One qualified
        // Mint Support adds8.4 to Mint and5.04 to Edgar (peer share3/5).
        assert_eq!(
            report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|hit| hit["damage_effect_id"] == "GE_Player_Mint_QTE1_Damage")
                .count(),
            1
        );
        assert_eq!(report["final_actor_states"][0]["granted_energy"], 65);
        assert_eq!(
            report["final_actor_states"][0]["charge_remainder"],
            json!("47/50")
        );
        assert_eq!(report["final_actor_states"][1]["granted_energy"], 44);
        assert_eq!(
            report["final_actor_states"][1]["charge_remainder"],
            json!("47/50")
        );
    }
}
