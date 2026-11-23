use super::*;

#[test]
fn fadia_initial_burst_hp_arrays_match_the_frozen_nine_level_values_and_metadata() {
    let (tables, _, _) = action_fixture();
    let expected = [
        (
            "GE_Player_Fadia_UltraSkill1_Damage",
            [39, 42, 45, 49, 53, 57, 62, 67, 72],
        ),
        (
            "GE_Player_Fadia_UltraSkill2_Damage",
            [12, 13, 14, 15, 16, 18, 19, 21, 22],
        ),
        (
            "GE_Player_Fadia_UltraSkill3_Damage",
            [63, 68, 73, 79, 86, 93, 100, 108, 117],
        ),
    ];
    for (effect, per_mille) in expected {
        for (level, value) in per_mille.into_iter().enumerate() {
            let projection = project_native_packet_coefficient(
                &tables,
                &PacketCoefficientRequest {
                    content_id: "ntesim-current",
                    character_id: "character:1039",
                    ability_id: "GA_Fadia_UltraSkill",
                    damage_effect_id: effect,
                    array_index: level as u32,
                    research_metadata: None,
                    model_assumptions: &BTreeMap::new(),
                },
            )
            .unwrap();
            assert_eq!(
                projection["compiled_scaling_stat"],
                json!(crate::simulation::NativeScalingStat::Hp)
            );
            assert_eq!(
                ExactFraction::from_ratio(
                    serde_json::from_value::<Ratio>(projection["compiled_rate"].clone()).unwrap()
                )
                .unwrap(),
                ExactFraction::new(value as u128, 1000).unwrap()
            );
            assert_eq!(
                projection["packet_metadata"]["native_damage_type_ex"],
                "EExecutionDamageTypeEX::DAMAGE_TYPE_PSYCHICALLY"
            );
            assert_eq!(projection["packet_metadata"]["damage_kind"], "mental");
        }
    }
}

#[test]
fn fadia_skill_hp_projection_binds_the_native_hp_coefficient() {
    let (tables, _, _) = action_fixture();
    let model = crate::characters::fadia::effects::MODEL;
    let assumptions = BTreeMap::from([(model.into(), model.description().into())]);
    let projection = project_native_packet_coefficient(
        &tables,
        &PacketCoefficientRequest {
            content_id: "ntesim-current",
            character_id: "character:1039",
            ability_id: "GA_Fadia_Skill",
            damage_effect_id: "GE_Player_Fadia_Skill1_Damage",
            array_index: 0,
            research_metadata: None,
            model_assumptions: &assumptions,
        },
    )
    .unwrap();
    assert_eq!(projection["compiled_scaling_stat"], "hp");
    assert_eq!(projection["compiled_rate"], serde_json::json!("1/40"));
}
