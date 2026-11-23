use super::*;

fn scenario() -> GameScenarioRequest {
    let mut req = phase2_request();
    for id in [
        crate::HypothesisId::SkiaGnawEntryStateV1,
        crate::HypothesisId::SkiaSkillBurstPacketsV1,
        crate::HypothesisId::SkiaOrdinaryHitEnergyV1,
        crate::HypothesisId::BaicangSkiaEntryV1,
    ] {
        req.model_assumptions
            .insert(id.into(), id.description().into());
    }
    let mut skia = req.actors[0].clone();
    skia.slot = 1;
    skia.character_id = "character:1008".into();
    req.actors.push(skia);
    req.active_actor_index = 1;
    req.actions = vec![skia_skill(0), switch(1_335_873, 0)];
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actors[1].static_stats.crit_rate = Ratio::zero();
    req
}

fn skia_skill(time: u64) -> GameActionCommand {
    make_action(
        time,
        1,
        "GA_Skia_Skill",
        "GE_Player_Skia_Skill1_Damage",
        0,
        0,
    )
}

fn switch(time: u64, target: usize) -> GameActionCommand {
    GameActionCommand::SwitchActor {
        timestamp_us: time,
        target_actor_index: target,
        cooldown_us: 0,
    }
}

fn pulses(report: &serde_json::Value) -> Vec<&serde_json::Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Cang_Skia_Damage")
        .collect()
}

#[test]
fn actual_skia_state_selects_three_aoe_pulses_and_one_word_from_the_switch() {
    let mut req = scenario();
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    req.actions.push(normal(1_735_339));
    let report = run(&req);
    let hits = pulses(&report);
    assert_eq!(hits.len(), 6);
    assert_eq!(
        hits.iter()
            .map(|h| h["time_us"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![
            1_913_903, 1_913_903, 1_993_903, 1_993_903, 2_073_903, 2_073_903
        ]
    );
    assert!(hits.iter().all(|hit| hit["command_index"] == 1));
    assert!(
        hits.iter()
            .all(|hit| hit["non_crit_damage"] == json!("405/1"))
    );
    assert_eq!(report["final_actor_states"][0]["hp"], 9700);
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["event_kind"] == "owner_entry")
            .count(),
        1
    );
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["event_kind"] == "support_dispatch")
    );
    assert!(report["final_actor_states"][1]["cycle"].is_null());
}

#[test]
fn party_presence_unselected_state_or_missing_passive_does_not_create_an_assist() {
    let baseline = scenario();
    for mutation in 0..3 {
        let mut req = baseline.clone();
        match mutation {
            0 => {
                req.actions.remove(0);
            }
            1 => {
                req.model_assumptions
                    .remove(crate::HypothesisId::SkiaGnawEntryStateV1.as_str());
                req.actions[1] = switch(2_000_000, 0);
            }
            _ => {
                req.actors[0].character_level = 40;
                req.actors[0].purchased_qte_level = Some(1);
                req.actors[0].character_max_level = Some(40);
            }
        };
        let report = run(&req);
        assert!(pulses(&report).is_empty());
        assert_eq!(report["final_actor_states"][0]["hp"], 10000);
    }
}

#[test]
fn special_entry_cooldown_is_independent_and_exactly_half_open_across_field_exits() {
    for (at, count) in [(31_335_872, 3), (31_335_873, 6)] {
        let mut req = scenario();
        req.relative_duration_us = 34_000_000;
        req.actions
            .extend([switch(29_999_998, 1), skia_skill(29_999_999), switch(at, 0)]);
        let report = run(&req);
        assert_eq!(pulses(&report).len(), count);
        assert_eq!(
            report["final_actor_states"][0]["hp"],
            if count == 3 { 9700 } else { 9409 }
        );
    }
}

#[test]
fn forged_proactive_packet_rejects_and_retained_gnaw_respects_time_stop() {
    let mut req = scenario();
    req.active_actor_index = 0;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Cang_Melee",
        "GE_Player_Cang_Skia_Damage",
        0,
        0,
    )];
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("engine-selected")
    );
    req = scenario();
    req.actions.pop();
    req.relative_duration_us = 2_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 7);
    assert_eq!(
        report["hit_receipts"].as_array().unwrap().last().unwrap()["damage_effect_id"],
        "GE_Player_Skia_SkillShadowAtk_Damage"
    );
    assert_eq!(
        report["hit_receipts"].as_array().unwrap().last().unwrap()["time_us"],
        1_398_984
    );
    req.relative_duration_us = 1_398_983;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true); // Later Gnaw work stays pending after the time stop.
    assert_eq!(report["hits_settled"], 6);
    assert_eq!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|h| h["time_us"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![153_592, 216_117, 296_382, 369_313, 871_421, 913_168]
    );
}

#[test]
fn two_real_skia_entry_cycles_preserve_party_resources_words_and_pending_pulses() {
    let mut req = scenario();
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    req.relative_duration_us = 44_000_000;
    req.actions.push(normal(1_735_339));
    req.actions.extend([
        switch(40_000_000, 1),
        skia_skill(40_000_001),
        switch(41_335_874, 0),
        normal(41_735_340),
    ]);
    let report = run(&req);
    assert_eq!(report["hits_settled"], 28); // each cycle six Skia hits, six assist recipients, two Normal recipients
    assert_eq!(report["total_damage_exact"], json!("7500/1"));
    let cang = &report["final_actor_states"][0];
    assert_eq!(cang["hp"], 9409);
    assert_eq!(cang["granted_energy"], 24);
    assert_eq!(cang["charge_remainder"], json!("397/500"));
    assert_eq!(cang["cycle"]["granted"], 30);
    assert_eq!(cang["cycle"]["carry"], json!("171/500"));
}

#[test]
fn special_entry_uses_invested_normal_without_inventing_a_nonadjacent_reaction() {
    for (level, numerator, denominator) in [(1, 405, 1), (8, 3471, 5), (9, 3747, 5)] {
        let mut req = scenario();
        req.actors[0].talent_levels = Some([level, 1, 1]);
        let report = run(&req);
        assert!(
            pulses(&report).iter().all(
                |hit| hit["non_crit_damage"] == json!(format!("{}/{}", numerator, denominator))
            )
        );
    }
    let mut req = scenario();
    let id = crate::HypothesisId::CombatEsperCycleReactionsV1;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    req.actors[1].initial_cycle = 100;
    let report = run(&req);
    assert_eq!(pulses(&report).len(), 3);
    assert!(report["final_actor_states"][1]["cycle"].is_null()); // Lakshana/Incantation is nonadjacent, even at full Cycle
    let mut req = scenario();
    req.model_assumptions
        .remove(crate::HypothesisId::BaicangSkiaEntryV1.as_str());
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("UNSUPPORTED_BAICANG_SKIA_ENTRY")
    );
}

#[test]
fn a_supported_pair_reports_spending_an_authored_full_pool_before_any_accrual() {
    let mut req = scenario();
    let id = crate::HypothesisId::CombatEsperCycleReactionsV1;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    req.active_actor_index = 0;
    req.actors[0].initial_cycle = 100;
    req.actors[1].character_id = "character:1004".into(); // Incantation/Chaos admits Scorch
    req.actions = vec![switch(0, 1)];
    let report = run(&req);
    let cycle = &report["final_actor_states"][0]["cycle"];
    assert_eq!(cycle["initial"], 100);
    assert_eq!(cycle["spent"], 100);
    assert_eq!(cycle["current"], 0);
    assert_eq!(cycle["granted"], 0);
    assert_eq!(cycle["conserved"], true);
}
