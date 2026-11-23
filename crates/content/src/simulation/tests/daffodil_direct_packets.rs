use super::daffodil_actions::{make_action, request};
use super::*;
use serde_json::json;

#[test]
fn daffodil_branch_and_entry_packets_are_bounded_direct_inputs() {
    let cases = [
        ("GE_Player_Daffodill_Branch1_Damage", [116, 199, 215]),
        ("GE_Player_Daffodill_Branch2_Damage", [96, 165, 178]),
        ("GE_Player_Daffodill_Branch3_Damage", [389, 667, 720]),
        ("GE_Player_Daffodill_Branch4_Damage", [113, 194, 209]),
        ("GE_Player_Daffodill_Branch5_Damage", [158, 271, 292]),
        ("GE_Player_Daffodill_Branch6_Damage", [96, 165, 178]),
        ("GE_Player_Daffodill_Branch7_Damage", [96, 165, 178]),
        ("GE_Player_Daffodill_EntryAttack1_Damage", [681, 1167, 1260]),
        ("GE_Player_Daffodill_EntryAttack2_Damage", [553, 948, 1024]),
        (
            "GE_Player_Daffodill_EntryAttack4_Damage",
            [1105, 1894, 2045],
        ),
    ];

    for (damage_effect_id, coefficients) in cases {
        for (coordinate_index, level) in [(0, 1), (1, 8), (2, 9)] {
            let mut req = request();
            req.actors[0].talent_levels = Some([level, 1, 1]);
            req.actors[0].initial_energy = 0;
            req.actions = vec![make_action(
                0,
                0,
                "GA_Daffodill_Melee",
                damage_effect_id,
                0,
                0,
            )];

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
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"
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
                    [crate::characters::daffodil::skill::DIRECT_PACKET_INPUTS_HYPOTHESIS_ID
                        .as_str()],
                crate::characters::daffodil::skill::DIRECT_PACKET_INPUTS_HYPOTHESIS_ID
                    .description()
            );
            assert_eq!(
                report["resolved_model_assumptions"]
                    [crate::simulation::energy::DAFFODIL_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID
                        .as_str()],
                crate::simulation::energy::DAFFODIL_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID
                    .description()
            );
            assert!(
                !report["resolved_model_assumptions"]
                    .as_object()
                    .unwrap()
                    .contains_key(
                        crate::simulation::energy::DAFFODIL_HIT_ENERGY_HYPOTHESIS_ID.as_str()
                    )
            );
            assert!(
                report["trace"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|event| { event["event_kind"] != "daffodil_ordinary_hit_energy_grant" })
            );
            assert_eq!(
                report["trace"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|event| event["event_kind"] == "daffodil_direct_packet_energy_grant")
                    .count(),
                1
            );
        }
    }

    let mut resonance = request();
    resonance.actors[0].talent_levels = Some([8, 1, 1]);
    resonance.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(3);
    resonance.actions = vec![make_action(
        0,
        0,
        "GA_Daffodill_Melee",
        "GE_Player_Daffodill_Branch3_Damage",
        0,
        0,
    )];
    let report = run_game_scenario(&resonance, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hit_receipts"][0]["non_crit_damage"], json!("360/1"));
}

#[test]
fn daffodil_direct_branch_packet_does_not_change_normal_chain_stage() {
    let mut req = request();
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Daffodill_Melee",
            "GE_Player_Daffodill_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            100_000,
            0,
            "GA_Daffodill_Melee",
            "GE_Player_Daffodill_Branch1_Damage",
            0,
            0,
        ),
        make_action(
            200_000,
            0,
            "GA_Daffodill_Melee",
            "GE_Player_Daffodill_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 4);
    let receipts = report["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts
            .iter()
            .map(|receipt| receipt["damage_effect_id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "GE_Player_Daffodill_Melee1_Damage",
            "GE_Player_Daffodill_Branch1_Damage",
            "GE_Player_Daffodill_Melee2_Damage",
            "GE_Player_Daffodill_Melee2_1_Damage",
        ]
    );
}

#[test]
fn daffodil_direct_branch_packet_uses_shared_break_only_with_explicit_profile() {
    let mut req = request();
    req.targets[0].break_profile = Some(crate::simulation::GameTargetBreakProfileConfig {
        capacity: Ratio::one(),
        current_gauge: Ratio {
            numerator: 148,
            denominator: 1000,
        },
        broken_duration_us: 1_000_000,
        discord_fraction: None,
    });
    req.relative_duration_us = 500;
    req.actions[0] = make_action(
        0,
        0,
        "GA_Daffodill_Melee",
        "GE_Player_Daffodill_Branch1_Damage",
        0,
        0,
    );

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 2);
    assert!(report["hit_receipts"][1]["break_contribution"].is_object());
    assert_eq!(
        report["resolved_model_assumptions"]
            [crate::characters::daffodil::skill::DIRECT_PACKET_INPUTS_HYPOTHESIS_ID.as_str()],
        crate::characters::daffodil::skill::DIRECT_PACKET_INPUTS_HYPOTHESIS_ID.description()
    );
    assert_eq!(
        report["resolved_model_assumptions"]
            [crate::simulation::break_state::HYPOTHESIS_ID.as_str()],
        crate::simulation::break_state::HYPOTHESIS_ID.description()
    );
    assert_eq!(
        report["final_target_states"][0]["break_state"]["gauge"],
        json!("0/1")
    );
    assert_eq!(
        report["final_target_states"][0]["break_state"]["broken_until_us"],
        1_000_100
    );
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "target_broken")
            .count(),
        1
    );
}

#[test]
fn daffodil_direct_branch_packet_requires_its_strict_hypothesis() {
    let mut req = request();
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 1;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Daffodill_Melee",
        "GE_Player_Daffodill_Branch1_Damage",
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
        crate::characters::daffodil::skill::DIRECT_PACKET_INPUTS_HYPOTHESIS_ID.into(),
        crate::characters::daffodil::skill::DIRECT_PACKET_INPUTS_HYPOTHESIS_ID
            .description()
            .into(),
    );

    req.model_assumptions.insert(
        crate::simulation::energy::DAFFODIL_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::DAFFODIL_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID
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
fn daffodil_direct_packet_boundary_rejects_unqualified_entry_and_wrong_ability() {
    let mut unsupported_entry = request();
    unsupported_entry.actions = vec![make_action(
        0,
        0,
        "GA_Daffodill_Melee",
        "GE_Player_Daffodill_EntryAttack3_Damage",
        0,
        0,
    )];
    let error = run_game_scenario(&unsupported_entry, &ExecutionLimits::default()).unwrap_err();
    assert!(
        error.contains("Daffodil Normal chain requires Melee1 as its admitted primary packet"),
        "{error}"
    );

    let mut wrong_ability = request();
    wrong_ability.actions = vec![make_action(
        0,
        0,
        "GA_Daffodill_Skill",
        "GE_Player_Daffodill_Branch1_Damage",
        0,
        0,
    )];
    let error = run_game_scenario(&wrong_ability, &ExecutionLimits::default()).unwrap_err();
    assert!(
        error.contains("unsupported action/packet relationship"),
        "{error}"
    );
}
