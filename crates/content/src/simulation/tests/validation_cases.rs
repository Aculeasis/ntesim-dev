use super::*;

#[test]
fn reviewed_weapon_deadlines_check_delivery_and_the_longest_cooldown() {
    let amount = Ratio {
        numerator: 1,
        denominator: 10,
    };
    let mut actor = research_request().actors[0].clone();
    let action = super::support::declared_test_action(
        "caller-test",
        crate::simulation::metadata::ActionCategory::Skill,
    );
    for (weapon, proc, delivery_overflows) in [
        (
            "weapon:fork_ThiefCandy",
            GearProcConfig::ThiefCandyDodge {
                damage_bonus: amount,
                duration_us: 10,
            },
            false,
        ),
        (
            "weapon:fork_TigerTally",
            GearProcConfig::TigerTallyAtk {
                atk_bonus: amount,
                normal_damage_bonus: amount,
                duration_us: 10,
                talisman_window_us: 10,
                boss_damage_bonus: amount,
                boss_damage_duration_us: 10,
            },
            false,
        ),
        (
            "weapon:fork_jingmotingyuan",
            GearProcConfig::JingmotingyuanCrit {
                crit_damage_bonus: amount,
                duration_us: 10,
            },
            false,
        ),
        (
            "weapon:fork_dustbin",
            GearProcConfig::DustbinBreak {
                break_intensity_bonus: amount,
                duration_us: 10,
                cooldown_us: 20,
            },
            true,
        ),
        (
            "weapon:fork_Rose",
            GearProcConfig::RoseThorn {
                atk_bonus: amount,
                crit_damage_per_stack: amount,
                duration_us: 10,
                broken_extension_us: 10,
            },
            true,
        ),
    ] {
        actor.weapon_id = weapon.into();
        actor.gear_procs = vec![proc];
        assert_eq!(
            crate::gear::validate_action_deadlines(
                &actor.gear_procs,
                &action,
                u64::MAX - 25,
                20,
                0
            )
            .is_err(),
            delivery_overflows,
            "{weapon}"
        );
        assert!(
            crate::gear::validate_action_deadlines(&actor.gear_procs, &action, u64::MAX - 5, 0, 0)
                .is_err(),
            "{weapon}"
        );
    }
    actor.weapon_id = "weapon:fork_dustbin".into();
    actor.gear_procs = vec![GearProcConfig::DustbinBreak {
        break_intensity_bonus: amount,
        duration_us: 10,
        cooldown_us: 20,
    }];
    assert!(
        crate::gear::validate_config(&actor, 0, u64::MAX - 15, &ExecutionLimits::default())
            .is_err()
    );
}

#[test]
fn target_indexes_must_match_the_contiguous_runtime_order() {
    let mut request = research_request();
    let mut second = request.targets[0].clone();
    second.target_index = 2;
    request.targets.push(second);
    let error = run_game_scenario(&request, &ExecutionLimits::default()).unwrap_err();
    assert!(error.contains("target index 2 must match its contiguous position 1"));
}

#[test]
fn target_elemental_resistances_reject_noncanonical_and_invalid_ratios() {
    for (element, resistance) in [
        ("anima", Ratio::zero()),
        (
            "NATURE",
            Ratio {
                numerator: 1,
                denominator: 0,
            },
        ),
    ] {
        let mut request = research_request();
        request.targets[0]
            .elemental_resistances
            .insert(element.into(), resistance);
        assert!(validate_game_request(&request, &ExecutionLimits::default()).is_err());
    }

    let request = research_request();
    let serialized = serde_json::to_value(&request).unwrap();
    assert!(
        serialized["targets"][0]
            .get("elemental_resistances")
            .is_none()
    );
    let legacy: GameScenarioRequest = serde_json::from_value(serialized).unwrap();
    assert!(legacy.targets[0].elemental_resistances.is_empty());
}

#[test]
fn modeling_metadata_is_bounded_preserved_and_cannot_authorize_strict_execution() {
    let mut request = research_request();
    request.model_assumptions.insert(
        "test.coefficient_v1".into(),
        "Reviewed coefficient hypothesis".into(),
    );
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["model_assumptions"],
        serde_json::to_value(&request.model_assumptions).unwrap()
    );
    for (id, description) in [("", "valid"), ("invalid id", "valid"), ("valid", " ")] {
        let mut invalid = request.clone();
        invalid
            .model_assumptions
            .insert(id.into(), description.into());
        assert!(
            run_game_scenario(&invalid, &ExecutionLimits::default())
                .unwrap_err()
                .contains("model_assumptions")
        );
    }
    let mut invalid = request.clone();
    invalid
        .model_assumptions
        .insert("oversized".into(), "x".repeat(4097));
    assert!(run_game_scenario(&invalid, &ExecutionLimits::default()).is_err());
    let mut invalid = request.clone();
    for i in 0..32 {
        invalid
            .model_assumptions
            .insert(format!("extra.{i}"), "Hypothesis".into());
    }
    assert!(
        run_game_scenario(&invalid, &ExecutionLimits::default())
            .unwrap_err()
            .contains("32 entries")
    );
    let mut canonical = request.clone();
    canonical.model_assumptions = ntesim_contracts::hypothesis_descriptions()
        .map(|(id, description)| (id.to_string(), description.to_string()))
        .collect();
    assert!(canonical.model_assumptions.len() > 32);
    let first_id = canonical.model_assumptions.keys().next().unwrap().clone();
    for i in 0..32 {
        canonical
            .model_assumptions
            .insert(format!("custom.{i}"), "Hypothesis".into());
    }
    crate::simulation::validation::validate_model_assumptions(&canonical).unwrap();
    // A spoofed canonical identity must consume a custom slot, not bypass it.
    canonical
        .model_assumptions
        .insert(first_id, "Mismatched description".into());
    assert!(
        crate::simulation::validation::validate_model_assumptions(&canonical)
            .unwrap_err()
            .contains("32 entries")
    );
}

#[test]
fn action_metadata_rejects_stale_indices_identities_and_native_overrides() {
    let limits = ExecutionLimits::default();

    let mut out_of_range = research_request();
    out_of_range
        .action_metadata
        .insert(6, out_of_range.action_metadata[&0].clone());
    assert!(
        validate_game_request(&out_of_range, &limits)
            .unwrap_err()
            .contains("index 6 must identify an InvokeAbility")
    );

    let mut mismatched_declaration = research_request();
    mismatched_declaration
        .action_metadata
        .get_mut(&0)
        .unwrap()
        .ability_id = "stale-ability".into();
    assert!(
        validate_game_request(&mismatched_declaration, &limits)
            .unwrap_err()
            .contains("identity at action 0 does not match")
    );

    let mut native_override = sample_s1_request();
    native_override.action_metadata.insert(
        0,
        crate::simulation::ResearchActionMetadata {
            hypothesis_id: crate::simulation::metadata::NORMAL_PHYSICAL_HYPOTHESIS_ID
                .as_str()
                .into(),
            ability_id: "GA_Mint019_Melee".into(),
            damage_effect_id: "GE_Player_Mint_Melee1_Damage_New".into(),
            action_gameplay_tags: Some(vec!["Ability.Skill".into()]),
            input_action_role: None,
            damage_type_ex: None,
            damage_source_category: None,
            fixed_crit_rate: None,
            unbal_value: None,
            packet_ga_name_binding: None,
        },
    );
    assert!(
        validate_game_request(&native_override, &limits)
            .unwrap_err()
            .contains("cannot override native action metadata")
    );
}

#[test]
pub(crate) fn configured_kit_buff_deadlines_reject_overflow_and_allow_exact_u64_endpoints() {
    let limits = ExecutionLimits {
        max_horizon_us: u64::MAX,
        ..ExecutionLimits::default()
    };
    for (character, ability, effect, awakening, duration) in [
        (
            "character:1019",
            "GA_Mint019_Skill",
            "GE_Player_Mint_Skill1_Damage_New",
            5,
            6_000_000,
        ),
        (
            "character:1003",
            "GA_Sagiri_UltraSkill",
            "GE_Player_Sagiri_UltraSkill1_Damage",
            4,
            20_000_000,
        ),
        (
            "character:1075",
            "GA_Oneiroi_Skill",
            "GE_Player_Oneiroi_Skill_Gun_Damage",
            0,
            20_000_000,
        ),
    ] {
        let mut req = sample_s1_request();
        req.relative_duration_us = u64::MAX;
        req.absolute_limit_us = req.relative_duration_us;
        req.actors[0].gear_procs.clear();
        req.actors[0].character_id = character.into();
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        let mut command = req.actions[1].clone();
        if let GameActionCommand::InvokeAbility {
            timestamp_us,
            ability_id,
            damage_effect_id,
            recovery_us,
            cooldown_us,
            hit_delay_us,
            ..
        } = &mut command
        {
            *timestamp_us = u64::MAX - duration;
            *ability_id = ability.into();
            *damage_effect_id = effect.into();
            *recovery_us = 0;
            *cooldown_us = 0;
            *hit_delay_us = 0;
        }
        req.action_metadata.clear();
        req.actions = vec![command];
        let report = run_game_scenario(&req, &limits).unwrap();
        assert_eq!(report["success"], true, "{ability}");
        assert_eq!(report["invariants_passed"], true);
        if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut req.actions[0] {
            *timestamp_us += 1;
        }
        assert!(
            run_game_scenario(&req, &limits)
                .unwrap_err()
                .contains("configured kit buff deadline overflow"),
            "{ability}"
        );
        if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut req.actions[0] {
            *timestamp_us = u64::MAX;
        }
        assert!(run_game_scenario(&req, &limits).is_err());
    }
    let mut req = sample_s1_request();
    req.relative_duration_us = u64::MAX;
    req.absolute_limit_us = req.relative_duration_us;
    req.actors[0].gear_procs.clear();
    req.action_metadata.clear();
    req.actions.truncate(1);
    req.action_metadata
        .retain(|index, _| *index < req.actions.len());
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        recovery_us,
        ..
    } = &mut req.actions[0]
    {
        *timestamp_us = u64::MAX;
        *recovery_us = 0;
    }
    assert_eq!(run_game_scenario(&req, &limits).unwrap()["success"], true);
}

#[test]
pub(crate) fn cumulative_energy_spending_overflow_returns_an_error() {
    let limits = ExecutionLimits {
        max_pool_capacity: u64::MAX,
        ..ExecutionLimits::default()
    };
    let mut req = sample_s1_request();
    req.actors[0].initial_energy = u64::MAX;
    req.actors[0].energy_capacity = u64::MAX;
    req.actors[0].gear_procs = vec![GearProcConfig::MindRoyaleEnergy {
        energy_grant: u64::MAX,
        cooldown_us: 0,
    }];
    // Keep one living dummy without generating on-hit energy. Initial defeat
    // now ends execution before any payment can reach the overflow boundary.
    req.targets[0].hp_current = 1;
    req.actors[0].static_stats.atk = Ratio::zero();
    req.actors[0].static_stats.base_atk = 0;
    let mut ultra = req.actions[2].clone();
    let mut skill = req.actions[1].clone();
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        energy_cost,
        recovery_us,
        cooldown_us,
        ..
    } = &mut ultra
    {
        *timestamp_us = 0;
        *energy_cost = u64::MAX;
        *recovery_us = 0;
        *cooldown_us = 0;
    }
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        recovery_us,
        cooldown_us,
        ..
    } = &mut skill
    {
        *timestamp_us = 1;
        *recovery_us = 0;
        *cooldown_us = 0;
    }
    let mut later_ultra = ultra.clone();
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut later_ultra {
        *timestamp_us = 2;
    }
    req.action_metadata.clear();
    req.actions = vec![ultra, skill, later_ultra];
    assert!(
        run_game_scenario(&req, &limits)
            .unwrap_err()
            .contains("energy spend accounting overflow")
    );
}

#[test]
pub(crate) fn rejected_admissions_preserve_the_complete_accepted_prefix() {
    let limits = ExecutionLimits::default();
    let mut prefix = sample_s1_request();
    prefix.actors[0].gear_procs.clear();
    let mut ally = prefix.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1003".into();
    ally.gear_procs.clear();
    prefix.actors.push(ally);
    prefix.action_metadata.clear();
    prefix.actions.truncate(1);
    prefix
        .action_metadata
        .retain(|index, _| *index < prefix.actions.len());
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        recovery_us,
        cooldown_us,
        hit_delay_us,
        energy_cost,
        ..
    } = &mut prefix.actions[0]
    {
        *timestamp_us = 0;
        *recovery_us = 1_000_000;
        *cooldown_us = 5_000_000;
        *hit_delay_us = 0;
        *energy_cost = 25;
    }

    let assert_rejection =
        |prefix: &GameScenarioRequest, rejected: GameActionCommand, diagnostic: &str| {
            let before = run_game_scenario(prefix, &limits).unwrap();
            assert_eq!(before["success"], true);
            let mut request = prefix.clone();
            request.actions.push(rejected);
            let after = run_game_scenario(&request, &limits).unwrap();
            assert_eq!(after["success"], false);
            assert_eq!(after["execution_completed"], false);
            assert_eq!(after["invariants_passed"], true);
            assert!(
                after["halt_diagnostic"]
                    .as_str()
                    .unwrap()
                    .starts_with(diagnostic)
            );
            for field in [
                "final_actor_states",
                "final_target_states",
                "hit_receipts",
                "trace",
                "total_damage_exact",
                "hits_settled",
            ] {
                assert_eq!(after[field], before[field], "{diagnostic}: {field}");
            }
        };

    for (actor, timestamp, cost, diagnostic) in [
        (1, 500_000, 25, "ACTOR_NOT_ACTIVE"),
        (0, 500_000, 25, "ACTOR_LOCKED"),
        (0, 2_000_000, 25, "ABILITY_ON_COOLDOWN"),
        (0, 6_000_000, 100, "RESOURCE_SHORTAGE"),
    ] {
        let mut rejected = prefix.actions[0].clone();
        if let GameActionCommand::InvokeAbility {
            actor_index,
            timestamp_us,
            energy_cost,
            ability_id,
            damage_effect_id,
            ..
        } = &mut rejected
        {
            *actor_index = actor;
            *timestamp_us = timestamp;
            *energy_cost = cost;
            if actor == 1 {
                *ability_id = "GA_Sagiri_Melee".into();
                *damage_effect_id = "GE_Player_Sagiri_AirAttack1_Damage".into();
            }
        }
        assert_rejection(&prefix, rejected, diagnostic);
    }
    assert_rejection(
        &prefix,
        GameActionCommand::SwitchActor {
            timestamp_us: 500_000,
            target_actor_index: 1,
            cooldown_us: 4_000_000,
        },
        "ACTOR_LOCKED",
    );

    prefix.actions.extend([
        GameActionCommand::SwitchActor {
            timestamp_us: 1_000_000,
            target_actor_index: 1,
            cooldown_us: 4_000_000,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 2_000_000,
            target_actor_index: 0,
            cooldown_us: 4_000_000,
        },
    ]);
    assert_rejection(
        &prefix,
        GameActionCommand::SwitchActor {
            timestamp_us: 3_000_000,
            target_actor_index: 1,
            cooldown_us: 4_000_000,
        },
        "SWITCH_ON_COOLDOWN",
    );
}
