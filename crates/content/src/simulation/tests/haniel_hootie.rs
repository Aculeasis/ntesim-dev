use super::haniel_actions::{make_action, request};
use super::*;
use serde_json::{Value, json};

fn skill(at: u64) -> GameActionCommand {
    make_action(
        at,
        0,
        "GA_Haniel_Skill",
        "GE_Player_Haniel_Skill1_Damage",
        0,
        0,
    )
}

fn normal(at: u64, actor: usize) -> GameActionCommand {
    make_action(
        at,
        actor,
        "GA_Haniel_Melee",
        "GE_Player_Haniel_Melee1_Damage",
        0,
        0,
    )
}

fn run(actions: Vec<GameActionCommand>, awakening: u32, targets: usize) -> Value {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
    req.actions = actions;
    req.relative_duration_us = 25_000_000;
    req.targets = vec![req.targets[0].clone(); targets];
    for (index, target) in req.targets.iter_mut().enumerate() {
        target.target_index = index;
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true, "{report}");
    report
}

fn packets<'a>(report: &'a Value, effect: &str) -> Vec<&'a Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == effect)
        .collect()
}

#[test]
fn haniel_harmony_icd_multitarget_live_stats_and_owned_ensemble() {
    let report = run(
        vec![
            skill(0),
            normal(100_000, 0),
            normal(500_000, 0),
            normal(500_100, 0),
            normal(1_000_100, 0),
            normal(1_500_100, 0),
            normal(2_000_100, 0),
        ],
        0,
        4,
    );
    let ensemble = packets(&report, crate::characters::haniel::hootie::ENSEMBLE);
    assert_eq!(ensemble.len(), 4);
    for (target, hit) in ensemble.iter().enumerate() {
        assert_eq!(hit["target_index"], target);
        assert_eq!(hit["command_index"], 0);
        assert_eq!(hit["time_us"], 2_000_200);
        // Native .20 * 1000 ATK * .5 DEF, no fabricated raw energy.
        assert_eq!(hit["non_crit_damage"], json!("100/1"));
        assert_eq!(
            support::hit_metadata(&report, hit)["action"]["ability_id"],
            "GA_Haniel_Skill"
        );
    }
}

#[test]
fn haniel_ace_symphony_single_use_token_a4_coefficient_and_no_recursion() {
    for awakening in [0, 4] {
        let mut actions = vec![
            skill(0),
            make_action(
                600_000,
                0,
                "GA_Haniel_UltraSkill",
                "GE_Player_Haniel_Ultraskill1_Damage",
                0,
                0,
            ),
        ];
        actions.extend((1..=5).map(|second| normal(second * 1_000_000, 0)));
        let report = run(actions, awakening, 3);
        let ensemble = packets(&report, crate::characters::haniel::hootie::ENSEMBLE);
        let symphony = packets(&report, crate::characters::haniel::hootie::SYMPHONY);
        let extra = packets(&report, crate::characters::haniel::hootie::EXTRA);
        assert_eq!(ensemble.len(), 3);
        assert_eq!(symphony.len(), 3);
        assert_eq!(extra.len(), 1);
        assert_eq!(extra[0]["target_index"], 0);
        assert_eq!(extra[0]["command_index"], 1);
        assert_eq!(
            extra[0]["non_crit_damage"],
            json!(format!("{}/{}", if awakening == 4 { 324 } else { 150 }, 1))
        );
        for hit in symphony {
            assert_eq!(hit["command_index"], 1);
        }
    }
}

#[test]
fn haniel_harmony_recast_preserves_gate_and_expiry_resets() {
    let mut actions = vec![
        skill(0),
        normal(1_000_000, 0),
        normal(2_000_000, 0),
        skill(2_100_000),
        normal(3_000_000, 0),
        normal(4_000_000, 0),
    ];
    // The recast does not reset the three accumulated gates or .5s ICD.
    let report = run(actions.clone(), 0, 1);
    assert_eq!(
        packets(&report, crate::characters::haniel::hootie::ENSEMBLE).len(),
        1
    );
    // A new speaker after natural expiry cannot inherit those gates.
    actions.extend([
        skill(12_000_000),
        normal(13_000_000, 0),
        normal(14_000_000, 0),
    ]);
    let report = run(actions, 0, 1);
    assert_eq!(
        packets(&report, crate::characters::haniel::hootie::ENSEMBLE).len(),
        1
    );
}

#[test]
fn haniel_owned_packets_use_captured_effective_skill_and_burst_coordinates() {
    for (level, awakening, expected, denominator) in
        [(1, 0, 100, 1), (8, 0, 343, 2), (8, 3, 185, 1)]
    {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, level, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![
            skill(0),
            normal(1_000_000, 0),
            normal(2_000_000, 0),
            normal(3_000_000, 0),
            normal(3_600_000, 0),
        ];
        req.relative_duration_us = 4_000_000;
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        let hits = packets(&report, crate::characters::haniel::hootie::ENSEMBLE);
        assert_eq!(hits.len(), 1);
        // At level8, .343 * 1000 * .5 = 171.5; R3 uses .370.
        assert_eq!(
            hits[0]["non_crit_damage"],
            json!(format!("{}/{}", expected, denominator))
        );
    }
}

#[test]
fn haniel_ace_shareable_source_emits_all_three_owned_native_packets() {
    let source = crate::simulation::PreparedSourceScenario::new(
        include_str!("../../../../../examples/haniel-ace-ensemble.nte"),
        Default::default(),
        &ExecutionLimits::default(),
    )
    .unwrap()
    .run(42)
    .unwrap();
    assert!(
        source.report.success && source.report.execution_completed,
        "{:?}",
        source.diagnostic
    );
    super::assert_native_replay(&source);
    let report = serde_json::to_value(source.report).unwrap();
    for effect in [
        crate::characters::haniel::hootie::ENSEMBLE,
        crate::characters::haniel::hootie::SYMPHONY,
        crate::characters::haniel::hootie::EXTRA,
    ] {
        assert!(!packets(&report, effect).is_empty(), "{effect}");
    }
}

#[test]
fn haniel_recast_trigger_keeps_the_previous_speakers_admitted_parent() {
    let report = run(
        vec![
            skill(0),
            normal(1_000_000, 0),
            normal(2_000_000, 0),
            normal(3_000_000, 0),
            skill(4_000_000),
        ],
        0,
        1,
    );
    let hits = packets(&report, crate::characters::haniel::hootie::ENSEMBLE);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0]["command_index"], 0);
}
