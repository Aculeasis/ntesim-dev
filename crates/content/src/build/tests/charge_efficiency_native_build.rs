use super::*;

#[test]
fn native_build_replaces_manual_efficiency_and_preserves_fractional_energy() {
    let (tables, coverage, mut request) = skill_fixture();
    let build: equipment::Request = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/build-stats.json"
    )))
    .unwrap();
    let report = equipment::compile(&tables, &build).unwrap();
    let actor = &mut request.scenario.actors[0];
    actor.character_id = build.character_id.clone();
    actor.weapon_id = build.weapon_id.clone();
    actor.weapon_tier = build.weapon_tier;
    actor.set_id = Some(build.set_id.clone());
    actor.initial_hp_max = integer(&report["known_derived_stats"]["hp_receipt"], "HP").unwrap();
    actor.initial_hp = actor.initial_hp_max;
    actor.initial_energy = 0;
    actor.gear_procs = vec![GearProcConfig::ChargeEfficiency {
        multiplier: Ratio {
            numerator: 9,
            denominator: 5,
        },
    }];
    request.builds[0].build = build;
    request.builds[0].purchased_skill_levels = None;
    request.scenario.actions.clear();
    request.action_projection = None;
    request.gear_projection = None;
    let compiled = compile(&tables, &coverage, &request, &ExecutionLimits::default()).unwrap();
    let multiplier = Ratio {
        numerator: 13,
        denominator: 10,
    };
    assert_eq!(compiled.scenario.actors[0].gear_procs.len(), 1);
    assert!(
        compiled
            .scenario
            .model_assumptions
            .contains_key("weapons.good_boy_v3")
    );
    assert!(
        matches!(compiled.scenario.actors[0].gear_procs[0], GearProcConfig::ChargeEfficiency { multiplier: value } if value == multiplier)
    );
    let mut state = crate::simulation::state::GameSimulatorState::new(&compiled.scenario);
    // Set's .12 plus Good Boy's .18 gives 1.30; 9 * 1.30 = 11.7 each.
    assert_eq!(state.actors[0].grant_energy(9).unwrap(), 11);
    assert_eq!(state.actors[0].grant_energy(9).unwrap(), 12);
    assert_eq!(
        state.actors[0].charge_remainder,
        ExactFraction::new(2, 5).unwrap()
    );
    let previous = request.scenario.actors[0].gear_procs[0].clone();
    request.scenario.actors[0].gear_procs.push(previous);
    assert!(
        compile(&tables, &coverage, &request, &ExecutionLimits::default())
            .unwrap_err()
            .contains("duplicate charge efficiency")
    );
}

#[test]
fn native_charge_projection_removes_unit_override_and_rejects_invalid_multipliers() {
    let (_, _, request) = skill_fixture();
    let mut actor = request.scenario.actors[0].clone();
    actor.gear_procs.push(GearProcConfig::ChargeEfficiency {
        multiplier: Ratio {
            numerator: 2,
            denominator: 1,
        },
    });
    let report = json!({"known_attributes":{"ChargeGetEfficiencyBase":{"receipt":{"numerator":"1","denominator":"1"}}}});
    super::super::stats::project_charge_efficiency(&mut actor, &report, None).unwrap();
    assert!(
        !actor
            .gear_procs
            .iter()
            .any(|proc| matches!(proc, GearProcConfig::ChargeEfficiency { .. }))
    );
    for value in ["0", "101"] {
        let mut invalid = report.clone();
        invalid["known_attributes"]["ChargeGetEfficiencyBase"]["receipt"]["numerator"] =
            json!(value);
        assert!(
            super::super::stats::project_charge_efficiency(&mut actor, &invalid, None).is_err()
        );
    }
}
