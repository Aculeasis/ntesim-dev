use super::*;

#[test]
fn fireflies_equipped_projection_overwrites_or_inserts_source_bound_parameters() {
    let (tables, coverage, mut request) = fireflies_fixture();
    for insert in [false, true] {
        if insert {
            request.scenario.actors[0]
                .gear_procs
                .retain(|p| !matches!(p, GearProcConfig::FirefliesCritDamage { .. }));
        }
        let compiled = compile(&tables, &coverage, &request, &ExecutionLimits::default()).unwrap();
        let receipt = &compiled.receipt["gear_receipts"][1];
        assert_eq!(receipt["declared_set_count"], 4);
        assert_eq!(
            receipt["compiled_proc"],
            json!({"kind":"fireflies_crit_damage","crit_damage_per_stack":"2/25","max_stacks":7,"duration_us":10_000_000})
        );
        assert_eq!(receipt["replaced_proc"].is_null(), insert);
        assert!(
            compiled
                .scenario
                .model_assumptions
                .contains_key("fireflies.solo_proc_v2")
        );
        assert!(run_game_scenario(&compiled.scenario, &ExecutionLimits::default()).unwrap()["execution_completed"].as_bool().unwrap());
    }
}

#[test]
fn fireflies_solo_projection_accepts_all_in_range_dummies_without_broadcasting_attacks() {
    let (tables, coverage, mut request) = fireflies_fixture();
    let mut extra = request.scenario.targets[0].clone();
    extra.target_index = 1;
    request.scenario.targets.push(extra);
    request.scenario.actions.retain(|action| {
        matches!(
            action,
            GameActionCommand::InvokeAbility { ability_id, .. } if ability_id == "GA_Mint019_Melee"
        )
    });
    let compiled = compile(&tables, &coverage, &request, &ExecutionLimits::default()).unwrap();
    let report = run_game_scenario(&compiled.scenario, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true);
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|hit| hit["target_index"] == 0)
    );
    assert_eq!(report["final_target_states"][1]["hp_current"], 10_000_000);
    assert_eq!(
        report["resolved_model_assumptions"]
            [crate::sets::fireflies::hit::ALL_IN_RANGE_HYPOTHESIS_ID.as_str()],
        crate::sets::fireflies::hit::ALL_IN_RANGE_HYPOTHESIS_ID.description()
    );
}

#[test]
fn native_fireflies_team_builds_bind_every_wearer_and_buff_later_off_field_damage() {
    let (tables, coverage, mut request) = fireflies_fixture();
    request.gear_projection = None;
    request.scenario.actors[0].gear_procs.clear();
    let mut ally = request.scenario.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1010".into();
    ally.weapon_id = "weapon:fork_wushoutieyu".into();
    ally.weapon_tier = 1;
    request.builds.push(ActorBuild {
        slot: 1,
        build: legal_native_build(
            &tables,
            &coverage,
            &ally.character_id,
            &ally.weapon_id,
            "set:Suit2",
        ),
        purchased_skill_levels: None,
    });
    let ally_stats = project(
        &equipment::compile(&tables, &request.builds[1].build).unwrap(),
        "DamageUpNatureBase",
    )
    .unwrap();
    ally.initial_hp = ally_stats.hp;
    ally.initial_hp_max = ally_stats.hp;
    request.scenario.actors.push(ally);
    request.scenario.targets[0].defense = 0;
    request.scenario.targets[0].resistance = Ratio::zero();
    let mut extra = request.scenario.targets[0].clone();
    extra.target_index = 1;
    request.scenario.targets.push(extra);
    let mut first = request.scenario.actions[0].clone();
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        hit_delay_us,
        recovery_us,
        cooldown_us,
        ..
    } = &mut first
    {
        *timestamp_us = 0;
        *hit_delay_us = 1_000_000;
        *recovery_us = 0;
        *cooldown_us = 0;
    }
    let mut next = first.clone();
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index,
        ability_id,
        damage_effect_id,
        hit_delay_us,
        ..
    } = &mut next
    {
        *timestamp_us = 2_000_000;
        *actor_index = 1;
        *ability_id = "GA_Nanally_Melee".into();
        *damage_effect_id = "GE_Player_Nanally_Melee1_Damage".into();
        *hit_delay_us = 0;
    }
    request.scenario.actions = vec![
        first,
        GameActionCommand::SwitchActor {
            timestamp_us: 500_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        next,
    ];
    request.scenario.relative_duration_us = 3_000_000;
    let compiled = compile(&tables, &coverage, &request, &ExecutionLimits::default()).unwrap();
    let expected = json!({"kind":"fireflies_crit_damage","crit_damage_per_stack":"2/25","max_stacks":7,"duration_us":10_000_000});
    for index in 0..2 {
        assert_eq!(
            compiled.receipt["actor_build_receipts"][index]["native_fireflies_projection"]["compiled_proc"],
            expected
        );
    }
    let report = run_game_scenario(&compiled.scenario, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true);
    let hits = report["hit_receipts"].as_array().unwrap();
    let ally_hit = hits.iter().find(|hit| hit["actor_index"] == 1).unwrap();
    // Mint's two coincident packets grant exactly two stacks to each wearer;
    // the later Nanally packet gets +16 percentage points of CRIT DMG.
    let noncrit: ExactFraction =
        serde_json::from_value(ally_hit["non_crit_damage"].clone()).unwrap();
    let expected_crit = noncrit
        .mul(
            ExactFraction::new(1, 1)
                .unwrap()
                .add(
                    ExactFraction::from_ratio(
                        compiled.scenario.actors[1].static_stats.crit_damage_bonus,
                    )
                    .unwrap(),
                )
                .unwrap()
                .add(ExactFraction::new(4, 25).unwrap())
                .unwrap(),
        )
        .unwrap();
    assert_eq!(
        ally_hit["crit_damage"],
        serde_json::to_value(expected_crit).unwrap()
    );
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_kind"] == "fireflies_crit_stack"
                && event["time_us"] == 1_000_000
                && event["active_actor"] == 1)
    );
    for state in report["final_actor_states"].as_array().unwrap() {
        assert_eq!(state["fireflies_stacks"], hits.len());
    }
    // Only the automatic uncapped Support reaches the second dummy.
    let secondary_loss: u64 = hits
        .iter()
        .filter(|hit| hit["target_index"] == 1)
        .map(|hit| hit["applied_hp_loss"].as_u64().unwrap())
        .sum();
    assert!(secondary_loss > 0);
    assert_eq!(
        report["final_target_states"][1]["hp_current"],
        10_000_000 - secondary_loss
    );

    let mut duplicate = request.clone();
    let proc: GearProcConfig = serde_json::from_value(expected).unwrap();
    duplicate.scenario.actors[0].gear_procs = vec![proc.clone(), proc];
    assert!(
        compile(&tables, &coverage, &duplicate, &ExecutionLimits::default())
            .unwrap_err()
            .contains("duplicate Fireflies")
    );
}
