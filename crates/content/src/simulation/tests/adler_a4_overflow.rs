use super::adler_actions::{def_action, make_action, request};
use super::adler_shields::{settle_adler_callback, settle_incoming_event};
use super::*;

#[test]
fn adler_blessing_a4_overflow_absorbs_after_shields_before_hp() {
    let mut a4 = request();
    a4.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(4);
    a4.actors[0].static_stats.base_def = Some(1000);
    a4.actors[0].static_stats.def = 1000;
    a4.relative_duration_us = 2_000;
    a4.actions = vec![def_action(
        0,
        "GA_Adler_Skill",
        "GE_Player_Adler_Skill2_Damage",
        Ratio {
            numerator: 1,
            denominator: 5,
        },
    )];
    a4.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
        timestamp_us: 1_000,
        actor_index: 0,
        delivered_damage: 2_728,
    }];
    a4.model_assumptions.insert(
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&a4, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        run_game_scenario(&a4, &ExecutionLimits::default()).unwrap(),
        report
    );
    assert_eq!(
        report["incoming_damage_receipts"][0]["absorbed_by_shields"][0]["amount"],
        2328
    );
    assert_eq!(
        report["incoming_damage_receipts"][0]["absorbed_by_effects"][0]["amount"],
        400
    );
    assert_eq!(report["incoming_damage_receipts"][0]["actual_hp_loss"], 0);
    assert_eq!(
        report["resolved_model_assumptions"]
            [crate::characters::adler::skill::A4_OVERFLOW_HYPOTHESIS_ID.as_str()],
        crate::characters::adler::skill::A4_OVERFLOW_HYPOTHESIS_ID.description()
    );
    let mut followup = a4.clone();
    followup
        .incoming_damage_events
        .push(crate::simulation::GameActorDamageEvent {
            timestamp_us: 1_100,
            actor_index: 0,
            delivered_damage: 100,
        });
    let followup_report = run_game_scenario(&followup, &ExecutionLimits::default()).unwrap();
    assert!(
        followup_report["incoming_damage_receipts"][1]
            .get("absorbed_by_effects")
            .is_none()
    );
    assert_eq!(
        followup_report["incoming_damage_receipts"][1]["actual_hp_loss"],
        100
    );

    let mut hp_loss = a4.clone();
    hp_loss.incoming_damage_events[0].delivered_damage = 3_000;
    let hp_report = run_game_scenario(&hp_loss, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        hp_report["incoming_damage_receipts"][0]["absorbed_by_effects"][0]["amount"],
        500
    );
    assert_eq!(
        hp_report["incoming_damage_receipts"][0]["actual_hp_loss"],
        172
    );

    let mut rounded_cap = a4.clone();
    rounded_cap.actors[0].static_stats.base_def = Some(1003);
    rounded_cap.actors[0].static_stats.def = 1003;
    rounded_cap.incoming_damage_events[0].delivered_damage = 2833;
    let rounded_report = run_game_scenario(&rounded_cap, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        rounded_report["incoming_damage_receipts"][0]["absorbed_by_effects"][0]["amount"],
        502
    );

    let mut exact = a4.clone();
    exact.incoming_damage_events[0].delivered_damage = 2328;
    let exact_report = run_game_scenario(&exact, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        exact_report["incoming_damage_receipts"][0]["absorbed_by_effects"][0]["amount"],
        0
    );
    assert_eq!(
        exact_report["incoming_damage_receipts"][0]["actual_hp_loss"],
        0
    );

    let mut expired_a4 = request();
    expired_a4.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(4);
    expired_a4.actors[0].static_stats.base_def = Some(1000);
    expired_a4.actors[0].static_stats.def = 1000;
    expired_a4.relative_duration_us = 8_000_001;
    expired_a4.actions = vec![def_action(
        0,
        "GA_Adler_Skill",
        "GE_Player_Adler_Skill2_Damage",
        Ratio {
            numerator: 1,
            denominator: 5,
        },
    )];
    expired_a4.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
        timestamp_us: 8_000_000,
        actor_index: 0,
        delivered_damage: 1_000_000,
    }];
    expired_a4.model_assumptions.insert(
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let expired_report = run_game_scenario(&expired_a4, &ExecutionLimits::default()).unwrap();
    assert_eq!(expired_report["final_actor_states"][0]["hp"], 0);
    assert!(
        expired_report["incoming_damage_receipts"][0]
            .get("absorbed_by_shields")
            .and_then(serde_json::Value::as_array)
            .is_none_or(Vec::is_empty)
    );
    assert!(
        expired_report["resolved_model_assumptions"]
            .get(crate::characters::adler::skill::A4_OVERFLOW_HYPOTHESIS_ID.as_str())
            .is_none()
    );
}

#[test]
fn adler_a4_overflow_cap_uses_additive_live_source_def_and_living_a4_eligibility() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(4);
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def_percent = Ratio {
        numerator: 1,
        denominator: 10,
    };
    req.actors[0].static_stats.def_flat = Ratio {
        numerator: 20,
        denominator: 1,
    };
    req.actors[0].static_stats.def = 1120;
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    state
        .buffs
        .apply_buff(
            ntesim_core::buffs::dispatch::BuffDefinition {
                stacking_scope: ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                semantic_tags: Default::default(),
                source_lifetime: ntesim_core::buffs::dispatch::BuffSourceLifetime::Snapshot,
                id: "a4_live_def_test".into(),
                source: "test".into(),
                recipient: ntesim_core::buffs::dispatch::BuffRecipient::TeamAll,
                duration_us: None,
                refresh_policy: ntesim_core::buffs::dispatch::RefreshPolicy::Disallow,
                max_stacks: 1,
                decay_policy: ntesim_core::buffs::dispatch::DecayPolicy::AllAtOnce,
                stat_modifiers: std::collections::BTreeMap::from([
                    (
                        "def_percent".into(),
                        Ratio {
                            numerator: 1,
                            denominator: 5,
                        },
                    ),
                    (
                        "def_add".into(),
                        Ratio {
                            numerator: 50,
                            denominator: 1,
                        },
                    ),
                ]),
            },
            0,
            0,
        )
        .unwrap();
    assert_eq!(
        crate::characters::adler::skill::a4_overflow_cap(&req, &state, 0).unwrap(),
        Some(685)
    );
    state.actors[0].hp = 0;
    assert_eq!(
        crate::characters::adler::skill::a4_overflow_cap(&req, &state, 0).unwrap(),
        None
    );
    state.actors[0].hp = req.actors[0].initial_hp;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(3);
    assert_eq!(
        crate::characters::adler::skill::a4_overflow_cap(&req, &state, 0).unwrap(),
        None
    );
}

#[test]
fn adler_a4_incoming_uses_source_def_after_ordinary_shields_for_foreign_recipients() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(4);
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def = 1000;
    let mut recipient = req.actors[0].clone();
    recipient.slot = 1;
    recipient.character_id = "character:1019".into();
    recipient.weapon_id.clear();
    recipient.gear_procs.clear();
    recipient.set_id = None;
    recipient.static_stats.base_def = Some(3000);
    recipient.static_stats.def = 3000;
    req.actors.push(recipient);

    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    let recipient_hp_max = state.actors[1].hp_max;
    crate::simulation::state::replace_actor_shield(
        &mut state.actors[1],
        recipient_hp_max,
        0,
        "ordinary:1",
        "ordinary",
        100,
        8_000_000,
    )
    .unwrap();
    settle_adler_callback(&req, &mut state, 0, 1, 1940);
    let (receipt, _) = settle_incoming_event(&req, &mut state, 0, 1, 3_228);
    assert_eq!(
        receipt
            .absorbed_by_shields
            .iter()
            .map(|shield| (shield.shield.as_str(), shield.amount))
            .collect::<Vec<_>>(),
        [("ordinary:1", 100), ("adler.blessing:0", 2328)]
    );
    assert_eq!(receipt.absorbed_by_effects[0].source_actor_index, 0);
    assert_eq!(receipt.absorbed_by_effects[0].amount, 500);
    assert_eq!(receipt.actual_hp_loss, 300);
    assert_eq!(receipt.overflow, 0);
}

#[test]
fn adler_a4_sources_absorb_in_blessing_break_order() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(4);
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def = 1000;
    let mut second_source = req.actors[0].clone();
    second_source.slot = 1;
    second_source.static_stats.base_def = Some(2000);
    second_source.static_stats.def = 2000;
    let mut recipient = req.actors[0].clone();
    recipient.slot = 2;
    recipient.character_id = "character:1019".into();
    recipient.weapon_id.clear();
    recipient.gear_procs.clear();
    recipient.set_id = None;
    req.actors.extend([second_source, recipient]);
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    settle_adler_callback(&req, &mut state, 0, 2, 1940);
    settle_adler_callback(&req, &mut state, 1, 2, 3880);

    let (receipt, _) = settle_incoming_event(&req, &mut state, 0, 2, 7_684);
    assert_eq!(receipt.absorbed_by_shields.len(), 2);
    assert_eq!(receipt.absorbed_by_shields[0].shield, "adler.blessing:0");
    assert_eq!(receipt.absorbed_by_shields[1].shield, "adler.blessing:1");
    assert_eq!(
        receipt
            .absorbed_by_effects
            .iter()
            .map(|entry| (entry.source_actor_index, entry.amount))
            .collect::<Vec<_>>(),
        [(0, 500), (1, 200)]
    );
    assert_eq!(receipt.actual_hp_loss, 0);
}

#[test]
fn adler_a4_samples_live_def_at_settlement_without_changing_shield_capacity() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(4);
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def = 1000;
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    settle_adler_callback(&req, &mut state, 0, 0, 1940);
    assert_eq!(state.actors[0].shields[0].max_hp, 2328);
    state
        .buffs
        .apply_buff(
            ntesim_core::buffs::dispatch::BuffDefinition {
                stacking_scope: ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                semantic_tags: Default::default(),
                source_lifetime: ntesim_core::buffs::dispatch::BuffSourceLifetime::Snapshot,
                id: "a4_live_def_test".into(),
                source: "test".into(),
                recipient: ntesim_core::buffs::dispatch::BuffRecipient::TeamAll,
                duration_us: None,
                refresh_policy: ntesim_core::buffs::dispatch::RefreshPolicy::Disallow,
                max_stacks: 1,
                decay_policy: ntesim_core::buffs::dispatch::DecayPolicy::AllAtOnce,
                stat_modifiers: std::collections::BTreeMap::from([
                    (
                        "def_percent".into(),
                        Ratio {
                            numerator: 1,
                            denominator: 5,
                        },
                    ),
                    (
                        "def_add".into(),
                        Ratio {
                            numerator: 50,
                            denominator: 1,
                        },
                    ),
                ]),
            },
            0,
            0,
        )
        .unwrap();
    let (receipt, _) = settle_incoming_event(&req, &mut state, 0, 0, 2_953);
    assert_eq!(receipt.absorbed_by_effects[0].amount, 625);
    assert_eq!(receipt.actual_hp_loss, 0);
    assert!(state.actors[0].shields.is_empty());
}

#[test]
fn adler_a4_dead_sources_keep_shield_capacity_but_do_not_absorb_overflow() {
    for (awakening, kill_source) in [(4, true), (3, false)] {
        let mut req = request();
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actors[0].static_stats.base_def = Some(1000);
        req.actors[0].static_stats.def = 1000;
        let mut recipient = req.actors[0].clone();
        recipient.slot = 1;
        recipient.character_id = "character:1019".into();
        recipient.weapon_id.clear();
        recipient.gear_procs.clear();
        recipient.set_id = None;
        req.actors.push(recipient);
        let mut state = crate::simulation::state::GameSimulatorState::new(&req);
        settle_adler_callback(&req, &mut state, 0, 1, 1940);
        if kill_source {
            state.actors[0].hp = 0;
        }
        let (receipt, _) = settle_incoming_event(&req, &mut state, 0, 1, 2_928);
        assert_eq!(receipt.absorbed_by_shields[0].amount, 2328);
        assert!(receipt.absorbed_by_effects.is_empty());
        assert_eq!(receipt.actual_hp_loss, 600);
    }
}

#[test]
fn adler_a4_lethal_overflow_conserves_hp_and_overkill() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(4);
    req.actors[0].initial_hp = 100;
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def = 1000;
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    settle_adler_callback(&req, &mut state, 0, 0, 1940);
    let (receipt, _) = settle_incoming_event(&req, &mut state, 0, 0, 10_000);
    assert_eq!(receipt.absorbed_by_shields[0].amount, 2328);
    assert_eq!(receipt.absorbed_by_effects[0].amount, 500);
    assert_eq!(receipt.actual_hp_loss, 100);
    assert_eq!(receipt.overflow, 7_072);
    assert_eq!(
        receipt
            .absorbed_by_shields
            .iter()
            .map(|item| item.amount)
            .sum::<u64>()
            + receipt
                .absorbed_by_effects
                .iter()
                .map(|item| item.amount)
                .sum::<u64>()
            + receipt.actual_hp_loss
            + receipt.overflow,
        10_000
    );
    assert!(receipt.defeated);
}

#[test]
fn adler_a4_hp_loss_proc_runs_only_for_damage_left_after_overflow() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(4);
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def = 1000;
    req.actors[0].weapon_id = "weapon:fork_bittercake".into();
    req.actors[0].gear_procs = vec![crate::simulation::GearProcConfig::BitterCake {
        def_bonus: Ratio {
            numerator: 13,
            denominator: 50,
        },
        duration_us: 10_000_000,
        cooldown_us: 20_000_000,
    }];
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    settle_adler_callback(&req, &mut state, 0, 0, 1940);
    let (absorbed, absorbed_trace) = settle_incoming_event(&req, &mut state, 0, 0, 2_828);
    assert_eq!(absorbed.actual_hp_loss, 0);
    assert!(
        absorbed_trace
            .iter()
            .all(|event| event.event_kind != "bittercake_defense")
    );
    let (hp_loss, hp_loss_trace) = settle_incoming_event(&req, &mut state, 1, 0, 1);
    assert_eq!(hp_loss.actual_hp_loss, 1);
    assert_eq!(
        hp_loss_trace
            .iter()
            .filter(|event| event.event_kind == "bittercake_defense")
            .count(),
        1
    );
}

#[test]
fn snowman_no_longer_sees_blessing_after_a4_prevents_hp_loss() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(4);
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 2000,
        denominator: 1,
    };
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def = 1000;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actors[0].weapon_id = "weapon:fork_snowman".into();
    req.actors[0].gear_procs = vec![crate::simulation::GearProcConfig::SnowmanShieldAtk {
        atk_bonus: Ratio {
            numerator: 9,
            denominator: 50,
        },
    }];
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.actions = vec![
        def_action(
            0,
            "GA_Adler_Skill",
            "GE_Player_Adler_Skill2_Damage",
            Ratio {
                numerator: 1,
                denominator: 5,
            },
        ),
        make_action(
            2_000,
            0,
            "GA_Adler_Melee",
            "GE_Player_Adler_Melee1_Damage",
            0,
            0,
        ),
    ];
    req.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
        timestamp_us: 1_000,
        actor_index: 0,
        delivered_damage: 2_828,
    }];
    req.model_assumptions.insert(
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.relative_duration_us = 3_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["incoming_damage_receipts"][0]["actual_hp_loss"], 0);
    assert_eq!(
        report["incoming_damage_receipts"][0]["absorbed_by_effects"][0]["amount"],
        500
    );
    let mut baseline = req.clone();
    baseline.actors[0].weapon_id.clear();
    baseline.actors[0].gear_procs.clear();
    let baseline_report = run_game_scenario(&baseline, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["hit_receipts"][1]["non_crit_damage"],
        baseline_report["hit_receipts"][1]["non_crit_damage"]
    );
}
