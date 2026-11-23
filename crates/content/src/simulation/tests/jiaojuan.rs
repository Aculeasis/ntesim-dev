use super::*;
use crate::runtime_api::HitModifiers;
use crate::simulation::state::GameSimulatorState;
use ntesim_core::combat::{BreakDamageRequest, NegativeResistanceModel, break_damage};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

fn t1_proc() -> GearProcConfig {
    GearProcConfig::JiaojuanBreak {
        break_intensity_bonus: Ratio {
            numerator: 48,
            denominator: 1,
        },
        broken_damage_bonus: Ratio {
            numerator: 1,
            denominator: 10,
        },
    }
}

fn projection_tables() -> std::collections::BTreeMap<String, serde_json::Value> {
    super::support::weapon_projection_tables()
}

#[test]
fn jiaojuan_projection_binds_five_curves_rejects_group_and_unit_changes() {
    let tables = projection_tables();
    let req = research_request();
    let mut actor = req.actors[0].clone();
    actor.character_id = "character:1021".into();
    actor.weapon_id = "weapon:fork_jiaojuan".into();
    actor.gear_procs.clear();
    for tier in 1..=5u32 {
        actor.weapon_tier = tier;
        let report = crate::weapons::jiaojuan::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            if tier == 1 { &[] } else { &[0] },
        )
        .unwrap();
        let proc = &report["compiled_proc"];
        let ratio_of = |key: &str| {
            ExactFraction::from_ratio(serde_json::from_value(proc[key].clone()).unwrap()).unwrap()
        };
        let bi = ExactFraction::new(u128::from(39 + 9 * tier), 1).unwrap();
        let dmg = ExactFraction::new(u128::from(8 + 2 * tier), 100).unwrap();
        assert_eq!(
            ratio_of("break_intensity_bonus"),
            bi,
            "tier {tier} BI curve"
        );
        assert_eq!(
            ratio_of("broken_damage_bonus"),
            dmg,
            "tier {tier} damage curve"
        );
        assert_eq!(actor.gear_procs.len(), 1);
        assert_eq!(report["replaced_proc"].is_null(), tier == 1);
        // Both the wearer-only Break Intensity and the target's live Broken
        // tag are consumed in continuing native Break/ordinary damage paths.
        assert_eq!(report["runtime_execution_validated"], true);
        assert_eq!(
            report["native_description"]["localized"],
            json!(
                "Increases Break Intensity by <lv>{0}</>.\nIncreases DMG by <lv>{1}</> against Broken units."
            )
        );
    }

    let mut cross_group = req.actors[0].clone();
    cross_group.character_id = "character:1072".into();
    cross_group.weapon_id = "weapon:fork_jiaojuan".into();
    cross_group.weapon_tier = 1;
    cross_group.gear_procs.clear();
    assert!(
        crate::weapons::jiaojuan::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut cross_group,
            &[]
        )
        .unwrap_err()
        .contains("incompatible")
    );

    let mut mutated = tables.clone();
    let row = &mut mutated.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_jiaojuan_5"]
        ["DataList"];
    row[1]["bIsPercent"] = json!(false);
    let mut unit_actor = actor.clone();
    unit_actor.weapon_tier = 5;
    unit_actor.gear_procs.clear();
    assert!(
        crate::weapons::jiaojuan::projection::project(
            &mutated,
            1,
            "ntesim-current",
            &mut unit_actor,
            &[]
        )
        .unwrap_err()
        .contains("unit hint")
    );
}

fn apply_broken(
    buffs: &mut ntesim_core::buffs::dispatch::BuffManager,
    target_index: usize,
    duration_us: u64,
) {
    crate::simulation::state::apply_target_tagged_status(
        buffs,
        crate::simulation::state::TargetStatusDeclaration {
            source_actor: 0,
            source: "test".into(),
            target_index,
            id: "test-broken".into(),
            now_us: 0,
            duration_us,
            reductions: BTreeMap::new(),
        },
        BTreeSet::from([crate::weapons::jiaojuan::BROKEN_STATUS_TAG.into()]),
        1,
        ntesim_core::buffs::dispatch::RefreshPolicy::Reset,
        ntesim_core::buffs::dispatch::DecayPolicy::AllAtOnce,
    )
    .unwrap();
}

fn zone_modifiers(
    req: &crate::simulation::GameScenarioRequest,
    state: &GameSimulatorState,
    target_index: usize,
) -> HitModifiers {
    super::support::gear_zone_modifiers(req, state, 1, target_index)
}

fn damage_request() -> crate::simulation::GameScenarioRequest {
    let mut req = research_request();
    req.stochastic_crit = None;
    let actor = &mut req.actors[0];
    actor.character_id = "character:1021".into();
    actor.weapon_id = "weapon:fork_jiaojuan".into();
    actor.weapon_tier = 1;
    actor.gear_procs = vec![t1_proc()];
    actor.static_stats.base_atk = 1000;
    actor.static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    actor.static_stats.crit_rate = Ratio::zero();
    actor.static_stats.damage_bonus = Ratio::zero();
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.actions.truncate(1);
    req.action_metadata
        .retain(|index, _| *index < req.actions.len());
    req
}

#[test]
fn jiaojuan_broken_damage_bonus_is_additive_and_broken_gated() {
    let mut req = damage_request();
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    let mut state = GameSimulatorState::new(&req);
    apply_broken(&mut state.buffs, 1, 30_000_000);

    let plain = zone_modifiers(&req, &state, 0);
    assert!(
        plain.damage_bonus.is_zero(),
        "no Broken damage bonus against an unbroken target"
    );
    assert!(
        plain.atk_percent.is_zero(),
        "Film - MANISH grants no ATK bonus"
    );

    let broken = zone_modifiers(&req, &state, 1);
    assert_eq!(
        broken.damage_bonus,
        ExactFraction::new(1, 10).unwrap(),
        "Broken damage bonus joins the additive damage zone"
    );
    assert!(
        broken.atk_percent.is_zero(),
        "Film - MANISH grants no ATK bonus even when Broken applies"
    );
}

#[test]
fn jiaojuan_unbroken_scenario_matches_baseline() {
    let req = damage_request();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(
        report["hit_receipts"][0]["non_crit_damage"],
        json!("1000/1"),
        "unbroken target sees no damage change from the proc"
    );

    let mut bare = req.clone();
    bare.actors[0].gear_procs.clear();
    let baseline = run_game_scenario(&bare, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        baseline["hit_receipts"][0]["non_crit_damage"],
        report["hit_receipts"][0]["non_crit_damage"]
    );
}

#[test]
fn jiaojuan_validate_config_rejects_wrong_weapon_and_bad_ratios() {
    let mut req = damage_request();
    req.actors[0].weapon_id = "weapon:fork_Crowbar".into();
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires equipped")
    );

    let bad_ratio = damage_request();
    let mut bad_actor = bad_ratio.actors[0].clone();
    bad_actor.gear_procs = vec![GearProcConfig::JiaojuanBreak {
        break_intensity_bonus: Ratio {
            numerator: 1,
            denominator: 0,
        },
        broken_damage_bonus: Ratio {
            numerator: 1,
            denominator: 10,
        },
    }];
    let mut bad_req = bad_ratio.clone();
    bad_req.actors[0] = bad_actor;
    let bad_error = run_game_scenario(&bad_req, &ExecutionLimits::default()).unwrap_err();
    assert!(
        bad_error.contains("denominator") || bad_error.contains("ratio"),
        "unexpected error: {bad_error}"
    );

    let mut duplicate = damage_request();
    let extra = duplicate.actors[0].gear_procs[0].clone();
    duplicate.actors[0].gear_procs.push(extra);
    assert!(
        run_game_scenario(&duplicate, &ExecutionLimits::default())
            .unwrap_err()
            .contains("duplicates")
    );
}

#[test]
fn jiaojuan_break_intensity_joins_break_damage_zone_additively() {
    let base = BreakDamageRequest {
        base_break: 91.0,
        intensity: 300.0,
        original_cap: 45.0,
        attacker_level: 1,
        enemy_defense: 654.0,
        resistance: 0.20,
        extra_bonus: 0.0,
        shred: 0.0,
        ignore: 0.0,
        special: 1.0,
        negative_res_model: NegativeResistanceModel::StandardRational,
    };
    let plain = break_damage(&base).unwrap();
    let staged = break_damage(&BreakDamageRequest {
        intensity: 348.0,
        ..base
    })
    .unwrap();
    let expected_zone_ratio = (1.0 + 348.0 / 300.0) / (1.0 + 300.0 / 300.0);
    assert!((staged / plain - expected_zone_ratio).abs() < 1e-12);
    assert!((expected_zone_ratio - 1.08).abs() < 1e-12);
}

#[test]
fn runtime_recorder_records_and_validates_the_jiaojuan_hypothesis() {
    let mut req = damage_request();
    let mut reached = BTreeMap::new();
    crate::runtime_api::record_hypothesis(
        &req,
        &mut reached,
        crate::weapons::jiaojuan::HYPOTHESIS_ID,
        crate::weapons::jiaojuan::HYPOTHESIS_ID.description(),
    )
    .unwrap();
    assert_eq!(
        reached[crate::weapons::jiaojuan::HYPOTHESIS_ID.as_str()],
        crate::weapons::jiaojuan::HYPOTHESIS_ID.description()
    );
    req.model_assumptions.insert(
        crate::weapons::jiaojuan::HYPOTHESIS_ID.into(),
        "conflicting description".into(),
    );
    let rejected = crate::runtime_api::record_hypothesis(
        &req,
        &mut reached,
        crate::weapons::jiaojuan::HYPOTHESIS_ID,
        crate::weapons::jiaojuan::HYPOTHESIS_ID.description(),
    )
    .unwrap_err();
    assert!(rejected.contains("conflicting reached model hypothesis"));
}

#[test]
fn jiaojuan_zone_math_hand_derived_t1_t3_t5() {
    fn gcd(mut a: u128, mut b: u128) -> u128 {
        while b != 0 {
            let rest = a % b;
            a = b;
            b = rest;
        }
        a
    }
    fn reduced(n: u128, d: u128) -> (u128, u128) {
        let g = gcd(n, d);
        (n / g, d / g)
    }
    // Break-zone ratio against the BI=300 baseline:
    // (1 + (300 + BI)/300) / (1 + 300/300) = (600 + BI)/600.
    assert_eq!(reduced(600 + 48, 600), (27, 25));
    assert_ne!(reduced(600 + 48, 600), (37, 25));
    assert_eq!(reduced(600 + 66, 600), (111, 100));
    assert_ne!(reduced(600 + 66, 600), (83, 50));
    assert_eq!(reduced(600 + 84, 600), (57, 50));
    assert_ne!(reduced(600 + 84, 600), (46, 25));
}

#[test]
fn jiaojuan_broken_bonus_joins_additive_zone() {
    let req = damage_request();
    let mut state = GameSimulatorState::new(&req);
    apply_broken(&mut state.buffs, 0, 30_000_000);
    let modifiers = zone_modifiers(&req, &state, 0);
    // The hit pipeline seeds the zone with the static bonus and adds gear
    // bonuses on top; addition commutes, so the sum is the zone value.
    let total = modifiers
        .damage_bonus
        .add(ExactFraction::new(1, 2).unwrap())
        .unwrap();
    assert_eq!(total, ExactFraction::new(3, 5).unwrap());
    let m_bonus = ExactFraction::new(1, 1).unwrap().add(total).unwrap();
    assert_eq!(m_bonus, ExactFraction::new(8, 5).unwrap());
    assert_ne!(m_bonus, ExactFraction::new(33, 20).unwrap());
}

#[test]
fn jiaojuan_broken_scenario_end_to_end() {
    let mut req = super::edgar_actions::request();
    req.relative_duration_us = 2_500_100;
    req.actors[0].weapon_id = "weapon:fork_jiaojuan".into();
    req.actors[0].weapon_tier = 1;
    req.actors[0].gear_procs = vec![t1_proc()];
    req.actors[0].talent_levels = Some([8, 8, 8]);
    req.targets[0].hp_current = 10_000_000;
    req.targets[0].hp_max = 10_000_000;
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.targets[0].break_profile = Some(crate::simulation::GameTargetBreakProfileConfig {
        capacity: Ratio::one(),
        current_gauge: Ratio {
            numerator: 1,
            denominator: 2,
        },
        broken_duration_us: 2_000_000,
        discord_fraction: None,
    });
    req.actions = [0, 300_000, 600_000, 900_000, 2_400_000]
        .into_iter()
        .map(|time| {
            super::edgar_actions::make_action(
                time,
                0,
                "GA_Edgar_Melee",
                "GE_Player_Edgar_Melee1_Damage",
                0,
                0,
            )
        })
        .collect();

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true, "{report}");
    let broken_events = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "target_broken")
        .collect::<Vec<_>>();
    assert_eq!(broken_events.len(), 1, "{report}");
    assert_eq!(broken_events[0]["time_us"], 300_100);
    let recoveries = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "target_break_recovered")
        .collect::<Vec<_>>();
    assert_eq!(recoveries.len(), 1, "{report}");
    assert_eq!(recoveries[0]["time_us"], 2_300_100);

    let break_hits = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit.get("break_contribution").is_some())
        .collect::<Vec<_>>();
    assert_eq!(break_hits.len(), 1);
    assert_eq!(
        break_hits[0]["break_contribution"]["intensity"],
        json!("48/1")
    );
    assert_eq!(break_hits[0]["damage"], json!("34829/25"));
    let hit_receipts = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit.get("break_contribution").is_none())
        .collect::<Vec<_>>();
    let packets = hit_receipts
        .iter()
        .map(|hit| hit["damage_effect_id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        packets,
        [
            "GE_Player_Edgar_Melee1_Damage",
            "GE_Player_Edgar_Melee2_Damage",
            "GE_Player_Edgar_Melee3_Damage",
            "GE_Player_Edgar_Melee3_Damage",
            "GE_Player_Edgar_Melee4_Damage",
            "GE_Player_Edgar_Melee5_Damage",
        ]
    );
    // Frozen level-eight attack coefficients against zero DEF/RES give
    // 530, 1075, 668, 668, 1428, 1196 at 1000 ATK.
    // The T1 +10% additive Broken zone changes only packets delivered while
    // the profile-produced Broken tag is active.
    let expected = [
        json!("530/1"),
        json!("1075/1"),
        json!("3674/5"),
        json!("3674/5"),
        json!("7854/5"),
        json!("1196/1"),
    ];
    assert_eq!(
        hit_receipts
            .iter()
            .map(|hit| hit["non_crit_damage"].clone())
            .collect::<Vec<_>>(),
        expected
    );

    let mut bare = req.clone();
    bare.actors[0].gear_procs.clear();
    let baseline = run_game_scenario(&bare, &ExecutionLimits::default()).unwrap();
    let baseline_hits = baseline["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit.get("break_contribution").is_none())
        .collect::<Vec<_>>();
    assert_eq!(baseline_hits.len(), hit_receipts.len());
    for index in [0, 1, 5] {
        assert_eq!(
            hit_receipts[index]["non_crit_damage"], baseline_hits[index]["non_crit_damage"],
            "no Jiaojuan bonus before Break or after recovery at packet {index}"
        );
    }
    for index in [2, 3, 4] {
        let actual: ExactFraction =
            serde_json::from_value(hit_receipts[index]["non_crit_damage"].clone()).unwrap();
        let base: ExactFraction =
            serde_json::from_value(baseline_hits[index]["non_crit_damage"].clone()).unwrap();
        assert_eq!(
            actual,
            base.mul(ExactFraction::new(11, 10).unwrap()).unwrap()
        );
    }
}

#[test]
fn jiaojuan_unbal_unit_hint_falsifier() {
    let tables = projection_tables();
    let mut mutated = tables.clone();
    let row = &mut mutated.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_jiaojuan_5"]
        ["DataList"];
    row[0]["bIsPercent"] = json!(true);
    let req = research_request();
    let mut actor = req.actors[0].clone();
    actor.character_id = "character:1021".into();
    actor.weapon_id = "weapon:fork_jiaojuan".into();
    actor.weapon_tier = 5;
    actor.gear_procs.clear();
    assert!(
        crate::weapons::jiaojuan::projection::project(
            &mutated,
            1,
            "ntesim-current",
            &mut actor,
            &[]
        )
        .unwrap_err()
        .contains("unit hint")
    );
}
