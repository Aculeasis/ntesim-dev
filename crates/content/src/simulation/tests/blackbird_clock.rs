use super::*;
use serde_json::Value;

fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::blackbird_actions::request();
    let id = crate::HypothesisId::BlackbirdOrdinaryClockV1;
    req.model_assumptions
        .insert(id.as_str().into(), id.description().into());
    req.actors[0].initial_energy = 0;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.relative_duration_us = 30_000_000;
    for target_index in 1..targets {
        let mut target = req.targets[0].clone();
        target.target_index = target_index;
        req.targets.push(target);
    }
    for target in &mut req.targets {
        target.hp_current = 100_000_000;
        target.hp_max = 100_000_000;
    }
    req
}

fn normal(time: u64) -> GameActionCommand {
    let mut action = super::blackbird_actions::request().actions.remove(0);
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut action {
        *timestamp_us = time;
    }
    action
}
fn skill(time: u64) -> GameActionCommand {
    let mut action = normal(time);
    if let GameActionCommand::InvokeAbility {
        ability_id,
        damage_effect_id,
        cooldown_us,
        ..
    } = &mut action
    {
        *ability_id = "GA_BlackBird_Skill".into();
        *damage_effect_id = "GE_Player_BlackBird_Skill_Damage".into();
        // The owner clock, rather than a caller's zero CD, owns this16s deadline.
        *cooldown_us = 0;
    }
    action
}
fn run(req: &GameScenarioRequest) -> Value {
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}
fn times(report: &Value, command: u64, target: u64) -> Vec<u64> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["command_index"] == command && h["target_index"] == target)
        .map(|h| h["time_us"].as_u64().unwrap())
        .collect()
}

#[test]
fn blackbird_clock_skill_integrates_four_causal_pauses_once_per_packet_not_dummy() {
    for targets in [1, 4] {
        let mut req = request(targets);
        req.actions = vec![skill(0), normal(1_451_491)];
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        for target in 0..targets {
            assert_eq!(
                times(&report, 0, target as u64),
                [67_549, 446_023, 776_471, 990_873]
            );
        }
        assert_eq!(times(&report, 1, 0), [1_601_436]);
        req.actions[1] = normal(1_451_490);
        let report = run(&req);
        assert_eq!(report["execution_completed"], false);
        assert_eq!(report["rejected_action"]["reason"]["kind"], "recovery");
    }
}

#[test]
fn blackbird_clock_miss_adds_no_hitstop_and_cooldown_stays_on_real_time() {
    let mut req = request(1);
    req.targets[0].resistance = Ratio::one();
    req.actions = vec![skill(0), normal(1_332_691), skill(16_000_000)];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(times(&report, 0, 0), [67_549, 416_323, 717_071, 901_773]);
    assert_eq!(times(&report, 1, 0), [1_482_636]);
    assert_eq!(
        times(&report, 2, 0),
        [16_067_549, 16_416_323, 16_717_071, 16_901_773]
    );
    req.actions[2] = skill(15_999_999);
    let report = run(&req);
    assert_eq!(report["execution_completed"], false);
    assert_eq!(report["rejected_action"]["reason"]["kind"], "cooldown");
}

#[test]
fn blackbird_clock_full_chain_has_seventeen_contacts_and_recovery_relative_continuity() {
    let mut req = request(1);
    // Independent source recovery + n*.0297s for each stage's1/2/4/5/5 contacts.
    req.actions = [0, 785_620, 1_578_963, 3_427_968, 4_875_378, 5_940_361]
        .map(normal)
        .into();
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hit_receipts"].as_array().unwrap().len(), 18);
    assert_eq!(
        times(&report, 2, 0),
        [1_913_139, 1_992_282, 2_088_999, 2_184_382]
    );
    assert_eq!(times(&report, 5, 0), [6_090_306]);
    // Normal3 can legally take1.849005s, then an explicit1s wait; its own
    // montage duration must not falsely reset the chain to Normal1.
    req.actions.truncate(4);
    req.actions[3] = normal(4_427_968);
    let report = run(&req);
    assert_eq!(times(&report, 3, 0).len(), 5);
    for (delta, count) in [(2_000_000, 2), (2_000_001, 1)] {
        req.actions = vec![normal(0), normal(785_620 + delta)];
        assert_eq!(times(&run(&req), 1, 0).len(), count);
    }
}

#[test]
fn blackbird_clock_first_living_contact_and_native_bird_lifecycle_share_real_time() {
    let mut req = super::blackbird_bird::request(&[
        1_750_000, 3_750_000, 5_750_000, 7_750_000, 9_750_000, 11_750_000,
    ]);
    let id = crate::HypothesisId::BlackbirdOrdinaryClockV1;
    req.model_assumptions
        .insert(id.as_str().into(), id.description().into());
    req.actions[1] = super::reactions::switch(1_500_000, 1);
    req.targets[0].hp_current = 0;
    // Lacrimosa's historical ranged Normal targets dummy0. Use the real Cang
    // AoE producer so a living marked recipient actually receives the trigger.
    req.actors[1].character_id = "character:1023".into();
    let id = crate::HypothesisId::BaicangPowerWordsV1;
    req.model_assumptions
        .insert(id.as_str().into(), id.description().into());
    for action in &mut req.actions[2..] {
        if let GameActionCommand::InvokeAbility {
            ability_id,
            damage_effect_id,
            ..
        } = action
        {
            *ability_id = "GA_Cang_Melee".into();
            *damage_effect_id = "GE_Player_Cang_Melee1_Damage".into();
        }
    }
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(times(&report, 0, 1), [67_549, 446_023, 776_471, 990_873]);
    assert!(!report["autonomous_events"].as_array().unwrap().is_empty());
}
