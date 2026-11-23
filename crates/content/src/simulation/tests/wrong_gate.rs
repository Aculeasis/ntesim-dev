use super::*;

fn ability(
    timestamp_us: u64,
    actor_index: usize,
    ability_id: &str,
    effect_id: &str,
) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index,
        ability_id: ability_id.into(),
        damage_effect_id: effect_id.into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 0,
    }
}

fn request(with_weapon: bool) -> GameScenarioRequest {
    let mut request = sample_s1_request();
    request.relative_duration_us = 31_000_000;
    request.actors[0].character_id = "character:1021".into();
    request.actors[0].weapon_id = if with_weapon { "weapon:fork_Door" } else { "" }.into();
    request.actors[0].weapon_tier = 1;
    request.actors[0].set_id = None;
    request.actors[0].static_stats.base_atk = 1000;
    request.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    request.actors[0].initial_hp = 20_000;
    request.actors[0].initial_hp_max = 20_000;
    request.actors[0].static_stats.hp = 20_000;
    request.actors[0].talent_levels = Some([1, 1, 1]);
    request.actors[0].character_level = 80;
    request.actors[0].character_max_level = Some(80);
    request.actors[0].initial_energy = 100;
    request.actors[0].energy_capacity = 100;
    request.actors[0].gear_procs = if with_weapon {
        vec![GearProcConfig::WrongGate {
            atk_bonus: Ratio {
                numerator: 4,
                denominator: 25,
            },
            anima_damage_bonus: Ratio {
                numerator: 3,
                denominator: 10,
            },
            ally_damage_bonus: Ratio {
                numerator: 3,
                denominator: 20,
            },
            duration_us: 20_000_000,
        }]
    } else {
        Vec::new()
    };
    let mut mint = request.actors[0].clone();
    mint.slot = 1;
    mint.character_id = "character:1019".into();
    mint.weapon_id.clear();
    mint.gear_procs.clear();
    mint.static_stats.base_atk = 1000;
    mint.static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    mint.initial_hp = 20_000;
    request.actors.push(mint);
    request.active_actor_index = 0;
    request.targets[0].hp_max = 10_000_000;
    request.targets[0].hp_current = 10_000_000;
    request.targets[0].defense = 0;
    request.targets[0].resistance = Ratio::zero();
    request.actions = vec![
        ability(
            0,
            0,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 200_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        ability(
            500_000,
            1,
            "GA_Mint019_Skill",
            "GE_Player_Mint_Skill1_Damage_New",
        ),
        ability(
            11_000_000,
            1,
            "GA_Mint019_Skill",
            "GE_Player_Mint_Skill1_Damage_New",
        ),
        ability(
            30_200_000,
            1,
            "GA_Mint019_Skill",
            "GE_Player_Mint_Skill1_Damage_New",
        ),
    ];
    request
}

fn nature_request(with_weapon: bool) -> GameScenarioRequest {
    let mut request = request(with_weapon);
    request.actions = vec![
        ability(
            0,
            0,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
        ),
        ability(
            11_500_000,
            0,
            "wrong-gate-test-nature",
            "wrong-gate-test-nature",
        ),
        ability(
            33_402_224,
            0,
            "wrong-gate-test-nature",
            "wrong-gate-test-nature",
        ),
    ];
    let nature_hypothesis =
        "A test-only synthetic NATURE packet verifies The Wrong Gate's typed wearer bonus.";
    request.model_assumptions.insert(
        "wrong_gate.test_nature_packet".into(),
        nature_hypothesis.into(),
    );
    for (command_index, element, effect_id) in [
        (1, "NATURE", "wrong-gate-test-nature"),
        (2, "NATURE", "wrong-gate-test-nature"),
    ] {
        request.action_metadata.insert(
            command_index,
            crate::simulation::ResearchActionMetadata {
                hypothesis_id: "wrong_gate.test_nature_packet".into(),
                ability_id: effect_id.into(),
                damage_effect_id: effect_id.into(),
                action_gameplay_tags: Some(vec!["Ability.Skill".into()]),
                input_action_role: None,
                damage_type_ex: Some(format!("EExecutionDamageTypeEX::DAMAGE_TYPE_{element}")),
                damage_source_category: None,
                fixed_crit_rate: Some(crate::simulation::ResearchFixedCritRate::NoOverride),
                unbal_value: None,
                packet_ga_name_binding: None,
            },
        );
    }
    request
}

fn replacement_request(with_weapon: bool) -> GameScenarioRequest {
    let mut request = request(with_weapon);
    let mut mint = request.actors.pop().unwrap();
    let first_door = request.actors[0].gear_procs.clone();
    let mut second_edgar = request.actors[0].clone();
    second_edgar.slot = 1;
    second_edgar.weapon_id = if with_weapon { "weapon:fork_Door" } else { "" }.into();
    second_edgar.gear_procs = first_door;
    second_edgar.initial_hp = 20_000;
    second_edgar.initial_energy = 100;
    second_edgar.talent_levels = Some([1, 1, 8]);
    mint.slot = 2;
    mint.character_id = "character:1019".into();
    mint.weapon_id.clear();
    mint.gear_procs.clear();
    mint.talent_levels = Some([8, 8, 8]);
    request.actors.push(second_edgar);
    request.actors.push(mint);
    request.relative_duration_us = 16_000_000;
    request.actions = vec![
        ability(
            0,
            0,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 200_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        ability(
            11_500_000,
            1,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 12_000_000,
            target_actor_index: 2,
            cooldown_us: 0,
        },
        ability(
            15_000_000,
            2,
            "GA_Mint019_Skill",
            "GE_Player_Mint_Skill1_Damage_New",
        ),
    ];
    request.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
        timestamp_us: 14_000_000,
        actor_index: 1,
        delivered_damage: 20_000,
    }];
    request.model_assumptions.insert(
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    request
}

fn actor_damage(report: &serde_json::Value, actor: usize, time_us: u64) -> Vec<(u64, u64)> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["actor_index"] == actor && event["time_us"] == time_us)
        .map(|hit| {
            let f: ExactFraction = serde_json::from_value(hit["non_crit_damage"].clone()).unwrap();
            (f.numerator as u64, f.denominator as u64)
        })
        .collect()
}

fn assert_damage_factor(actual: (u64, u64), baseline: (u64, u64), factor: (u64, u64)) {
    assert_eq!(
        u128::from(actual.0) * u128::from(baseline.1) * u128::from(factor.1),
        u128::from(baseline.0) * u128::from(actual.1) * u128::from(factor.0),
        "actual {actual:?}, baseline {baseline:?}, expected factor {factor:?}"
    );
}

fn assert_damage_sequence_factor(
    actual: &[(u64, u64)],
    baseline: &[(u64, u64)],
    factor: (u64, u64),
) {
    assert_eq!(actual.len(), baseline.len());
    for (actual_hit, baseline_hit) in actual.iter().copied().zip(baseline.iter().copied()) {
        assert_damage_factor(actual_hit, baseline_hit, factor);
    }
}

fn synthetic_damage(
    report: &serde_json::Value,
    actor: usize,
    effect_id: &str,
    time_us: u64,
) -> (u64, u64) {
    let parsed: ExactFraction = serde_json::from_value(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|hit| {
                hit["actor_index"] == actor
                    && hit["damage_effect_id"] == effect_id
                    && hit["time_us"] == time_us
            })
            .unwrap()["non_crit_damage"]
            .clone(),
    )
    .unwrap();
    (parsed.numerator as u64, parsed.denominator as u64)
}

#[test]
fn actual_edgar_healing_refreshes_wrong_gate_and_damage_expires_half_open() {
    let equipped =
        crate::simulation::run_game_scenario(&request(true), &ExecutionLimits::default()).unwrap();
    let baseline =
        crate::simulation::run_game_scenario(&request(false), &ExecutionLimits::default()).unwrap();
    assert_eq!(
        equipped["execution_completed"], true,
        "{}",
        equipped["halt_diagnostic"]
    );
    assert_eq!(
        baseline["execution_completed"], true,
        "{}",
        baseline["halt_diagnostic"]
    );
    let mut active_effect_request = request(true);
    active_effect_request.relative_duration_us = 11_500_000;
    active_effect_request.actions.truncate(4);
    let active_effects =
        crate::simulation::run_game_scenario(&active_effect_request, &ExecutionLimits::default())
            .unwrap();
    let trigger_times = equipped["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "the_wrong_gate_triggered")
        .map(|event| event["time_us"].as_u64().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        trigger_times,
        (1..=10)
            .map(|s| s * 1_000_000 + 200_000)
            .collect::<Vec<_>>()
    );
    assert!(
        equipped["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "edgar_healing_tick")
            .all(|event| event["data"]["gross"] == 570)
    );
    assert_eq!(
        equipped["resolved_model_assumptions"][crate::weapons::wrong_gate::HYPOTHESIS_ID.as_str()],
        crate::weapons::wrong_gate::HYPOTHESIS_ID.description()
    );
    let active_buffs = crate::simulation::tests::support::final_buffs(&active_effects);
    let anima_buff = active_buffs
        .iter()
        .find(|buff| buff["id"] == "the_wrong_gate.anima_damage")
        .unwrap();
    let ally_buff = active_buffs
        .iter()
        .find(|buff| buff["id"] == "the_wrong_gate.ally_damage")
        .unwrap();
    assert_eq!(anima_buff["recipient"]["target"], 0);
    assert_eq!(ally_buff["recipient"]["target"], 1);
    assert_eq!(anima_buff["source_lifetime"], "snapshot");
    assert_eq!(anima_buff["stacks"], 1);
    assert_eq!(
        anima_buff["stat_modifiers"]["damage_bonus:ECharacterElementType::CHARACTER_ELEMENT_TYPE_NATURE"],
        "3/10"
    );
    assert_eq!(ally_buff["stat_modifiers"]["damage_bonus"], "3/20");

    let pre = actor_damage(&equipped, 1, 500_000);
    let during = actor_damage(&equipped, 1, 11_000_000);
    let expired = actor_damage(&equipped, 1, 30_200_000);
    let baseline_pre = actor_damage(&baseline, 1, 500_000);
    let baseline_mid = actor_damage(&baseline, 1, 11_000_000);
    let baseline_expired = actor_damage(&baseline, 1, 30_200_000);
    assert!(!pre.is_empty());
    assert_damage_sequence_factor(&pre, &baseline_pre, (1, 1));
    assert_damage_sequence_factor(
        &actor_damage(&equipped, 0, 0),
        &actor_damage(&baseline, 0, 0),
        (29, 25),
    );
    assert_eq!(baseline_pre, baseline_mid);
    assert_eq!(baseline_mid, baseline_expired);
    assert_damage_sequence_factor(&during, &baseline_mid, (25, 22));
    assert_damage_sequence_factor(&expired, &baseline_expired, (1, 1));
    let final_buffs = crate::simulation::tests::support::final_buffs(&equipped);
    assert_eq!(
        final_buffs
            .iter()
            .filter(|buff| buff["id"] == "the_wrong_gate.anima_damage")
            .count(),
        0,
        "{final_buffs:?}"
    );
    assert_eq!(
        final_buffs
            .iter()
            .filter(|buff| buff["id"] == "the_wrong_gate.ally_damage")
            .count(),
        0,
        "{final_buffs:?}"
    );
}

#[test]
fn wrong_gate_wearer_bonus_selects_typed_anima_and_expires_at_deadline() {
    let equipped = run_game_scenario(&nature_request(true), &ExecutionLimits::default()).unwrap();
    let baseline = run_game_scenario(&nature_request(false), &ExecutionLimits::default()).unwrap();
    assert_eq!(
        equipped["execution_completed"], true,
        "{}",
        equipped["halt_diagnostic"]
    );
    assert_eq!(
        baseline["execution_completed"], true,
        "{}",
        baseline["halt_diagnostic"]
    );
    assert_damage_factor(
        synthetic_damage(&equipped, 0, "wrong-gate-test-nature", 11_500_000),
        synthetic_damage(&baseline, 0, "wrong-gate-test-nature", 11_500_000),
        (377, 250),
    );
    assert_damage_factor(
        synthetic_damage(&equipped, 0, "wrong-gate-test-nature", 33_402_224),
        synthetic_damage(&baseline, 0, "wrong-gate-test-nature", 33_402_224),
        (29, 25),
    );
}

#[test]
fn wrong_gate_does_not_trigger_without_a_living_edgar_healing_recipient() {
    let mut failed_recipient = request(true);
    failed_recipient.actions.truncate(2);
    failed_recipient.actors[1].initial_hp = 1_000;
    failed_recipient
        .incoming_damage_events
        .push(crate::simulation::GameActorDamageEvent {
            timestamp_us: 700_000,
            actor_index: 1,
            delivered_damage: 1_000,
        });
    failed_recipient.model_assumptions.insert(
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report =
        crate::simulation::run_game_scenario(&failed_recipient, &ExecutionLimits::default())
            .unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_kind"] == "the_wrong_gate_triggered")
    );

    let mut no_heal = request(true);
    no_heal.actions = vec![ability(
        0,
        0,
        "GA_Edgar_Skill",
        "GE_Player_Edgar_Skill_Damage",
    )];
    no_heal.action_metadata.clear();
    no_heal.relative_duration_us = 1_000_000;
    let report =
        crate::simulation::run_game_scenario(&no_heal, &ExecutionLimits::default()).unwrap();
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_kind"] == "the_wrong_gate_triggered")
    );
}

#[test]
fn later_edgar_source_replaces_shared_effect_and_snapshot_survives_source_death() {
    let equipped =
        run_game_scenario(&replacement_request(true), &ExecutionLimits::default()).unwrap();
    let baseline =
        run_game_scenario(&replacement_request(false), &ExecutionLimits::default()).unwrap();
    assert_eq!(
        equipped["execution_completed"], true,
        "{}",
        equipped["halt_diagnostic"]
    );
    assert_eq!(
        baseline["execution_completed"], true,
        "{}",
        baseline["halt_diagnostic"]
    );
    assert_eq!(equipped["final_actor_states"][1]["hp"], 0);

    let triggers = equipped["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "the_wrong_gate_triggered")
        .collect::<Vec<_>>();
    assert_eq!(
        triggers
            .iter()
            .map(|event| event["time_us"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        (1..=10)
            .map(|second| second * 1_000_000 + 200_000)
            .chain([13_000_000, 14_000_000])
            .collect::<Vec<_>>()
    );
    assert_eq!(triggers[0]["to_actor"], 1);
    assert_eq!(triggers[10]["from_actor"], 1);
    assert_eq!(triggers[10]["to_actor"], 2);
    assert_eq!(triggers[11]["from_actor"], 1);
    assert_eq!(triggers[11]["to_actor"], 2);

    let buffs = crate::simulation::tests::support::final_buffs(&equipped);
    let wrong_gate_buffs = buffs
        .iter()
        .filter(|buff| buff["id"].as_str().unwrap().starts_with("the_wrong_gate."))
        .collect::<Vec<_>>();
    assert_eq!(wrong_gate_buffs.len(), 3);
    assert_eq!(
        wrong_gate_buffs
            .iter()
            .filter(|buff| buff["id"] == "the_wrong_gate.anima_damage")
            .count(),
        1
    );
    assert_eq!(
        wrong_gate_buffs
            .iter()
            .filter(|buff| buff["id"] == "the_wrong_gate.ally_damage")
            .count(),
        2
    );
    assert!(wrong_gate_buffs.iter().all(|buff| {
        buff["owner_actor"] == 1
            && buff["stacks"] == 1
            && buff["source_lifetime"] == "snapshot"
            && buff["duration"]["expires_at_us"] == 33_300_000
    }));
    let anima = wrong_gate_buffs
        .iter()
        .find(|buff| buff["id"] == "the_wrong_gate.anima_damage")
        .unwrap();
    assert_eq!(anima["recipient"]["target"], 1);
    let ally_recipients = wrong_gate_buffs
        .iter()
        .filter(|buff| buff["id"] == "the_wrong_gate.ally_damage")
        .map(|buff| buff["recipient"]["target"].as_u64().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ally_recipients, std::collections::BTreeSet::from([0, 2]));

    assert_damage_sequence_factor(
        &actor_damage(&equipped, 2, 15_000_000),
        &actor_damage(&baseline, 2, 15_000_000),
        (25, 22),
    );
}

#[test]
fn equipped_iroi_skill_healing_triggers_wrong_gate_before_later_damage() {
    let make_iroi_request = |with_weapon| {
        let mut req = request(with_weapon);
        req.relative_duration_us = 2_000_000;
        req.actors[0].character_id = "character:1075".into();
        req.actors[0].talent_levels = Some([1, 1, 1]);
        req.actors[0].initial_hp = 20_000;
        req.actors[0].initial_hp_max = 20_000;
        req.actors[1].initial_hp = 20_000;
        req.actors[1].initial_hp_max = 20_000;
        let mut skill = ability(
            0,
            0,
            "GA_Oneiroi_Skill",
            "GE_Player_Oneiroi_Skill_Gun_Damage",
        );
        let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut skill else {
            unreachable!();
        };
        *hit_delay_us = 100;
        req.actions = vec![
            skill,
            GameActionCommand::SwitchActor {
                timestamp_us: 300_000,
                target_actor_index: 1,
                cooldown_us: 0,
            },
            ability(
                500_000,
                1,
                "GA_Mint019_Skill",
                "GE_Player_Mint_Skill1_Damage_New",
            ),
        ];
        req
    };
    let equipped =
        run_game_scenario(&make_iroi_request(true), &ExecutionLimits::default()).unwrap();
    let baseline =
        run_game_scenario(&make_iroi_request(false), &ExecutionLimits::default()).unwrap();
    assert_eq!(
        equipped["execution_completed"], true,
        "{}",
        equipped["halt_diagnostic"]
    );
    assert_eq!(
        baseline["execution_completed"], true,
        "{}",
        baseline["halt_diagnostic"]
    );
    let triggers = equipped["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "the_wrong_gate_triggered")
        .collect::<Vec<_>>();
    assert_eq!(triggers.len(), 2);
    assert_eq!(
        triggers
            .iter()
            .map(|event| event["time_us"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [0, 0]
    );
    assert_eq!(triggers[0]["from_actor"], 0);
    assert_eq!(triggers[0]["to_actor"], 0);
    assert_eq!(triggers[0]["data"]["amount"], 1094);
    assert_eq!(triggers[1]["from_actor"], 0);
    assert_eq!(triggers[1]["to_actor"], 1);
    assert_eq!(triggers[1]["data"]["amount"], 1094);
    assert_damage_sequence_factor(
        &actor_damage(&equipped, 0, 100),
        &actor_damage(&baseline, 0, 100),
        (406, 275),
    );
    assert_damage_sequence_factor(
        &actor_damage(&equipped, 1, 500_000),
        &actor_damage(&baseline, 1, 500_000),
        (25, 22),
    );
    let buffs = crate::simulation::tests::support::final_buffs(&equipped);
    assert!(buffs.iter().any(|buff| {
        buff["id"] == "the_wrong_gate.anima_damage" && buff["recipient"]["target"] == 0
    }));
    assert!(buffs.iter().any(|buff| {
        buff["id"] == "the_wrong_gate.ally_damage" && buff["recipient"]["target"] == 1
    }));
}

#[test]
fn equipped_iroi_skill_then_single_burst_heal_caps_hp_but_wrong_gate_uses_gross() {
    let mut req = request(true);
    req.relative_duration_us = 2_000_000;
    req.actors[0].character_id = "character:1075".into();
    req.actors[0].talent_levels = Some([1, 1, 1]);
    req.actors[0].initial_hp = 19_000;
    req.actors[0].initial_hp_max = 20_000;
    req.actors[1].initial_hp = 19_000;
    req.actors[1].initial_hp_max = 20_000;
    let mut skill = ability(
        0,
        0,
        "GA_Oneiroi_Skill",
        "GE_Player_Oneiroi_Skill_Gun_Damage",
    );
    let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut skill else {
        unreachable!();
    };
    *hit_delay_us = 100;
    let mut burst = ability(
        1_000_000,
        0,
        "GA_Oneiroi_UltraSkill",
        "GE_Player_Oneiroi_UltraSkill_Damage",
    );
    let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut burst else {
        unreachable!();
    };
    *hit_delay_us = 100;
    req.actions = vec![
        skill,
        GameActionCommand::SwitchActor {
            timestamp_us: 300_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        ability(
            500_000,
            1,
            "GA_Mint019_Skill",
            "GE_Player_Mint_Skill1_Damage_New",
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 800_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        burst,
    ];

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    let trace = report["trace"].as_array().unwrap();
    let skill_heals = trace
        .iter()
        .filter(|event| event["event_kind"] == "iroi_skill_heal")
        .collect::<Vec<_>>();
    assert_eq!(skill_heals.len(), 2);
    // 1160 live ATK * .943 = 1093.88 -> 1094, filling 1000 missing HP.
    assert!(skill_heals.iter().all(|event| {
        event["data"]["gross"] == 1094
            && event["data"]["applied"] == 1000
            && event["data"]["overflow"] == 94
    }));
    assert!(skill_heals.iter().all(|event| event["time_us"] == 0));

    let burst_heals = trace
        .iter()
        .filter(|event| event["event_kind"] == "iroi_burst_heal")
        .collect::<Vec<_>>();
    assert_eq!(burst_heals.len(), 2);
    // 1160 live ATK * 1.413 = 1639.08 -> 1639. Skill filled both recipients,
    // so this single post-cast heal is all overheal while still reaching Wrong Gate.
    assert!(burst_heals.iter().all(|event| {
        event["data"]["gross"] == 1639
            && event["data"]["applied"] == 0
            && event["data"]["overflow"] == 1639
    }));
    assert!(
        burst_heals
            .iter()
            .all(|event| event["time_us"] == 1_000_100)
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    let direct_hit_index = trace
        .iter()
        .position(|event| {
            event["event_kind"] == "hit_damage"
                && event["hit_index"].as_u64().is_some_and(|hit_index| {
                    hits[hit_index as usize]["damage_effect_id"]
                        == "GE_Player_Oneiroi_UltraSkill_Damage"
                })
        })
        .unwrap();
    let burst_heal_index = trace
        .iter()
        .position(|event| event["event_kind"] == "iroi_burst_heal")
        .unwrap();
    assert!(direct_hit_index < burst_heal_index);

    let triggers = trace
        .iter()
        .filter(|event| event["event_kind"] == "the_wrong_gate_triggered")
        .collect::<Vec<_>>();
    assert_eq!(triggers.len(), 4);
    assert_eq!(
        triggers
            .iter()
            .map(|event| event["time_us"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [0, 0, 1_000_100, 1_000_100]
    );
    assert!(
        triggers[2..]
            .iter()
            .all(|event| event["data"]["amount"] == 1639)
    );
    assert!(report["resolved_model_assumptions"]["iroi.live_atk_healing_v1"].is_string());
    assert!(report["resolved_model_assumptions"]["weapons.the_wrong_gate_healing_v1"].is_string());
}

#[test]
fn unrelated_healer_and_rejected_iroi_skill_do_not_trigger_wrong_gate() {
    let mut unrelated = request(true);
    unrelated.relative_duration_us = 2_000_000;
    unrelated.actors[0].character_id = "character:1075".into();
    unrelated.actors[1].character_id = "character:1021".into();
    unrelated.actors[1].weapon_id.clear();
    unrelated.actors[1].gear_procs.clear();
    unrelated.actions = vec![
        GameActionCommand::SwitchActor {
            timestamp_us: 100_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        ability(
            200_000,
            1,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
        ),
    ];
    let report = run_game_scenario(&unrelated, &ExecutionLimits::default()).unwrap();
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| { event["event_kind"] == "edgar_healing_tick" })
    );
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| { event["event_kind"] == "the_wrong_gate_triggered" })
    );

    let mut rejected = request(true);
    rejected.actors[0].character_id = "character:1075".into();
    rejected.actions = vec![ability(
        0,
        0,
        "GA_Oneiroi_Skill",
        "GE_Player_Oneiroi_Skill_Syringe_Damage",
    )];
    let error = run_game_scenario(&rejected, &ExecutionLimits::default()).unwrap_err();
    assert!(error.contains("modeled Gun packet"), "{error}");
}
