use super::*;
use serde_json::json;

fn ultra_action(timestamp_us: u64) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index: 0,
        ability_id: "GA_Zankou_UltraSkill".into(),
        damage_effect_id: "GE_Player_Zankou_UltraSkill1_Damage".into(),
        array_index: 0,
        atk_rate: Ratio {
            numerator: 1251,
            denominator: 1000,
        },
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 120,
        recovery_us: 0,
        cooldown_us: 20_000_000,
        hit_delay_us: 0,
    }
}

fn request() -> GameScenarioRequest {
    let mut req = sample_s1_request();
    req.actors[0].character_id = "character:1036".into();
    req.actors[0].weapon_id.clear();
    req.actors[0].gear_procs.clear();
    req.actors[0].set_id = None;
    req.actors[0].talent_levels = Some([1; 3]);
    req.actors[0].character_max_level = Some(80);
    req.actors[0].character_level = 80;
    req.actors[0].initial_energy = 1000;
    req.actors[0].energy_capacity = 1000;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([4]).unwrap();
    let model = crate::characters::zankou::forms::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actors[0].static_stats.crit_damage_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].static_stats.damage_bonus = Ratio::zero();
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.targets[0].hp_max = 10_000_000;
    req.targets[0].hp_current = 10_000_000;
    req.model_assumptions.insert(
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::packet_semantics::ZANKOU_DOT_PACKET_HYPOTHESIS_ID.into(),
        crate::characters::packet_semantics::ZANKOU_DOT_PACKET_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req
}

fn dot_ticks(report: &serde_json::Value) -> Vec<&serde_json::Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Zankou_DotUltraDamage")
        .collect()
}

fn assert_tick_shape(report: &serde_json::Value, tick: &serde_json::Value) {
    assert_eq!(tick["actor_index"], 0);
    assert_eq!(tick["ability_id"], "GA_Zankou_UltraSkill");
    let tags = super::support::hit_metadata(report, tick)["packet"]["semantic_tags"]
        .as_array()
        .unwrap();
    assert!(
        tags.iter().any(|tag| tag == "State.Damage.Dot"),
        "tick at {} missing State.Damage.Dot: {tags:?}",
        tick["time_us"]
    );
}

#[test]
fn bloodfeast_schedules_twenty_nine_half_open_ticks_at_five_stacks() {
    let mut req = request();
    req.relative_duration_us = 40_000_000;
    req.actions = vec![ultra_action(0)];
    let report = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let ticks = dot_ticks(&report);
    assert_eq!(ticks.len(), 29);
    for (i, tick) in ticks.iter().enumerate() {
        assert_eq!(tick["time_us"], 5_697_196 + (i as u64 + 1) * 1_000_000);
        assert_eq!(tick["non_crit_damage"], "75/1");
        assert_tick_shape(&report, tick);
    }
    assert_eq!(
        ticks
            .iter()
            .map(
                |t| serde_json::from_value::<ExactFraction>(t["non_crit_damage"].clone())
                    .unwrap()
                    .numerator
            )
            .sum::<u128>(),
        2175
    );
}

#[test]
fn dot_train_continues_while_source_is_off_field() {
    let mut req = request();
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.awakening = ntesim_data::AwakeningSelection::prefix(0);
    req.actors.push(ally);
    req.relative_duration_us = 40_000_000;
    req.actions = vec![
        ultra_action(0),
        GameActionCommand::SwitchActor {
            timestamp_us: 6_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    let report = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["final_buff_state"]["active_actor_index"], 1);
    let ticks = dot_ticks(&report);
    assert_eq!(ticks.len(), 29);
    for (i, tick) in ticks.iter().enumerate() {
        assert_eq!(tick["time_us"], 5_697_196 + (i as u64 + 1) * 1_000_000);
        assert_eq!(tick["non_crit_damage"], "75/1");
        assert_tick_shape(&report, tick);
    }
}

#[test]
fn ultra_recast_replaces_pending_snapshot_and_refreshes_stacks_and_deadline() {
    let mut req = request();
    req.relative_duration_us = 60_000_000;
    req.actions = vec![ultra_action(0), ultra_action(26_197_196)];
    let report = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let ticks = dot_ticks(&report);
    assert_eq!(ticks.len(), 50);
    for (i, tick) in ticks.iter().enumerate() {
        assert_eq!(
            tick["time_us"],
            5_697_196 + (i as u64 + 1) * 1_000_000 + if i < 20 { 0 } else { 5_697_196 }
        );
        assert_eq!(
            tick["non_crit_damage"],
            if i < 20 { "75/1" } else { "150/1" }
        );
        assert_tick_shape(&report, tick);
    }
    assert_eq!(ticks[49]["time_us"], 61_394_392);
}

#[test]
fn expired_stacks_do_not_leak_into_the_next_cycle() {
    for second in [35_697_196, 36_697_196] {
        let mut req = request();
        req.relative_duration_us = 70_000_000;
        req.actions = vec![ultra_action(0), ultra_action(second)];
        let report = run_game_scenario(&req, &Default::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let ticks = dot_ticks(&report);
        assert_eq!(ticks.len(), 58);
        assert_eq!(ticks[28]["time_us"], 34_697_196);
        assert_eq!(ticks[29]["time_us"], second + 6_697_196);
        assert!(ticks.iter().all(|h| h["non_crit_damage"] == "75/1"));
    }
}

#[test]
fn recast_at_a_tick_settles_that_tick_before_reapplication_without_parallel_trains() {
    let mut req = request();
    req.relative_duration_us = 60_000_000;
    req.actions = vec![ultra_action(0), ultra_action(25_697_196)];
    let report = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let ticks = dot_ticks(&report);
    assert_eq!(ticks.len(), 49);
    assert_eq!(ticks[19]["time_us"], 25_697_196);
    assert_eq!(ticks[19]["non_crit_damage"], "75/1");
    assert_eq!(ticks[20]["time_us"], 32_394_392);
    assert!(ticks[20..].iter().all(|h| h["non_crit_damage"] == "150/1"));
}

#[test]
fn reapplication_during_cinematic_retains_phase_without_emitting_or_catching_up() {
    let mut req = request();
    req.relative_duration_us = 60_000_000;
    req.actions = vec![ultra_action(0), ultra_action(26_197_196)];
    let report = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(report["execution_completed"], true);
    let ticks = dot_ticks(&report);
    assert_eq!(ticks.len(), 50);
    assert!(ticks[..20].iter().all(|h| h["non_crit_damage"] == "75/1"));
    assert_eq!(ticks[19]["time_us"], 25_697_196);
    // The old phase has half a World second left at commitment. Reapplication
    // occurs during the pause and replaces the snapshot without resetting it.
    assert!(
        !ticks
            .iter()
            .any(|h| (26_197_196..31_894_392).contains(&h["time_us"].as_u64().unwrap()))
    );
    assert_eq!(ticks[20]["time_us"], 32_394_392);
    assert!(ticks[20..].iter().all(|h| h["non_crit_damage"] == "150/1"));
}

fn equipped_request(tier: u32) -> GameScenarioRequest {
    let mut req = request();
    let reduction = [(9, 50), (21, 100), (6, 25), (27, 100), (3, 10)][tier as usize - 1];
    req.actors[0].weapon_id = "weapon:fork_NestBird".into();
    req.actors[0].weapon_tier = tier;
    req.actors[0].gear_procs = vec![GearProcConfig::NestBirdGaze {
        damage_reduction: Ratio {
            numerator: reduction.0,
            denominator: reduction.1,
        },
        duration_us: 20_000_000,
    }];
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.weapon_id.clear();
    ally.gear_procs.clear();
    req.actors.push(ally);
    req.actions = vec![
        ultra_action(0),
        GameActionCommand::SwitchActor {
            timestamp_us: 6_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    req
}

fn gaze_marks(report: &serde_json::Value) -> Vec<serde_json::Value> {
    crate::simulation::tests::support::final_buffs(report)
        .into_iter()
        .filter(|buff| buff["id"] == "nest_bird.warning_gaze")
        .collect()
}

#[test]
fn nest_bird_all_refinements_refresh_each_dot_recipient_while_wearer_is_off_field() {
    let reductions = [(9, 50), (21, 100), (6, 25), (27, 100), (3, 10)];
    for tier in 1..=5 {
        let mut req = equipped_request(tier);
        req.relative_duration_us = 48_999_999;
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let ticks = dot_ticks(&report);
        assert_eq!(ticks.len(), 58);
        let marks = gaze_marks(&report);
        assert_eq!(marks.len(), 2);
        let (n, d) = reductions[tier as usize - 1];
        for (index, mark) in marks.iter().enumerate() {
            assert_eq!(mark["owner_actor"], 0);
            assert_eq!(mark["stacks"], 1);
            assert_eq!(mark["recipient"]["target"], index.to_string());
            assert_eq!(mark["duration"]["expires_at_us"], 49_000_000);
            assert_eq!(
                mark["stat_modifiers"]["outgoing_damage_reduction"],
                json!(format!("{}/{}", n, d))
            );
        }
        assert_eq!(report["final_buff_state"]["active_actor_index"], 1);
        for tick in ticks {
            assert_tick_shape(&report, tick);
        }
    }
}

#[test]
fn nest_bird_recast_and_exact_expiry_preserve_full_horizon_damage() {
    let mut req = equipped_request(1);
    req.actions = vec![
        ultra_action(0),
        ultra_action(26_197_196),
        GameActionCommand::SwitchActor {
            timestamp_us: 32_697_196,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    req.relative_duration_us = 69_999_999;
    let report = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let ticks = dot_ticks(&report);
    assert_eq!(ticks.len(), 100);
    let marks = gaze_marks(&report);
    assert_eq!(marks.len(), 2);
    for mark in marks {
        assert_eq!(mark["stacks"], 1);
        assert_eq!(mark["duration"]["expires_at_us"], 70_000_000);
    }
    let total = ticks
        .iter()
        .fold(ExactFraction::new(0, 1).unwrap(), |s, h| {
            s.add(serde_json::from_value(h["non_crit_damage"].clone()).unwrap())
                .unwrap()
        });
    assert_eq!(total, ExactFraction::new(12000, 1).unwrap());
    req.relative_duration_us = 70_000_000;
    let expired = run_game_scenario(&req, &Default::default()).unwrap();
    assert!(gaze_marks(&expired).is_empty());
    assert_eq!(expired["total_damage_dealt"], report["total_damage_dealt"]);
    assert_eq!(expired["hits_settled"], report["hits_settled"]);
}

#[test]
fn vile_ash_application_uses_living_recipients_at_impact() {
    let mut req = equipped_request(1);
    req.targets[0].hp_current = 1;
    req.targets[0].hp_max = 1;
    req.relative_duration_us = 51_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let ticks = dot_ticks(&report);
    assert_eq!(ticks.len(), 29);
    assert!(ticks.iter().all(|hit| hit["target_index"] == 1));
    assert_eq!(report["final_target_states"][0]["hp_current"], 0);
}

#[test]
fn defeated_wearer_stops_future_dot_and_equipment_refresh() {
    let mut req = equipped_request(1);
    let hypothesis = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(hypothesis.into(), hypothesis.description().into());
    req.relative_duration_us = 51_000_000;
    req.incoming_damage_events = vec![GameActorDamageEvent {
        timestamp_us: 6_500_000,
        actor_index: 0,
        delivered_damage: req.actors[0].initial_hp,
    }];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let ticks = dot_ticks(&report);
    assert_eq!(ticks.len(), 12);
    assert_eq!(ticks.last().unwrap()["time_us"], 11_697_196);
    assert_eq!(report["final_actor_states"][0]["hp"], 0);
    assert!(gaze_marks(&report).is_empty());
}

#[test]
fn strict_scheduling_replay_requires_the_new_policy() {
    let mut req = request();
    req.actions = vec![ultra_action(0)];
    req.relative_duration_us = 40_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let assumptions: std::collections::BTreeMap<String, String> =
        serde_json::from_value(report["resolved_model_assumptions"].clone()).unwrap();
    req.model_assumptions.extend(assumptions);
    for actor in &mut req.actors {
        actor.talent_levels = None;
        actor.purchased_skill_level = 1;
    }
    assert_eq!(
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap()["execution_completed"],
        true
    );
}
