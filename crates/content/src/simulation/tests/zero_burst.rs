use super::*;
use crate::simulation::tests::support::formula_damage_total;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

const BURST_PACKETS: [&str; 7] = [
    "GE_Player_Female046_UltraSkill1_Damage",
    "GE_Player_Female046_UltraSkill2_Damage",
    "GE_Player_Female046_UltraSkill2_Damage",
    "GE_Player_Female046_UltraSkill2_Damage",
    "GE_Player_Female046_UltraSkill2_Damage",
    "GE_Player_Female046_UltraSkill3_Damage",
    "GE_Player_Female046_UltraSkill4_Damage",
];

fn burst_request(level: u32, awakening: u32) -> GameScenarioRequest {
    let mut req = sample_s1_request();
    let actor = &mut req.actors[0];
    actor.character_id = "character:1046".into();
    actor.weapon_id.clear();
    actor.gear_procs.clear();
    actor.set_id = None;
    actor.awakening = ntesim_data::AwakeningSelection::prefix(awakening);
    actor.talent_levels = Some([1, 1, level]);
    actor.character_max_level = Some(80);
    actor.character_level = 80;
    actor.initial_energy = 100;
    actor.energy_capacity = 100;
    actor.static_stats.base_atk = 1000;
    actor.static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    actor.static_stats.crit_rate = Ratio::zero();
    actor.static_stats.crit_damage_bonus = Ratio::zero();
    actor.static_stats.damage_bonus = Ratio::zero();
    actor.static_stats.elemental_bonus = Ratio::zero();
    actor.static_stats.elemental_bonuses.clear();
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.targets[0].level = 80;
    req.actions = vec![GameActionCommand::InvokeAbility {
        timestamp_us: 0,
        actor_index: 0,
        ability_id: "GA_Female046_UltraSkill".into(),
        array_index: 0,
        damage_effect_id: BURST_PACKETS[0].into(),
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 100,
        recovery_us: 0,
        cooldown_us: 20_000_000,
        hit_delay_us: 100,
    }];
    req
}

fn frozen_row(_table: &str, row: &str) -> Value {
    ntesim_data::native_damage::lookup(ntesim_data::skill_levels::frozen::tables(), row)
        .unwrap()
        .1
        .clone()
}

fn fraction_value(value: &Value) -> f64 {
    serde_json::from_value::<ExactFraction>(value.clone())
        .unwrap()
        .to_f64()
}

#[test]
fn zero_burst_all_in_range_hits_every_declared_dummy() {
    let expected_milli = [1327, 53, 53, 53, 53, 1775, 1686];
    let mut req = burst_request(1, 0);
    let mut other = req.targets[0].clone();
    other.target_index = 1;
    other.defense = 1080;
    other.resistance = Ratio {
        numerator: 1,
        denominator: 5,
    };
    req.targets.push(other);

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 14);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
    assert_eq!(report["hit_receipts"].as_array().unwrap().len(), 14);
    assert_eq!(
        report["resolved_model_assumptions"]
            [crate::characters::zero::skill::BURST_ALL_IN_RANGE_HYPOTHESIS_ID.as_str()],
        crate::characters::zero::skill::BURST_ALL_IN_RANGE_HYPOTHESIS_ID.description()
    );

    let receipts = report["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts
            .iter()
            .map(|hit| hit["target_index"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [vec![0; 7], vec![1; 7]].concat(),
        "Burst receipts use ascending target-major order"
    );
    for target_index in 0..2 {
        let target_hits: Vec<_> = receipts
            .iter()
            .filter(|hit| hit["target_index"] == target_index)
            .collect();
        assert_eq!(target_hits.len(), 7, "target {target_index}");
        for (packet_index, hit) in target_hits.iter().enumerate() {
            assert_eq!(hit["damage_effect_id"], BURST_PACKETS[packet_index]);
            assert_eq!(hit["time_us"], 100);
            assert_eq!(hit["command_index"], 0);
            let mitigation = if target_index == 0 { 1.0 } else { 0.4 };
            assert!(
                (fraction_value(&hit["non_crit_damage"])
                    - expected_milli[packet_index] as f64 * mitigation)
                    .abs()
                    < 1e-9,
                "target {target_index}, packet {packet_index}: {}",
                hit["non_crit_damage"]
            );
        }
    }
    assert_eq!(
        fraction_value(&serde_json::json!(formula_damage_total(&report))),
        7000.0
    );
}

fn rate_milli(row: &Value, level_index: usize) -> u64 {
    (row["AtkRateBaseArray"][level_index].as_f64().unwrap() * 1000.0).round() as u64
}

fn source_coefficients(level_index: usize) -> [u64; 4] {
    [
        rate_milli(
            &frozen_row("DT_SkillDamageData", BURST_PACKETS[0]),
            level_index,
        ),
        rate_milli(
            &frozen_row("DT_SkillDamageData", BURST_PACKETS[1]),
            level_index,
        ),
        rate_milli(
            &frozen_row("DT_SkillDamageData", BURST_PACKETS[5]),
            level_index,
        ),
        rate_milli(
            &frozen_row("DT_SkillDamageData", BURST_PACKETS[6]),
            level_index,
        ),
    ]
}

#[test]
fn zero_burst_native_ratios_preserve_seven_packet_multiplicity() {
    // Cache-only native tooltip lists UltraSkill1/2/3/4 and multiplies the
    // second ratio by four: initial + four flurries + heavy cut + finisher.
    // Execution and independent damage checks below preserve that causal train.
    for (level_index, expected, total) in [
        (0, [1327, 53, 1775, 1686], 5000),
        (7, [2274, 91, 3042, 2890], 8570),
        (8, [2456, 98, 3285, 3121], 9254),
    ] {
        assert_eq!(source_coefficients(level_index), expected);
        assert_eq!(
            expected[0] + 4 * expected[1] + expected[2] + expected[3],
            total
        );
    }
}

#[test]
fn zero_burst_train_delivers_each_native_ratio_once_per_declared_multiplicity() {
    for (level, awakening, index, expected_milli, expected_total) in [
        (1, 0, 0, [1327, 53, 53, 53, 53, 1775, 1686], 5000),
        (8, 0, 7, [2274, 91, 91, 91, 91, 3042, 2890], 8570),
        (8, 3, 8, [2456, 98, 98, 98, 98, 3285, 3121], 9254),
    ] {
        let req = burst_request(level, awakening);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 7);
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
        assert_eq!(
            report["resolved_model_assumptions"]
                [crate::characters::zero::skill::BURST_PACKETS_HYPOTHESIS_ID.as_str()],
            crate::characters::zero::skill::BURST_PACKETS_HYPOTHESIS_ID.description()
        );
        let receipts = report["hit_receipts"].as_array().unwrap();
        let mut total = 0;
        for (i, receipt) in receipts.iter().enumerate() {
            assert_eq!(receipt["damage_effect_id"], BURST_PACKETS[i]);
            assert_eq!(receipt["time_us"], 100);
            assert_eq!(receipt["actor_index"], 0);
            assert_eq!(receipt["target_index"], 0);
            assert_eq!(
                super::support::hit_metadata(&report, receipt)["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
            );
            assert_eq!(
                receipt["non_crit_damage"],
                json!(format!("{}/{}", expected_milli[i], 1))
            );
            total += expected_milli[i];
            let native = frozen_row("DT_SkillDamageData", BURST_PACKETS[i]);
            assert_eq!(native["FixedCritRate"], 0.0);
            assert_eq!(rate_milli(&native, index), expected_milli[i]);
        }
        assert_eq!(total, expected_total);
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            json!(format!("{}/{}", expected_total, 1))
        );
    }
}

#[test]
fn zero_burst_train_keeps_pending_dead_postmortem_and_off_field_boundaries() {
    let mut pending_req = burst_request(1, 0);
    pending_req.absolute_limit_us = 99;
    pending_req.relative_duration_us = 99;
    let pending = run_game_scenario(&pending_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(pending["execution_completed"], false);
    assert_eq!(pending["hits_settled"], 0);
    assert_eq!(pending["pending_events"], 8);
    pending_req.absolute_limit_us = 3_600_000_000;
    pending_req.relative_duration_us = 100;
    let settled = run_game_scenario(&pending_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(settled["execution_completed"], true);
    assert_eq!(settled["hits_settled"], 7);

    for dead_target in [0, 1] {
        let mut dead_req = burst_request(1, 0);
        let mut other = dead_req.targets[0].clone();
        other.target_index = 1;
        dead_req.targets.push(other);
        dead_req.targets[dead_target].hp_current = 0;
        let dead = run_game_scenario(&dead_req, &ExecutionLimits::default()).unwrap();
        assert_eq!(dead["execution_completed"], true);
        assert_eq!(dead["hits_settled"], 7);
        assert!(
            dead["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .all(|hit| hit["target_index"] == 1 - dead_target)
        );
    }

    let mut postmortem_req = burst_request(1, 0);
    postmortem_req.targets[0].hp_current = 1400;
    let mut surviving_target = postmortem_req.targets[0].clone();
    surviving_target.target_index = 1;
    surviving_target.hp_current = 100_000;
    postmortem_req.targets.push(surviving_target);
    let postmortem = run_game_scenario(&postmortem_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(postmortem["hits_settled"], 10);
    let postmortem_hits = postmortem["hit_receipts"].as_array().unwrap();
    assert_eq!(
        postmortem_hits
            .iter()
            .filter(|hit| hit["target_index"] == 0)
            .count(),
        3
    );
    assert_eq!(
        postmortem_hits
            .iter()
            .filter(|hit| hit["target_index"] == 1)
            .count(),
        7
    );
    assert_eq!(
        postmortem["hit_receipts"][0]["damage_effect_id"],
        BURST_PACKETS[0]
    );
    assert_eq!(
        postmortem["hit_receipts"][1]["damage_effect_id"],
        BURST_PACKETS[1]
    );

    let mut off_field_req = burst_request(1, 0);
    let mut ally = off_field_req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.weapon_id.clear();
    ally.gear_procs.clear();
    off_field_req.actors.push(ally);
    off_field_req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 50,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    let off_field = run_game_scenario(&off_field_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(off_field["hits_settled"], 7);
    assert!(
        off_field["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|hit| hit["actor_index"] == 0)
    );

    let mut defeated_owner_req = burst_request(1, 0);
    defeated_owner_req.actors[0].initial_hp = 10;
    defeated_owner_req.actors[0].initial_hp_max = 10;
    defeated_owner_req.incoming_damage_events = vec![GameActorDamageEvent {
        timestamp_us: 0,
        actor_index: 0,
        delivered_damage: 10,
    }];
    defeated_owner_req.model_assumptions.insert(
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let defeated_owner =
        run_game_scenario(&defeated_owner_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(defeated_owner["final_actor_states"][0]["hp"], 0);
    assert_eq!(defeated_owner["hits_settled"], 0);
}

#[test]
fn zero_burst_failed_energy_admission_has_no_packets_passives_or_nakupeda_selection() {
    let mut req = burst_request(1, 4);
    let mut other = req.targets[0].clone();
    other.target_index = 1;
    req.targets.push(other);
    req.actors[0].initial_energy = 99;
    req.actors[0].weapon_id = "weapon:fork_Nakupeda".into();
    req.actors[0].gear_procs.push(GearProcConfig::Nakupeda {
        hp_bonus: Ratio {
            numerator: 6,
            denominator: 25,
        },
        lowest_hp_heal_ratio: Ratio {
            numerator: 1,
            denominator: 5,
        },
        shield_ratio: Ratio {
            numerator: 1,
            denominator: 5,
        },
        shield_duration_us: 15_000_000,
        team_heal_ratio: Ratio {
            numerator: 1,
            denominator: 10,
        },
        cooldown_us: 30_000_000,
    });
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 0);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 0);
    assert!(
        report["proc_selection_receipts"]
            .as_array()
            .is_none_or(Vec::is_empty)
    );
    assert!(
        report["resolved_model_assumptions"]
            [crate::characters::zero::skill::BURST_PACKETS_HYPOTHESIS_ID.as_str()]
        .is_null()
    );
    assert!(
        report["resolved_model_assumptions"]
            [crate::characters::zero::skill::BURST_A3_CRIT_HYPOTHESIS_ID.as_str()]
        .is_null()
    );
    assert!(
        report["resolved_model_assumptions"]
            [crate::characters::zero::skill::BURST_A4_DAMAGE_BONUS_HYPOTHESIS_ID.as_str()]
        .is_null()
    );
}

#[test]
fn zero_a3_crit_adds_per_burst_packet_and_does_not_leak_to_skill_or_normal() {
    let mut req = burst_request(1, 3);
    let mut other = req.targets[0].clone();
    other.target_index = 1;
    req.targets.push(other);
    req.actors[0].static_stats.crit_rate = Ratio {
        numerator: 1,
        denominator: 10,
    };
    req.stochastic_crit = Some(crate::simulation::ResearchCritPolicy {
        roll_policy: crate::simulation::ResearchCritRollPolicy::IndependentPerTargetHitV1,
        hypothesis:
            "Independent research crit rolls per delivered packet; game roll sharing is unknown"
                .into(),
        eligible_effects: BTreeMap::from([("character:1046".into(), BTreeSet::from(["*".into()]))]),
        actor_sequences: BTreeMap::from([("character:1046".into(), 54)]),
    });
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["rng"]["streams"]["character:1046"]["draws"], 14);
    let mut per_target_rolls = [0, 0];
    for (index, hit) in report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let target_index = hit["target_index"].as_u64().unwrap() as usize;
        per_target_rolls[target_index] += 1;
        assert_eq!(hit["crit_rate"], json!("3/5"));
        assert_eq!(hit["crit_sample"]["draws"][0]["index"], index);
    }
    assert_eq!(per_target_rolls, [7, 7]);
    assert_eq!(
        report,
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap()
    );
    for hit in report["hit_receipts"].as_array().unwrap() {
        let sample = &hit["crit_sample"];
        assert_eq!(sample["bound"], 5);
        assert_eq!(
            hit["critical"],
            sample["value"].as_u64().unwrap() < 3,
            "bounded sample below 3 of 5 is a crit: {sample}"
        );
    }

    let mut capped = burst_request(1, 3);
    capped.actors[0].static_stats.crit_rate = Ratio {
        numerator: 3,
        denominator: 4,
    };
    capped.stochastic_crit = req.stochastic_crit.clone();
    let cap_report = run_game_scenario(&capped, &ExecutionLimits::default()).unwrap();
    assert_eq!(cap_report["rng"]["streams"]["character:1046"]["draws"], 0);
    assert!(
        cap_report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|hit| { hit["critical"] == true && hit.get("crit_sample").is_none() })
    );

    for (ability_id, packet_id) in [
        ("GA_Female046_Melee", "GE_Player_Female046_Melee1_Damage"),
        ("GA_Female046_Skill", "GE_Player_Female046_Skill1_Damage"),
    ] {
        let mut no_leak = burst_request(1, 3);
        no_leak.actors[0].static_stats.crit_rate = Ratio {
            numerator: 1,
            denominator: 10,
        };
        let mut other = no_leak.targets[0].clone();
        other.target_index = 1;
        no_leak.targets.push(other);
        no_leak.stochastic_crit = None;
        no_leak.actions = vec![GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: ability_id.into(),
            array_index: 0,
            damage_effect_id: packet_id.into(),
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 100,
        }];
        let no_leak_report = run_game_scenario(&no_leak, &ExecutionLimits::default()).unwrap();
        assert!(
            no_leak_report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .all(|hit| { hit["crit_rate"] == json!("1/10") })
        );
        assert!(
            no_leak_report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .all(|hit| hit["target_index"] == 0)
        );
    }
}

#[test]
fn zero_burst_hypotheses_are_known_reached_and_exact_in_strict_mode() {
    let mut req = burst_request(1, 4);
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 1;
    let role =
        crate::characters::action_roles::resolve("character:1046", "GA_Female046_UltraSkill")
            .unwrap();
    req.model_assumptions
        .insert(role.hypothesis_id.into(), role.description.into());
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    assert!(crate::simulation::validation::validate_model_assumptions(&req).is_ok());
    req.model_assumptions.insert(
        crate::characters::zero::skill::BURST_PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::zero::skill::BURST_PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );

    req.model_assumptions.insert(
        crate::characters::zero::skill::BURST_ALL_IN_RANGE_HYPOTHESIS_ID.into(),
        crate::characters::zero::skill::BURST_ALL_IN_RANGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::zero::skill::BURST_ALL_IN_RANGE_HYPOTHESIS_ID.into(),
        "Each target within a fixed 20m sphere receives Zero Burst damage.".into(),
    );
    req.model_assumptions.insert(
        crate::characters::zero::skill::BURST_ALL_IN_RANGE_HYPOTHESIS_ID.into(),
        crate::characters::zero::skill::BURST_ALL_IN_RANGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::zero::skill::BURST_A3_CRIT_HYPOTHESIS_ID.into(),
        crate::characters::zero::skill::BURST_A3_CRIT_HYPOTHESIS_ID
            .description()
            .into(),
    );

    req.model_assumptions.insert(
        crate::characters::zero::skill::BURST_A4_DAMAGE_BONUS_HYPOTHESIS_ID.into(),
        crate::characters::zero::skill::BURST_A4_DAMAGE_BONUS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::simulation::energy::ZERO_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::ZERO_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    assert!(crate::simulation::validation::validate_model_assumptions(&req).is_ok());
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 7);
    assert_eq!(
        report["resolved_model_assumptions"]
            [crate::characters::zero::skill::BURST_ALL_IN_RANGE_HYPOTHESIS_ID.as_str()],
        crate::characters::zero::skill::BURST_ALL_IN_RANGE_HYPOTHESIS_ID.description()
    );
    assert!(report["rng"]["streams"]["character:1046"].is_null());

    req.model_assumptions.insert(
        crate::characters::zero::skill::BURST_A4_DAMAGE_BONUS_HYPOTHESIS_ID.into(),
        "A mismatched A4 rule".into(),
    );
}

#[test]
fn zero_a4_uses_capped_base_atk_and_adds_within_damage_bonus_zone() {
    for (base_atk, expected_bonus) in [
        (100, 1.5),
        (249, 1.649),
        (250, 1.65),
        (251, 1.65),
        (1000, 1.65),
    ] {
        let mut req = burst_request(1, 4);
        req.actors[0].static_stats.base_atk = base_atk;
        req.actors[0].static_stats.damage_bonus = Ratio {
            numerator: 2,
            denominator: 5,
        };
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        let first_damage = fraction_value(&report["hit_receipts"][0]["non_crit_damage"]);
        assert!(
            (first_damage / 1433.0 - expected_bonus).abs() < 1e-12,
            "base ATK {base_atk}: first damage {first_damage}, expected multiplier {expected_bonus}; receipt {}",
            report["hit_receipts"][0]
        );
        assert_eq!(
            report["resolved_model_assumptions"]
                [crate::characters::zero::skill::BURST_A4_DAMAGE_BONUS_HYPOTHESIS_ID.as_str()],
            crate::characters::zero::skill::BURST_A4_DAMAGE_BONUS_HYPOTHESIS_ID.description()
        );
    }

    let mut base = burst_request(1, 4);
    base.actors[0].static_stats.base_atk = 1000;
    base.actors[0].static_stats.damage_bonus = Ratio {
        numerator: 2,
        denominator: 5,
    };
    let base_report = run_game_scenario(&base, &ExecutionLimits::default()).unwrap();
    let mut final_atk_changed = base.clone();
    final_atk_changed.actors[0].static_stats.atk = Ratio {
        numerator: 4000,
        denominator: 1,
    };
    let changed_report =
        run_game_scenario(&final_atk_changed, &ExecutionLimits::default()).unwrap();
    let base_ratio = fraction_value(&base_report["hit_receipts"][0]["non_crit_damage"]) / 1433.0;
    let changed_ratio =
        fraction_value(&changed_report["hit_receipts"][0]["non_crit_damage"]) / (1433.0 * 4.0);
    assert!((base_ratio - 1.65).abs() < 1e-12);
    assert!((changed_ratio - 1.65).abs() < 1e-12);

    for (ability_id, packet_id, expected) in [
        (
            "GA_Female046_Melee",
            "GE_Player_Female046_Melee1_Damage",
            json!("1736/5"),
        ),
        (
            "GA_Female046_Skill",
            "GE_Player_Female046_Skill1_Damage",
            json!("1512/5"),
        ),
    ] {
        let mut other = burst_request(1, 4);
        other.actors[0].static_stats.damage_bonus = Ratio {
            numerator: 2,
            denominator: 5,
        };
        other.actions = vec![GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: ability_id.into(),
            array_index: 0,
            damage_effect_id: packet_id.into(),
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 100,
        }];
        let other_report = run_game_scenario(&other, &ExecutionLimits::default()).unwrap();
        assert_eq!(other_report["hit_receipts"][0]["non_crit_damage"], expected);
    }
}
