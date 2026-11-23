use super::*;
use serde_json::json;

use super::adler_actions::{def_action, make_action, request};

pub(super) fn settle_adler_callback(
    request: &GameScenarioRequest,
    state: &mut crate::simulation::state::GameSimulatorState,
    granter_index: usize,
    recipient_index: usize,
    amount: u64,
) -> Vec<crate::simulation::GameEventReceipt> {
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    crate::callbacks::apply(
        crate::callbacks::ContentEvent::AdlerBlessingShield {
            granter_index,
            recipient_index,
            amount: ExactFraction::new(amount.into(), 1).unwrap(),
            duration_us: if request.actors[granter_index].awakening.has_effect(6) {
                12_000_000
            } else {
                8_000_000
            },
        },
        &mut crate::runtime_api::CallbackContext {
            state,
            request,
            queue: &mut queue,
            sequence: &mut sequence,
            trace: &mut trace,
        },
    )
    .unwrap();
    trace
}

pub(super) fn settle_incoming_event(
    request: &GameScenarioRequest,
    state: &mut crate::simulation::state::GameSimulatorState,
    event_index: usize,
    actor_index: usize,
    delivered_damage: u64,
) -> (
    crate::simulation::GameActorDamageReceipt,
    Vec<crate::simulation::GameEventReceipt>,
) {
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    let receipt = crate::simulation::incoming::settle(
        &mut crate::runtime_api::CallbackContext {
            state,
            request,
            queue: &mut queue,
            sequence: &mut sequence,
            trace: &mut trace,
        },
        event_index,
        actor_index,
        delivered_damage,
    )
    .unwrap();
    (receipt, trace)
}

#[test]
fn adler_blessing_shield_uses_native_level_def_and_additive_efficiency() {
    for (awakening, skill_level, static_bonus, expected) in [
        (0, 1, Ratio::zero(), 1800),
        (
            0,
            1,
            Ratio {
                numerator: 1,
                denominator: 5,
            },
            2160,
        ),
        (
            3,
            1,
            Ratio {
                numerator: 1,
                denominator: 5,
            },
            2716,
        ),
        (0, 8, Ratio::zero(), 3084),
        (3, 8, Ratio::zero(), 3997),
    ] {
        let mut req = request();
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actors[0].talent_levels = Some([1, skill_level, 1]);
        req.actors[0].static_stats.base_def = Some(1000);
        req.actors[0].static_stats.def = 1000;
        req.actors[0].static_stats.shield_bonus = static_bonus;
        req.relative_duration_us = 1_000_000;
        req.actions = vec![def_action(
            0,
            "GA_Adler_Skill",
            "GE_Player_Adler_Skill2_Damage",
            Ratio {
                numerator: 1,
                denominator: 5,
            },
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["execution_completed"], true);
        assert_eq!(
            report["final_actor_states"][0]["active_shields"][0]["max_hp"],
            expected
        );
        assert_eq!(
            support::receipt_assumption(
                &report,
                &report["action_metadata_ledger"][0],
                crate::characters::adler::skill::BLESSING_HYPOTHESIS_ID.as_str()
            ),
            Some(crate::characters::adler::skill::BLESSING_HYPOTHESIS_ID.description())
        );
    }

    let mut a6 = request();
    a6.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    a6.actors[0].static_stats.base_def = Some(1000);
    a6.actors[0].static_stats.def = 1000;
    a6.actions = vec![def_action(
        0,
        "GA_Adler_Skill",
        "GE_Player_Adler_Skill2_Damage",
        Ratio {
            numerator: 1,
            denominator: 5,
        },
    )];
    let metadata = crate::simulation::metadata::resolve_requested_action_packet(
        &a6.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        "character:1033",
        "GA_Adler_Skill",
        "GE_Player_Adler_Skill2_Damage",
        None,
        &a6.model_assumptions,
    )
    .unwrap();
    let mut state = crate::simulation::state::GameSimulatorState::new(&a6);
    let team_hp_snapshot = state
        .actors
        .iter()
        .map(|actor| (actor.hp, actor.hp_max))
        .collect::<Vec<_>>();
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    {
        let mut assumptions = std::collections::BTreeMap::new();
        crate::characters::adler::skill::on_cast(&mut crate::runtime_api::CastContext {
            origins: &mut state.origins,
            command_index: 0,
            native_array_index: Some(0),
            execution_assumptions: &mut assumptions,
            config: &a6.actors[0],
            request: &a6,
            actor_index: 0,
            active_actor_index: 0,
            target_index: 0,
            ability_id: "GA_Adler_Skill",
            action_metadata: &metadata.action,
            packet_metadata: Some(&metadata.packet),
            hit_time_us: 100,
            world_time_us: 0,
            time_us: 0,
            buffs: &mut state.buffs,
            actor: &mut state.actors[0],
            stamina: state.combat_stamina.as_mut(),
            team_hp_snapshot: &team_hp_snapshot,
            targets: &state.targets,
            queue: &mut queue,
            sequence: &mut sequence,
        })
        .unwrap();
    }
    match queue.pop().unwrap().data {
        crate::simulation::events::GameEventData::Content(
            crate::callbacks::ContentEvent::AdlerBlessingShield { duration_us, .. },
        ) => assert_eq!(duration_us, 12_000_000),
        _ => panic!("Adler Skill did not schedule its Blessing callback"),
    }
}

#[test]
fn selected_a3_and_a6_effects_apply_without_implying_resonance_count() {
    for (effect, expected_capacity, expected_expiry) in
        [(3, 2160, 8_000_000), (6, 1800, 12_000_000)]
    {
        let mut req = request();
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([effect]).unwrap();
        req.actors[0].static_stats.base_def = Some(1000);
        req.actors[0].static_stats.def = 1000;
        req.actors[0].talent_levels = Some([1, 1, 1]);
        req.relative_duration_us = 1_000_000;
        req.actions = vec![def_action(
            0,
            "GA_Adler_Skill",
            "GE_Player_Adler_Skill2_Damage",
            Ratio::one(),
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        let shield = report["final_actor_states"][0]["active_shields"][0].clone();
        assert_eq!(shield["max_hp"], expected_capacity, "{report}");
        assert_eq!(shield["expires_at_us"], expected_expiry, "{report}");
    }
}

#[test]
fn adler_blessing_def_is_snapshotted_at_successful_skill_admission() {
    let mut req = request();
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def = 1000;
    req.actions = vec![def_action(
        0,
        "GA_Adler_Skill",
        "GE_Player_Adler_Skill2_Damage",
        Ratio {
            numerator: 1,
            denominator: 5,
        },
    )];
    let metadata = crate::simulation::metadata::resolve_requested_action_packet(
        &req.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        "character:1033",
        "GA_Adler_Skill",
        "GE_Player_Adler_Skill2_Damage",
        None,
        &req.model_assumptions,
    )
    .unwrap();
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    state
        .apply_buff(
            0,
            crate::simulation::state::TimedBuffDeclaration::reset(
                "test.admission_def",
                "test",
                ntesim_core::buffs::dispatch::BuffRecipient::SpecificActor(0),
                1,
                ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                std::collections::BTreeMap::from([
                    ("def_percent".into(), ExactFraction::new(1, 5).unwrap()),
                    ("def_add".into(), ExactFraction::new(50, 1).unwrap()),
                ]),
            ),
        )
        .unwrap();
    let team_hp_snapshot = state
        .actors
        .iter()
        .map(|actor| (actor.hp, actor.hp_max))
        .collect::<Vec<_>>();
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    {
        let mut assumptions = std::collections::BTreeMap::new();
        crate::characters::adler::skill::on_cast(&mut crate::runtime_api::CastContext {
            origins: &mut state.origins,
            command_index: 0,
            native_array_index: Some(0),
            execution_assumptions: &mut assumptions,
            config: &req.actors[0],
            request: &req,
            actor_index: 0,
            active_actor_index: 0,
            target_index: 0,
            ability_id: "GA_Adler_Skill",
            action_metadata: &metadata.action,
            packet_metadata: Some(&metadata.packet),
            hit_time_us: 100,
            world_time_us: 0,
            time_us: 0,
            buffs: &mut state.buffs,
            actor: &mut state.actors[0],
            stamina: state.combat_stamina.as_mut(),
            team_hp_snapshot: &team_hp_snapshot,
            targets: &state.targets,
            queue: &mut queue,
            sequence: &mut sequence,
        })
        .unwrap();
    }
    let event = queue.pop().unwrap();
    let captured_amount = match event.data {
        crate::simulation::events::GameEventData::Content(
            crate::callbacks::ContentEvent::AdlerBlessingShield { amount, .. },
        ) => amount,
        _ => panic!("Adler Skill did not schedule its Blessing callback"),
    };
    state.advance_unpaused_test_clock(2);
    state.buffs.advance_time(2);
    state
        .apply_buff(
            0,
            crate::simulation::state::TimedBuffDeclaration::reset(
                "test.after_admission_def",
                "test",
                ntesim_core::buffs::dispatch::BuffRecipient::SpecificActor(0),
                100,
                ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                std::collections::BTreeMap::from([
                    ("def_percent".into(), ExactFraction::new(1, 2).unwrap()),
                    ("def_add".into(), ExactFraction::new(100, 1).unwrap()),
                ]),
            ),
        )
        .unwrap();
    assert_eq!(captured_amount, ExactFraction::new(2000, 1).unwrap());
    settle_adler_callback(&req, &mut state, 0, 0, captured_amount.round_u64().unwrap());
    assert_eq!(state.actors[0].shields[0].max_hp, 2000);
}

#[test]
fn adler_recast_preserves_other_shields_and_death_lifecycle() {
    let mut req = request();
    let mut recipient = req.actors[0].clone();
    recipient.slot = 1;
    recipient.character_id = "character:1019".into();
    recipient.weapon_id.clear();
    recipient.gear_procs.clear();
    recipient.set_id = None;
    req.actors.push(recipient);
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    let recipient_hp_max = state.actors[1].hp_max;
    crate::simulation::state::replace_actor_shield(
        &mut state.actors[1],
        recipient_hp_max,
        0,
        "other.producer:1",
        "other.producer",
        75,
        8_000_000,
    )
    .unwrap();
    settle_adler_callback(&req, &mut state, 0, 1, 1800);
    state.advance_unpaused_test_clock(1);
    settle_adler_callback(&req, &mut state, 0, 1, 2400);
    assert_eq!(state.actors[1].shields.len(), 2);
    assert!(state.actors[1].shields.iter().any(|shield| {
        shield.id == "other.producer:1" && shield.max_hp == 75 && shield.remaining_hp == 75
    }));
    assert!(state.actors[1].shields.iter().any(|shield| {
        shield.id == "adler.blessing:0" && shield.max_hp == 2400 && shield.remaining_hp == 2400
    }));

    state.actors[0].hp = 0;
    assert!(
        state.actors[1]
            .shields
            .iter()
            .any(|shield| shield.id == "adler.blessing:0" && shield.remaining_hp > 0)
    );
    state.actors[1].hp = 0;
    state.actors[1].expire_shields(1);
    assert!(state.actors[1].shields.is_empty());

    for dead_index in [0, 1] {
        let mut cancelled = crate::simulation::state::GameSimulatorState::new(&req);
        cancelled.actors[dead_index].hp = 0;
        let trace = settle_adler_callback(&req, &mut cancelled, 0, 1, 1800);
        assert!(trace.is_empty());
        assert!(cancelled.actors[1].shields.is_empty());
    }
}

#[test]
fn adler_blessing_shield_rounds_only_after_flat_coefficient_and_efficiency() {
    let mut req = request();
    req.actors[0].static_stats.base_def = Some(1);
    req.actors[0].static_stats.def = 1;
    req.actors[0].static_stats.shield_bonus = Ratio {
        numerator: 1,
        denominator: 3,
    };
    req.relative_duration_us = 1_000_000;
    req.actions = vec![def_action(
        0,
        "GA_Adler_Skill",
        "GE_Player_Adler_Skill2_Damage",
        Ratio {
            numerator: 1,
            denominator: 5,
        },
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["final_actor_states"][0]["active_shields"][0]["max_hp"],
        1334
    );
}

#[test]
fn adler_failed_skill_admission_grants_no_blessing_shield() {
    let mut req = request();
    req.actors[0].initial_energy = 0;
    let mut action = def_action(
        0,
        "GA_Adler_Skill",
        "GE_Player_Adler_Skill2_Damage",
        Ratio {
            numerator: 1,
            denominator: 5,
        },
    );
    if let GameActionCommand::InvokeAbility { energy_cost, .. } = &mut action {
        *energy_cost = 1;
    }
    req.actions = vec![action];
    req.relative_duration_us = 1_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| event["event_kind"] != "adler_blessing_shield")
    );
    assert!(
        report["final_actor_states"][0]
            .get("active_shields")
            .and_then(serde_json::Value::as_array)
            .is_none_or(Vec::is_empty)
    );
}

#[test]
fn adler_blessing_adds_static_and_live_efficiency_once() {
    let mut req = request();
    let mut recipient = req.actors[0].clone();
    recipient.slot = 1;
    recipient.character_id = "character:1019".into();
    recipient.weapon_id.clear();
    recipient.gear_procs.clear();
    recipient.set_id = None;
    recipient.static_stats.shield_bonus = Ratio {
        numerator: 3,
        denominator: 5,
    };
    req.actors.push(recipient);
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def = 1000;
    req.actors[0].static_stats.shield_bonus = Ratio {
        numerator: 1,
        denominator: 5,
    };
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    state
        .apply_buff(
            0,
            crate::simulation::state::TimedBuffDeclaration::reset(
                "test.live_shield_efficiency",
                "test",
                ntesim_core::buffs::dispatch::BuffRecipient::SpecificActor(0),
                1,
                ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                std::collections::BTreeMap::from([(
                    "shield_bonus".into(),
                    ExactFraction::new(1, 5).unwrap(),
                )]),
            ),
        )
        .unwrap();
    state
        .apply_buff(
            1,
            crate::simulation::state::TimedBuffDeclaration::reset(
                "test.recipient_shield_efficiency",
                "test",
                ntesim_core::buffs::dispatch::BuffRecipient::SpecificActor(1),
                1,
                ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                std::collections::BTreeMap::from([(
                    "shield_bonus".into(),
                    ExactFraction::new(1, 2).unwrap(),
                )]),
            ),
        )
        .unwrap();
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    crate::characters::adler::skill::apply_blessing_shield(
        &mut crate::runtime_api::CallbackContext {
            state: &mut state,
            request: &req,
            queue: &mut queue,
            sequence: &mut sequence,
            trace: &mut trace,
        },
        0,
        1,
        ExactFraction::new(1800, 1).unwrap(),
        8_000_000,
    )
    .unwrap();
    assert_eq!(state.actors[1].shields[0].max_hp, 2520);
}

#[test]
fn adler_rainman_shield_uses_additive_grant_snapshot_and_living_wearer() {
    let mut req = request();
    req.actors[0].static_stats.base_hp = Some(1000);
    req.actors[0].initial_hp_max = 1000;
    req.actors[0].initial_hp = 501;
    req.actors[0].weapon_id = "weapon:fork_yuren".into();
    req.actors[0].gear_procs = vec![crate::simulation::GearProcConfig::YurenHp {
        hp_bonus: Ratio {
            numerator: 1,
            denominator: 10,
        },
        shield_bonus: Ratio {
            numerator: 1,
            denominator: 10,
        },
    }];
    req.actors[0].static_stats.shield_bonus = Ratio {
        numerator: 1,
        denominator: 5,
    };
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def = 1000;
    let mut settled_state = crate::simulation::state::GameSimulatorState::new(&req);
    settled_state
        .apply_buff(
            0,
            crate::simulation::state::TimedBuffDeclaration::reset(
                "test.live_shield_efficiency",
                "test",
                ntesim_core::buffs::dispatch::BuffRecipient::SpecificActor(0),
                10,
                ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                std::collections::BTreeMap::from([(
                    "shield_bonus".into(),
                    ExactFraction::new(3, 10).unwrap(),
                )]),
            ),
        )
        .unwrap();
    let trace = settle_adler_callback(&req, &mut settled_state, 0, 0, 1800);
    assert_eq!(settled_state.actors[0].shields[0].max_hp, 2880);
    assert_eq!(trace[0].event_kind, "adler_blessing_shield");
    settled_state.actors[0].hp = 500;
    settled_state.advance_unpaused_test_clock(10);
    settled_state.expire_buffs(&mut Vec::new());
    assert_eq!(settled_state.actors[0].shields[0].max_hp, 2880);

    let mut max_hp_state = crate::simulation::state::GameSimulatorState::new(&req);
    max_hp_state
        .apply_buff(
            0,
            crate::simulation::state::TimedBuffDeclaration::reset(
                crate::sets::hp_suit::BUFF_ID,
                "set:Suit10",
                ntesim_core::buffs::dispatch::BuffRecipient::SelfActor,
                10,
                ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                std::collections::BTreeMap::from([(
                    crate::sets::hp_suit::MAX_HP_BONUS_KEY.into(),
                    ExactFraction::new(1, 5).unwrap(),
                )]),
            ),
        )
        .unwrap();
    max_hp_state
        .apply_buff(
            0,
            crate::simulation::state::TimedBuffDeclaration::reset(
                "test.live_shield_efficiency",
                "test",
                ntesim_core::buffs::dispatch::BuffRecipient::SpecificActor(0),
                10,
                ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                std::collections::BTreeMap::from([(
                    "shield_bonus".into(),
                    ExactFraction::new(3, 10).unwrap(),
                )]),
            ),
        )
        .unwrap();
    assert_eq!(
        crate::simulation::state::effective_hp_max(
            &max_hp_state.buffs,
            max_hp_state.actors[0].hp_max_base,
            &max_hp_state.actors[0].hp_max_static_bonus,
            0,
            max_hp_state.actors[0].hp_max,
        )
        .unwrap(),
        1200
    );
    settle_adler_callback(&req, &mut max_hp_state, 0, 0, 1800);
    assert_eq!(max_hp_state.actors[0].shields[0].max_hp, 2700);
    max_hp_state.advance_unpaused_test_clock(10);
    max_hp_state.expire_buffs(&mut Vec::new());
    assert_eq!(max_hp_state.actors[0].hp, 501);
    assert_eq!(
        crate::simulation::state::effective_hp_max(
            &max_hp_state.buffs,
            max_hp_state.actors[0].hp_max_base,
            &max_hp_state.actors[0].hp_max_static_bonus,
            0,
            max_hp_state.actors[0].hp_max,
        )
        .unwrap(),
        1000
    );
    assert_eq!(max_hp_state.actors[0].shields[0].max_hp, 2700);
    settle_adler_callback(&req, &mut max_hp_state, 0, 0, 1800);
    assert_eq!(max_hp_state.actors[0].shields[0].max_hp, 2340);

    max_hp_state.actors[0].hp = 0;
    max_hp_state.actors[0].shields.clear();
    settle_adler_callback(&req, &mut max_hp_state, 0, 0, 1800);
    assert!(max_hp_state.actors[0].shields.is_empty());
}

#[test]
fn adler_skill_and_snowman_use_settled_shields_through_switch_depletion_and_expiry() {
    let mut req = request();
    let mut teammate = req.actors[0].clone();
    teammate.slot = 1;
    teammate.character_id = "character:1019".into();
    teammate.weapon_id.clear();
    teammate.gear_procs.clear();
    teammate.set_id = None;
    req.actors.push(teammate);
    req.actors[0].weapon_id = "weapon:fork_snowman".into();
    req.actors[0].gear_procs = vec![crate::simulation::GearProcConfig::SnowmanShieldAtk {
        atk_bonus: Ratio {
            numerator: 9,
            denominator: 50,
        },
    }];
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 2000,
        denominator: 1,
    };
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def = 1000;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.active_actor_index = 0;
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.relative_duration_us = 20_000_000;
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
        GameActionCommand::SwitchActor {
            timestamp_us: 1_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 2_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        make_action(
            3_000,
            0,
            "GA_Adler_Melee",
            "GE_Player_Adler_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            8_004_000,
            0,
            "GA_Adler_Melee",
            "GE_Player_Adler_Melee1_Damage",
            0,
            0,
        ),
    ];
    req.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
        timestamp_us: 4_000,
        actor_index: 0,
        delivered_damage: 1800,
    }];
    req.model_assumptions.insert(
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let hits = report["hit_receipts"].as_array().unwrap();
    let normal_hits = hits
        .iter()
        .filter(|hit| hit["ability_id"] == "GA_Adler_Melee")
        .collect::<Vec<_>>();
    assert_eq!(normal_hits.len(), 2);
    assert_eq!(normal_hits[0]["non_crit_damage"], json!("21909/50"));
    assert_eq!(normal_hits[1]["non_crit_damage"], json!("402/1"));
    assert_eq!(
        report["incoming_damage_receipts"][0]["absorbed_by_shields"][0]["amount"],
        1800
    );
    assert!(
        report["final_actor_states"][0]
            .get("active_shields")
            .and_then(serde_json::Value::as_array)
            .is_none_or(Vec::is_empty)
    );
    assert!(
        report["final_actor_states"][1]
            .get("active_shields")
            .and_then(serde_json::Value::as_array)
            .is_none_or(Vec::is_empty)
    );

    let mut expiry = request();
    expiry.actors[0].weapon_id = "weapon:fork_snowman".into();
    expiry.actors[0].gear_procs = vec![crate::simulation::GearProcConfig::SnowmanShieldAtk {
        atk_bonus: Ratio {
            numerator: 9,
            denominator: 50,
        },
    }];
    expiry.actors[0].static_stats.base_atk = 1000;
    expiry.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    expiry.actors[0].static_stats.base_def = Some(1000);
    expiry.actors[0].static_stats.def = 1000;
    expiry.actors[0].static_stats.crit_rate = Ratio::zero();
    expiry.targets[0].defense = 0;
    expiry.targets[0].resistance = Ratio::zero();
    expiry.relative_duration_us = 8_000_200;
    expiry.actions = vec![
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
            8_000_000,
            0,
            "GA_Adler_Melee",
            "GE_Player_Adler_Melee1_Damage",
            0,
            0,
        ),
    ];
    let expiry_report = run_game_scenario(&expiry, &ExecutionLimits::default()).unwrap();
    let expiry_melee = expiry_report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["ability_id"] == "GA_Adler_Melee")
        .unwrap();
    assert_eq!(expiry_melee["non_crit_damage"], json!("201/1"));
    assert!(
        expiry_report["final_actor_states"][0]
            .get("active_shields")
            .and_then(serde_json::Value::as_array)
            .is_none_or(Vec::is_empty)
    );
}
