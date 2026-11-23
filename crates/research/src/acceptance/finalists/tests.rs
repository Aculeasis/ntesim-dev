use super::*;
use crate::simulation::GameActorStaticStats;

fn test_actor_overrides(slugs: &[&str]) -> BTreeMap<String, ActorOverride> {
    let mut map = BTreeMap::new();
    for slug in slugs {
        let p = get_character_profile(slug).unwrap();
        map.insert(
            slug.to_string(),
            ActorOverride {
                weapon_id: Some(p.weapon_id.to_string()),
                weapon_tier: Some(p.weapon_tier),
                set_id: None,
                static_stats: GameActorStaticStats {
                    base_atk: p.base_atk,
                    atk: p.atk_ratio,
                    hp: p.hp,
                    base_hp: None,
                    hp_percent: Ratio::zero(),
                    hp_flat: Ratio::zero(),
                    def: p.def,
                    base_def: None,
                    def_percent: Ratio::zero(),
                    def_flat: Ratio::zero(),
                    crit_rate: Ratio {
                        numerator: 1,
                        denominator: 2,
                    },
                    crit_damage_bonus: Ratio {
                        numerator: 1,
                        denominator: 1,
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
                        numerator: 1,
                        denominator: 10,
                    },
                },
            },
        );
    }
    map
}

fn fixture_data() -> (Value, &'static str) {
    let coverage = ntesim_data::character::released_inventory();
    let variants = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/finalists.csv"
    ));
    (coverage, variants)
}

#[test]
fn finalist_evaluation_verifies_sustainability_timing_and_classification() {
    let (coverage, variants_csv) = fixture_data();

    let comp = "blackbird+daffodil+haniel+sakiri";
    let overrides = test_actor_overrides(&["blackbird", "daffodil", "haniel", "sakiri"]);

    let req = FinalistValidationRequest {
        schema_version: 1,
        content_id: "ntesim-current".into(),
        hypothesis: DEFAULT_FINALIST_HYPOTHESIS.into(),
        composition: comp.into(),
        scenario_kind: AcceptanceScenarioKind::SingleTargetReal120,
        seeds: vec![42, 43, 44, 45, 46],
        timing_multipliers: default_timing_multipliers(),
        research_crit: None,
        actor_overrides: overrides,
        baseline_damage: Some(100_000.0),
        baseline_archetype: Some("Blackbird Nova".into()),
    };

    let report =
        evaluate_finalist(&coverage, variants_csv, &req, &ExecutionLimits::default()).unwrap();

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.report_kind, "validate-finalist-evaluation-v1");
    assert_eq!(report.status, "INCOMPLETE");
    assert!(!report.game_dps_computed);
    assert!(!report.acceptance_ready);
    assert!(report.is_published_variant);
    assert_eq!(
        report.classification,
        FinalistClassification::PublishedVariant
    );
    assert!(!report.resource_sustainable);
    assert!(!report.superiority_confirmed);
    assert!(report.all_invariants_passed);
    assert!(report.all_energy_conserved);
    assert_eq!(report.timing_summaries.len(), 3);
    assert!(report.nominal_mean_damage > 0.0);

    let unconv_comp = "chiz+lacrimosa+skia+nanally";
    let unconv_overrides = test_actor_overrides(&["chiz", "lacrimosa", "skia", "nanally"]);

    let unconv_req = FinalistValidationRequest {
        schema_version: 1,
        content_id: "ntesim-current".into(),
        hypothesis: DEFAULT_FINALIST_HYPOTHESIS.into(),
        composition: unconv_comp.into(),
        scenario_kind: AcceptanceScenarioKind::SingleTargetReal120,
        seeds: vec![10, 20, 30],
        timing_multipliers: default_timing_multipliers(),
        research_crit: None,
        actor_overrides: unconv_overrides,
        baseline_damage: Some(50_000.0),
        baseline_archetype: Some("Lacrimosa Quickswap".into()),
    };

    let unconv_report = evaluate_finalist(
        &coverage,
        variants_csv,
        &unconv_req,
        &ExecutionLimits::default(),
    )
    .unwrap();

    assert!(!unconv_report.is_published_variant);
    assert!(!unconv_report.resource_sustainable);
    assert!(!unconv_report.superiority_confirmed);
    assert_eq!(unconv_report.timing_summaries.len(), 3);
}

#[test]
fn explicit_empty_crit_policy_is_rejected_instead_of_replaced_and_nominal_timing_is_identified_by_value()
 {
    let (coverage, variants) = fixture_data();
    let mut request = FinalistValidationRequest {
        schema_version: 1,
        content_id: "ntesim-current".into(),
        hypothesis: "test explicit inputs".into(),
        composition: "mint+sakiri+zero+iroi".into(),
        scenario_kind: AcceptanceScenarioKind::SingleTargetReal120,
        seeds: vec![42],
        timing_multipliers: vec![
            Ratio {
                numerator: 11,
                denominator: 10,
            },
            Ratio {
                numerator: 1,
                denominator: 1,
            },
        ],
        research_crit: Some(ResearchCritPolicy {
            roll_policy: ResearchCritRollPolicy::IndependentPerTargetHitV1,
            hypothesis: "explicit empty policy".into(),
            eligible_effects: BTreeMap::new(),
            actor_sequences: BTreeMap::new(),
        }),
        actor_overrides: test_actor_overrides(&["mint", "sakiri", "zero", "iroi"]),
        baseline_damage: None,
        baseline_archetype: None,
    };
    assert!(
        evaluate_finalist(&coverage, variants, &request, &ExecutionLimits::default())
            .unwrap_err()
            .contains("eligibility")
    );
    request.research_crit = None;
    let report =
        evaluate_finalist(&coverage, variants, &request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report.nominal_mean_damage,
        report.timing_summaries[1].mean_damage
    );
    assert_eq!(report.timing_summaries[0].multiplier_label, "11/10x");
    request.baseline_damage = Some(0.0);
    assert!(evaluate_finalist(&coverage, variants, &request, &ExecutionLimits::default()).is_err());
}
