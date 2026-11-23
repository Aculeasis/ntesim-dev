use super::*;
use ntesim_content::simulation::{
    CompletedSourceTrial, GameActorShieldSummary, GameActorStateDelta, GameActorStateSummary,
    GameStateBoundary, GameStateFrame, PreparedSourceScenario, SourceTrialOutcome,
};

fn completed_batch(program: &PreparedSourceScenario, seed: u64) -> CompletedSourceTrial {
    match program.run_batch(seed).unwrap() {
        SourceTrialOutcome::Completed(trial) => trial,
        SourceTrialOutcome::Failed(execution) => panic!(
            "batch seed {seed} failed: {:?}; {:?}",
            execution.diagnostic, execution.report.halt_diagnostic
        ),
    }
}

fn program(options: &str, rotation: &str) -> PreparedSourceScenario {
    PreparedSourceScenario::new(
        &format!("options duration=10 iteration=2 seed=42 crit=fixed {options}; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0; target hp=100000000 def=0 resist=0; active mint; {rotation}"),
        Default::default(), &Default::default(),
    ).unwrap()
}

#[test]
fn live_observations_reconcile_damage_energy_actions_time_and_transactional_failure() {
    let program = program("", "repeat 2 { mint attack; wait(1); }");
    let mut collector = Collector::new(program.export().request, 2).unwrap();
    let first = program.run(42).unwrap();
    collector.add(&first.report).unwrap();
    let a = collector.snapshot(1000.).unwrap();
    let damage = first.report.total_damage_dealt;
    assert_eq!(a.quality, first.report.quality);
    let mut transported = Collector::new(program.export().request, 2).unwrap();
    let row = observe(&program.export().request, &first.report, true).unwrap();
    transported.add_observation(row).unwrap();
    assert_eq!(
        serde_json::to_value(transported.snapshot(1000.).unwrap()).unwrap(),
        serde_json::to_value(&a).unwrap()
    );
    let next = program.run(43).unwrap();
    let row = observe(&program.export().request, &next.report, false).unwrap();
    let mut inconsistent = row.clone();
    inconsistent.quality = Quality::Incomplete;
    assert!(transported.add_observation(inconsistent).is_err());
    assert_eq!(transported.completed(), 1);
    transported.add_observation(row).unwrap();
    assert!(a.game_dps_computed);
    assert_eq!(a.metrics.duration.mean, 10.);
    assert_eq!(a.metrics.dps.relative.unwrap().mean, damage / 10.);
    assert_eq!(a.metrics.dps.absolute.unwrap().mean, damage / 10.);
    assert_eq!(a.metrics.hps.relative.unwrap().mean, 0.);
    assert_eq!(a.metrics.hps.absolute.unwrap().mean, 0.);
    assert_eq!(a.metrics.shp.relative.unwrap().mean, 0.);
    assert_eq!(a.metrics.shp.absolute.unwrap().mean, 0.);
    assert!(
        !collector.samples()[0]
            .shp
            .relative
            .unwrap()
            .is_sign_negative()
    );
    assert!(
        !collector.samples()[0]
            .shp
            .absolute
            .unwrap()
            .is_sign_negative()
    );
    assert_eq!(a.actors[0].field.relative.as_ref().unwrap().mean, 10.);
    assert_eq!(a.actors[0].field.absolute.as_ref().unwrap().mean, 10.);
    assert_eq!(a.actors[0].actions["attack"].mean, 2.);
    assert!((a.sources.iter().map(|s| s.damage.mean).sum::<f64>() - damage).abs() < 1e-8);
    assert!((a.timeline.last().unwrap().cumulative.mean - damage).abs() < 1e-8);
    assert_eq!(
        a.energy_sources.iter().map(|s| s.energy.mean).sum::<f64>(),
        first.report.final_actor_states[0].granted_energy as f64
    );
    let mut second = program.run(43).unwrap();
    second.report.trace.retain(|e| e.energy_grants().is_empty());
    assert!(collector.add(&second.report).is_err());
    assert_eq!(collector.completed(), 1);
    let mut invalid_ledger = program.run(43).unwrap();
    invalid_ledger.report.final_actor_states[0].overflow_energy = u64::MAX;
    invalid_ledger
        .report
        .state_timeline
        .last_mut()
        .unwrap()
        .actors = invalid_ledger
        .report
        .final_actor_states
        .iter()
        .map(GameActorStateDelta::from_summary)
        .collect();
    assert!(collector.add(&invalid_ledger.report).is_err());
    assert_eq!(collector.completed(), 1);
    assert_eq!(
        collector.snapshot(1000.).unwrap().metrics.damage.mean,
        damage
    );
    collector.add(&program.run(43).unwrap().report).unwrap();
    assert_eq!(
        collector
            .snapshot(1000.)
            .unwrap()
            .metrics
            .dps
            .relative
            .unwrap()
            .std,
        0.
    );
    assert_eq!(
        collector
            .snapshot(1000.)
            .unwrap()
            .metrics
            .dps
            .absolute
            .unwrap()
            .std,
        0.
    );
    assert!(collector.add(&first.report).is_err());
}

#[test]
fn relative_damage_rates_keep_absolute_support_and_undefined_seeds_unavailable() {
    let source = "options duration=120 iteration=2 seed=42 crit=fixed; hotori char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_timepiece=120; target hp=1000000000 def=0 resist=0; active hotori; hotori burst;";
    let program =
        PreparedSourceScenario::new(source, Default::default(), &Default::default()).unwrap();
    let first = program.run_sample(42).unwrap();
    let mut collector = Collector::new(program.export().request, 2).unwrap();
    collector.add(&first.report).unwrap();
    let aggregate = collector.snapshot(0.).unwrap();
    assert_eq!(aggregate.metrics.duration.mean, 120.);
    assert_eq!(aggregate.metrics.absolute_duration.mean, 134.183333);
    assert_eq!(
        aggregate.metrics.dps.relative.unwrap().mean,
        first.report.total_damage_dealt / 120.
    );
    assert_eq!(
        aggregate.metrics.dps.absolute.unwrap().mean,
        first.report.total_damage_dealt / 134.183333
    );
    assert_eq!(
        aggregate.actors[0].field.relative.as_ref().unwrap().mean,
        120.
    );
    assert_eq!(
        aggregate.actors[0].field.absolute.as_ref().unwrap().mean,
        134.183333
    );

    let mut row = observe_batch(completed_batch(&program, 43));
    // Independent scalar oracle: 120000 / 120 relative seconds = 1000 DPS,
    // despite a 140-second absolute run. Different seed rates aggregate first.
    row.sample.damage = 120000.;
    row.sample.dps = ntesim_contracts::TimeBasisPair {
        relative: Some(1000.),
        absolute: Some(120000. / 140.),
    };
    row.sample.absolute_elapsed_us = 140_000_000;
    row.field_times[0].absolute = Some(140_000_000);
    row.bins.resize(140, 0);
    row.bins.fill(0);
    row.bins[135] = 120000;
    row.actors[0].insert("damage".into(), 120000.);
    row.targets[0] = 120000.;
    collector.add_observation(row).unwrap();
    let aggregate = collector.snapshot(0.).unwrap();
    assert_eq!(
        aggregate.metrics.dps.relative.unwrap().mean,
        (first.report.total_damage_dealt / 120. + 1000.) / 2.
    );
    assert_eq!(
        aggregate.metrics.dps.absolute.unwrap().mean,
        (first.report.total_damage_dealt / 134.183333 + 120000. / 140.) / 2.
    );
    assert_eq!(aggregate.timeline.last().unwrap().time, 140.);
    assert!(
        aggregate
            .timeline
            .iter()
            .any(|bin| bin.time == 136. && bin.dps.mean >= 60000.)
    );
    assert!(
        (aggregate.timeline.last().unwrap().cumulative.mean - aggregate.metrics.damage.mean).abs()
            < 1e-8
    );

    let early = source.replace("hp=1000000000", "hp=1");
    let early =
        PreparedSourceScenario::new(&early, Default::default(), &Default::default()).unwrap();
    let zero = early.run_sample(42).unwrap();
    assert!(zero.report.success && zero.report.execution_completed);
    assert!(crate::trials::source::completed(&zero));
    assert!(!crate::trials::source::eligible(&zero));
    let mut collector = Collector::new(early.export().request, 2).unwrap();
    collector.add(&zero.report).unwrap();
    let mut positive = observe(
        &early.export().request,
        &early.run_sample(43).unwrap().report,
        false,
    )
    .unwrap();
    positive.sample.relative_elapsed_us = 1;
    positive.field_times[0].relative = Some(1);
    positive.sample.dps = ntesim_contracts::TimeBasisPair {
        relative: Some(positive.sample.damage / 0.000001),
        absolute: Some(positive.sample.damage / positive.sample.absolute_seconds()),
    };
    collector.add_observation(positive).unwrap();
    let aggregate = collector.snapshot(0.).unwrap();
    assert_eq!(aggregate.completed, 2);
    assert!(!aggregate.game_dps_computed);
    assert!(aggregate.metrics.dps.relative.is_none());
    assert!(aggregate.metrics.dps.absolute.is_some());
    assert!(
        aggregate.actors[0].dps.relative.is_none() && aggregate.targets[0].dps.relative.is_none()
    );
    assert!(
        aggregate
            .sources
            .iter()
            .all(|source| source.dps.relative.is_none())
    );
    assert!(
        aggregate.actors[0].dps.absolute.is_some() && aggregate.targets[0].dps.absolute.is_some()
    );
    assert!(
        aggregate
            .sources
            .iter()
            .all(|source| source.dps.absolute.is_some())
    );
    assert_eq!(aggregate.metrics.duration.min, 0.);
    assert!(aggregate.metrics.absolute_duration.mean > 0.);
}

#[test]
fn absolute_buckets_promote_on_aligned_dyadic_grids_and_preserve_shorter_totals() {
    let program = program("", "");
    let mut request = program.export().request;
    request.relative_duration_us = 120_000_000;
    let base = observe(
        &program.export().request,
        &program.run(42).unwrap().report,
        false,
    )
    .unwrap();
    let make = |seed, end_us: u64, second: usize, damage| {
        let mut row = base.clone();
        row.sample.seed = seed;
        row.sample.absolute_elapsed_us = end_us;
        row.sample.relative_elapsed_us = 120_000_000;
        row.field_times[0] = ntesim_contracts::TimeBasisPair {
            relative: Some(120_000_000),
            absolute: Some(end_us),
        };
        row.sample.damage = damage;
        row.sample.dps = ntesim_contracts::TimeBasisPair {
            relative: Some(damage / 120.),
            absolute: Some(damage / (end_us as f64 / 1e6)),
        };
        row.bucket_us = 1_000_000;
        while end_us.div_ceil(row.bucket_us) > 240 {
            row.bucket_us *= 2;
        }
        row.bins = vec![0; end_us.div_ceil(row.bucket_us) as usize];
        row.bins[second / (row.bucket_us as usize / 1_000_000)] = damage as u128;
        row.actors[0].insert("damage".into(), damage);
        row.targets[0] = damage;
        row
    };
    let mut collector = Collector::new(request, 3).unwrap();
    collector
        .add_observation(make(42, 140_000_000, 135, 120000.))
        .unwrap();
    collector
        .add_observation(make(43, 260_000_000, 250, 240000.))
        .unwrap();
    collector
        .add_observation(make(44, 600_000_000, 550, 360000.))
        .unwrap();
    let aggregate = collector.snapshot(0.).unwrap();
    assert_eq!(aggregate.bucket_seconds, 4.);
    assert_eq!(aggregate.timeline.len(), 150);
    assert_eq!(aggregate.timeline.last().unwrap().time, 600.);
    assert_eq!(aggregate.timeline.last().unwrap().cumulative.mean, 240000.);
    assert_eq!(aggregate.metrics.dps.relative.as_ref().unwrap().mean, 2000.);
    assert_eq!(aggregate.metrics.absolute_duration.max, 600.);
    let restored = collector.rows.clone();
    let mut reordered = Collector::new(collector.request.clone(), 3).unwrap();
    for row in restored {
        reordered.add_observation(row).unwrap();
    }
    assert_eq!(
        serde_json::to_value(aggregate).unwrap(),
        serde_json::to_value(reordered.snapshot(0.).unwrap()).unwrap()
    );
}

#[test]
fn support_statistics_use_canonical_heals_actual_duration_and_zero_activity() {
    let source = "options duration=10 iteration=2 seed=42 crit=fixed; iroi char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_hp%=0.99; target hp=100000000 def=0 resist=0; active iroi; iroi skill;";
    let program =
        PreparedSourceScenario::new(source, Default::default(), &Default::default()).unwrap();
    let mut collector = Collector::new(program.export().request.clone(), 2).unwrap();
    let first = program.run(42).unwrap();
    assert!(first.report.execution_completed);
    let missing_hp = program.export().request.actors[0].static_stats.hp
        - program.export().request.actors[0].initial_hp;
    assert!(missing_hp > 0);
    assert_eq!(first.report.healing_receipts.len(), 1);
    let heal = &first.report.healing_receipts[0];
    assert_eq!(heal.applied, missing_hp);
    assert!(heal.excess > 0);
    collector.add(&first.report).unwrap();
    let snapshot = collector.snapshot(0.).unwrap();
    assert_eq!(snapshot.metrics.healing.mean, missing_hp as f64);
    assert_eq!(snapshot.metrics.gross_healing.mean, heal.gross as f64);
    assert_eq!(
        snapshot.metrics.excess_healing.mean,
        (heal.gross - missing_hp) as f64
    );
    assert_eq!(
        snapshot.metrics.hps.relative.unwrap().mean,
        heal.gross as f64 / 10.
    );
    assert_eq!(
        snapshot.metrics.hps.absolute.unwrap().mean,
        heal.gross as f64 / 10.
    );
    assert_eq!(
        collector.samples()[0].hps.relative.unwrap(),
        heal.gross as f64 / 10.
    );
    assert_eq!(
        collector.samples()[0].hps.absolute.unwrap(),
        heal.gross as f64 / 10.
    );
    assert_eq!(snapshot.metrics.shp.relative.unwrap().mean, 0.);
    assert_eq!(snapshot.metrics.shp.absolute.unwrap().mean, 0.);

    // Full-health overheal: gross healing contributes to HPS even with zero applied healing.
    let full_source = source.replace("start_hp%=0.99", "start_hp%=1.0");
    let full_program =
        PreparedSourceScenario::new(&full_source, Default::default(), &Default::default()).unwrap();
    let mut full_collector = Collector::new(full_program.export().request, 2).unwrap();
    let full_run = full_program.run(42).unwrap();
    assert_eq!(full_run.report.healing_receipts.len(), 1);
    let full_heal = &full_run.report.healing_receipts[0];
    assert_eq!(full_heal.applied, 0);
    assert_eq!(full_heal.excess, full_heal.gross);
    full_collector.add(&full_run.report).unwrap();
    let full_snapshot = full_collector.snapshot(0.).unwrap();
    assert_eq!(full_snapshot.metrics.healing.mean, 0.);
    assert_eq!(
        full_snapshot.metrics.excess_healing.mean,
        full_heal.gross as f64
    );
    assert_eq!(
        full_snapshot.metrics.gross_healing.mean,
        full_heal.gross as f64
    );
    assert_eq!(
        full_snapshot.metrics.hps.relative.unwrap().mean,
        full_heal.gross as f64 / 10.
    );
    assert_eq!(
        full_snapshot.metrics.hps.absolute.unwrap().mean,
        full_heal.gross as f64 / 10.
    );
    assert_eq!(
        full_collector.samples()[0].hps.relative.unwrap(),
        full_heal.gross as f64 / 10.
    );
    assert_eq!(
        full_collector.samples()[0].hps.absolute.unwrap(),
        full_heal.gross as f64 / 10.
    );

    // A separate no-healing program must still supply all support metrics as actual zeros.
    let no_heal = PreparedSourceScenario::new(
        &source.replace("iroi skill;", "wait(1);"),
        Default::default(),
        &Default::default(),
    )
    .unwrap();
    let mut collector = Collector::new(no_heal.export().request, 2).unwrap();
    collector.add(&no_heal.run(42).unwrap().report).unwrap();
    let snapshot = collector.snapshot(0.).unwrap();
    assert_eq!(snapshot.metrics.hps.relative.as_ref().unwrap().mean, 0.);
    assert_eq!(snapshot.metrics.hps.relative.as_ref().unwrap().min, 0.);
    assert_eq!(snapshot.metrics.hps.absolute.as_ref().unwrap().mean, 0.);
    assert_eq!(snapshot.metrics.gross_healing.mean, 0.);

    let shorter = PreparedSourceScenario::new(
        &source.replace("duration=10", "duration=5"),
        Default::default(),
        &Default::default(),
    )
    .unwrap();
    let mut collector = Collector::new(shorter.export().request, 2).unwrap();
    collector.add(&shorter.run(42).unwrap().report).unwrap();
    assert_eq!(
        collector
            .snapshot(0.)
            .unwrap()
            .metrics
            .hps
            .relative
            .as_ref()
            .unwrap()
            .mean,
        heal.gross as f64 / 5.
    );
    assert_eq!(
        collector
            .snapshot(0.)
            .unwrap()
            .metrics
            .hps
            .absolute
            .as_ref()
            .unwrap()
            .mean,
        heal.gross as f64 / 5.
    );
}

#[test]
fn shield_statistics_count_each_delivery_but_exclude_other_prevention() {
    use ntesim_content::simulation::{GameActorDamageReceipt, GameShieldAbsorptionReceipt};
    let mut root = GameActorDamageReceipt {
        defense_mitigation: None,
        event_index: 0,
        time_us: 0,
        actor_index: 0,
        delivered_damage: 1000,
        redirected: vec![],
        prevented_hp_loss: 200,
        absorbed_by_shields: vec![GameShieldAbsorptionReceipt {
            shield: "first".into(),
            amount: 300,
        }],
        absorbed_by_effects: vec![],
        hp_before: 1000,
        actual_hp_loss: 500,
        overflow: 0,
        hp_after: 500,
        defeated: false,
    };
    let mut child = root.clone();
    child.actor_index = 1;
    child.absorbed_by_shields[0].amount = 70;
    root.redirected.push(child);
    assert_eq!(super::observe::shield_absorption(&[root]), 370.);
    assert_eq!(super::observe::shield_absorption(&[]), 0.);

    let source = include_str!("../../../../../examples/incoming-shield-cycles.nte");
    let program =
        PreparedSourceScenario::new(source, Default::default(), &Default::default()).unwrap();
    let execution = program.run(42).unwrap();
    let mut collector = Collector::new(program.export().request, 1).unwrap();
    collector.add(&execution.report).unwrap();
    let snapshot = collector.snapshot(0.).unwrap();
    // This fixture delivers 8,000 damage without immunity, redirection or overkill.
    let hp_loss = execution
        .report
        .incoming_damage_receipts
        .iter()
        .map(|receipt| receipt.actual_hp_loss)
        .sum::<u64>();
    let absorbed = 8000. - hp_loss as f64;
    assert!(absorbed > 0.);
    assert_eq!(snapshot.metrics.shield_absorbed.mean, absorbed);
    assert!(snapshot.metrics.shp.relative.as_ref().unwrap().mean > 0.);
    assert_eq!(
        collector.samples()[0].shp.relative.unwrap(),
        snapshot.metrics.shp.relative.as_ref().unwrap().mean
    );

    // Adler produces positive SHP without any incoming damage events.
    let adler_source = include_str!("../../../../../examples/adler-continuing-energy.nte");
    let adler_program =
        PreparedSourceScenario::new(adler_source, Default::default(), &Default::default()).unwrap();
    let adler_execution = adler_program.run(42).unwrap();
    assert!(adler_execution.report.incoming_damage_receipts.is_empty());
    let mut adler_collector = Collector::new(adler_program.export().request, 1).unwrap();
    adler_collector.add(&adler_execution.report).unwrap();
    let adler_snapshot = adler_collector.snapshot(0.).unwrap();
    assert_eq!(adler_snapshot.metrics.shield_absorbed.mean, 0.);
    let adler_shp = adler_snapshot.metrics.shp.relative.as_ref().unwrap().mean;
    assert!(adler_shp > 0.);
    assert_eq!(
        adler_collector.samples()[0].shp.relative.unwrap(),
        adler_shp
    );
    assert_eq!(
        adler_snapshot.metrics.shp.relative.as_ref().unwrap().min,
        adler_shp
    );
    assert_eq!(
        adler_snapshot.metrics.shp.relative.as_ref().unwrap().max,
        adler_shp
    );
    assert_eq!(
        adler_snapshot.metrics.shp.relative.as_ref().unwrap().std,
        0.
    );
}

#[test]
fn victory_uses_real_elapsed_duration_and_failed_runs_never_enter_statistics() {
    let program = program("stop=victory", "mint attack;");
    let mut input = program.export();
    input.input.source = input.input.source.replace("hp=100000000", "hp=1");
    let program =
        PreparedSourceScenario::new(&input.input.source, Default::default(), &Default::default())
            .unwrap();
    let execution = program.run(42).unwrap();
    assert!(execution.report.absolute_elapsed_us < execution.report.relative_duration_us);
    let mut collector = Collector::new(program.export().request, 2).unwrap();
    collector.add(&execution.report).unwrap();
    let a = collector.snapshot(0.).unwrap();
    assert_eq!(
        a.metrics.duration.mean,
        execution.report.absolute_elapsed_us as f64 / 1e6
    );
    let dps_mean = a.metrics.dps.relative.as_ref().unwrap().mean;
    assert_eq!(
        dps_mean,
        execution.report.total_damage_dealt / a.metrics.duration.mean
    );
    assert_eq!(a.actors[0].dps.relative.as_ref().unwrap().mean, dps_mean);
    assert_eq!(a.targets[0].dps.relative.as_ref().unwrap().mean, dps_mean);
    assert!(
        (a.sources
            .iter()
            .map(|s| s.dps.relative.as_ref().unwrap().mean)
            .sum::<f64>()
            - dps_mean)
            .abs()
            < 1e-8
    );
    let failed = super::tests::program("", "mint burst;");
    let mut collector = Collector::new(failed.export().request, 2).unwrap();
    assert!(collector.add(&failed.run(42).unwrap().report).is_err());
    assert_eq!(collector.completed(), 0);
    assert!(collector.snapshot(0.).is_err());
}

#[test]
fn timeline_intervals_cover_fractional_and_long_horizons_without_zero_width_bins() {
    let source = program("", "mint attack;").export().input.source;
    for (duration, bins) in [("0.1", 1), ("20.5", 21), ("260", 130)] {
        let source = source.replace("duration=10", &format!("duration={duration}"));
        let source = if duration == "0.1" {
            source.replace("mint attack;", "wait(0.01);")
        } else {
            source
        };
        let program =
            PreparedSourceScenario::new(&source, Default::default(), &Default::default()).unwrap();
        let execution = program.run(42).unwrap();
        let mut collector = Collector::new(program.export().request, 2).unwrap();
        collector.add(&execution.report).unwrap();
        let a = collector.snapshot(0.).unwrap();
        assert_eq!(a.timeline.len(), bins);
        let mut previous = 0.;
        let mut damage = 0.;
        for bin in a.timeline {
            assert!(bin.time > previous);
            damage += bin.dps.mean * (bin.time - previous);
            previous = bin.time;
        }
        assert_eq!(previous, duration.parse::<f64>().unwrap());
        assert!((damage - execution.report.total_damage_dealt).abs() < 1e-7);
    }
}

#[test]
fn victory_timeline_bounds_to_elapsed_duration_without_full_horizon_padding() {
    let source = "options stop=victory crit=fixed; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=1000 def=0 resist=0; active mint; while .time < .duration { mint attack; }";
    let program =
        PreparedSourceScenario::new(source, Default::default(), &Default::default()).unwrap();
    let execution = program.run(42).unwrap();
    let mut collector = Collector::new(program.export().request, 1).unwrap();
    collector.add(&execution.report).unwrap();
    let a = collector.snapshot(0.).unwrap();
    let elapsed = execution.report.absolute_elapsed_us as f64 / 1e6;
    assert!(elapsed < 100.0);
    let last_bin_time = a.timeline.last().unwrap().time;
    assert_eq!(last_bin_time, elapsed);
    assert!(a.timeline.len() < 240);
}

fn shield_fixture() -> (
    GameScenarioRequest,
    GameScenarioReport,
    GameActorStateSummary,
) {
    let p = program("", "wait(1);");
    let base_req = p.export().request;
    let base_rep = p.run(42).unwrap().report;
    let base_actor = base_rep.final_actor_states[0].clone();
    (base_req, base_rep, base_actor)
}

fn make_actor(
    base: &GameActorStateSummary,
    slot: usize,
    hp: u64,
    shields: Vec<GameActorShieldSummary>,
) -> GameActorStateDelta {
    let mut a = base.clone();
    a.slot = slot;
    a.hp = hp;
    a.active_shields = shields;
    GameActorStateDelta::from_summary(&a)
}

fn make_frame(
    time_us: u64,
    boundary: GameStateBoundary,
    active_actor: usize,
    actors: Vec<GameActorStateDelta>,
) -> GameStateFrame {
    GameStateFrame {
        time_us,
        boundary,
        active_actor_index: active_actor,
        actors,
        targets: vec![],
        buffs: None,
        buff_diff: None,
        stamina: None,
        timers: Some(Vec::new()),
        hits_end: 0,
        healing_end: 0,
        trace_end: 0,
    }
}

#[test]
fn available_shield_hp_half_uptime_idle_expiry_and_early_completion() {
    let (base_req, base_rep, base_actor) = shield_fixture();
    let mut req = base_req.clone();
    req.actors = vec![req.actors[0].clone()];
    let mut rep = base_rep.clone();

    // 1. 5000 HP half-uptime: 10,000 HP shield for 5s of a 10s trial
    rep.absolute_elapsed_us = 10_000_000;
    rep.relative_elapsed_us = 10_000_000;
    let s1 = GameActorShieldSummary {
        clock_basis: ntesim_core::kernel::timing::ClockDomain::World,
        id: "s1".into(),
        source: "test".into(),
        remaining_hp: 10_000,
        max_hp: 10_000,
        expires_at_us: 5_000_000,
    };
    rep.state_timeline = vec![
        make_frame(
            0,
            GameStateBoundary::Initial,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![s1.clone()])],
        ),
        make_frame(
            5_000_000,
            GameStateBoundary::Event { ordinal: 1 },
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![])],
        ),
        make_frame(
            10_000_000,
            GameStateBoundary::Final,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![])],
        ),
    ];
    let shp = super::observe::calculate_available_shield_hp(&req, &rep).unwrap();
    assert_eq!(shp.relative, Some(5000.0));
    assert_eq!(shp.absolute, Some(5000.0));

    // 2. 3000 HP idle-expiry: 10,000 HP shield starting at 2s, expires at 5s, next event at 9s in 10s trial
    let s_idle = GameActorShieldSummary {
        clock_basis: ntesim_core::kernel::timing::ClockDomain::World,
        id: "s_idle".into(),
        source: "test".into(),
        remaining_hp: 10_000,
        max_hp: 10_000,
        expires_at_us: 5_000_000,
    };
    rep.state_timeline = vec![
        make_frame(
            0,
            GameStateBoundary::Initial,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![])],
        ),
        make_frame(
            2_000_000,
            GameStateBoundary::Event { ordinal: 1 },
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![s_idle])],
        ),
        make_frame(
            9_000_000,
            GameStateBoundary::Event { ordinal: 2 },
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![])],
        ),
        make_frame(
            10_000_000,
            GameStateBoundary::Final,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![])],
        ),
    ];
    let shp = super::observe::calculate_available_shield_hp(&req, &rep).unwrap();
    assert_eq!(shp.relative, Some(3000.0));
    assert_eq!(shp.absolute, Some(3000.0));

    // 3. Early completion: 6s elapsed, shield expires at 10s -> clips at 6s
    rep.absolute_elapsed_us = 6_000_000;
    rep.relative_elapsed_us = 6_000_000;
    let s_early = GameActorShieldSummary {
        clock_basis: ntesim_core::kernel::timing::ClockDomain::World,
        id: "s1".into(),
        source: "test".into(),
        remaining_hp: 10_000,
        max_hp: 10_000,
        expires_at_us: 10_000_000,
    };
    rep.state_timeline = vec![
        make_frame(
            0,
            GameStateBoundary::Initial,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![s_early.clone()])],
        ),
        make_frame(
            6_000_000,
            GameStateBoundary::Final,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![s_early])],
        ),
    ];
    let shp = super::observe::calculate_available_shield_hp(&req, &rep).unwrap();
    assert_eq!(shp.relative, Some(10_000.0));
    assert_eq!(shp.absolute, Some(10_000.0));

    // 4. Zero activity: no shields anywhere
    rep.absolute_elapsed_us = 10_000_000;
    rep.relative_elapsed_us = 10_000_000;
    rep.state_timeline = vec![
        make_frame(
            0,
            GameStateBoundary::Initial,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![])],
        ),
        make_frame(
            10_000_000,
            GameStateBoundary::Final,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![])],
        ),
    ];
    let shp = super::observe::calculate_available_shield_hp(&req, &rep).unwrap();
    assert_eq!(shp.relative, Some(0.0));
    assert_eq!(shp.absolute, Some(0.0));
    assert!(!shp.relative.unwrap().is_sign_negative());
    assert!(!shp.absolute.unwrap().is_sign_negative());
}

#[test]
fn available_shield_hp_actor_switches_and_additive_replacement() {
    let (base_req, base_rep, base_actor) = shield_fixture();

    // 1. Actor switch: Actor 0 has 10,000 HP shield active for first 4s of 10s trial, then unshielded Actor 1 for 6s
    let mut req_2 = base_req.clone();
    let mut a0_def = req_2.actors[0].clone();
    a0_def.slot = 0;
    let mut a1_def = req_2.actors[0].clone();
    a1_def.slot = 1;
    a1_def.character_id = "second".into();
    req_2.actors = vec![a0_def, a1_def];
    let s_switch = GameActorShieldSummary {
        clock_basis: ntesim_core::kernel::timing::ClockDomain::World,
        id: "s_switch".into(),
        source: "test".into(),
        remaining_hp: 10_000,
        max_hp: 10_000,
        expires_at_us: 10_000_000,
    };
    let a0_shielded = make_actor(&base_actor, 0, 5000, vec![s_switch]);
    let a1_unshielded = make_actor(&base_actor, 1, 5000, vec![]);
    let mut rep_switch = base_rep.clone();
    rep_switch.absolute_elapsed_us = 10_000_000;
    rep_switch.relative_elapsed_us = 10_000_000;
    rep_switch.state_timeline = vec![
        make_frame(
            0,
            GameStateBoundary::Initial,
            0,
            vec![a0_shielded.clone(), a1_unshielded.clone()],
        ),
        make_frame(
            4_000_000,
            GameStateBoundary::Event { ordinal: 1 },
            1,
            vec![],
        ),
        make_frame(
            10_000_000,
            GameStateBoundary::Final,
            1,
            vec![a0_shielded, a1_unshielded],
        ),
    ];
    let shp = super::observe::calculate_available_shield_hp(&req_2, &rep_switch).unwrap();
    assert_eq!(shp.relative, Some(4000.0));
    assert_eq!(shp.absolute, Some(4000.0));

    // 2. Additive and replaced shields: 3000 + 2000 = 5000 for 5s, replaced 4000 + 2000 = 6000 for 5s
    let mut req = base_req.clone();
    req.actors = vec![req.actors[0].clone()];
    let mut rep = base_rep.clone();
    rep.absolute_elapsed_us = 10_000_000;
    rep.relative_elapsed_us = 10_000_000;
    let s_add1 = GameActorShieldSummary {
        clock_basis: ntesim_core::kernel::timing::ClockDomain::World,
        id: "s1".into(),
        source: "test".into(),
        remaining_hp: 3000,
        max_hp: 3000,
        expires_at_us: 10_000_000,
    };
    let s_add2 = GameActorShieldSummary {
        clock_basis: ntesim_core::kernel::timing::ClockDomain::World,
        id: "s2".into(),
        source: "test".into(),
        remaining_hp: 2000,
        max_hp: 2000,
        expires_at_us: 10_000_000,
    };
    let s_add1_replaced = GameActorShieldSummary {
        clock_basis: ntesim_core::kernel::timing::ClockDomain::World,
        id: "s1".into(),
        source: "test".into(),
        remaining_hp: 4000,
        max_hp: 4000,
        expires_at_us: 10_000_000,
    };
    rep.state_timeline = vec![
        make_frame(
            0,
            GameStateBoundary::Initial,
            0,
            vec![make_actor(
                &base_actor,
                0,
                5000,
                vec![s_add1, s_add2.clone()],
            )],
        ),
        make_frame(
            5_000_000,
            GameStateBoundary::Event { ordinal: 1 },
            0,
            vec![make_actor(
                &base_actor,
                0,
                5000,
                vec![s_add1_replaced, s_add2],
            )],
        ),
        make_frame(
            10_000_000,
            GameStateBoundary::Final,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![])],
        ),
    ];
    let shp = super::observe::calculate_available_shield_hp(&req, &rep).unwrap();
    assert_eq!(shp.relative, Some(5500.0));
    assert_eq!(shp.absolute, Some(5500.0));
}

#[test]
fn available_shield_hp_depletion_defeat_and_equal_time() {
    let (base_req, base_rep, base_actor) = shield_fixture();
    let mut req = base_req.clone();
    req.actors = vec![req.actors[0].clone()];
    let mut rep = base_rep.clone();
    rep.absolute_elapsed_us = 10_000_000;
    rep.relative_elapsed_us = 10_000_000;

    // 1. Depletion and defeat: 6000 HP for 2s (12,000 HP*s), 2000 HP for 4s (8,000 HP*s), defeat at 6s (0 for 4s)
    let s_init = GameActorShieldSummary {
        clock_basis: ntesim_core::kernel::timing::ClockDomain::World,
        id: "s1".into(),
        source: "test".into(),
        remaining_hp: 6000,
        max_hp: 6000,
        expires_at_us: 10_000_000,
    };
    let s_depleted = GameActorShieldSummary {
        clock_basis: ntesim_core::kernel::timing::ClockDomain::World,
        id: "s1".into(),
        source: "test".into(),
        remaining_hp: 2000,
        max_hp: 6000,
        expires_at_us: 10_000_000,
    };
    rep.state_timeline = vec![
        make_frame(
            0,
            GameStateBoundary::Initial,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![s_init])],
        ),
        make_frame(
            2_000_000,
            GameStateBoundary::Event { ordinal: 1 },
            0,
            vec![make_actor(&base_actor, 0, 3000, vec![s_depleted.clone()])],
        ),
        make_frame(
            6_000_000,
            GameStateBoundary::Event { ordinal: 2 },
            0,
            vec![make_actor(&base_actor, 0, 0, vec![s_depleted])],
        ),
        make_frame(
            10_000_000,
            GameStateBoundary::Final,
            0,
            vec![make_actor(&base_actor, 0, 0, vec![])],
        ),
    ];
    let shp = super::observe::calculate_available_shield_hp(&req, &rep).unwrap();
    assert_eq!(shp.relative, Some(2000.0));
    assert_eq!(shp.absolute, Some(2000.0));

    // 2. Equal-time boundaries: frame at 2s updated immediately by another frame at 2s
    let s1 = GameActorShieldSummary {
        clock_basis: ntesim_core::kernel::timing::ClockDomain::World,
        id: "s1".into(),
        source: "test".into(),
        remaining_hp: 10_000,
        max_hp: 10_000,
        expires_at_us: 5_000_000,
    };
    let s_eq1 = GameActorShieldSummary {
        clock_basis: ntesim_core::kernel::timing::ClockDomain::World,
        id: "s1".into(),
        source: "test".into(),
        remaining_hp: 4000,
        max_hp: 10_000,
        expires_at_us: 10_000_000,
    };
    let s_eq2 = GameActorShieldSummary {
        clock_basis: ntesim_core::kernel::timing::ClockDomain::World,
        id: "s1".into(),
        source: "test".into(),
        remaining_hp: 2000,
        max_hp: 10_000,
        expires_at_us: 10_000_000,
    };
    rep.state_timeline = vec![
        make_frame(
            0,
            GameStateBoundary::Initial,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![s1])],
        ),
        make_frame(
            2_000_000,
            GameStateBoundary::Event { ordinal: 1 },
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![s_eq1])],
        ),
        make_frame(
            2_000_000,
            GameStateBoundary::Event { ordinal: 2 },
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![s_eq2])],
        ),
        make_frame(
            10_000_000,
            GameStateBoundary::Final,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![])],
        ),
    ];
    let shp = super::observe::calculate_available_shield_hp(&req, &rep).unwrap();
    assert_eq!(shp.relative, Some(3600.0));
    assert_eq!(shp.absolute, Some(3600.0));
}

fn make_pause(
    start_absolute_us: u64,
    end_absolute_us: u64,
) -> ntesim_core::kernel::timing::OwnedModePause {
    ntesim_core::kernel::timing::OwnedModePause {
        owner: ntesim_core::kernel::timing::PauseOwner {
            owner: 0,
            generation: 0,
        },
        kind: ntesim_core::kernel::timing::ModePauseKind::Cinematic,
        start_absolute_us,
        end_absolute_us,
    }
}

#[test]
fn available_shield_hp_oracle_handles_pauses_mutations_and_zero_relative() {
    let (base_req, base_rep, base_actor) = shield_fixture();
    let mut req = base_req.clone();
    req.actors = vec![req.actors[0].clone()];

    // 1. Pause-only capacity: 20s absolute trial, 10s pause ([5s, 15s]).
    // Shield 10,000 HP exists from 5s to 15s (only during the pause).
    // Absolute: 10,000 HP * 10s = 100,000 HP*s / 20s = 5,000 HP.
    // Relative: 0 unpaused seconds = 0 HP.
    let mut rep = base_rep.clone();
    rep.absolute_elapsed_us = 20_000_000;
    rep.relative_elapsed_us = 10_000_000;
    rep.timer_pauses = Some(vec![make_pause(5_000_000, 15_000_000)]);
    let s_paused = GameActorShieldSummary {
        clock_basis: ntesim_core::kernel::timing::ClockDomain::World,
        id: "s_paused".into(),
        source: "test".into(),
        remaining_hp: 10_000,
        max_hp: 10_000,
        expires_at_us: 15_000_000,
    };
    rep.state_timeline = vec![
        make_frame(
            0,
            GameStateBoundary::Initial,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![])],
        ),
        make_frame(
            5_000_000,
            GameStateBoundary::Event { ordinal: 1 },
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![s_paused])],
        ),
        make_frame(
            15_000_000,
            GameStateBoundary::Event { ordinal: 2 },
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![])],
        ),
        make_frame(
            20_000_000,
            GameStateBoundary::Final,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![])],
        ),
    ];
    let shp = super::observe::calculate_available_shield_hp(&req, &rep).unwrap();
    assert_eq!(shp.absolute, Some(5000.0));
    assert_eq!(shp.relative, Some(0.0));

    // 2. Capacity mutation during pause:
    // From 5s to 10s: 10,000 HP. From 10s to 15s: 6,000 HP. From 15s to 18s (unpaused): 6,000 HP.
    // Absolute: 10,000 * 5s + 6,000 * 5s + 6,000 * 3s = 50,000 + 30,000 + 18,000 = 98,000 HP*s / 20s = 4,900 HP.
    // Relative: unpaused portion is only 15s..18s (3s) with 6,000 HP -> 18,000 HP*s / 10s = 1,800 HP.
    let s_mut1 = GameActorShieldSummary {
        clock_basis: ntesim_core::kernel::timing::ClockDomain::World,
        id: "s_mut".into(),
        source: "test".into(),
        remaining_hp: 10_000,
        max_hp: 10_000,
        expires_at_us: 18_000_000,
    };
    let s_mut2 = GameActorShieldSummary {
        clock_basis: ntesim_core::kernel::timing::ClockDomain::World,
        id: "s_mut".into(),
        source: "test".into(),
        remaining_hp: 6_000,
        max_hp: 10_000,
        expires_at_us: 18_000_000,
    };
    rep.state_timeline = vec![
        make_frame(
            0,
            GameStateBoundary::Initial,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![])],
        ),
        make_frame(
            5_000_000,
            GameStateBoundary::Event { ordinal: 1 },
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![s_mut1])],
        ),
        make_frame(
            10_000_000,
            GameStateBoundary::Event { ordinal: 2 },
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![s_mut2])],
        ),
        make_frame(
            18_000_000,
            GameStateBoundary::Event { ordinal: 3 },
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![])],
        ),
        make_frame(
            20_000_000,
            GameStateBoundary::Final,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![])],
        ),
    ];
    let shp = super::observe::calculate_available_shield_hp(&req, &rep).unwrap();
    assert_eq!(shp.absolute, Some(4900.0));
    assert_eq!(shp.relative, Some(1800.0));

    // 3. Overlapping pauses: [4s, 8s] and [6s, 10s] -> union is [4s, 10s] (6s paused, 14s unpaused in 20s absolute trial).
    // Shield 10,000 HP from 0s to 12s.
    // Overlap with [4s, 10s] is [4s, 10s] (6s paused).
    // Absolute: 10,000 * 12s = 120,000 HP*s / 20s = 6,000 HP.
    // Relative active time: 12s - 6s = 6s unpaused.
    // Relative elapsed: 20s - 6s = 14s.
    // Relative: 10,000 * 6s = 60,000 HP*s / 14s = 60,000 / 14.
    rep.relative_elapsed_us = 14_000_000;
    rep.timer_pauses = Some(vec![
        make_pause(4_000_000, 8_000_000),
        make_pause(6_000_000, 10_000_000),
    ]);
    let s_overlap = GameActorShieldSummary {
        clock_basis: ntesim_core::kernel::timing::ClockDomain::World,
        id: "s_overlap".into(),
        source: "test".into(),
        remaining_hp: 10_000,
        max_hp: 10_000,
        expires_at_us: 12_000_000,
    };
    rep.state_timeline = vec![
        make_frame(
            0,
            GameStateBoundary::Initial,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![s_overlap])],
        ),
        make_frame(
            12_000_000,
            GameStateBoundary::Event { ordinal: 1 },
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![])],
        ),
        make_frame(
            20_000_000,
            GameStateBoundary::Final,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![])],
        ),
    ];
    let shp = super::observe::calculate_available_shield_hp(&req, &rep).unwrap();
    assert_eq!(shp.absolute, Some(6000.0));
    assert_eq!(shp.relative, Some(60_000.0 / 14.0));

    // 4. Switches during pause:
    let mut req_2 = base_req.clone();
    let mut a0_def = req_2.actors[0].clone();
    a0_def.slot = 0;
    let mut a1_def = req_2.actors[0].clone();
    a1_def.slot = 1;
    a1_def.character_id = "second".into();
    req_2.actors = vec![a0_def, a1_def];
    let s_sw = GameActorShieldSummary {
        clock_basis: ntesim_core::kernel::timing::ClockDomain::World,
        id: "s_sw".into(),
        source: "test".into(),
        remaining_hp: 10_000,
        max_hp: 10_000,
        expires_at_us: 20_000_000,
    };
    let a0_sw = make_actor(&base_actor, 0, 5000, vec![s_sw]);
    let a1_sw = make_actor(&base_actor, 1, 5000, vec![]);
    let mut rep_sw = base_rep.clone();
    rep_sw.absolute_elapsed_us = 20_000_000;
    rep_sw.relative_elapsed_us = 10_000_000;
    rep_sw.timer_pauses = Some(vec![make_pause(5_000_000, 15_000_000)]);
    rep_sw.state_timeline = vec![
        make_frame(
            0,
            GameStateBoundary::Initial,
            0,
            vec![a0_sw.clone(), a1_sw.clone()],
        ),
        make_frame(
            7_000_000,
            GameStateBoundary::Event { ordinal: 1 },
            1,
            vec![],
        ),
        make_frame(20_000_000, GameStateBoundary::Final, 1, vec![a0_sw, a1_sw]),
    ];
    let shp = super::observe::calculate_available_shield_hp(&req_2, &rep_sw).unwrap();
    // Actor 0 was active 0..7s:
    // 0..5s unpaused: 5s * 10,000 = 50,000
    // 5..7s paused: 2s * 10,000 = 20,000
    // Total absolute: 70,000 / 20 = 3500
    // Total relative: 50,000 / 10 = 5000
    assert_eq!(shp.absolute, Some(3500.0));
    assert_eq!(shp.relative, Some(5000.0));

    // 5. Early / zero-relative completion: relative_elapsed_us == 0, absolute_elapsed_us == 5s.
    rep.absolute_elapsed_us = 5_000_000;
    rep.relative_elapsed_us = 0;
    rep.timer_pauses = Some(vec![make_pause(0, 5_000_000)]);
    let s_zero = GameActorShieldSummary {
        clock_basis: ntesim_core::kernel::timing::ClockDomain::World,
        id: "s_zero".into(),
        source: "test".into(),
        remaining_hp: 10_000,
        max_hp: 10_000,
        expires_at_us: 5_000_000,
    };
    rep.state_timeline = vec![
        make_frame(
            0,
            GameStateBoundary::Initial,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![s_zero])],
        ),
        make_frame(
            5_000_000,
            GameStateBoundary::Final,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![])],
        ),
    ];
    let shp = super::observe::calculate_available_shield_hp(&req, &rep).unwrap();
    assert_eq!(shp.absolute, Some(10_000.0));
    assert_eq!(shp.relative, None);
}

#[test]
fn available_shield_hp_rejects_invalid_observations() {
    let (base_req, base_rep, base_actor) = shield_fixture();
    let mut req = base_req.clone();
    req.actors = vec![req.actors[0].clone()];
    let mut rep = base_rep.clone();
    rep.absolute_elapsed_us = 10_000_000;

    // Duplicate shield id
    let s_dup = GameActorShieldSummary {
        clock_basis: ntesim_core::kernel::timing::ClockDomain::World,
        id: "dup".into(),
        source: "test".into(),
        remaining_hp: 1000,
        max_hp: 1000,
        expires_at_us: 10_000_000,
    };
    rep.state_timeline = vec![
        make_frame(
            0,
            GameStateBoundary::Initial,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![s_dup.clone(), s_dup])],
        ),
        make_frame(
            10_000_000,
            GameStateBoundary::Final,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![])],
        ),
    ];
    assert!(super::observe::calculate_available_shield_hp(&req, &rep).is_err());

    // Remaining HP exceeds max HP
    let s_over = GameActorShieldSummary {
        clock_basis: ntesim_core::kernel::timing::ClockDomain::World,
        id: "over".into(),
        source: "test".into(),
        remaining_hp: 2000,
        max_hp: 1000,
        expires_at_us: 10_000_000,
    };
    rep.state_timeline = vec![
        make_frame(
            0,
            GameStateBoundary::Initial,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![s_over])],
        ),
        make_frame(
            10_000_000,
            GameStateBoundary::Final,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![])],
        ),
    ];
    assert!(super::observe::calculate_available_shield_hp(&req, &rep).is_err());

    // Active actor out of bounds
    rep.state_timeline = vec![
        make_frame(
            0,
            GameStateBoundary::Initial,
            99,
            vec![make_actor(&base_actor, 0, 5000, vec![])],
        ),
        make_frame(
            10_000_000,
            GameStateBoundary::Final,
            99,
            vec![make_actor(&base_actor, 0, 5000, vec![])],
        ),
    ];
    assert!(super::observe::calculate_available_shield_hp(&req, &rep).is_err());

    // Actor slot out of bounds
    rep.state_timeline = vec![
        make_frame(
            0,
            GameStateBoundary::Initial,
            0,
            vec![make_actor(&base_actor, 99, 5000, vec![])],
        ),
        make_frame(
            10_000_000,
            GameStateBoundary::Final,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![])],
        ),
    ];
    assert!(super::observe::calculate_available_shield_hp(&req, &rep).is_err());

    // Timeline decreasing
    rep.state_timeline = vec![
        make_frame(
            0,
            GameStateBoundary::Initial,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![])],
        ),
        make_frame(
            5_000_000,
            GameStateBoundary::Event { ordinal: 1 },
            0,
            vec![],
        ),
        make_frame(
            4_000_000,
            GameStateBoundary::Event { ordinal: 2 },
            0,
            vec![],
        ),
        make_frame(
            10_000_000,
            GameStateBoundary::Final,
            0,
            vec![make_actor(&base_actor, 0, 5000, vec![])],
        ),
    ];
    assert!(super::observe::calculate_available_shield_hp(&req, &rep).is_err());
}

#[test]
fn accumulated_shield_hp_matches_timeline_calculated_shield_hp() {
    for source in [
        include_str!("../../../../../examples/incoming-shield-cycles.nte"),
        include_str!("../../../../../examples/adler-continuing-energy.nte"),
    ] {
        let program =
            PreparedSourceScenario::new(source, Default::default(), &Default::default()).unwrap();
        let execution = program.run(42).unwrap();
        let timeline_shp = super::observe::calculate_available_shield_hp(
            &program.export().request,
            &execution.report,
        )
        .unwrap();
        let state_shp = execution.report.available_shield_hp;
        assert_eq!(timeline_shp, state_shp);
        let batch = completed_batch(&program, 42);
        assert_eq!(batch.available_shield_hp, timeline_shp);
    }
}

#[test]
fn batch_observations_preserve_full_trial_metrics_and_failure_inspection() {
    for source in [
        include_str!("../../../../../presets/native/live-control.nte"),
        include_str!("../../../../../examples/incoming-shield-cycles.nte"),
        include_str!("../../../../../examples/adler-continuing-energy.nte"),
    ] {
        let program =
            PreparedSourceScenario::new(source, Default::default(), &Default::default()).unwrap();
        let mut full = Collector::new(program.export().request.clone(), 3).unwrap();
        let mut lean = Collector::new(program.export().request, 3).unwrap();
        for seed in program.seed()..program.seed() + 3 {
            let sample = program.run(seed).unwrap();
            assert!(
                sample.report.success && sample.report.execution_completed,
                "streaming workload must complete: {:?}; {:?}",
                sample.diagnostic,
                sample.report.halt_diagnostic
            );
            let batch = completed_batch(&program, seed);
            assert!(sample.report.timer_catalog.is_some());
            sample.report.validate_state_timeline().unwrap();
            let sample_obs = super::observe(program.request(), &sample.report, true).unwrap();
            let batch_obs = observe_batch(batch);
            assert_eq!(
                serde_json::to_value(sample_obs).unwrap(),
                serde_json::to_value(&batch_obs).unwrap()
            );
            full.add(&sample.report).unwrap();
            lean.add_observation(batch_obs).unwrap();
        }
        assert_eq!(
            serde_json::to_value(full.snapshot(0.).unwrap()).unwrap(),
            serde_json::to_value(lean.snapshot(0.).unwrap()).unwrap()
        );
        let mut invalid = program.run(program.seed()).unwrap();
        invalid.report.state_timeline.clear();
        assert!(invalid.report.validate_state_timeline().is_err());
    }
    let program = program("", "mint burst;");
    let full = program.run(42).unwrap();
    let SourceTrialOutcome::Failed(batch) = program.run_batch(42).unwrap() else {
        panic!("failed admission must produce a detailed run");
    };
    assert!(batch.diagnostic.is_some());
    assert_eq!(
        serde_json::to_value(full).unwrap(),
        serde_json::to_value(batch).unwrap()
    );
    // S5 executes its revised rotation and completes both streaming paths.
    let s5 = PreparedSourceScenario::new(
        include_str!("../../../../../presets/studio/s5.nte"),
        Default::default(),
        &Default::default(),
    )
    .unwrap();
    let full = s5.run(s5.seed()).unwrap();
    let batch = completed_batch(&s5, s5.seed());
    assert!(full.report.execution_completed && full.report.game_dps_computed);
    assert!(full.diagnostic.is_none());
    assert!(crate::trials::source::eligible(&full));
    assert_eq!(
        serde_json::to_value(observe_batch(batch)).unwrap(),
        serde_json::to_value(observe(s5.request(), &full.report, true).unwrap()).unwrap()
    );
    assert!(
        Collector::new(s5.export().request, 1)
            .unwrap()
            .add(&full.report)
            .is_ok()
    );
}

#[test]
fn reaction_aggregation_across_trials_and_mono_element_exclusion() {
    let mono = PreparedSourceScenario::new(
        "options duration=5 iteration=2 seed=42 crit=fixed;
         mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
         target hp=100000000 def=0 resist=0; active mint; wait(1);",
        Default::default(),
        &Default::default(),
    )
    .unwrap();
    let mut collector = Collector::new(mono.export().request, 1).unwrap();
    let run = mono.run(42).unwrap();
    collector.add(&run.report).unwrap();
    let a = collector.snapshot(0.).unwrap();
    assert!(
        a.reactions.is_empty(),
        "mono-element party must have empty reactions"
    );

    let dual = PreparedSourceScenario::new(
        "options duration=5 iteration=2 seed=42 crit=fixed;
         zero char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
         mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=0;
         target hp=100000000 def=0 resist=0; active zero; mint swap; wait(3);",
        Default::default(),
        &Default::default(),
    )
    .unwrap();
    let mut dual_collector = Collector::new(dual.export().request, 2).unwrap();
    let run1 = dual.run(42).unwrap();
    let run2 = dual.run(43).unwrap();
    dual_collector.add(&run1.report).unwrap();
    dual_collector.add(&run2.report).unwrap();
    let dual_a = dual_collector.snapshot(0.).unwrap();
    assert_eq!(dual_a.reactions.len(), 1);
    assert_eq!(dual_a.reactions[0].name, "Blossom");
    assert_eq!(dual_a.reactions[0].stats.mean, 1.0);
    assert_eq!(dual_a.reactions[0].stats.min, 1.0);
    assert_eq!(dual_a.reactions[0].stats.max, 1.0);
    assert_eq!(dual_a.reactions[0].stats.std, 0.0);
}

#[test]
fn field_time_accounting_records_exact_durations_for_two_characters_pauses_and_switches() {
    let source = "options duration=10 iteration=2 seed=42 crit=fixed;
         sakiri char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=120;
         mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
         target hp=100000000 def=0 resist=0; active sakiri; wait(2); sakiri burst; mint swap; wait(3);";
    let program =
        PreparedSourceScenario::new(source, Default::default(), &Default::default()).unwrap();
    let sample = program.run_sample(42).unwrap();
    let batch42 = completed_batch(&program, 42);
    assert_eq!(sample.report.field_times, batch42.field_times);
    assert_eq!(sample.report.field_times.len(), 2);

    let pauses = sample.report.timer_pauses.as_ref().unwrap();
    let pause_us: u64 = pauses
        .iter()
        .map(|p| p.end_absolute_us - p.start_absolute_us)
        .sum();
    assert!(
        pause_us > 0,
        "Sakiri burst must produce an Ultimate cinematic pause"
    );

    // Independently calculated times:
    // Sakiri: active from start (0) to wait(2) = 2,000,000 relative us.
    // Sakiri burst produces a cinematic pause of pause_us, credited to Sakiri.
    // Mint swap executes at relative 2,000,000 us (absolute 2,000,000 + pause_us).
    // Mint is active until the 10s duration horizon = 8,000,000 relative us and 8,000,000 absolute us.
    let sakiri_rel = sample.report.field_times[0].relative.unwrap();
    let sakiri_abs = sample.report.field_times[0].absolute.unwrap();
    let mint_rel = sample.report.field_times[1].relative.unwrap();
    let mint_abs = sample.report.field_times[1].absolute.unwrap();

    assert_eq!(sakiri_rel, 2_000_000);
    assert_eq!(sakiri_abs, 2_000_000 + pause_us);
    assert_eq!(mint_rel, 8_000_000);
    assert_eq!(mint_abs, 8_000_000);

    // Both sets reconcile with trial elapsed totals
    assert_eq!(sakiri_rel + mint_rel, sample.report.relative_elapsed_us);
    assert_eq!(sakiri_abs + mint_abs, sample.report.absolute_elapsed_us);

    let mut collector = Collector::new(program.export().request, 2).unwrap();
    let batch43 = completed_batch(&program, 43);
    assert_eq!(batch43.field_times, sample.report.field_times);
    collector.add(&sample.report).unwrap();
    collector.add_observation(observe_batch(batch43)).unwrap();
    let a = collector.snapshot(0.).unwrap();
    assert_eq!(a.actors[0].field.relative.as_ref().unwrap().mean, 2.0);
    assert_eq!(
        a.actors[0].field.absolute.as_ref().unwrap().mean,
        (2_000_000 + pause_us) as f64 / 1e6
    );
    assert_eq!(a.actors[1].field.relative.as_ref().unwrap().mean, 8.0);
    assert_eq!(a.actors[1].field.absolute.as_ref().unwrap().mean, 8.0);
    assert_eq!(
        a.actors[0].field.relative.as_ref().unwrap().mean
            + a.actors[1].field.relative.as_ref().unwrap().mean,
        a.metrics.duration.mean
    );
    assert_eq!(
        a.actors[0].field.absolute.as_ref().unwrap().mean
            + a.actors[1].field.absolute.as_ref().unwrap().mean,
        a.metrics.absolute_duration.mean
    );
}

#[test]
fn compact_observations_carry_field_times_and_validate_boundary_and_zero_relative_aggregates() {
    let source = "options duration=10 iteration=2 seed=42 crit=fixed;
         mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
         sakiri char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
         target hp=100000000 def=0 resist=0; active mint; wait(4); sakiri swap; wait(6);";
    let program =
        PreparedSourceScenario::new(source, Default::default(), &Default::default()).unwrap();
    let sample = program.run_sample(42).unwrap();
    let batch = completed_batch(&program, 42);
    let mut collector = Collector::new(program.export().request.clone(), 3).unwrap();
    let base_row = observe_batch(batch);

    // 1. Rejection of malformed field-time pairs at observation boundary:
    // Actor count mismatch
    let mut bad_row = base_row.clone();
    bad_row.field_times.pop();
    assert!(collector.add_observation(bad_row).is_err());

    // Non-null requirement
    let mut bad_row = base_row.clone();
    bad_row.field_times[0].relative = None;
    assert!(collector.add_observation(bad_row).is_err());

    let mut bad_row = base_row.clone();
    bad_row.field_times[0].absolute = None;
    assert!(collector.add_observation(bad_row).is_err());

    // Clock ordering: relative > absolute
    let mut bad_row = base_row.clone();
    bad_row.field_times[0].relative = Some(bad_row.field_times[0].absolute.unwrap() + 1);
    assert!(collector.add_observation(bad_row).is_err());

    // Sum mismatch: relative sum != sample.relative_elapsed_us
    let mut bad_row = base_row.clone();
    bad_row.field_times[0].relative = Some(bad_row.field_times[0].relative.unwrap() + 100);
    assert!(collector.add_observation(bad_row).is_err());

    // Sum mismatch: absolute sum != sample.absolute_elapsed_us
    let mut bad_row = base_row.clone();
    bad_row.field_times[0].absolute = Some(bad_row.field_times[0].absolute.unwrap() + 100);
    assert!(collector.add_observation(bad_row).is_err());

    // 2. Parity between sample and batch observations
    let sample_row = observe(&program.export().request, &sample.report, true).unwrap();
    assert_eq!(sample_row.field_times, base_row.field_times);

    // 3. Zero-relative duration coverage and mixed seeds aggregation
    let mut zero_row = base_row.clone();
    zero_row.sample.seed = 42;
    zero_row.sample.relative_elapsed_us = 0;
    zero_row.sample.dps.relative = None;
    zero_row.sample.eps.relative = None;
    zero_row.sample.granted_eps.relative = None;
    zero_row.sample.hps.relative = None;
    zero_row.sample.shp.relative = None;
    zero_row.field_times[0].relative = Some(0);
    zero_row.field_times[1].relative = Some(0);
    collector.add_observation(zero_row).unwrap();

    let mut pos_row = base_row.clone();
    pos_row.sample.seed = 43;
    collector.add_observation(pos_row).unwrap();

    let a = collector.snapshot(0.).unwrap();
    // Mixed seeds: relative DPS unavailable
    assert!(a.metrics.dps.relative.is_none());
    assert!(a.metrics.dps.absolute.is_some());
    // Both actor field statistics are present and correctly aggregated over both seeds
    assert!(a.actors[0].field.relative.is_some());
    assert!(a.actors[0].field.absolute.is_some());
    assert!(a.actors[1].field.relative.is_some());
    assert!(a.actors[1].field.absolute.is_some());

    let actor0_rel_expected = (0.0 + 4.0) / 2.0;
    let actor0_abs_expected = (4.0 + 4.0) / 2.0;
    let actor1_rel_expected = (0.0 + 6.0) / 2.0;
    let actor1_abs_expected = (6.0 + 6.0) / 2.0;
    assert_eq!(
        a.actors[0].field.relative.as_ref().unwrap().mean,
        actor0_rel_expected
    );
    assert_eq!(
        a.actors[0].field.absolute.as_ref().unwrap().mean,
        actor0_abs_expected
    );
    assert_eq!(
        a.actors[1].field.relative.as_ref().unwrap().mean,
        actor1_rel_expected
    );
    assert_eq!(
        a.actors[1].field.absolute.as_ref().unwrap().mean,
        actor1_abs_expected
    );
}

#[test]
fn rounded_observations_sum_integer_receipts_before_conversion_for_both_rates() {
    let program = program("", "repeat 2 { mint attack; wait(1); }");
    let request = program.export().request;
    for (numerator, denominator, rounded, expected) in [(502, 5, 100, 200), (201, 2, 101, 202)] {
        let mut report = program.run(42).unwrap().report;
        assert!(report.hit_receipts.len() >= 2);
        report.hit_receipts.truncate(2);
        report.hits_settled = 2;
        for hit in &mut report.hit_receipts {
            hit.damage =
                ntesim_content::simulation::ExactFraction::new(numerator, denominator).unwrap();
            hit.rounded_damage = rounded;
        }
        report.total_damage_exact =
            ntesim_content::simulation::ExactFraction::new(expected, 1).unwrap();
        report.total_damage_dealt = expected as f64;
        let row = observe(&request, &report, false).unwrap();
        assert_eq!(row.sample.damage, expected as f64);
        assert_eq!(row.actors[0]["damage"], expected as f64);
        assert_eq!(
            row.sources.values().map(|s| s.0).sum::<f64>(),
            expected as f64
        );
        assert_eq!(row.targets.iter().sum::<f64>(), expected as f64);
        assert_eq!(row.bins.iter().sum::<u128>() as f64, expected as f64);
        assert_eq!(
            row.sample.dps.relative,
            Some(expected as f64 / row.sample.relative_seconds())
        );
        assert_eq!(
            row.sample.dps.absolute,
            Some(expected as f64 / row.sample.absolute_seconds())
        );
    }
}
