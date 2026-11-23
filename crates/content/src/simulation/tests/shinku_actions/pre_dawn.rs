use super::*;

fn shinku_burst_schedule() -> [(&'static str, u128, usize); 5] {
    // Lv1 burst rates from the existing burst oracle test, in delivery order.
    [
        ("GE_Player_Shinku_UltraSkill1_Damage", 185, 1),
        ("GE_Player_Shinku_UltraSkill2_Damage", 1200, 1),
        ("GE_Player_Shinku_UltraSkill3_Damage", 550, 5),
        ("GE_Player_Shinku_UltraSkill4_Damage", 1440, 1),
        ("GE_Player_Shinku_UltraSkill5_Damage", 425, 1),
    ]
}

fn shinku_reduced(value: serde_json::Value) -> serde_json::Value {
    // Hand-derived: an active -10% Cosmos RES at fixture resistance 0 is
    // F_res = 12/11 under StandardRational (1 + 0.1/1.1).
    let base: ExactFraction = serde_json::from_value(value).unwrap();
    fraction_json(base.numerator * 12, base.denominator * 11)
}

#[test]
fn shinku_a2_gate_applies_only_from_prefix_two() {
    // Awakening gate: prefix(1) applies no status, prefix(2) applies it.
    // The first same-time Ultimate hit is always unreduced; it applies the status.
    for (awakening, reduced) in [(1u32, false), (2u32, true)] {
        let mut req = request();
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actors[0].talent_levels = Some([1, 1, 1]);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Shinku_UltraSkill",
            "GE_Player_Shinku_UltraSkill1_Damage",
            100,
            0,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 9);
        let receipts = report["hit_receipts"].as_array().unwrap();
        let mut index = 0;
        for (packet_id, rate_milli, multiplicity) in shinku_burst_schedule() {
            for _ in 0..multiplicity {
                let base = shinku_rage_packet_damage(rate_milli, awakening);
                let expected = if reduced && index > 0 {
                    shinku_reduced(base)
                } else {
                    base
                };
                assert_eq!(
                    receipts[index]["damage_effect_id"], packet_id,
                    "awakening {awakening} hit {index}"
                );
                assert_eq!(
                    receipts[index]["non_crit_damage"], expected,
                    "awakening {awakening} hit {index}"
                );
                index += 1;
            }
        }
        let assumption = &report["resolved_model_assumptions"]
            [crate::characters::shinku::skill::PRE_DAWN_REFRESH_HYPOTHESIS_ID.as_str()];
        if reduced {
            assert_eq!(
                assumption.as_str().unwrap(),
                crate::characters::shinku::skill::PRE_DAWN_REFRESH_HYPOTHESIS_ID.description(),
                "awakening {awakening}"
            );
        } else {
            assert!(assumption.is_null(), "awakening {awakening}");
        }
    }
}

#[test]
fn shinku_a2_cosmos_reduction_matches_frozen_resistance_oracle() {
    // Fixture resistance 1/5. Frozen base: R = 0.2 -> F_res = 4/5.
    // With the -10% Cosmos shred: R = 0.1 -> F_res = 9/10, i.e. base * 9/8.
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(2);
    req.actors[0].talent_levels = Some([1, 1, 1]);
    req.targets[0].resistance = Ratio {
        numerator: 1,
        denominator: 5,
    };
    req.actions = vec![make_action(
        0,
        0,
        "GA_Shinku_UltraSkill",
        "GE_Player_Shinku_UltraSkill1_Damage",
        100,
        0,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 9);
    let receipts = report["hit_receipts"].as_array().unwrap();
    let mut index = 0;
    for (packet_id, rate_milli, multiplicity) in shinku_burst_schedule() {
        for _ in 0..multiplicity {
            let base = shinku_rage_packet_damage(rate_milli, 2);
            // shinku_base_packet_damage assumes F_res = 1; scale to the
            // resistance-1/5 factors: 4/5 unreduced, 9/10 once shredded.
            let (factor_num, factor_den) = if index == 0 { (4, 5) } else { (9, 10) };
            let base_exact: ExactFraction = serde_json::from_value(base.clone()).unwrap();
            let expected = fraction_json(
                base_exact.numerator * factor_num,
                base_exact.denominator * factor_den,
            );
            assert_eq!(
                receipts[index]["damage_effect_id"], packet_id,
                "hit {index}"
            );
            assert_eq!(receipts[index]["non_crit_damage"], expected, "hit {index}");
            index += 1;
        }
    }
}

#[test]
fn shinku_a2_pre_dawn_expires_after_twenty_seconds() {
    // Status window: [100, 22_000_000). Buff expiry is now_us >= expires_at_us,
    // so a Cosmos hit at exactly 22_000_000 sees the status expired.
    // Lv1 Skill rates: 173 / 231 / 796.
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(2);
    req.actors[0].talent_levels = Some([1, 1, 1]);
    req.relative_duration_us = 40_000_000;
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Shinku_UltraSkill",
            "GE_Player_Shinku_UltraSkill1_Damage",
            100,
            0,
        ),
        make_action(
            21_999_900,
            0,
            "GA_Shinku_Skill",
            "GE_Player_Shinku_Skill1_1_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 12);
    let receipts = report["hit_receipts"].as_array().unwrap();
    let mut index = 0;
    for (packet_id, rate_milli, multiplicity) in shinku_burst_schedule() {
        for _ in 0..multiplicity {
            let base = shinku_rage_packet_damage(rate_milli, 2);
            let expected = if index == 0 {
                base
            } else {
                shinku_reduced(base)
            };
            assert_eq!(receipts[index]["time_us"], 100, "ult hit {index}");
            assert_eq!(
                receipts[index]["damage_effect_id"], packet_id,
                "ult hit {index}"
            );
            assert_eq!(
                receipts[index]["non_crit_damage"], expected,
                "ult hit {index}"
            );
            index += 1;
        }
    }
    let skill_packets = [
        ("GE_Player_Shinku_Skill1_1_Damage", 173u128),
        ("GE_Player_Shinku_Skill1_2_Damage", 231u128),
        ("GE_Player_Shinku_Skill1_3_Damage", 796u128),
    ];
    for (i, (packet_id, rate_milli)) in skill_packets.into_iter().enumerate() {
        let receipt = &receipts[9 + i];
        assert_eq!(receipt["time_us"], 22_000_000, "skill hit {i}");
        assert_eq!(receipt["damage_effect_id"], packet_id, "skill hit {i}");
        assert_eq!(
            receipt["non_crit_damage"],
            shinku_base_packet_damage(rate_milli, 2),
            "skill hit {i}: expired status must leave base damage"
        );
    }
}

#[test]
fn shinku_a2_pre_dawn_refreshes_on_later_ultimate_hits() {
    // Window 1: [100, 20_000_100). The second Ultimate hits at 19_000_100 and
    // re-anchors the window to [19_000_100, 39_999_900) (RefreshPolicy::Reset).
    // A Cosmos hit at 20_000_000 is still reduced; one at exactly 39_999_900 is expired.
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(2);
    req.actors[0].talent_levels = Some([1, 1, 1]);
    req.actors[0].initial_energy = 200;
    req.actors[0].energy_capacity = 200;
    req.relative_duration_us = 60_000_000;
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Shinku_UltraSkill",
            "GE_Player_Shinku_UltraSkill1_Damage",
            100,
            0,
        ),
        make_action(
            18_999_900,
            0,
            "GA_Shinku_UltraSkill",
            "GE_Player_Shinku_UltraSkill1_Damage",
            100,
            0,
        ),
        make_action(
            19_999_900,
            0,
            "GA_Shinku_Skill",
            "GE_Player_Shinku_Skill1_1_Damage",
            0,
            0,
        ),
        make_action(
            39_999_800,
            0,
            "GA_Shinku_Skill",
            "GE_Player_Shinku_Skill1_1_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 24);
    let receipts = report["hit_receipts"].as_array().unwrap();
    let mut index = 0;
    for (cast, cast_time_us) in [(0, 100u64), (1, 19_000_000u64)] {
        for (packet_id, rate_milli, multiplicity) in shinku_burst_schedule() {
            for _ in 0..multiplicity {
                let base = shinku_rage_packet_damage(rate_milli, 2);
                // First cast: first hit applies the status. Second cast lands
                // inside window 1, so all its hits are reduced and re-anchor it.
                let expected = if cast == 0 && index == 0 {
                    base
                } else {
                    shinku_reduced(base)
                };
                assert_eq!(
                    receipts[index]["time_us"], cast_time_us,
                    "cast {cast} hit {index}"
                );
                assert_eq!(
                    receipts[index]["damage_effect_id"], packet_id,
                    "cast {cast} hit {index}"
                );
                assert_eq!(
                    receipts[index]["non_crit_damage"], expected,
                    "cast {cast} hit {index}"
                );
                index += 1;
            }
        }
    }
    let skill_packets = [
        ("GE_Player_Shinku_Skill1_1_Damage", 173u128),
        ("GE_Player_Shinku_Skill1_2_Damage", 231u128),
        ("GE_Player_Shinku_Skill1_3_Damage", 796u128),
    ];
    for (i, (packet_id, rate_milli)) in skill_packets.into_iter().enumerate() {
        let rage_base = shinku_rage_packet_damage(rate_milli, 2);
        let base = shinku_base_packet_damage(rate_milli, 2);
        let still_reduced = &receipts[18 + i];
        assert_eq!(still_reduced["time_us"], 20_000_000, "skill hit {i}");
        assert_eq!(
            still_reduced["damage_effect_id"], packet_id,
            "skill hit {i}"
        );
        assert_eq!(
            still_reduced["non_crit_damage"],
            shinku_reduced(rage_base),
            "skill hit {i}: inside refreshed window, must be reduced"
        );
        let expired = &receipts[21 + i];
        assert_eq!(expired["time_us"], 39_999_900, "late skill hit {i}");
        assert_eq!(expired["damage_effect_id"], packet_id, "late skill hit {i}");
        assert_eq!(
            expired["non_crit_damage"], base,
            "late skill hit {i}: at exact expiry, must be base damage"
        );
    }
}

#[test]
fn shinku_a2_pre_dawn_applies_only_to_cosmos_hits() {
    // Mint (Nature element) hits the same target inside the 20s window:
    // the Cosmos-only shred must not touch Nature packets.
    // Mint Lv1 melee: Melee1 = 104 milli -> 52/1, Melee2 = 231 milli -> 231/2.
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(2);
    req.actors[0].talent_levels = Some([1, 1, 1]);
    let mut ally = req.actors[0].clone();
    ally.character_id = "character:1019".into();
    ally.slot = 1;
    ally.awakening = ntesim_data::AwakeningSelection::prefix(0);
    req.actors.push(ally);
    req.active_actor_index = 0;
    req.relative_duration_us = 40_000_000;
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Shinku_UltraSkill",
            "GE_Player_Shinku_UltraSkill1_Damage",
            100,
            0,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 100_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        make_action(
            200_000,
            1,
            "GA_Mint019_Melee",
            "GE_Player_Mint_Melee1_Damage_New",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 11);
    let receipts = report["hit_receipts"].as_array().unwrap();
    let mut index = 0;
    for (packet_id, rate_milli, multiplicity) in shinku_burst_schedule() {
        for _ in 0..multiplicity {
            let base = shinku_rage_packet_damage(rate_milli, 2);
            let expected = if index == 0 {
                base
            } else {
                shinku_reduced(base)
            };
            assert_eq!(
                receipts[index]["damage_effect_id"], packet_id,
                "ult hit {index}"
            );
            assert_eq!(
                receipts[index]["non_crit_damage"], expected,
                "ult hit {index}"
            );
            index += 1;
        }
    }
    for (i, (packet_id, expected)) in [
        ("GE_Player_Mint_Melee1_Damage_New", json!("52/1")),
        ("GE_Player_Mint_Melee2_Damage_New", json!("231/2")),
    ]
    .into_iter()
    .enumerate()
    {
        let receipt = &receipts[9 + i];
        assert_eq!(receipt["time_us"], 200_100, "mint hit {i}");
        assert_eq!(receipt["actor_index"], 1, "mint hit {i}");
        assert_eq!(receipt["damage_effect_id"], packet_id, "mint hit {i}");
        assert_eq!(
            receipt["non_crit_damage"], expected,
            "mint hit {i}: Nature packet must ignore the Cosmos shred"
        );
    }
}
