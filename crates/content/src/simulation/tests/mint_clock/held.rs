use super::*;
use crate::simulation::tests::support::formula_damage_total;

fn held(initial: u64, recovery: u64) -> GameScenarioRequest {
    let mut req = request(2);
    req.combat_stamina = Some(CombatStaminaProfile {
        capacity: Ratio {
            numerator: 200,
            denominator: 1,
        },
        initial: Ratio {
            numerator: initial,
            denominator: 1,
        },
        use_efficiency: Ratio::one(),
        recovery_rate: Ratio {
            numerator: recovery,
            denominator: 1,
        },
    });
    req.actions = vec![action(0, "GA_Mint019_Melee")];
    req.hold_inputs.insert(
        0,
        GameHoldInput {
            duration_us: 10_000_000,
            reserved_stocks: None,
            moving: false,
        },
    );
    req
}
fn times(report: &Value, effect: &str) -> Vec<u64> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["target_index"] == 0 && h["damage_effect_id"] == effect)
        .map(|h| h["time_us"].as_u64().unwrap())
        .collect()
}

#[test]
fn mint_held_two_cycles_keep_normal_prefix_finite_paid_spins_and_fractional_resources() {
    let mut req = held(200, 0);
    req.actions.push(action(10_000_000, "GA_Mint019_Melee"));
    req.hold_inputs.insert(1, req.hold_inputs[&0]);
    let report = run(&req);
    // Each cycle: two real Normal1 contacts, four opener contacts, thirty spins.
    assert_eq!(report["hits_settled"], 144);
    // (Normal1 .574 + opener2.255 +30*.228)*1000*.5*two cycles*two dummies.
    assert_eq!(
        serde_json::json!(formula_damage_total(&report)),
        json!("19338/1")
    );
    assert_eq!(
        &times(&report, "GE_Player_Mint_Melee1_Damage_New"),
        &[154636, 10154636]
    );
    assert_eq!(
        &times(&report, "GE_Player_Mint_Evade1_Damage")[..3],
        &[719044, 1067598, 1246159]
    );
    let spins = times(&report, "GE_Player_Mint_Evade5_Damage");
    assert_eq!(spins.len(), 60);
    assert_eq!(
        (spins[0], spins[29], spins[30]),
        (1998014, 5864789, 11998014)
    );
    let stamina = &report["combat_stamina"];
    assert_eq!(stamina["total_spent"], json!("120/1"));
    assert_eq!(stamina["current"], json!("80/1"));
    assert_eq!(stamina["spends"].as_array().unwrap().len(), 60);
    assert!(stamina.get("recovery_suppressions").is_none());
    let actor = &report["final_actor_states"][0];
    // 2 * (1.861 Normal prefix + 26.599 hold) =56.920 raw Charge.
    assert_eq!(actor["energy"], 56);
    assert_eq!(actor["charge_remainder"], json!("23/25"));
    assert_eq!(report, run(&req));
}

#[test]
fn mint_held_exhaustion_pays_only_affordable_packets_and_retains_exact_end_recovery() {
    let mut req = held(5, 0);
    req.actions.push(action(2_881_644, "GA_Mint019_Melee"));
    let report = run(&req);
    assert_eq!(
        times(&report, "GE_Player_Mint_Evade5_Damage"),
        [1998014, 2132377]
    );
    assert_eq!(report["combat_stamina"]["current"], json!("1/1"));
    assert_eq!(report["combat_stamina"]["total_spent"], json!("4/1"));
    req.actions[1] = action(2_881_643, "GA_Mint019_Melee");
    let failed = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(failed["rejected_action"]["reason"]["until_us"], 2_881_644);
    assert_eq!(
        failed["combat_stamina"]["spends"].as_array().unwrap().len(),
        2
    );
    // Every opportunity is paid even with zero HP loss; no pause is then caused.
    let mut miss = held(5, 0);
    miss.targets
        .iter_mut()
        .for_each(|t| t.resistance = Ratio::one());
    let report = run(&miss);
    assert_eq!(
        times(&report, "GE_Player_Mint_Evade5_Damage"),
        [1832981, 1967344]
    );
    assert_eq!(report["combat_stamina"]["total_spent"], json!("4/1"));
}

#[test]
fn mint_held_release_and_redirect_cancel_only_future_contacts_without_end_tax_or_refund() {
    let mut req = held(200, 0);
    req.hold_inputs.get_mut(&0).unwrap().duration_us = 1_100_000;
    // Release during the second opener's pause: End inherits the unexpired part.
    req.actions.push(action(1_734_062, "GA_Mint019_Melee"));
    let report = run(&req);
    assert_eq!(
        times(&report, "GE_Player_Mint_Evade1_Damage"),
        [719044, 1067598]
    );
    assert!(times(&report, "GE_Player_Mint_Evade4_Damage").is_empty());
    assert!(times(&report, "GE_Player_Mint_Evade5_Damage").is_empty());
    assert!(
        report["combat_stamina"]["spends"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let mut redirect = held(200, 0);
    redirect.actions.push(action(2_383_779, "GA_Mint019_Skill"));
    let report = run(&redirect);
    assert_eq!(times(&report, "GE_Player_Mint_Evade5_Damage").len(), 3);
    assert_eq!(
        times(&report, "GE_Player_Mint_Skill1_Damage_New"),
        [2583957, 2949203]
    );
    assert_eq!(report["combat_stamina"]["total_spent"], json!("6/1"));
    assert!(
        report["combat_stamina"]
            .get("recovery_suppressions")
            .is_none()
    );
    redirect.actions[1] = action(2_383_778, "GA_Mint019_Skill");
    let failed = run_game_scenario(&redirect, &Default::default()).unwrap();
    assert_eq!(failed["rejected_action"]["reason"]["until_us"], 2383779);
}

#[test]
fn mint_held_prefix_uses_its_reached_normal_section_and_release_clock() {
    for (stage, threshold, prefix) in [
        (0, 416533, 2),
        (1, 229700, 1),
        (2, 549500, 2),
        (3, 309400, 1),
        (4, 759400, 2),
    ] {
        let mut req = held(200, 0);
        let offsets = [0, 848860, 1544304, 2777262, 3552663];
        req.actions = offsets[..=stage]
            .iter()
            .map(|t| action(*t, "GA_Mint019_Melee"))
            .collect();
        req.hold_inputs.clear();
        req.hold_inputs.insert(
            stage,
            GameHoldInput {
                duration_us: threshold + 1,
                reserved_stocks: None,
                moving: false,
            },
        );
        let report = run(&req);
        // Begin is reached, then release1us later cancels its entire opener.
        assert!(times(&report, "GE_Player_Mint_Evade1_Damage").is_empty());
        assert_eq!(
            report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|h| h["command_index"] == stage)
                .count(),
            prefix * 2
        );
    }
}

#[test]
fn mint_held_interruption_and_death_release_recovery_gate_and_never_revive_packets() {
    for death in [false, true] {
        let mut req = held(100, 20);
        // Recovery is suppressed at1.9817s; two paid opportunities precede2.2s.
        if death {
            let model = crate::HypothesisId::CombatIncomingDamageFixtureV1;
            req.model_assumptions
                .insert(model.into(), model.description().into());
            req.incoming_damage_events.push(GameActorDamageEvent {
                timestamp_us: 2200000,
                actor_index: 0,
                delivered_damage: req.actors[0].initial_hp,
            });
        } else {
            req.interrupt_events.push(GameActorInterruptEvent {
                timestamp_us: 2200000,
                actor_index: 0,
                armor_damage: None,
            });
        }
        req.relative_duration_us = 4_500_000;
        let report = run(&req);
        assert_eq!(times(&report, "GE_Player_Mint_Evade5_Damage").len(), 2);
        assert_eq!(report["combat_stamina"]["total_spent"], json!("4/1"));
        assert!(
            report["combat_stamina"]
                .get("recovery_suppressions")
                .is_none()
        );
        // Last spend2.132377 => recovery resumes4.132377, including off-field/dead time.
        assert_eq!(report["combat_stamina"]["recovery_ready_at_us"], 4132377);
        assert_eq!(report["combat_stamina"]["current"], json!("7149323/50000"));
        if death {
            assert_eq!(report["final_actor_states"][0]["hp"], 0);
        }
    }
}

#[test]
fn mint_held_field_exit_cancels_train_and_releases_only_its_stamina_gate() {
    let mut req = held(100, 20);
    let mut ally = crate::dsl::compiler::compile_scenario(
        "options duration=10; zero char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
         target hp=100000000 def=1080 resist=0; active zero;",
        Default::default(),
    )
    .unwrap()
    .request
    .actors
    .remove(0);
    ally.slot = 1;
    req.actors.push(ally);
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 2383779,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    let report = run(&req);
    assert_eq!(times(&report, "GE_Player_Mint_Evade5_Damage").len(), 3);
    assert_eq!(report["combat_stamina"]["total_spent"], json!("6/1"));
    assert_eq!(report["combat_stamina"]["current"], json!("200/1"));
    assert!(
        report["combat_stamina"]
            .get("recovery_suppressions")
            .is_none()
    );
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| h["actor_index"] == 0)
            .all(|h| h["time_us"].as_u64().unwrap() < 2383779)
    );
}

#[test]
fn mint_tenacity_nested_cues_pause_only_named_end_timer_and_use_minimum_actor_rate() {
    let mut req = held(200, 0);
    req.tenacity_events = vec![
        GameActorTenacityEvent {
            timestamp_us: 3_000_000,
            actor_index: 0,
        },
        GameActorTenacityEvent {
            timestamp_us: 3_200_000,
            actor_index: 0,
        },
    ];
    req.actions.push(action(7_341_672, "GA_Mint019_Melee"));
    let report = run(&req);
    let spins = times(&report, "GE_Player_Mint_Evade5_Damage");
    assert_eq!(spins.len(), 30);
    // The .75s union advances animation by.0075s but the named timer by zero.
    assert_eq!((spins[8], spins[29]), (3_807_271, 6_607_289));
    assert_eq!(report["combat_stamina"]["total_spent"], json!("60/1"));
    let cue: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "mint_tenacity_cue")
        .collect();
    assert_eq!(cue.len(), 4);
    assert_eq!(cue[2]["time_us"], 3_550_000);
    assert_eq!(cue[2]["data"]["count"], 1);
    assert_eq!(cue[3]["data"]["count"], 0);
    req.actions[1] = action(7_341_671, "GA_Mint019_Melee");
    let failed = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(failed["rejected_action"]["reason"]["until_us"], 7_341_672);
}

#[test]
fn mint_tenacity_release_and_interrupt_cancel_timer_without_cancelling_unexpired_cue() {
    let mut req = held(200, 0);
    req.tenacity_events = vec![
        GameActorTenacityEvent {
            timestamp_us: 3_000_000,
            actor_index: 0,
        },
        GameActorTenacityEvent {
            timestamp_us: 3_200_000,
            actor_index: 0,
        },
    ];
    req.hold_inputs.get_mut(&0).unwrap().duration_us = 3_400_000;
    req.actions.push(action(4_363_139, "GA_Mint019_Melee"));
    let released = run(&req);
    assert_eq!(times(&released, "GE_Player_Mint_Evade5_Damage").len(), 8);
    assert_eq!(released["combat_stamina"]["total_spent"], json!("16/1"));
    assert!(
        released["combat_stamina"]
            .get("recovery_suppressions")
            .is_none()
    );
    req.hold_inputs.get_mut(&0).unwrap().duration_us = 10_000_000;
    req.interrupt_events.push(GameActorInterruptEvent {
        timestamp_us: 3_400_000,
        actor_index: 0,
        armor_damage: None,
    });
    req.actions[1] = action(3_500_000, "GA_Mint019_Melee");
    let interrupted = run(&req);
    assert_eq!(times(&interrupted, "GE_Player_Mint_Evade5_Damage").len(), 8);
    assert_eq!(
        times(&interrupted, "GE_Player_Mint_Melee1_Damage_New"),
        [154636, 3902136]
    );
    assert_eq!(
        interrupted["interrupt_receipts"][0]["interrupted_command_index"],
        0
    );
    for report in [released, interrupted] {
        assert_eq!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["event_kind"] == "mint_tenacity_cue")
                .count(),
            4
        );
    }
}

#[test]
fn mint_tenacity_validates_owner_and_order_and_roundtrips_the_native_assumption() {
    let mut req = held(200, 0);
    req.tenacity_events = vec![GameActorTenacityEvent {
        timestamp_us: 3_000_000,
        actor_index: 0,
    }];
    let restored: GameScenarioRequest =
        serde_json::from_value(serde_json::to_value(&req).unwrap()).unwrap();
    assert_eq!(run(&req), run(&restored));
    for (actor, time) in [(1, 3_000_000), (0, req.absolute_limit_us + 1)] {
        let mut invalid = req.clone();
        invalid.tenacity_events[0].actor_index = actor;
        invalid.tenacity_events[0].timestamp_us = time;
        assert!(
            run_game_scenario(&invalid, &Default::default())
                .unwrap_err()
                .contains("tenacity event0")
        );
    }
    req.tenacity_events.push(GameActorTenacityEvent {
        timestamp_us: 2_000_000,
        actor_index: 0,
    });
    assert!(
        run_game_scenario(&req, &Default::default())
            .unwrap_err()
            .contains("ordered times")
    );
}

#[test]
fn mint_tenacity_does_not_pause_a_real_skill_cooldown_across_held_interruption() {
    let mut req = held(200, 0);
    let hold = req.hold_inputs.remove(&0).unwrap();
    req.hold_inputs.insert(1, hold);
    req.actions = vec![
        action(0, "GA_Mint019_Skill"),
        action(1_000_000, "GA_Mint019_Melee"),
        action(6_182_791, "GA_Mint019_Skill"),
    ];
    req.tenacity_events = vec![GameActorTenacityEvent {
        timestamp_us: 4_000_000,
        actor_index: 0,
    }];
    req.interrupt_events.push(GameActorInterruptEvent {
        timestamp_us: 6_000_000,
        actor_index: 0,
        armor_damage: None,
    });
    let report = run(&req);
    assert_eq!(
        report["interrupt_receipts"][0]["skill_cooldown_before_us"],
        6_182_791
    );
    assert_eq!(
        report["interrupt_receipts"][0]["skill_cooldown_after_us"],
        6_182_791
    );
    assert_eq!(
        times(&report, "GE_Player_Mint_Skill1_Damage_New"),
        [200178, 565424, 6382969, 6748215]
    );
}
