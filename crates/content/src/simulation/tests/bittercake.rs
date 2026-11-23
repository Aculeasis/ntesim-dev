use super::*;

fn adler_request() -> GameScenarioRequest {
    let mut request = sample_s1_request();
    request.actors[0].character_id = "character:1033".into();
    request.actors[0].talent_levels = Some([1; 3]);
    request.actors[0].weapon_id = "weapon:fork_BitterCake".into();
    request.actors[0].gear_procs = vec![GearProcConfig::BitterCake {
        def_bonus: Ratio {
            numerator: 13,
            denominator: 50,
        },
        duration_us: 10_000_000,
        cooldown_us: 20_000_000,
    }];
    request.actors[0].initial_hp = 1000;
    request.actors[0].initial_hp_max = 1000;
    request.actors[0].static_stats.base_def = Some(1000);
    request.actors[0].static_stats.def_percent = Ratio::zero();
    request.actors[0].static_stats.def_flat = Ratio {
        numerator: 240,
        denominator: 1,
    };
    request.actors[0].static_stats.def = 1240;
    request.actors[0].static_stats.crit_rate = Ratio::zero();
    request.actors[0].static_stats.crit_damage_bonus = Ratio::zero();
    request.actors[0].static_stats.damage_bonus = Ratio::zero();
    request.actors[0].static_stats.elemental_bonus = Ratio::zero();
    request.actors[0].static_stats.elemental_bonuses.clear();
    request.targets[0].defense = 0;
    request.targets[0].resistance = Ratio::zero();
    request.action_metadata.clear();
    request.model_assumptions.insert(
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_ID
            .description()
            .into(),
    );
    request.model_assumptions.insert(
        crate::characters::packet_semantics::ADLER_SKILL_DOT_PACKET_HYPOTHESIS_ID.into(),
        crate::characters::packet_semantics::ADLER_SKILL_DOT_PACKET_HYPOTHESIS_ID
            .description()
            .into(),
    );
    request.model_assumptions.insert(
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    request.relative_duration_us = 22_000_000;
    request.actions.clear();
    request
}

fn adler_skill(time: u64) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us: time,
        actor_index: 0,
        ability_id: "GA_Adler_Skill".into(),
        array_index: 0,
        damage_effect_id: "GE_Player_Adler_Skill2_Damage".into(),
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Def,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 0,
    }
}

fn adler_melee(time: u64) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us: time,
        actor_index: 0,
        ability_id: "GA_Adler_Melee".into(),
        array_index: 0,
        damage_effect_id: "GE_Player_Adler_Melee1_Damage".into(),
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 0,
    }
}

fn exact(value: &serde_json::Value) -> f64 {
    serde_json::from_value::<crate::simulation::ExactFraction>(value.clone())
        .unwrap()
        .to_f64()
}

#[test]
fn bittercake_damage_proc_has_native_def_oracle_expiry_and_exact_cooldown() {
    let mut request = adler_request();
    request.incoming_damage_events = vec![
        crate::simulation::GameActorDamageEvent {
            timestamp_us: 0,
            actor_index: 0,
            delivered_damage: 1,
        },
        crate::simulation::GameActorDamageEvent {
            timestamp_us: 1_000_000,
            actor_index: 0,
            delivered_damage: 1,
        },
        crate::simulation::GameActorDamageEvent {
            timestamp_us: 20_000_000,
            actor_index: 0,
            delivered_damage: 1,
        },
    ];
    request.actions = vec![
        adler_skill(1_000_000),
        adler_skill(10_000_000),
        adler_skill(20_000_001),
    ];

    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    let encoded = serde_json::to_value(&request).unwrap();
    assert_eq!(
        encoded["incoming_damage_events"],
        serde_json::to_value(&request.incoming_damage_events).unwrap()
    );
    let decoded: GameScenarioRequest = serde_json::from_value(encoded).unwrap();
    let replay = run_game_scenario(&decoded, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        replay["incoming_damage_receipts"],
        report["incoming_damage_receipts"]
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(
        hits.iter()
            .filter(|hit| hit["damage_effect_id"] == "GE_Player_Adler_Skill2_Damage")
            .map(|hit| exact(&hit["non_crit_damage"]))
            .collect::<Vec<_>>(),
        [1050.0, 868.0, 1050.0]
    );
    let receipts = report["incoming_damage_receipts"].as_array().unwrap();
    assert_eq!(receipts.len(), 3);
    assert_eq!(receipts[0]["actual_hp_loss"], 1);
    assert_eq!(receipts[0]["overflow"], 0);
    assert_eq!(receipts[0]["hp_after"], 999);
    assert_eq!(receipts[0]["event_index"], 0);
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "bittercake_defense")
            .count(),
        2
    );
    assert_eq!(
        report["resolved_model_assumptions"][crate::weapons::bittercake::HYPOTHESIS_ID.as_str()],
        crate::weapons::bittercake::HYPOTHESIS_ID.description()
    );
}

#[test]
fn bittercake_damage_at_an_action_timestamp_resolves_before_the_new_hit() {
    let mut request = adler_request();
    request.actions = vec![adler_skill(1_000_000)];
    request.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
        timestamp_us: 1_000_000,
        actor_index: 0,
        delivered_damage: 1,
    }];
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(exact(&report["hit_receipts"][0]["non_crit_damage"]), 1050.0);
    assert_eq!(report["incoming_damage_receipts"][0]["event_index"], 0);
}

#[test]
fn bittercake_triggers_for_off_field_wearer_only_and_uses_settled_loss() {
    let mut request = adler_request();
    let mut ally = request.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.weapon_id = "weapon:none".into();
    ally.gear_procs.clear();
    ally.initial_hp = 100;
    ally.initial_hp_max = 100;
    request.actors.push(ally);
    request.actions = vec![
        adler_melee(3),
        GameActionCommand::SwitchActor {
            timestamp_us: 10,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 2_000_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        adler_skill(3_000_000),
        adler_skill(21_000_001),
    ];
    request.incoming_damage_events = vec![
        crate::simulation::GameActorDamageEvent {
            timestamp_us: 1_000_000,
            actor_index: 0,
            delivered_damage: 50,
        },
        crate::simulation::GameActorDamageEvent {
            timestamp_us: 2_000_000,
            actor_index: 1,
            delivered_damage: 100,
        },
        crate::simulation::GameActorDamageEvent {
            timestamp_us: 21_000_000,
            actor_index: 0,
            delivered_damage: u64::MAX,
        },
    ];

    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    let receipts = report["incoming_damage_receipts"].as_array().unwrap();
    assert_eq!(receipts[0]["actual_hp_loss"], 50);
    assert_eq!(receipts[1]["actual_hp_loss"], 100);
    assert_eq!(receipts[1]["defeated"], true);
    assert_eq!(receipts[2]["actual_hp_loss"], 950);
    assert!(receipts[2].get("absorbed_by_shields").is_none());
    assert!(receipts[2].get("absorbed_by_effects").is_none());
    assert_eq!(receipts[2]["overflow"].as_u64().unwrap(), u64::MAX - 950);
    assert_eq!(receipts[2]["defeated"], true);
    assert_eq!(exact(&report["hit_receipts"][1]["non_crit_damage"]), 1071.0);
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "bittercake_defense")
            .count(),
        2
    );
    assert_eq!(report["hits_settled"], 14);
    assert_eq!(report["rejected_action"]["command_index"], 4);
}

#[test]
fn bittercake_rejects_malformed_damage_events_and_does_not_trigger_on_zero_or_dead() {
    let mut request = adler_request();
    request.actors[0].initial_hp = 0;
    request.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
        timestamp_us: 0,
        actor_index: 0,
        delivered_damage: 1,
    }];
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["incoming_damage_receipts"][0]["actual_hp_loss"], 0);
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| event["event_kind"] != "bittercake_defense")
    );

    let mut zero = adler_request();
    zero.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
        timestamp_us: 0,
        actor_index: 0,
        delivered_damage: 0,
    }];
    assert_eq!(
        run_game_scenario(&zero, &ExecutionLimits::default()).unwrap()["incoming_damage_receipts"]
            [0]["actual_hp_loss"],
        0
    );

    for events in [
        vec![crate::simulation::GameActorDamageEvent {
            timestamp_us: ExecutionLimits::default().max_horizon_us + 1,
            actor_index: 0,
            delivered_damage: 1,
        }],
        vec![crate::simulation::GameActorDamageEvent {
            timestamp_us: 0,
            actor_index: 1,
            delivered_damage: 1,
        }],
        vec![
            crate::simulation::GameActorDamageEvent {
                timestamp_us: 2,
                actor_index: 0,
                delivered_damage: 1,
            },
            crate::simulation::GameActorDamageEvent {
                timestamp_us: 1,
                actor_index: 0,
                delivered_damage: 1,
            },
        ],
    ] {
        let mut invalid = adler_request();
        invalid.incoming_damage_events = events;
        assert!(run_game_scenario(&invalid, &ExecutionLimits::default()).is_err());
    }
    let mut overflow = adler_request();
    overflow.relative_duration_us = u64::MAX;
    overflow.absolute_limit_us = overflow.relative_duration_us;
    overflow.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
        timestamp_us: u64::MAX - 1,
        actor_index: 0,
        delivered_damage: 1,
    }];
    assert!(run_game_scenario(&overflow, &ExecutionLimits::default()).is_err());
    let mut malformed = serde_json::to_value(adler_request()).unwrap();
    malformed["incoming_damage_events"] = serde_json::json!([{
        "timestamp_us": 0,
        "actor_index": 0,
        "delivered_damage": -1
    }]);
    assert!(serde_json::from_value::<GameScenarioRequest>(malformed).is_err());
}

#[test]
fn fatal_incoming_damage_cancels_admitted_direct_hits_and_allows_switching_away() {
    for damage_time in [9, 10] {
        let mut request = adler_request();
        request.actors[0].initial_hp = 10;
        request.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
            timestamp_us: damage_time,
            actor_index: 0,
            delivered_damage: 10,
        }];
        let mut delayed = adler_melee(0);
        if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut delayed {
            *hit_delay_us = 10;
        }
        request.actions = vec![delayed];
        let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 0);
        assert!(report["trace"].as_array().unwrap().iter().any(|event| {
            event["event_kind"] == "hit_owner_unavailable" && event["from_actor"] == 0
        }));
    }

    let mut request = adler_request();
    request.actors[0].initial_hp = 10;
    let mut ally = request.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.weapon_id = "weapon:none".into();
    ally.gear_procs.clear();
    request.actors.push(ally);
    request.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
        timestamp_us: 0,
        actor_index: 0,
        delivered_damage: 10,
    }];
    request.actions = vec![
        GameActionCommand::SwitchActor {
            timestamp_us: 1,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 2,
            target_actor_index: 0,
            cooldown_us: 0,
        },
    ];
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["final_buff_state"]["active_actor_index"], 1);
    assert_eq!(report["rejected_action"]["command_index"], 1);
    assert!(
        report["halt_diagnostic"]
            .as_str()
            .unwrap()
            .contains("cannot switch to actor 0 at zero HP")
    );
}

#[test]
fn same_time_action_admission_precedes_damage_but_its_hit_is_cancelled_after_death() {
    let mut request = adler_request();
    request.actors[0].initial_hp = 10;
    request.actions = vec![adler_skill(10)];
    request.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
        timestamp_us: 10,
        actor_index: 0,
        delivered_damage: 10,
    }];
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    let trace = report["trace"].as_array().unwrap();
    let admitted = trace
        .iter()
        .position(|event| event["event_kind"] == "invoke_ability")
        .unwrap();
    let damage = trace
        .iter()
        .position(|event| event["event_kind"] == "actor_damage")
        .unwrap();
    let cancelled = trace
        .iter()
        .position(|event| event["event_kind"] == "hit_owner_unavailable")
        .unwrap();
    assert!(admitted < damage && damage < cancelled);
    assert_eq!(report["hits_settled"], 0);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
}

#[test]
fn generic_incoming_damage_requires_a_hypothesis_and_respects_horizon_and_event_limits() {
    let mut request = adler_request();
    request.actors[0].weapon_id = "weapon:none".into();
    request.actors[0].gear_procs.clear();
    request.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
        timestamp_us: request.relative_duration_us,
        actor_index: 0,
        delivered_damage: 4,
    }];
    let mut no_assumption = request.clone();
    no_assumption
        .model_assumptions
        .remove(crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.as_str());
    assert!(
        run_game_scenario(&no_assumption, &ExecutionLimits::default())
            .unwrap_err()
            .contains("require accepted model assumption")
    );

    let exact_horizon = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(exact_horizon["execution_completed"], true);
    assert_eq!(
        exact_horizon["incoming_damage_receipts"][0]["actual_hp_loss"],
        4
    );
    assert_eq!(
        exact_horizon["resolved_model_assumptions"]
            [crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.as_str()],
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.description()
    );

    let mut horizon_pending = request.clone();
    horizon_pending.relative_duration_us = 10;
    horizon_pending.incoming_damage_events.clear();
    horizon_pending.actions = vec![adler_melee(0)];
    if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut horizon_pending.actions[0] {
        *hit_delay_us = 11;
    }
    let horizon_report = run_game_scenario(&horizon_pending, &ExecutionLimits::default()).unwrap();
    assert_eq!(horizon_report["execution_completed"], true);
    assert_eq!(horizon_report["pending_events"], 1);
    assert!(horizon_report["halt_diagnostic"].is_null());

    let mut partial = request;
    partial.relative_duration_us = 10;
    partial.incoming_damage_events[0].timestamp_us = 1;
    partial.actions = vec![adler_melee(0)];
    if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut partial.actions[0] {
        *hit_delay_us = 5;
    }
    let report = run_game_scenario(
        &partial,
        &ExecutionLimits {
            max_events: 2,
            ..ExecutionLimits::default()
        },
    )
    .unwrap();
    assert_eq!(report["execution_completed"], false);
    assert_eq!(report["pending_events"], 1);
    assert_eq!(
        report["incoming_damage_receipts"].as_array().unwrap().len(),
        1
    );
}
