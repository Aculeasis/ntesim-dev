// nest_bird weapon tests (implemented by task agent)
use super::*;
use serde_json::json;

fn nest_bird_request(tier: u32, numerator: u64, denominator: u64) -> GameScenarioRequest {
    let mut req = research_request();
    req.actors[0].weapon_id = "weapon:fork_NestBird".into();
    req.actors[0].weapon_tier = tier;
    req.actors[0].gear_procs = vec![GearProcConfig::NestBirdGaze {
        damage_reduction: Ratio {
            numerator,
            denominator,
        },
        duration_us: 20_000_000,
    }];
    req
}

fn synthetic_hit(timestamp_us: u64) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index: 0,
        ability_id: "caller-test".into(),
        array_index: 0,
        damage_effect_id: "caller-effect".into(),
        atk_rate: Ratio {
            numerator: 1,
            denominator: 1,
        },
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 0,
    }
}

fn request_with_categories_tier(
    tier: u32,
    numerator: u64,
    denominator: u64,
    hits: &[(u64, &str)],
) -> GameScenarioRequest {
    let mut req = nest_bird_request(tier, numerator, denominator);
    let template = req.action_metadata[&0].clone();
    req.actions = hits
        .iter()
        .map(|(timestamp_us, _)| synthetic_hit(*timestamp_us))
        .collect();
    req.action_metadata = hits
        .iter()
        .enumerate()
        .map(|(index, (_, category))| {
            let mut metadata = template.clone();
            metadata.damage_source_category = Some(format!(
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_{category}"
            ));
            (index, metadata)
        })
        .collect();
    req
}

fn request_with_categories(hits: &[(u64, &str)]) -> GameScenarioRequest {
    request_with_categories_tier(1, 18, 100, hits)
}

fn warning_gaze_marks(report: &serde_json::Value) -> Vec<serde_json::Value> {
    crate::simulation::tests::support::final_buffs(report)
        .into_iter()
        .filter(|buff| buff["id"] == "nest_bird.warning_gaze")
        .collect()
}

#[test]
fn nest_bird_marks_ultimate_hits_with_frozen_tier_ratio() {
    // Frozen buff_NestBird_down magnitudes reduce to these exact fractions.
    let expected = [(9u64, 50u64), (21, 100), (6, 25), (27, 100), (3, 10)];
    for (tier, (numerator, denominator)) in (1u32..=5).zip(expected) {
        let mut req = nest_bird_request(tier, numerator, denominator);
        let template = req.action_metadata[&0].clone();
        req.actions = vec![synthetic_hit(0), synthetic_hit(1_000_000)];
        req.relative_duration_us = 10_000_000;
        req.action_metadata = (0..2)
            .map(|index| {
                let mut metadata = template.clone();
                metadata.damage_source_category = Some(format!(
                    "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_{}",
                    if index == 0 { "Q" } else { "A" }
                ));
                (index, metadata)
            })
            .collect();
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let marks = warning_gaze_marks(&report);
        assert_eq!(marks.len(), 1, "tier {tier}: exactly one mark instance");
        let mark = &marks[0];
        assert_eq!(mark["source"], "weapon:fork_NestBird");
        assert_eq!(mark["owner_actor"], 0);
        assert_eq!(
            mark["recipient"],
            json!({"type": "specific_target", "target": "0"})
        );
        assert_eq!(
            mark["stat_modifiers"]["outgoing_damage_reduction"],
            json!(format!("{}/{}", numerator, denominator))
        );
        assert_eq!(mark["stacks"], 1);
        assert_eq!(
            mark["duration"],
            json!({"kind": "timed", "duration_us": 20_000_000, "expires_at_us": 20_000_000})
        );
        assert_eq!(mark["semantic_tags"], json!(["nest_bird.warning_gaze"]));
        assert_eq!(
            report["resolved_model_assumptions"][crate::weapons::nest_bird::HYPOTHESIS_ID.as_str()],
            crate::weapons::nest_bird::HYPOTHESIS_ID.description()
        );
    }
}

#[test]
fn nest_bird_ignores_non_ultimate_hits_and_zero_damage_ultimates() {
    let req = request_with_categories(&[(0, "A"), (1_000_000, "E"), (2_000_000, "H")]);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hit_receipts"].as_array().unwrap().len(), 3);
    assert!(warning_gaze_marks(&report).is_empty());

    let mut req = request_with_categories(&[(0, "Q")]);
    if let GameActionCommand::InvokeAbility { atk_rate, .. } = &mut req.actions[0] {
        *atk_rate = Ratio {
            numerator: 0,
            denominator: 1,
        };
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert!(warning_gaze_marks(&report).is_empty());
}

#[test]
fn nest_bird_does_not_stack_and_refreshes_the_window() {
    let mut req = request_with_categories(&[(0, "Q"), (10_000_000, "Q")]);
    req.relative_duration_us = 25_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let marks = warning_gaze_marks(&report);
    assert_eq!(marks.len(), 1);
    assert_eq!(
        marks[0]["duration"],
        json!({"kind": "timed", "duration_us": 20_000_000, "expires_at_us": 30_000_000})
    );
    assert_eq!(
        marks[0]["stat_modifiers"]["outgoing_damage_reduction"],
        json!("18/100")
    );
}

#[test]
fn nest_bird_mark_expires_at_the_window_end() {
    let mut req = request_with_categories(&[(0, "Q")]);
    req.relative_duration_us = 20_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert!(
        warning_gaze_marks(&report).is_empty(),
        "the mark retires exactly at its 20s deadline"
    );

    let mut req = request_with_categories(&[(0, "Q")]);
    req.relative_duration_us = 19_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let marks = warning_gaze_marks(&report);
    assert_eq!(marks.len(), 1);
    assert_eq!(
        marks[0]["duration"],
        json!({"kind": "timed", "duration_us": 20_000_000, "expires_at_us": 20_000_000})
    );
}

#[test]
fn nest_bird_mark_can_reapply_after_expiry() {
    let mut req = request_with_categories(&[(0, "Q"), (25_000_000, "Q")]);
    req.relative_duration_us = 40_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let marks = warning_gaze_marks(&report);
    assert_eq!(marks.len(), 1);
    assert_eq!(
        marks[0]["duration"],
        json!({"kind": "timed", "duration_us": 20_000_000, "expires_at_us": 45_000_000})
    );
}

#[test]
fn nest_bird_rejects_bad_weapon_id_ratio_and_duration() {
    let req = nest_bird_request(1, 18, 100);
    let mut bad = req.clone();
    bad.actors[0].weapon_id = "weapon:fork_PaperPlane".into();
    assert!(
        run_game_scenario(&bad, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires equipped")
    );
    let mut ok = request_with_categories(&[(0, "Q")]);
    ok.actors[0].weapon_id = "WEAPON:FORK_NESTBIRD".into();
    assert!(
        run_game_scenario(&ok, &ExecutionLimits::default())
            .unwrap()
            .get("execution_completed")
            .is_some()
    );
    let mut bad = req.clone();
    bad.actors[0].gear_procs = vec![GearProcConfig::NestBirdGaze {
        damage_reduction: Ratio {
            numerator: 18,
            denominator: 100,
        },
        duration_us: 0,
    }];
    assert!(
        run_game_scenario(&bad, &ExecutionLimits::default())
            .unwrap_err()
            .contains("positive duration")
    );
    for (numerator, denominator) in [(0u64, 1u64), (3, 2)] {
        let mut bad = req.clone();
        bad.actors[0].gear_procs = vec![GearProcConfig::NestBirdGaze {
            damage_reduction: Ratio {
                numerator,
                denominator,
            },
            duration_us: 20_000_000,
        }];
        assert!(
            run_game_scenario(&bad, &ExecutionLimits::default())
                .unwrap_err()
                .contains("damage reduction ratio"),
            "ratio {numerator}/{denominator} must be rejected"
        );
    }
    let mut bad = req.clone();
    bad.actors[0].gear_procs = vec![GearProcConfig::NestBirdGaze {
        damage_reduction: Ratio {
            numerator: 1,
            denominator: 0,
        },
        duration_us: 20_000_000,
    }];
    assert!(
        run_game_scenario(&bad, &ExecutionLimits::default())
            .unwrap_err()
            .contains("denominator cannot be zero")
    );
}

#[test]
fn nest_bird_projection_binds_frozen_curves_and_rejects_group_mismatch() {
    let tables: std::collections::BTreeMap<String, serde_json::Value> =
        super::support::weapon_projection_tables();
    let item = ntesim_data::weapon_definitions::lookup(&tables, "fork_NestBird")
        .unwrap()
        .1;
    assert_eq!(
        item["ItemName"]["LocalizedString"],
        "Tears Beneath the Mask"
    );
    assert_eq!(
        item["ElementData"]["ApplyGroupType"],
        "ECharacterGroupType::CHARACTER_GROUP_TYPE_FOUR"
    );
    assert_eq!(item["ElementData"]["ExclusiveCharacterIDArray"], json!([]));
    let mut actor: GameActorConfig = serde_json::from_value(json!({
        "slot": 0, "character_id": "character:1036", "character_level": 80, "awakening": [],
        "purchased_skill_level": 8, "initial_hp": 1, "initial_hp_max": 1, "initial_energy": 0,
        "energy_capacity": 100, "weapon_id": "weapon:fork_NestBird", "weapon_tier": 1,
        "static_stats": {
            "base_atk": 1, "atk": "1/1", "hp": 1, "def": 1,
            "crit_rate": "0/1", "crit_damage_bonus": "0/1"
        }
    }))
    .unwrap();

    let expected = [
        ("18%", 9u64, 50u64),
        ("21%", 21, 100),
        ("24%", 6, 25),
        ("27%", 27, 100),
        ("30%", 3, 10),
    ];
    for (tier, (percent, numerator, denominator)) in (1u32..=5).zip(expected) {
        actor.weapon_tier = tier;
        actor.gear_procs.clear();
        let report = crate::weapons::nest_bird::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap();
        assert_eq!(
            report["compiled_proc"]["damage_reduction"],
            json!(format!("{}/{}", numerator, denominator)),
            "tier {tier}"
        );
        assert_eq!(report["compiled_proc"]["duration_us"], 20_000_000);
        assert_eq!(
            report["native_description"]["key"],
            format!("buff_fork_NestBird_effect{tier}")
        );
        assert!(
            report["native_description"]["localized"]
                .as_str()
                .unwrap()
                .contains(percent),
            "tier {tier}"
        );
        assert!(report.get("unapplied_curve").is_none());
        assert_eq!(actor.gear_procs.len(), 1);
    }

    actor.character_id = "character:1010".into();
    assert!(
        crate::weapons::nest_bird::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[]
        )
        .unwrap_err()
        .contains("group is incompatible")
    );

    actor.character_id = "character:1036".into();
    actor.weapon_tier = 1;
    let mut changed = tables.clone();
    changed.get_mut("CT_ForkBuff").unwrap()["rows"]["buff_NestBird_down"]["Keys"][0]["Value"] =
        json!(0.18);
    assert!(
        crate::weapons::nest_bird::projection::project(
            &changed,
            1,
            "ntesim-current",
            &mut actor,
            &[]
        )
        .unwrap_err()
        .contains("negative")
    );
    let mut missing = tables.clone();
    missing.get_mut("CT_ForkBuff").unwrap()["rows"]
        .as_object_mut()
        .unwrap()
        .remove("buff_NestBird_CD1");
    assert!(
        crate::weapons::nest_bird::projection::project(
            &missing,
            1,
            "ntesim-current",
            &mut actor,
            &[]
        )
        .unwrap_err()
        .contains("buff_NestBird_CD1")
    );
}

#[test]
fn nest_bird_all_known_non_ultimate_categories_mark_nothing() {
    // Closes the coverage gap behind the "other known categories do not mark"
    // claim: A/E/H were exercised by
    // nest_bird_ignores_non_ultimate_hits_and_zero_damage_ultimates, but
    // B/F/P/R/T/X/Z were not.
    let req = request_with_categories(&[
        (0, "B"),
        (1_000_000, "F"),
        (2_000_000, "P"),
        (3_000_000, "R"),
        (4_000_000, "T"),
        (5_000_000, "X"),
        (6_000_000, "Z"),
    ]);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hit_receipts"].as_array().unwrap().len(), 7);
    assert!(warning_gaze_marks(&report).is_empty());
}

#[test]
fn nest_bird_missing_or_unrecognized_category_fails_loudly() {
    // A missing classification fails the hit instead of silently skipping.
    let mut missing = request_with_categories(&[(0, "Q")]);
    missing
        .action_metadata
        .get_mut(&0)
        .unwrap()
        .damage_source_category = None;
    assert!(
        run_game_scenario(&missing, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires damage source classification")
    );

    // An unrecognized classification fails the hit instead of silently skipping.
    let mut unknown = request_with_categories(&[(0, "Q")]);
    unknown
        .action_metadata
        .get_mut(&0)
        .unwrap()
        .damage_source_category =
        Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_QX".into());
    assert!(
        run_game_scenario(&unknown, &ExecutionLimits::default())
            .unwrap_err()
            .contains("unsupported")
    );
}

#[test]
fn nest_bird_t2_refresh_uses_tier_ratio() {
    // Hand-derived from frozen buff_NestBird_down (T2 = -0.21 -> 21/100):
    // Ultimates at 0 and 10s leave one refreshed instance expiring at 30s.
    let mut req = request_with_categories_tier(2, 21, 100, &[(0, "Q"), (10_000_000, "Q")]);
    req.relative_duration_us = 25_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let marks = warning_gaze_marks(&report);
    assert_eq!(marks.len(), 1);
    assert_eq!(
        marks[0]["stat_modifiers"]["outgoing_damage_reduction"],
        json!("21/100")
    );
    assert_eq!(
        marks[0]["duration"],
        json!({"kind": "timed", "duration_us": 20_000_000, "expires_at_us": 30_000_000})
    );
}
