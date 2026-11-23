use super::*;
use serde_json::Value;

const SOURCE: &str = include_str!("../../../../../examples/jiuyuan-natural-pact-cycles.nte");

fn request() -> GameScenarioRequest {
    let mut req = crate::dsl::compiler::compile_scenario(SOURCE, Default::default())
        .unwrap()
        .request;
    let actor = &mut req.actors[0];
    actor.static_stats.base_atk = 1000;
    actor.static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    actor.static_stats.crit_rate = Ratio::zero();
    actor.static_stats.damage_bonus = Ratio::zero();
    actor.static_stats.elemental_bonus = Ratio::zero();
    actor.static_stats.elemental_bonuses.clear();
    actor.gear_procs.clear();
    req
}

fn attachments(report: &Value) -> Vec<&Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| {
            crate::characters::jiuyuan::pact::is_packet(hit["damage_effect_id"].as_str().unwrap())
        })
        .collect()
}

#[test]
fn jiuyuan_natural_pact_two_off_field_cycles_keep_endpoints_ownership_and_live_normal_talent() {
    for (normal, awakening, periodic) in [(1, vec![], 150), (8, vec![], 257), (8, vec![2], 514)] {
        let mut req = request();
        req.actors[0].talent_levels = Some([normal, 1, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(awakening).unwrap();
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        let hits = attachments(&report);
        assert_eq!(hits.len(), 124); // (30 periodic + 1 natural) * 2 targets * 2 generations.
        let origins: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| {
                h["damage_effect_id"] == "GE_Player_Kuhara_Skill1_Damage" && h["target_index"] == 0
            })
            .map(|h| h["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(origins.len(), 2);
        for origin in &origins {
            // Check actual source-time grid rather than a reset rotation clock.
            let ticks: Vec<_> = hits
                .iter()
                .filter(|h| {
                    h["target_index"] == 0
                        && h["time_us"].as_u64().unwrap() > *origin
                        && h["time_us"].as_u64().unwrap() <= origin + 30_000_000
                })
                .collect();
            assert_eq!(ticks.len(), 31);
            for (index, hit) in ticks.iter().take(30).enumerate() {
                assert_eq!(hit["time_us"], origin + (index as u64 + 1) * 1_000_000);
                assert_eq!(
                    hit["non_crit_damage"],
                    serde_json::json!(format!("{}/{}", periodic, 1))
                );
                assert_eq!(hit["actor_index"], 0);
                let tags = super::support::hit_metadata(&report, hit)["packet"]["semantic_tags"]
                    .as_array()
                    .unwrap();
                assert!(tags.iter().any(|tag| tag == "ntesim.damage.attachment"));
                assert!(!tags.iter().any(|tag| tag == "State.Damage.Dot"));
            }
            assert_eq!(
                ticks[30]["damage_effect_id"],
                "GE_Player_Kuhara_BudEnd_Damage"
            );
            assert_eq!(
                ticks[30]["non_crit_damage"],
                serde_json::json!(format!("{}/{}", if normal == 1 { 1000 } else { 1714 }, 1))
            );
        }
        let replay: GameScenarioRequest =
            serde_json::from_value(serde_json::to_value(&req).unwrap()).unwrap();
        assert_eq!(
            report,
            run_game_scenario(&replay, &ExecutionLimits::default()).unwrap()
        );
    }
}

#[test]
fn jiuyuan_pact_reapplication_is_inert_and_a_dead_target_stops_its_train() {
    let source = SOURCE.replace("wait_until(31)", "wait_until(7)");
    let req = crate::dsl::compiler::compile_scenario(&source, Default::default())
        .unwrap()
        .request;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(attachments(&report).len(), 62);
    let mut req = request();
    req.targets[0].hp_current = 1;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert!(attachments(&report).iter().all(|h| h["target_index"] == 1));
}

#[test]
fn jiuyuan_pact_immune_contacts_keep_the_train_and_source_death_removes_it() {
    let mut req = request();
    for target in &mut req.targets {
        target.resistance = Ratio::one();
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(attachments(&report).len(), 124);
    assert!(
        attachments(&report)
            .iter()
            .all(|hit| hit["rounded_damage"] == 0)
    );

    let first_cycle = SOURCE.split("wait_until(31)").next().unwrap();
    let mut req = crate::dsl::compiler::compile_scenario(first_cycle, Default::default())
        .unwrap()
        .request;
    req.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
        timestamp_us: 5_500_000,
        actor_index: 0,
        delivered_damage: req.actors[0].initial_hp,
    }];
    let hypothesis = crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID;
    req.model_assumptions
        .insert(hypothesis.into(), hypothesis.description().into());
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert!(
        attachments(&report)
            .iter()
            .all(|hit| hit["time_us"].as_u64().unwrap() < 5_500_000)
    );
    assert_eq!(attachments(&report).len(), 10);
    assert!(
        crate::simulation::tests::support::final_buffs(&report)
            .iter()
            .all(|buff| buff["id"] != "jiuyuan.rose_pact")
    );
}
