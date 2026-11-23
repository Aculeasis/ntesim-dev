use super::*;
use crate::simulation::metadata::{ActionMetadata, InputActionRole, MetadataProvenance};
use crate::simulation::state::GameSimulatorState;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

fn t1_proc() -> GearProcConfig {
    GearProcConfig::JingmotingyuanCrit {
        crit_damage_bonus: Ratio {
            numerator: 3,
            denominator: 25,
        },
        duration_us: 5_000_000,
    }
}

fn arc_request(tier: u32, seed: u64) -> GameScenarioRequest {
    let mut req = cast_request();
    req.actors[0].character_id = "character:1023".into();
    req.actors[0].weapon_tier = tier;
    req.actors[0].initial_hp = 1000;
    req.actors[0].initial_hp_max = 1000;
    req.actors[0].gear_procs = vec![GearProcConfig::JingmotingyuanCrit {
        crit_damage_bonus: Ratio {
            numerator: 10 + 2 * u64::from(tier),
            denominator: 100,
        },
        duration_us: 5_000_000,
    }];
    req.action_metadata.clear();
    req.seed = seed;
    req.actions = vec![arc_input(0)];
    req
}

fn arc_input(timestamp_us: u64) -> GameActionCommand {
    GameActionCommand::PlayerInput {
        timestamp_us,
        actor_index: 0,
        input: PlayerInput::Arc,
        critical_success: false,
    }
}

#[test]
fn jingmotingyuan_arc_all_refinements_both_trains_match_independent_packet_oracle() {
    for tier in 1..=5 {
        let mut seen = BTreeSet::new();
        for seed in 0..16 {
            let req = arc_request(tier, seed);
            let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
            assert_eq!(
                report["execution_completed"], true,
                "{} {}",
                report["halt_diagnostic"], report["rejected_action"]
            );
            let choice = report["proc_selection_receipts"][0]["outcome_index"]
                .as_u64()
                .unwrap();
            seen.insert(choice);
            let hits = report["hit_receipts"].as_array().unwrap();
            assert_eq!(hits.len(), if choice == 0 { 5 } else { 6 });
            // ATK 1000 × DEF 1/2 × RES 4/5. Harsh is 24..40%,
            // Nagging is 18..30%; only its sixth packet doubles.
            for (index, hit) in hits.iter().enumerate() {
                let expected = if choice == 0 {
                    80 + 16 * tier
                } else {
                    (60 + 12 * tier) * if index == 5 { 2 } else { 1 }
                };
                assert_eq!(hit["damage"], json!(format!("{}/{}", expected, 1)));
                assert!(crate::weapons::jingmotingyuan::arc::is_packet(
                    &serde_json::from_value(super::support::hit_metadata(&report, hit).clone())
                        .unwrap()
                ));
            }
            assert_eq!(report["final_actor_states"][0]["hp"], 950);
            assert_eq!(report["final_actor_states"][0]["granted_energy"], 0);
            assert_eq!(report["player_input_receipts"][0]["input"], "arc");
            let replay: GameScenarioRequest =
                serde_json::from_value(serde_json::to_value(&req).unwrap()).unwrap();
            assert_eq!(
                run_game_scenario(&replay, &ExecutionLimits::default()).unwrap(),
                report
            );
        }
        assert_eq!(seen, BTreeSet::from([0, 1]));
    }
}

#[test]
fn jingmotingyuan_arc_continues_costs_and_rng_at_cooldown_boundary_and_rejects_early() {
    let mut req = arc_request(1, 0);
    req.relative_duration_us = 55_000_000;
    req.actions.push(arc_input(25_000_000));
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["final_actor_states"][0]["hp"], 902); // 950 × 5%=47.5 -> 48
    assert_eq!(
        report["proc_selection_receipts"].as_array().unwrap().len(),
        2
    );
    assert_ne!(
        report["proc_selection_receipts"][0]["sample"],
        report["proc_selection_receipts"][1]["sample"]
    );
    req.actions[1] = arc_input(24_999_999);
    let early = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(early["execution_completed"], false);
    assert_eq!(early["final_actor_states"][0]["hp"], 950);
    assert_eq!(
        early["proc_selection_receipts"].as_array().unwrap().len(),
        1
    );
}

#[test]
fn jingmotingyuan_arc_source_compiler_and_equipment_guard() {
    let script = "options iteration=1 duration=30 crit=fixed; baicang char lvl=80/80 awakening=0 talent=1,1,1 qte=1; baicang add weapon=fork_jingmotingyuan refine=5; target hp=100000000 def=0 resist=0; active baicang; baicang arc; wait(25); baicang arc;";
    let compiled = crate::dsl::compiler::compile_scenario(script, Default::default()).unwrap();
    let report = run_game_scenario(&compiled.request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(
        report["proc_selection_receipts"].as_array().unwrap().len(),
        2
    );
    let mut req = arc_request(1, 0);
    req.actors[0].weapon_id = "weapon:fork_Kite".into();
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires equipped")
    );
    let mut req = arc_request(1, 0);
    req.actors[0].gear_procs.clear();
    assert!(run_game_scenario(&req, &ExecutionLimits::default()).is_err());
}

#[test]
fn jingmotingyuan_arc_failed_admission_pays_nothing_and_consumes_no_randomness() {
    for case in 0..3 {
        let mut req = arc_request(1, 0);
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
            _ => {
                req.actions.insert(
                    0,
                    GameActionCommand::PlayerInput {
                        timestamp_us: 0,
                        actor_index: 0,
                        input: PlayerInput::Dash,
                        critical_success: false,
                    },
                );
            }
        }
        let report = run_game_scenario_report(&req, &ExecutionLimits::default()).unwrap();
        assert!(!report.execution_completed);
        assert!(report.proc_selection_receipts.is_empty());
        assert!(report.hit_receipts.is_empty());
        assert_eq!(report.final_actor_states[0].hp, req.actors[0].initial_hp);
    }
}

#[test]
fn jingmotingyuan_arc_cost_is_nonlethal_and_all_targets_share_one_choice() {
    let mut req = arc_request(1, 0);
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    req.actors[0].initial_hp = 1;
    let report = run_game_scenario_report(&req, &ExecutionLimits::default()).unwrap();
    assert!(report.execution_completed);
    assert_eq!(report.final_actor_states[0].hp, 1);
    assert_eq!(report.proc_selection_receipts.len(), 1);
    assert!(
        !report
            .trace
            .iter()
            .any(|entry| entry.event_kind == "jingmotingyuan_crit_stack")
    );
    for target in 0..2 {
        let hits: Vec<_> = report
            .hit_receipts
            .iter()
            .filter(|hit| hit.target_index == target)
            .collect();
        assert_eq!(
            hits.len(),
            if report.proc_selection_receipts[0].outcome_index == 0 {
                5
            } else {
                6
            }
        );
    }
}

#[test]
fn jingmotingyuan_arc_pending_hits_survive_switch_but_cancel_after_owner_death() {
    let mut req = arc_request(1, 0);
    let mut teammate = req.actors[0].clone();
    teammate.slot = 1;
    teammate.character_id = "character:1019".into();
    teammate.weapon_id.clear();
    teammate.gear_procs.clear();
    req.actors.push(teammate);
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 0,
        target_actor_index: 1,
        cooldown_us: 250_000,
    });
    let report = run_game_scenario_report(&req, &ExecutionLimits::default()).unwrap();
    assert!(report.execution_completed);
    assert!(!report.hit_receipts.is_empty());
    assert!(report.hit_receipts.iter().all(|hit| hit.actor_index == 0));
    assert_eq!(report.final_actor_states[0].hp, 950);
    assert_eq!(report.final_actor_states[1].hp, 1000);
    req.model_assumptions.insert(
        super::super::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
        super::super::events::INCOMING_DAMAGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.incoming_damage_events = vec![GameActorDamageEvent {
        timestamp_us: 0,
        actor_index: 0,
        delivered_damage: 950,
    }];
    let report = run_game_scenario_report(&req, &ExecutionLimits::default()).unwrap();
    assert!(report.execution_completed);
    assert!(report.hit_receipts.is_empty());
    assert_eq!(report.proc_selection_receipts.len(), 1);
    assert_eq!(report.final_actor_states[0].hp, 0);
    assert!(
        report
            .trace
            .iter()
            .any(|entry| entry.event_kind == "hit_owner_unavailable")
    );
}

#[test]
fn jingmotingyuan_arc_consumption_stack_applies_to_critical_packets_without_granting_energy() {
    let mut req = arc_request(1, 0);
    let mut policy = research_request().stochastic_crit.unwrap();
    policy.actor_sequences = BTreeMap::from([("character:1023".into(), 54)]);
    policy.eligible_effects =
        BTreeMap::from([("character:1023".into(), BTreeSet::from(["*".into()]))]);
    req.stochastic_crit = Some(policy);
    req.actors[0].static_stats.crit_rate = Ratio::one();
    let report = run_game_scenario_report(&req, &ExecutionLimits::default()).unwrap();
    for hit in &report.hit_receipts {
        assert!(hit.critical);
        assert_eq!(
            hit.damage,
            hit.non_crit_damage
                .mul(ExactFraction::new(53, 25).unwrap())
                .unwrap()
        );
    }
    assert_eq!(report.final_actor_states[0].granted_energy, 0);
    req.stochastic_crit
        .as_mut()
        .unwrap()
        .actor_sequences
        .insert("character:1023".into(), 1 << 61);
    assert!(
        run_game_scenario_report(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("collides")
    );
}

fn cast_request() -> GameScenarioRequest {
    let mut req = research_request();
    req.stochastic_crit = None;
    req.actors[0].weapon_id = "weapon:fork_jingmotingyuan".into();
    req.actors[0].gear_procs = vec![t1_proc()];
    req.model_assumptions.insert(
        crate::simulation::hp_cost::HYPOTHESIS_ID.into(),
        crate::simulation::hp_cost::HYPOTHESIS_ID
            .description()
            .into(),
    );
    req
}

fn action_metadata(tagged: bool) -> ActionMetadata {
    let mut tags = vec!["Ability.Melee".to_owned()];
    if tagged {
        tags.push("ntesim.trigger.hp_cost".into());
    }
    ActionMetadata {
        categories: BTreeSet::new(),
        gameplay_tags: tags,
        ability_id: "caller-test".into(),
        native_ability_index: None,
        input_role: Some(InputActionRole::Normal),
        input_role_provenance: None,
        primary_slot: None,
        source_evidence: None,
        identity_evidence: None,
        provenance: MetadataProvenance::AcceptedModelHypothesis {
            id: "test.jingmotingyuan_cast".into(),
            description: "Test-owned cast metadata.".into(),
        },
    }
}

fn settle_cost(
    tagged: bool,
    req: &GameScenarioRequest,
    state: &mut GameSimulatorState,
    time_us: u64,
) {
    let mut request = req.clone();
    request.hp_cost_events = vec![GameActorHpCostEvent {
        timestamp_us: time_us,
        actor_index: 0,
        source: "test.self_hp_consumption".into(),
        amount: u64::from(tagged),
    }];
    state.advance_unpaused_test_clock(time_us);
    super::super::hp_cost::settle(
        &mut crate::runtime_api::CallbackContext {
            state,
            request: &request,
            queue: &mut crate::simulation::timer_queue::EventQueue::new(),
            sequence: &mut 0,
            trace: &mut Vec::new(),
        },
        0,
    )
    .unwrap();
}

fn buff_expiries(state: &GameSimulatorState) -> Vec<u64> {
    state
        .buffs
        .instances()
        .iter()
        .filter(|buff| buff.id == crate::weapons::jingmotingyuan::BUFF_ID)
        .map(|buff| match buff.duration {
            ntesim_core::buffs::dispatch::BuffDuration::Timed { expires_at_us, .. } => {
                expires_at_us
            }
            ntesim_core::buffs::dispatch::BuffDuration::Permanent => u64::MAX,
        })
        .collect()
}

fn crit_damage_bonus_at(state: &GameSimulatorState, time_us: u64) -> Option<ExactFraction> {
    state
        .buffs
        .get_stat_modifiers_for_actor(0, 0, time_us)
        .unwrap()
        .get("crit_damage_bonus")
        .copied()
        .map(|ratio| ExactFraction::new(ratio.numerator.into(), ratio.denominator.into()).unwrap())
}

fn projection_tables() -> BTreeMap<String, serde_json::Value> {
    super::support::weapon_projection_tables()
}

fn projection_actor() -> GameActorConfig {
    serde_json::from_value(json!({
        "slot":0,"character_id":"character:1023","initial_hp":1,"initial_hp_max":1,"initial_energy":0,
        "energy_capacity":100,"weapon_id":"weapon:fork_jingmotingyuan","weapon_tier":1,
        "static_stats":{"base_atk":1,"atk":"1/1","hp":1,"def":1,
        "crit_rate":"0/1","crit_damage_bonus":"0/1"}
    }))
    .unwrap()
}

#[test]
fn jingmotingyuan_projection_binds_five_refinement_tiers() {
    let tables = projection_tables();
    let expected = [(3u64, 25u64), (7, 50), (4, 25), (9, 50), (1, 5)];
    let mut actor = projection_actor();
    for (tier, (numerator, denominator)) in (1..=5u32).zip(expected) {
        actor.weapon_tier = tier;
        actor.gear_procs.clear();
        let report = crate::weapons::jingmotingyuan::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap();
        let proc = &report["compiled_proc"];
        assert_eq!(proc["kind"], "jingmotingyuan_crit");
        assert_eq!(
            proc["crit_damage_bonus"],
            json!(format!("{}/{}", numerator, denominator)),
            "tier {tier} crit damage bonus"
        );
        assert_eq!(proc["duration_us"], 5_000_000);
        assert_eq!(report["native_parameter_report"]["parameter_count"], 6);
        assert_eq!(actor.gear_procs.len(), 1);
    }
}

#[test]
fn jingmotingyuan_projection_rejects_group_mismatch_and_changed_units() {
    let tables = projection_tables();
    let mut actor = projection_actor();
    actor.character_id = "character:1003".into();
    assert!(
        crate::weapons::jingmotingyuan::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap_err()
        .contains("group is incompatible")
    );
    actor.character_id = "character:1023".into();
    let mut changed = tables.clone();
    changed.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]
        ["upgradestar_pack_fork_jingmotingyuan_1"]["DataList"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["NameID"] == "buff_tingyuan_CritDamageUp")
        .unwrap()["bIsPercent"] = json!(false);
    assert!(
        crate::weapons::jingmotingyuan::projection::project(
            &changed,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap_err()
        .contains("unit hint")
    );
}

#[test]
fn jingmotingyuan_validate_config_rejects_mismatches() {
    let limits = ExecutionLimits::default();
    let mut actor = projection_actor();
    let valid = t1_proc();
    let ctx = crate::gear::validation::ConfigContext {
        actor: &actor,
        actor_index: 0,
        limits: &limits,
    };
    let validation = crate::weapons::jingmotingyuan::validate_config(&valid, &ctx).unwrap();
    assert_eq!(validation.kind, "jingmotingyuan");
    assert_eq!(validation.duration_us, 5_000_000);

    actor.weapon_id = "weapon:fork_Kite".into();
    let ctx = crate::gear::validation::ConfigContext {
        actor: &actor,
        actor_index: 0,
        limits: &limits,
    };
    let err = crate::weapons::jingmotingyuan::validate_config(&valid, &ctx)
        .err()
        .expect("expected weapon mismatch rejection");
    assert!(err.contains("weapon:fork_jingmotingyuan"));

    actor.weapon_id = "weapon:fork_jingmotingyuan".into();
    let ctx = crate::gear::validation::ConfigContext {
        actor: &actor,
        actor_index: 0,
        limits: &limits,
    };
    let zero_duration = GearProcConfig::JingmotingyuanCrit {
        crit_damage_bonus: Ratio {
            numerator: 3,
            denominator: 25,
        },
        duration_us: 0,
    };
    let err = crate::weapons::jingmotingyuan::validate_config(&zero_duration, &ctx)
        .err()
        .expect("expected zero duration rejection");
    assert!(err.contains("positive duration"));
    let bad_ratio = GearProcConfig::JingmotingyuanCrit {
        crit_damage_bonus: Ratio {
            numerator: 3,
            denominator: 0,
        },
        duration_us: 5_000_000,
    };
    assert!(crate::weapons::jingmotingyuan::validate_config(&bad_ratio, &ctx).is_err());
}

#[test]
fn jingmotingyuan_buff_granted_only_by_positive_hp_consumption() {
    let req = cast_request();
    let mut state = GameSimulatorState::new(&req);
    settle_cost(false, &req, &mut state, 0);
    settle_cost(false, &req, &mut state, 1_000_000);
    assert!(buff_expiries(&state).is_empty());
    settle_cost(true, &req, &mut state, 2_000_000);
    let expiries = buff_expiries(&state);
    assert_eq!(expiries.len(), 1);
    assert_eq!(expiries[0], 7_000_000);
    let buff = state
        .buffs
        .instances()
        .iter()
        .find(|buff| buff.id == crate::weapons::jingmotingyuan::BUFF_ID)
        .unwrap();
    assert_eq!(buff.source, "weapon:fork_jingmotingyuan");
    assert_eq!(buff.owner_actor, 0);
}

#[test]
fn jingmotingyuan_stacks_cap_at_four_and_refresh_duration() {
    let req = cast_request();
    let mut state = GameSimulatorState::new(&req);
    for time in [0, 1_000_000, 2_000_000, 3_000_000, 4_000_000] {
        settle_cost(true, &req, &mut state, time);
    }
    let expiries = buff_expiries(&state);
    assert_eq!(expiries.len(), 1, "Reset policy keeps one instance");
    assert_eq!(expiries[0], 9_000_000);
    let stacks = state
        .buffs
        .instances()
        .iter()
        .find(|buff| buff.id == crate::weapons::jingmotingyuan::BUFF_ID)
        .unwrap()
        .stacks;
    assert_eq!(stacks, 4, "fifth grant at the cap adds no stack");
    assert_eq!(
        crit_damage_bonus_at(&state, 4_500_000),
        Some(ExactFraction::new(12, 25).unwrap())
    );
}

#[test]
fn jingmotingyuan_buff_expires_after_duration() {
    let req = cast_request();
    let mut state = GameSimulatorState::new(&req);
    settle_cost(true, &req, &mut state, 0);
    assert_eq!(
        crit_damage_bonus_at(&state, 4_999_999),
        Some(ExactFraction::new(3, 25).unwrap())
    );
    assert_eq!(crit_damage_bonus_at(&state, 5_000_001), None);
}

#[test]
fn jingmotingyuan_crit_damage_joins_additive_zone() {
    let req = cast_request();
    let mut state = GameSimulatorState::new(&req);
    settle_cost(true, &req, &mut state, 0);
    settle_cost(true, &req, &mut state, 1_000_000);
    let bonus = crit_damage_bonus_at(&state, 2_000_000).unwrap();
    let zone = ExactFraction::new(1, 1).unwrap().add(bonus).unwrap();
    assert_eq!(zone, ExactFraction::new(31, 25).unwrap());
}

#[test]
fn jingmotingyuan_cap_four_stacks_t1_is_12_25() {
    // Hand-derived: T1 per-stack 0.12 = 3/25, four stacks cap -> 4 * 3/25 = 12/25
    // additive into the crit-damage zone (COMBAT_FORMULA_CONTRACT section 1).
    let req = cast_request();
    let mut state = GameSimulatorState::new(&req);
    for time in [0, 1_000_000, 2_000_000, 3_000_000] {
        settle_cost(true, &req, &mut state, time);
    }
    assert_eq!(
        crit_damage_bonus_at(&state, 3_500_000),
        Some(ExactFraction::new(12, 25).unwrap())
    );
    // A fifth grant at the cap adds nothing: still 12/25, not 15/25.
    settle_cost(true, &req, &mut state, 4_000_000);
    assert_eq!(
        crit_damage_bonus_at(&state, 4_500_000),
        Some(ExactFraction::new(12, 25).unwrap())
    );
}

#[test]
fn jingmotingyuan_wearer_only_no_spill_to_teammate() {
    let mut req = cast_request();
    let mut teammate = req.actors[0].clone();
    teammate.weapon_id = "weapon:fork_Kite".into();
    teammate.gear_procs.clear();
    req.actors.push(teammate);
    let mut state = GameSimulatorState::new(&req);
    settle_cost(true, &req, &mut state, 0);
    assert_eq!(
        crit_damage_bonus_at(&state, 500_000),
        Some(ExactFraction::new(3, 25).unwrap())
    );
    let teammate_modifiers = state
        .buffs
        .get_stat_modifiers_for_actor(1, 0, 500_000)
        .unwrap();
    assert!(
        !teammate_modifiers.contains_key("crit_damage_bonus"),
        "BuffRecipient::SelfActor must not spill onto actor 1"
    );
}

#[test]
fn jingmotingyuan_trigger_after_expiry_grants_fresh_buff() {
    // RefreshPolicy::Reset: after expiry there is nothing to refresh; the next
    // settled HP cost grants a fresh single stack with a fresh 5s deadline.
    let req = cast_request();
    let mut state = GameSimulatorState::new(&req);
    settle_cost(true, &req, &mut state, 0);
    assert_eq!(crit_damage_bonus_at(&state, 5_000_001), None);
    settle_cost(true, &req, &mut state, 6_000_000);
    let buff = state
        .buffs
        .instances()
        .iter()
        .find(|buff| buff.id == crate::weapons::jingmotingyuan::BUFF_ID)
        .unwrap();
    assert_eq!(buff.stacks, 1);
    assert_eq!(
        crit_damage_bonus_at(&state, 6_500_000),
        Some(ExactFraction::new(3, 25).unwrap())
    );
    assert_eq!(buff_expiries(&state), vec![11_000_000]);
}

#[test]
fn jingmotingyuan_projection_reports_runtime_execution_validated() {
    // Honesty lock: the projection claims runtime execution of the buff chain.
    // The staged "crit_damage_bonus" modifier is consumed by the live hit path
    // (critical_parameters in hit/calculation.rs joins it into the additive
    // crit-damage zone), so the true flag is verified, not aspirational.
    let tables = projection_tables();
    let mut actor = projection_actor();
    let report = crate::weapons::jingmotingyuan::projection::project(
        &tables,
        1,
        "ntesim-current",
        &mut actor,
        &[],
    )
    .unwrap();
    assert_eq!(
        report["runtime_execution_validated"],
        serde_json::json!(true)
    );
}

#[test]
fn jingmotingyuan_action_tag_alone_cannot_grant_a_stack() {
    let req = cast_request();
    let mut state = GameSimulatorState::new(&req);
    super::support::invoke_gear_cast(&req, &mut state, 0, &action_metadata(true));
    assert!(buff_expiries(&state).is_empty());
}

fn hp_cost(time: u64, amount: u64) -> GameActorHpCostEvent {
    GameActorHpCostEvent {
        timestamp_us: time,
        actor_index: 0,
        source: "test.self_hp_consumption".into(),
        amount,
    }
}

#[test]
fn jingmotingyuan_live_consumption_has_independent_damage_hp_and_expiry_oracles() {
    for (tier, per_stack) in (1..=5).zip([12, 14, 16, 18, 20]) {
        let mut req = cast_request();
        req.stochastic_crit = research_request().stochastic_crit;
        req.actors[0].static_stats.crit_rate = Ratio::one();
        req.actors[0].weapon_tier = tier;
        req.actors[0].gear_procs = vec![GearProcConfig::JingmotingyuanCrit {
            crit_damage_bonus: Ratio {
                numerator: per_stack,
                denominator: 100,
            },
            duration_us: 5_000_000,
        }];
        req.actors[0].initial_hp = 1000;
        req.actors[0].initial_hp_max = 1000;
        req.hp_cost_events = vec![
            hp_cost(0, 50),
            hp_cost(1_000_000, 100),
            hp_cost(2_000_000, 200),
            hp_cost(3_000_000, 100),
            hp_cost(4_000_000, 100),
        ];
        let times = [0, 1_000_000, 2_000_000, 3_000_000, 4_000_000, 9_000_000];
        for (action, time) in req.actions.iter_mut().zip(times) {
            if let GameActionCommand::InvokeAbility { timestamp_us, .. } = action {
                *timestamp_us = time;
            }
        }
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        // ATK 1000, coefficient 1, DEF factor 1/2, RES factor 4/5 => 400.
        // Base additional CD=1; stacks add 0.12..0.20, never multiply CD.
        for (hit, stacks) in report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .zip([1, 2, 3, 4, 4, 0])
        {
            assert_eq!(hit["non_crit_damage"], json!("400/1"));
            assert_eq!(
                hit["damage"],
                json!(format!("{}/{}", 800 + 4 * stacks * per_stack, 1))
            );
        }
        assert_eq!(report["final_actor_states"][0]["hp"], 450);
        let receipts = report["hp_cost_receipts"].as_array().unwrap();
        assert_eq!(receipts.len(), 5);
        assert_eq!(receipts[2]["hp_before"], 850);
        assert_eq!(receipts[2]["hp_after"], 650);
        assert_eq!(receipts[2]["source"], "test.self_hp_consumption");
        let replay: GameScenarioRequest =
            serde_json::from_value(serde_json::to_value(&req).unwrap()).unwrap();
        assert_eq!(
            report,
            run_game_scenario(&replay, &ExecutionLimits::default()).unwrap()
        );
    }
}

#[test]
fn jingmotingyuan_hp_consumption_is_nonlethal_skips_zero_dead_and_incoming_damage() {
    let mut req = cast_request();
    req.actions.clear();
    req.action_metadata.clear();
    req.actors[0].initial_hp = 100;
    req.actors[0].initial_hp_max = 100;
    req.hp_cost_events = vec![hp_cost(0, 0), hp_cost(1, u64::MAX), hp_cost(2, 10)];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hp_cost_receipts"][0]["actual_hp_loss"], 0);
    assert_eq!(report["hp_cost_receipts"][1]["actual_hp_loss"], 99);
    assert_eq!(report["hp_cost_receipts"][2]["actual_hp_loss"], 0);
    assert_eq!(report["final_actor_states"][0]["hp"], 1);
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["event_kind"] == "jingmotingyuan_crit_stack")
            .count(),
        1
    );

    req.model_assumptions.insert(
        super::super::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
        super::super::events::INCOMING_DAMAGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.incoming_damage_events = vec![GameActorDamageEvent {
        timestamp_us: 0,
        actor_index: 0,
        delivered_damage: 100,
    }];
    req.hp_cost_events = vec![hp_cost(0, 50)];
    let dead = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(dead["hp_cost_receipts"][0]["hp_before"], 0);
    assert_eq!(dead["hp_cost_receipts"][0]["actual_hp_loss"], 0);
    assert!(
        !dead["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["event_kind"] == "jingmotingyuan_crit_stack")
    );
}

#[test]
fn hp_cost_fixture_rejects_unacknowledged_invalid_or_out_of_budget_events() {
    let mut req = cast_request();
    req.actions.clear();
    req.action_metadata.clear();
    req.hp_cost_events = vec![hp_cost(0, 1)];
    let mut unacknowledged = req.clone();
    unacknowledged
        .model_assumptions
        .remove(super::super::hp_cost::HYPOTHESIS_ID.as_str());
    assert!(
        validate_game_request(&unacknowledged, &ExecutionLimits::default())
            .unwrap_err()
            .contains("accepted model assumption")
    );
    for bad in [
        hp_cost(req.absolute_limit_us + 1, 1),
        GameActorHpCostEvent {
            actor_index: 4,
            ..hp_cost(0, 1)
        },
        GameActorHpCostEvent {
            source: " ".into(),
            ..hp_cost(0, 1)
        },
    ] {
        let mut invalid = req.clone();
        invalid.hp_cost_events = vec![bad];
        assert!(validate_game_request(&invalid, &ExecutionLimits::default()).is_err());
    }
    req.hp_cost_events = vec![hp_cost(1, 1), hp_cost(0, 1)];
    assert!(validate_game_request(&req, &ExecutionLimits::default()).is_err());
    req.hp_cost_events = vec![hp_cost(0, 1), hp_cost(0, 1)];
    let limits = ExecutionLimits {
        max_events: 1,
        ..ExecutionLimits::default()
    };
    assert!(
        validate_game_request(&req, &limits)
            .unwrap_err()
            .contains("event count")
    );
}

#[test]
fn jingmotingyuan_consumption_bypasses_shields_and_buffs_the_off_field_payer_only() {
    let mut req = cast_request();
    let mut teammate = req.actors[0].clone();
    teammate.slot = 1;
    teammate.gear_procs.clear();
    teammate.weapon_id.clear();
    req.actors.push(teammate);
    req.active_actor_index = 1;
    req.actors[0].initial_hp = 1000;
    req.actors[0].initial_hp_max = 1000;
    req.hp_cost_events = vec![hp_cost(0, 50)];
    let mut state = GameSimulatorState::new(&req);
    crate::simulation::state::replace_actor_shield(
        &mut state.actors[0],
        1000,
        0,
        "test.shield",
        "test.source",
        500,
        10_000_000,
    )
    .unwrap();
    let receipt = super::super::hp_cost::settle(
        &mut crate::runtime_api::CallbackContext {
            state: &mut state,
            request: &req,
            queue: &mut crate::simulation::timer_queue::EventQueue::new(),
            sequence: &mut 0,
            trace: &mut Vec::new(),
        },
        0,
    )
    .unwrap();
    assert_eq!(receipt.hp_before, 1000);
    assert_eq!(receipt.hp_after, 950);
    assert_eq!(state.actors[0].shields[0].remaining_hp, 500);
    assert_eq!(
        crit_damage_bonus_at(&state, 0),
        Some(ExactFraction::new(3, 25).unwrap())
    );
    assert!(
        !state
            .buffs
            .get_stat_modifiers_for_actor(1, 1, 0)
            .unwrap()
            .contains_key("crit_damage_bonus")
    );
}
