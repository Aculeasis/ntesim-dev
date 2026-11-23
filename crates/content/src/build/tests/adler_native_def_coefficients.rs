use super::*;

#[test]
fn adler_primary_skill_and_burst_project_native_def_coefficients() {
    let (tables, _, _) = action_fixture();
    let assumptions = BTreeMap::from([(
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_ID
            .description()
            .into(),
    )]);
    for (ability, effect, index, expected) in [
        (
            "GA_Adler_Skill",
            "GE_Player_Adler_Skill2_Damage",
            0,
            (7, 10),
        ),
        ("GA_Adler_Skill", "GE_Player_Adler_Skill2_Damage", 7, (6, 5)),
        (
            "GA_Adler_Skill",
            "GE_Player_Adler_Skill2_Damage",
            8,
            (162, 125),
        ),
        (
            "GA_Adler_UltraSkill",
            "GE_Player_Adler_UltraSkill_Damage",
            0,
            (1, 1),
        ),
        (
            "GA_Adler_UltraSkill",
            "GE_Player_Adler_UltraSkill_Damage",
            7,
            (857, 500),
        ),
        (
            "GA_Adler_UltraSkill",
            "GE_Player_Adler_UltraSkill_Damage",
            8,
            (1851, 1000),
        ),
    ] {
        let projection = project_native_packet_coefficient(
            &tables,
            &PacketCoefficientRequest {
                content_id: "ntesim-current",
                character_id: "character:1033",
                ability_id: ability,
                damage_effect_id: effect,
                array_index: index,
                research_metadata: None,
                model_assumptions: &assumptions,
            },
        )
        .unwrap();
        assert_eq!(
            projection["compiled_scaling_stat"],
            json!(crate::simulation::NativeScalingStat::Def)
        );
        assert_eq!(
            projection["compiled_rate"],
            json!(format!("{}/{}", expected.0, expected.1))
        );
        assert_eq!(
            projection["native_damage_report"]["effects"][0]["arrays"]["AtkRateBaseArray"]["state"],
            "declared_empty"
        );
    }
}

#[test]
fn adler_native_projection_preserves_mixed_terms_and_rejects_unsupported_hp() {
    let (mut tables, _, _) = action_fixture();
    let assumptions = BTreeMap::from([(
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID.into(),
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID
            .description()
            .into(),
    )]);
    let request = PacketCoefficientRequest {
        content_id: "ntesim-current",
        character_id: "character:1033",
        ability_id: "GA_Adler_Skill",
        damage_effect_id: "GE_Player_Adler_Skill2_Damage",
        array_index: 0,
        research_metadata: None,
        model_assumptions: &assumptions,
    };
    tables.get_mut("DT_SkillDamageData").unwrap()["rows"][request.damage_effect_id]["AtkRateBaseArray"] =
        json!([1, 1, 1, 1, 1, 1, 1, 1, 1]);
    let mixed = project_native_packet_coefficient(&tables, &request).unwrap();
    assert_eq!(
        mixed["compiled_scaling_stat"],
        json!({"mixed":{
            "atk":"1/1",
            "def":"7/10",
            "hp":"0/1"
        }})
    );
    assert_eq!(mixed["compiled_rate"], json!("1/1"));

    let (mut tables, _, _) = action_fixture();
    tables.get_mut("DT_SkillDamageData").unwrap()["rows"][request.damage_effect_id]["HPRateBaseArray"] =
        json!([1, 1, 1, 1, 1, 1, 1, 1, 1]);
    assert!(
        project_native_packet_coefficient(&tables, &request)
            .unwrap_err()
            .contains("HP scaling is unsupported")
    );
}

#[test]
fn native_action_projection_overwrites_forged_adler_stat_and_rate() {
    let (tables, mut scenario, projection) = action_fixture();
    scenario.actors[0].character_id = "character:1033".into();
    scenario.actions = [
        ("GA_Adler_Skill", "GE_Player_Adler_Skill2_Damage"),
        ("GA_Adler_UltraSkill", "GE_Player_Adler_UltraSkill_Damage"),
    ]
    .into_iter()
    .map(
        |(ability_id, damage_effect_id)| GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: ability_id.into(),
            array_index: 0,
            damage_effect_id: damage_effect_id.into(),
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 0,
        },
    )
    .collect();
    scenario.model_assumptions.insert(
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_ID
            .description()
            .into(),
    );
    let receipts = project_actions(&tables, &mut scenario, &projection).unwrap();
    assert_eq!(receipts[0]["compiled_scaling_stat"], "def");
    assert_eq!(receipts[0]["compiled_rate"], json!("7/10"));
    assert_eq!(receipts[1]["compiled_scaling_stat"], "def");
    assert_eq!(receipts[1]["compiled_rate"], json!("1/1"));
    for (action, (expected_effect, expected_rate)) in scenario.actions.into_iter().zip([
        (
            "GE_Player_Adler_Skill2_Damage",
            Ratio {
                numerator: 7,
                denominator: 10,
            },
        ),
        ("GE_Player_Adler_UltraSkill_Damage", Ratio::one()),
    ]) {
        let GameActionCommand::InvokeAbility {
            damage_effect_id,
            atk_rate,
            scaling_stat,
            ..
        } = action
        else {
            panic!()
        };
        assert_eq!(scaling_stat, crate::simulation::NativeScalingStat::Def);
        assert_eq!(damage_effect_id, expected_effect);
        assert_eq!(atk_rate, expected_rate);
    }
}
