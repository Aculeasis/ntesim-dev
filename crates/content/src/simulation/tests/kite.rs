use super::*;
use crate::simulation::metadata::InputActionRole;
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn tables() -> BTreeMap<String, Value> {
    BTreeMap::from([("CT_ForkBuff".into(), ntesim_data::weapon_curves::overlay())])
}

fn request(with_weapon: bool) -> GameScenarioRequest {
    let mut request = research_request();
    request.stochastic_crit = None;
    request.relative_duration_us = 20_000_000;
    let actor = &mut request.actors[0];
    actor.character_id = "character:1003".into();
    actor.weapon_id = "weapon:fork_Kite".into();
    actor.weapon_tier = 1;
    actor.gear_procs = if with_weapon {
        vec![GearProcConfig::KiteAtk {
            atk_bonus: Ratio {
                numerator: 1,
                denominator: 10,
            },
            lakshana_bonus: Ratio {
                numerator: 1,
                denominator: 10,
            },
            duration_us: 15_000_000,
        }]
    } else {
        Vec::new()
    };
    actor.static_stats.base_atk = 1000;
    actor.static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    actor.static_stats.crit_rate = Ratio::zero();
    actor.static_stats.damage_bonus = Ratio::zero();
    actor.static_stats.elemental_bonus = Ratio::zero();
    actor.static_stats.elemental_bonuses.clear();
    request.targets[0].defense = 0;
    request.targets[0].resistance = Ratio::zero();
    request.actions.clear();
    request.action_metadata.clear();
    request
}

fn kite_role_tag(role: InputActionRole) -> &'static str {
    match role {
        InputActionRole::Normal => "Ability.Melee",
        InputActionRole::RedirectSkill => "Ability.Skill",
        InputActionRole::Ultimate => "Ability.UltraSkill",
        InputActionRole::Qte => "Owner.QTE",
        _ => "Owner.Other",
    }
}

fn kite_role_category(role: InputActionRole) -> &'static str {
    match role {
        InputActionRole::Normal => "A",
        InputActionRole::RedirectSkill => "E",
        InputActionRole::Ultimate => "Q",
        InputActionRole::Qte => "QTE",
        _ => "OTHER",
    }
}

fn append_synthetic_action(
    request: &mut GameScenarioRequest,
    time_us: u64,
    role: InputActionRole,
    damage_type: &str,
) {
    let index = request.actions.len();
    let ability = format!("kite-synthetic-skill-{index}");
    let effect = format!("kite-synthetic-effect-{index}");
    request.actions.push(GameActionCommand::InvokeAbility {
        timestamp_us: time_us,
        actor_index: 0,
        ability_id: ability.clone(),
        damage_effect_id: effect.clone(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 0,
    });
    let hypothesis = "test.kite_synthetic_skill_v1";
    request
        .model_assumptions
        .insert(hypothesis.into(), "Synthetic single-packet actions isolate the Redirect Skill grant and the ATK window boundaries.".into());
    request.action_metadata.insert(
        index,
        crate::simulation::ResearchActionMetadata {
            hypothesis_id: hypothesis.into(),
            ability_id: ability,
            damage_effect_id: effect,
            action_gameplay_tags: Some(vec![kite_role_tag(role).into()]),
            input_action_role: Some(role),
            damage_type_ex: Some(format!("EExecutionDamageTypeEX::DAMAGE_TYPE_{damage_type}")),
            damage_source_category: Some(format!(
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_{}",
                kite_role_category(role)
            )),
            fixed_crit_rate: Some(crate::simulation::ResearchFixedCritRate::NoOverride),
            unbal_value: None,
            packet_ga_name_binding: None,
        },
    );
}

fn rounded(report: &Value) -> Vec<(u64, u64)> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|hit| {
            (
                hit["time_us"].as_u64().unwrap(),
                hit["rounded_damage"].as_u64().unwrap(),
            )
        })
        .collect()
}

#[test]
fn kite_projection_binds_five_refinements_with_independent_curve_oracles() {
    let tables = tables();
    let mut actor: GameActorConfig = serde_json::from_value(json!({
        "slot": 0, "character_id": "character:1003", "character_level": 80, "awakening": [],
        "purchased_skill_level": 8, "initial_hp": 1, "initial_hp_max": 1, "initial_energy": 0,
        "energy_capacity": 100, "weapon_id": "weapon:fork_Kite", "weapon_tier": 1,
        "static_stats": {
            "base_atk": 1, "atk": "1/1", "hp": 1, "def": 1,
            "crit_rate": "0/1", "crit_damage_bonus": "0/1"
        }
    }))
    .unwrap();
    let expected = [(1u64, 10u64), (3, 25), (7, 50), (4, 25), (9, 50)];
    for (tier, (numerator, denominator)) in (1..=5u32).zip(expected) {
        actor.weapon_tier = tier;
        actor.gear_procs.clear();
        let report = crate::weapons::kite::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap();
        assert_eq!(
            report["compiled_proc"]["atk_bonus"],
            json!(format!("{}/{}", numerator, denominator)),
            "tier {tier} atk"
        );
        assert_eq!(
            report["compiled_proc"]["lakshana_bonus"],
            json!(format!("{}/{}", numerator, denominator)),
            "tier {tier} lakshana"
        );
        assert_eq!(report["compiled_proc"]["duration_us"], 15_000_000);
        assert_eq!(report["native_parameter_report"]["parameter_count"], 3);
        let proc: GearProcConfig = serde_json::from_value(report["compiled_proc"].clone()).unwrap();
        assert!(matches!(proc, GearProcConfig::KiteAtk { .. }));
        assert_eq!(actor.gear_procs.len(), 1);
    }
    assert_eq!(
        ntesim_data::weapon_definitions::lookup(&tables, "fork_Kite")
            .unwrap()
            .1["ElementData"]["ApplyGroupType"],
        "ECharacterGroupType::CHARACTER_GROUP_TYPE_FOUR"
    );
}

#[test]
fn kite_projection_rejects_incompatible_group() {
    let tables = tables();
    let mut actor: GameActorConfig = serde_json::from_value(json!({
        "slot": 0, "character_id": "character:1019", "character_level": 80, "awakening": [],
        "purchased_skill_level": 8, "initial_hp": 1, "initial_hp_max": 1, "initial_energy": 0,
        "energy_capacity": 100, "weapon_id": "weapon:fork_Kite", "weapon_tier": 1,
        "static_stats": {
            "base_atk": 1, "atk": "1/1", "hp": 1, "def": 1,
            "crit_rate": "0/1", "crit_damage_bonus": "0/1"
        }
    }))
    .unwrap();
    assert!(
        crate::weapons::kite::projection::project(&tables, 1, "ntesim-current", &mut actor, &[],)
            .unwrap_err()
            .contains("incompatible")
    );
    assert!(actor.gear_procs.is_empty());
}

#[test]
fn kite_requires_its_equipped_weapon_and_a_positive_duration() {
    let mut request = request(true);
    append_synthetic_action(&mut request, 0, InputActionRole::RedirectSkill, "NATURE");
    let mut wrong_weapon = request.clone();
    wrong_weapon.actors[0].weapon_id = "weapon:fork_twinbirds".into();
    assert!(
        run_game_scenario(&wrong_weapon, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires equipped weapon:fork_Kite")
    );
    let mut zero_duration = request.clone();
    if let GearProcConfig::KiteAtk { duration_us, .. } = &mut zero_duration.actors[0].gear_procs[0]
    {
        *duration_us = 0;
    }
    assert!(
        run_game_scenario(&zero_duration, &ExecutionLimits::default())
            .unwrap_err()
            .contains("positive duration")
    );
}

#[test]
fn kite_redirect_skill_grants_wearer_only_atk_without_refresh() {
    let mut request = request(true);
    request.relative_duration_us = 12_000_000;
    append_synthetic_action(&mut request, 0, InputActionRole::RedirectSkill, "NATURE");
    append_synthetic_action(
        &mut request,
        10_000_000,
        InputActionRole::RedirectSkill,
        "NATURE",
    );
    let mid = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(mid["execution_completed"], true);
    let buffs = crate::simulation::tests::support::final_buffs(&mid);
    let instances: Vec<&Value> = buffs
        .iter()
        .filter(|buff| buff["id"] == "kite.redirect_skill_atk_up")
        .collect();
    assert_eq!(instances.len(), 1);
    let instance = instances[0];
    assert_eq!(instance["source"], "weapon:fork_Kite");
    assert_eq!(instance["owner_actor"], 0);
    assert_eq!(instance["stat_modifiers"]["atk_percent"], json!("1/10"));
    assert_eq!(instance["refresh_policy"]["policy"], "disallow");
    assert_eq!(instance["duration"]["expires_at_us"], 15_000_000);
    request.relative_duration_us = 20_000_000;
    let expired = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(expired["execution_completed"], true);
    assert!(
        !crate::simulation::tests::support::final_buffs(&expired)
            .iter()
            .any(|buff| buff["id"] == "kite.redirect_skill_atk_up")
    );
    assert_eq!(
        expired["resolved_model_assumptions"][crate::weapons::kite::HYPOTHESIS_ID.as_str()],
        crate::weapons::kite::HYPOTHESIS_ID.description()
    );
}

#[test]
fn kite_atk_bonus_scales_damage_and_expires_without_refresh() {
    let mut equipped = request(true);
    let mut baseline = request(false);
    for current in [&mut equipped, &mut baseline] {
        append_synthetic_action(current, 0, InputActionRole::RedirectSkill, "NATURE");
        append_synthetic_action(current, 1_000_000, InputActionRole::Normal, "NATURE");
        append_synthetic_action(
            current,
            10_000_000,
            InputActionRole::RedirectSkill,
            "NATURE",
        );
        append_synthetic_action(current, 14_000_000, InputActionRole::Normal, "NATURE");
        append_synthetic_action(current, 15_000_000, InputActionRole::Normal, "NATURE");
        append_synthetic_action(current, 16_000_000, InputActionRole::Normal, "NATURE");
    }
    let equipped = run_game_scenario(&equipped, &ExecutionLimits::default()).unwrap();
    let baseline = run_game_scenario(&baseline, &ExecutionLimits::default()).unwrap();
    assert_eq!(equipped["execution_completed"], true);
    assert_eq!(baseline["execution_completed"], true);
    let equipped_hits = rounded(&equipped);
    let baseline_hits = rounded(&baseline);
    assert_eq!(equipped_hits.len(), baseline_hits.len());
    assert_eq!(equipped_hits.len(), 6);
    for ((time, actual), (_, base)) in equipped_hits.iter().zip(baseline_hits.iter()) {
        assert_eq!(*base, 1000, "baseline hit at {time}");
        let expected = if *time < 15_000_000 { 1100 } else { 1000 };
        assert_eq!(*actual, expected, "equipped hit at {time}");
    }
}

#[test]
fn kite_non_redirect_casts_do_not_grant() {
    let mut equipped = request(true);
    append_synthetic_action(&mut equipped, 0, InputActionRole::Normal, "NATURE");
    append_synthetic_action(
        &mut equipped,
        5_000_000,
        InputActionRole::Ultimate,
        "NATURE",
    );
    append_synthetic_action(&mut equipped, 10_000_000, InputActionRole::Qte, "NATURE");
    let report = run_game_scenario(&equipped, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert!(
        !crate::simulation::tests::support::final_buffs(&report)
            .iter()
            .any(|buff| buff["id"] == "kite.redirect_skill_atk_up")
    );
    assert!(rounded(&report).iter().all(|(_, damage)| *damage == 1000));
}

#[test]
fn kite_unmarked_lakshana_during_active_window_receives_only_atk() {
    let mut request = request(true);
    append_synthetic_action(&mut request, 0, InputActionRole::RedirectSkill, "NATURE");
    append_synthetic_action(&mut request, 1_000_000, InputActionRole::Normal, "LAKSHANA");
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(rounded(&report), [(0, 1100), (1_000_000, 1100)]);
}

#[test]
fn kite_target_or_predicate_is_per_recipient_element_and_half_open_deadline() {
    let stain = crate::simulation::reactions::STAIN_STATUS_TAG;
    let remora = crate::simulation::reactions::REMORA_STATUS_TAG;
    for tags in [vec![stain], vec![remora], vec![stain, remora]] {
        let mut req = request(true);
        req.targets.push(req.targets[0].clone());
        req.targets[1].target_index = 1;
        let mut state = crate::simulation::state::GameSimulatorState::new(&req);
        let mut action = super::support::declared_test_action(
            "test-kite",
            crate::simulation::metadata::ActionCategory::Skill,
        );
        action.input_role = Some(InputActionRole::RedirectSkill);
        super::support::invoke_gear_cast(&req, &mut state, 0, &action);
        crate::simulation::state::apply_target_tagged_status(
            &mut state.buffs,
            crate::simulation::state::TargetStatusDeclaration {
                source_actor: 0,
                source: "test".into(),
                target_index: 1,
                id: "test-kite-marks".into(),
                now_us: 0,
                duration_us: 1_000_000,
                reductions: BTreeMap::new(),
            },
            tags.iter().map(|s| s.to_string()).collect(),
            1,
            ntesim_core::buffs::dispatch::RefreshPolicy::Reset,
            ntesim_core::buffs::dispatch::DecayPolicy::AllAtOnce,
        )
        .unwrap();
        for (time, target, element, expected) in [
            (
                999_999,
                1,
                Some(crate::simulation::metadata::DamageElement::Lakshana),
                1,
            ),
            (
                999_999,
                0,
                Some(crate::simulation::metadata::DamageElement::Lakshana),
                0,
            ),
            (
                1_000_000,
                1,
                Some(crate::simulation::metadata::DamageElement::Lakshana),
                0,
            ),
            (
                999_999,
                1,
                Some(crate::simulation::metadata::DamageElement::Nature),
                0,
            ),
        ] {
            let modifiers = super::support::gear_zone_modifiers_for_element(
                &req, &state, time, target, element,
            );
            assert_eq!(
                modifiers.damage_bonus,
                ExactFraction::new(expected, 10).unwrap()
            );
        }
    }
}

#[test]
fn kite_lakshana_after_window_expiry_resolves_without_bonus() {
    let mut equipped = request(true);
    let mut baseline = request(false);
    for current in [&mut equipped, &mut baseline] {
        append_synthetic_action(current, 0, InputActionRole::RedirectSkill, "NATURE");
        append_synthetic_action(current, 16_000_000, InputActionRole::Normal, "LAKSHANA");
    }
    let equipped = run_game_scenario(&equipped, &ExecutionLimits::default()).unwrap();
    let baseline = run_game_scenario(&baseline, &ExecutionLimits::default()).unwrap();
    assert_eq!(equipped["execution_completed"], true);
    let equipped_hits = rounded(&equipped);
    let baseline_hits = rounded(&baseline);
    assert_eq!(equipped_hits.len(), 2);
    assert_eq!(baseline_hits.len(), 2);
    assert_eq!(equipped_hits[1], (16_000_000, 1000));
    assert_eq!(baseline_hits[1], (16_000_000, 1000));
}

fn request_with_kite_proc(atk_num: u64, atk_den: u64) -> GameScenarioRequest {
    let mut request = request(false);
    request.actors[0].gear_procs = vec![GearProcConfig::KiteAtk {
        atk_bonus: Ratio {
            numerator: atk_num,
            denominator: atk_den,
        },
        lakshana_bonus: Ratio {
            numerator: atk_num,
            denominator: atk_den,
        },
        duration_us: 15_000_000,
    }];
    request
}

#[test]
fn kite_t5_window_damage_oracles() {
    // Hand-derived from frozen buff_Kite_AtkUp (T5 = 9/50): base 1000, static
    // 1000 -> 1000 + 1000*9/50 = 1180 inside the 15s window, 1000 after expiry.
    let mut equipped = request_with_kite_proc(9, 50);
    append_synthetic_action(&mut equipped, 0, InputActionRole::RedirectSkill, "NATURE");
    append_synthetic_action(&mut equipped, 1_000_000, InputActionRole::Normal, "NATURE");
    append_synthetic_action(&mut equipped, 14_900_000, InputActionRole::Normal, "NATURE");
    append_synthetic_action(&mut equipped, 15_000_000, InputActionRole::Normal, "NATURE");
    append_synthetic_action(&mut equipped, 16_000_000, InputActionRole::Normal, "NATURE");
    let report = run_game_scenario(&equipped, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = rounded(&report);
    assert_eq!(hits.len(), 5);
    assert_eq!(
        hits[0],
        (0, 1180),
        "triggering redirect hit is inside the window"
    );
    assert_eq!(hits[1], (1_000_000, 1180));
    assert_eq!(hits[2], (14_900_000, 1180));
    assert_eq!(
        hits[3],
        (15_000_000, 1000),
        "exact 15s boundary is already expired"
    );
    assert_eq!(hits[4], (16_000_000, 1000));
}

#[test]
fn kite_atk_bonus_scales_from_base_not_compiled_static() {
    // COMBAT_FORMULA_CONTRACT §2: a live ATK% buff adds base_atk * p to the
    // compiled static ATK; it never multiplies the compiled static.
    // base 800 / static 1000, T3 bonus 7/50: 1000 + 800*7/50 = 1112, not 1140.
    let mut request = request_with_kite_proc(7, 50);
    request.actors[0].static_stats.base_atk = 800;
    append_synthetic_action(&mut request, 0, InputActionRole::RedirectSkill, "NATURE");
    append_synthetic_action(&mut request, 1_000_000, InputActionRole::Normal, "NATURE");
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = rounded(&report);
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0], (0, 1112));
    assert_eq!(hits[1], (1_000_000, 1112));
}
