use super::*;
use serde_json::{Value, json};

fn request(tier: u32) -> GameScenarioRequest {
    let source = include_str!("../../../../../examples/bopu-support-cycles.nte")
        .replace("refine=1", &format!("refine={tier}"));
    let mut req = crate::dsl::compiler::compile_scenario(&source, Default::default())
        .unwrap()
        .request;
    let stats = &mut req.actors[1].static_stats;
    stats.base_atk = 1000;
    stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    stats.crit_rate = Ratio::zero();
    stats.damage_bonus = Ratio::zero();
    stats.elemental_bonus = Ratio::zero();
    stats.elemental_bonuses.clear();
    req
}

fn support_hits(report: &Value) -> Vec<&Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["ability_id"] == "GA_Sagiri_QTE")
        .collect()
}

#[test]
fn bopu_refinements_follow_the_gear_cooldown_across_three_funded_entries() {
    for (tier, buffed) in [(1, 2360), (2, 2420), (3, 2480), (4, 2540), (5, 2600)] {
        let req = request(tier);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        let hits = support_hits(&report);
        assert_eq!(hits.len(), 6);
        for (index, hit) in hits.into_iter().enumerate() {
            let cycle = index / 2;
            assert_eq!(hit["time_us"], cycle as u64 * 15_000_000 + 100_000);
            assert_eq!(hit["actor_index"], 1);
            assert_eq!(hit["target_index"], index % 2);
            assert_eq!(hit["command_index"], cycle * 3);
            assert_eq!(
                support::hit_metadata(&report, hit)["packet"]["element"],
                "incantation"
            );
            assert_eq!(
                hit["non_crit_damage"],
                json!(format!(
                    "{}/{}",
                    if cycle % 2 == 0 { buffed } else { 2000 },
                    1
                ))
            );
        }
        let buffs: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "bopu_damage")
            .map(|event| event["time_us"].as_u64().unwrap())
            .collect();
        // A paid QTE at 15s falls inside the independent 20s gear ICD.
        assert_eq!(buffs, [0, 30_000_000]);
        // Three packets *10.9, once across both AoE recipients.
        assert_eq!(report["final_actor_states"][1]["energy"], 32);
        assert_eq!(
            report["final_actor_states"][1]["charge_remainder"],
            json!("7/10")
        );
        assert_eq!(report["final_actor_states"][0]["energy"], 19);
        assert_eq!(
            report["final_actor_states"][0]["charge_remainder"],
            json!("31/50")
        );
    }
}

#[test]
fn sakiri_support_keeps_purchased_qte_coordinate_without_a_resonance_level_bonus() {
    for (talent, awakening, damage) in [(1, 0, 2000), (8, 0, 3428), (8, 3, 3428), (10, 0, 3998)] {
        let mut req = request(1);
        req.actors[1].gear_procs.clear();
        req.actors[1].purchased_qte_level = Some(talent);
        req.actors[1].talent_levels = Some([1, talent, 1]);
        req.actors[1].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        for hit in support_hits(&report) {
            assert_eq!(hit["non_crit_damage"], json!(format!("{}/{}", damage, 1)));
        }
    }
}

#[test]
fn sakiri_support_grants_once_from_first_positive_recipient_and_not_after_same_time_defeat() {
    let mut req = request(1);
    req.targets[0].resistance = Ratio::one();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["final_actor_states"][1]["energy"], 32);
    req.targets[1].resistance = Ratio::one();
    let immune = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(immune["final_actor_states"][1]["energy"], 0);
    assert_eq!(
        immune["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "bopu_damage")
            .count(),
        2
    );
    req.targets[0].resistance = Ratio::zero();
    req.actions.truncate(1);
    req.relative_duration_us = 1_000_000;
    let id = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    req.incoming_damage_events
        .push(crate::simulation::GameActorDamageEvent {
            timestamp_us: 100_000,
            actor_index: 1,
            delivered_damage: req.actors[1].initial_hp,
        });
    let dead = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(dead["execution_completed"], true);
    assert!(support_hits(&dead).is_empty());
    assert_eq!(dead["final_actor_states"][1]["energy"], 0);
}

#[test]
fn sakiri_support_fanout_identity_preserves_the_accepted_prefix_before_a_rejected_suffix() {
    let req = request(1);
    let before = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let mut invalid = req.clone();
    invalid.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 30_200_000,
        target_actor_index: 0,
        cooldown_us: 0,
    });
    let after = run_game_scenario(&invalid, &ExecutionLimits::default()).unwrap();
    assert_eq!(after["execution_completed"], false);
    assert_eq!(
        after["rejected_action"]["reason"],
        json!({"kind":"recovery","until_us":30_600_000})
    );
    for field in [
        "final_actor_states",
        "final_target_states",
        "hit_receipts",
        "trace",
        "total_damage_exact",
        "hits_settled",
    ] {
        assert_eq!(after[field], before[field], "{field}");
    }
}
