// police_rat weapon tests: Justice Executioner static ATK + Boss-gated damage.
use super::*;
use crate::simulation::state::GameSimulatorState;
use serde_json::json;
use std::collections::BTreeMap;

fn t5_proc() -> GearProcConfig {
    GearProcConfig::PoliceRatBoss {
        atk_bonus: Ratio {
            numerator: 1,
            denominator: 4,
        },
        boss_damage_bonus: Ratio {
            numerator: 1,
            denominator: 4,
        },
    }
}

fn projection_tables() -> std::collections::BTreeMap<String, serde_json::Value> {
    super::support::weapon_projection_tables()
}

#[test]
fn police_rat_projection_binds_curves_rejects_group_and_unit_changes() {
    let tables = projection_tables();
    let req = research_request();
    let mut actor = req.actors[0].clone();
    actor.character_id = "character:1020".into();
    actor.weapon_id = "weapon:fork_PoliceRat".into();
    actor.gear_procs.clear();
    // Hardcoded from the frozen CT_ForkBuff curves: 0.15, 0.175, 0.2, 0.225, 0.25.
    let expected = [(3u128, 20u128), (7, 40), (1, 5), (9, 40), (1, 4)];
    for (tier, (numerator, denominator)) in expected.iter().enumerate() {
        let tier = tier as u32 + 1;
        actor.weapon_tier = tier;
        let report = crate::weapons::police_rat::projection::project(
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
        let curve = ExactFraction::new(*numerator, *denominator).unwrap();
        assert_eq!(ratio_of("atk_bonus"), curve, "tier {tier} atk curve");
        assert_eq!(
            ratio_of("boss_damage_bonus"),
            curve,
            "tier {tier} boss curve"
        );
        // The summon curves are observed, never projected into the proc.
        assert!(proc.get("skill_damage").is_none());
        assert!(proc.get("summon_cooldown_us").is_none());
        assert_eq!(actor.gear_procs.len(), 1);
        assert_eq!(report["replaced_proc"].is_null(), tier == 1);
        assert_eq!(
            report["native_description"]["localized"],
            json!(
                "Increases ATK by <lv>{0}</>.\nIncreases damage dealt to Bosses by <lv>{1}</>.\nUnlocks Arc: Officer Whisker.\nArc: Officer Whisker \u{2014} Summons Officer Whisker to assist in combat. Officer Whisker continuously attacks enemies, dealing <lv>{2}</> of the wearer's ATK as damage per hit. Lasts <lv>30</>s. Cooldown: <lv>{3}</>s."
            )
        );
    }

    let mut cross_group = req.actors[0].clone();
    cross_group.character_id = "character:1019".into();
    cross_group.weapon_id = "weapon:fork_PoliceRat".into();
    cross_group.weapon_tier = 1;
    cross_group.gear_procs.clear();
    assert!(
        crate::weapons::police_rat::projection::project(
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
    let row = &mut mutated.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_PoliceRat_5"]
        ["DataList"];
    row[0]["bIsPercent"] = json!(false);
    let mut unit_actor = actor.clone();
    unit_actor.weapon_tier = 5;
    unit_actor.gear_procs.clear();
    assert!(
        crate::weapons::police_rat::projection::project(
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

fn atk_request() -> crate::simulation::GameScenarioRequest {
    let mut req = research_request();
    req.stochastic_crit = None;
    let actor = &mut req.actors[0];
    actor.character_id = "character:1020".into();
    actor.weapon_id = "weapon:fork_PoliceRat".into();
    actor.weapon_tier = 5;
    actor.gear_procs = vec![t5_proc()];
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
fn police_rat_atk_bonus_joins_base_scaled_atk_sum_unconditionally() {
    let req = atk_request();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // 1000 static ATK + 1000 base ATK * 1/4 = 1250; M_bonus stays 1 on a non-Boss target.
    assert_eq!(
        report["hit_receipts"][0]["non_crit_damage"],
        json!("1250/1")
    );
    assert_eq!(report["hit_receipts"][0]["rounded_damage"], 1250);

    let mut bare = req.clone();
    bare.actors[0].gear_procs.clear();
    let baseline = run_game_scenario(&bare, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        baseline["hit_receipts"][0]["non_crit_damage"],
        json!("1000/1")
    );
}

#[test]
fn police_rat_boss_damage_bonus_is_additive_and_boss_gated() {
    for (tier, ratio, non_boss, expected_boss, non_boss_rounded) in [
        (
            1,
            Ratio {
                numerator: 3,
                denominator: 20,
            },
            (2875, 2),
            (1610, 1),
            1438,
        ),
        (
            5,
            Ratio {
                numerator: 1,
                denominator: 4,
            },
            (3125, 2),
            (1875, 1),
            1563,
        ),
    ] {
        let mut request = atk_request();
        request.actors[0].gear_procs = vec![GearProcConfig::PoliceRatBoss {
            atk_bonus: ratio,
            boss_damage_bonus: ratio,
        }];
        request.actors[0].weapon_tier = tier;
        // Independent non-weapon bonus distinguishes additive M_bonus from a second multiplier.
        request.actors[0].static_stats.damage_bonus = Ratio {
            numerator: 1,
            denominator: 4,
        };
        let mut second_target = request.targets[0].clone();
        second_target.target_index = 1;
        second_target.is_boss = true;
        request.targets.push(second_target);

        let plain = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
        let plain_hit = &plain["hit_receipts"][0];
        assert_eq!(
            plain_hit["non_crit_damage"],
            json!(format!("{}/{}", non_boss.0, non_boss.1))
        );
        assert_eq!(plain_hit["rounded_damage"], non_boss_rounded);
        assert_eq!(plain["final_target_states"][0]["is_boss"], false);
        assert_eq!(plain["final_target_states"][1]["is_boss"], true);

        request.targets[0].is_boss = true;
        request.targets[1].is_boss = false;
        let boss_report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
        let boss_hit = &boss_report["hit_receipts"][0];
        assert_eq!(
            boss_hit["non_crit_damage"],
            json!(format!("{}/{}", expected_boss.0, expected_boss.1))
        );
        assert_eq!(
            boss_hit["rounded_damage"],
            expected_boss.0 / expected_boss.1
        );
        assert_eq!(boss_report["final_target_states"][0]["is_boss"], true);
        assert_eq!(boss_report["final_target_states"][1]["is_boss"], false);
        let assumption = boss_report["resolved_model_assumptions"]
            [crate::weapons::police_rat::HYPOTHESIS_ID.as_str()]
        .as_str()
        .unwrap();
        assert!(assumption.contains("is_boss"));
        assert!(assumption.contains("does not classify a target as non-Boss"));
    }
}

#[test]
fn police_rat_boss_metadata_is_scoped_to_the_selected_target() {
    let mut request = atk_request();
    request.targets[0].is_boss = false;
    let mut second_target = request.targets[0].clone();
    second_target.target_index = 1;
    second_target.is_boss = true;
    request.targets.push(second_target);
    let state = GameSimulatorState::new(&request);

    let first = super::support::gear_zone_modifiers(&request, &state, 1, 0);
    let second = super::support::gear_zone_modifiers(&request, &state, 1, 1);
    assert!(first.damage_bonus.is_zero());
    assert_eq!(
        second.damage_bonus,
        ExactFraction::new(1, 4).unwrap(),
        "the same proc reads only the selected target's explicit Boss flag"
    );
    assert_eq!(first.atk_percent, ExactFraction::new(1, 4).unwrap());
    assert_eq!(second.atk_percent, ExactFraction::new(1, 4).unwrap());
}

#[test]
fn police_rat_validate_config_rejects_wrong_weapon() {
    let mut req = atk_request();
    req.actors[0].weapon_id = "weapon:fork_PaperPlane".into();
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires equipped")
    );

    let mut duplicate = atk_request();
    let extra = duplicate.actors[0].gear_procs[0].clone();
    duplicate.actors[0].gear_procs.push(extra);
    assert!(
        run_game_scenario(&duplicate, &ExecutionLimits::default())
            .unwrap_err()
            .contains("duplicates")
    );
}

#[test]
fn police_rat_no_effect_without_proc_even_when_scenario_target_is_boss() {
    let mut req = atk_request();
    req.targets[0].is_boss = true;
    req.actors[0].gear_procs.clear();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["final_target_states"][0]["is_boss"], true);
    assert_eq!(
        report["hit_receipts"][0]["non_crit_damage"],
        json!("1000/1")
    );
    assert!(
        !report["resolved_model_assumptions"]
            .as_object()
            .unwrap()
            .contains_key(crate::weapons::police_rat::HYPOTHESIS_ID.as_str())
    );
}

#[test]
fn police_rat_initialize_records_hypothesis_when_proc_present() {
    let req = atk_request();
    let mut state = GameSimulatorState::new(&req);
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    crate::gear::initialize(&mut crate::runtime_api::CallbackContext {
        state: &mut state,
        request: &req,
        queue: &mut queue,
        sequence: &mut sequence,
        trace: &mut trace,
    })
    .unwrap();
    assert_eq!(
        state.execution_model_assumptions[crate::weapons::police_rat::HYPOTHESIS_ID.as_str()],
        crate::weapons::police_rat::HYPOTHESIS_ID.description()
    );

    let mut bare_req = atk_request();
    bare_req.actors[0].gear_procs.clear();
    let mut bare_state = GameSimulatorState::new(&bare_req);
    crate::gear::initialize(&mut crate::runtime_api::CallbackContext {
        state: &mut bare_state,
        request: &bare_req,
        queue: &mut queue,
        sequence: &mut sequence,
        trace: &mut trace,
    })
    .unwrap();
    assert!(
        !bare_state
            .execution_model_assumptions
            .contains_key(crate::weapons::police_rat::HYPOTHESIS_ID.as_str()),
        "no hypothesis without the proc"
    );
}

#[test]
fn police_rat_runtime_recorder_records_and_validates_the_hypothesis() {
    let mut req = atk_request();
    req.actors[0].gear_procs.clear();
    let mut reached = BTreeMap::new();
    crate::runtime_api::record_hypothesis(
        &req,
        &mut reached,
        crate::weapons::police_rat::HYPOTHESIS_ID,
        crate::weapons::police_rat::HYPOTHESIS_ID.description(),
    )
    .unwrap();
    assert_eq!(
        reached[crate::weapons::police_rat::HYPOTHESIS_ID.as_str()],
        crate::weapons::police_rat::HYPOTHESIS_ID.description()
    );
    req.model_assumptions.insert(
        crate::weapons::police_rat::HYPOTHESIS_ID.into(),
        "conflicting description".into(),
    );
    let rejected = crate::runtime_api::record_hypothesis(
        &req,
        &mut reached,
        crate::weapons::police_rat::HYPOTHESIS_ID,
        crate::weapons::police_rat::HYPOTHESIS_ID.description(),
    )
    .unwrap_err();
    assert!(rejected.contains("conflicting reached model hypothesis"));
}

#[test]
fn police_rat_projection_reports_runtime_execution_validated_true() {
    // Honesty pin: the projection claims runtime_execution_validated=true, and the
    // batch-11 audit verified that both proc components (ATK percent and Boss ratio)
    // are consumed in the executed damage path (gear::damage_bonus, called from
    // simulation/hit/calculation.rs), unlike crowbar's unconsumed break-intensity
    // modifier. This test guards the flag against silent drift.
    let tables = projection_tables();
    let req = research_request();
    let mut actor = req.actors[0].clone();
    actor.character_id = "character:1020".into();
    actor.weapon_id = "weapon:fork_PoliceRat".into();
    actor.weapon_tier = 5;
    actor.gear_procs.clear();
    let report = crate::weapons::police_rat::projection::project(
        &tables,
        1,
        "ntesim-current",
        &mut actor,
        &[],
    )
    .unwrap();
    assert_eq!(report["runtime_execution_validated"], true);
}

fn arc_request(tier: u32) -> crate::simulation::GameScenarioRequest {
    let mut req = research_request();
    req.stochastic_crit = None;
    let actor = &mut req.actors[0];
    actor.character_id = "character:1020".into();
    actor.weapon_id = "weapon:fork_PoliceRat".into();
    actor.weapon_tier = tier;
    actor.gear_procs.clear();
    actor.static_stats.base_atk = 1000;
    actor.static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    actor.static_stats.crit_rate = Ratio::zero();
    actor.static_stats.damage_bonus = Ratio::zero();
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.relative_duration_us = 35_000_000;
    req.actions = vec![GameActionCommand::PlayerInput {
        timestamp_us: 0,
        actor_index: 0,
        input: PlayerInput::Arc,
        critical_success: false,
    }];
    req.action_metadata.clear();
    req
}

#[test]
fn police_rat_arc_all_refinements_deliver_30_periodic_hits() {
    let expected_damages = [1000, 1250, 1500, 1750, 2000];
    for tier in 1..=5u32 {
        let req = arc_request(tier);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let hits = report["hit_receipts"].as_array().unwrap();
        assert_eq!(hits.len(), 30, "tier {tier} delivers 30 periodic hits");
        let expected_dmg = expected_damages[(tier - 1) as usize];
        for (i, hit) in hits.iter().enumerate() {
            let expected_time = (i as u64 + 1) * 1_000_000;
            assert_eq!(hit["time_us"], expected_time);
            assert_eq!(hit["damage"], json!(format!("{}/{}", expected_dmg, 1)));
            assert_eq!(hit["rounded_damage"], expected_dmg);
            let meta: crate::simulation::metadata::NormalizedActionPacketMetadata =
                serde_json::from_value(super::support::hit_metadata(&report, hit).clone()).unwrap();
            assert!(crate::weapons::police_rat::arc::is_packet(&meta));
        }
        let trace = report["trace"].as_array().unwrap();
        assert!(trace.iter().any(|r| r["event_kind"] == "police_rat_arc"));
    }
}

#[test]
fn police_rat_arc_cooldown_rejection_and_postmortem_stop() {
    let mut req = arc_request(1);
    req.actions.push(GameActionCommand::PlayerInput {
        timestamp_us: 10_000_000,
        actor_index: 0,
        input: PlayerInput::Arc,
        critical_success: false,
    });
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert_eq!(report["rejected_action"]["reason"]["kind"], "cooldown");
    assert_eq!(report["rejected_action"]["reason"]["until_us"], 60_000_000);

    // Defeated carrier stops periodic attacks
    let mut death_req = arc_request(1);
    death_req.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
        timestamp_us: 5_000_000,
        actor_index: 0,
        delivered_damage: 100_000,
    }];
    death_req.model_assumptions.insert(
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let death_report = run_game_scenario(&death_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(death_report["execution_completed"], true);
    let hits = death_report["hit_receipts"].as_array().unwrap();
    // Ticks at 1s, 2s, 3s, 4s delivered; at 5s fatal damage happens before 5s hit or cancels it
    assert!(hits.len() <= 5);
    for hit in hits {
        assert!(hit["time_us"].as_u64().unwrap() <= 5_000_000);
    }
}
