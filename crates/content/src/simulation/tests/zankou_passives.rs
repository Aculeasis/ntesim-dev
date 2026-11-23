use super::*;
use crate::formula::Ratio;
use crate::simulation::ExactFraction;
use crate::simulation::GameActionCommand;
use serde_json::json;
use std::collections::BTreeSet;

#[test]
fn a3_live_self_healing_and_normalized_armor_pressure_use_exact_selected_nodes() {
    for nodes in [vec![1], vec![3], vec![1, 3], vec![2, 4, 5]] {
        let mut req = entry_request();
        req.actions.clear();
        req.relative_duration_us = 3_000_000;
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(nodes.clone()).unwrap();
        req.actors[0].initial_hp = 5000;
        req.actors[0].initial_hp_max = 10_000;
        req.actors[1].initial_hp = 5000;
        req.actors[1].initial_hp_max = 10_000;
        req.actors[0].static_stats.healing_bonus = Ratio {
            numerator: 1,
            denominator: 2,
        };
        let report = run_game_scenario(&req, &Default::default()).unwrap();
        let expected = if nodes == [1, 3] { 6800 } else { 5000 };
        assert_eq!(report["final_actor_states"][0]["hp"], expected, "{nodes:?}");
        assert_eq!(report["final_actor_states"][1]["hp"], 5000);
    }
    for nodes in [vec![], vec![3], vec![1, 3], vec![2, 4, 5]] {
        let source = "options duration=3; zankou char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=100000000 def=0 resist=0; active zankou; zankou attack;";
        let mut req = PreparedSourceScenario::new(source, Default::default(), &Default::default())
            .unwrap()
            .run(42)
            .unwrap()
            .request;
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(nodes.clone()).unwrap();
        req.interrupt_events = [200_000, 300_000]
            .map(|timestamp_us| crate::simulation::GameActorInterruptEvent {
                timestamp_us,
                actor_index: 0,
                armor_damage: Some(Ratio::one()),
            })
            .into();
        let report = run_game_scenario(&req, &Default::default()).unwrap();
        let receipts = report["interrupt_receipts"].as_array().unwrap();
        let a3 = nodes.contains(&3);
        assert_eq!(
            receipts[0]["interrupted_command_index"].is_null(),
            a3,
            "{nodes:?}"
        );
        assert_eq!(
            receipts[1]["interrupted_command_index"].is_number(),
            a3,
            "{nodes:?}"
        );
        let pressure: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["event_kind"] == "zankou_armor_pressure")
            .collect();
        assert_eq!(
            pressure[0]["data"]["exact_delta"],
            json!(if a3 { "1/2" } else { "1/1" })
        );
    }
}

#[test]
fn attached_nightmare_and_adler_counts_use_the_actual_owners_and_decay() {
    let source = "options duration=20;adler char lvl=80/80 awakening=0 talent=1,1,1 qte=1;lacrimosa char lvl=80/80 awakening=0 talent=1,1,1 qte=1;target hp=100000000 def=0 resist=0;active adler;adler skill;lacrimosa swap;lacrimosa skill;";
    let mut req = PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap()
        .request;
    req.relative_duration_us = 4_000_000;
    let packets =
        crate::simulation::validation::validate_and_resolve(&req, &Default::default()).unwrap();
    let (mut execution, _) = PreparedExecution::new(&req, req.seed, &[], &[]).unwrap();
    execution
        .process_events(&req, &Default::default(), &packets, &[], &[])
        .unwrap();
    assert_eq!(
        crate::registry::attached_dot_stacks(&req, &execution.state, 0, 4_000_000).unwrap(),
        6
    );
    assert_eq!(
        crate::registry::attached_dot_stacks(&req, &execution.state, 0, 10_000_000).unwrap(),
        3
    );
    execution.state.actors[1].hp = 0;
    assert_eq!(
        crate::registry::attached_dot_stacks(&req, &execution.state, 0, 4_000_000).unwrap(),
        1
    );
}

#[test]
fn a5_once_per_target_break_uses_story_boss_exclusion_and_real_combat_reset() {
    let source = "options duration=35; zankou char lvl=80/80 awakening=[5] talent=1,1,1 qte=1;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; target hp=100000000 def=0 resist=0;
        active zankou; zankou attack:4;zankou skill;wait(8);zankou attack:4;zankou skill;";
    let base = PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap()
        .request;
    for story in [false, true] {
        let mut req = base.clone();
        req.targets[1].is_boss = true;
        if story {
            let id = crate::HypothesisId::ZankouStoryModeV1;
            req.model_assumptions
                .insert(id.into(), id.description().into());
        }
        for target in &mut req.targets {
            target.break_profile = Some(crate::simulation::GameTargetBreakProfileConfig {
                capacity: Ratio {
                    numerator: 1000,
                    denominator: 1,
                },
                current_gauge: Ratio {
                    numerator: 1000,
                    denominator: 1,
                },
                broken_duration_us: 1_000_000,
                discord_fraction: None,
            });
        }
        let report = run_game_scenario(&req, &Default::default()).unwrap();
        let impulses: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["event_kind"] == "zankou_a5_impulse")
            .collect();
        assert_eq!(impulses.len(), if story { 1 } else { 2 }, "{report}");
        assert!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["event_kind"] == "target_broken_enter"
                    || e["event_kind"] == "target_broken"),
            "{report}"
        );
        req.combat_transition_events = vec![
            crate::simulation::GameActorCombatTransitionEvent {
                timestamp_us: 15_000_000,
                actor_index: 0,
                leaving: true,
            },
            crate::simulation::GameActorCombatTransitionEvent {
                timestamp_us: 16_000_000,
                actor_index: 0,
                leaving: false,
            },
        ];
        let report = run_game_scenario(&req, &Default::default()).unwrap();
        assert_eq!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["event_kind"] == "zankou_a5_impulse")
                .count(),
            if story { 2 } else { 4 }
        );
    }
}

#[test]
fn a5_team_break_bonus_is_fourfold_for_each_living_recipient_and_ends_with_source_death() {
    let mut req = entry_request();
    req.relative_duration_us = 1_000_000;
    req.targets[0].break_profile = Some(crate::simulation::GameTargetBreakProfileConfig {
        capacity: Ratio {
            numerator: 1000,
            denominator: 1,
        },
        current_gauge: Ratio {
            numerator: 1000,
            denominator: 1,
        },
        broken_duration_us: 1_000_000,
        discord_fraction: None,
    });
    req.actions = vec![GameActionCommand::TargetGauge {
        timestamp_us: 0,
        target_index: 0,
        gauge: Ratio::zero(),
    }];
    let baseline = run_game_scenario(&req, &Default::default()).unwrap();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(vec![5]).unwrap();
    let enhanced = run_game_scenario(&req, &Default::default()).unwrap();
    for owner in [0, 1] {
        let damage = |report: &serde_json::Value| -> ExactFraction {
            serde_json::from_value(
                report["hit_receipts"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|h| {
                        h["damage_effect_id"] == "ntesim.break.contribution"
                            && h["actor_index"] == owner
                    })
                    .unwrap()["non_crit_damage"]
                    .clone(),
            )
            .unwrap()
        };
        assert_eq!(
            damage(&enhanced),
            damage(&baseline)
                .mul(ExactFraction::new(4, 1).unwrap())
                .unwrap()
        );
    }
    req.actors[0].initial_hp = 0;
    let dead = run_game_scenario(&req, &Default::default()).unwrap();
    let damage = |report: &serde_json::Value| {
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|h| h["actor_index"] == 1)
            .unwrap()["non_crit_damage"]
            .clone()
    };
    assert_eq!(damage(&dead), damage(&baseline));
}

#[test]
fn a3_delusion_phase_survives_refresh_and_ends_on_swap_or_actual_defeat() {
    let source = "options duration=10;zankou char lvl=80/80 awakening=[3] talent=1,1,1 qte=1;mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;target hp=100000000 def=0 resist=0;active zankou;zankou skill;zankou attack;mint swap;wait(4);";
    let result = PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap();
    assert!(
        result.report.execution_completed,
        "{:?}",
        result.report.halt_diagnostic
    );
    super::super::tests::assert_native_replay(&result);
    let mut req = result.request;
    req.actors[0].initial_hp = 5000;
    req.actors[0].initial_hp_max = 10_000;
    let report = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(report["final_actor_states"][0]["hp"], 5400);
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["event_kind"] == "zankou_a3_heal")
            .count(),
        1
    );
    req.actions.clear();
    req.input_time_contexts.clear();
    req.active_actor_index = 1;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(vec![1, 3]).unwrap();
    let id = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    req.incoming_damage_events
        .push(crate::simulation::GameActorDamageEvent {
            timestamp_us: 1_500_000,
            actor_index: 0,
            delivered_damage: 100_000,
        });
    let report = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["event_kind"] == "zankou_a3_heal")
            .count(),
        1
    );
    assert_eq!(report["final_actor_states"][0]["hp"], 0);
}

fn entry_request() -> GameScenarioRequest {
    let source = "options duration=35; zankou char lvl=80/80 awakening=0 talent=1,1,1 qte=1; lacrimosa char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=10000000 def=1080 resist=0; active zankou; wait(1);";
    let mut req = crate::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    req.actions = [(0, 1), (3_000_000, 0), (31_000_000, 1)]
        .map(
            |(timestamp_us, target_actor_index)| GameActionCommand::SwitchActor {
                timestamp_us,
                target_actor_index,
                cooldown_us: 0,
            },
        )
        .into();
    req
}

#[test]
fn crimson_reverie_sets_cycle_once_and_does_not_refill_on_field_entry_or_after_thirty_seconds() {
    for initial in [0, 40, 100] {
        let mut req = entry_request();
        req.actors[0].initial_cycle = initial;
        let (mut execution, _) = PreparedExecution::new(&req, req.seed, &[], &[]).unwrap();
        assert_eq!(execution.state.actors[0].cycle.current(), 100);
        assert_eq!(execution.state.actors[1].cycle.current(), 0);
        assert!(
            execution
                .state
                .actors
                .iter()
                .all(|actor| actor.granted_energy == 0)
        );
        let limits = ExecutionLimits::default();
        let packets = crate::simulation::validation::validate_and_resolve(&req, &limits).unwrap();
        execution
            .process_events(&req, &limits, &packets, &[], &[])
            .unwrap();
        assert_eq!(execution.state.actors[0].cycle.current(), 0);
        let actor = &execution.state.actors[0];
        let ledger = actor.cycle_ledger.summary(&actor.cycle).unwrap();
        assert_eq!(ledger.initial, 100);
        assert_eq!(ledger.spent, 100);
        assert!(ledger.conserved);
        assert_eq!(
            execution
                .trace
                .iter()
                .filter(|e| e.event_kind == "reaction_scorch_applied")
                .count(),
            1
        );
        let auras: Vec<_> = execution
            .state
            .buffs
            .instances()
            .iter()
            .filter(|b| b.id == "zankou.crimson_reverie_ci")
            .collect();
        assert_eq!(auras.len(), 1);
        assert_eq!(auras[0].stacks, 1);
    }
}

#[test]
fn crimson_reverie_self_ci_does_not_change_twinflight_nova_captures_or_child() {
    let source = "options duration=15 crit=fixed;
        lacrimosa char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        blackbird char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        blackbird add weapon=fork_twinbirds refine=3;
        zankou char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=1080 resist=0;
        active lacrimosa; blackbird swap; wait(3); lacrimosa swap;";
    let mut req = crate::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    for source_alive in [true, false] {
        if !source_alive {
            req.actors[2].initial_hp = 0;
        }
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let hits: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "ntesim.reaction.nova")
            .collect();
        assert_eq!(hits.len(), 3);
        for hit in hits {
            // Only Twinflight's own108 CI applies to Blackbird.
            // Level80 Nova:45000*(1+108/600)=53100, ignoring DEF and
            // the weapon's ordinary Psyche bonus. Both authored captures and
            // Blackbird's one automatic child keep the captured owner and CI.
            assert_eq!(hit["actor_index"], 1);
            assert_eq!(hit["non_crit_damage"], json!("53100/1"));
        }
        let id = crate::characters::zankou::passives::HYPOTHESIS_ID;
        assert_eq!(
            report["resolved_model_assumptions"][id.as_str()],
            id.description()
        );
    }
}

#[test]
fn crimson_reverie_dead_owner_gets_no_entry_cycle() {
    let mut req = entry_request();
    req.actors[0].initial_hp = 0;
    let (execution, _) = PreparedExecution::new(&req, req.seed, &[], &[]).unwrap();
    assert_eq!(execution.state.actors[0].cycle.current(), 0);
}

#[test]
fn silent_sunset_applies_on_switch() {
    let mut req = entry_request();
    let id = crate::characters::zankou::passives::HYPOTHESIS_ID;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    let base = crate::HypothesisId::CombatEsperCycleReactionsV1;
    req.model_assumptions
        .insert(base.into(), base.description().into());
    let (mut execution, _) = PreparedExecution::new(&req, req.seed, &[], &[]).unwrap();
    execution.switch_actor(&req, &[], 0, 1, 0).unwrap();
    assert_eq!(execution.state.active_actor_index, 1);
    assert!(
        execution
            .state
            .execution_model_assumptions
            .contains_key(crate::characters::zankou::scorch::HYPOTHESIS_ID.as_str())
    );
}

#[test]
fn failed_skill_admission_preserves_earned_entry_and_predecessor_state() {
    let source =
        "options duration=20 crit=fixed; zankou char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active zankou; zankou attack:4; zankou skill;";
    let mut rejected = PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap()
        .request;
    rejected.input_time_contexts.clear();
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } =
        rejected.actions.last_mut().unwrap()
    {
        *timestamp_us = 3_960_936;
    }
    let packets =
        crate::simulation::validation::validate_and_resolve(&rejected, &Default::default())
            .unwrap();
    let (mut execution, _) = PreparedExecution::new(&rejected, rejected.seed, &[], &[]).unwrap();
    execution
        .process_events(&rejected, &Default::default(), &packets, &[], &[])
        .unwrap();
    let actor = &execution.state.actors[0];
    assert_eq!(crate::characters::zankou::clock::command(actor), Some(3));
    assert!(!actor.ability_cooldowns.contains_key("GA_Zankou_Skill"));
    let crate::characters::RuntimeState::Zankou(state) = &actor.character else {
        unreachable!()
    };
    assert_eq!(
        state.skill_variant(3_960_937),
        crate::characters::zankou::forms::Variant::Entry(true)
    );
    assert_eq!(state.normal_stage, 4);
}

#[test]
fn managed_hunt_delusion_overlap_expire_exactly_and_never_change_ally_damage_bonus() {
    let source = "options duration=40 crit=fixed;
        zankou char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0;
        active zankou; zankou attack:4; zankou skill; zankou attack:4; zankou skill;";
    let base = PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap()
        .request;
    for (now, hunt, delusion) in [
        (13_833_889, true, true),
        (13_833_890, false, true),
        (21_409_661, false, true),
        (21_409_662, false, false),
    ] {
        let mut req = base.clone();
        req.relative_duration_us = now;
        let packets =
            crate::simulation::validation::validate_and_resolve(&req, &Default::default()).unwrap();
        let (mut execution, _) = PreparedExecution::new(&req, req.seed, &[], &[]).unwrap();
        execution
            .process_events(&req, &Default::default(), &packets, &[], &[])
            .unwrap();
        let live = BTreeSet::from([0, 1]);
        for owner in [0, 1] {
            let stats = execution
                .state
                .buffs
                .get_stat_modifiers_for_actor_with_living_sources(owner, 0, now, &live)
                .unwrap();
            assert_eq!(
                stats.get("damage_bonus").copied(),
                (hunt && owner == 0).then_some(crate::formula::Ratio {
                    numerator: 1,
                    denominator: 4
                })
            );
            assert_eq!(
                stats.get("crit_damage_bonus_dot").copied(),
                delusion.then_some(crate::formula::Ratio {
                    numerator: 1,
                    denominator: 4
                })
            );
            assert_eq!(
                stats.get("cycle_intensity").copied(),
                (owner == 0).then_some(crate::formula::Ratio {
                    numerator: 100,
                    denominator: 1
                })
            );
        }
    }
}

#[test]
fn permanent_form_modifiers_require_selected_a1_and_a_living_source() {
    for nodes in [vec![], vec![2], vec![1]] {
        let mut req = entry_request();
        req.actions.clear();
        req.relative_duration_us = 6_000_000;
        req.active_actor_index = 1;
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(nodes.clone()).unwrap();
        let (execution, _) = PreparedExecution::new(&req, req.seed, &[], &[]).unwrap();
        let stats = execution
            .state
            .buffs
            .get_stat_modifiers_for_actor_with_living_sources(0, 1, 0, &BTreeSet::from([0, 1]))
            .unwrap();
        assert_eq!(
            stats.get("damage_bonus").copied(),
            nodes.contains(&1).then_some(crate::formula::Ratio {
                numerator: 2,
                denominator: 5
            })
        );
        let ally = execution
            .state
            .buffs
            .get_stat_modifiers_for_actor_with_living_sources(1, 1, 0, &BTreeSet::from([0, 1]))
            .unwrap();
        assert!(!ally.contains_key("damage_bonus"));
        assert_eq!(
            ally.get("crit_damage_bonus_dot").copied(),
            nodes.contains(&1).then_some(crate::formula::Ratio {
                numerator: 1,
                denominator: 2
            })
        );
        let model = crate::HypothesisId::CombatIncomingDamageFixtureV1;
        req.model_assumptions
            .insert(model.into(), model.description().into());
        req.incoming_damage_events
            .push(crate::simulation::GameActorDamageEvent {
                timestamp_us: 5_000_000,
                actor_index: 0,
                delivered_damage: req.actors[0].initial_hp,
            });
        let packets =
            crate::simulation::validation::validate_and_resolve(&req, &Default::default()).unwrap();
        let (mut execution, _) = PreparedExecution::new(&req, req.seed, &[], &[]).unwrap();
        execution
            .process_events(&req, &Default::default(), &packets, &[], &[])
            .unwrap();
        assert!(
            !execution.state.actors[0]
                .live_state_active("hunt", 6_000_000)
                .unwrap()
        );
        assert!(
            !execution.state.actors[0]
                .live_state_active("delusion", 6_000_000)
                .unwrap()
        );
        let ally = execution
            .state
            .buffs
            .get_stat_modifiers_for_actor_with_living_sources(1, 1, 6_000_000, &BTreeSet::from([1]))
            .unwrap();
        assert!(!ally.contains_key("crit_damage_bonus_dot"));
    }
}

#[test]
fn kindling_admission_and_actual_defeat_clear_stored_or_captured_one_stack() {
    let initial = crate::characters::zankou::passives::initial_kindling;
    assert_eq!(initial(None).unwrap(), 0);
    assert_eq!(initial(Some(crate::formula::Ratio::one())).unwrap(), 1);
    assert!(
        initial(Some(crate::formula::Ratio {
            numerator: 2,
            denominator: 1
        }))
        .is_err()
    );
    assert!(
        initial(Some(crate::formula::Ratio {
            numerator: 1,
            denominator: 2
        }))
        .is_err()
    );
    let source = "options duration=20 crit=fixed;
        zankou char lvl=80/80 awakening=[2,4] talent=1,1,1 qte=1 start_energy=120;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        target hp=100000000 def=0 resist=0; active zankou; mint swap; zankou burst;";
    let base = PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap()
        .request;
    let (index, time) = base
        .actions
        .iter()
        .enumerate()
        .find_map(|(index, a)| match a {
            GameActionCommand::InvokeAbility {
                timestamp_us,
                ability_id,
                ..
            } if ability_id == "GA_Zankou_UltraSkill" => Some((index, *timestamp_us)),
            _ => None,
        })
        .unwrap();
    for capture in [false, true] {
        let mut req = base.clone();
        req.actions.truncate(index + usize::from(capture));
        req.input_time_contexts.clear();
        req.relative_duration_us = time + 2_000_000;
        let model = crate::HypothesisId::CombatIncomingDamageFixtureV1;
        req.model_assumptions
            .insert(model.into(), model.description().into());
        req.incoming_damage_events
            .push(crate::simulation::GameActorDamageEvent {
                timestamp_us: time + 1_000_000,
                actor_index: 0,
                delivered_damage: req.actors[0].initial_hp,
            });
        let packets =
            crate::simulation::validation::validate_and_resolve(&req, &Default::default()).unwrap();
        let (mut execution, _) = PreparedExecution::new(&req, req.seed, &[], &[]).unwrap();
        execution
            .process_events(&req, &Default::default(), &packets, &[], &[])
            .unwrap();
        let actor = &execution.state.actors[0];
        assert_eq!(actor.hp, 0);
        let crate::characters::RuntimeState::Zankou(state) = &actor.character else {
            unreachable!()
        };
        assert_eq!(state.kindling_stacks, 0);
        assert!(!state.kindling_active_for_ultra);
        assert_eq!(state.free_inferno_until_us, 0);
        assert!(state.clock.is_none());
    }
}
