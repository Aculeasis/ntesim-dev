use super::*;
use crate::simulation::metadata::InputActionRole;

fn hp_suit_request() -> GameScenarioRequest {
    let mut request = research_request();
    request.relative_duration_us = 30_000_000;
    request.actors[0].character_id = "character:1033".into();
    request.actors[0].set_id = Some("set:Suit10".into());
    request.actors[0].gear_procs = vec![GearProcConfig::HpSuitMaxHp {
        max_hp_bonus_per_stack: Ratio {
            numerator: 1,
            denominator: 25,
        },
        duration_us: 10_000_000,
        max_stacks: 10,
    }];
    request.actors[0].initial_hp = 1000;
    request.actors[0].initial_hp_max = 1000;
    request.actors[0].static_stats.crit_rate = Ratio::zero();
    request.actors[0].static_stats.crit_damage_bonus = Ratio::zero();
    request.actors[0].static_stats.damage_bonus = Ratio::zero();
    request.actors[0].static_stats.elemental_bonus = Ratio::zero();
    request.actors[0].static_stats.elemental_bonuses.clear();
    request.targets[0].defense = 0;
    request.targets[0].resistance = Ratio::zero();
    request.action_metadata.clear();
    request.stochastic_crit = None;
    request.model_assumptions.insert(
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    request.actions.clear();
    request
}

fn damage_at(
    timestamp_us: u64,
    actor_index: usize,
    delivered_damage: u64,
) -> crate::simulation::GameActorDamageEvent {
    crate::simulation::GameActorDamageEvent {
        timestamp_us,
        actor_index,
        delivered_damage,
    }
}

fn stack_count(report: &serde_json::Value) -> u32 {
    crate::simulation::tests::support::final_buffs(report)
        .iter()
        .filter(|buff| buff["id"] == "hp_suit.max_hp")
        .map(|buff| buff["stacks"].as_u64().unwrap() as u32)
        .sum()
}

fn append_plain_hit(request: &mut GameScenarioRequest, actor: usize, time: u64) {
    let template = research_request();
    let action_template = template.actions[0].clone();
    let metadata_template = template.action_metadata[&0].clone();
    let index = request.actions.len();
    let mut action = action_template;
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index,
        ..
    } = &mut action
    {
        *timestamp_us = time;
        *actor_index = actor;
    }
    request.actions.push(action);
    // Template metadata already carries input_action_role None: a plain damaging
    // hit that must not trigger the Ultimate stack grant.
    request.action_metadata.insert(index, metadata_template);
}

fn clamp_request() -> GameScenarioRequest {
    // Clamp-on-expiry needs current HP above the post-expiry effective max.
    // No generic heal event exists, so the character-specific Be Happy weapon
    // (heal on target defeat) supplies the heal; this exercises the generic
    // max-HP expiry clamp machinery through the Suit10 stacks.
    let mut request = hp_suit_request();
    request.relative_duration_us = 20_000_000;
    request.actors[0].weapon_id = "weapon:fork_vine".into();
    request.actors[0].gear_procs.push(GearProcConfig::BeHappy {
        heal_ratio: Ratio {
            numerator: 1,
            denominator: 2,
        },
        cooldown_us: 20_000_000,
    });
    request.targets[0].hp_max = 1500;
    request.targets[0].hp_current = 1500;
    // t=0: Ultimate grants ten stacks (effective Max HP 1400) and weakens the
    // target with 1000 damage, non-lethal.
    append_ultimate(&mut request, 0, 0);
    // t=1s: self damage drops HP to 400; the grant arrives at cap and is discarded.
    request.incoming_damage_events = vec![damage_at(1_000_000, 0, 600)];
    // t=2s: killing hit -> Be Happy heals 1/2 * 1400 = 700 -> HP 1100, above base.
    append_plain_hit(&mut request, 0, 4_025_936);
    request
}

fn append_ultimate(request: &mut GameScenarioRequest, actor: usize, time: u64) {
    let template = research_request();
    let action_template = template.actions[0].clone();
    let metadata_template = template.action_metadata[&0].clone();
    let index = request.actions.len();
    let mut action = action_template;
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index,
        ..
    } = &mut action
    {
        *timestamp_us = time;
        *actor_index = actor;
    }
    request.actions.push(action);
    let mut metadata = metadata_template;
    metadata.input_action_role = Some(InputActionRole::Ultimate);
    request.action_metadata.insert(index, metadata);
}

#[test]
fn hp_suit_damage_grants_stack_and_raises_effective_max_hp() {
    let mut request = hp_suit_request();
    request.relative_duration_us = 5_000_000;
    // Independent oracle: 1000 base * 1.04 = 1040.
    request.incoming_damage_events = vec![damage_at(0, 0, 100)];
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(stack_count(&report), 1);
    assert_eq!(report["final_actor_states"][0]["hp_max"], 1040);
    assert_eq!(report["final_actor_states"][0]["hp"], 900);
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "hp_suit_max_hp_stack")
            .count(),
        1
    );
    assert_eq!(
        report["resolved_model_assumptions"][crate::sets::hp_suit::HYPOTHESIS_ID.as_str()],
        crate::sets::hp_suit::HYPOTHESIS_ID.description()
    );
}

#[test]
fn hp_suit_caps_at_ten_stacks() {
    let mut request = hp_suit_request();
    request.relative_duration_us = 9_999_999;
    // 12 damage events at 0.5s spacing; cap is 10, each stack lasts 10s so none expire yet.
    request.incoming_damage_events = (0..12).map(|i| damage_at(i * 500_000, 0, 10)).collect();
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // Independent oracle: 1000 * 1.40 = 1400.
    assert_eq!(stack_count(&report), 10);
    assert_eq!(report["final_actor_states"][0]["hp_max"], 1400);
    // Every grant call emits one trace event, including the two discarded at cap:
    // the last two events still report 10/10, proving the discards added no stacks.
    let stack_events: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "hp_suit_max_hp_stack")
        .collect();
    assert_eq!(stack_events.len(), 12);
    for event in &stack_events[10..] {
        assert_eq!(
            event["data"]["stacks"], 10,
            "cap-discarded grants must not raise the stack count"
        );
        assert_eq!(event["data"]["max_stacks"], 10);
    }
}

#[test]
fn hp_suit_cap_hit_does_not_refresh_durations() {
    let mut request = hp_suit_request();
    request.relative_duration_us = 10_000_001;
    append_ultimate(&mut request, 0, 0);
    // Ten stacks granted at t=0, each lasting 10s. The t=9s hit arrives at the
    // ten-stack cap and must be discarded WITHOUT refreshing existing durations.
    request.incoming_damage_events = vec![damage_at(9_000_000, 0, 100)];
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // Falsifier: independent expiry removes every stack at t=10s -> 0 stacks,
    // effective Max HP back to the 1000 base. A refresh-at-cap model would keep
    // the stacks alive past t=10s -> 10 stacks, effective Max HP 1400.
    assert_eq!(stack_count(&report), 0);
    assert_eq!(report["final_actor_states"][0]["hp_max"], 1000);
    assert_eq!(report["final_actor_states"][0]["hp"], 900);
}

#[test]
fn hp_suit_zero_damage_grants_no_stack() {
    let mut request = hp_suit_request();
    request.relative_duration_us = 5_000_000;
    request.incoming_damage_events = vec![damage_at(0, 0, 0)];
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // Grants require positive settled HP loss: no stack, no Max HP change, and
    // not even a grant trace event.
    assert_eq!(stack_count(&report), 0);
    assert_eq!(report["final_actor_states"][0]["hp_max"], 1000);
    assert_eq!(report["final_actor_states"][0]["hp"], 1000);
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "hp_suit_max_hp_stack")
            .count(),
        0
    );
}

#[test]
fn hp_suit_ultimate_applies_before_same_time_damage() {
    let mut request = hp_suit_request();
    request.relative_duration_us = 5_000_000;
    append_ultimate(&mut request, 0, 0);
    request.incoming_damage_events = vec![damage_at(0, 0, 100)];
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // Both the Ultimate grant and the damage grant emit one trace event each.
    // CausalPriority orders the cast grant (BuffTickOrExpiry) before the
    // same-timestamp hit (HitDelivery): the first event already reports 10/10,
    // so the damage grant arrived at a full cap and was discarded.
    let stack_events: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "hp_suit_max_hp_stack")
        .collect();
    assert_eq!(stack_events.len(), 2);
    assert_eq!(
        stack_events[0]["data"]["stacks"], 10,
        "Ultimate grant of ten stacks must be visible before the same-time hit"
    );
    assert_eq!(stack_events[0]["data"]["max_stacks"], 10);
    assert_eq!(
        stack_events[1]["data"]["stacks"], 10,
        "same-time damage grant at cap must not raise the stack count"
    );
    assert_eq!(stack_events[1]["data"]["max_stacks"], 10);
    assert_eq!(stack_count(&report), 10);
    assert_eq!(report["final_actor_states"][0]["hp_max"], 1400);
    assert_eq!(report["final_actor_states"][0]["hp"], 900);
}

#[test]
fn hp_suit_stacks_expire_independently_after_ten_seconds() {
    let mut request = hp_suit_request();
    request.relative_duration_us = 10_000_001;
    // Two stacks at t=0 and t=5_000_000; the first expires at t=10s.
    request.incoming_damage_events = vec![damage_at(0, 0, 10), damage_at(5_000_000, 0, 10)];
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // Independent oracle: 1000 * 1.04 = 1040 with one stack left.
    assert_eq!(stack_count(&report), 1);
    assert_eq!(report["final_actor_states"][0]["hp_max"], 1040);
}

#[test]
fn hp_suit_ultimate_grants_ten_stacks() {
    let mut request = hp_suit_request();
    request.relative_duration_us = 5_000_000;
    append_ultimate(&mut request, 0, 0);
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // Independent oracle: 1000 * 1.40 = 1400.
    assert_eq!(stack_count(&report), 10);
    assert_eq!(report["final_actor_states"][0]["hp_max"], 1400);
}

#[test]
fn hp_suit_expiry_clamps_current_hp_to_base_max() {
    // Counter-scenario first: just before expiry the HP is still above the base
    // max, proving the clamp (not missing healing) produces the final 1000.
    let mut before_expiry = clamp_request();
    before_expiry.relative_duration_us = 9_999_999;
    let early = run_game_scenario(&before_expiry, &ExecutionLimits::default()).unwrap();
    assert_eq!(early["execution_completed"], true);
    // Independent oracle: ten stacks alive -> 1000 * 1.40 = 1400 effective max,
    // HP 400 + 700 heal = 1100.
    assert_eq!(stack_count(&early), 10);
    assert_eq!(early["final_actor_states"][0]["hp_max"], 1400);
    assert_eq!(
        early["final_actor_states"][0]["hp"], 1100,
        "{}",
        early["trace"]
    );

    let report = run_game_scenario(&clamp_request(), &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // Independent oracle: at t=10s all stacks expire, effective Max HP falls
    // 1400 -> 1000, and current HP 1100 is clamped to 1000.
    assert_eq!(stack_count(&report), 0);
    assert_eq!(report["final_actor_states"][0]["hp_max"], 1000);
    assert_eq!(report["final_actor_states"][0]["hp"], 1000);
    let clamp_events: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "max_hp_stack_expired")
        .collect();
    assert_eq!(clamp_events.len(), 1);
    assert_eq!(clamp_events[0]["data"]["hp_max_before"], 1400);
    assert_eq!(clamp_events[0]["data"]["hp_max"], 1000);
    assert_eq!(clamp_events[0]["data"]["reason"], "hp_clamped");
}

#[test]
fn hp_suit_expiry_returns_effective_max_to_base() {
    let mut request = hp_suit_request();
    request.relative_duration_us = 30_000_000;
    request.incoming_damage_events = vec![damage_at(0, 0, 100)];
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(stack_count(&report), 0);
    assert_eq!(report["final_actor_states"][0]["hp_max"], 1000);
    assert_eq!(report["final_actor_states"][0]["hp"], 900);
}

#[test]
fn hp_suit_off_field_wearer_still_gains_stacks() {
    let mut request = hp_suit_request();
    request.relative_duration_us = 5_000_000;
    let mut ally = request.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.set_id = None;
    ally.gear_procs.clear();
    request.actors.push(ally);
    request.actions = vec![GameActionCommand::SwitchActor {
        timestamp_us: 1,
        target_actor_index: 1,
        cooldown_us: 0,
    }];
    request.incoming_damage_events = vec![damage_at(2_000_000, 0, 100)];
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(stack_count(&report), 1);
    assert_eq!(report["final_actor_states"][0]["hp_max"], 1040);
    assert_eq!(report["final_actor_states"][1]["hp_max"], 1000);
}

#[test]
fn hp_suit_replay_matches() {
    let mut request = hp_suit_request();
    request.relative_duration_us = 8_000_000;
    request.incoming_damage_events = vec![damage_at(0, 0, 100), damage_at(5_000_000, 0, 50)];
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    let encoded = serde_json::to_value(&request).unwrap();
    let decoded: GameScenarioRequest = serde_json::from_value(encoded).unwrap();
    let replay = run_game_scenario(&decoded, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        replay["incoming_damage_receipts"],
        report["incoming_damage_receipts"]
    );
    assert_eq!(
        replay["final_actor_states"][0]["hp_max"],
        report["final_actor_states"][0]["hp_max"]
    );
    assert_eq!(replay["trace"], report["trace"]);
}

#[test]
fn hp_suit_rejects_wrong_set_and_zero_duration() {
    let mut request = hp_suit_request();
    request.actors[0].set_id = Some("set:Suit12".into());
    assert!(
        run_game_scenario(&request, &ExecutionLimits::default())
            .unwrap_err()
            .contains("set:Suit10")
    );

    let mut bad_duration = hp_suit_request();
    bad_duration.actors[0].gear_procs = vec![GearProcConfig::HpSuitMaxHp {
        max_hp_bonus_per_stack: Ratio {
            numerator: 1,
            denominator: 25,
        },
        duration_us: 0,
        max_stacks: 10,
    }];
    assert!(
        run_game_scenario(&bad_duration, &ExecutionLimits::default())
            .unwrap_err()
            .contains("positive duration")
    );
}

#[test]
fn hp_suit_two_piece_and_stacks_compose_additively_per_equipment_contract() {
    // EQUIPMENT_CONTRACT.md: set packs use the additive pack interpreter.
    // 2pc (+10% static HPMaxUp) and 4pc stacks (+4% each) sum in one pool:
    // 1000 * (1 + 0.10 + 0.40) = 1500, not 1000 * 1.10 * 1.40 = 1540.
    let mut request = hp_suit_request();
    request.relative_duration_us = 5_000_000;
    request.actors[0].static_stats.base_hp = Some(1000);
    request.actors[0].static_stats.hp_percent = Ratio {
        numerator: 1,
        denominator: 10,
    };
    request.actors[0].initial_hp_max = 1100;
    request.incoming_damage_events = vec![damage_at(0, 0, 100)];
    append_ultimate(&mut request, 0, 1_000_000);
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // Independent oracle: 1000 * (1 + 1/10 + 10 * 1/25) = 1000 * 1.5 = 1500.
    assert_eq!(report["final_actor_states"][0]["hp_max"], 1500);
}

#[test]
fn hp_suit_preserves_unrounded_static_hp_before_dynamic_projection() {
    let mut request = hp_suit_request();
    request.relative_duration_us = 1;
    let actor = &mut request.actors[0];
    actor.static_stats.base_hp = Some(101);
    actor.static_stats.hp_percent = Ratio {
        numerator: 1,
        denominator: 2,
    };
    actor.static_stats.hp_flat = Ratio::zero();
    actor.static_stats.hp = 152;
    actor.initial_hp_max = 152;
    actor.initial_hp = 152;
    actor.gear_procs = vec![GearProcConfig::HpSuitMaxHp {
        max_hp_bonus_per_stack: Ratio {
            numerator: 1,
            denominator: 2,
        },
        duration_us: 10_000_000,
        max_stacks: 1,
    }];
    request.incoming_damage_events = vec![damage_at(0, 0, 1)];
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // 101 * (1 + .5 + .5) = 202; rounding 151.5 first would yield 203.
    assert_eq!(report["final_actor_states"][0]["hp_max"], 202);
    assert_eq!(report["final_actor_states"][0]["hp"], 151);

    // Explicit legacy totals remain authoritative when they do not recompose.
    request.actors[0].initial_hp_max = 160;
    request.actors[0].initial_hp = 160;
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["final_actor_states"][0]["hp_max"], 211);
}

#[test]
fn hp_projection_rejects_invalid_ratios_before_runtime_construction() {
    for flat in [false, true] {
        let mut request = hp_suit_request();
        let invalid = Ratio {
            numerator: 1,
            denominator: 0,
        };
        if flat {
            request.actors[0].static_stats.hp_flat = invalid;
        } else {
            request.actors[0].static_stats.hp_percent = invalid;
        }
        assert!(run_game_scenario(&request, &ExecutionLimits::default()).is_err());
    }
}
