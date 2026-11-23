use super::*;
use crate::simulation::metadata::InputActionRole;

fn fixture() -> GameScenarioRequest {
    super::redirect_weapon_fixture(
        "weapon:fork_nonos",
        vec![GearProcConfig::NorosAtk {
            atk_bonus: Ratio {
                numerator: 3,
                denominator: 25,
            },
            duration_us: 10_000_000,
            cooldown_us: 20_000_000,
        }],
        Ratio {
            numerator: 2000,
            denominator: 1,
        },
    )
}

fn append(req: &mut GameScenarioRequest, time: u64, role: InputActionRole, delay: u64) {
    // Input role triggers the cast buff; a Skill-classified packet alone cannot.
    super::append_synthetic_cast(
        req,
        time,
        role,
        delay,
        Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"),
    );
}

#[test]
fn noros_buffs_only_successful_skill_and_uses_separate_buff_and_cooldown_boundaries() {
    let mut req = fixture();
    for (time, role) in [
        (0, InputActionRole::Normal),
        (100, InputActionRole::RedirectSkill),
        (1_000_000, InputActionRole::RedirectSkill),
        (10_000_099, InputActionRole::Normal),
        (10_000_100, InputActionRole::Normal),
        (20_000_099, InputActionRole::RedirectSkill),
        (20_000_100, InputActionRole::RedirectSkill),
        (30_000_100, InputActionRole::Ultimate),
    ] {
        append(&mut req, time, role, 0);
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // 2000 total ATK +1000 base ATK*.12, then.5DEF. Recasts inside ICD never refresh.
    assert_eq!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|hit| hit["rounded_damage"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![1000, 1060, 1060, 1060, 1000, 1000, 1060, 1000]
    );
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "noros_atk")
            .map(|event| event["time_us"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![100, 20_000_100]
    );
    assert_eq!(
        report["resolved_model_assumptions"][crate::weapons::noros::HYPOTHESIS_ID.as_str()],
        crate::weapons::noros::HYPOTHESIS_ID.description()
    );
}

#[test]
fn noros_off_field_hits_sample_self_buff_and_switching_keeps_cooldown() {
    let mut req = fixture();
    append(&mut req, 0, InputActionRole::RedirectSkill, 3_000_000);
    append(&mut req, 100, InputActionRole::Normal, 9_999_900);
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
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 12_000_000,
        target_actor_index: 0,
        cooldown_us: 0,
    });
    append(&mut req, 13_000_000, InputActionRole::RedirectSkill, 0);
    append(&mut req, 20_000_000, InputActionRole::RedirectSkill, 0);
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
        vec![1000, 1060, 1000, 1000, 1060]
    );
}

#[test]
fn noros_rejects_bad_equipment_and_zero_deadlines_and_failed_admission_grants_nothing() {
    let mut req = fixture();
    append(&mut req, 0, InputActionRole::RedirectSkill, 0);
    let mut bad = req.clone();
    bad.actors[0].weapon_id.clear();
    assert!(
        run_game_scenario(&bad, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires equipped")
    );
    let mut bad = req.clone();
    bad.actors[0]
        .gear_procs
        .push(req.actors[0].gear_procs[0].clone());
    assert!(
        run_game_scenario(&bad, &ExecutionLimits::default())
            .unwrap_err()
            .contains("duplicates")
    );
    let mut bad = req.clone();
    if let GearProcConfig::NorosAtk { cooldown_us, .. } = &mut bad.actors[0].gear_procs[0] {
        *cooldown_us = 0;
    }
    assert!(
        run_game_scenario(&bad, &ExecutionLimits::default())
            .unwrap_err()
            .contains("positive")
    );
    req.actors[0].initial_energy = 0;
    if let GameActionCommand::InvokeAbility { energy_cost, .. } = &mut req.actions[0] {
        *energy_cost = 1;
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_kind"] == "noros_atk")
    );
}
