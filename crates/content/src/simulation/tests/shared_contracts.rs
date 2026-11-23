//! Independent contract oracles, deliberately using unequal base and final ATK.
use super::*;
use crate::runtime_api::{DamageContext, HitModifiers, RecipientContext};
use crate::simulation::state::GameSimulatorState;

fn value(v: &serde_json::Value) -> f64 {
    serde_json::from_value::<ExactFraction>(v.clone())
        .unwrap()
        .to_f64()
}

#[test]
fn aggregate_defense_ignore_preserves_exact_damage_below_at_and_above_the_cap() {
    let q = |n, d| ExactFraction::new(n, d).unwrap();
    for (buffs, effective_ignore, expected_damage) in [
        (q(0, 1), q(3, 4), q(800, 1)),
        (q(1, 4), q(1, 1), q(1000, 1)),
        (q(3, 10), q(1, 1), q(1000, 1)),
    ] {
        let ignore = crate::runtime_api::aggregate_defense_ignore(q(3, 4), buffs).unwrap();
        assert_eq!(ignore, effective_ignore);
        // Lv80: A=1080; DEF1080*(1-.75)=270 -> Fdef=4/5.
        // At/above 100% ignore, Fdef=1, never greater than one.
        let factor =
            ntesim_core::combat::exact::defense_factor(80, q(1080, 1), q(0, 1), ignore).unwrap();
        assert_eq!(q(1000, 1).mul(factor).unwrap(), expected_damage);
    }
    assert!(
        ntesim_core::combat::exact::defense_factor(80, q(1080, 1), q(0, 1), q(21, 20),).is_err()
    );
}

#[test]
fn atk_damage_ratio_and_final_multipliers_follow_independent_algebra() {
    let q = ExactFraction::new;
    // 150 final ATK includes 100 base; ATK+20% adds 20, ratio+30% adds 0.3.
    // This counterexample rejects substituting either channel with DMG+20/30%.
    let exact = ntesim_core::combat::exact::direct_damage(
        q(170, 1).unwrap(),
        q(23, 10).unwrap(),
        q(2, 5).unwrap(),
        q(4, 5).unwrap(),
        None,
    )
    .unwrap();
    assert_eq!(exact, q(10948, 25).unwrap());
    assert!(
        (exact.to_f64() - (150.0 + 100.0 * 0.2) * (2.0 + 0.3) * (1.0 + 0.4) * 0.8).abs() < 1e-9
    );
    let final_damage = exact.mul(q(5, 2).unwrap()).unwrap();
    assert_eq!(final_damage, q(5474, 5).unwrap());
}

#[test]
fn new_status_records_share_target_scope_refresh_and_expiry_without_id_branches() {
    let mut req = research_request();
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    let mut state = GameSimulatorState::new(&req);
    let reductions = BTreeMap::from([(
        "ECharacterElementType::CHARACTER_ELEMENT_TYPE_NATURE".into(),
        ExactFraction::new(1, 5).unwrap(),
    )]);
    // A newly invented content status uses the same lifecycle without registry changes.
    crate::simulation::state::apply_target_status(
        &mut state.buffs,
        crate::simulation::state::TargetStatusDeclaration {
            source_actor: 0,
            source: "test:carrier".into(),
            target_index: 0,
            id: "new_test_carrier".into(),
            now_us: 0,
            duration_us: 10,
            reductions: reductions.clone(),
        },
    )
    .unwrap();
    crate::simulation::state::apply_target_status(
        &mut state.buffs,
        crate::simulation::state::TargetStatusDeclaration {
            source_actor: 0,
            source: "test:carrier".into(),
            target_index: 0,
            id: "new_test_carrier".into(),
            now_us: 3,
            duration_us: 10,
            reductions: reductions.clone(),
        },
    )
    .unwrap();
    crate::simulation::state::apply_target_status(
        &mut state.buffs,
        crate::simulation::state::TargetStatusDeclaration {
            source_actor: 1,
            source: "test:carrier".into(),
            target_index: 0,
            id: "new_test_carrier".into(),
            now_us: 0,
            duration_us: 10,
            reductions,
        },
    )
    .unwrap();
    assert_eq!(
        crate::simulation::state::target_status_count(&state.buffs, 0),
        2
    );
    assert_eq!(
        crate::simulation::state::target_resistance_reduction(
            &state.buffs,
            0,
            "ECharacterElementType::CHARACTER_ELEMENT_TYPE_NATURE",
        )
        .unwrap(),
        ExactFraction::new(2, 5).unwrap()
    );
    assert_eq!(
        crate::simulation::state::target_resistance_reduction(&state.buffs, 0, "COSMOS").unwrap(),
        ExactFraction::new(0, 1).unwrap()
    );
    assert_eq!(
        crate::simulation::state::target_status_count(&state.buffs, 1),
        0
    );
    state.advance_unpaused_test_clock(10);
    state.expire_buffs(&mut Vec::new());
    assert_eq!(
        crate::simulation::state::target_status_count(&state.buffs, 0),
        1
    );
    state.advance_unpaused_test_clock(13);
    state.expire_buffs(&mut Vec::new());
    assert_eq!(
        crate::simulation::state::target_status_count(&state.buffs, 0),
        0
    );
    assert!(
        crate::simulation::state::apply_target_status(
            &mut state.buffs,
            crate::simulation::state::TargetStatusDeclaration {
                source_actor: 0,
                source: "test:overflow".into(),
                target_index: 0,
                id: "overflow".into(),
                now_us: u64::MAX,
                duration_us: 1,
                reductions: Default::default(),
            },
        )
        .is_err()
    );
}

#[test]
fn zankou_a6_uses_active_dot_stacks_on_each_target_with_a_twenty_stack_cap() {
    let mut req = research_request();
    req.actors[0].character_id = "character:1036".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    let mut state = GameSimulatorState::new(&req);
    let dot_tags = std::collections::BTreeSet::from([
        crate::characters::packet_semantics::DOT_DAMAGE_TAG.into(),
    ]);
    let apply_dot =
        |buffs: &mut ntesim_core::buffs::dispatch::BuffManager, target_index, owner, id: &str| {
            crate::simulation::state::apply_target_tagged_status(
                buffs,
                crate::simulation::state::TargetStatusDeclaration {
                    source_actor: owner,
                    source: format!("test-dot-owner-{owner}"),
                    target_index,
                    id: id.into(),
                    now_us: 0,
                    duration_us: 30_000_000,
                    reductions: BTreeMap::new(),
                },
                dot_tags.clone(),
                20,
                ntesim_core::buffs::dispatch::RefreshPolicy::Reset,
                ntesim_core::buffs::dispatch::DecayPolicy::AllAtOnce,
            )
            .unwrap();
        };
    apply_dot(&mut state.buffs, 0, 0, "dot-a");
    apply_dot(&mut state.buffs, 0, 0, "dot-a");
    apply_dot(&mut state.buffs, 0, 1, "dot-b");
    apply_dot(&mut state.buffs, 1, 1, "dot-b");
    // A non-DoT target status is present but does not enter the A6 stack count.
    crate::simulation::state::apply_target_tagged_status(
        &mut state.buffs,
        crate::simulation::state::TargetStatusDeclaration {
            source_actor: 0,
            source: "test-control".into(),
            target_index: 0,
            id: "stun".into(),
            now_us: 0,
            duration_us: 30_000_000,
            reductions: BTreeMap::new(),
        },
        std::collections::BTreeSet::from(["State.Control.Stun".into()]),
        1,
        ntesim_core::buffs::dispatch::RefreshPolicy::Reset,
        ntesim_core::buffs::dispatch::DecayPolicy::AllAtOnce,
    )
    .unwrap();

    fn score(
        req: &crate::simulation::GameScenarioRequest,
        state: &GameSimulatorState,
        target_index: usize,
    ) -> ExactFraction {
        let owned_status_counts = BTreeMap::new();
        let action_metadata = super::support::declared_test_action(
            "GA_Zankou_Skill",
            crate::simulation::metadata::ActionCategory::Skill,
        );
        let ctx = DamageContext {
            foreign_dot_stacks: 0,
            packet_group_id: None,
            command_index: 0,
            config: &req.actors[0],
            start_fons: None,
            ability_id: "GA_Zankou_Skill",
            action_metadata: &action_metadata,
            target: &state.targets[target_index],
            actor: &state.actors[0],
            world_time_us: 1,
            time_us: 1,
            debuff_count: 0,
            damage_effect_id: "GE_Player_Zankou_Skill3_2_Damage",
            actor_index: 0,
            active_actor_index: 0,
            target_count: 2,
            non_boss_target_count: 2,
            target_index,
            buffs: &state.buffs,
            buff_stats: Default::default(),
            owned_status_counts: &owned_status_counts,
            team_has_three_elements: false,
        };
        crate::characters::zankou::awakenings::damage_bonus(
            &ctx,
            HitModifiers::new(ExactFraction::new(0, 1).unwrap()).unwrap(),
        )
        .unwrap()
        .damage_bonus
    }
    assert_eq!(score(&req, &state, 0), ExactFraction::new(36, 100).unwrap());
    assert_eq!(score(&req, &state, 1), ExactFraction::new(12, 100).unwrap());
    for _ in 0..22 {
        apply_dot(&mut state.buffs, 0, 0, "dot-a");
    }
    assert_eq!(
        score(&req, &state, 0),
        ExactFraction::new(240, 100).unwrap()
    );
}

#[test]
fn zankou_dot_crit_passive_requires_a_living_source_while_snapshot_buffs_persist() {
    let mut req = research_request();
    req.actors[0].character_id = "character:1036".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
    req.actors[0].initial_hp = 0;
    let mut state = GameSimulatorState::new(&req);
    crate::registry::initialize_team_effect(
        &req.actors[0],
        0,
        &mut state.buffs,
        &req,
        &mut state.execution_model_assumptions,
    )
    .unwrap();
    state
        .buffs
        .apply_buff(
            ntesim_core::buffs::dispatch::BuffDefinition {
                stacking_scope: ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                semantic_tags: Default::default(),
                source_lifetime: ntesim_core::buffs::dispatch::BuffSourceLifetime::Snapshot,
                id: "ordinary_snapshot".into(),
                source: "character:1036".into(),
                recipient: ntesim_core::buffs::dispatch::BuffRecipient::TeamAll,
                duration_us: None,
                refresh_policy: ntesim_core::buffs::dispatch::RefreshPolicy::Disallow,
                max_stacks: 1,
                decay_policy: ntesim_core::buffs::dispatch::DecayPolicy::AllAtOnce,
                stat_modifiers: BTreeMap::from([("atk_add".into(), Ratio::one())]),
            },
            0,
            0,
        )
        .unwrap();

    let living = BTreeSet::from([0]);
    let initial_stats = state
        .buffs
        .get_stat_modifiers_for_actor_with_living_sources(0, 0, 0, &living)
        .unwrap();
    assert_eq!(
        initial_stats["crit_damage_bonus_dot"],
        Ratio {
            numerator: 1,
            denominator: 2
        }
    );

    state.actors[0].hp = 0;
    let living = BTreeSet::new();
    let after_death = state
        .buffs
        .get_stat_modifiers_for_actor_with_living_sources(0, 0, 0, &living)
        .unwrap();
    assert!(!after_death.contains_key("crit_damage_bonus_dot"));
    assert_eq!(after_death["atk_add"], Ratio::one());
}

#[test]
fn jiuyuan_a1_queries_generic_live_target_status_counts_in_its_owner() {
    let mut req = research_request();
    req.actors[0].character_id = "character:1055".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
    for index in 1..4 {
        let mut target = req.targets[0].clone();
        target.target_index = index;
        req.targets.push(target);
    }
    req.targets[3].hp_current = 0;
    let mut state = GameSimulatorState::new(&req);
    for target_index in 0..3 {
        crate::simulation::state::apply_target_status(
            &mut state.buffs,
            crate::simulation::state::TargetStatusDeclaration {
                source_actor: 0,
                source: "character:1055".into(),
                target_index,
                id: "jiuyuan.rose_pact".into(),
                now_us: 0,
                duration_us: 25,
                reductions: BTreeMap::new(),
            },
        )
        .unwrap();
    }
    // A different source's application and an application on a defeated target
    // must not increase this owner's live target count.
    crate::simulation::state::apply_target_status(
        &mut state.buffs,
        crate::simulation::state::TargetStatusDeclaration {
            source_actor: 1,
            source: "character:1055".into(),
            target_index: 0,
            id: "jiuyuan.rose_pact".into(),
            now_us: 0,
            duration_us: 25,
            reductions: BTreeMap::new(),
        },
    )
    .unwrap();
    crate::simulation::state::apply_target_status(
        &mut state.buffs,
        crate::simulation::state::TargetStatusDeclaration {
            source_actor: 0,
            source: "character:1055".into(),
            target_index: 3,
            id: "jiuyuan.rose_pact".into(),
            now_us: 0,
            duration_us: 25,
            reductions: BTreeMap::new(),
        },
    )
    .unwrap();
    let owned_status_counts =
        crate::simulation::state::owned_status_counts(&state.buffs, &state.targets, 0);
    assert_eq!(owned_status_counts["jiuyuan.rose_pact"], 3);

    let ctx = DamageContext {
        foreign_dot_stacks: 0,
        packet_group_id: None,
        command_index: 0,
        config: &req.actors[0],
        start_fons: None,
        ability_id: "GA_Kuhara_Melee",
        action_metadata: &super::support::declared_test_action(
            "GA_Kuhara_Melee",
            crate::simulation::metadata::ActionCategory::Melee,
        ),
        target: &state.targets[0],
        actor: &state.actors[0],
        world_time_us: 0,
        time_us: 0,
        debuff_count: 2,
        damage_effect_id: "GE_Player_Kuhara_Melee1_Damage",
        actor_index: 0,
        active_actor_index: 0,
        target_count: 3,
        non_boss_target_count: 3,
        target_index: 0,
        buffs: &state.buffs,
        buff_stats: Default::default(),
        owned_status_counts: &owned_status_counts,
        team_has_three_elements: false,
    };
    let modifiers = crate::characters::jiuyuan::awakenings::damage_bonus(
        &ctx,
        HitModifiers::new(ExactFraction::new(0, 1).unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(modifiers.atk_percent, ExactFraction::new(3, 20).unwrap());
}

#[test]
fn successful_sparse_run_expires_statuses_at_horizon_without_synthetic_events() {
    let mut req = research_request();
    req.actors[0].character_id = "character:1055".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
    req.actors[0].gear_procs.clear();
    req.stochastic_crit = None;
    req.action_metadata.clear();
    req.actions = vec![GameActionCommand::InvokeAbility {
        timestamp_us: 0,
        actor_index: 0,
        ability_id: "GA_Kuhara_Skill".into(),
        damage_effect_id: "GE_Player_Kuhara_Skill1_Damage".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 0,
    }];
    req.relative_duration_us = 30_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["success"], true);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hit_receipts"].as_array().unwrap().len(), 6);
    assert_eq!(
        report["final_target_states"][0]
            .get("statuses")
            .and_then(serde_json::Value::as_array)
            .map_or(0, Vec::len),
        0
    );
    let trace = report["trace"].as_array().unwrap();
    assert_eq!(trace.len(), 13);
    assert!(trace.iter().all(|event| event["time_us"] == 0));
    assert_eq!(
        trace
            .iter()
            .filter(|event| event["energy_change"]["source"] == "jiuyuan_hit")
            .count(),
        6
    );
}

#[test]
fn recipient_declarations_cover_alive_expiry_target_count_and_element_boundaries() {
    for (owner, element, atk, damage, crit) in [
        ("character:1042", "COSMOS", 0.2, 0.12, 0.3),
        ("character:1020", "COSMOS", 0.0, 0.0, 0.12),
        ("character:1023", "COSMOS", 0.2, 0.0, 0.0),
        ("character:1052", "COSMOS", 0.3, 0.15, 0.0),
        ("character:1072", "NATURE", 0.0, 0.3, 0.0),
        ("character:1072", "INCANTATION", 0.0, 0.3, 0.0),
        ("character:1072", "COSMOS", 0.0, 0.0, 0.0),
    ] {
        let mut req = research_request();
        req.actors[0].character_id = owner.into();
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
        let mut state = GameSimulatorState::new(&req);
        if let crate::characters::RuntimeState::Baicang(s) = &mut state.actors[0].character {
            s.domain_until_us = 10;
        }
        if owner == "character:1020" {
            state
                .apply_buff(
                    0,
                    crate::simulation::state::TimedBuffDeclaration::reset(
                        "haniel.paranormal_ace",
                        owner,
                        ntesim_core::buffs::dispatch::BuffRecipient::TeamAll,
                        10,
                        ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                        BTreeMap::from([(
                            "crit_damage_bonus".into(),
                            ExactFraction::new(3, 25).unwrap(),
                        )]),
                    ),
                )
                .unwrap();
        }
        if owner == "character:1072" {
            let mut stats = BTreeMap::new();
            for element in ["NATURE", "INCANTATION"] {
                stats.insert(
                    format!("damage_bonus:ECharacterElementType::CHARACTER_ELEMENT_TYPE_{element}"),
                    ExactFraction::new(3, 10).unwrap(),
                );
            }
            state
                .apply_buff(
                    0,
                    crate::simulation::state::TimedBuffDeclaration::reset(
                        "linko.resonance_field",
                        owner,
                        ntesim_core::buffs::dispatch::BuffRecipient::TeamAll,
                        10,
                        ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                        stats,
                    ),
                )
                .unwrap();
        }
        if owner == "character:1042" {
            state
                .apply_buff(
                    0,
                    crate::simulation::state::TimedBuffDeclaration::reset(
                        "blackbird.requies_animarum_team",
                        owner,
                        ntesim_core::buffs::dispatch::BuffRecipient::TeamAll,
                        10,
                        ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                        BTreeMap::from([
                            ("atk_percent".into(), ExactFraction::new(1, 5).unwrap()),
                            ("damage_bonus".into(), ExactFraction::new(3, 25).unwrap()),
                        ]),
                    ),
                )
                .unwrap();
        }
        let eval = |actor: &crate::simulation::state::ActorRuntime, time, count| {
            let mut result = crate::characters::team::modifiers(
                &RecipientContext {
                    source_index: 0,
                    source_config: &req.actors[0],
                    source_actor: actor,
                    team_actors: &state.actors,

                    time_us: time,
                    target_count: count,
                    packet: None,
                    request: Some(&req),
                },
                HitModifiers::new(ExactFraction::new(0, 1).unwrap()).unwrap(),
            )
            .unwrap();
            let stats = state
                .buffs
                .get_stat_modifiers_for_actor(0, 0, time)
                .unwrap();
            for (key, target) in [
                ("atk_percent", &mut result.atk_percent),
                ("damage_bonus", &mut result.damage_bonus),
                ("crit_damage_bonus", &mut result.crit_damage_bonus),
            ] {
                if let Some(value) = stats.get(key) {
                    *target = target
                        .add(ExactFraction::from_ratio(*value).unwrap())
                        .unwrap();
                }
            }
            let element_key =
                format!("damage_bonus:ECharacterElementType::CHARACTER_ELEMENT_TYPE_{element}");
            if let Some(value) = stats.get(&element_key) {
                result.damage_bonus = result
                    .damage_bonus
                    .add(ExactFraction::from_ratio(*value).unwrap())
                    .unwrap();
            }
            result
        };
        let result = eval(&state.actors[0], 9, 1);
        assert_eq!(result.atk_percent.to_f64(), atk, "{owner}");
        assert_eq!(result.damage_bonus.to_f64(), damage, "{owner}");
        assert_eq!(result.crit_damage_bonus.to_f64(), crit, "{owner}");
        let expired = eval(&state.actors[0], 10, 2);
        assert_eq!(expired.damage_bonus.to_f64(), 0.0, "{owner}");
        state.actors[0].hp = 0;
        // Owner-declared passive recipient effects are suppressed for a dead
        // source; manager-owned timed snapshots have their own lifecycle below.
        let dead = crate::characters::team::modifiers(
            &RecipientContext {
                source_index: 0,
                source_config: &req.actors[0],
                source_actor: &state.actors[0],
                team_actors: &state.actors,

                time_us: 9,
                target_count: 1,
                packet: None,
                request: Some(&req),
            },
            HitModifiers::new(ExactFraction::new(0, 1).unwrap()).unwrap(),
        )
        .unwrap();
        assert_eq!(dead.damage_bonus.to_f64(), 0.0);
        assert_eq!(dead.atk_percent.to_f64(), 0.0);
        assert_eq!(dead.crit_damage_bonus.to_f64(), 0.0);
    }
}

#[test]
fn recipient_lifecycle_distinguishes_dead_source_passives_from_timed_snapshots() {
    use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope};
    let mut req = research_request();
    req.actors[0].character_id = "character:1042".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    let mut recipient = req.actors[0].clone();
    recipient.slot = 1;
    recipient.character_id = "character:1020".into();
    req.actors.push(recipient);
    let mut state = GameSimulatorState::new(&req);
    state.actors[0].hp = 0;
    state
        .apply_buff(
            0,
            crate::simulation::state::TimedBuffDeclaration::reset(
                "hypothesis.detached_timed_snapshot",
                "character:1042",
                BuffRecipient::TeamAll,
                100,
                BuffStackingScope::PerSource,
                BTreeMap::from([(
                    "crit_damage_bonus".into(),
                    ExactFraction::new(1, 4).unwrap(),
                )]),
            ),
        )
        .unwrap();

    let passive = crate::characters::team::modifiers(
        &RecipientContext {
            source_index: 0,
            source_config: &req.actors[0],
            source_actor: &state.actors[0],
            team_actors: &state.actors,

            time_us: 0,
            target_count: 1,
            packet: None,
            request: Some(&req),
        },
        HitModifiers::new(ExactFraction::new(0, 1).unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(passive.crit_damage_bonus, ExactFraction::new(0, 1).unwrap());

    // Tested modeling hypothesis: generic timed applications retain their
    // captured value until expiry unless an owner declares source-death removal.
    let recipient_stats = state.buffs.get_stat_modifiers_for_actor(1, 0, 0).unwrap();
    assert_eq!(
        recipient_stats["crit_damage_bonus"],
        Ratio {
            numerator: 1,
            denominator: 4
        }
    );
}

#[test]
fn team_buff_damage_uses_each_recipients_base_atk_in_actual_execution() {
    for descriptor in crate::registry::character_descriptors().unwrap() {
        if descriptor.character_id == "character:1052" {
            continue;
        }
        let mut req = research_request();
        req.stochastic_crit = None;
        req.actors[0].character_id = "character:1052".into();
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
        let mut recipient = req.actors[0].clone();
        recipient.slot = 1;
        recipient.character_id = descriptor.character_id.into();
        recipient.awakening = ntesim_data::AwakeningSelection::prefix(0);
        recipient.static_stats.atk = Ratio {
            numerator: 1000,
            denominator: 1,
        };
        recipient.static_stats.base_atk = 100;
        req.actors.push(recipient);
        req.targets[0].defense = 0;
        req.targets[0].resistance = Ratio::zero();
        req.action_metadata.clear();
        req.actions = vec![
            GameActionCommand::SwitchActor {
                timestamp_us: 0,
                target_actor_index: 1,
                cooldown_us: 0,
            },
            GameActionCommand::InvokeAbility {
                timestamp_us: 1_310_000,
                actor_index: 1,
                ability_id: "new_test_action".into(),
                array_index: 0,
                damage_effect_id: "new_test_effect".into(),
                atk_rate: Ratio {
                    numerator: 2,
                    denominator: 1,
                },
                scaling_stat: crate::simulation::NativeScalingStat::Atk,
                energy_cost: 0,
                recovery_us: 0,
                cooldown_us: 0,
                hit_delay_us: 0,
            },
        ];
        let metadata_hypothesis = "test.synthetic_action";
        let metadata_description = "Test-only explicit metadata for a synthetic action.";
        req.model_assumptions
            .insert(metadata_hypothesis.into(), metadata_description.into());
        req.action_metadata.insert(
            1,
            crate::simulation::ResearchActionMetadata {
                hypothesis_id: metadata_hypothesis.into(),
                ability_id: "new_test_action".into(),
                damage_effect_id: "new_test_effect".into(),
                action_gameplay_tags: Some(vec!["Ability.Skill".into()]),
                input_action_role: None,
                damage_type_ex: Some("EExecutionDamageTypeEX::DAMAGE_TYPE_NATURE".into()),
                damage_source_category: None,
                fixed_crit_rate: Some(crate::simulation::ResearchFixedCritRate::NoOverride),
                unbal_value: None,
                packet_ga_name_binding: None,
            },
        );
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        // Base ATK 100 * 30% contributes exactly 30 ATK; DMG +15% is separate.
        assert!(
            (value(
                &report["hit_receipts"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|hit| hit["damage_effect_id"] == "new_test_effect")
                    .unwrap()["non_crit_damage"]
            ) - (1000.0 + 100.0 * 0.3) * 2.0 * 1.15)
                .abs()
                < 1e-9,
            "{}",
            descriptor.slug
        );
    }
}

#[test]
fn shared_and_per_source_buff_owners_use_distinct_manager_keys() {
    use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope};
    let req = research_request();
    let mut state = GameSimulatorState::new(&req);
    let stat = |numerator| {
        BTreeMap::from([(
            "atk_percent".into(),
            ExactFraction::new(numerator, 10).unwrap(),
        )])
    };

    state
        .apply_buff(
            0,
            crate::simulation::state::TimedBuffDeclaration::reset(
                "shared-test",
                "gear:test",
                BuffRecipient::TeamAll,
                100,
                BuffStackingScope::Shared,
                stat(1),
            ),
        )
        .unwrap();
    state
        .apply_buff(
            1,
            crate::simulation::state::TimedBuffDeclaration::reset(
                "shared-test",
                "gear:test",
                BuffRecipient::TeamAll,
                100,
                BuffStackingScope::Shared,
                stat(3),
            ),
        )
        .unwrap();
    let shared = state.buffs.get_stat_modifiers_for_actor(0, 0, 0).unwrap();
    assert_eq!(
        shared["atk_percent"],
        Ratio {
            numerator: 3,
            denominator: 10
        }
    );
    assert_eq!(state.buffs.instances().len(), 1);
    assert_eq!(state.buffs.instances()[0].owner_actor, 1);

    let mut state = GameSimulatorState::new(&req);
    state
        .apply_buff(
            0,
            crate::simulation::state::TimedBuffDeclaration::reset(
                "per-source-test",
                "character:test",
                BuffRecipient::TeamAll,
                100,
                BuffStackingScope::PerSource,
                stat(1),
            ),
        )
        .unwrap();
    state
        .apply_buff(
            1,
            crate::simulation::state::TimedBuffDeclaration::reset(
                "per-source-test",
                "character:test",
                BuffRecipient::TeamAll,
                100,
                BuffStackingScope::PerSource,
                stat(3),
            ),
        )
        .unwrap();
    let per_source = state.buffs.get_stat_modifiers_for_actor(0, 0, 0).unwrap();
    assert_eq!(
        per_source["atk_percent"],
        Ratio {
            numerator: 2,
            denominator: 5
        }
    );
    assert_eq!(state.buffs.instances().len(), 2);
}

#[test]
fn synthetic_action_categories_drive_gear_by_declaration_not_ability_name() {
    let mut req = research_request();
    req.actors[0].initial_energy = 0;
    req.actors[0].gear_procs = vec![GearProcConfig::MindRoyaleEnergy {
        energy_grant: 18,
        cooldown_us: 20_000_000,
    }];
    req.stochastic_crit = None;
    req.model_assumptions.insert(
        "test.misleading_action_names".into(),
        "Test-only explicit action and packet metadata for synthetic abilities.".into(),
    );
    req.actions = vec![
        GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_TestSkill".into(),
            array_index: 0,
            damage_effect_id: "effect_without_skill_suffix".into(),
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 0,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 1,
            actor_index: 0,
            ability_id: "GA_TestAbilityWithoutSkillSuffix".into(),
            array_index: 0,
            damage_effect_id: "effect_without_skill_suffix".into(),
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 0,
        },
    ];
    req.action_metadata.clear();
    for (index, (ability_id, tag)) in [
        ("GA_TestSkill", "Ability.Melee"),
        ("GA_TestAbilityWithoutSkillSuffix", "Ability.Skill"),
    ]
    .into_iter()
    .enumerate()
    {
        req.action_metadata.insert(
            index,
            crate::simulation::ResearchActionMetadata {
                hypothesis_id: "test.misleading_action_names".into(),
                ability_id: ability_id.into(),
                damage_effect_id: "effect_without_skill_suffix".into(),
                action_gameplay_tags: Some(vec![tag.into()]),
                input_action_role: Some(if tag == "Ability.Melee" {
                    crate::simulation::metadata::InputActionRole::Normal
                } else {
                    crate::simulation::metadata::InputActionRole::RedirectSkill
                }),
                damage_type_ex: Some("EExecutionDamageTypeEX::DAMAGE_TYPE_NATURE".into()),
                damage_source_category: None,
                fixed_crit_rate: Some(crate::simulation::ResearchFixedCritRate::NoOverride),
                unbal_value: None,
                packet_ga_name_binding: None,
            },
        );
    }

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 2);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 18);
    assert_eq!(
        super::support::hit_metadata(&report, &report["hit_receipts"][0])["action"]["categories"],
        serde_json::json!(["melee"])
    );
    assert_eq!(
        super::support::hit_metadata(&report, &report["hit_receipts"][1])["action"]["categories"],
        serde_json::json!(["skill"])
    );
    assert_eq!(
        report["resolved_model_assumptions"]["test.misleading_action_names"],
        "Test-only explicit action and packet metadata for synthetic abilities."
    );
}

#[test]
fn owner_ratio_bonus_applies_only_to_the_named_packet_and_not_sibling_damage() {
    let mut req = research_request();
    req.actors[0].character_id = "character:1008".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
    let state = GameSimulatorState::new(&req);
    let owned_status_counts = std::collections::BTreeMap::new();
    for (packet, expected) in [
        ("GE_Player_Skia_SkillShadowAtk_Damage", 0.6),
        ("GE_Player_Skia_Skill1_Damage", 0.0),
        ("GE_Player_Skia_Melee1_Damage", 0.0),
    ] {
        let ctx = DamageContext {
            foreign_dot_stacks: 0,
            packet_group_id: None,
            command_index: 0,
            config: &req.actors[0],
            start_fons: None,
            ability_id: "GA_Skia_Skill",
            action_metadata: &super::support::declared_test_action(
                "GA_Skia_Skill",
                crate::simulation::metadata::ActionCategory::Skill,
            ),
            target: &state.targets[0],
            actor: &state.actors[0],
            world_time_us: 0,
            time_us: 0,
            debuff_count: 0,
            damage_effect_id: packet,
            actor_index: 0,
            active_actor_index: 0,
            target_count: 1,
            non_boss_target_count: 1,
            target_index: 0,
            buffs: &state.buffs,
            buff_stats: Default::default(),
            owned_status_counts: &owned_status_counts,
            team_has_three_elements: false,
        };
        let modifier = crate::registry::damage_bonus(
            &ctx,
            HitModifiers::new(ExactFraction::new(0, 1).unwrap()).unwrap(),
        )
        .unwrap();
        assert_eq!(modifier.ratio_add.to_f64(), expected);
        assert!(modifier.damage_bonus.is_zero());
    }
}
