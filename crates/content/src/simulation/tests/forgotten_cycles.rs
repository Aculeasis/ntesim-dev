use super::*;

#[test]
fn forgotten_native_hp_thresholds_and_team_healing_keep_live_atk_through_continuing_cycles() {
    for (tier, boosted_atk) in [(1, 1200), (2, 1240), (3, 1280), (4, 1320), (5, 1360)] {
        let source = include_str!("../../../../../examples/forgotten-hp-cycles.nte")
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
            actor.static_stats.base_hp = Some(1000);
            actor.static_stats.hp = 1000;
            actor.initial_hp_max = 1000;
            actor.initial_hp = 1000;
            compiled.projection.static_stats = actor.static_stats.clone();
        }
        config.actors[0].actor.initial_hp = 501;
        for ((event, _), damage) in config.incoming_damage.iter_mut().zip([1, 1, 501]) {
            event.delivered_damage = damage;
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
            report["halt_diagnostic"]
        );
        let hits: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["actor_index"] == 0)
            .collect();
        assert_eq!(hits.len(), 6);
        // Native Haniel Normal1 = .419 ATK. HP501/1000 qualifies;
        // equality500 and below499 do not. Iroi supplies flat200 team ATK
        // from her base1000, so her .943*1200=1131.6 heal fills this MaxHP.
        // Her second Skill replaces the flat grant rather than stacking it.
        for (hit, atk) in hits.iter().zip([
            boosted_atk,
            1000,
            1000,
            boosted_atk + 200,
            1200,
            boosted_atk + 200,
        ]) {
            assert_eq!(hit["damage_effect_id"], "GE_Player_Haniel_Melee1_Damage");
            let expected = ExactFraction::new(atk * 419, 1000).unwrap();
            assert_eq!(
                hit["non_crit_damage"],
                serde_json::to_value(expected).unwrap(),
                "tier {tier} at {}",
                hit["time_us"]
            );
        }
        let damage = report["incoming_damage_receipts"].as_array().unwrap();
        assert_eq!(
            damage
                .iter()
                .map(|receipt| receipt["hp_after"].as_u64().unwrap())
                .collect::<Vec<_>>(),
            [500, 499, 499]
        );
        assert_eq!(report["final_actor_states"][0]["hp"], 1000);
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 0);
        assert!(
            report["final_actor_states"][0]["granted_energy"]
                .as_u64()
                .unwrap()
                > 10
        );
    }
}
