use super::*;

#[test]
fn native_source_quality_separates_disclosed_dps_from_failure_and_strict_evidence() {
    use crate::dsl::compiler::SourceRequest;
    let source = "options duration=10 crit=fixed; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0; target hp=100000000 def=0 resist=0; active mint; mint attack;";
    let trial = run_source(
        &SourceRequest {
            source: source.into(),
            limits: Default::default(),
        },
        &Default::default(),
    )
    .unwrap()
    .execution
    .unwrap();
    assert_eq!(
        trial.report.quality,
        ntesim_contracts::Quality::BoundedHypothesis,
        "{:?}",
        trial.report.model_assumptions
    );
    assert!(trial.report.game_dps_computed);
    assert_eq!(trial.report.absolute_elapsed_us, 10_000_000);
    let replay = run_game_scenario_report(&trial.request, &Default::default()).unwrap();
    assert_eq!(replay.quality, trial.report.quality);
    assert_eq!(replay.game_dps_computed, trial.report.game_dps_computed);
    let failed = run_source(
        &SourceRequest {
            source: source.replace("mint attack;", "mint burst;"),
            limits: Default::default(),
        },
        &Default::default(),
    )
    .unwrap()
    .execution
    .unwrap();
    assert_eq!(
        failed.report.quality,
        ntesim_contracts::Quality::BoundedHypothesis
    );
    assert!(!failed.report.game_dps_computed && !failed.report.execution_completed);
    let mut unsupported = trial.request;
    unsupported
        .model_assumptions
        .insert("unsupported.model".into(), "Unresolved rule".into());
    let report = run_game_scenario_report(&unsupported, &Default::default()).unwrap();
    assert_eq!(
        report.quality,
        ntesim_contracts::Quality::UnboundedHypothesis
    );
    assert!(!report.game_dps_computed);
}

#[test]
pub(crate) fn stochastic_crit_matches_pcg_golden_words_and_exact_damage() {
    let req = research_request();
    let limits = ExecutionLimits::default();
    let report = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report, run_game_scenario(&req, &limits).unwrap());
    let hits = report["hit_receipts"].as_array().unwrap();
    let words = [
        0xa15c02b7u32,
        0x7b47f409,
        0xba1d3330,
        0x83d2f293,
        0xbfa4784b,
        0xcbed606e,
    ];
    for (i, (hit, word)) in hits.iter().zip(words).enumerate() {
        assert_eq!(hit["critical"], word % 2 == 0);
        assert_eq!(hit["crit_sample"]["draws"][0]["value"], word);
        assert_eq!(hit["crit_sample"]["draws"][0]["index"], i);
        assert_eq!(hit["rounded_damage"], if word % 2 == 0 { 800 } else { 400 });
    }
    assert_eq!(report["total_damage_exact"], serde_json::json!("3200/1"));
    assert_eq!(report["rng"]["streams"]["character:1019"]["draws"], 6);
    assert_eq!(report["quality"], "incomplete");
    assert_eq!(report["game_dps_computed"], false);
    let mut changed = req.clone();
    changed.seed = 43;
    assert_ne!(
        report["hit_receipts"],
        run_game_scenario(&changed, &limits).unwrap()["hit_receipts"]
    );
}

#[test]
pub(crate) fn research_probability_endpoints_and_ineligible_effects_do_not_draw() {
    let limits = ExecutionLimits::default();
    for (numerator, denominator, critical) in [(0, 1, false), (1, 1, true)] {
        let mut req = research_request();
        req.actors[0].static_stats.crit_rate = Ratio {
            numerator,
            denominator,
        };
        let report = run_game_scenario(&req, &limits).unwrap();
        assert_eq!(report["rng"]["streams"]["character:1019"]["draws"], 0);
        for hit in report["hit_receipts"].as_array().unwrap() {
            assert_eq!(hit["critical"], critical);
            assert!(hit.get("crit_sample").is_none());
        }
    }
    let mut req = research_request();
    req.stochastic_crit
        .as_mut()
        .unwrap()
        .eligible_effects
        .insert(
            "character:1019".into(),
            BTreeSet::from(["excluded-effect".into()]),
        );
    let report = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report["rng"]["streams"]["character:1019"]["draws"], 0);
    assert_eq!(report["total_damage_dealt"], 2400.0);
}

#[test]
pub(crate) fn stochastic_crit_rejects_implicit_policy_bad_rates_and_streams() {
    let limits = ExecutionLimits::default();
    for rate in [
        Ratio {
            numerator: 1,
            denominator: 0,
        },
        Ratio {
            numerator: 3,
            denominator: 2,
        },
        Ratio {
            numerator: 1,
            denominator: (1 << 32) + 1,
        },
    ] {
        let mut req = research_request();
        req.actors[0].static_stats.crit_rate = rate;
        assert!(run_game_scenario(&req, &limits).is_err());
    }
    let mut req = research_request();
    req.stochastic_crit.as_mut().unwrap().hypothesis.clear();
    assert!(run_game_scenario(&req, &limits).is_err());
    let mut req = research_request();
    req.stochastic_crit
        .as_mut()
        .unwrap()
        .actor_sequences
        .insert("unknown-owner".into(), 54);
    assert!(run_game_scenario(&req, &limits).is_err());
    let mut value = serde_json::to_value(research_request()).unwrap();
    value["stochastic_crit"]["roll_policy"] = serde_json::json!("implicit_game_policy");
    assert!(serde_json::from_value::<GameScenarioRequest>(value).is_err());
}

#[test]
pub(crate) fn failed_admission_and_hits_beyond_horizon_consume_no_random_words() {
    let limits = ExecutionLimits::default();
    let mut req = research_request();
    if let GameActionCommand::InvokeAbility { energy_cost, .. } = &mut req.actions[0] {
        *energy_cost = 101;
    }
    let report = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report["success"], false);
    assert_eq!(report["rng"]["streams"]["character:1019"]["draws"], 0);
    let mut req = research_request();
    req.actions.truncate(1);
    req.action_metadata
        .retain(|index, _| *index < req.actions.len());
    req.relative_duration_us = 100;
    if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut req.actions[0] {
        *hit_delay_us = 101;
    }
    let report = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report["execution_completed"], true);
    assert_eq!(report["pending_events"], 1);
    assert_eq!(report["hits_settled"], 0);
    assert_eq!(report["rng"]["streams"]["character:1019"]["draws"], 0);
    if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut req.actions[0] {
        *hit_delay_us = 100;
    }
    assert_eq!(run_game_scenario(&req, &limits).unwrap()["hits_settled"], 1);
}

#[test]
pub(crate) fn unhit_targets_do_not_consume_research_random_words() {
    let mut req = research_request();
    req.actions.truncate(1);
    req.action_metadata
        .retain(|index, _| *index < req.actions.len());
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 1);
    assert_eq!(report["hit_receipts"][0]["target_index"], 0);
    assert_eq!(
        report["final_target_states"][1]["hp_current"],
        req.targets[1].hp_current
    );
    assert_eq!(report["rng"]["streams"]["character:1019"]["draws"], 1);
}

#[test]
fn explicit_all_hit_wildcard_includes_new_packets_and_content_child_hits() {
    let limits = ExecutionLimits::default();
    let mut req = research_request();
    req.stochastic_crit
        .as_mut()
        .unwrap()
        .eligible_effects
        .insert("character:1019".into(), BTreeSet::from(["*".into()]));
    for (index, command) in req.actions.iter_mut().enumerate() {
        if let GameActionCommand::InvokeAbility {
            damage_effect_id, ..
        } = command
        {
            *damage_effect_id = format!("new_packet_{index}");
            req.action_metadata
                .get_mut(&index)
                .expect("each synthetic action has an explicit metadata declaration")
                .damage_effect_id = damage_effect_id.clone();
        }
    }
    let report = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report["rng"]["streams"]["character:1019"]["draws"], 6);
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|hit| hit.get("crit_sample").is_some())
    );

    req.action_metadata.clear();
    req.actions.truncate(1);
    req.action_metadata
        .retain(|index, _| *index < req.actions.len());
    req.actors[0].character_id = "character:1046".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
    let policy = req.stochastic_crit.as_mut().unwrap();
    policy.eligible_effects =
        BTreeMap::from([("character:1046".into(), BTreeSet::from(["*".into()]))]);
    policy.actor_sequences = BTreeMap::from([("character:1046".into(), 54)]);
    if let GameActionCommand::InvokeAbility { ability_id, .. } = &mut req.actions[0] {
        *ability_id = "GA_Female046_Skill".into();
    }
    if let GameActionCommand::InvokeAbility {
        damage_effect_id, ..
    } = &mut req.actions[0]
    {
        *damage_effect_id = "GE_Player_Female046_Skill1_Damage".into();
    }
    let report = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report["hits_settled"], 5);
    assert_eq!(report["rng"]["streams"]["character:1046"]["draws"], 5);
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(
        hits.iter()
            .map(|hit| hit["damage_effect_id"].as_str().unwrap())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "GE_Player_Female046_Skill1_Damage",
            "GE_Player_Female046_Skill2_Damage",
            "GE_Player_Female046_Skill3_Damage",
            "GE_Player_Female046_Skill4_Damage",
            "GE_Player_Female046_Skill_Kill_Damage_lv1",
        ])
    );
    assert!(hits.iter().all(|hit| hit.get("crit_sample").is_some()));
}

#[test]
pub(crate) fn fixed_packet_crit_probability_replaces_actor_rate_for_manual_actions() {
    let mut req = research_request();
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.model_assumptions.insert(
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_ID
            .description()
            .into(),
    );
    for metadata in req.action_metadata.values_mut() {
        metadata.fixed_crit_rate = Some(crate::simulation::ResearchFixedCritRate::Probability {
            rate: Ratio {
                numerator: 1,
                denominator: 2,
            },
        });
    }

    let limits = ExecutionLimits::default();
    let fixed_rate_report = run_game_scenario(&req, &limits).unwrap();
    let fixed_rate_hits = fixed_rate_report["hit_receipts"].as_array().unwrap();
    assert_eq!(fixed_rate_hits.len(), 6);
    for hit in fixed_rate_hits {
        assert_eq!(hit["crit_rate"], serde_json::json!("1/2"));
    }
    assert_eq!(
        fixed_rate_report["rng"]["streams"]["character:1019"]["draws"],
        6
    );

    let mut actor_rate_changed = req;
    actor_rate_changed.actors[0].static_stats.crit_rate = Ratio::one();
    let changed_report = run_game_scenario(&actor_rate_changed, &limits).unwrap();
    assert_eq!(
        fixed_rate_report["hit_receipts"],
        changed_report["hit_receipts"]
    );
}

#[test]
fn true_damage_does_not_apply_or_sample_critical_rate() {
    let mut req = research_request();
    req.actions.truncate(1);
    req.action_metadata
        .retain(|index, _| *index < req.actions.len());
    req.stochastic_crit
        .as_mut()
        .unwrap()
        .eligible_effects
        .insert("character:1019".into(), BTreeSet::from(["*".into()]));
    req.action_metadata.get_mut(&0).unwrap().damage_type_ex =
        Some("EExecutionDamageTypeEX::DAMAGE_TYPE_TRUE".into());

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let hit = &report["hit_receipts"][0];
    assert_eq!(hit["crit_rate"], serde_json::json!("0/1"));
    assert_eq!(hit["critical"], false);
    assert!(hit["crit_sample"].is_null());
    assert_eq!(hit["damage"], hit["non_crit_damage"]);
    assert_eq!(hit["damage"], hit["crit_damage"]);
    assert_eq!(report["rng"]["streams"]["character:1019"]["draws"], 0);
}
