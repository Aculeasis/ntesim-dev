use super::*;
use serde_json::json;

#[test]
fn wrong_gate_five_refinements_use_real_iroi_heals_and_two_half_open_buff_windows() {
    for (tier, gun_damage, normal_num, normal_den, mint_num, mint_den) in [
        (1, 2262, 28652, 25, 3588, 25),
        (2, 2475, 1254, 1, 741, 5),
        (3, 2697, 34162, 25, 3822, 25),
        (4, 2928, 37088, 25, 3939, 25),
        (5, 3168, 40128, 25, 4056, 25),
    ] {
        let source = include_str!("../../../../../examples/wrong-gate-healing-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        let mut req = crate::dsl::compiler::compile_scenario(&source, Default::default())
            .unwrap()
            .request;
        for actor in &mut req.actors {
            actor.static_stats.base_atk = 1000;
            actor.static_stats.atk = Ratio {
                numerator: 1000,
                denominator: 1,
            };
            actor.static_stats.damage_bonus = Ratio::zero();
            actor.static_stats.elemental_bonus = Ratio::zero();
            actor.static_stats.elemental_bonuses.clear();
            actor.static_stats.crit_rate = Ratio::zero();
        }
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        let hits = report["hit_receipts"].as_array().unwrap();
        let gun: Vec<_> = hits
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "GE_Player_Oneiroi_Skill_Gun_Damage")
            .collect();
        assert_eq!(gun.len(), 2);
        // Under native action clock, Skill heal callback fires at 498,799 us,
        // before Gun impacts at 1,698,827 us, so Gun receives the Anima damage bonus:
        // T1 Gun = 1.5 * (1000 + 1000 * .16) * 1.30 = 2262.
        for hit in gun {
            assert_eq!(
                hit["non_crit_damage"],
                json!(format!("{}/{}", gun_damage, 1))
            );
        }
        let normals: Vec<_> = hits
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "GE_Player_Oneiroi_Melee1_Damage")
            .collect();
        assert_eq!(normals.len(), 2);
        // Later owner Normal: .760 * 1160 * 1.30 = 1146.08, without ally .15.
        for hit in normals {
            assert_eq!(
                hit["non_crit_damage"],
                json!(format!("{}/{}", normal_num, normal_den))
            );
        }
        // Mint receives Iroi's separate flat200 plus Door's allied damage bonus.
        let mint: Vec<_> = hits
            .iter()
            .filter(|hit| {
                hit["actor_index"] == 1
                    && hit["damage_effect_id"] == "GE_Player_Mint_Melee1_Damage_New"
            })
            .collect();
        assert_eq!(mint.len(), 4); // First packet of each stage-one input (.104 ATK).
        for (index, hit) in mint.iter().enumerate() {
            let (num, den) = if index % 2 == 0 {
                (mint_num, mint_den)
            } else {
                (104, 1)
            };
            assert_eq!(
                hit["non_crit_damage"],
                json!(format!("{}/{}", num, den)),
                "Mint at {}",
                hit["time_us"]
            );
        }
        let triggers: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "the_wrong_gate_triggered")
            .collect();
        assert_eq!(triggers.len(), 4); // One callback per living team recipient, two per cast.
        for (index, event) in triggers.iter().enumerate() {
            assert_eq!(
                event["time_us"],
                if index < 2 { 498_799 } else { 30_748_799 }
            );
        }
        assert!(
            crate::simulation::tests::support::final_buffs(&report)
                .iter()
                .all(|buff| buff["source"] != "weapon:fork_Door")
        );
    }
}
