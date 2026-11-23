use super::*;

fn native_kill_request() -> crate::simulation::GameScenarioRequest {
    let mut req = sample_s1_request();
    let actor = &mut req.actors[0];
    actor.character_id = "character:1008".into();
    actor.weapon_id = "weapon:fork_vine".into();
    actor.weapon_tier = 1;
    actor.gear_procs = vec![crate::simulation::GearProcConfig::BeHappy {
        heal_ratio: Ratio {
            numerator: 3,
            denominator: 25,
        },
        cooldown_us: 20_000_000,
    }];
    actor.initial_hp = 50;
    actor.initial_hp_max = 100;
    actor.talent_levels = Some([1; 3]);
    actor.character_max_level = Some(80);
    actor.static_stats.base_atk = 1000;
    actor.static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    actor.static_stats.crit_rate = Ratio::zero();
    actor.static_stats.damage_bonus = Ratio::zero();
    actor.static_stats.elemental_bonus = Ratio::zero();
    req.targets[0].hp_max = 1;
    req.targets[0].hp_current = 1;
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.actions = vec![crate::simulation::GameActionCommand::InvokeAbility {
        timestamp_us: 0,
        actor_index: 0,
        ability_id: "GA_Skia_Melee".into(),
        array_index: 0,
        damage_effect_id: "GE_Player_Skia_Melee1_Damage".into(),
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 0,
    }];
    req.relative_duration_us = 1_000_000;
    req
}

fn callback_fixture() -> crate::simulation::GameScenarioRequest {
    let mut req = native_kill_request();
    req.actors[0].initial_hp = 100;
    let mut second = req.actors[0].clone();
    second.slot = 1;
    second.character_id = "character:1042".into();
    second.initial_hp = 40;
    req.actors.push(second);
    req
}

fn schedule_defeat(
    req: &crate::simulation::GameScenarioRequest,
    state: &mut crate::simulation::state::GameSimulatorState,
    queue: &mut crate::simulation::timer_queue::EventQueue,
    sequence: &mut u64,
    actor_index: usize,
    target_index: usize,
    time_us: u64,
) {
    state.advance_unpaused_test_clock(time_us);
    let mut trace = Vec::new();
    crate::gear::on_target_defeated(
        &mut crate::runtime_api::CallbackContext {
            state,
            request: req,
            queue,
            sequence,
            trace: &mut trace,
        },
        actor_index,
        target_index,
    )
    .unwrap();
}

fn apply_queued_heals(
    req: &crate::simulation::GameScenarioRequest,
    state: &mut crate::simulation::state::GameSimulatorState,
    queue: &mut crate::simulation::timer_queue::EventQueue,
    sequence: &mut u64,
    trace: &mut Vec<crate::simulation::GameEventReceipt>,
) {
    while let Some(scheduled) = queue.pop() {
        let crate::simulation::events::GameEventData::Content(event) = scheduled.data else {
            panic!("Be Happy queued a non-content callback")
        };
        crate::callbacks::apply(
            event,
            &mut crate::runtime_api::CallbackContext {
                state,
                request: req,
                queue,
                sequence,
                trace,
            },
        )
        .unwrap();
    }
}

#[test]
fn be_happy_captures_live_max_hp_on_defeat_before_callback_changes() {
    let mut req = native_kill_request();
    req.actors[0].static_stats.base_hp = Some(1000);
    req.actors[0].static_stats.hp_percent = Ratio {
        numerator: 1,
        denominator: 10,
    };
    req.actors[0].static_stats.hp = 1100;
    req.actors[0].initial_hp_max = 1100;
    req.actors[0].initial_hp = 50;
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    crate::sets::hp_suit::apply(
        &mut crate::runtime_api::CallbackContext {
            state: &mut state,
            request: &req,
            queue: &mut queue,
            sequence: &mut sequence,
            trace: &mut trace,
        },
        crate::sets::hp_suit::StackGrant {
            actor_index: 0,
            max_hp_bonus_per_stack: Ratio {
                numerator: 1,
                denominator: 25,
            },
            duration_us: 10_000_000,
            max_stacks: 10,
            stack_count: 10,
        },
    )
    .unwrap();
    schedule_defeat(&req, &mut state, &mut queue, &mut sequence, 0, 0, 0);
    state.buffs.advance_time(10_000_000);
    apply_queued_heals(&req, &mut state, &mut queue, &mut sequence, &mut trace);
    assert_eq!(state.actors[0].hp, 230);
    assert!(trace.iter().any(|event| {
        event.event_kind == "be_happy_heal"
            && event.data.as_ref().and_then(|d| d.gross) == Some(180)
    }));
}

#[test]
fn native_skia_kill_heals_owner_and_reports_capped_exact_hp_settlement() {
    let mut req = native_kill_request();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 1);
    assert_eq!(report["max_proc_depth_reached"], 1);
    assert_eq!(report["final_actor_states"][0]["hp"], 62, "{report}");
    let event = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["event_kind"] == "be_happy_heal")
        .unwrap();
    assert_eq!(event["data"]["gross"], 12);
    assert_eq!(event["data"]["applied"], 12);
    assert_eq!(event["data"]["overflow"], 0);
    assert!(
        report["resolved_model_assumptions"][crate::weapons::be_happy::HYPOTHESIS_ID.as_str()]
            .is_string()
    );

    req.actors[0].gear_procs.clear();
    let no_gear = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(no_gear["final_actor_states"][0]["hp"], 50);
    assert!(
        no_gear["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| event["event_kind"] != "be_happy_heal")
    );
}

#[test]
fn be_happy_rounds_fractional_gross_once_and_full_hp_still_consumes_trigger() {
    let mut req = native_kill_request();
    req.actors[0].initial_hp = 1;
    req.actors[0].initial_hp_max = 25;
    req.actors[0].weapon_tier = 2;
    req.actors[0].gear_procs[0] = crate::simulation::GearProcConfig::BeHappy {
        heal_ratio: Ratio {
            numerator: 7,
            denominator: 50,
        },
        cooldown_us: 20_000_000,
    };
    let fractional = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(fractional["final_actor_states"][0]["hp"], 5);
    assert!(fractional["trace"].as_array().unwrap().iter().any(|event| {
        event["event_kind"] == "be_happy_heal"
            && event["data"]["gross"] == 4
            && event["data"]["applied"] == 4
            && event["data"]["overflow"] == 0
    }));

    req.actors[0].initial_hp = 100;
    req.actors[0].initial_hp_max = 100;
    req.actors[0].weapon_tier = 1;
    req.actors[0].gear_procs[0] = crate::simulation::GearProcConfig::BeHappy {
        heal_ratio: Ratio {
            numerator: 3,
            denominator: 25,
        },
        cooldown_us: 20_000_000,
    };
    let full = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(full["final_actor_states"][0]["hp"], 100);
    assert!(full["trace"].as_array().unwrap().iter().any(|event| {
        event["event_kind"] == "be_happy_heal"
            && event["data"]["gross"] == 12
            && event["data"]["applied"] == 0
            && event["data"]["overflow"] == 12
    }));
}

#[test]
fn runtime_recorder_records_and_validates_the_be_happy_hypothesis() {
    let mut req = native_kill_request();
    req.actors[0].gear_procs.clear();
    let mut reached = std::collections::BTreeMap::new();
    crate::runtime_api::record_hypothesis(
        &req,
        &mut reached,
        crate::weapons::be_happy::HYPOTHESIS_ID,
        crate::weapons::be_happy::HYPOTHESIS_ID.description(),
    )
    .unwrap();
    assert_eq!(
        reached[crate::weapons::be_happy::HYPOTHESIS_ID.as_str()],
        crate::weapons::be_happy::HYPOTHESIS_ID.description()
    );
    req.model_assumptions.insert(
        crate::weapons::be_happy::HYPOTHESIS_ID.into(),
        "conflicting description".into(),
    );
    let rejected = crate::runtime_api::record_hypothesis(
        &req,
        &mut reached,
        crate::weapons::be_happy::HYPOTHESIS_ID,
        crate::weapons::be_happy::HYPOTHESIS_ID.description(),
    )
    .unwrap_err();
    assert!(rejected.contains("conflicting reached model hypothesis"));
}

#[test]
fn nonfatal_hit_does_not_heal() {
    let mut req = native_kill_request();
    req.targets[0].hp_max = 10_000;
    req.targets[0].hp_current = 10_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["final_actor_states"][0]["hp"], 50);
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| event["event_kind"] != "be_happy_heal")
    );
}

#[test]
fn zero_damage_already_dead_target_and_rejected_cast_do_not_trigger() {
    let mut zero = native_kill_request();
    zero.actors[0].static_stats.atk = Ratio::zero();
    let zero_report = run_game_scenario(&zero, &ExecutionLimits::default()).unwrap();
    assert_eq!(zero_report["final_target_states"][0]["hp_current"], 1);
    assert!(
        zero_report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| event["event_kind"] != "be_happy_heal")
    );

    let mut already_dead = native_kill_request();
    already_dead.actions.push(already_dead.actions[0].clone());
    if let crate::simulation::GameActionCommand::InvokeAbility { timestamp_us, .. } =
        &mut already_dead.actions[1]
    {
        *timestamp_us = 1;
    }
    let dead_report = run_game_scenario(&already_dead, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        dead_report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "be_happy_heal")
            .count(),
        1
    );
    assert!(
        dead_report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_kind"] == "hit_target_unavailable")
    );

    let mut rejected = native_kill_request();
    if let crate::simulation::GameActionCommand::InvokeAbility { energy_cost, .. } =
        &mut rejected.actions[0]
    {
        *energy_cost = 101;
    }
    let rejected_report = run_game_scenario(&rejected, &ExecutionLimits::default()).unwrap();
    assert_eq!(rejected_report["hits_settled"], 0);
    assert!(
        rejected_report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| event["event_kind"] != "be_happy_heal")
    );
}

fn append_mint_teammate(req: &mut crate::simulation::GameScenarioRequest) {
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.weapon_id = "weapon:none".into();
    ally.gear_procs.clear();
    ally.talent_levels = None;
    req.actors.push(ally);
}

#[test]
fn teammate_defeat_does_not_heal_be_happy_owner() {
    let mut req = native_kill_request();
    append_mint_teammate(&mut req);
    req.actions = vec![
        crate::simulation::GameActionCommand::SwitchActor {
            timestamp_us: 0,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        crate::simulation::GameActionCommand::InvokeAbility {
            timestamp_us: 1,
            actor_index: 1,
            ability_id: "GA_Mint019_Melee".into(),
            array_index: 0,
            damage_effect_id: "GE_Player_Mint_Melee1_Damage_New".into(),
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 0,
        },
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["final_target_states"][0]["hp_current"], 0);
    assert_eq!(report["final_actor_states"][0]["hp"], 50);
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| event["event_kind"] != "be_happy_heal")
    );
}

#[test]
fn delayed_native_kill_heals_its_off_field_owner() {
    let mut req = native_kill_request();
    append_mint_teammate(&mut req);
    if let crate::simulation::GameActionCommand::InvokeAbility { hit_delay_us, .. } =
        &mut req.actions[0]
    {
        *hit_delay_us = 1_000_000;
    }
    req.actions
        .push(crate::simulation::GameActionCommand::SwitchActor {
            timestamp_us: 10,
            target_actor_index: 1,
            cooldown_us: 0,
        });
    req.relative_duration_us = 2_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["final_actor_states"][0]["hp"], 62);
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .find(|event| event["event_kind"] == "be_happy_heal")
            .unwrap()["active_actor"],
        1
    );
    assert!(
        report["resolved_model_assumptions"][crate::weapons::be_happy::HYPOTHESIS_ID.as_str()]
            .is_string()
    );
}

#[test]
fn actual_defeat_handler_and_heal_callback_reserve_full_hp_cd_and_isolate_wearers() {
    let req = callback_fixture();
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();

    schedule_defeat(&req, &mut state, &mut queue, &mut sequence, 0, 4, 0);
    schedule_defeat(&req, &mut state, &mut queue, &mut sequence, 0, 5, 0);
    assert_eq!(
        queue.len(),
        1,
        "duplicate same-time target defeats share one cooldown"
    );
    schedule_defeat(&req, &mut state, &mut queue, &mut sequence, 1, 6, 0);
    assert_eq!(
        queue.len(),
        2,
        "a second wearer has independent cooldown state"
    );
    apply_queued_heals(&req, &mut state, &mut queue, &mut sequence, &mut trace);
    assert_eq!(state.actors[0].hp, 100);
    assert_eq!(state.actors[1].hp, 52);
    assert!(trace.iter().any(|event| {
        event.event_kind == "be_happy_heal"
            && event.from_actor == Some(0)
            && event.data.as_ref().is_some_and(|d| {
                d.target_index == Some(4)
                    && d.gross == Some(12)
                    && d.applied == Some(0)
                    && d.overflow == Some(12)
            })
    }));
    assert!(trace.iter().any(|event| {
        event.event_kind == "be_happy_heal"
            && event.from_actor == Some(1)
            && event.data.as_ref().is_some_and(|d| {
                d.target_index == Some(6)
                    && d.gross == Some(12)
                    && d.applied == Some(12)
                    && d.overflow == Some(0)
            })
    }));

    state.actors[0].hp = 50;
    schedule_defeat(
        &req,
        &mut state,
        &mut queue,
        &mut sequence,
        0,
        7,
        19_999_999,
    );
    assert!(
        queue.is_empty(),
        "cooldown is still active just before its deadline"
    );
    schedule_defeat(
        &req,
        &mut state,
        &mut queue,
        &mut sequence,
        0,
        8,
        20_000_000,
    );
    assert_eq!(queue.len(), 1, "exact cooldown boundary is ready");
    apply_queued_heals(&req, &mut state, &mut queue, &mut sequence, &mut trace);
    assert_eq!(state.actors[0].hp, 62);
}

#[test]
fn be_happy_applies_static_source_healing_bonus() {
    let mut req = callback_fixture();
    req.actors[0].static_stats.healing_bonus = Ratio {
        numerator: 1,
        denominator: 5,
    };
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    state.advance_unpaused_test_clock(1);
    state.actors[0].hp = 50;
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    crate::callbacks::apply(
        crate::callbacks::ContentEvent::BeHappyHeal {
            actor_index: 0,
            target_index: 0,
            base_heal: crate::simulation::ExactFraction::new(7, 2).unwrap(),
        },
        &mut crate::runtime_api::CallbackContext {
            state: &mut state,
            request: &req,
            queue: &mut queue,
            sequence: &mut sequence,
            trace: &mut trace,
        },
    )
    .unwrap();
    assert_eq!(state.actors[0].hp, 54);
    assert_eq!(
        trace[0].data.as_ref().and_then(|d| d.gross.zip(d.applied)),
        Some((4, 4))
    );
    assert_eq!(
        state.execution_model_assumptions
            [crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID.as_str()],
        crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID.description()
    );
}

#[test]
fn be_happy_applies_both_live_bonuses_before_rounding_exact_base_heal() {
    let hypothesis_id = crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID.as_str();
    let req = callback_fixture();
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    state.advance_unpaused_test_clock(1);
    state.actors[0].hp = 50;
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    crate::callbacks::apply(
        crate::callbacks::ContentEvent::BeHappyHeal {
            actor_index: 0,
            target_index: 0,
            base_heal: crate::simulation::ExactFraction::new(7, 2).unwrap(),
        },
        &mut crate::runtime_api::CallbackContext {
            state: &mut state,
            request: &req,
            queue: &mut queue,
            sequence: &mut sequence,
            trace: &mut trace,
        },
    )
    .unwrap();
    assert!(
        !state
            .execution_model_assumptions
            .contains_key(hypothesis_id)
    );
    assert_eq!(state.actors[0].hp, 54);
    for modifier in ["healing_bonus", "incoming_healing_bonus"] {
        state.actors[0].hp = 50;
        state
            .apply_buff(
                0,
                crate::simulation::state::TimedBuffDeclaration::reset(
                    &format!("test.{modifier}"),
                    "test",
                    ntesim_core::buffs::dispatch::BuffRecipient::SpecificActor(0),
                    10,
                    ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                    std::collections::BTreeMap::from([(
                        modifier.into(),
                        crate::simulation::ExactFraction::new(1, 5).unwrap(),
                    )]),
                ),
            )
            .unwrap();
        let event = crate::callbacks::ContentEvent::BeHappyHeal {
            actor_index: 0,
            target_index: 0,
            base_heal: crate::simulation::ExactFraction::new(7, 2).unwrap(),
        };
        state.actors[0].hp = 50;
        crate::callbacks::apply(
            event,
            &mut crate::runtime_api::CallbackContext {
                state: &mut state,
                request: &req,
                queue: &mut queue,
                sequence: &mut sequence,
                trace: &mut trace,
            },
        )
        .unwrap();
        assert_eq!(state.actors[0].hp, 54);
        assert_eq!(
            trace
                .last()
                .unwrap()
                .data
                .as_ref()
                .and_then(|d| d.gross.zip(d.applied)),
            Some((4, 4))
        );
        assert_eq!(
            state.execution_model_assumptions[hypothesis_id],
            crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID.description()
        );
        state.buffs = Default::default();
        trace.clear();
    }

    state.actors[0].hp = 40;
    for modifier in ["healing_bonus", "incoming_healing_bonus"] {
        state
            .apply_buff(
                0,
                crate::simulation::state::TimedBuffDeclaration::reset(
                    &format!("test.stacked.{modifier}"),
                    "test",
                    ntesim_core::buffs::dispatch::BuffRecipient::SpecificActor(0),
                    10,
                    ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                    std::collections::BTreeMap::from([(
                        modifier.into(),
                        crate::simulation::ExactFraction::new(1, 5).unwrap(),
                    )]),
                ),
            )
            .unwrap();
    }
    let modeled = req;
    crate::callbacks::apply(
        crate::callbacks::ContentEvent::BeHappyHeal {
            actor_index: 0,
            target_index: 0,
            base_heal: crate::simulation::ExactFraction::new(7, 2).unwrap(),
        },
        &mut crate::runtime_api::CallbackContext {
            state: &mut state,
            request: &modeled,
            queue: &mut queue,
            sequence: &mut sequence,
            trace: &mut trace,
        },
    )
    .unwrap();
    assert_eq!(state.actors[0].hp, 45);
    assert_eq!(
        trace
            .last()
            .unwrap()
            .data
            .as_ref()
            .and_then(|d| d.gross.zip(d.applied)),
        Some((5, 5))
    );
}

#[test]
fn be_happy_hand_derived_heal_math_t1_t5() {
    fn gcd(mut a: u128, mut b: u128) -> u128 {
        while b != 0 {
            let rest = a % b;
            a = b;
            b = rest;
        }
        a
    }
    fn reduced(n: u128, d: u128) -> (u128, u128) {
        let g = gcd(n, d);
        (n / g, d / g)
    }
    // Frozen buff_vine_Hp: T1 3/25, T2 7/50, T3 4/25, T4 9/50, T5 1/5.
    // T1: MaxHP 100 * 3/25 = 12 exact.
    assert_eq!(reduced(100 * 3, 25), (12, 1));
    // T2: MaxHP 25 * 7/50 = 175/50 = 7/2 = 3.5 -> nearest-integer ties-up -> 4.
    assert_eq!(reduced(25 * 7, 50), (7, 2));
    // T5: MaxHP 100 * 1/5 = 20 exact.
    assert_eq!(reduced(100, 5), (20, 1));
    // Modifier ordering: exact base 3.5 with one 1/5 live bonus:
    // 7/2 * 6/5 = 21/5 = 4.2 -> 4 (one final rounding).
    assert_eq!(reduced(7 * 6, 2 * 5), (21, 5));
    // Two-rounding misreading rejected: rounding the 3.5 base first gives 4,
    // then 4 * 6/5 = 24/5 = 4.8 -> 5, contradicting the one-final-rounding 4.
    assert_eq!(reduced(4 * 6, 5), (24, 5));
    assert_ne!((21, 5), (24, 5));
}
