use super::nanally_actions::{make_action, request};
use super::*;
use serde_json::{Value, json};

fn scenario() -> GameScenarioRequest {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([1, 3]).unwrap();
    req.actors[0].talent_levels = Some([8, 1, 1]);
    req.actors[0].initial_energy = 100;
    req.targets[0].hp_current = 10_000_000;
    req.targets[0].hp_max = 10_000_000;
    req.targets[0].defense = 1080;
    req.relative_duration_us = 13_000_000;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Nanally_Skill",
        "GE_Player_Nanally_Skill1_Damage",
        0,
        0,
    )];
    req
}

fn followups(report: &Value) -> Vec<&Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == crate::characters::nanally::authority::PACKET)
        .collect()
}

#[test]
fn authority_a3_uses_normal_talent_on_skill_normal_burst_and_exact_icd_boundary() {
    let req = scenario();
    for (at, accepted) in [(1_306_308, false), (1_306_309, true), (1_306_310, true)] {
        let mut req = req.clone();
        req.actions.push(make_action(
            at,
            0,
            "GA_Nanally_Melee",
            "GE_Player_Nanally_Melee1_Damage",
            0,
            0,
        ));
        req.actions.push(make_action(
            2_600_000,
            0,
            "GA_Nanally_UltraSkill",
            "GE_Player_Nanally_UltraSkill1_Damage",
            100,
            0,
        ));
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        let hits = followups(&report);
        assert_eq!(
            hits.iter()
                .map(|hit| hit["time_us"].as_u64().unwrap())
                .collect::<Vec<_>>(),
            if accepted {
                vec![497_363, at + 191_054, 4_900_192]
            } else {
                vec![497_363, 4_900_192]
            }
        );
        assert_eq!(
            hits.iter()
                .map(|hit| hit["ability_id"].as_str().unwrap())
                .collect::<Vec<_>>(),
            if accepted {
                vec![
                    "GA_Nanally_Skill",
                    "GA_Nanally_Melee",
                    "GA_Nanally_UltraSkill",
                ]
            } else {
                vec!["GA_Nanally_Skill", "GA_Nanally_UltraSkill"]
            }
        );
        for hit in hits {
            // Published A3 Normal level 8 is .857, independent of Skill/Burst level 1.
            assert_eq!(hit["non_crit_damage"], json!("857/2"));
            assert_eq!(
                super::support::hit_metadata(&report, hit)["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"
            );
        }
        let grants = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "nanally_a1_energy_grant")
            .collect::<Vec<_>>();
        assert_eq!(
            grants
                .iter()
                .map(|event| event["time_us"].as_u64().unwrap())
                .collect::<Vec<_>>(),
            if accepted {
                vec![497_363, at + 191_054, 4_900_192]
            } else {
                vec![497_363, 4_900_192]
            }
        );
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
        assert_eq!(
            run_game_scenario(&req, &ExecutionLimits::default()).unwrap(),
            report
        );
    }
}

#[test]
fn authority_a3_checks_selected_identity_half_open_lifetime_and_field_exit_at_delivery() {
    for effects in [&[1][..], &[3][..], &[1, 3, 6][..]] {
        let mut req = scenario();
        req.actors[0].awakening =
            ntesim_data::AwakeningSelection::selected(effects.iter().copied()).unwrap();
        req.actions.push(make_action(
            12_238_863,
            0,
            "GA_Nanally_Melee",
            "GE_Player_Nanally_Melee1_Damage",
            0,
            0,
        ));
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            followups(&report).len(),
            if effects.contains(&6) {
                2
            } else {
                usize::from(effects.contains(&3))
            }
        );
    }
    let mut req = scenario();
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.awakening = Default::default();
    req.actors.push(ally);
    let clock = crate::characters::nanally::clock::MODEL;
    req.model_assumptions
        .insert(clock.into(), clock.description().into());
    // An explicit interrupt cancels unborn Authority before a legal field exit.
    req.interrupt_events.push(GameActorInterruptEvent {
        timestamp_us: 400_000,
        actor_index: 0,
        armor_damage: None,
    });
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 500_000,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    let report = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert!(followups(&report).is_empty());
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_kind"] == "nanally_a1_energy_grant")
    );
    req.interrupt_events.clear();
    req.actions.truncate(1);
    req.relative_duration_us = 21_000_000;
    req.actions.extend([
        GameActionCommand::SwitchActor {
            timestamp_us: 1_200_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 1_300_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        make_action(
            16_500_000,
            0,
            "GA_Nanally_Skill",
            "GE_Player_Nanally_Skill1_Damage",
            0,
            16_000_000,
        ),
        make_action(
            18_000_000,
            0,
            "GA_Nanally_Melee",
            "GE_Player_Nanally_Melee1_Damage",
            0,
            0,
        ),
    ]);
    let report = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    assert_eq!(
        followups(&report)
            .iter()
            .map(|hit| hit["time_us"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [497_363, 16_997_363, 18_191_054]
    );
}

#[test]
fn authority_a3_requires_positive_hp_loss_and_never_attacks_a_defeated_recipient() {
    for boundary in [0, 1, 2] {
        let mut req = scenario();
        match boundary {
            0 => req.targets[0].resistance = Ratio::one(),
            1 => {
                req.actors[0].static_stats.atk = Ratio {
                    numerator: 1,
                    denominator: 1000,
                }
            }
            _ => req.targets[0].hp_current = 1,
        }
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert!(followups(&report).is_empty());
        assert!(
            !report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .any(|event| event["event_kind"] == "nanally_a1_energy_grant")
        );
    }
}
