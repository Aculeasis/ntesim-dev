use super::*;
use crate::HypothesisId;
use serde_json::json;

#[test]
fn source_lacrimosa_sleep_in_preserves_two_ordinary_deadlines_and_continuing_state() {
    let source = include_str!("../../../../../examples/lacrimosa-sleep-in.nte");
    let prepared =
        PreparedSourceScenario::new(source, Default::default(), &Default::default()).unwrap();
    let prepared = PreparedSourceScenario::from_export(
        &serde_json::from_slice(&serde_json::to_vec(&prepared.export()).unwrap()).unwrap(),
        &Default::default(),
    )
    .unwrap();
    let result = prepared.run(42).unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "success={} completed={} {:?}; {:?}",
        result.report.success,
        result.report.execution_completed,
        result.diagnostic,
        result.report.halt_diagnostic
    );
    super::assert_native_replay(&result);
    let skills: Vec<_> = result
        .request
        .actions
        .iter()
        .filter_map(|command| {
            if let GameActionCommand::InvokeAbility {
                timestamp_us,
                ability_id,
                ..
            } = command
                && ability_id == "GA_Lacrimosa_Skill"
            {
                Some(*timestamp_us)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(skills, [0, 5_200_000, 16_000_000, 21_200_000]);
    assert!(
        result
            .report
            .resolved_model_assumptions
            .contains_key(crate::characters::lacrimosa::sleep_in::MODEL.as_str())
    );
    assert!(result.report.final_actor_states[0].granted_energy > 0);
}

#[test]
fn source_lacrimosa_sleep_in_counts_sections_once_and_expires_unused_permission() {
    for (actions, ready_at, rejected_at) in [
        (
            "lacrimosa skill; lacrimosa attack:4; lacrimosa skill;",
            16_000_000,
            4_400_000,
        ),
        (
            "lacrimosa skill; lacrimosa attack:5; lacrimosa skill; lacrimosa attack:5; lacrimosa skill;",
            16_000_000,
            10_400_000,
        ),
        (
            "lacrimosa skill; lacrimosa attack:5; wait_until(16); lacrimosa skill, skill;",
            32_000_000,
            17_200_000,
        ),
        (
            "lacrimosa skill; lacrimosa attack:4; mint swap; wait(2); lacrimosa swap, attack, skill, attack:5, skill;",
            16_000_000,
            12_400_000,
        ),
    ] {
        let source = format!(
            "options duration=50 crit=random;
            lacrimosa char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
            mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
            target hp=100000000 def=1080 resist=0;
            target hp=100000000 def=1080 resist=0;
            active lacrimosa; {actions}"
        );
        let result = PreparedSourceScenario::new(&source, Default::default(), &Default::default())
            .unwrap()
            .run(42)
            .unwrap();
        assert!(
            !result.report.success && !result.report.execution_completed,
            "{actions}"
        );
        let rejected = result.report.rejected_action.as_ref().unwrap();
        assert_eq!(rejected.time_us, rejected_at, "{actions}");
        assert!(
            matches!(rejected.reason, GameAdmissionReason::Cooldown { until_us } if until_us==ready_at)
        );
        super::assert_native_replay(&result);
    }
}

#[test]
fn lacrimosa_broken_nightmare_without_discord_does_not_trigger_passive() {
    let source = include_str!("../../../../../examples/bohemian-rose-break-cycles.nte");
    let request = crate::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    assert!(
        hits.iter()
            .any(|hit| hit["damage_effect_id"] == "GE_Player_Lacrimosa_Blood_Damage"),
        "Nightmare DoT must still tick on Broken targets"
    );
    assert_eq!(
        hits.iter()
            .filter(|hit| hit["damage_effect_id"] == "GE_Player_Lacrimosa_AnHunZhouTwo_Damage")
            .count(),
        0,
        "Broken targets without Discord must not trigger Tomato Banquet passive"
    );
}

#[test]
fn lacrimosa_passive_triggers_on_discord_reaction() {
    let mut req = super::reactions::request(&[
        (0, "character:1003", 100, 0), // Sakiri (Incantation)
        (1, "character:1004", 100, 0), // Lacrimosa (Chaos)
        (2, "character:1020", 100, 0), // Haniel (Psyche)
    ]);
    req.relative_duration_us = 10_000_000;
    req.targets[0].hp_max = 1_000_000_000;
    req.targets[0].hp_current = 1_000_000_000;
    req.targets[0].break_profile = Some(crate::simulation::GameTargetBreakProfileConfig {
        capacity: Ratio {
            numerator: 100,
            denominator: 1,
        },
        current_gauge: Ratio {
            numerator: 100,
            denominator: 1,
        },
        broken_duration_us: 1_000_000,
        discord_fraction: Some(Ratio {
            numerator: 1,
            denominator: 5,
        }),
    });
    req.actions = vec![
        super::reactions::switch(0, 1),
        super::reactions::switch(1_000_000, 2),
    ];

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let passive_hits = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Lacrimosa_AnHunZhouTwo_Damage")
        .collect::<Vec<_>>();
    assert_eq!(
        passive_hits.len(),
        1,
        "Tomato Banquet must trigger exactly once on Discord reaction"
    );
    let hit = passive_hits[0];
    assert_eq!(
        hit["actor_index"], 1,
        "Lacrimosa is the owner of Tomato Banquet damage"
    );
    assert_eq!(
        hit["scaling_rate"],
        json!("4/1"),
        "Tomato Banquet scales at 400% ATK"
    );
    let origin =
        &report["autonomous_events"][hit["autonomous_event_index"].as_u64().unwrap() as usize];
    assert_eq!(origin["ability_id"], "GA_lacrimosa_Passive_1");
}

#[test]
fn public_lacrimosa_skill_activates_all_target_nightmare_without_equipped_or_test_injection() {
    let source = "options duration=20 crit=fixed;
        lacrimosa char lvl=80/80 awakening=0 talent=8,1,1 qte=1;
        target hp=10000000 def=0 resist=0; target hp=10000000 def=0 resist=0;
        active lacrimosa; lacrimosa skill;";
    let mut req = crate::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    for model in [
        HypothesisId::LacrimosaNightmareDotSchedulingV1,
        HypothesisId::LacrimosaNightmareAllTargetsV1,
        HypothesisId::DamageFixedCritRateAsProbabilityV1,
    ] {
        assert_eq!(req.model_assumptions[model.as_str()], model.description());
    }
    // Isolate the public activation/recipient binding with an independent
    // fixed-ATK oracle. No model declarations or synthetic statuses are added.
    let stats = &mut req.actors[0].static_stats;
    stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    stats.base_atk = 1000;
    stats.crit_damage_bonus = Ratio::zero();
    stats.damage_bonus = Ratio::zero();
    stats.elemental_bonus = Ratio::zero();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    let blood: Vec<_> = hits
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Lacrimosa_Blood_Damage")
        .collect();
    assert_eq!(blood.len(), 28);
    for target in [0, 1] {
        let first = blood
            .iter()
            .find(|hit| hit["target_index"] == target)
            .unwrap();
        // Normal8 .103 * five stacks * ATK1000 = 515. Skill1's talent
        // coordinate must not substitute Normal1's .060 (which gives 300).
        assert_eq!(first["non_crit_damage"], json!("515/1"));
        assert_eq!(first["time_us"], 1_000_000);
        assert!(hits.iter().any(|hit| hit["damage_effect_id"]
            == "GE_Player_Lacrimosa_Skill1_Damage"
            && hit["target_index"] == target));
    }
}

#[test]
fn public_lacrimosa_normal_does_not_fabricate_a_nightmare_application() {
    let source = "options duration=20 crit=fixed; lacrimosa char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=10000000 def=0 resist=0; active lacrimosa; lacrimosa attack;";
    let request = crate::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    assert!(
        !request
            .model_assumptions
            .contains_key(HypothesisId::LacrimosaNightmareAllTargetsV1.as_str())
    );
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|hit| hit["damage_effect_id"] != "GE_Player_Lacrimosa_Blood_Damage")
    );
}
