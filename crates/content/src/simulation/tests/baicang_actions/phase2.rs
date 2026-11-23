use super::*;

mod binding;
mod cancel;
mod clock;
mod continuing;
mod cycles;
mod dodge;
mod entry_jump;
mod etd;
mod fadia;
mod hold;
mod interrupt;
mod lifecycle;
mod plunge;
mod scorch;

fn phase2_request() -> GameScenarioRequest {
    let mut req = request();
    for id in [
        crate::HypothesisId::BaicangPowerWordsV1,
        crate::HypothesisId::BaicangHoldBranchesV1,
        crate::HypothesisId::BaicangRiposteV1,
        crate::HypothesisId::BaicangPlungeV1,
        crate::HypothesisId::BaicangExecuteAndFatalHitDebtV1,
    ] {
        req.model_assumptions
            .insert(id.as_str().into(), id.description().into());
    }
    req.actors[0].initial_hp = 10000;
    req.actors[0].initial_hp_max = 10000;
    req.relative_duration_us = 40_000_000;
    req
}

fn normal(time: u64) -> GameActionCommand {
    make_action(
        time,
        0,
        "GA_Cang_Melee",
        "GE_Player_Cang_Melee1_Damage",
        0,
        0,
    )
}

fn burst(time: u64) -> GameActionCommand {
    make_action(
        time,
        0,
        "GA_Cang_UltraSkill",
        "GE_Player_Cang_UltraSkill2_Damage",
        120,
        20_000_000,
    )
}

fn run(req: &GameScenarioRequest) -> serde_json::Value {
    let report = run_game_scenario(req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}; {}",
        report["rejected_action"], report["halt_diagnostic"]
    );
    report
}

fn dodge(req: &mut GameScenarioRequest) {
    req.combat_stamina = Some(crate::simulation::CombatStaminaProfile {
        capacity: Ratio {
            numerator: 100,
            denominator: 1,
        },
        initial: Ratio {
            numerator: 100,
            denominator: 1,
        },
        recovery_rate: Ratio::zero(),
        use_efficiency: Ratio::one(),
    });
    req.actions.push(GameActionCommand::PlayerInput {
        timestamp_us: 0,
        actor_index: 0,
        input: PlayerInput::Dash,
        critical_success: true,
    });
}

#[test]
fn words_charge_current_hp_once_per_connected_input() {
    use crate::characters::baicang::words::deduct_hp_cost;
    for (hp, max, expected) in [
        (1, 1000, 0),
        (500, 1000, 8),
        (501, 1000, 15),
        (22204, 22204, 666),
        (u64::MAX, u64::MAX, 553402322211286548),
    ] {
        assert_eq!(deduct_hp_cost(hp, max), expected);
    }
    let mut req = phase2_request();
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    req.actions = vec![
        normal(0),
        normal(1_000_000),
        normal(2_000_000),
        normal(3_300_000),
    ];
    let report = run(&req);
    assert_eq!(report["hits_settled"], 20); // 1+1+1+7, each on two dummies
    assert_eq!(report["final_actor_states"][0]["hp"], 9700);
    // Stage4 carries seven distinct impacts, rather than seven coincident copies.
    let stage4 = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| {
            h["target_index"] == 0 && h["damage_effect_id"] == "GE_Player_Cang_Melee4_Damage"
        })
        .map(|h| h["time_us"].as_u64().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        stage4,
        [
            3433698, 3549436, 3665174, 3780912, 3896650, 4012388, 4128126
        ]
    );
    req.actors[0].static_stats.base_atk = 0;
    req.actors[0].static_stats.atk = Ratio::zero();
    assert_eq!(run(&req)["final_actor_states"][0]["hp"], 10000); // no connected damage
}

#[test]
fn branches_keep_three_slots_and_derive_one_shared_skill() {
    for (packet, explosion, times, recovery) in [
        (
            "GE_Player_Cang_Branch1_Damage",
            "GE_Player_Cang_Branch1_1_Damage",
            [736667, 736667, 736667],
            1_500_000,
        ),
        (
            "GE_Player_Cang_Branch2_Damage",
            "GE_Player_Cang_Branch2_1_Damage",
            [600174, 617079, 633235],
            1_400_000,
        ),
    ] {
        let mut req = phase2_request();
        req.targets.push(req.targets[0].clone());
        req.targets[1].target_index = 1;
        req.actions = vec![
            make_action(0, 0, "GA_Cang_Melee", packet, 0, 0),
            make_action(
                recovery,
                0,
                "GA_Cang_Skill",
                "GE_Player_Cang_Skill_Damage",
                0,
                14_000_000,
            ),
        ];
        let report = run(&req);
        assert_eq!(report["hits_settled"], 10);
        let hits = report["hit_receipts"].as_array().unwrap();
        let explosion_times = hits
            .iter()
            .filter(|h| h["target_index"] == 0 && h["damage_effect_id"] == explosion)
            .map(|h| h["time_us"].as_u64().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(explosion_times, times);
        assert_eq!(
            hits.last().unwrap()["damage_effect_id"],
            "GE_Player_Cang_SkillB_Damage"
        );
        assert_eq!(hits.last().unwrap()["time_us"], recovery + 666652);
        assert_eq!(report["final_actor_states"][0]["hp"], 9700); // one B, not per AoE recipient
    }
}

#[test]
fn dodge_skill_missile_groups_are_single_target() {
    let mut req = phase2_request();
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    req.actions.clear();
    dodge(&mut req);
    req.actions.push(make_action(
        500000,
        0,
        "GA_Cang_Skill",
        "GE_Player_Cang_Skill_Damage",
        0,
        14000000,
    ));
    let report = run(&req);
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 8);
    assert!(hits.iter().all(|h| h["target_index"] == 0));
    assert_eq!(
        hits.iter()
            .map(|h| h["time_us"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [
            716575, 716575, 716575, 716581, 716581, 716683, 716683, 1016575
        ]
    );
    assert_eq!(report["final_actor_states"][0]["hp"], 9700);
}

#[test]
fn riposte_consumes_critical_success_once_and_plunge_is_aoe() {
    let mut req = phase2_request();
    req.actions.clear();
    dodge(&mut req);
    req.actions.extend([normal(500000), normal(1700000)]);
    let report = run(&req);
    assert_eq!(
        report["hit_receipts"][0]["damage_effect_id"],
        "GE_Player_Cang_PerfectEvadeAttack_Damage"
    );
    assert_eq!(
        report["hit_receipts"][1]["damage_effect_id"],
        "GE_Player_Cang_Melee1_Damage"
    );
    let mut req = phase2_request();
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Cang_Melee",
        "GE_Player_Cang_AirAttack_Damage",
        0,
        0,
    )];
    assert_eq!(run(&req)["hits_settled"], 2);
}

#[test]
fn field_marker_activates_retained_words_and_heals_once() {
    let mut req = phase2_request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(vec![4]).unwrap();
    req.actors[0].initial_energy = 120;
    req.actors[0].energy_capacity = 120;
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    req.actions = vec![normal(0), normal(1000000), normal(2000000), burst(3300000)];
    let report = run(&req);
    let hits = report["hit_receipts"].as_array().unwrap();
    let c = hits
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_Cang_WorldC_Damage")
        .collect::<Vec<_>>();
    assert_eq!(c.len(), 1);
    assert_eq!(c[0]["time_us"], 6964744);
    assert_eq!(report["final_actor_states"][0]["hp"], 10000);
    assert_eq!(
        hits.iter()
            .filter(|h| h["damage_effect_id"] == "GE_Player_Cang_UltraSkill_Damage")
            .count(),
        12
    );
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["event_kind"] == "baicang_word_heal")
            .count(),
        1
    );
}

#[test]
fn execute_is_separate_true_remaining_hp_damage() {
    for (boss, hp, executed) in [
        (false, 12000, false),
        (false, 11999, true),
        (true, 7000, false),
        (true, 6999, true),
    ] {
        let mut req = phase2_request();
        req.actors[0].initial_energy = 120;
        req.actors[0].energy_capacity = 120;
        req.actors[0].static_stats.crit_rate = Ratio::zero();
        req.targets[0].hp_max = 100000;
        req.targets[0].hp_current = hp;
        req.targets[0].is_boss = boss;
        req.actions = vec![burst(0)];
        req.absolute_limit_us = 2800000; // Isolate the primary expansion before later Domain ticks.
        // B4 makes expansion 2400, half DEF; post-hit exact threshold uses 2400.
        req.targets[0].hp_current += 400;
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        let hits = report["hit_receipts"].as_array().unwrap();
        assert_eq!(
            hits.iter()
                .any(|h| h["damage_effect_id"] == "ntesim.baicang.execute"),
            executed,
            "boss={boss} hp={hp}"
        );
        if executed {
            assert_eq!(report["final_target_states"][0]["hp_current"], 0);
            assert_eq!(hits[1]["rounded_damage"], hp - 2000);
            assert_eq!(hits[1]["critical"], false);
        }
    }
}

#[test]
fn fatal_interception_does_not_erase_debt() {
    let mut req = phase2_request();
    req.actors[0].initial_energy = 120;
    req.actors[0].energy_capacity = 120;
    req.actors[0].initial_hp = 1000;
    let id = crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID;
    req.model_assumptions
        .insert(id.as_str().into(), id.description().into());
    req.actions = vec![burst(0)];
    req.incoming_damage_events = vec![
        crate::simulation::GameActorDamageEvent {
            timestamp_us: 3000000,
            actor_index: 0,
            delivered_damage: 1200,
        },
        crate::simulation::GameActorDamageEvent {
            timestamp_us: 4000000,
            actor_index: 0,
            delivered_damage: 500,
        },
    ];
    let report = run(&req);
    assert_eq!(report["incoming_damage_receipts"][0]["hp_after"], 1);
    assert_eq!(report["incoming_damage_receipts"][1]["actual_hp_loss"], 0);
    assert_eq!(report["final_actor_states"][0]["hp"], 0);
}
