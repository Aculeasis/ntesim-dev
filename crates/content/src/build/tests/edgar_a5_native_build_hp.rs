use super::*;

fn edgar_build_fixture() -> (BTreeMap<String, Value>, Value, Request) {
    let (mut tables, coverage, _) = skill_fixture();
    tables.insert(
        "1021AwakenEffect".into(),
        ntesim_data::resonance_levels::overlay("1021").unwrap(),
    );
    let mut request: Request = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/edgar-kit-built-game-research.json"
    )))
    .unwrap();
    request.scenario.actors.truncate(1);
    request.builds.truncate(1);
    request.scenario.actions.clear();
    request.action_projection = None;
    request.gear_projection = None;
    request.builds[0].purchased_skill_levels = None;
    request.scenario.actors[0].gear_procs.clear();
    (tables, coverage, request)
}

fn set_compiled_build_vitals(tables: &BTreeMap<String, Value>, request: &mut Request) -> u64 {
    let actor = &request.scenario.actors[0];
    let report = equipment::compile(tables, &request.builds[0].build).unwrap();
    assert_eq!(report["character_id"], actor.character_id);
    let character = actor.character_id.strip_prefix("character:").unwrap();
    let element = ntesim_data::character::definition(&format!("character:{}", character))
        .unwrap()
        .element
        .native_key();
    let stats = project(&report, element_attribute(element).unwrap()).unwrap();
    request.scenario.actors[0].initial_hp = stats.hp;
    request.scenario.actors[0].initial_hp_max = stats.hp;
    stats.hp
}

#[test]
fn theas_night_tavern_native_build_projects_healing_bonus_with_authored_stats() {
    let (tables, coverage, mut request) = edgar_build_fixture();
    request.scenario.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
    request.builds[0].build.set_id = "set:Suit9".into();
    request.scenario.actors[0].set_id = Some("set:Suit9".into());
    let base_hp = set_compiled_build_vitals(&tables, &mut request);
    let compiled = compile(&tables, &coverage, &request, &ExecutionLimits::default()).unwrap();
    let actor = &compiled.scenario.actors[0];
    assert_eq!(actor.initial_hp_max, base_hp);
    assert_eq!(
        actor.static_stats.healing_bonus,
        Ratio {
            numerator: 1,
            denominator: 5
        }
    );
    request.builds[0].build.set_count = 5;
    assert!(compile(&tables, &coverage, &request, &ExecutionLimits::default()).is_err());
}

#[test]
fn edgar_a5_native_build_preserves_full_and_damaged_hp_and_uses_frozen_scalar() {
    let (tables, coverage, mut request) = edgar_build_fixture();
    let base_hp = set_compiled_build_vitals(&tables, &mut request);
    let report = equipment::compile(&tables, &request.builds[0].build).unwrap();
    let character = "1021";
    let element = ntesim_data::character::definition(&format!("character:{}", character))
        .unwrap()
        .element
        .native_key();
    let native = project(&report, element_attribute(element).unwrap()).unwrap();
    assert_eq!(native.hp, base_hp);
    request.scenario.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(5);
    let compiled = compile(&tables, &coverage, &request, &ExecutionLimits::default()).unwrap();
    let mut selected_a5 = request.clone();
    selected_a5.scenario.actors[0].awakening =
        ntesim_data::AwakeningSelection::selected([5]).unwrap();
    let selected_compiled = compile(
        &tables,
        &coverage,
        &selected_a5,
        &ExecutionLimits::default(),
    )
    .unwrap();
    assert_eq!(
        selected_compiled.scenario.actors[0].static_stats.hp,
        compiled.scenario.actors[0].static_stats.hp
    );
    assert_eq!(
        selected_compiled.scenario.actors[0].initial_hp_max,
        compiled.scenario.actors[0].initial_hp_max
    );
    let full_percent = ExactFraction::from_ratio(native.hp_percent)
        .unwrap()
        .add(ExactFraction::new(1, 5).unwrap())
        .unwrap();
    let expected = ExactFraction::new(native.base_hp.unwrap().into(), 1)
        .unwrap()
        .mul(ExactFraction::new(1, 1).unwrap().add(full_percent).unwrap())
        .unwrap()
        .add(ExactFraction::from_ratio(native.hp_flat).unwrap())
        .unwrap()
        .round_u64()
        .unwrap();
    assert_eq!(
        compiled.scenario.actors[0].static_stats.hp_percent,
        ratio(full_percent).unwrap()
    );
    assert_eq!(compiled.scenario.actors[0].static_stats.hp, expected);
    assert_eq!(compiled.scenario.actors[0].initial_hp_max, expected);
    assert_eq!(compiled.scenario.actors[0].initial_hp, expected);
    assert!(compiled.receipt["actor_build_receipts"][0]["awakening_hp_projection"].is_object());
    assert!(
        !compiled
            .scenario
            .model_assumptions
            .contains_key("edgar.a5_max_hp_v1")
    );

    let mut damaged = request.clone();
    damaged.scenario.actors[0].initial_hp = base_hp - 137;
    let compiled_damaged =
        compile(&tables, &coverage, &damaged, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        compiled_damaged.scenario.actors[0].initial_hp,
        base_hp - 137
    );
    assert_eq!(compiled_damaged.scenario.actors[0].initial_hp_max, expected);

    // Cache-only Effect5 rationale declares +20%; the independent HP checks above
    // and owned bonus below exercise the executable rule.
    assert_eq!(
        crate::characters::edgar::awakenings::owned_max_hp_bonus(
            "character:1021",
            &ntesim_data::AwakeningSelection::prefix(5)
        ),
        Ratio {
            numerator: 1,
            denominator: 5
        }
    );
}

#[test]
fn non_edgar_a5_does_not_receive_edgar_hp_when_static_weapon_hp_projects() {
    for character_id in ["character:1019", "character:1039"] {
        assert_eq!(
            crate::characters::edgar::awakenings::owned_max_hp_bonus(
                character_id,
                &ntesim_data::AwakeningSelection::prefix(5)
            ),
            Ratio::zero(),
            "Arachne and Koinobori static HP projection must not apply Edgar A5 to {character_id}"
        );
    }
}
