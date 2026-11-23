use super::*;
use crate::simulation::tests::support::formula_damage_total;
use serde_json::json;

fn make_action(
    timestamp_us: u64,
    actor_index: usize,
    ability_id: &str,
    damage_effect_id: &str,
    energy_cost: u64,
    cooldown_us: u64,
) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index,
        ability_id: ability_id.into(),
        damage_effect_id: damage_effect_id.into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost,
        recovery_us: 0,
        cooldown_us,
        hit_delay_us: 100,
    }
}

pub(super) fn request() -> GameScenarioRequest {
    let mut req = sample_s1_request();
    req.actors[0].character_id = "character:1073".into();
    req.actors[0].weapon_id.clear();
    req.actors[0].gear_procs.clear();
    req.actors[0].set_id = None;
    req.actors[0].talent_levels = Some([1; 3]);
    req.actors[0].character_max_level = Some(80);
    req.actors[0].character_level = 80;
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.targets[0].resistance = Ratio::zero();
    req.targets[0].level = 80;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Chiichan073_Melee",
        "GE_Player_Chiichan073_Melee1_Damage",
        0,
        0,
    )];
    req
}

#[test]
fn chiz_typed_multipliers_preserve_exact_fractional_capacity_and_acquisition() {
    let mut req = request();
    req.actors[0].character_level = 3;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([1]).unwrap();
    // Native level-three cap525 *1.10 =577.5, without a rounding policy.
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["final_actor_states"][0]["fractional_resources"]["chiz.grain"]["capacity"],
        json!("1155/2")
    );
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([4]).unwrap();
    // Native acquisition21 *1.20 =25.2 stays exact through two five-input chains.
    req.actions = (0..10)
        .map(|index| {
            make_action(
                index * 200_000,
                0,
                "GA_Chiichan073_Melee",
                "GE_Player_Chiichan073_Melee1_Damage",
                0,
                0,
            )
        })
        .collect();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let grain = &report["final_actor_states"][0]["fractional_resources"]["chiz.grain"];
    assert_eq!(grain["current"], json!("252/1"));
    assert_eq!(grain["granted"], json!("252/1"));
    assert_eq!(grain["conserved"], true);
}

#[test]
fn chiz_fractional_grain_preserves_the_last_partial_deposit_and_exact_overflow() {
    let mut req = request();
    req.actors[0].character_level = 3;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([1, 4]).unwrap();
    req.actions = (0..23)
        .map(|index| {
            make_action(
                index * 200_000,
                0,
                "GA_Chiichan073_Melee",
                "GE_Player_Chiichan073_Melee1_Damage",
                0,
                0,
            )
        })
        .collect();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let grain = &report["final_actor_states"][0]["fractional_resources"]["chiz.grain"];
    //23*25.2 =579.6, capacity577.5, final partial deposit23.1 and excess2.1.
    assert_eq!(grain["current"], json!("1155/2"));
    assert_eq!(grain["granted"], json!("2898/5"));
    assert_eq!(grain["overflow"], json!("21/10"));
    assert_eq!(grain["conserved"], true);
}

#[test]
fn chiz_normal_uses_native_talent_coefficients() {
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 345, 2),
        (8, 0, 591, 2),
        (8, 1, 591, 2),
        (8, 3, 591, 2),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([level, 1, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 1);
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            json!(format!("{}/{}", expected_num, expected_den))
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"
        );
    }
}

#[test]
fn chiz_normal_chain_settles_twelve_hits_and_loops_at_levels_one_and_eight() {
    for (level, awakening, expected_5_num, expected_5_den, expected_6_num, expected_6_den) in [
        (1, 0, 3257, 2, 1801, 1),
        (8, 0, 2790, 1, 6171, 2),
        (8, 3, 2790, 1, 6171, 2),
        (8, 4, 2790, 1, 6171, 2),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([level, 1, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = (0..5)
            .map(|index| {
                make_action(
                    index * 200_000,
                    0,
                    "GA_Chiichan073_Melee",
                    "GE_Player_Chiichan073_Melee1_Damage",
                    0,
                    0,
                )
            })
            .collect();
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 12);
        let per_input = if awakening == 4 { 540 } else { 450 };
        assert_eq!(
            report["final_actor_states"][0]["fractional_resources"]["chiz.grain"]["current"],
            json!(format!("{}/{}", 5 * per_input, 1))
        );
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            json!(format!("{}/{}", expected_5_num, expected_5_den))
        );

        let receipts = report["hit_receipts"].as_array().unwrap();
        assert_eq!(
            receipts[0]["damage_effect_id"],
            "GE_Player_Chiichan073_Melee1_Damage"
        );
        for item in &receipts[1..4] {
            assert_eq!(
                item["damage_effect_id"],
                "GE_Player_Chiichan073_Melee2_Damage"
            );
        }
        assert_eq!(
            receipts[4]["damage_effect_id"],
            "GE_Player_Chiichan073_Melee3_Damage"
        );
        for item in &receipts[5..10] {
            assert_eq!(
                item["damage_effect_id"],
                "GE_Player_Chiichan073_Melee4_Damage"
            );
        }
        assert_eq!(
            receipts[10]["damage_effect_id"],
            "GE_Player_Chiichan073_Melee5_Damage"
        );
        assert_eq!(
            receipts[11]["damage_effect_id"],
            "GE_Player_Chiichan073_Melee5_Damage"
        );

        req.actions.push(make_action(
            1_000_000,
            0,
            "GA_Chiichan073_Melee",
            "GE_Player_Chiichan073_Melee1_Damage",
            0,
            0,
        ));
        let loop_report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(loop_report["execution_completed"], true);
        assert_eq!(loop_report["hits_settled"], 13);
        assert_eq!(
            loop_report["final_actor_states"][0]["fractional_resources"]["chiz.grain"]["current"],
            json!(format!("{}/{}", 6 * per_input, 1))
        );
        assert_eq!(
            serde_json::json!(formula_damage_total(&loop_report)),
            json!(format!("{}/{}", expected_6_num, expected_6_den))
        );
        assert_eq!(
            loop_report["hit_receipts"][12]["damage_effect_id"],
            "GE_Player_Chiichan073_Melee1_Damage"
        );
    }
}

#[test]
fn chiz_normal_chain_resets_on_ability_gap_and_field_exit() {
    let mut req = request();
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Chiichan073_Melee",
            "GE_Player_Chiichan073_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            200_000,
            0,
            "GA_Chiichan073_Skill",
            "GE_Player_Chiichan073_Skill1_1Damage",
            0,
            0,
        ),
        make_action(
            400_000,
            0,
            "GA_Chiichan073_Melee",
            "GE_Player_Chiichan073_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 3);
    assert_eq!(
        report["hit_receipts"][0]["damage_effect_id"],
        "GE_Player_Chiichan073_Melee1_Damage"
    );
    assert_eq!(
        report["hit_receipts"][1]["damage_effect_id"],
        "GE_Player_Chiichan073_Skill1_1Damage"
    );
    assert_eq!(
        report["hit_receipts"][2]["damage_effect_id"],
        "GE_Player_Chiichan073_Melee1_Damage"
    );

    let mut gap_req = request();
    gap_req.actions = vec![
        make_action(
            0,
            0,
            "GA_Chiichan073_Melee",
            "GE_Player_Chiichan073_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            2_100_000,
            0,
            "GA_Chiichan073_Melee",
            "GE_Player_Chiichan073_Melee1_Damage",
            0,
            0,
        ),
    ];
    let gap_report = run_game_scenario(&gap_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(gap_report["hits_settled"], 2);
    assert_eq!(
        gap_report["hit_receipts"][0]["damage_effect_id"],
        "GE_Player_Chiichan073_Melee1_Damage"
    );
    assert_eq!(
        gap_report["hit_receipts"][1]["damage_effect_id"],
        "GE_Player_Chiichan073_Melee1_Damage"
    );

    let mut switch_req = request();
    switch_req.actions = vec![
        make_action(
            0,
            0,
            "GA_Chiichan073_Melee",
            "GE_Player_Chiichan073_Melee1_Damage",
            0,
            0,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 100_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        make_action(
            200_000,
            0,
            "GA_Chiichan073_Melee",
            "GE_Player_Chiichan073_Melee1_Damage",
            0,
            0,
        ),
    ];
    let switch_report = run_game_scenario(&switch_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(switch_report["hits_settled"], 4);
    assert_eq!(
        switch_report["hit_receipts"][1]["damage_effect_id"],
        "GE_Player_Chiichan073_Melee2_Damage"
    );
}

#[test]
fn chiz_normal_chain_strict_mode_acceptance() {
    let mut strict_req = request();
    let passive = crate::characters::chiz::passives::MODEL;
    strict_req
        .model_assumptions
        .insert(passive.into(), passive.description().into());
    let model = crate::characters::chiz::resource::MODEL;
    strict_req
        .model_assumptions
        .insert(model.into(), model.description().into());
    strict_req.model_assumptions.insert(
        crate::simulation::energy::CHIZ_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::CHIZ_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    strict_req.actors[0].talent_levels = None;
    strict_req.actors[0].purchased_skill_level = 1;
    strict_req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    strict_req.model_assumptions.insert(
        crate::characters::chiz::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::chiz::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    strict_req.model_assumptions.insert(
        crate::HypothesisId::ChizNormalGrainV1.into(),
        crate::HypothesisId::ChizNormalGrainV1.description().into(),
    );
    let report = run_game_scenario(&strict_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 1);
}

#[test]
fn chiz_skill_multi_target_hits_all_dummies() {
    let mut req = request();
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Chiichan073_Skill",
        "GE_Player_Chiichan073_Skill1_1Damage",
        0,
        0,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 2);
    let receipts = report["hit_receipts"].as_array().unwrap();
    assert_eq!(receipts[0]["target_index"], 0);
    assert_eq!(
        receipts[0]["damage_effect_id"],
        "GE_Player_Chiichan073_Skill1_1Damage"
    );
    assert_eq!(receipts[1]["target_index"], 1);
    assert_eq!(
        receipts[1]["damage_effect_id"],
        "GE_Player_Chiichan073_Skill1_1Damage"
    );
}

#[test]
fn chiz_skill_multi_target_strict_acceptance() {
    let mut req = request();
    let passive = crate::characters::chiz::passives::MODEL;
    req.model_assumptions
        .insert(passive.into(), passive.description().into());
    let model = crate::characters::chiz::resource::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 1;
    req.model_assumptions.insert(
        crate::simulation::energy::CHIZ_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::CHIZ_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actions = vec![make_action(
        0,
        0,
        "GA_Chiichan073_Skill",
        "GE_Player_Chiichan073_Skill1_1Damage",
        0,
        0,
    )];
    req.model_assumptions.insert(
        crate::characters::chiz::skill::ALL_IN_RANGE_HYPOTHESIS_ID.into(),
        crate::characters::chiz::skill::ALL_IN_RANGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 2);
}

#[test]
fn chiz_normal_grain_requires_positive_rounded_hp_loss_and_fatal_damage_qualifies() {
    let mut tiny = request();
    tiny.actors[0].static_stats.atk = Ratio {
        numerator: 1,
        denominator: 1000,
    };
    let report = run_game_scenario(&tiny, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hit_receipts"][0]["applied_hp_loss"], 0);
    assert_eq!(
        report["final_actor_states"][0]["fractional_resources"]["chiz.grain"]["current"],
        json!("0/1")
    );

    let mut fatal = request();
    fatal.targets[0].hp_current = 1;
    let report = run_game_scenario(&fatal, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hit_receipts"][0]["applied_hp_loss"], 1);
    assert_eq!(
        report["final_actor_states"][0]["fractional_resources"]["chiz.grain"]["current"],
        json!("450/1")
    );
}
