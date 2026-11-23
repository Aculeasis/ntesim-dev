use super::*;

#[test]
fn reviewed_weapon_assumptions_survive_no_action_runs_and_enforce_strict_acceptance() {
    let initialize = |scenario: &GameScenarioRequest| {
        let mut state = crate::simulation::state::GameSimulatorState::new(scenario);
        crate::gear::initialize(&mut crate::runtime_api::CallbackContext {
            state: &mut state,
            request: scenario,
            queue: &mut crate::simulation::timer_queue::EventQueue::new(),
            sequence: &mut 0,
            trace: &mut Vec::new(),
        })
    };
    let (tables, _, request) = fireflies_fixture();
    let projection = GearProjection {
        policy: GearProjectionPolicy::NativeWeaponCurveV1,
        hypothesis: "Reviewed weapon startup".into(),
    };
    for (weapon, id) in [
        (
            "weapon:fork_jiaojuan",
            crate::weapons::jiaojuan::HYPOTHESIS_ID,
        ),
        (
            "weapon:fork_snowman",
            crate::weapons::snowman::HYPOTHESIS_ID,
        ),
        ("weapon:fork_yuren", crate::weapons::yuren::HYPOTHESIS_ID),
        (
            "weapon:fork_tuansanlang",
            crate::weapons::tuansanlang::HYPOTHESIS_ID,
        ),
        ("weapon:fork_yaodao", crate::weapons::yaodao::HYPOTHESIS_ID),
        (
            "weapon:fork_ThiefCandy",
            crate::weapons::thief_candy::HYPOTHESIS_ID,
        ),
        (
            "weapon:fork_dustbin",
            crate::weapons::dustbin::HYPOTHESIS_ID,
        ),
        (
            "weapon:fork_PoliceRat",
            crate::weapons::police_rat::HYPOTHESIS_ID,
        ),
        ("weapon:fork_Rose", crate::weapons::rose::HYPOTHESIS_ID),
        (
            "weapon:fork_TigerTally",
            crate::weapons::tiger_tally::HYPOTHESIS_ID,
        ),
        (
            "weapon:fork_Nakupeda",
            crate::weapons::nakupeda::HYPOTHESIS_ID,
        ),
        (
            "weapon:fork_BlackBook",
            crate::weapons::black_book::HYPOTHESIS_ID,
        ),
        (
            "weapon:fork_jingmotingyuan",
            crate::weapons::jingmotingyuan::HYPOTHESIS_ID,
        ),
        ("weapon:fork_rishi", crate::weapons::rishi::HYPOTHESIS_ID),
        ("weapon:fork_mamen", crate::weapons::mamen::HYPOTHESIS_ID),
    ] {
        let adapter = gear::WEAPON_ADAPTERS
            .iter()
            .find(|a| a.weapon_id == weapon)
            .unwrap();
        let mut scenario = request.scenario.clone();
        scenario.actions.clear();
        scenario.action_metadata.clear();
        scenario.stochastic_crit = None;
        scenario.actors.truncate(1);
        let actor = &mut scenario.actors[0];
        actor.weapon_id = weapon.into();
        actor.character_id = adapter.preview_character_id.into();
        actor.gear_procs.clear();
        gear::project_gear(&tables, &mut scenario, &projection).unwrap();
        scenario.start_fons = Some(0);
        let report = run_game_scenario(&scenario, &ExecutionLimits::default()).unwrap();
        assert!(
            report["resolved_model_assumptions"][id.as_str()].is_string(),
            "{weapon}"
        );
        scenario.model_assumptions =
            serde_json::from_value(report["resolved_model_assumptions"].clone()).unwrap();
        initialize(&scenario).unwrap();
    }
}

#[test]
fn registered_native_weapon_projections_satisfy_runtime_validation() {
    let (tables, _, request) = fireflies_fixture();
    let projection = GearProjection {
        policy: GearProjectionPolicy::NativeWeaponCurveV1,
        hypothesis: "Native owner configuration reaches runtime validation".into(),
    };
    for adapter in gear::WEAPON_ADAPTERS {
        for tier in [1, 5] {
            let mut scenario = request.scenario.clone();
            let actor = &mut scenario.actors[0];
            actor.weapon_id = adapter.weapon_id.into();
            actor.character_id = adapter.preview_character_id.into();
            actor.weapon_tier = tier;
            actor.gear_procs.clear();
            let receipts = gear::project_gear(&tables, &mut scenario, &projection)
                .unwrap_or_else(|error| panic!("{} T{tier}: {error}", adapter.weapon_id));
            assert_eq!(receipts.len(), 1);
            assert_eq!(scenario.actors[0].gear_procs.len(), 1);
            crate::gear::validate_config(
                &scenario.actors[0],
                0,
                scenario.relative_duration_us,
                &ExecutionLimits::default(),
            )
            .unwrap_or_else(|error| panic!("{} T{tier}: {error}", adapter.weapon_id));
        }
    }
}
