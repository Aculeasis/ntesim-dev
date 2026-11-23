use super::*;

fn crimson_request(hit_count: usize, spacing_us: u64) -> GameScenarioRequest {
    // Caller-supplied proc parameters bind the frozen curves:
    // CT_Equipmentbuff Incantation_4_1 = 0.06 per stack (3/50),
    // Incantation_4_2 = 6 max stacks, Incantation_4_3 = 10s per stack.
    let mut req = research_request();
    req.stochastic_crit = None;
    req.relative_duration_us = 30_000_000;
    req.actors[0].set_id = Some("set:Suit4".into());
    req.actors[0].gear_procs = vec![GearProcConfig::CrimsonAtk {
        atk_bonus_per_stack: Ratio {
            numerator: 3,
            denominator: 50,
        },
        duration_us: 10_000_000,
        max_stacks: 6,
    }];
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actors[0].static_stats.crit_damage_bonus = Ratio::zero();
    req.actors[0].static_stats.damage_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonuses.clear();
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    let action = req.actions[0].clone();
    let metadata = req.action_metadata[&0].clone();
    req.actions.clear();
    req.action_metadata.clear();
    for i in 0..hit_count {
        let mut action = action.clone();
        if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut action {
            *timestamp_us = i as u64 * spacing_us;
        }
        req.actions.push(action);
        req.action_metadata.insert(i, metadata.clone());
    }
    req
}

// Hand-derived oracles for every test below come from the live ATK zone:
// ATK_live(n) = ATK_static + base_atk * n * 3/50 = 1000 + 1151 * n * 3/50
// (COMBAT_FORMULA_CONTRACT section 2), rounded half up to integer damage:
// n=0 -> 1000, n=1 -> 1069, n=2 -> 1138, n=3 -> 1207,
// n=4 -> 1276, n=5 -> 1345, n=6 -> 1414.
// A multiply-static model would give 1060/1120 instead of 1069/1138.

fn stack_count(report: &serde_json::Value) -> u32 {
    crate::simulation::tests::support::final_buffs(report)
        .iter()
        .filter(|buff| buff["id"] == "crimson.atk")
        .map(|buff| buff["stacks"].as_u64().unwrap() as u32)
        .sum()
}

fn owner_stack_count(report: &serde_json::Value, actor_index: usize) -> u32 {
    crate::simulation::tests::support::final_buffs(report)
        .iter()
        .filter(|buff| buff["id"] == "crimson.atk" && buff["owner_actor"] == actor_index)
        .map(|buff| buff["stacks"].as_u64().unwrap() as u32)
        .sum()
}

fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a.max(1)
}

fn total_atk_percent(report: &serde_json::Value) -> (i64, i64) {
    let (mut num, mut den) = (0i64, 1i64);
    for buff in crate::simulation::tests::support::final_buffs(report)
        .iter()
        .filter(|buff| buff["id"] == "crimson.atk")
    {
        let atk: crate::simulation::ExactFraction =
            serde_json::from_value(buff["stat_modifiers"]["atk_percent"].clone()).unwrap();
        let n = atk.numerator as i64;
        let d = atk.denominator as i64;
        let (new_num, new_den) = (num * d + n * den, den * d);
        let g = gcd(new_num.abs(), new_den.abs());
        num = new_num / g;
        den = new_den / g;
    }
    (num, den)
}

fn stack_events(report: &serde_json::Value) -> usize {
    report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "crimson_atk_stack")
        .count()
}

fn hit_damages(report: &serde_json::Value) -> Vec<u64> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|hit| hit["rounded_damage"].as_u64().unwrap())
        .collect()
}

#[test]
fn crimson_hit_grants_stack_and_raises_atk() {
    let mut req = crimson_request(2, 1_000_000);
    req.relative_duration_us = 5_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // Two hits grant two independent stacks: aggregate 2 * 3/50 = 3/25.
    assert_eq!(stack_count(&report), 2);
    assert_eq!(total_atk_percent(&report), (3, 25));
    assert_eq!(stack_events(&report), 2);
    // The spawning hit is not buffed by its own stack (grant after delivery);
    // the second hit sees one stack: 1000 + 1151 * 3/50 = 1069.06 -> 1069.
    // Multiplicative static would give 1000 * 1.06 = 1060.
    assert_eq!(hit_damages(&report), vec![1000, 1069]);
    assert_eq!(
        report["resolved_model_assumptions"][crate::sets::crimson::HYPOTHESIS_ID.as_str()],
        crate::sets::crimson::HYPOTHESIS_ID.description()
    );
}

#[test]
fn crimson_caps_at_six_stacks() {
    let mut req = crimson_request(8, 1_000_000);
    req.relative_duration_us = 9_999_999;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // 8 hits, cap 6, each stack lasts 10s so none expire yet.
    assert_eq!(stack_count(&report), 6);
    // Aggregate 6 * 3/50 = 9/25.
    assert_eq!(total_atk_percent(&report), (9, 25));
    // The last two hits see the cap: 1000 + 1151 * 6 * 3/50 = 1414.36 -> 1414.
    assert_eq!(
        hit_damages(&report),
        vec![1000, 1069, 1138, 1207, 1276, 1345, 1414, 1414]
    );
    // apply() pushes a trace event even for grants discarded at the cap.
    assert_eq!(stack_events(&report), 8);
}

#[test]
fn crimson_stacks_expire_independently_after_ten_seconds() {
    let mut req = crimson_request(3, 1_000_000);
    req.relative_duration_us = 25_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // Hits at 0s, 1s, 2s; stacks expire at 10s, 11s, 12s. All 3 expired by 25s.
    assert_eq!(stack_count(&report), 0);

    // Midpoint: independent expiry pins the 0s grant as dead while the 1s and
    // 2s grants survive. Refresh-all or a shared 10s window would give 3.
    let mut req = crimson_request(3, 1_000_000);
    req.relative_duration_us = 10_500_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(stack_count(&report), 2);
    assert_eq!(total_atk_percent(&report), (3, 25));
}

#[test]
fn crimson_stacks_accumulate_across_hits() {
    let mut req = crimson_request(3, 1_000_000);
    req.relative_duration_us = 5_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // 3 hits grant 3 stacks; aggregate 3 * 3/50 = 9/50.
    assert_eq!(stack_count(&report), 3);
    assert_eq!(total_atk_percent(&report), (9, 50));
    // 1138 for the third hit falsifies the multiplicative model (1120).
    assert_eq!(hit_damages(&report), vec![1000, 1069, 1138]);
}

#[test]
fn crimson_replay_matches() {
    let req = crimson_request(3, 1_000_000);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let replay = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hit_receipts"], replay["hit_receipts"]);
    assert_eq!(
        report["final_buff_state"]["instances"],
        replay["final_buff_state"]["instances"]
    );
    assert_eq!(stack_events(&report), stack_events(&replay));
}

#[test]
fn crimson_rejects_wrong_set_and_zero_duration() {
    let mut req = crimson_request(1, 1_000_000);
    req.actors[0].set_id = Some("set:Suit10".into());
    let err = run_game_scenario(&req, &ExecutionLimits::default()).unwrap_err();
    assert!(
        err.contains("Suit4"),
        "wrong set must name set:Suit4: {err}"
    );

    let mut req = crimson_request(1, 1_000_000);
    req.actors[0].gear_procs = vec![GearProcConfig::CrimsonAtk {
        atk_bonus_per_stack: Ratio {
            numerator: 3,
            denominator: 50,
        },
        duration_us: 0,
        max_stacks: 6,
    }];
    let err = run_game_scenario(&req, &ExecutionLimits::default()).unwrap_err();
    assert!(
        err.contains("duration"),
        "zero duration must be named: {err}"
    );
}

#[test]
fn crimson_discards_at_cap_without_refresh() {
    // 10 hits at 1s spacing; horizon 14.5s. Grants 0s..5s fill the cap;
    // grants 6s..9s are discarded. Under discard-at-cap only the 5s grant
    // survives (expires at 15s). Refresh-at-cap would give 6 stacks;
    // evict-oldest would give 5.
    let mut req = crimson_request(10, 1_000_000);
    req.relative_duration_us = 14_500_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(stack_count(&report), 1);
    assert_eq!(total_atk_percent(&report), (3, 50));
}

#[test]
fn crimson_switch_preserves_stacks_and_keeps_self_recipient() {
    // Evidence: "Effect remains active when the character is off-field"
    // and BuffRecipient::SelfActor in apply().
    let mut req = crimson_request(2, 1_000_000);
    req.relative_duration_us = 5_000_000;
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.gear_procs.clear();
    req.actors.push(ally);
    let mut ally_hit = req.actions[0].clone();
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index,
        ..
    } = &mut ally_hit
    {
        *timestamp_us = 3_000_000;
        *actor_index = 1;
    }
    req.actions.push(ally_hit);
    req.action_metadata
        .insert(3, req.action_metadata[&0].clone());
    // The switch must sort before the ally hit: actions are timestamp-ordered.
    req.actions.insert(
        2,
        GameActionCommand::SwitchActor {
            timestamp_us: 2_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // Stacks granted by the wearer survive the switch off-field.
    assert_eq!(owner_stack_count(&report, 0), 2);
    // The ally cannot grant or receive: no stacks owned by actor 1, and the
    // ally hit at 3s is unbuffed.
    assert_eq!(owner_stack_count(&report, 1), 0);
    let ally_damage = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["actor_index"] == 1)
        .map(|hit| hit["rounded_damage"].as_u64().unwrap());
    assert_eq!(ally_damage, Some(1000));
}

#[test]
fn crimson_expiry_precedes_equal_time_hit() {
    // Hit at 0s grants a stack expiring at exactly 10s; a second hit lands at
    // the same 10s boundary. Expiry is processed before the equal-time hit,
    // so the second hit sees zero stacks.
    let mut req = crimson_request(2, 10_000_000);
    req.relative_duration_us = 15_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(hit_damages(&report), vec![1000, 1000]);
    // The second hit grants its own stack, alive at the 15s horizon.
    assert_eq!(stack_count(&report), 1);
}

#[test]
fn crimson_rejects_zero_max_stacks() {
    let mut req = crimson_request(1, 1_000_000);
    req.actors[0].gear_procs = vec![GearProcConfig::CrimsonAtk {
        atk_bonus_per_stack: Ratio {
            numerator: 3,
            denominator: 50,
        },
        duration_us: 10_000_000,
        max_stacks: 0,
    }];
    let err = run_game_scenario(&req, &ExecutionLimits::default()).unwrap_err();
    assert!(
        err.contains("1..=64"),
        "zero max_stacks must be named: {err}"
    );
}
