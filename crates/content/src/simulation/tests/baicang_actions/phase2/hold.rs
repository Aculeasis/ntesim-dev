use super::*;
use crate::simulation::GameHoldInput;

fn held(req: &mut GameScenarioRequest, time: u64, duration_us: u64, moving: bool) {
    req.hold_inputs.insert(
        req.actions.len(),
        GameHoldInput {
            reserved_stocks: None,
            duration_us,
            moving,
        },
    );
    req.actions.push(normal(time));
}

fn skill(time: u64) -> GameActionCommand {
    make_action(
        time,
        0,
        "GA_Cang_Skill",
        "GE_Player_Cang_Skill_Damage",
        0,
        14_000_000,
    )
}

#[test]
fn live_normal_stage_selects_heavy_and_retains_release_time_and_three_explosions() {
    for (stage, expected, impact, explosions) in [
        (
            1,
            "GE_Player_Cang_Branch1_Damage",
            416_667,
            vec![736_667; 3],
        ),
        (
            2,
            "GE_Player_Cang_Branch1_Damage",
            416_667,
            vec![736_667; 3],
        ),
        (
            3,
            "GE_Player_Cang_Branch2_Damage",
            251_127,
            vec![600_174, 617_079, 633_235],
        ),
    ] {
        let mut req = phase2_request();
        req.actions = [0, 1_000_000, 2_000_000]
            .into_iter()
            .take(stage)
            .map(normal)
            .collect();
        let time = if stage == 3 {
            4_000_000
        } else {
            stage as u64 * 1_000_000
        };
        held(&mut req, time, 200_000, false);
        let report = run(&req);
        let hits = report["hit_receipts"].as_array().unwrap();
        assert_eq!(hits[stage]["damage_effect_id"], expected);
        assert_eq!(hits[stage]["time_us"], time + 200_000 + impact);
        assert_eq!(hits.len(), stage + 4);
        assert_eq!(
            hits[stage + 1..]
                .iter()
                .map(|hit| hit["time_us"].as_u64().unwrap())
                .collect::<Vec<_>>(),
            explosions
                .into_iter()
                .map(|offset| time + 200_000 + offset)
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn short_holds_remain_taps_and_the_threshold_selects_heavy() {
    for (duration, packet) in [
        (0, "GE_Player_Cang_Melee2_Damage"),
        (199_999, "GE_Player_Cang_Melee2_Damage"),
        (200_000, "GE_Player_Cang_Branch1_Damage"),
    ] {
        let mut req = phase2_request();
        req.actions = vec![normal(0)];
        held(&mut req, 1_000_000, duration, false);
        assert_eq!(run(&req)["hit_receipts"][1]["damage_effect_id"], packet);
    }
}

#[test]
fn dodge_heavy_selects_movement_pays_only_one_dodge_and_discards_riposte() {
    for (moving, packet) in [
        (false, "GE_Player_Cang_Branch1_Damage"),
        (true, "GE_Player_Cang_Branch2_Damage"),
    ] {
        let mut req = phase2_request();
        req.actions.clear();
        dodge(&mut req);
        held(&mut req, 500_000, 200_000, moving);
        req.actions.push(normal(2_500_000));
        let report = run(&req);
        assert_eq!(report["hit_receipts"][0]["damage_effect_id"], packet);
        assert_eq!(
            report["hit_receipts"][4]["damage_effect_id"],
            "GE_Player_Cang_Melee1_Damage"
        );
        assert_eq!(report["combat_stamina"]["total_spent"], json!("15/1"));
    }
}

#[test]
fn heavy_recovery_is_a_real_deadline_and_failed_followup_keeps_existing_packets() {
    for (time, complete) in [(2_681_223, false), (2_681_224, true)] {
        let mut req = phase2_request();
        req.actions = vec![normal(0)];
        held(&mut req, 1_000_000, 200_000, false);
        req.actions.push(skill(time));
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["execution_completed"], complete);
        assert_eq!(report["hits_settled"], if complete { 6 } else { 5 });
        assert_eq!(
            report["final_actor_states"][0]["hp"],
            if complete { 9700 } else { 10000 }
        );
    }
}

#[test]
fn two_continuing_heavy_cycles_preserve_word_counter_resources_and_aoe_opportunities() {
    let mut req = phase2_request();
    req.actors[0].initial_energy = 0;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    req.actions.clear();
    for base in [0, 20_000_000] {
        req.actions.push(normal(base));
        held(&mut req, base + 1_000_000, 200_000, false);
        req.actions.push(skill(base + 3_000_000));
    }
    let report = run(&req);
    assert_eq!(report["hits_settled"], 24);
    assert_eq!(report["total_damage_exact"], json!("11028/1"));
    let actor = &report["final_actor_states"][0];
    assert_eq!(actor["hp"], 9127); // B, the continuing third Normal input's C, then B.
    assert_eq!(actor["granted_energy"], 17);
    assert_eq!(actor["charge_remainder"], json!("157/500"));
    assert_eq!(actor["cycle"]["granted"], 28);
    assert_eq!(actor["cycle"]["carry"], json!("371/500"));
}

#[test]
fn invalid_selector_fails_atomically_instead_of_inventing_a_heavy() {
    for (normal_count, time, moving) in [
        (0, 0, false),
        (1, 4_000_000, false),
        (1, 1_000_000, true),
        (4, 6_000_000, false),
    ] {
        let mut req = phase2_request();
        req.actions = [0, 1_000_000, 2_000_000, 3_300_000]
            .into_iter()
            .take(normal_count)
            .map(normal)
            .collect();
        held(&mut req, time, 200_000, moving);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["execution_completed"], false);
        assert_eq!(
            report["action_metadata_ledger"]
                .as_array()
                .map_or(0, |entries| {
                    entries
                        .iter()
                        .map(|entry| entry["command_indices"].as_array().unwrap().len())
                        .sum()
                }),
            normal_count
        );
        assert!(
            report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .all(|hit| !hit["damage_effect_id"].as_str().unwrap().contains("Branch"))
        );
    }
}

#[test]
fn hold_profiles_reject_foreign_bindings_indices_and_deadline_overflow() {
    let mut req = phase2_request();
    req.actions = vec![normal(0)];
    for (index, duration_us, moving) in [(1, 200_000, false), (0, u64::MAX, false), (0, 1, true)] {
        let mut invalid = req.clone();
        invalid.hold_inputs.insert(
            index,
            GameHoldInput {
                reserved_stocks: None,
                duration_us,
                moving,
            },
        );
        assert!(run_game_scenario(&invalid, &ExecutionLimits::default()).is_err());
    }
    req.actions[0] = skill(0);
    req.hold_inputs.insert(
        0,
        GameHoldInput {
            reserved_stocks: None,
            duration_us: 200_000,
            moving: false,
        },
    );
    assert!(run_game_scenario(&req, &ExecutionLimits::default()).is_err());
}

#[test]
fn heavy_selector_keeps_the_normal_boundary_and_excludes_the_dodge_expiry_boundary() {
    for (time, complete) in [(2_923_724, true), (2_923_725, false)] {
        let mut req = phase2_request();
        req.actions = vec![normal(0)];
        held(&mut req, time, 200_000, false);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["execution_completed"], complete);
    }
    for (time, complete) in [(2_499_999, true), (2_500_000, false)] {
        let mut req = phase2_request();
        req.actions.clear();
        dodge(&mut req);
        held(&mut req, time, 200_000, true);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["execution_completed"], complete);
        assert_eq!(report["combat_stamina"]["total_spent"], json!("15/1"));
    }
}

#[test]
fn heavy_does_not_renew_dodge_immunity_or_offer_a_free_dodge_without_the_word_model() {
    let mut req = phase2_request();
    req.actions.clear();
    dodge(&mut req);
    held(&mut req, 500_000, 200_000, true);
    let id = crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    req.incoming_damage_events
        .push(crate::simulation::GameActorDamageEvent {
            timestamp_us: 800_000,
            actor_index: 0,
            delivered_damage: 100,
        });
    assert_eq!(
        run(&req)["incoming_damage_receipts"][0]["actual_hp_loss"],
        100
    );
    req.combat_stamina = None;
    req.model_assumptions
        .remove(crate::HypothesisId::BaicangPowerWordsV1.as_str());
    req.model_assumptions
        .remove(crate::HypothesisId::BaicangRiposteV1.as_str());
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("stamina profile")
    );
}
