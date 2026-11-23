use super::*;
use crate::simulation::metadata::InputActionRole;

fn fixture(tier: u32) -> GameScenarioRequest {
    let off_field_atk_base = [
        Ratio {
            numerator: 1,
            denominator: 10,
        },
        Ratio {
            numerator: 11,
            denominator: 100,
        },
        Ratio {
            numerator: 3,
            denominator: 25,
        },
        Ratio {
            numerator: 13,
            denominator: 100,
        },
        Ratio {
            numerator: 7,
            denominator: 50,
        },
    ];
    let off_field_atk_per_stack = [
        Ratio {
            numerator: 1,
            denominator: 50,
        },
        Ratio {
            numerator: 23,
            denominator: 1000,
        },
        Ratio {
            numerator: 13,
            denominator: 500,
        },
        Ratio {
            numerator: 29,
            denominator: 1000,
        },
        Ratio {
            numerator: 4,
            denominator: 125,
        },
    ];
    let on_field_psyche_base = [
        Ratio {
            numerator: 3,
            denominator: 25,
        },
        Ratio {
            numerator: 7,
            denominator: 50,
        },
        Ratio {
            numerator: 4,
            denominator: 25,
        },
        Ratio {
            numerator: 9,
            denominator: 50,
        },
        Ratio {
            numerator: 1,
            denominator: 5,
        },
    ];
    let on_field_psyche_per_stack = [
        Ratio {
            numerator: 1,
            denominator: 50,
        },
        Ratio {
            numerator: 23,
            denominator: 1000,
        },
        Ratio {
            numerator: 13,
            denominator: 500,
        },
        Ratio {
            numerator: 29,
            denominator: 1000,
        },
        Ratio {
            numerator: 4,
            denominator: 125,
        },
    ];
    let i = (tier as usize - 1).min(4);
    let mut req = research_request();
    req.stochastic_crit = None;
    req.relative_duration_us = 40_000_000;

    // Actor 0: Jiuyuan (1055), wears Blow up the Crowd (fork_BitGame)
    req.actors[0].character_id = "character:1055".into();
    req.actors[0].weapon_id = "weapon:fork_BitGame".into();
    req.actors[0].weapon_tier = tier;
    req.actors[0].gear_procs = vec![GearProcConfig::BitGame {
        off_field_atk_base: off_field_atk_base[i],
        off_field_atk_per_stack: off_field_atk_per_stack[i],
        off_field_icd_us: 2_000_000,
        on_field_psyche_base: on_field_psyche_base[i],
        on_field_psyche_per_stack: on_field_psyche_per_stack[i],
    }];
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actors[0].static_stats.crit_damage_bonus = Ratio::zero();
    req.actors[0].static_stats.damage_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();

    // Actor 1: Mint (1019)
    let mut actor1 = req.actors[0].clone();
    actor1.slot = 1;
    actor1.character_id = "character:1019".into();
    actor1.weapon_id = "weapon:fork_lingganzhongjiezhe".into();
    actor1.weapon_tier = 1;
    actor1.gear_procs.clear();
    req.actors.push(actor1);

    req.model_assumptions.insert(
        "test.synthetic_action".into(),
        "Test-only explicit metadata for synthetic actions.".into(),
    );
    req.active_actor_index = 0;
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();

    req.actions.clear();
    req.action_metadata.clear();
    req
}

fn append_action(
    req: &mut GameScenarioRequest,
    actor_idx: usize,
    time: u64,
    role: InputActionRole,
    delay: u64,
    damage_type: Option<&str>,
    category: Option<&str>,
) {
    let index = req.actions.len();
    req.actions.push(GameActionCommand::InvokeAbility {
        timestamp_us: time,
        actor_index: actor_idx,
        ability_id: format!("caller-test-{index}"),
        array_index: 0,
        damage_effect_id: format!("caller-effect-{index}"),
        atk_rate: Ratio {
            numerator: 1,
            denominator: 1,
        },
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: delay,
    });
    req.action_metadata.insert(
        index,
        crate::simulation::ResearchActionMetadata {
            hypothesis_id: "test.synthetic_action".into(),
            ability_id: format!("caller-test-{index}"),
            damage_effect_id: format!("caller-effect-{index}"),
            action_gameplay_tags: Some(vec!["Ability.Melee".into()]),
            input_action_role: Some(role),
            damage_type_ex: Some(
                damage_type
                    .unwrap_or("EExecutionDamageTypeEX::DAMAGE_TYPE_NORMAL")
                    .into(),
            ),
            damage_source_category: category.map(str::to_string),
            fixed_crit_rate: Some(crate::simulation::ResearchFixedCritRate::NoOverride),
            unbal_value: None,
            packet_ga_name_binding: None,
        },
    );
}

#[test]
fn bit_game_on_field_psyche_bonus_and_normal_attack_icd_stacks() {
    let mut req = fixture(1);

    // Hit 1 at t=0: Basic attack, Psyche element -> receives base 12% Psyche bonus (1120), gains stack 1
    append_action(
        &mut req,
        0,
        0,
        InputActionRole::Normal,
        0,
        Some("EExecutionDamageTypeEX::DAMAGE_TYPE_PSYCHE"),
        Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"),
    );

    // Hit 2 at t=200_000 (< 300_000 ICD): Receives base 12% + 1 stack (2%) = 14% (1140), ICD blocks 2nd stack
    append_action(
        &mut req,
        0,
        200_000,
        InputActionRole::Normal,
        0,
        Some("EExecutionDamageTypeEX::DAMAGE_TYPE_PSYCHE"),
        Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"),
    );

    // Hit 3 at t=400_000 (>= 300_000 ICD): Receives base 12% + 1 stack (2%) = 14% (1140), gains stack 2
    append_action(
        &mut req,
        0,
        400_000,
        InputActionRole::Normal,
        0,
        Some("EExecutionDamageTypeEX::DAMAGE_TYPE_PSYCHE"),
        Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"),
    );

    // Hit 4 at t=800_000: Receives base 12% + 2 stacks (4%) = 16% (1160)
    append_action(
        &mut req,
        0,
        800_000,
        InputActionRole::Normal,
        0,
        Some("EExecutionDamageTypeEX::DAMAGE_TYPE_PSYCHE"),
        Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"),
    );

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 4);
    assert_eq!(hits[0]["rounded_damage"], 1120);
    assert_eq!(hits[1]["rounded_damage"], 1140);
    assert_eq!(hits[2]["rounded_damage"], 1140);
    assert_eq!(hits[3]["rounded_damage"], 1160);

    let stack_events: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "bit_game_on_field_stack")
        .collect();
    assert_eq!(stack_events.len(), 3);
}

#[test]
fn bit_game_off_field_base_atk_and_damage_stacks() {
    let mut req = fixture(1);

    // Cast 3 delayed abilities by Jiuyuan (actor 0) before switching off-field:
    // Hit 1 lands at t=100_000
    append_action(&mut req, 0, 0, InputActionRole::Normal, 100_000, None, None);
    // Hit 2 lands at t=501_000
    append_action(
        &mut req,
        0,
        1_000,
        InputActionRole::Normal,
        500_000,
        None,
        None,
    );
    // Hit 3 lands at t=2_202_000
    append_action(
        &mut req,
        0,
        2_000,
        InputActionRole::Normal,
        2_200_000,
        None,
        None,
    );

    // Switch to Mint (actor 1) at t=10_000 so Jiuyuan is off-field
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 10_000,
        target_actor_index: 1,
        cooldown_us: 1_000_000,
    });

    // Mint attacks at t=50_000: Active character has base 10% ATK buff -> 1000 * 1.10 = 1100
    append_action(&mut req, 1, 50_000, InputActionRole::Normal, 0, None, None);

    // Mint attacks at t=600_000: Base 10% ATK + 1 stack (2%) = 12% ATK -> 1000 * 1.12 = 1120
    append_action(&mut req, 1, 600_000, InputActionRole::Normal, 0, None, None);

    // Mint attacks at t=2_300_000: Base 10% ATK + 2 stacks (4%) = 14% ATK -> 1000 * 1.14 = 1140
    append_action(
        &mut req,
        1,
        2_300_000,
        InputActionRole::Normal,
        0,
        None,
        None,
    );

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["rejected_action"], serde_json::Value::Null);
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 6);
    assert_eq!(hits[0]["rounded_damage"], 1100); // Mint hit at t=50_000
    assert_eq!(hits[1]["rounded_damage"], 1000); // Jiuyuan hit 1 at t=100_000
    assert_eq!(hits[2]["rounded_damage"], 1000); // Jiuyuan hit 2 at t=501_000
    assert_eq!(hits[3]["rounded_damage"], 1120); // Mint hit at t=600_000
    assert_eq!(hits[4]["rounded_damage"], 1000); // Jiuyuan hit 3 at t=2_202_000
    assert_eq!(hits[5]["rounded_damage"], 1140); // Mint hit at t=2_300_000

    let off_field_stack_events: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "bit_game_off_field_stack")
        .collect();
    assert_eq!(off_field_stack_events.len(), 2);
}

#[test]
fn bit_game_resets_on_switch() {
    let mut req = fixture(1);

    // 1. Jiuyuan builds 1 on-field Psyche stack at t=0
    append_action(
        &mut req,
        0,
        0,
        InputActionRole::Normal,
        0,
        Some("EExecutionDamageTypeEX::DAMAGE_TYPE_PSYCHE"),
        Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"),
    );

    // 2. Jiuyuan prepares delayed hit to land while off-field at t=601_000
    append_action(
        &mut req,
        0,
        1_000,
        InputActionRole::Normal,
        600_000,
        None,
        None,
    );

    // 3. Switch to Mint at t=10_000 -> clears Jiuyuan on-field stacks, applies off-field base ATK to Mint
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 10_000,
        target_actor_index: 1,
        cooldown_us: 1_000_000,
    });

    // 4. Mint attacks at t=700_000: 10% base ATK + 2% stack = 12% ATK (1120)
    append_action(&mut req, 1, 700_000, InputActionRole::Normal, 0, None, None);

    // 5. Switch back to Jiuyuan at t=1_600_000 -> clears off-field base and off-field stack buffs
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 1_600_000,
        target_actor_index: 0,
        cooldown_us: 1_000_000,
    });

    // 6. Jiuyuan attacks at t=1_700_000: On-field Psyche stacks were reset on switch out,
    // so Jiuyuan only has base 12% Psyche bonus (1120), not 14%
    append_action(
        &mut req,
        0,
        1_700_000,
        InputActionRole::Normal,
        0,
        Some("EExecutionDamageTypeEX::DAMAGE_TYPE_PSYCHE"),
        Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"),
    );

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["rejected_action"], serde_json::Value::Null);
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits[0]["rounded_damage"], 1120); // Jiuyuan initial
    assert_eq!(hits[1]["rounded_damage"], 1000); // Jiuyuan delayed hit
    assert_eq!(hits[2]["rounded_damage"], 1120); // Mint with off-field ATK
    assert_eq!(hits[3]["rounded_damage"], 1120); // Jiuyuan reset, only base Psyche bonus
}

#[test]
fn bit_game_t1_through_t5_scaling() {
    let expected_psyche_base = [1120, 1140, 1160, 1180, 1200];
    for tier in 1..=5 {
        let mut req = fixture(tier);
        append_action(
            &mut req,
            0,
            0,
            InputActionRole::Normal,
            0,
            Some("EExecutionDamageTypeEX::DAMAGE_TYPE_PSYCHE"),
            Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"),
        );
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        let hit = report["hit_receipts"][0]["rounded_damage"]
            .as_u64()
            .unwrap();
        assert_eq!(hit, expected_psyche_base[(tier - 1) as usize]);
    }
}

#[test]
fn bit_game_off_field_stacks_cap_at_four() {
    // Cap falsifier: 5 off-field hits spaced beyond the 2.0s ICD would grant 5 stacks
    // without a cap. Hand-derived: max_stacks = 4, so the 5th hit adds nothing.
    // Mint's ATK at t=9_000_000 = 1000 x (1 + 1/10 + 4 x 1/50) = 1000 x 59/50 = 1180;
    // without the cap it would be 1000 x (1 + 1/10 + 5/50) = 1200.
    let mut req = fixture(1);
    for (index, land_at) in [100_000, 2_300_000, 4_500_000, 6_700_000, 8_900_000]
        .iter()
        .enumerate()
    {
        let cast_at = (index as u64) * 1_000;
        append_action(
            &mut req,
            0,
            cast_at,
            InputActionRole::Normal,
            land_at - cast_at,
            None,
            None,
        );
    }
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 10_000,
        target_actor_index: 1,
        cooldown_us: 1_000_000,
    });
    append_action(
        &mut req,
        1,
        9_000_000,
        InputActionRole::Normal,
        0,
        None,
        None,
    );

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["rejected_action"], serde_json::Value::Null);
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 6);
    let mint_hit = &hits[5];
    assert_eq!(mint_hit["rounded_damage"], 1180);
    assert_ne!(mint_hit["rounded_damage"], 1200);

    let stack_events: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "bit_game_off_field_stack")
        .collect();
    assert_eq!(
        stack_events.len(),
        5,
        "all 5 ICD-spaced hits attempt a grant"
    );
    assert_eq!(
        stack_events[4]["data"]["stacks"], 4,
        "the 5th grant stays at the 4-stack cap"
    );
    assert_eq!(stack_events[4]["data"]["max_stacks"], 4);
}

#[test]
fn bit_game_on_field_stacks_cap_at_ten() {
    // Cap falsifier: 11 on-field Psyche Basic hits spaced beyond the 0.3s ICD would grant
    // 11 stacks without a cap. Hand-derived: max_stacks = 10.
    // Hit 11 uses 10 stacks: 1000 x (1 + 3/25 + 10 x 1/50) = 1000 x 33/25 = 1320;
    // without the cap it would be 1000 x (1 + 3/25 + 11/50) = 1340.
    let mut req = fixture(1);
    for i in 0..11 {
        append_action(
            &mut req,
            0,
            (i as u64) * 400_000,
            InputActionRole::Normal,
            0,
            Some("EExecutionDamageTypeEX::DAMAGE_TYPE_PSYCHE"),
            Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"),
        );
    }

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 11);
    assert_eq!(hits[10]["rounded_damage"], 1320);
    assert_ne!(hits[10]["rounded_damage"], 1340);

    let stack_events: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "bit_game_on_field_stack")
        .collect();
    assert_eq!(
        stack_events.len(),
        11,
        "all 11 ICD-spaced hits attempt a grant"
    );
    assert_eq!(
        stack_events[10]["data"]["stacks"], 10,
        "the 11th grant stays at the 10-stack cap"
    );
    assert_eq!(stack_events[10]["data"]["max_stacks"], 10);
}
