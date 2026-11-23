use super::*;
use serde_json::{Value, json};

fn source_request(source: &str, tier: u32) -> GameScenarioRequest {
    let source = source.replace("refine=1", &format!("refine={tier}"));
    let mut req = crate::dsl::compiler::compile_scenario(&source, Default::default())
        .unwrap()
        .request;
    for actor in &mut req.actors {
        actor.static_stats.base_atk = 1000;
        actor.static_stats.atk = Ratio {
            numerator: 1000,
            denominator: 1,
        };
        actor.static_stats.damage_bonus = Ratio::zero();
        actor.static_stats.elemental_bonus = Ratio::zero();
        actor.static_stats.elemental_bonuses.clear();
        actor.static_stats.crit_rate = Ratio::one();
        actor.static_stats.crit_damage_bonus = Ratio {
            numerator: 1,
            denominator: 2,
        };
    }
    req
}

fn hits<'a>(report: &'a Value, effect: &str) -> Vec<&'a Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == effect)
        .collect()
}

#[test]
fn nanally_support_two_cycles_grant_once_per_aoe_and_cap_gold_record_at_each_rank() {
    for (tier, num, den) in [
        (1, 26784, 5),
        (2, 6045, 1),
        (3, 33864, 5),
        (4, 37701, 5),
        (5, 41736, 5),
    ] {
        let req = source_request(
            include_str!("../../../../../examples/gold-record-support-cycles.nte"),
            tier,
        );
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        let support = hits(
            &report,
            crate::characters::nanally::support::ENTRY.damage_effect_id,
        );
        assert_eq!(support.len(), 6);
        for (index, hit) in support.iter().enumerate() {
            assert_eq!(hit["time_us"], if index < 3 { 666_410 } else { 60_666_410 });
            assert_eq!(hit["damage"], json!(format!("{}/{}", num, den)));
            assert_eq!(
                super::support::hit_metadata(&report, hit)["action"]["input_role"],
                "qte"
            );
        }
        let grants: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "nanally_support_packet_energy_grant")
            .collect();
        assert_eq!(grants.len(), 2);
        for grant in grants {
            let fanout = &grant["energy_fanout"];
            assert_eq!(fanout["base"], json!("7/1"));
            assert_eq!(fanout["source_actor_index"], 1);
            let recipients = fanout["recipients"].as_array().unwrap();
            assert_eq!(recipients.len(), 3);
            for receipt in recipients {
                if receipt["recipient_actor_index"] == 1 {
                    assert_eq!(receipt["gross"], 7);
                } else {
                    assert_eq!(receipt["charge_basis"], json!("21/5"));
                }
            }
        }
        let stacks = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "gold_record_stack")
            .count();
        assert_eq!(stacks, 6);
        let factor = [(54, 25), (93, 40), (249, 100), (531, 200), (141, 50)][tier as usize - 1];
        for hit in hits(&report, "GE_Player_Nanally_UltraSkill1_Damage") {
            let base: ExactFraction =
                serde_json::from_value(hit["non_crit_damage"].clone()).unwrap();
            assert_eq!(
                hit["crit_damage"],
                serde_json::to_value(
                    base.mul(ExactFraction::new(factor.0, factor.1).unwrap())
                        .unwrap()
                )
                .unwrap()
            );
        }
        // The three-World-second Support window is retained through every cinematic packet.
        for effect in [
            "GE_Player_Nanally_UltraSkill2_Damage",
            "GE_Player_Nanally_UltraSkill3_Damage",
        ] {
            for hit in hits(&report, effect) {
                let base: ExactFraction =
                    serde_json::from_value(hit["non_crit_damage"].clone()).unwrap();
                assert_eq!(
                    hit["crit_damage"],
                    serde_json::to_value(
                        base.mul(ExactFraction::new(factor.0, factor.1).unwrap())
                            .unwrap()
                    )
                    .unwrap()
                );
            }
        }
        assert!(
            crate::simulation::tests::support::final_buffs(&report)
                .iter()
                .all(|buff| buff["id"] != crate::weapons::gold_record::BUFF_ID)
        );
    }
}

#[test]
fn nanally_cat_support_uses_burst_talent_and_cannot_inherit_support_crit_or_stacks() {
    let req = source_request(
        include_str!("../../../../../examples/nanally-cat-support.nte"),
        1,
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["rejected_action"]
    );
    let cat = hits(&report, crate::characters::nanally::support::CAT_PACKET);
    assert_eq!(cat.len(), 1);
    assert_eq!(cat[0]["time_us"], 5_666_410);
    // Burst10 Cat QTE2.999, source1240 ATK, only ordinary1.5 crit factor.
    assert_eq!(cat[0]["damage"], json!("278907/50"));
    let cat_meta = super::support::hit_metadata(&report, cat[0]);
    assert_eq!(cat_meta["binding"]["kind"], "owner_child");
    assert_eq!(cat_meta["action"]["input_role"], "qte");
    assert_eq!(
        cat_meta["packet"]["native_damage_source_category"],
        "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"
    );
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "gold_record_stack")
            .count(),
        3
    );
}

#[test]
fn nanally_support_talent_is_independent_of_r3_and_failed_primary_energy_falls_back_once() {
    for (talent, coefficient) in [(1, 2000), (8, 3428), (10, 3998)] {
        for resonance in [0, 3] {
            let source = include_str!("../../../../../examples/gold-record-support-cycles.nte")
                .replace(
                    "awakening=0 talent=1,1,1 qte=1 start_energy=100",
                    &format!("awakening={resonance} talent=1,1,1 qte={talent} start_energy=100"),
                );
            let mut req = source_request(&source, 1);
            req.actions.truncate(1);
            req.relative_duration_us = 2_000_000;
            req.targets[0].resistance = Ratio::one();
            let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
            assert_eq!(
                report["execution_completed"], true,
                "{} {}",
                report["halt_diagnostic"], report["rejected_action"]
            );
            let packets = hits(
                &report,
                crate::characters::nanally::support::ENTRY.damage_effect_id,
            );
            assert_eq!(packets.len(), 3);
            assert_eq!(packets[0]["applied_hp_loss"], 0);
            let expected = ExactFraction::new(coefficient * 1240 * 54, 1000 * 25).unwrap();
            assert_eq!(
                packets[1]["damage"],
                serde_json::to_value(expected).unwrap()
            );
            assert_eq!(report["final_actor_states"][0]["energy"], 4);
            assert_eq!(report["final_actor_states"][1]["granted_energy"], 7);
            assert_eq!(
                report["trace"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|event| event["event_kind"] == "gold_record_stack")
                    .count(),
                2
            );
        }
    }
}

#[test]
fn gold_record_support_requires_positive_integer_hp_loss() {
    let mut req = source_request(
        include_str!("../../../../../examples/gold-record-support-cycles.nte"),
        1,
    );
    req.actions.truncate(1);
    req.relative_duration_us = 2_000_000;
    req.actors[1].static_stats.base_atk = 0;
    req.actors[1].static_stats.atk = Ratio {
        numerator: 1,
        denominator: 100,
    };
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let packets = hits(
        &report,
        crate::characters::nanally::support::ENTRY.damage_effect_id,
    );
    assert_eq!(packets.len(), 3);
    assert!(packets.iter().all(|hit| {
        hit["applied_hp_loss"] == 0
            && serde_json::from_value::<ExactFraction>(hit["damage"].clone())
                .unwrap()
                .numerator
                != 0
    }));
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_kind"] == "gold_record_stack"
                || event["event_kind"] == "nanally_support_packet_energy_grant")
    );
}

#[test]
fn nanally_cat_support_respects_summon_expiry_and_owner_defeat_before_delivery() {
    for defeated in [false, true] {
        let source = include_str!("../../../../../examples/nanally-cat-support.nte");
        let source = if defeated {
            source.into()
        } else {
            source
                .replace("awakening=2", "awakening=0")
                .replace("wait_until(5)", "wait_until(10)")
        };
        let mut req = source_request(&source, 1);
        if defeated {
            req.incoming_damage_events
                .push(crate::simulation::GameActorDamageEvent {
                    timestamp_us: 1_917_860,
                    actor_index: 0,
                    delivered_damage: req.actors[0].initial_hp,
                });
            let id = crate::HypothesisId::CombatIncomingDamageFixtureV1;
            req.model_assumptions
                .insert(id.into(), id.description().into());
        }
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert!(hits(&report, crate::characters::nanally::support::CAT_PACKET).is_empty());
    }
}
