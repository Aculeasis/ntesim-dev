use super::chaos_actions::make_action;
use super::*;
use serde_json::{Value, json};

fn request(tier: u8) -> GameScenarioRequest {
    let mut req = super::chaos_clock::request(1);
    req.relative_duration_us = 30_000_000;
    let actor = &mut req.actors[0];
    actor.weapon_id = "weapon:fork_GoldWool".into();
    actor.gear_procs = vec![GearProcConfig::GoldWool {
        lakshana_bonus: Ratio {
            numerator: 15 + u64::from(tier) * 5,
            denominator: 100,
        },
        crit_damage_bonus: Ratio {
            numerator: 30 + u64::from(tier) * 10,
            denominator: 100,
        },
        duration_us: 20_000_000,
    }];
    actor.static_stats.crit_rate = Ratio::one();
    actor.static_stats.crit_damage_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };
    req
}

fn skill(time: u64) -> GameActionCommand {
    make_action(
        time,
        0,
        "GA_Chaos071_Skill",
        "GE_Player_Chaos_Skill1_Damage",
        0,
        0,
    )
}

fn normal(time: u64) -> GameActionCommand {
    make_action(
        time,
        0,
        "GA_Chaos071_Melee",
        "GE_Player_Chaos_Melee1_Damage",
        0,
        0,
    )
}

fn run(req: &GameScenarioRequest) -> Value {
    let report = run_game_scenario(req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    report
}

fn starts(report: &Value) -> Vec<u64> {
    report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "gold_wool_crit_damage")
        .map(|event| event["time_us"].as_u64().unwrap())
        .collect()
}

#[test]
fn gold_wool_chaos_all_ranks_grant_at_successful_recovery_after_both_bites() {
    for tier in 1..=5 {
        let mut req = request(tier);
        req.actions = vec![skill(0), normal(1_054_994)];
        let report = run(&req);
        assert_eq!(starts(&report), [1_054_994]);
        for (index, coefficient) in [825, 2175, 182].into_iter().enumerate() {
            // ATK1000 * native coefficient/1000 * (Warrant .30 + permanent) * DEF .5.
            let noncrit =
                ExactFraction::new(coefficient * (145 + u128::from(tier) * 5), 200).unwrap();
            let multiplier = ExactFraction::new(
                150 + if index == 2 {
                    30 + u128::from(tier) * 10
                } else {
                    0
                },
                100,
            )
            .unwrap();
            let hit = &report["hit_receipts"][index];
            assert_eq!(
                hit["non_crit_damage"],
                serde_json::to_value(noncrit).unwrap()
            );
            assert_eq!(
                hit["crit_damage"],
                serde_json::to_value(noncrit.mul(multiplier).unwrap()).unwrap()
            );
        }
    }
}

#[test]
fn gold_wool_chaos_interruption_cancels_only_the_unfinished_grant() {
    for (time, expected) in [(1_054_993, vec![]), (1_054_995, vec![1_054_994])] {
        let mut req = request(1);
        let model = crate::HypothesisId::ChaosForcedInterruptV1;
        req.model_assumptions
            .insert(model.into(), model.description().into());
        req.interrupt_events = vec![crate::simulation::GameActorInterruptEvent {
            timestamp_us: time,
            actor_index: 0,
            armor_damage: None,
        }];
        req.actions = vec![skill(0), normal(2_000_000)];
        let report = run(&req);
        assert_eq!(starts(&report), expected);
        assert_eq!(report["hits_settled"], 3);
        assert_eq!(
            report["final_actor_states"][0]["fractional_resources"]["chaos.crime"]["granted"],
            json!("122/1")
        );
        let hit = &report["hit_receipts"][2];
        assert_eq!(
            hit["crit_damage"],
            if time < 1_054_994 {
                json!("819/4")
            } else {
                json!("5187/20")
            }
        );
    }
}

#[test]
fn gold_wool_chaos_no_damage_finish_does_not_invent_hitstop_or_require_a_hit() {
    let mut req = request(1);
    req.targets[0].resistance = Ratio::one();
    req.actions = vec![skill(0), normal(955_994)];
    let report = run(&req);
    assert_eq!(starts(&report), [955_994]);
    assert_eq!(report["hit_receipts"][1]["time_us"], 636_395);
    assert_eq!(
        report["final_actor_states"][0]["fractional_resources"]["chaos.crime"]["granted"],
        json!("0/1")
    );
}

#[test]
fn gold_wool_chaos_successful_window_expires_at_exact_finish_plus_twenty_seconds() {
    for (start, numerator, denominator) in [(20_840_420, 5187, 25), (20_840_421, 819, 5)] {
        let mut req = request(1);
        req.actions = vec![skill(0), normal(start)];
        let report = run(&req);
        assert_eq!(starts(&report), [1_054_994]);
        assert_eq!(
            report["hit_receipts"][2]["crit_damage"],
            json!(format!("{}/{}", numerator, denominator))
        );
    }
}

#[test]
fn gold_wool_chaos_burst_commits_immediately_and_skill_refreshes_one_existing_window() {
    let mut req = request(1);
    let model = crate::HypothesisId::ChaosPrimaryFormV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.actors[0].initial_energy = 100;
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Chaos071_UltraSkill",
            "GE_Player_Chaos_UltraSkill1_Damage",
            100,
            0,
        ),
        skill(6_000_000),
        normal(7_054_994),
    ];
    let report = run(&req);
    assert_eq!(starts(&report), [0, 7_054_994]);
    for hit in report["hit_receipts"].as_array().unwrap() {
        let damage: Ratio = serde_json::from_value(hit["non_crit_damage"].clone()).unwrap();
        assert_eq!(
            hit["crit_damage"],
            serde_json::to_value(
                ExactFraction::from_ratio(damage)
                    .unwrap()
                    .mul(ExactFraction::new(19, 10).unwrap())
                    .unwrap()
            )
            .unwrap()
        );
    }
}

#[test]
fn gold_wool_chaos_inherited_riposte_pause_and_both_bite_pauses_delay_realfinish() {
    let mut req = request(1);
    let model = crate::HypothesisId::ChaosSecondaryActionsV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    super::blackbird_secondary::stamina(&mut req, 100);
    req.actions = vec![
        GameActionCommand::PlayerInput {
            timestamp_us: 0,
            actor_index: 0,
            input: crate::simulation::PlayerInput::Dash,
            critical_success: true,
        },
        normal(998_284),
        skill(1_248_442),
    ];
    let report = run(&req);
    assert_eq!(starts(&report), [2_600_435]);
    assert_eq!(report["hit_receipts"][1]["time_us"], 1_843_721);
}

#[test]
fn gold_wool_chaos_death_before_realfinish_does_not_grant_a_window() {
    let mut req = request(1);
    let model = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.incoming_damage_events
        .push(crate::simulation::GameActorDamageEvent {
            timestamp_us: 1_000_000,
            actor_index: 0,
            delivered_damage: req.actors[0].initial_hp,
        });
    req.actions = vec![skill(0)];
    let report = run(&req);
    assert!(starts(&report).is_empty());
    assert_eq!(report["hits_settled"], 2);
}

#[test]
fn gold_wool_chaos_field_exit_cancels_unfinished_grant_but_preserves_an_earlier_window() {
    for prior in [false, true] {
        let mut req = request(1);
        let mut ally = req.actors[0].clone();
        ally.slot = 1;
        ally.weapon_id.clear();
        ally.gear_procs.clear();
        req.actors.push(ally);
        if prior {
            req.actors[0].initial_energy = 100;
            let model = crate::HypothesisId::ChaosPrimaryFormV1;
            req.model_assumptions
                .insert(model.into(), model.description().into());
            req.actions.push(make_action(
                0,
                0,
                "GA_Chaos071_UltraSkill",
                "GE_Player_Chaos_UltraSkill1_Damage",
                100,
                0,
            ));
        }
        let time = if prior { 6_000_000 } else { 0 };
        req.actions.push(skill(time));
        // A natural switch cannot bypass recovery. An authored forced interrupt
        // removes the montage, then the accepted exit leaves old gear intact.
        let model = crate::HypothesisId::ChaosForcedInterruptV1;
        req.model_assumptions
            .insert(model.into(), model.description().into());
        req.interrupt_events
            .push(crate::simulation::GameActorInterruptEvent {
                timestamp_us: time + 1_000_000,
                actor_index: 0,
                armor_damage: None,
            });
        req.actions.push(GameActionCommand::SwitchActor {
            timestamp_us: time + 1_000_001,
            target_actor_index: 1,
            cooldown_us: 0,
        });
        req.actions.push(GameActionCommand::SwitchActor {
            timestamp_us: time + 2_000_000,
            target_actor_index: 0,
            cooldown_us: 0,
        });
        req.actions.push(normal(time + 2_000_001));
        let report = run(&req);
        assert_eq!(starts(&report), if prior { vec![0] } else { vec![] });
        let last = report["hit_receipts"].as_array().unwrap().last().unwrap();
        let damage: Ratio = serde_json::from_value(last["non_crit_damage"].clone()).unwrap();
        assert_eq!(
            last["crit_damage"],
            serde_json::to_value(
                ExactFraction::from_ratio(damage)
                    .unwrap()
                    .mul(ExactFraction::new(if prior { 19 } else { 15 }, 10).unwrap())
                    .unwrap()
            )
            .unwrap()
        );
    }
}
