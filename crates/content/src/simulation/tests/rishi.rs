use crate::simulation::timer_queue::EventQueue as BinaryHeap;
// rishi weapon tests: Eclipse static ATK in the additive base-scaled zone,
// plus the Arc: Eclipse field / defeat-energy hooks (weapons.rishi_v2).
use super::*;
use crate::runtime_api::{CastContext, HitModifiers};
use crate::simulation::events::GameEventData;
use crate::simulation::metadata::{ActionCategory, InputActionRole};
use crate::simulation::state::GameSimulatorState;
use ntesim_core::combat::NegativeResistanceModel;
use serde_json::json;

fn t1_proc() -> GearProcConfig {
    GearProcConfig::RishiAtk {
        atk_bonus: Ratio {
            numerator: 3,
            denominator: 10,
        },
        eclipse_duration_us: 40_000_000,
        defeat_energy: 6,
        max_defeats: 5,
        arc_cooldown_us: 300_000_000,
    }
}

fn t5_proc() -> GearProcConfig {
    GearProcConfig::RishiAtk {
        atk_bonus: Ratio {
            numerator: 1,
            denominator: 2,
        },
        eclipse_duration_us: 40_000_000,
        defeat_energy: 10,
        max_defeats: 5,
        arc_cooldown_us: 300_000_000,
    }
}

fn projection_tables() -> std::collections::BTreeMap<String, serde_json::Value> {
    super::support::weapon_projection_tables()
}

#[test]
fn rishi_projection_binds_five_curves_t1_t5_rejects_group_and_unit_changes() {
    let tables = projection_tables();
    let req = research_request();
    let mut actor = req.actors[0].clone();
    actor.character_id = "character:1020".into();
    actor.weapon_id = "weapon:fork_rishi".into();
    actor.gear_procs.clear();
    // Hardcoded from the frozen CT_ForkBuff rows:
    // buff_rishi_AtkUp 0.30..0.50, buff_rishi_CD2 40s, buff_rishi_energy 6..10,
    // buff_rishi_stack 5, buff_rishi_CD 300s.
    let expected: [(u32, u128, u128, u64); 5] = [
        (1, 3, 10, 6),
        (2, 7, 20, 7),
        (3, 2, 5, 8),
        (4, 9, 20, 9),
        (5, 1, 2, 10),
    ];
    for (tier, numerator, denominator, energy) in expected {
        actor.weapon_tier = tier;
        let report = crate::weapons::rishi::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            if tier == 1 { &[] } else { &[0] },
        )
        .unwrap();
        let proc = &report["compiled_proc"];
        let bonus =
            ExactFraction::from_ratio(serde_json::from_value(proc["atk_bonus"].clone()).unwrap())
                .unwrap();
        assert_eq!(
            bonus,
            ExactFraction::new(numerator, denominator).unwrap(),
            "tier {tier} ATK curve"
        );
        assert_eq!(
            proc["eclipse_duration_us"], 40_000_000,
            "tier {tier}: buff_rishi_CD2 binds the 40s field window"
        );
        assert_eq!(
            proc["defeat_energy"], energy,
            "tier {tier}: buff_rishi_energy binds the flat per-defeat energy"
        );
        assert_eq!(
            proc["max_defeats"], 5,
            "tier {tier}: buff_rishi_stack binds the per-field defeat cap"
        );
        assert_eq!(
            proc["arc_cooldown_us"], 300_000_000,
            "tier {tier}: buff_rishi_CD binds the 300s arc cooldown"
        );
        assert!(
            report["native_description"]["localized"]
                .as_str()
                .unwrap()
                .contains("Increases wearer's ATK"),
            "native description preserved"
        );
    }
    assert_eq!(
        actor.gear_procs.len(),
        1,
        "projection replaces the tier-1 proc in place"
    );

    let mut cross_group = req.actors[0].clone();
    cross_group.character_id = "character:1019".into();
    cross_group.weapon_id = "weapon:fork_rishi".into();
    cross_group.weapon_tier = 1;
    cross_group.gear_procs.clear();
    assert!(
        crate::weapons::rishi::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut cross_group,
            &[]
        )
        .unwrap_err()
        .contains("incompatible")
    );

    let mut unit_actor = req.actors[0].clone();
    unit_actor.character_id = "character:1020".into();
    unit_actor.weapon_id = "weapon:fork_rishi".into();
    unit_actor.weapon_tier = 1;
    unit_actor.gear_procs.clear();
    let mut hint_tables = projection_tables();
    hint_tables.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_rishi_1"]
        ["DataList"][0]["bIsPercent"] = json!(false);
    assert!(
        crate::weapons::rishi::projection::project(
            &hint_tables,
            1,
            "ntesim-current",
            &mut unit_actor,
            &[]
        )
        .unwrap_err()
        .contains("unit hint")
    );

    let mut fractional_actor = req.actors[0].clone();
    fractional_actor.character_id = "character:1020".into();
    fractional_actor.weapon_id = "weapon:fork_rishi".into();
    fractional_actor.weapon_tier = 1;
    fractional_actor.gear_procs.clear();
    let mut fractional_tables = projection_tables();
    fractional_tables.get_mut("CT_ForkBuff").unwrap()["rows"]["buff_rishi_energy"]["Keys"][0]["Value"] =
        json!(6.5);
    assert!(
        crate::weapons::rishi::projection::project(
            &fractional_tables,
            1,
            "ntesim-current",
            &mut fractional_actor,
            &[]
        )
        .unwrap_err()
        .contains("quantization"),
        "fractional defeat energy must be rejected, never rounded"
    );
}

#[test]
fn rishi_config_rejects_invalid_proc() {
    let req = research_request();
    let limits = ExecutionLimits::default();
    let mut wrong_weapon = req.actors[0].clone();
    wrong_weapon.weapon_id = "weapon:fork_Whale".into();
    let bad_ctx = crate::gear::validation::ConfigContext {
        actor: &wrong_weapon,
        actor_index: 0,
        limits: &limits,
    };
    match crate::weapons::rishi::validate_config(&t1_proc(), &bad_ctx) {
        Ok(_) => panic!("rishi proc accepted on the wrong weapon"),
        Err(error) => assert!(error.contains("fork_rishi"), "unexpected error: {error}"),
    }
    let bad_ratio = GearProcConfig::RishiAtk {
        atk_bonus: Ratio {
            numerator: 1,
            denominator: 0,
        },
        eclipse_duration_us: 40_000_000,
        defeat_energy: 6,
        max_defeats: 5,
        arc_cooldown_us: 300_000_000,
    };
    let bad_ratio_ctx = crate::gear::validation::ConfigContext {
        actor: &req.actors[0],
        actor_index: 0,
        limits: &limits,
    };
    assert!(crate::weapons::rishi::validate_config(&bad_ratio, &bad_ratio_ctx).is_err());
    for (field, zeroed) in [
        (
            "eclipse_duration_us",
            GearProcConfig::RishiAtk {
                atk_bonus: Ratio {
                    numerator: 3,
                    denominator: 10,
                },
                eclipse_duration_us: 0,
                defeat_energy: 6,
                max_defeats: 5,
                arc_cooldown_us: 300_000_000,
            },
        ),
        (
            "defeat_energy",
            GearProcConfig::RishiAtk {
                atk_bonus: Ratio {
                    numerator: 3,
                    denominator: 10,
                },
                eclipse_duration_us: 40_000_000,
                defeat_energy: 0,
                max_defeats: 5,
                arc_cooldown_us: 300_000_000,
            },
        ),
        (
            "max_defeats",
            GearProcConfig::RishiAtk {
                atk_bonus: Ratio {
                    numerator: 3,
                    denominator: 10,
                },
                eclipse_duration_us: 40_000_000,
                defeat_energy: 6,
                max_defeats: 0,
                arc_cooldown_us: 300_000_000,
            },
        ),
        (
            "arc_cooldown_us",
            GearProcConfig::RishiAtk {
                atk_bonus: Ratio {
                    numerator: 3,
                    denominator: 10,
                },
                eclipse_duration_us: 40_000_000,
                defeat_energy: 6,
                max_defeats: 5,
                arc_cooldown_us: 0,
            },
        ),
    ] {
        let ctx = crate::gear::validation::ConfigContext {
            actor: &req.actors[0],
            actor_index: 0,
            limits: &limits,
        };
        assert!(
            crate::weapons::rishi::validate_config(&zeroed, &ctx).is_err(),
            "zero {field} must be rejected"
        );
    }
    let mut right_weapon = req.actors[0].clone();
    right_weapon.weapon_id = "weapon:fork_RISHI".into();
    let ci_ctx = crate::gear::validation::ConfigContext {
        actor: &right_weapon,
        actor_index: 0,
        limits: &limits,
    };
    let validation = crate::weapons::rishi::validate_config(&t1_proc(), &ci_ctx)
        .unwrap_or_else(|error| panic!("valid rishi proc rejected: {error}"));
    assert_eq!(validation.kind, "rishi_atk");
    assert_eq!(validation.duration_us, 40_000_000);
}

fn zone_modifiers(
    req: &crate::simulation::GameScenarioRequest,
    state: &GameSimulatorState,
    time_us: u64,
) -> HitModifiers {
    super::support::gear_zone_modifiers(req, state, time_us, 0)
}

fn rishi_request(proc: GearProcConfig) -> crate::simulation::GameScenarioRequest {
    let mut req = research_request();
    req.stochastic_crit = None;
    let actor = &mut req.actors[0];
    actor.character_id = "character:1020".into();
    actor.weapon_id = "weapon:fork_rishi".into();
    actor.weapon_tier = 1;
    actor.gear_procs = vec![proc];
    actor.static_stats.base_atk = 1000;
    actor.static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    actor.static_stats.crit_rate = Ratio::zero();
    actor.static_stats.damage_bonus = Ratio::zero();
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req
}

fn cast_with_role(
    req: &crate::simulation::GameScenarioRequest,
    state: &mut GameSimulatorState,
    time_us: u64,
    role: Option<InputActionRole>,
) -> BinaryHeap {
    let mut metadata = super::support::declared_test_action("caller-test", ActionCategory::Skill);
    metadata.input_role = role;
    let mut assumptions = BTreeMap::new();
    let mut queue = BinaryHeap::new();
    let mut sequence = 0u64;
    let mut ctx = CastContext {
        origins: &mut state.origins,
        command_index: 0,
        native_array_index: None,
        execution_assumptions: &mut assumptions,
        config: &req.actors[0],
        request: req,
        actor_index: 0,
        active_actor_index: 0,
        target_index: 0,
        ability_id: "caller-test",
        action_metadata: &metadata,
        packet_metadata: None,
        hit_time_us: time_us,
        world_time_us: time_us,
        time_us,
        buffs: &mut state.buffs,
        actor: &mut state.actors[0],
        stamina: state.combat_stamina.as_mut(),
        team_hp_snapshot: &[],
        targets: &[],
        queue: &mut queue,
        sequence: &mut sequence,
    };
    crate::gear::on_cast(&mut ctx).unwrap();
    queue
}

fn activate_arc(
    req: &crate::simulation::GameScenarioRequest,
    state: &mut GameSimulatorState,
    time_us: u64,
) -> BinaryHeap {
    let mut queue = BinaryHeap::new();
    state.advance_unpaused_test_clock(time_us);
    let (duration, cooldown) = crate::weapons::rishi::arc_parameters(&req.actors[0]).unwrap();
    let Ok(timing) =
        state.prepare_ability_admission(0, crate::weapons::rishi::ARC_ABILITY, 0, 0, cooldown)
    else {
        return queue;
    };
    state.actors[0].ability_cooldowns.insert(
        crate::weapons::rishi::ARC_ABILITY.into(),
        timing.ready_at_us,
    );
    crate::weapons::rishi::activate_arc(
        &mut crate::runtime_api::CallbackContext {
            state,
            request: req,
            queue: &mut queue,
            sequence: &mut 0,
            trace: &mut Vec::new(),
        },
        0,
        duration,
    )
    .unwrap();
    queue
}

fn drain_queue(
    req: &crate::simulation::GameScenarioRequest,
    state: &mut GameSimulatorState,
    queue: &mut BinaryHeap,
) -> Vec<crate::simulation::GameEventReceipt> {
    let mut trace = Vec::new();
    let mut sequence = 0u64;
    while let Some(scheduled) = queue.pop() {
        let time_us = scheduled.time_us;
        let GameEventData::Content(event) = scheduled.data else {
            panic!("rishi queued a non-content callback");
        };
        state.advance_unpaused_test_clock(time_us);
        crate::callbacks::apply(
            event,
            &mut crate::runtime_api::CallbackContext {
                state,
                request: req,
                queue,
                sequence: &mut sequence,
                trace: &mut trace,
            },
        )
        .unwrap();
    }
    trace
}

fn defeat(
    req: &crate::simulation::GameScenarioRequest,
    state: &mut GameSimulatorState,
    actor_index: usize,
    target_index: usize,
    time_us: u64,
) -> BinaryHeap {
    state.advance_unpaused_test_clock(time_us);
    let mut queue = BinaryHeap::new();
    let mut sequence = 0u64;
    let mut trace = Vec::new();
    crate::gear::on_target_defeated(
        &mut crate::runtime_api::CallbackContext {
            state,
            request: req,
            queue: &mut queue,
            sequence: &mut sequence,
            trace: &mut trace,
        },
        actor_index,
        target_index,
    )
    .unwrap();
    queue
}

fn eclipse(req: &GameSimulatorState) -> &crate::weapons::rishi::State {
    &req.actors[0].equipment.rishi
}

#[test]
fn rishi_explicit_arc_creates_field() {
    let req = rishi_request(t1_proc());
    let mut state = GameSimulatorState::new(&req);
    // Hand-derived oracle: cast at t=1_000_000 opens the field
    // [1_000_000, 41_000_000) and locks the arc until 301_000_000.
    let mut queue = activate_arc(&req, &mut state, 1_000_000);
    assert_eq!(eclipse(&state).field_until_us, 41_000_000);
    assert_eq!(eclipse(&state).defeats_used, 0);
    assert_eq!(
        state.actors[0]
            .ability_cooldowns
            .get(crate::weapons::rishi::ARC_ABILITY)
            .copied()
            .unwrap_or(0),
        301_000_000
    );
    let trace = drain_queue(&req, &mut state, &mut queue);
    assert_eq!(trace.len(), 1);
    assert_eq!(trace[0].event_kind, "rishi_eclipse_field");
    assert_eq!(trace[0].phase, "ProcCallback");
    assert_eq!(trace[0].from_actor, Some(0));
    assert_eq!(
        trace[0].data.as_ref().and_then(|d| d.expires_at_us),
        Some(41_000_000)
    );
    assert_eq!(trace[0].data.as_ref().and_then(|d| d.count), Some(0));
}

#[test]
fn rishi_non_ultimate_cast_creates_no_field() {
    let req = rishi_request(t1_proc());
    let mut state = GameSimulatorState::new(&req);
    let queue = cast_with_role(
        &req,
        &mut state,
        1_000_000,
        Some(InputActionRole::RedirectSkill),
    );
    assert_eq!(eclipse(&state).field_until_us, 0);
    assert_eq!(eclipse(&state).defeats_used, 0);
    assert_eq!(
        state.actors[0]
            .ability_cooldowns
            .get(crate::weapons::rishi::ARC_ABILITY)
            .copied()
            .unwrap_or(0),
        0
    );
    assert!(queue.is_empty(), "non-Ultimate casts schedule nothing");
}

fn source_arc(tier: u32, rotation: &str) -> GameScenarioRequest {
    let source = format!(
        "options iteration=1 duration=310 crit=fixed stamina_capacity=60 initial_stamina=60
        stamina_use_efficiency=1 stamina_recovery_rate=0;
        haniel char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
        haniel add weapon=fork_rishi refine={tier};
        target hp=100000000 def=0 resist=0; active haniel; {rotation}"
    );
    crate::dsl::compiler::compile_scenario(&source, Default::default())
        .unwrap()
        .request
}

#[test]
fn rishi_explicit_source_arc_repeats_without_cost_at_exact_cooldown_for_all_tiers() {
    for tier in 1..=5 {
        let req = source_arc(tier, "haniel arc; wait(300); haniel arc;");
        let report = run_game_scenario_report(&req, &ExecutionLimits::default()).unwrap();
        assert!(report.execution_completed);
        assert_eq!(report.player_input_receipts.len(), 2);
        assert!(report.hit_receipts.is_empty());
        assert_eq!(report.final_actor_states[0].spent_energy, 0);
        assert_eq!(report.final_actor_states[0].hp, req.actors[0].initial_hp);
        let fields: Vec<_> = report
            .trace
            .iter()
            .filter(|event| event.event_kind == "rishi_eclipse_field")
            .map(|event| event.time_us)
            .collect();
        assert_eq!(fields, [0, 300_000_000]);
        let early = source_arc(tier, "haniel arc; wait(299.999999); haniel arc;");
        let report = run_game_scenario_report(&early, &ExecutionLimits::default()).unwrap();
        assert!(!report.execution_completed);
        assert_eq!(report.player_input_receipts.len(), 1);
        assert!(matches!(
            report.rejected_action.unwrap().reason,
            GameAdmissionReason::Cooldown {
                until_us: 300_000_000
            }
        ));
    }
}

#[test]
fn rishi_explicit_arc_refuses_dead_off_field_and_recovering_owners() {
    for case in 0..3 {
        let mut req = source_arc(1, "haniel arc;");
        match case {
            0 => req.actors[0].initial_hp = 0,
            1 => {
                let mut teammate = req.actors[0].clone();
                teammate.slot = 1;
                teammate.character_id = "character:1019".into();
                teammate.weapon_id.clear();
                teammate.gear_procs.clear();
                req.actors.push(teammate);
                req.active_actor_index = 1;
            }
            _ => req.actions.insert(
                0,
                GameActionCommand::PlayerInput {
                    timestamp_us: 0,
                    actor_index: 0,
                    input: PlayerInput::Dash,
                    critical_success: false,
                },
            ),
        }
        let report = run_game_scenario_report(&req, &ExecutionLimits::default()).unwrap();
        assert!(!report.execution_completed);
        assert_eq!(report.final_actor_states[0].spent_energy, 0);
        assert!(
            !report
                .trace
                .iter()
                .any(|event| event.event_kind == "rishi_eclipse_field")
        );
    }
}

#[test]
fn rishi_source_arc_defeat_energy_uses_native_refinements_and_charge_efficiency() {
    for (tier, gross, remainder) in [
        (1u32, 7, 98),
        (2, 9, 31),
        (3, 10, 64),
        (4, 11, 97),
        (5, 13, 30),
    ] {
        let source = format!(
            "options iteration=1 duration=20 crit=fixed;
            haniel char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=120;
            haniel add weapon=fork_rishi refine={tier};
            target hp=1 def=0 resist=0; active haniel; haniel arc, burst;"
        );
        let source = crate::simulation::PreparedSourceScenario::new(
            &source,
            Default::default(),
            &ExecutionLimits::default(),
        )
        .unwrap()
        .run(42)
        .unwrap();
        assert!(
            source.report.success && source.report.execution_completed,
            "{:?}",
            source.diagnostic
        );
        super::assert_native_replay(&source);
        let report = source.report;
        let grants: Vec<_> = report
            .trace
            .iter()
            .filter_map(|event| match &event.energy_change {
                Some(GameEnergyReceipt::Grant(grant))
                    if matches!(grant.source, GameEnergySource::RishiEclipse) =>
                {
                    Some(grant)
                }
                _ => None,
            })
            .collect();
        assert_eq!(grants.len(), 1);
        // Lv80 native weapon gives +33% charge efficiency. Apply once to
        // flat 6/7/8/9/10 energy, with the fractional remainder retained.
        assert_eq!(grants[0].gross, gross);
        assert_eq!(
            grants[0].charge_remainder_after,
            Some(ExactFraction::new(remainder, 100).unwrap())
        );
        assert_eq!(
            grants[0].charge_basis,
            Some(ExactFraction::new((5 + tier).into(), 1).unwrap())
        );
        assert_eq!(report.final_actor_states[0].spent_energy, 120);
    }
}

#[test]
fn rishi_explicit_arc_rejects_cooldown_deadline_overflow() {
    let mut req = source_arc(1, "haniel arc;");
    req.relative_duration_us = u64::MAX - 300_000_000 + 1;
    req.absolute_limit_us = req.relative_duration_us;
    req.actions[0] = GameActionCommand::PlayerInput {
        timestamp_us: req.absolute_limit_us,
        actor_index: 0,
        input: PlayerInput::Arc,
        critical_success: false,
    };
    let limits = ExecutionLimits {
        max_horizon_us: u64::MAX,
        ..ExecutionLimits::default()
    };
    assert!(
        run_game_scenario_report(&req, &limits)
            .unwrap_err()
            .contains("Arc cooldown deadline overflow")
    );
}

#[test]
fn rishi_second_arc_within_cooldown_does_not_refresh() {
    let req = rishi_request(t1_proc());
    let mut state = GameSimulatorState::new(&req);
    let mut first = activate_arc(&req, &mut state, 1_000_000);
    drain_queue(&req, &mut state, &mut first);
    // Second Arc at t=2_000_000, inside the 300s arc lockout.
    let second = activate_arc(&req, &mut state, 2_000_000);
    assert_eq!(eclipse(&state).field_until_us, 41_000_000);
    assert_eq!(
        state.actors[0]
            .ability_cooldowns
            .get(crate::weapons::rishi::ARC_ABILITY)
            .copied()
            .unwrap_or(0),
        301_000_000
    );
    assert!(second.is_empty(), "locked arc schedules no field");
}

#[test]
fn rishi_new_field_after_arc_cooldown_resets_counter() {
    let req = rishi_request(t1_proc());
    let mut state = GameSimulatorState::new(&req);
    let mut first = activate_arc(&req, &mut state, 0);
    drain_queue(&req, &mut state, &mut first);
    let mut kill = defeat(&req, &mut state, 0, 0, 1_000_000);
    drain_queue(&req, &mut state, &mut kill);
    assert_eq!(eclipse(&state).defeats_used, 1);
    // Cast exactly at the lockout boundary is allowed: t >= arc_ready_at_us.
    let mut second = activate_arc(&req, &mut state, 300_000_000);
    assert_eq!(eclipse(&state).field_until_us, 340_000_000);
    assert_eq!(eclipse(&state).defeats_used, 0);
    assert_eq!(
        state.actors[0]
            .ability_cooldowns
            .get(crate::weapons::rishi::ARC_ABILITY)
            .copied()
            .unwrap_or(0),
        600_000_000
    );
    let trace = drain_queue(&req, &mut state, &mut second);
    assert_eq!(trace.len(), 1);
    assert_eq!(trace[0].event_kind, "rishi_eclipse_field");
}

#[test]
fn rishi_defeat_during_field_grants_flat_energy_with_source_receipt() {
    for (proc, expected) in [(t1_proc(), 6u64), (t5_proc(), 10u64)] {
        let req = rishi_request(proc);
        let mut state = GameSimulatorState::new(&req);
        let mut cast = activate_arc(&req, &mut state, 0);
        drain_queue(&req, &mut state, &mut cast);
        // Fixture starts at full energy (capacity 100): spend to make room for the grant.
        state.actors[0].spend_energy(50).unwrap();
        let before = state.actors[0].energy.current();
        let mut kill = defeat(&req, &mut state, 0, 0, 1_000_000);
        let trace = drain_queue(&req, &mut state, &mut kill);
        assert_eq!(
            state.actors[0].energy.current(),
            before + expected,
            "flat defeat energy is not scaled by anything in the fixture"
        );
        assert_eq!(eclipse(&state).defeats_used, 1);
        assert_eq!(trace.len(), 1);
        assert_eq!(trace[0].event_kind, "rishi_eclipse_energy");
        let change = trace[0]
            .energy_change
            .as_ref()
            .expect("energy grant must carry a receipt");
        let crate::simulation::GameEnergyReceipt::Grant(grant) = change else {
            panic!("expected an energy grant receipt");
        };
        assert!(matches!(
            grant.source,
            crate::simulation::GameEnergySource::RishiEclipse
        ));
        assert_eq!(grant.gross, expected);
        assert_eq!(grant.source_actor_index, 0);
        assert_eq!(grant.recipient_actor_index, 0);
    }
}

#[test]
fn rishi_eclipse_energy_runs_after_haniel_burst_defeats_target() {
    let mut req = super::haniel_actions::request();
    let actor = &mut req.actors[0];
    actor.character_id = "character:1020".into();
    actor.weapon_id = "weapon:fork_rishi".into();
    actor.weapon_tier = 1;
    actor.gear_procs = vec![t1_proc()];
    actor.static_stats.base_atk = 1000;
    actor.static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    actor.energy_capacity = 120;
    actor.initial_energy = 120;
    req.targets[0].hp_max = 3500;
    req.targets[0].hp_current = 3500;
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    let burst = super::haniel_actions::make_action(
        0,
        0,
        "GA_Haniel_UltraSkill",
        "GE_Player_Haniel_Ultraskill1_Damage",
        120,
        20_000_000,
    );
    req.actions = vec![
        GameActionCommand::PlayerInput {
            timestamp_us: 0,
            actor_index: 0,
            input: PlayerInput::Arc,
            critical_success: false,
        },
        burst.clone(),
    ];

    // Independent oracle: Rishi adds 1000*0.30 and Haniel's Lv1 Burst adds
    // 1000*0.10 Hootie ATK before its same-time hit: ATK_live = 1400.
    // Haniel A0 Lv1 Burst is 3.0x ATK, with 0 DEF/RES: 1400*3 = 4200.
    // It defeats 3500 HP, then T1 Eclipse grants 6 after the 120-cost cast.
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hit_receipts"].as_array().unwrap().len(), 1);
    assert_eq!(
        report["hit_receipts"][0]["non_crit_damage"],
        json!("4200/1")
    );
    assert_eq!(report["hit_receipts"][0]["applied_hp_loss"], 3500);
    assert_eq!(report["final_target_states"][0]["hp_current"], 0);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 120);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 6);
    assert_eq!(report["final_actor_states"][0]["energy"], 6);
    let energy_event = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["event_kind"] == "rishi_eclipse_energy")
        .expect("defeat callback emits the Eclipse energy receipt");
    let change = energy_event["energy_change"].as_object().unwrap();
    assert_eq!(change["source"], "rishi_eclipse");
    assert_eq!(change["gross"], 6);
    req.actions = vec![burst];
    let without_arc = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(without_arc["final_actor_states"][0]["granted_energy"], 0);
    assert!(
        !without_arc["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_kind"] == "rishi_eclipse_field")
    );
    assert_eq!(change["charge_basis"], json!("6/1"));
}

#[test]
fn rishi_defeat_cap_is_five_per_field() {
    let req = rishi_request(t1_proc());
    let mut state = GameSimulatorState::new(&req);
    let mut cast = activate_arc(&req, &mut state, 0);
    drain_queue(&req, &mut state, &mut cast);
    // Fixture starts at full energy (capacity 100): spend to make room for the grants.
    state.actors[0].spend_energy(50).unwrap();
    let before = state.actors[0].energy.current();
    // Hand-derived oracle: T1 grants 6 per defeat, 5 defeats per field: 5*6=30.
    for i in 0..5 {
        let mut kill = defeat(&req, &mut state, 0, 0, 1_000_000 + i);
        let trace = drain_queue(&req, &mut state, &mut kill);
        assert_eq!(trace.len(), 1, "defeat {} must grant", i + 1);
    }
    assert_eq!(state.actors[0].energy.current(), before + 30);
    assert_eq!(eclipse(&state).defeats_used, 5);
    // Sixth defeat inside the window grants nothing and queues nothing.
    let mut sixth = defeat(&req, &mut state, 0, 0, 2_000_000);
    assert!(sixth.is_empty());
    drain_queue(&req, &mut state, &mut sixth);
    assert_eq!(state.actors[0].energy.current(), before + 30);
    assert_eq!(eclipse(&state).defeats_used, 5);
}

#[test]
fn rishi_defeat_after_field_expiry_grants_nothing() {
    let req = rishi_request(t1_proc());
    let mut state = GameSimulatorState::new(&req);
    let mut cast = activate_arc(&req, &mut state, 0);
    drain_queue(&req, &mut state, &mut cast);
    let before = state.actors[0].energy.current();
    // The window is [t_admit, t_admit+40_000_000); the endpoint is exclusive.
    let mut kill = defeat(&req, &mut state, 0, 0, 40_000_000);
    assert!(kill.is_empty());
    drain_queue(&req, &mut state, &mut kill);
    assert_eq!(state.actors[0].energy.current(), before);
    assert_eq!(eclipse(&state).defeats_used, 0);
}

#[test]
fn rishi_pending_energy_rechecks_life_without_confusing_off_field_ownership() {
    for (proc, energy) in [(t1_proc(), 6), (t5_proc(), 10)] {
        for dead in [false, true] {
            let mut req = rishi_request(proc.clone());
            let mut teammate = req.actors[0].clone();
            teammate.slot = 1;
            teammate.gear_procs.clear();
            req.actors.push(teammate);
            let mut state = GameSimulatorState::new(&req);
            state.actors[0].spend_energy(50).unwrap();
            let mut cast = activate_arc(&req, &mut state, 0);
            drain_queue(&req, &mut state, &mut cast);
            let mut pending = defeat(&req, &mut state, 0, 0, 1_000_000);
            assert_eq!(pending.len(), 1);
            state.active_actor_index = 1;
            state.actors[0].charge_remainder = ExactFraction::new(1, 3).unwrap();
            if dead {
                state.actors[0].hp = 0;
            }
            let trace = drain_queue(&req, &mut state, &mut pending);
            assert_eq!(
                state.actors[0].energy.current(),
                50 + if dead { 0 } else { energy }
            );
            assert_eq!(
                state.actors[0].granted_energy,
                if dead { 0 } else { energy }
            );
            assert_eq!(state.actors[0].overflow_energy, 0);
            assert_eq!(
                state.actors[0].charge_remainder,
                ExactFraction::new(1, 3).unwrap()
            );
            assert_eq!(trace.len(), usize::from(!dead));
            assert_eq!(state.actors[1].granted_energy, 0);
            if dead {
                let used = eclipse(&state).defeats_used;
                assert!(defeat(&req, &mut state, 0, 0, 2_000_000).is_empty());
                assert_eq!(eclipse(&state).defeats_used, used);
            }
        }
    }
}

#[test]
fn rishi_pending_field_does_not_activate_for_a_dead_owner() {
    let req = rishi_request(t1_proc());
    let mut state = GameSimulatorState::new(&req);
    let mut pending = activate_arc(&req, &mut state, 0);
    state.actors[0].hp = 0;
    assert!(drain_queue(&req, &mut state, &mut pending).is_empty());
}

#[test]
fn rishi_teammate_credited_defeat_grants_nothing() {
    let mut req = rishi_request(t1_proc());
    let mut teammate = req.actors[0].clone();
    teammate.slot = 1;
    teammate.gear_procs.clear();
    req.actors.push(teammate);
    let mut state = GameSimulatorState::new(&req);
    let mut cast = activate_arc(&req, &mut state, 0);
    drain_queue(&req, &mut state, &mut cast);
    let before = state.actors[0].energy.current();
    // Teammate (actor 1) defeats the target: wearer-kill credit only.
    let mut kill = defeat(&req, &mut state, 1, 0, 1_000_000);
    assert!(kill.is_empty());
    drain_queue(&req, &mut state, &mut kill);
    assert_eq!(state.actors[0].energy.current(), before);
    assert_eq!(eclipse(&state).defeats_used, 0);
}

#[test]
fn rishi_atk_bonus_joins_atk_percent_zone() {
    let req = rishi_request(t1_proc());
    let state = GameSimulatorState::new(&req);
    let modifiers = zone_modifiers(&req, &state, 1);
    assert_eq!(
        modifiers.atk_percent,
        ExactFraction::new(3, 10).unwrap(),
        "T1 30% ATK bonus joins the ATK percent zone"
    );
    assert!(
        modifiers.damage_bonus.is_zero(),
        "static ATK bonus must not leak into the damage-bonus zone"
    );
    assert!(
        modifiers.final_bonus.is_zero(),
        "static ATK bonus must not leak into the final zone"
    );
}

#[test]
fn rishi_t5_atk_bonus_is_additive_not_multiplicative() {
    let req = rishi_request(t5_proc());
    let state = GameSimulatorState::new(&req);
    let modifiers = zone_modifiers(&req, &state, 1);
    assert_eq!(
        modifiers.atk_percent,
        ExactFraction::new(1, 2).unwrap(),
        "T5 50% ATK bonus joins the ATK percent zone as a plain addend"
    );
    // Independent counterexample: a multiplicative 1.5x static ATK would read
    // as final_bonus or a scaled base, never as atk_percent == 1/2.
    assert!(
        modifiers.final_bonus.is_zero(),
        "additive placement rejected: no final-zone multiplication"
    );
}

#[test]
fn rishi_no_bonus_without_proc() {
    let mut req = rishi_request(t1_proc());
    req.actors[0].gear_procs.clear();
    let state = GameSimulatorState::new(&req);
    let modifiers = zone_modifiers(&req, &state, 1);
    assert!(
        modifiers.atk_percent.is_zero(),
        "no ATK bonus without the rishi proc"
    );
}

#[test]
fn rishi_foreign_proc_is_ignored() {
    let mut req = rishi_request(t1_proc());
    req.actors[0].gear_procs = vec![GearProcConfig::SnowmanShieldAtk {
        atk_bonus: Ratio {
            numerator: 9,
            denominator: 50,
        },
    }];
    let state = GameSimulatorState::new(&req);
    let modifiers = zone_modifiers(&req, &state, 1);
    assert!(
        modifiers.atk_percent.is_zero(),
        "foreign procs must not contribute to the rishi ATK zone"
    );
}

fn settle_non_crit(stat: ExactFraction) -> ExactFraction {
    ntesim_core::combat::exact::evaluate_direct_damage(
        &ntesim_core::combat::exact::ExactDirectDamageRequest {
            kind: ntesim_core::combat::DamageKind::Elemental,
            attacker_level: 80,
            stat,
            ratio: ExactFraction::new(1, 1).unwrap(),
            damage_bonus: ExactFraction::new(0, 1).unwrap(),
            target_defense: ExactFraction::new(0, 1).unwrap(),
            defense_shred: ExactFraction::new(0, 1).unwrap(),
            defense_ignore: ExactFraction::new(0, 1).unwrap(),
            target_resistance: ExactFraction::new(0, 1).unwrap(),
            resistance_reduction: ExactFraction::new(0, 1).unwrap(),
            negative_resistance_model: NegativeResistanceModel::StandardRational,
            final_multiplier: ExactFraction::new(1, 1).unwrap(),
            special_multiplier: ExactFraction::new(1, 1).unwrap(),
            crit_bonus: ExactFraction::new(0, 1).unwrap(),
        },
    )
    .unwrap()
    .non_crit
}

#[test]
fn rishi_t5_live_atk_additive_rejects_multiplicative() {
    // Hand-derived oracle, not copied from test output. Fixture: base_atk =
    // 1000, compiled static ATK = 1200 (deliberately different so the
    // additive and multiplicative readings diverge).
    // COMBAT_FORMULA_CONTRACT section 2: ATK_live = ATK_static + base_atk * p
    //   = 1200 + 1000 * 1/2 = 1700.
    // Multiplicative falsifier: ATK_static * (1 + p) = 1200 * 3/2 = 1800.
    let mut req = rishi_request(t5_proc());
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1200,
        denominator: 1,
    };
    let state = GameSimulatorState::new(&req);
    let modifiers = zone_modifiers(&req, &state, 1);
    assert_eq!(
        modifiers.atk_percent,
        ExactFraction::new(1, 2).unwrap(),
        "T5 50% reaches the ATK percent zone through the real gear dispatch"
    );
    let live_atk = ExactFraction::new(1200, 1)
        .unwrap()
        .add(
            ExactFraction::new(1000, 1)
                .unwrap()
                .mul(modifiers.atk_percent)
                .unwrap(),
        )
        .unwrap();
    assert_eq!(live_atk, ExactFraction::new(1700, 1).unwrap());
    assert_ne!(
        live_atk,
        ExactFraction::new(1800, 1).unwrap(),
        "multiplicative reading rejected"
    );
    assert_eq!(
        settle_non_crit(live_atk),
        ExactFraction::new(1700, 1).unwrap(),
        "ratio 1, 0 DEF, 0 RES, non-crit: damage equals live ATK"
    );

    let mut plain_req = rishi_request(t5_proc());
    plain_req.actors[0].static_stats.atk = Ratio {
        numerator: 1200,
        denominator: 1,
    };
    plain_req.actors[0].gear_procs.clear();
    let plain_state = GameSimulatorState::new(&plain_req);
    assert!(
        zone_modifiers(&plain_req, &plain_state, 1)
            .atk_percent
            .is_zero()
    );
    assert_eq!(
        settle_non_crit(ExactFraction::new(1200, 1).unwrap()),
        ExactFraction::new(1200, 1).unwrap(),
        "proc-less baseline"
    );
    assert_eq!(
        ExactFraction::new(1700, 1)
            .unwrap()
            .div(ExactFraction::new(1200, 1).unwrap())
            .unwrap(),
        ExactFraction::new(17, 12).unwrap(),
        "damage ratio with/without the T5 proc"
    );
}

#[test]
fn rishi_arc_curves_projected() {
    let tables = projection_tables();
    let req = research_request();
    for tier in 1..=5u32 {
        let mut actor = req.actors[0].clone();
        actor.character_id = "character:1020".into();
        actor.weapon_id = "weapon:fork_rishi".into();
        actor.weapon_tier = tier;
        actor.gear_procs.clear();
        let report = crate::weapons::rishi::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap();
        let proc = &report["compiled_proc"];
        let keys: Vec<&str> = proc
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            keys,
            vec![
                "arc_cooldown_us",
                "atk_bonus",
                "defeat_energy",
                "eclipse_duration_us",
                "kind",
                "max_defeats",
            ],
            "tier {tier}: static ATK plus the four Arc: Eclipse curves"
        );
        assert_eq!(
            report["runtime_execution_validated"],
            json!(true),
            "tier {tier}: the executed ATK% + defeat-energy paths keep the honesty flag"
        );
        let scope = report["scope"].as_str().unwrap();
        assert!(
            scope.contains("explicit Arc admission"),
            "tier {tier}: scope names the field-creation trigger hypothesis"
        );
        let arc_scope = report["arc_scope"].as_str().unwrap();
        for curve in [
            "buff_rishi_CD2",
            "buff_rishi_energy",
            "buff_rishi_stack",
            "buff_rishi_CD",
        ] {
            assert!(
                arc_scope.contains(curve),
                "tier {tier}: arc_scope binds the curve {curve}"
            );
        }
    }
}
