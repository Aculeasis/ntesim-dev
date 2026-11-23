use super::*;
use serde_json::{Value, json};

fn request(tier: u32) -> GameScenarioRequest {
    let source = include_str!("../../../../../examples/officer-whisker-cycles.nte")
        .replace("refine=1", &format!("refine={tier}"));
    native_request(&source)
}

fn native_request(source: &str) -> GameScenarioRequest {
    let (script, mut config) = crate::dsl::inspection::prepare(source).unwrap();
    let stats = &mut config.actors[0].actor.static_stats;
    stats.base_atk = 1000;
    stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    stats.crit_rate = Ratio::zero();
    stats.damage_bonus = Ratio::zero();
    stats.elemental_bonus = Ratio::zero();
    stats.elemental_bonuses.clear();
    config.actors[0].projection.static_stats = stats.clone();
    let result = crate::simulation::PreparedSourceScenario::from_configuration(
        source,
        config,
        Default::default(),
        &ExecutionLimits::default(),
        &script.options[0].loc,
    )
    .unwrap()
    .run(42)
    .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    super::assert_native_replay(&result);
    result.request
}

fn whisker(report: &Value) -> Vec<&Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["ability_id"] == "weapon:fork_PoliceRat:arc")
        .collect()
}

#[test]
fn officer_whisker_keeps_two_actual_arc_parents_and_live_atk_boss_physical_zones() {
    for (tier, ordinary, boss) in [
        (1, (1150, 1), (2645, 2)),
        (2, (5875, 4), (55225, 32)),
        (3, (1800, 1), (2160, 1)),
        (4, (8575, 4), (84035, 32)),
        (5, (2500, 1), (3125, 1)),
    ] {
        for (retarget, expected) in [(false, ordinary), (true, boss)] {
            let mut req = request(tier);
            if retarget {
                // The ordinary opening attack defeats only the first dummy.
                req.targets[0].hp_current = 1;
                req.targets[0].hp_max = 1;
            }
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
            assert_eq!(commands, [1, 6]);
            let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
            assert_eq!(
                report["execution_completed"], true,
                "{}",
                report["rejected_action"]
            );
            let hits = whisker(&report);
            assert_eq!(hits.len(), 60);
            for (index, hit) in hits.iter().enumerate() {
                let cycle = index / 30;
                let start = [887_510, 60_887_510][cycle];
                assert_eq!(hit["time_us"], start + (index % 30 + 1) as u64 * 1_000_000);
                assert_eq!(hit["command_index"], commands[cycle]);
                assert_eq!(hit["actor_index"], 0);
                assert_eq!(hit["target_index"], usize::from(retarget));
                assert_eq!(
                    hit["non_crit_damage"],
                    json!(format!("{}/{}", expected.0, expected.1))
                );
                let meta = super::support::hit_metadata(&report, hit);
                assert_eq!(meta["packet"]["element"], "physical");
                assert_eq!(
                    meta["action"]["input_role_provenance"],
                    meta["action"]["provenance"]
                );
                assert!(hit["switch_origin"].is_null());
            }
            assert_eq!(report["final_buff_state"]["active_actor_index"], 1);
            assert_eq!(
                report["player_input_receipts"][0]["ability_id"],
                "weapon:fork_PoliceRat:arc"
            );
        }
    }
}

#[test]
fn officer_whisker_reads_a_live_team_atk_window_after_its_wearer_leaves_the_field() {
    let req = native_request(
        "options duration=32 crit=fixed;
         haniel char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
         haniel add weapon=fork_PoliceRat refine=1;
         sakiri char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=120;
         sakiri add weapon=fork_mofeikesi refine=1;
         target hp=100000000 def=0 resist=0 boss=false;
         active haniel; haniel attack, arc; sakiri swap;
         wait_until(5); sakiri burst;",
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["rejected_action"]
    );
    let hits = whisker(&report);
    assert_eq!(hits.len(), 30);
    for hit in hits {
        let time = hit["time_us"].as_u64().unwrap();
        // Static 1000 ATK plus 150 from the weapon; Good Boy's 10% plus 6%
        // control upgrade uses this recipient's 1000 base ATK, giving 1310.
        // Ordinary Sakiri Burst also grants .30 * source base1094 = 328.2.
        let expected = if (5_000_000..25_000_000).contains(&support::world_time_at(&report, time)) {
            "8191/5"
        } else {
            "1150/1"
        };
        assert_eq!(hit["non_crit_damage"], json!(expected));
    }
}

#[test]
fn officer_whisker_stops_at_equal_time_wearer_defeat_without_a_second_cycle_reset() {
    let mut req = request(1);
    req.actions
        .retain(|command| command.timestamp_us() <= 1_000_000);
    req.input_time_contexts.clear();
    req.relative_duration_us = 10_000_000;
    let id = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    req.incoming_damage_events
        .push(crate::simulation::GameActorDamageEvent {
            timestamp_us: 3_887_510,
            actor_index: 0,
            delivered_damage: req.actors[0].initial_hp,
        });
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["rejected_action"]
    );
    let hits = whisker(&report);
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0]["time_us"], 1_887_510);
    assert_eq!(hits[1]["time_us"], 2_887_510);
}
