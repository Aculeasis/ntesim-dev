use super::*;

#[test]
fn universal_skill_level_projection_covers_all_twenty_three_characters() {
    let (tables, _, _) = skill_fixture();

    for owner in ntesim_data::character_actions::CHARACTERS {
        let character_id = format!("character:{}", owner.native_id);

        for (slot, ability) in owner.abilities.iter().enumerate() {
            // Level 1, Breakthrough 0, Awakening 0 -> Effective Level 1, Coordinate 0
            let a0_receipt = super::preview_projection::effective_skill_level(
                &tables,
                &character_id,
                ability,
                1,
                0,
                &ntesim_data::AwakeningSelection::prefix(0),
            )
            .unwrap_or_else(|err| {
                panic!("failed A0 projection for {character_id} {ability}: {err}")
            });

            assert_eq!(a0_receipt["purchased_level"], 1);
            assert_eq!(a0_receipt["effective_level"], 1);
            assert_eq!(a0_receipt["array_index"], 0);
            assert_eq!(a0_receipt["resonance"]["applied_bonus"], 0);

            // Level 10, Breakthrough 6, Awakening 6 -> Effective Level 10 or 11
            let a6_receipt = super::preview_projection::effective_skill_level(
                &tables,
                &character_id,
                ability,
                10,
                6,
                &ntesim_data::AwakeningSelection::prefix(6),
            )
            .unwrap_or_else(|err| {
                panic!("failed A6 projection for {character_id} {ability}: {err}")
            });

            let expected_resonance = match owner.native_id {
                "1073" => 0,                           // Chiz has no skill level bonus
                "1021" | "1072" => (slot != 0) as u64, // Edgar / Linko: slot 0 is excluded
                _ => (slot != 3) as u64,               // Standard: slot 3 (QTE) is excluded
            };

            assert_eq!(a6_receipt["purchased_level"], 10);
            assert_eq!(a6_receipt["effective_level"], 10 + expected_resonance);
            assert_eq!(a6_receipt["array_index"], 9 + expected_resonance);
            assert_eq!(a6_receipt["resonance"]["applied_bonus"], expected_resonance);

            // Case insensitivity: lowercase ability resolves to the exact same canonical ability
            let lower_ability = ability.to_ascii_lowercase();
            let case_receipt = super::preview_projection::effective_skill_level(
                &tables,
                &character_id,
                &lower_ability,
                1,
                0,
                &ntesim_data::AwakeningSelection::prefix(0),
            )
            .expect("lowercase ability should resolve cleanly");
            assert_eq!(case_receipt["ability_id"], *ability);

            // Level 10 requires breakthrough >= 6; breakthrough 2 must fail
            let bt_err = super::preview_projection::effective_skill_level(
                &tables,
                &character_id,
                ability,
                10,
                2,
                &ntesim_data::AwakeningSelection::prefix(0),
            )
            .unwrap_err();
            assert!(
                bt_err.contains("prerequisites"),
                "expected prerequisite error for {character_id} {ability}, got {bt_err}"
            );

            // Level 0 out of range
            let zero_err = super::preview_projection::effective_skill_level(
                &tables,
                &character_id,
                ability,
                0,
                6,
                &ntesim_data::AwakeningSelection::prefix(0),
            )
            .unwrap_err();
            assert!(
                zero_err.contains("supported range"),
                "expected range error for level 0, got {zero_err}"
            );

            // Level 11 out of range
            let high_err = super::preview_projection::effective_skill_level(
                &tables,
                &character_id,
                ability,
                11,
                6,
                &ntesim_data::AwakeningSelection::prefix(6),
            )
            .unwrap_err();
            assert!(
                high_err.contains("supported range"),
                "expected range error for level 11, got {high_err}"
            );
        }

        // Unknown ability must fail
        let unknown_err = super::preview_projection::effective_skill_level(
            &tables,
            &character_id,
            "GA_UnknownAbility",
            1,
            0,
            &ntesim_data::AwakeningSelection::prefix(0),
        )
        .unwrap_err();
        assert!(unknown_err.contains("exactly one configured combat ability"));
    }
}
