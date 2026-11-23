use super::*;
use crate::characters::haniel::secondary::{MODEL, PLUNGE, QTE, RIPOSTE};
use serde_json::{Value, json};
fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::haniel_clock::request();
    req.model_assumptions
        .insert(MODEL.into(), MODEL.description().into());
    for target_index in 1..targets {
        let mut t = req.targets[0].clone();
        t.target_index = target_index;
        req.targets.push(t);
    }
    super::blackbird_secondary::stamina(&mut req, 100);
    req
}
fn normal(time: u64) -> GameActionCommand {
    super::haniel_actions::make_action(
        time,
        0,
        "GA_Haniel_Melee",
        "GE_Player_Haniel_Melee1_Damage",
        0,
        0,
    )
}
fn skill(time: u64) -> GameActionCommand {
    super::haniel_actions::make_action(
        time,
        0,
        "GA_Haniel_Skill",
        "GE_Player_Haniel_Skill1_Damage",
        0,
        0,
    )
}
fn run(req: &GameScenarioRequest) -> Value {
    super::edgar_full::run(req)
}
fn jump(time: u64) -> GameActionCommand {
    GameActionCommand::PlayerInput {
        timestamp_us: time,
        actor_index: 0,
        input: PlayerInput::Jump,
        critical_success: false,
    }
}
#[test]
fn haniel_secondary_paid_dodge_consumes_one_response_with_z_and_instant_cannon() {
    for targets in [1, 3] {
        let mut req = request(targets);
        req.actions = vec![
            super::blackbird_secondary::dash(0, true),
            normal(836_769),
            normal(2_110_776),
        ];
        let r = run(&req);
        let hits = r["hit_receipts"].as_array().unwrap();
        assert_eq!(hits.len(), 2 * targets + 1);
        let z: Vec<_> = hits
            .iter()
            .filter(|h| h["damage_effect_id"] == RIPOSTE)
            .collect();
        assert_eq!(z.len(), targets);
        for h in z {
            assert_eq!(h["time_us"], 1_236_769);
            assert_eq!(h["non_crit_damage"], json!("1803/2"));
        }
        let cannon: Vec<_> = hits
            .iter()
            .filter(|h| h["damage_effect_id"] == crate::characters::haniel::cannon::PACKET)
            .collect();
        assert_eq!(cannon.len(), 1);
        assert_eq!(cannon[0]["time_us"], 1_236_769);
        assert_eq!(
            hits.last().unwrap()["damage_effect_id"],
            "GE_Player_Haniel_Melee1_Damage"
        );
        assert_eq!(r["combat_stamina"]["spends"].as_array().unwrap().len(), 1);
        //2.6+.88+2.625=6.105, and4.4+1.46+4.369=10.229 regardless of Z AoE recipients.
        let actor = &r["final_actor_states"][0];
        assert_eq!(actor["energy"], 6);
        assert_eq!(actor["charge_remainder"], json!("21/200"));
        assert_eq!(actor["cycle"]["current"], 10);
        assert_eq!(actor["cycle"]["carry"], json!("229/1000"));
    }
}
#[test]
fn haniel_secondary_recovery_is_per_connected_packet_and_another_action_consumes_response() {
    let mut req = request(3);
    req.actions = vec![
        super::blackbird_secondary::dash(0, true),
        normal(836_769),
        normal(2_110_775),
    ];
    let r = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(r["execution_completed"], false);
    assert_eq!(r["rejected_action"]["reason"]["until_us"], 2_110_776);
    req.actions = vec![
        super::blackbird_secondary::dash(0, true),
        skill(333_611),
        normal(1_642_965),
    ];
    let r = run(&req);
    assert!(
        r["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|h| h["damage_effect_id"] != RIPOSTE)
    );
    super::blackbird_secondary::stamina(&mut req, 14);
    let r = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(r["execution_completed"], false);
    assert!(r["combat_stamina"]["spends"].as_array().unwrap().is_empty());
}
#[test]
fn haniel_secondary_front_back_dodge_skill_and_jump_gates_cancel_only_after_admission() {
    for (back, skill_gate, jump_gate) in [(false, 333_611, 94_234), (true, 494_283, 0)] {
        let mut req = request(1);
        if back {
            req.dash_directions.insert(0, GameDashDirection::Back);
        }
        req.actions = vec![
            super::blackbird_secondary::dash(0, false),
            skill(skill_gate - 1),
        ];
        let r = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(r["rejected_action"]["reason"]["until_us"], skill_gate);
        req.actions[1] = skill(skill_gate);
        run(&req);
        req.actions[1] = jump(jump_gate);
        run(&req);
    }
    let mut req = request(1);
    req.actions = vec![normal(0), jump(208_125), normal(208_125)];
    let r = run(&req);
    assert_eq!(r["hits_settled"], 2);
    assert_eq!(
        r["hit_receipts"][1]["damage_effect_id"],
        "GE_Player_Haniel_Melee1_Damage"
    );
    req.actions = vec![skill(0), jump(1_229_058), normal(1_229_058)];
    let r = run(&req);
    assert_eq!(r["hits_settled"], 3);
}
#[test]
fn haniel_secondary_plunge_uses_explicit_landing_height_and_one_resource_group() {
    for height in [0, 1] {
        let mut req = request(3);
        let mut action = normal(0);
        if let GameActionCommand::InvokeAbility {
            damage_effect_id, ..
        } = &mut action
        {
            *damage_effect_id = PLUNGE.into();
        }
        req.actions = vec![action, normal(775_616)];
        req.plunge_inputs.insert(
            0,
            GamePlungeInput {
                landing_delay_us: 600_000,
                height_bonus: Ratio {
                    numerator: height,
                    denominator: 1,
                },
            },
        );
        let r = run(&req);
        let hits = r["hit_receipts"].as_array().unwrap();
        assert_eq!(hits.len(), 6);
        for hit in hits.iter().take(3) {
            assert_eq!(hit["time_us"], 665_670);
            assert_eq!(
                hit["non_crit_damage"],
                if height == 0 {
                    json!("1405/2")
                } else {
                    json!("1405/1")
                }
            );
        }
        assert_eq!(r["final_actor_states"][0]["energy"], 4);
        assert_eq!(
            r["final_actor_states"][0]["charge_remainder"],
            json!("29/40")
        );
        req.actions[1] = normal(775_615);
        assert_eq!(
            run_game_scenario(&req, &ExecutionLimits::default()).unwrap()["rejected_action"]["reason"]
                ["until_us"],
            775_616
        );
    }
}
#[test]
fn haniel_secondary_parry_requires_paid_dodge_and_has_no_free_damage_or_resource_grant() {
    let mut req = request(1);
    req.actions = vec![super::blackbird_secondary::dash(0, false), normal(836_769)];
    req.parry_events = vec![GameActorParryEvent::At {
        timestamp_us: 100_000,
        actor_index: 0,
    }];
    let r = run(&req);
    assert_eq!(r["hits_settled"], 2);
    assert_eq!(r["parry_receipts"].as_array().unwrap().len(), 1);
    assert_eq!(
        r["final_actor_states"][0]["charge_remainder"],
        json!("12/25")
    );
    req.actions.clear();
    let r = run_game_scenario(&req, &ExecutionLimits::default());
    assert!(r.is_err() || r.unwrap()["execution_completed"] == false);
    req.actions = vec![super::blackbird_secondary::dash(0, false), normal(846_769)];
    req.parry_events[0] = GameActorParryEvent::At {
        timestamp_us: 826_770,
        actor_index: 0,
    };
    let early = run_game_scenario(&req, &ExecutionLimits::default());
    assert!(early.is_err() || early.unwrap()["execution_completed"] == false);
    req.actions[1] = normal(846_770);
    run(&req);
}
#[test]
fn haniel_secondary_support_is_engine_qualified_and_ordinary_entry_has_no_attack_recovery() {
    for cycle in [0, 100] {
        let mut req = request(1);
        let mut old = req.actors[0].clone();
        old.slot = 1;
        old.character_id = "character:1025".into();
        old.initial_cycle = cycle;
        req.actors.push(old);
        req.active_actor_index = 1;
        req.targets[0].original_atk = Some(1000);
        req.actions = vec![GameActionCommand::SwitchActor {
            timestamp_us: 0,
            target_actor_index: 0,
            cooldown_us: 0,
        }];
        let r = run(&req);
        if cycle == 100 {
            assert_eq!(r["hit_receipts"][0]["damage_effect_id"], QTE);
            assert_eq!(r["hit_receipts"][0]["time_us"], 712_638);
            assert_eq!(
                r["hit_receipts"][0]["non_crit_damage"],
                // Qualified Lakshana -> Psyche Stain adds20% to3428*1000*.5.
                json!("10284/5")
            );
            assert_eq!(r["final_actor_states"][0]["energy"], 5);
            req.actions.push(normal(1_387_903));
            assert_eq!(
                run_game_scenario(&req, &ExecutionLimits::default()).unwrap()["rejected_action"]["reason"]
                    ["until_us"],
                1_387_904
            );
            *req.actions.last_mut().unwrap() = normal(1_387_904);
            run(&req);
        } else {
            assert_eq!(r["hits_settled"], 0);
            req.actions.push(normal(0));
            let manual = run(&req);
            assert_eq!(
                manual["hit_receipts"][0]["damage_effect_id"],
                "GE_Player_Haniel_Melee1_Damage"
            );
        }
    }
}

#[test]
fn haniel_secondary_riposte_skill_gate_includes_z_and_cannon_pause() {
    let mut req = request(3);
    req.actions = vec![
        super::blackbird_secondary::dash(0, true),
        normal(836_769),
        skill(1_667_853),
    ];
    let early = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(early["rejected_action"]["reason"]["until_us"], 1_667_854);
    req.actions[2] = skill(1_667_854);
    run(&req);
}

#[test]
fn haniel_secondary_field_exit_clears_response_and_reentry_uses_normal_one() {
    let mut req = request(1);
    let mut peer = req.actors[0].clone();
    peer.slot = 1;
    peer.character_id = "character:1025".into();
    req.actors.push(peer);
    req.actions = vec![
        super::blackbird_secondary::dash(0, true),
        GameActionCommand::SwitchActor {
            timestamp_us: 836_769,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 3_000_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        normal(3_896_381),
    ];
    let r = run(&req);
    let owned: Vec<_> = r["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["actor_index"] == 0)
        .collect();
    assert_eq!(owned.len(), 1);
    assert_eq!(
        owned[0]["damage_effect_id"],
        "GE_Player_Haniel_Melee1_Damage"
    );
    assert_eq!(owned[0]["command_index"], 3);
}

#[test]
fn haniel_secondary_bond_requires_breakthrough_four_without_disabling_ensemble() {
    let mut req = super::haniel_cannon::plot_two(1);
    req.actors[0].character_level = 20;
    req.actors[0].purchased_qte_level = Some(1);
    req.actors[0].character_max_level = Some(20);
    req.actors[0].talent_levels = Some([1; 3]);
    let r = run(&req);
    assert_eq!(
        r["final_actor_states"][0]["resources"]["haniel.plot_armor"]["current"],
        0
    );
    assert_eq!(
        r["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| h["damage_effect_id"] == "GE_Player_Haniel_Skill2_Damage")
            .count(),
        2
    );
}
