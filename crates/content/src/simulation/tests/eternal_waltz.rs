use super::*;
use crate::simulation::metadata::InputActionRole;

fn fixture() -> GameScenarioRequest {
    let mut request = research_request();
    request.relative_duration_us = 20_000_000;
    request.actors[0].character_id = "character:1039".into();
    request.actors[0].weapon_id = "weapon:fork_Arachne".into();
    request.actors[0].gear_procs = vec![GearProcConfig::EternalWaltz {
        max_hp_bonus: Ratio {
            numerator: 1,
            denominator: 5,
        },
        mental_bonus: Ratio {
            numerator: 1,
            denominator: 10,
        },
        duration_us: 10_000_000,
    }];
    request.actors[0].static_stats.base_atk = 1000;
    request.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    request.actors[0].static_stats.damage_bonus = Ratio::zero();
    request.actors[0].static_stats.elemental_bonus = Ratio::zero();
    request.targets[0].resistance = Ratio::zero();
    request.targets[0].defense = 1080;
    let metadata = request.action_metadata[&0].clone();
    request.actions.clear();
    request.action_metadata.clear();
    request.action_metadata.insert(0, metadata);
    request.stochastic_crit = None;
    request
}

fn append(
    request: &mut GameScenarioRequest,
    actor: usize,
    time: u64,
    role: InputActionRole,
    element: &str,
    energy_cost: u64,
    delay: u64,
) {
    let index = request.actions.len();
    request.actions.push(GameActionCommand::InvokeAbility {
        timestamp_us: time,
        actor_index: actor,
        ability_id: "caller-test".into(),
        damage_effect_id: "caller-effect".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: delay,
    });
    let mut metadata = request.action_metadata[&0].clone();
    metadata.input_action_role = Some(role);
    metadata.action_gameplay_tags = Some(vec!["Ability.Melee".into()]);
    metadata.damage_source_category =
        Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A".into());
    metadata.damage_type_ex = Some(format!("EExecutionDamageTypeEX::DAMAGE_TYPE_{element}"));
    request.action_metadata.insert(index, metadata);
}

#[test]
fn eternal_waltz_triggers_only_after_admitted_ultimate_and_keeps_wearer_scope_across_switches() {
    let mut request = fixture();
    let mut ally = request.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.weapon_id.clear();
    ally.gear_procs.clear();
    request.actors.push(ally);
    append(&mut request, 0, 0, InputActionRole::Normal, "PSYCHE", 0, 0);
    append(
        &mut request,
        0,
        1_000_000,
        InputActionRole::Ultimate,
        "PSYCHICALLY",
        0,
        0,
    );
    request.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 2_000_000,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    append(
        &mut request,
        1,
        3_000_000,
        InputActionRole::Normal,
        "PSYCHE",
        0,
        0,
    );
    request.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 4_000_000,
        target_actor_index: 0,
        cooldown_us: 0,
    });
    append(
        &mut request,
        0,
        5_000_000,
        InputActionRole::Normal,
        "PSYCHE",
        0,
        0,
    );
    append(
        &mut request,
        0,
        6_000_000,
        InputActionRole::Normal,
        "PSYCHE",
        0,
        0,
    );
    append(
        &mut request,
        0,
        7_000_000,
        InputActionRole::Normal,
        "PSYCHICALLY",
        0,
        0,
    );
    append(
        &mut request,
        0,
        12_000_000,
        InputActionRole::Normal,
        "PSYCHICALLY",
        0,
        0,
    );

    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    let damage = hits
        .iter()
        .map(|hit| hit["rounded_damage"].as_u64().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(damage, vec![500, 1100, 500, 500, 500, 1100, 1000]);
    assert_eq!(
        report["resolved_model_assumptions"][crate::weapons::eternal_waltz::HYPOTHESIS_ID.as_str()],
        crate::weapons::eternal_waltz::HYPOTHESIS_ID.description()
    );
    assert_eq!(
        report["resolved_model_assumptions"][crate::gear::CAST_BUFF_HYPOTHESIS_ID.as_str()],
        crate::gear::CAST_BUFF_HYPOTHESIS_ID.description()
    );
}

#[test]
fn eternal_waltz_refreshes_and_expires_before_equal_time_hits() {
    let mut request = fixture();
    append(
        &mut request,
        0,
        0,
        InputActionRole::Ultimate,
        "PSYCHICALLY",
        0,
        0,
    );
    append(
        &mut request,
        0,
        8_000_000,
        InputActionRole::Ultimate,
        "PSYCHICALLY",
        0,
        0,
    );
    append(
        &mut request,
        0,
        17_000_000,
        InputActionRole::Normal,
        "PSYCHICALLY",
        0,
        0,
    );
    append(
        &mut request,
        0,
        22_417_366,
        InputActionRole::Normal,
        "PSYCHICALLY",
        0,
        0,
    );

    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits[2]["rounded_damage"], 1100);
    assert_eq!(hits[3]["rounded_damage"], 1000);
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "eternal_waltz_mental")
            .count(),
        2
    );
}

#[test]
fn eternal_waltz_rejects_foreign_weapon_and_failed_ultimate_does_not_trigger() {
    let mut request = fixture();
    append(
        &mut request,
        0,
        0,
        InputActionRole::Ultimate,
        "PSYCHE",
        0,
        0,
    );
    let mut wrong_weapon = request.clone();
    wrong_weapon.actors[0].weapon_id = "weapon:fork_twinbirds".into();
    assert!(
        run_game_scenario(&wrong_weapon, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires equipped weapon:fork_Arachne")
    );

    request.actors[0].initial_energy = 0;
    if let GameActionCommand::InvokeAbility { energy_cost, .. } = &mut request.actions[0] {
        *energy_cost = 100;
    }
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_kind"] == "eternal_waltz_mental")
    );
}
