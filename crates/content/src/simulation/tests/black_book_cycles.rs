use super::*;
use serde_json::{Value, json};

fn request(tier: u32) -> GameScenarioRequest {
    let source = include_str!("../../../../../examples/black-tome-cycles.nte")
        .replace("refine=1", &format!("refine={tier}"));
    let mut req = crate::dsl::compiler::compile_scenario(&source, Default::default())
        .unwrap()
        .request;
    let stats = &mut req.actors[0].static_stats;
    stats.base_atk = 1000;
    stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    stats.crit_rate = Ratio::zero();
    stats.damage_bonus = Ratio::zero();
    stats.elemental_bonus = Ratio::zero();
    stats.elemental_bonuses.clear();
    req
}

fn followups(report: &Value) -> Vec<&Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Fork_BlackBook_damage")
        .collect()
}

#[test]
fn automatic_support_unlocks_two_live_arc_cycles_with_exact_parent_and_refinement_damage() {
    for (tier, expected) in [(1, 2400), (2, 2852), (3, 3328), (4, 3828), (5, 4352)] {
        let req = request(tier);
        let commands: Vec<_> = req
            .actions
            .iter()
            .enumerate()
            .filter_map(|(index, command)| {
                matches!(
                    command,
                    GameActionCommand::PlayerInput {
                        input: PlayerInput::Arc,
                        ..
                    }
                )
                .then_some(index)
            })
            .collect();
        assert_eq!(commands.len(), 2);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["event_kind"] == "support_dispatch")
                .count(),
            4
        );
        let inputs = report["player_input_receipts"].as_array().unwrap();
        assert_eq!(inputs.len(), 2);
        for (input, command) in inputs.iter().zip(&commands) {
            assert_eq!(input["command_index"], *command);
            assert_eq!(input["ability_id"], "weapon:fork_BlackBook:arc");
        }
        assert_eq!(inputs[0]["time_us"], 16_100_000);
        assert_eq!(inputs[1]["time_us"], 46_100_000);
        let summons: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "black_book_arc")
            .collect();
        assert_eq!(summons.len(), 2);
        assert_eq!(summons[0]["time_us"], 16_300_000);
        assert_eq!(summons[1]["time_us"], 46_300_000);
        let hits = followups(&report);
        assert!(!hits.is_empty());
        for hit in &hits {
            let time = hit["time_us"].as_u64().unwrap();
            assert_eq!(hit["actor_index"], 0);
            let meta = support::hit_metadata(&report, hit);
            assert_eq!(
                meta["action"]["input_role_provenance"],
                meta["action"]["provenance"]
            );
            assert_eq!(hit["target_index"], 0);
            assert_eq!(
                hit["command_index"],
                commands[usize::from(time >= 46_100_000)]
            );
            assert!(hit["switch_origin"].is_null());
            // Independent oracle: 1000 ATK * [2,2.3,2.6,2.9,3.2]
            // * [1.20,1.24,1.28,1.32,1.36], separate from Break Intensity.
            assert_eq!(hit["non_crit_damage"], json!(format!("{}/{}", expected, 1)));
            assert!(
                report["hit_receipts"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|trigger| {
                        trigger["time_us"] == time && trigger["break_contribution"].is_object()
                    })
            );
        }
        for time in [16_700_000, 30_100_000, 36_100_000, 46_700_000, 66_100_000] {
            assert_eq!(
                hits.iter().filter(|hit| hit["time_us"] == time).count(),
                4,
                "one callback per delivered team Break contribution, tier {tier} at {time}; actual times {:?}",
                hits.iter().map(|hit| &hit["time_us"]).collect::<Vec<_>>()
            );
        }
        assert_eq!(hits.len(), 20);
        assert!(
            hits.iter()
                .all(|hit| hit["time_us"] != 40_000_000 && hit["time_us"] != 67_000_000)
        );
    }
}

#[test]
fn defeated_off_field_wearer_cannot_emit_followups_from_a_later_team_break() {
    let mut req = request(1);
    req.actions
        .retain(|command| command.timestamp_us() <= 30_500_000);
    req.actions.push(GameActionCommand::InvokeAbility {
        timestamp_us: 34_000_000,
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
    req.relative_duration_us = 40_000_000;
    let id = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    req.incoming_damage_events
        .push(crate::simulation::GameActorDamageEvent {
            timestamp_us: 32_000_000,
            actor_index: 0,
            delivered_damage: req.actors[0].initial_hp,
        });
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert!(
        followups(&report)
            .iter()
            .all(|hit| hit["time_us"].as_u64().unwrap() < 32_000_000)
    );
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|hit| { hit["time_us"] == 34_000_000 && hit["break_contribution"].is_object() })
    );
}

#[test]
fn designation_retargets_only_at_its_five_second_tick_and_stops_on_wearer_death() {
    use crate::runtime_api::CallbackContext;
    use crate::weapons::black_book::arc;
    let mut req = request(1);
    let mut other = req.targets[0].clone();
    other.target_index = 1;
    req.targets.push(other);
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    state.advance_unpaused_test_clock(16_100_000);
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    let mut ctx = CallbackContext {
        request: &req,
        state: &mut state,
        queue: &mut queue,
        sequence: &mut sequence,
        trace: &mut trace,
    };
    let parent = req
        .actions
        .iter()
        .position(|command| {
            matches!(
                command,
                GameActionCommand::PlayerInput {
                    input: PlayerInput::Arc,
                    ..
                }
            )
        })
        .unwrap();
    let parameters = arc::parameters(&req.actors[0]).unwrap();
    arc::activate(&mut ctx, parent, 0, parameters, 16_100_000).unwrap();
    let startup = ctx.state.actors[0]
        .equipment
        .black_book
        .pending_startup
        .unwrap();
    ctx.queue.clear();
    ctx.state
        .advance_unpaused_test_clock(startup.completes_at_us);
    arc::startup_complete(
        &mut ctx,
        0,
        parent,
        startup.generation,
        16_100_000,
        parameters,
    )
    .unwrap();
    let generation = ctx.state.actors[0].equipment.black_book.generation;
    assert_eq!(
        ctx.state.actors[0].equipment.black_book.arc_active_until_us,
        36_300_000
    );
    assert_eq!(
        ctx.state.actors[0].equipment.black_book.designated_target,
        Some(0)
    );
    ctx.state.targets[0].hp_current = 0;
    ctx.state.advance_unpaused_test_clock(17_100_000);
    let before = ctx.queue.len();
    arc::on_target_break_damage(&mut ctx, 1).unwrap();
    assert_eq!(
        ctx.queue.len(),
        before,
        "no immediate retarget on target defeat"
    );
    ctx.state.advance_unpaused_test_clock(21_300_000);
    arc::designate_tick(&mut ctx, 0, generation, 36_300_000, 5_000_000).unwrap();
    assert_eq!(
        ctx.state.actors[0].equipment.black_book.designated_target,
        Some(1)
    );
    let before = ctx.queue.len();
    arc::on_target_break_damage(&mut ctx, 1).unwrap();
    assert_eq!(ctx.queue.len(), before + 1);
    assert!(ctx.queue.iter().any(|event| matches!(&event.data,
        crate::simulation::events::GameEventData::HitDelivery { command_index, target_index: 1, .. }
            if *command_index == parent)));
    ctx.state.actors[0].hp = 0;
    let before = ctx.queue.len();
    ctx.state.advance_unpaused_test_clock(26_300_000);
    arc::designate_tick(&mut ctx, 0, generation, 36_300_000, 5_000_000).unwrap();
    arc::on_target_break_damage(&mut ctx, 1).unwrap();
    assert_eq!(
        ctx.queue.len(),
        before,
        "dead wearer cannot reschedule or emit a hit"
    );
    ctx.state.actors[0].hp = 1;
    ctx.state.advance_unpaused_test_clock(36_300_000);
    let before = ctx.queue.len();
    arc::on_target_break_damage(&mut ctx, 1).unwrap();
    assert_eq!(ctx.queue.len(), before, "the summon end is exclusive");
}

#[test]
fn black_book_startup_interrupt_cancels_the_summon_and_starts_the_thirty_second_cooldown() {
    let mut req = request(1);
    let arc_index = req
        .actions
        .iter()
        .position(|command| {
            matches!(
                command,
                GameActionCommand::PlayerInput {
                    input: PlayerInput::Arc,
                    ..
                }
            )
        })
        .unwrap();
    req.actions.truncate(arc_index + 1);
    req.relative_duration_us = 20_000_000;
    req.interrupt_events.push(GameActorInterruptEvent {
        timestamp_us: 16_200_000,
        actor_index: 0,
        armor_damage: None,
    });

    let cancelled = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(cancelled["execution_completed"], true, "{cancelled}");
    let receipt = &cancelled["interrupt_receipts"][0];
    assert_eq!(receipt["time_us"], 16_200_000);
    assert_eq!(receipt["interrupted_command_index"], arc_index);
    assert_eq!(receipt["recovery_before_us"], 16_300_000);
    assert_eq!(receipt["recovery_after_us"], 16_200_000);
    let trace = cancelled["trace"].as_array().unwrap();
    assert!(trace.iter().any(|event| {
        event["event_kind"] == "black_book_arc_startup_cancelled" && event["time_us"] == 16_200_000
    }));
    assert!(
        trace
            .iter()
            .all(|event| event["event_kind"] != "black_book_arc")
    );

    let mut retry = req;
    retry.relative_duration_us = 50_000_000;
    let mut arc = retry.actions[arc_index].clone();
    if let GameActionCommand::PlayerInput { timestamp_us, .. } = &mut arc {
        *timestamp_us = 16_400_000;
    }
    retry.actions.push(arc);
    let rejected = run_game_scenario(&retry, &ExecutionLimits::default()).unwrap();
    assert_eq!(rejected["execution_completed"], false);
    assert_eq!(
        rejected["rejected_action"]["reason"]["until_us"], 46_200_000,
        "an interrupted startup is retriable only after its 30-second cancel cooldown"
    );
}

#[test]
fn black_book_startup_completed_after_wearer_death_cannot_spawn_or_designate() {
    let mut req = request(1);
    let arc_index = req
        .actions
        .iter()
        .position(|command| {
            matches!(
                command,
                GameActionCommand::PlayerInput {
                    input: PlayerInput::Arc,
                    ..
                }
            )
        })
        .unwrap();
    req.actions.truncate(arc_index + 1);
    req.relative_duration_us = 20_000_000;
    let incoming_id = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(incoming_id.into(), incoming_id.description().into());
    req.incoming_damage_events
        .push(crate::simulation::GameActorDamageEvent {
            timestamp_us: 16_200_000,
            actor_index: 0,
            delivered_damage: req.actors[0].initial_hp,
        });

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true, "{report}");
    assert!(report["trace"].as_array().unwrap().iter().any(|event| {
        event["event_kind"] == "black_book_arc_startup_cancelled" && event["time_us"] == 16_300_000
    }));
    assert!(report["trace"].as_array().unwrap().iter().all(|event| {
        event["event_kind"] != "black_book_arc"
            && event["event_kind"] != "black_book_designate_tick"
    }));
}
