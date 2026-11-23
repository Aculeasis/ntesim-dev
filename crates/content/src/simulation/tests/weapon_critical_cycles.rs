use super::*;

struct CriticalStackOracle<'a> {
    coefficients: &'a [(&'a str, u128)],
    cap: u128,
    lifetime_us: u64,
    element_milli: u128,
    stack_ten_thousandths: u128,
    event: &'a str,
    source: &'a str,
}

fn check_native_cycles(source: &str, oracle: CriticalStackOracle<'_>) {
    let mut req = crate::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    let stats = &mut req.actors[0].static_stats;
    stats.base_atk = 1000;
    stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    stats.crit_damage_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };
    stats.elemental_bonus = Ratio::zero();
    stats.elemental_bonuses.clear();
    check_off_field_snapshot(&req, &oracle);
    let mut baseline = req.clone();
    baseline.actors[0].gear_procs.clear();
    let plain = run_game_scenario(&baseline, &ExecutionLimits::default()).unwrap();
    let result = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        result["execution_completed"], true,
        "{}",
        result["halt_diagnostic"]
    );
    assert_eq!(plain["execution_completed"], true);
    let hits = result["hit_receipts"].as_array().unwrap();
    let bare = plain["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), bare.len());
    let mut last_group = None;
    let mut after = 0;
    let mut visible = 0;
    let mut fresh = 0;
    let mut peak = 0;
    let mut groups = 0;
    let mut crescendo = 0;
    let mut crescendo_until = 0;
    for (hit, reference) in hits.iter().zip(bare) {
        assert_eq!(hit["time_us"], reference["time_us"]);
        if hit["actor_index"] != 0 {
            assert_eq!(hit["non_crit_damage"], reference["non_crit_damage"]);
            assert_eq!(hit["crit_damage"], reference["crit_damage"]);
            continue;
        }
        let time = hit["time_us"].as_u64().unwrap();
        if last_group != Some(time) {
            if last_group.is_none_or(|previous| time >= previous + oracle.lifetime_us) {
                after = 0;
                fresh += 1;
            }
            // Same-time native siblings all settle before the proc callback.
            visible = after;
            peak = peak.max(visible);
            after = (after + 1).min(oracle.cap);
            last_group = Some(time);
            groups += 1;
        }
        let effect = hit["damage_effect_id"].as_str().unwrap();
        if time >= crescendo_until {
            crescendo = 0;
        }
        let coefficient = oracle
            .coefficients
            .iter()
            .find(|(id, _)| *id == effect)
            .unwrap()
            .1;
        // Aurelia's native bullet callbacks add 1% of base ATK per hit,
        // including between same-time siblings. This is independent of Moon's
        // delayed, once-per-group crit-damage stack callback.
        let noncrit = ExactFraction::new(
            coefficient * (1250 + oracle.element_milli) * (100 + crescendo),
            100_000,
        )
        .unwrap();
        let crit = noncrit
            .mul(ExactFraction::new(15000 + visible * oracle.stack_ten_thousandths, 10000).unwrap())
            .unwrap();
        let actual: ExactFraction = serde_json::from_value(hit["non_crit_damage"].clone()).unwrap();
        let actual_crit: ExactFraction =
            serde_json::from_value(hit["crit_damage"].clone()).unwrap();
        assert_eq!(actual, noncrit, "time {time}, {effect}");
        assert_eq!(actual_crit, crit, "time {time}, visible stacks {visible}");
        assert_eq!(hit["critical"], true);
        assert_eq!(hit["rounded_damage"], crit.round_u64().unwrap());
        if req.actors[0].character_id == "character:1070"
            && effect == "GE_Player_Mitsuki_Melee3_Damage"
        {
            crescendo = (crescendo + 1).min(10);
            crescendo_until = time + 5_000_000;
        }
    }
    assert_eq!(fresh, 2);
    assert_eq!(peak, oracle.cap);
    let grants = result["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["event_kind"] == oracle.event)
        .count();
    assert_eq!(grants, groups);
    assert!(
        !crate::simulation::tests::support::final_buffs(&result)
            .iter()
            .any(|buff| buff["source"] == oracle.source)
    );
}

fn check_off_field_snapshot(req: &GameScenarioRequest, oracle: &CriticalStackOracle<'_>) {
    let departure = req
        .actions
        .iter()
        .position(|action| {
            matches!(
                action,
                GameActionCommand::SwitchActor {
                    target_actor_index: 1,
                    ..
                }
            )
        })
        .unwrap();
    let GameActionCommand::SwitchActor { timestamp_us, .. } = req.actions[departure] else {
        unreachable!()
    };
    let mut prefix = req.clone();
    prefix.actions.truncate(departure + 1);
    prefix
        .action_metadata
        .retain(|index, _| *index <= departure);
    prefix.relative_duration_us = timestamp_us + 500_000;
    let snapshot = run_game_scenario(&prefix, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        snapshot["execution_completed"], true,
        "{}",
        snapshot["halt_diagnostic"]
    );
    assert_eq!(snapshot["final_buff_state"]["active_actor_index"], 1);
    let snapshot_buffs = crate::simulation::tests::support::final_buffs(&snapshot);
    let retained = snapshot_buffs
        .iter()
        .find(|buff| buff["source"] == oracle.source)
        .unwrap();
    assert_eq!(
        retained["stacks"].as_u64().map(u128::from),
        Some(oracle.cap)
    );
}

#[test]
fn moon_native_sibling_packets_share_icd_and_rebuild_critical_stacks_after_expiry() {
    let coefficients = [
        ("GE_Player_Mitsuki_Melee1_Damage", 347),
        ("GE_Player_Mitsuki_Melee2_Damage", 630),
        ("GE_Player_Mitsuki_Melee3_Damage", 300),
        ("GE_Player_Mitsuki_Melee4_Damage", 99),
    ];
    for (tier, element_milli, stack_ten_thousandths) in [
        (1, 120, 200),
        (2, 145, 240),
        (3, 170, 280),
        (4, 195, 320),
        (5, 210, 360),
    ] {
        let source = include_str!("../../../../../examples/stellar-veil-critical-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        check_native_cycles(
            &source,
            CriticalStackOracle {
                coefficients: &coefficients,
                cap: 10,
                lifetime_us: 5_000_000,
                element_milli,
                stack_ten_thousandths,
                event: "moon_crit_stack",
                source: "weapon:fork_moon",
            },
        );
    }
}

#[test]
fn demon_blade_native_support_and_normals_share_capped_critical_windows() {
    let coefficients = [
        ("GE_Player_Sagiri_Melee1_Damage", 403),
        ("GE_Player_Sagiri_Melee2_Damage", 138),
        ("GE_Player_Sagiri_Melee3_Damage", 757),
        ("GE_Player_Sagiri_Melee8_Damage", 16),
        ("GE_Player_Sagiri_Melee4_Damage", 716),
        ("GE_Player_Sagiri_Melee5_Damage", 331),
        ("GE_Player_Sagiri_Melee6_Damage", 35),
        ("GE_Player_Sagiri_Melee7_Damage", 1062),
        ("GE_Player_Sagiri_QTE1_Damage", 2000),
    ];
    for (tier, stack_ten_thousandths) in [(1, 900), (2, 1125), (3, 1350), (4, 1575), (5, 1800)] {
        let source = include_str!("../../../../../examples/ravenous-blade-critical-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        check_native_cycles(
            &source,
            CriticalStackOracle {
                coefficients: &coefficients,
                cap: 7,
                lifetime_us: 15_000_000,
                element_milli: 0,
                stack_ten_thousandths,
                event: "demon_blade_crit_stack",
                source: "weapon:fork_DemonBlade",
            },
        );
    }
}
