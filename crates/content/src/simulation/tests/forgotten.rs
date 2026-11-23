use super::*;

fn request() -> GameScenarioRequest {
    let mut request = research_request();
    request.actions.truncate(1);
    request.action_metadata.retain(|index, _| *index == 0);
    request.actors[0].weapon_id = "weapon:fork_wuhuakuang".into();
    request.actors[0].gear_procs = vec![GearProcConfig::ForgottenHpStats {
        atk_bonus: Ratio {
            numerator: 1,
            denominator: 5,
        },
        def_bonus: Ratio {
            numerator: 1,
            denominator: 5,
        },
    }];
    request.actors[0].static_stats.base_atk = 1000;
    request.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    request.actors[0].static_stats.crit_rate = Ratio::zero();
    request.actors[0].static_stats.crit_damage_bonus = Ratio::zero();
    request.actors[0].static_stats.base_def = Some(1000);
    request.actors[0].static_stats.def_percent = Ratio {
        numerator: 3,
        denominator: 10,
    };
    request.actors[0].static_stats.def_flat = Ratio {
        numerator: 240,
        denominator: 1,
    };
    request.actors[0].static_stats.def = 1540;
    request.targets[0].defense = 0;
    request.targets[0].resistance = Ratio::zero();
    if let GameActionCommand::InvokeAbility { scaling_stat, .. } = &mut request.actions[0] {
        *scaling_stat = crate::simulation::NativeScalingStat::Def;
    }
    request
}

#[test]
fn forgotten_def_percent_is_strictly_below_half_and_requires_base_provenance() {
    let mut baseline = request();
    baseline.actors[0].gear_procs.clear();
    let baseline_hit = run_game_scenario(&baseline, &ExecutionLimits::default()).unwrap()["hit_receipts"][0]["rounded_damage"].as_u64().unwrap();
    for (hp, max_hp, expected) in [
        (1001, 2000, 0),
        (1000, 2000, 0),
        (999, 2000, 1),
        (1000, 2001, 1),
        (1001, 2001, 0),
        (u64::MAX / 2, u64::MAX, 1),
        (u64::MAX / 2 + 1, u64::MAX, 0),
    ] {
        let mut scenario = request();
        scenario.actors[0].initial_hp = hp;
        scenario.actors[0].initial_hp_max = max_hp;
        let report = run_game_scenario(&scenario, &ExecutionLimits::default()).unwrap();
        let hit = report["hit_receipts"][0]["rounded_damage"]
            .as_u64()
            .unwrap();
        assert_eq!(hit, baseline_hit + expected * 200);
        assert_eq!(
            report["resolved_model_assumptions"][crate::weapons::forgotten::HYPOTHESIS_ID.as_str()],
            crate::weapons::forgotten::HYPOTHESIS_ID.description()
        );
    }
    let mut missing = request();
    missing.actors[0].initial_hp = 999;
    missing.actors[0].initial_hp_max = 2000;
    missing.actors[0].static_stats.base_def = None;
    assert!(
        run_game_scenario(&missing, &ExecutionLimits::default())
            .unwrap_err()
            .contains("native base-DEF projection")
    );
}

#[test]
fn forgotten_def_oracle_composes_static_live_flat_and_weapon_base_percent() {
    use crate::simulation::{
        context::EventContext,
        state::{GameSimulatorState, TimedBuffDeclaration},
    };
    use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope};

    let mut scenario = request();
    scenario.actors[0].initial_hp = 999;
    scenario.actors[0].initial_hp_max = 2000;
    scenario.actors[0].static_stats.def_percent = Ratio {
        numerator: 3,
        denominator: 10,
    };
    scenario.actors[0].static_stats.def_flat = Ratio {
        numerator: 240,
        denominator: 1,
    };
    scenario.actors[0].static_stats.def = 1540;
    let prepared =
        crate::simulation::validation::validate_and_resolve(&scenario, &ExecutionLimits::default())
            .unwrap();

    let damage = |with_weapon: bool, scaling_stat| {
        let mut local = scenario.clone();
        if !with_weapon {
            local.actors[0].gear_procs.clear();
        }
        if let GameActionCommand::InvokeAbility {
            scaling_stat: scaling,
            ..
        } = &mut local.actions[0]
        {
            *scaling = scaling_stat;
        }
        let mut state = GameSimulatorState::new(&local);
        state
            .apply_buff(
                0,
                TimedBuffDeclaration::reset(
                    "test.live_def_percent",
                    &local.actors[0].character_id,
                    BuffRecipient::SelfActor,
                    100,
                    BuffStackingScope::PerSource,
                    BTreeMap::from([(
                        "def_percent".into(),
                        crate::simulation::ExactFraction::new(1, 10).unwrap(),
                    )]),
                ),
            )
            .unwrap();
        let mut queue = crate::simulation::timer_queue::EventQueue::new();
        let mut sequence = 0;
        let mut trace = Vec::new();
        let mut rng = None;
        let mut hits = Vec::new();
        let mut ledger = Vec::new();
        let mut ctx = EventContext {
            request: &local,
            prepared_packets: &prepared,
            state: &mut state,
            queue: &mut queue,
            sequence: &mut sequence,
            trace: &mut trace,
            rng: &mut rng,
            hits: &mut hits,
            action_metadata_ledger: &mut ledger,
        };
        let (ability_id, damage_effect_id) = match &local.actions[0] {
            GameActionCommand::InvokeAbility {
                ability_id,
                damage_effect_id,
                ..
            } => (ability_id.clone(), damage_effect_id.clone()),
            _ => unreachable!(),
        };
        crate::simulation::hit::settle(
            &mut ctx,
            crate::simulation::hit::HitIdentity {
                parent_hit_index: None,
                command_index: 0,
                actor_index: 0,
                target_index: 0,
            },
            ability_id,
            damage_effect_id,
            Ratio::one(),
            scaling_stat,
            prepared[0].as_ref().unwrap().clone(),
        )
        .unwrap();
        ctx.hits[0].rounded_damage
    };

    assert_eq!(
        damage(false, crate::simulation::NativeScalingStat::Def),
        1640
    );
    assert_eq!(
        damage(true, crate::simulation::NativeScalingStat::Def),
        1840
    );
    assert_eq!(
        damage(true, crate::simulation::NativeScalingStat::Atk),
        1000
    );
}

#[test]
fn forgotten_hypothesis_requires_an_exact_description_and_matching_weapon() {
    let mut req = request();
    req.model_assumptions.insert(
        crate::weapons::forgotten::HYPOTHESIS_ID.into(),
        "HP behavior is different".into(),
    );
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("conflicting reached model hypothesis")
    );

    let mut mismatched = request();
    mismatched.actors[0].weapon_id = "weapon:fork_moon".into();
    assert!(
        run_game_scenario(&mismatched, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires equipped weapon:fork_wuhuakuang")
    );
}

#[test]
fn forgotten_samples_the_off_field_wearers_live_hp_at_hit_delivery() {
    use crate::simulation::{context::EventContext, state::GameSimulatorState};

    let mut request = request();
    request.actors[0].character_id = "character:1046".into();
    request.actors[0].initial_hp = 1001;
    request.actors[0].initial_hp_max = 2000;
    request.actors[0].static_stats.base_atk = 1000;
    request.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    let mut ally = request.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.weapon_id.clear();
    ally.gear_procs.clear();
    request.actors.push(ally);
    let mut owner_first = request.actions[0].clone();
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        hit_delay_us,
        recovery_us,
        ..
    } = &mut owner_first
    {
        *timestamp_us = 0;
        *hit_delay_us = 100;
        *recovery_us = 0;
    }
    let mut owner_second = owner_first.clone();
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        hit_delay_us,
        ..
    } = &mut owner_second
    {
        *timestamp_us = 1;
        *hit_delay_us = 199;
    }
    let switch = GameActionCommand::SwitchActor {
        timestamp_us: 10,
        target_actor_index: 1,
        cooldown_us: 0,
    };
    let mut ally_hit = owner_first.clone();
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index,
        hit_delay_us,
        ..
    } = &mut ally_hit
    {
        *timestamp_us = 20;
        *actor_index = 1;
        *hit_delay_us = 280;
    }
    request.actions = vec![owner_first, owner_second, switch, ally_hit];
    for command in &mut request.actions {
        if let GameActionCommand::InvokeAbility { scaling_stat, .. } = command {
            *scaling_stat = crate::simulation::NativeScalingStat::Atk;
        }
    }
    request
        .action_metadata
        .insert(1, request.action_metadata[&0].clone());
    request
        .action_metadata
        .insert(3, request.action_metadata[&0].clone());
    let packets =
        crate::simulation::validation::validate_and_resolve(&request, &ExecutionLimits::default())
            .unwrap();
    let mut state = GameSimulatorState::new(&request);
    let (mut queue, mut sequence, _) = crate::simulation::queue::initial_queue(&request, &[], &[]);
    let mut trace = Vec::new();
    let mut rng = None;
    let mut hits = Vec::new();
    let mut ledger = Vec::new();
    while let Some(event) = queue.pop() {
        state.advance_unpaused_test_clock(event.time_us);
        if matches!(
            event.data,
            crate::simulation::events::GameEventData::HitDelivery { actor_index: 0, .. }
        ) {
            // Player HP mutations are not events yet; inject changed vitals between admitted deliveries.
            if event.time_us == 100 {
                state.actors[0].hp = 999;
            } else if event.time_us == 200 {
                state.actors[0].hp = 1001;
            }
        }
        let mut ctx = EventContext {
            request: &request,
            prepared_packets: &packets,
            state: &mut state,
            queue: &mut queue,
            sequence: &mut sequence,
            trace: &mut trace,
            rng: &mut rng,
            hits: &mut hits,
            action_metadata_ledger: &mut ledger,
        };
        match event.data {
            crate::simulation::events::GameEventData::ActionAdmission { command_index } => {
                assert!(
                    crate::simulation::admission::admit(&mut ctx, command_index)
                        .unwrap()
                        .is_none()
                );
            }
            crate::simulation::events::GameEventData::SwitchActor {
                target_actor_index, ..
            } => {
                ctx.state.active_actor_index = target_actor_index;
            }
            crate::simulation::events::GameEventData::HitDelivery {
                command_index,
                actor_index,
                target_index,
                ability_id,
                damage_effect_id,
                atk_rate,
                scaling_stat,
                metadata,
            } => {
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
            crate::simulation::events::GameEventData::ReactionScorchRefresh { .. }
            | crate::simulation::events::GameEventData::CinematicRelease { .. }
            | crate::simulation::events::GameEventData::FollowupHit { .. }
            | crate::simulation::events::GameEventData::DaffodilStepAfterSupport { .. }
            | crate::simulation::events::GameEventData::DummyCombat { .. }
            | crate::simulation::events::GameEventData::Content(_)
            | crate::simulation::events::GameEventData::IncomingDamage { .. }
            | crate::simulation::events::GameEventData::HpCost { .. }
            | crate::simulation::events::GameEventData::ActorInterrupt { .. }
            | crate::simulation::events::GameEventData::ActorParry { .. }
            | crate::simulation::events::GameEventData::CombatTransition { .. }
            | crate::simulation::events::GameEventData::LiveBranch { .. }
            | crate::simulation::events::GameEventData::LiveEnergyWait { .. }
            | crate::simulation::events::GameEventData::ResumeLiveEnergyWait
            | crate::simulation::events::GameEventData::TargetUnbalance { .. }
            | crate::simulation::events::GameEventData::TargetBreakDamage(_)
            | crate::simulation::events::GameEventData::TargetBreakRecovery { .. }
            | crate::simulation::events::GameEventData::ReactionPistil(_)
            | crate::simulation::events::GameEventData::ReactionScorchTick(_)
            | crate::simulation::events::GameEventData::ReactionNovaDetonate(_)
            | crate::simulation::events::GameEventData::ReactionNovaEnd(_)
            | crate::simulation::events::GameEventData::ReactionRemoraEnd { .. } => {
                unreachable!()
            }
        }
    }
    let damage = hits
        .iter()
        .map(|hit| hit.rounded_damage)
        .collect::<Vec<_>>();
    assert_eq!(damage, vec![1000, 1200, 1000]);
    assert_eq!(state.active_actor_index, 1);
}

#[test]
fn forgotten_atk_branch_applies_above_half_on_atk_packets() {
    // Hand-derived (T1 atk_bonus = 1/5, base-scaled per damage-and-stat-zones §1):
    // hp=1500/2000 -> ATK branch -> ATK_live = 1000 + 1000*1/5 = 1200.
    // hp=1000/2000 -> equality -> no branch -> 1000.
    // hp=999/2000 -> DEF branch -> ATK-scaled packet untouched -> 1000.
    for (hp, expected) in [(1500, 1200), (1000, 1000), (999, 1000)] {
        let mut scenario = request();
        if let GameActionCommand::InvokeAbility { scaling_stat, .. } = &mut scenario.actions[0] {
            *scaling_stat = crate::simulation::NativeScalingStat::Atk;
        }
        scenario.actors[0].initial_hp = hp;
        scenario.actors[0].initial_hp_max = 2000;
        let report = run_game_scenario(&scenario, &ExecutionLimits::default()).unwrap();
        let hit = report["hit_receipts"][0]["rounded_damage"]
            .as_u64()
            .unwrap();
        assert_eq!(hit, expected, "hp {hp}/2000");
    }
}

#[test]
fn forgotten_zone_math_hand_derived_t1_t5() {
    fn gcd(mut a: u128, mut b: u128) -> u128 {
        while b != 0 {
            let rest = a % b;
            a = b;
            b = rest;
        }
        a
    }
    fn reduced(n: u128, d: u128) -> (u128, u128) {
        let g = gcd(n, d);
        (n / g, d / g)
    }
    // T1 atk_bonus 1/5 is base-scaled and joins the live atk_percent zone additively.
    // Against a pre-existing live 1/10: 1000 * (1 + 1/10 + 1/5) = 1000 * 13/10 = 1300.
    assert_eq!(reduced(13, 10), (13, 10));
    let additive = 1000 * 13 / 10;
    assert_eq!(additive, 1300);
    // Multiplicative misreading 1000 * 11/10 * 6/5 = 1320 is rejected.
    assert_ne!(additive, 1000 * 11 / 10 * 6 / 5);
    // Double-application misreading 1000 * (6/5)^2 = 1440 is rejected.
    assert_ne!(additive, 1000 * 36 / 25);
    // T5 atk_bonus 9/25: 1000 + 1000 * 9/25 = 1360.
    assert_eq!(reduced(1000 + 1000 * 9 / 25, 1), (1360, 1));
}
