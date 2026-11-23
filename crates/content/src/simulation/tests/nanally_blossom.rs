use super::*;

fn request() -> GameScenarioRequest {
    let mut req = super::reactions::request(&[
        (0, "character:1046", 100, 0),
        (1, "character:1019", 100, 600),
        (2, "character:1010", 0, 0),
    ]);
    req.relative_duration_us = 40_000_000;
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].hp_max = 100_000_000;
    req.actors[2].character_level = 80;
    req.actors[2].character_max_level = Some(80);
    req.actors[2].awakening = Default::default();
    req.actions = vec![super::reactions::switch(0, 1)];
    req
}

fn run(req: &GameScenarioRequest) -> GameScenarioReport {
    let report = run_game_scenario_report(req, &ExecutionLimits::default()).unwrap();
    assert!(
        report.success && report.execution_completed,
        "{:?}",
        report.halt_diagnostic
    );
    report.validate_damage_parentage().unwrap();
    report
}

fn pistils(report: &GameScenarioReport) -> Vec<&GameHitReceipt> {
    report
        .hit_receipts
        .iter()
        .filter(|hit| hit.damage_effect_id == "ntesim.reaction.blossom")
        .collect()
}

#[test]
fn nanally_blossom_off_field_b2_enhances_two_continuing_generations_without_taking_ownership() {
    let mut req = request();
    req.actions.push(super::reactions::switch(15_000_000, 0));
    let report = run(&req);
    let hits = pistils(&report);
    assert_eq!(hits.len(), 20);
    for (index, hit) in hits.iter().enumerate() {
        let birth = if index < 10 { 0 } else { 15_000_000 };
        assert_eq!(hit.time_us, birth + (index as u64 % 10 + 1) * 1_000_000);
        assert_eq!(hit.actor_index, 1); // Mint owns both captured CI600 reactions; Nanally stays off field.
        // Native level80 base9000, Q(CI600)=2, DEF1/2, resistance factor4/5.
        assert_eq!(hit.non_crit_damage, ExactFraction::new(7200, 1).unwrap());
        assert!(!hit.critical);
    }
    assert!(
        report
            .resolved_model_assumptions
            .contains_key(crate::characters::nanally::blossom::MODEL.as_str())
    );
    let replay = serde_json::from_value(serde_json::to_value(&req).unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(report).unwrap(),
        serde_json::to_value(run(&replay)).unwrap()
    );
}

#[test]
fn nanally_blossom_locked_or_defeated_provider_keeps_the_base_schedule() {
    for defeated in [false, true] {
        let mut req = request();
        if defeated {
            req.actors[2].initial_hp = 0;
        } else {
            req.actors[2].character_level = 20;
            req.actors[2].purchased_qte_level = Some(1);
            req.actors[2].character_max_level = Some(20);
        }
        let report = run(&req);
        let hits = pistils(&report);
        assert_eq!(hits.len(), 5);
        assert_eq!(
            hits.iter().map(|hit| hit.time_us).collect::<Vec<_>>(),
            [2, 4, 6, 8, 10].map(|s| s * 1_000_000)
        );
        assert!(
            !report
                .resolved_model_assumptions
                .contains_key(crate::characters::nanally::blossom::MODEL.as_str())
        );
    }
}

#[test]
fn nanally_blossom_provider_defeat_preserves_existing_buds_and_rechecks_new_births() {
    let mut req = request();
    req.actions.push(super::reactions::switch(15_000_000, 0));
    req.incoming_damage_events.push(GameActorDamageEvent {
        timestamp_us: 4_500_000,
        actor_index: 2,
        delivered_damage: req.actors[2].initial_hp,
    });
    let model = super::super::events::INCOMING_DAMAGE_HYPOTHESIS_ID;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    let report = run(&req);
    assert_eq!(
        pistils(&report)
            .iter()
            .map(|hit| hit.time_us)
            .collect::<Vec<_>>(),
        (1..=10)
            .map(|s| s * 1_000_000)
            .chain([17, 19, 21, 23, 25].map(|s| s * 1_000_000))
            .collect::<Vec<_>>()
    );
}

#[test]
fn nanally_blossom_party_cap_still_replaces_oldest_pending_emissions() {
    let mut req = request();
    let mut edgar = req.actors[0].clone();
    edgar.slot = 3;
    edgar.character_id = "character:1021".into();
    req.actors.push(edgar);
    req.actors[2].initial_cycle = 100;
    req.actions = vec![
        super::reactions::switch(0, 1),
        super::reactions::switch(2_000_000, 3),
        super::reactions::switch(4_000_000, 2),
        super::reactions::switch(6_000_000, 0),
    ];
    let report = run(&req);
    // Ordinary switch admission replaces the oldest before its equal-time ReactionOrBreak emission.
    // The first five hits survive; three later Buds each finish ten.
    assert_eq!(pistils(&report).len(), 5 + 3 * 10);
}

#[test]
fn nanally_blossom_profile_combines_with_jiuyuan_bud_count_without_replacing_its_owner() {
    let mut req = request();
    let mut jiuyuan = req.actors[0].clone();
    jiuyuan.slot = 3;
    jiuyuan.character_id = "character:1055".into();
    jiuyuan.awakening = ntesim_data::AwakeningSelection::selected([2]).unwrap();
    req.actors.push(jiuyuan);
    let report = run(&req);
    let hits = pistils(&report);
    assert_eq!(hits.len(), 20); // Jiuyuan creates two Buds, Nanally supplies ten emissions each.
    for second in 1..=10 {
        assert_eq!(
            hits.iter()
                .filter(|hit| hit.time_us == second * 1_000_000)
                .count(),
            2
        );
    }
    assert!(hits.iter().all(|hit| hit.actor_index == 1));
}
