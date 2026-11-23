use super::*;

#[test]
fn mixed_packets_use_general_and_only_their_own_element_bonus() {
    let mut req = research_request();
    req.stochastic_crit = None;
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.actors[0].static_stats.damage_bonus = Ratio {
        numerator: 1,
        denominator: 10,
    };
    req.actors[0].static_stats.mental_damage_bonus = Ratio::one();
    let elements = [
        "CHAOS",
        "COSMOS",
        "INCANTATION",
        "LAKSHANA",
        "NATURE",
        "PSYCHE",
    ];
    req.actors[0].static_stats.elemental_bonuses = elements
        .iter()
        .enumerate()
        .map(|(i, name)| {
            (
                (*name).into(),
                Ratio {
                    numerator: i as u64 + 1,
                    denominator: 100,
                },
            )
        })
        .collect();
    for (index, element) in elements.iter().enumerate() {
        req.action_metadata.get_mut(&index).unwrap().damage_type_ex =
            Some(format!("EExecutionDamageTypeEX::DAMAGE_TYPE_{element}"));
        // All six packets were initiated before any landed; delivery is off-field
        // after the switch, so bonus identity must not depend on field ownership.
        if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut req.actions[index] {
            *hit_delay_us = 100;
        }
    }
    let mut other = req.actors[0].clone();
    other.slot = 1;
    other.character_id = "character:1019".into();
    req.actors.push(other);
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 60,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    for (index, hit) in report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        // Independent arithmetic: 1000 * (1 + 10% + (index+1)%).
        assert_eq!(
            hit["non_crit_damage"],
            serde_json::json!(format!("{}/{}", 1110 + index * 10, 1))
        );
        assert_eq!(
            hit["static_general_damage_bonus"],
            serde_json::json!("1/10")
        );
        assert_eq!(
            hit["static_element_damage_bonus"],
            serde_json::to_value(ExactFraction::new(index as u128 + 1, 100).unwrap()).unwrap()
        );
        assert_eq!(hit["static_mental_damage_bonus"], serde_json::json!("0/1"));
    }
    assert_eq!(
        report,
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap()
    );
}

#[test]
fn only_mental_uses_its_static_bonus_while_true_damage_bypasses_general_bonus() {
    for (kind, expected, mental) in [
        ("NORMAL", 1100, 0),
        ("PSYCHICALLY", 1350, 1),
        ("TRUE", 1000, 0),
    ] {
        let mut req = research_request();
        req.stochastic_crit = None;
        req.actions.truncate(1);
        req.action_metadata.retain(|index, _| *index == 0);
        req.model_assumptions.insert(
            crate::simulation::metadata::NORMAL_PHYSICAL_HYPOTHESIS_ID
                .as_str()
                .into(),
            crate::simulation::metadata::NORMAL_PHYSICAL_ID
                .description()
                .into(),
        );
        req.action_metadata.get_mut(&0).unwrap().damage_type_ex =
            Some(format!("EExecutionDamageTypeEX::DAMAGE_TYPE_{kind}"));
        req.actors[0].static_stats.damage_bonus = Ratio {
            numerator: 1,
            denominator: 10,
        };
        req.actors[0].static_stats.mental_damage_bonus = Ratio {
            numerator: 1,
            denominator: 4,
        };
        req.actors[0]
            .static_stats
            .elemental_bonuses
            .insert("NATURE".into(), Ratio::one());
        req.targets[0].defense = 0;
        req.targets[0].resistance = Ratio::zero();
        let mut other = req.actors[0].clone();
        other.slot = 1;
        other.character_id = "character:1003".into();
        other.static_stats.mental_damage_bonus = Ratio::one();
        req.actors.push(other);
        if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut req.actions[0] {
            *hit_delay_us = 100;
        }
        req.actions.push(GameActionCommand::SwitchActor {
            timestamp_us: 60,
            target_actor_index: 1,
            cooldown_us: 0,
        });
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            serde_json::json!(format!("{}/{}", expected, 1)),
            "{kind}"
        );
        assert_eq!(
            report["hit_receipts"][0]["static_mental_damage_bonus"],
            serde_json::json!(format!("{}/{}", mental, if mental == 1 { 4 } else { 1 }))
        );
    }
}

#[test]
fn target_resistance_overrides_follow_delivered_element_and_common_fallback_off_field() {
    let mut req = research_request();
    req.stochastic_crit = None;
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio {
        numerator: 1,
        denominator: 5,
    };
    req.targets[0].elemental_resistances = BTreeMap::from([
        (
            "NATURE".into(),
            Ratio {
                numerator: 1,
                denominator: 10,
            },
        ),
        (
            "CHAOS".into(),
            Ratio {
                numerator: 7,
                denominator: 10,
            },
        ),
    ]);
    for (index, element) in ["NATURE", "CHAOS", "COSMOS"].iter().enumerate() {
        req.action_metadata.get_mut(&index).unwrap().damage_type_ex =
            Some(format!("EExecutionDamageTypeEX::DAMAGE_TYPE_{element}"));
        if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut req.actions[index] {
            *hit_delay_us = 100;
        }
    }
    req.actions.truncate(3);
    req.action_metadata.retain(|index, _| *index < 3);
    let mut other = req.actors[0].clone();
    other.slot = 1;
    other.character_id = "character:1019".into();
    req.actors.push(other);
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 60,
        target_actor_index: 1,
        cooldown_us: 0,
    });

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|hit| hit["non_crit_damage"].clone())
            .collect::<Vec<_>>(),
        [
            serde_json::json!("900/1"),
            serde_json::json!("300/1"),
            serde_json::json!("800/1"),
        ]
    );
}

#[test]
fn unused_invalid_element_keys_and_ratios_fail_before_combat() {
    for (element, bonus) in [
        ("NATUER", Ratio::one()),
        (
            "PSYCHE",
            Ratio {
                numerator: 1,
                denominator: 0,
            },
        ),
    ] {
        let mut req = research_request();
        req.actors[0]
            .static_stats
            .elemental_bonuses
            .insert(element.into(), bonus);
        assert!(run_game_scenario(&req, &ExecutionLimits::default()).is_err());
    }
    let mut req = research_request();
    req.actors[0].static_stats.damage_bonus = Ratio {
        numerator: 0,
        denominator: 0,
    };
    assert!(run_game_scenario(&req, &ExecutionLimits::default()).is_err());
    let mut req = research_request();
    req.actors[0].static_stats.mental_damage_bonus = Ratio {
        numerator: 0,
        denominator: 0,
    };
    assert!(run_game_scenario(&req, &ExecutionLimits::default()).is_err());
}
