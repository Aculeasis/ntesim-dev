use super::*;
use crate::simulation::tests::support::formula_damage_total;
const BURST: &str = "GA_Oneiroi_UltraSkill";
const DIRECT: &str = "GE_Player_Oneiroi_UltraSkill_Damage";
const PULSE: &str = "GE_Player_Oneiroi_UltraSkill1_Damage";
fn request_with_moon(moon: Ratio, dummies: usize) -> GameScenarioRequest {
    let mut req = request(dummies);
    req.relative_duration_us = 20_000_000;
    req.actors[0].initial_energy = 100;
    req.actors[0].initial_hp = 1000;
    req.actors[0].initial_resources.moon = Some(moon);
    let model = crate::characters::iroi::dream::HYPOTHESIS_ID;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req
}
fn burst(time: u64) -> GameActionCommand {
    let mut command = action(time, BURST);
    if let GameActionCommand::InvokeAbility {
        energy_cost,
        cooldown_us,
        ..
    } = &mut command
    {
        *energy_cost = 100;
        *cooldown_us = 16_000_000;
    }
    command
}
fn hits(report: &Value, effect: &str) -> Vec<u64> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == effect && hit["target_index"] == 0)
        .map(|hit| hit["time_us"].as_u64().unwrap())
        .collect()
}
fn heals(report: &Value, kind: &str) -> Vec<u64> {
    report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == kind)
        .map(|event| event["time_us"].as_u64().unwrap())
        .collect()
}

#[test]
fn iroi_clock_burst_separates_initial_heal_ivory_hit_release_and_periodic_expiry() {
    for (moon, ivory, periodic, dummies) in [
        (Ratio::zero(), false, 0, 1),
        (
            Ratio {
                numerator: 29_999_999,
                denominator: 1_000_000,
            },
            false,
            3,
            1,
        ),
        (
            Ratio {
                numerator: 30,
                denominator: 1,
            },
            true,
            3,
            1,
        ),
        (
            Ratio {
                numerator: 32,
                denominator: 1,
            },
            true,
            3,
            1,
        ),
        (
            Ratio {
                numerator: 100,
                denominator: 1,
            },
            true,
            12,
            25,
        ),
    ] {
        let mut req = request_with_moon(moon, dummies);
        req.actions = vec![burst(0)];
        let report = run(&req);
        assert_eq!(
            hits(&report, DIRECT),
            if ivory { vec![3_385_389] } else { vec![] }
        );
        assert_eq!(
            hits(&report, PULSE),
            if ivory {
                (1..=periodic)
                    .map(|index| 4_004_747 + index * 1_000_000)
                    .collect::<Vec<_>>()
            } else {
                vec![]
            }
        );
        assert_eq!(heals(&report, "iroi_burst_heal"), [1_925_956]);
        assert_eq!(
            heals(&report, "iroi_dream_periodic_heal").len(),
            periodic as usize
        );
        let effective_dummies = dummies.min(20);
        assert_eq!(
            report["hits_settled"],
            if ivory {
                (periodic + 1) * effective_dummies as u64
            } else {
                0
            }
        );
        assert_eq!(report["final_actor_states"][0]["energy"], 0);
        // Initial2422 healing installs B4 before Q; its separate zone gives20/39 DEF.
        if ivory {
            let damage = ExactFraction::new(
                (5141 + periodic as u128 * 428) * 20 * effective_dummies as u128,
                39,
            )
            .unwrap();
            assert_eq!(
                serde_json::json!(formula_damage_total(&report)),
                serde_json::to_value(damage).unwrap()
            );
        }
    }
}

#[test]
fn iroi_clock_burst_setup_pauses_with_late_skill_but_dream_does_not_follow_later_normals() {
    let mut req = request_with_moon(
        Ratio {
            numerator: 32,
            denominator: 1,
        },
        1,
    );
    req.actions = vec![
        action(0, "GA_Oneiroi_Skill"),
        burst(1_449_905),
        action(5_465_717, "GA_Oneiroi_Melee"),
    ];
    let report = run(&req);
    assert_eq!(heals(&report, "iroi_burst_heal"), [3_375_861]);
    assert_eq!(hits(&report, DIRECT), [4_835_294]);
    assert_eq!(
        hits(&report, PULSE),
        (1..=5)
            .map(|tick| 5_454_652 + tick * 1_000_000)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        heals(&report, "iroi_dream_periodic_heal"),
        hits(&report, PULSE)
    );
    req.actions[2] = action(5_465_716, "GA_Oneiroi_Melee");
    let rejected = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(rejected["rejected_action"]["reason"]["until_us"], 5_465_717);
}

#[test]
fn iroi_clock_burst_interruption_keeps_only_installed_carriers_and_real_cooldown() {
    for (cancel, initial, direct, damage_periodic, healing_periodic) in [
        (1_000_000, false, false, 0, 0),
        (2_000_000, true, false, 0, 3),
        (3_500_000, true, true, 0, 3),
        (3_900_000, true, true, 3, 3),
    ] {
        let mut req = request_with_moon(
            Ratio {
                numerator: 32,
                denominator: 1,
            },
            1,
        );
        req.actions = vec![burst(0), action(cancel, "GA_Oneiroi_Melee")];
        req.interrupt_events.push(GameActorInterruptEvent {
            timestamp_us: cancel,
            actor_index: 0,
            armor_damage: None,
        });
        let report = run(&req);
        assert_eq!(
            heals(&report, "iroi_burst_heal").len(),
            usize::from(initial)
        );
        assert_eq!(hits(&report, DIRECT).len(), usize::from(direct));
        assert_eq!(
            hits(&report, PULSE),
            (1..=damage_periodic)
                .map(|i| cancel + i * 1_000_000)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            heals(&report, "iroi_dream_periodic_heal"),
            (1..=healing_periodic)
                .map(|i| cancel + i * 1_000_000)
                .collect::<Vec<_>>()
        );
        req.actions[1] = burst(15_999_999);
        // Cancelled Burst retains the actual16s cooldown; payment is still insufficient,
        // so refill only this fixture's independent initial pool to distinguish the gate.
        req.actors[0].energy_capacity = 200;
        req.actors[0].initial_energy = 200;
        let rejected = run_game_scenario(&req, &Default::default()).unwrap();
        assert_eq!(
            rejected["rejected_action"]["reason"]["until_us"],
            16_000_000 + cancel
        );
    }
}

#[test]
fn iroi_clock_two_bursts_recharge_through_continuing_normals_and_skill_without_resets() {
    let mut req = request_with_moon(
        Ratio {
            numerator: 32,
            denominator: 1,
        },
        1,
    );
    req.relative_duration_us = 45_000_000;
    req.actions = vec![burst(0)];
    for start in [8_100_000, 12_479_897, 16_859_794, 21_239_691] {
        chain(&mut req, start);
    }
    req.actions.push(action(25_619_588, "GA_Oneiroi_Skill"));
    req.actions.push(burst(27_069_493));
    let report = run(&req);
    assert_eq!(hits(&report, DIRECT), [3_385_389, 30_454_882]);
    let pulses = [
        (1..=3)
            .map(|tick| 4_004_747 + tick * 1_000_000)
            .collect::<Vec<_>>(),
        (1..=12)
            .map(|tick| 31_074_240 + tick * 1_000_000)
            .collect::<Vec<_>>(),
    ]
    .concat();
    assert_eq!(hits(&report, PULSE), pulses);
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["damage_effect_id"] == PULSE
                && hit["time_us"].as_u64().unwrap() > 30_000_000)
            .all(|hit| hit["command_index"] == 22)
    );
    assert_eq!(report["final_actor_states"][0]["energy"], 0);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        json!("493/500")
    );
    assert_eq!(report["final_actor_states"][0]["hp"], 15530);
    // B4 remains live: (4*6064+6862.1+6425+10277)*20/39.
    assert_eq!(
        serde_json::json!(formula_damage_total(&report)),
        serde_json::to_value(ExactFraction::new(956402, 39).unwrap()).unwrap()
    );
    assert_eq!(report, run(&req));
}

#[test]
fn iroi_clock_burst_switch_gate_cancels_unborn_damage_but_keeps_released_healing_off_field() {
    let mut req = request_with_moon(
        Ratio {
            numerator: 32,
            denominator: 1,
        },
        1,
    );
    let mut ally = request(1).actors.remove(0);
    ally.character_id = "character:1019".into();
    ally.slot = 1;
    req.actors.push(ally);
    req.actions = vec![
        burst(0),
        GameActionCommand::SwitchActor {
            timestamp_us: 3_764_481,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    let rejected = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(rejected["rejected_action"]["reason"]["until_us"], 3_764_482);
    if let GameActionCommand::SwitchActor { timestamp_us, .. } = &mut req.actions[1] {
        *timestamp_us += 1;
    }
    let report = run(&req);
    assert_eq!(hits(&report, DIRECT), [3_385_389]);
    assert!(hits(&report, PULSE).is_empty());
    assert_eq!(
        heals(&report, "iroi_dream_periodic_heal"),
        [
            4_764_482, 4_764_482, 5_764_482, 5_764_482, 6_764_482, 6_764_482
        ]
    );
}
