use super::*;
use crate::characters::haniel::clock;
use crate::simulation::tests::support::formula_damage_total;
use serde_json::{Value, json};

pub(super) fn request() -> GameScenarioRequest {
    let mut req = super::haniel_actions::request();
    req.actions.clear();
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actors[0].talent_levels = Some([8; 3]);
    req.actors[0].initial_energy = 0;
    req.actors[0].energy_capacity = 120;
    req.relative_duration_us = 30_000_000;
    req.model_assumptions
        .insert(clock::MODEL.into(), clock::MODEL.description().into());
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
fn burst(time: u64) -> GameActionCommand {
    super::haniel_actions::make_action(
        time,
        0,
        "GA_Haniel_UltraSkill",
        "GE_Player_Haniel_Ultraskill1_Damage",
        120,
        0,
    )
}
fn run(req: &GameScenarioRequest) -> Value {
    super::edgar_full::run(req)
}
fn buff(r: &Value, id: &str) -> Option<Value> {
    crate::simulation::tests::support::final_buffs(r)
        .into_iter()
        .find(|b| b["id"] == id)
}

#[test]
fn haniel_clock_exact_normal_train_aoe_resources_and_no_implicit_wait() {
    for targets in [1, 2] {
        let mut req = request();
        if targets == 2 {
            let mut t = req.targets[0].clone();
            t.target_index = 1;
            req.targets.push(t);
        }
        req.actions = [0, 887510, 1609777, 2027383, 2683151, 3633183]
            .into_iter()
            .map(normal)
            .collect();
        let r = run(&req);
        assert_eq!(r["hits_settled"], 10 * targets);
        assert_eq!(
            serde_json::json!(formula_damage_total(&r)),
            json!(format!("{}/{}", 2395 * targets, 1))
        );
        let times: Vec<_> = r["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| h["target_index"] == 0)
            .map(|h| h["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(
            times,
            [
                179633, 1115402, 1317756, 1728808, 2360020, 2401299, 2439471, 2483140, 3291784,
                3812816
            ]
        );
        let actor = &r["final_actor_states"][0];
        assert_eq!(actor["energy"], 17);
        assert_eq!(actor["charge_remainder"], json!("527/1000"));
        assert_eq!(actor["cycle"]["current"], 29);
        assert_eq!(actor["cycle"]["carry"], json!("21/125"));
        req.actions[1] = normal(887509);
        let early = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(early["execution_completed"], false);
        assert_eq!(early["rejected_action"]["reason"]["until_us"], 887510);
        assert_eq!(early["hits_settled"], targets);
    }
}

#[test]
fn haniel_clock_normal_idle_and_other_action_reset_without_a_two_second_guess() {
    let mut req = request();
    req.actions = vec![
        normal(0),
        normal(887_511),
        skill(2_000_000),
        normal(3_309_354),
    ];
    let r = run(&req);
    for index in [0, 1, 4] {
        assert_eq!(
            r["hit_receipts"][index]["damage_effect_id"],
            "GE_Player_Haniel_Melee1_Damage"
        );
    }
    assert_eq!(r["hits_settled"], 5);
}

#[test]
fn haniel_clock_skill_impacts_precede_aura_and_ace_waits_for_its_actual_notify() {
    let mut req = request();
    req.actors[0].initial_energy = 120;
    req.actions = vec![skill(0), burst(2_000_000)];
    req.relative_duration_us = 10_000_000;
    let r = run(&req);
    let hits = r["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 3);
    assert_eq!(hits[0]["time_us"], 345126);
    assert_eq!(hits[1]["time_us"], 719512);
    for hit in &hits[..2] {
        assert_eq!(hit["non_crit_damage"], json!("857/1"));
    }
    assert_eq!(hits[2]["time_us"], 5843514);
    //1000+170 Hootie+170 Ace, then5.141ATK and half DEF.
    assert_eq!(hits[2]["non_crit_damage"], json!("344447/100"));
    assert_eq!(
        buff(&r, "haniel.hootie_team_atk").unwrap()["duration"]["expires_at_us"],
        18719512
    );
    assert_eq!(
        buff(&r, "haniel.paranormal_ace_team_atk").unwrap()["duration"]["expires_at_us"],
        12000000
    );
    for (horizon, active) in [(4_869_165, false), (4_869_166, true)] {
        req.absolute_limit_us = horizon;
        let partial = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            buff(&partial, "haniel.paranormal_ace_team_atk").is_some(),
            active
        );
    }
}

#[test]
fn haniel_clock_skill_interrupt_cooldown_marker_and_emitted_spawn_are_causal() {
    for a6 in [false, true] {
        for (time, count, spawned, cooldown) in [
            (300000, 0, false, 3300000),
            (600000, 1, false, 3600000),
            (684620, 1, false, if a6 { 16684620 } else { 20684620 }),
            (700000, 1, false, if a6 { 16684620 } else { 20684620 }),
            (719512, 2, true, if a6 { 16684620 } else { 20684620 }),
        ] {
            let mut req = request();
            req.relative_duration_us = 2_000_000;
            if a6 {
                req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([6]).unwrap();
            }
            req.actions = vec![skill(0)];
            req.interrupt_events = vec![GameActorInterruptEvent {
                timestamp_us: time,
                actor_index: 0,
                armor_damage: None,
            }];
            let r = run(&req);
            assert_eq!(r["hits_settled"], count);
            assert_eq!(buff(&r, "haniel.hootie_team_atk").is_some(), spawned);
            assert_eq!(
                r["interrupt_receipts"][0]["skill_cooldown_after_us"],
                cooldown
            );
        }
    }
}

#[test]
fn haniel_clock_burst_interrupt_before_and_after_ace_keeps_cost_and_existing_buffs() {
    for (time, ace) in [(2_000_000, false), (3_000_000, true)] {
        let mut req = request();
        req.actors[0].initial_energy = 120;
        req.actions = vec![burst(0)];
        req.relative_duration_us = 4_000_000;
        req.interrupt_events = vec![GameActorInterruptEvent {
            timestamp_us: time,
            actor_index: 0,
            armor_damage: None,
        }];
        let r = run(&req);
        assert_eq!(r["hits_settled"], 0);
        assert_eq!(r["final_actor_states"][0]["spent_energy"], 120);
        assert_eq!(r["final_actor_states"][0]["energy"], 0);
        assert_eq!(buff(&r, "haniel.paranormal_ace_team_atk").is_some(), ace);
    }
}

#[test]
fn haniel_clock_nominal_skill_cooldown_is_not_caller_timing() {
    for (awakening, until) in [(0, 20684620), (6, 16684620)] {
        let mut req = request();
        if awakening == 6 {
            req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([6]).unwrap();
        }
        req.actions = vec![skill(0), skill(until - 1)];
        let r = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(r["execution_completed"], false);
        assert_eq!(r["rejected_action"]["reason"]["until_us"], until);
        req.actions[1] = skill(until);
        assert_eq!(run(&req)["hits_settled"], 4);
    }
}

#[test]
fn haniel_clock_field_exit_preserves_owned_ensemble_coordinate_and_live_aura() {
    let mut req = request();
    req.relative_duration_us = 5_000_000;
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    req.actors.push(ally);
    req.actions = vec![
        skill(0),
        GameActionCommand::SwitchActor {
            timestamp_us: 2_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    for time in [2_100_000, 2_600_000, 3_100_000, 3_600_000] {
        req.actions.push(super::haniel_actions::make_action(
            time,
            1,
            "GA_Mint019_Melee",
            "GE_Player_Mint_Melee1_Damage_New",
            0,
            0,
        ));
    }
    let r = run(&req);
    let ensemble: Vec<_> = r["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_Haniel_Skill2_Damage")
        .collect();
    assert_eq!(ensemble.len(), 1);
    assert_eq!(ensemble[0]["actor_index"], 0);
    assert_eq!(ensemble[0]["command_index"], 0);
    assert_eq!(ensemble[0]["time_us"], 3_600_100);
    assert_eq!(ensemble[0]["non_crit_damage"], json!("40131/200"));
    assert_eq!(
        r["final_actor_states"][0]["resources"]["haniel.plot_armor"]["current"],
        1
    );
}

#[test]
fn haniel_clock_skill_projectile_fallback_spawns_after_the_primary_killed_the_only_dummy() {
    let mut req = request();
    req.relative_duration_us = 2_000_000;
    req.targets[0].hp_current = 1;
    req.actions = vec![skill(0)];
    let r = run(&req);
    assert_eq!(r["hits_settled"], 1);
    assert_eq!(r["final_target_states"][0]["hp_current"], 0);
    assert!(buff(&r, "haniel.hootie_team_atk").is_some());
}
