use super::*;
use ntesim_core::combat::{BreakDamageRequest, NegativeResistanceModel, break_damage};
use serde_json::json;
use std::collections::BTreeMap;

fn projection_tables() -> BTreeMap<String, serde_json::Value> {
    super::support::weapon_projection_tables()
}

fn projection_actor() -> GameActorConfig {
    serde_json::from_value(json!({
        "slot":0,"character_id":"character:1039","initial_hp":1,"initial_hp_max":1,"initial_energy":0,
        "energy_capacity":100,"weapon_id":"weapon:fork_tuansanlang","weapon_tier":1,
        "static_stats":{"base_atk":1,"atk":"1/1","hp":1,"def":1,
        "crit_rate":"0/1","crit_damage_bonus":"0/1"}
    }))
    .unwrap()
}

fn tuansanlang_proc(bonus: u64) -> GearProcConfig {
    GearProcConfig::TuansanlangTeam {
        break_intensity_bonus: Ratio {
            numerator: bonus,
            denominator: 1,
        },
    }
}

#[test]
fn tuansanlang_projection_binds_five_refinement_tiers() {
    let tables = projection_tables();
    let expected_bonus = [70, 84, 98, 112, 126];
    let mut actor = projection_actor();
    for (tier, bonus) in (1..=5).zip(expected_bonus) {
        actor.weapon_tier = tier as u32;
        actor.gear_procs.clear();
        let report = crate::weapons::tuansanlang::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap();
        let proc = &report["compiled_proc"];
        assert_eq!(proc["kind"], "tuansanlang_team");
        assert_eq!(
            proc["break_intensity_bonus"],
            json!(format!("{}/{}", bonus, 1))
        );
        assert_eq!(report["native_parameter_report"]["parameter_count"], 1);
        assert_eq!(actor.gear_procs.len(), 1);
    }
}

#[test]
fn tuansanlang_projection_rejects_group_mismatch_and_changed_units() {
    let tables = projection_tables();
    let mut actor = projection_actor();
    actor.character_id = "character:1052".into();
    assert!(
        crate::weapons::tuansanlang::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap_err()
        .contains("group is incompatible")
    );
    actor.character_id = "character:1039".into();
    let mut changed = tables.clone();
    changed.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]
        ["upgradestar_pack_fork_tuansanlang_1"]["DataList"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["NameID"] == "buff_tuansanlang_Unbal")
        .unwrap()["bIsPercent"] = json!(true);
    assert!(
        crate::weapons::tuansanlang::projection::project(
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
fn tuansanlang_validate_config_rejects_mismatches() {
    let limits = ExecutionLimits::default();
    let mut actor = projection_actor();
    let valid = tuansanlang_proc(70);
    let ctx = crate::gear::validation::ConfigContext {
        actor: &actor,
        actor_index: 0,
        limits: &limits,
    };
    let validation = crate::weapons::tuansanlang::validate_config(&valid, &ctx).unwrap();
    assert_eq!(validation.kind, "tuansanlang");

    actor.weapon_id = "weapon:fork_Crowbar".into();
    let ctx = crate::gear::validation::ConfigContext {
        actor: &actor,
        actor_index: 0,
        limits: &limits,
    };
    let err = crate::weapons::tuansanlang::validate_config(&valid, &ctx)
        .err()
        .expect("expected weapon mismatch rejection");
    assert!(err.contains("weapon:fork_tuansanlang"));

    actor.weapon_id = "weapon:fork_tuansanlang".into();
    let ctx = crate::gear::validation::ConfigContext {
        actor: &actor,
        actor_index: 0,
        limits: &limits,
    };
    let zero_bonus = tuansanlang_proc(0);
    let err = crate::weapons::tuansanlang::validate_config(&zero_bonus, &ctx)
        .err()
        .expect("expected zero bonus rejection");
    assert!(err.contains("positive"));
    let bad_ratio = GearProcConfig::TuansanlangTeam {
        break_intensity_bonus: Ratio {
            numerator: 70,
            denominator: 0,
        },
    };
    assert!(crate::weapons::tuansanlang::validate_config(&bad_ratio, &ctx).is_err());
}

#[test]
fn tuansanlang_zone_math_t1_t3_t5_hand_derived() {
    // Hand-derived from frozen buff_tuansanlang_Unbal (T1 = 70, T3 = 98, T5 = 126
    // points, bIsPercent=false): the bonus points join the break damage zone
    // additively as 1 + BI/300 + break_bonus_sum (COMBAT_FORMULA_CONTRACT section 5).
    // With baseline BI = 300 the zone ratio is (600 + B)/600:
    // T1: 67/60, T3: 349/300, T5: 121/100.
    // A multiplicative misreading ("+70%/+98%/+126% damage") would give
    // 1.70/1.98/1.26 instead.
    // Like the projection tests, this is core zone math, not the adapter:
    // break_intensity has no consumer in the content simulation.
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
    let t1 = break_damage(&BreakDamageRequest {
        intensity: 370.0,
        ..base()
    })
    .unwrap();
    assert!((t1 / plain - 67.0 / 60.0).abs() < 1e-12);
    assert!((t1 / plain - 1.70).abs() > 1e-9);
    let t3 = break_damage(&BreakDamageRequest {
        intensity: 398.0,
        ..base()
    })
    .unwrap();
    assert!((t3 / plain - 349.0 / 300.0).abs() < 1e-12);
    assert!((t3 / plain - 1.98).abs() > 1e-9);
    let t5 = break_damage(&BreakDamageRequest {
        intensity: 426.0,
        ..base()
    })
    .unwrap();
    assert!((t5 / plain - 121.0 / 100.0).abs() < 1e-12);
    assert!((t5 / plain - 1.26).abs() > 1e-9);
}
