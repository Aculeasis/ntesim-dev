use super::*;

#[test]
fn twinbirds_native_blackbird_cycles_keep_psyche_and_team_self_critical_zones_separate() {
    for (tier, psyche, team) in [
        (1, 20, 16),
        (2, 25, 20),
        (3, 30, 24),
        (4, 35, 28),
        (5, 40, 32),
    ] {
        let source = include_str!("../../../../../examples/twinbirds-critical-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        let (script, mut config) = crate::dsl::inspection::prepare(&source).unwrap();
        for compiled in &mut config.actors {
            let actor = &mut compiled.actor;
            actor.static_stats.base_atk = 1000;
            actor.static_stats.atk = Ratio {
                numerator: 1500,
                denominator: 1,
            };
            actor.static_stats.crit_rate = Ratio::one();
            actor.static_stats.crit_damage_bonus = Ratio {
                numerator: 1,
                denominator: 2,
            };
            compiled.projection.static_stats = actor.static_stats.clone();
        }
        let source_run = PreparedSourceScenario::from_configuration(
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
            source_run.report.execution_completed,
            "{:?}",
            source_run.diagnostic
        );
        let req = source_run.request;
        let mut control = req.clone();
        control.actors[0]
            .gear_procs
            .retain(|proc| !matches!(proc, GearProcConfig::Twinbirds { .. }));
        let plain = run_game_scenario(&control, &ExecutionLimits::default()).unwrap();
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        assert!(
            report == serde_json::to_value(source_run.report).unwrap(),
            "Twinbirds source/native T{tier}"
        );
        assert_eq!(plain["execution_completed"], true);
        let grants: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "twinbirds_crit_damage")
            .map(|event| event["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(grants.len(), 2);
        assert!(grants[1] > grants[0] + 20_000_000);
        let hits = report["hit_receipts"].as_array().unwrap();
        let references = plain["hit_receipts"].as_array().unwrap();
        assert_eq!(hits.len(), references.len());
        let mut bursts = 0;
        let mut buffed_ally_hits = 0;
        let mut expired_ally_hits = 0;
        for (hit, reference) in hits.iter().zip(references) {
            let value = |name: &str| -> ExactFraction {
                serde_json::from_value(hit[name].clone()).unwrap()
            };
            let time = hit["time_us"].as_u64().unwrap();
            let active = grants.iter().any(|start| {
                support::world_time_at(&report, *start) <= support::world_time_at(&report, time)
                    && support::world_time_at(&report, time)
                        < support::world_time_at(&report, *start) + 15_000_000
            });
            let wearer = hit["actor_index"] == 0;
            let plain_damage: ExactFraction =
                serde_json::from_value(reference["non_crit_damage"].clone()).unwrap();
            let noncrit = if wearer {
                plain_damage
                    .mul(ExactFraction::new(125 + psyche, 125).unwrap())
                    .unwrap()
            } else {
                if active {
                    buffed_ally_hits += 1;
                } else {
                    expired_ally_hits += 1;
                }
                plain_damage
            };
            assert_eq!(value("non_crit_damage"), noncrit);
            let crit_bonus = if active {
                team * if wearer { 3 } else { 1 }
            } else {
                0
            };
            let critical = noncrit
                .mul(ExactFraction::new(150 + crit_bonus, 100).unwrap())
                .unwrap();
            assert_eq!(value("crit_damage"), critical, "T{tier}: {hit}");
            assert_eq!(hit["critical"], true);
            assert_eq!(hit["rounded_damage"], critical.round_u64().unwrap());
            if hit["damage_effect_id"] == "GE_Player_BlackBird_UltraSkill_Damage" {
                // Nine level-one0.467 ATK packets, live base-ATK team20%,
                // additive general/Psyche and separate additional CRIT DMG.
                assert_eq!(
                    critical,
                    ExactFraction::new(467 * 1700 * (125 + psyche) * (150 + team * 3), 10_000_000)
                        .unwrap()
                );
                bursts += 1;
            }
        }
        assert_eq!(bursts, 18);
        assert!(buffed_ally_hits > 0);
        assert_eq!(expired_ally_hits, 4); // Two first-Normal packets per cycle.
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 200);
        let buffs = crate::simulation::tests::support::final_buffs(&report);
        assert_eq!(
            buffs
                .iter()
                .filter(|buff| buff["source"] == "weapon:fork_twinbirds")
                .count(),
            1,
            "only permanent wearer CI survives both windows"
        );
        assert!(
            buffs
                .iter()
                .any(|buff| buff["id"] == "twinbirds.cycle_intensity")
        );
    }
}
