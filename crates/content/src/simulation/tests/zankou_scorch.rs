use super::*;
use serde_json::{Value, json};

fn request(two_cycles: bool) -> GameScenarioRequest {
    let tail = if two_cycles {
        "wait(10); adler swap; wait(1.2); adler skill;"
    } else {
        ""
    };
    let source = format!(
        "options duration=45 crit=fixed;
        adler char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        lacrimosa char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        zankou char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=1080 resist=0;
        active adler; adler skill; lacrimosa swap; {tail}"
    );
    crate::dsl::compiler::compile_scenario(&source, Default::default())
        .unwrap()
        .request
}

fn scorch(report: &Value) -> Vec<&Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "ntesim.reaction.scorch")
        .collect()
}

fn damage_with_live_temperance(report: &Value, hit: &Value, damage: u64) -> ExactFraction {
    let time = hit["time_us"].as_u64().unwrap();
    let has_resistance_reduction = report["trace"].as_array().unwrap().iter().any(|event| {
        let applied = event["time_us"].as_u64().unwrap();
        event["event_kind"] == "adler_temperance_applied"
            && event["data"]["buff_id"] == "adler.temperance_resistance"
            && event["data"]["target_index"] == hit["target_index"]
            && (applied..applied + 15_000_000).contains(&time)
    });
    // At base RES 0, a .10 reduction gives 1 + .10/1.10 = 12/11.
    if has_resistance_reduction {
        ExactFraction::new(u128::from(damage) * 12, 11).unwrap()
    } else {
        ExactFraction::new(damage.into(), 1).unwrap()
    }
}

#[test]
fn silent_sunset_adds_capped_stacks_recaptures_zankou_and_preserves_the_original_phase() {
    let req = request(false);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = scorch(&report);
    // Scorch anchors at the switch at1.2, never at DoT callbacks2..10.
    // Final callback10 refreshes fifteen seconds; ticks2.19..24.96 are24.
    assert_eq!(hits.len(), 24);
    for (index, hit) in hits.iter().enumerate() {
        assert_eq!(hit["time_us"], 1_200_000 + 990_000 * (index as u64 + 1));
        assert_eq!(
            hit["actor_index"], 2,
            "neither original switch participant owns the replacement"
        );
        // Level80/CI100: 2700*(1+100/600)*.5=1575 per stack.
        let damage = if index == 0 { 3150 } else { 4725 };
        assert_eq!(
            hit["non_crit_damage"],
            json!(damage_with_live_temperance(&report, hit, damage))
        );
        let origin = &hit["switch_origin"];
        assert_eq!(origin["kind"], "esper_cycle_replacement");
        assert_eq!(origin["outgoing_actor_index"], 0);
        assert_eq!(origin["incoming_actor_index"], 1);
        assert_eq!(origin["owner_actor_index"], 2);
        let parent_index = origin["trigger_hit_index"].as_u64().unwrap() as usize;
        let parent = &report["hit_receipts"][parent_index];
        assert_eq!(parent["actor_index"], 0);
        assert_eq!(parent["target_index"], hit["target_index"]);
        assert!(parent["time_us"].as_u64().unwrap() < hit["time_us"].as_u64().unwrap());
        assert!(
            super::support::hit_metadata(&report, parent)["packet"]["semantic_tags"]
                .as_array()
                .unwrap()
                .iter()
                .any(|tag| tag == crate::characters::packet_semantics::DOT_DAMAGE_TAG)
        );
    }
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "reaction_scorch_replacement")
            .count(),
        9,
        "Scorch's own24 ticks never reenter the listener"
    );
}

#[test]
fn silent_sunset_uses_new_scorch_payload_instead_of_ally_damage_and_keeps_two_cycles() {
    let mut req = request(true);
    req.targets[0].elemental_resistances.insert(
        "CHAOS".into(),
        Ratio {
            numerator: 1,
            denominator: 2,
        },
    );
    let ordinary = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    req.actors[0].static_stats.atk = Ratio {
        numerator: 100000,
        denominator: 1,
    };
    let stronger_dot = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    for report in [&ordinary, &stronger_dot] {
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert!(
            scorch(report)
                .iter()
                .filter(|hit| hit["switch_origin"]["kind"] == "esper_cycle_replacement")
                .all(
                    |hit| [3150, 4725].into_iter().any(|n| hit["non_crit_damage"]
                        == json!(damage_with_live_temperance(report, hit, n)))
                )
        );
        assert_eq!(
            report["action_metadata_ledger"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|entry| entry["actor_index"] == 0
                    && super::support::action_metadata(report, entry)["action"]["ability_id"]
                        == "GA_Adler_Skill")
                .map(|entry| entry["command_indices"].as_array().unwrap().len())
                .sum::<usize>(),
            2
        );
    }
}

#[test]
fn silent_sunset_captures_team_dot_critical_damage_in_its_own_zone() {
    let mut req = request(false);
    req.seed = 42;
    req.actors[2].awakening = ntesim_data::AwakeningSelection::prefix(1);
    req.actors[2].static_stats.crit_damage_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let hits = scorch(&report);
    assert!(hits.iter().any(|hit| hit["critical"] == true));
    for (index, hit) in hits.iter().enumerate() {
        let base = if index == 0 { 3150 } else { 4725 };
        let damage = if hit["critical"] == true {
            base * 2
        } else {
            base
        };
        assert_eq!(
            hit["damage"],
            json!(damage_with_live_temperance(&report, hit, damage)),
            "ordinary CD.5 plus A1 DoT CD.5, separately from stack multiplication"
        );
        assert_eq!(
            super::support::hit_metadata(&report, hit)["packet"]["native_damage_type_ex"],
            "EExecutionDamageTypeEX::DAMAGE_TYPE_INCANTATION"
        );
    }
}

#[test]
fn silent_sunset_requires_living_zankou_and_keeps_armed_snapshots_after_his_death() {
    let mut req = request(false);
    req.actors[2].initial_hp = 0;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(scorch(&report).len(), 15);
    assert!(scorch(&report).iter().all(|hit| hit["actor_index"] == 0
        && hit["non_crit_damage"] == json!(damage_with_live_temperance(&report, hit, 1350))));
    req.actors[2].initial_hp = req.actors[2].initial_hp_max;
    let id = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    req.incoming_damage_events
        .push(crate::simulation::GameActorDamageEvent {
            timestamp_us: 2_500_000,
            actor_index: 2,
            delivered_damage: req.actors[2].initial_hp,
        });
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    // Only the callback at2s ran: two stacks remain armed until17s.
    assert_eq!(scorch(&report).len(), 15);
    assert!(scorch(&report).iter().all(|hit| hit["actor_index"] == 2
        && hit["non_crit_damage"] == json!(damage_with_live_temperance(&report, hit, 3150))));
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["event_kind"] == "reaction_scorch_replacement")
            .count(),
        1
    );
}

#[test]
fn silent_sunset_dot_ticks_do_not_retrigger_discord_against_active_nova() {
    let source = "options duration=15 crit=fixed;
        sakiri char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        lacrimosa char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        haniel char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        zankou char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=1080 resist=0 original_atk=1000 break_gauge=30 max_break_gauge=30 broken_duration=10 discord_fraction=0.1;
        active sakiri;
        lacrimosa swap; wait_until(0.6);
        lacrimosa skill;
        haniel swap;
        wait(10);";
    let req = crate::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let discord_events: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "reaction_discord_triggered")
        .collect();
    assert_eq!(
        discord_events.len(),
        1,
        "Discord reaction must be emitted exactly once"
    );
    let tomato_hits: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Lacrimosa_AnHunZhouTwo_Damage")
        .collect();
    assert_eq!(
        tomato_hits.len(),
        1,
        "Lacrimosa Tomato Banquet passive must trigger exactly once on Discord"
    );
    let scorch_replacements = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "reaction_scorch_replacement")
        .count();
    assert!(
        scorch_replacements > 1,
        "Multiple ally DoT ticks refreshed Scorch"
    );
}
