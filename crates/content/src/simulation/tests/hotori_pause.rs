use super::*;

fn burst(timestamp_us: u64) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index: 0,
        ability_id: "GA_Jin_UltraSkill".into(),
        damage_effect_id: "GE_Player_Jin_UltraSkill1_Damage".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: crate::characters::hotori::skill::public_burst_timing().0,
        cooldown_us: 0,
        hit_delay_us: 100,
    }
}

fn switch(timestamp_us: u64, target: usize) -> GameActionCommand {
    GameActionCommand::SwitchActor {
        timestamp_us,
        target_actor_index: target,
        cooldown_us: 0,
    }
}

fn accept_incoming_damage(req: &mut GameScenarioRequest) {
    let id = crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID;
    req.model_assumptions
        .insert(id.as_str().into(), id.description().into());
}

fn incoming(timestamp_us: u64, actor_index: usize, delivered_damage: u64) -> GameActorDamageEvent {
    GameActorDamageEvent {
        timestamp_us,
        actor_index,
        delivered_damage,
    }
}

fn trace_times(report: &serde_json::Value, kind: &str) -> Vec<u64> {
    report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == kind)
        .map(|event| event["time_us"].as_u64().unwrap())
        .collect()
}

fn reaction_actor(
    req: &mut GameScenarioRequest,
    slot: usize,
    character_id: &str,
    cycle: u64,
    ci: u64,
) {
    let mut base = req.actors[0].clone();
    base.slot = slot;
    base.character_id = character_id.into();
    base.initial_cycle = cycle;
    base.static_stats.cycle_intensity = Ratio {
        numerator: ci,
        denominator: 1,
    };
    base.static_stats.crit_damage_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };
    while req.actors.len() <= slot {
        let mut filler = req.actors[0].clone();
        filler.slot = req.actors.len();
        req.actors.push(filler);
    }
    req.actors[slot] = base;
}

/// Burst at 20s enters at 22.183333 and ends at the 34.183333 deadline; enemy
/// schedules retain their delay through both the 2.183333s cinematic setup
/// and the 12s active interval, with their union counted once.
#[test]
fn hotori_time_stop_pauses_enemy_schedules_for_the_actual_interval() {
    let mut req = super::hotori_actions::request();
    req.relative_duration_us = 60_000_000;
    req.actions = vec![burst(20_000_000)];
    req.incoming_damage_events = vec![incoming(25_000_000, 0, 100), incoming(40_000_000, 0, 100)];
    accept_incoming_damage(&mut req);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    assert_eq!(
        trace_times(&report, "hotori_time_stop_enter"),
        vec![22_183_333]
    );
    assert_eq!(
        trace_times(&report, "hotori_time_stop_expired"),
        vec![34_183_333]
    );

    // The first input had five seconds remaining at commitment. It resumes
    // at 34.183333, so delivery is 39.183333. The later input shifts equally.
    let receipts = report["incoming_damage_receipts"].as_array().unwrap();
    assert_eq!(receipts.len(), 2);
    assert_eq!(receipts[0]["time_us"], 39_183_333);
    assert_eq!(receipts[1]["time_us"], 54_183_333);

    // The finite committing Burst context has an explicit exception. Its
    // direct packet train keeps the original absolute delivery times.
    let burst_hits: Vec<u64> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| {
            hit["damage_effect_id"]
                .as_str()
                .unwrap()
                .starts_with("GE_Player_Jin_UltraSkill")
        })
        .map(|hit| hit["time_us"].as_u64().unwrap())
        .collect();
    assert_eq!(burst_hits.len(), 10);
    assert_eq!(burst_hits[0], 22_316_080);
    assert_eq!(burst_hits[9], 24_002_235);
}

#[test]
fn hotori_pause_forced_cancellation_clips_setup_and_active_stop_without_refund() {
    for cancel_us in [21_000_000, 25_000_000] {
        let mut req = super::hotori_actions::request();
        req.relative_duration_us = 40_000_000;
        req.actions = vec![burst(20_000_000)];
        req.interrupt_events = vec![GameActorInterruptEvent {
            timestamp_us: cancel_us,
            actor_index: 0,
            armor_damage: None,
        }];
        req.incoming_damage_events = vec![incoming(21_000_000, 0, 100)];
        accept_incoming_damage(&mut req);
        let report = run_game_scenario(&req, &Default::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        assert_eq!(
            trace_times(&report, "hotori_time_stop_cancelled"),
            vec![cancel_us]
        );
        assert!(trace_times(&report, "hotori_time_stop_expired").is_empty());
        let pauses = report["timer_pauses"].as_array().unwrap();
        assert_eq!(pauses.first().unwrap()["start_absolute_us"], 20_000_000);
        assert_eq!(pauses.last().unwrap()["end_absolute_us"], cancel_us);
        assert_eq!(report["relative_elapsed_us"], 40_000_000);
        assert_eq!(
            report["absolute_elapsed_us"],
            40_000_000 + cancel_us - 20_000_000
        );
        assert_eq!(
            report["incoming_damage_receipts"][0]["time_us"],
            cancel_us + 1_000_000
        );
        if cancel_us < 22_183_333 {
            assert!(trace_times(&report, "hotori_time_stop_enter").is_empty());
            assert_eq!(report["hits_settled"], 0);
            assert_eq!(
                report["interrupt_receipts"][0]["cancelled_hit_deliveries"],
                10
            );
        }
        // Twenty world seconds after commitment would recover 120 naturally;
        // no refund or stale deadline may add an earlier grant.
        let cancelled = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .find(|event| event["event_kind"] == "hotori_time_stop_cancelled")
            .unwrap();
        assert_eq!(cancelled["data"]["exact_amount"], "0/1");
    }
}

/// A bud created at 28s retains its ten-second lifetime through cinematic
/// setup, then ages during active stop. Ordinary emissions expire without a
/// backlog; living B2 emits all five volleys on the retained phase after setup.
#[test]
fn hotori_time_stop_pauses_vita_buds_only_without_b2() {
    for (max_level, expected) in [
        (30, vec![30_000_000]),
        (
            80,
            vec![30_000_000, 34_183_333, 36_183_333, 38_183_333, 40_183_333],
        ),
    ] {
        let mut req = super::hotori_actions::request();
        req.actors[0].character_max_level = Some(max_level);
        req.actors[0].character_level = max_level;
        req.actors[0].purchased_qte_level = Some(1);
        reaction_actor(&mut req, 1, "character:1046", 100, 0);
        reaction_actor(&mut req, 2, "character:1019", 0, 600);
        req.relative_duration_us = 60_000_000;
        req.actions = vec![
            switch(0, 1),
            switch(28_000_000, 2),
            switch(29_000_000, 0),
            burst(30_000_000),
        ];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        let pistils: Vec<u64> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "ntesim.reaction.blossom")
            .map(|hit| hit["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(pistils, expected, "breakthrough cap {max_level}");
    }
}

#[test]
fn hotori_pause_b2_buds_cannot_farm_charge_or_defer_energy() {
    let source = "options duration=25 iteration=1 crit=fixed;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        hotori char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100 start_timepiece=120;
        hathor char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        target hp=100000000 def=0 resist=0; active mint;
        hotori swap; wait(1); hathor swap; wait(1); hotori burst;";
    let result = PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    let pauses = result.report.timer_pauses.as_ref().unwrap();
    let stop = pauses
        .iter()
        .find(|pause| pause.kind == ntesim_core::kernel::timing::ModePauseKind::ActiveStop)
        .unwrap();
    let inside = |time| time >= stop.start_absolute_us && time < stop.end_absolute_us;
    assert!(
        result
            .report
            .hit_receipts
            .iter()
            .any(|hit| hit.damage_effect_id == "ntesim.reaction.blossom" && inside(hit.time_us))
    );
    assert!(
        !result
            .report
            .trace
            .iter()
            .any(|event| event.event_kind == "reaction_charge_energy" && inside(event.time_us))
    );
    // The bud expires before release; frozen ordinary carriers cannot create
    // a deferred grant on release or rebuild suppressed fractional carry.
    assert!(
        !result
            .report
            .trace
            .iter()
            .any(|event| event.event_kind == "reaction_charge_energy"
                && event.time_us >= stop.end_absolute_us)
    );
    assert_native_replay(&result);
}
