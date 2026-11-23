use super::blackbird_secondary::{dash, stamina};
use super::*;
use crate::simulation::GameDashDirection;
use serde_json::{Value, json};

fn run(req: &GameScenarioRequest) -> Value {
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}
fn chaos_normal(time: u64, skill: bool) -> GameActionCommand {
    super::chaos_actions::make_action(
        time,
        0,
        if skill {
            "GA_Chaos071_Skill"
        } else {
            "GA_Chaos071_Melee"
        },
        if skill {
            "GE_Player_Chaos_Skill1_Damage"
        } else {
            "GE_Player_Chaos_Melee1_Damage"
        },
        0,
        0,
    )
}
fn blackbird_normal(time: u64, skill: bool) -> GameActionCommand {
    let mut action = super::blackbird_actions::request().actions.remove(0);
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        ability_id,
        damage_effect_id,
        ..
    } = &mut action
    {
        *timestamp_us = time;
        if skill {
            *ability_id = "GA_BlackBird_Skill".into();
            *damage_effect_id = "GE_Player_BlackBird_Skill_Damage".into();
        }
    }
    action
}

fn blackbird_request() -> GameScenarioRequest {
    let mut req = super::blackbird_evolved_secondary::request();
    let id = crate::HypothesisId::BlackbirdCancelGatesV1;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    req.relative_duration_us = 30_000_000;
    stamina(&mut req, 100);
    req
}

#[test]
fn chaos_back_dash_pays_stamina_and_keeps_two_real_critical_riposte_cycles() {
    let mut req = super::chaos_secondary::request(false);
    req.actions = vec![
        dash(0, true),
        chaos_normal(795_195, false),
        dash(2_000_000, true),
        chaos_normal(2_795_195, false),
    ];
    req.dash_directions = [(0, GameDashDirection::Back), (2, GameDashDirection::Back)].into();
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 6);
    assert_eq!(hits[0]["time_us"], 1_045_352);
    assert_eq!(hits[3]["time_us"], 3_045_352);
    assert!(
        hits.iter()
            .all(|hit| hit["damage_effect_id"] == "GE_Player_Chaos_PerfectEvadeAttack1_Damage")
    );
    assert_eq!(report["combat_stamina"]["current"], json!("70/1"));
    assert_eq!(
        report["final_actor_states"][0]["fractional_resources"]["chaos.crime"]["current"],
        json!("200/1")
    );
    assert_eq!(
        support::receipt_assumption(
            &report,
            &report["player_input_receipts"][0],
            "inputs.dash_directions_v1"
        ),
        Some(crate::HypothesisId::InputsDashDirectionsV1.description())
    );
    let replay = serde_json::from_slice(&serde_json::to_vec(&req).unwrap()).unwrap();
    assert_eq!(report, run(&replay));
}

#[test]
fn chaos_back_dash_uses_strict_jump_skill_and_recovery_gates() {
    for (boundary, kind) in [(100, "jump"), (590_149, "skill"), (795_195, "normal")] {
        for time in [boundary - 1, boundary] {
            let mut req = super::chaos_secondary::request(false);
            let next = if kind == "jump" {
                GameActionCommand::PlayerInput {
                    timestamp_us: time,
                    actor_index: 0,
                    input: PlayerInput::Jump,
                    critical_success: false,
                }
            } else {
                chaos_normal(time, kind == "skill")
            };
            req.actions = vec![dash(0, false), next];
            req.dash_directions.insert(0, GameDashDirection::Back);
            let report = run(&req);
            assert_eq!(report["execution_completed"], time == boundary);
            if time < boundary {
                assert_eq!(report["rejected_action"]["reason"]["until_us"], boundary);
            }
        }
    }
}

#[test]
fn blackbird_back_dash_has_zero_jump_gate_and_distinct_skill_gate() {
    for time in [531_888, 531_889] {
        let mut req = blackbird_request();
        req.relative_duration_us = 30_000_000;
        stamina(&mut req, 100);
        req.actions = vec![dash(0, false), blackbird_normal(time, true)];
        req.dash_directions.insert(0, GameDashDirection::Back);
        let report = run(&req);
        assert_eq!(
            report["execution_completed"],
            time == 531_889,
            "{}",
            report["rejected_action"]
        );
    }
    let mut req = blackbird_request();
    stamina(&mut req, 100);
    req.actions = vec![
        dash(0, true),
        GameActionCommand::PlayerInput {
            timestamp_us: 0,
            actor_index: 0,
            input: PlayerInput::Jump,
            critical_success: false,
        },
    ];
    req.dash_directions.insert(0, GameDashDirection::Back);
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 0);
    assert_eq!(report["combat_stamina"]["current"], json!("85/1"));
}

#[test]
fn blackbird_back_dash_captures_ordinary_and_evolved_recovery() {
    for evolved in [false, true] {
        let start = if evolved { 6_000_000 } else { 0 };
        let recovery = if evolved { 1_008_023 } else { 1_009_134 };
        for time in [start + recovery - 1, start + recovery] {
            let mut req = blackbird_request();
            stamina(&mut req, 100);
            if evolved {
                req.actions
                    .push(super::blackbird_evolved_secondary::burst(0));
            }
            req.dash_directions
                .insert(req.actions.len(), GameDashDirection::Back);
            req.actions.push(dash(start, false));
            req.actions.push(if evolved {
                super::blackbird_evolved_secondary::evolve_melee(time)
            } else {
                blackbird_normal(time, false)
            });
            let report = run(&req);
            assert_eq!(report["execution_completed"], time == start + recovery);
            if time < start + recovery {
                assert_eq!(
                    report["rejected_action"]["reason"]["until_us"],
                    start + recovery
                );
            }
        }
    }
}

#[test]
fn dash_direction_rejects_wrong_command_unknown_values_and_unbound_owners() {
    let mut req = super::chaos_secondary::request(false);
    req.actions = vec![chaos_normal(0, false)];
    req.dash_directions.insert(0, GameDashDirection::Back);
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("Dash input")
    );
    req.actions = vec![dash(0, true)];
    req.combat_stamina = None;
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("stamina")
    );
    stamina(&mut req, 100);
    req.actors[0].character_id = "character:1019".into();
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("Back Dodge")
    );
    let mut value = serde_json::to_value(&req).unwrap();
    value["dash_directions"]["0"] = json!("sideways");
    assert!(serde_json::from_value::<GameScenarioRequest>(value).is_err());
}

#[test]
fn explicit_front_dash_profile_preserves_the_default_full_report() {
    let mut req = super::chaos_secondary::request(false);
    req.actions = vec![dash(0, true), chaos_normal(998_284, false)];
    let expected = run(&req);
    req.dash_directions.insert(0, GameDashDirection::Front);
    assert_eq!(run(&req), expected);
}
