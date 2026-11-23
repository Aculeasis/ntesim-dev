use super::*;
use crate::simulation::tests::support::formula_damage_total;

fn scenario() -> GameScenarioRequest {
    let mut req = request(2);
    req.combat_stamina = Some(CombatStaminaProfile {
        capacity: Ratio {
            numerator: 200,
            denominator: 1,
        },
        initial: Ratio {
            numerator: 200,
            denominator: 1,
        },
        use_efficiency: Ratio::one(),
        recovery_rate: Ratio::zero(),
    });
    req
}
fn input(time: u64, input: PlayerInput, critical: bool) -> GameActionCommand {
    GameActionCommand::PlayerInput {
        timestamp_us: time,
        actor_index: 0,
        input,
        critical_success: critical,
    }
}
fn hits<'a>(report: &'a Value, effect: &str) -> Vec<&'a Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["target_index"] == 0 && h["damage_effect_id"] == effect)
        .collect()
}
fn plunge(req: &mut GameScenarioRequest, time: u64, height: Ratio) {
    let mut cmd = action(time, "GA_Mint019_Melee");
    if let GameActionCommand::InvokeAbility {
        damage_effect_id, ..
    } = &mut cmd
    {
        *damage_effect_id = crate::characters::mint::plunge::PACKET.into();
    }
    req.plunge_inputs.insert(
        req.actions.len(),
        GamePlungeInput {
            landing_delay_us: 500000,
            height_bonus: height,
        },
    );
    req.actions.push(cmd);
}

#[test]
fn mint_secondary_two_paid_ripostes_keep_real_guard_and_skill_reset_without_extra_normals() {
    let mut req = scenario();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
    for start in [0, 10000000] {
        req.actions.extend([
            action(start, "GA_Mint019_Skill"),
            input(start + 1400000, PlayerInput::Dash, true),
            action(start + 1400000, "GA_Mint019_Melee"),
            action(start + 2200000, "GA_Mint019_Skill"),
        ]);
    }
    let report = run(&req);
    let riposte = hits(&report, crate::characters::mint::riposte::PACKET);
    assert_eq!(riposte.len(), 2);
    assert_eq!(
        (
            riposte[0]["time_us"].as_u64(),
            riposte[1]["time_us"].as_u64()
        ),
        (Some(1627216), Some(11627216))
    );
    assert_eq!(riposte[0]["non_crit_damage"], json!("1491/1"));
    assert!(hits(&report, "GE_Player_Mint_Melee1_Damage_New").is_empty());
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["event_kind"] == "mint_a1_skill_cooldown_reset")
            .count(),
        2
    );
    assert_eq!(report["combat_stamina"]["total_spent"], json!("30/1"));
    assert_eq!(report["final_actor_states"][0]["energy"], 28);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        json!("2/5")
    );
    assert_eq!(report, run(&req));
}

#[test]
fn mint_secondary_front_back_skill_windows_and_paid_dash_cooldown_are_independent() {
    for (back, gate) in [(false, 340205), (true, 448540)] {
        let mut req = scenario();
        req.actions = vec![
            input(0, PlayerInput::Dash, false),
            action(gate, "GA_Mint019_Skill"),
        ];
        if back {
            req.dash_directions.insert(0, GameDashDirection::Back);
        }
        let report = run(&req);
        assert_eq!(
            hits(&report, "GE_Player_Mint_Skill1_Damage_New")[0]["time_us"],
            gate + 200178
        );
        assert!(hits(&report, crate::characters::mint::riposte::PACKET).is_empty());
        req.actions[1] = action(gate - 1, "GA_Mint019_Skill");
        let failed = run_game_scenario(&req, &Default::default()).unwrap();
        assert_eq!(failed["rejected_action"]["reason"]["until_us"], gate);
        assert_eq!(
            failed["combat_stamina"]["spends"].as_array().unwrap().len(),
            1
        );
    }
    let mut req = scenario();
    req.dash_directions.insert(0, GameDashDirection::Back);
    req.actions = vec![
        input(0, PlayerInput::Dash, false),
        input(797188, PlayerInput::Dash, false),
    ];
    let failed = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(failed["rejected_action"]["reason"]["kind"], "cooldown");
    assert_eq!(failed["rejected_action"]["reason"]["until_us"], 900000);
    assert_eq!(
        failed["combat_stamina"]["spends"].as_array().unwrap().len(),
        1
    );
}

#[test]
fn mint_secondary_jump_landing_and_two_height_endpoints_preserve_one_plunge_per_flight() {
    let mut req = scenario();
    req.actions = vec![
        input(0, PlayerInput::Dash, false),
        input(82544, PlayerInput::Jump, false),
    ];
    plunge(&mut req, 100000, Ratio::zero());
    req.actions.push(input(816850, PlayerInput::Jump, false));
    plunge(&mut req, 816850, Ratio::one());
    let report = run(&req);
    let landings = hits(&report, crate::characters::mint::plunge::PACKET);
    assert_eq!(landings.len(), 2);
    assert_eq!(
        (
            landings[0]["time_us"].as_u64(),
            landings[1]["time_us"].as_u64()
        ),
        (Some(664514), Some(1381364))
    );
    assert_eq!(landings[0]["non_crit_damage"], json!("557/1"));
    assert_eq!(landings[1]["non_crit_damage"], json!("1114/1"));
    assert_eq!(report["combat_stamina"]["total_spent"], json!("15/1"));
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        json!("3/5")
    );
    // A second Jump before the declared landing fails without paying another Dodge.
    req.actions
        .insert(2, input(90000, PlayerInput::Jump, false));
    req.plunge_inputs.clear();
    req.actions.truncate(3);
    let failed = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(failed["rejected_action"]["reason"]["kind"], "other");
    assert_eq!(
        failed["combat_stamina"]["spends"].as_array().unwrap().len(),
        1
    );
}

#[test]
fn mint_secondary_approach_replaces_normal_one_and_advances_the_real_chain() {
    let mut req = scenario();
    req.approach_inputs.insert(0);
    req.actions = vec![
        action(0, "GA_Mint019_Melee"),
        action(1000000, "GA_Mint019_Melee"),
    ];
    let report = run(&req);
    assert_eq!(
        hits(&report, "GE_Player_Mint_Melee1_Damage_New")[0]["time_us"],
        205847
    );
    assert_eq!(
        hits(&report, "GE_Player_Mint_Melee2_Damage_New")[0]["time_us"],
        376738
    );
    assert_eq!(
        hits(&report, "GE_Player_Mint_Melee3_Damage_New")[0]["time_us"],
        1089514
    );
    assert_eq!(report["hits_settled"], 6);
    assert_eq!(
        serde_json::json!(formula_damage_total(&report)),
        json!("865/1")
    );
    assert!(
        report["combat_stamina"]["spends"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    for (duration, next) in [
        (466533, "GE_Player_Mint_Melee3_Damage_New"),
        (466534, "GE_Player_Mint_Melee1_Damage_New"),
    ] {
        let mut held = req.clone();
        held.hold_inputs.insert(
            0,
            GameHoldInput {
                duration_us: duration,
                moving: false,
                reserved_stocks: None,
            },
        );
        held.actions[1] = action(1100000, "GA_Mint019_Melee");
        let report = run(&held);
        assert_eq!(
            report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .find(|h| h["command_index"] == 1)
                .unwrap()["damage_effect_id"],
            next
        );
    }
}

#[test]
fn mint_secondary_explicit_parry_requests_delayed_back_without_a_free_damage_response() {
    let mut req = scenario();
    req.actions = vec![
        input(0, PlayerInput::Dash, false),
        action(568540, "GA_Mint019_Skill"),
    ];
    req.parry_events.push(GameActorParryEvent::At {
        timestamp_us: 100000,
        actor_index: 0,
    });
    let report = run(&req);
    assert_eq!(report["parry_receipts"].as_array().unwrap().len(), 1);
    assert_eq!(report["combat_stamina"]["total_spent"], json!("15/1"));
    assert!(hits(&report, crate::characters::mint::riposte::PACKET).is_empty());
    assert_eq!(
        hits(&report, "GE_Player_Mint_Skill1_Damage_New")[0]["time_us"],
        768718
    );
    req.actions[1] = action(568539, "GA_Mint019_Skill");
    let failed = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(failed["rejected_action"]["reason"]["until_us"], 568540);
}

fn party(cycle: u64) -> GameScenarioRequest {
    let mut req = scenario();
    let mut mint = req.actors.remove(0);
    req.actors = super::super::reactions::request(&[
        (0, "character:1046", cycle, 0),
        (1, "character:1019", cycle, 0),
        (2, "character:1052", cycle, 0),
    ])
    .actors;
    mint.slot = 1;
    mint.initial_cycle = cycle;
    req.actors[1] = mint;
    req
}
fn swap(time: u64, actor: usize) -> GameActionCommand {
    GameActionCommand::SwitchActor {
        timestamp_us: time,
        target_actor_index: actor,
        cooldown_us: 0,
    }
}
fn mint_normal(time: u64) -> GameActionCommand {
    let mut cmd = action(time, "GA_Mint019_Melee");
    if let GameActionCommand::InvokeAbility { actor_index, .. } = &mut cmd {
        *actor_index = 1;
    }
    cmd
}

#[test]
fn mint_secondary_two_qualified_supports_keep_paid_outgoing_cycle_and_decline_an_unfunded_third() {
    let mut req = party(100);
    req.actions = vec![
        swap(0, 1),
        swap(2000000, 2),
        swap(5000000, 1),
        swap(7000000, 0),
        swap(10000000, 1),
        mint_normal(10500000),
    ];
    let report = run(&req);
    let support = hits(
        &report,
        crate::characters::mint::clock::secondary::SUPPORT_PACKET,
    );
    assert_eq!(support.len(), 2);
    assert_eq!(
        (
            support[0]["time_us"].as_u64(),
            support[1]["time_us"].as_u64()
        ),
        (Some(739424), Some(5739424))
    );
    assert_eq!(support[0]["non_crit_damage"], json!("1714/1"));
    for actor in 0..3 {
        assert_eq!(report["final_actor_states"][actor]["cycle"]["spent"], 100);
    }
    assert_eq!(
        hits(&report, "GE_Player_Mint_Melee1_Damage_New")[0]["time_us"],
        10699890
    );
    assert_eq!(report["combat_stamina"]["total_spent"], json!("0/1"));
    assert_eq!(report, run(&req));
}

#[test]
fn mint_secondary_entry_waits_for_basic_advances_to_two_and_keeps_switch_distinct_from_recovery() {
    let mut req = party(0);
    req.actions = vec![
        swap(0, 1),
        mint_normal(500000),
        swap(1400000, 1),
        mint_normal(1500000),
    ];
    let report = run(&req);
    assert!(
        hits(
            &report,
            crate::characters::mint::clock::secondary::SUPPORT_PACKET
        )
        .is_empty()
    );
    assert_eq!(
        hits(&report, "GE_Player_Mint_Melee1_Damage_New")[0]["time_us"],
        699890
    );
    assert_eq!(
        hits(&report, "GE_Player_Mint_Melee2_Damage_New")[0]["time_us"],
        866642
    );
    assert_eq!(
        hits(&report, "GE_Player_Mint_Melee3_Damage_New")[0]["time_us"],
        1589514
    );
    let mut support = party(100);
    support.actions = vec![swap(0, 1), swap(1653238, 2)];
    let report = run(&support);
    assert_eq!(
        hits(
            &report,
            crate::characters::mint::clock::secondary::SUPPORT_PACKET
        )
        .len(),
        1
    );
    support.actions[1] = swap(1653237, 2);
    let failed = run_game_scenario(&support, &Default::default()).unwrap();
    assert_eq!(failed["rejected_action"]["reason"]["until_us"], 1653238);
}

#[test]
fn mint_secondary_landing_prefix_can_hold_and_interruptions_clear_both_flight_routes() {
    let mut req = scenario();
    req.actions = vec![input(0, PlayerInput::Jump, false)];
    plunge(&mut req, 0, Ratio::zero());
    req.hold_inputs.insert(
        1,
        GameHoldInput {
            duration_us: 1700000,
            moving: false,
            reserved_stocks: None,
        },
    );
    req.actions.push(action(3000000, "GA_Mint019_Melee"));
    let report = run(&req);
    assert_eq!(
        hits(&report, crate::characters::mint::plunge::PACKET)[0]["time_us"],
        564514
    );
    assert_eq!(hits(&report, "GE_Player_Mint_Evade1_Damage").len(), 2);
    assert_eq!(
        serde_json::json!(formula_damage_total(&report)),
        json!("2658/1")
    );
    assert_eq!(report["combat_stamina"]["total_spent"], json!("0/1"));
    for during_plunge in [false, true] {
        let mut req = scenario();
        req.actions = vec![input(0, PlayerInput::Jump, false)];
        if during_plunge {
            plunge(&mut req, 0, Ratio::zero());
        }
        req.interrupt_events.push(GameActorInterruptEvent {
            timestamp_us: 250000,
            actor_index: 0,
            armor_damage: None,
        });
        req.actions.push(action(300000, "GA_Mint019_Skill"));
        let report = run(&req);
        assert_eq!(
            report["interrupt_receipts"][0]["interrupted_command_index"],
            usize::from(during_plunge)
        );
        assert!(hits(&report, crate::characters::mint::plunge::PACKET).is_empty());
        assert_eq!(
            hits(&report, "GE_Player_Mint_Skill1_Damage_New")[0]["time_us"],
            500178
        );
    }
}

#[test]
fn mint_secondary_jump_windows_cancel_each_normal_at_its_own_owner_progress() {
    let offsets = [0, 848860, 1544304, 2777262, 3552663];
    // Nominal Jump offsets plus only the already caused .01 contact losses.
    let gates = [349881, 278194, 566232, 225870, 727627];
    for stage in 0..5 {
        let mut req = scenario();
        req.actions = offsets[..=stage]
            .iter()
            .map(|t| action(*t, "GA_Mint019_Melee"))
            .collect();
        req.actions.push(input(
            offsets[stage] + gates[stage],
            PlayerInput::Jump,
            false,
        ));
        let report = run(&req);
        assert_eq!(
            report["player_input_receipts"][0]["time_us"],
            offsets[stage] + gates[stage]
        );
        *req.actions.last_mut().unwrap() =
            input(offsets[stage] + gates[stage] - 1, PlayerInput::Jump, false);
        let failed = run_game_scenario(&req, &Default::default()).unwrap();
        assert_eq!(
            failed["rejected_action"]["reason"]["until_us"],
            offsets[stage] + gates[stage]
        );
    }
}
