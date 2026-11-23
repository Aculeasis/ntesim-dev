use super::*;

fn scenario() -> GameScenarioRequest {
    let mut req = phase2_request();
    let id = crate::HypothesisId::BaicangCancelWindowsV1;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    req
}

fn jump(time: u64) -> GameActionCommand {
    GameActionCommand::PlayerInput {
        timestamp_us: time,
        actor_index: 0,
        input: PlayerInput::Jump,
        critical_success: false,
    }
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
fn normal_skill_cancel_uses_its_own_exact_window_without_unlocking_other_inputs() {
    for (stage, start, offset) in [
        (1, 0, 378_894),
        (2, 1_000_000, 379_190),
        (3, 2_000_000, 477_995),
    ] {
        for early in [true, false] {
            let mut req = scenario();
            req.actions = [0, 1_000_000, 2_000_000]
                .into_iter()
                .take(stage)
                .map(normal)
                .collect();
            req.actions.push(skill(start + offset - u64::from(early)));
            let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
            assert_eq!(report["execution_completed"], !early);
            if early {
                assert_eq!(
                    report["rejected_action"]["reason"],
                    json!({"kind":"recovery","until_us":start+offset})
                );
            } else {
                assert_eq!(report["hits_settled"], stage + 1);
                assert_eq!(
                    report["hit_receipts"][stage]["time_us"],
                    start + offset + 668_070
                );
            }
            req.actions[stage] = normal(start + offset);
            assert_eq!(
                run_game_scenario(&req, &ExecutionLimits::default()).unwrap()["execution_completed"],
                false
            );
        }
    }
}

#[test]
fn jump_cancel_preserves_spawned_heavy_explosions_and_arms_the_correct_derived_skill() {
    for (packet, window, times) in [
        ("GE_Player_Cang_Branch1_Damage", 506_294, vec![736_667; 3]),
        (
            "GE_Player_Cang_Branch2_Damage",
            300_989,
            vec![600_174, 617_079, 633_235],
        ),
    ] {
        for early in [true, false] {
            let mut req = scenario();
            req.actions = vec![
                make_action(0, 0, "GA_Cang_Melee", packet, 0, 0),
                jump(window - u64::from(early)),
                skill(window + 1),
            ];
            let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
            assert_eq!(report["execution_completed"], !early);
            if !early {
                let hits = report["hit_receipts"].as_array().unwrap();
                assert_eq!(hits.len(), 5);
                assert_eq!(
                    hits[1..4]
                        .iter()
                        .map(|hit| hit["time_us"].as_u64().unwrap())
                        .collect::<Vec<_>>(),
                    times
                );
                assert_eq!(hits[4]["damage_effect_id"], "GE_Player_Cang_SkillB_Damage");
                assert_eq!(report["final_actor_states"][0]["hp"], 9700);
                assert!(report["combat_stamina"].is_null());
            }
        }
    }
}

#[test]
fn default_skill_actor_survives_jump_and_still_owns_its_original_cooldown() {
    let mut req = scenario();
    req.actions = vec![skill(0), jump(249_464), normal(249_465)];
    let report = run(&req);
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0]["damage_effect_id"], "GE_Player_Cang_Melee1_Damage");
    assert_eq!(hits[1]["damage_effect_id"], "GE_Player_Cang_Skill_Damage");
    assert_eq!(hits[1]["time_us"], 668_070);
    req.actions.push(skill(13_999_999));
    let failed = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(failed["execution_completed"], false);
    assert_eq!(
        failed["rejected_action"]["reason"],
        json!({"kind":"cooldown","until_us":14_000_000})
    );
}

#[test]
fn derived_skill_projectiles_and_one_word_survive_jump_without_a_second_stamina_cost() {
    let mut req = scenario();
    req.actions.clear();
    dodge(&mut req);
    req.actions
        .extend([skill(500_000), jump(747_532), normal(747_533)]);
    let report = run(&req);
    assert_eq!(report["hits_settled"], 9);
    assert_eq!(report["final_actor_states"][0]["hp"], 9700);
    assert_eq!(report["combat_stamina"]["total_spent"], json!("15/1"));
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(
        hits.last().unwrap()["damage_effect_id"],
        "GE_Player_Cang_SkillA_Damage1"
    );
    assert_eq!(hits.last().unwrap()["time_us"], 1_016_575);
}

#[test]
fn jump_does_not_cancel_the_burst_field_or_refund_its_energy() {
    let mut req = scenario();
    req.actors[0].initial_energy = 120;
    req.actors[0].energy_capacity = 120;
    req.actions = vec![burst(0), jump(2_871_112), normal(3_000_000)];
    let report = run(&req);
    assert_eq!(report["hits_settled"], 8); // expansion, one Normal, six continuing domain ticks
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 120);
}

#[test]
fn missing_models_and_inactive_or_expired_windows_cannot_create_free_jump_actions() {
    let mut req = scenario();
    req.actions = vec![jump(0)];
    assert_eq!(
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap()["execution_completed"],
        false
    );
    req.actions = vec![skill(0), jump(961_683)];
    assert_eq!(
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap()["execution_completed"],
        false
    );
    req.model_assumptions
        .remove(crate::HypothesisId::BaicangPowerWordsV1.as_str());
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires baicang.power_words")
    );
}

#[test]
fn continuing_cancel_cycles_keep_word_and_fractional_resource_ledgers() {
    let mut req = scenario();
    req.actors[0].initial_energy = 0;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actions.clear();
    for base in [0, 20_000_000] {
        req.actions.extend([
            normal(base),
            skill(base + 378_894),
            jump(base + 628_358),
            normal(base + 628_359),
        ]);
    }
    let report = run(&req);
    assert_eq!(report["hits_settled"], 6);
    assert_eq!(report["total_damage_exact"], json!("2580/1"));
    let actor = &report["final_actor_states"][0];
    assert_eq!(actor["hp"], 9700);
    assert_eq!(actor["granted_energy"], 15);
    assert_eq!(actor["charge_remainder"], json!("207/250"));
    assert_eq!(actor["cycle"]["granted"], 26);
    assert_eq!(actor["cycle"]["carry"], json!("71/250"));
}

#[test]
fn held_branch_jump_counts_hold_time_and_ends_derived_selection_persistence_at_jump() {
    for (skill_time, packet) in [
        (3_706_293, "GE_Player_Cang_SkillB_Damage"),
        (3_706_294, "GE_Player_Cang_Skill_Damage"),
    ] {
        let mut req = scenario();
        req.actions = vec![
            normal(0),
            normal(1_000_000),
            jump(1_706_294),
            skill(skill_time),
        ];
        req.hold_inputs.insert(
            1,
            crate::simulation::GameHoldInput {
                reserved_stocks: None,
                duration_us: 200_000,
                moving: false,
            },
        );
        let report = run(&req);
        assert_eq!(report["hit_receipts"][1]["time_us"], 1_616_667);
        assert_eq!(report["hit_receipts"][5]["damage_effect_id"], packet);
    }
}
