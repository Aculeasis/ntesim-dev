use super::*;
use crate::simulation::GamePlungeInput;

fn scenario(height: Ratio) -> GameScenarioRequest {
    let mut req = phase2_request();
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    req.actions = [0, 1_000_000]
        .into_iter()
        .map(|time| {
            make_action(
                time,
                0,
                "GA_Cang_Melee",
                "GE_Player_Cang_AirAttack_Damage",
                0,
                0,
            )
        })
        .collect();
    for index in 0..2 {
        req.plunge_inputs.insert(
            index,
            GamePlungeInput {
                landing_delay_us: 800_000,
                height_bonus: height,
            },
        );
    }
    req
}

#[test]
fn height_scales_each_native_packet_once_without_multiplying_resources_or_resetting_state() {
    for (height, numerator, denominator) in [
        (Ratio::zero(), 1254, 5),
        (
            Ratio {
                numerator: 1,
                denominator: 2,
            },
            1881,
            5,
        ),
        (Ratio::one(), 2508, 5),
    ] {
        let report = run(&scenario(height));
        let hits = report["hit_receipts"].as_array().unwrap();
        assert_eq!(hits.len(), 4);
        for (index, hit) in hits.iter().enumerate() {
            assert_eq!(hit["time_us"], 849_876 + (index as u64 / 2) * 1_000_000);
            // Live1000+B4 200, coefficient .418*(1+h), halfDEF.
            assert_eq!(
                hit["non_crit_damage"],
                json!(format!("{}/{}", numerator, denominator))
            );
            assert_eq!(hit["damage_effect_id"], "GE_Player_Cang_AirAttack_Damage");
        }
        let actor = &report["final_actor_states"][0];
        assert_eq!(
            actor["hp"], 10000,
            "plunge is not a connected Normal Word input"
        );
        assert_eq!(actor["granted_energy"], 3);
        assert_eq!(actor["charge_remainder"], json!("4/5"));
        assert_eq!(actor["cycle"]["granted"], 6);
        assert_eq!(actor["cycle"]["carry"], json!("2/5"));
    }
}

#[test]
fn plunge_requires_landing_recovery_and_keeps_an_early_followup_failed() {
    let mut req = scenario(Ratio::one());
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut req.actions[1] {
        *timestamp_us = 998_547;
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert_eq!(report["hits_settled"], 2);
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut req.actions[1] {
        *timestamp_us = 998_548;
    }
    assert_eq!(run(&req)["hits_settled"], 4);
}

#[test]
fn input_profiles_reject_invalid_ratios_deadlines_indices_and_owner_bindings() {
    let req = scenario(Ratio::one());
    for input in [
        GamePlungeInput {
            landing_delay_us: 0,
            height_bonus: Ratio::zero(),
        },
        GamePlungeInput {
            landing_delay_us: 1,
            height_bonus: Ratio {
                numerator: 0,
                denominator: 0,
            },
        },
        GamePlungeInput {
            landing_delay_us: 1,
            height_bonus: Ratio {
                numerator: 3,
                denominator: 2,
            },
        },
        GamePlungeInput {
            landing_delay_us: u64::MAX,
            height_bonus: Ratio::zero(),
        },
    ] {
        let mut invalid = req.clone();
        invalid.plunge_inputs.insert(0, input);
        assert!(run_game_scenario(&invalid, &ExecutionLimits::default()).is_err());
    }
    let mut invalid = req.clone();
    invalid.plunge_inputs.insert(2, invalid.plunge_inputs[&0]);
    assert!(run_game_scenario(&invalid, &ExecutionLimits::default()).is_err());
    let mut invalid = req;
    invalid.actions[0] = normal(0);
    assert!(run_game_scenario(&invalid, &ExecutionLimits::default()).is_err());
}

#[test]
fn explicit_plunge_preserves_its_profile_and_discards_unused_critical_riposte() {
    let mut req = scenario(Ratio::one());
    let plunge = req.actions[0].clone();
    let input = req.plunge_inputs[&0];
    req.actions.clear();
    req.plunge_inputs.clear();
    dodge(&mut req);
    req.actions.push(plunge);
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut req.actions[1] {
        *timestamp_us = 500_000;
    }
    req.plunge_inputs.insert(1, input);
    req.actions.push(normal(1_500_000));
    let report = run(&req);
    assert_eq!(
        report["hit_receipts"][0]["damage_effect_id"],
        "GE_Player_Cang_AirAttack_Damage"
    );
    assert_eq!(report["hit_receipts"][0]["time_us"], 1_349_876);
    assert_eq!(
        report["hit_receipts"][2]["damage_effect_id"],
        "GE_Player_Cang_Melee1_Damage"
    );
}
