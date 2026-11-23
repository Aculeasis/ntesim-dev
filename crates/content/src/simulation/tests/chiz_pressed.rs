use super::*;
use serde_json::{Value, json};

pub(super) fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::chiz_clock::request(targets);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([]).unwrap();
    req.actors[0].talent_levels = Some([8; 3]);
    req.relative_duration_us = 18_000_000;
    for model in [
        crate::characters::chiz::pressed::MODEL,
        crate::characters::chiz::finance::MODEL,
    ] {
        req.model_assumptions
            .insert(model.into(), model.description().into());
    }
    req
}

pub(super) fn press(time: u64) -> GameActionCommand {
    serde_json::from_value(json!({"kind":"invoke_ability","timestamp_us":time,"actor_index":0,
        "ability_id":"GA_Chiichan073_Skill","damage_effect_id":"GE_Player_Chiichan073_Skill1_1Damage",
        "array_index":0,"atk_rate":"1/1","scaling_stat":"atk",
        "energy_cost":0,"recovery_us":1,"cooldown_us":3_000_000,"hit_delay_us":1})).unwrap()
}

pub(super) fn chain(req: &mut GameScenarioRequest, time: u64) {
    for offset in [0, 1_279_881, 2_330_365] {
        req.actions.push(press(time + offset));
    }
}
fn run(req: &GameScenarioRequest) -> Value {
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}

#[test]
fn chiz_three_pressed_stages_keep_native_clocks_channels_and_two_continuing_stock_cycles() {
    for targets in [1, 3] {
        let mut req = request(targets);
        chain(&mut req, 0);
        chain(&mut req, 9_000_000);
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        assert_eq!(report["hits_settled"], 16 * targets);
        let times: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["target_index"] == 0)
            .take(8)
            .map(|hit| hit["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(
            times,
            [
                183_298, 779_558, 1_879_774, 2_246_742, 2_582_538, 3_081_621, 3_275_602, 3_440_044
            ]
        );
        let actor = &report["final_actor_states"][0];
        assert_eq!(actor["energy"], 38);
        assert_eq!(actor["charge_remainder"], json!("503/1250"));
        assert_eq!(actor["cycle"]["current"], 53);
        assert_eq!(actor["cycle"]["carry"], json!("99/500"));
        assert_eq!(actor["resources"]["chiz.skill_stocks"]["current"], 3);
        assert_eq!(
            actor["fractional_resources"]["chiz.grain"]["current"],
            json!("0/1")
        );
        let refills: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "chiz_skill_stock_refill")
            .map(|event| event["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(
            refills,
            [
                3_000_000, 6_000_000, 9_000_000, 12_000_000, 15_000_000, 18_000_000
            ]
        );
    }
}

#[test]
fn chiz_positive_surplus_coin_replaces_full_atk_base_and_is_nonconsuming_noncritical_with_a6_only_on_coin()
 {
    for atk in [1000, 9000] {
        let mut req = request(3);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([5, 6]).unwrap();
        let stats = &mut req.actors[0].static_stats;
        stats.base_atk = atk;
        stats.atk = Ratio {
            numerator: atk,
            denominator: 1,
        };
        stats.damage_bonus = Ratio::zero();
        stats.elemental_bonus = Ratio::zero();
        stats.elemental_bonuses.clear();
        stats.crit_rate = Ratio::one();
        chain(&mut req, 0);
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        let hits = report["hit_receipts"].as_array().unwrap();
        let coins: Vec<_> = hits
            .iter()
            .filter(|hit| hit["damage_base_override"].is_string())
            .collect();
        assert_eq!(coins.len(), 9);
        for coin in coins {
            assert_eq!(coin["damage_base_override"], json!("15000/1"));
            assert_eq!(coin["non_crit_damage"], json!("8625/1"));
            assert_eq!(coin["crit_rate"], json!("0/1"));
            assert_eq!(coin["critical"], false);
            assert!(coin.get("crit_sample").is_none());
            assert_eq!(coin["scaling_rate"], json!("1011/1000"));
        }
        assert_eq!(
            hits[0]["non_crit_damage"],
            serde_json::to_value(ExactFraction::new((163 * atk).into(), 1000).unwrap()).unwrap()
        );
        let grain = &report["final_actor_states"][0]["fractional_resources"]["chiz.grain"];
        assert_eq!(grain["current"], json!("15000/1"));
        assert_eq!(grain["spent"], json!("0/1"));
    }
}

#[test]
fn chiz_spends_initial_three_stocks_and_only_real_refill_releases_the_fourth_and_fifth() {
    let mut req = request(1);
    chain(&mut req, 0);
    req.actions.push(press(4_070_633));
    req.actions.push(press(5_350_514));
    let failed = run(&req);
    assert_eq!(failed["execution_completed"], false);
    assert_eq!(failed["rejected_action"]["command_index"], 4);
    assert_eq!(failed["rejected_action"]["reason"]["kind"], "other");
    assert!(
        failed["rejected_action"]["reason"]["message"]
            .as_str()
            .unwrap()
            .contains("insufficient stocks")
    );
    *req.actions.last_mut().unwrap() = press(6_000_000);
    req.relative_duration_us = 7_500_000;
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    assert_eq!(
        report["final_actor_states"][0]["resources"]["chiz.skill_stocks"]["current"],
        0
    );
    let mut req = request(1);
    req.actions = vec![press(0), press(1_279_880)];
    let report = run(&req);
    assert_eq!(report["rejected_action"]["reason"]["until_us"], 1_279_881);
}

#[test]
fn chiz_pressed_combo_expiry_is_inclusive_and_other_inputs_reset_only_the_stage() {
    for (time, expected) in [
        (3_279_881, "GE_Player_Chiichan073_Skill2_2Damage"),
        (3_279_882, "GE_Player_Chiichan073_Skill1_1Damage"),
    ] {
        let mut req = request(1);
        req.actions = vec![press(0), press(time)];
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        assert_eq!(report["hit_receipts"][2]["damage_effect_id"], expected);
    }
    for switched in [false, true] {
        let mut req = request(1);
        req.actions = vec![press(0)];
        if switched {
            let mut ally = req.actors[0].clone();
            ally.slot = 1;
            ally.character_id = "character:1019".into();
            req.actors.push(ally);
            req.actions.push(GameActionCommand::SwitchActor {
                timestamp_us: 1_279_881,
                target_actor_index: 1,
                cooldown_us: 0,
            });
            req.actions.push(GameActionCommand::SwitchActor {
                timestamp_us: 2_000_000,
                target_actor_index: 0,
                cooldown_us: 0,
            });
            req.actions.push(press(2_000_001));
        } else {
            req.actions.push(super::chiz_clock::normal(1_279_881));
            req.actions.push(press(2_000_001));
        }
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        let second = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|hit| {
                hit["ability_id"] == "GA_Chiichan073_Skill"
                    && hit["time_us"].as_u64().unwrap() > 2_000_000
            })
            .unwrap();
        assert_eq!(
            second["damage_effect_id"],
            "GE_Player_Chiichan073_Skill1_1Damage"
        );
    }
}

#[test]
fn chiz_financial_guard_reads_same_time_interest_then_retains_per_cast_floored_proceeds() {
    let mut req = request(1);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([2]).unwrap();
    req.actions = vec![
        super::chiz_clock::normal(0),
        press(800_000),
        press(3_279_882),
    ];
    req.relative_duration_us = 5_000_000;
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    let coins: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_base_override"].is_string())
        .collect();
    assert_eq!(coins.len(), 2);
    // First capture is exactly1s:450*1.1=495. Later3s balance598.95
    // floors only in Coin1, while the own wallet retains every fraction.
    assert_eq!(coins[0]["damage_base_override"], json!("495/1"));
    assert_eq!(coins[1]["damage_base_override"], json!("598/1"));
    assert_eq!(
        report["final_actor_states"][0]["fractional_resources"]["chiz.grain"]["current"],
        json!("1449459/2000")
    );
}

#[test]
fn chiz_defeat_cancels_unresolved_skill_packets_and_preserves_real_stock_refill() {
    for death in [100_000, 500_000] {
        let mut req = request(1);
        req.actions = vec![press(0)];
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([5]).unwrap();
        let model = crate::HypothesisId::CombatIncomingDamageFixtureV1;
        req.model_assumptions
            .insert(model.into(), model.description().into());
        req.incoming_damage_events
            .push(crate::simulation::GameActorDamageEvent {
                timestamp_us: death,
                actor_index: 0,
                delivered_damage: req.actors[0].initial_hp,
            });
        req.relative_duration_us = 4_000_000;
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], if death == 100_000 { 0 } else { 1 });
        assert_eq!(
            report["final_actor_states"][0]["resources"]["chiz.skill_stocks"]["current"],
            3
        );
        assert!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .any(|event| event["event_kind"] == "chiz_skill_stock_refill"
                    && event["time_us"] == 3_000_000)
        );
    }
}

#[test]
fn chiz_r6_causal_primary_uses_short_target_resistance_status_before_calculation() {
    let mut req = request(1);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actors[0].static_stats.damage_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.targets[0].resistance = Ratio {
        numerator: 1,
        denominator: 5,
    };
    req.actions = vec![press(0)];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    // .326*1000*.5*(1-(.2-.1))=146.7; the very first hit benefits.
    assert_eq!(
        report["hit_receipts"][0]["non_crit_damage"],
        json!("1467/10")
    );
    let mut buffs = ntesim_core::buffs::dispatch::BuffManager::default();
    for time in [100_000, 120_000] {
        crate::characters::chiz::skill::before_hit(&mut crate::runtime_api::HitContext {
            request: &req,
            config: &req.actors[0],
            actor_index: 0,
            ability_id: "GA_Chiichan073_Skill",
            damage_effect_id: "GE_Player_Chiichan073_Skill1_1Damage",
            world_time_us: time,

            target_index: 0,
            buffs: &mut buffs,
        })
        .unwrap();
    }
    buffs.advance_time(169_999);
    assert_eq!(
        crate::simulation::state::target_resistance_reduction(
            &buffs,
            0,
            "ECharacterElementType::CHARACTER_ELEMENT_TYPE_COSMOS"
        )
        .unwrap(),
        ExactFraction::new(1, 10).unwrap()
    );
    buffs.advance_time(170_000);
    assert!(
        crate::simulation::state::target_resistance_reduction(
            &buffs,
            0,
            "ECharacterElementType::CHARACTER_ELEMENT_TYPE_COSMOS"
        )
        .unwrap()
        .is_zero()
    );
}
