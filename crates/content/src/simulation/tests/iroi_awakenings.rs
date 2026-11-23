//! Iroi awakening auras granted as scenario-start team buffs
//! (resonance_6 Crit DMG, Effect3 healing outgoing, resonance_3 DEF).
//!
//! Check direct grants and native initialization's hypothesis provenance.
use super::*;
use crate::runtime_api::DamageContext;
use crate::simulation::state::GameSimulatorState;

#[test]
fn native_initialization_records_selected_iroi_aura_hypotheses_and_requires_strict_acceptance() {
    use crate::characters::iroi::awakenings::*;
    let mut req = request_pair(ntesim_data::AwakeningSelection::prefix(6));
    req.actions.clear();
    let hypotheses = [
        (
            R6_TEAM_CRIT_AURA_HYPOTHESIS_ID,
            R6_TEAM_CRIT_AURA_HYPOTHESIS,
        ),
        (R3_TEAM_DEF_AURA_HYPOTHESIS_ID, R3_TEAM_DEF_AURA_HYPOTHESIS),
        (
            EFFECT3_HEALING_OUTGOING_HYPOTHESIS_ID,
            EFFECT3_HEALING_OUTGOING_HYPOTHESIS,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    for (id, description) in hypotheses {
        assert_eq!(
            report["resolved_model_assumptions"][id.as_str()],
            description
        );
    }
    for (id, description) in hypotheses {
        req.model_assumptions.insert(id.into(), description.into());
    }
    assert_eq!(
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap()["execution_completed"],
        true
    );
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
    req.model_assumptions.clear();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    for (id, _) in hypotheses {
        assert!(report["resolved_model_assumptions"][id.as_str()].is_null());
    }
}

fn request_pair(awakening: ntesim_data::AwakeningSelection) -> GameScenarioRequest {
    let mut req = sample_s1_request();
    req.actors[0].character_id = "character:1075".into();
    req.actors[0].weapon_id.clear();
    req.actors[0].gear_procs.clear();
    req.actors[0].set_id = None;
    req.actors[0].awakening = awakening;
    req.actors[0].static_stats.crit_damage_bonus = Ratio::zero();
    let mut teammate = req.actors[0].clone();
    teammate.slot = 1;
    teammate.character_id = "character:1019".into();
    teammate.awakening = ntesim_data::AwakeningSelection::prefix(0);
    req.actors.push(teammate);
    req
}

fn heal_request(awakening: ntesim_data::AwakeningSelection) -> GameScenarioRequest {
    let mut req = request_pair(awakening);
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    for actor in &mut req.actors {
        actor.initial_hp_max = 20_000;
        actor.initial_hp = 20_000;
    }
    req
}

fn initialize(req: &GameScenarioRequest) -> GameSimulatorState {
    let mut state = GameSimulatorState::new(req);
    crate::characters::iroi::awakenings::initialize_team_effect(
        &req.actors[0],
        0,
        &mut state.buffs,
    )
    .unwrap();
    state
}

fn live_mods(state: &GameSimulatorState, actor_index: usize) -> BTreeMap<String, Ratio> {
    let living: BTreeSet<usize> = state
        .actors
        .iter()
        .enumerate()
        .filter_map(|(index, actor)| (actor.hp > 0).then_some(index))
        .collect();
    state
        .buffs
        .get_stat_modifiers_for_actor_with_living_sources(
            actor_index,
            state.active_actor_index,
            state.current_time_us,
            &living,
        )
        .unwrap()
}

fn packet_kit_crit_bonus(req: &GameScenarioRequest, state: &GameSimulatorState) -> ExactFraction {
    let action_metadata = super::support::declared_test_action(
        "GA_Oneiroi_Melee",
        crate::simulation::metadata::ActionCategory::Melee,
    );
    let buff_stats = BTreeMap::new();
    let owned_status_counts = BTreeMap::new();
    let ctx = DamageContext {
        foreign_dot_stacks: 0,
        packet_group_id: None,
        command_index: 0,
        config: &req.actors[0],
        start_fons: None,
        ability_id: "GA_Oneiroi_Melee",
        action_metadata: &action_metadata,
        target: &state.targets[0],
        actor: &state.actors[0],
        world_time_us: 0,
        time_us: 0,
        debuff_count: 0,
        damage_effect_id: "GE_Player_Oneiroi_Melee1_Damage",
        actor_index: 0,
        active_actor_index: 0,
        target_count: 1,
        non_boss_target_count: 1,
        target_index: 0,
        buffs: &state.buffs,
        buff_stats: (&buff_stats).into(),
        owned_status_counts: &owned_status_counts,
        team_has_three_elements: false,
    };
    crate::registry::crit_bonus(&ctx).unwrap()
}

fn run_skill_heal(
    req: &GameScenarioRequest,
    state: &mut GameSimulatorState,
    recipient_actor_index: usize,
) -> Vec<crate::simulation::GameEventReceipt> {
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    crate::callbacks::apply(
        crate::callbacks::ContentEvent::IroiSkillHeal {
            source_actor_index: 0,
            recipient_actor_index,
            atk_ratio: crate::simulation::ExactFraction::new(943, 1_000).unwrap(),
        },
        &mut crate::runtime_api::CallbackContext {
            state,
            request: req,
            queue: &mut queue,
            sequence: &mut sequence,
            trace: &mut trace,
        },
    )
    .unwrap();
    trace
}

#[test]
fn iroi_r6_crit_aura_grants_team_crit_damage_while_iroi_lives() {
    assert_eq!(
        crate::characters::iroi::awakenings::R6_TEAM_CRIT_AURA_HYPOTHESIS_ID,
        "iroi.r6_team_crit_aura_v1"
    );
    let req = request_pair(ntesim_data::AwakeningSelection::prefix(6));
    let state = initialize(&req);
    let expected = Ratio {
        numerator: 1,
        denominator: 4,
    };
    assert_eq!(
        live_mods(&state, 1).get("crit_damage_bonus"),
        Some(&expected)
    );
    assert_eq!(
        live_mods(&state, 0).get("crit_damage_bonus"),
        Some(&expected)
    );
    // The per-packet registry hook is neutered: Iroi's own packets pick the aura up
    // through the live buff zone only, so the additive zone consumed by
    // critical_parameters (static + live buff + kit hook) totals exactly 1/4 once.
    assert_eq!(
        packet_kit_crit_bonus(&req, &state),
        ExactFraction::new(0, 1).unwrap()
    );
    let live = ExactFraction::from_ratio(live_mods(&state, 0)["crit_damage_bonus"]).unwrap();
    let total = ExactFraction::from_ratio(req.actors[0].static_stats.crit_damage_bonus)
        .unwrap()
        .add(live)
        .unwrap()
        .add(packet_kit_crit_bonus(&req, &state))
        .unwrap();
    assert_eq!(total, ExactFraction::new(1, 4).unwrap());
}

#[test]
fn iroi_r6_crit_aura_requires_count_six() {
    for awakening in [
        ntesim_data::AwakeningSelection::prefix(5),
        ntesim_data::AwakeningSelection::selected([6]).unwrap(),
    ] {
        let req = request_pair(awakening);
        let state = initialize(&req);
        assert!(!live_mods(&state, 1).contains_key("crit_damage_bonus"));
        assert!(!live_mods(&state, 0).contains_key("crit_damage_bonus"));
    }
}

#[test]
fn iroi_r6_crit_aura_drops_when_iroi_is_defeated() {
    let mut req = request_pair(ntesim_data::AwakeningSelection::prefix(6));
    req.actors[0].initial_hp = 0;
    let state = initialize(&req);
    assert!(!live_mods(&state, 1).contains_key("crit_damage_bonus"));
}

#[test]
fn iroi_effect3_amplifies_skill_heal_gross_through_outgoing_zone() {
    assert_eq!(
        crate::characters::iroi::awakenings::EFFECT3_HEALING_OUTGOING_HYPOTHESIS_ID,
        "iroi.effect3_healing_outgoing_v1"
    );
    // Independent oracle: 1000 ATK * .943 * (1 + .36) = 1282.48 -> 1282.
    // MaxHP20000 is deliberately unrelated to healing magnitude.
    for (awakening, expected_gross) in [
        (
            ntesim_data::AwakeningSelection::selected([3]).unwrap(),
            1_282u64,
        ),
        (ntesim_data::AwakeningSelection::prefix(0), 943u64),
    ] {
        let req = heal_request(awakening);
        let mut state = initialize(&req);
        let trace = run_skill_heal(&req, &mut state, 1);
        let heal = trace
            .iter()
            .find(|event| event.event_kind == "iroi_skill_heal")
            .expect("expected an iroi_skill_heal receipt");
        let payload = heal.data.as_ref().expect("iroi_skill_heal payload");
        assert_eq!(
            (payload.gross, payload.applied, payload.overflow),
            (Some(expected_gross), Some(0), Some(expected_gross)),
            "unexpected heal payload: {payload:?}"
        );
        // Effect3 is self-scoped: the teammate carries no healing_bonus.
        assert!(!live_mods(&state, 1).contains_key("healing_bonus"));
    }
    let req = heal_request(ntesim_data::AwakeningSelection::selected([3]).unwrap());
    let state = initialize(&req);
    assert_eq!(
        live_mods(&state, 0).get("healing_bonus"),
        Some(&Ratio {
            numerator: 9,
            denominator: 25
        })
    );
}

#[test]
fn iroi_effect3_composes_additively_with_static_healing_bonus() {
    let mut req = heal_request(ntesim_data::AwakeningSelection::selected([3]).unwrap());
    req.actors[0].static_stats.healing_bonus = Ratio {
        numerator: 1,
        denominator: 10,
    };
    req.actors[1].initial_hp_max = 40_000;
    req.actors[1].initial_hp = 10_000;
    let mut state = initialize(&req);
    let trace = run_skill_heal(&req, &mut state, 1);
    let heal = trace
        .iter()
        .find(|event| event.event_kind == "iroi_skill_heal")
        .expect("expected an iroi_skill_heal receipt");
    // Independent oracle: 1000 * .943 * (1 + .10 + .36) = 1376.78 -> 1377.
    let payload = heal.data.as_ref().expect("iroi_skill_heal payload");
    assert_eq!(
        (payload.gross, payload.applied, payload.overflow),
        (Some(1377), Some(1377), Some(0)),
        "unexpected heal payload: {payload:?}"
    );
    assert_eq!(state.actors[1].hp, 11_377);
}

#[test]
fn iroi_r3_def_aura_scales_teammate_base_def_while_iroi_lives() {
    assert_eq!(
        crate::characters::iroi::awakenings::R3_TEAM_DEF_AURA_HYPOTHESIS_ID,
        "iroi.r3_team_def_aura_v1"
    );
    let mut req = request_pair(ntesim_data::AwakeningSelection::prefix(3));
    for actor in &mut req.actors {
        actor.static_stats.base_def = Some(1_000);
        actor.static_stats.def_percent = Ratio::zero();
        actor.static_stats.def_flat = Ratio::zero();
    }
    let state = initialize(&req);
    let expected = Ratio {
        numerator: 3,
        denominator: 10,
    };
    let mods = live_mods(&state, 1);
    assert_eq!(mods.get("def_percent"), Some(&expected));
    // Independent oracle through the shared DEF zone: 1000 x (1 + 3/10) = 1300.
    let percent = ExactFraction::from_ratio(mods["def_percent"]).unwrap();
    let effective = ExactFraction::new(1_000, 1)
        .unwrap()
        .mul(ExactFraction::new(1, 1).unwrap().add(percent).unwrap())
        .unwrap();
    assert_eq!(effective, ExactFraction::new(1_300, 1).unwrap());
    // Iroi herself is covered by the team aura too.
    assert_eq!(live_mods(&state, 0).get("def_percent"), Some(&expected));
}

#[test]
fn iroi_r3_def_aura_requires_count_three_and_a_living_iroi() {
    let req = request_pair(ntesim_data::AwakeningSelection::prefix(2));
    let state = initialize(&req);
    assert!(!live_mods(&state, 1).contains_key("def_percent"));

    let mut req = request_pair(ntesim_data::AwakeningSelection::prefix(3));
    req.actors[0].initial_hp = 0;
    let state = initialize(&req);
    assert!(!live_mods(&state, 1).contains_key("def_percent"));
}

#[test]
fn iroi_initialize_team_effect_ignores_other_characters() {
    let mut req = request_pair(ntesim_data::AwakeningSelection::prefix(6));
    req.actors[0].character_id = "character:1019".into();
    let state = initialize(&req);
    assert!(!live_mods(&state, 1).contains_key("crit_damage_bonus"));
    assert!(!live_mods(&state, 1).contains_key("def_percent"));
    assert!(!live_mods(&state, 0).contains_key("healing_bonus"));
}
