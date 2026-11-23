use super::*;

#[test]
fn owner_packet_coefficients_keep_support_parent_and_native_burst_child_separate() {
    let tables = ntesim_data::skill_levels::frozen::tables();
    let assumptions = BTreeMap::new();
    for (coordinate, numerator, denominator) in [(0, 3, 2), (7, 2571, 1000)] {
        let request = crate::build::PacketCoefficientRequest {
            content_id: "ntesim-current",
            character_id: "character:1010",
            ability_id: "GA_Nanally_QTE",
            damage_effect_id: "GE_Player_Nanally_Cat_QTE_Damage",
            array_index: coordinate,
            research_metadata: None,
            model_assumptions: &assumptions,
        };
        let projected = crate::build::project_native_packet_coefficient(tables, &request).unwrap();
        assert_eq!(
            projected["compiled_rate"],
            json!(format!("{}/{}", numerator, denominator))
        );
        assert_eq!(projected["binding"]["kind"], "owner_child");
        assert_eq!(
            projected["binding"]["initiating_ability_id"],
            "GA_Nanally_QTE"
        );
        assert_eq!(
            projected["binding"]["packet_ga_name"],
            "GA_Nanally_UltraSkill"
        );
        assert_eq!(projected["action_metadata"]["input_role"], "qte");
        let wrong = crate::build::PacketCoefficientRequest {
            ability_id: "GA_Nanally_Skill",
            ..request
        };
        assert!(
            crate::build::project_native_packet_coefficient(tables, &wrong)
                .unwrap_err()
                .contains("belongs to another ability")
        );
    }
}

#[test]
fn owner_packet_aliases_preserve_canonical_parents_hp_scaling_and_strict_provenance() {
    let tables = ntesim_data::skill_levels::frozen::tables();
    for (character, ability, effect) in [
        (
            "character:1010",
            "GA_Nanally_QTE",
            "GE_Player_Nanally_Cat_QTE_Damage",
        ),
        (
            "character:1039",
            "GA_Fadia_UltraSkill",
            "GE_Player_Fadia_UltraSkill1_Damage",
        ),
    ] {
        let assumptions = BTreeMap::new();
        let request = PacketCoefficientRequest {
            content_id: "ntesim-current",
            character_id: character,
            ability_id: ability,
            damage_effect_id: effect,
            array_index: 7,
            research_metadata: None,
            model_assumptions: &assumptions,
        };
        let canonical = project_native_packet_coefficient(tables, &request).unwrap();
        let lower_ability = ability.to_ascii_lowercase();
        let lower_effect = effect.to_ascii_lowercase();
        let alias = PacketCoefficientRequest {
            ability_id: &lower_ability,
            damage_effect_id: &lower_effect,
            ..request
        };
        assert_eq!(
            project_native_packet_coefficient(tables, &alias).unwrap(),
            canonical
        );
        assert_eq!(canonical["native_damage_report"]["ability_id"], ability);
        assert_eq!(canonical["packet_metadata"]["damage_effect_id"], effect);
    }
}
