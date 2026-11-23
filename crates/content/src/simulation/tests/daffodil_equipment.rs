use super::daffodil_secondary::request as support_request;
use super::*;
use crate::characters::daffodil::secondary::QTE;
use serde_json::Value;

fn actor_request(powered_diabolos: bool) -> GameScenarioRequest {
    let mut req = support_request(1);
    for (slot, character_id) in [(1, "character:1042"), (2, "character:1039")] {
        let mut ally = req.actors[0].clone();
        ally.slot = slot;
        ally.character_id = character_id.into();
        ally.initial_cycle = 100;
        ally.initial_energy = 0;
        ally.awakening = Default::default();
        req.actors.push(ally);
    }
    req.active_actor_index = 1;
    let (diabolos, _) =
        crate::sets::diabolos::projection::compile(ntesim_data::stat_projection::frozen_tables())
            .unwrap();
    let mut diabolos = diabolos;
    if !powered_diabolos
        && let GearProcConfig::DiabolosChaosIgnore {
            chaos_res_ignore,
            powered_chaos_res_ignore,
            ..
        } = &mut diabolos
    {
        *powered_chaos_res_ignore = *chaos_res_ignore;
    }
    req.actors[0].weapon_id = "weapon:fork_BlackBook".into();
    req.actors[0].weapon_tier = 1;
    req.actors[0].set_id = Some("set:Suit1".into());
    req.actors[0].gear_procs = vec![
        GearProcConfig::BlackBookBreak {
            break_intensity_bonus: Ratio {
                numerator: 60,
                denominator: 1,
            },
        },
        diabolos,
    ];
    req.targets[0].elemental_resistances.insert(
        "CHAOS".into(),
        Ratio {
            numerator: 1,
            denominator: 5,
        },
    );
    req.actions = vec![
        switch(0, 0),
        switch(1_500_000, 2),
        switch(2_000_000, 0),
        GameActionCommand::PlayerInput {
            timestamp_us: 3_300_000,
            actor_index: 0,
            input: PlayerInput::Arc,
            critical_success: false,
        },
    ];
    req.relative_duration_us = 20_000_000;
    for target in &mut req.targets {
        target.break_profile = Some(GameTargetBreakProfileConfig {
            discord_fraction: None,
            capacity: Ratio {
                numerator: 100,
                denominator: 1,
            },
            current_gauge: Ratio {
                numerator: 100,
                denominator: 1,
            },
            broken_duration_us: 2_000_000,
        });
    }
    req
}

fn run(req: &GameScenarioRequest) -> Value {
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}

fn daffodil_qtes(report: &Value) -> Vec<&Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| {
            hit["actor_index"] == 0 && hit["target_index"] == 0 && hit["damage_effect_id"] == QTE
        })
        .collect()
}

fn fraction(value: &Value) -> ExactFraction {
    serde_json::from_value(value.clone()).unwrap()
}

#[test]
fn daffodil_support_funds_black_tome_arc_and_diabolos_only_powers_qualified_chaos_hits() {
    let powered_req = actor_request(true);
    let reference_req = actor_request(false);
    let powered = run(&powered_req);
    let reference = run(&reference_req);

    for report in [&powered, &reference] {
        assert_eq!(
            report["execution_completed"], true,
            "halt={:?}, rejected={:?}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let trace = report["trace"].as_array().unwrap();
        assert_eq!(
            trace
                .iter()
                .filter(|event| {
                    event["event_kind"] == "support_dispatch"
                        && event["data"]["ability_id"] == "GA_Daffodill_QTE"
                })
                .count(),
            2,
            "two engine-qualified Daffodil Supports fund the weapon"
        );
        let input = report["player_input_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|receipt| receipt["ability_id"] == "weapon:fork_BlackBook:arc")
            .expect("the funded Arc should be admitted");
        assert_eq!(input["time_us"], 3_300_000);
        let summon = trace
            .iter()
            .find(|event| event["event_kind"] == "black_book_arc")
            .expect("startup should complete into a live summon");
        assert_eq!(summon["time_us"], 3_500_000);
        assert_eq!(summon["data"]["target_index"], 0);
        assert_eq!(summon["data"]["expires_at_us"], 23_500_000);
    }

    let powered_hits = daffodil_qtes(&powered);
    let reference_hits = daffodil_qtes(&reference);
    assert_eq!(powered_hits.len(), 2);
    assert_eq!(reference_hits.len(), 2);
    for (enhanced, base) in powered_hits.iter().zip(reference_hits.iter()) {
        assert_eq!(enhanced["time_us"], base["time_us"]);
        // At 20% Chaos resistance the native 12% ignore leaves 8% RES (23/25).
        // The qualified window replaces it with 24% ignore, yielding -4% RES
        // and the standard-rational factor 27/26. The damage ratio is therefore
        // (27/26)/(23/25) = 675/598; no additive stack or unrelated packet gets it.
        let base_damage = fraction(&base["non_crit_damage"]);
        let expected = base_damage
            .try_mul(ExactFraction::new(675, 598).unwrap())
            .unwrap();
        assert_eq!(fraction(&enhanced["non_crit_damage"]), expected);
    }
    assert!(powered["trace"].as_array().unwrap().iter().any(|event| {
        event["event_kind"] == "reaction_equipment_window" && event["to_actor"] == 0
    }));
    assert!(reference["trace"].as_array().unwrap().iter().any(|event| {
        event["event_kind"] == "reaction_equipment_window" && event["to_actor"] == 0
    }));
    assert_eq!(powered_req.actors[0].weapon_tier, 1);
    assert_eq!(powered_req.actors[0].set_id.as_deref(), Some("set:Suit1"));
    assert_eq!(
        powered["resolved_model_assumptions"][crate::sets::diabolos::HYPOTHESIS_ID.as_str()],
        crate::sets::diabolos::HYPOTHESIS_ID.description()
    );
}

#[test]
fn daffodil_live_break_during_a_funded_arc_emits_the_designated_black_tome_hit() {
    let mut req = actor_request(true);
    req.relative_duration_us = 20_000_000;
    let arc_command = req.actions.len() - 1;
    req.actions.push(super::daffodil_clock::skill(5_000_000));
    req.targets[0].break_profile = Some(GameTargetBreakProfileConfig {
        discord_fraction: None,
        capacity: Ratio {
            numerator: 1,
            denominator: 1,
        },
        current_gauge: Ratio {
            numerator: 1,
            denominator: 1,
        },
        broken_duration_us: 100_000,
    });

    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "halt={:?}, rejected={:?}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let book_hits: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Fork_BlackBook_damage")
        .collect();
    assert!(
        !book_hits.is_empty(),
        "a live Daffodil Break should release the Arc follow-up; break packets={}, reported hits={}",
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["break_contribution"].is_object())
            .count(),
        book_hits.len()
    );
    assert!(book_hits.iter().all(|hit| {
        hit["actor_index"] == 0
            && hit["target_index"] == 0
            && hit["command_index"] == arc_command
            && hit["switch_origin"].is_null()
    }));
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|hit| {
                hit["ability_id"] == "ntesim.break"
                    && hit["actor_index"] == 0
                    && hit["break_contribution"].is_object()
            })
    );
}

#[test]
fn black_tome_startup_cannot_be_shortened_by_daffodil_jump_or_skill_recovery_overrides() {
    for followup in [
        GameActionCommand::PlayerInput {
            timestamp_us: 3_350_000,
            actor_index: 0,
            input: PlayerInput::Jump,
            critical_success: false,
        },
        super::daffodil_clock::skill(3_350_000),
    ] {
        let mut req = actor_request(true);
        req.actions.push(followup);
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], false,
            "halt={:?}, rejected={:?}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(
            report["rejected_action"]["reason"]["kind"], "recovery",
            "Black Tome owns this .2-second recovery even when Daffodil has a cancel window"
        );
        assert_eq!(report["rejected_action"]["reason"]["until_us"], 3_500_000);
    }
}

fn switch(timestamp_us: u64, target_actor_index: usize) -> GameActionCommand {
    GameActionCommand::SwitchActor {
        timestamp_us,
        target_actor_index,
        cooldown_us: 0,
    }
}
