use super::*;
use serde_json::json;

#[test]
fn skia_spread_reaches_each_living_dummy_and_converge_stays_on_the_primary() {
    for (level, spread, converge) in [(1, 2902, 699), (8, 4974, 1198), (9, 5371, 1294)] {
        for dead_primary in [false, true] {
            let source = format!(
                "options iteration=1 duration=3 crit=fixed;
                skia char lvl=80/80 awakening=0 talent=1,1,{level} qte=1 start_energy=100;
                target hp=100000000 def=0 resist=0;
                target hp=100000000 def=0 resist=0;
                target hp=100000000 def=0 resist=0;
                target hp=100000000 def=0 resist=0;
                active skia; skia burst;"
            );
            let mut req = crate::dsl::compiler::compile_scenario(&source, Default::default())
                .unwrap()
                .request;
            req.actors[0].static_stats.atk = Ratio {
                numerator: 1000,
                denominator: 1,
            };
            req.actors[0].static_stats.damage_bonus = Ratio::zero();
            req.actors[0].static_stats.elemental_bonus = Ratio::zero();
            req.actors[0].static_stats.elemental_bonuses.clear();
            req.targets[1].hp_current = 0;
            if dead_primary {
                req.targets[0].hp_current = 0;
            }
            let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
            assert_eq!(
                report["execution_completed"], true,
                "{} {}",
                report["halt_diagnostic"], report["rejected_action"]
            );
            let hits = report["hit_receipts"].as_array().unwrap();
            let spread_targets: Vec<_> = hits
                .iter()
                .filter(|hit| hit["damage_effect_id"] == "GE_Player_Skia_UltraSkill1_Damage")
                .map(|hit| hit["target_index"].as_u64().unwrap())
                .collect();
            assert_eq!(
                spread_targets,
                if dead_primary {
                    vec![2, 3]
                } else {
                    vec![0, 2, 3]
                }
            );
            for (index, hit) in hits.iter().enumerate() {
                let is_spread = index < spread_targets.len();
                assert_eq!(
                    hit["damage_effect_id"],
                    if is_spread {
                        "GE_Player_Skia_UltraSkill1_Damage"
                    } else {
                        "GE_Player_Skia_UltraSkill2_Damage"
                    }
                );
                if !is_spread {
                    assert_eq!(hit["target_index"], 0);
                }
                assert_eq!(hit["time_us"], 0);
                assert_eq!(
                    hit["non_crit_damage"],
                    json!(format!(
                        "{}/{}",
                        if is_spread { spread } else { converge },
                        1
                    ))
                );
            }
            assert_eq!(
                hits.len(),
                spread_targets.len() + if dead_primary { 0 } else { 3 }
            );
            assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
            assert_eq!(report["final_actor_states"][0]["granted_energy"], 0);
        }
    }
}
