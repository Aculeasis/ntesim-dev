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
    req.actors[0].character_id = "character:1003".into();
    req.actors[0].weapon_id.clear();
    req.actors[0].gear_procs.clear();
    req.actors[0].set_id = None;
    req.actors[0].talent_levels = Some([1; 3]);
    req.actors[0].character_max_level = Some(80);
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.targets[0].resistance = Ratio::zero();
    req.actions = vec![make_action(
        0,
        0,
        "GA_Sagiri_Melee",
        "GE_Player_Sagiri_Melee1_Damage",
        0,
        0,
    )];
    req
}

#[test]
fn sakiri_typed_defeat_and_debuff_parameters_preserve_caps_and_expiry() {
    use crate::characters::sakiri::awakenings::{damage_bonus, on_team_defeat};
    use crate::runtime_api::{DamageContext, HitModifiers};
    use crate::simulation::state::GameSimulatorState;

    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    let mut state = GameSimulatorState::new(&req);
    for _ in 0..11 {
        on_team_defeat(&req.actors[0], &mut state.actors[0], 0).unwrap();
    }
    let score = |state: &GameSimulatorState, time_us, debuff_count| {
        let action_metadata = super::support::declared_test_action(
            "GA_Sagiri_Skill",
            crate::simulation::metadata::ActionCategory::Skill,
        );
        let ctx = DamageContext {
            foreign_dot_stacks: 0,
            packet_group_id: None,
            command_index: 0,
            config: &req.actors[0],
            start_fons: None,
            ability_id: "GA_Sagiri_Skill",
            action_metadata: &action_metadata,
            target: &state.targets[0],
            actor: &state.actors[0],
            world_time_us: time_us,
            time_us,
            debuff_count,
            damage_effect_id: "GE_Player_Sagiri_Skill1_Damage",
            actor_index: 0,
            active_actor_index: 0,
            target_count: 1,
            non_boss_target_count: 1,
            target_index: 0,
            buffs: &state.buffs,
            buff_stats: Default::default(),
            owned_status_counts: &Default::default(),
            team_has_three_elements: false,
        };
        damage_bonus(
            &ctx,
            HitModifiers::new(ExactFraction::new(1, 5).unwrap()).unwrap(),
        )
        .unwrap()
        .damage_bonus
    };
    // Twenty percent existing DMG plus ten capped six-percent defeat stacks;
    // debuffs add three points each, up to twelve, in the same additive zone.
    for (debuffs, numerator) in [(0, 80), (1, 83), (4, 92), (5, 92)] {
        assert_eq!(
            score(&state, 14_999_999, debuffs),
            ExactFraction::new(numerator, 100).unwrap()
        );
    }
    assert_eq!(
        score(&state, 15_000_000, 5),
        ExactFraction::new(8, 25).unwrap()
    );
    // The next cycle removes expired stacks and starts one new 15s lifetime.
    on_team_defeat(&req.actors[0], &mut state.actors[0], 15_000_000).unwrap();
    assert_eq!(
        score(&state, 29_999_999, 0),
        ExactFraction::new(13, 50).unwrap()
    );
    assert_eq!(
        score(&state, 30_000_000, 0),
        ExactFraction::new(1, 5).unwrap()
    );
}

#[test]
fn sakiri_normal_chain_native_packet_order_damage_loop_and_resonance() {
    for (level, awakening, coefficients, total_numerator) in [
        (
            1,
            0,
            [403, 138, 757, 16, 716, 331, 331, 331, 35, 1062, 403],
            4523,
        ),
        (
            8,
            0,
            [691, 237, 1297, 27, 1227, 567, 567, 567, 60, 1820, 691],
            7751,
        ),
        (
            8,
            3,
            [746, 255, 1401, 30, 1325, 613, 613, 613, 65, 1966, 746],
            8373,
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
                    "GA_Sagiri_Melee",
                    "GE_Player_Sagiri_Melee1_Damage",
                    0,
                    0,
                )
            })
            .collect();
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 11);
        let expected_packets = [1, 2, 3, 8, 4, 5, 5, 5, 6, 7, 1];
        for ((hit, coefficient), packet) in report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .zip(coefficients)
            .zip(expected_packets)
        {
            let divisor = if coefficient % 2 == 0 { 1 } else { 2 };
            assert_eq!(
                hit["non_crit_damage"],
                json!(format!("{}/{}", coefficient / (2 / divisor), divisor))
            );
            assert_eq!(
                hit["damage_effect_id"],
                format!("GE_Player_Sagiri_Melee{packet}_Damage")
            );
        }
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            json!(format!("{}/{}", total_numerator, 2))
        );
    }
}

#[test]
fn sakiri_normal_chain_reset_boundaries_and_noop_switch() {
    for boundary in 0..5 {
        let mut req = request();
        req.relative_duration_us = 4_000_000;
        match boundary {
            2 => req.actions.push(make_action(
                1000,
                0,
                "GA_Sagiri_Skill",
                "GE_Player_Sagiri_Skill1_Damage",
                0,
                0,
            )),
            3 => {
                let mut ally = req.actors[0].clone();
                ally.character_id = "character:1019".into();
                req.actors.push(ally);
                req.actions.push(GameActionCommand::SwitchActor {
                    timestamp_us: 1000,
                    target_actor_index: 1,
                    cooldown_us: 0,
                });
                req.actions.push(GameActionCommand::SwitchActor {
                    timestamp_us: 700_000,
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
        let timestamp = match boundary {
            0 => 2_000_001,
            1 => 2_000_000,
            3 => 1_300_000,
            _ => 3000,
        };
        req.actions.push(make_action(
            timestamp,
            0,
            "GA_Sagiri_Melee",
            "GE_Player_Sagiri_Melee1_Damage",
            0,
            0,
        ));
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let expected = if boundary == 1 || boundary == 4 {
            "GE_Player_Sagiri_Melee3_Damage"
        } else {
            "GE_Player_Sagiri_Melee1_Damage"
        };
        assert_eq!(
            report["hit_receipts"].as_array().unwrap().last().unwrap()["damage_effect_id"],
            expected
        );
    }
}

#[test]
fn sakiri_normal_chain_strict_and_pending_final_stage() {
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
        crate::characters::sakiri::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::sakiri::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::simulation::energy::SAKIRI_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::SAKIRI_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actors.push(req.actors[0].clone());
    req.actions = (0..5)
        .map(|index| {
            make_action(
                index * 1000,
                0,
                "GA_Sagiri_Melee",
                "GE_Player_Sagiri_Melee1_Damage",
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
    assert_eq!(pending["hits_settled"], 8);
    req.relative_duration_us = 4100;
    let delivered = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(delivered["hits_settled"], 10);
    for hit in &delivered["hit_receipts"].as_array().unwrap()[8..] {
        assert_eq!(hit["actor_index"], 0);
    }
}

#[test]
fn sakiri_normal_skill_and_burst_use_native_talent_coefficients_and_resonance_bonus() {
    // Test Normal Attack (GA_Sagiri_Melee / GE_Player_Sagiri_Melee1_Damage)
    // Coeffs: Lv1 = 0.403, Lv8 = 0.691, Lv9 (A3 resonance) = 0.746
    // At 1000 ATK, 0 RES, 0.5 DEF:
    // Lv1: 1000 * 0.403 * 0.5 = 403 / 2
    // Lv8: 1000 * 0.691 * 0.5 = 691 / 2
    // Lv9: 1000 * 0.746 * 0.5 = 373 / 1
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 403, 2),
        (8, 0, 691, 2),
        (8, 1, 691, 2),
        (8, 3, 373, 1),
        (8, 4, 373, 1),
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

    // Test Skill (GA_Sagiri_Skill) delivers 2 hits:
    // Hit 0: GE_Player_Sagiri_Skill1_Damage (Lv1: 1.678, Lv8: 2.876, Lv9: 3.106)
    // Hit 1: GE_Player_Sagiri_Skill2_Damage (Lv1: 2.322, Lv8: 3.979, Lv9: 4.298)
    // At 1000 ATK, 0 RES, 0.5 DEF:
    // Lv1: 839/1 and 1161/1
    // Lv8: 1438/1 and 3979/2
    // Lv9 (A3 resonance): 1553/1 and 2149/1
    for (level, awakening, exp1_num, exp1_den, exp2_num, exp2_den) in [
        (1, 0, 839, 1, 1161, 1),
        (8, 0, 1438, 1, 3979, 2),
        (8, 1, 1438, 1, 3979, 2),
        (8, 3, 1553, 1, 2149, 1),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, level, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Sagiri_Skill",
            "GE_Player_Sagiri_Skill1_Damage",
            0,
            6_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 2);
        assert_eq!(
            report["hit_receipts"][0]["damage_effect_id"],
            "GE_Player_Sagiri_Skill1_Damage"
        );
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            json!(format!("{}/{}", exp1_num, exp1_den))
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
        );
        assert_eq!(
            report["hit_receipts"][1]["damage_effect_id"],
            "GE_Player_Sagiri_Skill2_Damage"
        );
        assert_eq!(
            report["hit_receipts"][1]["non_crit_damage"],
            json!(format!("{}/{}", exp2_num, exp2_den))
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][1])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
        );
        assert!(
            report["resolved_model_assumptions"]
                .get("sakiri.skill_burst_packets_v1")
                .is_some()
        );
    }

    // Test Ultimate/Burst (GA_Sagiri_UltraSkill) delivers 6 hits:
    // Hits 0..4 (5 hits): GE_Player_Sagiri_UltraSkill1_Damage (Lv1: 1.2, Lv8: 2.057, Lv9: 2.221)
    // Hit 5 (1 hit): GE_Player_Sagiri_UltraSkill2_Damage (Lv1: 3.0, Lv8: 5.141, Lv9: 5.553)
    // At 1000 ATK, 0 RES, 0.5 DEF:
    // Lv1: 600/1 each (5x), 1500/1 (1x)
    // Lv8: 2057/2 each (5x), 5141/2 (1x)
    // Lv9 (A3 resonance): 2221/2 each (5x), 5553/2 (1x)
    for (level, awakening, exp1_num, exp1_den, exp2_num, exp2_den) in [
        (1, 0, 600, 1, 1500, 1),
        (8, 0, 2057, 2, 5141, 2),
        (8, 1, 2057, 2, 5141, 2),
        (8, 3, 2221, 2, 5553, 2),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, 1, level]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Sagiri_UltraSkill",
            "GE_Player_Sagiri_UltraSkill1_Damage",
            100,
            20_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 6);
        for i in 0..5 {
            assert_eq!(
                report["hit_receipts"][i]["damage_effect_id"],
                "GE_Player_Sagiri_UltraSkill1_Damage"
            );
            assert_eq!(
                report["hit_receipts"][i]["non_crit_damage"],
                json!(format!("{}/{}", exp1_num, exp1_den))
            );
            assert_eq!(
                super::support::hit_metadata(&report, &report["hit_receipts"][i])["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
            );
        }
        assert_eq!(
            report["hit_receipts"][5]["damage_effect_id"],
            "GE_Player_Sagiri_UltraSkill2_Damage"
        );
        assert_eq!(
            report["hit_receipts"][5]["non_crit_damage"],
            json!(format!("{}/{}", exp2_num, exp2_den))
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][5])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
        );
        assert!(
            report["resolved_model_assumptions"]
                .get("sakiri.skill_burst_packets_v1")
                .is_some()
        );
    }
}

#[test]
fn sakiri_a4_burst_buff_excludes_self_and_grants_team_atk_to_allies() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(4); // A4 Wishful Reliance: 30% of base ATK = 300 to allies
    req.actors[0].talent_levels = Some([1, 1, 1]);

    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.awakening = ntesim_data::AwakeningSelection::prefix(0);
    ally.talent_levels = Some([1, 1, 1]);
    req.actors.push(ally);

    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Sagiri_UltraSkill",
            "GE_Player_Sagiri_UltraSkill1_Damage",
            100,
            20_000_000,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 1_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        make_action(
            2_000_000,
            1,
            "GA_Mint019_Melee",
            "GE_Player_Mint_Melee1_Damage_New",
            0,
            0,
        ),
    ];
    req.relative_duration_us = 3_000_000;

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 8);

    // Effect4 alone keeps ordinary+300 and separate A4+300 ATK, but lacks resonance_3.
    // Lv1 Burst = 1.2 x5 + 3.0; prefix(4) uses Lv2 = 1.296 x5 + 3.24.
    let mut only_a4 = req.clone();
    only_a4.actors[0].awakening = ntesim_data::AwakeningSelection::selected([4]).unwrap();
    let selected_report = run_game_scenario(&only_a4, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        selected_report["hit_receipts"][0]["non_crit_damage"],
        json!("600/1")
    );
    assert_eq!(
        selected_report["hit_receipts"][5]["non_crit_damage"],
        json!("1500/1")
    );
    assert_eq!(
        selected_report["hit_receipts"][6]["non_crit_damage"],
        json!("416/5")
    );

    for i in 0..5 {
        assert_eq!(report["hit_receipts"][i]["non_crit_damage"], json!("648/1"));
    }
    assert_eq!(
        report["hit_receipts"][5]["non_crit_damage"],
        json!("1620/1")
    );

    // Hit 6: Ally Mint's Normal Attack at t=2.0001s.
    // Ally base ATK = 1000, but has+300 ordinary and+300 A4 ATK! Total ATK = 1600.
    // Mint Lv1 Melee coefficient = 0.104 (13/125).
    // Expected damage = 1600 * 0.104 * 0.5 = 83.2 -> exact: 1600 * (104/1000) * (1/2) = 416/5 = 83.2.
    assert_eq!(report["hit_receipts"][6]["non_crit_damage"], json!("416/5"));

    // Verify self-exclusion: Sakiri attacking after burst receives no self-buff.
    // Her Normal attack at A4 also benefits from resonance_3 (Lv1 + 1 = Lv2, coeff 0.435),
    // but remains at 1000 ATK (not 1300 ATK): 1000 * 0.435 * 0.5 = 435 / 2.
    let mut req_self = request();
    req_self.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(4);
    req_self.actors[0].talent_levels = Some([1, 1, 1]);
    req_self.actions = vec![
        make_action(
            0,
            0,
            "GA_Sagiri_UltraSkill",
            "GE_Player_Sagiri_UltraSkill1_Damage",
            100,
            20_000_000,
        ),
        make_action(
            1_000_000,
            0,
            "GA_Sagiri_Melee",
            "GE_Player_Sagiri_Melee1_Damage",
            0,
            0,
        ),
    ];
    req_self.relative_duration_us = 2_000_000;
    let report_self = run_game_scenario(&req_self, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_self["hits_settled"], 7);
    assert_eq!(
        report_self["hit_receipts"][6]["non_crit_damage"],
        json!("435/2")
    );
}

#[test]
fn sakiri_ordinary_and_a4_carriers_use_only_source_base_and_leave_break_damage_unchanged() {
    let run = |nodes: Vec<u8>, source_base: u64, source_atk: u64, recipient_base: u64| {
        let mut req = request();
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(nodes).unwrap();
        req.actors[0].static_stats.base_atk = source_base;
        req.actors[0].static_stats.atk = Ratio {
            numerator: source_atk,
            denominator: 1,
        };
        let mut ally = req.actors[0].clone();
        ally.slot = 1;
        ally.character_id = "character:1019".into();
        ally.awakening = Default::default();
        ally.static_stats.base_atk = recipient_base;
        ally.static_stats.atk = Ratio {
            numerator: 1000,
            denominator: 1,
        };
        req.actors.push(ally);
        let mut off_field = req.actors[1].clone();
        off_field.slot = 2;
        off_field.character_id = "character:1046".into();
        req.actors.push(off_field);
        let mut dead = req.actors[1].clone();
        dead.slot = 3;
        dead.character_id = "character:1008".into();
        dead.initial_hp = 0;
        req.actors.push(dead);
        req.targets[0].break_profile = Some(crate::simulation::GameTargetBreakProfileConfig {
            capacity: Ratio {
                numerator: 1000,
                denominator: 1,
            },
            current_gauge: Ratio {
                numerator: 1000,
                denominator: 1,
            },
            broken_duration_us: 1_000_000,
            discord_fraction: None,
        });
        req.actions = vec![
            make_action(
                0,
                0,
                "GA_Sagiri_UltraSkill",
                "GE_Player_Sagiri_UltraSkill1_Damage",
                100,
                20_000_000,
            ),
            GameActionCommand::TargetGauge {
                timestamp_us: 500_000,
                target_index: 0,
                gauge: Ratio::zero(),
            },
            GameActionCommand::SwitchActor {
                timestamp_us: 1_000_000,
                target_actor_index: 1,
                cooldown_us: 0,
            },
            make_action(
                2_000_000,
                1,
                "GA_Mint019_Melee",
                "GE_Player_Mint_Melee1_Damage_New",
                0,
                0,
            ),
        ];
        req.relative_duration_us = 3_000_000;
        run_game_scenario(&req, &Default::default()).unwrap()
    };
    let baseline = run(vec![], 1000, 1000, 1000);
    let a4 = run(vec![4], 1000, 1000, 1000);
    for (nodes, base, atk, recipient) in [
        (vec![4], 1000, 5000, 9000),
        (vec![4], 1300, 1000, 1000),
        (vec![1, 2, 3], 1000, 1000, 1000),
    ] {
        let report = run(nodes.clone(), base, atk, recipient);
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        let buffs = super::support::final_buffs(&report);
        let grants = buffs
            .iter()
            .filter(|b| b["id"] == "Sagiri_QDamUp_AtkAdd" || b["id"] == "Sagiri_Level4_AtkAdd")
            .collect::<Vec<_>>();
        assert_eq!(grants.len(), if nodes.contains(&4) { 4 } else { 2 });
        for buff in grants {
            assert_eq!(
                buff["stat_modifiers"]["atk_add"],
                json!(format!("{}/1", base * 3 / 10))
            );
        }
    }
    let ally_hits = |report: &serde_json::Value| {
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| {
                h["actor_index"] == 1 && h["damage_effect_id"] == "GE_Player_Mint_Melee1_Damage_New"
            })
            .map(|h| serde_json::from_value::<ExactFraction>(h["non_crit_damage"].clone()).unwrap())
            .collect::<Vec<_>>()
    };
    for (a, b) in ally_hits(&baseline).iter().zip(ally_hits(&a4)) {
        assert_eq!(b, a.mul(ExactFraction::new(16, 13).unwrap()).unwrap());
    }
    let breaks = |report: &serde_json::Value| {
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| h["damage_effect_id"] == "ntesim.break.contribution")
            .map(|h| h["non_crit_damage"].clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(breaks(&baseline).len(), 3);
    assert_eq!(breaks(&baseline), breaks(&a4));
}

#[test]
fn sakiri_burst_source_weapon_base_projection_and_rejected_refresh_preserve_both_carriers() {
    let run = |weapon: &str, console: &str, nodes: &str| {
        let source = format!(
            "options duration=25 crit=fixed;sakiri char lvl=80/80 awakening={nodes} talent=1,1,1 qte=1 start_energy=120;{weapon}{console}mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;target hp=100000000 def=0 resist=0;active sakiri;sakiri burst;mint swap;mint attack;while .time<20 {{wait(20-.time);}}mint attack;"
        );
        let result = crate::simulation::PreparedSourceScenario::new(
            &source,
            Default::default(),
            &Default::default(),
        )
        .unwrap()
        .run(42)
        .unwrap();
        assert!(
            result.report.execution_completed,
            "{:?}",
            result.report.halt_diagnostic
        );
        super::assert_native_replay(&result);
        result
    };
    let low = run(
        "sakiri add weapon=fork_mofeikesi refine=1 lvl=1/20;",
        "",
        "[4]",
    );
    let high = run(
        "sakiri add weapon=fork_mofeikesi refine=1 lvl=80/80;",
        "",
        "[4]",
    );
    let console = run(
        "sakiri add weapon=fork_mofeikesi refine=1 lvl=80/80;",
        "sakiri add stats atk=5000 atk%=3;",
        "[4]",
    );
    let ordinary = run(
        "sakiri add weapon=fork_mofeikesi refine=1 lvl=80/80;",
        "",
        "[]",
    );
    let base =
        |r: &crate::simulation::SourceScenarioReport| r.request.actors[0].static_stats.base_atk;
    assert!(base(&high) > base(&low));
    assert_eq!(base(&high), base(&console));
    let grants = |r: &crate::simulation::SourceScenarioReport| {
        r.report
            .trace
            .iter()
            .filter(|e| e.event_kind == "sakiri_atk_buff")
            .map(|e| e.data.as_ref().unwrap().atk_add.unwrap())
            .collect::<Vec<_>>()
    };
    assert_eq!(grants(&high), grants(&console));
    assert_eq!(grants(&high).len(), 2);
    let hits = |r: &crate::simulation::SourceScenarioReport| {
        r.report
            .hit_receipts
            .iter()
            .filter(|h| {
                h.actor_index == 1 && h.damage_effect_id == "GE_Player_Mint_Melee1_Damage_New"
            })
            .map(|h| h.non_crit_damage)
            .collect::<Vec<_>>()
    };
    let a = hits(&ordinary);
    let b = hits(&high);
    assert_eq!(a.len(), 2);
    assert_eq!(b.len(), 2);
    assert_eq!(
        b[0].sub(a[0]).unwrap(),
        ExactFraction::new(u128::from(base(&high)) * 39, 1250).unwrap()
    );
    assert_eq!(a[1], b[1]);
    for (a, b) in grants(&low).iter().zip(grants(&high)) {
        assert_eq!(
            b.sub(*a).unwrap(),
            ExactFraction::new(u128::from(base(&high) - base(&low)) * 3, 10).unwrap()
        );
    }
    let mut req = high.request;
    req.input_time_contexts.clear();
    req.actions.truncate(1);
    req.source_exhausted = false;
    let mut rejected = req.actions[0].clone();
    rejected.set_timestamp_us(1_000_000);
    req.actions.push(rejected);
    let report = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["event_kind"] == "sakiri_atk_buff")
            .count(),
        2
    );
    req.actions.truncate(1);
    req.actors[0].initial_energy = 0;
    let report = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|e| e["event_kind"] != "sakiri_atk_buff")
    );
}

#[test]
fn sakiri_legal_a5_reset_refreshes_each_cap_one_atk_carrier_until_the_exact_new_deadline() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(vec![4, 5]).unwrap();
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.awakening = Default::default();
    req.actors.push(ally);
    req.targets[0].airborne_until_us = 5_000_000;
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Sagiri_UltraSkill",
            "GE_Player_Sagiri_UltraSkill1_Damage",
            100,
            20_000_000,
        ),
        make_action(
            1_000_000,
            0,
            "GA_Sagiri_Skill",
            "GE_Player_Sagiri_Skill1_Damage",
            0,
            12_000_000,
        ),
        make_action(
            2_000_000,
            0,
            "GA_Sagiri_UltraSkill",
            "GE_Player_Sagiri_UltraSkill1_Damage",
            100,
            20_000_000,
        ),
    ];
    req.source_exhausted = true;
    req.action_admission_order = crate::simulation::GameActionAdmissionOrder::AfterSettledEvents;
    for (now, count) in [(20_999_999, 2), (21_000_000, 0)] {
        req.relative_duration_us = now;
        let report = run_game_scenario(&req, &Default::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        let buffs = super::support::final_buffs(&report)
            .into_iter()
            .filter(|b| b["id"] == "Sagiri_QDamUp_AtkAdd" || b["id"] == "Sagiri_Level4_AtkAdd")
            .collect::<Vec<_>>();
        assert_eq!(buffs.len(), count);
        for buff in buffs {
            assert_eq!(buff["stacks"], 1);
            assert_eq!(buff["duration"]["expires_at_us"], 21_000_000);
            assert_eq!(buff["stat_modifiers"]["atk_add"], "300/1");
        }
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 200);
    }
}

#[test]
fn sakiri_a4_requires_effect_four_not_any_three_selected_effects() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([1, 2, 3]).unwrap();
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.awakening = ntesim_data::AwakeningSelection::default();
    req.actors.push(ally);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Sagiri_UltraSkill",
            "GE_Player_Sagiri_UltraSkill1_Damage",
            100,
            20_000_000,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 1_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        make_action(
            2_000_000,
            1,
            "GA_Mint019_Melee",
            "GE_Player_Mint_Melee1_Damage_New",
            0,
            0,
        ),
    ];
    req.relative_duration_us = 3_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hit_receipts"][6]["non_crit_damage"], json!("338/5"));
}

#[test]
fn sakiri_off_field_delivery_and_dead_target_boundary() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
    req.actors[0].talent_levels = Some([1, 1, 1]);

    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    req.actors.push(ally);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Sagiri_Melee",
            "GE_Player_Sagiri_Melee1_Damage",
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
    assert_eq!(off_field["hits_settled"], 1);
    assert_eq!(off_field["hit_receipts"][0]["actor_index"], 0);
    assert_eq!(
        off_field["hit_receipts"][0]["non_crit_damage"],
        json!("403/2")
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
    assert_eq!(pending["pending_events"], 1);
}

#[test]
fn sakiri_skill_burst_timing_requires_explicit_strict_acceptance() {
    let mut req = request();
    req.actors[0].talent_levels = None;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Sagiri_Skill",
        "GE_Player_Sagiri_Skill1_Damage",
        0,
        6_000_000,
    )];
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::sakiri::skill::PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::sakiri::skill::PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::simulation::energy::SAKIRI_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::SAKIRI_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    assert_eq!(
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap()["hits_settled"],
        2
    );
}

#[test]
fn sakiri_continuing_rotation_delivers_packet_trains_without_invented_regeneration() {
    let mut req = request();
    req.actors[0].initial_energy = 0;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
    req.actors[0].talent_levels = Some([8, 8, 8]);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Sagiri_Skill",
            "GE_Player_Sagiri_Skill1_Damage",
            0,
            6_000_000,
        ),
        make_action(
            6_500_000,
            0,
            "GA_Sagiri_Skill",
            "GE_Player_Sagiri_Skill1_Damage",
            0,
            6_000_000,
        ),
    ];
    req.relative_duration_us = 10_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 4);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 0);
    // Two Skill casts grant ordinary hit energy: per cast Skill1 (1951/500 -> 3) and
    // Skill2 (2699/500 + 451/500 carry -> 6), then Skill1 (1951/500 + 150/500 carry -> 4)
    // and Skill2 (2699/500 + 101/500 carry -> 5): 9 + 9 = 18 modeled sakiri_hit units.
    assert_eq!(report["final_actor_states"][0]["energy"], 18);
    for event in report["trace"].as_array().unwrap() {
        if let Some(change) = event.get("energy_change")
            && change["source"] != "ability_cost"
        {
            assert_eq!(change["source"], "sakiri_hit");
        }
    }
    // Total damage: 2 casts * (1438 + 3979/2) = 2 * (6855/2) = 6855.
    assert_eq!(
        serde_json::json!(formula_damage_total(&report)),
        json!("6855/1")
    );
}

#[test]
fn sakiri_direct_air_packet_preserves_rates_chain_and_energy_boundary() {
    let rates = [500, 540, 583, 630, 680, 735, 793, 857, 925];
    for (level, rate) in (1_u32..=9).zip(rates) {
        let mut req = request();
        req.actors[0].talent_levels = Some([level, 1, 1]);
        req.actors[0].initial_energy = 0;
        req.actions = vec![make_action(
            0,
            0,
            "GA_Sagiri_Melee",
            "GE_Player_Sagiri_AirAttack1_Damage",
            0,
            0,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        let expected = if rate % 2 == 0 {
            json!(format!("{}/{}", rate / 2, 1))
        } else {
            json!(format!("{}/{}", rate, 2))
        };
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 1);
        assert_eq!(report["hit_receipts"][0]["ability_id"], "GA_Sagiri_Melee");
        assert_eq!(
            report["hit_receipts"][0]["damage_effect_id"],
            "GE_Player_Sagiri_AirAttack1_Damage"
        );
        assert_eq!(report["hit_receipts"][0]["non_crit_damage"], expected);
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_X"
        );
        assert_eq!(report["final_actor_states"][0]["energy"], 2);
        assert_eq!(
            report["final_actor_states"][0]["charge_remainder"],
            json!("1/2")
        );
        assert!(
            report["resolved_model_assumptions"]
                .get(crate::characters::sakiri::skill::DIRECT_PACKET_INPUTS_HYPOTHESIS_ID.as_str())
                .is_some()
        );
        assert!(
            report["resolved_model_assumptions"]
                .get(crate::simulation::energy::SAKIRI_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID.as_str())
                .is_some()
        );
        assert!(
            report["resolved_model_assumptions"]
                .get(crate::simulation::energy::SAKIRI_HIT_ENERGY_HYPOTHESIS_ID.as_str())
                .is_none()
        );
    }

    let mut chain = request();
    chain.actors[0].initial_energy = 0;
    chain.actions = vec![
        make_action(
            0,
            0,
            "GA_Sagiri_Melee",
            "GE_Player_Sagiri_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            300_000,
            0,
            "GA_Sagiri_Melee",
            "GE_Player_Sagiri_AirAttack1_Damage",
            0,
            0,
        ),
        make_action(
            600_000,
            0,
            "GA_Sagiri_Melee",
            "GE_Player_Sagiri_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&chain, &ExecutionLimits::default()).unwrap();
    let effects: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|receipt| receipt["damage_effect_id"].as_str().unwrap())
        .collect();
    assert_eq!(
        effects,
        [
            "GE_Player_Sagiri_Melee1_Damage",
            "GE_Player_Sagiri_AirAttack1_Damage",
            "GE_Player_Sagiri_Melee2_Damage",
            "GE_Player_Sagiri_Melee3_Damage"
        ]
    );
}

#[test]
fn sakiri_direct_packet_input_requires_exact_strict_acceptance() {
    let mut req = request();
    req.actors[0].talent_levels = None;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Sagiri_Melee",
        "GE_Player_Sagiri_AirAttack1_Damage",
        0,
        0,
    )];
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::sakiri::skill::DIRECT_PACKET_INPUTS_HYPOTHESIS_ID.into(),
        crate::characters::sakiri::skill::DIRECT_PACKET_INPUTS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::simulation::energy::SAKIRI_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::SAKIRI_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 1);
}

#[test]
fn sakiri_skill_and_burst_all_in_range_dummy_multi_target() {
    let mut req = request();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);

    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Sagiri_Skill",
            "GE_Player_Sagiri_Skill1_Damage",
            0,
            6_000_000,
        ),
        make_action(
            2_000_000,
            0,
            "GA_Sagiri_UltraSkill",
            "GE_Player_Sagiri_UltraSkill1_Damage",
            0,
            20_000_000,
        ),
    ];
    req.relative_duration_us = 3_000_000;

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );

    // Skill: 2 hits * 2 targets = 4 hits
    // Burst: 6 hits * 2 targets = 12 hits
    // Total = 16 hits settled
    assert_eq!(report["hits_settled"], 16);

    let hits = report["hit_receipts"].as_array().unwrap();
    let skill_hits = hits
        .iter()
        .filter(|h| h["ability_id"] == "GA_Sagiri_Skill")
        .collect::<Vec<_>>();
    assert_eq!(skill_hits.len(), 4);
    assert_eq!(
        skill_hits.iter().filter(|h| h["target_index"] == 0).count(),
        2
    );
    assert_eq!(
        skill_hits.iter().filter(|h| h["target_index"] == 1).count(),
        2
    );

    let burst_hits = hits
        .iter()
        .filter(|h| h["ability_id"] == "GA_Sagiri_UltraSkill")
        .collect::<Vec<_>>();
    assert_eq!(burst_hits.len(), 12);
    assert_eq!(
        burst_hits.iter().filter(|h| h["target_index"] == 0).count(),
        6
    );
    assert_eq!(
        burst_hits.iter().filter(|h| h["target_index"] == 1).count(),
        6
    );

    assert_eq!(
        report["resolved_model_assumptions"]
            [crate::characters::sakiri::skill::ALL_IN_RANGE_HYPOTHESIS_ID.as_str()],
        crate::characters::sakiri::skill::ALL_IN_RANGE_HYPOTHESIS_ID.description()
    );
}

#[test]
fn sakiri_held_skill_continues_after_switch_and_cancels_on_source_death() {
    let mut req = request();
    let mut teammate = req.actors[0].clone();
    teammate.slot = 1;
    teammate.character_id = "character:1039".into(); // Fadia (unbound Support)
    teammate.weapon_id.clear();
    teammate.gear_procs.clear();
    req.actors.push(teammate);

    // Sakiri (actor 0) starts at t=0 and casts Skill (GA_Sagiri_Skill).
    // Devour Whole: hit 1 (Skill1) at 150_000us, hit 2 (Skill2) at 450_000us.
    // Recovery window is 650_000us, cooldown 12_000_000us.
    // At t=650_000us, Sakiri switches to teammate (actor 1).
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Sagiri_Skill",
            "GE_Player_Sagiri_Skill1_Damage",
            0,
            12_000_000,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 650_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    req.relative_duration_us = 2_000_000;

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 2);

    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits[0]["actor_index"], 0);
    assert_eq!(
        hits[0]["damage_effect_id"],
        "GE_Player_Sagiri_Skill1_Damage"
    );
    assert_eq!(hits[1]["actor_index"], 0);
    assert_eq!(
        hits[1]["damage_effect_id"],
        "GE_Player_Sagiri_Skill2_Damage"
    );
    assert_eq!(report["final_buff_state"]["active_actor_index"], 1);

    // Death scenario: Sakiri takes fatal damage before child packet delivery.
    let mut death_req = req.clone();
    death_req.incoming_damage_events = vec![GameActorDamageEvent {
        timestamp_us: 0,
        actor_index: 0,
        delivered_damage: 17720,
    }];
    death_req.model_assumptions.insert(
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID
            .description()
            .into(),
    );

    let death_report = run_game_scenario(&death_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(death_report["execution_completed"], true);
    assert_eq!(death_report["final_actor_states"][0]["hp"], 0);
    assert!(
        death_report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["event_kind"] == "hit_owner_unavailable")
    );
}

#[test]
fn sakiri_riposte_from_critical_dodge_consumed_once_and_grants_energy() {
    let source = r#"
options iteration=1 duration=8 crit=fixed stamina_capacity=60 initial_stamina=60 stamina_use_efficiency=1 stamina_recovery_rate=0;
sakiri char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
target hp=100000000 def=0 resist=0;
active sakiri;

sakiri dash[critical=true]; wait(0.4); sakiri attack;
wait(0.8); sakiri attack;
"#;
    let mut req = crate::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    req.model_assumptions.insert(
        crate::characters::sakiri::riposte::HYPOTHESIS_ID.into(),
        crate::characters::sakiri::riposte::HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );

    let hits: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_Sagiri_PerfectEvadeAttack_Damage")
        .collect();
    assert_eq!(hits.len(), 1);
    let riposte_meta = super::support::hit_metadata(&report, hits[0]);
    assert_eq!(riposte_meta["action"]["input_role"], "critical_riposte");
    assert_eq!(
        riposte_meta["packet"]["native_ga_name"],
        "GA_Sagiri_ExtremEvadeAtk"
    );
    assert_eq!(
        riposte_meta["packet"]["native_damage_source_category"],
        "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Z"
    );

    let melee1_hits: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_Sagiri_Melee1_Damage")
        .collect();
    assert_eq!(melee1_hits.len(), 1);

    let riposte_grants: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "sakiri_riposte_energy_grant")
        .collect();
    assert_eq!(riposte_grants.len(), 1);
    assert_eq!(
        riposte_grants[0]["energy_change"]["hit_basis"]["base"],
        json!("27/5")
    );
}

#[test]
fn sakiri_air_skill_eating_delivers_three_packets_and_fans_out() {
    let mut req = request();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);

    req.actions = vec![make_action(
        0,
        0,
        "GA_Sagiri_Skill",
        "GE_Player_Sagiri_Skill3_Damage",
        0,
        6_000_000,
    )];
    req.relative_duration_us = 2_000_000;

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 6);

    let hits = report["hit_receipts"].as_array().unwrap();
    let s3 = hits
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_Sagiri_Skill3_Damage")
        .count();
    let s4 = hits
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_Sagiri_Skill4_Damage")
        .count();
    let s5 = hits
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_Sagiri_Skill5_Damage")
        .count();
    assert_eq!(s3, 2);
    assert_eq!(s4, 2);
    assert_eq!(s5, 2);
}

#[test]
fn sakiri_a5_reset_triggers_from_air_skill_swallow() {
    let limits = ExecutionLimits::default();
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(5);
    req.actors[0].initial_energy = 0;
    req.actors[0].energy_capacity = 100;
    req.targets[0].airborne_until_us = 2_500_000;

    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Sagiri_UltraSkill",
            "GE_Player_Sagiri_UltraSkill1_Damage",
            0,
            20_000_000,
        ),
        make_action(
            1_000_000,
            0,
            "GA_Sagiri_Skill",
            "GE_Player_Sagiri_Skill3_Damage",
            0,
            6_000_000,
        ),
    ];
    req.relative_duration_us = 3_000_000;

    let report = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );

    let a5_events: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "sakiri_a5_reset")
        .collect();
    assert_eq!(a5_events.len(), 1);
    assert_eq!(report["final_actor_states"][0]["energy"], 100);
}

#[test]
fn sakiri_plunge_damage_and_multi_target_scope() {
    let mut req = request();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);

    req.actions = vec![make_action(
        0,
        0,
        "GA_Sagiri_Melee",
        "GE_Player_Sagiri_AirAttack1_Damage",
        0,
        0,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 2);

    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(
        hits[0]["damage_effect_id"],
        "GE_Player_Sagiri_AirAttack1_Damage"
    );
    assert_eq!(hits[0]["target_index"], 0);
    assert_eq!(
        hits[1]["damage_effect_id"],
        "GE_Player_Sagiri_AirAttack1_Damage"
    );
    assert_eq!(hits[1]["target_index"], 1);
}
