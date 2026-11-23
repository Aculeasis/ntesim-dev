use super::*;

fn scenario() -> GameScenarioRequest {
    let mut req = phase2_request();
    for id in [
        crate::HypothesisId::BaicangActorHitstopV1,
        crate::HypothesisId::BaicangCancelWindowsV1,
    ] {
        req.model_assumptions
            .insert(id.into(), id.description().into());
    }
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req
}

fn heavy(time: u64) -> GameActionCommand {
    make_action(
        time,
        0,
        "GA_Cang_Melee",
        "GE_Player_Cang_Branch1_Damage",
        0,
        0,
    )
}

fn jump(time: u64) -> GameActionCommand {
    GameActionCommand::PlayerInput {
        timestamp_us: time,
        actor_index: 0,
        input: PlayerInput::Jump,
        critical_success: false,
    }
}

fn skill(time: u64) -> GameActionCommand {
    make_action(
        time,
        0,
        "GA_Cang_Skill",
        "GE_Player_Cang_Skill_Damage",
        0,
        14_000_000,
    )
}

fn times(report: &serde_json::Value, effect: &str) -> Vec<u64> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["damage_effect_id"] == effect)
        .map(|h| h["time_us"].as_u64().unwrap())
        .collect()
}

#[test]
fn causal_normal_chain_retains_all_seven_slots_and_the_full_recovery_boundary() {
    let mut req = scenario();
    req.actions = [0, 923_724, 1_764_001, 2_982_688, 4_689_029, 5_509_308]
        .into_iter()
        .map(normal)
        .collect();
    let report = run(&req);
    assert_eq!(report["hits_settled"], 12);
    assert_eq!(
        times(&report, "GE_Player_Cang_Melee4_Damage"),
        [
            133_698, 249_436, 365_174, 480_912, 596_650, 712_388, 828_126
        ]
        .map(|t| t + 2_982_688)
    );
    req.actions[5] = normal(5_509_307);
    let early = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        early["rejected_action"]["reason"],
        json!({"kind":"recovery","until_us":5_509_308})
    );
}

#[test]
fn exact_jump_and_residual_pause_retime_an_unreleased_actor_but_preserve_its_later_travel() {
    let mut req = scenario();
    req.actions = vec![heavy(0), jump(482_091), skill(482_092), normal(1_547_860)];
    let report = run(&req);
    // Jump progress: .416667 + (.436294-.416667)/.3 => .482090333, ceil to microseconds.
    assert_eq!(
        times(&report, "GE_Player_Cang_Branch1_1_Damage"),
        vec![666_667; 3]
    );
    // .034575s of the original pause remains; the actor releases at .522947.
    // The later explosion pause cannot slow its independent .65s travel.
    assert_eq!(
        times(&report, "GE_Player_Cang_SkillB_Damage"),
        vec![1_172_947]
    );
    assert_eq!(report["final_actor_states"][0]["hp"], 9700);
    req.actions[3] = normal(1_547_859);
    let early = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        early["rejected_action"]["reason"],
        json!({"kind":"recovery","until_us":1_547_860})
    );
    req.actions[1] = jump(482_090);
    let early = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        early["rejected_action"]["reason"],
        json!({"kind":"recovery","until_us":482_091})
    );
}

#[test]
fn a_released_heavy_actor_slows_the_next_normal_once_across_all_aoe_recipients() {
    let mut req = scenario();
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    req.actions = vec![heavy(0), jump(482_091), normal(520_000), normal(1_513_724)];
    let report = run(&req);
    assert_eq!(
        times(&report, "GE_Player_Cang_Melee1_Damage"),
        vec![774_377; 2]
    );
    let pauses = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "baicang_hitstop")
        .count();
    assert_eq!(pauses, 4); // direct heavy, its coincident explosions, then two Normals
    req.actions[3] = normal(1_513_723);
    let early = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        early["rejected_action"]["reason"],
        json!({"kind":"recovery","until_us":1_513_724})
    );
}

#[test]
fn execute_overlap_uses_the_strongest_pause_and_does_not_freeze_cooldowns_or_other_targets() {
    for aim in [0, 1] {
        for target in [0, 1] {
            let mut req = scenario();
            req.actors[0].initial_energy = 120;
            req.actors[0].energy_capacity = 120;
            req.targets.push(req.targets[0].clone());
            req.targets[1].target_index = 1;
            req.targets[target].hp_max = 10_000;
            req.targets[target].hp_current = 3650; // Burst2400 leaves1250; Normal285 leaves965 and executes.
            let boundary = if target == aim { 4_110_624 } else { 3_923_724 };
            req.actions = vec![burst(0), normal(3_000_000), normal(boundary)];
            req.aim_targets = (0..req.actions.len())
                .map(|command| (command, aim))
                .collect();
            let report = run(&req);
            assert_eq!(times(&report, "ntesim.baicang.execute"), vec![3_184_377]);
            assert_eq!(
                report["trace"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|e| e["event_kind"] == "hp_finish")
                    .count(),
                1
            );
            req.actions[2] = normal(boundary - 1);
            let early = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
            assert_eq!(
                early["rejected_action"]["reason"],
                json!({"kind":"recovery","until_us":boundary})
            );
        }
    }
}

#[test]
fn zero_hp_damage_creates_no_pause_and_cannot_keep_the_baked_contact_recovery() {
    let mut req = scenario();
    req.actors[0].static_stats.base_atk = 0;
    req.actors[0].static_stats.atk = Ratio::zero();
    req.actions = vec![normal(0), normal(894_024)];
    let report = run(&req);
    assert_eq!(report["hits_settled"], 2);
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["event_kind"] == "baicang_hitstop")
    );
    req.model_assumptions
        .remove(crate::HypothesisId::BaicangPowerWordsV1.as_str());
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires baicang.power_words_v1")
    );
}

#[test]
fn field_activation_follows_caster_progress_then_all_six_ticks_keep_the_real_lifetime() {
    let mut req = scenario();
    req.actors[0].initial_energy = 120;
    req.actors[0].energy_capacity = 120;
    req.actions = vec![heavy(0), jump(482_091), burst(520_000), heavy(4_000_000)];
    let report = run(&req);
    assert_eq!(
        times(&report, "GE_Player_Cang_UltraSkill2_Damage"),
        vec![3_185_367]
    );
    assert_eq!(
        times(&report, "GE_Player_Cang_UltraSkill_Damage"),
        (1..=6)
            .map(|slot| 3_374_959 + slot * 2_000_000)
            .collect::<Vec<_>>()
    );
    // A later heavy pauses the caster, while already activated world ticks keep their clock.
    let id = crate::HypothesisId::BaicangForcedInterruptV1;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    req.actions.pop();
    for (at, cancelled, ticks) in [(3_184_743, true, 0), (3_184_744, false, 6)] {
        req.interrupt_events = vec![crate::simulation::GameActorInterruptEvent {
            timestamp_us: at,
            actor_index: 0,
            armor_damage: None,
        }];
        let report = run(&req);
        assert_eq!(
            report["interrupt_receipts"][0]["field_cancelled"],
            cancelled
        );
        assert_eq!(
            times(&report, "GE_Player_Cang_UltraSkill_Damage").len(),
            ticks
        );
    }
}

#[test]
fn interruption_uses_projected_emission_and_retains_contact_pauses_after_the_action_ends() {
    let mut req = scenario();
    let id = crate::HypothesisId::BaicangForcedInterruptV1;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    req.actions = vec![heavy(0), normal(450_001)];
    req.interrupt_events = vec![crate::simulation::GameActorInterruptEvent {
        timestamp_us: 450_000,
        actor_index: 0,
        armor_damage: None,
    }];
    let report = run(&req);
    assert_eq!(
        report["interrupt_receipts"][0]["cancelled_hit_deliveries"],
        0
    );
    assert_eq!(
        times(&report, "GE_Player_Cang_Melee1_Damage"),
        vec![714_591]
    );
    // The original SkillB release was shifted from .498744 to .522947.
    req.actions = vec![heavy(0), jump(482_091), skill(482_092)];
    for (at, removed, cd) in [(522_946, 1, 3_522_946), (522_947, 0, 14_482_092)] {
        req.interrupt_events[0].timestamp_us = at;
        let report = run(&req);
        assert_eq!(
            report["interrupt_receipts"][0]["cancelled_hit_deliveries"],
            removed
        );
        assert_eq!(
            report["interrupt_receipts"][0]["skill_cooldown_after_us"],
            cd
        );
    }
}

#[test]
fn two_causal_clock_cycles_keep_exact_damage_words_and_resource_carries() {
    let mut req = scenario();
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    req.actions.clear();
    for base in [0, 20_000_000] {
        req.actions.extend([
            heavy(base),
            jump(base + 482_091),
            skill(base + 482_092),
            normal(base + 1_547_860),
        ]);
    }
    let report = run(&req);
    assert_eq!(report["hits_settled"], 24);
    assert_eq!(report["total_damage_exact"], json!("11028/1"));
    let actor = &report["final_actor_states"][0];
    assert_eq!(actor["hp"], 9127);
    assert_eq!(actor["granted_energy"], 17);
    assert_eq!(actor["charge_remainder"], json!("157/500"));
    assert_eq!(actor["cycle"]["granted"], 28);
    assert_eq!(actor["cycle"]["carry"], json!("371/500"));
}

#[test]
fn field_exit_keeps_the_owner_pause_without_locking_the_active_ally() {
    let mut req = scenario();
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1004".into();
    req.actors.push(ally);
    req.actions = vec![
        heavy(0),
        jump(482_091),
        GameActionCommand::SwitchActor {
            timestamp_us: 520_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 700_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        normal(700_001),
    ];
    let report = run(&req);
    assert_eq!(
        times(&report, "GE_Player_Cang_Melee1_Damage"),
        vec![931_045]
    );
    let off_field = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["time_us"] == 666_667 && e["event_kind"] == "baicang_hitstop")
        .unwrap();
    assert_eq!(off_field["active_actor"], 1);
}

#[test]
fn activated_word_heal_and_missiles_are_real_timers_during_later_caster_pauses() {
    let mut req = scenario();
    req.actors[0].initial_energy = 120;
    req.actors[0].energy_capacity = 120;
    req.actions = vec![
        normal(0),
        normal(1_000_000),
        normal(2_000_000),
        burst(4_000_000),
        heavy(7_000_000),
    ];
    let report = run(&req);
    assert_eq!(
        times(&report, "GE_Player_Cang_WorldC_Damage"),
        vec![7_664_744]
    );
    assert_eq!(
        times(&report, "GE_Player_Cang_WorldC_1_Damage"),
        vec![7_964_744; 2]
    );
    let heals: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "baicang_word_heal")
        .map(|e| e["time_us"].as_u64().unwrap())
        .collect();
    assert_eq!(heals, vec![7_854_959]);
    assert_eq!(report["final_actor_states"][0]["hp"], 10000);
}
