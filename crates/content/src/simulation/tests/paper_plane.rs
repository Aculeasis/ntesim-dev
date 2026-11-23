use super::*;

#[test]
fn paper_plane_uses_packet_source_and_element_in_the_additive_damage_zone() {
    let mut req = research_request();
    req.stochastic_crit = None;
    req.actors[0].weapon_id = "weapon:fork_PaperPlane".into();
    req.actors[0].gear_procs = vec![GearProcConfig::PaperPlaneDamage {
        damage_bonus: Ratio {
            numerator: 1,
            denominator: 5,
        },
    }];
    req.actors[0].static_stats.damage_bonus = Ratio {
        numerator: 1,
        denominator: 10,
    };
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.targets[0].resistance = Ratio::zero();
    for (index, (source, element)) in [
        ("A", "NATURE"),
        ("E", "NATURE"),
        ("Q", "NATURE"),
        ("H", "NATURE"),
        ("E", "COSMOS"),
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
    // 1000 ATK * 1/2 DEF mitigation * (1 + .1 general + .2 eligible weapon).
    assert_eq!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|hit| hit["rounded_damage"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![550, 650, 650, 550, 550, 550]
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
            .contains("unsupported Paper Plane damage source classification")
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
    req.actors[0].weapon_id = "weapon:fork_mofeikesi".into();
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires equipped")
    );
}

#[test]
fn paper_plane_delayed_hit_retains_wearer_eligibility_after_switching() {
    let mut req = sample_s1_request();
    req.actors[0].weapon_id = "weapon:fork_PaperPlane".into();
    req.actors[0].gear_procs = vec![GearProcConfig::PaperPlaneDamage {
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
            ability_id: "GA_Mint019_Skill".into(),
            damage_effect_id: "GE_Player_Mint_Skill1_Damage_New".into(),
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
        report["resolved_model_assumptions"][crate::weapons::paper_plane::HYPOTHESIS_ID.as_str()],
        crate::weapons::paper_plane::HYPOTHESIS_ID.description()
    );
}
