use super::*;

#[test]
pub(crate) fn action_triggered_gear_proc_and_transition_deadlines_reject_overflow() {
    let limits = ExecutionLimits {
        max_horizon_us: u64::MAX,
        ..ExecutionLimits::default()
    };

    {
        let mut req = sample_s1_request();
        let cooldown = 20_000_000;
        req.relative_duration_us = u64::MAX - cooldown;
        req.absolute_limit_us = req.relative_duration_us;
        req.actors[0]
            .gear_procs
            .retain(|p| matches!(p, GearProcConfig::MindRoyaleEnergy { .. }));
        let mut skill = req.actions[1].clone();
        if let GameActionCommand::InvokeAbility {
            timestamp_us,
            recovery_us,
            cooldown_us,
            hit_delay_us,
            ..
        } = &mut skill
        {
            *timestamp_us = u64::MAX - cooldown;
            *recovery_us = 0;
            *cooldown_us = 0;
            *hit_delay_us = 0;
        }
        req.action_metadata.clear();
        req.actions = vec![skill];
        let report = run_game_scenario(&req, &limits).unwrap();
        assert_eq!(report["success"], true);
        assert_eq!(report["invariants_passed"], true);

        req.relative_duration_us = u64::MAX - cooldown + 1;
        req.absolute_limit_us = req.relative_duration_us;
        if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut req.actions[0] {
            *timestamp_us += 1;
        }
        assert!(
            run_game_scenario(&req, &limits)
                .unwrap_err()
                .contains("gear proc deadline overflow")
        );
    }

    {
        let mut req = sample_s1_request();
        let buff_duration = 10_000_000;
        req.relative_duration_us = u64::MAX - buff_duration;
        req.absolute_limit_us = req.relative_duration_us;
        req.actors[0]
            .gear_procs
            .retain(|p| matches!(p, GearProcConfig::FirefliesCritDamage { .. }));
        let mut action = req.actions[0].clone();
        let hit_delay = 2_000_000;
        if let GameActionCommand::InvokeAbility {
            timestamp_us,
            recovery_us,
            cooldown_us,
            hit_delay_us,
            ..
        } = &mut action
        {
            *timestamp_us = u64::MAX - buff_duration - hit_delay;
            *recovery_us = 0;
            *cooldown_us = 0;
            *hit_delay_us = hit_delay;
        }
        req.action_metadata.clear();
        req.actions = vec![action];
        let report = run_game_scenario(&req, &limits).unwrap();
        assert_eq!(report["success"], true);
        assert_eq!(report["invariants_passed"], true);

        if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut req.actions[0] {
            *timestamp_us += 1;
        }
        assert!(
            run_game_scenario(&req, &limits)
                .unwrap_err()
                .contains("action 0 gear proc deadline overflow")
        );
    }

    {
        let mut req = sample_s1_request();
        req.actors[0].gear_procs.clear();
        req.actors.push(req.actors[0].clone());
        req.relative_duration_us = u64::MAX;
        req.absolute_limit_us = req.relative_duration_us;
        let mut switch = GameActionCommand::SwitchActor {
            timestamp_us: u64::MAX - 5_000_000,
            target_actor_index: 1,
            cooldown_us: 5_000_000,
        };
        req.action_metadata.clear();
        req.actions = vec![switch.clone()];
        let report = run_game_scenario(&req, &limits).unwrap();
        assert_eq!(report["success"], true);

        if let GameActionCommand::SwitchActor { timestamp_us, .. } = &mut switch {
            *timestamp_us += 1;
        }
        req.action_metadata.clear();
        req.actions = vec![switch.clone()];
        assert!(
            run_game_scenario(&req, &limits)
                .unwrap_err()
                .contains("switch 0 deadline overflow")
        );

        let oob_switch = GameActionCommand::SwitchActor {
            timestamp_us: 0,
            target_actor_index: 2,
            cooldown_us: 0,
        };
        req.action_metadata.clear();
        req.actions = vec![oob_switch];
        assert!(
            run_game_scenario(&req, &limits)
                .unwrap_err()
                .contains("target_actor_index 2 out of bounds for team size 2")
        );
    }
}
