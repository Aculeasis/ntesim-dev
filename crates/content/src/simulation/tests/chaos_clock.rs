use super::chaos_actions::make_action;
use super::*;
use serde_json::{Value, json};

pub(super) fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::chaos_crime::request(targets);
    let id = crate::HypothesisId::ChaosOrdinaryClockV1;
    req.model_assumptions
        .insert(id.as_str().into(), id.description().into());
    req.actions.clear();
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
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}

#[test]
fn chaos_ordinary_clock_projects_source_contacts_and_full_chain_recovery_once_per_aoe_packet() {
    for targets in [1, 3] {
        let mut req = request(targets);
        let mut time = 0;
        super::chaos_crime::chain(&mut req, &mut time);
        for command in &mut req.actions {
            if let GameActionCommand::InvokeAbility {
                recovery_us,
                hit_delay_us,
                ..
            } = command
            {
                *recovery_us = 1;
                *hit_delay_us = 1;
            }
        }
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let times: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["target_index"] == 0)
            .map(|hit| hit["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(
            times,
            [
                214_573, 756_182, 918_306, 2_035_883, 2_192_267, 2_433_904, 2_594_110, 2_707_672,
                2_907_616, 4_351_551, 4_510_985, 4_987_471
            ]
        );
        req.actions.push(normal(5_330_913));
        let report = run(&req);
        assert_eq!(report["execution_completed"], false);
        assert_eq!(report["rejected_action"]["reason"]["until_us"], 5_330_914);
        req.actions[5] = normal(5_330_914);
        assert_eq!(run(&req)["execution_completed"], true);
    }
}

#[test]
fn chaos_skill_marker_marks_two_recipients_before_bites_and_uses_independent_cooldown() {
    let mut req = request(3);
    req.actions = vec![skill(0)];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let markers: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "chaos_skill_warrant")
        .collect();
    assert_eq!(markers.len(), 1);
    assert_eq!(markers[0]["time_us"], 283_733);
    let receipts = report["hit_receipts"].as_array().unwrap();
    for (index, hit) in receipts.iter().enumerate() {
        assert_eq!(hit["time_us"], if index < 3 { 298_280 } else { 685_895 });
        // B4 general+.30 on marked targets0/1, neutral target2, DEF .5.
        let damage = match (index < 3, hit["target_index"].as_u64().unwrap() < 2) {
            (true, true) => json!("2145/4"),
            (true, false) => json!("825/2"),
            (false, true) => json!("5655/4"),
            (false, false) => json!("2175/2"),
        };
        assert_eq!(hit["non_crit_damage"], damage);
    }
    req.actions.push(normal(1_054_993));
    assert_eq!(
        run(&req)["rejected_action"]["reason"]["until_us"],
        1_054_994
    );
    req.actions[1] = skill(15_243_639);
    assert_eq!(
        run(&req)["rejected_action"]["reason"]["until_us"],
        15_243_640
    );
    req.actions[1] = skill(15_243_640);
    assert_eq!(run(&req)["execution_completed"], true);
}

#[test]
fn chaos_warrant_expires_from_marker_and_a5_counts_living_marks_instead_of_declared_dummies() {
    for (start, expected) in [(15_069_159, json!("1183/10")), (15_069_160, json!("91/1"))] {
        let mut req = request(3);
        req.actions = vec![skill(0), normal(start)];
        let report = run(&req);
        let hit = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|hit| hit["command_index"] == 1 && hit["target_index"] == 0)
            .unwrap();
        assert_eq!(hit["non_crit_damage"], expected);
    }
    let mut req = request(3);
    req.targets[1].hp_current = 0;
    req.targets[2].hp_current = 0;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([5]).unwrap();
    req.actions = vec![skill(0)];
    assert_eq!(
        run(&req)["hit_receipts"][0]["non_crit_damage"],
        json!("2475/4")
    );
}

#[test]
fn chaos_missed_contacts_add_no_pause_and_sparse_waits_add_no_polling() {
    let mut req = request(1);
    req.targets[0].resistance = Ratio::one();
    req.actions = vec![skill(0), normal(955_994)];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hit_receipts"][1]["time_us"], 636_395);
    req.actions.truncate(1);
    req.relative_duration_us = 30_000_000;
    let ordinary = run(&req);
    req.relative_duration_us = 500_000_000;
    assert_eq!(
        run(&req)["total_events_processed"].as_u64().unwrap(),
        ordinary["total_events_processed"].as_u64().unwrap()
    );
}

#[test]
fn chaos_ordinary_clock_diagnoses_missing_resource_or_form_dependencies() {
    let mut req = request(1);
    req.model_assumptions
        .remove(crate::HypothesisId::ChaosOrdinaryCrimeV1.as_str());
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("ordinary_crime_v1")
    );
    req = request(1);
    req.actors[0].initial_energy = 100;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Chaos071_UltraSkill",
        "GE_Player_Chaos_UltraSkill1_Damage",
        100,
        0,
    )];
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("UNSUPPORTED_CHAOS_FORM")
    );
}

#[test]
fn chaos_death_before_warrant_marker_cancels_pending_montage_requests() {
    let mut req = request(3);
    let id = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(id.as_str().into(), id.description().into());
    req.incoming_damage_events = vec![GameActorDamageEvent {
        timestamp_us: 100_000,
        actor_index: 0,
        delivered_damage: 1_000_000,
    }];
    req.actions = vec![skill(0)];
    let report = run(&req);
    assert_eq!(report["hits_settled"], 0);
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| event["event_kind"] != "chaos_skill_warrant")
    );
    assert_eq!(
        report["final_actor_states"][0]["fractional_resources"]["chaos.crime"]["current"],
        json!("0/1")
    );
}

#[test]
fn chaos_a2_uses_warrant_per_recipient_and_defense_zone_before_damage() {
    let mut req = request(3);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([2]).unwrap();
    req.actions = vec![skill(0)];
    let report = run(&req);
    // Marked: .825*1000*(1+.30)*5/9; unmarked: .825*1000*.5.
    assert_eq!(
        report["hit_receipts"][0]["non_crit_damage"],
        json!("3575/6")
    );
    assert_eq!(report["hit_receipts"][2]["non_crit_damage"], json!("825/2"));
}
