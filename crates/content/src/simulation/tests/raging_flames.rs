use super::*;
use crate::simulation::metadata::{DamageElement, InputActionRole};
use serde_json::Value;

fn request(with_weapon: bool) -> GameScenarioRequest {
    let mut request = sample_s1_request();
    request.relative_duration_us = 20_000_000;
    let actor = &mut request.actors[0];
    actor.character_id = "character:1010".into();
    actor.weapon_id = if with_weapon {
        "weapon:fork_wushoutieyu".into()
    } else {
        String::new()
    };
    actor.gear_procs = if with_weapon {
        vec![GearProcConfig::RagingFlames {
            lakshana_bonus: Ratio {
                numerator: 3,
                denominator: 20,
            },
            redirect_ultimate_bonus: Ratio {
                numerator: 1,
                denominator: 10,
            },
            redirect_skill_stack_bonus: Ratio {
                numerator: 1,
                denominator: 20,
            },
            duration_us: 10_000_000,
        }]
    } else {
        Vec::new()
    };
    actor.talent_levels = None;
    actor.static_stats.base_atk = 1000;
    actor.static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    actor.static_stats.damage_bonus = Ratio::zero();
    actor.static_stats.elemental_bonus = Ratio::zero();
    actor.static_stats.elemental_bonuses.clear();
    actor.static_stats.crit_rate = Ratio::zero();
    request.targets[0].resistance = Ratio::zero();
    request.targets[0].defense = 1080;
    request.actions.clear();
    request.action_metadata.clear();
    request
}

fn append(
    request: &mut GameScenarioRequest,
    time_us: u64,
    ability: &str,
    effect: &str,
    energy: u64,
) {
    request.actions.push(GameActionCommand::InvokeAbility {
        timestamp_us: time_us,
        actor_index: 0,
        ability_id: ability.into(),
        damage_effect_id: effect.into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: energy,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 0,
    });
}

fn append_synthetic_action(
    request: &mut GameScenarioRequest,
    time_us: u64,
    role: InputActionRole,
    damage_type: &str,
) {
    let index = request.actions.len();
    let ability = format!("raging-flames-synthetic-skill-{index}");
    let effect = format!("raging-flames-synthetic-effect-{index}");
    append(request, time_us, &ability, &effect, 0);
    let hypothesis = "test.raging_flames_synthetic_skill_v1";
    let description = "Synthetic single-packet Redirect Skill declarations isolate buff stack boundaries; native Nanally Skill and Ultimate packet trains are exercised in the paired actions.";
    request
        .model_assumptions
        .insert(hypothesis.into(), description.into());
    request.action_metadata.insert(
        index,
        crate::simulation::ResearchActionMetadata {
            hypothesis_id: hypothesis.into(),
            ability_id: ability,
            damage_effect_id: effect,
            action_gameplay_tags: Some(vec![
                match role {
                    InputActionRole::Normal => "Ability.Melee",
                    InputActionRole::RedirectSkill => "Ability.Skill",
                    InputActionRole::Ultimate => "Ability.UltraSkill",
                    InputActionRole::Qte => "Owner.QTE",
                    _ => "Owner.Other",
                }
                .into(),
            ]),
            input_action_role: Some(role),
            damage_type_ex: Some(format!("EExecutionDamageTypeEX::DAMAGE_TYPE_{damage_type}")),
            damage_source_category: Some(format!(
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_{}",
                match role {
                    InputActionRole::Normal => "A",
                    InputActionRole::RedirectSkill => "E",
                    InputActionRole::Ultimate => "Q",
                    InputActionRole::Qte => "QTE",
                    _ => "OTHER",
                }
            )),
            fixed_crit_rate: Some(crate::simulation::ResearchFixedCritRate::NoOverride),
            unbal_value: None,
            packet_ga_name_binding: None,
        },
    );
}

fn append_synthetic_action_with_category(
    request: &mut GameScenarioRequest,
    time_us: u64,
    role: InputActionRole,
    damage_type: &str,
    category: &str,
) {
    append_synthetic_action(request, time_us, role, damage_type);
    let index = request.actions.len() - 1;
    request
        .action_metadata
        .get_mut(&index)
        .unwrap()
        .damage_source_category = Some(format!(
        "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_{category}"
    ));
}

fn add_ally_for_switch(request: &mut GameScenarioRequest) {
    let mut ally = request.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1046".into();
    ally.initial_cycle = 100;
    ally.weapon_id.clear();
    ally.gear_procs.clear();
    ally.talent_levels = None;
    request.actors.push(ally);
}

fn exact(value: &Value) -> ExactFraction {
    serde_json::from_value(value.clone()).unwrap()
}

fn assert_research_action_identities(request: &GameScenarioRequest) {
    for (index, metadata) in &request.action_metadata {
        let GameActionCommand::InvokeAbility {
            ability_id,
            damage_effect_id,
            ..
        } = &request.actions[*index]
        else {
            panic!("research metadata must refer to an invoked ability");
        };
        assert_eq!(&metadata.ability_id, ability_id);
        assert_eq!(&metadata.damage_effect_id, damage_effect_id);
    }
}

#[test]
fn raging_flames_uses_native_nanally_packets_with_delivery_time_categories_and_caps() {
    let mut baseline = request(false);
    let mut equipped = request(true);
    for current in [&mut baseline, &mut equipped] {
        append_synthetic_action(current, 0, InputActionRole::RedirectSkill, "LAKSHANA");
        append(
            current,
            1_000_000,
            "GA_Nanally_UltraSkill",
            "GE_Player_Nanally_UltraSkill1_Damage",
            100,
        );
        append(
            current,
            5_000_000,
            "GA_Nanally_Skill",
            "GE_Player_Nanally_Skill1_Damage",
            0,
        );
        append_synthetic_action(
            current,
            6_500_000,
            InputActionRole::RedirectSkill,
            "LAKSHANA",
        );
        append_synthetic_action(current, 7_000_000, InputActionRole::RedirectSkill, "NATURE");
        current.actions.push(GameActionCommand::SwitchActor {
            timestamp_us: 7_500_000,
            target_actor_index: 1,
            cooldown_us: 0,
        });
        current.actions.push(GameActionCommand::SwitchActor {
            timestamp_us: 7_600_000,
            target_actor_index: 0,
            cooldown_us: 0,
        });
        append_synthetic_action(current, 9_000_000, InputActionRole::Normal, "NATURE");
        append_synthetic_action_with_category(
            current,
            9_500_000,
            InputActionRole::Normal,
            "NATURE",
            "E",
        );
        append_synthetic_action_with_category(
            current,
            9_750_000,
            InputActionRole::Normal,
            "NATURE",
            "Q",
        );
        append_synthetic_action(
            current,
            10_000_000,
            InputActionRole::RedirectSkill,
            "LAKSHANA",
        );
        append_synthetic_action_with_category(
            current,
            10_500_000,
            InputActionRole::Normal,
            "LAKSHANA",
            "E",
        );
        if let Some(GameActionCommand::InvokeAbility { hit_delay_us, .. }) =
            current.actions.last_mut()
        {
            *hit_delay_us = 3_000_000;
        }
        append_synthetic_action(
            current,
            14_000_000,
            InputActionRole::RedirectSkill,
            "LAKSHANA",
        );
        add_ally_for_switch(current);
        assert_research_action_identities(current);
    }
    let baseline = run_game_scenario(&baseline, &ExecutionLimits::default()).unwrap();
    let equipped = run_game_scenario(&equipped, &ExecutionLimits::default()).unwrap();
    assert_eq!(baseline["execution_completed"], true);
    assert_eq!(equipped["execution_completed"], true);
    let direct = |report: &Value| {
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["actor_index"] == 0 && hit["switch_origin"]["kind"] != "esper_cycle")
            .cloned()
            .collect::<Vec<_>>()
    };
    let base_hits = direct(&baseline);
    let weapon_hits = direct(&equipped);
    assert_eq!(base_hits.len(), weapon_hits.len());
    assert_eq!(base_hits.len(), 7 + 5 + 5 + 9 + 2); // Seven direct Q packets and five later Cat Skill packets.
    assert!(weapon_hits.iter().any(|hit| hit["time_us"] == 13_500_000));
    assert!(
        !crate::simulation::tests::support::final_buffs(&equipped)
            .iter()
            .any(|buff| buff["id"]
                .as_str()
                .is_some_and(|id| id.starts_with("raging_flames.")))
    );

    for (base, actual) in base_hits.iter().zip(&weapon_hits) {
        assert_eq!(base["command_index"], actual["command_index"]);
        assert_eq!(base["damage_effect_id"], actual["damage_effect_id"]);
        let time = actual["time_us"].as_u64().unwrap();
        let actual_meta = super::support::hit_metadata(&equipped, actual);
        let role: InputActionRole =
            serde_json::from_value(actual_meta["action"]["input_role"].clone()).unwrap();
        let category = actual_meta["packet"]["native_damage_source_category"]
            .as_str()
            .unwrap_or_default();
        let element: Option<DamageElement> =
            serde_json::from_value(actual_meta["packet"]["element"].clone()).unwrap();
        let mut bonus = ExactFraction::new(0, 1).unwrap();
        if element == Some(DamageElement::Lakshana) {
            bonus = bonus.add(ExactFraction::new(3, 20).unwrap()).unwrap();
        }
        if time == 9_500_000 || time == 9_750_000 {
            assert_eq!(role, InputActionRole::Normal);
        }
        if (1_000_000..11_000_000).contains(&support::world_time_at(&equipped, time))
            && (category.ends_with("_E") || category.ends_with("_Q"))
        {
            bonus = bonus.add(ExactFraction::new(1, 10).unwrap()).unwrap();
            if category.ends_with("_E") {
                let stacks = [5_000_000, 6_500_000, 7_000_000, 10_000_000]
                    .into_iter()
                    .filter(|cast| *cast <= time)
                    .count()
                    .min(2);
                bonus = bonus
                    .add(ExactFraction::new(stacks as u128, 20).unwrap())
                    .unwrap();
            }
        }
        let expected = exact(&base["non_crit_damage"])
            .mul(ExactFraction::new(1, 1).unwrap().add(bonus).unwrap())
            .unwrap();
        assert_eq!(exact(&actual["non_crit_damage"]), expected, "hit {actual}");
    }
    assert_eq!(
        equipped["resolved_model_assumptions"]
            [crate::weapons::raging_flames::HYPOTHESIS_ID.as_str()],
        crate::weapons::raging_flames::HYPOTHESIS_ID.description()
    );
}

#[test]
fn raging_flames_requires_its_equipped_weapon_and_does_not_stack_outside_the_window() {
    let mut request = request(true);
    append(
        &mut request,
        0,
        "GA_Nanally_Skill",
        "GE_Player_Nanally_Skill1_Damage",
        0,
    );
    let mut wrong_weapon = request.clone();
    wrong_weapon.actors[0].weapon_id = "weapon:fork_twinbirds".into();
    assert!(
        run_game_scenario(&wrong_weapon, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires equipped weapon:fork_wushoutieyu")
    );
}

#[test]
fn rejected_nanally_skill_recast_inside_authority_cannot_apply_a_raging_flames_stack() {
    let mut request = request(true);
    append(
        &mut request,
        0,
        "GA_Nanally_UltraSkill",
        "GE_Player_Nanally_UltraSkill1_Damage",
        100,
    );
    append(
        &mut request,
        4_000_000,
        "GA_Nanally_Skill",
        "GE_Player_Nanally_Skill1_Damage",
        0,
    );
    append(
        &mut request,
        5_500_000,
        "GA_Nanally_Skill",
        "GE_Player_Nanally_Skill1_Damage",
        0,
    );
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert!(
        report["halt_diagnostic"]
            .as_str()
            .unwrap()
            .contains("on cooldown until 6429917")
    );
    assert_eq!(
        report["action_metadata_ledger"].as_array().unwrap().len(),
        2
    );
}

#[test]
fn nanally_accepted_utility_removal_cannot_apply_a_raging_flames_skill_stack() {
    let mut request = request(true);
    request.relative_duration_us = 8_000_000;
    append(
        &mut request,
        0,
        "GA_Nanally_UltraSkill",
        "GE_Player_Nanally_UltraSkill1_Damage",
        100,
    );
    append(
        &mut request,
        4_000_000,
        "GA_Nanally_Skill",
        "GE_Player_Nanally_Skill1_Damage",
        0,
    );
    append(
        &mut request,
        6_500_000,
        "GA_Nanally_Skill",
        "GE_Player_Nanally_Skill1_Damage",
        0,
    );
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "raging_flames_skill_stack")
            .count(),
        1
    );
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "utility_ability")
            .count(),
        1
    );
}

#[test]
fn ultimate_refresh_clears_redirect_skill_stacks_without_ending_the_new_window() {
    let mut baseline = request(false);
    let mut equipped = request(true);
    for current in [&mut baseline, &mut equipped] {
        append(
            current,
            0,
            "GA_Nanally_UltraSkill",
            "GE_Player_Nanally_UltraSkill1_Damage",
            100,
        );
        append(
            current,
            4_000_000,
            "GA_Nanally_Skill",
            "GE_Player_Nanally_Skill1_Damage",
            0,
        );
        append_synthetic_action(
            current,
            5_500_000,
            InputActionRole::RedirectSkill,
            "LAKSHANA",
        );
        assert_research_action_identities(current);
        append(
            current,
            18_584_608,
            "GA_Nanally_UltraSkill",
            "GE_Player_Nanally_UltraSkill1_Damage",
            0,
        );
        append_synthetic_action(
            current,
            22_584_608,
            InputActionRole::RedirectSkill,
            "LAKSHANA",
        );
    }
    let baseline = run_game_scenario(&baseline, &ExecutionLimits::default()).unwrap();
    let equipped = run_game_scenario(&equipped, &ExecutionLimits::default()).unwrap();
    assert_eq!(baseline["execution_completed"], true);
    assert_eq!(equipped["execution_completed"], true);
    let base_hit = baseline["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["command_index"] == 4)
        .unwrap();
    let equipped_hit = equipped["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["command_index"] == 4)
        .unwrap();
    let expected = exact(&base_hit["non_crit_damage"])
        .mul(ExactFraction::new(13, 10).unwrap())
        .unwrap();
    assert_eq!(exact(&equipped_hit["non_crit_damage"]), expected);
}
