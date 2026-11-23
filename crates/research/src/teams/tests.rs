use super::*;

const SAMPLE_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/profile-variants.csv"
));

const BATCH_SAMPLE_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/mixed-variants.csv"
));

#[test]
fn partial_override_map_never_falls_back_to_manual_profiles() {
    let variant = parse_published_variants_csv(SAMPLE_CSV).unwrap().remove(2);
    let baseline = compile_variant_scenario(&variant).unwrap();
    let mut overrides = variant
        .slots
        .iter()
        .zip(&baseline.actors)
        .map(|(slug, actor)| (slug.clone(), actor.static_stats.clone()))
        .collect::<std::collections::BTreeMap<_, _>>();
    overrides.remove("mint");
    let error = compile_variant_scenario_with_overrides(&variant, Some(&overrides)).unwrap_err();
    assert!(error.contains("missing actor override") && error.contains("mint"));
    assert!(
        compile_variant_scenario_with_overrides(&variant, Some(&std::collections::BTreeMap::new()))
            .is_err()
    );
}

#[test]
fn overridden_build_hp_initializes_vitals_and_zero_hp_is_rejected() {
    let variant = parse_published_variants_csv(SAMPLE_CSV).unwrap().remove(2);
    let baseline = compile_variant_scenario(&variant).unwrap();
    let mut overrides = variant
        .slots
        .iter()
        .zip(&baseline.actors)
        .map(|(slug, actor)| (slug.clone(), actor.static_stats.clone()))
        .collect::<std::collections::BTreeMap<_, _>>();
    overrides.get_mut("mint").unwrap().hp = 20000;
    let scenario = compile_variant_scenario_with_overrides(&variant, Some(&overrides)).unwrap();
    assert_eq!(scenario.actors[0].initial_hp, 20000);
    assert_eq!(scenario.actors[0].initial_hp_max, 20000);
    let report = run_game_scenario(&scenario, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["final_actor_states"][0]["hp_max"], 20000);
    assert_eq!(report["quality"], "incomplete");
    overrides.get_mut("mint").unwrap().hp = 0;
    let scenario = compile_variant_scenario_with_overrides(&variant, Some(&overrides)).unwrap();
    assert!(run_game_scenario(&scenario, &ExecutionLimits::default()).is_err());
}

#[test]
fn parse_published_variants_works() {
    let variants = parse_published_variants_csv(SAMPLE_CSV).unwrap();
    assert_eq!(variants.len(), 3);
    assert_eq!(variants[0].archetype, "Blackbird Nova");
    assert_eq!(variants[0].tier, "T0");
    assert_eq!(variants[0].slots[0], "blackbird");
    assert_eq!(variants[2].archetype, "Mint Hyper");
}

#[test]
fn sample_variants_execute_and_conserve_energy() {
    let limits = ExecutionLimits::default();
    let first_variant = parse_published_variants_csv(SAMPLE_CSV).unwrap().remove(0);
    let scenario = compile_variant_scenario(&first_variant).unwrap();
    assert!(
        scenario
            .model_assumptions
            .contains_key("published.preview_inputs_v1")
    );
    assert!(
        scenario
            .model_assumptions
            .contains_key("crit.fixed_noncrit_preview_v1")
    );
    let report = run_published_batch(SAMPLE_CSV, &limits).unwrap();
    assert_eq!(report.total_variants, 3);
    assert_eq!(report.passed_variants + report.failed_variants, 3);
    assert!(report.failed_variants > 0);
    // Rejected admission preserves conservation and execution invariants.
    // Incomplete execution still excludes the variant from successful metrics.
    assert!(report.all_invariants_passed);
    assert!(report.all_energy_conserved);

    // Successful variants expose completed metrics; unsupported variants retain
    // a failure reason and no zero-valued damage substitute.
    for v in report.variants.iter().filter(|variant| variant.success) {
        assert!(!v.readiness.game_dps_computed);
        assert!(!v.readiness.acceptance_ready);
        assert_eq!(v.status, "completed");
        assert!(v.hits_settled.unwrap() >= 4);
        assert!(v.total_damage_dealt.unwrap() > 0.0);
        assert!(v.final_target_hp.unwrap() < 10_000_000);
        assert_eq!(v.energy_conserved, Some(true));
        assert!(v.failure_reason.is_none());
    }
    for v in report.variants.iter().filter(|variant| !variant.success) {
        assert!(matches!(v.status.as_str(), "unsupported" | "failed"));
        assert!(
            v.failure_reason
                .as_deref()
                .is_some_and(|reason| !reason.is_empty())
        );
        if v.status == "unsupported" {
            assert!(v.total_damage_dealt.is_none());
        } else {
            assert!(v.total_damage_dealt.is_some());
            assert_eq!(v.invariants_passed, Some(true));
            assert_eq!(v.energy_conserved, Some(true));
        }
    }
}

#[test]
fn published_variants_preserve_results_and_visible_gaps() {
    let limits = ExecutionLimits::default();
    let report = run_published_batch(BATCH_SAMPLE_CSV, &limits).unwrap();

    assert_eq!(report.total_variants, 7);
    assert_eq!(
        report.passed_variants + report.failed_variants,
        report.total_variants
    );
    assert!(report.passed_variants > 0);
    assert!(report.failed_variants > 0);
    let shinku_variants = report
        .variants
        .iter()
        .filter(|variant| variant.slots[0] == "shinku")
        .collect::<Vec<_>>();
    assert_eq!(shinku_variants.len(), 2);
    for variant in shinku_variants {
        assert!(
            variant.success,
            "{}: {:?}",
            variant.archetype, variant.failure_reason
        );
        assert!(variant.hits_settled.unwrap() >= 3);
    }
    for variant in &report.variants {
        if variant.success {
            assert_eq!(variant.status, "completed");
            assert!(
                variant
                    .total_damage_dealt
                    .is_some_and(|damage| damage > 0.0)
            );
            assert!(variant.hits_settled.is_some_and(|hits| hits > 0));
            assert!(variant.failure_reason.is_none());
        } else {
            assert!(matches!(variant.status.as_str(), "unsupported" | "failed"));
            assert!(
                variant
                    .failure_reason
                    .as_deref()
                    .is_some_and(|reason| !reason.is_empty())
            );
            if variant.status == "unsupported" {
                assert!(variant.total_damage_dealt.is_none());
                assert!(variant.hits_settled.is_none());
            } else {
                assert!(variant.total_damage_dealt.is_some());
                assert!(variant.hits_settled.is_some());
                assert_eq!(variant.invariants_passed, Some(true));
                assert_eq!(variant.energy_conserved, Some(true));
            }
        }
    }
    let failure_reasons: Vec<_> = report
        .variants
        .iter()
        .filter_map(|variant| variant.failure_reason.as_deref())
        .collect();
    for expected in [
        "native action HP scaling is unsupported",
        "Zankou canonical Skill requires zero energy and its real five-second cooldown",
    ] {
        assert!(
            failure_reasons
                .iter()
                .any(|reason| reason.contains(expected)),
            "published variants no longer exercise unsupported case: {expected}"
        );
    }
    let encoded = serde_json::to_value(&report).unwrap();
    assert_eq!(encoded["status"], "INCOMPLETE");
    assert_eq!(encoded["quality"], "incomplete");
    assert_eq!(encoded["acceptance_ready"], false);
    assert!(!report.readiness.blockers.is_empty());
}

#[test]
fn published_shinku_skill_uses_native_three_packet_train_and_research_rules() {
    let variants = parse_published_variants_csv(BATCH_SAMPLE_CSV).unwrap();
    let shinku_variants = variants
        .iter()
        .filter(|variant| variant.slots[0] == "shinku")
        .collect::<Vec<_>>();
    assert_eq!(shinku_variants.len(), 2);

    let packet_ids = [
        "GE_Player_Shinku_Skill1_1_Damage",
        "GE_Player_Shinku_Skill1_2_Damage",
        "GE_Player_Shinku_Skill1_3_Damage",
    ];
    let coefficients = [0.296, 0.396, 1.364];
    let profile = get_character_profile("shinku").unwrap();
    assert_eq!(profile.external_atk_rate_claim.numerator, 37);
    assert_eq!(profile.external_atk_rate_claim.denominator, 125);
    for variant in shinku_variants {
        let scenario = compile_variant_scenario(variant).unwrap();
        let actor = &scenario.actors[0];
        assert_eq!(actor.character_id, "character:1076");
        assert_eq!(actor.weapon_id, profile.weapon_id);
        assert_eq!(
            serde_json::to_value(&actor.gear_procs).unwrap(),
            serde_json::to_value(&profile.gear_procs).unwrap()
        );
        let action_rate = scenario.actions.iter().find_map(|action| match action {
            GameActionCommand::InvokeAbility {
                actor_index: 0,
                ability_id,
                damage_effect_id,
                array_index,
                atk_rate,
                ..
            } if ability_id == "GA_Shinku_Skill" => Some((damage_effect_id, array_index, atk_rate)),
            _ => None,
        });
        let (damage_effect_id, array_index, action_rate) = action_rate
            .unwrap_or_else(|| panic!("Shinku action missing from {:?}", scenario.actions));
        assert_eq!(damage_effect_id, packet_ids[0]);
        assert_eq!(*array_index, 7);
        assert!(
            (action_rate.numerator as f64 / action_rate.denominator as f64 - 0.296).abs() < 1e-12
        );

        assert!(scenario.stochastic_crit.is_none());
        assert!(
            scenario
                .model_assumptions
                .contains_key("crit.fixed_noncrit_preview_v1")
        );

        let report = serde_json::from_value::<GameScenarioReport>(
            run_game_scenario(&scenario, &ExecutionLimits::default()).unwrap(),
        )
        .unwrap();
        let shinku_hits = report
            .hit_receipts
            .iter()
            .filter(|hit| hit.actor_index == 0 && hit.ability_id == "GA_Shinku_Skill")
            .collect::<Vec<_>>();
        assert_eq!(shinku_hits.len(), 3);
        assert_eq!(
            shinku_hits
                .iter()
                .map(|hit| hit.damage_effect_id.as_str())
                .collect::<Vec<_>>(),
            packet_ids
        );

        let target = &scenario.targets[0];
        let atk =
            actor.static_stats.atk.numerator as f64 / actor.static_stats.atk.denominator as f64;
        let element_bonus = actor.static_stats.elemental_bonus.numerator as f64
            / actor.static_stats.elemental_bonus.denominator as f64;
        let static_damage_bonus = actor.static_stats.damage_bonus.numerator as f64
            / actor.static_stats.damage_bonus.denominator as f64;
        assert_eq!(actor.character_level, 80);
        assert_eq!(target.level, 80);
        assert_eq!(target.defense, 1080);
        let level_defense_base = 600.0 + 6.0 * actor.character_level as f64;
        let defense_factor = level_defense_base / (level_defense_base + target.defense as f64);
        let resistance_factor =
            1.0 - target.resistance.numerator as f64 / target.resistance.denominator as f64;
        let mut expected_skill_damage = 0.0;
        for (hit, coefficient) in shinku_hits.iter().zip(coefficients) {
            assert!(
                (hit.scaling_rate.numerator as f64 / hit.scaling_rate.denominator as f64
                    - coefficient)
                    .abs()
                    < 1e-12
            );
            let expected_non_crit = atk
                * coefficient
                * (1.0 + static_damage_bonus + element_bonus)
                * defense_factor
                * resistance_factor;
            assert!((hit.non_crit_damage.to_f64() - expected_non_crit).abs() < 1e-8);
            assert!(!hit.critical);
            assert!(hit.crit_sample.is_none());
            let expected_damage = expected_non_crit;
            assert!((hit.damage.to_f64() - expected_damage).abs() < 1e-8);
            expected_skill_damage += expected_damage;
        }
        let actual_skill_damage = shinku_hits
            .iter()
            .map(|hit| hit.damage.to_f64())
            .sum::<f64>();
        assert!((actual_skill_damage - expected_skill_damage).abs() < 1e-8);

        let stochastic = crate::profiling::build_pilot_scenario(
            &variant.slots.join("+"),
            crate::acceptance_spec::AcceptanceScenarioKind::SingleTargetReal120,
            Ratio {
                numerator: 1,
                denominator: 1,
            },
            42,
            None,
        )
        .unwrap();
        let stochastic_report = serde_json::from_value::<GameScenarioReport>(
            run_game_scenario(&stochastic, &ExecutionLimits::default()).unwrap(),
        )
        .unwrap();
        let stochastic_skill_hits = stochastic_report
            .hit_receipts
            .iter()
            .filter(|hit| hit.actor_index == 0 && hit.ability_id == "GA_Shinku_Skill")
            .collect::<Vec<_>>();
        assert_eq!(stochastic_skill_hits.len(), 3);
        assert!(
            stochastic_skill_hits
                .iter()
                .all(|hit| hit.crit_sample.is_some())
        );
    }
}
