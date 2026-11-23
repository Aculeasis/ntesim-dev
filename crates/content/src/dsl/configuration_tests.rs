use super::{configuration::compile_configuration, *};

fn compile(
    source: &str,
) -> Result<configuration::CompiledConfiguration, Vec<validation::Diagnostic>> {
    compile_configuration(
        &parse_nte(source).unwrap(),
        ntesim_data::stat_projection::frozen_tables(),
    )
}

const MINIMAL: &str = "options duration=20; mint char lvl=80/80 awakening=0 talent=8,8,8 qte=8; target hp=100000 def=0 resist=0; active mint;";

#[test]
fn configuration_compiles_builds_through_the_shared_native_projector() {
    let config = compile(&format!("{MINIMAL} mint attack;")).unwrap();
    assert_eq!(config.actors[0].actor.character_id, "character:1019");
    let actor = &config.actors[0].actor;
    assert_eq!(actor.initial_hp, actor.static_stats.hp);
    assert_eq!(actor.initial_energy, actor.energy_capacity);
    assert_eq!(actor.talent_levels, Some([8, 8, 8]));
    assert_eq!(config.rotation.len(), 1);
    assert_eq!(config.active_actor_index, 0);
}

#[test]
fn exact_starting_hp_and_repeated_stats_use_final_projected_max_hp() {
    let source = MINIMAL.replace(
        "talent=8,8,8",
        "talent=8,8,8 +params=[start_hp%=0.3-0.2,start_energy=0]",
    );
    let source = source.replace(
        "target hp=",
        "mint add stats hp=100 atk%=0.1; mint add stats hp=50 atk%=0.2; target hp=",
    );
    let config = compile(&source).unwrap();
    let actor = &config.actors[0];
    assert_eq!(actor.actor.initial_hp, actor.actor.initial_hp_max / 10);
    assert_eq!(actor.actor.initial_energy, 0);
    assert_eq!(
        actor.contributions["atk_percent"],
        ntesim_core::formula::Ratio {
            numerator: 3,
            denominator: 10
        }
    );
    assert_eq!(actor.contributions["hp"].numerator, 150);
}

#[test]
fn configuration_constants_are_evaluated_before_their_consuming_declarations() {
    let source = MINIMAL
        .replace("mint char", "const n=4; mint char")
        .replace("talent=8,8,8", "talent=n*2,n*2,n*2")
        .replace("target hp=100000", "target hp=n*25000");
    let config = compile(&source).unwrap();
    assert_eq!(config.actors[0].actor.talent_levels, Some([8, 8, 8]));
    assert_eq!(config.targets[0].hp_current, 100_000);
    assert!(config.constants.0.contains_key("n"));
}

#[test]
fn public_presets_compile_equipment_and_keep_rotation_source_nodes() {
    for source in [
        include_str!("../../../../presets/studio/s1.nte"),
        include_str!("../../../../presets/studio/s2.nte"),
        include_str!("../../../../presets/studio/s3.nte"),
        include_str!("../../../../presets/studio/s4.nte"),
        include_str!("../../../../presets/studio/s5.nte"),
        include_str!("../../../../presets/studio/language-reference.nte"),
    ] {
        let config = compile(source).unwrap();
        assert!(!config.actors.is_empty());
        assert!(!config.rotation.is_empty());
        assert!(!config.actors[0].actor.weapon_id.is_empty());
        assert!(!config.model_assumptions.is_empty());
    }
}

#[test]
fn illegal_build_inputs_fail_at_their_owning_declaration() {
    for declaration in [
        "mint add stats unknown=1;",
        "mint add stats atk=-1;",
        "mint add stats cr=true;",
        "mint add weapon=missing refine=1;",
        "mint add weapon=mindroyale refine=0;",
        "mint add weapon=mindroyale;",
        "mint add cartridge=firefliesandtheforest count=5;",
        "mint add weapon=mindroyale refine=1 other=1;",
        "mint add cartridge=missing count=4;",
    ] {
        let source = MINIMAL.replace("target hp=", &format!("{declaration} target hp="));
        assert!(compile(&source).is_err(), "{declaration}");
    }
    for parameters in [
        "start_hp=0",
        "start_hp%=0",
        "start_hp%=1.1",
        "start_hp%=0.00000001",
        "start_hp=999999999",
        "start_energy=99999",
        "start_hp=10,start_hp%=0.5",
    ] {
        assert!(
            compile(&MINIMAL.replace(
                "talent=8,8,8",
                &format!("talent=8,8,8 +params=[{parameters}]")
            ))
            .is_err(),
            "{parameters}"
        );
    }
    let script = parse_nte(MINIMAL).unwrap();
    assert!(compile_configuration(&script, &Default::default()).is_ok());
    let invalid =
        std::collections::BTreeMap::from([("DT_SkillDamageData".into(), serde_json::Value::Null)]);
    assert!(compile_configuration(&script, &invalid).is_err());
}
