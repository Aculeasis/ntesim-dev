use super::*;

fn request() -> GameScenarioRequest {
    let mut req = research_request();
    req.stochastic_crit = None;
    req.actors[0].weapon_id = "weapon:fork_BoxingCandy".into();
    req.actors[0].gear_procs = vec![GearProcConfig::BoxingCandyDamage {
        damage_bonus: Ratio {
            numerator: 22,
            denominator: 100,
        },
        low_hp_damage_bonus: Ratio {
            numerator: 28,
            denominator: 100,
        },
    }];
    req.actors[0].static_stats.damage_bonus = Ratio {
        numerator: 1,
        denominator: 10,
    };
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.targets[0].resistance = Ratio::zero();
    req
}

#[test]
fn boxing_candy_samples_live_hp_before_each_hit_and_replaces_the_bonus() {
    let mut req = request();
    req.targets[0].hp_max = 10_000;
    req.targets[0].hp_current = 6_320;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    let damage = |target| {
        hits.iter()
            .filter(|hit| hit["target_index"] == target)
            .map(|hit| hit["rounded_damage"].as_u64().unwrap())
            .collect::<Vec<_>>()
    };
    // 1000 ATK * .5 DEF * (1 + .1 + .22) = 660 above and at 50%.
    // 6320 -> 5660 -> 5000 -> 4340: the crossing hit keeps 660.
    // Below 50% replace .22 by .28: 1000 * .5 * 1.38 = 690.
    assert_eq!(damage(0), vec![660, 660, 660, 690, 690, 690]);
    assert_eq!(
        report["resolved_model_assumptions"][crate::weapons::boxing_candy::HYPOTHESIS_ID.as_str()],
        crate::weapons::boxing_candy::HYPOTHESIS_ID.description()
    );
    let mut duplicate = req.clone();
    let extra = duplicate.actors[0].gear_procs[0].clone();
    duplicate.actors[0].gear_procs.push(extra);
    assert!(
        run_game_scenario(&duplicate, &ExecutionLimits::default())
            .unwrap_err()
            .contains("duplicates")
    );
    req.actors[0].weapon_id = "weapon:fork_mofeikesi".into();
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires equipped")
    );
}

#[test]
fn boxing_candy_reads_each_target_independently() {
    use crate::simulation::{context::EventContext, state::GameSimulatorState};
    let mut req = request();
    req.targets[0].hp_max = 10_000;
    req.targets[0].hp_current = 6_320;
    let mut low = req.targets[0].clone();
    low.target_index = 1;
    low.hp_current = 4_999;
    req.targets.push(low);
    let packets =
        crate::simulation::validation::validate_and_resolve(&req, &ExecutionLimits::default())
            .unwrap();
    let mut state = GameSimulatorState::new(&req);
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    let mut rng = None;
    let mut hits = Vec::new();
    let mut ledger = Vec::new();
    let mut ctx = EventContext {
        request: &req,
        prepared_packets: &packets,
        state: &mut state,
        queue: &mut queue,
        sequence: &mut sequence,
        trace: &mut trace,
        rng: &mut rng,
        hits: &mut hits,
        action_metadata_ledger: &mut ledger,
    };
    for target_index in [1, 0, 1, 0] {
        crate::simulation::hit::settle(
            &mut ctx,
            crate::simulation::hit::HitIdentity {
                parent_hit_index: None,
                command_index: 0,
                actor_index: 0,
                target_index,
            },
            "caller-test".into(),
            "caller-effect".into(),
            Ratio::one(),
            crate::simulation::NativeScalingStat::Atk,
            packets[0].as_ref().unwrap().clone(),
        )
        .unwrap();
    }
    assert_eq!(
        ctx.hits
            .iter()
            .map(|hit| hit.rounded_damage)
            .collect::<Vec<_>>(),
        vec![690, 660, 690, 660]
    );
}

#[test]
fn boxing_candy_delivery_uses_changed_hp_and_off_field_owner() {
    let mut req = request();
    req.targets[0].hp_max = 10_000;
    req.targets[0].hp_current = 5_500;
    let mut other = req.actors[0].clone();
    other.slot = 1;
    other.character_id = "character:1019".into();
    other.weapon_id.clear();
    other.gear_procs.clear();
    req.actors.push(other);
    req.actions.truncate(2);
    req.action_metadata.retain(|index, _| *index < 2);
    if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut req.actions[0] {
        *hit_delay_us = 100;
    }
    if let GameActionCommand::InvokeAbility { actor_index, .. } = &mut req.actions[1] {
        *actor_index = 1;
    }
    req.actions.insert(
        1,
        GameActionCommand::SwitchActor {
            timestamp_us: 5,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    );
    let meta = req.action_metadata.remove(&1).unwrap();
    req.action_metadata.insert(2, meta);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hit_receipts"][0]["actor_index"], 1);
    assert_eq!(report["hit_receipts"][0]["rounded_damage"], 550);
    assert_eq!(report["hit_receipts"][1]["actor_index"], 0);
    assert_eq!(report["hit_receipts"][1]["rounded_damage"], 690);
    req.targets[0].hp_max = u64::MAX;
    req.targets[0].hp_current = u64::MAX / 2;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hit_receipts"][1]["rounded_damage"], 690);
    req.actors[0].gear_procs = vec![GearProcConfig::BoxingCandyDamage {
        damage_bonus: Ratio::one(),
        low_hp_damage_bonus: Ratio::zero(),
    }];
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("enhanced bonus")
    );
}
