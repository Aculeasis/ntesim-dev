use super::*;
use serde_json::{Value, json};

fn request() -> GameScenarioRequest {
    let mut request = sample_s1_request();
    let actor = &mut request.actors[0];
    actor.character_id = "character:1054".into();
    actor.weapon_id = "weapon:fork_prism".into();
    actor.gear_procs = vec![GearProcConfig::FarSideMoon {
        crit_rate_bonus: Ratio {
            numerator: 16,
            denominator: 100,
        },
        skill_damage_per_stack: Ratio {
            numerator: 7,
            denominator: 100,
        },
        basic_attack_damage_bonus: Ratio {
            numerator: 28,
            denominator: 100,
        },
        lakshana_damage_bonus: Ratio {
            numerator: 16,
            denominator: 100,
        },
        duration_us: 15_000_000,
    }];
    actor.talent_levels = Some([8; 3]);
    actor.awakening = ntesim_data::AwakeningSelection::prefix(0);
    actor.initial_energy = actor.energy_capacity;
    actor.static_stats.base_atk = 1000;
    actor.static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    actor.static_stats.crit_rate = Ratio {
        numerator: 90,
        denominator: 100,
    };
    actor.static_stats.damage_bonus = Ratio::zero();
    actor.static_stats.elemental_bonus = Ratio::zero();
    actor.static_stats.elemental_bonuses.clear();
    request.targets[0].resistance = Ratio::zero();
    request.targets[0].defense = 1080;
    request.relative_duration_us = 20_000_000;
    request.actions.clear();
    request.action_metadata.clear();
    request
}

#[test]
fn far_side_of_the_moon_hypothesis_records_and_validates_conflicts() {
    let mut request = request();
    let id = crate::weapons::far_side_moon::HYPOTHESIS_ID.as_str();
    let description = crate::weapons::far_side_moon::HYPOTHESIS_ID.description();
    let mut resolved = std::collections::BTreeMap::new();
    crate::runtime_api::record_hypothesis(&request, &mut resolved, id, description).unwrap();
    assert_eq!(resolved[id], description);

    request
        .model_assumptions
        .insert(id.into(), "mismatched hypothesis".into());
    let mismatch =
        crate::runtime_api::record_hypothesis(&request, &mut Default::default(), id, description)
            .unwrap_err();
    assert!(mismatch.contains("conflicting reached model hypothesis"));
}

fn append(
    request: &mut GameScenarioRequest,
    time_us: u64,
    ability: &str,
    effect: &str,
    hit_delay_us: u64,
) {
    request.actions.push(GameActionCommand::InvokeAbility {
        timestamp_us: time_us,
        actor_index: 0,
        ability_id: ability.into(),
        damage_effect_id: effect.into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us,
    });
}

fn exact(value: &Value) -> ExactFraction {
    serde_json::from_value(value.clone()).unwrap()
}

fn append_synthetic(
    request: &mut GameScenarioRequest,
    time_us: u64,
    role: crate::simulation::metadata::InputActionRole,
    damage_type: &str,
    category: &str,
    hit_delay_us: u64,
) {
    let index = request.actions.len();
    let ability_id = format!("far-side-moon-synthetic-{index}");
    let damage_effect_id = format!("far-side-moon-effect-{index}");
    append(
        request,
        time_us,
        &ability_id,
        &damage_effect_id,
        hit_delay_us,
    );
    request.model_assumptions.insert(
        "test.far_side_moon_synthetic_v1".into(),
        "Synthetic packets keep delivery categories, elements, cast roles and impact times independently selectable for weapon boundary checks.".into(),
    );
    request.action_metadata.insert(
        index,
        crate::simulation::ResearchActionMetadata {
            hypothesis_id: "test.far_side_moon_synthetic_v1".into(),
            ability_id,
            damage_effect_id,
            action_gameplay_tags: Some(vec!["Ability.Skill".into()]),
            input_action_role: Some(role),
            damage_type_ex: Some(format!("EExecutionDamageTypeEX::DAMAGE_TYPE_{damage_type}")),
            damage_source_category: Some(format!(
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_{category}"
            )),
            fixed_crit_rate: Some(crate::simulation::ResearchFixedCritRate::NoOverride),
            unbal_value: None,
            packet_ga_name_binding: None,
        },
    );
}

#[test]
fn far_side_of_the_moon_scales_native_daffodil_skill_normal_and_burst_packets_by_delivery() {
    let mut equipped = request();
    append(
        &mut equipped,
        0,
        "GA_Daffodill_Skill",
        "GE_Player_Daffodill_Skill1_Damage",
        0,
    );
    append(
        &mut equipped,
        1_000_000,
        "GA_Daffodill_Melee",
        "GE_Player_Daffodill_Melee1_Damage",
        14_000_000,
    );
    append(
        &mut equipped,
        2_000_000,
        "GA_Daffodill_UltraSkill",
        "GE_Player_Daffodill_UltraSkill1_Damage",
        0,
    );
    append(
        &mut equipped,
        14_000_000,
        "GA_Daffodill_Melee",
        "GE_Player_Daffodill_Melee1_Damage",
        0,
    );
    append(
        &mut equipped,
        15_000_000,
        "GA_Daffodill_Melee",
        "GE_Player_Daffodill_Melee1_Damage",
        0,
    );
    append(
        &mut equipped,
        16_000_000,
        "GA_Daffodill_Skill",
        "GE_Player_Daffodill_Skill1_Damage",
        0,
    );

    let mut baseline = equipped.clone();
    baseline.actors[0].gear_procs.clear();
    baseline.actors[0].weapon_id.clear();
    let base_report = run_game_scenario(&baseline, &ExecutionLimits::default()).unwrap();
    let report = run_game_scenario(&equipped, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(base_report["execution_completed"], true);
    let base_hits = base_report["hit_receipts"].as_array().unwrap();
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 34);
    assert_eq!(hits.len(), base_hits.len());
    for (base, hit) in base_hits.iter().zip(hits) {
        let category =
            super::support::hit_metadata(&report, hit)["packet"]["native_damage_source_category"]
                .as_str()
                .unwrap_or_default();
        let time = hit["time_us"].as_u64().unwrap();
        let zone = if time >= 2_000_000 {
            ExactFraction::new(11, 10).unwrap()
        } else {
            ExactFraction::new(1, 1).unwrap()
        };
        let mut bonus = ExactFraction::new(0, 1).unwrap();
        if category.ends_with("_E") && time >= 16_000_000 {
            bonus = bonus.add(ExactFraction::new(14, 100).unwrap()).unwrap();
        }
        if category.ends_with("_A")
            && ((support::world_time_at(&report, time) < 15_000_000)
                || (support::world_time_at(&report, 16_000_000)
                    ..support::world_time_at(&report, 16_000_000) + 15_000_000)
                    .contains(&support::world_time_at(&report, time)))
        {
            bonus = bonus.add(ExactFraction::new(28, 100).unwrap()).unwrap();
        }
        let expected = exact(&base["non_crit_damage"])
            .mul(zone.add(bonus).unwrap())
            .unwrap()
            .div(zone)
            .unwrap();
        assert_eq!(exact(&hit["non_crit_damage"]), expected, "packet {hit}");
        assert_eq!(base["damage_effect_id"], hit["damage_effect_id"]);
    }
    assert_eq!(report["hit_receipts"][0]["crit_rate"], json!("53/50"));
    assert!(
        report["resolved_model_assumptions"][crate::weapons::far_side_moon::HYPOTHESIS_ID.as_str()]
            .is_string()
    );
}

#[test]
fn far_side_of_the_moon_crit_bonus_clamps_random_rate_and_fixed_packet_rate_overrides_it() {
    let mut saturated = request();
    append_synthetic(
        &mut saturated,
        0,
        crate::simulation::metadata::InputActionRole::Other,
        "NATURE",
        "E",
        0,
    );
    saturated.stochastic_crit = Some(crate::simulation::ResearchCritPolicy {
        roll_policy: crate::simulation::ResearchCritRollPolicy::IndependentPerTargetHitV1,
        hypothesis: "Test whether additive weapon crit rate clamps the sampled probability.".into(),
        eligible_effects: std::collections::BTreeMap::from([(
            "character:1054".into(),
            std::collections::BTreeSet::from(["far-side-moon-effect-0".into()]),
        )]),
        actor_sequences: std::collections::BTreeMap::from([("character:1054".into(), 13)]),
    });
    let report = run_game_scenario(&saturated, &ExecutionLimits::default()).unwrap();
    let hit = &report["hit_receipts"][0];
    assert_eq!(hit["crit_rate"], json!("53/50"));
    assert_eq!(hit["critical"], true);
    assert!(hit.get("crit_sample").is_none());
    assert_eq!(report["rng"]["streams"]["character:1054"]["draws"], 0);

    let mut fixed = saturated;
    fixed.actors[0].static_stats.crit_rate = Ratio::zero();
    fixed.model_assumptions.insert(
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_ID
            .description()
            .into(),
    );
    fixed.action_metadata.get_mut(&0).unwrap().fixed_crit_rate =
        Some(crate::simulation::ResearchFixedCritRate::Probability {
            rate: Ratio {
                numerator: 1,
                denominator: 2,
            },
        });
    let report = run_game_scenario(&fixed, &ExecutionLimits::default()).unwrap();
    let hit = &report["hit_receipts"][0];
    assert_eq!(hit["crit_rate"], json!("1/2"));
    assert_eq!(
        super::support::hit_metadata(&report, hit)["packet"]["fixed_crit_rate_provenance"]["hypothesis_id"],
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID.as_str()
    );
    assert_eq!(report["rng"]["streams"]["character:1054"]["draws"], 1);
}

#[test]
fn far_side_of_the_moon_requires_its_equipped_weapon_and_positive_window() {
    let mut request = request();
    request.actors[0].weapon_id = "weapon:fork_GoldWool".into();
    assert!(
        run_game_scenario(&request, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires equipped weapon:fork_prism")
    );
    request.actors[0].weapon_id = "weapon:fork_prism".into();
    if let GearProcConfig::FarSideMoon { duration_us, .. } = &mut request.actors[0].gear_procs[0] {
        *duration_us = 0;
    }
    assert!(
        run_game_scenario(&request, &ExecutionLimits::default())
            .unwrap_err()
            .contains("positive duration")
    );
}

#[test]
fn far_side_of_the_moon_filters_delivered_categories_and_elements_and_refreshes_one_owner_window() {
    use crate::simulation::metadata::InputActionRole;
    let mut equipped = request();
    equipped.actors[0].static_stats.crit_rate = Ratio::zero();
    equipped.relative_duration_us = 24_000_000;
    append_synthetic(
        &mut equipped,
        0,
        InputActionRole::RedirectSkill,
        "CHAOS",
        "E",
        0,
    );
    append_synthetic(
        &mut equipped,
        1_000_000,
        InputActionRole::Normal,
        "LAKSHANA",
        "A",
        0,
    );
    append_synthetic(
        &mut equipped,
        2_000_000,
        InputActionRole::Normal,
        "NATURE",
        "E",
        0,
    );
    append_synthetic(
        &mut equipped,
        3_000_000,
        InputActionRole::RedirectSkill,
        "LAKSHANA",
        "E",
        0,
    );
    equipped.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 4_000_000,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    equipped.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 4_500_000,
        target_actor_index: 0,
        cooldown_us: 0,
    });
    append_synthetic(
        &mut equipped,
        5_000_000,
        InputActionRole::Normal,
        "CHAOS",
        "A",
        0,
    );
    append_synthetic(
        &mut equipped,
        6_000_000,
        InputActionRole::Other,
        "LAKSHANA",
        "Q",
        0,
    );
    equipped.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 7_000_000,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    append_synthetic(
        &mut equipped,
        7_500_000,
        InputActionRole::RedirectSkill,
        "LAKSHANA",
        "E",
        0,
    );
    if let Some(GameActionCommand::InvokeAbility { actor_index, .. }) = equipped.actions.last_mut()
    {
        *actor_index = 1;
    }
    equipped.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 8_000_000,
        target_actor_index: 0,
        cooldown_us: 0,
    });
    append_synthetic(
        &mut equipped,
        17_000_000,
        InputActionRole::Normal,
        "LAKSHANA",
        "A",
        1_000_000,
    );
    append_synthetic(
        &mut equipped,
        17_000_000,
        InputActionRole::Normal,
        "LAKSHANA",
        "A",
        1_100_000,
    );
    let mut ally = equipped.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.weapon_id.clear();
    ally.gear_procs.clear();
    equipped.actors.push(ally);

    let mut baseline = equipped.clone();
    baseline.actors[0].gear_procs.clear();
    baseline.actors[0].weapon_id.clear();
    let base_report = run_game_scenario(&baseline, &ExecutionLimits::default()).unwrap();
    let report = run_game_scenario(&equipped, &ExecutionLimits::default()).unwrap();
    let base_hits = base_report["hit_receipts"].as_array().unwrap();
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), base_hits.len());
    for (base, hit) in base_hits.iter().zip(hits) {
        let category =
            super::support::hit_metadata(&report, hit)["packet"]["native_damage_source_category"]
                .as_str()
                .unwrap_or_default();
        let element = super::support::hit_metadata(&report, hit)["packet"]["element"]
            .as_str()
            .unwrap_or_default();
        let time = hit["time_us"].as_u64().unwrap();
        let owner = hit["actor_index"] == 0;
        let active = owner && time < 18_000_000;
        let mut bonus = ExactFraction::new(0, 1).unwrap();
        let stacks = if time > 5_000_000 && time <= 10_000_000 {
            2u32
        } else if time > 1_000_000 && time <= 6_000_000 {
            1u32
        } else {
            0u32
        };
        if owner && category.ends_with("_E") && stacks > 0 {
            bonus = bonus
                .add(ExactFraction::new(u128::from(stacks) * 7, 100).unwrap())
                .unwrap();
        }
        if active && category.ends_with("_A") {
            bonus = bonus.add(ExactFraction::new(28, 100).unwrap()).unwrap();
        }
        if active && stacks > 0 && element == "lakshana" {
            bonus = bonus.add(ExactFraction::new(16, 100).unwrap()).unwrap();
        }
        let expected = exact(&base["non_crit_damage"])
            .mul(ExactFraction::new(1, 1).unwrap().add(bonus).unwrap())
            .unwrap();
        assert_eq!(exact(&hit["non_crit_damage"]), expected, "packet {hit}");
    }
    let events = report["trace"].as_array().unwrap();
    assert_eq!(
        events
            .iter()
            .filter(|entry| entry["event_kind"] == "far_side_moon_redirect_window")
            .count(),
        2
    );
    assert_eq!(
        events
            .iter()
            .filter(|entry| entry["event_kind"] == "far_side_moon_skill_stack")
            .count(),
        3
    );
    assert!(
        crate::simulation::tests::support::final_buffs(&report)
            .iter()
            .all(|buff| buff["id"] != "far_side_moon.redirect_window"
                && buff["id"] != "far_side_moon.skill_stacks")
    );
}

#[test]
fn rejected_redirect_skill_does_not_refresh_the_far_side_of_the_moon_window() {
    use crate::simulation::metadata::InputActionRole;
    let mut equipped = request();
    equipped.actors[0].initial_energy = 0;
    append_synthetic(
        &mut equipped,
        0,
        InputActionRole::RedirectSkill,
        "CHAOS",
        "Q",
        0,
    );
    if let GameActionCommand::InvokeAbility { energy_cost, .. } = &mut equipped.actions[0] {
        *energy_cost = 1;
    }
    append_synthetic(
        &mut equipped,
        1_000_000,
        InputActionRole::Normal,
        "LAKSHANA",
        "A",
        0,
    );
    let mut baseline = equipped.clone();
    baseline.actors[0].gear_procs.clear();
    baseline.actors[0].weapon_id.clear();
    let baseline = run_game_scenario(&baseline, &ExecutionLimits::default()).unwrap();
    let report = run_game_scenario(&equipped, &ExecutionLimits::default()).unwrap();
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|entry| entry["event_kind"] != "far_side_moon_redirect_window")
    );
    assert_eq!(
        report["hit_receipts"][0]["non_crit_damage"],
        baseline["hit_receipts"][0]["non_crit_damage"]
    );
}

fn projection_tables() -> std::collections::BTreeMap<String, serde_json::Value> {
    super::support::weapon_projection_tables()
}

fn projection_actor() -> GameActorConfig {
    serde_json::from_value(json!({
        "slot":0,"character_id":"character:1054","initial_hp":1,"initial_hp_max":1,"initial_energy":0,
        "energy_capacity":100,"weapon_id":"weapon:fork_prism","weapon_tier":1,
        "static_stats":{"base_atk":1,"atk":"1/1","hp":1,"def":1,
        "crit_rate":"0/1","crit_damage_bonus":"0/1"}
    }))
    .unwrap()
}

#[test]
fn far_side_moon_projection_binds_five_curves() {
    // Hand-derived from frozen CT_ForkBuff rows (independent of the implementation):
    // CritUp = LakshanaUp: 0.16/0.20/0.24/0.28/0.32 = 4/25, 1/5, 6/25, 7/25, 8/25;
    // DamageUp1 = DamageUp2: 0.28/0.35/0.42/0.49/0.56 = 7/25, 7/20, 21/50, 49/100, 14/25;
    // DamageUp2_CD = 15.0s.
    let tables = projection_tables();
    let expected_crit = [(4, 25), (1, 5), (6, 25), (7, 25), (8, 25)];
    let expected_damage = [(7, 25), (7, 20), (21, 50), (49, 100), (14, 25)];
    let expected_skill_stack = [(7, 100), (7, 80), (21, 200), (49, 400), (7, 50)];
    let mut actor = projection_actor();
    for tier in 1..=5 {
        let i = (tier - 1) as usize;
        actor.weapon_tier = tier as u32;
        actor.gear_procs.clear();
        let report = crate::weapons::far_side_moon::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap();
        let proc = &report["compiled_proc"];
        let (cn, cd) = expected_crit[i];
        let (dn, dd) = expected_damage[i];
        let (sn, sd) = expected_skill_stack[i];
        assert_eq!(proc["crit_rate_bonus"], json!(format!("{}/{}", cn, cd)));
        assert_eq!(
            proc["lakshana_damage_bonus"],
            json!(format!("{}/{}", cn, cd))
        );
        assert_eq!(
            proc["skill_damage_per_stack"],
            json!(format!("{}/{}", sn, sd))
        );
        assert_eq!(
            proc["basic_attack_damage_bonus"],
            json!(format!("{}/{}", dn, dd))
        );
        assert_eq!(proc["duration_us"], 15_000_000);
        assert_eq!(report["native_parameter_report"]["parameter_count"], 5);
        assert_eq!(report["runtime_execution_validated"], true);
        assert_eq!(actor.gear_procs.len(), 1);
    }
    // Group TWO enforcement: character:1003 is CHARACTER_GROUP_TYPE_FOUR.
    actor.character_id = "character:1003".into();
    assert!(
        crate::weapons::far_side_moon::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[]
        )
        .unwrap_err()
        .contains("group is incompatible")
    );
    // Unit-hint falsifier: flipping DamageUp1 to non-percent is rejected.
    actor.character_id = "character:1054".into();
    actor.weapon_tier = 1;
    let mut changed = tables.clone();
    changed
        .get_mut("DT_ForkUpgradeStarDataTable")
        .unwrap()["rows"]["upgradestar_pack_fork_prism_1"]["DataList"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["NameID"] == "buff_Prism_DamageUp1")
        .unwrap()["bIsPercent"] = json!(false);
    assert!(
        crate::weapons::far_side_moon::projection::project(
            &changed,
            1,
            "ntesim-current",
            &mut actor,
            &[]
        )
        .unwrap_err()
        .contains("unit hint")
    );
}

#[test]
fn far_side_moon_tier_values_hand_derived() {
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
    // Frozen CT_ForkBuff decimals reduced to exact fractions, no core calls.
    // CritUp = LakshanaUp: 0.16 -> 4/25, 0.20 -> 1/5, 0.24 -> 6/25, 0.28 -> 7/25, 0.32 -> 8/25.
    let crit_decimals = [(16, 100), (20, 100), (24, 100), (28, 100), (32, 100)];
    let crit_expected = [(4, 25), (1, 5), (6, 25), (7, 25), (8, 25)];
    // DamageUp1 = DamageUp2: 0.28 -> 7/25, 0.35 -> 7/20, 0.42 -> 21/50, 0.49 -> 49/100, 0.56 -> 14/25.
    let damage_decimals = [(28, 100), (35, 100), (42, 100), (49, 100), (56, 100)];
    let damage_expected = [(7, 25), (7, 20), (21, 50), (49, 100), (14, 25)];
    for ((n, d), (en, ed)) in crit_decimals.iter().zip(crit_expected.iter()) {
        assert_eq!(reduced(*n, *d), (*en, *ed));
    }
    for ((n, d), (en, ed)) in damage_decimals.iter().zip(damage_expected.iter()) {
        assert_eq!(reduced(*n, *d), (*en, *ed));
    }
    // Skill stack per-stack decimals (DamageUp1 / 4) reduced to exact fractions:
    // 0.07 -> 7/100, 0.0875 -> 7/80, 0.105 -> 21/200, 0.1225 -> 49/400, 0.14 -> 7/50.
    let skill_stack_decimals = [(28, 400), (35, 400), (42, 400), (49, 400), (56, 400)];
    let skill_stack_expected = [(7, 100), (7, 80), (21, 200), (49, 400), (7, 50)];
    for ((n, d), (en, ed)) in skill_stack_decimals.iter().zip(skill_stack_expected.iter()) {
        assert_eq!(reduced(*n, *d), (*en, *ed));
    }
    // Overlap falsifier: at T1 an A-category Lakshana packet inside the window gets
    // 1 + 7/25 + 4/25 = 36/25 additively, not (1+7/25) x (1+4/25) = 928/625 multiplicatively.
    assert_eq!(reduced(25 + 7 + 4, 25), (36, 25));
    assert_ne!(reduced(25 + 7 + 4, 25), reduced(32 * 29, 25 * 25));
}

#[test]
fn far_side_of_the_moon_stacks_bounded_refresh_at_cap_and_respect_acquisition_gate_and_positive_hp_loss()
 {
    use crate::simulation::metadata::InputActionRole;
    let mut request = request();
    // Hit 0 at 0us: Non-Basic (category E) packet -> does NOT grant stack
    append_synthetic(&mut request, 0, InputActionRole::Normal, "CHAOS", "E", 0);
    // Hit 1 at 100_000us: Basic Attack (category A) packet -> grants stack 1 (expires at 5_100_000)
    append_synthetic(
        &mut request,
        100_000,
        InputActionRole::Normal,
        "CHAOS",
        "A",
        0,
    );
    // Hit 2 at 200_000us (< 300ms since Hit 1): category A packet -> rejected by acquisition gate
    append_synthetic(
        &mut request,
        200_000,
        InputActionRole::Normal,
        "CHAOS",
        "A",
        0,
    );
    // Hit 3 at 450_000us (>= 300ms since Hit 1): category A packet -> grants stack 2 (expires at 5_450_000)
    append_synthetic(
        &mut request,
        450_000,
        InputActionRole::Normal,
        "CHAOS",
        "A",
        0,
    );
    // Hit 4 at 800_000us: category A packet -> grants stack 3 (expires at 5_800_000)
    append_synthetic(
        &mut request,
        800_000,
        InputActionRole::Normal,
        "CHAOS",
        "A",
        0,
    );
    // Hit 5 at 1_200_000us: category A packet -> grants stack 4 (cap reached, expires at 6_200_000)
    append_synthetic(
        &mut request,
        1_200_000,
        InputActionRole::Normal,
        "CHAOS",
        "A",
        0,
    );
    // Hit 6 at 1_600_000us: category A packet at cap -> stays at 4 stacks, refreshes deadline to 6_600_000
    append_synthetic(
        &mut request,
        1_600_000,
        InputActionRole::Normal,
        "CHAOS",
        "A",
        0,
    );
    // Hit 7 at 2_000_000us by ally (actor_index 1): category A packet -> does not grant wearer stacks
    request.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 1_900_000,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    append_synthetic(
        &mut request,
        2_000_000,
        InputActionRole::Normal,
        "CHAOS",
        "A",
        0,
    );
    if let Some(GameActionCommand::InvokeAbility { actor_index, .. }) = request.actions.last_mut() {
        *actor_index = 1;
    }
    request.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 2_100_000,
        target_actor_index: 0,
        cooldown_us: 0,
    });
    // Hit 8 at 6_599_999us: Category E packet inside the 4-stack window -> gains 4 * 7/100 = 28/100
    append_synthetic(
        &mut request,
        6_599_999,
        InputActionRole::Normal,
        "CHAOS",
        "E",
        0,
    );
    // Hit 9 at 6_600_000us: Category E packet exactly at expiry (expiry precedes hit) -> 0 bonus
    append_synthetic(
        &mut request,
        6_600_000,
        InputActionRole::Normal,
        "CHAOS",
        "E",
        0,
    );

    let mut ally = request.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.weapon_id.clear();
    ally.gear_procs.clear();
    request.actors.push(ally);

    let mut baseline = request.clone();
    baseline.actors[0].gear_procs.clear();
    baseline.actors[0].weapon_id.clear();
    let base_report = run_game_scenario(&baseline, &ExecutionLimits::default()).unwrap();
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true);

    let base_hits = base_report["hit_receipts"].as_array().unwrap();
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), base_hits.len());

    let hit8 = hits
        .iter()
        .find(|h| h["time_us"] == 6_599_999 && h["actor_index"] == 0)
        .unwrap();
    let base8 = base_hits
        .iter()
        .find(|h| h["time_us"] == 6_599_999 && h["actor_index"] == 0)
        .unwrap();
    let expected8 = exact(&base8["non_crit_damage"])
        .mul(ExactFraction::new(128, 100).unwrap())
        .unwrap();
    assert_eq!(exact(&hit8["non_crit_damage"]), expected8);

    let hit9 = hits
        .iter()
        .find(|h| h["time_us"] == 6_600_000 && h["actor_index"] == 0)
        .unwrap();
    let base9 = base_hits
        .iter()
        .find(|h| h["time_us"] == 6_600_000 && h["actor_index"] == 0)
        .unwrap();
    assert_eq!(
        exact(&hit9["non_crit_damage"]),
        exact(&base9["non_crit_damage"])
    );

    let stack_events: Vec<&Value> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "far_side_moon_skill_stack")
        .collect();
    assert_eq!(stack_events.len(), 5);
    assert_eq!(stack_events[0]["data"]["stacks"], 1);
    assert_eq!(stack_events[0]["time_us"], 100_000);
    assert_eq!(stack_events[1]["data"]["stacks"], 2);
    assert_eq!(stack_events[1]["time_us"], 450_000);
    assert_eq!(stack_events[2]["data"]["stacks"], 3);
    assert_eq!(stack_events[2]["time_us"], 800_000);
    assert_eq!(stack_events[3]["data"]["stacks"], 4);
    assert_eq!(stack_events[3]["time_us"], 1_200_000);
    assert_eq!(stack_events[4]["data"]["stacks"], 4);
    assert_eq!(stack_events[4]["time_us"], 1_600_000);
    assert_eq!(
        stack_events[4]["data"]["expires_at_us"],
        1_600_000 + 5_000_000
    );
}
