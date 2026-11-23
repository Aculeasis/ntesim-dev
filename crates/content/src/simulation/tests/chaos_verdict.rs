use super::chaos_actions::make_action;
use super::*;
use crate::simulation::{GameActorDamageEvent, GameActorInterruptEvent, GameHoldInput};
use serde_json::{Value, json};

fn model(req: &mut GameScenarioRequest, id: crate::HypothesisId) {
    req.model_assumptions
        .insert(id.into(), id.description().into());
}
pub(super) fn request(targets: usize, form: bool) -> GameScenarioRequest {
    let mut req = if form {
        super::chaos_form::request(targets)
    } else {
        super::chaos_clock::request(targets)
    };
    model(&mut req, crate::HypothesisId::ChaosNeutralVerdictV1);
    req.relative_duration_us = 25_000_000;
    req
}
fn normal(time: u64) -> GameActionCommand {
    make_action(
        time,
        0,
        "GA_Chaos071_Melee",
        "GE_Player_Chaos_Melee1_Damage",
        0,
        0,
    )
}
fn hold(req: &mut GameScenarioRequest, time: u64) {
    req.hold_inputs.insert(
        req.actions.len(),
        GameHoldInput {
            reserved_stocks: None,
            duration_us: 1_000_000,
            moving: false,
        },
    );
    req.actions.push(normal(time));
}
fn run(req: &GameScenarioRequest) -> Value {
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}
fn crime(report: &Value) -> &Value {
    &report["final_actor_states"][0]["fractional_resources"]["chaos.crime"]
}
fn branch_hits(report: &Value) -> Vec<&Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| {
            hit["damage_effect_id"]
                .as_str()
                .unwrap()
                .contains("_Branch")
        })
        .collect()
}

#[test]
fn chaos_plain_verdict_pays_neutral_hold_and_preserves_partial_crime_and_native_channels() {
    let mut req = request(3, false);
    let mut time = 0;
    super::chaos_crime::chain(&mut req, &mut time);
    hold(&mut req, 6_000_000);
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = branch_hits(&report);
    assert_eq!(hits.len(), 6);
    assert_eq!(
        hits.iter()
            .filter(|hit| hit["target_index"] == 0)
            .map(|hit| hit["time_us"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [7_166_691, 7_883_658]
    );
    assert_eq!(hits[0]["scaling_rate"], json!("23/40"));
    assert_eq!(crime(&report)["current"], json!("382/1"));
    assert_eq!(crime(&report)["spent"], json!("0/1"));
    assert_eq!(report["final_actor_states"][0]["energy"], 24);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        json!("1/500")
    );
    assert_eq!(report["final_actor_states"][0]["cycle"]["current"], 39);
    assert_eq!(
        report["final_actor_states"][0]["cycle"]["carry"],
        json!("899/1000")
    );
    req.actions.push(normal(7_898_333));
    assert_eq!(
        run(&req)["rejected_action"]["reason"]["until_us"],
        7_898_334
    );
    *req.actions.last_mut().unwrap() = normal(7_898_334);
    assert_eq!(run(&req)["execution_completed"], true);
}

#[test]
fn chaos_full_verdict_spends_once_across_targets_and_cannot_repeat_without_new_crime() {
    for targets in [1, 3] {
        let mut req = request(targets, false);
        req.actors[0].static_stats.break_intensity = Ratio::zero();
        req.targets[0].break_profile = Some(crate::simulation::GameTargetBreakProfileConfig {
            capacity: Ratio {
                numerator: 1000,
                denominator: 1,
            },
            current_gauge: Ratio {
                numerator: 1000,
                denominator: 1,
            },
            broken_duration_us: 1_000_000,
            discord_fraction: None,
        });
        let mut time = 0;
        for _ in 0..3 {
            super::chaos_crime::chain(&mut req, &mut time);
        }
        hold(&mut req, 17_000_000);
        hold(&mut req, 19_902_033);
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let hits = branch_hits(&report);
        assert_eq!(hits.len(), 6 * targets);
        let full: Vec<_> = hits
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "GE_Player_Chaos_Branch2_Damage")
            .collect();
        assert_eq!(full.len(), 2 * targets);
        assert_eq!(
            full.iter()
                .filter(|hit| hit["target_index"] == 0)
                .map(|hit| hit["time_us"].as_u64().unwrap())
                .collect::<Vec<_>>(),
            [18_964_181, 19_550_444]
        );
        assert_eq!(full[0]["scaling_rate"], json!("4/1"));
        assert_eq!(
            super::support::hit_metadata(&report, full[0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"
        );
        assert_eq!(crime(&report)["spent"], json!("1000/1"));
        assert_eq!(crime(&report)["current"], json!("0/1"));
        assert_eq!(crime(&report)["overflow"], json!("146/1"));
        assert_eq!(crime(&report)["conserved"], true);
        // Three3.6-Break chains + full2.1 + plain1.2 =14.1, without breaking.
        assert_eq!(
            report["final_target_states"][0]["break_state"]["gauge"],
            json!("9859/10")
        );
    }
}

#[test]
fn chaos_verdict_late_upgrade_settles_equal_time_auto_tick_once_before_read_and_action() {
    let mut req = request(3, true);
    hold(&mut req, 9_703_042);
    let report = run(&req);
    let hits = branch_hits(&report);
    assert_eq!(hits.len(), 12);
    assert_eq!(
        hits.iter()
            .filter(|hit| hit["target_index"] == 0)
            .map(|hit| hit["time_us"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [10_870_238, 11_586_229, 11_668_340, 12_218_928]
    );
    assert_eq!(crime(&report)["spent"], json!("1000/1"));
    assert_eq!(crime(&report)["current"], json!("780/1"));
    assert_eq!(crime(&report)["overflow"], json!("20/1"));
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["event_kind"] == "chaos_auto_crime")
            .count(),
        60
    );
    req.actions.push(normal(11_602_493));
    assert_eq!(
        run(&req)["rejected_action"]["reason"]["until_us"],
        12_235_686
    ); // Initial stage2 recovery, before future hit pauses.
}

#[test]
fn chaos_captured_transformed_verdict_continues_after_natural_expiry_and_retains_effective_talent()
{
    let mut req = request(1, true);
    req.actors[0].talent_levels = Some([8; 3]);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(3);
    hold(&mut req, 15_583_605);
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = branch_hits(&report);
    assert_eq!(
        hits.iter()
            .map(|hit| hit["time_us"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [16_750_801, 17_466_792, 17_548_903, 18_099_491]
    );
    for hit in &hits {
        assert!(
            hit["damage_effect_id"]
                .as_str()
                .unwrap()
                .ends_with("_Ultra")
        );
    }
    assert_eq!(hits[0]["scaling_rate"], json!("133/125"));
    assert_eq!(hits[2]["scaling_rate"], json!("1851/250"));
    assert_eq!(crime(&report)["current"], json!("0/1"));
    req.actions.push(normal(18_486_754));
    assert_eq!(
        run(&req)["rejected_action"]["reason"]["until_us"],
        18_486_755
    );
}

#[test]
fn chaos_interrupted_verdict_cancels_pending_spend_but_never_refunds_committed_full_stage() {
    for (time, spent, cancelled) in [
        (11_602_492, 0, 0),
        (11_602_493, 1000, 6),
        (11_668_340, 1000, 3),
    ] {
        let mut req = request(3, true);
        model(&mut req, crate::HypothesisId::ChaosForcedInterruptV1);
        hold(&mut req, 9_703_042);
        req.interrupt_events = vec![GameActorInterruptEvent {
            timestamp_us: time,
            actor_index: 0,
            armor_damage: None,
        }];
        let report = run(&req);
        assert_eq!(crime(&report)["spent"], json!(format!("{}/{}", spent, 1)));
        assert_eq!(
            report["interrupt_receipts"][0]["cancelled_hit_deliveries"],
            cancelled
        );
        assert!(report["interrupt_receipts"][0]["skill_cooldown_after_us"].is_null());
    }
}

#[test]
fn chaos_full_verdict_immunity_ends_at_actual_recovery_or_interruption_and_misses_do_not_refund() {
    let mut req = request(1, true);
    hold(&mut req, 9_703_042);
    model(&mut req, crate::HypothesisId::CombatIncomingDamageFixtureV1);
    req.incoming_damage_events = [7_803_698, 7_803_699]
        .map(|timestamp_us| GameActorDamageEvent {
            timestamp_us,
            actor_index: 0,
            delivered_damage: 100,
        })
        .to_vec();
    let report = run(&req);
    assert_eq!(report["incoming_damage_receipts"][0]["actual_hp_loss"], 0);
    assert_eq!(report["incoming_damage_receipts"][1]["actual_hp_loss"], 100);
    req.targets[0].resistance = Ratio::one();
    req.incoming_damage_events = [7_433_192, 7_433_193]
        .map(|timestamp_us| GameActorDamageEvent {
            timestamp_us,
            actor_index: 0,
            delivered_damage: 100,
        })
        .to_vec();
    let report = run(&req);
    assert_eq!(crime(&report)["spent"], json!("1000/1"));
    assert_eq!(report["incoming_damage_receipts"][0]["actual_hp_loss"], 0);
    assert_eq!(report["incoming_damage_receipts"][1]["actual_hp_loss"], 100);
    model(&mut req, crate::HypothesisId::ChaosForcedInterruptV1);
    req.interrupt_events = vec![GameActorInterruptEvent {
        timestamp_us: 11_683_605,
        actor_index: 0,
        armor_damage: None,
    }];
    req.incoming_damage_events = vec![GameActorDamageEvent {
        timestamp_us: 11_683_606,
        actor_index: 0,
        delivered_damage: 100,
    }];
    assert_eq!(
        run(&req)["incoming_damage_receipts"][0]["actual_hp_loss"],
        100
    );
}

#[test]
fn chaos_neutral_verdict_rejects_unbound_short_combo_and_caller_selected_branch() {
    let mut req = request(1, false);
    hold(&mut req, 0);
    req.hold_inputs.get_mut(&0).unwrap().duration_us = 999_999;
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("combo holds")
    );
    req.hold_inputs.get_mut(&0).unwrap().duration_us = 1_000_000;
    if let GameActionCommand::InvokeAbility {
        damage_effect_id, ..
    } = &mut req.actions[0]
    {
        *damage_effect_id = "GE_Player_Chaos_Branch2_Damage".into();
    }
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("caller-selected")
    );
}

#[test]
fn chaos_verdict_completion_settles_same_time_lethal_hp_before_spend_and_ordinary_admission() {
    let mut req = request(1, true);
    hold(&mut req, 9_703_042);
    model(&mut req, crate::HypothesisId::CombatIncomingDamageFixtureV1);
    req.incoming_damage_events = vec![GameActorDamageEvent {
        timestamp_us: 6_800_000,
        actor_index: 0,
        delivered_damage: 100_000,
    }];
    req.actions.push(normal(11_602_493));
    let report = run(&req);
    assert!(
        report["incoming_damage_receipts"][0]["actual_hp_loss"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert_eq!(report["final_actor_states"][0]["hp"], 0);
    assert_eq!(crime(&report)["spent"], json!("0/1"));
    assert_eq!(report["execution_completed"], false);
    assert_eq!(branch_hits(&report).len(), 2);
}

#[test]
fn chaos_neutral_verdict_pays_longer_authored_hold_and_sparse_time_adds_no_polling() {
    let mut req = request(1, false);
    hold(&mut req, 0);
    req.hold_inputs.get_mut(&0).unwrap().duration_us = 2_000_000;
    let report = run(&req);
    assert_eq!(branch_hits(&report)[0]["time_us"], 2_166_691);
    req.relative_duration_us = 600_000_000;
    let later = run(&req);
    assert_eq!(
        report["total_events_processed"],
        later["total_events_processed"]
    );
    req.actions.push(normal(2_898_333));
    assert_eq!(
        run(&req)["rejected_action"]["reason"]["until_us"],
        2_898_334
    );
}
