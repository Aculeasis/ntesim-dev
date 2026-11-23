use super::*;

#[test]
pub(crate) fn invalid_and_duplicated_gear_parameters_fail_before_execution() {
    let limits = ExecutionLimits::default();
    let mut mismatched_weapon = research_request();
    mismatched_weapon.actors[0].weapon_id = "weapon:other".into();
    mismatched_weapon.actors[0].gear_procs = vec![GearProcConfig::MindRoyaleEnergy {
        energy_grant: 1,
        cooldown_us: 1,
    }];
    assert!(
        run_game_scenario(&mismatched_weapon, &limits)
            .unwrap_err()
            .contains("Mind Royale proc requires equipped")
    );

    let mut mismatched_set = research_request();
    mismatched_set.actors[0].set_id = Some("set:other".into());
    mismatched_set.actors[0].gear_procs = vec![GearProcConfig::FirefliesCritDamage {
        crit_damage_per_stack: Ratio {
            numerator: 1,
            denominator: 2,
        },
        max_stacks: 1,
        duration_us: 1,
    }];
    assert!(
        run_game_scenario(&mismatched_set, &limits)
            .unwrap_err()
            .contains("Fireflies proc requires equipped")
    );

    for proc in [
        GearProcConfig::FirefliesCritDamage {
            crit_damage_per_stack: Ratio {
                numerator: 1,
                denominator: 0,
            },
            max_stacks: 1,
            duration_us: 1,
        },
        GearProcConfig::FirefliesCritDamage {
            crit_damage_per_stack: Ratio {
                numerator: 1,
                denominator: 2,
            },
            max_stacks: 0,
            duration_us: 1,
        },
        GearProcConfig::FirefliesCritDamage {
            crit_damage_per_stack: Ratio {
                numerator: 1,
                denominator: 2,
            },
            max_stacks: 65,
            duration_us: 1,
        },
        GearProcConfig::FirefliesCritDamage {
            crit_damage_per_stack: Ratio {
                numerator: 1,
                denominator: 2,
            },
            max_stacks: 1,
            duration_us: 0,
        },
        GearProcConfig::MindRoyaleEnergy {
            energy_grant: limits.max_pool_capacity + 1,
            cooldown_us: 1,
        },
        GearProcConfig::MindRoyaleEnergy {
            energy_grant: 1,
            cooldown_us: u64::MAX,
        },
    ] {
        let mut req = research_request();
        req.actors[0].gear_procs = vec![proc];
        assert!(run_game_scenario(&req, &limits).is_err());
    }
    let mut req = sample_s1_request();
    let duplicate = req.actors[0].gear_procs[0].clone();
    req.actors[0].gear_procs.push(duplicate);
    assert!(
        run_game_scenario(&req, &limits)
            .unwrap_err()
            .contains("duplicates")
    );
}
