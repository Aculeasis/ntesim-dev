use super::*;
use serde_json::Value;

const SOURCE: &str = include_str!("../../../../../examples/marching-beyond-time-cycles.nte");

fn fraction(n: u128, d: u128) -> ExactFraction {
    ExactFraction::new(n, d).unwrap()
}

fn exact(value: &Value) -> ExactFraction {
    serde_json::from_value(value.clone()).unwrap()
}

fn request(tier: u32) -> GameScenarioRequest {
    let source = SOURCE.replace("refine=1", &format!("refine={tier}"));
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
        actor.static_stats.crit_damage_bonus = Ratio {
            numerator: 1,
            denominator: 2,
        };
    }
    req
}

#[test]
fn marching_beyond_time_native_support_cycles_consume_three_then_one_stack_without_reset() {
    for tier in 1..=5 {
        let req = request(tier);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        // Public engine Support casts on ally entries, not caller-tagged QTE
        // actions. Hathor enters at 17s and Zero at 20.4s and 63s.
        let support_times: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "support_dispatch")
            .map(|event| event["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(support_times, [17_000_000, 20_400_000, 63_000_000]);
        let rank = u128::from(tier + 3);
        let atk = fraction(1, 1).add(fraction(rank, 25)).unwrap();
        let defense = fraction(100, 200 - 3 * rank);
        let hits = report["hit_receipts"].as_array().unwrap();
        // The three-stack Ultimate is cast at 36.5s and the one-stack Ultimate
        // at 86.25s; each native ten-packet train's first petal lands 2.31608s
        // after the cast, so packets within a train settle at staggered times.
        let ultimates: Vec<_> = hits
            .iter()
            .filter(|hit| {
                hit["actor_index"] == 0
                    && hit["damage_effect_id"]
                        .as_str()
                        .unwrap()
                        .contains("UltraSkill")
            })
            .collect();
        assert_eq!(ultimates.len(), 20);
        for (burst, (time, stacks)) in ultimates.chunks(10).zip([(38_816_080, 3), (88_566_080, 1)])
        {
            assert_eq!(burst.len(), 10);
            assert_eq!(burst[0]["time_us"], time);
            let crit = fraction(3, 2)
                .add(fraction(rank * (3 + stacks), 50))
                .unwrap();
            let mut initial = 0;
            let mut final_pair = 0;
            for hit in burst {
                let effect = hit["damage_effect_id"].as_str().unwrap();
                // Independent talent-one native coefficients. Six distinct
                // initial petals, one Ultra2, one Ultra4 and two Ultra3 packets.
                let base = if effect.starts_with("GE_Player_Jin_UltraSkill1") {
                    initial += 1;
                    170
                } else {
                    match effect {
                        "GE_Player_Jin_UltraSkill2_Damage" => 1303,
                        "GE_Player_Jin_UltraSkill4_Damage" => 443,
                        "GE_Player_Jin_UltraSkill3_Damage" => {
                            final_pair += 1;
                            2616
                        }
                        other => panic!("unexpected Burst packet {other}"),
                    }
                };
                let expected = fraction(base, 1).mul(atk).unwrap().mul(defense).unwrap();
                assert_eq!(
                    exact(&hit["non_crit_damage"]),
                    expected,
                    "T{tier} {time} {effect}"
                );
                assert_eq!(exact(&hit["crit_damage"]), expected.mul(crit).unwrap());
                let rolled = if hit["critical"].as_bool().unwrap() {
                    expected.mul(crit).unwrap()
                } else {
                    expected
                };
                assert_eq!(exact(&hit["damage"]), rolled);
            }
            assert_eq!((initial, final_pair), (6, 2));
        }
        // Ultimate's captured critical damage cannot leak to ordinary packets.
        for hit in hits.iter().filter(|hit| {
            hit["actor_index"] == 0
                && !hit["damage_effect_id"]
                    .as_str()
                    .unwrap()
                    .contains("UltraSkill")
        }) {
            assert_eq!(
                exact(&hit["crit_damage"]),
                exact(&hit["non_crit_damage"]).mul(fraction(3, 2)).unwrap()
            );
        }
        // The first Ultimate's DEF-ignore window is granted at 36.5s and
        // expires at 106.5s. The one-stack Ultimate cannot renew it, so the
        // 108s normal no longer ignores DEF.
        for (time, factor) in [(133_366_666, defense), (136_366_666, fraction(1, 2))] {
            let hit = hits
                .iter()
                .find(|hit| hit["actor_index"] == 0 && hit["time_us"] == time)
                .unwrap();
            assert_eq!(hit["damage_effect_id"], "GE_Player_Jin_Melee1_Damage");
            assert_eq!(
                exact(&hit["non_crit_damage"]),
                fraction(200, 1).mul(atk).unwrap().mul(factor).unwrap(),
                "the second one-stack consumption cannot renew DEF ignore: T{tier} at {time}"
            );
        }
        // World's Tide spends Timepiece, not the conventional energy pool.
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 0);
        assert_eq!(report["final_actor_states"][0]["energy"], 100);
        assert_eq!(report["final_actor_states"][0]["energy_conserved"], true);
        assert!(
            crate::simulation::tests::support::final_buffs(&report)
                .iter()
                .all(|buff| buff["id"] != "marching_beyond_time.def_ignore")
        );
        let replay: GameScenarioRequest =
            serde_json::from_value(serde_json::to_value(&req).unwrap()).unwrap();
        assert_eq!(
            report,
            run_game_scenario(&replay, &ExecutionLimits::default()).unwrap()
        );
    }
}
