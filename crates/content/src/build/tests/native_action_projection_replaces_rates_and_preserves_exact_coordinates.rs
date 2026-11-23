use super::*;

#[test]
fn native_action_projection_replaces_rates_and_preserves_exact_coordinates() {
    let (tables, mut scenario, projection) = action_fixture();
    let GameActionCommand::InvokeAbility {
        atk_rate,
        scaling_stat,
        ability_id,
        damage_effect_id,
        ..
    } = &mut scenario.actions[0]
    else {
        panic!()
    };
    *atk_rate = Ratio::one();
    *scaling_stat = crate::simulation::NativeScalingStat::Def;
    *ability_id = ability_id.to_ascii_lowercase();
    *damage_effect_id = damage_effect_id.to_ascii_lowercase();
    let receipts = project_actions(&tables, &mut scenario, &projection).unwrap();
    assert_eq!(receipts.len(), 3);
    assert_eq!(receipts[0]["replaced_atk_rate"], json!("1/1"));
    assert_eq!(receipts[0]["compiled_atk_rate"], json!("89/500"));
    assert_eq!(receipts[0]["compiled_scaling_stat"], "atk");
    assert_eq!(receipts[0]["compiled_rate"], json!("89/500"));
    assert_eq!(receipts[1]["compiled_atk_rate"], json!("771/500"));
    assert_eq!(receipts[2]["compiled_atk_rate"], json!("2273/1000"));
    assert_eq!(
        receipts[0]["native_damage_report"]["effects"][0]["hit_count"],
        Value::Null
    );
    assert!(
        receipts[0]["native_damage_report"]
            .get("ability_cost")
            .is_none()
    );
}

#[test]
fn selected_zero_is_executable_without_inventing_an_absent_scaling_stat() {
    let (mut tables, mut scenario, projection) = action_fixture();
    let row = "GE_Player_Mint_AirAttack1_Damage";
    tables.get_mut("DT_SkillDamageData").unwrap()["rows"][row]["AtkRateBaseArray"] =
        json!(vec![0; 8]);
    scenario.actions.truncate(1);
    if let GameActionCommand::InvokeAbility {
        damage_effect_id, ..
    } = &mut scenario.actions[0]
    {
        *damage_effect_id = row.into();
    }
    let mut second = scenario.actions[0].clone();
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut second {
        *timestamp_us = 1_000_000;
    }
    scenario.actions.push(second);
    scenario.actors[0].initial_energy = 1;
    scenario.actors[0].weapon_id.clear();
    scenario.actors[0].gear_procs.clear();
    scenario.actors[0].set_id = None;
    let receipts = project_actions(&tables, &mut scenario, &projection).unwrap();
    assert_eq!(receipts.len(), 2);
    for receipt in &receipts {
        assert_eq!(receipt["compiled_scaling_stat"], "atk");
        assert_eq!(receipt["compiled_rate"], json!("0/1"));
        let arrays = &receipt["native_damage_report"]["effects"][0]["arrays"];
        assert_eq!(
            arrays["AtkRateBaseArray"]["state"],
            "selected_native_coordinate"
        );
        assert_eq!(arrays["DefRateBaseArray"]["state"], "declared_empty");
        assert!(arrays["DefRateBaseArray"]["selected_value"].is_null());
    }
    let report = run_game_scenario(&scenario, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true);
    assert_eq!(report["hits_settled"], 2);
    for hit in report["hit_receipts"].as_array().unwrap() {
        assert_eq!(hit["rounded_damage"], 0);
        assert_eq!(hit["applied_hp_loss"], 0);
    }
    assert_eq!(report["final_actor_states"][0]["energy"], 1);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 0);

    tables.get_mut("DT_SkillDamageData").unwrap()["rows"][row]["AtkRateBaseArray"] = json!([]);
    assert!(
        project_actions(&tables, &mut scenario, &projection)
            .unwrap_err()
            .contains("no selected ATK or DEF scaling")
    );
}

#[test]
fn explicit_zero_terms_do_not_mask_the_selected_def_or_hp_term() {
    let (tables, _, _) = action_fixture();
    for (character, ability, effect, stat, numerator, denominator) in [
        (
            "character:1033",
            "GA_Adler_UltraSkill",
            "GE_Player_Adler_UltraSkill_Damage",
            "def",
            1,
            1,
        ),
        (
            "character:1039",
            "GA_Fadia_UltraSkill",
            "GE_Player_Fadia_UltraSkill1_Damage",
            "hp",
            39,
            1000,
        ),
    ] {
        let mut changed = tables.clone();
        changed.get_mut("DT_SkillDamageData").unwrap()["rows"][effect]["AtkRateBaseArray"] =
            json!([0]);
        let receipt = project_native_packet_coefficient(
            &changed,
            &PacketCoefficientRequest {
                content_id: "ntesim-current",
                character_id: character,
                ability_id: ability,
                damage_effect_id: effect,
                array_index: 0,
                research_metadata: None,
                model_assumptions: &BTreeMap::new(),
            },
        )
        .unwrap();
        assert_eq!(receipt["compiled_scaling_stat"], stat);
        assert_eq!(
            receipt["compiled_rate"],
            json!(format!("{}/{}", numerator, denominator))
        );
        assert_eq!(receipt["compiled_atk_rate"], json!("0/1"));
    }
}
