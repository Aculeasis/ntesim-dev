// whale weapon tests (implemented by task agent)
use super::*;
use crate::runtime_api::{DamageContext, HitModifiers};
use crate::simulation::state::GameSimulatorState;
use serde_json::json;

fn t1_proc() -> GearProcConfig {
    GearProcConfig::WhaleAtk {
        atk_bonus: Ratio {
            numerator: 3,
            denominator: 25,
        },
        broken_damage_bonus: Ratio {
            numerator: 3,
            denominator: 25,
        },
        kill_heal: Ratio {
            numerator: 3,
            denominator: 10,
        },
        kill_heal_cooldown_us: 30_000_000,
    }
}

fn projection_tables() -> std::collections::BTreeMap<String, serde_json::Value> {
    super::support::weapon_projection_tables()
}

#[test]
fn whale_projection_binds_five_curves_rejects_group_and_unit_changes() {
    let tables = projection_tables();
    let req = research_request();
    let mut actor = req.actors[0].clone();
    actor.character_id = "character:1072".into();
    actor.weapon_id = "weapon:fork_Whale".into();
    actor.gear_procs.clear();
    for tier in 1..=5u32 {
        actor.weapon_tier = tier;
        let report = crate::weapons::whale::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            if tier == 1 { &[] } else { &[0] },
        )
        .unwrap();
        let proc = &report["compiled_proc"];
        let ratio_of = |key: &str| {
            ExactFraction::from_ratio(serde_json::from_value(proc[key].clone()).unwrap()).unwrap()
        };
        let curve = ExactFraction::new(u128::from(10 + 2 * tier), 100).unwrap();
        assert_eq!(ratio_of("atk_bonus"), curve, "tier {tier} atk curve");
        assert_eq!(
            ratio_of("broken_damage_bonus"),
            curve,
            "tier {tier} broken curve"
        );
        assert_eq!(
            ratio_of("kill_heal"),
            ExactFraction::new(3, 10).unwrap(),
            "tier {tier} heal ratio"
        );
        assert_eq!(proc["kill_heal_cooldown_us"], json!(30_000_000));
        assert_eq!(actor.gear_procs.len(), 1);
        assert_eq!(report["replaced_proc"].is_null(), tier == 1);
        assert_eq!(
            report["native_description"]["localized"],
            json!(
                "Increases ATK by <lv>{0}</>.\nIncreases the wearer's damage to Broken enemies by <lv>{1}</>. Restores <lv>{2}</> HP to the wearer if a Broken enemy is defeated. Triggers at most <lv>once</> every <lv>{3}</>s."
            )
        );
    }

    let mut cross_group = req.actors[0].clone();
    cross_group.character_id = "character:1019".into();
    cross_group.weapon_id = "weapon:fork_Whale".into();
    cross_group.weapon_tier = 1;
    cross_group.gear_procs.clear();
    assert!(
        crate::weapons::whale::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut cross_group,
            &[]
        )
        .unwrap_err()
        .contains("incompatible")
    );

    let mut mutated = tables.clone();
    let row = &mut mutated.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_Whale_5"]
        ["DataList"];
    row[3]["bIsPercent"] = json!(true);
    let mut cooldown_actor = actor.clone();
    cooldown_actor.weapon_tier = 5;
    cooldown_actor.gear_procs.clear();
    assert!(
        crate::weapons::whale::projection::project(
            &mutated,
            1,
            "ntesim-current",
            &mut cooldown_actor,
            &[]
        )
        .unwrap_err()
        .contains("unit hint")
    );
}

fn atk_request() -> crate::simulation::GameScenarioRequest {
    let mut req = research_request();
    req.stochastic_crit = None;
    let actor = &mut req.actors[0];
    actor.character_id = "character:1072".into();
    actor.weapon_id = "weapon:fork_Whale".into();
    actor.weapon_tier = 1;
    actor.gear_procs = vec![t1_proc()];
    actor.static_stats.base_atk = 1000;
    actor.static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    actor.static_stats.crit_rate = Ratio::zero();
    actor.static_stats.damage_bonus = Ratio::zero();
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.actions.truncate(1);
    req.action_metadata
        .retain(|index, _| *index < req.actions.len());
    req
}

#[test]
fn whale_atk_bonus_joins_base_scaled_atk_sum_unconditionally() {
    let req = atk_request();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // 1000 static ATK + 1000 base ATK * 12/100 = 1120; M_bonus stays 1.
    assert_eq!(
        report["hit_receipts"][0]["non_crit_damage"],
        json!("1120/1")
    );
    assert_eq!(report["hit_receipts"][0]["rounded_damage"], 1120);
    assert!(
        report["final_target_states"][0]["hp_current"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert_eq!(
        report["resolved_model_assumptions"][crate::weapons::whale::HYPOTHESIS_ID.as_str()],
        crate::weapons::whale::HYPOTHESIS_ID.description()
    );

    let mut bare = req.clone();
    bare.actors[0].gear_procs.clear();
    let baseline = run_game_scenario(&bare, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        baseline["hit_receipts"][0]["non_crit_damage"],
        json!("1000/1")
    );
    assert!(
        baseline["resolved_model_assumptions"][crate::weapons::whale::HYPOTHESIS_ID.as_str()]
            .is_null()
    );
}

#[test]
fn whale_initialization_records_the_model_without_actions_and_rejects_conflicts() {
    let mut req = atk_request();
    req.actions.clear();
    req.action_metadata.clear();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hit_receipts"], json!([]));
    assert_eq!(
        report["callback_model_assumptions"][crate::weapons::whale::HYPOTHESIS_ID.as_str()],
        crate::weapons::whale::HYPOTHESIS_ID.description()
    );
    assert_eq!(
        report["resolved_model_assumptions"][crate::weapons::whale::HYPOTHESIS_ID.as_str()],
        crate::weapons::whale::HYPOTHESIS_ID.description()
    );
    req.model_assumptions.insert(
        crate::weapons::whale::HYPOTHESIS_ID.into(),
        "Conflicting Whale model".into(),
    );
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("conflicting reached model hypothesis weapons.whale_v1")
    );
}

#[test]
fn whale_unbroken_kill_never_heals_but_records_the_hypothesis() {
    let mut req = atk_request();
    req.targets[0].hp_max = 1;
    req.targets[0].hp_current = 1;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["final_target_states"][0]["hp_current"], 0);
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| event["event_kind"] != "whale_heal")
    );
    assert_eq!(
        report["resolved_model_assumptions"][crate::weapons::whale::HYPOTHESIS_ID.as_str()],
        crate::weapons::whale::HYPOTHESIS_ID.description()
    );
}

fn apply_broken(
    buffs: &mut ntesim_core::buffs::dispatch::BuffManager,
    target_index: usize,
    duration_us: u64,
) {
    crate::simulation::state::apply_target_tagged_status(
        buffs,
        crate::simulation::state::TargetStatusDeclaration {
            source_actor: 0,
            source: "test".into(),
            target_index,
            id: "test-broken".into(),
            now_us: 0,
            duration_us,
            reductions: BTreeMap::new(),
        },
        BTreeSet::from([crate::weapons::whale::BROKEN_STATUS_TAG.into()]),
        1,
        ntesim_core::buffs::dispatch::RefreshPolicy::Reset,
        ntesim_core::buffs::dispatch::DecayPolicy::AllAtOnce,
    )
    .unwrap();
}

fn zone_modifiers(
    req: &crate::simulation::GameScenarioRequest,
    state: &GameSimulatorState,
    target_index: usize,
) -> HitModifiers {
    let action_metadata = super::support::declared_test_action(
        "GA_Test",
        crate::simulation::metadata::ActionCategory::Melee,
    );
    let packet = crate::simulation::metadata::PacketMetadata {
        damage_effect_id: "GE_Test_Damage".into(),
        native_ga_name: None,
        native_damage_type_ex: "EExecutionDamageTypeEX::DAMAGE_TYPE_NATURE".into(),
        native_damage_source_category: None,
        damage_kind: ntesim_core::combat::DamageKind::Elemental,
        element: None,
        fixed_crit_rate: None,
        unbal_value: None,
        fixed_crit_rate_provenance: None,
        unbal_value_provenance: None,
        semantic_tags: BTreeSet::new(),
        semantic_provenance: None,
        source_evidence: None,
        provenance: crate::simulation::metadata::MetadataProvenance::AcceptedModelHypothesis {
            id: "test".into(),
            description: "Test-owned packet metadata.".into(),
        },
        packet_kind: Default::default(),
    };
    let owned_status_counts = BTreeMap::new();
    let buff_stats = BTreeMap::new();
    let ctx = DamageContext {
        foreign_dot_stacks: 0,
        packet_group_id: None,
        command_index: 0,
        config: &req.actors[0],
        start_fons: None,
        ability_id: "GA_Test",
        action_metadata: &action_metadata,
        target: &state.targets[target_index],
        actor: &state.actors[0],
        world_time_us: 1,
        time_us: 1,
        debuff_count: 0,
        damage_effect_id: "GE_Test_Damage",
        actor_index: 0,
        active_actor_index: 0,
        target_count: 2,
        non_boss_target_count: 2,
        target_index,
        buffs: &state.buffs,
        buff_stats: (&buff_stats).into(),
        owned_status_counts: &owned_status_counts,
        team_has_three_elements: false,
    };
    crate::gear::damage_bonus(
        &ctx,
        &packet,
        HitModifiers::new(ExactFraction::new(0, 1).unwrap()).unwrap(),
    )
    .unwrap()
}

#[test]
fn whale_broken_damage_bonus_is_additive_and_broken_gated() {
    let mut req = atk_request();
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    let mut state = GameSimulatorState::new(&req);
    apply_broken(&mut state.buffs, 1, 30_000_000);

    let plain = zone_modifiers(&req, &state, 0);
    assert_eq!(
        plain.atk_percent,
        ExactFraction::new(3, 25).unwrap(),
        "unconditional ATK bonus applies without Broken"
    );
    assert!(
        plain.damage_bonus.is_zero(),
        "no Broken damage bonus against an unbroken target"
    );

    let broken = zone_modifiers(&req, &state, 1);
    assert_eq!(broken.atk_percent, ExactFraction::new(3, 25).unwrap());
    assert_eq!(
        broken.damage_bonus,
        ExactFraction::new(3, 25).unwrap(),
        "Broken damage bonus joins the additive damage zone"
    );
}

fn kill_request() -> crate::simulation::GameScenarioRequest {
    let mut req = atk_request();
    req.actors[0].initial_hp = 50;
    req.actors[0].initial_hp_max = 100;
    req.targets[0].hp_max = 1;
    req.targets[0].hp_current = 1;
    req
}

fn schedule_defeat(
    req: &crate::simulation::GameScenarioRequest,
    state: &mut GameSimulatorState,
    queue: &mut crate::simulation::timer_queue::EventQueue,
    sequence: &mut u64,
    trace: &mut Vec<crate::simulation::GameEventReceipt>,
    actor_index: usize,
    target_index: usize,
    time_us: u64,
) {
    state.advance_unpaused_test_clock(time_us);
    crate::gear::on_target_defeated(
        &mut crate::runtime_api::CallbackContext {
            state,
            request: req,
            queue,
            sequence,
            trace,
        },
        actor_index,
        target_index,
    )
    .unwrap();
}

fn apply_queued_heals(
    req: &crate::simulation::GameScenarioRequest,
    state: &mut GameSimulatorState,
    queue: &mut crate::simulation::timer_queue::EventQueue,
    sequence: &mut u64,
    trace: &mut Vec<crate::simulation::GameEventReceipt>,
) {
    while let Some(scheduled) = queue.pop() {
        let crate::simulation::events::GameEventData::Content(event) = scheduled.data else {
            panic!("Song of the Whale queued a non-content callback")
        };
        crate::callbacks::apply(
            event,
            &mut crate::runtime_api::CallbackContext {
                state,
                request: req,
                queue,
                sequence,
                trace,
            },
        )
        .unwrap();
    }
}

#[test]
fn whale_heals_broken_kill_inside_exact_icd_window_and_isolates_wearers() {
    let mut req = kill_request();
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1010".into();
    ally.initial_hp = 40;
    req.actors.push(ally);

    let mut state = GameSimulatorState::new(&req);
    apply_broken(&mut state.buffs, 0, 120_000_000);
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();

    schedule_defeat(
        &req,
        &mut state,
        &mut queue,
        &mut sequence,
        &mut trace,
        0,
        0,
        0,
    );
    assert_eq!(queue.len(), 1);
    assert_eq!(
        state.execution_model_assumptions[crate::weapons::whale::HYPOTHESIS_ID.as_str()],
        crate::weapons::whale::HYPOTHESIS_ID.description()
    );
    schedule_defeat(
        &req,
        &mut state,
        &mut queue,
        &mut sequence,
        &mut trace,
        0,
        0,
        1,
    );
    schedule_defeat(
        &req,
        &mut state,
        &mut queue,
        &mut sequence,
        &mut trace,
        0,
        0,
        29_999_999,
    );
    assert_eq!(queue.len(), 1, "kills inside the 30s ICD share one trigger");
    schedule_defeat(
        &req,
        &mut state,
        &mut queue,
        &mut sequence,
        &mut trace,
        0,
        0,
        30_000_000,
    );
    assert_eq!(queue.len(), 2, "the exact ICD boundary is ready again");
    schedule_defeat(
        &req,
        &mut state,
        &mut queue,
        &mut sequence,
        &mut trace,
        1,
        0,
        30_000_000,
    );
    assert_eq!(
        queue.len(),
        3,
        "a second wearer has an independent cooldown"
    );

    apply_queued_heals(&req, &mut state, &mut queue, &mut sequence, &mut trace);
    // gross = round(100 * 3/10) = 30; wearer 0: 50 -> 80 -> 100 (capped).
    assert_eq!(state.actors[0].hp, 100);
    assert_eq!(state.actors[1].hp, 70);
    assert!(trace.iter().any(|event| {
        event.event_kind == "whale_heal"
            && event.from_actor == Some(0)
            && event.data.as_ref().is_some_and(|d| {
                d.target_index == Some(0)
                    && d.gross == Some(30)
                    && d.applied == Some(30)
                    && d.overflow == Some(0)
            })
    }));
    assert!(trace.iter().any(|event| {
        event.event_kind == "whale_heal"
            && event.from_actor == Some(0)
            && event.data.as_ref().is_some_and(|d| {
                d.target_index == Some(0)
                    && d.gross == Some(30)
                    && d.applied == Some(20)
                    && d.overflow == Some(10)
            })
    }));
    assert!(trace.iter().any(|event| {
        event.event_kind == "whale_heal"
            && event.from_actor == Some(1)
            && event.data.as_ref().is_some_and(|d| {
                d.target_index == Some(0)
                    && d.gross == Some(30)
                    && d.applied == Some(30)
                    && d.overflow == Some(0)
            })
    }));
}

#[test]
fn whale_unbroken_defeat_queues_nothing_and_dead_wearer_cannot_revive() {
    let req = kill_request();
    let mut state = GameSimulatorState::new(&req);
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    schedule_defeat(
        &req,
        &mut state,
        &mut queue,
        &mut sequence,
        &mut trace,
        0,
        0,
        0,
    );
    assert!(queue.is_empty(), "an unbroken kill never triggers the heal");

    apply_broken(&mut state.buffs, 0, 60_000_000);
    state.actors[0].hp = 0;
    schedule_defeat(
        &req,
        &mut state,
        &mut queue,
        &mut sequence,
        &mut trace,
        0,
        0,
        0,
    );
    assert_eq!(
        queue.len(),
        1,
        "the trigger is consumed even for a dead wearer"
    );
    apply_queued_heals(&req, &mut state, &mut queue, &mut sequence, &mut trace);
    assert_eq!(state.actors[0].hp, 0);
    assert!(trace.iter().any(|event| {
        event.event_kind == "whale_heal"
            && event
                .data
                .as_ref()
                .is_some_and(|d| d.reason.as_deref() == Some("dead_wearer") && d.applied == Some(0))
    }));
}

#[test]
fn whale_validate_config_rejects_wrong_weapon_and_zero_cooldown() {
    let mut req = atk_request();
    req.actors[0].weapon_id = "weapon:fork_PaperPlane".into();
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires equipped")
    );

    let mut zero_cooldown = atk_request();
    zero_cooldown.actors[0].gear_procs = vec![GearProcConfig::WhaleAtk {
        atk_bonus: Ratio {
            numerator: 3,
            denominator: 25,
        },
        broken_damage_bonus: Ratio {
            numerator: 3,
            denominator: 25,
        },
        kill_heal: Ratio {
            numerator: 3,
            denominator: 10,
        },
        kill_heal_cooldown_us: 0,
    }];
    assert!(
        run_game_scenario(&zero_cooldown, &ExecutionLimits::default())
            .unwrap_err()
            .contains("positive kill-heal cooldown")
    );

    let mut duplicate = atk_request();
    let extra = duplicate.actors[0].gear_procs[0].clone();
    duplicate.actors[0].gear_procs.push(extra);
    assert!(
        run_game_scenario(&duplicate, &ExecutionLimits::default())
            .unwrap_err()
            .contains("duplicates")
    );
}

#[test]
fn runtime_recorder_records_and_validates_the_whale_hypothesis() {
    let mut req = atk_request();
    req.actors[0].gear_procs.clear();
    let mut reached = BTreeMap::new();
    crate::runtime_api::record_hypothesis(
        &req,
        &mut reached,
        crate::weapons::whale::HYPOTHESIS_ID,
        crate::weapons::whale::HYPOTHESIS_ID.description(),
    )
    .unwrap();
    assert_eq!(
        reached[crate::weapons::whale::HYPOTHESIS_ID.as_str()],
        crate::weapons::whale::HYPOTHESIS_ID.description()
    );
    req.model_assumptions.insert(
        crate::weapons::whale::HYPOTHESIS_ID.into(),
        "conflicting description".into(),
    );
    let rejected = crate::runtime_api::record_hypothesis(
        &req,
        &mut reached,
        crate::weapons::whale::HYPOTHESIS_ID,
        crate::weapons::whale::HYPOTHESIS_ID.description(),
    )
    .unwrap_err();
    assert!(rejected.contains("conflicting reached model hypothesis"));
}

fn t5_proc() -> GearProcConfig {
    GearProcConfig::WhaleAtk {
        atk_bonus: Ratio {
            numerator: 1,
            denominator: 5,
        },
        broken_damage_bonus: Ratio {
            numerator: 1,
            denominator: 5,
        },
        kill_heal: Ratio {
            numerator: 3,
            denominator: 10,
        },
        kill_heal_cooldown_us: 30_000_000,
    }
}

fn atk_request_with_proc(proc: GearProcConfig) -> crate::simulation::GameScenarioRequest {
    let mut req = atk_request();
    req.actors[0].gear_procs = vec![proc];
    req
}

#[test]
fn whale_t5_broken_hit_end_to_end() {
    // Hand-derived from frozen buff_Whale_AtkUp / buff_Whale_Up (T5 = 0.2 = 1/5):
    // static 1000, base 1000, 0 DEF/0 RES. ATK = 1000 + 1000*1/5 = 1200;
    // M_bonus against a Broken target = 1 + 1/5, so the Broken hit is
    // 1200 * 6/5 = 1440. Without the tag it is 1200; without the proc, 1000.
    //
    // This unit bridge isolates a Broken-tagged state and the end-to-end
    // 1200/1000 unbroken hits. whale_cycles independently exercises actual
    // authored gauges, Broken admission, ordinary bonuses and fatal Break HP
    // contributions through the native hit pipeline at all five refinements.
    let req = atk_request_with_proc(t5_proc());
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hit_receipts"][0]["rounded_damage"], 1200);

    let mut bare = req.clone();
    bare.actors[0].gear_procs.clear();
    let baseline = run_game_scenario(&bare, &ExecutionLimits::default()).unwrap();
    assert_eq!(baseline["hit_receipts"][0]["rounded_damage"], 1000);

    let mut tagged = req.clone();
    tagged.targets.push(tagged.targets[0].clone());
    tagged.targets[1].target_index = 1;
    let mut state = GameSimulatorState::new(&tagged);
    apply_broken(&mut state.buffs, 1, 30_000_000);

    let broken = zone_modifiers(&tagged, &state, 1);
    assert_eq!(
        broken.atk_percent,
        ExactFraction::new(1, 5).unwrap(),
        "T5 unconditional ATK bonus"
    );
    assert_eq!(
        broken.damage_bonus,
        ExactFraction::new(1, 5).unwrap(),
        "T5 Broken damage bonus joins the additive damage zone"
    );
    let plain = zone_modifiers(&tagged, &state, 0);
    assert_eq!(plain.atk_percent, ExactFraction::new(1, 5).unwrap());
    assert!(
        plain.damage_bonus.is_zero(),
        "no Broken damage bonus without the tag"
    );
}

#[test]
fn whale_teammate_kill_does_not_trigger_wearer_heal() {
    // Trigger credit is the killer's own delivered fatal hit:
    // simulation/hit/effects.rs calls gear::on_target_defeated with the
    // delivering actor_index, and the adapter only scans that actor's procs.
    // A defeat delivered by a teammate without the proc must not heal the
    // wearer, even though the wearer carries Song of the Whale.
    let mut req = atk_request();
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.weapon_id.clear();
    ally.gear_procs.clear();
    req.actors.push(ally);
    req.targets[0].hp_max = 1;
    req.targets[0].hp_current = 1;
    let metadata = req.action_metadata.remove(&0).unwrap();
    req.actions.clear();
    req.action_metadata.clear();
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 0,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    req.actions.push(GameActionCommand::InvokeAbility {
        timestamp_us: 10,
        actor_index: 1,
        ability_id: "caller-test".into(),
        array_index: 0,
        damage_effect_id: "caller-effect".into(),
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 0,
    });
    req.action_metadata.insert(1, metadata);

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["final_target_states"][0]["hp_current"], 0);
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| event["event_kind"] != "whale_heal"),
        "a teammate kill must not trigger the wearer's heal"
    );
}

#[test]
fn whale_heal_rounds_fractional_gross_once() {
    // COMBAT_FORMULA_CONTRACT §5/§6: gross is the exact fraction rounded once
    // to nearest integer, ties upward, then capped at missing HP.
    // Max 99 / HP 50, T1 kill_heal 3/10: 99 * 3/10 = 29.7 -> gross 30,
    // applied 30, HP 50 -> 80, overflow 0.
    let mut req = kill_request();
    req.actors[0].initial_hp_max = 99;
    let mut state = GameSimulatorState::new(&req);
    apply_broken(&mut state.buffs, 0, 60_000_000);
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    schedule_defeat(
        &req,
        &mut state,
        &mut queue,
        &mut sequence,
        &mut trace,
        0,
        0,
        0,
    );
    assert_eq!(queue.len(), 1);
    apply_queued_heals(&req, &mut state, &mut queue, &mut sequence, &mut trace);
    assert_eq!(state.actors[0].hp, 80);
    assert!(trace.iter().any(|event| {
        event.event_kind == "whale_heal"
            && event.from_actor == Some(0)
            && event.data.as_ref().is_some_and(|d| {
                d.target_index == Some(0)
                    && d.gross == Some(30)
                    && d.applied == Some(30)
                    && d.overflow == Some(0)
            })
    }));
}
