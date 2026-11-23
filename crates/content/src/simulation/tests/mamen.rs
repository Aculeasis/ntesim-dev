use super::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn projection_tables() -> BTreeMap<String, serde_json::Value> {
    super::support::weapon_projection_tables()
}

fn projection_actor() -> GameActorConfig {
    serde_json::from_value(json!({
        "slot": 0, "character_id": "character:1003", "character_level": 80, "awakening": [],
        "purchased_skill_level": 8, "initial_hp": 1, "initial_hp_max": 1, "initial_energy": 100,
        "energy_capacity": 100, "weapon_id": "weapon:fork_mamen", "weapon_tier": 1,
        "static_stats": {
            "base_atk": 1, "atk": "1/1", "hp": 1, "def": 1,
            "crit_rate": "0/1",
            "crit_damage_bonus": "0/1"
        }
    }))
    .unwrap()
}

fn mamen_proc() -> GearProcConfig {
    GearProcConfig::MamenFons {
        cosmos_bonus_per_stack: Ratio {
            numerator: 1,
            denominator: 40,
        },
        fons_per_stack: 100_000,
        max_stacks: 10,
    }
}

#[test]
fn mamen_projection_binds_five_refinements_with_independent_oracles() {
    let tables = projection_tables();
    let item = ntesim_data::weapon_definitions::lookup(&tables, "fork_mamen")
        .unwrap()
        .1;
    assert_eq!(
        item["ElementData"]["ApplyGroupType"],
        "ECharacterGroupType::CHARACTER_GROUP_TYPE_FOUR"
    );
    assert_eq!(item["ElementData"]["ExclusiveCharacterIDArray"], json!([]));
    let mut actor = projection_actor();
    let expected = [(1u64, 40u64), (3, 100), (7, 200), (1, 25), (9, 200)];
    for (tier, (numerator, denominator)) in (1..=5u32).zip(expected) {
        actor.weapon_tier = tier;
        actor.gear_procs.clear();
        let report = crate::weapons::mamen::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap();
        assert_eq!(
            report["compiled_proc"]["cosmos_bonus_per_stack"],
            json!(format!("{}/{}", numerator, denominator)),
            "tier {tier} cosmos"
        );
        assert_eq!(report["compiled_proc"]["fons_per_stack"], 100_000);
        assert_eq!(report["compiled_proc"]["max_stacks"], 10);
        assert_eq!(report["native_parameter_report"]["parameter_count"], 2);
        assert_eq!(report["runtime_execution_validated"], true);
        assert_eq!(
            report["scope"],
            Value::String(crate::weapons::mamen::HYPOTHESIS_ID.description().into())
        );
        assert_eq!(
            report["passive_description"]["localized"],
            "Increases Cosmos DMG by <lv>{1}</> for every <lv>{0}</> Fons the wearer holds, up to <lv>10</> stacks."
        );
        assert!(report.get("fons_input").is_some());
        assert!(report["compiled_proc"].get("fons_held").is_none());
        let proc: GearProcConfig = serde_json::from_value(report["compiled_proc"].clone()).unwrap();
        assert!(matches!(proc, GearProcConfig::MamenFons { .. }));
        assert_eq!(actor.gear_procs.len(), 1);
    }
}

#[test]
fn mamen_projection_does_not_create_a_weapon_local_fons_balance() {
    let tables = projection_tables();
    let mut actor = projection_actor();
    actor.weapon_tier = 3;
    actor.gear_procs = vec![mamen_proc()];
    let report =
        crate::weapons::mamen::projection::project(&tables, 1, "ntesim-current", &mut actor, &[0])
            .unwrap();
    assert_eq!(
        report["compiled_proc"]["cosmos_bonus_per_stack"],
        json!("7/200")
    );
    assert_eq!(report["compiled_proc"]["fons_per_stack"], 100_000);
    assert_eq!(report["compiled_proc"]["max_stacks"], 10);
    assert!(report["compiled_proc"].get("fons_held").is_none());
    assert!(report["replaced_proc"].get("fons_held").is_none());
}

#[test]
fn mamen_projection_rejects_incompatible_group() {
    let tables = projection_tables();
    let mut actor = projection_actor();
    actor.character_id = "character:1019".into();
    assert!(
        crate::weapons::mamen::projection::project(&tables, 1, "ntesim-current", &mut actor, &[],)
            .unwrap_err()
            .contains("incompatible")
    );
    assert!(actor.gear_procs.is_empty());
}

#[test]
fn mamen_projection_rejects_flipped_unit_hint() {
    let mut tables = projection_tables();
    tables.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_mamen_1"]
        ["DataList"][1]["bIsPercent"] = json!(false);
    let mut actor = projection_actor();
    assert!(
        crate::weapons::mamen::projection::project(&tables, 1, "ntesim-current", &mut actor, &[],)
            .unwrap_err()
            .contains("unit hint")
    );
}

#[test]
fn mamen_validate_config_accepts_legal_proc_and_rejects_bad_inputs() {
    let limits = ExecutionLimits::default();
    let mut actor = research_request().actors[0].clone();
    actor.weapon_id = "weapon:fork_mamen".into();
    actor.gear_procs = vec![mamen_proc()];
    crate::gear::validate_config(&actor, 0, 40_000_000, &limits).unwrap();

    actor.weapon_id = "weapon:fork_Kite".into();
    assert!(
        crate::gear::validate_config(&actor, 0, 40_000_000, &limits)
            .unwrap_err()
            .contains("requires equipped")
    );

    actor.weapon_id = "weapon:fork_mamen".into();
    actor.gear_procs = vec![mamen_proc()];
    crate::gear::validate_config(&actor, 0, 40_000_000, &limits).unwrap();

    let mut zero_step = mamen_proc();
    if let GearProcConfig::MamenFons { fons_per_stack, .. } = &mut zero_step {
        *fons_per_stack = 0;
    }
    actor.gear_procs = vec![zero_step];
    assert!(
        crate::gear::validate_config(&actor, 0, 40_000_000, &limits)
            .unwrap_err()
            .contains("positive fons_per_stack")
    );

    let mut zero_cap = mamen_proc();
    if let GearProcConfig::MamenFons { max_stacks, .. } = &mut zero_cap {
        *max_stacks = 0;
    }
    actor.gear_procs = vec![zero_cap];
    assert!(
        crate::gear::validate_config(&actor, 0, 40_000_000, &limits)
            .unwrap_err()
            .contains("positive max_stacks")
    );

    let mut zero_bonus = mamen_proc();
    if let GearProcConfig::MamenFons {
        cosmos_bonus_per_stack,
        ..
    } = &mut zero_bonus
    {
        *cosmos_bonus_per_stack = Ratio {
            numerator: 0,
            denominator: 1,
        };
    }
    actor.gear_procs = vec![zero_bonus];
    assert!(
        crate::gear::validate_config(&actor, 0, 40_000_000, &limits)
            .unwrap_err()
            .contains("positive Cosmos bonus per stack")
    );
}

fn cosmos_fixture() -> GameScenarioRequest {
    let mut req = research_request();
    req.start_fons = Some(250_000);
    req.stochastic_crit = None;
    req.actors[0].weapon_id = "weapon:fork_mamen".into();
    req.actors[0].gear_procs = vec![mamen_proc()];
    req.actors[0].static_stats.damage_bonus = Ratio {
        numerator: 1,
        denominator: 10,
    };
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.targets[0].resistance = Ratio::zero();
    req.action_metadata.get_mut(&1).unwrap().damage_type_ex =
        Some("EExecutionDamageTypeEX::DAMAGE_TYPE_COSMOS".into());
    req
}

#[test]
fn mamen_uses_party_fons_for_wearer_only_across_a_real_cosmos_party_rotation() {
    let mut req = super::chiz_actions::request();
    req.start_fons = Some(250_000);
    req.actors[0].weapon_id = "weapon:fork_mamen".into();
    req.actors[0].gear_procs = vec![mamen_proc()];
    req.actors[0].static_stats.damage_bonus = Ratio {
        numerator: 1,
        denominator: 10,
    };
    req.actors[0].static_stats.crit_rate = Ratio::zero();

    let mut edgar = req.actors[0].clone();
    edgar.slot = 1;
    edgar.character_id = "character:1021".into();
    edgar.weapon_id.clear();
    edgar.gear_procs.clear();
    req.actors.push(edgar);
    req.relative_duration_us = 6_000_000;
    req.actions.extend([
        GameActionCommand::SwitchActor {
            timestamp_us: 1_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 1_000_001,
            actor_index: 1,
            ability_id: "GA_Edgar_Skill".into(),
            damage_effect_id: "GE_Player_Edgar_Skill_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 100,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 5_000_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 5_000_001,
            actor_index: 0,
            ability_id: "GA_Chiichan073_Melee".into(),
            damage_effect_id: "GE_Player_Chiichan073_Melee1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 100,
        },
    ]);

    let mut baseline = req.clone();
    baseline.actors[0].gear_procs.clear();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let bare_report = run_game_scenario(&baseline, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true, "{report}");
    assert_eq!(req.start_fons, Some(250_000));

    let cosmos_hits = |report: &Value, actor_index| {
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| {
                hit["actor_index"] == actor_index
                    && super::support::hit_metadata(report, hit)["packet"]["element"] == "cosmos"
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    let chiz_hits = cosmos_hits(&report, 0);
    let chiz_baseline_hits = cosmos_hits(&bare_report, 0);
    assert_eq!(chiz_hits.len(), 2);
    assert_eq!(chiz_baseline_hits.len(), 2);
    // 250000 Fons -> 2 stacks x T1 1/40 = 1/20. With Chiz's existing
    // 1/10 damage bonus, the additive-zone ratio is (1 + 1/10 + 1/20)/(1 + 1/10) = 23/22.
    for (boosted, bare) in chiz_hits.iter().zip(&chiz_baseline_hits) {
        let boosted: ExactFraction =
            serde_json::from_value(boosted["non_crit_damage"].clone()).unwrap();
        let bare: ExactFraction = serde_json::from_value(bare["non_crit_damage"].clone()).unwrap();
        assert_eq!(
            boosted.numerator * bare.denominator * 22,
            bare.numerator * boosted.denominator * 23
        );
    }

    let edgar_hits = cosmos_hits(&report, 1);
    let edgar_baseline_hits = cosmos_hits(&bare_report, 1);
    assert_eq!(edgar_hits.len(), 7);
    assert_eq!(
        edgar_hits
            .iter()
            .map(|hit| hit["non_crit_damage"].clone())
            .collect::<Vec<_>>(),
        edgar_baseline_hits
            .iter()
            .map(|hit| hit["non_crit_damage"].clone())
            .collect::<Vec<_>>()
    );
    assert!(
        report["resolved_model_assumptions"]
            .as_object()
            .unwrap()
            .contains_key("weapons.contemplative_cat_v1")
    );
}

fn rounded(report: &Value) -> Vec<u64> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|hit| hit["rounded_damage"].as_u64().unwrap())
        .collect()
}

#[test]
fn mamen_cosmos_bonus_adds_two_stacks_into_the_damage_zone() {
    // Hand-derived oracle: 250000 Fons -> stacks = min(10, floor(250000/100000)) = 2;
    // bonus = 2 x 1/40 = 1/20. Fixture: 1000 ATK, F_def = 1/2, general bonus 1/10.
    // M_bonus = 1 + 1/10 + 1/20 = 23/20; D = 1000 x 1/2 x 23/20 = 575 exact.
    // Multiplicative reading rejected: (1 + 1/10) x (1 + 1/20) = 231/200 = 1.155;
    // D' = 500 x 1.155 = 577.5 -> ties-up 578, not 575.
    let req = cosmos_fixture();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(rounded(&report), vec![550, 575, 550, 550, 550, 550]);

    let mut bare = req.clone();
    bare.actors[0].gear_procs.clear();
    let baseline = run_game_scenario(&bare, &ExecutionLimits::default()).unwrap();
    assert_eq!(rounded(&baseline), vec![550, 550, 550, 550, 550, 550]);

    let mut duplicate = req.clone();
    let extra = duplicate.actors[0].gear_procs[0].clone();
    duplicate.actors[0].gear_procs.push(extra);
    assert!(
        run_game_scenario(&duplicate, &ExecutionLimits::default())
            .unwrap_err()
            .contains("duplicates")
    );
}

#[test]
fn mamen_stacks_cap_at_ten_and_floor_at_the_fons_threshold() {
    // Hand-derived oracles (T1 = 1/40 per stack; fixture M baseline: 1000 x 1/2 x (1 + 1/10)):
    // 0 -> 0 stacks -> M = 11/10 -> 550.
    // 99999 -> min(10, 0) = 0 stacks -> 550.
    // 100000 -> 1 stack -> bonus 1/40 -> M = 1 + 1/10 + 1/40 = 45/40 = 9/8;
    //   D = 500 x 9/8 = 562.5 -> nearest-integer ties-up = 563 (FORMULA_CONTRACT §6).
    // 400000 -> 4 stacks -> bonus 4/40 = 1/10 -> M = 6/5 -> D = 500 x 6/5 = 600.
    // 1000000 -> min(10, 10) = 10 stacks -> bonus 10/40 = 1/4 -> M = 27/20 -> D = 675.
    // 9999999 -> min(10, 99) = 10 stacks -> 675 (cap holds far past the threshold).
    let mut req = cosmos_fixture();
    for (start_fons, expected_cosmos) in [
        (0, 550),
        (99_999u64, 550u64),
        (100_000, 563),
        (400_000, 600),
        (1_000_000, 675),
        (9_999_999, 675),
    ] {
        req.start_fons = Some(start_fons);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        let hits = rounded(&report);
        assert_eq!(hits[1], expected_cosmos, "start_fons {start_fons}");
        assert!(
            hits.iter()
                .enumerate()
                .all(|(i, hit)| i == 1 || *hit == 550),
            "start_fons {start_fons}"
        );
    }
}

#[test]
fn mamen_tier_five_per_stack_bonus_applies_exactly() {
    // Hand-derived: 200000 Fons -> 2 stacks x 9/200 = 18/200 = 9/100;
    // M_bonus = 1 + 1/10 + 9/100 = 119/100; D = 500 x 119/100 = 595 exact.
    let mut req = cosmos_fixture();
    if let GearProcConfig::MamenFons {
        cosmos_bonus_per_stack,
        ..
    } = &mut req.actors[0].gear_procs[0]
    {
        *cosmos_bonus_per_stack = Ratio {
            numerator: 9,
            denominator: 200,
        };
    }
    req.start_fons = Some(200_000);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(rounded(&report), vec![550, 595, 550, 550, 550, 550]);
}

#[test]
fn mamen_cap_falsifier_two_million_fons_stays_ten_stacks() {
    // Cap falsifier: 2000000 Fons would give 20 stacks without a cap.
    // stacks = min(10, floor(2000000/100000)) = min(10, 20) = 10 -> T1 +25% additive.
    // M_bonus = 1 + 1/10 + 10/40 = 1 + 1/10 + 1/4 = 27/20; D = 500 x 27/20 = 675 exact.
    let mut req = cosmos_fixture();
    req.start_fons = Some(2_000_000);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let hits = rounded(&report);
    assert_eq!(hits[1], 675, "start_fons 2000000 must stay at 10 stacks");
    assert!(
        hits.iter()
            .enumerate()
            .all(|(i, hit)| i == 1 || *hit == 550),
        "only the Cosmos packet gains the bonus"
    );
}

#[test]
fn mamen_missing_party_fons_fails_only_when_passive_damage_is_reached() {
    let mut req = cosmos_fixture();
    req.start_fons = None;
    crate::gear::validate_config(
        &req.actors[0],
        0,
        req.relative_duration_us,
        &ExecutionLimits::default(),
    )
    .unwrap();
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("explicit party start_fons")
    );

    let mut unreachable = req;
    unreachable.actors[0].gear_procs.clear();
    assert!(run_game_scenario(&unreachable, &ExecutionLimits::default()).is_ok());
}

#[test]
fn mamen_initialize_records_the_hypothesis_in_combat_provenance() {
    let req = cosmos_fixture();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(
        report["callback_model_assumptions"][crate::weapons::mamen::HYPOTHESIS_ID.as_str()],
        crate::weapons::mamen::HYPOTHESIS_ID.description()
    );
}

#[test]
fn mamen_projection_report_carries_the_modeled_hypothesis() {
    let tables = projection_tables();
    let mut actor = projection_actor();
    let report =
        crate::weapons::mamen::projection::project(&tables, 1, "ntesim-current", &mut actor, &[])
            .unwrap();
    assert_eq!(
        report["scope"],
        Value::String(crate::weapons::mamen::HYPOTHESIS_ID.description().into())
    );
    assert_eq!(
        crate::weapons::mamen::HYPOTHESIS_ID,
        "weapons.contemplative_cat_v1"
    );
}
