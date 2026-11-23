use super::*;
use crate::simulation::tests::support::formula_damage_total;
use serde_json::json;

pub(super) fn make_action(
    timestamp_us: u64,
    actor_index: usize,
    ability_id: &str,
    damage_effect_id: &str,
    energy_cost: u64,
    cooldown_us: u64,
) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index,
        ability_id: ability_id.into(),
        damage_effect_id: damage_effect_id.into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost,
        recovery_us: 0,
        cooldown_us,
        hit_delay_us: 100,
    }
}

pub(super) fn add_assumption(req: &mut GameScenarioRequest, id: crate::HypothesisId) {
    req.model_assumptions
        .insert(id.as_str().into(), id.description().into());
}

pub(super) fn request() -> GameScenarioRequest {
    let mut req = sample_s1_request();
    req.actors[0].character_id = "character:1021".into();
    req.actors[0].weapon_id.clear();
    req.actors[0].gear_procs.clear();
    req.actors[0].set_id = None;
    req.actors[0].talent_levels = Some([1; 3]);
    req.actors[0].character_max_level = Some(80);
    req.actors[0].character_level = 80;
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].initial_energy = 100;
    req.actors[0].energy_capacity = 100;
    req.targets[0].resistance = Ratio::zero();
    req.targets[0].level = 80;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Edgar_Melee",
        "GE_Player_Edgar_Melee1_Damage",
        0,
        0,
    )];
    req
}

pub(super) fn set_a5_vitals(actor: &mut crate::simulation::GameActorConfig, base_hp: u64) {
    actor.static_stats.base_hp = Some(base_hp);
    actor.static_stats.hp_percent = Ratio {
        numerator: 1,
        denominator: 5,
    };
    actor.static_stats.hp = base_hp * 6 / 5;
    actor.initial_hp = actor.static_stats.hp;
    actor.initial_hp_max = actor.static_stats.hp;
}

#[test]
fn edgar_a6_def_hypothesis_is_a_strict_native_boundary() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 1;
    set_a5_vitals(&mut req.actors[0], 13_000);
    req.relative_duration_us = 11_000_000;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Edgar_UltraSkill",
        "GE_Player_Edgar_UltraSkill_Damage",
        100,
        0,
    )];
    add_assumption(
        &mut req,
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID,
    );
    add_assumption(
        &mut req,
        crate::characters::edgar::skill::PACKETS_HYPOTHESIS_ID,
    );
    add_assumption(
        &mut req,
        crate::characters::edgar::skill::HEALING_HYPOTHESIS_ID,
    );
    add_assumption(
        &mut req,
        crate::characters::edgar::skill::A6_DEF_HYPOTHESIS_ID,
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(
        report["resolved_model_assumptions"]
            [crate::characters::edgar::skill::A6_DEF_HYPOTHESIS_ID.as_str()],
        crate::characters::edgar::skill::A6_DEF_HYPOTHESIS_ID.description()
    );
}

#[test]
fn edgar_normal_chain_uses_all_stages_and_loops_with_independent_totals() {
    for (level, coefficients, total) in [
        (1, [309, 627, 390, 390, 833, 698, 309], 1778),
        (8, [530, 1075, 668, 668, 1428, 1196, 530], 6095),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([level, 1, 1]);
        req.actions = (0..6)
            .map(|index| {
                make_action(
                    index * 300_000,
                    0,
                    "GA_Edgar_Melee",
                    "GE_Player_Edgar_Melee1_Damage",
                    0,
                    0,
                )
            })
            .collect();
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["hits_settled"], 7);
        for (hit, coefficient) in report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .zip(coefficients)
        {
            let divisor = if coefficient % 2 == 0 { 1 } else { 2 };
            assert_eq!(
                hit["non_crit_damage"],
                json!(format!("{}/{}", coefficient / (2 / divisor), divisor))
            );
        }
        // Sum level-one ratios = 3.556; level-eight = 6.095, times 1000 ATK * 0.5 DEF.
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            json!(format!("{}/{}", total, if level == 1 { 1 } else { 2 }))
        );
        let packets: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|hit| hit["damage_effect_id"].as_str().unwrap())
            .collect();
        assert_eq!(
            packets,
            [
                "GE_Player_Edgar_Melee1_Damage",
                "GE_Player_Edgar_Melee2_Damage",
                "GE_Player_Edgar_Melee3_Damage",
                "GE_Player_Edgar_Melee3_Damage",
                "GE_Player_Edgar_Melee4_Damage",
                "GE_Player_Edgar_Melee5_Damage",
                "GE_Player_Edgar_Melee1_Damage"
            ]
        );
    }
}

#[test]
fn edgar_normal_chain_strict_requires_own_hypothesis() {
    let mut req = request();
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 1;
    add_assumption(
        &mut req,
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID,
    );
    add_assumption(
        &mut req,
        crate::characters::edgar::skill::CHAIN_HYPOTHESIS_ID,
    );
    add_assumption(
        &mut req,
        crate::simulation::energy::EDGAR_HIT_ENERGY_HYPOTHESIS_ID,
    );
    req.actions = (0..5)
        .map(|index| {
            make_action(
                index * 1000,
                0,
                "GA_Edgar_Melee",
                "GE_Player_Edgar_Melee1_Damage",
                0,
                0,
            )
        })
        .collect();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 6);
}

#[test]
fn edgar_normal_chain_resets_after_gap_other_ability_and_field_exit() {
    for boundary in [0, 1, 2] {
        let mut req = request();
        req.relative_duration_us = 5_000_000;
        req.actions = vec![make_action(
            0,
            0,
            "GA_Edgar_Melee",
            "GE_Player_Edgar_Melee1_Damage",
            0,
            0,
        )];
        match boundary {
            0 => {}
            1 => req.actions.push(make_action(
                1000,
                0,
                "GA_Edgar_Skill",
                "GE_Player_Edgar_Skill_Damage",
                0,
                0,
            )),
            _ => {
                req.actors.push(req.actors[0].clone());
                req.actions.push(GameActionCommand::SwitchActor {
                    timestamp_us: 1000,
                    target_actor_index: 1,
                    cooldown_us: 0,
                });
                req.actions.push(GameActionCommand::SwitchActor {
                    timestamp_us: 2000,
                    target_actor_index: 0,
                    cooldown_us: 0,
                });
            }
        }
        req.actions.push(make_action(
            if boundary == 0 { 2_000_001 } else { 3000 },
            0,
            "GA_Edgar_Melee",
            "GE_Player_Edgar_Melee1_Damage",
            0,
            0,
        ));
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(
            report["hit_receipts"].as_array().unwrap().last().unwrap()["damage_effect_id"],
            "GE_Player_Edgar_Melee1_Damage"
        );
    }
}

#[test]
fn edgar_normal_chain_preserves_noop_switch_and_exact_gap_boundary() {
    let mut req = request();
    req.relative_duration_us = 3_000_000;
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 1000,
        target_actor_index: 0,
        cooldown_us: 0,
    });
    req.actions.push(make_action(
        2_000_000,
        0,
        "GA_Edgar_Melee",
        "GE_Player_Edgar_Melee1_Damage",
        0,
        0,
    ));
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 2);
    assert_eq!(
        report["hit_receipts"][1]["damage_effect_id"],
        "GE_Player_Edgar_Melee2_Damage"
    );
}

#[test]
fn edgar_third_stage_pending_projectiles_keep_owner_after_switch() {
    let mut req = request();
    req.actors.push(req.actors[0].clone());
    req.actions = (0..3)
        .map(|index| {
            make_action(
                index * 1000,
                0,
                "GA_Edgar_Melee",
                "GE_Player_Edgar_Melee1_Damage",
                0,
                0,
            )
        })
        .collect();
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 2050,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    req.relative_duration_us = 2099;
    let pending = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(pending["hits_settled"], 2);
    req.relative_duration_us = 2100;
    let delivered = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(delivered["hits_settled"], 4);
    for hit in &delivered["hit_receipts"].as_array().unwrap()[2..] {
        assert_eq!(hit["actor_index"], 0);
        assert_eq!(hit["damage_effect_id"], "GE_Player_Edgar_Melee3_Damage");
    }
}

#[test]
fn edgar_normal_skill_and_burst_use_native_talent_coefficients_and_resonance_bonus() {
    // Normal attack (GA_Edgar_Melee / GE_Player_Edgar_Melee1_Damage)
    // Coeffs: Lv1 = 0.309, Lv8 = 0.530.
    // Edgar's resonance_3 modifies GA_Edgar_QTE, GA_Edgar_Skill, GA_Edgar_UltraSkill (not Melee).
    // At A0..A3: ATK = 1000, DEF factor = 0.5.
    //   Lv1: 1000 * 0.309 * 0.5 = 309 / 2
    //   Lv8: 1000 * 0.530 * 0.5 = 265 / 1
    //   Lv8 at A3: still 265 / 1 (resonance_3 does not include Melee).
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 309, 2),
        (8, 0, 265, 1),
        (8, 1, 265, 1),
        (8, 3, 265, 1),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([level, 1, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 1);
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            json!(format!("{}/{}", expected_num, expected_den))
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"
        );
    }

    // Skill (GA_Edgar_Skill / GE_Player_Edgar_Skill_Damage + 6 child packets)
    // Coeffs: Lv1 = 0.800, Lv8 = 1.371, Lv9 (A3 resonance) = 1.481.
    // At A0: ATK = 1000, DEF factor = 0.5.
    //   Lv1: 1000 * 0.800 * 0.5 = 400 / 1 per hit -> total 7 * 400 = 2800 / 1
    //   Lv8: 1000 * 1.371 * 0.5 = 1371 / 2 per hit -> total 7 * 1371 / 2 = 9597 / 2
    // At A3: resonance_3 grants +1 level -> Lv9:
    //   Lv9: 1000 * 1.481 * 0.5 = 1481 / 2 per hit -> total 7 * 1481 / 2 = 10367 / 2
    // At A4: A4 grants +50% (1/2) Skill damage bonus -> factor (1 + 1/2) = 3/2:
    //   Lv9: (1481 / 2) * (3 / 2) = 4443 / 4 per hit -> total 7 * 4443 / 4 = 31101 / 4
    for (level, awakening, expected_num, expected_den, total_num, total_den) in [
        (1, 0, 400, 1, 2800, 1),
        (8, 0, 1371, 2, 9597, 2),
        (8, 3, 1481, 2, 10367, 2),
        (8, 4, 4443, 4, 31101, 4),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, level, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Edgar_Skill",
            "GE_Player_Edgar_Skill_Damage",
            0,
            16_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 7);
        let receipts = report["hit_receipts"].as_array().unwrap();
        assert_eq!(receipts.len(), 7);
        for receipt in receipts {
            assert_eq!(
                receipt["non_crit_damage"],
                json!(format!("{}/{}", expected_num, expected_den))
            );
            assert_eq!(receipt["damage_effect_id"], "GE_Player_Edgar_Skill_Damage");
            assert_eq!(
                super::support::hit_metadata(&report, receipt)["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
            );
        }
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            json!(format!("{}/{}", total_num, total_den))
        );
    }

    // Burst (GA_Edgar_UltraSkill / GE_Player_Edgar_UltraSkill_Damage)
    // Coeffs: Lv1 = 4.000, Lv8 = 6.855, Lv9 (A3 resonance) = 7.404.
    // At A0: ATK = 1000, DEF factor = 0.5.
    //   Lv1: 1000 * 4.000 * 0.5 = 2000 / 1
    //   Lv8: 1000 * 6.855 * 0.5 = 6855 / 2
    // At A3: resonance_3 grants +1 level -> Lv9:
    //   Lv9: 1000 * 7.404 * 0.5 = 3702 / 1
    for (level, awakening, expected_num, expected_den) in
        [(1, 0, 2000, 1), (8, 0, 6855, 2), (8, 3, 3702, 1)]
    {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, 1, level]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
            100,
            20_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 1);
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            json!(format!("{}/{}", expected_num, expected_den))
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
        );
    }
}

#[test]
fn edgar_key_extends_a0_wake_without_changing_packet_damage() {
    let mut req = request();
    req.actors[0].talent_levels = Some([1, 8, 8]);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Edgar_Skill",
            "GE_Player_Edgar_Skill_Damage",
            0,
            16_000_000,
        ),
        make_action(
            1_000_000,
            0,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
            100,
            20_000_000,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 8);
    assert_eq!(
        report["hit_receipts"][7]["non_crit_damage"],
        json!("6855/2")
    );
    let ticks = healing_ticks(&report);
    assert_eq!(ticks.len(), 11);
    assert_eq!(
        ticks
            .iter()
            .map(|tick| tick["time_us"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        (2..=12)
            .map(|second| second * 1_000_000 + 3_402_224)
            .collect::<Vec<_>>()
    );
}

#[test]
fn edgar_teammate_qte_grants_a1_key_and_three_keys_cap_wake_at_thirteen_ticks() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    req.actors.push(ally);
    req.actions = vec![
        GameActionCommand::SwitchActor {
            timestamp_us: 0,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        make_action(1, 1, "GA_Mint019_QTE", "GE_Player_Mint_QTE1_Damage", 0, 0),
        GameActionCommand::SwitchActor {
            timestamp_us: 2,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        make_action(
            1_000_000,
            0,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
            100,
            0,
        ),
    ];
    req.relative_duration_us = 12_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true, "{report}");
    assert_eq!(healing_ticks(&report).len(), 11);
    let grants: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "edgar_key_of_truth_granted")
        .collect();
    assert_eq!(grants.len(), 1);
    assert_eq!(grants[0]["data"]["stacks_before"], 0);
    assert_eq!(grants[0]["data"]["stacks"], 1);
    assert_eq!(grants[0]["data"]["max_stacks"], 3);

    let mut capped = request();
    capped.relative_duration_us = 17_000_000;
    capped.actions = [0, 1_000_000, 2_000_000]
        .into_iter()
        .map(|time| {
            make_action(
                time,
                0,
                "GA_Edgar_Skill",
                "GE_Player_Edgar_Skill_Damage",
                0,
                0,
            )
        })
        .chain([
            make_action(
                3_000_000,
                0,
                "GA_Edgar_QTE",
                "GE_Player_Edgar_QTE_Damage",
                0,
                0,
            ),
            make_action(
                4_000_000,
                0,
                "GA_Edgar_UltraSkill",
                "GE_Player_Edgar_UltraSkill_Damage",
                100,
                0,
            ),
        ])
        .collect();
    let capped_report = run_game_scenario(&capped, &ExecutionLimits::default()).unwrap();
    assert_eq!(healing_ticks(&capped_report).len(), 13);
    let grants: Vec<_> = capped_report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "edgar_key_of_truth_granted")
        .collect();
    assert_eq!(grants.len(), 4);
    assert_eq!(grants[3]["data"]["stacks_before"], 3);
    assert_eq!(grants[3]["data"]["stacks"], 3);
    assert_eq!(grants[3]["data"]["max_stacks"], 3);
    let sampled = capped_report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["event_kind"] == "edgar_key_of_truth_sampled")
        .unwrap();
    assert_eq!(sampled["data"]["stacks"], 3);
    assert_eq!(sampled["data"]["max_stacks"], 3);
    assert_eq!(sampled["data"]["consumed"], 0);
}

#[test]
fn edgar_teammate_qte_skill_a4_train_and_burst_share_the_key_of_truth_path() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([1, 4]).unwrap();
    req.actors[0].talent_levels = Some([1, 8, 1]);
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.awakening = ntesim_data::AwakeningSelection::prefix(0);
    req.actors.push(ally);
    req.relative_duration_us = 14_000_000;
    req.actions = vec![
        GameActionCommand::SwitchActor {
            timestamp_us: 0,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        make_action(1, 1, "GA_Mint019_QTE", "GE_Player_Mint_QTE1_Damage", 0, 0),
        GameActionCommand::SwitchActor {
            timestamp_us: 2,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        make_action(
            1_000_000,
            0,
            "GA_Edgar_Skill",
            "GE_Player_Edgar_Skill_Damage",
            0,
            0,
        ),
        make_action(
            2_000_000,
            0,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
            100,
            0,
        ),
    ];

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true, "{report}");
    let edgar_hits: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["actor_index"] == 0)
        .collect();
    assert_eq!(edgar_hits.len(), 8, "{report}");

    // A1 + A4 are independently selected, so A4's +50% applies without R3.
    // Lv8 Skill: 1000 ATK * 1.371 * 1/2 DEF * (1 + 1/2 A4) = 4113/4 per hit.
    assert_eq!(
        edgar_hits[..7]
            .iter()
            .map(|hit| hit["damage_effect_id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["GE_Player_Edgar_Skill_Damage"; 7]
    );
    for hit in &edgar_hits[..7] {
        assert_eq!(hit["non_crit_damage"], json!("4113/4"));
    }
    assert_eq!(
        edgar_hits[7]["damage_effect_id"],
        "GE_Player_Edgar_UltraSkill_Damage"
    );
    assert_eq!(edgar_hits[7]["non_crit_damage"], json!("2000/1"));

    let grants: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "edgar_key_of_truth_granted")
        .collect();
    assert_eq!(grants.len(), 2, "{report}");
    assert_eq!(grants[0]["data"]["stacks_before"], 0);
    assert_eq!(grants[0]["data"]["stacks"], 1);
    assert_eq!(grants[0]["data"]["max_stacks"], 3);
    assert_eq!(grants[1]["data"]["stacks_before"], 1);
    assert_eq!(grants[1]["data"]["stacks"], 2);
    assert_eq!(grants[1]["data"]["max_stacks"], 3);
    let sampled = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["event_kind"] == "edgar_key_of_truth_sampled")
        .unwrap();
    assert_eq!(sampled["data"]["stacks"], 2);
    assert_eq!(sampled["data"]["max_stacks"], 3);
    assert_eq!(sampled["data"]["consumed"], 0);
    assert_eq!(sampled["data"]["expires_at_us"], 14_000_000);
}

#[test]
fn edgar_own_qte_grants_once_and_keys_gained_during_wake_extend_only_the_next_burst() {
    let mut own_qte = request();
    own_qte.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
    own_qte.actions = vec![
        make_action(0, 0, "GA_Edgar_QTE", "GE_Player_Edgar_QTE_Damage", 0, 0),
        make_action(
            1_000_000,
            0,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
            100,
            0,
        ),
    ];
    own_qte.relative_duration_us = 12_000_000;
    let own_report = run_game_scenario(&own_qte, &ExecutionLimits::default()).unwrap();
    assert_eq!(healing_ticks(&own_report).len(), 11);
    assert_eq!(
        own_report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "edgar_key_of_truth_granted")
            .count(),
        1
    );

    let mut later_key = request();
    later_key.actors[0].initial_energy = 200;
    later_key.actors[0].energy_capacity = 200;
    later_key.relative_duration_us = 11_000_000;
    later_key.actions = vec![
        make_action(
            0,
            0,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
            100,
            0,
        ),
        make_action(
            2_000_000,
            0,
            "GA_Edgar_Skill",
            "GE_Player_Edgar_Skill_Damage",
            0,
            0,
        ),
    ];
    let later_report = run_game_scenario(&later_key, &ExecutionLimits::default()).unwrap();
    assert_eq!(healing_ticks(&later_report).len(), 10);
    assert!(
        later_report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_kind"] == "edgar_key_of_truth_sampled"
                && event["data"]["stacks"] == 0
                && event["data"]["max_stacks"] == 3
                && event["data"]["consumed"] == 0)
    );

    later_key.actions.push(make_action(
        12_000_001,
        0,
        "GA_Edgar_UltraSkill",
        "GE_Player_Edgar_UltraSkill_Damage",
        100,
        0,
    ));
    later_key.relative_duration_us = 22_000_000;
    let persisted_report = run_game_scenario(&later_key, &ExecutionLimits::default()).unwrap();
    assert_eq!(healing_ticks(&persisted_report).len(), 21);
    let sampled: Vec<_> = persisted_report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "edgar_key_of_truth_sampled")
        .collect();
    assert_eq!(sampled.len(), 2);
    assert_eq!(sampled[0]["data"]["stacks"], 0);
    assert_eq!(sampled[0]["data"]["max_stacks"], 3);
    assert_eq!(sampled[0]["data"]["consumed"], 0);
    assert_eq!(sampled[1]["data"]["stacks"], 1);
    assert_eq!(sampled[1]["data"]["max_stacks"], 3);
    assert_eq!(sampled[1]["data"]["consumed"], 0);
}

#[test]
fn edgar_qte_grants_other_edgar_by_actor_index() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
    let mut other_edgar = req.actors[0].clone();
    other_edgar.slot = 1;
    req.actors.push(other_edgar);
    req.actions = vec![make_action(
        0,
        0,
        "GA_Edgar_QTE",
        "GE_Player_Edgar_QTE_Damage",
        0,
        0,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let grants: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "edgar_key_of_truth_granted")
        .collect();
    assert_eq!(grants.len(), 2);
    assert!(grants.iter().any(|event| {
        event["from_actor"] == 0
            && event["to_actor"] == 0
            && event["data"]["ability_id"] == "GA_Edgar_QTE"
            && event["data"]["stacks_before"] == 0
            && event["data"]["stacks"] == 1
            && event["data"]["max_stacks"] == 3
    }));
    assert!(grants.iter().any(|event| {
        event["from_actor"] == 0
            && event["to_actor"] == 1
            && event["data"]["reason"] == "teammate_qte"
            && event["data"]["stacks_before"] == 0
            && event["data"]["stacks"] == 1
            && event["data"]["max_stacks"] == 3
    }));
}

#[test]
fn edgar_key_hypothesis_is_required_in_strict_mode_and_accepted_for_own_qte() {
    let mut req = request();
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 1;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Edgar_QTE",
        "GE_Player_Edgar_QTE_Damage",
        0,
        0,
    )];
    add_assumption(
        &mut req,
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID,
    );
    let qte_role =
        crate::characters::action_roles::resolve("character:1021", "GA_Edgar_QTE").unwrap();
    req.model_assumptions
        .insert(qte_role.hypothesis_id.into(), qte_role.description.into());
    add_assumption(&mut req, crate::characters::edgar::skill::KEY_HYPOTHESIS_ID);
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
            .filter(|event| event["event_kind"] == "edgar_key_of_truth_granted")
            .count(),
        1
    );
}

#[test]
fn edgar_a1_key_ignores_non_qte_actions_and_failed_qte_admission() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    req.actors.push(ally);
    req.actions = vec![
        GameActionCommand::SwitchActor {
            timestamp_us: 0,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        make_action(
            1,
            1,
            "GA_Mint019_Skill",
            "GE_Player_Mint_Skill1_Damage_New",
            0,
            0,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 2,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        make_action(
            3,
            0,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
            100,
            0,
        ),
    ];
    req.relative_duration_us = 10_000_003;
    let non_qte_report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(healing_ticks(&non_qte_report).len(), 10);
    assert!(
        non_qte_report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| event["event_kind"] != "edgar_key_of_truth_granted")
    );

    req.actions = vec![
        GameActionCommand::SwitchActor {
            timestamp_us: 0,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        make_action(
            1,
            1,
            "GA_Mint019_QTE",
            "GE_Player_Mint_QTE1_Damage",
            100,
            10_000_000,
        ),
        make_action(2, 1, "GA_Mint019_QTE", "GE_Player_Mint_QTE1_Damage", 1, 0),
        GameActionCommand::SwitchActor {
            timestamp_us: 3,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        make_action(
            4,
            0,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
            100,
            0,
        ),
    ];
    req.relative_duration_us = 10_000_004;
    let failed_report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let grants: Vec<_> = failed_report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "edgar_key_of_truth_granted")
        .collect();
    assert_eq!(grants.len(), 1);
    assert_eq!(grants[0]["time_us"], 1);
    assert_eq!(grants[0]["data"]["stacks_before"], 0);
    assert_eq!(grants[0]["data"]["stacks"], 1);
    assert_eq!(grants[0]["data"]["max_stacks"], 3);
    assert_eq!(failed_report["rejected_action"]["time_us"], 2);

    let mut failed_only = request();
    failed_only.actions = vec![make_action(
        0,
        0,
        "GA_Edgar_QTE",
        "GE_Player_Edgar_QTE_Damage",
        101,
        0,
    )];
    let failed_only_report = run_game_scenario(&failed_only, &ExecutionLimits::default()).unwrap();
    assert_eq!(failed_only_report["execution_completed"], false);
    assert_eq!(
        failed_only_report["rejected_action"]["time_us"], 0,
        "{failed_only_report}"
    );
    assert!(
        failed_only_report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| event["event_kind"] != "edgar_key_of_truth_granted")
    );
}

#[test]
fn edgar_team_qte_grants_no_key_to_or_from_a_defeated_actor() {
    let mut dead_owner = request();
    dead_owner.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
    dead_owner.actors[0].initial_hp = 0;
    let mut living_ally = dead_owner.actors[0].clone();
    living_ally.slot = 1;
    living_ally.character_id = "character:1019".into();
    living_ally.initial_hp = 100;
    dead_owner.actors.push(living_ally);
    dead_owner.actions = vec![
        GameActionCommand::SwitchActor {
            timestamp_us: 0,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        make_action(1, 1, "GA_Mint019_QTE", "GE_Player_Mint_QTE1_Damage", 0, 0),
    ];
    let dead_owner_report = run_game_scenario(&dead_owner, &ExecutionLimits::default()).unwrap();
    assert_eq!(dead_owner_report["execution_completed"], true);
    assert_eq!(dead_owner_report["hits_settled"], 1);
    assert!(
        dead_owner_report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| event["event_kind"] != "edgar_key_of_truth_granted")
    );

    let mut dead_caster = request();
    dead_caster.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
    let mut defeated_ally = dead_caster.actors[0].clone();
    defeated_ally.slot = 1;
    defeated_ally.character_id = "character:1019".into();
    defeated_ally.initial_hp = 0;
    dead_caster.actors.push(defeated_ally);
    dead_caster.actions = vec![make_action(
        0,
        1,
        "GA_Mint019_QTE",
        "GE_Player_Mint_QTE1_Damage",
        0,
        0,
    )];
    let dead_caster_report = run_game_scenario(&dead_caster, &ExecutionLimits::default()).unwrap();
    assert!(!dead_caster_report["rejected_action"].is_null());
    assert!(
        dead_caster_report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| event["event_kind"] != "edgar_key_of_truth_granted")
    );
}

#[test]
fn edgar_off_field_delivery_and_target_boundary_checks() {
    let mut req = request();
    let mut ally = req.actors[0].clone();
    ally.character_id = "character:1019".into();
    ally.slot = 1;
    req.actors.push(ally);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Edgar_Skill",
            "GE_Player_Edgar_Skill_Damage",
            0,
            16_000_000,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 10,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 7);
    for receipt in report["hit_receipts"].as_array().unwrap() {
        assert_eq!(receipt["actor_index"], 0);
    }

    let mut req_dead = request();
    req_dead.targets[0].hp_current = 0;
    let report_dead = run_game_scenario(&req_dead, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_dead["execution_completed"], true);
    assert_eq!(report_dead["hits_settled"], 0);
}

#[test]
fn edgar_strict_mode_rejects_missing_hypothesis_and_accepts_valid_hypothesis() {
    let mut req = request();
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 8;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Edgar_Skill",
        "GE_Player_Edgar_Skill_Damage",
        0,
        16_000_000,
    )];
    add_assumption(
        &mut req,
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID,
    );
    add_assumption(
        &mut req,
        crate::characters::edgar::skill::PACKETS_HYPOTHESIS_ID,
    );
    add_assumption(&mut req, crate::characters::edgar::skill::KEY_HYPOTHESIS_ID);
    add_assumption(
        &mut req,
        crate::simulation::energy::EDGAR_HIT_ENERGY_HYPOTHESIS_ID,
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 7);
}

#[test]
fn edgar_two_skill_one_burst_rotation_hit_count_and_energy() {
    let mut req = request();
    req.actors[0].talent_levels = Some([1, 8, 8]);
    req.actors[0].initial_energy = 100;
    req.actors[0].energy_capacity = 100;
    req.relative_duration_us = 35_000_000;
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Edgar_Skill",
            "GE_Player_Edgar_Skill_Damage",
            0,
            16_000_000,
        ),
        make_action(
            1_000_000,
            0,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
            100,
            20_000_000,
        ),
        make_action(
            19_402_224,
            0,
            "GA_Edgar_Skill",
            "GE_Player_Edgar_Skill_Damage",
            0,
            16_000_000,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 15);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
}

fn healing_ticks(report: &serde_json::Value) -> Vec<serde_json::Value> {
    report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "edgar_healing_tick")
        .cloned()
        .collect()
}

#[test]
fn edgar_ultimate_healing_curves_follow_native_burst_coordinate() {
    for (purchase, awakening, flat, permyriad) in
        [(1, 0, 350, 110), (8, 0, 600, 188), (8, 3, 648, 203)]
    {
        let mut req = request();
        req.relative_duration_us = 1_000_000;
        req.actors[0].initial_hp = 1_000;
        req.actors[0].initial_hp_max = 10_000;
        req.actors[0].talent_levels = Some([1, 1, purchase]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
            0,
            0,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        let ticks = healing_ticks(&report);
        assert_eq!(ticks.len(), 1);
        assert_eq!(ticks[0]["time_us"], 4_402_224);
        let expected = flat + (10_000 * permyriad + 5_000) / 10_000;
        assert_eq!(ticks[0]["data"]["gross"], expected);
    }
}

#[test]
fn edgar_healing_tracks_field_switch_overflow_deadlines_and_recast_generation() {
    let mut req = request();
    req.relative_duration_us = 12_000_000;
    req.actors[0].initial_hp = 10_000;
    req.actors[0].initial_hp_max = 10_000;
    req.actors[0].talent_levels = Some([1, 1, 1]);
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.initial_hp = 100;
    ally.initial_hp_max = 2_000;
    req.actors.push(ally);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
            0,
            0,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 1_500_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 3_500_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        make_action(
            4_000_000,
            0,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let ticks = healing_ticks(&report);
    let times = ticks
        .iter()
        .map(|tick| tick["time_us"].as_u64().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        times,
        [
            2_500_000, 3_500_000, 8_402_224, 9_402_224, 10_402_224, 11_402_224, 12_402_224,
            13_402_224, 14_402_224, 15_402_224, 16_402_224
        ]
    );
    assert!(ticks.iter().all(|tick| tick["phase"] == "BuffTickOrExpiry"));
    assert_eq!(ticks[0]["data"]["applied"], 460);
    assert_eq!(ticks[1]["active_actor"], 0);
    assert_eq!(ticks[1]["to_actor"], 0);
    assert_eq!(report["final_actor_states"][1]["hp"], 560);
    assert_eq!(report["final_actor_states"][0]["hp"], 10_000);
    assert_eq!(
        ticks
            .iter()
            .filter(|tick| tick["data"]["applied"] == 0 && tick["data"]["overflow"] == 460)
            .count(),
        10
    );

    req.relative_duration_us = 9_999_999;
    req.actions.truncate(2);
    let prior = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(healing_ticks(&prior).len(), 9);
    assert_eq!(prior["execution_completed"], true);
    req.relative_duration_us = 10_000_000;
    let exact = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(healing_ticks(&exact).len(), 10);
    assert_eq!(exact["execution_completed"], true);
}

#[test]
fn edgar_healing_strict_hypothesis_and_failed_admission_control_stream_creation() {
    let mut req = request();
    req.actors[0].talent_levels = None;
    add_assumption(
        &mut req,
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID,
    );
    req.actions = vec![make_action(
        0,
        0,
        "GA_Edgar_UltraSkill",
        "GE_Player_Edgar_UltraSkill_Damage",
        0,
        0,
    )];
    add_assumption(
        &mut req,
        crate::characters::edgar::skill::HEALING_HYPOTHESIS_ID,
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(healing_ticks(&report).len(), 10);
    req.actors[0].initial_energy = 0;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Edgar_UltraSkill",
        "GE_Player_Edgar_UltraSkill_Damage",
        100,
        0,
    )];
    let failed = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(failed["execution_completed"], false);
    assert!(healing_ticks(&failed).is_empty());
}

#[test]
fn edgar_a5_burst_heals_from_projected_max_hp_and_rejects_missing_or_contradictory_provenance() {
    let mut req = request();
    req.relative_duration_us = 1_000_000;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(5);
    set_a5_vitals(&mut req.actors[0], 10_000);
    req.actors[0].static_stats.hp_flat = Ratio::zero();
    req.actors[0].initial_hp = 1;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Edgar_UltraSkill",
        "GE_Player_Edgar_UltraSkill_Damage",
        0,
        0,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let ticks = healing_ticks(&report);
    assert_eq!(ticks.len(), 1);
    assert_eq!(ticks[0]["data"]["hp_max"], 12_000);
    assert_eq!(ticks[0]["data"]["gross"], 520);
    assert_eq!(report["final_actor_states"][0]["hp_max"], 12_000);
    assert_eq!(report["final_actor_states"][0]["hp"], 521);
    let repeated = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(repeated["final_actor_states"][0]["hp_max"], 12_000);
    let mut full = req.clone();
    full.actors[0].initial_hp = 12_000;
    let full_report = run_game_scenario(&full, &ExecutionLimits::default()).unwrap();
    assert_eq!(full_report["final_actor_states"][0]["hp"], 12_000);

    let mut missing = req.clone();
    missing.actors[0].static_stats.base_hp = None;
    let error = run_game_scenario(&missing, &ExecutionLimits::default()).unwrap_err();
    assert!(error.contains("Edgar A5 Burst requires base-HP provenance"));

    let mut omitted = req.clone();
    omitted.actors[0].static_stats.hp_percent = Ratio::zero();
    omitted.actors[0].static_stats.hp = 10_000;
    omitted.actors[0].initial_hp_max = 10_000;
    let error = run_game_scenario(&omitted, &ExecutionLimits::default()).unwrap_err();
    assert!(error.contains("HP percent pool omits the A5 contribution"));

    let mut contradictory = req;
    contradictory.actors[0].initial_hp_max = 11_999;
    let error = run_game_scenario(&contradictory, &ExecutionLimits::default()).unwrap_err();
    assert!(error.contains("Edgar A5 MaxHP provenance mismatch"));
}

#[test]
fn edgar_healing_callback_cannot_revive_a_dead_source_or_recipient() {
    let mut req = request();
    req.actors[0].static_stats.healing_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    req.actors.push(ally);
    let event = crate::callbacks::ContentEvent::EdgarHealTick {
        actor_index: 0,
        generation: 1,
        end_time_us: 10_000_000,
        interval_us: 1_000_000,
        flat_heal: crate::simulation::ExactFraction::new(350, 1).unwrap(),
        max_hp_ratio: crate::simulation::ExactFraction::new(11, 1000).unwrap(),
    };

    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    state.advance_unpaused_test_clock(1_000_000);
    state.active_actor_index = 1;
    state.actors[1].hp = 0;
    if let crate::characters::RuntimeState::Edgar(edgar) = &mut state.actors[0].character {
        edgar.healing_generation = 1;
    }
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    crate::callbacks::apply(
        event.clone(),
        &mut crate::runtime_api::CallbackContext {
            state: &mut state,
            request: &req,
            queue: &mut queue,
            sequence: &mut sequence,
            trace: &mut trace,
        },
    )
    .unwrap();
    assert_eq!(state.actors[1].hp, 0);
    assert!(
        trace[0]
            .data
            .as_ref()
            .is_some_and(|d| d.target_alive == Some(false) && d.applied == Some(0))
    );

    state.actors[0].hp = 0;
    if let crate::characters::RuntimeState::Edgar(edgar) = &mut state.actors[0].character {
        edgar.healing_generation = 1;
    }
    trace.clear();
    queue.clear();
    crate::callbacks::apply(
        event,
        &mut crate::runtime_api::CallbackContext {
            state: &mut state,
            request: &req,
            queue: &mut queue,
            sequence: &mut sequence,
            trace: &mut trace,
        },
    )
    .unwrap();
    assert_eq!(trace[0].event_kind, "edgar_healing_cancelled");
    assert!(queue.is_empty());
}

#[test]
fn edgar_healing_rejects_zero_denominator_static_source_bonus() {
    let mut req = request();
    req.actors[0].static_stats.healing_bonus = Ratio {
        numerator: 1,
        denominator: 0,
    };
    let error = run_game_scenario(&req, &ExecutionLimits::default()).unwrap_err();
    assert!(error.contains("fraction denominator cannot be zero"));
}

#[test]
fn edgar_healing_callback_samples_live_source_max_hp_and_applies_healing_modifiers() {
    let mut req = request();
    let event = crate::callbacks::ContentEvent::EdgarHealTick {
        actor_index: 0,
        generation: 1,
        end_time_us: 10_000_000,
        interval_us: 1_000_000,
        flat_heal: crate::simulation::ExactFraction::new(350, 1).unwrap(),
        max_hp_ratio: crate::simulation::ExactFraction::new(11, 1000).unwrap(),
    };
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    state.advance_unpaused_test_clock(1_000_000);
    state.actors[0].hp = 100;
    state.actors[0].hp_max = 20_000;
    if let crate::characters::RuntimeState::Edgar(edgar) = &mut state.actors[0].character {
        edgar.healing_generation = 1;
    }
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    crate::callbacks::apply(
        event.clone(),
        &mut crate::runtime_api::CallbackContext {
            state: &mut state,
            request: &req,
            queue: &mut queue,
            sequence: &mut sequence,
            trace: &mut trace,
        },
    )
    .unwrap();
    assert_eq!(
        trace[0].data.as_ref().and_then(|d| d.hp_max.zip(d.gross)),
        Some((20_000, 570))
    );
    assert_eq!(state.actors[0].hp, 670);

    let mut fractional_req = request();
    let mut ally = fractional_req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.initial_hp = 100;
    ally.initial_hp_max = 2_000;
    fractional_req.actors.push(ally);
    let mut fractional_state = crate::simulation::state::GameSimulatorState::new(&fractional_req);
    fractional_state.advance_unpaused_test_clock(1_000_000);
    fractional_state.active_actor_index = 1;
    fractional_state.actors[0].hp_max = 50;
    if let crate::characters::RuntimeState::Edgar(edgar) = &mut fractional_state.actors[0].character
    {
        edgar.healing_generation = 1;
    }
    let mut fractional_queue = crate::simulation::timer_queue::EventQueue::new();
    let mut fractional_sequence = 0;
    let mut fractional_trace = Vec::new();
    crate::callbacks::apply(
        event.clone(),
        &mut crate::runtime_api::CallbackContext {
            state: &mut fractional_state,
            request: &fractional_req,
            queue: &mut fractional_queue,
            sequence: &mut fractional_sequence,
            trace: &mut fractional_trace,
        },
    )
    .unwrap();
    assert!(fractional_trace[0].data.as_ref().is_some_and(|d| {
        d.gross == Some(351) && d.applied == Some(351) && d.overflow == Some(0)
    }));
    assert_eq!(fractional_state.actors[1].hp, 451);

    for (modifier, actor_index) in [("healing_bonus", 0), ("incoming_healing_bonus", 0)] {
        req.actors[0].initial_hp = 100;
        let mut state = crate::simulation::state::GameSimulatorState::new(&req);
        state.advance_unpaused_test_clock(1_000_000);
        state.actors[0].hp_max = 20_000;
        state.actors[0].hp = 100;
        state
            .apply_buff(
                0,
                crate::simulation::state::TimedBuffDeclaration::reset(
                    "test.edgar_healing_modifier",
                    "test",
                    ntesim_core::buffs::dispatch::BuffRecipient::SpecificActor(actor_index),
                    10_000_000,
                    ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                    std::collections::BTreeMap::from([(
                        modifier.into(),
                        crate::simulation::ExactFraction::new(1, 10).unwrap(),
                    )]),
                ),
            )
            .unwrap();
        if let crate::characters::RuntimeState::Edgar(edgar) = &mut state.actors[0].character {
            edgar.healing_generation = 1;
        }
        let mut queue = crate::simulation::timer_queue::EventQueue::new();
        let mut sequence = 0;
        let mut trace = Vec::new();
        crate::callbacks::apply(
            event.clone(),
            &mut crate::runtime_api::CallbackContext {
                state: &mut state,
                request: &req,
                queue: &mut queue,
                sequence: &mut sequence,
                trace: &mut trace,
            },
        )
        .unwrap();
        assert_eq!(
            trace[0].data.as_ref().and_then(|d| d.gross.zip(d.applied)),
            Some((627, 627))
        );
        assert_eq!(state.actors[0].hp, 727);
        assert_eq!(
            state.execution_model_assumptions
                [crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID.as_str()],
            crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID.description()
        );
    }
}

#[test]
fn edgar_heal_uses_stacked_source_outgoing_and_recipient_incoming_only() {
    let mut req = request();
    for (slot, character_id) in [(1, "character:1019"), (2, "character:1042")] {
        let mut ally = req.actors[0].clone();
        ally.slot = slot;
        ally.character_id = character_id.into();
        ally.initial_hp = 10;
        req.actors.push(ally);
    }
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    state.advance_unpaused_test_clock(1);
    state.active_actor_index = 1;
    state.actors[0].hp_max = 50;
    state.actors[1].hp = 10;
    state.actors[2].hp = 10;
    if let crate::characters::RuntimeState::Edgar(edgar) = &mut state.actors[0].character {
        edgar.healing_generation = 1;
    }
    let mut apply = |id: &str, actor: usize, stat: &str, numerator: u128, denominator: u128| {
        state
            .apply_buff(
                0,
                crate::simulation::state::TimedBuffDeclaration::reset(
                    id,
                    "test",
                    ntesim_core::buffs::dispatch::BuffRecipient::SpecificActor(actor),
                    100,
                    ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                    std::collections::BTreeMap::from([(
                        stat.into(),
                        crate::simulation::ExactFraction::new(numerator, denominator).unwrap(),
                    )]),
                ),
            )
            .unwrap();
    };
    apply("outgoing.one", 0, "healing_bonus", 1, 10);
    apply("outgoing.two", 0, "healing_bonus", 1, 10);
    apply("source.incoming", 0, "incoming_healing_bonus", 9, 10);
    apply("recipient.incoming", 1, "incoming_healing_bonus", 1, 5);
    apply("recipient.outgoing", 1, "healing_bonus", 9, 10);
    req.actors[0].static_stats.healing_bonus = Ratio {
        numerator: 1,
        denominator: 5,
    };

    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    crate::callbacks::apply(
        crate::callbacks::ContentEvent::EdgarHealTick {
            actor_index: 0,
            generation: 1,
            end_time_us: 1,
            interval_us: 1,
            flat_heal: crate::simulation::ExactFraction::new(7, 2).unwrap(),
            max_hp_ratio: crate::simulation::ExactFraction::new(0, 1).unwrap(),
        },
        &mut crate::runtime_api::CallbackContext {
            state: &mut state,
            request: &req,
            queue: &mut queue,
            sequence: &mut sequence,
            trace: &mut trace,
        },
    )
    .unwrap();
    assert_eq!(
        trace[0].data.as_ref().and_then(|d| d.gross.zip(d.applied)),
        Some((6, 6))
    );
    assert_eq!(state.actors[1].hp, 16);
    assert_eq!(state.actors[2].hp, 10);
    assert!(queue.is_empty());
}

#[test]
fn edgar_healing_samples_off_field_source_current_recipient_expiry_and_source_life() {
    let mut req = request();
    for (slot, character_id) in [
        (1, "character:1019"),
        (2, "character:1042"),
        (3, "character:1046"),
    ] {
        let mut ally = req.actors[0].clone();
        ally.slot = slot;
        ally.character_id = character_id.into();
        ally.initial_hp = 100;
        req.actors.push(ally);
    }
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    state.active_actor_index = 1;
    state.actors[0].hp = 100;
    state.actors[0].hp_max = 100;
    state.actors[1].hp = 100;
    state.actors[2].hp = 100;
    state.actors[3].hp = 100;
    if let crate::characters::RuntimeState::Edgar(edgar) = &mut state.actors[0].character {
        edgar.healing_generation = 1;
    }
    let add_buff = |state: &mut crate::simulation::state::GameSimulatorState,
                    owner_actor: usize,
                    id: &str,
                    recipient,
                    stat: &str,
                    duration_us: u64,
                    amount: crate::simulation::ExactFraction,
                    source_lifetime| {
        state
            .buffs
            .apply_buff(
                ntesim_core::buffs::dispatch::BuffDefinition {
                    stacking_scope: ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                    semantic_tags: Default::default(),
                    source_lifetime,
                    id: id.into(),
                    source: id.into(),
                    recipient,
                    duration_us: Some(duration_us),
                    refresh_policy: ntesim_core::buffs::dispatch::RefreshPolicy::Disallow,
                    max_stacks: 1,
                    decay_policy: ntesim_core::buffs::dispatch::DecayPolicy::AllAtOnce,
                    stat_modifiers: std::collections::BTreeMap::from([(
                        stat.into(),
                        ntesim_core::kernel::formula::Ratio {
                            numerator: u64::try_from(amount.numerator).unwrap(),
                            denominator: u64::try_from(amount.denominator).unwrap(),
                        },
                    )]),
                },
                owner_actor,
                0,
            )
            .unwrap();
    };
    add_buff(
        &mut state,
        0,
        "edgar.outgoing",
        ntesim_core::buffs::dispatch::BuffRecipient::SpecificActor(0),
        "healing_bonus",
        20,
        crate::simulation::ExactFraction::new(1, 5).unwrap(),
        ntesim_core::buffs::dispatch::BuffSourceLifetime::WhileAlive,
    );
    add_buff(
        &mut state,
        3,
        "ally.incoming.long",
        ntesim_core::buffs::dispatch::BuffRecipient::ActiveActor,
        "incoming_healing_bonus",
        20,
        crate::simulation::ExactFraction::new(1, 5).unwrap(),
        ntesim_core::buffs::dispatch::BuffSourceLifetime::WhileAlive,
    );
    add_buff(
        &mut state,
        3,
        "ally.incoming.short",
        ntesim_core::buffs::dispatch::BuffRecipient::ActiveActor,
        "incoming_healing_bonus",
        8,
        crate::simulation::ExactFraction::new(1, 10).unwrap(),
        ntesim_core::buffs::dispatch::BuffSourceLifetime::WhileAlive,
    );
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    let mut tick = |state: &mut crate::simulation::state::GameSimulatorState,
                    time_us: u64,
                    active_actor_index: usize| {
        state.advance_unpaused_test_clock(time_us);
        state.active_actor_index = active_actor_index;
        crate::callbacks::apply(
            crate::callbacks::ContentEvent::EdgarHealTick {
                actor_index: 0,
                generation: 1,
                end_time_us: time_us,
                interval_us: 1,
                flat_heal: crate::simulation::ExactFraction::new(100, 1).unwrap(),
                max_hp_ratio: crate::simulation::ExactFraction::new(0, 1).unwrap(),
            },
            &mut crate::runtime_api::CallbackContext {
                state,
                request: &req,
                queue: &mut queue,
                sequence: &mut sequence,
                trace: &mut trace,
            },
        )
        .unwrap();
        trace.last().unwrap().data.clone().unwrap()
    };
    let first = tick(&mut state, 1, 1);
    assert_eq!(state.actors[1].hp, 256);
    assert_eq!((first.gross, first.applied), (Some(156), Some(156)));
    let second = tick(&mut state, 8, 2);
    assert_eq!(state.actors[2].hp, 244);
    assert_eq!((second.gross, second.applied), (Some(144), Some(144)));
    state.actors[3].hp = 0;
    let third = tick(&mut state, 9, 1);
    assert_eq!(state.actors[1].hp, 376);
    assert_eq!((third.gross, third.applied), (Some(120), Some(120)));
}

#[test]
fn healing_modifier_hypothesis_is_known_and_exact_text_is_required() {
    let mut req = request();
    crate::simulation::validate_model_assumptions(&req).unwrap();
    let mut reached = std::collections::BTreeMap::new();
    crate::runtime_api::record_hypothesis(
        &req,
        &mut reached,
        crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID,
        crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID.description(),
    )
    .unwrap();
    assert_eq!(
        reached[crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID.as_str()],
        crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID.description()
    );
    req.model_assumptions.insert(
        crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID.into(),
        "different formula".into(),
    );
    let error = crate::runtime_api::record_hypothesis(
        &req,
        &mut reached,
        crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID,
        crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID.description(),
    )
    .unwrap_err();
    assert!(error.contains("conflicting reached model hypothesis"));
}
