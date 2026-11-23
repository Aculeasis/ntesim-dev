use super::*;

#[test]
fn source_mixed_curves_survive_projection_native_execution_and_replay() {
    let (mut tables, original, mut projection) = action_fixture();
    projection.hypothesis = "Use exact selected native stat terms within one packet base".into();
    let packet = "GE_Player_Mint_AirAttack1_Damage";
    let row = &mut tables.get_mut("DT_SkillDamageData").unwrap()["rows"][packet];
    row["AtkRateBaseArray"] = json!(vec![0.2; 13]);
    row["AtkRateBaseArray"][7] = json!(0.6);
    row["AtkRateBaseArray"][12] = json!(0.9);
    row["DefRateBaseArray"] = json!(vec![0.3; 13]);
    for (index, numerator, denominator, expected) in
        [(0, 1, 5, 350), (7, 3, 5, 750), (12, 9, 10, 1050)]
    {
        let mut scenario = original.clone();
        let actor = &mut scenario.actors[0];
        actor.weapon_id.clear();
        actor.gear_procs.clear();
        actor.set_id = None;
        actor.static_stats.atk = Ratio {
            numerator: 1000,
            denominator: 1,
        };
        actor.static_stats.base_def = Some(500);
        actor.static_stats.def = 500;
        actor.static_stats.elemental_bonus = Ratio::zero();
        actor.static_stats.crit_rate = Ratio::zero();
        scenario.targets[0].defense = 0;
        scenario.targets[0].resistance = Ratio::zero();
        scenario.actions = [0, 1_000_000]
            .into_iter()
            .map(|timestamp_us| GameActionCommand::InvokeAbility {
                timestamp_us,
                actor_index: 0,
                ability_id: "GA_Mint019_Melee".into(),
                damage_effect_id: packet.into(),
                array_index: index,
                atk_rate: Ratio {
                    numerator: 999,
                    denominator: 1,
                },
                scaling_stat: crate::simulation::NativeScalingStat::Hp,
                energy_cost: 0,
                recovery_us: 0,
                cooldown_us: 0,
                hit_delay_us: 0,
            })
            .collect();
        let receipts = project_actions(&tables, &mut scenario, &projection).unwrap();
        let expected_stat = json!({"mixed":{
            "atk":format!("{}/{}", numerator, denominator),
            "def":"3/10",
            "hp":"0/1"
        }});
        assert_eq!(receipts[0]["compiled_scaling_stat"], expected_stat);
        assert_eq!(
            receipts[0]["compiled_atk_rate"],
            expected_stat["mixed"]["atk"]
        );
        assert_eq!(receipts[0]["compiled_rate"], json!("1/1"));
        assert_eq!(
            receipts[0]["native_damage_report"]["effects"][0]["arrays"]["HPRateBaseArray"]["state"],
            "declared_empty"
        );
        let report = run_game_scenario(&scenario, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["execution_completed"], true);
        assert_eq!(report["hits_settled"], 2);
        assert_eq!(
            report["total_damage_exact"],
            json!(format!("{}/{}", 2 * expected, 1))
        );
        for hit in report["hit_receipts"].as_array().unwrap() {
            assert_eq!(hit["non_crit_damage"], json!(format!("{}/{}", expected, 1)));
            assert_eq!(hit["scaling_stat"], expected_stat);
        }
        let restored: GameScenarioRequest =
            serde_json::from_value(serde_json::to_value(&scenario).unwrap()).unwrap();
        assert_eq!(
            run_game_scenario(&restored, &ExecutionLimits::default()).unwrap(),
            report
        );
    }
}

#[test]
fn supported_hp_mixed_projection_preserves_all_three_native_coefficients() {
    let (mut tables, _, _) = action_fixture();
    let packet = "GE_Player_Fadia_UltraSkill1_Damage";
    let row = &mut tables.get_mut("DT_SkillDamageData").unwrap()["rows"][packet];
    row["AtkRateBaseArray"] = json!([0.2]);
    row["DefRateBaseArray"] = json!([0.3]);
    let receipt = project_native_packet_coefficient(
        &tables,
        &PacketCoefficientRequest {
            content_id: "ntesim-current",
            character_id: "character:1039",
            ability_id: "GA_Fadia_UltraSkill",
            damage_effect_id: packet,
            array_index: 0,
            research_metadata: None,
            model_assumptions: &BTreeMap::new(),
        },
    )
    .unwrap();
    assert_eq!(
        receipt["compiled_scaling_stat"]["mixed"]["hp"],
        json!("39/1000")
    );
    assert_eq!(
        receipt["compiled_scaling_stat"]["mixed"]["atk"],
        json!("1/5")
    );
    assert_eq!(
        receipt["compiled_scaling_stat"]["mixed"]["def"],
        json!("3/10")
    );
}

#[test]
fn preview_and_purchased_level_projection_keep_mixed_weights_instead_of_an_atk_scalar() {
    let (mut tables, mut scenario, mut policy) = action_fixture();
    let packet = "GE_Player_Mint_Skill1_Damage_New";
    tables.get_mut("DT_SkillDamageData").unwrap()["rows"][packet]["DefRateBaseArray"] =
        json!(vec![0.3; 13]);
    let preview = super::super::preview_projection::project_preview_action(
        &tables,
        &super::super::preview_projection::PreviewActionRequest {
            content_id: "ntesim-current",
            character_id: "character:1019",
            ability_id: "GA_Mint019_Skill",
            damage_effect_id: packet,
            purchased_skill_level: 8,
            awakening: ntesim_data::AwakeningSelection::prefix(3),
            model_assumptions: &BTreeMap::new(),
        },
    )
    .unwrap();
    assert_eq!(preview["array_index"], 8);
    assert_eq!(preview["compiled_rate"], json!("1/1"));
    assert_eq!(
        preview["compiled_scaling_stat"]["mixed"]["atk"],
        json!("833/500")
    );
    assert_eq!(
        preview["compiled_scaling_stat"]["mixed"]["def"],
        json!("3/10")
    );

    scenario.actions = vec![scenario.actions[1].clone()];
    policy.policy = ActionProjectionPolicy::NativeSkillLevelV1;
    let levels = BTreeMap::from([(
        0,
        BTreeMap::from([(
            "GA_Mint019_Skill".into(),
            json!({"array_index":8,"effective_level":9}),
        )]),
    )]);
    let receipts = project_actions_with_levels(&tables, &mut scenario, &policy, &levels).unwrap();
    assert_eq!(
        receipts[0]["compiled_scaling_stat"],
        preview["compiled_scaling_stat"]
    );
    assert_eq!(receipts[0]["compiled_array_index"], 8);
}
