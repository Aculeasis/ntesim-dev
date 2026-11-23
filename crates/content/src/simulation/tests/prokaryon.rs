use super::*;

#[test]
fn prokaryon_uses_packet_category_across_elements_and_rejects_invalid_configs() {
    let mut req = research_request();
    req.stochastic_crit = None;
    req.actors[0].weapon_id = "weapon:fork_Prokaryon".into();
    req.actors[0].gear_procs = vec![GearProcConfig::ProkaryonDamage {
        damage_bonus: Ratio {
            numerator: 3,
            denominator: 25,
        },
    }];
    req.actors[0].static_stats.damage_bonus = Ratio {
        numerator: 1,
        denominator: 10,
    };
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.targets[0].resistance = Ratio::zero();
    let cases = [
        ("A", "NATURE"),
        ("A", "NORMAL"),
        ("A", "COSMOS"),
        ("E", "NATURE"),
        ("Q", "NATURE"),
        ("H", "NATURE"),
        ("B", "NATURE"),
        ("F", "NATURE"),
        ("P", "NATURE"),
        ("R", "NATURE"),
        ("T", "NATURE"),
        ("X", "NATURE"),
        ("Z", "NATURE"),
    ];
    let action = req.actions[0].clone();
    let metadata = req.action_metadata[&0].clone();
    req.actions.clear();
    req.action_metadata.clear();
    for (index, (category, element)) in cases.into_iter().enumerate() {
        let mut command = action.clone();
        if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut command {
            *timestamp_us = index as u64 * 10;
        }
        req.actions.push(command);
        let mut m = metadata.clone();
        // The input is a Skill; A packets must still receive Basic Attack bonuses.
        m.action_gameplay_tags = Some(vec!["Ability.Skill".into()]);
        m.damage_source_category = Some(format!(
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_{category}"
        ));
        m.damage_type_ex = Some(format!("EExecutionDamageTypeEX::DAMAGE_TYPE_{element}"));
        req.action_metadata.insert(index, m);
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // 1000 ATK * .5 DEF * (1 + .1 general + .12 eligible weapon).
    for (index, hit) in report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        assert_eq!(hit["rounded_damage"], if index < 3 { 610 } else { 550 });
    }
    let mut duplicate = req.clone();
    duplicate.actors[0]
        .gear_procs
        .push(req.actors[0].gear_procs[0].clone());
    assert!(
        run_game_scenario(&duplicate, &ExecutionLimits::default())
            .unwrap_err()
            .contains("duplicates")
    );
    for category in [
        None,
        Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_NULL"),
    ] {
        let mut invalid = req.clone();
        invalid
            .action_metadata
            .get_mut(&0)
            .unwrap()
            .damage_source_category = category.map(str::to_owned);
        assert!(
            run_game_scenario(&invalid, &ExecutionLimits::default())
                .unwrap_err()
                .contains("classification")
        );
    }
    req.actors[0].weapon_id.clear();
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires equipped")
    );
}

#[test]
fn prokaryon_delayed_hit_retains_wearer_eligibility_after_switching() {
    let mut req = sample_s1_request();
    req.actors[0].weapon_id = "weapon:fork_Prokaryon".into();
    req.actors[0].gear_procs = vec![GearProcConfig::ProkaryonDamage {
        damage_bonus: Ratio {
            numerator: 1,
            denominator: 5,
        },
    }];
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
    ally.character_id = "character:1003".into();
    ally.weapon_id = String::new();
    ally.gear_procs.clear();
    req.actors.push(ally);
    req.targets[0].resistance = Ratio::zero();
    req.actions = vec![
        GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Mint019_Melee".into(),
            damage_effect_id: "GE_Player_Mint_Melee1_Damage_New".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 2_000_000,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 1_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 1_600_000,
            actor_index: 1,
            ability_id: "GA_Sagiri_Melee".into(),
            damage_effect_id: "GE_Player_Sagiri_Melee1_Damage".into(),
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
    super::support::assert_non_support_damage_prefix(&report, &[550, 650]);
    assert_eq!(
        report["resolved_model_assumptions"][crate::weapons::prokaryon::HYPOTHESIS_ID.as_str()],
        crate::weapons::prokaryon::HYPOTHESIS_ID.description()
    );
}
