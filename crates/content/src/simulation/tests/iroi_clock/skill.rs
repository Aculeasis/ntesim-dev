use super::*;
use crate::simulation::tests::support::formula_damage_total;

#[test]
fn iroi_clock_two_taps_keep_real_actor_clocks_carrier_charge_and_overlapping_pause() {
    let second = 12_249_777;
    for dummies in [1, 2] {
        let mut req = request(dummies);
        req.relative_duration_us = 16_000_000;
        req.actions = vec![
            action(0, "GA_Oneiroi_Skill"),
            action(second, "GA_Oneiroi_Skill"),
        ];
        let report = run(&req);
        assert_eq!(report["hits_settled"], 20 * dummies);
        let times = |effect: &str| {
            report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|hit| hit["target_index"] == 0 && hit["damage_effect_id"] == effect)
                .map(|hit| hit["time_us"].as_u64().unwrap())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            times("GE_Player_Oneiroi_Skill_Damage"),
            [498799, second + 498799]
        );
        assert_eq!(
            times("GE_Player_Oneiroi_Skill_Gun_Damage"),
            [1698827, second + 1698827]
        );
        assert_eq!(
            times("GE_Player_Oneiroi_Skill_Trumpet_Damage"),
            [1057715, second + 1057715]
        );
        assert_eq!(
            times("GE_Player_Oneiroi_Skill_Syringe_Damage"),
            [0, second]
                .into_iter()
                .flat_map(|start| (0..7).map(move |tick| start + 551115 + tick * 100000))
                .collect::<Vec<_>>()
        );
        assert_eq!(report["final_actor_states"][0]["energy"], 29);
        assert_eq!(
            report["final_actor_states"][0]["charge_remainder"],
            json!("149/250")
        );
        // First Tiny .05 precedes B4; all later packets use DEF ignore .05.
        // Two periods: .05 + (2*6862+.1)*20/39 =7038.05 per dummy.
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            serde_json::to_value(ExactFraction::new(140761 * dummies as u128, 20).unwrap())
                .unwrap()
        );
        assert_eq!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["event_kind"] == "iroi_skill_carrier_energy")
                .count(),
            2
        );
        assert_eq!(report, run(&req));
        if dummies == 1 {
            req.actions[1] = action(second - 1, "GA_Oneiroi_Skill");
            let rejected = run_game_scenario(&req, &Default::default()).unwrap();
            assert_eq!(rejected["rejected_action"]["reason"]["until_us"], second);
            assert_eq!(rejected["hits_settled"], 10);
        }
    }
}

#[test]
fn iroi_clock_late_gun_slows_the_next_montage_without_delaying_its_world_contact() {
    let mut req = request(1);
    req.actions = vec![
        action(0, "GA_Oneiroi_Skill"),
        action(1449905, "GA_Oneiroi_Melee"),
    ];
    let report = run(&req);
    let normal = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["damage_effect_id"] == "GE_Player_Oneiroi_Melee1_Damage")
        .unwrap();
    assert_eq!(normal["time_us"], 1856572);
    let gun = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["damage_effect_id"] == "GE_Player_Oneiroi_Skill_Gun_Damage")
        .unwrap();
    assert_eq!(gun["time_us"], 1698827);
    req.actions[1] = action(1449904, "GA_Oneiroi_Melee");
    let failed = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(failed["rejected_action"]["reason"]["until_us"], 1449905);
}

#[test]
fn iroi_clock_tiny_carrier_does_not_double_charge_when_integer_hp_loss_is_positive() {
    for atk in [1000, 1_000_000] {
        let mut req = request(2);
        req.actors[0].static_stats.atk = Ratio {
            numerator: atk,
            denominator: 1,
        };
        req.actions = vec![action(0, "GA_Oneiroi_Skill")];
        let report = run(&req);
        let tiny = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|hit| hit["damage_effect_id"] == "GE_Player_Oneiroi_Skill_Damage")
            .unwrap();
        assert_eq!(tiny["applied_hp_loss"], if atk == 1000 { 0 } else { 50 });
        assert_eq!(report["final_actor_states"][0]["energy"], 14);
        assert_eq!(
            report["final_actor_states"][0]["charge_remainder"],
            json!("399/500")
        );
        assert_eq!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["event_kind"] == "iroi_skill_carrier_energy")
                .count(),
            1
        );
    }
}

#[test]
fn iroi_clock_skill_interruption_keeps_started_cd_and_only_launched_actors() {
    for (time, cd) in [(200000, 3200000), (400000, 12249777)] {
        let mut req = request(1);
        req.actions = vec![action(0, "GA_Oneiroi_Skill")];
        req.interrupt_events.push(GameActorInterruptEvent {
            timestamp_us: time,
            actor_index: 0,
            armor_damage: None,
        });
        let report = run(&req);
        assert_eq!(
            report["interrupt_receipts"][0]["skill_cooldown_after_us"],
            cd
        );
        assert_eq!(report["hits_settled"], 0);
        assert_eq!(report["final_actor_states"][0]["energy"], 0);
    }
    let mut req = request(1);
    req.actions = vec![
        action(0, "GA_Oneiroi_Skill"),
        action(800000, "GA_Oneiroi_Melee"),
    ];
    req.interrupt_events.push(GameActorInterruptEvent {
        timestamp_us: 800000,
        actor_index: 0,
        armor_damage: None,
    });
    let report = run(&req);
    for (effect, count) in [
        ("GE_Player_Oneiroi_Skill_Gun_Damage", 1),
        ("GE_Player_Oneiroi_Skill_Syringe_Damage", 7),
        ("GE_Player_Oneiroi_Skill_Trumpet_Damage", 0),
    ] {
        assert_eq!(
            report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|hit| hit["damage_effect_id"] == effect)
                .count(),
            count
        );
    }
}

#[test]
fn iroi_clock_skill_without_a_living_contact_keeps_healing_but_no_carrier_charge() {
    let mut req = request(1);
    req.targets[0].hp_current = 1500;
    req.targets[0].hp_max = 1500;
    req.actors[0].initial_hp = 1000;
    chain(&mut req, 0);
    req.actions.truncate(3);
    req.actions.push(action(2330586, "GA_Oneiroi_Skill"));
    let report = run(&req);
    assert_eq!(report["final_actor_states"][0]["hp"], 2616);
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_kind"] == "iroi_skill_carrier_energy")
    );
}

#[test]
fn iroi_clock_a2_assist_triggers_on_skill_cast_and_executes() {
    let mut req = request(1);
    req.actors[0].awakening = serde_json::from_value(json!([2])).unwrap();
    req.actions = vec![
        action(0, "GA_Oneiroi_Melee"),
        action(537029, "GA_Oneiroi_Skill"),
    ];
    let report = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert!(report["hits_settled"].as_u64().unwrap() >= 11);
}

#[test]
fn iroi_clock_healing_samples_live_atk_before_each_a5_recipient_callback() {
    let mut req = request(1);
    req.actors[0].initial_hp = 1000;
    req.actors[0].awakening = serde_json::from_value(json!([5])).unwrap();
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.awakening = Default::default();
    req.actors.push(ally);
    req.actions = vec![action(0, "GA_Oneiroi_Skill")];
    let report = run(&req);
    // Self first:1000*1.616=1616. Its healing grants150 flat ATK to all;
    // the next recipient sees1150*1.616=1858.4, rounded1858.
    assert_eq!(report["final_actor_states"][0]["hp"], 2616);
    assert_eq!(report["final_actor_states"][1]["hp"], 2858);
}
