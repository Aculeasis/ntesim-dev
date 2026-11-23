use super::*;
use serde_json::json;

fn fixture(timeline: &[(u64, bool)]) -> GameScenarioRequest {
    let mut req = research_request();
    req.stochastic_crit = None;
    req.relative_duration_us = 10_000_000;
    req.actors[0].set_id = Some("set:Suit12".into());
    req.actors[0].gear_procs = vec![GearProcConfig::QuietManorMental {
        mental_bonus_per_stack: Ratio {
            numerator: 3,
            denominator: 25,
        },
        duration_us: 6_000_000,
        max_stacks: 3,
    }];
    req.actors[0].static_stats.mental_damage_bonus = Ratio {
        numerator: 1,
        denominator: 10,
    };
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    let action = req.actions[0].clone();
    let metadata = req.action_metadata[&0].clone();
    req.actions.clear();
    req.action_metadata.clear();
    for (index, (timestamp, normal)) in timeline.iter().enumerate() {
        let mut action = action.clone();
        if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut action {
            *timestamp_us = *timestamp;
        }
        req.actions.push(action);
        let mut metadata = metadata.clone();
        metadata.damage_type_ex = Some("EExecutionDamageTypeEX::DAMAGE_TYPE_PSYCHICALLY".into());
        metadata.input_action_role = Some(if *normal {
            crate::simulation::metadata::InputActionRole::Normal
        } else {
            crate::simulation::metadata::InputActionRole::Other
        });
        req.action_metadata.insert(index, metadata);
    }
    req
}

#[test]
fn quiet_manor_independent_expiry_discards_applications_at_cap() {
    let req = fixture(&[
        (0, true),
        (1_000_000, true),
        (2_000_000, true),
        (3_000_000, true),
        (5_999_999, false),
        (6_000_000, false),
        (7_000_000, false),
        (8_000_000, false),
        (9_000_000, true),
    ]);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(
        hits.iter()
            .map(|hit| hit["non_crit_damage"].clone())
            .collect::<Vec<_>>(),
        [1220, 1340, 1460, 1460, 1460, 1340, 1220, 1100, 1220]
            .map(|n| json!(format!("{}/{}", n, 1)))
    );
    assert_eq!(
        report["resolved_model_assumptions"][crate::sets::quiet_manor::HYPOTHESIS_ID.as_str()],
        crate::sets::quiet_manor::HYPOTHESIS_ID.description()
    );
    let buffs = crate::simulation::tests::support::final_buffs(&report);
    assert_eq!(buffs.len(), 1);
    assert_eq!(buffs[0]["duration"]["expires_at_us"], 15_000_000);
}

#[test]
fn quiet_manor_off_field_mental_uses_source_stacks_and_live_expiry() {
    let mut req = fixture(&[
        (0, true),
        (1_000_000, true),
        (2_000_000, true),
        (3_000_000, false),
    ]);
    req.relative_duration_us = 6_500_000;
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.gear_procs.clear();
    ally.set_id = None;
    ally.static_stats.mental_damage_bonus = Ratio::one();
    req.actors.push(ally);
    if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut req.actions[3] {
        *hit_delay_us = 3_000_000;
    }
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 4_000_000,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(
        report["hit_receipts"][3]["non_crit_damage"],
        json!("1340/1")
    );
    assert_eq!(report["final_buff_state"]["active_actor_index"], 1);
    let instances = crate::simulation::tests::support::final_buffs(&report);
    assert_eq!(instances.len(), 2);
    // Ally isolation (hypothesis sets.quiet_manor_v1): every Quiet Manor stack is
    // owned by the wearer (actor 0); the active ally (actor 1) holds none, so its
    // own static_stats.mental_damage_bonus (1/1 here) never enters the packet.
    // The 1340 oracle above already pins this: with the ally's stats the hit
    // would be 1000 * (1 + 1 + 0.24) = 2240, not 1340.
    assert!(
        instances
            .iter()
            .all(|instance| instance["id"] == "quiet_manor.mental_damage"
                && instance["owner_actor"] == 0),
        "Quiet Manor stacks must be wearer-owned; the ally must hold none"
    );
}

#[test]
fn quiet_manor_cap_discards_grant_without_refreshing_deadlines() {
    // Focused falsifier for the discard-at-cap hypothesis (sets.quiet_manor_v1):
    // grants at 0/1/2s are admitted (deadlines 6/7/8s), the 3s grant hits the cap
    // of 3 and must be discarded WITHOUT refreshing existing deadlines.
    // If the 4th input refreshed, the 6.0s probe would see three live stacks and
    // deal 1460 instead of 1340.
    let req = fixture(&[
        (0, true),
        (1_000_000, true),
        (2_000_000, true),
        (3_000_000, true),
        (6_000_000, false),
        (7_000_000, false),
    ]);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // Independent oracles, B = 1000, static Mental 1/10, live stack 3/25:
    // t=0: 1 stack -> 1220; t=1: 2 -> 1340; t=2: 3 -> 1460; t=3: 3 (cap) -> 1460;
    // t=6.0: grant@0 expired before equal-time hits -> 2 stacks -> 1340;
    // t=7.0: grants@0,1 expired -> 1 stack -> 1220.
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(
        hits.iter()
            .map(|hit| hit["non_crit_damage"].clone())
            .collect::<Vec<_>>(),
        [1220, 1340, 1460, 1460, 1340, 1220].map(|n| json!(format!("{}/{}", n, 1)))
    );
    assert_eq!(
        report["resolved_model_assumptions"][crate::sets::quiet_manor::HYPOTHESIS_ID.as_str()],
        crate::sets::quiet_manor::HYPOTHESIS_ID.description()
    );
    let grants: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "quiet_manor_mental_stack")
        .collect();
    assert_eq!(grants.len(), 4, "one grant event per Normal input");
    assert_eq!(
        grants
            .iter()
            .map(|e| e["time_us"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![0, 1_000_000, 2_000_000, 3_000_000]
    );
    // The 3s grant is discarded at the cap: the stack count stays 3/3 instead of
    // refreshing any deadline.
    assert_eq!(
        grants[3]["data"]["stacks"], 3,
        "cap-hit grant must report the unchanged 3/3 stacks"
    );
    assert_eq!(grants[3]["data"]["max_stacks"], 3);
}

#[test]
fn quiet_manor_multi_hit_normal_input_grants_single_stack() {
    // Falsifier for a per-hit grant model: one admitted Normal input that
    // delivers two hits (Mint's native stage-0 melee: Melee1 + Melee2 child)
    // must grant exactly ONE independent stack, not one per hit.
    let mut req = research_request();
    req.stochastic_crit = None;
    // Horizon 5s: the single 6s stack is still live at the end, so the final
    // buff state pins the applied stack count.
    req.relative_duration_us = 5_000_000;
    req.actors[0].set_id = Some("set:Suit12".into());
    req.actors[0].gear_procs = vec![GearProcConfig::QuietManorMental {
        mental_bonus_per_stack: Ratio {
            numerator: 3,
            denominator: 25,
        },
        duration_us: 6_000_000,
        max_stacks: 3,
    }];
    req.actors[0].static_stats.mental_damage_bonus = Ratio {
        numerator: 1,
        denominator: 10,
    };
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.action_metadata.clear();
    req.actions = vec![GameActionCommand::InvokeAbility {
        timestamp_us: 0,
        actor_index: 0,
        ability_id: "GA_Mint019_Melee".into(),
        damage_effect_id: "GE_Player_Mint_Melee1_Damage_New".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 0,
    }];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 2, "Mint stage-0 melee must deliver two hits");
    assert!(
        hits.iter().all(|hit| hit["command_index"] == 0),
        "both hits belong to the single admitted Normal input"
    );
    let grants: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "quiet_manor_mental_stack")
        .collect();
    assert_eq!(
        grants.len(),
        1,
        "one Normal input grants one stack even with two hits (a per-hit model would grant two)"
    );
    let instances = crate::simulation::tests::support::final_buffs(&report);
    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0]["id"], "quiet_manor.mental_damage");
    assert_eq!(instances[0]["stacks"], 1);
    assert_eq!(
        report["resolved_model_assumptions"][crate::sets::quiet_manor::HYPOTHESIS_ID.as_str()],
        crate::sets::quiet_manor::HYPOTHESIS_ID.description()
    );
}

#[test]
fn quiet_manor_live_stacks_do_not_buff_psyche_packets() {
    // Simulation-level Mental/Psyche zone isolation with live stacks active:
    // the Mental zone (static 1/10 + live 3/25 stacks) must not enter Psyche
    // packets, and the static PSYCHE bonus must not enter Mental packets.
    let mut req = fixture(&[(0, true), (1_000_000, false)]);
    req.actors[0].static_stats.elemental_bonuses.insert(
        "PSYCHE".into(),
        Ratio {
            numerator: 1,
            denominator: 4,
        },
    );
    req.action_metadata.get_mut(&1).unwrap().damage_type_ex =
        Some("EExecutionDamageTypeEX::DAMAGE_TYPE_PSYCHE".into());
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 2);
    // Independent oracles, B = 1000, DEF 0, RES 0, no crit:
    // Mental probe with 1 live stack: 1000 * (1 + 1/10 + 3/25) = 1220.
    // A Psyche-zone leak would add 1/4 and give 1470.
    assert_eq!(hits[0]["non_crit_damage"], json!("1220/1"));
    // Psyche action with 1 live Mental stack and static PSYCHE 1/4:
    // 1000 * (1 + 1/4) = 1250. A Mental-zone leak would add 1/10 + 3/25
    // and give 1470.
    assert_eq!(hits[1]["non_crit_damage"], json!("1250/1"));
    assert_eq!(
        report["resolved_model_assumptions"][crate::sets::quiet_manor::HYPOTHESIS_ID.as_str()],
        crate::sets::quiet_manor::HYPOTHESIS_ID.description()
    );
}

#[test]
fn quiet_manor_invalid_config_and_failed_admission_do_not_grant_stacks() {
    let mut req = fixture(&[(0, true)]);
    req.actors[0].set_id = None;
    assert!(run_game_scenario(&req, &ExecutionLimits::default()).is_err());
    for (duration_us, max_stacks) in [(0, 3), (6_000_000, 0), (6_000_000, 65)] {
        let mut req = fixture(&[(0, true)]);
        req.actors[0].gear_procs = vec![GearProcConfig::QuietManorMental {
            mental_bonus_per_stack: Ratio {
                numerator: 3,
                denominator: 25,
            },
            duration_us,
            max_stacks,
        }];
        assert!(run_game_scenario(&req, &ExecutionLimits::default()).is_err());
    }
    let mut req = fixture(&[(0, true)]);
    if let GameActionCommand::InvokeAbility { energy_cost, .. } = &mut req.actions[0] {
        *energy_cost = 1;
    }
    req.actors[0].initial_energy = 0;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert!(crate::simulation::tests::support::final_buffs(&report).is_empty());
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["event_kind"] == "quiet_manor_mental_stack")
    );
}
