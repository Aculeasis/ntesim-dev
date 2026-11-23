use super::*;
use crate::formula::Ratio;
use crate::simulation::GearProcConfig;
use serde_json::json;

fn fixture() -> GameScenarioRequest {
    let mut req = crate::simulation::tests::research_request();
    req.stochastic_crit = None;
    req.relative_duration_us = 100;
    req.actors[0].set_id = Some("set:Suit5".into());
    let (proc, _) = crate::sets::street_boxer::projection::compile(
        ntesim_data::stat_projection::frozen_tables(),
        false,
    )
    .unwrap();
    req.actors[0].gear_procs = vec![proc];
    req
}

fn run(req: &GameScenarioRequest) -> Value {
    let limits = ExecutionLimits::default();
    let packets = crate::simulation::validation::validate_and_resolve(req, &limits).unwrap();
    let (mut execution, assumptions) = PreparedExecution::new(req, req.seed, &[], &[]).unwrap();
    execution
        .process_events(req, &limits, &packets, &[], &[])
        .unwrap();
    serde_json::to_value(execution.into_report(req, req.seed, assumptions).unwrap()).unwrap()
}

fn reaction_fixture(other: &str, times: &[u64]) -> GameScenarioRequest {
    let mut req = fixture();
    req.relative_duration_us = 24_000_000;
    req.actors[0].character_id = "character:1025".into();
    req.actors[0].initial_cycle = 100;
    let mut partner = req.actors[0].clone();
    partner.slot = 1;
    partner.character_id = other.into();
    partner.set_id = None;
    partner.gear_procs.clear();
    req.actors.push(partner);
    let mut wearer = req.actors[0].clone();
    wearer.slot = 2;
    wearer.character_id = "character:1019".into();
    wearer.initial_cycle = 0;
    req.actors.push(wearer);
    let mut defeated = req.actors[2].clone();
    defeated.slot = 3;
    defeated.character_id = "character:1033".into();
    defeated.initial_hp = 0;
    req.actors.push(defeated);
    let hit = req.actions[0].clone();
    let metadata = req.action_metadata[&0].clone();
    req.actions = [(0, 1), (3_000_000, 0), (4_000_000, 2)]
        .map(|(timestamp_us, target_actor_index)| {
            crate::simulation::GameActionCommand::SwitchActor {
                timestamp_us,
                target_actor_index,
                cooldown_us: 0,
            }
        })
        .into();
    req.action_metadata.clear();
    for time in times {
        let mut packet = hit.clone();
        if let crate::simulation::GameActionCommand::InvokeAbility {
            actor_index,
            timestamp_us,
            ..
        } = &mut packet
        {
            *actor_index = 2;
            *timestamp_us = *time;
        }
        req.action_metadata
            .insert(req.actions.len(), metadata.clone());
        req.actions.push(packet);
    }
    req
}

#[test]
fn street_boxer_team_stain_and_remora_refresh_living_off_field_wearers_only() {
    for other in ["character:1042", "character:1052"] {
        let mut req = reaction_fixture(other, &[5_000_000]);
        req.relative_duration_us = 6_000_000;
        let report = run(&req);
        let buffs = crate::simulation::tests::support::final_buffs(&report);
        let windows: Vec<_> = buffs
            .iter()
            .filter(|buff| buff["id"] == "street_boxer.reaction_window")
            .collect();
        assert_eq!(windows.len(), 2);
        for (owner, window) in [0, 2].into_iter().zip(windows) {
            assert_eq!(window["owner_actor"], owner);
            assert_eq!(window["stacks"], 1);
            assert_eq!(window["recipient"]["type"], "self_actor");
            assert_eq!(window["duration"]["expires_at_us"], 23_000_000);
        }
        let hit = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|hit| hit["damage_effect_id"] == "caller-effect")
            .unwrap();
        assert_eq!(
            hit["crit_rate"],
            if other == "character:1052" {
                json!("37/50")
            } else {
                json!("16/25")
            },
            "the set's14% replaces14%; enhanced Remora independently adds10% on its target"
        );
        assert_eq!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["event_kind"] == "reaction_equipment_window")
                .count(),
            4
        );
    }
}

#[test]
fn street_boxer_powered_replacement_and_exact_expiry_are_observable_with_distinct_test_values() {
    let mut req = reaction_fixture("character:1042", &[22_999_999, 23_000_000]);
    for actor in &mut req.actors {
        for proc in &mut actor.gear_procs {
            if let crate::simulation::GearProcConfig::StreetBoxerCrit {
                powered_crit_rate_bonus,
                ..
            } = proc
            {
                // A research-only distinct value discriminates the branch even
                // though both actual frozen source curves currently equal .14.
                *powered_crit_rate_bonus = crate::formula::Ratio {
                    numerator: 7,
                    denominator: 25,
                };
            }
        }
    }
    let report = run(&req);
    assert_eq!(report["hit_receipts"][0]["crit_rate"], json!("39/50"));
    assert_eq!(report["hit_receipts"][1]["crit_rate"], json!("16/25"));
    // A Nova pair is unrelated to Street Boxer, even with an equipped third actor.
    req.actors[0].character_id = "character:1004".into();
    let report = run(&req);
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| event["event_kind"] != "reaction_equipment_window")
    );
    for hit in report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["damage_effect_id"] == "caller-effect")
    {
        assert_eq!(hit["crit_rate"], json!("16/25"));
    }
}

#[test]
fn street_boxer_compiled_baseline_and_legacy_request_keep_identical_replacement_and_expiry() {
    for baked in [false, true] {
        let mut req = reaction_fixture("character:1042", &[22_999_999, 23_000_000]);
        for actor in &mut req.actors {
            for proc in &mut actor.gear_procs {
                if let GearProcConfig::StreetBoxerCrit {
                    powered_crit_rate_bonus,
                    baseline_in_stats,
                    ..
                } = proc
                {
                    *powered_crit_rate_bonus = Ratio {
                        numerator: 7,
                        denominator: 25,
                    };
                    *baseline_in_stats = baked;
                    if baked {
                        actor.static_stats.crit_rate = Ratio {
                            numerator: 16,
                            denominator: 25,
                        };
                    }
                }
            }
        }
        let report = run(&req);
        assert_eq!(report["hit_receipts"][0]["crit_rate"], json!("39/50"));
        assert_eq!(report["hit_receipts"][1]["crit_rate"], json!("16/25"));
    }
}

#[test]
fn street_boxer_window_parameters_require_consumed_ratios_and_real_deadlines() {
    for (key, value) in [
        ("Lakshana_4_2", json!(0)),
        ("Lakshana_4_2", json!(14)),
        ("Lakshana_4_CD", json!(0)),
    ] {
        let mut tables = ntesim_data::stat_projection::frozen_tables().clone();
        tables.insert(
            "CT_Equipmentbuff".into(),
            ntesim_data::set_parameters::overlay(),
        );
        tables.get_mut("CT_Equipmentbuff").unwrap()["rows"][key]["Keys"][0]["Value"] = value;
        assert!(crate::sets::street_boxer::projection::compile(&tables, false).is_err());
    }
    let mut req = fixture();
    if let crate::simulation::GearProcConfig::StreetBoxerCrit { duration_us, .. } =
        &mut req.actors[0].gear_procs[0]
    {
        *duration_us = u64::MAX;
    }
    assert!(
        crate::simulation::validate_game_request(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("deadline overflow")
    );
}

#[test]
fn street_boxer_four_piece_adds_the_frozen_crit_ratio_to_every_packet() {
    let report = run(&fixture());
    // Fixture crit rate 1/2 plus the frozen 0.14 bonus joins additively: 16/25.
    for hit in report["hit_receipts"].as_array().unwrap() {
        assert_eq!(hit["crit_rate"], json!("16/25"));
    }
    assert_eq!(
        report["resolved_model_assumptions"][crate::sets::street_boxer::HYPOTHESIS_ID.as_str()],
        crate::sets::street_boxer::HYPOTHESIS_ID.description()
    );
    let mut baseline = fixture();
    baseline.actors[0].set_id = None;
    baseline.actors[0].gear_procs.clear();
    for hit in run(&baseline)["hit_receipts"].as_array().unwrap() {
        assert_eq!(hit["crit_rate"], json!("1/2"));
    }
}

#[test]
fn street_boxer_projection_binds_the_frozen_curve_and_validates_native_equipment() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    let (proc, native) = crate::sets::street_boxer::projection::compile(tables, false).unwrap();
    assert_eq!(
        serde_json::to_value(&proc).unwrap(),
        json!({"kind":"street_boxer_crit","crit_rate_bonus":"7/50",
            "powered_crit_rate_bonus":"7/50","duration_us":20_000_000})
    );
    assert_eq!(
        native["parameters"]["Lakshana_4_1"]["selected"],
        json!({"numerator":"7","denominator":"50"})
    );
    let mut actor = fixture().actors.remove(0);
    actor.gear_procs.clear();
    let equipped =
        json!({"set_id":"set:Suit5","set_count":4,"unresolved_set_buffs":[{"threshold":4}]});
    let project = crate::sets::street_boxer::projection::project_equipped;
    assert!(project(tables, &mut actor, &equipped).unwrap().is_some());
    assert!(project(tables, &mut actor, &equipped).unwrap().is_some());
    assert_eq!(actor.gear_procs.len(), 1);
    actor.gear_procs.push(proc);
    assert!(project(tables, &mut actor, &equipped).is_err());
    actor.gear_procs.clear();
    let two_piece = json!({"set_id":"set:Suit5","set_count":2,"unresolved_set_buffs":[]});
    assert!(project(tables, &mut actor, &two_piece).unwrap().is_none());
    assert!(
        project(
            tables,
            &mut actor,
            &json!({"set_id":"set:Suit5","set_count":4,"unresolved_set_buffs":[]})
        )
        .is_err()
    );
    for value in [json!(0), json!(14)] {
        let mut changed = tables.clone();
        changed.insert(
            "CT_Equipmentbuff".into(),
            ntesim_data::set_parameters::overlay(),
        );
        changed.get_mut("CT_Equipmentbuff").unwrap()["rows"]["Lakshana_4_1"]["Keys"][0]["Value"] =
            value;
        assert!(crate::sets::street_boxer::projection::compile(&changed, false).is_err());
    }
    let mut invalid = fixture();
    invalid.actors[0].set_id = None;
    assert!(
        crate::simulation::validate_game_request(&invalid, &ExecutionLimits::default())
            .unwrap_err()
            .contains("set:Suit5")
    );
}

#[test]
fn source_build_enables_street_boxer_four_piece_and_diabolos_stays_inert_off_element() {
    let run = |cartridge: &str, count: u8| {
        let compiled = crate::dsl::compiler::compile_scenario(
            &format!(
                "options iteration=1 duration=2 crit=fixed; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; \
                 mint add cartridge={cartridge} count={count}; \
                 target hp=100000000 def=0 resist=0; active mint; mint attack;"
            ),
            Default::default(),
        )
        .unwrap();
        crate::simulation::run_game_scenario(&compiled.request, &ExecutionLimits::default())
            .unwrap()
    };
    let crit_rate = |report: &Value| {
        serde_json::from_value::<crate::simulation::ExactFraction>(
            report["hit_receipts"][0]["crit_rate"].clone(),
        )
        .unwrap()
    };
    // Four pieces add exactly the frozen 0.14 to the compiled crit rate.
    assert_eq!(
        crit_rate(&run("Suit5", 4)),
        crit_rate(&run("Suit5", 2))
            .add(crate::simulation::ExactFraction::new(7, 50).unwrap())
            .unwrap()
    );
    // Diabolos binds its proc through the same source path but Mint's Nature
    // packets never meet the Chaos gate, so damage is unchanged.
    assert_eq!(
        run("Suit1", 4)["hit_receipts"][0]["non_crit_damage"],
        run("Suit1", 2)["hit_receipts"][0]["non_crit_damage"]
    );
}
