use super::*;
use crate::simulation::metadata::PacketMetadata;
use ntesim_core::buffs::dispatch::BuffManager;
use ntesim_core::combat::{BreakDamageRequest, NegativeResistanceModel, break_damage};
use serde_json::json;
use std::collections::BTreeMap;

fn dustbin_proc() -> GearProcConfig {
    GearProcConfig::DustbinBreak {
        break_intensity_bonus: Ratio {
            numerator: 60,
            denominator: 1,
        },
        duration_us: 10_000_000,
        cooldown_us: 20_000_000,
    }
}

fn projection_tables() -> BTreeMap<String, serde_json::Value> {
    super::support::weapon_projection_tables()
}

fn projection_actor() -> GameActorConfig {
    serde_json::from_value(json!({
        "slot":0,"character_id":"character:1023","initial_hp":1,"initial_hp_max":1,"initial_energy":0,
        "energy_capacity":100,"weapon_id":"weapon:fork_dustbin","weapon_tier":1,
        "static_stats":{"base_atk":1,"atk":"1/1","hp":1,"def":1,
        "crit_rate":"0/1","crit_damage_bonus":"0/1"}
    }))
    .unwrap()
}

#[test]
fn dustbin_projection_binds_five_refinement_tiers() {
    let tables = projection_tables();
    let expected_bonus = [60, 66, 72, 78, 84];
    let mut actor = projection_actor();
    for (tier, bonus) in (1..=5).zip(expected_bonus) {
        actor.weapon_tier = tier as u32;
        actor.gear_procs.clear();
        let report = crate::weapons::dustbin::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap();
        let proc = &report["compiled_proc"];
        assert_eq!(proc["kind"], "dustbin_break");
        assert_eq!(
            proc["break_intensity_bonus"],
            json!(format!("{}/{}", bonus, 1))
        );
        assert_eq!(proc["duration_us"], 10_000_000);
        assert_eq!(proc["cooldown_us"], 20_000_000);
        assert_eq!(report["native_parameter_report"]["parameter_count"], 3);
        // Break damage itself is not executed (native Break/BI lifecycle Q1-V1
        // is pending), so the projection must not claim validated execution.
        assert_eq!(report["runtime_execution_validated"], false);
        assert_eq!(actor.gear_procs.len(), 1);
    }
}

#[test]
fn dustbin_projection_rejects_group_mismatch_and_changed_units() {
    let tables = projection_tables();
    let mut actor = projection_actor();
    actor.character_id = "character:1003".into();
    assert!(
        crate::weapons::dustbin::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap_err()
        .contains("group is incompatible")
    );
    actor.character_id = "character:1023".into();
    let mut changed = tables.clone();
    changed.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]
        ["upgradestar_pack_fork_dustbin_1"]["DataList"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["NameID"] == "buff_dustbin_Unbal")
        .unwrap()["bIsPercent"] = json!(true);
    assert!(
        crate::weapons::dustbin::projection::project(
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
fn dustbin_validate_config_rejects_mismatches() {
    let limits = ExecutionLimits::default();
    let mut actor = projection_actor();
    let valid = dustbin_proc();
    let ctx = crate::gear::validation::ConfigContext {
        actor: &actor,
        actor_index: 0,
        limits: &limits,
    };
    let validation = crate::weapons::dustbin::validate_config(&valid, &ctx).unwrap();
    assert_eq!(validation.kind, "dustbin");
    assert_eq!(validation.duration_us, 20_000_000);

    actor.weapon_id = "weapon:fork_Worldrain".into();
    let ctx = crate::gear::validation::ConfigContext {
        actor: &actor,
        actor_index: 0,
        limits: &limits,
    };
    let err = crate::weapons::dustbin::validate_config(&valid, &ctx)
        .err()
        .expect("expected weapon mismatch rejection");
    assert!(err.contains("weapon:fork_dustbin"));

    actor.weapon_id = "weapon:fork_dustbin".into();
    let ctx = crate::gear::validation::ConfigContext {
        actor: &actor,
        actor_index: 0,
        limits: &limits,
    };
    let zero_duration = GearProcConfig::DustbinBreak {
        break_intensity_bonus: Ratio {
            numerator: 60,
            denominator: 1,
        },
        duration_us: 0,
        cooldown_us: 20_000_000,
    };
    let err = crate::weapons::dustbin::validate_config(&zero_duration, &ctx)
        .err()
        .expect("expected zero duration rejection");
    assert!(err.contains("positive duration"));
    let zero_cooldown = GearProcConfig::DustbinBreak {
        break_intensity_bonus: Ratio {
            numerator: 60,
            denominator: 1,
        },
        duration_us: 10_000_000,
        cooldown_us: 0,
    };
    let err = crate::weapons::dustbin::validate_config(&zero_cooldown, &ctx)
        .err()
        .expect("expected zero cooldown rejection");
    assert!(err.contains("positive trigger cooldown"));
    let bad_ratio = GearProcConfig::DustbinBreak {
        break_intensity_bonus: Ratio {
            numerator: 60,
            denominator: 0,
        },
        duration_us: 10_000_000,
        cooldown_us: 20_000_000,
    };
    assert!(crate::weapons::dustbin::validate_config(&bad_ratio, &ctx).is_err());
}

fn tagged_packet() -> PacketMetadata {
    serde_json::from_value(json!({
        "damage_effect_id": "caller-effect",
        "native_ga_name": null,
        "native_damage_type_ex": "EExecutionDamageTypeEX::DAMAGE_TYPE_NATURE",
        "native_damage_source_category": null,
        "damage_kind": "elemental",
        "element": "nature",
        "fixed_crit_rate": null,
        "fixed_crit_rate_provenance": null,
        "unbal_value": null,
        "unbal_value_provenance": null,
        "semantic_tags": ["ntesim.trigger.break_reduction"],
        "semantic_provenance": null,
        "source_evidence": null,
        "provenance": {"kind": "accepted_model_hypothesis", "id": "test", "description": "test"}
    }))
    .unwrap()
}

fn buff_instances(buffs: &BuffManager, id: &str) -> Vec<usize> {
    buffs
        .instances()
        .iter()
        .enumerate()
        .filter(|(_, instance)| instance.id == id)
        .map(|(index, _)| index)
        .collect()
}

#[test]
fn dustbin_grant_creates_buff_and_cooldown_marker() {
    let mut buffs = BuffManager::default();
    let granted =
        crate::weapons::dustbin::try_grant_break_buff(&mut buffs, 0, 0, &dustbin_proc()).unwrap();
    assert!(granted);
    assert_eq!(
        buff_instances(&buffs, crate::weapons::dustbin::BUFF_ID).len(),
        1
    );
    assert_eq!(
        buff_instances(&buffs, crate::weapons::dustbin::ICD_BUFF_ID).len(),
        1
    );
    let buff = &buffs.instances()[buff_instances(&buffs, crate::weapons::dustbin::BUFF_ID)[0]];
    assert_eq!(buff.owner_actor, 0);
    assert_eq!(
        buff.stat_modifiers["break_intensity"],
        ntesim_core::formula::Ratio {
            numerator: 60,
            denominator: 1
        }
    );
}

#[test]
fn dustbin_semantic_tag_cannot_grant_without_actual_gauge_reduction() {
    let mut request = research_request();
    request.actors[0].weapon_id = "weapon:fork_dustbin".into();
    request.actors[0].gear_procs = vec![dustbin_proc()];
    let mut state = crate::simulation::state::GameSimulatorState::new(&request);
    let packet = tagged_packet();
    super::support::invoke_gear_hook(
        &request,
        &mut state,
        0,
        crate::simulation::metadata::InputActionRole::Normal,
        &packet,
        false,
    );
    assert!(state.buffs.instances().is_empty());
    super::support::invoke_gear_hook(
        &request,
        &mut state,
        2_000_000,
        crate::simulation::metadata::InputActionRole::Normal,
        &packet,
        true,
    );
    assert!(state.buffs.instances().is_empty());
}

#[test]
fn dustbin_icd_suppresses_retrigger_inside_cooldown_window() {
    let mut buffs = BuffManager::default();
    assert!(
        crate::weapons::dustbin::try_grant_break_buff(&mut buffs, 0, 0, &dustbin_proc()).unwrap()
    );
    assert!(
        !crate::weapons::dustbin::try_grant_break_buff(&mut buffs, 0, 19_999_999, &dustbin_proc())
            .unwrap()
    );
    assert_eq!(
        buff_instances(&buffs, crate::weapons::dustbin::BUFF_ID).len(),
        1
    );
    assert!(
        crate::weapons::dustbin::try_grant_break_buff(&mut buffs, 0, 20_000_000, &dustbin_proc())
            .unwrap()
    );
    assert_eq!(
        buff_instances(&buffs, crate::weapons::dustbin::ICD_BUFF_ID).len(),
        1
    );
}

#[test]
fn dustbin_buff_expires_at_frozen_duration() {
    let mut buffs = BuffManager::default();
    crate::weapons::dustbin::try_grant_break_buff(&mut buffs, 0, 0, &dustbin_proc()).unwrap();
    buffs.advance_time(9_999_999);
    assert_eq!(
        buff_instances(&buffs, crate::weapons::dustbin::BUFF_ID).len(),
        1
    );
    buffs.advance_time(10_000_000);
    assert_eq!(
        buff_instances(&buffs, crate::weapons::dustbin::BUFF_ID).len(),
        0
    );
}

#[test]
fn dustbin_break_intensity_joins_break_damage_zone_additively() {
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
        intensity: 360.0,
        ..base
    })
    .unwrap();
    let expected_zone_ratio = (1.0 + 360.0 / 300.0) / (1.0 + 300.0 / 300.0);
    assert!((boosted / plain - expected_zone_ratio).abs() < 1e-12);
    assert!((expected_zone_ratio - 1.1).abs() < 1e-12);
    assert!((boosted / plain - 1.2).abs() > 1e-9);
}

#[test]
fn dustbin_zone_math_hand_derived_t1_t5() {
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
    // Frozen buff_dustbin_Unbal: T1 +60, T5 +84 points.
    assert_eq!(reduced(600 + 60, 600), (11, 10));
    assert_ne!(reduced(600 + 60, 600), (6, 5));
    assert_eq!(reduced(600 + 84, 600), (57, 50));
    assert_ne!(reduced(600 + 84, 600), (32, 25));
}

fn live_request() -> GameScenarioRequest {
    let mut request = research_request();
    request.stochastic_crit = None;
    request.relative_duration_us = 41_000_000;
    request.actors[0].weapon_id = "weapon:fork_dustbin".into();
    request.actors[0].weapon_tier = 1;
    request.actors[0].gear_procs = vec![dustbin_proc()];
    request.actors[0].static_stats.break_intensity = Ratio {
        numerator: 300,
        denominator: 1,
    };
    request.targets[0].break_profile = Some(GameTargetBreakProfileConfig {
        capacity: Ratio {
            numerator: 30,
            denominator: 1,
        },
        current_gauge: Ratio::one(),
        broken_duration_us: 100,
        discord_fraction: None,
    });
    let action = request.actions[0].clone();
    let mut metadata = request.action_metadata[&0].clone();
    metadata.unbal_value = Some(Ratio {
        numerator: 30,
        denominator: 1,
    });
    request.actions = [0, 2, 10, 20, 22, 40]
        .into_iter()
        .map(|second| {
            let mut action = action.clone();
            if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut action {
                *timestamp_us = second * 1_000_000;
            }
            action
        })
        .collect();
    request.action_metadata = (0..6).map(|index| (index, metadata.clone())).collect();
    request
}

#[test]
fn dustbin_partial_off_field_reduction_grants_only_its_owner_at_settlement() {
    use crate::simulation::{ExactFraction, state::GameSimulatorState};
    let mut request = live_request();
    let mut ally = request.actors[0].clone();
    ally.slot = 1;
    ally.gear_procs.clear();
    request.actors.push(ally);
    let mut state = GameSimulatorState::new(&request);
    state.active_actor_index = 1;
    state.advance_unpaused_test_clock(2_000_000);
    let mut trace = Vec::new();
    assert!(
        !crate::simulation::break_state::apply_unbalance(
            &request,
            &mut state,
            &mut Default::default(),
            &mut 0,
            &mut trace,
            0,
            0,
            ExactFraction::new(1, 3).unwrap(),
        )
        .unwrap()
    );
    assert_eq!(
        state.targets[0].break_state.as_ref().unwrap().gauge,
        ExactFraction::new(2, 3).unwrap()
    );
    let buff = state
        .buffs
        .instances()
        .iter()
        .find(|buff| buff.id == crate::weapons::dustbin::BUFF_ID)
        .unwrap();
    assert_eq!(buff.owner_actor, 0);
    assert!(crate::weapons::dustbin::icd_active(
        &state.buffs,
        0,
        21_999_999
    ));
    assert!(!crate::weapons::dustbin::icd_active(
        &state.buffs,
        0,
        22_000_000
    ));
    assert!(!crate::weapons::dustbin::icd_active(
        &state.buffs,
        1,
        2_000_000
    ));
}

#[test]
fn dustbin_live_reduction_boosts_triggering_break_and_continues_through_expiry_and_icd() {
    for (tier, bonus, boosted_n, boosted_d) in [
        (1, 60, 158532, 5),
        (2, 66, 799866, 25),
        (3, 72, 807072, 25),
        (4, 78, 814278, 25),
        (5, 84, 821484, 25),
    ] {
        let mut request = live_request();
        request.actors[0].weapon_tier = tier;
        if let GearProcConfig::DustbinBreak {
            break_intensity_bonus,
            ..
        } = &mut request.actors[0].gear_procs[0]
        {
            *break_intensity_bonus = Ratio {
                numerator: bonus,
                denominator: 1,
            };
        }
        let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
        let hits: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit.get("break_contribution").is_some())
            .collect();
        assert_eq!(hits.len(), 6);
        for (index, hit) in hits.iter().enumerate() {
            // 3603 * (1 + (300+bonus)/300) * 10 * .5 * .8.
            let expected = if index == 2 {
                json!("28824/1")
            } else {
                json!(format!("{}/{}", boosted_n, boosted_d))
            };
            assert_eq!(hit["damage"], expected, "tier {tier}, hit {index}");
        }
        let grants: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "dustbin_break_intensity_granted")
            .map(|event| event["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(grants, [0, 20_000_000, 40_000_000]);
    }
}

#[test]
fn dustbin_no_bar_zero_efficiency_dead_target_and_zero_start_do_not_trigger() {
    for case in 0..6 {
        let mut request = live_request();
        request.actions.truncate(1);
        request.action_metadata.retain(|index, _| *index == 0);
        match case {
            0 => request.targets[0].break_profile = None,
            1 => request.actors[0].static_stats.break_efficiency_multiplier = Ratio::zero(),
            2 => request.targets[0].hp_current = 1,
            3 => {
                request.targets[0]
                    .break_profile
                    .as_mut()
                    .unwrap()
                    .current_gauge = Ratio::zero()
            }
            4 => request.action_metadata.get_mut(&0).unwrap().unbal_value = Some(Ratio::zero()),
            _ => {
                // An external scenario input is not the wearer's gauge damage,
                // even when it enters Broken with outgoing efficiency zero.
                request.actors[0].static_stats.break_efficiency_multiplier = Ratio::zero();
                request.actions[0] = GameActionCommand::TargetGauge {
                    timestamp_us: 0,
                    target_index: 0,
                    gauge: Ratio::zero(),
                };
                request.action_metadata.clear();
            }
        }
        let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
        if case == 5 {
            let hits = report["hit_receipts"].as_array().unwrap();
            assert_eq!(hits.len(), 1);
            assert_eq!(hits[0]["damage"], json!("28824/1"));
            assert_eq!(
                report["final_target_states"][0]["break_state"]["generation"],
                1
            );
        }
        assert!(
            !report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .any(|event| event["event_kind"] == "dustbin_break_intensity_granted"),
            "case {case}"
        );
    }
}
