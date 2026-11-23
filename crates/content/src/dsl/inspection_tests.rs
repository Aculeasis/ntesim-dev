use super::{compiler::*, inspection::*, validation::DiagnosticCode};

const CONFIG: &str = "options duration=20 start_fons=0;\n\
mint char lvl=80/80 awakening=0 talent=1,2,3 qte=6 start_energy=0 start_cycle=20;\n\
mint add stats atk%=0.1+0.2 atk=10.1 hp=150;\n\
target hp=10000000 def=0 resist=0;\n\
active mint;\n";

#[test]
fn inspection_compiles_street_boxer_permanent_crit_at_four_pieces_without_execution() {
    for count in 0..=4 {
        let set = if count == 0 {
            String::new()
        } else {
            format!("mint add cartridge=Suit5 count={count};")
        };
        let response = inspect_response(&InspectionRequest {
            source: format!(
                "options duration=1; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; {set} mint add stats cr=0.30; target hp=1000000 def=0 resist=0; active mint; while 1 {{}}"
            ),
        });
        assert!(
            response.diagnostics.is_empty(),
            "{:?}",
            response.diagnostics
        );
        let party = response.party.unwrap();
        let actor = &party.actors[0];
        actor
            .build_decomposition
            .validate_static_stats(&actor.static_stats)
            .unwrap();
        assert_eq!(
            actor.static_stats.crit_rate,
            if count == 4 {
                ntesim_core::formula::Ratio {
                    numerator: 49,
                    denominator: 100,
                }
            } else {
                ntesim_core::formula::Ratio {
                    numerator: 7,
                    denominator: 20,
                }
            }
        );
    }
}

#[test]
fn inspection_returns_exact_static_party_without_executing_or_lowering_rotation() {
    let request = InspectionRequest {
        source: format!("{CONFIG}while 1 {{}}"),
    };
    let response = inspect_response(&request);
    assert!(
        response.diagnostics.is_empty(),
        "{:?}",
        response.diagnostics
    );
    let party = response.party.unwrap();
    assert_eq!(party.schema_version, 1);
    assert_eq!(party.content_id, "ntesim-current");
    assert_eq!(party.start_fons, Some(0));
    assert_eq!(party.active_actor_index, 0);
    assert_eq!(party.actors.len(), 1);
    let actor = &party.actors[0];
    assert_eq!(actor.slot, 0);
    assert_eq!(actor.character_id, "character:1019");
    assert_eq!((actor.loc.line, actor.loc.column), (2, 1));
    assert_eq!(actor.talent_levels, [1, 2, 3]);
    assert_eq!(actor.purchased_qte_level, 6);
    assert!(actor.weapon.is_none());
    assert!(actor.set.is_none());
    // Independently checked level-80 Mint: base ATK 596; 596*1.30+10.1=784.9.
    assert_eq!(actor.static_stats.base_atk, 596);
    let decomposition = &actor.build_decomposition;
    assert_eq!(
        decomposition.atk,
        crate::simulation::ExactFraction::new(7849, 10).unwrap()
    );
    assert_eq!(
        decomposition.hp,
        crate::simulation::ExactFraction::new(12270, 1).unwrap()
    );
    assert!(
        decomposition
            .contributions
            .iter()
            .any(|c| c.source == crate::build::StatSource::Authored
                && c.attribute_id == "AtkAdd"
                && c.amount == crate::simulation::ExactFraction::new(101, 10).unwrap())
    );
    assert_eq!(
        actor.static_stats.atk.numerator * 10,
        7849 * actor.static_stats.atk.denominator
    );
    assert_eq!(actor.static_stats.hp, 12_270);
    assert_eq!(actor.static_stats.def, 727);
    assert_eq!(actor.initial_hp, 12_270);
    assert_eq!(actor.initial_energy, 0);
    assert_eq!(actor.initial_cycle, 20);
    assert_eq!(actor.charge_efficiency, ntesim_core::formula::Ratio::one());
    assert!(!party.model_assumption_ids.is_empty());
    let json = serde_json::to_value(&party).unwrap();
    assert!(json["actors"][0].get("name").is_none());
    assert!(json.get("model_assumptions").is_none());
    let decoded: PartyInspection = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), json);
}

#[test]
fn inspected_equipped_party_matches_every_native_execution_actor_in_authored_order() {
    let source = include_str!("../../../../presets/studio/s4.nte");
    let response = compile_response(&SourceRequest {
        source: source.into(),
        limits: Default::default(),
    });
    assert!(response.diagnostics.is_empty());
    let party = response.party.unwrap();
    let compiled = response.compiled.unwrap();
    assert_eq!(party.actors.len(), 4);
    for (preview, actor) in party.actors.iter().zip(&compiled.request.actors) {
        assert_eq!(preview.slot, actor.slot);
        assert_eq!(preview.character_id, actor.character_id);
        assert_eq!(Some(preview.purchased_qte_level), actor.purchased_qte_level);
        assert_eq!(
            preview
                .weapon
                .as_ref()
                .map_or("", |weapon| weapon.id.as_str()),
            actor.weapon_id
        );
        if let Some(weapon) = &preview.weapon {
            assert_eq!(weapon.refinement, actor.weapon_tier);
            assert_eq!(weapon.level, 80);
        }
        assert_eq!(
            preview.set.as_ref().map(|set| &set.id),
            actor.set_id.as_ref()
        );
        assert_eq!(
            serde_json::to_value(&preview.static_stats).unwrap(),
            serde_json::to_value(&actor.static_stats).unwrap()
        );
        assert_eq!(preview.initial_energy, actor.initial_energy);
        assert_eq!(preview.initial_hp, actor.initial_hp);
    }
    assert_eq!(
        party.active_actor_index,
        compiled.request.active_actor_index
    );
    assert_eq!(party.start_fons, compiled.request.start_fons);
}

#[test]
fn invalid_qte_reports_the_owning_declaration_without_a_partial_party() {
    for (qte, message) in [
        ("", "requires qte"),
        (" qte=0", "invalid purchased QTE level"),
        (" qte=1/2", "integer"),
        (" qte=6 qte=8", "duplicate"),
    ] {
        let declaration = format!("zero char lvl=80/80 awakening=[] talent=1,2,3{qte};");
        let source = CONFIG.replace("active mint;", &format!("{declaration}\nactive mint;"));
        let response = inspect_response(&InspectionRequest { source });
        assert!(response.party.is_none());
        assert_eq!(response.diagnostics[0].loc.line, 5);
        assert!(
            response.diagnostics[0]
                .message
                .to_lowercase()
                .contains(&message.to_lowercase()),
            "{:?}",
            response.diagnostics
        );
    }
}

#[test]
fn rotation_compilation_failure_preserves_party_but_invalid_configuration_does_not() {
    let source = format!("{CONFIG}wait(-1);");
    let response = compile_response(&SourceRequest {
        source: source.clone(),
        limits: Default::default(),
    });
    assert!(response.compiled.is_none());
    assert!(response.party.is_some());
    assert_eq!(response.diagnostics[0].loc.line, 6);
    assert!(compile_scenario(&source, Default::default()).is_err());
    let inspected = inspect_response(&InspectionRequest { source });
    assert!(inspected.diagnostics.is_empty());
    assert!(inspected.party.is_some());
    for source in [
        CONFIG.replace("lvl=80/80", "lvl=81/80"),
        CONFIG.replace("atk=10.1", "atk=1/0"),
    ] {
        let response = inspect_response(&InspectionRequest { source });
        assert!(response.party.is_none());
        assert!(!response.diagnostics.is_empty());
    }
}

#[test]
fn public_diagnostics_keep_codes_lines_columns_and_utf16_ranges() {
    for (tail, code, column) in [
        ("@", DiagnosticCode::Lexical, 1),
        ("wait(1 2);", DiagnosticCode::Syntax, 8),
        ("other attack;", DiagnosticCode::Binding, 1),
    ] {
        let source = format!("# 🦀\r\n{CONFIG}{tail}");
        let response = compile_response(&SourceRequest {
            source: source.clone(),
            limits: Default::default(),
        });
        assert!(response.compiled.is_none());
        assert!(response.party.is_none());
        let diagnostic = &response.diagnostics[0];
        assert_eq!(diagnostic.code, code);
        assert_eq!((diagnostic.loc.line, diagnostic.loc.column), (7, column));
        let offset = source.find(tail).unwrap();
        assert_eq!(
            diagnostic.loc.offset,
            source[..offset].encode_utf16().count() + column - 1
        );
        assert!(diagnostic.loc.length > 0);
        let encoded = serde_json::to_value(diagnostic).unwrap();
        assert_eq!(encoded["loc"]["line"], 7);
    }
}

#[test]
fn inspection_rejects_oversized_party_and_accepts_more_than_four_dummies() {
    let actors = ["mint", "sakiri", "zero", "iroi", "adler"]
        .into_iter()
        .map(|name| format!("{name} char lvl=80/80 awakening=0 talent=1,1,1 qte=1;\n"))
        .collect::<String>();
    let source = format!("options duration=20;\n{actors}target hp=1 def=0 resist=0; active mint;");
    let response = inspect_response(&InspectionRequest { source });
    assert!(response.party.is_none());
    assert_eq!(response.diagnostics[0].loc.line, 6);
    assert!(response.diagnostics[0].message.contains("four actors"));
    let source = CONFIG.replace(
        "active mint;",
        &format!("{}active mint;", "target hp=1 def=0 resist=0;\n".repeat(4)),
    );
    let response = inspect_response(&InspectionRequest { source });
    assert!(response.party.is_some(), "{:?}", response.diagnostics);
    assert!(response.diagnostics.is_empty());
}
