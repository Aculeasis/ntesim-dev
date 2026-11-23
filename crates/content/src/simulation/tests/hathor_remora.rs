use super::*;
use serde_json::{Value, json};

fn fixture(times: &[u64], element: &str) -> GameScenarioRequest {
    let mut req = super::stain::fixture(times, element);
    req.actors[0].character_id = "character:1025".into();
    req.actors[0].character_level = 80;
    req.actors[0].character_max_level = Some(80);
    req.actors[1].character_id = "character:1052".into();
    req.actions.remove(1);
    req.action_metadata = std::mem::take(&mut req.action_metadata)
        .into_iter()
        .map(|(index, metadata)| (index - 1, metadata))
        .collect();
    for command in &mut req.actions[1..] {
        if let GameActionCommand::InvokeAbility { actor_index, .. } = command {
            *actor_index = 1;
        }
    }
    req
}
fn run(req: &GameScenarioRequest) -> Value {
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}

fn probe_hits(report: &Value) -> Vec<&Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "caller-effect")
        .collect()
}

#[test]
fn hathor_b2_remora_boosts_allied_ordinary_crit_until_exact_twelve_second_expiry() {
    for element in ["COSMOS", "LAKSHANA", "NATURE", "PSYCHICALLY"] {
        let req = fixture(&[5_000_000, 11_999_999, 12_000_000], element);
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let hits = probe_hits(&report);
        assert_eq!(hits.len(), 3);
        for (hit, numerator, denominator) in [(&hits[0], 3, 5), (&hits[1], 3, 5), (&hits[2], 1, 2)]
        {
            assert_eq!(hit["actor_index"], 1);
            assert_eq!(
                hit["crit_rate"],
                json!(format!("{}/{}", numerator, denominator))
            );
            assert_eq!(hit["non_crit_damage"], json!("1000/1"));
        }
        assert_eq!(
            report["resolved_model_assumptions"]["hathor.b2_remora_v1"],
            crate::HypothesisId::HathorB2RemoraV1.description()
        );
    }
}

#[test]
fn hathor_remora_fixed_critical_override_keeps_precedence() {
    let mut req = fixture(&[2_000_000], "COSMOS");
    let model = crate::HypothesisId::DamageFixedCritRateAsProbabilityV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.action_metadata.get_mut(&1).unwrap().fixed_crit_rate =
        Some(crate::simulation::ResearchFixedCritRate::Probability {
            rate: Ratio {
                numerator: 1,
                denominator: 4,
            },
        });
    let report = run(&req);
    assert_eq!(probe_hits(&report)[0]["crit_rate"], json!("1/4"));
}

#[test]
fn hathor_remora_breakthrough_gate_and_captured_status_survive_source_defeat() {
    let mut req = fixture(&[6_000_000], "COSMOS");
    req.actors[0].character_level = 20;
    req.actors[0].purchased_qte_level = Some(1);
    req.actors[0].character_max_level = Some(20);
    assert_eq!(probe_hits(&run(&req))[0]["crit_rate"], json!("1/2"));
    req.actors[0].character_level = 80;
    req.actors[0].character_max_level = Some(80);
    req.incoming_damage_events.push(GameActorDamageEvent {
        timestamp_us: 1_000_000,
        actor_index: 0,
        delivered_damage: req.actors[0].initial_hp,
    });
    let model = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    let report = run(&req);
    assert_eq!(report["final_actor_states"][0]["hp"], 0);
    assert_eq!(probe_hits(&report)[0]["crit_rate"], json!("3/5"));
}

#[test]
fn hathor_remora_native_aoe_bonus_is_specific_to_the_status_target() {
    let source = "options duration=5 crit=fixed;
        hotori char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        hathor char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=1080 resist=0;
        target hp=100000000 def=1080 resist=0; active hotori;";
    let mut req = crate::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    req.actions = vec![
        GameActionCommand::SwitchActor {
            timestamp_us: 0,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        super::hathor_actions::make_action(
            3_000_000,
            1,
            "GA_Hathor_Skill",
            "GE_Player_Hathor_Skill1_Damage",
            0,
            0,
        ),
    ];
    req.actors[1].static_stats.crit_rate = Ratio {
        numerator: 1,
        denominator: 5,
    };
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 10);
    for hit in hits {
        let expected = if hit["target_index"] == 0 {
            json!("3/10")
        } else {
            json!("1/5")
        };
        assert_eq!(hit["crit_rate"], expected);
    }
}

#[test]
fn hathor_remora_refresh_is_one_status_and_dead_source_cannot_enhance_a_new_generation() {
    let mut req = fixture(&[15_999_999, 16_000_000], "COSMOS");
    req.actors[1].initial_cycle = 100;
    req.actions.insert(
        1,
        GameActionCommand::SwitchActor {
            timestamp_us: 4_000_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
    );
    req.action_metadata = std::mem::take(&mut req.action_metadata)
        .into_iter()
        .map(|(index, metadata)| (index + 1, metadata))
        .collect();
    for command in &mut req.actions[2..] {
        if let GameActionCommand::InvokeAbility { actor_index, .. } = command {
            *actor_index = 0;
        }
    }
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(probe_hits(&report)[0]["crit_rate"], json!("3/5"));
    assert_eq!(probe_hits(&report)[1]["crit_rate"], json!("1/2"));

    let mut req = fixture(&[6_000_000], "COSMOS");
    let mut other = req.actors[0].clone();
    other.slot = 2;
    other.character_id = "character:1071".into();
    other.character_level = 20;
    other.purchased_qte_level = Some(1);
    other.character_max_level = Some(20);
    other.initial_cycle = 0;
    req.actors.push(other);
    req.actors[1].initial_cycle = 100;
    req.actions.insert(
        1,
        GameActionCommand::SwitchActor {
            timestamp_us: 4_000_000,
            target_actor_index: 2,
            cooldown_us: 0,
        },
    );
    let metadata = req.action_metadata.remove(&1).unwrap();
    req.action_metadata.insert(2, metadata);
    if let GameActionCommand::InvokeAbility { actor_index, .. } = &mut req.actions[2] {
        *actor_index = 2;
    }
    req.incoming_damage_events.push(GameActorDamageEvent {
        timestamp_us: 1_000_000,
        actor_index: 0,
        delivered_damage: req.actors[0].initial_hp,
    });
    let model = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(probe_hits(&report)[0]["crit_rate"], json!("1/2"));
    let statuses: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "reaction_remora_applied")
        .collect();
    assert_eq!(statuses.len(), 2);
    assert_eq!(statuses[1]["data"]["duration_us"], 5_000_000);
}
