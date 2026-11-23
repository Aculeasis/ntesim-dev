use super::*;

#[test]
fn universal_skill_level_projection_supports_sakiri_and_mint_team() {
    let (mut tables, coverage, _) = skill_fixture();
    for name in ["1003AwakenEffect", "1046AwakenEffect", "1075AwakenEffect"] {
        tables.insert(
            name.into(),
            ntesim_data::resonance_levels::overlay(name.strip_suffix("AwakenEffect").unwrap())
                .unwrap(),
        );
    }

    for (character, ability, count, expected) in [
        ("character:1003", "GA_Sagiri_UltraSkill", 3, 1),
        ("character:1003", "GA_Sagiri_UltraSkill", 0, 0),
        ("character:1046", "GA_Female046_Skill", 3, 1),
        ("character:1046", "GA_Female046_QTE", 3, 0),
        ("character:1075", "GA_Oneiroi_Skill", 3, 1),
    ] {
        let (bonus, _) = crate::characters::resonance::bonus(
            &tables,
            character,
            ability,
            &ntesim_data::AwakeningSelection::prefix(count),
        )
        .unwrap();
        assert_eq!(bonus, expected, "{character} {ability} {count}");
    }

    assert!(ntesim_data::AwakeningSelection::selected(1..=7).is_err());

    let mut request: Request = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/skill-level-built-game-research.json"
    )))
    .unwrap();
    request.action_projection = Some(ActionProjection {
        policy: ActionProjectionPolicy::NativeSkillLevelV1,
        hypothesis: "Universal skill-level coordinate projection across released characters."
            .into(),
    });

    let sakiri_build_req: equipment::Request = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/build-stats.json"
    )))
    .unwrap();

    let mut sakiri_levels = BTreeMap::new();
    sakiri_levels.insert("GA_Sagiri_Melee".into(), 1);
    sakiri_levels.insert("GA_Sagiri_Skill".into(), 5);
    sakiri_levels.insert("GA_Sagiri_UltraSkill".into(), 10);

    request.builds.push(ActorBuild {
        slot: 1,
        build: sakiri_build_req,
        purchased_skill_levels: Some(sakiri_levels),
    });

    let sakiri_actor = crate::simulation::GameActorConfig {
        slot: 1,
        character_id: "character:1003".into(),
        character_level: 80,
        awakening: ntesim_data::AwakeningSelection::prefix(3),
        purchased_skill_level: 10,
        purchased_qte_level: Some(10),
        talent_levels: None,
        character_max_level: None,
        initial_hp: 22083,
        initial_hp_max: 22083,
        initial_energy: 100,
        energy_capacity: 100,
        initial_cycle: 0,
        initial_resources: Default::default(),
        cycle_gain_efficiency: ntesim_core::formula::Ratio::one(),
        weapon_id: "weapon:fork_mofeikesi".into(),
        weapon_tier: 1,
        set_id: Some("set:Suit11".into()),
        static_stats: crate::simulation::GameActorStaticStats {
            base_atk: 1094,
            atk: Ratio {
                numerator: 48331,
                denominator: 20,
            },
            hp: 22083,
            base_hp: None,
            hp_percent: Ratio::zero(),
            hp_flat: Ratio::zero(),
            def: 909,
            base_def: Some(909),
            def_percent: Ratio::zero(),
            def_flat: Ratio::zero(),
            crit_rate: Ratio {
                numerator: 1,
                denominator: 20,
            },
            crit_damage_bonus: Ratio {
                numerator: 1,
                denominator: 2,
            },
            healing_bonus: Ratio::zero(),
            shield_bonus: Ratio::zero(),
            break_intensity: Ratio::zero(),
            cycle_intensity: Ratio::zero(),
            break_efficiency_multiplier: Ratio::one(),
            mental_damage_bonus: Ratio::zero(),
            damage_bonus: Ratio::zero(),
            elemental_bonuses: Default::default(),
            elemental_bonus: Ratio {
                numerator: 27,
                denominator: 50,
            },
        },
        gear_procs: Vec::new(),
    };
    request.scenario.actors.push(sakiri_actor);

    request
        .scenario
        .actions
        .push(GameActionCommand::SwitchActor {
            timestamp_us: 4_600_000,
            target_actor_index: 1,
            cooldown_us: 1_000_000,
        });

    request
        .scenario
        .actions
        .push(GameActionCommand::InvokeAbility {
            timestamp_us: 5_200_000,
            actor_index: 1,
            ability_id: "GA_Sagiri_UltraSkill".into(),
            damage_effect_id: "GE_Player_Sagiri_UltraSkill1_Damage".into(),
            array_index: 999,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 100,
            recovery_us: 1_000_000,
            cooldown_us: 20_000_000,
            hit_delay_us: 100_000,
        });

    let compiled = compile(&tables, &coverage, &request, &ExecutionLimits::default()).unwrap();
    let sakiri_action_receipt = &compiled.receipt["action_receipts"][3];
    assert_eq!(sakiri_action_receipt["compiled_array_index"], 10);
    assert_eq!(sakiri_action_receipt["skill_level"]["effective_level"], 11);
    assert_eq!(
        sakiri_action_receipt["skill_level"]["resonance"]["applied_bonus"],
        1
    );

    assert!(
        compiled
            .scenario
            .model_assumptions
            .contains_key("skill_levels.coordinates_v1")
    );
    assert!(
        compiled
            .scenario
            .model_assumptions
            .contains_key("skill_levels.resonance_3_v1")
    );

    let execution = run_game_scenario(&compiled.scenario, &ExecutionLimits::default()).unwrap();
    assert_eq!(execution["execution_completed"], true);

    let mut request_a2 = request.clone();
    request_a2.scenario.actors[1].awakening = ntesim_data::AwakeningSelection::prefix(2);
    let compiled_a2 =
        compile(&tables, &coverage, &request_a2, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        compiled_a2.receipt["action_receipts"][3]["compiled_array_index"],
        9
    );
    assert_eq!(
        compiled_a2.receipt["action_receipts"][3]["skill_level"]["effective_level"],
        10
    );
    assert_eq!(
        compiled_a2.receipt["action_receipts"][3]["skill_level"]["resonance"]["applied_bonus"],
        0
    );
}
