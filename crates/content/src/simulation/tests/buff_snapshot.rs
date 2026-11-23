use super::*;
use serde_json::json;

fn request() -> GameScenarioRequest {
    let mut req = research_request();
    req.actors[0].weapon_id = "weapon:fork_KnightCandy".into();
    req.actors[0].static_stats.crit_rate = Ratio::one();
    req.actors[0].gear_procs = vec![
        GearProcConfig::KnightCandyCritDamage {
            crit_damage_per_stack: Ratio {
                numerator: 1,
                denominator: 25,
            },
            duration_us: 10_000_000,
        },
        GearProcConfig::FirefliesCritDamage {
            crit_damage_per_stack: Ratio {
                numerator: 2,
                denominator: 25,
            },
            max_stacks: 7,
            duration_us: 10_000_000,
        },
    ];
    req.relative_duration_us = 60;
    req
}

#[test]
fn final_buff_snapshot_preserves_grouped_and_independent_stacks_after_switch() {
    let mut req = request();
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1004".into();
    ally.weapon_id.clear();
    ally.gear_procs.clear();
    req.actors.push(ally);
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 55,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let snapshot = &report["final_buff_state"];
    assert_eq!(snapshot["time_us"], 60);
    assert_eq!(snapshot["active_actor_index"], 1);
    let buffs = crate::simulation::tests::support::final_buffs(&report);
    assert_eq!(buffs.len(), 7);
    let knight = buffs
        .iter()
        .find(|b| b["id"] == "knight_candy.crit_damage")
        .unwrap();
    assert_eq!(knight["source"], "weapon:fork_KnightCandy");
    assert_eq!(knight["owner_actor"], 0);
    assert_eq!(
        knight["recipient"],
        json!({"type":"specific_actor","target":0})
    );
    assert_eq!(knight["stacks"], 6);
    assert_eq!(knight["max_stacks"], 10);
    assert_eq!(knight["duration"]["expires_at_us"], 10_000_050);
    assert_eq!(knight["stat_modifiers"]["crit_damage_bonus"], json!("1/25"));
    assert_eq!(knight["refresh_policy"]["policy"], "reset");
    let fireflies = buffs
        .iter()
        .filter(|b| b["id"] == "fireflies.crit_damage")
        .collect::<Vec<_>>();
    assert_eq!(fireflies.len(), 6);
    assert!(
        fireflies
            .iter()
            .all(|b| b["stacks"] == 1 && b["refresh_policy"]["policy"] == "independent")
    );
    assert_eq!(
        report,
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap()
    );
    req.relative_duration_us = 10_000_050;
    let expired = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(expired["final_buff_state"]["instances"], json!([]));
    assert_eq!(expired["final_buff_state"]["time_us"], 10_000_050);
}

#[test]
fn failed_admission_keeps_the_last_event_snapshot_and_unexpired_buffs() {
    let mut req = request();
    if let GameActionCommand::InvokeAbility { recovery_us, .. } = &mut req.actions[0] {
        *recovery_us = 100;
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert_eq!(report["rejected_action"]["time_us"], 10);
    assert_eq!(report["final_buff_state"]["time_us"], 10);
    assert_eq!(
        report["final_buff_state"]["instances"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(
        report["final_buff_state"]["instances"]
            .as_array()
            .unwrap()
            .iter()
            .all(|b| b["stacks"] == 1)
    );
}
