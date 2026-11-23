use super::*;
use crate::simulation::metadata::InputActionRole;

fn fixture() -> GameScenarioRequest {
    let mut req = research_request();
    req.stochastic_crit = None;
    req.relative_duration_us = 40_000_000;
    req.actors[0].weapon_id = "weapon:fork_lunarphase".into();
    req.actors[0].gear_procs = vec![GearProcConfig::LunarPhase {
        atk_bonus: Ratio {
            numerator: 1,
            denominator: 5,
        },
        cosmos_bonus: Ratio {
            numerator: 8,
            denominator: 25,
        },
        def_ignore: Ratio {
            numerator: 3,
            denominator: 25,
        },
        duration_us: 20_000_000,
    }];
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].static_stats.crit_rate = Ratio::zero();
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
fn lunarphase_applies_atk_bonus_and_cosmos_damage_and_def_ignore_on_ultimate() {
    let mut req = fixture();
    append(&mut req, 0, 0, InputActionRole::Normal, "COSMOS", 0);
    append(
        &mut req,
        0,
        1_000_000,
        InputActionRole::Ultimate,
        "COSMOS",
        0,
    );
    append(&mut req, 0, 5_000_000, InputActionRole::Normal, "NATURE", 0);
    append(
        &mut req,
        0,
        15_000_000,
        InputActionRole::Ultimate,
        "COSMOS",
        0,
    );
    append(
        &mut req,
        0,
        30_000_000,
        InputActionRole::Normal,
        "COSMOS",
        0,
    );
    append(
        &mut req,
        0,
        38_903_489,
        InputActionRole::Normal,
        "COSMOS",
        0,
    );

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // Base ATK 1000 + 20% = 1200 ATK. Without buff: 1200 * 0.5 = 600.
    // Under Ultimate: 12% DEF ignore -> effective DEF 950.4, factor 25/47.
    // Cosmos hit: 1200 * (25/47) * 1.32 = 39600/47 = 842.55 -> rounded 843.
    // Nature hit: 1200 * (25/47) = 30000/47 = 638.30 -> rounded 638.
    // Ultimate recast at 15s resets 20s duration to 35s; hit at 35s has expired (precedes equal-time hit).
    let hits = report["hit_receipts"].as_array().unwrap();
    let hit_values = hits
        .iter()
        .map(|hit| hit["rounded_damage"].as_u64().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(hit_values, vec![600, 843, 638, 843, 843, 600]);
    assert_eq!(
        report["resolved_model_assumptions"][crate::weapons::lunarphase::HYPOTHESIS_ID.as_str()],
        crate::weapons::lunarphase::HYPOTHESIS_ID.description()
    );
    assert_eq!(
        report["resolved_model_assumptions"][crate::gear::CAST_BUFF_HYPOTHESIS_ID.as_str()],
        crate::gear::CAST_BUFF_HYPOTHESIS_ID.description()
    );
}

#[test]
fn lunarphase_is_self_actor_and_does_not_buff_active_ally() {
    let mut req = fixture();
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.weapon_id = String::new();
    ally.gear_procs.clear();
    ally.static_stats.crit_rate = Ratio::zero();
    ally.static_stats.crit_damage_bonus = Ratio::zero();
    req.actors.push(ally);

    append(
        &mut req,
        0,
        1_000_000,
        InputActionRole::Ultimate,
        "COSMOS",
        0,
    );
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 2_000_000,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    append(&mut req, 1, 3_000_000, InputActionRole::Normal, "COSMOS", 0);
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 4_000_000,
        target_actor_index: 0,
        cooldown_us: 0,
    });
    append(&mut req, 0, 5_000_000, InputActionRole::Normal, "COSMOS", 0);

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    // Wearer hit under ultimate: 843.
    assert_eq!(hits[0]["rounded_damage"], 843);
    // Ally hit: base 1000 ATK * 0.5 DEF = 500 (no lunarphase buffs).
    let ally_receipt = hits.iter().find(|h| h["actor_index"] == 1).unwrap();
    assert_eq!(ally_receipt["rounded_damage"], 500);
    // Wearer hit after switch-back: 843.
    let wearer_after = hits
        .iter()
        .find(|h| h["actor_index"] == 0 && h["time_us"] == 5_000_000)
        .unwrap();
    assert_eq!(wearer_after["rounded_damage"], 843);
}

#[test]
fn lunarphase_rejects_bad_equipment_and_does_not_trigger_on_failed_admission() {
    let mut req = fixture();
    append(&mut req, 0, 0, InputActionRole::Ultimate, "COSMOS", 0);
    let limits = ExecutionLimits::default();
    let mut bad = req.clone();
    bad.actors[0].weapon_id = "weapon:fork_nonos".into();
    assert!(
        run_game_scenario(&bad, &limits)
            .unwrap_err()
            .contains("requires equipped weapon:fork_lunarphase")
    );

    let duplicate = req.actors[0].gear_procs[0].clone();
    bad = req.clone();
    bad.actors[0].gear_procs.push(duplicate);
    assert!(
        run_game_scenario(&bad, &limits)
            .unwrap_err()
            .contains("duplicates")
    );

    req.actors[0].initial_energy = 0;
    if let GameActionCommand::InvokeAbility { energy_cost, .. } = &mut req.actions[0] {
        *energy_cost = 100;
    }
    let report = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["event_kind"] == "lunarphase_proc")
    );
}
