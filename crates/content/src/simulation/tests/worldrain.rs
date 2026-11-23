use super::*;
use serde_json::json;

#[test]
fn worldrain_uses_packet_source_and_element_in_the_additive_damage_zone() {
    let mut req = research_request();
    req.stochastic_crit = None;
    req.actors[0].weapon_id = "weapon:fork_Worldrain".into();
    req.actors[0].gear_procs = vec![GearProcConfig::WorldrainDamage {
        cycle_intensity_bonus: Ratio {
            numerator: 36,
            denominator: 1,
        },
        duration_us: 15_000_000,
        damage_bonus: Ratio {
            numerator: 3,
            denominator: 10,
        },
    }];
    req.actors[0].static_stats.damage_bonus = Ratio {
        numerator: 1,
        denominator: 10,
    };
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.targets[0].resistance = Ratio::zero();
    for (index, (source, element)) in [
        ("A", "COSMOS"),
        ("E", "COSMOS"),
        ("Q", "COSMOS"),
        ("H", "COSMOS"),
        ("E", "NATURE"),
        ("Q", "NORMAL"),
    ]
    .into_iter()
    .enumerate()
    {
        let metadata = req.action_metadata.get_mut(&index).unwrap();
        metadata.action_gameplay_tags = Some(vec!["Ability.Melee".into()]);
        metadata.damage_source_category = Some(format!(
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_{source}"
        ));
        metadata.damage_type_ex = Some(format!("EExecutionDamageTypeEX::DAMAGE_TYPE_{element}"));
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // 1000 ATK * 1/2 DEF mitigation * (1 + .1 general + .3 eligible weapon).
    // Eligible: E COSMOS (700), Q COSMOS (700).
    // Ineligible: A COSMOS (550), H COSMOS (550), E NATURE (550), Q NORMAL (550).
    assert_eq!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|hit| hit["rounded_damage"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![550, 700, 700, 550, 550, 550]
    );
    let mut duplicate = req.clone();
    let extra = duplicate.actors[0].gear_procs[0].clone();
    duplicate.actors[0].gear_procs.push(extra);
    assert!(
        run_game_scenario(&duplicate, &ExecutionLimits::default())
            .unwrap_err()
            .contains("duplicates")
    );
    req.action_metadata
        .get_mut(&0)
        .unwrap()
        .damage_source_category =
        Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_NULL".into());
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("unsupported Worldrain damage source classification")
    );
    req.action_metadata
        .get_mut(&0)
        .unwrap()
        .damage_source_category = None;
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires damage source classification")
    );
    req.actors[0].weapon_id = "weapon:fork_PaperPlane".into();
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires equipped")
    );
}

#[test]
fn worldrain_delayed_hit_retains_wearer_eligibility_after_switching() {
    let mut req = sample_s1_request();
    req.actors[0].character_id = "character:1046".into();
    req.actors[0].character_level = 80;
    req.actors[0].character_max_level = Some(80);
    req.actors[0].talent_levels = Some([1, 1, 1]);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
    req.actors[0].weapon_id = "weapon:fork_Worldrain".into();
    req.actors[0].gear_procs = vec![GearProcConfig::WorldrainDamage {
        cycle_intensity_bonus: Ratio {
            numerator: 36,
            denominator: 1,
        },
        duration_us: 15_000_000,
        damage_bonus: Ratio {
            numerator: 3,
            denominator: 10,
        },
    }];
    req.actors[0].set_id = None;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.damage_bonus = Ratio {
        numerator: 1,
        denominator: 10,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].static_stats.crit_rate = Ratio::zero();

    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.weapon_id = String::new();
    ally.gear_procs.clear();
    ally.static_stats.damage_bonus = Ratio {
        numerator: 1,
        denominator: 10,
    };
    req.actors.push(ally);

    req.targets[0].resistance = Ratio::zero();
    req.targets[0].level = 80;
    req.actions = vec![
        GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Female046_Skill".into(),
            damage_effect_id: "GE_Player_Female046_Skill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 6_000_000,
            hit_delay_us: 2_000_000,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 1_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 1_500_000,
            actor_index: 1,
            ability_id: "GA_Mint019_Melee".into(),
            damage_effect_id: "GE_Player_Mint_Melee1_Damage_New".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 0,
        },
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // Mint melee lands first at 1.5s: 1000 ATK * 0.104 coeff * 1/2 DEF * 1.1 general = 57.2 -> 57.
    // Zero's skill hit lands off-field at 2.0s: 1000 ATK * 0.2 coeff * 1/2 DEF * (1 + 0.1 + 0.3) = 140.
    assert_eq!(report["hit_receipts"][0]["rounded_damage"], 57);
    // Mint's second packet: 1000 * .231 * .5 * 1.1 = 127.05.
    assert_eq!(report["hit_receipts"][1]["rounded_damage"], 127);
    assert_eq!(report["hit_receipts"][2]["rounded_damage"], 140);
    assert_eq!(
        report["resolved_model_assumptions"][crate::weapons::worldrain::HYPOTHESIS_ID.as_str()],
        crate::weapons::worldrain::HYPOTHESIS_ID.description()
    );
}

#[test]
fn worldrain_tiers_scale_cosmos_skill_and_burst_linearly() {
    let tiers = [
        (
            Ratio {
                numerator: 30,
                denominator: 100,
            },
            130,
            1,
            17251,
            20,
            4082,
            5,
            4368,
            5,
        ),
        (
            Ratio {
                numerator: 33,
                denominator: 100,
            },
            133,
            1,
            176491,
            200,
            20881,
            25,
            22344,
            25,
        ),
        (
            Ratio {
                numerator: 36,
                denominator: 100,
            },
            136,
            1,
            22559,
            25,
            21352,
            25,
            22848,
            25,
        ),
        (
            Ratio {
                numerator: 39,
                denominator: 100,
            },
            139,
            1,
            184453,
            200,
            21823,
            25,
            23352,
            25,
        ),
        (
            Ratio {
                numerator: 42,
                denominator: 100,
            },
            142,
            1,
            94217,
            100,
            22294,
            25,
            23856,
            25,
        ),
    ];
    for (
        i,
        (
            r,
            skill_num,
            skill_den,
            burst_num,
            burst_den,
            skill3_num,
            skill3_den,
            skill4_num,
            skill4_den,
        ),
    ) in tiers.into_iter().enumerate()
    {
        let mut req = sample_s1_request();
        req.actors[0].character_id = "character:1046".into();
        req.actors[0].character_level = 80;
        req.actors[0].character_max_level = Some(80);
        req.actors[0].talent_levels = Some([1, 1, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
        req.actors[0].weapon_id = "weapon:fork_Worldrain".into();
        req.actors[0].weapon_tier = (i + 1) as u32;
        req.actors[0].gear_procs = vec![GearProcConfig::WorldrainDamage {
            damage_bonus: r,
            cycle_intensity_bonus: Ratio {
                numerator: 36,
                denominator: 1,
            },
            duration_us: 15_000_000,
        }];
        req.actors[0].set_id = None;
        req.actors[0].static_stats.atk = Ratio {
            numerator: 1000,
            denominator: 1,
        };
        req.actors[0].static_stats.damage_bonus = Ratio::zero();
        req.actors[0].static_stats.elemental_bonus = Ratio::zero();
        req.actors[0].static_stats.crit_rate = Ratio::zero();
        req.targets[0].resistance = Ratio::zero();
        req.targets[0].level = 80;
        req.actions = vec![
            GameActionCommand::InvokeAbility {
                timestamp_us: 0,
                actor_index: 0,
                ability_id: "GA_Female046_Skill".into(),
                damage_effect_id: "GE_Player_Female046_Skill1_Damage".into(),
                array_index: 0,
                atk_rate: Ratio::one(),
                scaling_stat: crate::simulation::NativeScalingStat::Atk,
                energy_cost: 0,
                recovery_us: 0,
                cooldown_us: 6_000_000,
                hit_delay_us: 0,
            },
            GameActionCommand::InvokeAbility {
                timestamp_us: 100_000,
                actor_index: 0,
                ability_id: "GA_Female046_UltraSkill".into(),
                damage_effect_id: "GE_Player_Female046_UltraSkill1_Damage".into(),
                array_index: 0,
                atk_rate: Ratio::one(),
                scaling_stat: crate::simulation::NativeScalingStat::Atk,
                energy_cost: 100,
                recovery_us: 0,
                cooldown_us: 20_000_000,
                hit_delay_us: 0,
            },
        ];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let receipts = report["hit_receipts"].as_array().unwrap();
        assert_eq!(receipts.len(), 11);
        let damage_by_effect: std::collections::BTreeMap<_, _> = receipts
            .iter()
            .map(|receipt| {
                (
                    receipt["damage_effect_id"].as_str().unwrap(),
                    &receipt["non_crit_damage"],
                )
            })
            .collect();
        assert_eq!(
            damage_by_effect["GE_Player_Female046_Skill1_Damage"],
            &json!(format!("{}/{}", skill_num, skill_den))
        );
        assert_eq!(
            damage_by_effect["GE_Player_Female046_Skill2_Damage"],
            &json!(format!("{}/{}", skill_num, skill_den))
        );
        assert_eq!(
            damage_by_effect["GE_Player_Female046_Skill3_Damage"],
            &json!(format!("{}/{}", skill3_num, skill3_den))
        );
        assert_eq!(
            damage_by_effect["GE_Player_Female046_Skill4_Damage"],
            &json!(format!("{}/{}", skill4_num, skill4_den))
        );
        assert_eq!(
            damage_by_effect["GE_Player_Female046_UltraSkill1_Damage"],
            &json!(format!("{}/{}", burst_num, burst_den))
        );
        assert_burst_damage(receipts, r);
    }
}

fn assert_burst_damage(receipts: &[serde_json::Value], bonus: Ratio) {
    let burst = receipts
        .iter()
        .filter(|receipt| receipt["ability_id"] == "GA_Female046_UltraSkill")
        .collect::<Vec<_>>();
    assert_eq!(burst.len(), 7);
    for (receipt, coefficient) in burst.iter().zip([1327, 53, 53, 53, 53, 1775, 1686]) {
        let expected = ExactFraction::new(coefficient, 2)
            .unwrap()
            .mul(
                ExactFraction::new(1, 1)
                    .unwrap()
                    .add(ExactFraction::from_ratio(bonus).unwrap())
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(receipt["non_crit_damage"], json!(expected));
    }
}
