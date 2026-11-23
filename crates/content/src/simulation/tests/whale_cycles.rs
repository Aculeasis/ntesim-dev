use super::*;
use serde_json::Value;

const SOURCE: &str = include_str!("../../../../../examples/whale-break-defeat-cycles.nte");

fn fraction(n: u128, d: u128) -> ExactFraction {
    ExactFraction::new(n, d).unwrap()
}
fn exact(value: &Value) -> ExactFraction {
    serde_json::from_value(value.clone()).unwrap()
}

fn request(tier: u32) -> GameScenarioRequest {
    let source = SOURCE.replace("refine=1", &format!("refine={tier}"));
    let executed = PreparedSourceScenario::new(&source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap();
    assert!(
        executed.report.success && executed.report.execution_completed,
        "{:?}",
        executed.diagnostic
    );
    super::assert_native_replay(&executed);
    let mut req = executed.request;
    // Deliberately different static/base pools distinguish +base*p from
    // multiplying already compiled ATK by (1+p).
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1500,
        denominator: 1,
    };
    req.actors[0].static_stats.damage_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonuses.clear();
    req
}

#[test]
fn whale_native_broken_defeat_cycles_preserve_zones_and_exact_thirty_second_heal_icd() {
    for tier in 1..=5 {
        let req = request(tier);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        let ratio = fraction(u128::from(tier + 5), 50); //12/14/16/18/20 percent.
        let atk = fraction(1500, 1)
            .add(fraction(1000, 1).mul(ratio).unwrap())
            .unwrap();
        let hits = report["hit_receipts"].as_array().unwrap();
        for time in [2_216_746, 6_516_245, 32_216_746, 36_732_420] {
            let hit = hits
                .iter()
                .find(|hit| {
                    hit["time_us"] == time
                        && hit["damage_effect_id"] == "GE_Player_Hathor_Melee1_Damage"
                })
                .unwrap();
            let bonus = if time % 30_000_000 == 2_216_746 {
                fraction(1, 1).add(ratio).unwrap()
            } else {
                fraction(1, 1)
            };
            let expected = fraction(243, 1000).mul(atk).unwrap().mul(bonus).unwrap();
            assert_eq!(
                exact(&hit["non_crit_damage"]),
                expected,
                "T{tier} at {time}"
            );
        }
        let breaks: Vec<_> = hits
            .iter()
            .filter(|hit| hit["break_contribution"].is_object())
            .collect();
        // Every living authored gauge reaches Broken once per period: four
        // initial recipients, two survivors in the next generation. Native
        // base3603 * original capacity1 /3 =1201; ordinary gear zones cannot
        // multiply this independent Break HP contribution.
        assert_eq!(breaks.len(), 6);
        assert_eq!(
            breaks
                .iter()
                .map(|hit| (
                    hit["time_us"].as_u64().unwrap(),
                    hit["target_index"].as_u64().unwrap()
                ))
                .collect::<Vec<_>>(),
            [
                (516_245, 0),
                (516_245, 1),
                (516_245, 2),
                (516_245, 3),
                (30_732_420, 0),
                (30_732_420, 3)
            ]
        );
        for hit in breaks {
            assert_eq!(exact(&hit["non_crit_damage"]), fraction(1201, 1));
            assert_eq!(
                exact(&hit["break_contribution"]["base_break"]),
                fraction(3603, 1)
            );
            assert_eq!(hit["actor_index"], 0);
        }
        let heals: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "whale_heal")
            .collect();
        assert_eq!(
            heals.len(),
            2,
            "simultaneous Broken defeats share one wearer ICD"
        );
        for (event, time, target) in [(heals[0], 982_202, 1), (heals[1], 30_982_202, 3)] {
            assert_eq!(event["time_us"], time);
            assert_eq!(event["from_actor"], 0);
            assert_eq!(event["data"]["target_index"], target);
            assert_eq!(event["data"]["gross"], 4799);
            assert_eq!(event["data"]["applied"], 4799);
            assert_eq!(event["data"]["overflow"], 0);
        }
        assert_eq!(req.actors[0].initial_hp_max, 15998);
        assert_eq!(req.actors[0].initial_hp, 1599);
        assert_eq!(report["final_actor_states"][0]["hp"], 11197); //1599+2*round(15998*.30).
        assert_eq!(report["final_actor_states"][0]["energy_conserved"], true);
        assert!(
            report["resolved_model_assumptions"][crate::weapons::whale::HYPOTHESIS_ID.as_str()]
                .is_string()
        );
        let replay: GameScenarioRequest =
            serde_json::from_value(serde_json::to_value(&req).unwrap()).unwrap();
        assert_eq!(
            report,
            run_game_scenario(&replay, &ExecutionLimits::default()).unwrap()
        );

        // Same native actions and ordinary defeats without authored gauges
        // grant neither Broken damage nor healing.
        let mut unbroken = req.clone();
        for target in &mut unbroken.targets {
            target.break_profile = None;
        }
        let plain = run_game_scenario(&unbroken, &ExecutionLimits::default()).unwrap();
        assert_eq!(plain["execution_completed"], true);
        assert_eq!(plain["final_actor_states"][0]["hp"], 1599);
        assert_eq!(plain["final_target_states"][1]["hp_current"], 0);
        assert!(
            plain["trace"]
                .as_array()
                .unwrap()
                .iter()
                .all(|event| event["event_kind"] != "whale_heal")
        );
        for hit in plain["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "GE_Player_Hathor_Melee1_Damage")
        {
            assert_eq!(
                exact(&hit["non_crit_damage"]),
                fraction(243, 1000).mul(atk).unwrap()
            );
        }
    }
}
