use super::*;

#[test]
fn noros_native_build_projects_a_legal_jiuyuan_skill() {
    let (mut tables, coverage, mut request) = fireflies_fixture();
    tables.insert(
        "1055AwakenEffect".into(),
        ntesim_data::resonance_levels::overlay("1055").unwrap(),
    );
    request.scenario.actors[0].character_id = "character:1055".into();
    request.scenario.actors[0].weapon_id = "weapon:fork_nonos".into();
    request.scenario.actors[0].gear_procs.clear();
    request.scenario.stochastic_crit = None;
    request.scenario.actions = vec![GameActionCommand::InvokeAbility {
        timestamp_us: 0,
        actor_index: 0,
        ability_id: "GA_Kuhara_Skill".into(),
        damage_effect_id: "GE_Player_Kuhara_Skill1_Damage".into(),
        array_index: 7,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 0,
    }];
    request.builds[0].build.character_id = "character:1055".into();
    request.builds[0].build.weapon_id = "weapon:fork_nonos".into();
    request.builds[0].purchased_skill_levels =
        Some(BTreeMap::from([("GA_Kuhara_Skill".into(), 8)]));
    request.action_projection = Some(ActionProjection {
        policy: ActionProjectionPolicy::NativeSkillLevelV1,
        hypothesis: "Project Jiuyuan's purchased Skill coordinate from frozen native data.".into(),
    });
    request.gear_projection = Some(GearProjection {
        policy: GearProjectionPolicy::NativeWeaponCurveV1,
        hypothesis: crate::weapons::noros::HYPOTHESIS_ID.description().into(),
    });
    let limits = ExecutionLimits::default();
    let native_build = equipment::compile(&tables, &request.builds[0].build).unwrap();
    let element = ntesim_data::character::definition(&format!("character:{}", "1055"))
        .unwrap()
        .element
        .native_key();
    let stats = project(&native_build, element_attribute(element).unwrap()).unwrap();
    request.scenario.actors[0].initial_hp = stats.hp;
    request.scenario.actors[0].initial_hp_max = stats.hp;
    let compiled = compile(&tables, &coverage, &request, &limits).unwrap();
    assert_eq!(
        compiled.receipt["gear_receipts"][0]["compiled_proc"]["kind"],
        "noros_atk"
    );
    let report = run_game_scenario(&compiled.scenario, &limits).unwrap();
    assert_eq!(report["execution_completed"], true);
    assert_eq!(report["hit_receipts"].as_array().unwrap().len(), 6);
    let mut bare = compiled.scenario;
    bare.actors[0].gear_procs.clear();
    let baseline = run_game_scenario(&bare, &limits).unwrap();
    let atk = ExactFraction::from_ratio(bare.actors[0].static_stats.atk).unwrap();
    let grant =
        ExactFraction::new(u128::from(bare.actors[0].static_stats.base_atk) * 4, 25).unwrap();
    let multiplier = atk.add(grant).unwrap().div(atk).unwrap();
    assert_eq!(
        serde_json::from_value::<ExactFraction>(
            report["hit_receipts"][0]["non_crit_damage"].clone()
        )
        .unwrap(),
        serde_json::from_value::<ExactFraction>(
            baseline["hit_receipts"][0]["non_crit_damage"].clone()
        )
        .unwrap()
        .mul(multiplier)
        .unwrap()
    );
}
