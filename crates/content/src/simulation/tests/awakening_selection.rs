use super::*;

#[test]
fn chiz_selected_effects_keep_capacity_start_balance_and_acquisition_independent() {
    // Frozen Lv80 cap=15000; first Normal grant=450. Effect1 raises
    // cap by 10%, Effect5 starts full, Effect4 multiplies acquisition by 1.2.
    for (effects, capacity, initial, after_hit) in [
        (vec![1], 16500, 0, 450),
        (vec![4], 15000, 0, 540),
        (vec![5], 15000, 15000, 15000),
        (vec![2, 3, 6], 15000, 0, 450),
    ] {
        let mut req = super::chiz_actions::request();
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(effects).unwrap();
        let mut state = crate::simulation::state::GameSimulatorState::new(&req);
        let actor = &mut state.actors[0];
        crate::characters::chiz::resource::initialize(&req.actors[0], actor).unwrap();
        crate::characters::chiz::combat::deposit(&req.actors[0], actor, 0).unwrap();
        let grain = actor.character.chiz_mut().unwrap().grain.as_ref().unwrap();
        assert_eq!(
            grain.summary().unwrap().capacity,
            ExactFraction::new(capacity, 1).unwrap()
        );
        assert_eq!(grain.current(), ExactFraction::new(initial, 1).unwrap());
        assert_eq!(
            grain.summary().unwrap().initial,
            ExactFraction::new(0, 1).unwrap()
        );
        assert_eq!(
            grain.summary().unwrap().granted,
            ExactFraction::new(initial, 1).unwrap()
        );
        crate::characters::chiz::resource::grant_normal(
            &req.actors[0],
            actor,
            "GE_Player_Chiichan073_Melee1_Damage",
            0,
        )
        .unwrap();
        assert_eq!(
            actor
                .character
                .chiz_mut()
                .unwrap()
                .grain
                .as_ref()
                .unwrap()
                .current(),
            ExactFraction::new(after_hit, 1).unwrap()
        );
    }
}

#[test]
fn selected_awakenings_bind_additional_owner_damage_and_crit_by_identity() {
    // Frozen Lv1/Lv2 primary coefficients: Blackbird Skill 1/1.08,
    // Blackbird Burst .467/.504, Skia Skill .3/.324, Daffodil Normal
    // .167/.18, Aurelia Burst .89/.961. DEF factor is .5 except
    // Daffodil Effect6 (15% ignore => 20/37).
    for (owner, burst, effects, num, den, cr_num, cr_den) in [
        ("blackbird", false, vec![2], 1000, 1, 0, 1),
        ("blackbird", false, vec![3], 500, 1, 0, 1),
        ("blackbird", false, vec![1, 4, 5], 540, 1, 0, 1),
        ("blackbird", true, vec![6], 1401, 4, 0, 1),
        ("blackbird", true, vec![4], 39228, 125, 0, 1),
        ("skia", false, vec![2], 150, 1, 1, 10),
        ("skia", false, vec![3], 162, 1, 0, 1),
        ("skia", false, vec![1, 4, 5], 162, 1, 0, 1),
        ("daffodil", false, vec![6], 3340, 37, 0, 1),
        ("daffodil", false, vec![1, 2, 4], 90, 1, 0, 1),
        ("aurelia", true, vec![3], 445, 1, 0, 1),
        ("aurelia", true, vec![5], 445, 1, 3, 20),
        ("aurelia", true, vec![1, 4, 5], 961, 2, 3, 20),
        ("hathor", false, vec![6], 500, 1, 0, 1),
        ("hathor", false, vec![4], 500, 1, 0, 1),
        ("hathor", false, vec![1, 4, 5], 2754, 5, 0, 1),
        ("hotori", true, vec![6], 100, 1, 0, 1),
        ("hotori", true, vec![2], 102, 1, 0, 1),
        ("hotori", true, vec![5], 391, 4, 0, 1),
        ("linko", false, vec![5], 267, 2, 0, 1),
        ("linko", false, vec![6], 89, 1, 1, 4),
        ("linko", false, vec![1, 4, 5], 864, 5, 0, 1),
        ("edgar", false, vec![4], 600, 1, 0, 1),
        ("edgar", false, vec![1, 2, 4], 648, 1, 0, 1),
        ("mint", false, vec![2], 630, 1, 0, 1),
        ("mint", false, vec![1, 3, 4], 486, 1, 3, 25),
        ("shinku", false, vec![6], 173, 2, 0, 1),
        ("zankou", false, vec![1], 553, 5, 0, 1),
        ("zankou", false, vec![2], 395, 4, 0, 1),
    ] {
        let mut req = match owner {
            "blackbird" => super::blackbird_actions::request(),
            "skia" => super::skia_actions::request(),
            "daffodil" => super::daffodil_actions::request(),
            "aurelia" => super::aurelia_actions::request(),
            "hathor" => super::hathor_actions::request(),
            "hotori" => super::hotori_actions::request(),
            "linko" => super::linko_actions::request(),
            "edgar" => super::edgar_actions::request(),
            "mint" => super::mint_actions::request(),
            "shinku" => super::shinku_actions::request(),
            "zankou" => super::zankou_actions::request(),
            _ => unreachable!(),
        };
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(effects).unwrap();
        req.actors[0].static_stats.crit_rate = Ratio::zero();
        let (ability, packet) = match (owner, burst) {
            ("blackbird", true) => (
                "GA_BlackBird_UltraSkill",
                "GE_Player_BlackBird_UltraSkill_Damage",
            ),
            ("blackbird", false) => ("GA_BlackBird_Skill", "GE_Player_BlackBird_Skill_Damage"),
            ("skia", _) => ("GA_Skia_Skill", "GE_Player_Skia_Skill1_Damage"),
            ("aurelia", _) => (
                "GA_Mitsuki_UltraSkill",
                "GE_Player_Mitsuki_UltraSkill_Damage",
            ),
            ("daffodil", _) => ("GA_Daffodill_Melee", "GE_Player_Daffodill_Melee1_Damage"),
            ("hathor", _) => ("GA_Hathor_Skill", "GE_Player_Hathor_Skill1_Damage"),
            ("hotori", _) => ("GA_Jin_UltraSkill", "GE_Player_Jin_UltraSkill1_Damage"),
            ("linko", _) => ("GA_Radio072_Skill", "GE_Player_Radio072_Skill1_Damage"),
            ("edgar", _) => ("GA_Edgar_Skill", "GE_Player_Edgar_Skill_Damage"),
            ("mint", _) => ("GA_Mint019_Skill", "GE_Player_Mint_Skill1_Damage_New"),
            ("shinku", _) => ("GA_Shinku_Skill", "GE_Player_Shinku_Skill1_1_Damage"),
            ("zankou", _) => ("GA_Zankou_Melee", "GE_Player_Zankou_Melee1_Damage"),
            _ => unreachable!(),
        };
        if let GameActionCommand::InvokeAbility {
            ability_id,
            damage_effect_id,
            energy_cost,
            timestamp_us,
            ..
        } = &mut req.actions[0]
        {
            *ability_id = ability.into();
            *damage_effect_id = packet.into();
            *energy_cost = if burst { 100 } else { 0 };
            if owner == "hotori" {
                // World's Tide needs exactly 120 Timepiece and spends no
                // conventional energy; the 14.18s mode must finish in scope.
                *timestamp_us = 20_000_000;
                *energy_cost = 0;
            }
        }
        if owner == "hotori" {
            req.relative_duration_us = 35_000_000;
        }
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let first = &report["hit_receipts"][0];
        assert_eq!(
            first["non_crit_damage"],
            serde_json::json!(format!("{}/{}", num, den)),
            "{owner}"
        );
        assert_eq!(
            first["crit_rate"],
            serde_json::json!(format!("{}/{}", cr_num, cr_den)),
            "{owner}"
        );
    }
}

#[test]
fn selected_owner_buff_durations_follow_effect_identity_not_selection_count() {
    let select = |effects: &[u8]| {
        ntesim_data::AwakeningSelection::selected(effects.iter().copied()).unwrap()
    };
    assert_eq!(
        crate::characters::mint::buff_duration(&select(&[5]), "GA_Mint019_Skill"),
        Some(6_000_000)
    );
    assert_eq!(
        crate::characters::mint::buff_duration(&select(&[1, 2, 3, 4]), "GA_Mint019_Skill"),
        None
    );
    assert_eq!(
        crate::characters::sakiri::buff_duration(&select(&[4]), "GA_Sagiri_UltraSkill"),
        Some(20_000_000)
    );
    assert_eq!(
        crate::characters::sakiri::buff_duration(&select(&[1, 2, 3]), "GA_Sagiri_UltraSkill"),
        Some(20_000_000) // Ordinary Burst carrier is independent of A4.
    );
    assert_eq!(
        crate::characters::nanally::buff_duration(&select(&[6]), "GA_Nanally_Skill"),
        Some(15_000_000)
    );
    assert_eq!(
        crate::characters::nanally::buff_duration(&select(&[1, 2, 3, 4, 5]), "GA_Nanally_Skill"),
        Some(12_000_000)
    );
    assert_eq!(
        crate::characters::nanally::buff_duration(&select(&[2]), "GA_Nanally_UltraSkill"),
        Some(9_000_000)
    );
    assert_eq!(
        crate::characters::nanally::buff_duration(&select(&[1, 3, 4]), "GA_Nanally_UltraSkill"),
        Some(6_000_000)
    );
    assert_eq!(
        crate::characters::baicang::buff_duration(&select(&[5]), "GA_Cang_UltraSkill"),
        Some(16_000_000)
    );
    assert_eq!(
        crate::characters::baicang::buff_duration(&select(&[1, 2, 3, 4]), "GA_Cang_UltraSkill"),
        Some(12_000_000)
    );
    assert_eq!(
        crate::characters::lacrimosa::buff_duration(&select(&[2]), "GA_Lacrimosa_QTE"),
        Some(15_000_000)
    );
    assert_eq!(
        crate::characters::lacrimosa::buff_duration(&select(&[1, 3]), "GA_Lacrimosa_QTE"),
        None
    );
}

#[test]
fn nanally_selected_a1_energy_is_independent_of_r3_count() {
    for (effects, expected_a1_grants) in [(vec![1, 3], 1), (vec![2, 3, 4], 0)] {
        let mut req = super::nanally_actions::request();
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(effects).unwrap();
        req.actors[0].initial_energy = 0;
        req.actions = vec![super::nanally_actions::make_action(
            0,
            0,
            "GA_Nanally_Skill",
            "GE_Player_Nanally_Skill1_Damage",
            0,
            0,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["event_kind"] == "nanally_a1_energy_grant")
                .count(),
            expected_a1_grants
        );
        assert_eq!(
            report["final_actor_states"][0]["granted_energy"],
            if expected_a1_grants == 1 { 8 } else { 6 }
        );
    }
}

#[test]
fn baicang_selected_a5_extends_domain_tick_train_without_count_fallback() {
    for (effects, expected_ticks) in [(vec![5], 8), (vec![1, 2, 3, 4], 6)] {
        let mut req = super::baicang_actions::request();
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(effects).unwrap();
        req.actors[0].initial_energy = 120;
        req.actors[0].energy_capacity = 120;
        req.actions = vec![super::baicang_actions::make_action(
            0,
            0,
            "GA_Cang_UltraSkill",
            "GE_Player_Cang_UltraSkill2_Damage",
            120,
            20_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(
            report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|hit| hit["damage_effect_id"] == "GE_Player_Cang_UltraSkill_Damage")
                .count(),
            expected_ticks
        );
    }
}

#[test]
fn native_scenarios_accept_selected_effects_for_ordinary_mint() {
    let mut request = sample_s1_request();
    request.actors[0].awakening = ntesim_data::AwakeningSelection::selected([6]).unwrap();
    crate::simulation::validate_game_request(&request, &ExecutionLimits::default()).unwrap();
}

#[test]
fn selected_effects_are_admitted_for_all_released_owners_without_promoting_kit_coverage() {
    for owner in crate::registry::character_descriptors().unwrap() {
        for index in 1..=6 {
            crate::registry::validate_awakening_selection(
                owner.character_id,
                &ntesim_data::AwakeningSelection::selected([index]).unwrap(),
            )
            .unwrap();
        }
    }
}

#[test]
fn native_scenarios_accept_adler_effect_selection() {
    let mut request = crate::simulation::tests::adler_actions::request();
    request.actors[0].awakening = ntesim_data::AwakeningSelection::selected([6]).unwrap();
    crate::simulation::validate_game_request(&request, &ExecutionLimits::default()).unwrap();
}

#[test]
fn haniel_selected_awakenings_separate_ace_effects_from_resonance() {
    // Burst Lv1=3.0, Lv2=3.24; fixture base ATK zero excludes
    // the native flat team grant. A5 alone adds .3 Psyche bonus.
    for (effects, expected, crit_rate) in [
        (vec![3], 1500, 1),
        (vec![5], 1950, 1),
        (vec![6], 1500, 1),
        (vec![1, 4, 5], 2106, 1),
    ] {
        let mut req = super::haniel_actions::request();
        req.actors[0].static_stats.crit_rate = Ratio {
            numerator: 1,
            denominator: 20,
        };
        req.actors[0].static_stats.crit_damage_bonus = Ratio {
            numerator: 1,
            denominator: 2,
        };
        let has_a3 = effects.contains(&3);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(effects).unwrap();
        req.actions = vec![super::haniel_actions::make_action(
            0,
            0,
            "GA_Haniel_UltraSkill",
            "GE_Player_Haniel_Ultraskill1_Damage",
            100,
            20_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        let hit = &report["hit_receipts"][0];
        assert_eq!(
            hit["non_crit_damage"],
            serde_json::json!(format!("{}/{}", expected, 1))
        );
        assert_eq!(
            hit["crit_rate"],
            serde_json::json!(format!("{}/{}", crit_rate, if has_a3 { 4 } else { 20 }))
        );
        // Selecting Effect6 alone cannot grant resonance_6's team Crit DMG.
        assert_eq!(
            hit["crit_damage"],
            serde_json::json!(format!("{}/{}", expected * 3 / 2, 1))
        );
    }
}

#[test]
fn haniel_selected_a2_extends_live_hootie_without_unselected_effects() {
    // Lv1 Hootie grants .10 * base ATK until 8s, or 12s with Effect2.
    // Melee1=.419; with DEF factor .5, the 9s hit is 230.45 or 209.5.
    for (effects, num, den) in [(vec![2], 4609, 20), (vec![5], 419, 2)] {
        let mut req = super::haniel_actions::request();
        req.actors[0].static_stats.base_atk = 1000;
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(effects).unwrap();
        req.relative_duration_us = 10_000_000;
        req.actions = vec![
            super::haniel_actions::make_action(
                0,
                0,
                "GA_Haniel_Skill",
                "GE_Player_Haniel_Skill1_Damage",
                0,
                16_000_000,
            ),
            super::haniel_actions::make_action(
                9_000_000,
                0,
                "GA_Haniel_Melee",
                "GE_Player_Haniel_Melee1_Damage",
                0,
                0,
            ),
        ];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(
            report["hit_receipts"][2]["non_crit_damage"],
            serde_json::json!(format!("{}/{}", num, den))
        );
    }
}
