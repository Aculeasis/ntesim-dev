use super::*;

#[test]
fn charge_efficiency_carries_fractional_generation_and_restores_without_scaling() {
    let mut request = sample_s1_request();
    request.actors[0].initial_energy = 0;
    request.actors[0].gear_procs = vec![GearProcConfig::ChargeEfficiency {
        multiplier: Ratio {
            numerator: 28,
            denominator: 25,
        },
    }];
    let mut state = super::super::state::GameSimulatorState::new(&request);
    let actor = &mut state.actors[0];
    for n in 1..=25 {
        actor.grant_energy(18).unwrap();
        // Independent integer oracle: floor(n * 18 * 28 / 25).
        assert_eq!(actor.granted_energy, n * 18 * 28 / 25);
        assert_eq!(
            actor.charge_remainder,
            ExactFraction::new((n * 18 * 28 % 25).into(), 25).unwrap()
        );
    }
    assert_eq!(actor.energy.current(), 100);
    assert_eq!(actor.overflow_energy, 404);
    actor.spend_energy(100).unwrap();
    actor.restore_energy(100).unwrap();
    assert_eq!(actor.granted_energy, 604);
    assert_eq!(actor.overflow_energy, 404);
    state.verify_invariants().unwrap();
    let actor = &mut state.actors[0];
    actor.granted_energy = u64::MAX;
    let before = (
        actor.energy.clone(),
        actor.overflow_energy,
        actor.charge_remainder,
    );
    assert!(actor.grant_energy(18).is_err());
    assert_eq!(
        before,
        (
            actor.energy.clone(),
            actor.overflow_energy,
            actor.charge_remainder
        )
    );
}

#[test]
fn nanally_manual_burst_packet_does_not_create_a_free_a1_refund() {
    let mut request = sample_s1_request();
    request.actors[0].character_id = "character:1010".into();
    request.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
    request.actors[0].initial_energy = 0;
    request.actors[0].weapon_id.clear();
    request.actors[0].gear_procs = vec![GearProcConfig::ChargeEfficiency {
        multiplier: Ratio {
            numerator: 28,
            denominator: 25,
        },
    }];
    request.action_metadata.clear();
    request.actions = (0..3)
        .map(|n| GameActionCommand::InvokeAbility {
            timestamp_us: n * 1_000_000,
            actor_index: 0,
            ability_id: "GA_Nanally_UltraSkill".into(),
            damage_effect_id: "GE_Player_Nanally_Cat_AirAttack_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 0,
        })
        .collect();
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // Creating a summon/manual Q packet performs no follow-up attack.
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 0);
    assert!(report["final_actor_states"][0]["charge_remainder"].is_null());
    assert_eq!(report["final_actor_states"][0]["energy_conserved"], true);
}

#[test]
pub(crate) fn s1_scenario_executes_with_conservation_and_expected_damage() {
    let req = sample_s1_request();
    let limits = ExecutionLimits::default();
    let val = run_game_scenario(&req, &limits).unwrap();

    assert_eq!(val["status"], "ok");
    assert_eq!(val["success"], true);
    assert_eq!(val["execution_completed"], true);
    assert_eq!(val["invariants_passed"], true);
    assert_eq!(val["hits_settled"], 11);
    assert_eq!(val["max_proc_depth_reached"], 1);

    let actor0 = &val["final_actor_states"][0];
    assert_eq!(actor0["energy"], 0);
    assert_eq!(actor0["initial_energy"], 100);
    assert_eq!(actor0["granted_energy"], 24);
    assert_eq!(actor0["spent_energy"], 100);
    assert_eq!(actor0["overflow_energy"], 24);
    assert_eq!(actor0["energy_conserved"], true);
    assert_eq!(actor0["fireflies_stacks"], 0);

    // Native L8: .178 + .396 + 2*1.542 + 2.273 + 5*1.008 + 6.401.
    // ATK2319.15, elemental1.1, mitigation.4; Fireflies affects crit only.
    let total_dmg = val["total_damage_dealt"].as_f64().unwrap();
    let expected = [
        0.178, 0.396, 1.542, 1.542, 2.273, 1.008, 1.008, 1.008, 1.008, 1.008, 6.401,
    ]
    .into_iter()
    .map(|coefficient| (2319.15_f64 * 1.1 * 0.4 * coefficient).round())
    .sum::<f64>();
    assert_eq!(total_dmg, expected);
}

#[test]
pub(crate) fn core_pool_grants_match_independent_wide_integer_flow_at_boundaries() {
    let mut cases = 0;
    for capacity in [0, 1, 10, u64::MAX - 1, u64::MAX] {
        for energy in [0, capacity / 2, capacity] {
            for amount in [0, 1, 2, capacity, u64::MAX] {
                let mut request = sample_s1_request();
                request.actors[0].initial_energy = energy;
                request.actors[0].energy_capacity = capacity;
                let mut state = super::super::state::GameSimulatorState::new(&request);
                let actor = &mut state.actors[0];
                actor.restore_energy(amount).unwrap();

                let expected_energy = energy.saturating_add(amount).min(capacity);
                let expected_overflow = amount.saturating_sub(capacity - energy);
                assert_eq!(actor.energy.current(), expected_energy);
                assert_eq!(actor.granted_energy, amount);
                assert_eq!(actor.overflow_energy, expected_overflow);
                state.verify_invariants().unwrap();
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 75);
}

#[test]
pub(crate) fn mint_a6_large_pool_grants_conserve_energy_without_adding_before_clamping() {
    let limits = ExecutionLimits {
        max_pool_capacity: u64::MAX,
        ..ExecutionLimits::default()
    };
    let mut req = sample_s1_request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actors[0].initial_energy = u64::MAX;
    req.actors[0].energy_capacity = u64::MAX;
    let report = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report["success"], true);
    let actor = &report["final_actor_states"][0];
    assert_eq!(actor["granted_energy"], 26);
    assert_eq!(actor["overflow_energy"], 26);
    assert_eq!(actor["spent_energy"], 100);
    assert_eq!(actor["energy_conserved"], true);
    assert_eq!(actor["energy"], u64::MAX - 100);
    req.actors[0].initial_energy = 0;
    req.actors[0].gear_procs = vec![GearProcConfig::MindRoyaleEnergy {
        energy_grant: u64::MAX,
        cooldown_us: 0,
    }];
    req.action_metadata.clear();
    req.actions.remove(0);
    assert!(
        run_game_scenario(&req, &limits)
            .unwrap_err()
            .contains("energy grant accounting overflow")
    );
}

#[test]
pub(crate) fn resource_shortage_causes_transactional_rollback_and_preserves_invariants() {
    let mut req = sample_s1_request();
    req.actors[0].initial_energy = 50;
    let limits = ExecutionLimits::default();
    let val = run_game_scenario(&req, &limits).unwrap();

    assert_eq!(val["status"], "failed");
    assert_eq!(val["success"], false);
    assert_eq!(val["execution_completed"], false);
    assert_eq!(val["hits_settled"], 4);
    assert!(
        val["halt_diagnostic"]
            .as_str()
            .unwrap()
            .contains("RESOURCE_SHORTAGE")
    );

    assert_eq!(val["invariants_passed"], true);
    let actor0 = &val["final_actor_states"][0];
    assert_eq!(actor0["energy_conserved"], true);
}

#[test]
pub(crate) fn locked_actor_causes_rollback() {
    let mut req = sample_s1_request();
    match &mut req.actions[2] {
        GameActionCommand::InvokeAbility { timestamp_us, .. } => {
            *timestamp_us = 1_500_000;
        }
        _ => unreachable!(),
    }
    let limits = ExecutionLimits::default();
    let val = run_game_scenario(&req, &limits).unwrap();

    assert_eq!(val["status"], "failed");
    assert_eq!(val["success"], false);
    assert!(
        val["halt_diagnostic"]
            .as_str()
            .unwrap()
            .contains("ACTOR_LOCKED")
    );
    assert_eq!(val["invariants_passed"], true);
}

#[test]
pub(crate) fn mint_a6_awakening_applies_damage_bonus_below_hp_threshold_and_resonance_energy() {
    let mut req_exec = sample_s1_request();
    req_exec.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req_exec.targets[0].hp_max = 1_000_000;
    req_exec.targets[0].hp_current = 350_000;

    let limits = ExecutionLimits::default();
    let val_exec = run_game_scenario(&req_exec, &limits).unwrap();

    assert_eq!(val_exec["status"], "ok");
    assert_eq!(val_exec["success"], true);
    assert_eq!(val_exec["hits_settled"], 11);

    let actor0 = &val_exec["final_actor_states"][0];
    assert_eq!(actor0["granted_energy"], 26);
    assert_eq!(actor0["overflow_energy"], 26);
    assert_eq!(actor0["energy_conserved"], true);

    let mut req_above = req_exec.clone();
    req_above.targets[0].hp_current = 500_000;
    let val_above = run_game_scenario(&req_above, &limits).unwrap();

    let dmg_exec = val_exec["total_damage_dealt"].as_f64().unwrap();
    let dmg_above = val_above["total_damage_dealt"].as_f64().unwrap();

    assert!(
        dmg_exec > dmg_above,
        "dmg_exec {dmg_exec} must exceed dmg_above {dmg_above}"
    );
}
