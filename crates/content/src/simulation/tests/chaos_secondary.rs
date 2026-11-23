use super::blackbird_secondary::stamina;
use super::chaos_actions::make_action;
use super::*;
use serde_json::{Value, json};

pub(super) fn request(form: bool) -> GameScenarioRequest {
    let mut req = super::chaos_verdict::request(3, form);
    for model in [
        crate::HypothesisId::ChaosSecondaryActionsV1,
        crate::HypothesisId::ChaosJumpCancelV1,
    ] {
        req.model_assumptions
            .insert(model.into(), model.description().into());
    }
    stamina(&mut req, 100);
    req
}
fn normal(time: u64) -> GameActionCommand {
    make_action(
        time,
        0,
        "GA_Chaos071_Melee",
        "GE_Player_Chaos_Melee1_Damage",
        0,
        0,
    )
}
fn skill(time: u64) -> GameActionCommand {
    make_action(
        time,
        0,
        "GA_Chaos071_Skill",
        "GE_Player_Chaos_Skill1_Damage",
        0,
        0,
    )
}
fn dash(time: u64, critical: bool) -> GameActionCommand {
    GameActionCommand::PlayerInput {
        timestamp_us: time,
        actor_index: 0,
        input: crate::simulation::PlayerInput::Dash,
        critical_success: critical,
    }
}
fn plunge(req: &mut GameScenarioRequest, time: u64, height: Ratio) {
    req.plunge_inputs.insert(
        req.actions.len(),
        crate::simulation::GamePlungeInput {
            landing_delay_us: 600_000,
            height_bonus: height,
        },
    );
    req.actions.push(make_action(
        time,
        0,
        "GA_Chaos071_Melee",
        "GE_Player_Chaos_Airattack_Damage",
        0,
        0,
    ));
}
fn run(req: &GameScenarioRequest) -> Value {
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}
fn hits<'a>(report: &'a Value, effect: &str) -> Vec<&'a Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == effect)
        .collect()
}
fn crime(report: &Value) -> &Value {
    &report["final_actor_states"][0]["fractional_resources"]["chaos.crime"]
}
const RIPOSTE: &str = "GE_Player_Chaos_PerfectEvadeAttack1_Damage";

#[test]
fn chaos_paid_front_dodge_and_two_ripostes_keep_real_time_single_permission_and_aoe_channels() {
    let mut req = request(false);
    req.actions = vec![
        dash(0, true),
        normal(998_284),
        dash(2_000_000, true),
        normal(2_998_284),
    ];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let packets = hits(&report, RIPOSTE);
    assert_eq!(packets.len(), 6);
    assert_eq!(packets[0]["time_us"], 1_248_441);
    assert_eq!(packets[3]["time_us"], 3_248_441);
    assert_eq!(packets[0]["scaling_rate"], json!("53/50"));
    assert_eq!(packets[0]["non_crit_damage"], json!("530/1"));
    assert_eq!(packets[0]["command_index"], 1);
    assert_eq!(packets[0]["ability_id"], "GA_Chaos071_Melee");
    let packet_meta = support::hit_metadata(&report, packets[0]);
    assert_eq!(
        packet_meta["binding"]["packet_ga_name"],
        "GA_Chaos071_PerfectEvade"
    );
    assert_eq!(
        packet_meta["packet"]["native_damage_source_category"],
        "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Z"
    );
    assert_eq!(crime(&report)["current"], json!("200/1"));
    let actor = &report["final_actor_states"][0];
    assert_eq!(actor["energy"], 5);
    assert_eq!(actor["charge_remainder"], json!("2/5"));
    assert_eq!(actor["cycle"]["current"], 8);
    assert_eq!(actor["cycle"]["carry"], json!("4/5"));
    assert_eq!(report["combat_stamina"]["current"], json!("70/1"));
    req.actions[1] = normal(998_283);
    assert_eq!(run(&req)["rejected_action"]["reason"]["until_us"], 998_284);
}

#[test]
fn chaos_noncritical_dodge_and_consumed_or_replaced_permission_cannot_create_extra_riposte() {
    let mut req = request(false);
    req.actions = vec![dash(0, false), normal(998_284)];
    assert!(hits(&run(&req), RIPOSTE).is_empty());
    req.actions = vec![dash(0, true), normal(998_284), normal(1_661_491)];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(hits(&report, RIPOSTE).len(), 3);
    assert_eq!(
        hits(&report, "GE_Player_Chaos_Melee1_Damage")[0]["time_us"],
        1_876_064
    );
    assert_eq!(crime(&report)["current"], json!("122/1"));
    req.actions = vec![dash(0, true), skill(333_716), normal(1_388_710)];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert!(hits(&report, RIPOSTE).is_empty());
    assert_eq!(
        hits(&report, "GE_Player_Chaos_Melee1_Damage")[0]["time_us"],
        1_603_283
    );
}

#[test]
fn chaos_riposte_miss_has_no_pause_or_resources_and_cannot_be_called_without_paid_permission() {
    let mut req = request(false);
    for target in &mut req.targets {
        target.resistance = Ratio::one();
    }
    req.actions = vec![dash(0, true), normal(998_284), normal(1_364_491)];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["final_actor_states"][0]["energy"], 0);
    assert_eq!(crime(&report)["current"], json!("0/1"));
    assert_eq!(report["combat_stamina"]["current"], json!("85/1"));
    req.actions = vec![make_action(0, 0, "GA_Chaos071_Melee", RIPOSTE, 0, 0)];
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("owner-selected")
    );
    req.actions = vec![dash(0, true)];
    req.combat_stamina = None;
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("stamina")
    );
    stamina(&mut req, 14);
    let report = run(&req);
    assert_eq!(report["execution_completed"], false);
    assert_eq!(report["hits_settled"], 0);
}

#[test]
fn chaos_riposte_skill_cancel_keeps_committed_cost_and_only_caused_pause_in_the_new_cd_marker() {
    let mut req = request(false);
    req.actions = vec![dash(0, true), normal(998_284), skill(998_384)];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert!(hits(&report, RIPOSTE).is_empty());
    assert_eq!(crime(&report)["current"], json!("100/1"));
    req.actions[2] = skill(998_383);
    assert_eq!(run(&req)["rejected_action"]["reason"]["until_us"], 998_384);
    // Riposte connects at1.248441; its pause ends1.545441 after this early Skill begins.
    req.actions[2] = skill(1_248_442);
    req.actions.push(skill(16_789_080));
    req.relative_duration_us = 20_000_000;
    let report = run(&req);
    assert_eq!(report["execution_completed"], false);
    assert_eq!(
        hits(&report, "GE_Player_Chaos_Skill1_Damage")[0]["time_us"],
        1_843_721
    );
    assert_eq!(report["rejected_action"]["reason"]["until_us"], 16_789_081);
    *req.actions.last_mut().unwrap() = skill(16_789_081);
    assert_eq!(run(&req)["execution_completed"], true);
}

#[test]
fn chaos_riposte_marked_a1_gain_is_once_per_packet_and_form_uses_the_same_native_row() {
    let mut req = request(false);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([1]).unwrap();
    req.actions = vec![skill(0), dash(2_000_000, true), normal(2_998_284)];
    assert_eq!(crime(&run(&req))["current"], json!("280/1"));
    let mut req = request(true);
    req.actions
        .extend([dash(6_000_000, true), normal(6_998_284)]);
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(hits(&report, RIPOSTE)[0]["time_us"], 7_248_441);
    assert_eq!(
        support::hit_metadata(&report, hits(&report, RIPOSTE)[0])["binding"]["packet_ga_name"],
        "GA_Chaos071_PerfectEvade"
    );
}

#[test]
fn chaos_plunge_scales_only_coefficient_and_keeps_one_resource_group_across_all_dummies() {
    for (height, rate, damage) in [
        (Ratio::zero(), json!("9/20"), json!("225/1")),
        (
            Ratio {
                numerator: 1,
                denominator: 2,
            },
            json!("27/40"),
            json!("675/2"),
        ),
        (Ratio::one(), json!("9/10"), json!("450/1")),
    ] {
        let mut req = request(false);
        req.combat_stamina = None;
        req.actors[0].static_stats.break_intensity = Ratio::zero();
        req.targets[0].break_profile = Some(crate::simulation::GameTargetBreakProfileConfig {
            capacity: Ratio {
                numerator: 100,
                denominator: 1,
            },
            current_gauge: Ratio {
                numerator: 100,
                denominator: 1,
            },
            broken_duration_us: 1_000_000,
            discord_fraction: None,
        });
        plunge(&mut req, 0, height);
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 3);
        for hit in hits(&report, "GE_Player_Chaos_Airattack_Damage") {
            assert_eq!(hit["time_us"], 647_594);
            assert_eq!(hit["scaling_rate"], rate);
            assert_eq!(hit["non_crit_damage"], damage);
        }
        assert_eq!(crime(&report)["current"], json!("30/1"));
        assert_eq!(
            report["final_actor_states"][0]["charge_remainder"],
            json!("3/10")
        );
        assert_eq!(report["final_actor_states"][0]["cycle"]["current"], 3);
        assert_eq!(
            report["final_target_states"][0]["break_state"]["gauge"],
            json!("199/2")
        );
        req.actions.push(normal(759_288));
        assert_eq!(run(&req)["rejected_action"]["reason"]["until_us"], 759_289);
    }
}

#[test]
fn chaos_airborne_profile_is_required_and_captured_form_survives_landing_after_expiry() {
    let mut req = request(false);
    plunge(&mut req, 0, Ratio::one());
    req.plunge_inputs.get_mut(&0).unwrap().landing_delay_us = 0;
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("positive")
    );
    req.plunge_inputs.clear();
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("airborne")
    );
    let mut req = request(true);
    plunge(&mut req, 14_500_000, Ratio::one());
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let packets = hits(&report, "GE_Player_Chaos_Airattack_Damage_Ultra");
    assert_eq!(packets.len(), 3);
    assert_eq!(packets[0]["time_us"], 15_147_594);
    assert_eq!(crime(&report)["granted"], json!("1830/1"));
}

#[test]
fn chaos_plunge_combo_uses_real_hold_during_flight_and_strict_landing_progress() {
    for time in [720_000, 720_001] {
        let mut req = request(false);
        let model = crate::HypothesisId::ChaosNormalComboVerdictV1;
        req.model_assumptions
            .insert(model.into(), model.description().into());
        plunge(&mut req, 0, Ratio::one());
        req.hold_inputs.insert(
            1,
            crate::simulation::GameHoldInput {
                reserved_stocks: None,
                duration_us: 200_000,
                moving: false,
            },
        );
        req.actions.push(normal(time));
        let report = run(&req);
        assert_eq!(report["execution_completed"], time == 720_001);
        if time == 720_000 {
            assert_eq!(report["rejected_action"]["reason"]["until_us"], 720_001);
        } else {
            let packets = hits(&report, "GE_Player_Chaos_Branch1_Damage");
            assert_eq!(packets[0]["time_us"], 886_692);
            assert_eq!(packets[0]["scaling_rate"], json!("23/40"));
            assert_eq!(crime(&report)["current"], json!("30/1"));
        }
    }
}

#[test]
fn chaos_front_dodge_and_normal_movement_cancels_use_reached_exact_gates() {
    for (input, boundary) in [
        (crate::simulation::PlayerInput::Jump, 82_544),
        (crate::simulation::PlayerInput::Dash, 998_284),
    ] {
        for time in [boundary - 1, boundary] {
            let mut req = request(false);
            req.actions = vec![
                dash(0, false),
                GameActionCommand::PlayerInput {
                    timestamp_us: time,
                    actor_index: 0,
                    input,
                    critical_success: false,
                },
            ];
            assert_eq!(run(&req)["execution_completed"], time == boundary);
        }
    }
    for time in [333_715, 333_716] {
        let mut req = request(false);
        req.actions = vec![dash(0, true), skill(time)];
        assert_eq!(run(&req)["execution_completed"], time == 333_716);
    }
    for time in [493_254, 493_255] {
        let mut req = request(false);
        req.actions = vec![normal(0), dash(time, false)];
        let report = run(&req);
        assert_eq!(report["execution_completed"], time == 493_255);
        assert_eq!(report["hits_settled"], 3);
    }
}

#[test]
fn chaos_arc_consumes_riposte_permission_before_its_later_authored_normal() {
    let mut req = request(false);
    req.actors[0].weapon_id = "weapon:fork_jingmotingyuan".into();
    req.actors[0].gear_procs = vec![GearProcConfig::JingmotingyuanCrit {
        crit_damage_bonus: Ratio {
            numerator: 3,
            denominator: 25,
        },
        duration_us: 5_000_000,
    }];
    req.actions = vec![
        dash(0, true),
        GameActionCommand::PlayerInput {
            timestamp_us: 998_284,
            actor_index: 0,
            input: crate::simulation::PlayerInput::Arc,
            critical_success: false,
        },
        normal(4_000_000),
    ];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert!(hits(&report, "GE_Player_Chaos_PerfectEvadeAttack1_Damage").is_empty());
    assert_eq!(hits(&report, "GE_Player_Chaos_Melee1_Damage").len(), 3);
}

#[test]
fn chaos_plunge_zero_caller_delay_cannot_deliver_before_the_authored_landing() {
    let mut req = request(false);
    plunge(&mut req, 0, Ratio::one());
    if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut req.actions[0] {
        *hit_delay_us = 0;
    }
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hit_receipts"][0]["time_us"], 647_594);
}
