use super::*;
use crate::simulation::tests::support::formula_damage_total;
use serde_json::{Value, json};
mod held;
mod secondary;

fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::mint_actions::request();
    req.actors[0].talent_levels = Some([8; 3]);
    req.actors[0].initial_energy = 0;
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].hp_max = 100_000_000;
    for target in 1..targets {
        let mut next = req.targets[0].clone();
        next.target_index = target;
        req.targets.push(next);
    }
    req.actions.clear();
    req.relative_duration_us = 120_000_000;
    req.model_assumptions.insert(
        crate::HypothesisId::MintOrdinaryClockV1.into(),
        crate::HypothesisId::MintOrdinaryClockV1
            .description()
            .into(),
    );
    req
}
fn action(time: u64, ability: &str) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us: time,
        actor_index: 0,
        ability_id: ability.into(),
        damage_effect_id: crate::characters::mint::primary_packet(ability)
            .unwrap()
            .into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: NativeScalingStat::Atk,
        energy_cost: if ability == "GA_Mint019_UltraSkill" {
            100
        } else {
            0
        },
        recovery_us: 0,
        cooldown_us: if ability == "GA_Mint019_Skill" {
            6_000_000
        } else if ability == "GA_Mint019_UltraSkill" {
            15_000_000
        } else {
            0
        },
        hit_delay_us: 100,
    }
}
fn chain(req: &mut GameScenarioRequest, start: u64) {
    for offset in [0, 848860, 1544304, 2777262, 3552663] {
        req.actions.push(action(start + offset, "GA_Mint019_Melee"));
    }
}
fn run(req: &GameScenarioRequest) -> Value {
    let report = run_game_scenario(req, &Default::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    report
}

#[test]
fn mint_clock_two_chains_keep_distinct_contacts_and_one_owner_pause_across_dummies() {
    for targets in [1, 3] {
        let mut req = request(targets);
        chain(&mut req, 0);
        chain(&mut req, 4_598_213);
        let report = run(&req);
        assert_eq!(report["hits_settled"], 16 * targets);
        let hits: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| h["target_index"] == 0)
            .map(|h| h["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(
            &hits[..8],
            &[
                154636, 326256, 938374, 1786207, 1923059, 2927197, 3811981, 4152740
            ]
        );
        assert_eq!(hits[8], 4_598_213 + 154636);
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            json!(format!("{}/{}", 3546 * targets, 1))
        );
        req.actions.push(action(9_196_425, "GA_Mint019_Melee"));
        let failed = run_game_scenario(&req, &Default::default()).unwrap();
        assert_eq!(failed["rejected_action"]["reason"]["until_us"], 9_196_426);
    }
    let mut req = request(3);
    for target in &mut req.targets {
        target.resistance = Ratio::one();
    }
    req.actions = vec![
        action(0, "GA_Mint019_Melee"),
        action(832_326, "GA_Mint019_Melee"),
    ];
    let failed = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(failed["rejected_action"]["reason"]["until_us"], 832_327);
    let hits: Vec<_> = failed["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["target_index"] == 0)
        .collect();
    assert_eq!(hits[0]["time_us"], 154_636);
    assert_eq!(hits[1]["time_us"], 309_723);
    assert!(hits.iter().all(|h| h["applied_hp_loss"] == 0));
}

#[test]
fn mint_clock_skill_notify_and_real_interrupt_keep_one_live_deadline_without_refunds() {
    for (interrupt, next, hits) in [
        (100000, 3_100_000, 0),
        (190000, 6_182_791, 0),
        (300000, 6_182_791, 1),
    ] {
        let mut req = request(2);
        req.actions = vec![
            action(0, "GA_Mint019_Skill"),
            action(next, "GA_Mint019_Skill"),
        ];
        req.interrupt_events.push(GameActorInterruptEvent {
            timestamp_us: interrupt,
            actor_index: 0,
            armor_damage: None,
        });
        let report = run(&req);
        assert_eq!(report["hits_settled"], 2 * (2 + hits));
        assert_eq!(
            report["interrupt_receipts"][0]["skill_cooldown_after_us"],
            next
        );
        let second: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| h["command_index"] == 1 && h["target_index"] == 0)
            .map(|h| h["time_us"].as_u64().unwrap() - next)
            .collect();
        assert_eq!(second, [200178, 565424]);
    }
    // Interruption cannot erase the remaining30.178ms of the first contact's
    // .01-rate pause. The following Normal inherits29.87622ms lost progress.
    let mut req = request(2);
    req.actions = vec![
        action(0, "GA_Mint019_Skill"),
        action(220_000, "GA_Mint019_Melee"),
    ];
    req.interrupt_events.push(GameActorInterruptEvent {
        timestamp_us: 210_000,
        actor_index: 0,
        armor_damage: None,
    });
    let report = run(&req);
    let next = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["command_index"] == 1)
        .unwrap();
    assert_eq!(next["time_us"], 404_513);
    assert_eq!(
        report["interrupt_receipts"][0]["skill_cooldown_after_us"],
        6_182_791
    );
}

#[test]
fn mint_clock_burst_switch_gate_remains_distinct_from_ordinary_recovery() {
    let mut req = request(1);
    req.actors[0].initial_energy = 100;
    let mut ally = crate::dsl::compiler::compile_scenario(
        "options duration=10; zero char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
         target hp=100000000 def=1080 resist=0; active zero;",
        Default::default(),
    )
    .unwrap()
    .request
    .actors
    .remove(0);
    ally.slot = 1;
    req.actors.push(ally);
    req.actions = vec![
        action(0, "GA_Mint019_UltraSkill"),
        GameActionCommand::SwitchActor {
            timestamp_us: 3_792_166,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    let failed = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(failed["rejected_action"]["reason"]["until_us"], 3_792_167);
    if let GameActionCommand::SwitchActor { timestamp_us, .. } = &mut req.actions[1] {
        *timestamp_us = 3_792_167;
    }
    let report = run(&req);
    assert_eq!(report["hits_settled"], 7);
    assert_eq!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| h["actor_index"] == 0)
            .count(),
        7
    );
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
}

#[test]
fn mint_clock_zero_energy_two_periods_fund_two_bursts_without_resetting_carry_or_cooldowns() {
    let mut req = request(2);
    for start in [0, 60_000_000] {
        for index in 0..9 {
            chain(&mut req, start + index * 5_000_000);
        }
        req.actions
            .push(action(start + 45_000_000, "GA_Mint019_Skill"));
        req.actions
            .push(action(start + 52_000_000, "GA_Mint019_Skill"));
        req.actions
            .push(action(start + 54_000_000, "GA_Mint019_UltraSkill"));
    }
    let report = run(&req);
    assert_eq!(report["hits_settled"], 332);
    assert_eq!(
        serde_json::json!(formula_damage_total(&report)),
        json!("103592/1")
    );
    let actor = &report["final_actor_states"][0];
    assert_eq!(actor["energy"], 0);
    assert_eq!(actor["spent_energy"], 200);
    assert_eq!(actor["charge_remainder"], json!("291/500"));
    assert_eq!(report, run(&req));
}
