use super::source_causality::assert_native_replay;
use super::*;

#[test]
fn report_contributions_reconcile_overkill_exact_dps_and_off_field_ownership() {
    let source = "options duration=33 crit=fixed;
        jiuyuan char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0;
        target hp=1 def=0 resist=0;
        active jiuyuan; jiuyuan skill; mint attack;";
    let result = PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap();
    let analysis = result.report.damage_analysis().unwrap();
    assert_eq!(analysis.total.exact, result.report.total_damage_exact);
    assert_eq!(analysis.total.hits, result.report.hit_receipts.len());
    assert!(analysis.total.overkill > 0);
    assert_eq!(
        analysis.total.exact.numerator,
        analysis.total.applied_hp_loss + analysis.total.overkill
    );
    assert_eq!(
        analysis.total.dps,
        Some(
            analysis
                .total
                .exact
                .div(ExactFraction::new(33, 1).unwrap())
                .unwrap()
        )
    );
    assert!(
        analysis
            .contributions
            .iter()
            .any(|c| c.actor_index == 0 && !c.on_field)
    );
    assert_eq!(
        analysis
            .contributions
            .iter()
            .map(|c| c.totals.applied_hp_loss)
            .sum::<u128>(),
        analysis.total.applied_hp_loss
    );
    result.report.validate_state_timeline().unwrap();
    assert_native_replay(&result);

    let mut invalid = result.report;
    invalid.hit_receipts[0].applied_hp_loss = invalid.hit_receipts[0].rounded_damage + 1;
    assert!(invalid.damage_analysis().is_err());
}

#[test]
fn report_healing_and_resource_frames_retain_exact_settlements_and_failed_prefixes() {
    let source = "options duration=32 crit=fixed;
        edgar char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_hp%=0.99;
        target hp=100000000 def=0 resist=0;
        active edgar; edgar skill; wait_until(20); edgar skill;";
    let mut result = PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(1)
        .unwrap();
    assert!(!result.report.healing_receipts.is_empty());
    let hp_max = result.report.final_actor_states[0].hp_max;
    assert_eq!(
        result.report.healing_receipts[0].gross,
        (300_000 + 14 * hp_max + 500) / 1000
    );
    assert!(result.report.healing_receipts.iter().any(|h| h.applied > 0));
    assert!(result.report.healing_receipts.iter().any(|h| h.excess > 0));
    for heal in &result.report.healing_receipts {
        assert_eq!(heal.gross, heal.applied + heal.excess);
        assert_eq!(heal.hp_after - heal.hp_before, heal.applied);
        assert_eq!(heal.source_actor_index, 0);
    }
    assert!(result.report.state_timeline.len() > 2);
    assert_native_replay(&result);
    result.report.success = false;
    result.report.execution_completed = false;
    // Partial damage remains reportable, but is never promoted to completed DPS.
    let analysis = result.report.damage_analysis().unwrap();
    assert!(analysis.total.hits > 0);
    assert!(analysis.total.dps.is_none());
    result.report.state_timeline[1].hits_end = usize::MAX;
    assert!(result.report.validate_state_timeline().is_err());
}

#[test]
fn report_exact_aggregation_and_json_retain_totals_above_u64() {
    let source = "options duration=3; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000 def=0 resist=0; target hp=100000 def=0 resist=0;
        active mint; mint attack;";
    let mut report = PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(1)
        .unwrap()
        .report;
    // Report arithmetic fixture: two independently rounded fractions, each
    // representable in a packet; the aggregate exceeds u64 and IEEE-754 integers.
    let amount = u64::MAX - 2;
    let mut hit = report.hit_receipts[0].clone();
    hit.parent_hit_index = None;
    hit.damage = ExactFraction::new(u128::from(amount) * 3 + 1, 3).unwrap();
    hit.rounded_damage = amount;
    hit.applied_hp_loss = amount;
    report.hit_receipts = vec![hit.clone(), hit];
    report.hit_receipts[1].target_index = 1;
    report.hits_settled = 2;
    report.total_damage_exact = ExactFraction::new(u128::from(amount) * 2, 1).unwrap();
    report.total_damage_dealt = report.total_damage_exact.to_f64();
    for target in &mut report.state_timeline[0].targets {
        target.hp_current = amount;
    }
    for target in &mut report.final_target_states {
        target.hp_current = 0;
    }
    let analysis = report.damage_analysis().unwrap();
    assert_eq!(analysis.total.exact.numerator, u128::from(amount) * 2);
    assert_eq!(
        analysis.total.exact,
        ExactFraction::new(u128::from(amount) * 2, 1).unwrap()
    );
    let bytes = serde_json::to_vec(&analysis).unwrap();
    let decoded: crate::simulation::analysis::DamageAnalysis =
        serde_json::from_slice(&bytes).unwrap();
    assert_eq!(decoded, analysis);
}

#[test]
fn repeated_casts_deduplicate_catalogs_and_group_command_indices() {
    let source = "options duration=25 crit=fixed;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0;
        active mint; mint skill; wait(8); mint skill; wait(8); mint skill;";
    let result = PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap();
    let owned = &result.report;
    assert!(owned.execution_completed);
    let prepared = PreparedGameScenario::new(&result.request, &Default::default()).unwrap();
    let borrowed = prepared.run_borrowed(42).unwrap();
    let owned_json = serde_json::to_value(owned).unwrap();
    let borrowed_json = serde_json::to_value(&borrowed).unwrap();
    assert_eq!(owned_json, borrowed_json);
    assert_eq!(owned.action_metadata_ledger.len(), 1);
    assert_eq!(
        owned.action_metadata_ledger[0].command_indices,
        vec![0, 2, 4]
    );
    assert_eq!(owned.metadata_catalog.len(), 1);
    assert!(owned.hit_receipts.len() > 1);
    assert!(owned.hit_receipts.iter().all(|hit| hit.metadata_id == 0));
    assert_eq!(owned.talent_projection_receipts.len(), 1);
    assert_eq!(
        owned.talent_projection_receipts[0]["command_indices"],
        serde_json::json!([0, 2, 4])
    );
    assert!(owned.evidence_catalog.len() <= 3);
}

#[test]
fn delta_timeline_and_buff_catalog_reject_tampering_and_round_trip_null_clears() {
    let source = include_str!("../../../../../presets/studio/s5.nte");
    let program =
        PreparedSourceScenario::new(source, Default::default(), &Default::default()).unwrap();
    let base = program.run(program.seed()).unwrap().report;
    base.validate_state_timeline().unwrap();

    // JSON round-trip preserves delta_option Some(None) ("null") and full timeline parity.
    let mut clearance_actor = base.final_actor_states[0].clone();
    clearance_actor.charge_remainder = Some(ExactFraction::new(1, 2).unwrap());
    let mut cleared_actor = clearance_actor.clone();
    cleared_actor.charge_remainder = None;
    let clear_delta =
        crate::simulation::GameActorStateDelta::diff(&clearance_actor, &cleared_actor)
            .unwrap()
            .expect("clearing charge_remainder must emit a delta");
    assert_eq!(clear_delta.charge_remainder, Some(None));
    let clear_json = serde_json::to_string(&clear_delta).unwrap();
    assert!(clear_json.contains("\"charge_remainder\":null"));
    let decoded_delta: crate::simulation::GameActorStateDelta =
        serde_json::from_str(&clear_json).unwrap();
    assert_eq!(decoded_delta.charge_remainder, Some(None));
    let mut folded = clearance_actor.clone();
    decoded_delta.apply_to(&mut folded).unwrap();
    assert_eq!(folded.charge_remainder, None);

    // Omitted field deserializes as None (retain existing value), distinct from JSON null.
    let omit_delta: crate::simulation::GameActorStateDelta =
        serde_json::from_str("{\"slot\":0}").unwrap();
    assert_eq!(omit_delta.charge_remainder, None);
    assert!(!omit_delta.has_changes());
    let mut kept = clearance_actor.clone();
    omit_delta.apply_to(&mut kept).unwrap();
    assert_eq!(
        kept.charge_remainder,
        Some(ExactFraction::new(1, 2).unwrap())
    );

    // Tampering with buff_catalog fails validation.
    let mut bad_catalog = base.clone();
    bad_catalog.buff_catalog[0].id.clear();
    assert!(bad_catalog.validate_state_timeline().is_err());

    // Tampering with intermediate frame: snapshot buffs on Event frame is rejected.
    let mut event_with_snapshot = base.clone();
    event_with_snapshot.state_timeline[1].buffs = Some(vec![]);
    assert!(event_with_snapshot.validate_state_timeline().is_err());

    // Tampering with buff_diff: empty buff_diff is rejected.
    let mut empty_diff = base.clone();
    empty_diff.state_timeline[1].buff_diff = Some(crate::simulation::BuffTimelineDiff {
        upsert: vec![],
        remove: vec![],
    });
    assert!(empty_diff.validate_state_timeline().is_err());

    // Tampering with buff_diff: removing an inactive instance_id is rejected.
    let mut bad_remove = base.clone();
    bad_remove.state_timeline[1].buff_diff = Some(crate::simulation::BuffTimelineDiff {
        upsert: vec![],
        remove: vec![999_999],
    });
    assert!(bad_remove.validate_state_timeline().is_err());

    // Tampering with buff_diff: upsert and remove of the same instance_id is rejected.
    let mut conflicting_diff = base.clone();
    conflicting_diff.state_timeline[1].buff_diff = Some(crate::simulation::BuffTimelineDiff {
        upsert: vec![crate::simulation::ActiveBuffRef {
            instance_id: 999_999,
            buff_def_id: 0,
            stacks: 1,
            duration: ntesim_core::buffs::dispatch::BuffDuration::Permanent,
        }],
        remove: vec![999_999],
    });
    assert!(conflicting_diff.validate_state_timeline().is_err());

    // Tampering with intermediate actor delta fails final reconciliation.
    let frame_idx = base.state_timeline[1..base.state_timeline.len() - 1]
        .iter()
        .enumerate()
        .rfind(|(_, f)| f.actors.iter().any(|a| a.energy.is_some()))
        .map(|(idx, _)| idx + 1)
        .expect("s5 emits an intermediate actor energy delta");
    let mut bad_actor_delta = base.clone();
    let delta = bad_actor_delta.state_timeline[frame_idx]
        .actors
        .iter_mut()
        .find(|a| a.energy.is_some())
        .unwrap();
    delta.energy = Some(delta.energy.unwrap() + 1);
    assert!(bad_actor_delta.validate_state_timeline().is_err());
}

#[test]
fn primary_damage_sums_rounded_hits_and_keeps_hp_loss_separate() {
    let source = "options duration=3; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000 def=0 resist=0; target hp=100000 def=0 resist=0;
        active mint; mint attack;";
    let template = PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(1)
        .unwrap()
        .report;
    for (numerator, denominator, count, hp, expected_damage, expected_loss, expected_overkill) in [
        (502, 5, 2, 1000, 200, 200, 0),
        (201, 2, 2, 1000, 202, 202, 0),
        (100, 1, 1, 30, 100, 30, 70),
    ] {
        let mut report = template.clone();
        let mut hit = report.hit_receipts[0].clone();
        hit.parent_hit_index = None;
        hit.damage = ExactFraction::new(numerator, denominator).unwrap();
        hit.rounded_damage = if numerator == 201 { 101 } else { 100 };
        hit.applied_hp_loss = hp.min(hit.rounded_damage);
        report.hit_receipts = (0..count)
            .map(|target| {
                let mut copy = hit.clone();
                copy.target_index = target;
                copy
            })
            .collect();
        report.hits_settled = count;
        report.total_damage_exact = ExactFraction::new(expected_damage, 1).unwrap();
        report.total_damage_dealt = expected_damage as f64;
        for (index, target) in report.state_timeline[0].targets.iter_mut().enumerate() {
            target.hp_current = hp;
            report.final_target_states[index].hp_current = hp
                - if index < count {
                    hit.applied_hp_loss
                } else {
                    0
                };
        }
        let analysis = report.damage_analysis().unwrap();
        assert_eq!(
            analysis.total.exact,
            ExactFraction::new(expected_damage, 1).unwrap()
        );
        assert_eq!(analysis.total.applied_hp_loss, expected_loss);
        assert_eq!(analysis.total.overkill, expected_overkill);
        assert_eq!(
            analysis.total.dps,
            Some(ExactFraction::new(expected_damage, 3).unwrap())
        );
        assert_eq!(
            analysis
                .contributions
                .iter()
                .map(|c| c.totals.exact.numerator)
                .sum::<u128>(),
            expected_damage
        );
        assert_eq!(
            report.hit_receipts[0].damage,
            ExactFraction::new(numerator, denominator).unwrap()
        );
    }
}
