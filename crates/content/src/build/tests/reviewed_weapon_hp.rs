use super::*;

#[test]
fn reviewed_weapon_hp_adds_with_existing_percent_and_flat_stats_in_studio() {
    let (tables, _, _) = fireflies_fixture();
    for (character, weapon, bonuses, expected) in [
        (
            "character:1033",
            "weapon:fork_yuren",
            [10, 12, 14, 16, 18],
            [15000, 15200, 15400, 15600, 15800],
        ),
        (
            "character:1020",
            "weapon:fork_Nakupeda",
            [24, 28, 32, 36, 40],
            [16400, 16800, 17200, 17600, 18000],
        ),
    ] {
        for tier in 1..=5 {
            let input: studio_projection::Request = serde_json::from_value(json!({
                "content_id":"ntesim-current", "character_id":character,
                "level":80,"max_level":80,"awakening":[],"weapon_id":weapon,
                "weapon_level":80,"weapon_max_level":80,"refinement":tier,"set_count":0,
                "base_overrides":{"hp":10000},
                "additive_stats":{"hp_percent":"1/5","hp":"2000/1"}
            }))
            .unwrap();
            let report = studio_projection::project(&tables, &input).unwrap();
            assert_eq!(report.static_stats.hp, expected[tier - 1]);
            assert_eq!(
                ExactFraction::from_ratio(report.static_stats.hp_percent).unwrap(),
                ExactFraction::new((20 + bonuses[tier - 1]) as u128, 100).unwrap()
            );
            assert!(
                report
                    .model_assumptions
                    .keys()
                    .any(|id| id == "weapons.yuren_v1" || id == "weapons.nakupeda_v1")
            );
        }
    }
}

#[test]
fn reviewed_weapon_hp_native_build_settles_hp_and_fractional_def() {
    let (tables, coverage, template) = fireflies_fixture();
    for (character, weapon, bonus) in [
        (
            "character:1023",
            "weapon:fork_yuren",
            ExactFraction::new(1, 10).unwrap(),
        ),
        (
            "character:1020",
            "weapon:fork_Nakupeda",
            ExactFraction::new(6, 25).unwrap(),
        ),
    ] {
        let build = super::legal_native_build(&tables, &coverage, character, weapon, "set:Suit1");
        let bare = equipment::compile(&tables, &build).unwrap();
        let base_hp = receipt(&bare["known_attributes"]["HPMaxBase"]["receipt"]).unwrap();
        let hp = receipt(&bare["known_derived_stats"]["hp_receipt"]).unwrap();
        let expected_def = receipt(&bare["known_derived_stats"]["def_receipt"])
            .unwrap()
            .round_u64()
            .unwrap();
        let expected = hp
            .add(base_hp.mul(bonus).unwrap())
            .unwrap()
            .round_u64()
            .unwrap();
        let mut request = template.clone();
        request.builds.truncate(1);
        request.builds[0].build = build;
        request.builds[0].purchased_skill_levels = None;
        request.action_projection = None;
        request.scenario.actions.clear();
        request.scenario.action_metadata.clear();
        request.scenario.stochastic_crit = None;
        request.scenario.actors.truncate(1);
        request.gear_projection = Some(GearProjection {
            policy: GearProjectionPolicy::NativeWeaponCurveV1,
            hypothesis: "Reviewed native static HP".into(),
        });
        let actor = &mut request.scenario.actors[0];
        actor.character_id = character.into();
        actor.awakening = ntesim_data::AwakeningSelection::prefix(0);
        actor.weapon_id = weapon.into();
        actor.weapon_tier = 1;
        actor.set_id = Some("set:Suit1".into());
        actor.gear_procs.clear();
        actor.initial_hp_max = hp.round_u64().unwrap();
        for damaged in [false, true] {
            request.scenario.actors[0].initial_hp = if damaged {
                123
            } else {
                request.scenario.actors[0].initial_hp_max
            };
            let result = compile(&tables, &coverage, &request, &ExecutionLimits::default());
            let compiled = result.unwrap();
            let actor = &compiled.scenario.actors[0];
            assert_eq!(
                compiled.scenario.model_assumptions
                    [crate::build::assumptions::STAT_QUANTIZATION_HYPOTHESIS_ID.as_str()],
                crate::build::assumptions::STAT_QUANTIZATION_HYPOTHESIS_ID.description()
            );
            assert_eq!(actor.static_stats.hp, expected);
            assert_eq!(actor.static_stats.def, expected_def);
            assert_eq!(actor.initial_hp_max, expected);
            assert_eq!(actor.initial_hp, if damaged { 123 } else { expected });
            let report =
                run_game_scenario(&compiled.scenario, &ExecutionLimits::default()).unwrap();
            assert_eq!(report["final_actor_states"][0]["hp_max"], expected);
        }
    }
}
