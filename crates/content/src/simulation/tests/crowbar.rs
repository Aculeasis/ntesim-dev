use super::*;
use crate::simulation::metadata::InputActionRole;
use ntesim_core::combat::{BreakDamageRequest, NegativeResistanceModel, break_damage};
use serde_json::json;
use std::collections::BTreeMap;

fn fixture() -> GameScenarioRequest {
    super::redirect_weapon_fixture(
        "weapon:fork_Crowbar",
        vec![GearProcConfig::CrowbarBreak {
            break_intensity_bonus: Ratio {
                numerator: 90,
                denominator: 1,
            },
            duration_us: 50_000_000,
        }],
        Ratio {
            numerator: 1000,
            denominator: 1,
        },
    )
}

fn append(req: &mut GameScenarioRequest, time: u64, role: InputActionRole, delay: u64) {
    super::append_synthetic_cast(req, time, role, delay, None);
}

fn crowbar_buffs(report: &serde_json::Value) -> Vec<serde_json::Value> {
    crate::simulation::tests::support::final_buffs(report)
        .into_iter()
        .filter(|buff| buff["id"] == crate::weapons::crowbar::BUFF_ID)
        .collect()
}

fn projection_tables() -> BTreeMap<String, serde_json::Value> {
    super::support::weapon_projection_tables()
}

fn projection_actor() -> GameActorConfig {
    serde_json::from_value(json!({
        "slot":0,"character_id":"character:1052","initial_hp":1,"initial_hp_max":1,"initial_energy":0,
        "energy_capacity":100,"weapon_id":"weapon:fork_Crowbar","weapon_tier":1,
        "static_stats":{"base_atk":1,"atk":"1/1","hp":1,"def":1,
        "crit_rate":"0/1","crit_damage_bonus":"0/1"}
    }))
    .unwrap()
}

#[test]
fn crowbar_projection_binds_five_refinement_tiers() {
    let tables = projection_tables();
    let expected_bonus = [90, 108, 126, 144, 162];
    let mut actor = projection_actor();
    for (tier, bonus) in (1..=5).zip(expected_bonus) {
        actor.weapon_tier = tier as u32;
        actor.gear_procs.clear();
        let report = crate::weapons::crowbar::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap();
        let proc = &report["compiled_proc"];
        assert_eq!(proc["kind"], "crowbar_break");
        assert_eq!(
            proc["break_intensity_bonus"],
            json!(format!("{}/{}", bonus, 1))
        );
        assert_eq!(proc["duration_us"], 50_000_000);
        assert_eq!(report["native_parameter_report"]["parameter_count"], 2);
        // Continuing native Redirect cycles exercise live Break HP capture.
        assert_eq!(report["runtime_execution_validated"], true);
        assert_eq!(actor.gear_procs.len(), 1);
    }
}

#[test]
fn crowbar_projection_rejects_group_mismatch_and_changed_units() {
    let tables = projection_tables();
    let mut actor = projection_actor();
    actor.character_id = "character:1003".into();
    assert!(
        crate::weapons::crowbar::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap_err()
        .contains("group is incompatible")
    );
    actor.character_id = "character:1052".into();
    let mut changed = tables.clone();
    changed.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]
        ["upgradestar_pack_fork_Crowbar_1"]["DataList"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["NameID"] == "buff_Crowbar_Unbal")
        .unwrap()["bIsPercent"] = json!(true);
    assert!(
        crate::weapons::crowbar::projection::project(
            &changed,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap_err()
        .contains("unit hint")
    );
}

#[test]
fn crowbar_validate_config_rejects_mismatches() {
    let limits = ExecutionLimits::default();
    let mut actor = projection_actor();
    let valid = GearProcConfig::CrowbarBreak {
        break_intensity_bonus: Ratio {
            numerator: 90,
            denominator: 1,
        },
        duration_us: 50_000_000,
    };
    let ctx = crate::gear::validation::ConfigContext {
        actor: &actor,
        actor_index: 0,
        limits: &limits,
    };
    let validation = crate::weapons::crowbar::validate_config(&valid, &ctx).unwrap();
    assert_eq!(validation.kind, "crowbar");
    assert_eq!(validation.duration_us, 50_000_000);

    actor.weapon_id = "weapon:fork_Worldrain".into();
    let ctx = crate::gear::validation::ConfigContext {
        actor: &actor,
        actor_index: 0,
        limits: &limits,
    };
    let err = crate::weapons::crowbar::validate_config(&valid, &ctx)
        .err()
        .expect("expected weapon mismatch rejection");
    assert!(err.contains("weapon:fork_Crowbar"));

    actor.weapon_id = "weapon:fork_Crowbar".into();
    let ctx = crate::gear::validation::ConfigContext {
        actor: &actor,
        actor_index: 0,
        limits: &limits,
    };
    let zero_duration = GearProcConfig::CrowbarBreak {
        break_intensity_bonus: Ratio {
            numerator: 90,
            denominator: 1,
        },
        duration_us: 0,
    };
    let err = crate::weapons::crowbar::validate_config(&zero_duration, &ctx)
        .err()
        .expect("expected zero duration rejection");
    assert!(err.contains("positive duration"));
    let bad_ratio = GearProcConfig::CrowbarBreak {
        break_intensity_bonus: Ratio {
            numerator: 90,
            denominator: 0,
        },
        duration_us: 50_000_000,
    };
    assert!(crate::weapons::crowbar::validate_config(&bad_ratio, &ctx).is_err());
}

#[test]
fn crowbar_buff_granted_only_by_redirect_skill() {
    let mut req = fixture();
    append(&mut req, 0, InputActionRole::Normal, 0);
    append(&mut req, 1_000_000, InputActionRole::RedirectSkill, 0);
    append(&mut req, 2_000_000, InputActionRole::Ultimate, 0);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let buffs = crowbar_buffs(&report);
    assert_eq!(buffs.len(), 1);
    let buff = &buffs[0];
    assert_eq!(buff["source"], "weapon:fork_Crowbar");
    assert_eq!(buff["owner_actor"], 0);
    assert_eq!(buff["stacks"], 1);
    assert_eq!(buff["stat_modifiers"]["break_intensity"], json!("90/1"));
    assert_eq!(buff["duration"]["expires_at_us"], 51_000_000);
    assert_eq!(
        report["resolved_model_assumptions"][crate::weapons::crowbar::HYPOTHESIS_ID.as_str()],
        crate::weapons::crowbar::HYPOTHESIS_ID.description()
    );
}

#[test]
fn crowbar_recast_while_active_neither_stacks_nor_refreshes() {
    let mut req = fixture();
    append(&mut req, 0, InputActionRole::RedirectSkill, 0);
    append(&mut req, 30_000_000, InputActionRole::RedirectSkill, 0);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let buffs = crowbar_buffs(&report);
    assert_eq!(buffs.len(), 1);
    assert_eq!(buffs[0]["stacks"], 1);
    assert_eq!(buffs[0]["duration"]["expires_at_us"], 50_000_000);
}

#[test]
fn crowbar_buff_expires_and_regrants_after_expiry() {
    let mut req = fixture();
    req.relative_duration_us = 60_000_000;
    append(&mut req, 0, InputActionRole::RedirectSkill, 0);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(crowbar_buffs(&report).len(), 0);

    let mut req = fixture();
    req.relative_duration_us = 70_000_000;
    append(&mut req, 0, InputActionRole::RedirectSkill, 0);
    append(&mut req, 55_000_000, InputActionRole::RedirectSkill, 0);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let buffs = crowbar_buffs(&report);
    assert_eq!(buffs.len(), 1);
    assert_eq!(buffs[0]["duration"]["expires_at_us"], 105_000_000);
}

#[test]
fn crowbar_break_intensity_joins_break_damage_zone_additively() {
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
    let boosted = break_damage(&BreakDamageRequest {
        intensity: 390.0,
        ..base
    })
    .unwrap();
    let expected_zone_ratio = (1.0 + 390.0 / 300.0) / (1.0 + 300.0 / 300.0);
    assert!((boosted / plain - expected_zone_ratio).abs() < 1e-12);
    assert!((expected_zone_ratio - 1.15).abs() < 1e-12);
    assert!((boosted / plain - 1.30).abs() > 1e-9);
}

#[test]
fn crowbar_zone_math_t3_t5_hand_derived() {
    // Hand-derived from frozen buff_Crowbar_Unbal (T3 = 126, T5 = 162 points):
    // the bonus points join the break damage zone additively as
    // 1 + BI/300 + break_bonus_sum (COMBAT_FORMULA_CONTRACT section 5).
    // T3: (1 + 426/300) / (1 + 300/300) = 2.42/2 = 1.21.
    // T5: (1 + 462/300) / (1 + 300/300) = 2.54/2 = 1.27.
    // A multiplicative misreading ("+126%") would give 1.42 and 1.54 instead.
    // This independent core oracle complements the continuing content capture.
    let base = || BreakDamageRequest {
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
    let plain = break_damage(&base()).unwrap();
    let t3 = break_damage(&BreakDamageRequest {
        intensity: 426.0,
        ..base()
    })
    .unwrap();
    assert!((t3 / plain - 1.21).abs() < 1e-12);
    assert!((t3 / plain - 1.42).abs() > 1e-9);
    let t5 = break_damage(&BreakDamageRequest {
        intensity: 462.0,
        ..base()
    })
    .unwrap();
    assert!((t5 / plain - 1.27).abs() < 1e-12);
    assert!((t5 / plain - 1.54).abs() > 1e-9);
}
