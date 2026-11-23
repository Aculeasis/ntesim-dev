use super::*;
use serde_json::json;

fn mixed() -> NativeScalingStat {
    NativeScalingStat::Mixed {
        atk: Ratio {
            numerator: 1,
            denominator: 3,
        },
        def: Ratio {
            numerator: 2,
            denominator: 7,
        },
        hp: Ratio::zero(),
    }
}

fn request() -> GameScenarioRequest {
    let mut req = sample_s1_request();
    let actor = &mut req.actors[0];
    actor.weapon_id.clear();
    actor.gear_procs.clear();
    actor.set_id = None;
    actor.static_stats.base_atk = 100;
    actor.static_stats.atk = Ratio {
        numerator: 200,
        denominator: 1,
    };
    actor.static_stats.base_def = Some(100);
    actor.static_stats.def_percent = Ratio {
        numerator: 1,
        denominator: 2,
    };
    actor.static_stats.def = 150;
    actor.static_stats.elemental_bonus = Ratio::zero();
    actor.static_stats.damage_bonus = Ratio {
        numerator: 1,
        denominator: 4,
    };
    actor.static_stats.crit_rate = Ratio {
        numerator: 1,
        denominator: 2,
    };
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.actions = [0, 1_000_000]
        .into_iter()
        .map(|timestamp_us| GameActionCommand::InvokeAbility {
            timestamp_us,
            actor_index: 0,
            ability_id: "GA_Mint019_Melee".into(),
            damage_effect_id: "GE_Player_Mint_AirAttack1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: mixed(),
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 100,
        })
        .collect();
    req.stochastic_crit = Some(
        serde_json::from_value(json!({
            "hypothesis":"Independent critical roll per mixed target packet",
            "roll_policy":"independent_per_target_hit_v1", "actor_sequences":{"character:1019":54},
            "eligible_effects":{"character:1019":["*"]}
        }))
        .unwrap(),
    );
    req
}

#[test]
fn mixed_packet_sums_exact_terms_before_one_bonus_crit_roll_and_quantization() {
    let req = request();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 2);
    // (200/3 + 150*2/7) * (1 + 1/4) = 2875/21.
    for hit in report["hit_receipts"].as_array().unwrap() {
        assert_eq!(hit["non_crit_damage"], json!("2875/21"));
        assert_eq!(hit["scaling_stat"], json!(mixed()));
        assert!(!hit["crit_sample"].is_null());
    }
    let restored: GameScenarioRequest =
        serde_json::from_slice(&serde_json::to_vec(&req).unwrap()).unwrap();
    assert_eq!(
        report,
        run_game_scenario(&restored, &ExecutionLimits::default()).unwrap()
    );

    let mut tiny = req;
    tiny.actors[0].static_stats.atk = Ratio::one();
    tiny.actors[0].static_stats.base_def = Some(1);
    tiny.actors[0].static_stats.def_percent = Ratio::zero();
    tiny.actors[0].static_stats.damage_bonus = Ratio::zero();
    tiny.actors[0].static_stats.crit_rate = Ratio::zero();
    for action in &mut tiny.actions {
        if let GameActionCommand::InvokeAbility { scaling_stat, .. } = action {
            *scaling_stat = NativeScalingStat::Mixed {
                atk: Ratio {
                    numerator: 1,
                    denominator: 3,
                },
                def: Ratio {
                    numerator: 1,
                    denominator: 3,
                },
                hp: Ratio::zero(),
            };
        }
    }
    let tiny = run_game_scenario(&tiny, &ExecutionLimits::default()).unwrap();
    for hit in tiny["hit_receipts"].as_array().unwrap() {
        assert_eq!(hit["non_crit_damage"], json!("2/3"));
        assert_eq!(hit["rounded_damage"], 1); // Rounding two separate 1/3 terms would give zero.
    }
}

#[test]
fn mixed_scaling_rejects_invalid_weights_and_unsupported_hp_owners() {
    let req = request();
    for stat in [
        NativeScalingStat::Mixed {
            atk: Ratio::one(),
            def: Ratio::zero(),
            hp: Ratio::zero(),
        },
        NativeScalingStat::Mixed {
            atk: Ratio::one(),
            def: Ratio {
                numerator: 1,
                denominator: 0,
            },
            hp: Ratio::zero(),
        },
        NativeScalingStat::Mixed {
            atk: Ratio::one(),
            def: Ratio::zero(),
            hp: Ratio::one(),
        },
    ] {
        let mut changed = req.clone();
        if let GameActionCommand::InvokeAbility { scaling_stat, .. } = &mut changed.actions[0] {
            *scaling_stat = stat;
        }
        assert!(run_game_scenario(&changed, &ExecutionLimits::default()).is_err());
    }
}

#[test]
fn mixed_hp_uses_the_supported_owners_live_max_hp_and_preserves_other_burst_packets() {
    let mut req = request();
    req.stochastic_crit = None;
    req.actors[0].character_id = "character:1039".into();
    req.actors[0].initial_hp = 1000;
    req.actors[0].initial_hp_max = 1000;
    req.actions.truncate(1);
    if let GameActionCommand::InvokeAbility {
        ability_id,
        damage_effect_id,
        scaling_stat,
        ..
    } = &mut req.actions[0]
    {
        *ability_id = "GA_Fadia_UltraSkill".into();
        *damage_effect_id = "GE_Player_Fadia_UltraSkill1_Damage".into();
        *scaling_stat = NativeScalingStat::Mixed {
            atk: Ratio {
                numerator: 1,
                denominator: 2,
            },
            def: Ratio {
                numerator: 1,
                denominator: 4,
            },
            hp: Ratio {
                numerator: 1,
                denominator: 10,
            },
        };
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 6);
    assert_eq!(
        report["hit_receipts"][0]["non_crit_damage"],
        json!("2375/8")
    );
    for hit in &report["hit_receipts"].as_array().unwrap()[1..] {
        assert_eq!(hit["scaling_stat"], "hp");
    }
    req.actors[0].initial_hp = 2000;
    req.actors[0].initial_hp_max = 2000;
    let doubled = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    // Only the HP term grows: +100 * 1.25 = +125, not double the whole packet.
    assert_eq!(
        doubled["hit_receipts"][0]["non_crit_damage"],
        json!("3375/8")
    );
}

#[test]
fn mixed_coefficient_additions_require_an_owner_selected_term() {
    let mut req = request();
    req.stochastic_crit = None;
    req.actors[0].character_id = "character:1042".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([2]).unwrap();
    req.actions.truncate(1);
    if let GameActionCommand::InvokeAbility {
        ability_id,
        damage_effect_id,
        ..
    } = &mut req.actions[0]
    {
        *ability_id = "GA_BlackBird_Skill".into();
        *damage_effect_id = "GE_Player_BlackBird_Skill_Damage".into();
    }
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("mixed packet coefficient changes require an owner-selected scaling term")
    );
}
