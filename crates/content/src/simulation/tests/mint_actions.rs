use super::*;
use crate::simulation::tests::support::formula_damage_total;
use serde_json::json;

fn make_action(
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

pub(super) fn request() -> GameScenarioRequest {
    let mut req = sample_s1_request();
    req.actors[0].character_id = "character:1019".into();
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
    req.targets[0].resistance = Ratio::zero();
    req.targets[0].level = 80;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Mint019_Melee",
        "GE_Player_Mint_Melee1_Damage_New",
        0,
        0,
    )];
    req
}

#[test]
fn mint_normal_chain_damage_multiplicity_loop_and_resonance() {
    for (level, awakening, coefficients, total) in [
        (
            1,
            0,
            [104, 231, 170, 262, 341, 184, 255, 523, 104, 231],
            2405,
        ),
        (
            8,
            0,
            [178, 396, 291, 449, 584, 315, 437, 896, 178, 396],
            2060,
        ),
        (
            8,
            3,
            [192, 428, 315, 485, 631, 341, 472, 968, 192, 428],
            2226,
        ),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([level, 1, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = (0..6)
            .map(|index| {
                make_action(
                    index * 300_000,
                    0,
                    "GA_Mint019_Melee",
                    "GE_Player_Mint_Melee1_Damage_New",
                    0,
                    0,
                )
            })
            .collect();
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        assert_eq!(report["hits_settled"], 10);
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
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            json!(format!("{}/{}", total, if level == 1 { 2 } else { 1 }))
        );
        let stages = [1, 2, 3, 4, 5, 6, 7, 8, 1, 2];
        for (hit, stage) in report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .zip(stages)
        {
            assert_eq!(
                hit["damage_effect_id"],
                format!("GE_Player_Mint_Melee{stage}_Damage_New")
            );
        }
    }
}

#[test]
fn mint_normal_chain_reset_and_noop_boundaries() {
    for boundary in 0..5 {
        let mut req = request();
        req.relative_duration_us = 4_000_000;
        let timestamp = match boundary {
            0 => 2_000_001,
            1 => 2_000_000,
            _ => 3000,
        };
        match boundary {
            2 => req.actions.push(make_action(
                1000,
                0,
                "GA_Mint019_Skill",
                "GE_Player_Mint_Skill1_Damage_New",
                0,
                0,
            )),
            3 => {
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
            4 => req.actions.push(GameActionCommand::SwitchActor {
                timestamp_us: 1000,
                target_actor_index: 0,
                cooldown_us: 0,
            }),
            _ => {}
        }
        req.actions.push(make_action(
            timestamp,
            0,
            "GA_Mint019_Melee",
            "GE_Player_Mint_Melee1_Damage_New",
            0,
            0,
        ));
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        let expected = if boundary == 1 || boundary == 4 {
            "GE_Player_Mint_Melee3_Damage_New"
        } else {
            "GE_Player_Mint_Melee2_Damage_New"
        };
        let last_melee = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .rfind(|hit| {
                hit["damage_effect_id"]
                    .as_str()
                    .is_some_and(|packet| packet.starts_with("GE_Player_Mint_Melee"))
            })
            .unwrap();
        assert_eq!(last_melee["damage_effect_id"], expected);
    }
}

#[test]
fn mint_normal_chain_strict_hypothesis_and_off_field_two_hit_delivery() {
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
        crate::characters::mint::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::mint::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::simulation::energy::MINT_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::MINT_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actors.push(req.actors[0].clone());
    req.actions = (0..5)
        .map(|index| {
            make_action(
                index * 1000,
                0,
                "GA_Mint019_Melee",
                "GE_Player_Mint_Melee1_Damage_New",
                0,
                0,
            )
        })
        .collect();
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 4050,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    req.relative_duration_us = 4099;
    let pending = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(pending["hits_settled"], 6);
    req.relative_duration_us = 4100;
    let delivered = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(delivered["hits_settled"], 8);
    for hit in &delivered["hit_receipts"].as_array().unwrap()[6..] {
        assert_eq!(hit["actor_index"], 0);
        assert!(matches!(
            hit["damage_effect_id"].as_str(),
            Some("GE_Player_Mint_Melee7_Damage_New" | "GE_Player_Mint_Melee8_Damage_New")
        ));
    }
}

#[test]
fn mint_normal_skill_and_burst_use_native_talent_coefficients_and_resonance_bonus() {
    // Test Normal Attack (GA_Mint019_Melee / GE_Player_Mint_Melee1_Damage_New)
    // Coeffs: Lv1 = 0.104, Lv8 = 0.178, Lv9 (A3 resonance) = 0.192
    // At 1000 ATK, 0 RES, 0.5 DEF:
    // Lv1: 1000 * 0.104 * 0.5 = 52 / 1
    // Lv8: 1000 * 0.178 * 0.5 = 89 / 1
    // Lv9: 1000 * 0.192 * 0.5 = 96 / 1
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 52, 1),
        (8, 0, 89, 1),
        (8, 1, 89, 1),
        (8, 3, 96, 1),
        (8, 4, 96, 1),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([level, 1, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        assert_eq!(report["hits_settled"], 2);
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            json!(format!("{}/{}", expected_num, expected_den))
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"
        );
    }

    // Test Skill (GA_Mint019_Skill / GE_Player_Mint_Skill1_Damage_New)
    // Coeffs: Lv1 = 0.9, Lv8 = 1.542, Lv9 (A3 resonance) = 1.666
    // At 1000 ATK, 0 RES, 0.5 DEF:
    // Lv1, A0: 1000 * 0.9 * 0.5 = 450 / 1
    // Lv8, A0: 1000 * 1.542 * 0.5 = 771 / 1
    // Lv8, A1: 1000 * 1.542 * 0.5 = 771 / 1
    // Lv8, A3: resonance_3 (+1 lvl = 9, rate 1.666 = 833/500) and A2 (+40% dmg bonus):
    //          1000 * 1.666 * 0.5 * 1.4 = 5831 / 5
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 450, 1),
        (8, 0, 771, 1),
        (8, 1, 771, 1),
        (8, 3, 5831, 5),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, level, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Mint019_Skill",
            "GE_Player_Mint_Skill1_Damage_New",
            0,
            6_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        assert_eq!(report["hits_settled"], 2);
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            json!(format!("{}/{}", expected_num, expected_den))
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
        );
    }

    // Test Ultimate/Burst (GA_Mint019_UltraSkill / GE_Player_Mint_UtraSkill1_Damage_New)
    // Coeffs: Lv1 = 1.326, Lv8 = 2.273, Lv9 (A3 resonance) = 2.454
    // At 1000 ATK, 0 RES, 0.5 DEF:
    // Lv1: 1000 * 1.326 * 0.5 = 663 / 1
    // Lv8: 1000 * 2.273 * 0.5 = 2273 / 2
    // Lv8, A1: 1000 * 2.273 * 0.5 = 2273 / 2
    // Lv9 (A3 resonance): 1000 * 2.454 * 0.5 = 1227 / 1
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 663, 1),
        (8, 0, 2273, 2),
        (8, 1, 2273, 2),
        (8, 3, 1227, 1),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, 1, level]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Mint019_UltraSkill",
            "GE_Player_Mint_UtraSkill1_Damage_New",
            100,
            20_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        assert_eq!(report["hits_settled"], 7);
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
fn mint_skill_and_burst_packet_totals_follow_independent_source_ratios() {
    // Frozen description ratios, independent of the runtime's child list.
    // Burst at levels 1/8/9: 1.326+5*.588+3.735; 2.273+5*1.008+6.401;
    // 2.454+5*1.088+6.913. ATK1000, DEF .5, RES0.
    for (level, awakening, claw, slash, total_num, total_den) in [
        (1, 0, 294, 3735, 8001, 2),
        (8, 0, 504, 6401, 6857, 1),
        (8, 3, 544, 6913, 14807, 2),
    ] {
        let mut req = request();
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actors[0].talent_levels = Some([1, 1, level]);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Mint019_UltraSkill",
            "GE_Player_Mint_UtraSkill1_Damage_New",
            100,
            15_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        let hits = report["hit_receipts"].as_array().unwrap();
        assert_eq!(hits.len(), 7);
        for (index, hit) in hits.iter().enumerate() {
            assert_eq!(hit["time_us"], 100);
            assert_eq!(hit["command_index"], 0);
            assert_eq!(
                hit["damage_effect_id"],
                format!("GE_Player_Mint_UtraSkill{}_Damage_New", index + 1)
            );
            assert_eq!(
                super::support::hit_metadata(&report, hit)["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
            );
        }
        for hit in &hits[1..6] {
            assert_eq!(hit["non_crit_damage"], json!(format!("{}/{}", claw, 1)));
        }
        assert_eq!(
            hits[6]["non_crit_damage"],
            json!(format!("{}/{}", slash, 2))
        );
        let total = hits
            .iter()
            .fold(ExactFraction::new(0, 1).unwrap(), |sum, hit| {
                sum.add(serde_json::from_value(hit["non_crit_damage"].clone()).unwrap())
                    .unwrap()
            });
        assert_eq!(total, ExactFraction::new(total_num, total_den).unwrap());
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
        assert_eq!(report["final_actor_states"][0]["energy"], 0);
    }
    for (level, awakening, damage) in [
        (1, 0, json!("450/1")),
        (8, 0, json!("771/1")),
        (8, 3, json!("5831/5")),
    ] {
        let mut req = request();
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actors[0].talent_levels = Some([1, level, 1]);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Mint019_Skill",
            "GE_Player_Mint_Skill1_Damage_New",
            0,
            6_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["hits_settled"], 2);
        for hit in report["hit_receipts"].as_array().unwrap() {
            assert_eq!(hit["non_crit_damage"], damage);
            assert_eq!(hit["damage_effect_id"], "GE_Player_Mint_Skill1_Damage_New");
        }
    }
}

#[test]
fn mint_packet_trains_preserve_pending_ownership_and_atomic_admission() {
    for (ability, primary, count, last_hit_us) in [
        (
            "GA_Mint019_Skill",
            "GE_Player_Mint_Skill1_Damage_New",
            2,
            365_346,
        ),
        (
            "GA_Mint019_UltraSkill",
            "GE_Player_Mint_UtraSkill1_Damage_New",
            7,
            100,
        ),
    ] {
        let mut req = request();
        let mut ally = req.actors[0].clone();
        ally.slot = 1;
        ally.character_id = "character:1052".into();
        req.actors.push(ally);
        req.actions = vec![
            make_action(0, 0, ability, primary, 0, 0),
            GameActionCommand::SwitchActor {
                timestamp_us: 50,
                target_actor_index: 1,
                cooldown_us: 0,
            },
        ];
        req.relative_duration_us = if ability == "GA_Mint019_UltraSkill" {
            49
        } else {
            99
        };
        let pending = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(pending["pending_events"], count);
        assert_eq!(pending["execution_completed"], true);
        req.relative_duration_us = 100;
        let delivered = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        if ability == "GA_Mint019_Skill" {
            assert_eq!(delivered["hits_settled"], 1);
            assert_eq!(delivered["pending_events"], 1);
        }
        req.relative_duration_us = last_hit_us;
        let delivered = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(delivered["hits_settled"], count);
        assert!(delivered["pending_events"].as_u64().unwrap() == 0);
        assert!(
            delivered["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .all(|hit| hit["actor_index"] == 0)
        );
        req.targets[0].hp_current = 1;
        let lethal = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(lethal["hits_settled"], 1);
        req.targets[0].hp_current = 0;
        assert_eq!(
            run_game_scenario(&req, &ExecutionLimits::default()).unwrap()["hits_settled"],
            0
        );
        if let GameActionCommand::InvokeAbility { energy_cost, .. } = &mut req.actions[0] {
            *energy_cost = 101;
        }
        let rejected = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(rejected["execution_completed"], false);
        assert_eq!(rejected["hits_settled"], 0);
        assert_eq!(rejected["pending_events"], 0);
    }
}

#[test]
fn mint_packet_hypothesis_and_alternate_primary_are_guarded() {
    let mut req = request();
    req.actions = vec![make_action(
        0,
        0,
        "GA_Mint019_UltraSkill",
        "GE_Player_Mint_UtraSkill1_Damage_New",
        0,
        0,
    )];
    req.actors[0].talent_levels = None;
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::mint::skill::PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::mint::skill::PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    assert_eq!(
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap()["hits_settled"],
        7
    );
    if let GameActionCommand::InvokeAbility {
        damage_effect_id, ..
    } = &mut req.actions[0]
    {
        *damage_effect_id = "GE_Player_Mint_UtraSkill7_Damage_New".into();
    }
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("modeled first packet")
    );
}

#[test]
fn mint_second_skill_hit_observes_live_hp_and_cast_energy_is_granted_once() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actors[0].initial_energy = 0;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 100,
        denominator: 1,
    };
    req.targets[0].hp_max = 1000;
    req.targets[0].hp_current = 405;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Mint019_Skill",
        "GE_Player_Mint_Skill1_Damage_New",
        0,
        0,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    // Effective L2=.972. First hit above40%:100*.972*.5*(1+.4)=68.04.
    // Rounded HP loss68 crosses the A6 boundary. Second adds.15 to the bonus zone.
    assert_eq!(
        report["hit_receipts"][0]["non_crit_damage"],
        json!("1701/25")
    );
    assert_eq!(
        report["hit_receipts"][1]["non_crit_damage"],
        json!("7533/100")
    );
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 6);
    assert_eq!(report["final_actor_states"][0]["energy"], 6);
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "mint_a6_energy")
            .count(),
        1
    );
}

#[test]
fn mint_a6_low_hp_damage_bonus() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actors[0].talent_levels = Some([1, 1, 1]);
    req.targets[0].hp_max = 1000;
    req.targets[0].hp_current = 400; // At 40% HP, A6 grants +15% DMG.

    let low_hp = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(low_hp["hits_settled"], 2);
    // At A6, resonance_3 increases Lv1 Melee to Lv2 (coeff 0.112).
    // With A6 +15% DMG: 1000 * 0.112 * 0.5 * 1.15 = 322 / 5.
    assert_eq!(low_hp["hit_receipts"][0]["non_crit_damage"], json!("322/5"));

    let mut req_high = req.clone();
    req_high.targets[0].hp_current = 401;
    let high_hp = run_game_scenario(&req_high, &ExecutionLimits::default()).unwrap();
    assert_eq!(high_hp["hits_settled"], 2);
    assert_eq!(
        high_hp["hit_receipts"][1]["non_crit_damage"],
        json!("5727/40")
    );
    assert_eq!(high_hp["hit_receipts"][0]["non_crit_damage"], json!("56/1"));
}

#[test]
fn mint_off_field_delivery_and_dead_target_boundary() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
    req.actors[0].talent_levels = Some([1, 1, 1]);

    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1046".into();
    req.actors.push(ally);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Mint019_Melee",
            "GE_Player_Mint_Melee1_Damage_New",
            0,
            0,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 50,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    req.relative_duration_us = 200;

    let off_field = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(off_field["hits_settled"], 2);
    assert_eq!(off_field["hit_receipts"][0]["actor_index"], 0);
    assert_eq!(
        off_field["hit_receipts"][0]["non_crit_damage"],
        json!("52/1")
    );

    let mut req_dead = request();
    req_dead.targets[0].hp_current = 0;
    let dead = run_game_scenario(&req_dead, &ExecutionLimits::default()).unwrap();
    assert_eq!(dead["hits_settled"], 0);

    let mut req_pending = request();
    req_pending.relative_duration_us = 99;
    let pending = run_game_scenario(&req_pending, &ExecutionLimits::default()).unwrap();
    assert_eq!(pending["execution_completed"], true);
    assert_eq!(pending["hits_settled"], 0);
    assert_eq!(pending["pending_events"], 2);
}

#[test]
fn mint_skill_and_burst_all_in_range_dummy_multi_target() {
    let mut req = request();
    req.actors[0].talent_levels = Some([1, 1, 1]);
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);

    // Skill across 2 targets: 2 hits on target 0 + 2 hits on target 1 = 4 hits.
    req.actions = vec![make_action(
        0,
        0,
        "GA_Mint019_Skill",
        "GE_Player_Mint_Skill1_Damage_New",
        0,
        6_000_000,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    assert_eq!(report["hits_settled"], 4);
    assert_eq!(
        report["resolved_model_assumptions"]
            [crate::characters::mint::skill::ALL_IN_RANGE_HYPOTHESIS_ID.as_str()],
        crate::characters::mint::skill::ALL_IN_RANGE_HYPOTHESIS_ID.description()
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits[0]["target_index"], 0);
    assert_eq!(hits[1]["target_index"], 1);
    assert_eq!(hits[2]["target_index"], 0);
    assert_eq!(hits[3]["target_index"], 1);
    assert_eq!(hits[0]["time_us"], 100);
    assert_eq!(hits[1]["time_us"], 100);
    assert_eq!(hits[2]["time_us"], 365_346);
    assert_eq!(hits[3]["time_us"], 365_346);

    // Burst across 2 targets: 7 hits on target 0 + 7 hits on target 1 = 14 hits.
    req.actions = vec![make_action(
        0,
        0,
        "GA_Mint019_UltraSkill",
        "GE_Player_Mint_UtraSkill1_Damage_New",
        100,
        15_000_000,
    )];
    let report_burst = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_burst["execution_completed"], true);
    assert_eq!(report_burst["hits_settled"], 14);
    let burst_hits = report_burst["hit_receipts"].as_array().unwrap();
    assert_eq!(
        burst_hits.iter().filter(|h| h["target_index"] == 0).count(),
        7
    );
    assert_eq!(
        burst_hits.iter().filter(|h| h["target_index"] == 1).count(),
        7
    );
}

#[test]
fn mint_a4_crit_rate_single_vs_multi_target() {
    // A4 Lunch Break: +12% crit rate when target_count <= 1
    let mut req_single = request();
    req_single.actors[0].static_stats.crit_rate = Ratio::zero();
    req_single.actors[0].awakening = ntesim_data::AwakeningSelection::selected([4]).unwrap();
    req_single.actions = vec![make_action(
        0,
        0,
        "GA_Mint019_Skill",
        "GE_Player_Mint_Skill1_Damage_New",
        0,
        6_000_000,
    )];
    let report_single = run_game_scenario(&req_single, &ExecutionLimits::default()).unwrap();
    let hits_single = report_single["hit_receipts"].as_array().unwrap();
    // At static 0 crit rate, with A4 on single target: proposed_cr = 12/100 = 3/25
    assert_eq!(hits_single[0]["crit_rate"], json!("3/25"));

    let mut req_multi = req_single.clone();
    let mut target2 = req_multi.targets[0].clone();
    target2.target_index = 1;
    req_multi.targets.push(target2);
    let report_multi = run_game_scenario(&req_multi, &ExecutionLimits::default()).unwrap();
    let hits_multi = report_multi["hit_receipts"].as_array().unwrap();
    // With 2 targets, A4 does NOT activate: proposed_cr = 0
    assert_eq!(hits_multi[0]["crit_rate"], json!("0/1"));
}

#[test]
fn mint_a3_attendance_bonus_buff_application_and_damage() {
    use crate::characters::mint::awakenings::{
        A3_REACTION_TRIGGER_HYPOTHESIS_ID, apply_a3_attendance_bonus,
    };
    assert_eq!(
        A3_REACTION_TRIGGER_HYPOTHESIS_ID,
        "mint.a3_reaction_trigger_v1"
    );
    assert!(
        crate::characters::mint::awakenings::A3_REACTION_TRIGGER_HYPOTHESIS
            .contains("Attendance Bonus")
    );

    let mut buffs = ntesim_core::buffs::dispatch::BuffManager::default();
    apply_a3_attendance_bonus(&mut buffs, 0, 0).unwrap();
    let living = std::collections::BTreeSet::from([0]);
    let mods = buffs
        .get_stat_modifiers_for_actor_with_living_sources(0, 0, 100, &living)
        .unwrap();
    assert_eq!(
        mods.get("atk_percent").copied(),
        Some(ntesim_core::formula::Ratio {
            numerator: 3,
            denominator: 20
        })
    );
    for (now_us, active) in [(14_999_999, true), (15_000_000, false)] {
        let mods = buffs
            .get_stat_modifiers_for_actor_with_living_sources(0, 0, now_us, &living)
            .unwrap();
        assert_eq!(mods.contains_key("atk_percent"), active);
    }
}

#[test]
fn mint_a5_two_casts_preserve_exact_expiry_and_replay() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([5]).unwrap();
    req.actors[0].initial_energy = 0;
    req.actors[0].static_stats.crit_damage_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };
    req.relative_duration_us = 16_000_001;
    req.actions.clear();
    for start in [0, 10_000_000] {
        req.actions.push(make_action(
            start,
            0,
            "GA_Mint019_Skill",
            "GE_Player_Mint_Skill1_Damage_New",
            0,
            6_000_000,
        ));
        // Damage is evaluated at delivery: the first Normal lands 1us before
        // expiry, the next exactly at expiry. The second cast uses live state.
        for offset in [5_999_899, 5_999_900] {
            req.actions.push(make_action(
                start + offset,
                0,
                "GA_Mint019_Melee",
                "GE_Player_Mint_Melee1_Damage_New",
                0,
                0,
            ));
        }
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    assert_eq!(report["hits_settled"], 10);
    assert_eq!(report["final_actor_states"][0]["energy_conserved"], true);
    for hit in report["hit_receipts"].as_array().unwrap() {
        let command = hit["command_index"].as_u64().unwrap();
        let multiplier = if command == 2 || command == 5 {
            ExactFraction::new(3, 2).unwrap()
        } else {
            ExactFraction::new(7, 4).unwrap()
        };
        let non_crit: ExactFraction =
            serde_json::from_value(hit["non_crit_damage"].clone()).unwrap();
        let crit: ExactFraction = serde_json::from_value(hit["crit_damage"].clone()).unwrap();
        assert_eq!(crit, non_crit.mul(multiplier).unwrap());
    }
    assert_eq!(
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap(),
        report
    );
}

fn packets<'a>(report: &'a serde_json::Value, effect: &str) -> Vec<&'a serde_json::Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == effect)
        .collect()
}

#[test]
fn mint_riposte_from_critical_dodge_consumed_once_and_grants_energy() {
    let source = r#"
options iteration=1 duration=8 crit=fixed stamina_capacity=60 initial_stamina=60 stamina_use_efficiency=1 stamina_recovery_rate=0;
mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
target hp=100000000 def=0 resist=0;
active mint;

mint dash[critical=true]; wait(0.4); mint attack;
wait(0.8); mint attack;
"#;
    let req = crate::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );

    let hits = packets(&report, "GE_Player_Mint_PerfectEvadeAttack_Damage");
    assert_eq!(hits.len(), 1);
    let riposte_hit = hits[0];
    let riposte_meta = super::support::hit_metadata(&report, riposte_hit);
    assert_eq!(riposte_meta["action"]["input_role"], "critical_riposte");
    assert_eq!(
        riposte_meta["packet"]["native_ga_name"],
        "GA_Mint_ExtremEvadeAtk"
    );

    assert_eq!(
        packets(&report, "GE_Player_Mint_Melee1_Damage_New").len(),
        1
    );
    assert_eq!(
        packets(&report, "GE_Player_Mint_Melee2_Damage_New").len(),
        1
    );

    let riposte_grants: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "mint_riposte_energy_grant")
        .collect();
    assert_eq!(riposte_grants.len(), 1);
    assert_eq!(
        riposte_grants[0]["energy_change"]["hit_basis"]["base"],
        json!("22/5")
    );
}

#[test]
fn mint_a1_critical_riposte_resets_skill_cooldown_with_guard() {
    let source = r#"
options iteration=1 duration=8 crit=fixed stamina_capacity=60 initial_stamina=60 stamina_use_efficiency=1 stamina_recovery_rate=0;
mint char lvl=80/80 awakening=1 talent=1,1,1 qte=1 start_energy=0;
target hp=100000000 def=0 resist=0;
active mint;

mint skill;
wait(1.0);
mint dash[critical=true]; wait(0.4); mint attack;
"#;
    let req = crate::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );

    let resets: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "mint_a1_skill_cooldown_reset")
        .collect();
    assert_eq!(resets.len(), 1);
}

#[test]
fn mint_plunge_damage_and_charge() {
    let mut req = request();
    req.actions = vec![make_action(
        0,
        0,
        "GA_Mint019_Melee",
        "GE_Player_Mint_AirAttack1_Damage",
        0,
        0,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    let hits = packets(&report, "GE_Player_Mint_AirAttack1_Damage");
    assert_eq!(hits.len(), 1);
    assert_eq!(
        super::support::hit_metadata(&report, hits[0])["action"]["primary_slot"],
        "normal"
    );
}

#[test]
fn mint_hold_opener_delivers_four_hits() {
    let mut req = request();
    req.actions = vec![make_action(
        0,
        0,
        "GA_Mint019_Melee",
        "GE_Player_Mint_Evade1_Damage",
        0,
        0,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    assert_eq!(packets(&report, "GE_Player_Mint_Evade1_Damage").len(), 3);
    assert_eq!(packets(&report, "GE_Player_Mint_Evade4_Damage").len(), 1);
}

#[test]
fn mint_passive2_entry_def_buff_lifecycle() {
    let source = r#"
options iteration=1 duration=8 crit=fixed stamina_capacity=60 initial_stamina=60 stamina_use_efficiency=1 stamina_recovery_rate=0;
mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
nanally char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
target hp=100000000 def=0 resist=0;
active mint;

wait(1.0);
nanally swap;
wait(2.0);
mint swap;
"#;
    let mut req = crate::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    req.model_assumptions.insert(
        crate::characters::mint::skill::PASSIVE2_HYPOTHESIS_ID.into(),
        crate::characters::mint::skill::PASSIVE2_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );

    let entries: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "mint_passive2_entry")
        .collect();
    assert_eq!(entries.len(), 2);
}
