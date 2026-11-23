use super::*;
use crate::simulation::metadata::{DamageElement, InputActionRole};
use crate::simulation::state::GameSimulatorState;
use serde_json::json;

fn projection_tables() -> BTreeMap<String, serde_json::Value> {
    super::support::weapon_projection_tables()
}

fn projection_actor() -> crate::simulation::GameActorConfig {
    serde_json::from_value(json!({
        "slot": 0, "character_id": "character:1046", "character_level": 80, "awakening": [],
        "purchased_skill_level": 8, "initial_hp": 1, "initial_hp_max": 1, "initial_energy": 100,
        "energy_capacity": 100, "weapon_id": "weapon:fork_Butterfly", "weapon_tier": 1,
        "static_stats": {
            "base_atk": 1, "atk": "1/1", "hp": 1, "def": 1,
            "crit_rate": "0/1",
            "crit_damage_bonus": "0/1"
        }
    }))
    .unwrap()
}

#[test]
fn butterfly_projection_binds_five_curves_and_rejects_group_and_unit_changes() {
    let tables = projection_tables();
    let item = ntesim_data::weapon_definitions::lookup(&tables, "fork_Butterfly")
        .unwrap()
        .1;
    assert_eq!(
        item["ElementData"]["ApplyGroupType"],
        "ECharacterGroupType::CHARACTER_GROUP_TYPE_ONE"
    );
    assert_eq!(item["ElementData"]["ExclusiveCharacterIDArray"], json!([]));
    let mut actor = projection_actor();
    let expected = [
        ((3, 20), (1, 10), (1, 5)),
        ((7, 40), (9, 80), (9, 40)),
        ((1, 5), (1, 8), (1, 4)),
        ((9, 40), (11, 80), (11, 40)),
        ((1, 4), (3, 20), (3, 10)),
    ];
    for (tier, ((anima_num, anima_den), (attach_num, attach_den), (ult_num, ult_den))) in
        (1..=5u32).zip(expected)
    {
        actor.weapon_tier = tier;
        actor.gear_procs.clear();
        let report = crate::weapons::butterfly::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap();
        assert_eq!(
            report["compiled_proc"]["anima_bonus"],
            json!(format!("{}/{}", anima_num, anima_den))
        );
        assert_eq!(
            report["compiled_proc"]["attachment_bonus"],
            json!(format!("{}/{}", attach_num, attach_den))
        );
        assert_eq!(
            report["compiled_proc"]["ult_attachment_bonus"],
            json!(format!("{}/{}", ult_num, ult_den))
        );
        assert_eq!(report["compiled_proc"]["ult_duration_us"], 6_000_000);
        assert_eq!(report["native_parameter_report"]["parameter_count"], 4);
        assert_eq!(report["runtime_execution_validated"], true);
    }
    assert_eq!(
        tables["DT_ForkUpgradeStarDataTable"]["rows"]["upgradestar_pack_fork_Butterfly_1"]["Description"]
            ["LocalizedString"],
        "Increases Anima DMG by <lv>{0}</>.\nIncreases the wearer's Attachment DMG by <lv>{1}</>. Increases the Attachment DMG bonus to <lv>{2}</> for <lv>{3}</>s when the wearer casts an Ultimate. Effect does not stack."
    );

    actor.character_id = "character:1019".into();
    assert!(
        crate::weapons::butterfly::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[]
        )
        .unwrap_err()
        .contains("group is incompatible")
    );

    actor.character_id = "character:1046".into();
    actor.weapon_tier = 1;
    let mut changed = tables.clone();
    changed.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]
        ["upgradestar_pack_fork_Butterfly_1"]["DataList"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["NameID"] == "buff_Butterfly_CD")
        .unwrap()["bIsPercent"] = json!(true);
    assert!(
        crate::weapons::butterfly::projection::project(
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

fn butterfly_proc() -> GearProcConfig {
    GearProcConfig::ButterflyDamage {
        anima_bonus: Ratio {
            numerator: 3,
            denominator: 20,
        },
        attachment_bonus: Ratio {
            numerator: 1,
            denominator: 10,
        },
        ult_attachment_bonus: Ratio {
            numerator: 1,
            denominator: 5,
        },
        ult_duration_us: 6_000_000,
    }
}

fn butterfly_t5_proc() -> GearProcConfig {
    GearProcConfig::ButterflyDamage {
        anima_bonus: Ratio {
            numerator: 1,
            denominator: 4,
        },
        attachment_bonus: Ratio {
            numerator: 3,
            denominator: 20,
        },
        ult_attachment_bonus: Ratio {
            numerator: 3,
            denominator: 10,
        },
        ult_duration_us: 6_000_000,
    }
}

#[test]
fn butterfly_validate_config_rejects_wrong_weapon_and_zero_duration() {
    let limits = ExecutionLimits::default();
    let mut actor = research_request().actors[0].clone();
    actor.weapon_id = "weapon:fork_Butterfly".into();
    actor.gear_procs = vec![butterfly_proc()];
    crate::gear::validate_config(&actor, 0, 40_000_000, &limits).unwrap();

    actor.weapon_id = "weapon:fork_Worldrain".into();
    assert!(
        crate::gear::validate_config(&actor, 0, 40_000_000, &limits)
            .unwrap_err()
            .contains("requires equipped")
    );

    actor.weapon_id = "weapon:fork_Butterfly".into();
    actor.gear_procs = vec![GearProcConfig::ButterflyDamage {
        anima_bonus: Ratio {
            numerator: 3,
            denominator: 20,
        },
        attachment_bonus: Ratio {
            numerator: 1,
            denominator: 10,
        },
        ult_attachment_bonus: Ratio {
            numerator: 1,
            denominator: 5,
        },
        ult_duration_us: 0,
    }];
    assert!(
        crate::gear::validate_config(&actor, 0, 40_000_000, &limits)
            .unwrap_err()
            .contains("positive duration")
    );
}

fn anima_fixture_with(proc: GearProcConfig) -> GameScenarioRequest {
    let mut req = research_request();
    req.stochastic_crit = None;
    req.actors[0].weapon_id = "weapon:fork_Butterfly".into();
    req.actors[0].gear_procs = vec![proc];
    req.actors[0].static_stats.damage_bonus = Ratio {
        numerator: 1,
        denominator: 10,
    };
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.targets[0].resistance = Ratio::zero();
    req.action_metadata.get_mut(&1).unwrap().damage_type_ex =
        Some("EExecutionDamageTypeEX::DAMAGE_TYPE_COSMOS".into());
    req
}

#[test]
fn butterfly_static_anima_bonus_sums_in_the_damage_zone_for_nature_packets() {
    let req = anima_fixture();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // 1000 ATK * 1/2 DEF mitigation: Nature packets gain (1 + 0.1 + 0.15) = 625,
    // the Cosmos packet keeps (1 + 0.1) = 550.
    assert_eq!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|hit| hit["rounded_damage"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![625, 550, 625, 625, 625, 625]
    );
    assert_eq!(
        report["resolved_model_assumptions"][crate::weapons::butterfly::HYPOTHESIS_ID.as_str()],
        crate::weapons::butterfly::HYPOTHESIS_ID.description()
    );

    let mut bare = req.clone();
    bare.actors[0].gear_procs.clear();
    let baseline = run_game_scenario(&bare, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        baseline["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|hit| hit["rounded_damage"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![550, 550, 550, 550, 550, 550]
    );

    let mut duplicate = req.clone();
    let extra = duplicate.actors[0].gear_procs[0].clone();
    duplicate.actors[0].gear_procs.push(extra);
    assert!(
        run_game_scenario(&duplicate, &ExecutionLimits::default())
            .unwrap_err()
            .contains("duplicates")
    );
}

fn trigger_fixture() -> GameScenarioRequest {
    let mut req = research_request();
    req.relative_duration_us = 20_000_000;
    req.stochastic_crit = None;
    req.actors[0].character_id = "character:1046".into();
    req.actors[0].weapon_id = "weapon:fork_Butterfly".into();
    req.actors[0].gear_procs = vec![butterfly_proc()];
    req.actors[0].static_stats.damage_bonus = Ratio::zero();
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.targets[0].resistance = Ratio::zero();
    req.actions.clear();
    req.action_metadata.clear();
    req
}

fn append_cast(req: &mut GameScenarioRequest, actor: usize, time_us: u64, role: InputActionRole) {
    let index = req.actions.len();
    req.actions.push(GameActionCommand::InvokeAbility {
        timestamp_us: time_us,
        actor_index: actor,
        ability_id: "caller-test".into(),
        damage_effect_id: "caller-effect".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 0,
    });
    req.action_metadata.insert(
        index,
        crate::simulation::ResearchActionMetadata {
            hypothesis_id: "test.synthetic_action".into(),
            ability_id: "caller-test".into(),
            damage_effect_id: "caller-effect".into(),
            action_gameplay_tags: Some(vec!["Ability.Melee".into()]),
            input_action_role: Some(role),
            damage_type_ex: Some("EExecutionDamageTypeEX::DAMAGE_TYPE_NATURE".into()),
            damage_source_category: None,
            fixed_crit_rate: Some(crate::simulation::ResearchFixedCritRate::NoOverride),
            unbal_value: None,
            packet_ga_name_binding: None,
        },
    );
}

fn buff_instances(report: &serde_json::Value) -> Vec<serde_json::Value> {
    crate::simulation::tests::support::final_buffs(report)
        .into_iter()
        .filter(|buff| buff["id"] == "butterfly.ult_attachment")
        .collect()
}

#[test]
fn butterfly_ultimate_admission_refreshes_one_wearer_buff_without_stacking() {
    let mut req = trigger_fixture();
    append_cast(&mut req, 0, 0, InputActionRole::Normal);
    append_cast(&mut req, 0, 1_000_000, InputActionRole::Ultimate);
    append_cast(&mut req, 0, 4_000_000, InputActionRole::Ultimate);
    req.relative_duration_us = 5_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let buffs = buff_instances(&report);
    assert_eq!(buffs.len(), 1);
    let buff = &buffs[0];
    assert_eq!(buff["source"], "weapon:fork_Butterfly");
    assert_eq!(buff["owner_actor"], 0);
    assert_eq!(buff["stacks"], 1);
    assert_eq!(buff["refresh_policy"]["policy"], "reset");
    assert_eq!(
        buff["stat_modifiers"]["butterfly.ult_attachment_bonus"],
        json!("1/5")
    );
    assert_eq!(buff["duration"]["expires_at_us"], 8_000_000);
    assert_eq!(
        report["resolved_model_assumptions"][crate::weapons::butterfly::HYPOTHESIS_ID.as_str()],
        crate::weapons::butterfly::HYPOTHESIS_ID.description()
    );
}

#[test]
fn butterfly_buff_is_wearer_only_survives_switch_and_expires() {
    let mut req = trigger_fixture();
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.weapon_id.clear();
    ally.gear_procs.clear();
    req.actors.push(ally);
    append_cast(&mut req, 0, 1_000_000, InputActionRole::Ultimate);
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 2_000_000,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    req.relative_duration_us = 3_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let buffs = buff_instances(&report);
    assert_eq!(buffs.len(), 1);
    assert_eq!(buffs[0]["owner_actor"], 0);
    assert_eq!(buffs[0]["duration"]["expires_at_us"], 7_000_000);

    req.relative_duration_us = 12_000_000;
    let expired = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(expired["execution_completed"], true);
    assert!(buff_instances(&expired).is_empty());
}

#[test]
fn butterfly_non_ultimate_casts_do_not_grant_the_buff() {
    let mut req = trigger_fixture();
    append_cast(&mut req, 0, 0, InputActionRole::Normal);
    append_cast(&mut req, 0, 1_000_000, InputActionRole::RedirectSkill);
    append_cast(&mut req, 0, 2_000_000, InputActionRole::Qte);
    req.relative_duration_us = 5_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert!(buff_instances(&report).is_empty());
}

fn attachment_packet(
    tags: &[&str],
    element: Option<DamageElement>,
) -> crate::simulation::metadata::PacketMetadata {
    crate::simulation::metadata::PacketMetadata {
        damage_effect_id: "test-attachment".into(),
        native_ga_name: None,
        native_damage_type_ex: "EExecutionDamageTypeEX::DAMAGE_TYPE_NATURE".into(),
        native_damage_source_category: None,
        damage_kind: ntesim_core::combat::DamageKind::Elemental,
        element,
        fixed_crit_rate: None,
        unbal_value: None,
        fixed_crit_rate_provenance: None,
        unbal_value_provenance: None,
        semantic_tags: tags.iter().map(|tag| (*tag).to_owned()).collect(),
        semantic_provenance: None,
        source_evidence: None,
        provenance: crate::simulation::metadata::MetadataProvenance::AcceptedModelHypothesis {
            id: "test.attachment_packet".into(),
            description: "Test-owned synthetic packet metadata.".into(),
        },
        packet_kind: Default::default(),
    }
}

fn butterfly_damage(
    req: &GameScenarioRequest,
    buff_stats: &BTreeMap<String, Ratio>,
    packet: &crate::simulation::metadata::PacketMetadata,
) -> crate::simulation::ExactFraction {
    let state = GameSimulatorState::new(req);
    let action_metadata = super::support::declared_test_action(
        "caller-test",
        crate::simulation::metadata::ActionCategory::Skill,
    );
    let owned_status_counts = BTreeMap::new();
    let ctx = crate::runtime_api::DamageContext {
        foreign_dot_stacks: 0,
        packet_group_id: None,
        command_index: 0,
        config: &req.actors[0],
        start_fons: None,
        ability_id: "caller-test",
        action_metadata: &action_metadata,
        target: &state.targets[0],
        actor: &state.actors[0],
        world_time_us: 1,
        time_us: 1,
        debuff_count: 0,
        damage_effect_id: "test-attachment",
        actor_index: 0,
        active_actor_index: 0,
        target_count: 1,
        non_boss_target_count: 1,
        target_index: 0,
        buffs: &state.buffs,
        buff_stats: buff_stats.into(),
        owned_status_counts: &owned_status_counts,
        team_has_three_elements: false,
    };
    crate::weapons::butterfly::damage_bonus(
        &ctx,
        packet,
        crate::runtime_api::HitModifiers::new(crate::simulation::ExactFraction::new(0, 1).unwrap())
            .unwrap(),
    )
    .unwrap()
    .damage_bonus
}

#[test]
fn butterfly_attachment_override_replaces_static_bonus_while_active() {
    let mut req = research_request();
    req.actors[0].weapon_id = "weapon:fork_Butterfly".into();
    req.actors[0].gear_procs = vec![butterfly_proc()];
    let idle: BTreeMap<String, Ratio> = BTreeMap::new();
    let active: BTreeMap<String, Ratio> = BTreeMap::from([(
        "butterfly.ult_attachment_bonus".into(),
        Ratio {
            numerator: 1,
            denominator: 5,
        },
    )]);
    let attachment = crate::weapons::butterfly::ATTACHMENT_DAMAGE_TAG;
    let fraction = |numerator: u128, denominator: u128| {
        crate::simulation::ExactFraction::new(numerator, denominator).unwrap()
    };

    let packet = attachment_packet(&[attachment], Some(DamageElement::Nature));
    assert_eq!(butterfly_damage(&req, &idle, &packet), fraction(1, 4));
    assert_eq!(butterfly_damage(&req, &active, &packet), fraction(7, 20));

    let packet = attachment_packet(&[attachment], Some(DamageElement::Cosmos));
    assert_eq!(butterfly_damage(&req, &idle, &packet), fraction(1, 10));
    assert_eq!(butterfly_damage(&req, &active, &packet), fraction(1, 5));

    let packet = attachment_packet(&[], Some(DamageElement::Nature));
    assert_eq!(butterfly_damage(&req, &idle, &packet), fraction(3, 20));
    assert_eq!(butterfly_damage(&req, &active, &packet), fraction(3, 20));

    let packet = attachment_packet(&[], Some(DamageElement::Cosmos));
    assert_eq!(butterfly_damage(&req, &idle, &packet), fraction(0, 1));
    assert_eq!(butterfly_damage(&req, &active, &packet), fraction(0, 1));
}

fn anima_fixture() -> GameScenarioRequest {
    anima_fixture_with(butterfly_proc())
}

#[test]
fn butterfly_t5_damage_oracles() {
    // Hand-derived from frozen CT_ForkBuff rows (T5: anima 1/4, attachment 3/20,
    // ult attachment 3/10) and the additive damage zone: 1000 ATK, 1/10 general
    // bonus, 1/2 DEF mitigation.
    // Nature: (1 + 1/10 + 1/4) * 1000 * 1/2 = 675; Cosmos: (1 + 1/10) * 500 = 550.
    let req = anima_fixture_with(butterfly_t5_proc());
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
            .map(|hit| hit["rounded_damage"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![675, 550, 675, 675, 675, 675]
    );

    // Isolate the adapter's zones here; native Attachment train coverage lives
    // in butterfly_cycles and jiuyuan_pact.
    // Idle Nature+Attachment: 1/4 (anima) + 3/20 (attachment) = 2/5, which is
    // (1 + 1/10 + 2/5) * 500 = 750 rounded damage.
    // Active ult buff replaces the attachment bonus: 1/4 + 3/10 = 11/20, which
    // is (1 + 1/10 + 11/20) * 500 = 825 rounded damage.
    let mut req = research_request();
    req.actors[0].weapon_id = "weapon:fork_Butterfly".into();
    req.actors[0].gear_procs = vec![butterfly_t5_proc()];
    let idle: BTreeMap<String, Ratio> = BTreeMap::new();
    let active: BTreeMap<String, Ratio> = BTreeMap::from([(
        "butterfly.ult_attachment_bonus".into(),
        Ratio {
            numerator: 3,
            denominator: 10,
        },
    )]);
    let attachment = crate::weapons::butterfly::ATTACHMENT_DAMAGE_TAG;
    let fraction = |numerator: u128, denominator: u128| {
        crate::simulation::ExactFraction::new(numerator, denominator).unwrap()
    };
    let packet = attachment_packet(&[attachment], Some(DamageElement::Nature));
    assert_eq!(butterfly_damage(&req, &idle, &packet), fraction(2, 5));
    assert_eq!(butterfly_damage(&req, &active, &packet), fraction(11, 20));
    let packet = attachment_packet(&[attachment], Some(DamageElement::Cosmos));
    assert_eq!(butterfly_damage(&req, &idle, &packet), fraction(3, 20));
    assert_eq!(butterfly_damage(&req, &active, &packet), fraction(3, 10));
}
