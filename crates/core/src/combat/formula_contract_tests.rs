use super::*;

#[test]
fn defense_channels_sum_before_capping_and_snapshot_each_projection() {
    for stat in ["defense_ignore", "defense_shred"] {
        for (initial_value, amounts, expected_snapshot, expected_aggregate, expected_factor) in [
            (0.9, [0.3, -0.2], 100.0, 1.0, 1.0),
            (0.9, [-0.2, 0.3], 70.0, 1.0, 1.0),
            (0.1, [-0.3, 0.2], 0.0, 0.0, 0.5),
            (0.1, [0.2, -0.3], 30.0, 0.0, 0.5),
        ] {
            let mut initial = DynamicStats::new(100.0, 1000.0, 0.0);
            initial.apply_modifier(stat, initial_value).unwrap();
            let mut buffs = amounts
                .iter()
                .enumerate()
                .map(|(index, amount)| Buff {
                    id: format!("source-{index}"),
                    stat: stat.into(),
                    amount: Some(*amount),
                    amount_from: None,
                    stacking: StackingPolicy::ReplaceBySource,
                    duration_us: 0,
                })
                .collect::<Vec<_>>();
            let snapshot = Buff {
                id: "snapshot".into(),
                stat: "atk_flat".into(),
                amount: None,
                amount_from: Some(DynamicBuffAmountRule {
                    stat: stat.into(),
                    coefficient: 100.0,
                    maximum: None,
                    snapshot: SnapshotPolicy::AtApplication,
                }),
                stacking: StackingPolicy::ReplaceBySource,
                duration_us: 0,
            };
            let mut intermediate = buffs[..1].to_vec();
            intermediate.push(snapshot.clone());
            let (projected, _) = modifiers::project_active_buffs(&initial, &intermediate).unwrap();
            assert!((projected.atk_flat - expected_snapshot).abs() < 1e-12);
            buffs.push(snapshot);
            let (projected, _) = modifiers::project_active_buffs(&initial, &buffs).unwrap();
            assert!((projected.get_stat(stat).unwrap() - expected_aggregate).abs() < 1e-12);
            assert!((projected.atk_flat - expected_aggregate * 100.0).abs() < 1e-12);
            let factor = defense_factor(
                80,
                1080.0,
                projected.defense_shred,
                projected.defense_ignore,
                DamageKind::Elemental,
            )
            .unwrap();
            assert!((factor - expected_factor).abs() < 1e-12);
        }
    }
}

#[test]
fn cycle_helpers_reject_invalid_intensity_and_match_independent_values() {
    for invalid in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(cycle_intensity_zone(invalid).is_err());
        assert!(cycle_effect_zone(invalid).is_err());
        assert!(stain_factor(invalid, 1.2).is_err());
        assert!(hexed_fraction(invalid, 1.2).is_err());
    }
    for (intensity, q, e, stain, hexed) in
        [(0.0, 1.0, 1.0, 1.2, 0.2), (108.0, 1.18, 1.075, 1.29, 0.29)]
    {
        assert!((cycle_intensity_zone(intensity).unwrap() - q).abs() < 1e-12);
        assert!((cycle_effect_zone(intensity).unwrap() - e).abs() < 1e-12);
        assert!((stain_factor(intensity, 1.2).unwrap() - stain).abs() < 1e-12);
        assert!((hexed_fraction(intensity, 1.2).unwrap() - hexed).abs() < 1e-12);
    }
    assert!((cycle_effect_zone(f64::MAX).unwrap() - 1.2).abs() < 1e-12);
    for invalid_base in [-1.0, f64::NAN, f64::INFINITY, f64::MAX] {
        assert!(stain_factor(108.0, invalid_base).is_err());
        assert!(hexed_fraction(108.0, invalid_base).is_err());
    }
}

#[test]
fn damage_kinds_match_their_own_bonus_channel() {
    let mut stats = DynamicStats::new(1000.0, 10000.0, 0.0);
    for element in [
        "Anima",
        "Cosmos",
        "Chaos",
        "Incantation",
        "Lakshana",
        "Psyche",
    ] {
        stats.apply_modifier(element, 0.5).unwrap();
    }
    stats.apply_modifier("elemental:Mental", 0.2).unwrap();
    stats.apply_modifier("Physical", 0.3).unwrap();
    stats.damage_bonus = 0.1;
    stats.apply_modifier("action:skill", 0.4).unwrap();
    for (kind, element, expected) in [
        (DamageKind::Mental, "Psyche", 1700.0),
        (DamageKind::Mental, "Physical", 1700.0),
        (DamageKind::Elemental, "Physical", 1800.0),
        (DamageKind::Elemental, "Psyche", 2000.0),
        (DamageKind::Elemental, "Chaos", 2000.0),
        (DamageKind::True, "Psyche", 1000.0),
    ] {
        let hit = DirectHitSpec {
            base: Some(1000.0),
            ratio: None,
            stat: "atk".into(),
            flat: 0.0,
            element: element.into(),
            kind,
            crit_rule: CritRule::None,
            action_type: Some("skill".into()),
            damage_bonus: 0.0,
            crit_damage_bonus: 0.0,
            final_multiplier: 1.0,
            special_multiplier: 1.0,
        };
        let result = evaluate_direct_damage(&DirectDamageEvaluationRequest {
            attacker_level: 80,
            attacker_stats: &stats,
            target_defense: 0.0,
            target_resistance: 0.0,
            target_vulnerability: 0.0,
            hit: &hit,
            crit_mode: EvaluationCritMode::Expected,
            rounding: Rounding::NearestTiesEven,
            negative_res_model: NegativeResistanceModel::StandardRational,
        })
        .unwrap();
        assert!((result.raw_damage - expected).abs() < 1e-12);
    }
}
