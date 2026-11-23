use super::*;

#[test]
fn interval_union_merges_overlapping_intervals() {
    let raw = vec![
        [0, 3_000_000],
        [1_000_000, 4_000_000],
        [5_000_000, 7_000_000],
        [6_000_000, 8_000_000],
    ];
    let merged = merge_intervals(raw);
    assert_eq!(merged, vec![[0, 4_000_000], [5_000_000, 8_000_000]]);
}

#[test]
fn mode_clock_pauses_while_real_clock_continues() {
    let mut mode = ModeClockManager::new();
    mode.add_pause(ModePauseInterval {
        start_real_us: 1_000_000,
        duration_us: 3_000_000,
        reason: "burst_cinematic".to_string(),
    })
    .unwrap();

    assert_eq!(mode.real_to_mode_time(500_000), 500_000);

    assert_eq!(mode.real_to_mode_time(2_000_000), 1_000_000);

    assert_eq!(mode.real_to_mode_time(4_000_000), 1_000_000);

    assert_eq!(mode.real_to_mode_time(5_000_000), 2_000_000);

    assert_eq!(mode.mode_to_real_time(1_000_000).unwrap(), 1_000_000);
    assert_eq!(mode.mode_to_real_time(2_000_000).unwrap(), 5_000_000);
}

#[test]
fn hitlag_dilates_actor_clock_and_delays_recovery() {
    let mut hitlag = HitlagTracker::default();
    hitlag
        .add_window(HitlagWindow {
            actor_index: 0,
            start_real_us: 200_000,
            duration_real_us: 100_000,
            dilated_rate_ppm: 0,
        })
        .unwrap();

    let spec = ActionTimingSpec {
        action_id: "slash".to_string(),
        startup_us: 100_000,
        hit_offsets_us: vec![100_000],
        recovery_us: 300_000,
        earliest_switch_offset_us: 350_000,
    };

    let mode = ModeClockManager::new();
    let res = resolve_action_deadlines(0, 0, &spec, &hitlag, &mode).unwrap();

    assert_eq!(res.hit_deadlines_real_us[0], 100_000);

    assert_eq!(res.recovery_end_real_us, 500_000);

    assert_eq!(res.earliest_switch_real_us, 450_000);
}

#[test]
fn hotori_additional_cooldown_real_player_vs_copy() {
    assert_eq!(
        HotoriTimeModel::additional_skill_cooldown_us(Some(true)),
        Ok(0)
    );
    assert_eq!(
        HotoriTimeModel::additional_skill_cooldown_us(Some(false)),
        Ok(35_000_000)
    );
    assert!(HotoriTimeModel::additional_skill_cooldown_us(None).is_err());
}

#[test]
fn evaluate_timing_scenario_matches() {
    let request = TimingScenarioRequest {
        content_id: "ntesim-current".to_string(),
        team_size: 4,
        horizon_real_us: 10_000_000,
        mode_pauses: vec![ModePauseInterval {
            start_real_us: 2_000_000,
            duration_us: 3_000_000,
            reason: "burst".to_string(),
        }],
        hitlags: vec![HitlagWindow {
            actor_index: 0,
            start_real_us: 500_000,
            duration_real_us: 100_000,
            dilated_rate_ppm: 0,
        }],
        actions: vec![ScheduledActionInput {
            actor_index: 0,
            start_real_us: 0,
            spec: ActionTimingSpec {
                action_id: "skill".to_string(),
                startup_us: 400_000,
                hit_offsets_us: vec![400_000],
                recovery_us: 600_000,
                earliest_switch_offset_us: 800_000,
            },
        }],
        hotori_probe: Some(HotoriProbeInput {
            is_real_player: Some(true),
        }),
    };

    let res = evaluate_timing_scenario(&request).unwrap();
    assert_eq!(res["status"], "ok");
    assert_eq!(res["receipt"]["real_time_horizon_us"], 10_000_000);
    assert_eq!(res["receipt"]["game_mode_horizon_us"], 7_000_000);
    assert_eq!(res["receipt"]["total_mode_paused_us"], 3_000_000);
    assert_eq!(res["receipt"]["actor_clocks_at_horizon_us"][0], 9_900_000);
    assert_eq!(res["receipt"]["actor_clocks_at_horizon_us"][1], 10_000_000);
    assert_eq!(res["receipt"]["hotori"]["additional_skill_cooldown_us"], 0);
    assert_eq!(res["quality"]["timing_model_complete"], true);
}

#[test]
fn timing_scenario_propagates_checked_deadline_overflow() {
    let mut request = TimingScenarioRequest {
        content_id: "ntesim-current".to_string(),
        team_size: 1,
        horizon_real_us: u64::MAX,
        mode_pauses: vec![],
        hitlags: vec![],
        actions: vec![],
        hotori_probe: None,
    };

    request.mode_pauses.push(ModePauseInterval {
        start_real_us: u64::MAX - 1,
        duration_us: 2,
        reason: "overflow".into(),
    });
    assert!(
        evaluate_timing_scenario(&request)
            .unwrap_err()
            .contains("invalid mode pause")
    );

    request.mode_pauses.clear();
    request.hitlags.push(HitlagWindow {
        actor_index: 0,
        start_real_us: u64::MAX - 1,
        duration_real_us: 2,
        dilated_rate_ppm: 0,
    });
    assert!(
        evaluate_timing_scenario(&request)
            .unwrap_err()
            .contains("invalid hitlag window")
    );

    request.hitlags.clear();
    request.actions.push(ScheduledActionInput {
        actor_index: 0,
        start_real_us: u64::MAX - 1,
        spec: ActionTimingSpec {
            action_id: "overflowing_action".into(),
            startup_us: 2,
            hit_offsets_us: vec![2],
            recovery_us: 0,
            earliest_switch_offset_us: 0,
        },
    });
    assert!(
        evaluate_timing_scenario(&request)
            .unwrap_err()
            .contains("failed to resolve action overflowing_action")
    );
}
