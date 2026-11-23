use super::*;
use serde_json::json;

fn chains(count: u64, targets: usize) -> GameScenarioRequest {
    let mut req = super::blackbird_actions::request();
    req.relative_duration_us = 20_000_000;
    req.actors[0].initial_cycle = 0;
    req.actors[0].initial_energy = 0;
    req.actors[0].energy_capacity = 1000;
    for index in 1..targets {
        let mut target = req.targets[0].clone();
        target.target_index = index;
        req.targets.push(target);
    }
    let action = req.actions[0].clone();
    req.actions = (0..count * 5)
        .map(|index| {
            let mut action = action.clone();
            if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut action {
                *timestamp_us = index * 1000;
            }
            action
        })
        .collect();
    req
}

fn skill_at(mut action: GameActionCommand, time_us: u64) -> GameActionCommand {
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        ability_id,
        damage_effect_id,
        cooldown_us,
        ..
    } = &mut action
    {
        *timestamp_us = time_us;
        *ability_id = "GA_BlackBird_Skill".into();
        *damage_effect_id = "GE_Player_BlackBird_Skill_Damage".into();
        *cooldown_us = 16_000_000;
    }
    action
}

#[test]
fn blackbird_cycle_uses_exact_native_channels_once_per_primary_group_through_cap() {
    for targets in [1, 3] {
        let req = chains(4, targets);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        // One17-packet chain:1.161+2*2.391+4*1.476+5*1.265+5*1.785=27.097.
        assert_eq!(
            report["final_actor_states"][0]["cycle"],
            json!({
                "initial":0,"current":100,"capacity":100,"granted":108,"spent":0,"overflow":8,
                "carry":"97/250","conserved":true
            })
        );
        // Ultimate's distinct channel is19.564 per chain and never substitutes for Cycle.
        assert_eq!(report["final_actor_states"][0]["granted_energy"], 78);
        assert_eq!(
            report["final_actor_states"][0]["charge_remainder"],
            json!("32/125")
        );
    }
}

#[test]
fn blackbird_cycle_efficiency_and_first_living_recipient_are_independent_of_charge() {
    let mut req = chains(1, 2);
    req.targets[0].hp_current = 0;
    req.actors[0].cycle_gain_efficiency = Ratio {
        numerator: 3,
        denominator: 2,
    };
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["final_actor_states"][0]["cycle"]["current"], 40);
    assert_eq!(
        report["final_actor_states"][0]["cycle"]["carry"],
        json!("1291/2000")
    );
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 19);
    req.actors[0].cycle_gain_efficiency = Ratio::zero();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["final_actor_states"][0]["cycle"]["current"], 0);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 19);
    req.targets[1].resistance = Ratio::one();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert!(report["final_actor_states"][0].get("cycle").is_none());
}

#[test]
fn blackbird_gained_cycle_spends_on_qualified_switch_and_keeps_fractional_carry() {
    let mut req = chains(4, 1);
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1004".into();
    ally.initial_cycle = 37;
    req.actors.push(ally);
    req.actions.extend([
        GameActionCommand::SwitchActor {
            timestamp_us: 1_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 14_000_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
    ]);
    let mut action = req.actions[0].clone();
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut action {
        *timestamp_us = 14_000_000;
    }
    req.actions.push(action);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(
        report["final_actor_states"][0]["cycle"],
        json!({
            "initial":0,"current":1,"capacity":100,"granted":109,"spent":100,"overflow":8,
            "carry":"549/1000","conserved":true
        })
    );
}

#[test]
fn blackbird_cycle_skill_packet_channel_is_separate_from_ultimate_charge() {
    let mut req = chains(1, 2);
    req.actions = vec![skill_at(req.actions[0].clone(), 0)];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // Activation fills100. Four2.6Cycle still accrue into overflow; Charge is7.56.
    assert_eq!(report["final_actor_states"][0]["cycle"]["current"], 100);
    assert_eq!(report["final_actor_states"][0]["cycle"]["granted"], 110);
    assert_eq!(report["final_actor_states"][0]["cycle"]["overflow"], 10);
    assert_eq!(
        report["final_actor_states"][0]["cycle"]["carry"],
        json!("2/5")
    );
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 7);
}

#[test]
fn blackbird_cycle_skill_activation_fills_on_miss_without_efficiency_or_ultimate_gain() {
    for initial in [0, 37, 100] {
        let mut req = chains(1, 3);
        req.actors[0].initial_cycle = initial;
        req.actors[0].cycle_gain_efficiency = Ratio::zero();
        for target in &mut req.targets {
            target.resistance = Ratio::one();
        }
        req.actions = vec![skill_at(req.actions[0].clone(), 0)];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(
            report["final_actor_states"][0]["cycle"],
            json!({
                "initial":initial,"current":100,"capacity":100,"granted":100-initial,
                "spent":0,"overflow":0,"carry":"0/1","conserved":true
            })
        );
        assert_eq!(report["final_actor_states"][0]["energy"], 0);
        assert!(
            report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .all(|hit| hit["applied_hp_loss"] == 0)
        );
    }
}

#[test]
fn blackbird_cycle_skill_fill_preserves_previous_carry_across_two_real_cooldowns() {
    let mut req = chains(1, 1);
    let normal = req.actions[0].clone();
    req.actions = vec![
        normal.clone(),
        skill_at(normal.clone(), 1000),
        skill_at(normal, 16_001_000),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // Normal1.161, fill99, then two Skills each10.4: carry.961,20overflow.
    assert_eq!(
        report["final_actor_states"][0]["cycle"],
        json!({
            "initial":0,"current":100,"capacity":100,"granted":120,"spent":0,
            "overflow":20,"carry":"961/1000","conserved":true
        })
    );
}

#[test]
fn blackbird_cycle_rejected_skill_does_not_refill_after_a_qualified_switch_spend() {
    let mut req = chains(1, 1);
    let skill = skill_at(req.actions[0].clone(), 0);
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1004".into();
    req.actors.push(ally);
    req.actions = vec![
        skill.clone(),
        GameActionCommand::SwitchActor {
            timestamp_us: 1_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 2_000_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        skill_at(skill, 3_000_000),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert_eq!(report["rejected_action"]["command_index"], 3);
    assert_eq!(
        report["final_actor_states"][0]["cycle"],
        json!({
            "initial":0,"current":0,"capacity":100,"granted":110,"spent":100,
            "overflow":10,"carry":"2/5","conserved":true
        })
    );
}

#[test]
fn blackbird_cycle_requires_the_shared_packet_model_in_strict_mode() {
    let mut req = chains(1, 1);
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 1;
    for id in [
        crate::HypothesisId::AbilityIndexPrimaryActionRoleV1,
        crate::HypothesisId::BlackbirdNormalChainV1,
        crate::HypothesisId::BlackbirdOrdinaryHitEnergyV1,
    ] {
        req.model_assumptions
            .insert(id.into(), id.description().into());
    }
    let id = crate::HypothesisId::CombatPacketCycleAccrualV1;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["final_actor_states"][0]["cycle"]["current"], 27);
}

#[test]
fn blackbird_cycle_binding_does_not_adopt_other_owners_or_unimplemented_form_rows() {
    let req = chains(1, 1);
    for packet in [
        "GE_Player_Cang_Melee1_Damage",
        "GE_Player_Mint_Skill1_Damage_New",
        "GE_Player_BlackBird_Evolve_Skill_Damage",
    ] {
        assert!(
            crate::registry::packet_cycle_grant(&req, &req.actors[0], packet)
                .unwrap()
                .is_none()
        );
    }
}
