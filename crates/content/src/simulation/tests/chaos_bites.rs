use super::*;
use serde_json::json;

#[test]
fn chaos_bite_timing_requires_explicit_acceptance_in_strict() {
    let mut req = request();
    req.actors[0].talent_levels = None;
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::chaos::skill::SKILL_PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::chaos::skill::SKILL_PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::simulation::energy::CHAOS_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::CHAOS_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    assert_eq!(
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap()["hits_settled"],
        2
    );
}

fn request() -> GameScenarioRequest {
    let mut req = sample_s1_request();
    req.actors[0].character_id = "character:1071".into();
    req.actors[0].weapon_id.clear();
    req.actors[0].gear_procs.clear();
    req.actors[0].talent_levels = Some([1; 3]);
    req.actors[0].character_max_level = Some(80);
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.targets[0].resistance = Ratio::zero();
    req.actions = vec![GameActionCommand::InvokeAbility {
        timestamp_us: 0,
        actor_index: 0,
        ability_id: "GA_Chaos071_Skill".into(),
        damage_effect_id: "GE_Player_Chaos_Skill1_Damage".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 100,
    }];
    req
}

#[test]
fn chaos_two_bites_use_frozen_talent_coefficients_and_one_admission() {
    for (level, damages) in [(1, ["2145/4", "5655/4"]), (8, ["9191/10", "12116/5"])] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, level, 1]);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let hits = report["hit_receipts"].as_array().unwrap();
        assert_eq!(hits.len(), 2);
        for (index, hit) in hits.iter().enumerate() {
            assert_eq!(hit["time_us"], 100);
            assert_eq!(hit["command_index"], 0);
            assert_eq!(hit["non_crit_damage"], json!(damages[index]));
            assert_eq!(
                super::support::hit_metadata(&report, hit)["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
            );
        }
        assert_eq!(hits[0]["damage_effect_id"], "GE_Player_Chaos_Skill1_Damage");
        assert_eq!(hits[1]["damage_effect_id"], "GE_Player_Chaos_Skill2_Damage");
        assert_eq!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["event_kind"] == "invoke_ability")
                .count(),
            1
        );
        assert_eq!(
            report["resolved_model_assumptions"]
                [crate::characters::chaos::skill::SKILL_PACKETS_HYPOTHESIS_ID.as_str()],
            crate::characters::chaos::skill::SKILL_PACKETS_HYPOTHESIS_ID.description()
        );
    }
}

#[test]
fn chaos_child_inherits_effective_resonance_coordinate() {
    let mut req = request();
    req.actors[0].talent_levels = Some([1, 8, 1]);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(3);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["talent_projection_receipts"][0]["compiled_array_index"],
        8
    );
    // A2 DEF factor is 5/9. B4 Warrant uses 1.3 with Lv9 rates 1.527 and 4.026.
    assert_eq!(
        report["hit_receipts"][0]["non_crit_damage"],
        json!("6617/6")
    );
    assert_eq!(
        report["hit_receipts"][1]["non_crit_damage"],
        json!("8723/3")
    );
}

#[test]
fn chaos_bites_keep_pending_delivery_and_off_field_owner_and_skip_dead_target() {
    let mut req = request();
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    req.actors.push(ally);
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 50,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 2);
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|hit| hit["actor_index"] == 0)
    );
    req.relative_duration_us = 99;
    let short = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(short["execution_completed"], true);
    assert_eq!(short["pending_events"], 2);
    assert_eq!(short["hits_settled"], 0);
    req.relative_duration_us = 100;
    req.targets[0].hp_current = 1;
    let lethal = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(lethal["hits_settled"], 1);
    assert!(
        lethal["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_kind"] == "hit_target_unavailable")
    );
}

#[test]
fn chaos_failed_admission_adds_no_bite_and_bad_coordinates_or_primary_fail() {
    let mut req = request();
    if let GameActionCommand::InvokeAbility { energy_cost, .. } = &mut req.actions[0] {
        *energy_cost = 101;
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert_eq!(report["hits_settled"], 0);
    req = request();
    req.actors[0].talent_levels = None;
    if let GameActionCommand::InvokeAbility { array_index, .. } = &mut req.actions[0] {
        *array_index = 100;
    }
    assert!(run_game_scenario(&req, &ExecutionLimits::default()).is_err());
    req = request();
    if let GameActionCommand::InvokeAbility {
        damage_effect_id, ..
    } = &mut req.actions[0]
    {
        *damage_effect_id = "GE_Player_Chaos_Skill2_Damage".into();
    }
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("modeled first bite")
    );
    req = request();
    req.model_assumptions.insert(
        crate::characters::chaos::skill::SKILL_PACKETS_HYPOTHESIS_ID.into(),
        "conflict".into(),
    );
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("conflicting")
    );
}
