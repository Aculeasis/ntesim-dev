use super::*;

#[test]
fn quiet_manor_equipped_build_binds_native_proc_and_excludes_mint_anima() {
    let (tables, coverage, mut request) = fireflies_fixture();
    request.scenario.actors.truncate(1);
    request.builds.truncate(1);
    request.scenario.stochastic_crit = None;
    request.scenario.action_metadata.clear();
    let actor = &mut request.scenario.actors[0];
    actor.character_id = "character:1019".into();
    actor.awakening = ntesim_data::AwakeningSelection::prefix(0);
    actor.talent_levels = Some([1; 3]);
    actor.weapon_id = "weapon:fork_lingganzhongjiezhe".into();
    actor.weapon_tier = 1;
    actor.set_id = Some("set:Suit12".into());
    actor.gear_procs.clear();
    let build = &mut request.builds[0].build;
    build.character_id = actor.character_id.clone();
    build.weapon_id = actor.weapon_id.clone();
    build.weapon_tier = 1;
    build.set_id = "set:Suit12".into();
    let report = equipment::compile(&tables, build).unwrap();
    let stats = project(&report, "DamageUpNatureBase").unwrap();
    actor.initial_hp = stats.hp;
    actor.initial_hp_max = stats.hp;
    request.builds[0].purchased_skill_levels = None;
    request.action_projection = Some(ActionProjection {
        policy: ActionProjectionPolicy::NativeAtkCoordinateV1,
        hypothesis: "Bind native level-one Mint packet coefficients.".into(),
    });
    request.gear_projection = Some(GearProjection {
        policy: GearProjectionPolicy::NativeWeaponCurveV1,
        hypothesis: "Bind exact equipped Mind Royale T1 parameters.".into(),
    });
    request.scenario.targets[0].defense = 0;
    request.scenario.targets[0].resistance = Ratio::zero();
    request.scenario.actions = (0..6)
        .map(|index| {
            let burst = index == 5;
            GameActionCommand::InvokeAbility {
                timestamp_us: index * 800_000,
                actor_index: 0,
                ability_id: if burst {
                    "GA_Mint019_UltraSkill"
                } else {
                    "GA_Mint019_Melee"
                }
                .into(),
                damage_effect_id: if burst {
                    "GE_Player_Mint_UtraSkill1_Damage_New"
                } else {
                    "GE_Player_Mint_Melee1_Damage_New"
                }
                .into(),
                array_index: 0,
                atk_rate: Ratio::one(),
                scaling_stat: crate::simulation::NativeScalingStat::Atk,
                energy_cost: if burst { 100 } else { 0 },
                recovery_us: 0,
                cooldown_us: 0,
                hit_delay_us: 0,
            }
        })
        .collect();
    let compiled = compile(&tables, &coverage, &request, &ExecutionLimits::default()).unwrap();
    let actor = &compiled.scenario.actors[0];
    assert_eq!(
        actor.static_stats.mental_damage_bonus,
        Ratio {
            numerator: 1,
            denominator: 10
        }
    );
    let proc = actor
        .gear_procs
        .iter()
        .find(|proc| matches!(proc, GearProcConfig::QuietManorMental { .. }))
        .unwrap()
        .clone();
    assert_eq!(
        serde_json::to_value(&proc).unwrap(),
        json!({"kind":"quiet_manor_mental","mental_bonus_per_stack":"3/25","duration_us":6000000,"max_stacks":3})
    );
    assert!(compiled.receipt["actor_build_receipts"][0]["native_set_projection"].is_object());
    let report = run_game_scenario(&compiled.scenario, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true);
    assert_eq!(report["hits_settled"], 15);
    let multiplier = ExactFraction::new(1, 1)
        .unwrap()
        .add(ExactFraction::from_ratio(actor.static_stats.damage_bonus).unwrap())
        .unwrap()
        .add(ExactFraction::from_ratio(actor.static_stats.elemental_bonuses["NATURE"]).unwrap())
        .unwrap();
    for (hit, rate) in report["hit_receipts"].as_array().unwrap()[..8]
        .iter()
        .zip([104, 231, 170, 262, 341, 184, 255, 523])
    {
        let expected = ExactFraction::from_ratio(actor.static_stats.atk)
            .unwrap()
            .mul(ExactFraction::new(rate, 1000).unwrap())
            .unwrap()
            .mul(multiplier)
            .unwrap();
        assert_eq!(
            hit["non_crit_damage"],
            serde_json::to_value(expected).unwrap()
        );
    }
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|hit| hit["static_mental_damage_bonus"] == json!("0/1"))
    );
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "quiet_manor_mental_stack")
            .count(),
        5
    );
    request.scenario.actors[0].gear_procs = vec![GearProcConfig::QuietManorMental {
        mental_bonus_per_stack: Ratio::one(),
        duration_us: 1,
        max_stacks: 1,
    }];
    let rebound = compile(&tables, &coverage, &request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        serde_json::to_value(&rebound.scenario.actors[0].gear_procs[0]).unwrap(),
        serde_json::to_value(&proc).unwrap()
    );
    let mut wrong_units = tables.clone();
    wrong_units.insert(
        "CT_Equipmentbuff".into(),
        ntesim_data::set_parameters::overlay(),
    );
    wrong_units.get_mut("CT_Equipmentbuff").unwrap()["rows"]["Psychically_4"]["Keys"][0]["Value"] =
        json!(12);
    assert!(
        compile(
            &wrong_units,
            &coverage,
            &request,
            &ExecutionLimits::default()
        )
        .unwrap_err()
        .contains("units")
    );
    request.scenario.actors[0].gear_procs = vec![proc.clone(), proc];
    assert!(
        compile(&tables, &coverage, &request, &ExecutionLimits::default())
            .unwrap_err()
            .contains("duplicate Quiet Manor")
    );
    request.scenario.actors[0].gear_procs.clear();
    request.builds[0].build.set_count = 5;
    assert!(compile(&tables, &coverage, &request, &ExecutionLimits::default()).is_err());
}
