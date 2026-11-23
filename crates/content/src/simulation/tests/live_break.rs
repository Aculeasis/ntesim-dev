use super::*;
use serde_json::{Value, json};

fn break_request() -> GameScenarioRequest {
    let mut request = research_request();
    request.stochastic_crit = None;
    request.relative_duration_us = 1;
    request.actions.truncate(1);
    request.action_metadata.retain(|index, _| *index == 0);
    request.action_metadata.get_mut(&0).unwrap().unbal_value = Some(Ratio {
        numerator: 30,
        denominator: 1,
    });
    request.targets[0].break_profile = Some(GameTargetBreakProfileConfig {
        capacity: Ratio {
            numerator: 30,
            denominator: 1,
        },
        current_gauge: Ratio::one(),
        broken_duration_us: 100,
        discord_fraction: None,
    });
    request
}

fn contributions(report: &Value) -> Vec<&Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit.get("break_contribution").is_some())
        .collect()
}

#[test]
fn live_break_caps_combined_defense_ignore_without_changing_lower_values() {
    use crate::simulation::state::{GameSimulatorState, TimedBuffDeclaration};
    use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope};
    let request = break_request();
    for (second, expected) in [(0, 230592), (25, 288240), (30, 288240)] {
        let mut state = GameSimulatorState::new(&request);
        for (id, numerator) in [("first", 75), ("second", second)] {
            state
                .apply_buff(
                    0,
                    TimedBuffDeclaration::reset(
                        id,
                        "test:def_ignore",
                        BuffRecipient::SelfActor,
                        10,
                        BuffStackingScope::PerSource,
                        BTreeMap::from([(
                            "def_ignore".into(),
                            ExactFraction::new(numerator, 100).unwrap(),
                        )]),
                    ),
                )
                .unwrap();
        }
        let mut queue = crate::simulation::timer_queue::EventQueue::new();
        crate::simulation::break_damage::schedule(&request, &mut state, &mut queue, &mut 0, 0, 0)
            .unwrap();
        assert_eq!(queue.len(), 1);
        let event = queue.pop().unwrap();
        let crate::simulation::events::GameEventData::TargetBreakDamage(packet) = event.data else {
            panic!("missing Break packet")
        };
        // 3603*(1+0/300)*(30/3)*.8 =28824 before DEF;
        // Fdef at 75% ignore is 4/5, at 100% it is 1.
        assert_eq!(packet.damage, ExactFraction::new(expected, 10).unwrap());
    }
}

#[test]
fn live_break_uses_bi_and_source_resistance_without_ordinary_bonuses_or_energy() {
    let mut request = break_request();
    request.actors[0].static_stats.break_intensity = Ratio {
        numerator: 300,
        denominator: 1,
    };
    request.actors[0].static_stats.damage_bonus = Ratio {
        numerator: 9,
        denominator: 1,
    };
    request.actors[0].static_stats.crit_damage_bonus = Ratio {
        numerator: 99,
        denominator: 1,
    };
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    let hits = contributions(&report);
    assert_eq!(hits.len(), 1);
    // 3603*(1+300/300)*(30/3)*(1080/(1080+1080))*.8 = 28824.
    assert_eq!(hits[0]["damage"], json!("28824/1"));
    assert_eq!(hits[0]["critical"], false);
    assert!(hits[0].get("crit_sample").is_none());
    assert_eq!(hits[0]["scaling_rate"], json!("0/1"));
    assert_eq!(
        hits[0]["break_contribution"]["original_capacity"],
        json!("30/1")
    );
    let first_hp = report["hit_receipts"][0]["applied_hp_loss"]
        .as_u64()
        .unwrap();
    assert_eq!(
        report["final_target_states"][0]["hp_current"],
        10_000_000 - first_hp - 28824
    );
    request.targets[0].break_profile = None;
    let plain = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["final_actor_states"], plain["final_actor_states"]);
}

#[test]
fn live_break_preserves_rng_and_excludes_dead_contributors_but_retains_off_field_owners() {
    let mut request = break_request();
    request.actions.clear();
    request.action_metadata.clear();
    // Drive a captured group directly to isolate admission-time HP eligibility.
    let mut ally = request.actors[0].clone();
    ally.slot = 1;
    ally.character_id = crate::registry::get_character_descriptor("zero")
        .unwrap()
        .character_id
        .into();
    request.actors.push(ally);
    request.targets[0]
        .elemental_resistances
        .insert("NATURE".into(), Ratio::zero());
    request.targets[0].elemental_resistances.insert(
        "COSMOS".into(),
        Ratio {
            numerator: 1,
            denominator: 5,
        },
    );
    let mut state = crate::simulation::state::GameSimulatorState::new(&request);
    state.targets[0].break_state.as_mut().unwrap().generation = 1;
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    crate::simulation::break_damage::schedule(&request, &mut state, &mut queue, &mut 0, 0, 0)
        .unwrap();
    for event in queue.iter() {
        if let crate::simulation::events::GameEventData::TargetBreakDamage(packet) = &event.data {
            assert_eq!(
                packet.damage,
                ExactFraction::new(
                    if packet.actor_index == 0 {
                        18015
                    } else {
                        14412
                    },
                    1
                )
                .unwrap()
            );
        }
    }
    let mut owners = queue
        .iter()
        .map(|event| match &event.data {
            crate::simulation::events::GameEventData::TargetBreakDamage(packet) => {
                packet.actor_index
            }
            _ => unreachable!(),
        })
        .collect::<Vec<_>>();
    owners.sort_unstable();
    assert_eq!(owners, vec![0, 1]);
    state.actors[1].hp = 0;
    queue.clear();
    crate::simulation::break_damage::schedule(&request, &mut state, &mut queue, &mut 0, 0, 0)
        .unwrap();
    assert_eq!(queue.len(), 1);
    let mut stochastic = break_request();
    stochastic.relative_duration_us = 30;
    stochastic.actions = research_request().actions[..3].to_vec();
    stochastic.action_metadata = research_request().action_metadata;
    stochastic.action_metadata.retain(|index, _| *index < 3);
    stochastic.action_metadata.get_mut(&0).unwrap().unbal_value = Some(Ratio {
        numerator: 30,
        denominator: 1,
    });
    stochastic.stochastic_crit = research_request().stochastic_crit;
    let with = run_game_scenario(&stochastic, &ExecutionLimits::default()).unwrap();
    stochastic.targets[0].break_profile = None;
    let without = run_game_scenario(&stochastic, &ExecutionLimits::default()).unwrap();
    let ordinary = |report: &Value| {
        let catalog = report["metadata_catalog"].as_array().unwrap();
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit.get("break_contribution").is_none())
            .map(|hit| {
                let mut item = hit.clone();
                let id = item["metadata_id"].as_u64().unwrap() as usize;
                item.as_object_mut().unwrap().remove("metadata_id");
                item.as_object_mut()
                    .unwrap()
                    .insert("metadata".into(), catalog[id].clone());
                item
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(ordinary(&with), ordinary(&without));
}

#[test]
fn live_break_defeat_cancels_later_group_packets_and_keeps_causal_damage_accounting() {
    let mut request = break_request();
    let mut ally = request.actors[0].clone();
    ally.slot = 1;
    request.actors.push(ally);
    request.targets[0].hp_current = 5000;
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    let hits = contributions(&report);
    assert_eq!(hits.len(), 1);
    assert_eq!(report["final_target_states"][0]["hp_current"], 0);
    assert_eq!(hits[0]["actor_index"], 0);
    assert_eq!(hits[0]["command_index"], 0);
    assert_eq!(report["hits_settled"], 2);
}

#[test]
fn daffodil_light_prism_and_individually_selected_a6_use_break_only_zones() {
    for (awakening, expected) in [
        (ntesim_data::AwakeningSelection::prefix(0), (86472, 5)),
        (
            ntesim_data::AwakeningSelection::selected([6]).unwrap(),
            (691776, 37),
        ),
    ] {
        let mut request = break_request();
        request.actors[0].character_id = "character:1054".into();
        request.actors[0].awakening = awakening;
        let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            contributions(&report)[0]["damage"],
            json!(format!("{}/{}", expected.0, expected.1))
        );
    }
}
