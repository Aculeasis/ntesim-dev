use super::daffodil_actions::{make_action, request};
use super::*;
use serde_json::{Value, json};

fn qte(time: u64) -> GameActionCommand {
    make_action(
        time,
        0,
        "GA_Daffodill_QTE",
        "GE_Player_Daffodill_QTE_Damage",
        0,
        0,
    )
}

fn skill(time: u64) -> GameActionCommand {
    make_action(
        time,
        0,
        "GA_Daffodill_Skill",
        "GE_Player_Daffodill_Skill1_Damage",
        0,
        16_000_000,
    )
}

fn damage(report: &Value, command: usize) -> ExactFraction {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["command_index"] == command)
        .try_fold(ExactFraction::new(0, 1).unwrap(), |sum, hit| {
            sum.add(
                serde_json::from_value::<ExactFraction>(hit["damage"].clone())
                    .map_err(|err| err.to_string())?,
            )
        })
        .unwrap()
}

#[test]
fn daffodil_resonance_two_continuing_cycles_capture_stacks_echoes_damage_and_gauge() {
    // Lv1, ATK1000, DEF factor1/2. Ordinary sum3.001; Echoes6*.348+.911=2.999.
    // Ordinary raw gauge2*.164+5*.135+.199=1.202; Echoes6*.139+.365=1.199.
    for (a1, supports, expected_damage, expected_loss, packets) in [
        (false, 0, (3001, 2), (601, 500), 8),
        (true, 0, (3001, 2), (601, 500), 8),
        (false, 1, (9003, 4), (5409, 2000), 8),
        (true, 1, (3001, 1), (4207, 1000), 8),
        (false, 2, (2999, 1), (8393, 2000), 7),
        (true, 2, (8997, 2), (3597, 500), 7),
        (false, 3, (2999, 1), (1199, 250), 7),
    ] {
        let mut req = request();
        req.relative_duration_us = 22_000_000;
        // Nonneutral base distinguishes additive owner efficiency from a second multiplier.
        if supports == 3 {
            req.actors[0].static_stats.break_efficiency_multiplier = Ratio {
                numerator: 3,
                denominator: 2,
            };
        }
        req.actors[0].awakening =
            ntesim_data::AwakeningSelection::selected(if a1 { vec![1] } else { vec![] }).unwrap();
        req.targets[0].hp_current = 10_000_000;
        req.targets[0].break_profile = Some(GameTargetBreakProfileConfig {
            capacity: Ratio {
                numerator: 100,
                denominator: 1,
            },
            current_gauge: Ratio {
                numerator: 100,
                denominator: 1,
            },
            broken_duration_us: 1_000_000,
            discord_fraction: None,
        });
        req.actions.clear();
        let mut commands = Vec::new();
        for cycle in 0..2 {
            let start = cycle * 20_000_000;
            for support in 0..supports {
                req.actions.push(qte(start + support * 1000));
            }
            commands.push(req.actions.len());
            req.actions.push(skill(start + 10_000));
        }
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["execution_completed"], true, "{report}");
        for command in commands {
            assert_eq!(
                damage(&report, command),
                ExactFraction::new(expected_damage.0, expected_damage.1).unwrap(),
                "a1={a1}, supports={supports}"
            );
            assert_eq!(
                report["hit_receipts"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|hit| hit["command_index"] == command)
                    .count(),
                packets
            );
        }
        let loss = ExactFraction::new(expected_loss.0, expected_loss.1)
            .unwrap()
            .add(if supports == 3 {
                ExactFraction::new(u128::from(supports) * 15, 4).unwrap()
            } else {
                ExactFraction::new(u128::from(supports) * 5, 2).unwrap()
            })
            .unwrap()
            .mul(ExactFraction::new(2, 1).unwrap())
            .unwrap();
        let gauge = ExactFraction::new(100, 1).unwrap().sub(loss).unwrap();
        assert_eq!(
            report["final_target_states"][0]["break_state"]["gauge"],
            serde_json::to_value(gauge).unwrap(),
            "a1={a1}, supports={supports}"
        );
        assert_eq!(report["final_actor_states"][0]["energy_conserved"], true);
    }
}

#[test]
fn daffodil_resonance_later_support_does_not_rewrite_pending_skill_capture() {
    let mut req = request();
    req.relative_duration_us = 22_000_000;
    req.actions = vec![
        qte(0),
        skill(1000),
        qte(2000),
        skill(20_000_000),
        qte(20_001_000),
        skill(21_000_000),
    ];
    if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut req.actions[1] {
        *hit_delay_us = 10_000;
    }
    // The third Skill is intentionally unready: no silent wait, no extra capture.
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert_eq!(damage(&report, 1), ExactFraction::new(9003, 4).unwrap());
    assert_eq!(damage(&report, 3), ExactFraction::new(9003, 4).unwrap());
    assert_eq!(damage(&report, 5), ExactFraction::new(0, 1).unwrap());
    assert_eq!(
        report["final_actor_states"][0]["resources"]["daffodil.resonance"]["current"],
        1
    );
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "daffodil_resonance_granted")
            .count(),
        3
    );
    assert_eq!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|hit| hit["command_index"] == 1)
            .unwrap()["damage_effect_id"],
        json!("GE_Player_Daffodill_Skill1_Damage")
    );
}

#[test]
fn daffodil_resonance_teammate_support_is_admission_scoped_and_survives_field_changes() {
    let mut req = request();
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    req.actors.push(ally);
    let mut target = req.targets[0].clone();
    target.target_index = 1;
    req.targets.push(target);
    req.actions = vec![
        GameActionCommand::SwitchActor {
            timestamp_us: 0,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        make_action(1, 1, "GA_Mint019_QTE", "GE_Player_Mint_QTE1_Damage", 0, 0),
        GameActionCommand::SwitchActor {
            timestamp_us: 2,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        skill(1000),
        GameActionCommand::SwitchActor {
            timestamp_us: 1010,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(damage(&report, 3), ExactFraction::new(9003, 2).unwrap());
    assert_eq!(
        report["final_actor_states"][0]["resources"]["daffodil.resonance"]["current"],
        0
    );
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "daffodil_resonance_granted")
            .count(),
        1
    );

    // A rejected Support cannot grant; its field entry alone also grants none.
    if let GameActionCommand::InvokeAbility { energy_cost, .. } = &mut req.actions[1] {
        *energy_cost = 121;
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert_eq!(
        report["final_actor_states"][0]["resources"]["daffodil.resonance"]["current"],
        0
    );
}

#[test]
fn daffodil_resonance_requires_native_support_identity_but_not_a_connected_hit() {
    let mut fake = research_request();
    fake.stochastic_crit = None;
    fake.actors[0].character_id = "character:1054".into();
    fake.actions.truncate(1);
    fake.action_metadata.retain(|index, _| *index == 0);
    let metadata = fake.action_metadata.get_mut(&0).unwrap();
    metadata.action_gameplay_tags = Some(vec!["Owner.QTE".into()]);
    metadata.input_action_role = Some(crate::simulation::metadata::InputActionRole::Qte);
    let report = run_game_scenario(&fake, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(
        report["final_actor_states"][0]["resources"]["daffodil.resonance"]["current"],
        0
    );

    let mut missed = request();
    missed.actions = vec![qte(0)];
    missed.targets[0].hp_current = 0;
    let report = run_game_scenario(&missed, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 0);
    assert_eq!(
        report["final_actor_states"][0]["resources"]["daffodil.resonance"]["current"],
        1
    );
}
