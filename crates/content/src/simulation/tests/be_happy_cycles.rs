use super::*;
use serde_json::{Value, json};

const SOURCE: &str = include_str!("../../../../../examples/be-happy-defeat-cycles.nte");

fn request(tier: u32) -> GameScenarioRequest {
    let source = SOURCE.replace("refine=1", &format!("refine={tier}"));
    let mut req = crate::dsl::compiler::compile_scenario(&source, Default::default())
        .unwrap()
        .request;
    // Keep native HP/weapon/awakening projection; isolate each spread at2902.
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.damage_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonuses.clear();
    req
}

fn spreads(report: &Value, time: u64) -> Vec<u64> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| {
            hit["time_us"] == time && hit["damage_effect_id"] == "GE_Player_Skia_UltraSkill1_Damage"
        })
        .map(|hit| {
            assert_eq!(hit["non_crit_damage"], json!("2902/1"));
            hit["target_index"].as_u64().unwrap()
        })
        .collect()
}

#[test]
fn be_happy_continuing_spread_defeats_share_one_cooldown_and_heal_only_the_wearer() {
    // Native HP13332 plus the compiled weapon's 30% = 17331.6 -> 17332.
    // Quarter-HP admission floors to4333. Healing rounds once: MaxHP times
    // 12/14/16/18/20 percent, with no outgoing/incoming healing modifiers.
    for (tier, heal) in [(1, 2080), (2, 2426), (3, 2773), (4, 3120), (5, 3466)] {
        let req = request(tier);
        assert_eq!(req.actors[0].initial_hp_max, 17332);
        assert_eq!(req.actors[0].initial_hp, 4333);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        assert_eq!(spreads(&report, 0), [0, 1, 2, 3]);
        assert_eq!(spreads(&report, 40_000_000), [0, 3]);
        let heals: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "be_happy_heal")
            .collect();
        assert_eq!(
            heals.len(),
            2,
            "the simultaneous first two defeats share one ICD"
        );
        for (event, time, target) in [(heals[0], 0, 1), (heals[1], 40_000_000, 3)] {
            assert_eq!(event["time_us"], time);
            assert_eq!(event["from_actor"], 0);
            assert_eq!(event["data"]["target_index"], target);
            assert_eq!(event["data"]["gross"], heal);
            assert_eq!(event["data"]["applied"], heal);
            assert_eq!(event["data"]["overflow"], 0);
        }
        assert_eq!(report["final_actor_states"][0]["hp"], 4333 + 2 * heal);
        assert_eq!(report["final_actor_states"][0]["hp_max"], 17332);
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 200);
        assert_eq!(report["final_actor_states"][0]["energy_conserved"], true);
        assert!(
            report["final_actor_states"][0]["granted_energy"]
                .as_u64()
                .unwrap()
                >= 100
        );
        let targets = report["final_target_states"].as_array().unwrap();
        for target in &targets[1..4] {
            assert_eq!(target["hp_current"], 0);
        }
        let bites: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "GE_Player_Skia_UltraSkill2_Damage")
            .collect();
        assert_eq!(bites.len(), 6);
        assert!(bites.iter().all(|hit| hit["target_index"] == 0));
        assert!(
            report["resolved_model_assumptions"][crate::weapons::be_happy::HYPOTHESIS_ID.as_str()]
                .is_string()
        );
        let replay: GameScenarioRequest =
            serde_json::from_value(serde_json::to_value(&req).unwrap()).unwrap();
        assert_eq!(
            report,
            run_game_scenario(&replay, &ExecutionLimits::default()).unwrap()
        );
        // A variant replaces one simultaneous defeat with a near-surviving
        // target: 5805 - 2*2902 = 1HP, which must not create a third heal.
        let mut nonfatal = req.clone();
        nonfatal.targets[2].hp_max = 5805;
        nonfatal.targets[2].hp_current = 5805;
        let report = run_game_scenario(&nonfatal, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(spreads(&report, 40_000_000), [0, 2, 3]);
        assert_eq!(report["final_target_states"][2]["hp_current"], 1);
        assert_eq!(report["final_actor_states"][0]["hp"], 4333 + 2 * heal);
        assert_eq!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["event_kind"] == "be_happy_heal")
                .count(),
            2
        );
    }
}
