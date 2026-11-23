use super::*;
use serde_json::Value;

const SOURCE: &str = include_str!("../../../../../examples/reality-refuge-cycles.nte");

fn fraction(value: &Value) -> ExactFraction {
    serde_json::from_value(value.clone()).unwrap()
}

#[test]
fn butterfly_native_refinements_replace_attachment_bonus_across_two_continuing_bursts() {
    // Independent frozen card ratios, in units of 1/800. Summed damage zone:
    // T1 .15/.10/.20; T2 .175/.1125/.225; ...; T5 .25/.15/.30.
    for (tier, anima, idle, ultimate) in [
        (1, 120, 80, 160),
        (2, 140, 90, 180),
        (3, 160, 100, 200),
        (4, 180, 110, 220),
        (5, 200, 120, 240),
    ] {
        for empowered in [false, true] {
            let mut source = SOURCE.replace("refine=1", &format!("refine={tier}"));
            if empowered {
                source = source.replace("awakening=0 talent=1,1,1", "awakening=[2] talent=8,1,1");
            }
            let mut req = crate::dsl::compiler::compile_scenario(&source, Default::default())
                .unwrap()
                .request;
            let actor = &mut req.actors[0];
            actor.static_stats.base_atk = 1000;
            actor.static_stats.atk = Ratio {
                numerator: 1000,
                denominator: 1,
            };
            actor.static_stats.crit_rate = Ratio::zero();
            actor.static_stats.damage_bonus = Ratio::zero();
            actor.static_stats.elemental_bonus = Ratio::zero();
            actor.static_stats.elemental_bonuses.clear();
            let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
            assert_eq!(
                report["execution_completed"], true,
                "{}",
                report["rejected_action"]
            );
            let mut control = req.clone();
            control.actors[0].gear_procs.clear();
            let plain = run_game_scenario(&control, &ExecutionLimits::default()).unwrap();
            let hits = report["hit_receipts"].as_array().unwrap();
            let bare = plain["hit_receipts"].as_array().unwrap();
            assert_eq!(hits.len(), bare.len());
            let mut attachment_count = 0;
            let mut off_field_boosted = 0;
            for (hit, base) in hits.iter().zip(bare) {
                for key in [
                    "time_us",
                    "actor_index",
                    "target_index",
                    "command_index",
                    "damage_effect_id",
                ] {
                    assert_eq!(hit[key], base[key]);
                }
                if hit["actor_index"] != 0 {
                    assert_eq!(hit["non_crit_damage"], base["non_crit_damage"]);
                    continue;
                }
                let attachment = crate::characters::jiuyuan::pact::is_packet(
                    hit["damage_effect_id"].as_str().unwrap(),
                );
                let time = hit["time_us"].as_u64().unwrap();
                let active = (2_000_000..10_000_000).contains(&time)
                    || (36_000_000..44_000_000).contains(&time);
                let bonus = anima
                    + if attachment {
                        if active { ultimate } else { idle }
                    } else {
                        0
                    };
                let factor = ExactFraction::new(800 + bonus, 800).unwrap();
                assert_eq!(
                    fraction(&hit["non_crit_damage"]),
                    fraction(&base["non_crit_damage"]).mul(factor).unwrap(),
                    "T{tier}, {time}, {}",
                    hit["damage_effect_id"]
                );
                if attachment {
                    attachment_count += 1;
                    let periodic = hit["damage_effect_id"] == "GE_Player_Kuhara_Seed_Damage";
                    let independent_base = if periodic {
                        if empowered { 514 } else { 150 }
                    } else if empowered {
                        1714
                    } else {
                        1000
                    };
                    assert_eq!(
                        fraction(&base["non_crit_damage"]),
                        ExactFraction::new(independent_base, 1).unwrap()
                    );
                    if active && time >= 40_000_000 {
                        off_field_boosted += 1;
                    }
                }
            }
            assert_eq!(attachment_count, 124);
            assert!(off_field_boosted > 0);
            assert_eq!(report["final_actor_states"][0]["spent_energy"], 200);
            assert_eq!(report["final_actor_states"][0]["energy_conserved"], true);
            assert!(
                report["final_actor_states"][0]["overflow_energy"]
                    .as_u64()
                    .unwrap()
                    > 0
            );
            assert_eq!(
                report["resolved_model_assumptions"]
                    [crate::weapons::butterfly::HYPOTHESIS_ID.as_str()],
                crate::weapons::butterfly::HYPOTHESIS_ID.description()
            );
            let replay: GameScenarioRequest =
                serde_json::from_value(serde_json::to_value(&req).unwrap()).unwrap();
            assert_eq!(
                report,
                run_game_scenario(&replay, &ExecutionLimits::default()).unwrap()
            );
        }
    }
}

#[test]
fn butterfly_rejected_and_foreign_ultimates_cannot_open_the_attachment_window() {
    let rejected = SOURCE.replace("start_energy=100", "start_energy=0");
    let req = crate::dsl::compiler::compile_scenario(&rejected, Default::default())
        .unwrap()
        .request;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 0);
    assert!(
        crate::simulation::tests::support::final_buffs(&report)
            .iter()
            .all(|buff| buff["id"] != "butterfly.ult_attachment")
    );

    let foreign = SOURCE
        .replace(
            "mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;",
            "mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=100;",
        )
        .replacen(
            "wait_until(2); jiuyuan burst;",
            "wait_until(2); mint burst;",
            1,
        );
    let req = crate::dsl::compiler::compile_scenario(&foreign, Default::default())
        .unwrap()
        .request;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["rejected_action"]
    );
    let mut plain = req.clone();
    plain.actors[0].gear_procs.clear();
    let control = run_game_scenario(&plain, &ExecutionLimits::default()).unwrap();
    let mut observed = 0;
    for (hit, base) in report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .zip(control["hit_receipts"].as_array().unwrap())
    {
        let time = hit["time_us"].as_u64().unwrap();
        if hit["actor_index"] == 0
            && (2_000_000..10_000_000).contains(&time)
            && crate::characters::jiuyuan::pact::is_packet(
                hit["damage_effect_id"].as_str().unwrap(),
            )
        {
            observed += 1;
            // T1 idle: 1 + .15 Anima + .10 Attachment, despite ally Ultimate.
            assert_eq!(
                fraction(&hit["non_crit_damage"]),
                fraction(&base["non_crit_damage"])
                    .mul(ExactFraction::new(5, 4).unwrap())
                    .unwrap()
            );
        }
    }
    assert_eq!(observed, 12);
}
