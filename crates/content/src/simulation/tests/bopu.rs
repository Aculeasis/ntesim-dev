use super::*;
use crate::simulation::metadata::InputActionRole;

fn fixture(tier: u32) -> GameScenarioRequest {
    let ratios = [
        Ratio {
            numerator: 9,
            denominator: 50,
        },
        Ratio {
            numerator: 21,
            denominator: 100,
        },
        Ratio {
            numerator: 6,
            denominator: 25,
        },
        Ratio {
            numerator: 27,
            denominator: 100,
        },
        Ratio {
            numerator: 3,
            denominator: 10,
        },
    ];
    let damage_bonus = ratios[(tier as usize - 1).min(4)];
    let mut req = research_request();
    req.stochastic_crit = None;
    req.relative_duration_us = 40_000_000;
    req.actors[0].character_id = "character:1003".into();
    req.actors[0].weapon_id = "weapon:fork_bopu".into();
    req.actors[0].weapon_tier = tier;
    req.actors[0].gear_procs = vec![GearProcConfig::BopuDamage {
        damage_bonus,
        duration_us: 10_000_000,
        cooldown_us: 20_000_000,
    }];
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    let metadata = req.action_metadata[&0].clone();
    req.actions.clear();
    req.action_metadata.clear();
    req.action_metadata.insert(0, metadata);
    req
}

fn append(req: &mut GameScenarioRequest, time: u64, role: InputActionRole, delay: u64) {
    append_actor(req, 0, time, role, delay);
}

fn append_actor(
    req: &mut GameScenarioRequest,
    actor_index: usize,
    time: u64,
    role: InputActionRole,
    delay: u64,
) {
    let index = req.actions.len();
    req.actions.push(GameActionCommand::InvokeAbility {
        timestamp_us: time,
        actor_index,
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
    req.action_metadata.insert(index, metadata);
}

#[test]
fn bopu_buffs_only_support_skill_and_uses_separate_buff_and_cooldown_boundaries() {
    let mut req = fixture(1);
    for (time, role) in [
        (0, InputActionRole::Normal),
        (100, InputActionRole::Qte),
        (1_000_000, InputActionRole::Qte),
        (10_000_099, InputActionRole::Normal),
        (10_000_100, InputActionRole::Normal),
        (20_000_099, InputActionRole::Qte),
        (20_000_100, InputActionRole::Qte),
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
    // Base hit: 1000 ATK * 1.0 = 1000 damage.
    // T1 buff: +18% damage dealt -> 1000 * 1.18 = 1180 damage.
    // Recasts inside 20s ICD do not refresh buff or grant new buff.
    assert_eq!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|hit| hit["rounded_damage"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![1000, 1180, 1180, 1180, 1000, 1000, 1180, 1000]
    );
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "bopu_damage")
            .map(|event| event["time_us"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![100, 20_000_100]
    );
    assert_eq!(
        report["resolved_model_assumptions"][crate::weapons::bopu::HYPOTHESIS_ID.as_str()],
        crate::weapons::bopu::HYPOTHESIS_ID.description()
    );
}

#[test]
fn bopu_refinement_scaling_t1_through_t5() {
    let expected_damages = [1180, 1210, 1240, 1270, 1300];
    for (tier, expected) in (1..=5).zip(expected_damages) {
        let mut req = fixture(tier);
        append(&mut req, 0, InputActionRole::Qte, 0);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(
            report["hit_receipts"][0]["rounded_damage"]
                .as_u64()
                .unwrap(),
            expected,
            "refinement {tier} expected {expected}"
        );
    }
}

#[test]
fn bopu_off_field_hits_sample_self_buff_and_switching_keeps_cooldown() {
    let mut req = fixture(1);
    append(&mut req, 0, InputActionRole::Qte, 3_000_000);
    append(&mut req, 100, InputActionRole::Normal, 9_999_800);
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.weapon_id.clear();
    ally.gear_procs.clear();
    req.actors.push(ally);
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 1_000_000,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    append_actor(&mut req, 1, 2_000_000, InputActionRole::Normal, 0);
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 12_000_000,
        target_actor_index: 0,
        cooldown_us: 0,
    });
    append(&mut req, 13_000_000, InputActionRole::Qte, 0);
    append(&mut req, 20_000_000, InputActionRole::Qte, 0);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // Inspect authored synthetic packets separately from the engine Support
    // on Sakiri's re-entry, which still participates in admission and damage.
    // Hit 1: Actor 1 at 2.0s -> ally unbuffed 1000
    // Hit 2: Actor 0 delayed hit from t=0 at t=3.0s -> off-field wearer with active buff -> 1180
    // Hit 3: Actor 0 at 10.000s -> wearer buff active until 10.0001s -> 1180
    // Hit 4: Actor 0 at 13.0s -> buff expired, inside ICD -> 1000
    // Hit 5: Actor 0 at 20.0s -> ICD expired at 20.0s -> fresh buff -> 1180
    assert_eq!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["ability_id"] != "GA_Sagiri_QTE")
            .map(|hit| hit["rounded_damage"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![1000, 1180, 1180, 1000, 1180]
    );
}

#[test]
fn bopu_rejects_bad_equipment_and_zero_deadlines_and_failed_admission_grants_nothing() {
    let mut req = fixture(1);
    append(&mut req, 0, InputActionRole::Qte, 0);
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
    if let GearProcConfig::BopuDamage { cooldown_us, .. } = &mut bad.actors[0].gear_procs[0] {
        *cooldown_us = 0;
    }
    assert!(
        run_game_scenario(&bad, &ExecutionLimits::default())
            .unwrap_err()
            .contains("positive")
    );
    let mut bad = req.clone();
    if let GearProcConfig::BopuDamage { duration_us, .. } = &mut bad.actors[0].gear_procs[0] {
        *duration_us = 0;
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
            .any(|event| event["event_kind"] == "bopu_damage")
    );
}

fn projection_tables() -> std::collections::BTreeMap<String, serde_json::Value> {
    super::support::weapon_projection_tables()
}

fn projection_actor() -> GameActorConfig {
    serde_json::from_value(serde_json::json!({
        "slot":0,"character_id":"character:1003","initial_hp":1,"initial_hp_max":1,"initial_energy":0,
        "energy_capacity":100,"weapon_id":"weapon:fork_bopu","weapon_tier":1,
        "static_stats":{"base_atk":1,"atk":"1/1","hp":1,"def":1,
        "crit_rate":"0/1","crit_damage_bonus":"0/1"}
    }))
    .unwrap()
}

#[test]
fn bopu_projection_binds_five_curves() {
    // Hand-derived from frozen CT_ForkBuff rows (independent of the implementation):
    // buff_haokewu_Up T1..T5 = 0.18/0.21/0.24/0.27/0.30 = 9/50, 21/100, 6/25, 27/100, 3/10;
    // buff_haokewu_CD = 10.0s; buff_haokewu_CD2 = 20.0s.
    let tables = projection_tables();
    let expected_bonus = [(9, 50), (21, 100), (6, 25), (27, 100), (3, 10)];
    let mut actor = projection_actor();
    for (tier, (num, den)) in (1..=5).zip(expected_bonus) {
        actor.weapon_tier = tier as u32;
        actor.gear_procs.clear();
        let report = crate::weapons::bopu::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap();
        let proc = &report["compiled_proc"];
        assert_eq!(
            proc["damage_bonus"],
            serde_json::json!(format!("{}/{}", num, den))
        );
        assert_eq!(proc["duration_us"], 10_000_000);
        assert_eq!(proc["cooldown_us"], 20_000_000);
        assert_eq!(report["native_parameter_report"]["parameter_count"], 3);
        // The staged damage bonus is delivered by the live gear on_cast path and executed
        // in run_game_scenario, so the projection honestly claims validated execution.
        assert_eq!(report["runtime_execution_validated"], true);
        assert_eq!(actor.gear_procs.len(), 1);
    }
}

#[test]
fn bopu_projection_rejects_group_mismatch_and_changed_units() {
    let tables = projection_tables();
    let mut actor = projection_actor();
    // character:1019 is CHARACTER_GROUP_TYPE_TWO; fork_bopu requires FOUR.
    actor.character_id = "character:1019".into();
    assert!(
        crate::weapons::bopu::projection::project(&tables, 1, "ntesim-current", &mut actor, &[])
            .unwrap_err()
            .contains("group is incompatible")
    );
    actor.character_id = "character:1003".into();
    let mut changed = tables.clone();
    let data_list = changed
        .get_mut("DT_ForkUpgradeStarDataTable")
        .unwrap()["rows"]["upgradestar_pack_fork_bopu_1"]["DataList"]
        .as_array_mut()
        .unwrap();
    data_list
        .iter_mut()
        .find(|entry| entry["NameID"] == "buff_haokewu_Up")
        .unwrap()["bIsPercent"] = serde_json::json!(false);
    assert!(
        crate::weapons::bopu::projection::project(&changed, 1, "ntesim-current", &mut actor, &[])
            .unwrap_err()
            .contains("unit hint")
    );
}

#[test]
fn bopu_zone_math_hand_derived_t1_t5() {
    fn gcd(mut a: u128, mut b: u128) -> u128 {
        while b != 0 {
            let rest = a % b;
            a = b;
            b = rest;
        }
        a
    }
    fn reduced(n: u128, d: u128) -> (u128, u128) {
        let g = gcd(n, d);
        (n / g, d / g)
    }
    // Damage-zone multipliers 1 + bonus for T1..T5, hand-derived exact fractions,
    // no core calls. Frozen buff_haokewu_Up: 0.18/0.21/0.24/0.27/0.30.
    let expected = [(59, 50), (121, 100), (31, 25), (127, 100), (13, 10)];
    let bonuses = [(9, 50), (21, 100), (6, 25), (27, 100), (3, 10)];
    for ((n, d), (en, ed)) in bonuses.iter().zip(expected.iter()) {
        assert_eq!(reduced(d + n, *d), (*en, *ed));
    }
    // The T1 misreading 6/5 = 1.20 is rejected; the honest multiplier is 59/50 = 1.18.
    assert_ne!(reduced(50 + 9, 50), (6, 5));
}
