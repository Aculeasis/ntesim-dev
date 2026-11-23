use super::*;
use crate::simulation::metadata::InputActionRole;

fn fixture(tier: u32) -> GameScenarioRequest {
    let atk_per_stack = [
        Ratio {
            numerator: 1,
            denominator: 100,
        },
        Ratio {
            numerator: 23,
            denominator: 2000,
        },
        Ratio {
            numerator: 13,
            denominator: 1000,
        },
        Ratio {
            numerator: 29,
            denominator: 2000,
        },
        Ratio {
            numerator: 2,
            denominator: 125,
        },
    ];
    let atk_max_stack = [
        Ratio {
            numerator: 1,
            denominator: 50,
        },
        Ratio {
            numerator: 23,
            denominator: 1000,
        },
        Ratio {
            numerator: 13,
            denominator: 500,
        },
        Ratio {
            numerator: 29,
            denominator: 1000,
        },
        Ratio {
            numerator: 4,
            denominator: 125,
        },
    ];
    let i = (tier as usize - 1).min(4);
    let mut req = research_request();
    req.stochastic_crit = None;
    req.relative_duration_us = 40_000_000;
    req.actors[0].character_id = "character:1003".into();
    req.actors[0].weapon_id = "weapon:fork_spider".into();
    req.actors[0].weapon_tier = tier;
    req.actors[0].gear_procs = vec![GearProcConfig::SpiderTeamAtk {
        atk_bonus_per_stack: atk_per_stack[i],
        atk_bonus_max_stack: atk_max_stack[i],
        duration_us: 10_000_000,
    }];
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actors[0].static_stats.crit_damage_bonus = Ratio::zero();
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

fn append_action(
    req: &mut GameScenarioRequest,
    actor_idx: usize,
    time: u64,
    role: InputActionRole,
    delay: u64,
) {
    let index = req.actions.len();
    req.actions.push(GameActionCommand::InvokeAbility {
        timestamp_us: time,
        actor_index: actor_idx,
        ability_id: format!("caller-test-{index}"),
        array_index: 0,
        damage_effect_id: format!("caller-effect-{index}"),
        atk_rate: Ratio {
            numerator: 1,
            denominator: 1,
        },
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: delay,
    });
    let mut metadata = req.action_metadata[&0].clone();
    metadata.ability_id = format!("caller-test-{index}");
    metadata.damage_effect_id = format!("caller-effect-{index}");
    metadata.input_action_role = Some(role);
    metadata.action_gameplay_tags = Some(vec!["Ability.Melee".into()]);
    req.action_metadata.insert(index, metadata);
}

#[test]
fn spider_stacks_knowledge_on_normal_attack_and_respects_half_second_icd() {
    let mut req = fixture(1);

    // 1. Normal attack at t=0, hits at t=0 -> gains 1st stack
    append_action(&mut req, 0, 0, InputActionRole::Normal, 0);
    // 2. Normal attack at t=200_000, hits at t=200_000 (< 500_000 ICD) -> ignored by ICD
    append_action(&mut req, 0, 200_000, InputActionRole::Normal, 0);
    // 3. Normal attack at t=600_000, hits at t=600_000 (>= 500_000 ICD) -> gains 2nd stack
    append_action(&mut req, 0, 600_000, InputActionRole::Normal, 0);

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let stack_events: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "spider_knowledge_stack")
        .collect();

    assert_eq!(stack_events.len(), 2);
    assert_eq!(stack_events[0]["time_us"], 0);
    assert_eq!(stack_events[1]["time_us"], 600_000);
}

#[test]
fn spider_consumes_stacks_on_ultimate_and_buffs_team_atk() {
    let mut req = fixture(1);

    // 5 Normal attacks spaced by 600ms -> 5 stacks
    for i in 0..5 {
        append_action(&mut req, 0, i * 600_000, InputActionRole::Normal, 0);
    }

    // Ultimate at t=3_500_000 -> consumes 5 stacks -> grants +5% team ATK for 10s (until 13.5s)
    append_action(&mut req, 0, 3_500_000, InputActionRole::Ultimate, 0);

    // Normal attack at t=4_000_000 (inside 10s window)
    append_action(&mut req, 0, 4_000_000, InputActionRole::Normal, 0);

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let hits = report["hit_receipts"].as_array().unwrap();

    // First 5 hits: base 1000 ATK
    for hit in &hits[0..5] {
        assert_eq!(hit["rounded_damage"], 1000);
    }
    // Ultimate hit: benefits from the team ATK buff cast before hits!
    // 1000 base_atk * (1 + 5% team ATK) = 1050
    assert_eq!(hits[5]["rounded_damage"], 1050);
    // Subsequent hit at 4s: also benefits
    assert_eq!(hits[6]["rounded_damage"], 1050);

    let team_atk_events: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "spider_team_atk")
        .collect();
    assert_eq!(team_atk_events.len(), 1);
    assert_eq!(team_atk_events[0]["time_us"], 3_500_000);
}

#[test]
fn spider_max_eight_stacks_activates_additional_bonus() {
    let mut req = fixture(1);

    // 8 Normal attacks spaced by 600ms -> 8 stacks
    for i in 0..8 {
        append_action(&mut req, 0, i * 600_000, InputActionRole::Normal, 0);
    }
    // 9th Normal attack at t=5_000_000 -> capped at 8 stacks
    append_action(&mut req, 0, 5_000_000, InputActionRole::Normal, 0);

    // Ultimate at t=6_000_000 -> consumes 8 stacks -> 8 * 1% + 2% bonus = 10% team ATK for 10s (until 16.0s)
    append_action(&mut req, 0, 6_000_000, InputActionRole::Ultimate, 0);

    // Normal attack at t=7_000_000 (inside window): 10% team ATK bonus -> 1100
    append_action(&mut req, 0, 7_000_000, InputActionRole::Normal, 0);

    // Normal attack at t=18_000_000 (after 10s expiry at 16s): 0% bonus -> 1000
    append_action(&mut req, 0, 18_000_000, InputActionRole::Normal, 0);

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let hits = report["hit_receipts"].as_array().unwrap();

    // Ultimate hit at t=6s: 1000 * 1.10 = 1100
    assert_eq!(hits[9]["rounded_damage"], 1100);
    // Inside window hit at t=7s: 1100
    assert_eq!(hits[10]["rounded_damage"], 1100);
    // After expiry hit at t=18s: 1000
    assert_eq!(hits[11]["rounded_damage"], 1000);
}

#[test]
fn spider_refinement_progression_t1_to_t5() {
    let expected_damages = [1100, 1115, 1130, 1145, 1160];
    for tier in 1..=5 {
        let mut req = fixture(tier);
        // 8 Normal attacks spaced by 600ms -> 8 stacks
        for i in 0..8 {
            append_action(&mut req, 0, i * 600_000, InputActionRole::Normal, 0);
        }
        // Ultimate at t=5_000_000 -> consumes 8 stacks
        append_action(&mut req, 0, 5_000_000, InputActionRole::Ultimate, 0);

        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        let ult_hit = report["hit_receipts"][8]["rounded_damage"]
            .as_u64()
            .unwrap();
        assert_eq!(ult_hit, expected_damages[tier as usize - 1]);
    }
}

#[test]
fn spider_zero_stacks_ultimate_grants_no_buff() {
    let mut req = fixture(1);
    // Direct Ultimate at t=0 without prior normal attacks -> 0 stacks
    append_action(&mut req, 0, 0, InputActionRole::Ultimate, 0);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let ult_hit = report["hit_receipts"][0]["rounded_damage"]
        .as_u64()
        .unwrap();
    assert_eq!(ult_hit, 1000);

    let team_atk_events: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "spider_team_atk")
        .collect();
    assert_eq!(team_atk_events.len(), 0);
}

#[test]
fn spider_invalid_configs_reject() {
    let mut req = fixture(1);
    append_action(&mut req, 0, 0, InputActionRole::Normal, 0);
    req.actors[0].weapon_id = "weapon:fork_bopu".into();
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires equipped weapon:fork_spider")
    );

    let mut req_zero_duration = fixture(1);
    append_action(&mut req_zero_duration, 0, 0, InputActionRole::Normal, 0);
    req_zero_duration.actors[0].gear_procs = vec![GearProcConfig::SpiderTeamAtk {
        atk_bonus_per_stack: Ratio {
            numerator: 1,
            denominator: 100,
        },
        atk_bonus_max_stack: Ratio {
            numerator: 1,
            denominator: 50,
        },
        duration_us: 0,
    }];
    assert!(
        run_game_scenario(&req_zero_duration, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires a positive duration")
    );
}

#[test]
fn spider_team_atk_buff_applies_to_ally() {
    // Hand-derived oracle: 3 stacks at T1 -> team bonus 3 x 1/100 = 3/100.
    // Both wearer and ally hit at 1000 base ATK: 1000 x 103/100 = 1030 exact.
    // The ally must be switched in to act (inactive actors are rejected).
    let mut req = fixture(1);
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.weapon_id = "weapon:none".into();
    ally.gear_procs.clear();
    req.actors.push(ally);

    // Wearer builds 3 stacks with Normal attacks spaced by 600ms (>= 500_000 ICD).
    for i in 0..3 {
        append_action(&mut req, 0, i * 600_000, InputActionRole::Normal, 0);
    }
    // Wearer casts Ultimate at t=2_000_000 -> consumes 3 stacks -> team +3% ATK for 10s.
    append_action(&mut req, 0, 2_000_000, InputActionRole::Ultimate, 0);
    // Switch to the ally; ally Normal at t=3_000_000 (inside the buff window).
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 2_500_000,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    append_action(&mut req, 1, 3_000_000, InputActionRole::Normal, 0);
    // Switch back; wearer Normal after Support recovery (inside the buff window).
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 3_500_000,
        target_actor_index: 0,
        cooldown_us: 0,
    });
    append_action(&mut req, 0, 4_100_000, InputActionRole::Normal, 0);

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["ability_id"] != "GA_Sagiri_QTE")
        .collect();
    assert_eq!(hits.len(), 6);
    let ally_hit = hits.iter().find(|hit| hit["actor_index"] == 1).unwrap();
    assert_eq!(ally_hit["rounded_damage"], 1030);
    let wearer_post = hits.iter().rfind(|hit| hit["actor_index"] == 0).unwrap();
    assert_eq!(wearer_post["rounded_damage"], 1030);
}

#[test]
fn spider_eight_stack_bonus_hand_derived_t1_t5() {
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
    // Frozen buff_spider_AtkUp per-stack (T1..T5): 1/100, 23/2000, 13/1000, 29/2000, 2/125.
    // Frozen buff_spider_AtkUp2 8-stack extra: 1/50, 23/1000, 13/500, 29/1000, 4/125.
    // 8-stack total bonus = 8 * per_stack + max_stack.
    let totals: Vec<(u128, u128)> = [
        (1u128, 100u128, 1u128, 50u128),
        (23, 2000, 23, 1000),
        (13, 1000, 13, 500),
        (29, 2000, 29, 1000),
        (2, 125, 4, 125),
    ]
    .iter()
    .map(|(a, b, c, d)| reduced(8 * a * d + c * b, b * d))
    .collect();
    assert_eq!(totals, [(1, 10), (23, 200), (13, 100), (29, 200), (4, 25)]);
    // Cross-check against the scenario damages: 1000 * (1 + bonus) exact.
    for (bonus, damage) in [
        ((1u128, 10u128), 1100u128),
        ((23, 200), 1115),
        ((13, 100), 1130),
        ((29, 200), 1145),
        ((4, 25), 1160),
    ] {
        assert_eq!(1000 * (bonus.1 + bonus.0) / bonus.1, damage);
    }
    // No-cap misreading rejected: a 9th stack would add another per-stack bonus,
    // giving 9/100 + 1/50 = 11/100 at T1, not the capped 1/10.
    assert_ne!(reduced(9 + 2, 100), (1, 10));
}
