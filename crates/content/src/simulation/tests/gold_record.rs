use super::*;
use crate::simulation::metadata::InputActionRole;

fn fixture(tier: u32) -> GameScenarioRequest {
    let atk_ratios = [
        Ratio {
            numerator: 6,
            denominator: 25,
        },
        Ratio {
            numerator: 3,
            denominator: 10,
        },
        Ratio {
            numerator: 9,
            denominator: 25,
        },
        Ratio {
            numerator: 21,
            denominator: 50,
        },
        Ratio {
            numerator: 12,
            denominator: 25,
        },
    ];
    let qte_ratios = [
        Ratio {
            numerator: 33,
            denominator: 50,
        },
        Ratio {
            numerator: 33,
            denominator: 40,
        },
        Ratio {
            numerator: 99,
            denominator: 100,
        },
        Ratio {
            numerator: 231,
            denominator: 200,
        },
        Ratio {
            numerator: 33,
            denominator: 25,
        },
    ];
    let ult_ratios = [
        Ratio {
            numerator: 11,
            denominator: 50,
        },
        Ratio {
            numerator: 11,
            denominator: 40,
        },
        Ratio {
            numerator: 33,
            denominator: 100,
        },
        Ratio {
            numerator: 77,
            denominator: 200,
        },
        Ratio {
            numerator: 11,
            denominator: 25,
        },
    ];
    let i = (tier as usize - 1).min(4);
    let mut req = research_request();
    req.stochastic_crit = None;
    req.relative_duration_us = 40_000_000;
    req.actors[0].character_id = "character:1072".into();
    req.actors[0].weapon_id = "weapon:fork_GoldRecord".into();
    req.actors[0].weapon_tier = tier;
    req.actors[0].gear_procs = vec![GearProcConfig::GoldRecord {
        atk_bonus: atk_ratios[i],
        qte_crit_damage_bonus: qte_ratios[i],
        ult_crit_damage_per_stack: ult_ratios[i],
        duration_us: 3_000_000,
    }];
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actors[0].static_stats.crit_damage_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    }; // 50% base Crit DMG
    req.actors[0].static_stats.damage_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    let template = req.action_metadata[&0].clone();
    req.actions.clear();
    req.action_metadata.clear();
    req.action_metadata.insert(0, template);
    req
}

fn append_action(req: &mut GameScenarioRequest, time: u64, role: InputActionRole, delay: u64) {
    let index = req.actions.len();
    req.actions.push(GameActionCommand::InvokeAbility {
        timestamp_us: time,
        actor_index: 0,
        ability_id: "caller-test".into(),
        damage_effect_id: "caller-effect".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: delay,
    });
    let mut metadata = req.action_metadata[&0].clone();
    metadata.ability_id = "caller-test".into();
    metadata.damage_effect_id = "caller-effect".into();
    metadata.input_action_role = Some(role);
    metadata.action_gameplay_tags = Some(vec!["Ability.Melee".into()]);
    req.action_metadata.insert(index, metadata);
}

fn enable_guaranteed_crit(req: &mut GameScenarioRequest) {
    req.actors[0].static_stats.crit_rate = Ratio::one();
    req.stochastic_crit = Some(crate::simulation::ResearchCritPolicy {
        roll_policy: crate::simulation::ResearchCritRollPolicy::IndependentPerTargetHitV1,
        hypothesis: "Test guaranteed crits for Voice of the Voyager verification.".into(),
        eligible_effects: std::collections::BTreeMap::from([(
            req.actors[0].character_id.clone(),
            std::collections::BTreeSet::from(["*".into()]),
        )]),
        actor_sequences: std::collections::BTreeMap::from([(
            req.actors[0].character_id.clone(),
            1,
        )]),
    });
}

#[test]
fn gold_record_passive_atk_applies_unconditionally() {
    let mut req = fixture(1);
    append_action(&mut req, 0, InputActionRole::Normal, 0);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let hit = report["hit_receipts"][0]["rounded_damage"]
        .as_u64()
        .unwrap();
    // 1000 base_atk * (1 + 24% atk_percent) = 1240
    assert_eq!(hit, 1240);
    assert_eq!(
        report["resolved_model_assumptions"][crate::weapons::gold_record::HYPOTHESIS_ID.as_str()],
        crate::weapons::gold_record::HYPOTHESIS_ID.description()
    );

    let mut baseline = req.clone();
    baseline.actors[0].gear_procs.clear();
    let base_hit = run_game_scenario(&baseline, &ExecutionLimits::default()).unwrap()["hit_receipts"]
        [0]["rounded_damage"]
        .as_u64()
        .unwrap();
    assert_eq!(base_hit, 1000);
}

#[test]
fn gold_record_qte_crit_damage_bonus_applies_on_qte_crit() {
    let mut req = fixture(1);
    enable_guaranteed_crit(&mut req);
    // QTE attack
    append_action(&mut req, 0, InputActionRole::Qte, 0);
    // Normal attack
    append_action(&mut req, 1_000_000, InputActionRole::Normal, 0);

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let qte_hit = report["hit_receipts"][0]["rounded_damage"]
        .as_u64()
        .unwrap();
    let normal_hit = report["hit_receipts"][1]["rounded_damage"]
        .as_u64()
        .unwrap();

    // Normal crit: 1240 ATK * (1 + 50% base crit dmg) = 1860
    assert_eq!(normal_hit, 1860);
    // QTE crit: 1240 ATK * (1 + 50% base crit dmg + 66% qte crit dmg) = 1240 * 2.16 = 2678.4 -> 2678
    assert_eq!(qte_hit, 2678);
    // Hand-derived: 1240 * (1 + 1/2 + 33/50) = 1240 * 54/25 = 2678.4 -> 2678 (nearest integer).
    // The multiplicative misreading 1240 * 3/2 * 83/50 = 3087.6 -> 3088 is rejected.
    assert_ne!(qte_hit, 3088);
}

#[test]
fn gold_record_qte_hit_grants_stacks_and_benefits_ultimate() {
    let mut req = fixture(1);
    enable_guaranteed_crit(&mut req);

    // 1. QTE at t=0, hits at t=100_000 -> grants 1 stack (+22% ult crit dmg)
    append_action(&mut req, 0, InputActionRole::Qte, 100_000);
    // 2. QTE at t=500_000, hits at t=600_000 -> grants 2nd stack (+44% ult crit dmg)
    append_action(&mut req, 500_000, InputActionRole::Qte, 100_000);
    // 3. Ultimate at t=1_000_000, hits at t=1_100_000 (active stacks = 2)
    append_action(&mut req, 1_000_000, InputActionRole::Ultimate, 100_000);

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let ult_hit = report["hit_receipts"][2]["rounded_damage"]
        .as_u64()
        .unwrap();
    // 2 stacks = +44% crit dmg
    // Total crit multiplier = 1 + 50% base + 44% stacks = 1.94
    // 1240 ATK * 1.94 = 2405.6 -> rounded is 2406
    assert_eq!(ult_hit, 2406);
}

#[test]
fn gold_record_stacks_cap_at_three_and_expire_after_duration() {
    let mut req = fixture(1);
    enable_guaranteed_crit(&mut req);

    // 4 QTE hits: t=0, 200_000, 400_000, 600_000
    append_action(&mut req, 0, InputActionRole::Qte, 0);
    append_action(&mut req, 200_000, InputActionRole::Qte, 0);
    append_action(&mut req, 400_000, InputActionRole::Qte, 0);
    append_action(&mut req, 600_000, InputActionRole::Qte, 0); // 4th hit: capped at 3 stacks, resets expiry to 600_000 + 3_000_000 = 3_600_000

    // Ultimate at t=1_000_000 (within 3s window): has 3 stacks (+66% ult crit dmg)
    append_action(&mut req, 1_000_000, InputActionRole::Ultimate, 0);
    // Ultimate at t=4_000_000 (after 3.6s expiry): 0 stacks (+0% ult crit dmg)
    append_action(
        &mut req,
        4_000_000 + 4_237_614,
        InputActionRole::Ultimate,
        0,
    );

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let ult_inside = report["hit_receipts"][4]["rounded_damage"]
        .as_u64()
        .unwrap();
    let ult_expired = report["hit_receipts"][5]["rounded_damage"]
        .as_u64()
        .unwrap();

    // Inside window with 3 stacks: 1 + 0.50 + 0.66 = 2.16 -> 1240 * 2.16 = 2678.4 -> 2678
    assert_eq!(ult_inside, 2678);
    // After expiry: 1 + 0.50 = 1.50 -> 1240 * 1.50 = 1860
    assert_eq!(ult_expired, 1860);
}

#[test]
fn gold_record_refinement_scaling_progresses_from_t1_to_t5() {
    let expected_atk_hits = [1240, 1300, 1360, 1420, 1480];
    for tier in 1..=5 {
        let mut req = fixture(tier);
        append_action(&mut req, 0, InputActionRole::Normal, 0);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        let hit = report["hit_receipts"][0]["rounded_damage"]
            .as_u64()
            .unwrap();
        assert_eq!(hit, expected_atk_hits[tier as usize - 1]);
    }
}

#[test]
fn gold_record_zone_math_hand_derived_t1_t5() {
    fn gcd(mut a: u128, mut b: u128) -> u128 {
        while b != 0 {
            let rest = a % b;
            a = b;
            b = rest;
        }
        a
    }
    fn reduced(n: u128, d: u128) -> (u128, u128) {
        let g = gcd(n, d);
        (n / g, d / g)
    }
    // T1..T5 ATK_live = 1000 * (1 + p), p in {6/25, 3/10, 9/25, 21/50, 12/25}.
    let atk: Vec<u128> = [(6u128, 25u128), (3, 10), (9, 25), (21, 50), (12, 25)]
        .iter()
        .map(|(n, d)| 1000 * (d + n) / d)
        .collect();
    assert_eq!(atk, [1240, 1300, 1360, 1420, 1480]);
    // T1 QTE crit: C = 1 + 1/2 + 33/50 = 54/25; D = 1240 * 54/25 = 2678.4 -> 2678.
    assert_eq!(reduced(54, 25), (54, 25));
    assert_eq!(reduced(1240 * 54, 25), (13392, 5));
    // Multiplicative misreading 1240 * 3/2 * 83/50 = 3087.6 -> 3088 is rejected.
    assert_ne!(reduced(1240 * 54, 25), reduced(1240 * 3 * 83, 100));
    // T5 3-stack ult: C = 1 + 1/2 + 33/25 = 141/50; D = 1480 * 141/50 = 4173.6 -> 4174.
    assert_eq!(reduced(141, 50), (141, 50));
    assert_eq!(reduced(1480 * 141, 50), (20868, 5));
    // Multiplicative misreading 1480 * 3/2 * 58/25 = 5150.4 -> 5150 is rejected.
    assert_ne!(reduced(1480 * 141, 50), reduced(1480 * 3 * 58, 50));
}

#[test]
fn gold_record_invalid_configs_reject() {
    let mut req = fixture(1);
    append_action(&mut req, 0, InputActionRole::Normal, 0);
    req.actors[0].weapon_id = "weapon:fork_bopu".into();
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires equipped weapon:fork_GoldRecord")
    );

    let mut req_zero_duration = fixture(1);
    append_action(&mut req_zero_duration, 0, InputActionRole::Normal, 0);
    req_zero_duration.actors[0].gear_procs = vec![GearProcConfig::GoldRecord {
        atk_bonus: Ratio {
            numerator: 6,
            denominator: 25,
        },
        qte_crit_damage_bonus: Ratio {
            numerator: 33,
            denominator: 50,
        },
        ult_crit_damage_per_stack: Ratio {
            numerator: 11,
            denominator: 50,
        },
        duration_us: 0,
    }];
    assert!(
        run_game_scenario(&req_zero_duration, &ExecutionLimits::default())
            .unwrap_err()
            .contains("positive duration")
    );
}
