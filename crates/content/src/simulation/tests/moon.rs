use super::*;
use crate::simulation::metadata::InputActionRole;

fn fixture() -> GameScenarioRequest {
    let mut req = research_request();
    req.stochastic_crit = None;
    req.relative_duration_us = 20_000_000;
    req.actors[0].character_id = "character:1070".into();
    req.actors[0].weapon_id = "weapon:fork_moon".into();
    req.actors[0].gear_procs = vec![GearProcConfig::Moon {
        psyche_bonus: Ratio {
            numerator: 3,
            denominator: 25,
        },
        crit_damage_per_stack: Ratio {
            numerator: 1,
            denominator: 50,
        },
        duration_us: 5_000_000,
        icd_us: 100_000,
    }];
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].static_stats.crit_rate = Ratio::one();
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
fn test_moon_psyche_bonus_applied_only_to_psyche_hits() {
    let mut req = fixture();
    append(&mut req, 0, 0, InputActionRole::Normal, "COSMOS", 0);
    append(&mut req, 0, 200_000, InputActionRole::Normal, "PSYCHE", 0);

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 2);

    // Hit 1: non-crit COSMOS damage = 1000 * 1.0 * 0.5 * 1.0 = 500
    assert_eq!(hits[0]["non_crit_damage"], "500/1");

    // Hit 2: non-crit PSYCHE damage = 1000 * 1.0 * 0.5 * (1 + 0.12) = 560
    assert_eq!(hits[1]["non_crit_damage"], "560/1");
}

#[test]
fn test_moon_stacking_icd_and_capping() {
    let mut req = fixture();
    // Hit 0 at 0.0s: Psyche -> stack 1 (ready_at = 0.1s)
    append(&mut req, 0, 0, InputActionRole::Normal, "PSYCHE", 0);
    // Hit 1 at 0.05s: inside ICD (< 0.1s) -> ignored, still stack 1
    append(&mut req, 0, 50_000, InputActionRole::Normal, "PSYCHE", 0);
    // Hit 2 at 0.1s: -> stack 2 (ready_at = 0.2s)
    append(&mut req, 0, 100_000, InputActionRole::Normal, "PSYCHE", 0);
    // Hit 3 at 0.2s: -> stack 3 (ready_at = 0.3s)
    append(&mut req, 0, 200_000, InputActionRole::Normal, "PSYCHE", 0);
    // Hit 4 at 0.3s: -> stack 4 (ready_at = 0.4s)
    append(&mut req, 0, 300_000, InputActionRole::Normal, "PSYCHE", 0);
    // Hit 5 at 0.4s: -> stack 5 (ready_at = 0.5s)
    append(&mut req, 0, 400_000, InputActionRole::Normal, "PSYCHE", 0);
    // Hit 6 at 0.5s: -> stack 6 (ready_at = 0.6s)
    append(&mut req, 0, 500_000, InputActionRole::Normal, "PSYCHE", 0);
    // Hit 7 at 0.6s: -> stack 7 (ready_at = 0.7s)
    append(&mut req, 0, 600_000, InputActionRole::Normal, "PSYCHE", 0);
    // Hit 8 at 0.7s: -> stack 8 (ready_at = 0.8s)
    append(&mut req, 0, 700_000, InputActionRole::Normal, "PSYCHE", 0);
    // Hit 9 at 0.8s: -> stack 9 (ready_at = 0.9s)
    append(&mut req, 0, 800_000, InputActionRole::Normal, "PSYCHE", 0);
    // Hit 10 at 0.9s: -> stack 10 (cap reached, ready_at = 1.0s)
    append(&mut req, 0, 900_000, InputActionRole::Normal, "PSYCHE", 0);
    // Hit 11 at 1.0s: -> cap 10 remains, duration refreshed to 1.0s + 5.0s = 6.0s
    append(&mut req, 0, 1_000_000, InputActionRole::Normal, "PSYCHE", 0);
    // Hit 12 at 6.2s: buff expired (6.0s < 6.2s) -> stack 1
    append(&mut req, 0, 6_200_000, InputActionRole::Normal, "PSYCHE", 0);

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );

    let trace = report["trace"].as_array().unwrap();
    let stack_events: Vec<_> = trace
        .iter()
        .filter(|e| e["event_kind"] == "moon_crit_stack")
        .collect();
    // 10 stacks to cap + 1 refresh at cap + 1 after expiry = 12 stack events
    assert_eq!(stack_events.len(), 12);

    assert_eq!(stack_events[0]["data"]["stacks"], 1);
    assert_eq!(stack_events[1]["data"]["stacks"], 2);
    assert_eq!(stack_events[9]["data"]["stacks"], 10);
    assert_eq!(stack_events[10]["data"]["stacks"], 10);
    assert_eq!(stack_events[11]["data"]["stacks"], 1);
}
