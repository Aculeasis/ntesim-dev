use super::haniel_actions::{make_action, request};
use super::*;
use serde_json::{Value, json};

fn skill() -> GameActionCommand {
    make_action(
        0,
        0,
        "GA_Haniel_Skill",
        "GE_Player_Haniel_Skill1_Damage",
        0,
        0,
    )
}
fn burst() -> GameActionCommand {
    make_action(
        600_000,
        0,
        "GA_Haniel_UltraSkill",
        "GE_Player_Haniel_Ultraskill1_Damage",
        0,
        0,
    )
}
fn normal(at: u64) -> GameActionCommand {
    make_action(
        at,
        0,
        "GA_Haniel_Melee",
        "GE_Player_Haniel_Melee1_Damage",
        0,
        0,
    )
}
fn count(report: &Value, effect: &str) -> usize {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == effect)
        .count()
}
fn execute(req: &GameScenarioRequest) -> Value {
    let report = run_game_scenario(req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["rejected_action"]
    );
    report
}

#[test]
fn haniel_immune_startup_still_spawns_but_zero_damage_cannot_feed_harmony() {
    let mut req = request();
    req.relative_duration_us = 5_000_000;
    req.targets[0].resistance = Ratio::one();
    req.actions = vec![
        skill(),
        normal(1_000_000),
        normal(2_000_000),
        normal(3_000_000),
        normal(4_000_000),
    ];
    let report = execute(&req);
    assert_eq!(
        count(&report, crate::characters::haniel::hootie::ENSEMBLE),
        0
    );
    assert_eq!(
        report["final_actor_states"][0]["resources"]["haniel.harmony"]["current"],
        0
    );
    assert!(
        crate::simulation::tests::support::final_buffs(&report)
            .iter()
            .any(|buff| buff["id"] == "haniel.hootie_team_atk")
    );
}

#[test]
fn haniel_token_survives_ace_but_has_its_own_half_open_three_second_deadline() {
    for (next, expected) in [(11_500_000, 1), (12_000_000, 0)] {
        let mut req = request();
        req.relative_duration_us = 13_000_000;
        // Burst supplies the first gate; the third later direct packet arms at 9.0001s.
        req.actions = vec![
            skill(),
            burst(),
            normal(7_000_000),
            normal(8_000_000),
            normal(9_000_000),
            normal(next),
        ];
        let report = execute(&req);
        assert_eq!(
            count(&report, crate::characters::haniel::hootie::SYMPHONY),
            1
        );
        assert_eq!(
            count(&report, crate::characters::haniel::hootie::EXTRA),
            expected
        );
    }
}

#[test]
fn haniel_ensemble_plot_caps_persist_through_speaker_expiry_without_cycle_reset() {
    for (awakening, cap) in [(0, 4), (1, 6)] {
        let mut req = request();
        req.relative_duration_us = 40_000_000;
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![];
        for start in [0, 12_000_000, 24_000_000] {
            req.actions.push(make_action(
                start,
                0,
                "GA_Haniel_Skill",
                "GE_Player_Haniel_Skill1_Damage",
                0,
                0,
            ));
            req.actions
                .extend((1..=8).map(|step| normal(start + step * 700_000)));
        }
        let report = execute(&req);
        assert_eq!(
            count(&report, crate::characters::haniel::hootie::ENSEMBLE),
            6
        );
        let resources = &report["final_actor_states"][0]["resources"];
        assert_eq!(
            resources["haniel.plot_armor"],
            json!({"current":cap,"capacity":cap})
        );
        assert_eq!(resources["haniel.harmony"]["current"], 0);
    }
}

#[test]
fn haniel_off_field_defeat_removes_speaker_ace_tokens_plot_and_team_aura() {
    let mut req = request();
    req.relative_duration_us = 5_000_000;
    let mut mint = req.actors[0].clone();
    mint.character_id = "character:1019".into();
    mint.slot = 1;
    req.actors.push(mint);
    req.actions = vec![
        skill(),
        burst(),
        GameActionCommand::SwitchActor {
            timestamp_us: 700_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    req.actions.extend((1..=4).map(|second| {
        make_action(
            second * 1_000_000,
            1,
            "GA_Mint019_Melee",
            "GE_Player_Mint_Melee1_Damage_New",
            0,
            0,
        )
    }));
    let id = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    req.incoming_damage_events
        .push(crate::simulation::GameActorDamageEvent {
            timestamp_us: 2_500_000,
            actor_index: 0,
            delivered_damage: req.actors[0].initial_hp,
        });
    let report = execute(&req);
    assert_eq!(
        count(&report, crate::characters::haniel::hootie::ENSEMBLE),
        0
    );
    assert_eq!(
        report["final_actor_states"][0]["resources"]["haniel.plot_armor"]["current"],
        0
    );
    assert!(
        crate::simulation::tests::support::final_buffs(&report)
            .iter()
            .all(|buff| buff["source"] != "character:1020")
    );
}
