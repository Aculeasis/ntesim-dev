use super::*;

#[test]
fn direct_damage_matches_independent_hand_calculation() {
    // Lv80 attacker vs Lv80 enemy (DEF 1080, RES 0.20)
    // base = 1000, bonus = 0.50, crit_rate = 0.50, crit_damage = 1.00
    // DEF factor: (600 + 480) / (1080 + 1080) = 0.5
    // RES factor: 1 - 0.2 = 0.8
    // Crit factor: 1 + 1.0 * 0.5 = 1.5
    // Bonus factor: 1 + 0.5 = 1.5
    // Damage = 1000 * 1.5 * 1.5 * 0.5 * 0.8 = 900.0
    let mut stats = DynamicStats::new(1000.0, 10000.0, 800.0);
    stats.crit_rate = 0.5;
    stats.crit_damage = 1.0;
    stats.damage_bonus = 0.5;

    let hit = DirectHitSpec {
        base: Some(1000.0),
        ratio: None,
        stat: "atk".into(),
        flat: 0.0,
        element: "Incantation".into(),
        kind: DamageKind::Elemental,
        crit_rule: CritRule::Normal,
        action_type: None,
        damage_bonus: 0.0,
        crit_damage_bonus: 0.0,
        final_multiplier: 1.0,
        special_multiplier: 1.0,
    };

    let req = DirectDamageEvaluationRequest {
        attacker_level: 80,
        attacker_stats: &stats,
        target_defense: 1080.0,
        target_resistance: 0.20,
        target_vulnerability: 0.0,
        hit: &hit,
        crit_mode: EvaluationCritMode::Expected,
        rounding: Rounding::Floor,
        negative_res_model: NegativeResistanceModel::StandardRational,
    };
    let result = evaluate_direct_damage(&req).unwrap();

    assert_eq!(result.raw_damage, 900.0);
    assert_eq!(result.quantized_damage, 900);
    assert_eq!(result.defense_factor, 0.5);
    assert_eq!(result.resistance_factor, 0.8);
    assert_eq!(result.crit_factor, 1.5);
}

#[test]
fn nested_combat_inputs_reject_unknown_fields_instead_of_discarding_typos() {
    let mut stats = serde_json::to_value(DynamicStats::new(1000.0, 10000.0, 800.0)).unwrap();
    stats["atk_flatt"] = serde_json::json!(250.0);
    assert!(serde_json::from_value::<DynamicStats>(stats).is_err());

    let hit = DirectHitSpec {
        base: Some(1000.0),
        ratio: None,
        stat: "atk".into(),
        flat: 0.0,
        element: "Physical".into(),
        kind: DamageKind::True,
        crit_rule: CritRule::None,
        action_type: None,
        damage_bonus: 0.0,
        crit_damage_bonus: 0.0,
        final_multiplier: 1.0,
        special_multiplier: 1.0,
    };
    let mut hit = serde_json::to_value(hit).unwrap();
    hit["damage_bonuz"] = serde_json::json!(0.5);
    assert!(serde_json::from_value::<DirectHitSpec>(hit).is_err());

    let buff = Buff {
        id: "test".into(),
        stat: "atk_flat".into(),
        amount: Some(100.0),
        amount_from: None,
        stacking: StackingPolicy::ReplaceBySource,
        duration_us: 1_000,
    };
    let mut buff = serde_json::to_value(buff).unwrap();
    buff["duration"] = serde_json::json!(10_000);
    assert!(serde_json::from_value::<Buff>(buff).is_err());

    let rule = DynamicBuffAmountRule {
        stat: "base_atk".into(),
        coefficient: 0.2,
        maximum: None,
        snapshot: SnapshotPolicy::AtApplication,
    };
    let mut rule = serde_json::to_value(rule).unwrap();
    rule["maximun"] = serde_json::json!(500.0);
    assert!(serde_json::from_value::<DynamicBuffAmountRule>(rule).is_err());

    let healing = HealingSpec {
        ratio: 1.0,
        stat: "atk".into(),
        flat: 0.0,
        outgoing_bonus: 0.0,
        incoming_bonus: 0.0,
        rounding: Rounding::Floor,
    };
    let mut healing = serde_json::to_value(healing).unwrap();
    healing["incoming_bonuz"] = serde_json::json!(0.2);
    assert!(serde_json::from_value::<HealingSpec>(healing).is_err());

    let shield = ShieldSpec {
        ratio: 1.0,
        stat: "atk".into(),
        flat: 0.0,
        shield_bonus: 0.0,
        rounding: Rounding::Floor,
    };
    let mut shield = serde_json::to_value(shield).unwrap();
    shield["shield_bonuz"] = serde_json::json!(0.2);
    assert!(serde_json::from_value::<ShieldSpec>(shield).is_err());
}

#[test]
fn mental_damage_bypasses_defense_and_retains_resistance() {
    let stats = DynamicStats::new(1000.0, 10000.0, 800.0);
    let hit = DirectHitSpec {
        base: Some(1000.0),
        ratio: None,
        stat: "atk".into(),
        flat: 0.0,
        element: "Mental".into(),
        kind: DamageKind::Mental,
        crit_rule: CritRule::None,
        action_type: None,
        damage_bonus: 0.0,
        crit_damage_bonus: 0.0,
        final_multiplier: 1.0,
        special_multiplier: 1.0,
    };

    let req = DirectDamageEvaluationRequest {
        attacker_level: 80,
        attacker_stats: &stats,
        target_defense: 1_000_000.0,
        target_resistance: 0.20,
        target_vulnerability: 0.0,
        hit: &hit,
        crit_mode: EvaluationCritMode::Expected,
        rounding: Rounding::Floor,
        negative_res_model: NegativeResistanceModel::StandardRational,
    };
    let result = evaluate_direct_damage(&req).unwrap();

    // 1000 * 1.0 (crit) * 1.0 (def) * 0.8 (res) = 800.0
    assert_eq!(result.defense_factor, 1.0);
    assert_eq!(result.resistance_factor, 0.8);
    assert_eq!(result.raw_damage, 800.0);
    assert_eq!(result.quantized_damage, 800);
}

#[test]
fn true_damage_bypasses_defense_and_resistance() {
    let stats = DynamicStats::new(1000.0, 10000.0, 800.0);
    let hit = DirectHitSpec {
        base: Some(1000.0),
        ratio: None,
        stat: "atk".into(),
        flat: 0.0,
        element: "True".into(),
        kind: DamageKind::True,
        crit_rule: CritRule::Normal,
        action_type: None,
        damage_bonus: 99.0, // ignored by true damage
        crit_damage_bonus: 0.0,
        final_multiplier: 1.0,
        special_multiplier: 1.0,
    };

    let req = DirectDamageEvaluationRequest {
        attacker_level: 80,
        attacker_stats: &stats,
        target_defense: 5000.0,
        target_resistance: 0.80,
        target_vulnerability: 0.0,
        hit: &hit,
        crit_mode: EvaluationCritMode::Expected,
        rounding: Rounding::Floor,
        negative_res_model: NegativeResistanceModel::StandardRational,
    };
    let result = evaluate_direct_damage(&req).unwrap();

    assert_eq!(result.raw_damage, 1000.0);
    assert_eq!(result.quantized_damage, 1000);
}

#[test]
fn negative_resistance_models_distinguish_conflicts() {
    let standard = resistance_factor(-0.20, NegativeResistanceModel::StandardRational).unwrap();
    let drive = resistance_factor(-0.20, NegativeResistanceModel::LinearScaled).unwrap();

    // standard: 1 - (-0.2 / 1.2) = 1 + 1/6 = 7/6 = 1.1666666666666667
    assert!((standard - 7.0 / 6.0).abs() < 1e-12);
    // drive: 1 - (-0.2 / 1.1) = 1 + 2/11 = 13/11 = 1.1818181818181819
    assert!((drive - 13.0 / 11.0).abs() < 1e-12);
}

#[test]
fn published_reference_break_example_matches_zero_mint_lv1() {
    // Zero+Mint Lv1 vs frame Lv9 (enemy DEF 654)
    // base_break = 91, original_cap = 45, intensity = 0
    // Sum of break damage for res = 0.20 and res = 0.16 is 1076.66 (floor = 1076)
    let req1 = BreakDamageRequest {
        base_break: 91.0,
        intensity: 0.0,
        original_cap: 45.0,
        attacker_level: 1,
        enemy_defense: 654.0,
        resistance: 0.20,
        extra_bonus: 0.0,
        shred: 0.0,
        ignore: 0.0,
        special: 1.0,
        negative_res_model: NegativeResistanceModel::StandardRational,
    };
    let d1 = break_damage(&req1).unwrap();

    let req2 = BreakDamageRequest {
        base_break: 91.0,
        intensity: 0.0,
        original_cap: 45.0,
        attacker_level: 1,
        enemy_defense: 654.0,
        resistance: 0.16,
        extra_bonus: 0.0,
        shred: 0.0,
        ignore: 0.0,
        special: 1.0,
        negative_res_model: NegativeResistanceModel::StandardRational,
    };
    let d2 = break_damage(&req2).unwrap();

    let sum = d1 + d2;
    assert!((sum - 1076.66).abs() < 0.01);
    assert_eq!(quantize(sum, Rounding::Floor), 1076);
}

#[test]
fn dynamic_buff_capping_and_base_atk_scaling() {
    let stats_caster = DynamicStats::new(2500.0, 10000.0, 800.0);
    let buff = Buff {
        id: "iroi_buff".into(),
        stat: "atk_flat".into(),
        amount: None,
        amount_from: Some(DynamicBuffAmountRule {
            stat: "base_atk".into(),
            coefficient: 0.20,
            maximum: Some(500.0),
            snapshot: SnapshotPolicy::AtApplication,
        }),
        stacking: StackingPolicy::ReplaceBySource,
        duration_us: 20_000_000,
    };

    assert_eq!(buff.compute_amount(&stats_caster).unwrap(), 500.0);

    let stats_caster_high = DynamicStats::new(3500.0, 10000.0, 800.0);
    assert_eq!(buff.compute_amount(&stats_caster_high).unwrap(), 500.0);

    let stats_caster_low = DynamicStats::new(1000.0, 10000.0, 800.0);
    assert_eq!(buff.compute_amount(&stats_caster_low).unwrap(), 200.0);
}

#[test]
fn healing_and_shield_evaluations_respect_caps_and_bonuses() {
    let mut stats = DynamicStats::new(2000.0, 20000.0, 900.0);
    stats.healing_bonus = 0.20;
    stats.shield_bonus = 0.15;

    let heal_spec = HealingSpec {
        ratio: 1.5,
        stat: "atk".into(),
        flat: 500.0,
        outgoing_bonus: 0.0,
        incoming_bonus: 0.0,
        rounding: Rounding::Floor,
    };

    let heal_result = evaluate_healing(&stats, 18000, 20000, &heal_spec).unwrap();
    assert_eq!(heal_result.raw_healing, 4200.0);
    assert_eq!(heal_result.effective_healing, 2000);
    assert_eq!(heal_result.overheal, 2200);

    let shield_spec = ShieldSpec {
        ratio: 0.20,
        stat: "hp".into(),
        flat: 400.0,
        shield_bonus: 0.0,
        rounding: Rounding::Floor,
    };
    let shield_result = evaluate_shield(&stats, &shield_spec).unwrap();
    assert_eq!(shield_result.effective_shield, 5060);
}

#[test]
fn healing_uses_explicit_recipient_bonus_and_continuing_vitals() {
    let mut healer = DynamicStats::new(100.0, 1000.0, 0.0);
    healer.healing_bonus = 0.5;
    healer.incoming_healing_bonus = 9.0; // belongs to the healer, not recipient
    let spec = HealingSpec {
        ratio: 1.0,
        stat: "atk".into(),
        flat: 0.0,
        outgoing_bonus: 0.0,
        incoming_bonus: 0.2,
        rounding: Rounding::Floor,
    };
    let healed = evaluate_healing(&healer, 10, 1000, &spec).unwrap();
    assert_eq!(healed.effective_healing, 180); // 100 * 3/2 * 6/5
    assert!(matches!(
        evaluate_healing(&healer, 0, 1000, &spec),
        Err(CombatError::DeadRecipient)
    ));
    let mut input = serde_json::to_value(Request {
        schema_version: 1,
        content_id: "ntesim-current".into(),
        attacker_level: 80,
        attacker_stats: healer,
        target_defense: 0.0,
        target_resistance: 0.0,
        target_vulnerability: 0.0,
        buffs: vec![],
        direct_hits: vec![],
        healings: vec![],
        healing_target: None,
        shields: vec![],
        negative_resistance_model: NegativeResistanceModel::StandardRational,
        rounding: Rounding::Floor,
    })
    .unwrap();
    input["healings"] = serde_json::json!([spec.clone(), spec]);
    input["healing_target"] = serde_json::json!({"current_hp": 10, "max_hp": 300});
    let result = evaluate_request(&serde_json::from_value(input).unwrap()).unwrap();
    assert_eq!(result["healings"][0]["effective_healing"], 180);
    assert_eq!(result["healings"][1]["effective_healing"], 110);
    assert_eq!(result["healings"][1]["overheal"], 70);
    assert_eq!(result["healing_target_final"]["current_hp"], 300);
}

#[test]
fn shared_integer_mitigation_and_nonfinite_outputs_do_not_become_zero_or_saturated_damage() {
    assert_eq!(
        mitigate_integer_damage(10000, DamageKind::Elemental, 900, 80).unwrap(),
        5455
    );
    assert_eq!(
        mitigate_integer_damage(10000, DamageKind::Mental, 900, 80).unwrap(),
        10000
    );
    let stats = DynamicStats::new(f64::MAX, 100.0, 0.0);
    let spec = ShieldSpec {
        ratio: 2.0,
        stat: "atk".into(),
        flat: 0.0,
        shield_bonus: 0.0,
        rounding: Rounding::Floor,
    };
    assert!(evaluate_shield(&stats, &spec).is_err());
}

#[test]
fn dynamic_stat_modifier_overflow_is_rejected_without_changing_the_stat() {
    let mut stats = DynamicStats::new(1.0, 1.0, 1.0);
    stats.apply_modifier("atk_percent", f64::MAX).unwrap();
    assert!(matches!(
        stats.apply_modifier("atk_percent", f64::MAX),
        Err(CombatError::NonFiniteParameter(_))
    ));
    assert_eq!(stats.atk_percent, f64::MAX);
    assert!(stats.total_atk().is_finite());

    stats.apply_modifier("energy_efficiency", f64::MAX).unwrap();
    assert!(matches!(
        stats.apply_modifier("charge_efficiency", f64::MAX),
        Err(CombatError::NonFiniteParameter(_))
    ));
    assert_eq!(stats.energy_efficiency, f64::MAX);

    stats.apply_modifier("Chaos", f64::MAX).unwrap();
    assert!(matches!(
        stats.apply_modifier("elemental:Chaos", f64::MAX),
        Err(CombatError::NonFiniteParameter(_))
    ));
    assert_eq!(stats.elemental_bonuses.get("Chaos"), Some(&f64::MAX));

    stats.apply_modifier("skill", f64::MAX).unwrap();
    assert!(matches!(
        stats.apply_modifier("action:skill", f64::MAX),
        Err(CombatError::NonFiniteParameter(_))
    ));
    assert_eq!(stats.action_type_bonuses.get("skill"), Some(&f64::MAX));
}

#[test]
fn explicit_damage_base_does_not_hide_a_nonfinite_scaling_stat() {
    let mut stats = DynamicStats::new(f64::MAX, 100.0, 800.0);
    stats.apply_modifier("atk_percent", 1.0).unwrap();
    assert!(stats.get_stat("atk").unwrap().is_infinite());

    let hit = DirectHitSpec {
        base: Some(1.0),
        ratio: None,
        stat: "atk".into(),
        flat: 0.0,
        element: "True".into(),
        kind: DamageKind::True,
        crit_rule: CritRule::None,
        action_type: None,
        damage_bonus: 0.0,
        crit_damage_bonus: 0.0,
        final_multiplier: 1.0,
        special_multiplier: 1.0,
    };
    let request = DirectDamageEvaluationRequest {
        attacker_level: 80,
        attacker_stats: &stats,
        target_defense: 0.0,
        target_resistance: 0.0,
        target_vulnerability: 0.0,
        hit: &hit,
        crit_mode: EvaluationCritMode::Expected,
        rounding: Rounding::Floor,
        negative_res_model: NegativeResistanceModel::StandardRational,
    };

    assert!(matches!(
        evaluate_direct_damage(&request),
        Err(CombatError::NonFiniteParameter(stat)) if stat == "atk"
    ));
}

#[test]
fn static_buff_projection_honors_caps_replacements_and_application_snapshot() {
    let initial = DynamicStats::new(100.0, 1000.0, 0.0);
    let buff = |id: &str, amount, stacking| Buff {
        id: id.into(),
        stat: "atk_flat".into(),
        amount: Some(amount),
        amount_from: None,
        stacking,
        duration_us: 10,
    };
    let (stats, count) = modifiers::project_active_buffs(
        &initial,
        &[
            buff("source-a", 10.0, StackingPolicy::ReplaceBySource),
            buff("source-a", 20.0, StackingPolicy::ReplaceBySource),
            buff(
                "source-b",
                30.0,
                StackingPolicy::AddStacks { max_stacks: 2 },
            ),
            buff(
                "source-b",
                30.0,
                StackingPolicy::AddStacks { max_stacks: 2 },
            ),
            buff(
                "source-b",
                30.0,
                StackingPolicy::AddStacks { max_stacks: 2 },
            ),
        ],
    )
    .unwrap();
    assert_eq!((stats.total_atk(), count), (180.0, 3));
    let (stats, count) = modifiers::project_active_buffs(
        &initial,
        &[
            buff("source-a", 10.0, StackingPolicy::ReplaceBySource),
            buff("source-b", 50.0, StackingPolicy::ReplaceByTarget),
        ],
    )
    .unwrap();
    assert_eq!((stats.total_atk(), count), (150.0, 1));
    let mut scaled = Buff {
        id: "new-snapshot".into(),
        stat: "atk_flat".into(),
        amount: None,
        amount_from: Some(DynamicBuffAmountRule {
            stat: "atk".into(),
            coefficient: 0.5,
            maximum: None,
            snapshot: SnapshotPolicy::AtApplication,
        }),
        stacking: StackingPolicy::ReplaceBySource,
        duration_us: 10,
    };
    let (stats, _) = modifiers::project_active_buffs(
        &initial,
        &[
            buff("source-a", 100.0, StackingPolicy::ReplaceBySource),
            scaled.clone(),
        ],
    )
    .unwrap();
    assert_eq!(stats.total_atk(), 300.0); // (100 base +100 first) +0.5*200
    scaled.amount_from.as_mut().unwrap().snapshot = SnapshotPolicy::Dynamic;
    assert!(modifiers::project_active_buffs(&initial, &[scaled]).is_err());
}
