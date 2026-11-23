use super::*;
use serde_json::Value;

fn compile(source: &str) -> GameScenarioRequest {
    crate::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request
}
fn normalize(req: &mut GameScenarioRequest) {
    for actor in &mut req.actors {
        actor.static_stats.base_atk = 1000;
        actor.static_stats.atk = Ratio {
            numerator: 2000,
            denominator: 1,
        };
        actor.static_stats.damage_bonus = Ratio::zero();
        actor.static_stats.elemental_bonus = Ratio::zero();
        actor.static_stats.elemental_bonuses.clear();
    }
}
fn fraction(value: &Value) -> ExactFraction {
    ExactFraction::from_ratio(serde_json::from_value(value.clone()).unwrap()).unwrap()
}
fn attack(hit: &Value) -> ExactFraction {
    fraction(&hit["non_crit_damage"])
        .div(fraction(&hit["scaling_rate"]))
        .unwrap()
}
fn upgrades(report: &Value) -> Vec<&Value> {
    report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "good_boy_control_atk")
        .collect()
}

#[test]
fn good_boy_control_upgrades_once_per_cast_at_all_ranks_and_preserves_original_windows() {
    for (tier, base, boosted) in [
        (1, 2100, 2160),
        (2, 2115, 2185),
        (3, 2130, 2210),
        (4, 2145, 2235),
        (5, 2160, 2260),
    ] {
        let source = include_str!("../../../../../examples/good-boy-control-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        let mut req = compile(&source);
        normalize(&mut req);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 240);
        assert_eq!(upgrades(&report).len(), 2);
        let mut immune = req.clone();
        immune.targets[1].control_immune = true;
        // Airborne input is not a successful Ultimate-control receipt.
        for target in &mut immune.targets {
            target.airborne_until_us = 1_000_000_000;
        }
        let baseline = run_game_scenario(&immune, &ExecutionLimits::default()).unwrap();
        assert_eq!(baseline["execution_completed"], true);
        assert!(upgrades(&baseline).is_empty());
        let hits = report["hit_receipts"].as_array().unwrap();
        let bare = baseline["hit_receipts"].as_array().unwrap();
        assert_eq!(hits.len(), bare.len());
        let mut controls = 0;
        let mut after_control_expiry = false;
        let mut last_control_expiry = 0;
        let windows: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "good_boy_team_atk")
            .map(|event| event["time_us"].as_u64().unwrap())
            .collect();
        for (hit, original) in hits.iter().zip(bare) {
            if let Some(applied) = hit["control_applications"].as_array() {
                assert_eq!(applied.len(), 1);
                assert_eq!(applied[0]["target_index"], 1);
                assert_eq!(hit["non_crit_damage"], original["non_crit_damage"]);
                last_control_expiry = applied[0]["expires_at_us"].as_u64().unwrap();
                controls += 1;
            }
            if hit["non_crit_damage"] != original["non_crit_damage"] {
                // Ordinary Sakiri Burst adds 300 flat ATK to the other actor.
                let ordinary = if hit["actor_index"] == 0 { 0 } else { 300 };
                assert_eq!(
                    attack(original),
                    ExactFraction::new(base + ordinary, 1).unwrap()
                );
                assert_eq!(
                    attack(hit),
                    ExactFraction::new(boosted + ordinary, 1).unwrap()
                );
                let time = hit["time_us"].as_u64().unwrap();
                assert!(
                    windows
                        .iter()
                        .any(|start| support::world_time_at(&report, *start)
                            <= support::world_time_at(&report, time)
                            && support::world_time_at(&report, time)
                                < support::world_time_at(&report, *start) + 20_000_000)
                );
                after_control_expiry |= time >= last_control_expiry;
            }
        }
        assert_eq!(controls, 2);
        assert!(after_control_expiry);
        let mut two_recipients = req.clone();
        two_recipients.targets[0].control_immune = false;
        let double = run_game_scenario(&two_recipients, &ExecutionLimits::default()).unwrap();
        assert_eq!(upgrades(&double).len(), 2);
        assert_eq!(
            double["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|hit| hit.get("control_applications").is_some())
                .count(),
            4
        );
        assert_eq!(
            report,
            run_game_scenario(&req, &ExecutionLimits::default()).unwrap()
        );
    }
}

fn solo() -> GameScenarioRequest {
    let mut req = compile(
        "options duration=25 crit=fixed; sakiri char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=120; sakiri add weapon=fork_mofeikesi refine=1; target hp=100000000 def=0 resist=0 control_immune=false; active sakiri; sakiri burst;",
    );
    normalize(&mut req);
    req
}

#[test]
fn good_boy_delayed_control_keeps_cast_deadline_and_cannot_rearm_an_expired_window() {
    for delay in [21_999_999, 22_000_000, 23_000_000] {
        let mut req = solo();
        if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut req.actions[0] {
            *hit_delay_us = delay;
        }
        req.actions.push(GameActionCommand::InvokeAbility {
            timestamp_us: 22_000_000,
            actor_index: 0,
            ability_id: "GA_Sagiri_Melee".into(),
            damage_effect_id: "GE_Player_Sagiri_Melee1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 0,
        });
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(upgrades(&report).len(), usize::from(delay < 22_000_000));
        for event in upgrades(&report) {
            assert_eq!(event["data"]["expires_at_us"], 22_000_000);
        }
        let normal = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|hit| hit["ability_id"] == "GA_Sagiri_Melee")
            .unwrap();
        assert_eq!(attack(normal), ExactFraction::new(2000, 1).unwrap());
        assert!(
            report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .any(|hit| hit.get("control_applications").is_some())
        );
    }
}

#[test]
fn good_boy_old_wearer_control_cannot_upgrade_a_newer_shared_wearers_window() {
    let mut req = compile(
        "options duration=25 crit=fixed; sakiri char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=120; sakiri add weapon=fork_mofeikesi refine=1; skia char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=100; skia add weapon=fork_mofeikesi refine=5; target hp=100000000 def=0 resist=0 control_immune=false; active sakiri; sakiri burst; skia swap; skia burst;",
    );
    if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut req.actions[0] {
        *hit_delay_us = 10_000_000;
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["rejected_action"]
    );
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|hit| hit.get("control_applications").is_some())
    );
    assert!(upgrades(&report).is_empty());
}

#[test]
fn good_boy_stale_same_wearer_control_cannot_upgrade_a_later_cast() {
    let mut req = compile(include_str!(
        "../../../../../examples/good-boy-control-cycles.nte"
    ));
    req.relative_duration_us = 130_000_000;
    let mut cast = 0;
    for action in &mut req.actions {
        if let GameActionCommand::InvokeAbility {
            timestamp_us,
            ability_id,
            hit_delay_us,
            ..
        } = action
            && ability_id == "GA_Sagiri_UltraSkill"
        {
            let impact = if cast == 0 { 70_000_000 } else { 100_000_000 };
            *hit_delay_us = impact - *timestamp_us;
            cast += 1;
        }
    }
    assert_eq!(cast, 2);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["rejected_action"]
    );
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 240);
    let controls: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit.get("control_applications").is_some())
        .collect();
    assert_eq!(controls.len(), 2);
    assert_eq!(controls[0]["time_us"], 72_000_000);
    assert_eq!(controls[1]["time_us"], 100_000_000);
    assert!(upgrades(&report).is_empty());
}

#[test]
fn good_boy_successful_control_does_not_require_positive_hp_loss() {
    let mut req = solo();
    req.actors[0].static_stats.base_atk = 1;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1,
        denominator: 1000,
    };
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|hit| hit["applied_hp_loss"] == 0)
    );
    assert_eq!(upgrades(&report).len(), 1);
}
