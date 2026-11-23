use super::*;

fn fixture(tier: u32, extra_elements: &[&str]) -> GameScenarioRequest {
    let ratios = [
        Ratio {
            numerator: 1,
            denominator: 10,
        },
        Ratio {
            numerator: 3,
            denominator: 25,
        },
        Ratio {
            numerator: 7,
            denominator: 50,
        },
        Ratio {
            numerator: 4,
            denominator: 25,
        },
        Ratio {
            numerator: 9,
            denominator: 50,
        },
    ];
    let bonus = ratios[(tier as usize - 1).min(4)];
    let mut req = research_request();
    req.stochastic_crit = None;
    req.actions.truncate(1);
    req.action_metadata.retain(|index, _| *index == 0);
    req.actors[0].character_id = "character:1033".into();
    req.actors[0].weapon_id = "weapon:fork_koinobori".into();
    req.actors[0].weapon_tier = tier;
    req.actors[0].gear_procs = vec![GearProcConfig::KoinoboriStats {
        atk_bonus: bonus,
        def_bonus: bonus,
        hp_bonus: bonus,
    }];
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def = 1000;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actors[0].static_stats.crit_damage_bonus = Ratio::zero();
    req.actors[0].static_stats.damage_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();

    for (slot_offset, element) in extra_elements.iter().enumerate() {
        let (cid, _name) = match *element {
            "COSMOS" => ("character:1052", "Hotori"),
            "NATURE" => ("character:1055", "Jiuyuan"),
            "PSYCHE" => ("character:1020", "Haniel"),
            "LAKSHANA" => ("character:1025", "Hathor"),
            "CHAOS" => ("character:1054", "Daffodil"),
            "INCANTATION" => ("character:1003", "Sakiri"),
            _ => panic!("unsupported test element: {element}"),
        };
        let mut ally = req.actors[0].clone();
        ally.slot = slot_offset + 1;
        ally.character_id = cid.into();
        ally.weapon_id = "weapon:fork_Arachne".into();
        ally.gear_procs.clear();
        req.actors.push(ally);
    }
    req
}

#[test]
fn koinobori_stats_apply_when_three_or_more_distinct_elements_present() {
    let req = fixture(1, &["COSMOS", "NATURE"]);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let hit = report["hit_receipts"][0]["rounded_damage"]
        .as_u64()
        .unwrap();
    assert_eq!(hit, 1100);
    assert_eq!(
        report["resolved_model_assumptions"][crate::weapons::koinobori::HYPOTHESIS_ID.as_str()],
        crate::weapons::koinobori::HYPOTHESIS_ID.description()
    );

    let mut baseline = req.clone();
    baseline.actors[0].gear_procs.clear();
    let base_hit = run_game_scenario(&baseline, &ExecutionLimits::default()).unwrap()["hit_receipts"]
        [0]["rounded_damage"]
        .as_u64()
        .unwrap();
    assert_eq!(base_hit, 1000);

    let req_four = fixture(1, &["COSMOS", "NATURE", "PSYCHE"]);
    let report_four = run_game_scenario(&req_four, &ExecutionLimits::default()).unwrap();
    let hit_four = report_four["hit_receipts"][0]["rounded_damage"]
        .as_u64()
        .unwrap();
    assert_eq!(hit_four, 1100);
}

#[test]
fn koinobori_stats_do_not_apply_when_fewer_than_three_distinct_elements() {
    let req_solo = fixture(1, &[]);
    let report_solo = run_game_scenario(&req_solo, &ExecutionLimits::default()).unwrap();
    let hit_solo = report_solo["hit_receipts"][0]["rounded_damage"]
        .as_u64()
        .unwrap();
    assert_eq!(hit_solo, 1000);

    let req_same_elem = fixture(1, &["INCANTATION"]);
    let report_same = run_game_scenario(&req_same_elem, &ExecutionLimits::default()).unwrap();
    let hit_same = report_same["hit_receipts"][0]["rounded_damage"]
        .as_u64()
        .unwrap();
    assert_eq!(hit_same, 1000);

    let req_two_elem = fixture(1, &["COSMOS"]);
    let report_two = run_game_scenario(&req_two_elem, &ExecutionLimits::default()).unwrap();
    let hit_two = report_two["hit_receipts"][0]["rounded_damage"]
        .as_u64()
        .unwrap();
    assert_eq!(hit_two, 1000);

    let req_two_distinct_three_allies = fixture(1, &["INCANTATION", "COSMOS"]);
    let report_two_three =
        run_game_scenario(&req_two_distinct_three_allies, &ExecutionLimits::default()).unwrap();
    let hit_two_three = report_two_three["hit_receipts"][0]["rounded_damage"]
        .as_u64()
        .unwrap();
    assert_eq!(hit_two_three, 1000);
}

#[test]
fn koinobori_def_scaling_applies_when_condition_met() {
    let mut req = fixture(1, &["COSMOS", "NATURE"]);
    if let GameActionCommand::InvokeAbility { scaling_stat, .. } = &mut req.actions[0] {
        *scaling_stat = crate::simulation::NativeScalingStat::Def;
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let hit = report["hit_receipts"][0]["rounded_damage"]
        .as_u64()
        .unwrap();
    assert_eq!(hit, 1100);

    let mut req_fewer = fixture(1, &["COSMOS"]);
    if let GameActionCommand::InvokeAbility { scaling_stat, .. } = &mut req_fewer.actions[0] {
        *scaling_stat = crate::simulation::NativeScalingStat::Def;
    }
    let report_fewer = run_game_scenario(&req_fewer, &ExecutionLimits::default()).unwrap();
    let hit_fewer = report_fewer["hit_receipts"][0]["rounded_damage"]
        .as_u64()
        .unwrap();
    assert_eq!(hit_fewer, 1000);
}

#[test]
fn koinobori_scales_exact_ratios_across_all_five_tiers() {
    let expected_damages = [1100, 1120, 1140, 1160, 1180];
    for (tier, expected) in (1..=5).zip(expected_damages) {
        let req = fixture(tier, &["COSMOS", "NATURE"]);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        let hit = report["hit_receipts"][0]["rounded_damage"]
            .as_u64()
            .unwrap();
        assert_eq!(hit, expected);
    }
}

#[test]
fn koinobori_hp_bonus_adds_base_scaled_in_build_domain() {
    // Hand-derived: add_base_hp_ratio(derived, native_base, ratio)
    //   = derived + native_base * ratio (compiled build HP-percent domain).
    // T1 (1/10): 1000 + 800 * 1/10 = 1080. T5 (9/50): 1000 + 800 * 9/50 = 1144.
    let t1 = crate::weapons::koinobori::projection::add_base_hp_ratio(
        crate::simulation::ExactFraction::new(1000, 1).unwrap(),
        crate::simulation::ExactFraction::new(800, 1).unwrap(),
        crate::simulation::ExactFraction::new(1, 10).unwrap(),
    )
    .unwrap();
    assert_eq!(t1, crate::simulation::ExactFraction::new(1080, 1).unwrap());
    let t5 = crate::weapons::koinobori::projection::add_base_hp_ratio(
        crate::simulation::ExactFraction::new(1000, 1).unwrap(),
        crate::simulation::ExactFraction::new(800, 1).unwrap(),
        crate::simulation::ExactFraction::new(9, 50).unwrap(),
    )
    .unwrap();
    assert_eq!(t5, crate::simulation::ExactFraction::new(1144, 1).unwrap());
    // The derived-scaled misreading derived * (1 + ratio) = 1000 * 11/10 = 1100 is rejected.
    assert_ne!(t1, crate::simulation::ExactFraction::new(1100, 1).unwrap());
}

#[test]
fn koinobori_rejects_mismatched_weapon_id_and_invalid_ratios() {
    let mut req = fixture(1, &["COSMOS", "NATURE"]);
    req.actors[0].weapon_id = "weapon:fork_Arachne".into();
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires equipped weapon:fork_koinobori")
    );

    let mut invalid = fixture(1, &["COSMOS", "NATURE"]);
    invalid.actors[0].gear_procs = vec![GearProcConfig::KoinoboriStats {
        atk_bonus: Ratio {
            numerator: 1,
            denominator: 0,
        },
        def_bonus: Ratio::one(),
        hp_bonus: Ratio::one(),
    }];
    assert!(run_game_scenario(&invalid, &ExecutionLimits::default()).is_err());
}
