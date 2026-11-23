use super::*;
use crate::simulation::metadata::InputActionRole;

fn fixture() -> GameScenarioRequest {
    let mut req = research_request();
    req.relative_duration_us = 30_000_000;
    req.actors[0].weapon_id = "weapon:fork_GoldWool".into();
    req.actors[0].gear_procs = vec![GearProcConfig::GoldWool {
        lakshana_bonus: Ratio {
            numerator: 1,
            denominator: 5,
        },
        crit_damage_bonus: Ratio {
            numerator: 2,
            denominator: 5,
        },
        duration_us: 20_000_000,
    }];
    req.actors[0].static_stats.damage_bonus = Ratio {
        numerator: 1,
        denominator: 10,
    };
    req.actors[0].static_stats.crit_rate = Ratio::one();
    req.targets[0].resistance = Ratio::zero();
    let metadata = req.action_metadata[&0].clone();
    req.actions.clear();
    req.action_metadata.clear();
    req.action_metadata.insert(0, metadata);
    req
}

fn append(
    req: &mut GameScenarioRequest,
    time: u64,
    role: InputActionRole,
    element: &str,
    delay: u64,
) {
    let index = req.actions.len();
    req.actions.push(GameActionCommand::InvokeAbility {
        timestamp_us: time,
        actor_index: 0,
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
fn gold_wool_combines_element_bonus_with_input_cast_refresh_and_exact_expiry() {
    let mut req = fixture();
    for (time, role, element) in [
        (0, InputActionRole::Normal, "LAKSHANA"),
        (1_000_000, InputActionRole::RedirectSkill, "NATURE"),
        (5_000_000, InputActionRole::Ultimate, "LAKSHANA"),
        (21_000_000, InputActionRole::Normal, "LAKSHANA"),
        (28_903_488, InputActionRole::Normal, "LAKSHANA"),
        (28_903_489, InputActionRole::Normal, "LAKSHANA"),
    ] {
        append(&mut req, time, role, element, 0);
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // ATK1000 * DEF.5 * (1 + .1 general + .2 Lakshana) * (1 + 1 CD + .4 active CD).
    // Skill's Nature packet gets CD but no Lakshana bonus; Ultimate refreshes, never stacks.
    assert_eq!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|hit| hit["rounded_damage"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![1300, 1320, 1560, 1560, 1560, 1300]
    );
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "gold_wool_crit_damage")
            .count(),
        2
    );
    assert!(
        !crate::simulation::tests::support::final_buffs(&report)
            .iter()
            .any(|buff| buff["id"] == "gold_wool.crit_damage")
    );
    assert_eq!(
        report["resolved_model_assumptions"][crate::weapons::gold_wool::HYPOTHESIS_ID.as_str()],
        crate::weapons::gold_wool::HYPOTHESIS_ID.description()
    );
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    let noncritical = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        noncritical["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|hit| hit["rounded_damage"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![650, 550, 650, 650, 650, 650]
    );
}

#[test]
fn gold_wool_delayed_packets_sample_expiry_and_preserve_off_field_ownership() {
    let mut req = fixture();
    append(
        &mut req,
        0,
        InputActionRole::RedirectSkill,
        "LAKSHANA",
        3_000_000,
    );
    append(
        &mut req,
        100,
        InputActionRole::Normal,
        "LAKSHANA",
        19_999_900,
    );
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1003".into();
    ally.weapon_id.clear();
    ally.gear_procs.clear();
    req.actors.push(ally);
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 1_000_000,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    req.actions.push(GameActionCommand::InvokeAbility {
        timestamp_us: 2_000_000,
        actor_index: 1,
        ability_id: "GA_Sagiri_Melee".into(),
        damage_effect_id: "GE_Player_Sagiri_Melee1_Damage".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 0,
    });
    req.stochastic_crit
        .as_mut()
        .unwrap()
        .eligible_effects
        .insert(
            "character:1003".into(),
            BTreeSet::from(["GE_Player_Sagiri_Melee1_Damage".into()]),
        );
    req.stochastic_crit
        .as_mut()
        .unwrap()
        .actor_sequences
        .insert("character:1003".into(), 55);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["ability_id"] != "GA_Sagiri_QTE")
            .map(|hit| hit["rounded_damage"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![1100, 1560, 1300]
    );
}

#[test]
fn gold_wool_rejects_invalid_configs_and_failed_casts_grant_nothing() {
    let mut req = fixture();
    append(&mut req, 0, InputActionRole::Ultimate, "LAKSHANA", 0);
    let limits = ExecutionLimits::default();
    let mut bad = req.clone();
    bad.actors[0].weapon_id.clear();
    assert!(
        run_game_scenario(&bad, &limits)
            .unwrap_err()
            .contains("requires equipped")
    );
    let mut bad = req.clone();
    bad.actors[0]
        .gear_procs
        .push(req.actors[0].gear_procs[0].clone());
    assert!(
        run_game_scenario(&bad, &limits)
            .unwrap_err()
            .contains("duplicates")
    );
    let mut bad = req.clone();
    if let GearProcConfig::GoldWool { duration_us, .. } = &mut bad.actors[0].gear_procs[0] {
        *duration_us = 0;
    }
    assert!(
        run_game_scenario(&bad, &limits)
            .unwrap_err()
            .contains("positive")
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
            .any(|event| event["event_kind"] == "gold_wool_crit_damage")
    );
}
