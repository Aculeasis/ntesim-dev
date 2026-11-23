use super::*;
use serde_json::json;

fn request() -> GameScenarioRequest {
    let mut req = sample_s1_request();
    req.actors[0].character_id = "character:1046".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
    req.actors[0].gear_procs.clear();
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.targets[0].resistance = Ratio::zero();
    req.actions = vec![GameActionCommand::InvokeAbility {
        timestamp_us: 0,
        actor_index: 0,
        ability_id: "GA_Female046_Skill".into(),
        array_index: 0,
        damage_effect_id: "GE_Player_Female046_Skill1_Damage".into(),
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 100,
    }];
    req
}

#[test]
fn zero_a1_checks_adjacent_levels_zero_damage_death_and_a6_ratio() {
    for (level, hits) in [(79, 5), (80, 5), (81, 4)] {
        let mut req = request();
        req.targets[0].level = level;
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["hits_settled"], hits, "level {level}");
        assert_eq!(
            report["resolved_model_assumptions"]
                [crate::characters::zero::skill::A1_HYPOTHESIS_ID.as_str()],
            crate::characters::zero::skill::A1_HYPOTHESIS_ID.description()
        );
        if hits == 5 {
            // 200% * 1000 * 1080 / (1080 + 1080 * 25%) = 1600.
            assert_eq!(
                report["hit_receipts"][4]["non_crit_damage"],
                json!("1600/1")
            );
            assert_eq!(report["hit_receipts"][1]["command_index"], 0);
        }
    }
    for awakening in [0, 6] {
        let mut req = request();
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["hits_settled"], if awakening == 0 { 4 } else { 5 });
        if awakening == 6 {
            assert_eq!(
                report["hit_receipts"][4]["non_crit_damage"],
                json!("2400/1")
            );
        }
    }
    let mut req = request();
    if let GameActionCommand::InvokeAbility { atk_rate, .. } = &mut req.actions[0] {
        *atk_rate = Ratio::zero();
    }
    assert_eq!(
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap()["hits_settled"],
        5
    );
    let mut req = request();
    req.targets[0].hp_current = 1;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 1);
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_kind"] == "hit_target_unavailable")
    );
}

#[test]
fn zero_first_hit_eligibility_survives_overlapping_casts_and_off_field_delivery() {
    let mut req = request();
    let mut second = req.actions[0].clone();
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        hit_delay_us,
        ..
    } = &mut second
    {
        *timestamp_us = 10;
        *hit_delay_us = 10;
    }
    req.actions.push(second);
    let mut other = req.actors[0].clone();
    other.slot = 1;
    other.character_id = "character:1019".into();
    other.awakening = ntesim_data::AwakeningSelection::prefix(0);
    req.actors.push(other);
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 15,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 15);
    assert_eq!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["damage_effect_id"] != "ntesim.reaction.blossom")
            .map(|hit| hit["command_index"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![1, 1, 1, 1, 1, 0, 0, 0, 0, 0]
    );
    // Two overlapping Skill trains plus five level80/CI0/DEF1080/RES0 pistils.
    assert_eq!(report["total_damage_exact"], json!("29500/1"));
    assert_eq!(
        report,
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap()
    );
}

#[test]
fn zero_a1_requires_acceptance_in_strict_and_rejects_conflicting_descriptions() {
    let mut req = request();
    let role =
        crate::characters::action_roles::resolve("character:1046", "GA_Female046_Skill").unwrap();
    req.model_assumptions
        .insert(role.hypothesis_id.into(), role.description.into());
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::zero::skill::A1_HYPOTHESIS_ID.into(),
        crate::characters::zero::skill::A1_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::zero::skill::SKILL_PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::zero::skill::SKILL_PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::simulation::energy::ZERO_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::ZERO_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    assert_eq!(
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap()["hits_settled"],
        5
    );
    req.model_assumptions.insert(
        crate::characters::zero::skill::A1_HYPOTHESIS_ID.into(),
        "Conflicting level policy".into(),
    );
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("conflicting reached")
    );
}

#[test]
fn same_activation_deduplicates_each_target_and_child_cannot_retrigger() {
    use crate::simulation::{
        context::EventContext, events::GameEventData, state::GameSimulatorState,
    };
    let mut req = request();
    let mut other = req.targets[0].clone();
    other.target_index = 1;
    req.targets.push(other);
    let packets =
        crate::simulation::validation::validate_and_resolve(&req, &ExecutionLimits::default())
            .unwrap();
    let metadata = packets[0].as_ref().unwrap();
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
    for target in [1, 1, 0] {
        crate::simulation::hit::settle(
            &mut ctx,
            crate::simulation::hit::HitIdentity {
                parent_hit_index: None,
                command_index: 0,
                actor_index: 0,
                target_index: target,
            },
            "GA_Female046_Skill".into(),
            "GE_Player_Female046_Skill1_Damage".into(),
            Ratio::one(),
            crate::simulation::NativeScalingStat::Atk,
            metadata.clone(),
        )
        .unwrap();
    }
    assert_eq!(ctx.queue.len(), 2);
    while let Some(event) = ctx.queue.pop() {
        if let GameEventData::HitDelivery {
            command_index,
            actor_index,
            target_index,
            ability_id,
            damage_effect_id,
            atk_rate,
            scaling_stat,
            metadata,
        } = event.data
        {
            crate::simulation::hit::settle(
                &mut ctx,
                crate::simulation::hit::HitIdentity {
                    parent_hit_index: None,
                    command_index,
                    actor_index,
                    target_index,
                },
                ability_id,
                damage_effect_id,
                atk_rate,
                scaling_stat,
                *metadata,
            )
            .unwrap();
        }
    }
    assert_eq!(
        ctx.hits
            .iter()
            .map(|hit| hit.target_index)
            .collect::<Vec<_>>(),
        vec![1, 1, 0, 1, 0]
    );
    assert_eq!(ctx.hits.len(), 5);
}
