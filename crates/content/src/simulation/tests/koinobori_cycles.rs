use super::*;

fn request(tier: u32, qualified: bool, awakening: u32) -> GameScenarioRequest {
    let mut source = include_str!("../../../../../examples/koinobori-shield-cycles.nte")
        .replace("refine=1", &format!("refine={tier}"));
    if !qualified {
        source = source.replace("zero char", "sakiri char");
    }
    let mut req = crate::dsl::compiler::compile_scenario(&source, Default::default())
        .unwrap()
        .request;
    req.actors[0].awakening =
        ntesim_data::AwakeningSelection::selected(if awakening == 4 { vec![4] } else { vec![] })
            .unwrap();
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def = 1000;
    req.actors[0].static_stats.def_percent = Ratio::zero();
    req.actors[0].static_stats.def_flat = Ratio::zero();
    req.actors[0].static_stats.shield_bonus = Ratio::zero();
    req.incoming_damage_events = [1_000_000, 21_000_000]
        .into_iter()
        .map(|timestamp_us| crate::simulation::GameActorDamageEvent {
            timestamp_us,
            actor_index: 0,
            delivered_damage: 3000,
        })
        .collect();
    let id = crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    req
}

#[test]
fn koinobori_three_element_native_shields_and_a4_use_base_scaled_def_at_every_rank() {
    for (tier, shield, overflow) in [
        (1, 2040, 650),
        (2, 2056, 660),
        (3, 2072, 670),
        (4, 2088, 680),
        (5, 2104, 690),
    ] {
        for qualified in [false, true] {
            for awakening in [0, 4] {
                let req = request(tier, qualified, awakening);
                let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
                assert_eq!(
                    report["execution_completed"], true,
                    "{}",
                    report["rejected_action"]
                );
                let expected_shield = if qualified { shield } else { 1960 };
                let cap = if qualified { overflow } else { 600 };
                let extra = if awakening == 4 { cap } else { 0 };
                for receipt in report["incoming_damage_receipts"].as_array().unwrap() {
                    assert_eq!(receipt["absorbed_by_shields"][0]["amount"], expected_shield);
                    assert_eq!(receipt["actual_hp_loss"], 3000 - expected_shield - extra);
                    if awakening == 4 {
                        assert_eq!(receipt["absorbed_by_effects"][0]["amount"], cap);
                    }
                }
                // The shield snapshots DEF; the independent A4 cap uses live DEF
                // without a shield-efficiency multiplier. Both cycles retain HP loss.
                assert_eq!(
                    report["final_actor_states"][0]["hp"],
                    req.actors[0].initial_hp - 2 * (3000 - expected_shield - extra)
                );
                assert_eq!(
                    report,
                    run_game_scenario(&req, &ExecutionLimits::default()).unwrap()
                );
            }
        }
    }
}

#[test]
fn koinobori_def_does_not_multiply_existing_percentages_or_flat_def() {
    let mut req = request(1, true, 4);
    req.actors[0].static_stats.def_percent = Ratio {
        numerator: 1,
        denominator: 2,
    };
    req.actors[0].static_stats.def_flat = Ratio {
        numerator: 100,
        denominator: 1,
    };
    req.actors[0].static_stats.def = 1600;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    // DEF1000*(1+.5+.1+.2)+100=1900 and Blessing1000+.8*1900=2520.
    // A4 .5*1900 clamps to the remaining incoming 480.
    assert_eq!(
        report["incoming_damage_receipts"][0]["absorbed_by_shields"][0]["amount"],
        2520
    );
    assert_eq!(
        report["incoming_damage_receipts"][0]["absorbed_by_effects"][0]["amount"],
        480
    );
    assert_eq!(report["incoming_damage_receipts"][0]["actual_hp_loss"], 0);
}

#[test]
fn koinobori_party_element_condition_survives_an_off_field_ally_defeat() {
    let mut req = request(1, true, 0);
    req.actors[1].initial_hp = 0;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    for receipt in report["incoming_damage_receipts"].as_array().unwrap() {
        assert_eq!(receipt["absorbed_by_shields"][0]["amount"], 2040);
    }
    assert_eq!(report["final_actor_states"][1]["hp"], 0);
}
