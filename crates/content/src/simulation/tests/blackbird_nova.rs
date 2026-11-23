use super::*;
use crate::characters::blackbird::nova::{self, PACKET};
use crate::simulation::state::GameSimulatorState;
use serde_json::{Value, json};

fn request(times: &[(u64, u64)]) -> GameScenarioRequest {
    let template = research_request();
    let mut req =
        super::reactions::request(&[(0, "character:1042", 100, 0), (1, "character:1004", 0, 0)]);
    req.relative_duration_us = 12_000_000;
    req.actors[0].character_level = 80;
    req.actors[0].character_max_level = Some(80);
    for actor in &mut req.actors {
        actor.static_stats.base_atk = 1000;
        actor.static_stats.atk = Ratio {
            numerator: 1000,
            denominator: 1,
        };
        actor.initial_energy = 0;
    }
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].hp_max = 100_000_000;
    req.actions = vec![super::reactions::switch(0, 1)];
    for (time, damage) in times {
        let mut action = template.actions[0].clone();
        if let GameActionCommand::InvokeAbility {
            timestamp_us,
            actor_index,
            atk_rate,
            ..
        } = &mut action
        {
            *timestamp_us = *time;
            *actor_index = 1;
            *atk_rate = Ratio {
                numerator: *damage,
                denominator: 1000,
            };
        }
        req.action_metadata
            .insert(req.actions.len(), template.action_metadata[&0].clone());
        req.actions.push(action);
    }
    req
}

fn run(req: &GameScenarioRequest) -> Value {
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}
fn extras(report: &Value) -> Vec<&Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["damage_effect_id"] == PACKET)
        .collect()
}

#[test]
fn blackbird_nova_settles_actual_hp_once_per_generation_and_child_cannot_reapply() {
    let req = request(&[(1_000_000, 10_000), (6_000_000, 5_000)]);
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let extra = extras(&report);
    assert_eq!(extra.len(), 2);
    for (hit, time, value) in [(extra[0], 5_100_000, 2000), (extra[1], 10_200_000, 1000)] {
        assert_eq!(hit["time_us"], time);
        assert_eq!(hit["actor_index"], 0);
        assert_eq!(hit["non_crit_damage"], json!(format!("{}/{}", value, 1)));
        assert_eq!(hit["applied_hp_loss"], value);
        assert_eq!(hit["ability_id"], "GA_BlackBird_Passive_1");
        assert_eq!(hit["critical"], false);
        assert!(hit["break_contribution"].is_null());
        assert_eq!(hit["scaling_rate"], json!("0/1"));
    }
    assert_eq!(report["hit_receipts"].as_array().unwrap().len(), 7);
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["event_kind"] == "reaction_nova_reapplied")
            .count(),
        1
    );
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 4);
    assert_eq!(report["final_actor_states"][0]["charge_remainder"], "17/25");
}

#[test]
fn blackbird_nova_zero_accumulator_has_two_shared_ticks_and_no_fabricated_fixed_packet() {
    let report = run(&request(&[]));
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert!(extras(&report).is_empty());
    let hits: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "ntesim.reaction.nova")
        .collect();
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0]["time_us"], 5_000_000);
    assert_eq!(hits[1]["time_us"], 10_100_000);
}

#[test]
fn blackbird_nova_cap_is_base_atk_and_fixed_value_bypasses_res_def_bonuses_and_crit() {
    let mut req = request(&[(1_000_000, 10_000_000)]);
    req.actors[0].static_stats.atk = Ratio {
        numerator: 9000,
        denominator: 1,
    };
    req.actors[0].static_stats.mental_damage_bonus = Ratio {
        numerator: 7,
        denominator: 1,
    };
    req.actors[0].static_stats.damage_bonus = Ratio {
        numerator: 5,
        denominator: 1,
    };
    req.actors[0].static_stats.crit_rate = Ratio::one();
    req.targets[0].defense = 1_000_000;
    req.targets[0].resistance = Ratio {
        numerator: 9,
        denominator: 10,
    };
    req.action_metadata.get_mut(&1).unwrap().damage_type_ex =
        Some("EExecutionDamageTypeEX::DAMAGE_TYPE_TRUE".into());
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let extra = extras(&report);
    assert_eq!(extra.len(), 1);
    // 20% of10m is2m; captured base1000 caps at1m, independent of live ATK9000.
    assert_eq!(extra[0]["damage"], json!("1000000/1"));
    assert_eq!(extra[0]["rounded_damage"], 1_000_000);
    assert_eq!(extra[0]["critical"], false);
}

#[test]
fn blackbird_nova_filters_trigger_cast_and_nova_packets_and_keeps_targets_independent() {
    let mut req = request(&[]);
    let mut target = req.targets[0].clone();
    target.target_index = 1;
    req.targets.push(target);
    let mut state = GameSimulatorState::new(&req);
    for (target, generation) in [(0, 10), (1, 11)] {
        nova::create(&req, &mut state, target, generation, 0, 5_100_000, true).unwrap();
    }
    let template = research_request();
    let mut metadata = crate::simulation::metadata::resolve_requested_action_packet(
        &req.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        &req.actors[1].character_id,
        "caller-test",
        "caller-effect",
        Some(&template.action_metadata[&0]),
        &req.model_assumptions,
    )
    .unwrap();
    state.advance_unpaused_test_clock(1_000_000);
    nova::observe(&req, &mut state, 0, 0, &metadata, 3000).unwrap(); // Whole causal QTE cast.
    nova::observe(&req, &mut state, 0, 1, &metadata, 10000).unwrap();
    nova::observe(&req, &mut state, 1, 1, &metadata, 2500).unwrap();
    for packet in ["ntesim.reaction.nova", PACKET] {
        metadata.packet.damage_effect_id = packet.into();
        nova::observe(&req, &mut state, 0, 2, &metadata, 4000).unwrap();
    }
    metadata.packet.damage_effect_id = "caller-effect".into();
    state.advance_unpaused_test_clock(5_100_000);
    nova::observe(&req, &mut state, 0, 3, &metadata, 5000).unwrap(); // Half-open expiry.
    for (target, generation, expected) in [(0, 10, 2000), (1, 11, 500)] {
        let effect = nova::end(&req, &mut state, 0, target, generation)
            .unwrap()
            .unwrap();
        assert_eq!(
            effect.pending_damage,
            ExactFraction::new(expected, 1).unwrap()
        );
        assert!(
            nova::end(&req, &mut state, 0, target, generation)
                .unwrap()
                .is_none()
        );
    }
}

#[test]
fn blackbird_nova_source_defeat_stops_extra_and_child_but_keeps_shared_pending_damage() {
    let mut req = request(&[(1_000_000, 10_000)]);
    let incoming = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(incoming.as_str().into(), incoming.description().into());
    req.incoming_damage_events.push(GameActorDamageEvent {
        timestamp_us: 2_000_000,
        actor_index: 0,
        delivered_damage: req.actors[0].initial_hp,
    });
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert!(extras(&report).is_empty());
    assert_eq!(report["hit_receipts"].as_array().unwrap().len(), 3);
    assert_eq!(report["hit_receipts"][2]["time_us"], 5_000_000);
    req.incoming_damage_events.clear();
    req.targets[0].hp_current = 1;
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert!(extras(&report).is_empty());
    assert_eq!(report["hit_receipts"].as_array().unwrap().len(), 1);
}

#[test]
fn blackbird_nova_authored_refresh_wins_and_old_settlement_cannot_feed_the_new_generation() {
    let mut req = request(&[(1_000_000, 10_000)]);
    req.relative_duration_us = 14_000_000;
    req.actors[1].initial_cycle = 100;
    req.actions.push(super::reactions::switch(3_000_000, 0));
    let mut hit = req.actions[1].clone();
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index,
        atk_rate,
        ..
    } = &mut hit
    {
        *timestamp_us = 4_000_000;
        *actor_index = 0;
        *atk_rate = Ratio {
            numerator: 5,
            denominator: 1,
        };
    }
    req.actions.push(hit);
    req.action_metadata
        .insert(3, req.action_metadata[&1].clone());
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let extra = extras(&report);
    assert_eq!(extra.len(), 2);
    assert_eq!(extra[0]["time_us"], 3_000_000);
    assert_eq!(extra[0]["applied_hp_loss"], 2000);
    assert_eq!(extra[1]["time_us"], 8_100_000);
    assert_eq!(extra[1]["applied_hp_loss"], 1000);
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["event_kind"] == "reaction_nova_reapplied")
            .count(),
        1
    );
}

#[test]
fn blackbird_nova_fixed_overkill_is_clipped_to_hp_and_cannot_create_a_child_after_death() {
    let mut req = request(&[(1_000_000, 10_000)]);
    req.targets[0].hp_current = 11000;
    req.targets[0].hp_max = 11000;
    req.targets[0].resistance = Ratio::one();
    req.action_metadata.get_mut(&1).unwrap().damage_type_ex =
        Some("EExecutionDamageTypeEX::DAMAGE_TYPE_TRUE".into());
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let extra = extras(&report);
    assert_eq!(extra.len(), 1);
    assert_eq!(extra[0]["rounded_damage"], 2000);
    assert_eq!(extra[0]["applied_hp_loss"], 1000);
    assert_eq!(report["final_target_states"][0]["hp_current"], 0);
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["event_kind"] == "reaction_nova_reapplied")
    );
}

#[test]
fn blackbird_nova_includes_applied_party_break_damage_without_duplicating_nova_ticks() {
    let mut req = request(&[(1_000_000, 1000)]);
    for actor in &mut req.actors {
        actor.static_stats.break_intensity = Ratio::zero();
    }
    req.action_metadata.get_mut(&1).unwrap().unbal_value = Some(Ratio::one());
    req.targets[0].break_profile = Some(GameTargetBreakProfileConfig {
        capacity: Ratio {
            numerator: 30,
            denominator: 1,
        },
        current_gauge: Ratio {
            numerator: 7,
            denominator: 2,
        },
        broken_duration_us: 100_000,
        discord_fraction: None,
    });
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let extra = extras(&report);
    assert_eq!(extra.len(), 1);
    // Two living Lv80 contributors each supply3603*(30/3)=36030 at DEF/RES0.
    // The independent1000 direct hit makes accumulatedHP73060; one fifth=14612.
    assert_eq!(extra[0]["damage"], json!("14612/1"));
    assert_eq!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit.get("break_contribution").is_some())
            .count(),
        2
    );
}

#[test]
fn blackbird_nova_observes_unrelated_scorch_and_support_hp_in_each_live_generation_only() {
    let mut req = super::reactions::request(&[
        (0, "character:1042", 100, 0),
        (1, "character:1004", 100, 0),
        (2, "character:1033", 0, 0),
    ]);
    req.actors[0].character_level = 80;
    req.actors[0].character_max_level = Some(80);
    req.actors[0].static_stats.base_atk = 1000;
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].hp_max = 100_000_000;
    req.relative_duration_us = 17_000_000;
    req.actions = vec![
        super::reactions::switch(0, 1),
        super::reactions::switch(1_000_000, 2),
    ];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let extra = extras(&report);
    assert_eq!(extra.len(), 2);
    for (hit, start, until) in [(extra[0], 0, 5_100_000), (extra[1], 5_100_000, 10_200_000)] {
        let hp: u128 = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| {
                (hit["damage_effect_id"] == "ntesim.reaction.scorch"
                    || hit["ability_id"] == "GA_Adler_QTE")
                    && hit["time_us"].as_u64().unwrap() >= start
                    && hit["time_us"].as_u64().unwrap() < until
            })
            .map(|hit| u128::from(hit["applied_hp_loss"].as_u64().unwrap()))
            .sum();
        assert!(hp > 0);
        let expected = ExactFraction::new(hp, 5).unwrap();
        assert_eq!(hit["damage"], serde_json::to_value(expected).unwrap());
    }
}

#[test]
fn blackbird_nova_keeps_aurelia_end_callbacks_and_only_child_collects_their_later_hp() {
    let mut req = request(&[]);
    let mut aurelia = req.actors[0].clone();
    aurelia.slot = 2;
    aurelia.character_id = "character:1070".into();
    aurelia.initial_cycle = 0;
    aurelia.static_stats.crit_rate = Ratio::zero();
    req.actors.push(aurelia);
    let id = crate::HypothesisId::AureliaHarmonicsB2V1;
    req.model_assumptions
        .insert(id.as_str().into(), id.description().into());
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let extra = extras(&report);
    assert_eq!(extra.len(), 1);
    assert_eq!(extra[0]["time_us"], 10_200_000);
    // Parent ends before its three500HP Harmonics, which feed the child only.
    assert_eq!(extra[0]["damage"], json!("300/1"));
    assert_eq!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "GE_Reaction_4_new_1070_Damage")
            .count(),
        6
    );
}
