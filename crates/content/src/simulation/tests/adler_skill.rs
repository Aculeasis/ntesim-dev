use super::adler_actions::{def_action, make_action, request};
use super::*;
use serde_json::json;

#[test]
fn adler_skill_captures_and_clears_karma_in_final_damage_zone() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(2);
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def = 1000;
    req.actors[0].static_stats.damage_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };
    req.actors[0].initial_energy = 100;
    req.targets[0].hp_current = 1_000_000;
    req.targets[0].hp_max = 1_000_000;
    req.targets[0].defense = 1080;
    req.targets[0].level = 80;
    req.targets[0].resistance = Ratio::zero();
    req.relative_duration_us = 10_001_000;
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Adler_UltraSkill",
            "GE_Player_Adler_UltraSkill_Damage",
            100,
            0,
        ),
        def_action(
            200,
            "GA_Adler_Skill",
            "GE_Player_Adler_Skill2_Damage",
            Ratio::one(),
        ),
        make_action(
            400,
            0,
            "GA_Adler_Melee",
            "GE_Player_Adler_Melee1_Damage",
            0,
            0,
        ),
        def_action(
            600,
            "GA_Adler_Skill",
            "GE_Player_Adler_Skill2_Damage",
            Ratio::one(),
        ),
    ];
    req.model_assumptions.insert(
        crate::characters::adler::skill::SKILL_TRAIN_HYPOTHESIS_ID.into(),
        crate::characters::adler::skill::SKILL_TRAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::adler::skill::KARMA_HYPOTHESIS_ID.into(),
        crate::characters::adler::skill::KARMA_HYPOTHESIS_ID
            .description()
            .into(),
    );

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    let skill_hits = hits
        .iter()
        .filter(|hit| hit["ability_id"] == "GA_Adler_Skill")
        .collect::<Vec<_>>();
    assert_eq!(skill_hits.len(), 16);
    assert_eq!(skill_hits[0]["non_crit_damage"], json!("840/1"));
    assert_eq!(skill_hits[3]["non_crit_damage"], json!("2163/4"));
    assert!(
        skill_hits[3..]
            .iter()
            .all(|hit| hit["time_us"].as_u64().unwrap() >= 700)
    );
    assert_eq!(hits[13]["ability_id"], "GA_Adler_Melee");
    assert_eq!(hits[13]["non_crit_damage"], json!("603/4"));
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
    assert_eq!(
        report["resolved_model_assumptions"]
            [crate::characters::adler::skill::SKILL_TRAIN_HYPOTHESIS_ID.as_str()],
        crate::characters::adler::skill::SKILL_TRAIN_HYPOTHESIS_ID.description()
    );
    assert_eq!(
        report["resolved_model_assumptions"]
            [crate::characters::adler::skill::KARMA_HYPOTHESIS_ID.as_str()],
        crate::characters::adler::skill::KARMA_HYPOTHESIS_ID.description()
    );
}

#[test]
fn selected_effects_bind_individual_adler_effects_and_count_only_resonance() {
    let execute = |effects: &[u8]| {
        let mut req = request();
        req.actors[0].awakening =
            ntesim_data::AwakeningSelection::selected(effects.iter().copied()).unwrap();
        req.actors[0].initial_energy = 100;
        req.actors[0].static_stats.base_def = Some(1000);
        req.actors[0].static_stats.def = 1000;
        req.targets[0].defense = 0;
        req.targets[0].resistance = Ratio::zero();
        req.relative_duration_us = 10_001_000;
        req.actions = vec![
            make_action(
                0,
                0,
                "GA_Adler_UltraSkill",
                "GE_Player_Adler_UltraSkill_Damage",
                100,
                0,
            ),
            def_action(
                200,
                "GA_Adler_Skill",
                "GE_Player_Adler_Skill2_Damage",
                Ratio::one(),
            ),
        ];
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap()
    };

    let only_a5 = execute(&[5]);
    let a1_a2_a5 = execute(&[5, 2, 1]);
    let no_effects = execute(&[]);
    assert_eq!(only_a5["final_actor_states"][0]["energy"], 24);
    assert_eq!(no_effects["final_actor_states"][0]["energy"], 4);
    assert_eq!(
        only_a5["final_actor_states"][0]["energy"].as_u64().unwrap()
            - no_effects["final_actor_states"][0]["energy"]
                .as_u64()
                .unwrap(),
        20
    );
    assert_eq!(
        only_a5["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["energy_change"]["source"] == "adler_a5")
            .count(),
        1
    );
    let a5_refund = only_a5["trace"]
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["energy_change"]["source"] == "adler_a5")
        .unwrap();
    assert_eq!(a5_refund["energy_change"]["gross"], 20);
    let first_skill = |report: &serde_json::Value| {
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|hit| {
                hit["ability_id"] == "GA_Adler_Skill"
                    && hit["damage_effect_id"] == "GE_Player_Adler_Skill2_Damage"
            })
            .unwrap()["non_crit_damage"]
            .clone()
    };
    let baseline = first_skill(&only_a5);
    assert_eq!(first_skill(&no_effects), baseline);
    let selected = first_skill(&a1_a2_a5);
    let baseline_exact: crate::simulation::ExactFraction =
        serde_json::from_value(baseline.clone()).unwrap();
    let selected_exact: crate::simulation::ExactFraction =
        serde_json::from_value(selected.clone()).unwrap();
    assert_eq!(
        selected_exact.numerator * baseline_exact.denominator * 125,
        baseline_exact.numerator * selected_exact.denominator * 216,
        "selected {selected}, baseline {baseline}"
    );
}

#[test]
fn adler_delayed_initial_packets_keep_their_own_consumed_karma_after_recast() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(2);
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def = 1000;
    req.actors[0].static_stats.damage_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };
    req.actors[0].initial_energy = 100;
    req.targets[0].hp_current = 1_000_000;
    req.targets[0].hp_max = 1_000_000;
    req.targets[0].defense = 1080;
    let mut first = def_action(
        200,
        "GA_Adler_Skill",
        "GE_Player_Adler_Skill2_Damage",
        Ratio::one(),
    );
    if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut first {
        *hit_delay_us = 1_000_000;
    }
    let mut second = def_action(
        700,
        "GA_Adler_Skill",
        "GE_Player_Adler_Skill2_Damage",
        Ratio::one(),
    );
    if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut second {
        *hit_delay_us = 100;
    }
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Adler_UltraSkill",
            "GE_Player_Adler_UltraSkill_Damage",
            100,
            0,
        ),
        first,
        make_action(
            500,
            0,
            "GA_Adler_Melee",
            "GE_Player_Adler_Melee1_Damage",
            0,
            0,
        ),
        second,
    ];
    req.relative_duration_us = 20_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let hits = report["hit_receipts"].as_array().unwrap();
    let first_hit = hits
        .iter()
        .find(|h| {
            h["command_index"] == 1 && h["damage_effect_id"] == "GE_Player_Adler_Skill2_Damage"
        })
        .unwrap();
    let second_hit = hits
        .iter()
        .find(|h| {
            h["command_index"] == 3 && h["damage_effect_id"] == "GE_Player_Adler_Skill2_Damage"
        })
        .unwrap();
    assert_eq!(first_hit["non_crit_damage"], json!("840/1"));
    assert_eq!(second_hit["non_crit_damage"], json!("2163/4"));
    assert!(!hits.iter().any(
        |h| h["command_index"] == 1 && h["damage_effect_id"] == "GE_Player_Adler_Skill_Damage"
    ));
    assert!(hits.iter().any(
        |h| h["command_index"] == 3 && h["damage_effect_id"] == "GE_Player_Adler_Skill_Damage"
    ));
}

#[test]
fn adler_aim_and_normal_grant_distinct_karma_before_the_next_skill() {
    for (effect, count, zero_attack, expected) in [
        ("GE_Player_Adler_Aim_Damage", 1, false, 364),
        ("GE_Player_Adler_Melee1_Damage", 1, false, 357),
        ("GE_Player_Adler_Aim_Damage", 11, false, 490),
        ("GE_Player_Adler_Melee1_Damage", 1, true, 350),
    ] {
        let mut req = request();
        req.actors[0].static_stats.base_def = Some(1000);
        req.actors[0].static_stats.def = 1000;
        if zero_attack {
            req.actors[0].static_stats.atk = Ratio::zero();
        }
        req.targets[0].defense = 1080;
        req.targets[0].hp_current = 100_000;
        req.targets[0].hp_max = 100_000;
        req.actions = (0..count)
            .map(|index| make_action(index * 200, 0, "GA_Adler_Melee", effect, 0, 0))
            .collect();
        req.actions.push(def_action(
            count * 200,
            "GA_Adler_Skill",
            "GE_Player_Adler_Skill2_Damage",
            Ratio::one(),
        ));
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        let hit = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|h| {
                h["command_index"] == count
                    && h["damage_effect_id"] == "GE_Player_Adler_Skill2_Damage"
            })
            .unwrap();
        assert_eq!(hit["non_crit_damage"], json!(format!("{}/{}", expected, 1)));
    }
}

#[test]
fn adler_skill_ticks_continue_off_field_and_stop_after_owner_or_target_defeat() {
    let mut off_field = request();
    off_field.actors[0].static_stats.base_def = Some(1000);
    off_field.actors[0].static_stats.def = 1000;
    let mut ally = off_field.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    off_field.actors.push(ally);
    off_field.actions = vec![def_action(
        0,
        "GA_Adler_Skill",
        "GE_Player_Adler_Skill2_Damage",
        Ratio::one(),
    )];
    off_field.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 500,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    off_field.relative_duration_us = 1_000_100;
    let report = run_game_scenario(&off_field, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["final_buff_state"]["active_actor_index"], 1);
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|h| h["damage_effect_id"] == "GE_Player_Adler_Skill_Damage"
                && h["actor_index"] == 0)
    );

    for (owner_dies, target_dies) in [(true, false), (false, true)] {
        let mut req = request();
        req.actors[0].static_stats.base_def = Some(1000);
        req.actors[0].static_stats.def = 1000;
        req.actions = vec![def_action(
            0,
            "GA_Adler_Skill",
            "GE_Player_Adler_Skill2_Damage",
            Ratio::one(),
        )];
        if owner_dies {
            req.actors[0].initial_hp = 100;
            req.actors[0].initial_hp_max = 100;
            req.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
                timestamp_us: 1_500_000,
                actor_index: 0,
                delivered_damage: u64::MAX,
            }];
            req.model_assumptions.insert(
                crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
                crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID
                    .description()
                    .into(),
            );
        }
        if target_dies {
            req.targets[0].hp_current = 1;
            req.targets[0].hp_max = 1;
        }
        req.relative_duration_us = 2_000_100;
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        if owner_dies {
            assert_eq!(report["final_actor_states"][0]["hp"], 0, "{report}");
        }
        assert_eq!(
            report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|h| h["damage_effect_id"] == "GE_Player_Adler_Skill_Damage")
                .count(),
            usize::from(owner_dies),
            "owner_dies={owner_dies} target_dies={target_dies}"
        );
    }
}

#[test]
fn adler_skill_train_keeps_unexpired_ticks_pending_at_a_short_horizon() {
    let mut req = request();
    req.actions = vec![def_action(
        0,
        "GA_Adler_Skill",
        "GE_Player_Adler_Skill2_Damage",
        Ratio::one(),
    )];
    req.relative_duration_us = 500_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true);
    assert!(report["pending_events"].as_u64().unwrap() > 0);
    assert!(
        !report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|h| h["damage_effect_id"] == "GE_Player_Adler_Skill_Damage")
    );
}

#[test]
fn adler_skill_train_matches_native_def_coordinates_and_horizon() {
    for (level, awakening, initial_total, tick_damage, tick_rate) in [
        (
            1,
            0,
            1050,
            100,
            Ratio {
                numerator: 1,
                denominator: 5,
            },
        ),
        (
            8,
            0,
            1800,
            172,
            Ratio {
                numerator: 343,
                denominator: 1000,
            },
        ),
        (
            8,
            3,
            1944,
            185,
            Ratio {
                numerator: 37,
                denominator: 100,
            },
        ),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([level, level, level]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actors[0].static_stats.base_def = Some(1000);
        req.actors[0].static_stats.def = 1000;
        req.actors[0].initial_energy = 20;
        req.targets[0].hp_current = 100_000;
        req.targets[0].hp_max = 100_000;
        req.targets[0].defense = 1080;
        req.targets[0].level = 80;
        req.targets[0].resistance = Ratio::zero();
        req.relative_duration_us = 10_000_100;
        let mut skill = def_action(
            0,
            "GA_Adler_Skill",
            "GE_Player_Adler_Skill2_Damage",
            Ratio::zero(),
        );
        if let GameActionCommand::InvokeAbility { energy_cost, .. } = &mut skill {
            *energy_cost = 20;
        }
        req.actions = vec![skill];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["pending_events"], 0);
        assert_eq!(report["hits_settled"], 13);
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 20);
        let hits = report["hit_receipts"].as_array().unwrap();
        let initial = hits
            .iter()
            .filter(|hit| {
                hit["damage_effect_id"] == "GE_Player_Adler_Skill2_Damage"
                    || hit["damage_effect_id"] == "GE_Player_Adler_Skill3_Damage"
            })
            .collect::<Vec<_>>();
        assert_eq!(initial.len(), 3);
        let per_initial = initial_total / 3;
        assert!(initial.iter().all(|hit| {
            hit["non_crit_damage"] == json!(format!("{}/{}", per_initial, 1))
                && (hit["rounded_damage"] == per_initial
                    || hit["rounded_damage"] == per_initial * 2)
        }));
        let ticks = hits
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "GE_Player_Adler_Skill_Damage")
            .collect::<Vec<_>>();
        assert_eq!(ticks.len(), 10);
        assert!(ticks.iter().enumerate().all(|(index, hit)| {
            hit["time_us"].as_u64() == Some((index as u64 + 1) * 1_000_000 + 100)
                && hit["scaling_rate"] == json!(tick_rate)
                && (hit["rounded_damage"] == tick_damage
                    || hit["rounded_damage"] == tick_damage * 2)
        }));
    }
}

#[test]
fn adler_skill_uses_target_major_all_in_range_dummy_trains_and_spends_once() {
    let mut req = request();
    req.actors[0].talent_levels = Some([8; 3]);
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def = 1000;
    req.actors[0].initial_energy = 20;
    req.targets[0].hp_current = 100_000;
    req.targets[0].hp_max = 100_000;
    req.targets[0].defense = 1080;
    req.targets[0].resistance = Ratio::zero();
    let mut second_target = req.targets[0].clone();
    second_target.target_index = 1;
    req.targets.push(second_target);
    req.relative_duration_us = 10_000_100;
    let mut skill = def_action(
        0,
        "GA_Adler_Skill",
        "GE_Player_Adler_Skill2_Damage",
        Ratio::zero(),
    );
    if let GameActionCommand::InvokeAbility { energy_cost, .. } = &mut skill {
        *energy_cost = 20;
    }
    req.actions = vec![skill];

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 26);
    let hits = report["hit_receipts"].as_array().unwrap();
    let skill_hits = hits
        .iter()
        .filter(|hit| hit["ability_id"] == "GA_Adler_Skill")
        .collect::<Vec<_>>();
    assert_eq!(skill_hits.len(), 26);
    assert_eq!(
        skill_hits[..6]
            .iter()
            .map(|hit| hit["target_index"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [0, 0, 0, 1, 1, 1]
    );
    let tick_hits = skill_hits
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Adler_Skill_Damage")
        .collect::<Vec<_>>();
    assert_eq!(tick_hits.len(), 20);
    for pair in tick_hits.as_chunks::<2>().0 {
        assert_eq!(pair[0]["time_us"], pair[1]["time_us"]);
        assert_eq!(pair[0]["target_index"], 0);
        assert_eq!(pair[1]["target_index"], 1);
    }
    for target_index in 0..2 {
        let target_hits = skill_hits
            .iter()
            .filter(|hit| hit["target_index"] == target_index)
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(target_hits.len(), 13);
        assert_eq!(
            target_hits[..3]
                .iter()
                .map(|hit| hit["damage_effect_id"].as_str().unwrap())
                .collect::<Vec<_>>(),
            [
                "GE_Player_Adler_Skill2_Damage",
                "GE_Player_Adler_Skill3_Damage",
                "GE_Player_Adler_Skill3_Damage"
            ]
        );
        let ticks = target_hits
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "GE_Player_Adler_Skill_Damage")
            .collect::<Vec<_>>();
        assert_eq!(ticks.len(), 10);
        assert!(ticks.iter().enumerate().all(|(index, hit)| {
            hit["time_us"].as_u64() == Some((index as u64 + 1) * 1_000_000 + 100)
                && hit["scaling_rate"]
                    == json!(Ratio {
                        numerator: 343,
                        denominator: 1000
                    })
        }));
        assert!(target_hits.iter().all(|hit| {
            hit["non_crit_damage"] == json!("600/1") || hit["non_crit_damage"] == json!("343/2")
        }));
    }
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 20);
    assert_eq!(report["final_actor_states"][0]["energy"], 4);
    assert_eq!(
        report["resolved_model_assumptions"]
            [crate::characters::adler::skill::SKILL_ALL_IN_RANGE_HYPOTHESIS_ID.as_str()],
        crate::characters::adler::skill::SKILL_ALL_IN_RANGE_HYPOTHESIS_ID.description()
    );
}

#[test]
fn adler_skill_stops_only_the_defeated_targets_periodic_stream() {
    let mut req = request();
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def = 1000;
    req.targets[0].hp_current = 100_000;
    req.targets[0].hp_max = 100_000;
    let mut low_hp_target = req.targets[0].clone();
    low_hp_target.target_index = 1;
    low_hp_target.hp_current = 1;
    low_hp_target.hp_max = 1;
    req.targets.push(low_hp_target);
    req.relative_duration_us = 10_000_100;
    req.actions = vec![def_action(
        0,
        "GA_Adler_Skill",
        "GE_Player_Adler_Skill2_Damage",
        Ratio::one(),
    )];

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let hits = report["hit_receipts"].as_array().unwrap();
    let skill_hits = hits
        .iter()
        .filter(|hit| hit["ability_id"] == "GA_Adler_Skill")
        .collect::<Vec<_>>();
    assert_eq!(
        skill_hits
            .iter()
            .filter(|hit| hit["target_index"] == 0)
            .count(),
        13
    );
    assert_eq!(
        skill_hits
            .iter()
            .filter(|hit| hit["target_index"] == 1)
            .count(),
        1
    );
    assert_eq!(report["final_target_states"][1]["hp_current"], 0);
}

#[test]
fn adler_skill_recast_replaces_every_captured_target_stream() {
    let mut req = request();
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def = 1000;
    req.targets[0].hp_current = 100_000;
    req.targets[0].hp_max = 100_000;
    let mut second_target = req.targets[0].clone();
    second_target.target_index = 1;
    req.targets.push(second_target);
    req.actions = vec![
        def_action(
            0,
            "GA_Adler_Skill",
            "GE_Player_Adler_Skill2_Damage",
            Ratio::one(),
        ),
        def_action(
            500_000,
            "GA_Adler_Skill",
            "GE_Player_Adler_Skill2_Damage",
            Ratio::one(),
        ),
    ];
    req.relative_duration_us = 1_500_101;

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let skill_hits = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["ability_id"] == "GA_Adler_Skill")
        .collect::<Vec<_>>();
    let ticks = skill_hits
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Adler_Skill_Damage")
        .collect::<Vec<_>>();
    assert_eq!(ticks.len(), 2);
    assert!(
        ticks
            .iter()
            .all(|hit| { hit["command_index"] == 1 && hit["time_us"] == 1_500_100 })
    );
    assert_eq!(
        ticks
            .iter()
            .map(|hit| hit["target_index"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [0, 1]
    );
}

#[test]
fn adler_skill_all_in_range_semantics_acceptance() {
    let mut req = request();
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 1;
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::simulation::energy::ADLER_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::ADLER_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::adler::skill::BLESSING_HYPOTHESIS_ID.into(),
        crate::characters::adler::skill::BLESSING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::adler::skill::SKILL_ALL_IN_RANGE_HYPOTHESIS_ID.into(),
        crate::characters::adler::skill::SKILL_ALL_IN_RANGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actions = vec![def_action(
        0,
        "GA_Adler_Skill",
        "GE_Player_Adler_Skill2_Damage",
        Ratio::one(),
    )];
    req.relative_duration_us = 200;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["resolved_model_assumptions"]
            [crate::characters::adler::skill::SKILL_ALL_IN_RANGE_HYPOTHESIS_ID.as_str()],
        crate::characters::adler::skill::SKILL_ALL_IN_RANGE_HYPOTHESIS_ID.description()
    );
}
