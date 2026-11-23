use super::*;
use crate::simulation::metadata::InputActionRole;

fn fixture() -> GameScenarioRequest {
    let mut req = research_request();
    req.stochastic_crit = None;
    req.relative_duration_us = 40_000_000;
    req.actors[0].character_id = "character:1036".into();
    req.actors[0].weapon_id = "weapon:fork_DemonBlade".into();
    req.actors[0].gear_procs = vec![GearProcConfig::DemonBlade {
        crit_rate_bonus: Ratio {
            numerator: 4,
            denominator: 25,
        },
        crit_damage_per_stack: Ratio {
            numerator: 9,
            denominator: 100,
        },
        duration_us: 15_000_000,
        icd_us: 300_000,
    }];
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].static_stats.crit_rate = Ratio::one();
    req.actors[0].static_stats.crit_damage_bonus = Ratio::zero();
    req.targets[0].resistance = Ratio::zero();
    req.targets[0].defense = 1080;
    let metadata = req.action_metadata[&0].clone();
    req.actions.clear();
    req.action_metadata.clear();
    req.action_metadata.insert(0, metadata);
    req
}

fn append(
    req: &mut GameScenarioRequest,
    actor: usize,
    time: u64,
    role: InputActionRole,
    element: &str,
    delay: u64,
) {
    let index = req.actions.len();
    req.actions.push(GameActionCommand::InvokeAbility {
        timestamp_us: time,
        actor_index: actor,
        ability_id: "caller-test".into(),
        damage_effect_id: "caller-effect".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: delay,
    });
    let mut metadata = req.action_metadata[&0].clone();
    metadata.input_action_role = Some(role);
    metadata.action_gameplay_tags = Some(vec!["Ability.Melee".into()]);
    metadata.damage_source_category =
        Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A".into());
    metadata.damage_type_ex = Some(format!("EExecutionDamageTypeEX::DAMAGE_TYPE_{element}"));
    req.action_metadata.insert(index, metadata);
}

#[test]
fn demon_blade_applies_crit_rate_and_stacks_crit_damage_on_incantation_hits() {
    let mut req = fixture();
    // Hit 1 at 1.0s: Incantation DMG -> stack 1 (ready_at = 1.3s)
    append(
        &mut req,
        0,
        1_000_000,
        InputActionRole::Normal,
        "INCANTATION",
        0,
    );
    // Hit 2 at 1.1s: Incantation DMG inside ICD (< 1.3s) -> no new stack
    append(
        &mut req,
        0,
        1_100_000,
        InputActionRole::Normal,
        "INCANTATION",
        0,
    );
    // Hit 3 at 1.4s: Incantation DMG outside ICD (>= 1.3s) -> stack 2 (ready_at = 1.7s)
    append(
        &mut req,
        0,
        1_400_000,
        InputActionRole::Normal,
        "INCANTATION",
        0,
    );
    // Hit 4 at 1.8s: -> stack 3 (ready_at = 2.1s)
    append(
        &mut req,
        0,
        1_800_000,
        InputActionRole::Normal,
        "INCANTATION",
        0,
    );
    // Hit 5 at 2.2s: -> stack 4 (ready_at = 2.5s)
    append(
        &mut req,
        0,
        2_200_000,
        InputActionRole::Normal,
        "INCANTATION",
        0,
    );
    // Hit 6 at 2.6s: -> stack 5 (ready_at = 2.9s)
    append(
        &mut req,
        0,
        2_600_000,
        InputActionRole::Normal,
        "INCANTATION",
        0,
    );
    // Hit 7 at 3.0s: -> stack 6 (ready_at = 3.3s)
    append(
        &mut req,
        0,
        3_000_000,
        InputActionRole::Normal,
        "INCANTATION",
        0,
    );
    // Hit 8 at 3.4s: -> stack 7 (cap reached, ready_at = 3.7s)
    append(
        &mut req,
        0,
        3_400_000,
        InputActionRole::Normal,
        "INCANTATION",
        0,
    );
    // Hit 9 at 3.8s: -> cap 7 remains, duration refreshed to 3.8s + 15s = 18.8s
    append(
        &mut req,
        0,
        3_800_000,
        InputActionRole::Normal,
        "INCANTATION",
        0,
    );
    // Hit 10 at 19.0s: buff expired (18.8s < 19.0s) -> stack 1
    append(
        &mut req,
        0,
        19_000_000,
        InputActionRole::Normal,
        "INCANTATION",
        0,
    );

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );

    let receipts = report["hit_receipts"].as_array().unwrap();
    assert_eq!(receipts.len(), 10);

    // Hit 1 had 0 prior stacks: crit rate has +16% bonus
    let hit1 = &receipts[0];
    assert_eq!(hit1["crit_rate"], "29/25"); // 1.0 + 4/25 = 29/25, reported

    let stack_events: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "demon_blade_crit_stack")
        .collect();
    // Hits 1, 3, 4, 5, 6, 7, 8, 9, 10 triggered a stack event (hit 2 was in ICD)
    assert_eq!(stack_events.len(), 9);

    assert_eq!(
        report["resolved_model_assumptions"][crate::weapons::demon_blade::HYPOTHESIS_ID.as_str()],
        crate::weapons::demon_blade::HYPOTHESIS_ID.description()
    );
}

#[test]
fn demon_blade_requires_incantation_element_and_does_not_trigger_on_other_elements() {
    let mut req = fixture();
    // Hit with COSMOS element instead of INCANTATION
    append(&mut req, 0, 1_000_000, InputActionRole::Normal, "COSMOS", 0);

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );

    let stack_events: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "demon_blade_crit_stack")
        .collect();
    assert_eq!(stack_events.len(), 0);
}

#[test]
fn demon_blade_rejects_bad_equipment_and_does_not_trigger_on_failed_admission() {
    let limits = ExecutionLimits::default();
    let mut bad_weapon = fixture().actors[0].clone();
    bad_weapon.weapon_id = "weapon:fork_nonos".into();
    let err = crate::gear::validate_config(&bad_weapon, 0, 40_000_000, &limits).unwrap_err();
    assert!(err.contains("Ravenous Blade"));

    let mut zero_duration = fixture().actors[0].clone();
    zero_duration.gear_procs = vec![GearProcConfig::DemonBlade {
        crit_rate_bonus: Ratio {
            numerator: 4,
            denominator: 25,
        },
        crit_damage_per_stack: Ratio {
            numerator: 9,
            denominator: 100,
        },
        duration_us: 0,
        icd_us: 300_000,
    }];
    let err = crate::gear::validate_config(&zero_duration, 0, 40_000_000, &limits).unwrap_err();
    assert!(err.contains("positive duration"));

    let mut zero_icd = fixture().actors[0].clone();
    zero_icd.gear_procs = vec![GearProcConfig::DemonBlade {
        crit_rate_bonus: Ratio {
            numerator: 4,
            denominator: 25,
        },
        crit_damage_per_stack: Ratio {
            numerator: 9,
            denominator: 100,
        },
        duration_us: 15_000_000,
        icd_us: 0,
    }];
    let err = crate::gear::validate_config(&zero_icd, 0, 40_000_000, &limits).unwrap_err();
    assert!(err.contains("positive cooldown"));
}
