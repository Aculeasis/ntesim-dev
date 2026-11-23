use super::hathor_actions::{make_action, request};
use super::*;
use serde_json::json;

#[test]
fn hathor_branch_and_airattack_are_bounded_direct_packets() {
    let cases = [
        (
            "GE_Player_Hathor_Branch_Damage",
            [2400, 4113, 4442],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A",
        ),
        (
            "GE_Player_Hathor_AirAttack_Damage",
            [1250, 2142, 2314],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_X",
        ),
    ];

    for (damage_effect_id, coefficients, category) in cases {
        for (coordinate_index, level) in [(0, 1), (1, 8), (2, 9)] {
            let mut req = request();
            req.actors[0].talent_levels = Some([level, 1, 1]);
            req.actors[0].static_stats.damage_bonus = Ratio::zero();
            req.actors[0].static_stats.elemental_bonuses.clear();
            req.actors[0].initial_energy = 0;
            req.actions = vec![make_action(0, 0, "GA_Hathor_Melee", damage_effect_id, 0, 0)];

            let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
            assert_eq!(
                report["execution_completed"], true,
                "{} {}",
                report["halt_diagnostic"], report["rejected_action"]
            );
            assert_eq!(report["hits_settled"], 1);
            let hit = &report["hit_receipts"][0];
            assert_eq!(hit["time_us"], 100);
            assert_eq!(hit["damage_effect_id"], damage_effect_id);
            assert_eq!(
                super::support::hit_metadata(&report, hit)["packet"]["native_damage_source_category"],
                category
            );
            let coefficient_milli = coefficients[coordinate_index];
            let expected = if coefficient_milli % 2 == 0 {
                json!(format!("{}/{}", coefficient_milli / 2, 1))
            } else {
                json!(format!("{}/{}", coefficient_milli, 2))
            };
            assert_eq!(
                hit["non_crit_damage"], expected,
                "{damage_effect_id} Lv{level}"
            );
            assert_eq!(
                report["resolved_model_assumptions"]
                    [crate::characters::hathor::skill::DIRECT_PACKET_INPUTS_HYPOTHESIS_ID.as_str()],
                crate::characters::hathor::skill::DIRECT_PACKET_INPUTS_HYPOTHESIS_ID.description()
            );
            assert_eq!(
                report["resolved_model_assumptions"]
                    [crate::simulation::energy::HATHOR_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID.as_str()],
                crate::simulation::energy::HATHOR_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID.description()
            );
            assert!(
                !report["resolved_model_assumptions"]
                    .as_object()
                    .unwrap()
                    .contains_key(
                        crate::simulation::energy::HATHOR_HIT_ENERGY_HYPOTHESIS_ID.as_str()
                    )
            );
            let expected_energy = if damage_effect_id.ends_with("Branch_Damage") {
                12
            } else {
                6
            };
            assert_eq!(report["final_actor_states"][0]["energy"], expected_energy);
            assert_eq!(
                report["final_actor_states"][0]["charge_remainder"],
                if damage_effect_id.ends_with("Branch_Damage") {
                    serde_json::Value::Null
                } else {
                    json!("3/10")
                }
            );
            let grants: Vec<_> = report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["event_kind"] == "hathor_direct_packet_energy_grant")
                .collect();
            assert_eq!(grants.len(), 1);
            assert_eq!(grants[0]["energy_change"]["source"], "hathor_hit");
            assert_eq!(grants[0]["energy_change"]["source_actor_index"], 0);
            assert_eq!(grants[0]["energy_change"]["recipient_actor_index"], 0);
        }
    }
}

#[test]
fn hathor_direct_packets_leave_the_normal_chain_unchanged() {
    for direct_packet in [
        "GE_Player_Hathor_Branch_Damage",
        "GE_Player_Hathor_AirAttack_Damage",
    ] {
        let mut req = request();
        req.actions = vec![
            make_action(
                0,
                0,
                "GA_Hathor_Melee",
                "GE_Player_Hathor_Melee1_Damage",
                0,
                0,
            ),
            make_action(100_000, 0, "GA_Hathor_Melee", direct_packet, 0, 0),
            make_action(
                200_000,
                0,
                "GA_Hathor_Melee",
                "GE_Player_Hathor_Melee1_Damage",
                0,
                0,
            ),
        ];

        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["hits_settled"], 7);
        let receipts = report["hit_receipts"].as_array().unwrap();
        assert_eq!(
            receipts
                .iter()
                .map(|receipt| receipt["damage_effect_id"].as_str().unwrap())
                .collect::<Vec<_>>(),
            [
                "GE_Player_Hathor_Melee1_Damage",
                direct_packet,
                "GE_Player_Hathor_Melee2_Damage",
                "GE_Player_Hathor_Melee2_1_Damage",
                "GE_Player_Hathor_Melee2_1_Damage",
                "GE_Player_Hathor_Melee2_1_Damage",
                "GE_Player_Hathor_Melee2_1_Damage",
            ]
        );
    }
}

#[test]
fn hathor_direct_packet_inputs_require_their_strict_hypothesis() {
    let mut req = request();
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 1;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Hathor_Melee",
        "GE_Player_Hathor_Branch_Damage",
        0,
        0,
    )];
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::hathor::skill::DIRECT_PACKET_INPUTS_HYPOTHESIS_ID.into(),
        crate::characters::hathor::skill::DIRECT_PACKET_INPUTS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::simulation::energy::HATHOR_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::HATHOR_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 1);
}

#[test]
fn hathor_direct_packet_unbal_uses_only_the_explicit_shared_target_profile() {
    let cases = [
        ("GE_Player_Hathor_Branch_Damage", json!("13/5")),
        ("GE_Player_Hathor_AirAttack_Damage", json!("37/10")),
    ];
    for (packet, expected_gauge) in cases {
        let mut req = request();
        req.targets[0].break_profile = Some(crate::simulation::GameTargetBreakProfileConfig {
            capacity: Ratio {
                numerator: 10,
                denominator: 1,
            },
            current_gauge: Ratio {
                numerator: 5,
                denominator: 1,
            },
            broken_duration_us: 1_000_000,
            discord_fraction: None,
        });
        req.actions = vec![make_action(0, 0, "GA_Hathor_Melee", packet, 0, 0)];

        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["final_target_states"][0]["break_state"]["gauge"],
            expected_gauge
        );
        assert_eq!(
            report["resolved_model_assumptions"]
                [crate::simulation::break_state::HYPOTHESIS_ID.as_str()],
            crate::simulation::break_state::HYPOTHESIS_ID.description()
        );
    }
}

#[test]
fn hathor_direct_packet_boundary_rejects_other_packets_and_abilities() {
    let mut unsupported_packet = request();
    unsupported_packet.actions = vec![make_action(
        0,
        0,
        "GA_Hathor_Melee",
        "GE_Player_Hathor_Melee2_Damage",
        0,
        0,
    )];
    let error = run_game_scenario(&unsupported_packet, &ExecutionLimits::default()).unwrap_err();
    assert!(
        error.contains("requires the modeled first packet as its admitted input"),
        "{error}"
    );

    let mut wrong_ability = request();
    wrong_ability.actions = vec![make_action(
        0,
        0,
        "GA_Hathor_Skill",
        "GE_Player_Hathor_Branch_Damage",
        0,
        0,
    )];
    let error = run_game_scenario(&wrong_ability, &ExecutionLimits::default()).unwrap_err();
    assert!(
        error.contains("unsupported action/packet relationship"),
        "{error}"
    );
}
