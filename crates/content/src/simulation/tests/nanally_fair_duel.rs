use super::nanally_actions::make_action;
use super::*;
use serde_json::{Value, json};
const FAIR: &str = crate::characters::nanally::fair_duel::PACKET;

fn request(two: bool) -> GameScenarioRequest {
    let mut req = super::nanally_actions::request();
    req.relative_duration_us = 50_000_000;
    req.active_actor_index = 1;
    req.stochastic_crit = None;
    req.actors[0].initial_energy = 0;
    req.actors[0].initial_cycle = 0;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actors[0].static_stats.cycle_intensity = Ratio::zero();
    req.targets[0].defense = 1080;
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].hp_max = 100_000_000;
    for (slot, id) in [(1, "character:1046"), (2, "character:1021")] {
        let mut actor = req.actors[0].clone();
        actor.slot = slot;
        actor.character_id = id.into();
        actor.initial_cycle = 100;
        req.actors.push(actor);
    }
    req.actions = vec![
        super::reactions::switch(0, 0),
        make_action(
            1_400_000,
            0,
            "GA_Nanally_Skill",
            "GE_Player_Nanally_Skill1_Damage",
            0,
            16_000_000,
        ),
    ];
    if two {
        req.actions.extend([
            super::reactions::switch(15_000_000, 2),
            super::reactions::switch(20_000_000, 0),
            make_action(
                21_400_000,
                0,
                "GA_Nanally_Skill",
                "GE_Player_Nanally_Skill1_Damage",
                0,
                16_000_000,
            ),
        ]);
    }
    req
}
fn run(req: &GameScenarioRequest) -> Value {
    let result = run_game_scenario_report(req, &ExecutionLimits::default()).unwrap();
    assert!(
        result.success && result.execution_completed,
        "{:?}",
        result.halt_diagnostic
    );
    result.validate_damage_parentage().unwrap();
    serde_json::to_value(result).unwrap()
}
fn fair(report: &Value) -> Vec<&Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == FAIR)
        .collect()
}

#[test]
fn nanally_fair_duel_uses_normal_talent_and_keeps_both_real_parents_across_two_cycles() {
    for foreign in [false, true] {
        let mut req = request(true);
        req.actors[0].talent_levels = Some([8, 1, 10]);
        if foreign {
            for actor in &mut req.actors[1..] {
                actor.static_stats.cycle_intensity = Ratio {
                    numerator: 600,
                    denominator: 1,
                };
            }
        } else {
            req.actors[0].static_stats.cycle_intensity = Ratio {
                numerator: 600,
                denominator: 1,
            };
        }
        let report = run(&req);
        let hits = fair(&report);
        assert_eq!(hits.len(), 10);
        let times = [2, 4, 6, 8, 10, 22, 24, 26, 28, 30];
        for (hit, time) in hits.iter().zip(times) {
            assert_eq!(hit["time_us"], time * 1_000_000);
            // Normal8 Fair Duel1.028, ATK1000, mitigation1/2. Skill1/Burst10 do not select its curve.
            assert_eq!(hit["non_crit_damage"], json!("514/1"));
            assert!(hit["command_index"].is_null());
            let event = &report["autonomous_events"]
                [hit["autonomous_event_index"].as_u64().unwrap() as usize];
            assert_eq!(event["actor_index"], 0);
            assert_eq!(event["active_actor_index"], 0);
            assert_eq!(event["trigger"]["kind"], "coordinated_hit");
            assert_eq!(
                event["trigger"]["cast_command_index"],
                if time < 20 { 1 } else { 4 }
            );
            let parent =
                &report["hit_receipts"][event["trigger"]["hit_index"].as_u64().unwrap() as usize];
            assert_eq!(parent["time_us"], hit["time_us"]);
            assert_eq!(parent["damage_effect_id"], "ntesim.reaction.blossom");
            assert!(parent["applied_hp_loss"].as_u64().unwrap() > 0);
            assert_eq!(
                parent["actor_index"],
                if foreign {
                    if time < 20 { 1 } else { 2 }
                } else {
                    0
                }
            );
        }
        assert_eq!(report, run(&req));
    }
}

#[test]
fn nanally_fair_duel_rejects_missing_breakthrough_authority_zero_loss_and_field_exit() {
    let mut req = request(false);
    req.actors[0].character_level = 20;
    req.actors[0].purchased_qte_level = Some(1);
    req.actors[0].character_max_level = Some(20);
    assert!(fair(&run(&req)).is_empty());
    let mut req = request(false);
    req.actions.pop();
    assert!(fair(&run(&req)).is_empty());
    let mut req = request(false);
    req.targets[0].resistance = Ratio {
        numerator: 1_000_000_000,
        denominator: 1,
    };
    assert!(fair(&run(&req)).is_empty());
    let mut req = request(false);
    req.actions.push(super::reactions::switch(5_000_000, 1));
    assert_eq!(fair(&run(&req)).len(), 2);
}

#[test]
fn nanally_fair_duel_awakenings_use_one_followup_unit_and_only_self_energy() {
    let mut req = request(true);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([1, 5]).unwrap();
    let report = run(&req);
    let hits = fair(&report);
    assert_eq!(hits.len(), 10);
    for (hit, damage) in hits
        .iter()
        .zip([300, 306, 312, 318, 324, 300, 306, 312, 318, 324])
    {
        assert_eq!(hit["non_crit_damage"], json!(format!("{}/{}", damage, 1)));
    }
    // Each cycle: ordinary7+6.3 and five self-only2.5; two cycles total51.6.
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 51);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        json!("3/5")
    );
    for kind in ["nanally_a1_energy_grant", "nanally_a5_atk_buff"] {
        assert_eq!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["event_kind"] == kind)
                .count(),
            10
        );
    }
}

#[test]
fn nanally_fair_duel_can_trigger_a3_without_recursion_or_shared_guard() {
    for foreign in [false, true] {
        let mut req = request(false);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([3]).unwrap();
        req.actors[if foreign { 1 } else { 0 }]
            .static_stats
            .cycle_intensity = Ratio {
            numerator: 600,
            denominator: 1,
        };
        let report = run(&req);
        assert_eq!(fair(&report).len(), 5);
        let a3 = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["damage_effect_id"] == crate::characters::nanally::authority::PACKET)
            .collect::<Vec<_>>();
        // Skill1.4001; own one-second Pistils3..10, or foreign Fair Duel4/6/8/10.
        assert_eq!(a3.len(), if foreign { 5 } else { 9 });
        for hit in a3 {
            assert_eq!(hit["non_crit_damage"], json!("250/1"));
        }
    }
}

#[test]
fn nanally_fair_duel_level_ten_preserves_the_native_1199_coefficient() {
    let mut req = request(false);
    req.actors[0].talent_levels = Some([10, 1, 1]);
    let report = run(&req);
    assert_eq!(fair(&report).len(), 5);
    for hit in fair(&report) {
        assert_eq!(hit["non_crit_damage"], json!("1199/2"));
    }
}

#[test]
fn nanally_fair_duel_source_fixture_replays_without_fake_player_commands() {
    let source = include_str!("../../../../../presets/native/nanally-reaction-control.nte");
    let result = crate::simulation::PreparedSourceScenario::new(
        source,
        Default::default(),
        &Default::default(),
    )
    .unwrap()
    .run(42)
    .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    assert_eq!(result.request.actions.len(), 3);
    assert!(
        result
            .request
            .actions
            .iter()
            .all(|action| !matches!(action, GameActionCommand::InvokeAbility { ability_id, .. } if ability_id.contains("fair_duel")))
    );
    assert_eq!(result.report.hits_settled, 21);
    assert_eq!(
        result
            .report
            .hit_receipts
            .iter()
            .filter(|hit| hit.damage_effect_id == FAIR)
            .count(),
        5
    );
    super::assert_native_replay(&result);
}

#[test]
fn nanally_fair_duel_two_second_guard_coalesces_overlapping_buds() {
    let mut req = request(false);
    req.actors[1].initial_cycle = 0;
    req.actors[0].initial_cycle = 100;
    req.actions = vec![
        super::reactions::switch(0, 0),
        super::reactions::switch(1_500_000, 2),
        super::reactions::switch(3_000_000, 0),
        make_action(
            6_600_000,
            0,
            "GA_Nanally_Skill",
            "GE_Player_Nanally_Skill1_Damage",
            0,
            16_000_000,
        ),
    ];
    let report = run(&req);
    // Buds born1.5/3s interleave. Authority starts7.029917, so7s is ineligible;
    // after7.5s the independent2s guard coalesces the intervening emissions.
    assert_eq!(
        fair(&report)
            .iter()
            .map(|hit| hit["time_us"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [7_500_000, 9_500_000, 11_500_000]
    );
}

#[test]
fn nanally_fair_duel_fatal_cycle_captures_lowest_surviving_recipient() {
    let mut req = request(false);
    let mut second = req.targets[0].clone();
    second.target_index = 1;
    req.targets.push(second);
    req.targets[0].hp_current = 10000; // Support1714 + Pistil4500 + five Skill500 leaves1286.
    req.targets[0].hp_max = 10000;
    let report = run(&req);
    let hits = fair(&report);
    assert!(!hits.is_empty());
    assert_eq!(hits[0]["target_index"], 1);
    let event =
        &report["autonomous_events"][hits[0]["autonomous_event_index"].as_u64().unwrap() as usize];
    let parent = &report["hit_receipts"][event["trigger"]["hit_index"].as_u64().unwrap() as usize];
    assert_eq!(parent["target_index"], 0);
    assert!(parent["applied_hp_loss"].as_u64().unwrap() > 0);
}

#[test]
fn nanally_fair_duel_ordinary_hits_do_not_trigger_it_and_hexed_counts_actual_followups() {
    let mut req = request(false);
    req.actors[1].initial_cycle = 0;
    req.actions.push(make_action(
        3_500_000,
        0,
        "GA_Nanally_Melee",
        "GE_Player_Nanally_Melee1_Damage",
        0,
        0,
    ));
    assert!(fair(&run(&req)).is_empty());

    let mut req = request(false);
    req.actors[1].character_id = "character:1003".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([1, 5]).unwrap();
    req.actions.push(make_action(
        3_500_000,
        0,
        "GA_Nanally_Melee",
        "GE_Player_Nanally_Melee1_Damage",
        0,
        0,
    ));
    let report = run(&req);
    let owned_followups = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| {
            hit["actor_index"] == 0
                && (hit["damage_effect_id"] == FAIR
                    || super::support::hit_metadata(&report, hit)["packet"]["semantic_tags"]
                        .as_array()
                        .is_some_and(|tags| tags.iter().any(|tag| tag == "hexed_followup")))
        })
        .count();
    assert!(owned_followups > fair(&report).len());
    let stack_events = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "nanally_a5_atk_buff")
        .count();
    assert_eq!(stack_events, owned_followups);
    // Support7 +Skill6.3 +Normal1.138 +three separated actual A1 grants2.5 =21.938.
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 21);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        json!("469/500")
    );
}
