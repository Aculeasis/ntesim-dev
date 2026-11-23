use super::*;
use crate::simulation::{ExactFraction, GameActorConfig};
use serde_json::json;
use std::collections::BTreeMap;

fn t1_proc() -> GearProcConfig {
    GearProcConfig::YurenHp {
        hp_bonus: Ratio {
            numerator: 1,
            denominator: 10,
        },
        shield_bonus: Ratio {
            numerator: 1,
            denominator: 10,
        },
    }
}

fn t5_proc() -> GearProcConfig {
    GearProcConfig::YurenHp {
        hp_bonus: Ratio {
            numerator: 9,
            denominator: 50,
        },
        shield_bonus: Ratio {
            numerator: 9,
            denominator: 50,
        },
    }
}

fn projection_tables() -> BTreeMap<String, serde_json::Value> {
    super::support::weapon_projection_tables()
}

fn projection_actor() -> GameActorConfig {
    serde_json::from_value(json!({
        "slot":0,"character_id":"character:1033","initial_hp":1,"initial_hp_max":1,"initial_energy":0,
        "energy_capacity":100,"weapon_id":"weapon:fork_yuren","weapon_tier":1,
        "static_stats":{"base_atk":1,"atk":"1/1","hp":1,"def":1,
        "crit_rate":"0/1","crit_damage_bonus":"0/1"}
    }))
    .unwrap()
}

fn damage_request() -> crate::simulation::GameScenarioRequest {
    let mut req = research_request();
    req.stochastic_crit = None;
    let actor = &mut req.actors[0];
    actor.character_id = "character:1033".into();
    actor.weapon_id = "weapon:fork_yuren".into();
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
fn yuren_projection_binds_five_tiers_with_description_placeholder_mapping() {
    let tables = projection_tables();
    let mut actor = projection_actor();
    for tier in 1..=5u32 {
        actor.weapon_tier = tier;
        actor.gear_procs.clear();
        let report = crate::weapons::yuren::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap();
        let proc = &report["compiled_proc"];
        assert_eq!(proc["kind"], "yuren_hp");
        let expected = ExactFraction::new(u128::from(8 + 2 * tier), 100).unwrap();
        for key in ["hp_bonus", "shield_bonus"] {
            let bound: Ratio = serde_json::from_value(proc[key].clone()).unwrap();
            assert_eq!(
                ExactFraction::from_ratio(bound).unwrap(),
                expected,
                "tier {tier} {key}"
            );
        }
        assert_eq!(
            report["native_description"]["localized"],
            json!(
                "Increases HP by <lv>{0}</>.\r\nStrengthens shields by <lv>{1}</> while the wearer's HP is above 50%."
            )
        );
        assert_eq!(
            report["scope"],
            crate::weapons::yuren::HYPOTHESIS_ID.description()
        );
        assert!(report["runtime_execution_validated"].as_bool() == Some(true));
        assert_eq!(actor.gear_procs.len(), 1);
    }
}

#[test]
fn yuren_hp_parameter_binds_by_name_not_datalist_position() {
    // The DataList order is [shield, HP]; a positional read would swap the
    // fields. Mutating only the HP curve proves name-based binding.
    let mut tables = projection_tables();
    tables.get_mut("CT_ForkBuff").unwrap()["rows"]["buff_bowenchanggui_HP"]["Keys"][0]["Value"] =
        json!(0.99);
    let mut actor = projection_actor();
    let report =
        crate::weapons::yuren::projection::project(&tables, 1, "ntesim-current", &mut actor, &[])
            .unwrap();
    let proc = &report["compiled_proc"];
    let hp: Ratio = serde_json::from_value(proc["hp_bonus"].clone()).unwrap();
    let shield: Ratio = serde_json::from_value(proc["shield_bonus"].clone()).unwrap();
    assert_eq!(
        ExactFraction::from_ratio(hp).unwrap(),
        ExactFraction::new(99, 100).unwrap()
    );
    assert_eq!(
        ExactFraction::from_ratio(shield).unwrap(),
        ExactFraction::new(1, 10).unwrap()
    );
}

#[test]
fn yuren_projection_rejects_group_mismatch_and_changed_units() {
    let tables = projection_tables();
    let mut actor = projection_actor();
    actor.character_id = "character:1003".into();
    assert!(
        crate::weapons::yuren::projection::project(&tables, 1, "ntesim-current", &mut actor, &[])
            .unwrap_err()
            .contains("incompatible")
    );
    let mut changed = tables.clone();
    changed.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_yuren_1"]
        ["DataList"][0]["bIsPercent"] = json!(false);
    let mut unit_actor = projection_actor();
    assert!(
        crate::weapons::yuren::projection::project(
            &changed,
            1,
            "ntesim-current",
            &mut unit_actor,
            &[]
        )
        .unwrap_err()
        .contains("unit hint")
    );
}

#[test]
fn yuren_config_rejects_foreign_weapon() {
    let mut req = damage_request();
    req.actors[0].weapon_id = "weapon:fork_Whale".into();
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires equipped")
    );
}

#[test]
fn yuren_damage_hook_leaves_damage_untouched() {
    let req = damage_request();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(
        report["hit_receipts"][0]["non_crit_damage"],
        json!("1000/1")
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
fn yuren_shield_bonus_uses_exact_living_grant_time_wearer_gate() {
    let actor = GameActorConfig {
        gear_procs: vec![t1_proc()],
        ..projection_actor()
    };
    for (current_hp, max_hp, expected) in [
        (0, 1000, ExactFraction::new(0, 1).unwrap()),
        (500, 1000, ExactFraction::new(0, 1).unwrap()),
        (501, 1000, ExactFraction::new(1, 10).unwrap()),
        (1000, 2000, ExactFraction::new(0, 1).unwrap()),
        (1001, 2000, ExactFraction::new(1, 10).unwrap()),
        (1000, 2001, ExactFraction::new(0, 1).unwrap()),
        (1001, 2001, ExactFraction::new(1, 10).unwrap()),
        (u64::MAX / 2, u64::MAX, ExactFraction::new(0, 1).unwrap()),
        (
            u64::MAX / 2 + 1,
            u64::MAX,
            ExactFraction::new(1, 10).unwrap(),
        ),
    ] {
        assert_eq!(
            crate::weapons::yuren::grant_time_shield_bonus(&actor, current_hp, max_hp, 0, 0)
                .unwrap(),
            expected
        );
    }
    assert!(
        crate::weapons::yuren::grant_time_shield_bonus(&actor, 501, 1000, 0, 1)
            .unwrap_err()
            .contains("foreign recipient")
    );
}

#[test]
fn yuren_shield_bonus_t5_gate_is_exact() {
    // T5 frozen ratio is 9/50; the grant gate stays the exact 2*HP > MaxHP
    // comparison at every tier, so HP500 (2*500 == 1000) still rejects.
    let actor = GameActorConfig {
        gear_procs: vec![t5_proc()],
        ..projection_actor()
    };
    assert_eq!(
        crate::weapons::yuren::grant_time_shield_bonus(&actor, 501, 1000, 0, 0).unwrap(),
        ExactFraction::new(9, 50).unwrap()
    );
    assert_eq!(
        crate::weapons::yuren::grant_time_shield_bonus(&actor, 500, 1000, 0, 0).unwrap(),
        ExactFraction::new(0, 1).unwrap()
    );
}

#[test]
fn yuren_shield_bonus_rejects_zero_effective_max_hp() {
    let actor = GameActorConfig {
        gear_procs: vec![t1_proc()],
        ..projection_actor()
    };
    assert!(
        crate::weapons::yuren::grant_time_shield_bonus(&actor, 501, 0, 0, 0)
            .unwrap_err()
            .contains("zero effective Max HP")
    );
}

#[test]
fn yuren_native_self_grant_settles_with_grant_time_bonus() {
    // Base capacity 1800 is the Lv1 Adler Skill grant at DEF1000 with no other
    // ShieldEfficiency sources. Hand-derived: T1 at HP501 -> 1800 * 11/10 = 1980,
    // T1 at HP500 -> 1800, T5 at HP501 -> 1800 * 59/50 = 2124.
    for (proc, current_hp, expected) in [
        (t1_proc(), 501u64, 1980u64),
        (t1_proc(), 500, 1800),
        (t5_proc(), 501, 2124),
    ] {
        let mut req = super::adler_actions::request();
        req.actors[0].static_stats.base_hp = Some(1000);
        req.actors[0].initial_hp_max = 1000;
        req.actors[0].initial_hp = current_hp;
        req.actors[0].weapon_id = "weapon:fork_yuren".into();
        req.actors[0].gear_procs = vec![proc];
        req.actors[0].static_stats.base_def = Some(1000);
        req.actors[0].static_stats.def = 1000;
        let mut state = crate::simulation::state::GameSimulatorState::new(&req);
        super::adler_shields::settle_adler_callback(&req, &mut state, 0, 0, 1800);
        assert_eq!(
            state.actors[0].shields[0].max_hp, expected,
            "hp {current_hp}"
        );
    }
}
