use super::*;
use serde_json::json;

#[test]
fn bitgame_refinements_use_real_off_field_ensembles_across_continuing_cycles() {
    for (tier, numerator, denominator) in [
        (1, 624, 5),
        (2, 3146, 25),
        (3, 3172, 25),
        (4, 3198, 25),
        (5, 3224, 25),
    ] {
        let source = include_str!("../../../../../examples/bitgame-hootie-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        let (script, mut config) = crate::dsl::inspection::prepare(&source).unwrap();
        for compiled in &mut config.actors {
            let actor = &mut compiled.actor;
            actor.static_stats.base_atk = 1000;
            actor.static_stats.atk = Ratio {
                numerator: 1000,
                denominator: 1,
            };
            actor.static_stats.crit_rate = Ratio::zero();
            actor.static_stats.damage_bonus = Ratio::zero();
            actor.static_stats.elemental_bonus = Ratio::zero();
            actor.static_stats.elemental_bonuses.clear();
            compiled.projection.static_stats = actor.static_stats.clone();
        }
        let result = crate::simulation::PreparedSourceScenario::from_configuration(
            &source,
            config,
            Default::default(),
            &ExecutionLimits::default(),
            &script.options[0].loc,
        )
        .unwrap()
        .run(42)
        .unwrap();
        assert!(
            result.report.success && result.report.execution_completed,
            "{:?}",
            result.diagnostic
        );
        super::assert_native_replay(&result);
        let report = serde_json::to_value(result.report).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        let hits = report["hit_receipts"].as_array().unwrap();
        let ensemble: Vec<_> = hits
            .iter()
            .filter(|hit| hit["damage_effect_id"] == crate::characters::haniel::hootie::ENSEMBLE)
            .collect();
        assert_eq!(ensemble.len(), 8); // Four uncapped two-recipient packets, two per speaker.
        for hit in ensemble {
            assert_eq!(hit["actor_index"], 0);
            assert_eq!(hit["non_crit_damage"], json!("220/1"));
        }
        let mint = hits.iter().find(|hit| hit["actor_index"] == 1).unwrap();
        // .104 * (1000 * (1 + off-field base) + 100 Hootie flat ATK).
        assert_eq!(
            mint["non_crit_damage"],
            json!(format!("{}/{}", numerator, denominator))
        );
        let buffs = crate::simulation::tests::support::final_buffs(&report);
        let stacks = buffs
            .iter()
            .find(|buff| buff["id"] == "buff_fork_BitGame_off_field_stack")
            .unwrap();
        assert_eq!(stacks["stacks"], 2); // Re-entering Haniel cleared cycle one's two stacks.
        let grants: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "bit_game_off_field_stack")
            .collect();
        assert_eq!(grants.len(), 4); // Same-time AoE does not double proc the 2s ICD.
    }
}
