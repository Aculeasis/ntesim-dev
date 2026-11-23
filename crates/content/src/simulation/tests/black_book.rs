use super::*;
use crate::runtime_api::HitModifiers;
use crate::simulation::state::GameSimulatorState;
use ntesim_core::combat::{BreakDamageRequest, NegativeResistanceModel, break_damage};
use serde_json::json;
use std::collections::BTreeMap;

fn t1_proc() -> GearProcConfig {
    GearProcConfig::BlackBookBreak {
        break_intensity_bonus: Ratio {
            numerator: 60,
            denominator: 1,
        },
    }
}

fn projection_tables() -> std::collections::BTreeMap<String, serde_json::Value> {
    super::support::weapon_projection_tables()
}

#[test]
fn black_book_projection_binds_five_curves_rejects_group_and_unit_changes() {
    let tables = projection_tables();
    let req = research_request();
    let mut actor = req.actors[0].clone();
    actor.character_id = "character:1004".into();
    actor.weapon_id = "weapon:fork_BlackBook".into();
    actor.gear_procs.clear();
    let expected_bi = [60u128, 67, 74, 81, 88];
    for tier in 1..=5u32 {
        actor.weapon_tier = tier;
        let report = crate::weapons::black_book::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            if tier == 1 { &[] } else { &[0] },
        )
        .unwrap();
        let proc = &report["compiled_proc"];
        let bi = ExactFraction::from_ratio(
            serde_json::from_value(proc["break_intensity_bonus"].clone()).unwrap(),
        )
        .unwrap();
        assert_eq!(
            bi,
            ExactFraction::new(expected_bi[(tier - 1) as usize], 1).unwrap(),
            "tier {tier} BI curve"
        );
        assert_eq!(actor.gear_procs.len(), 1);
        assert_eq!(report["replaced_proc"].is_null(), tier == 1);
    }

    let mut cross_group = req.actors[0].clone();
    cross_group.character_id = "character:1072".into();
    cross_group.weapon_id = "weapon:fork_BlackBook".into();
    cross_group.weapon_tier = 1;
    cross_group.gear_procs.clear();
    assert!(
        crate::weapons::black_book::projection::project(
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
    let row = &mut mutated.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_BlackBook_5"]
        ["DataList"];
    row[0]["bIsPercent"] = json!(true);
    let mut unit_actor = actor.clone();
    unit_actor.weapon_tier = 5;
    unit_actor.gear_procs.clear();
    assert!(
        crate::weapons::black_book::projection::project(
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

fn damage_request() -> crate::simulation::GameScenarioRequest {
    let mut req = research_request();
    req.stochastic_crit = None;
    let actor = &mut req.actors[0];
    actor.character_id = "character:1004".into();
    actor.weapon_id = "weapon:fork_BlackBook".into();
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

fn zone_modifiers(
    req: &crate::simulation::GameScenarioRequest,
    state: &GameSimulatorState,
    target_index: usize,
) -> HitModifiers {
    super::support::gear_zone_modifiers(req, state, 1, target_index)
}

#[test]
fn black_book_damage_bonus_is_pure_passthrough() {
    let req = damage_request();
    let state = GameSimulatorState::new(&req);
    let modifiers = zone_modifiers(&req, &state, 0);
    assert!(
        modifiers.damage_bonus.is_zero(),
        "Black Tome grants no damage bonus"
    );
    assert!(
        modifiers.atk_percent.is_zero(),
        "Black Tome grants no ATK bonus"
    );
}

#[test]
fn black_book_scenario_matches_baseline() {
    let req = damage_request();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );

    let mut bare = req.clone();
    bare.actors[0].gear_procs.clear();
    let baseline = run_game_scenario(&bare, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        baseline["hit_receipts"][0]["non_crit_damage"],
        report["hit_receipts"][0]["non_crit_damage"],
        "static BI does not change ordinary damage"
    );
}

#[test]
fn black_book_validate_config_rejects_wrong_weapon_and_bad_ratios() {
    let mut req = damage_request();
    req.actors[0].weapon_id = "weapon:fork_Crowbar".into();
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires equipped")
    );

    let bad_ratio = damage_request();
    let mut bad_actor = bad_ratio.actors[0].clone();
    bad_actor.gear_procs = vec![GearProcConfig::BlackBookBreak {
        break_intensity_bonus: Ratio {
            numerator: 1,
            denominator: 0,
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
fn black_book_break_intensity_joins_break_damage_zone_additively() {
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
        intensity: 360.0,
        ..base
    })
    .unwrap();
    let expected_zone_ratio = (1.0 + 360.0 / 300.0) / (1.0 + 300.0 / 300.0);
    assert!((staged / plain - expected_zone_ratio).abs() < 1e-12);
    assert!((expected_zone_ratio - 1.10).abs() < 1e-12);
}

#[test]
fn runtime_recorder_records_and_validates_the_black_book_hypothesis() {
    let mut req = damage_request();
    let mut reached = BTreeMap::new();
    crate::runtime_api::record_hypothesis(
        &req,
        &mut reached,
        crate::weapons::black_book::HYPOTHESIS_ID,
        crate::weapons::black_book::HYPOTHESIS_ID.description(),
    )
    .unwrap();
    assert_eq!(
        reached[crate::weapons::black_book::HYPOTHESIS_ID.as_str()],
        crate::weapons::black_book::HYPOTHESIS_ID.description()
    );
    req.model_assumptions.insert(
        crate::weapons::black_book::HYPOTHESIS_ID.into(),
        "conflicting description".into(),
    );
    let rejected = crate::runtime_api::record_hypothesis(
        &req,
        &mut reached,
        crate::weapons::black_book::HYPOTHESIS_ID,
        crate::weapons::black_book::HYPOTHESIS_ID.description(),
    )
    .unwrap_err();
    assert!(rejected.contains("conflicting reached model hypothesis"));
}

fn break_damage_baseline() -> BreakDamageRequest {
    BreakDamageRequest {
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
    }
}

#[test]
fn black_book_zone_math_t1_hand_derived() {
    let base = break_damage_baseline();
    let plain = break_damage(&base).unwrap();
    let staged = break_damage(&BreakDamageRequest {
        intensity: 360.0,
        ..base.clone()
    })
    .unwrap();
    let ratio = staged / plain;
    let expected = 11.0 / 10.0;
    assert!((ratio - expected).abs() < 1e-12, "T1 +60 BI: {ratio}");
    assert!(
        (ratio - 1.60).abs() > 0.4,
        "rejects the multiplicative misreading of +60 BI points as x1.60"
    );
}

#[test]
fn black_book_zone_math_t3_t5_hand_derived() {
    let base = break_damage_baseline();
    let plain = break_damage(&base).unwrap();
    let t3 = break_damage(&BreakDamageRequest {
        intensity: 374.0,
        ..base.clone()
    })
    .unwrap();
    let ratio_t3 = t3 / plain;
    let expected_t3 = 337.0 / 300.0;
    assert!(
        (ratio_t3 - expected_t3).abs() < 1e-12,
        "T3 +74 BI: {ratio_t3}"
    );
    assert!(
        (ratio_t3 - 1.74).abs() > 0.4,
        "rejects the multiplicative misreading of +74 BI points as x1.74"
    );
    let t5 = break_damage(&BreakDamageRequest {
        intensity: 388.0,
        ..base
    })
    .unwrap();
    let ratio_t5 = t5 / plain;
    let expected_t5 = 86.0 / 75.0;
    assert!(
        (ratio_t5 - expected_t5).abs() < 1e-12,
        "T5 +88 BI: {ratio_t5}"
    );
    assert!(
        (ratio_t5 - 1.88).abs() > 0.4,
        "rejects the multiplicative misreading of +88 BI points as x1.88"
    );
}

#[test]
fn black_book_projection_reports_runtime_execution_validated_true() {
    let tables = projection_tables();
    let req = research_request();
    let mut actor = req.actors[0].clone();
    actor.character_id = "character:1004".into();
    actor.weapon_id = "weapon:fork_BlackBook".into();
    actor.weapon_tier = 5;
    actor.gear_procs.clear();
    let report = crate::weapons::black_book::projection::project(
        &tables,
        1,
        "ntesim-current",
        &mut actor,
        &[],
    )
    .unwrap();
    assert_eq!(report["runtime_execution_validated"], true);
}

fn append_action(
    req: &mut crate::simulation::GameScenarioRequest,
    time: u64,
    role: crate::simulation::metadata::InputActionRole,
    damage_type_ex: Option<&str>,
) {
    let index = req.actions.len();
    req.actions.push(GameActionCommand::InvokeAbility {
        timestamp_us: time,
        actor_index: 0,
        ability_id: "caller-test".into(),
        damage_effect_id: "caller-effect".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 0,
    });
    let mut metadata = research_request().action_metadata[&0].clone();
    metadata.input_action_role = Some(role);
    if let Some(damage_type) = damage_type_ex {
        metadata.damage_type_ex = Some(damage_type.into());
    }
    req.action_metadata.insert(index, metadata);
}

fn arc_scenario(tier: u32) -> crate::simulation::GameScenarioRequest {
    let mut req = research_request();
    req.stochastic_crit = None;
    let actor = &mut req.actors[0];
    actor.character_id = "character:1004".into();
    actor.weapon_id = "weapon:fork_BlackBook".into();
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
    req.relative_duration_us = 30_000_000;
    req.actions.clear();
    req.action_metadata.clear();
    req
}

#[test]
fn black_book_chain_observer_requires_native_primary_qte_identity() {
    let mut req = arc_scenario(1);
    req.actors[0].weapon_id = "weapon:fork_BlackBook".into();
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    let native = ntesim_data::native_action_metadata::resolve_ability(
        &req.content_id,
        ntesim_data::stat_projection::frozen_tables(),
        "character:1054",
        "GA_Daffodill_QTE",
    )
    .unwrap();
    let native_action =
        crate::simulation::metadata::resolve_action_metadata_with_owner("character:1054", &native)
            .unwrap();
    assert_eq!(
        native_action.input_role,
        Some(crate::simulation::metadata::InputActionRole::Qte)
    );
    assert_eq!(native_action.native_ability_index, Some(4));

    let mut spoof = native_action.clone();
    spoof.native_ability_index = None;
    crate::weapons::black_book::observe_team_admission(&req, &mut state, 0, 0, &spoof).unwrap();
    assert_eq!(state.actors[0].equipment.black_book.chains_remaining, 2);

    crate::weapons::black_book::observe_team_admission(&req, &mut state, 0, 0, &native_action)
        .unwrap();
    assert_eq!(state.actors[0].equipment.black_book.chains_remaining, 1);
}

#[test]
fn black_book_arc_ignores_role_only_research_packets_as_support_casts() {
    // 1. Initial attempt fails because 2 chains remain
    let mut sealed_req = arc_scenario(1);
    sealed_req.actions.push(GameActionCommand::PlayerInput {
        timestamp_us: 0,
        actor_index: 0,
        input: PlayerInput::Arc,
        critical_success: false,
    });
    let report = run_game_scenario(&sealed_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    let reason_msg = report["rejected_action"]["reason"]["message"]
        .as_str()
        .unwrap();
    assert!(reason_msg.contains("sealed by 2 chain(s)"));

    // A research packet that merely claims the QTE role has no native support identity.
    let mut one_qte = arc_scenario(1);
    append_action(
        &mut one_qte,
        0,
        crate::simulation::metadata::InputActionRole::Qte,
        None,
    );
    one_qte.actions.push(GameActionCommand::PlayerInput {
        timestamp_us: 100_000,
        actor_index: 0,
        input: PlayerInput::Arc,
        critical_success: false,
    });
    let report = run_game_scenario(&one_qte, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    let reason_msg = report["rejected_action"]["reason"]["message"]
        .as_str()
        .unwrap();
    assert!(reason_msg.contains("sealed by 2 chain(s)"));
}

#[test]
fn black_book_break_damage_callback_delivers_chaos_atk_packet() {
    let mut req = arc_scenario(1);
    append_action(
        &mut req,
        0,
        crate::simulation::metadata::InputActionRole::Qte,
        None,
    );
    append_action(
        &mut req,
        100_000,
        crate::simulation::metadata::InputActionRole::Qte,
        None,
    );
    req.actions.push(GameActionCommand::PlayerInput {
        timestamp_us: 200_000,
        actor_index: 0,
        input: PlayerInput::Arc,
        critical_success: false,
    });
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut trace = Vec::new();
    let mut sequence = 0;

    // Simulate Arc admission at t=200_000, then resolve its .2s startup.
    state.advance_unpaused_test_clock(200_000);
    state.active_actor_index = 0;
    let mut ctx = crate::runtime_api::CallbackContext {
        state: &mut state,
        request: &req,
        queue: &mut queue,
        sequence: &mut sequence,
        trace: &mut trace,
    };
    let params = crate::weapons::black_book::arc::parameters(&req.actors[0]).unwrap();
    crate::weapons::black_book::arc::activate(&mut ctx, 0, 0, params, 200_000).unwrap();
    assert_eq!(
        ctx.state.actors[0]
            .equipment
            .black_book
            .pending_startup
            .unwrap()
            .completes_at_us,
        400_000
    );
    assert_eq!(
        ctx.state.actors[0].equipment.black_book.designated_target,
        None
    );
    ctx.queue.clear();
    ctx.state.advance_unpaused_test_clock(400_000);
    crate::weapons::black_book::arc::startup_complete(&mut ctx, 0, 0, 1, 200_000, params).unwrap();
    assert_eq!(
        ctx.state.actors[0].equipment.black_book.designated_target,
        Some(0)
    );

    // When designated enemy takes Break damage, callback emits Chaos ATK packet
    crate::gear::on_target_break_damage(&mut ctx, 0).unwrap();
    assert_eq!(ctx.queue.len(), 2); // 1 designate tick + 1 hit delivery
    let hit_event = ctx
        .queue
        .iter()
        .find(|e| {
            matches!(
                &e.data,
                crate::simulation::events::GameEventData::HitDelivery { .. }
            )
        })
        .unwrap();
    if let crate::simulation::events::GameEventData::HitDelivery {
        ability_id,
        damage_effect_id,
        atk_rate,
        metadata,
        ..
    } = &hit_event.data
    {
        assert!(crate::weapons::black_book::arc::is_packet(metadata));
        assert_eq!(ability_id, crate::weapons::black_book::arc::ABILITY);
        assert_eq!(
            damage_effect_id,
            crate::weapons::black_book::arc::ATTACK_EFFECT_ID
        );
        // T1 break attack ratio is 200% (2.0)
        assert_eq!(
            *atk_rate,
            Ratio {
                numerator: 2,
                denominator: 1
            }
        );
    } else {
        panic!("expected HitDelivery event");
    }
}
