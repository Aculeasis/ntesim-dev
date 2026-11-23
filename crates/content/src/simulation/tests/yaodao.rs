use super::*;
use crate::runtime_api::{DamageContext, HitModifiers};
use crate::simulation::events::GameEventData;
use crate::simulation::metadata::{
    ActionCategory, ActionMetadata, ActionPacketBinding, ActionPacketBindingKind,
    ActionPacketMetadata, DamageElement, InputActionRole, MetadataProvenance,
};
use crate::simulation::state::GameSimulatorState;
use ntesim_core::combat::DamageKind;
use serde_json::json;
use std::collections::BTreeMap;

fn t1_proc() -> GearProcConfig {
    GearProcConfig::YaodaoParry {
        parry_damage_ratio: Ratio {
            numerator: 2,
            denominator: 1,
        },
    }
}

fn projection_tables() -> BTreeMap<String, serde_json::Value> {
    super::support::weapon_projection_tables()
}

#[test]
fn yaodao_projection_binds_five_refinement_tiers() {
    let tables = projection_tables();
    let req = research_request();
    let mut actor = req.actors[0].clone();
    actor.character_id = "character:1072".into();
    actor.weapon_id = "weapon:fork_yaodao".into();
    actor.gear_procs.clear();
    let expected = [(1, 2, 1), (2, 23, 10), (3, 13, 5), (4, 29, 10), (5, 16, 5)];
    for (tier, numerator, denominator) in expected {
        actor.weapon_tier = tier;
        let report = crate::weapons::yaodao::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            if tier == 1 { &[] } else { &[0] },
        )
        .unwrap();
        let proc = &report["compiled_proc"];
        let ratio: Ratio = serde_json::from_value(proc["parry_damage_ratio"].clone()).unwrap();
        assert_eq!(
            ratio,
            Ratio {
                numerator,
                denominator
            },
            "tier {tier} parry damage curve"
        );
        assert_eq!(actor.gear_procs.len(), 1);
        assert_eq!(report["replaced_proc"].is_null(), tier == 1);
        assert_eq!(
            report["native_description"]["localized"],
            json!(
                "Deals additional Incantation DMG equal to <lv>{0}</> of ATK when the wearer triggers a Parry Attack."
            )
        );
    }
}

#[test]
fn yaodao_projection_rejects_group_mismatch_and_changed_units() {
    let tables = projection_tables();
    let req = research_request();
    let mut cross_group = req.actors[0].clone();
    cross_group.character_id = "character:1019".into();
    cross_group.weapon_id = "weapon:fork_yaodao".into();
    cross_group.weapon_tier = 1;
    cross_group.gear_procs.clear();
    assert!(
        crate::weapons::yaodao::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut cross_group,
            &[]
        )
        .unwrap_err()
        .contains("incompatible")
    );

    let mut mutated = tables.clone();
    let row = &mut mutated.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_yaodao_5"]
        ["DataList"];
    row[0]["bIsPercent"] = json!(false);
    let mut unit_actor = req.actors[0].clone();
    unit_actor.character_id = "character:1072".into();
    unit_actor.weapon_id = "weapon:fork_yaodao".into();
    unit_actor.weapon_tier = 5;
    unit_actor.gear_procs.clear();
    assert!(
        crate::weapons::yaodao::projection::project(
            &mutated,
            1,
            "ntesim-current",
            &mut unit_actor,
            &[]
        )
        .unwrap_err()
        .contains("unit hint")
    );
}

#[test]
fn yaodao_validate_config_rejects_mismatches() {
    let req = research_request();
    let good_actor = req.actors[0].clone();
    let limits = crate::execution::ExecutionLimits::default();
    let mut good = good_actor.clone();
    good.weapon_id = "weapon:fork_yaodao".into();
    let good_ctx = crate::gear::validation::ConfigContext {
        actor: &good,
        actor_index: 0,
        limits: &limits,
    };
    let validation = crate::weapons::yaodao::validate_config(&t1_proc(), &good_ctx).unwrap();
    assert_eq!(validation.kind, "yaodao_parry");

    let mut wrong_weapon = good_actor.clone();
    wrong_weapon.weapon_id = "weapon:fork_Whale".into();
    let wrong_ctx = crate::gear::validation::ConfigContext {
        actor: &wrong_weapon,
        actor_index: 0,
        limits: &limits,
    };
    let err = match crate::weapons::yaodao::validate_config(&t1_proc(), &wrong_ctx) {
        Ok(_) => panic!("wrong weapon must be rejected"),
        Err(err) => err,
    };
    assert!(err.contains("weapon:fork_yaodao"), "unexpected: {err}");

    let zero = GearProcConfig::YaodaoParry {
        parry_damage_ratio: Ratio::zero(),
    };
    let err = match crate::weapons::yaodao::validate_config(&zero, &good_ctx) {
        Ok(_) => panic!("zero ratio must be rejected"),
        Err(err) => err,
    };
    assert!(err.contains("positive"), "unexpected: {err}");

    let wrong_kind = GearProcConfig::WhaleAtk {
        atk_bonus: Ratio {
            numerator: 3,
            denominator: 25,
        },
        broken_damage_bonus: Ratio {
            numerator: 3,
            denominator: 25,
        },
        kill_heal: Ratio {
            numerator: 3,
            denominator: 10,
        },
        kill_heal_cooldown_us: 30_000_000,
    };
    let err = match crate::weapons::yaodao::validate_config(&wrong_kind, &good_ctx) {
        Ok(_) => panic!("wrong proc kind must be rejected"),
        Err(err) => err,
    };
    assert!(err.contains("wrong proc kind"), "unexpected: {err}");
}

fn parry_packet(tags: &[&str]) -> crate::simulation::metadata::PacketMetadata {
    crate::simulation::metadata::PacketMetadata {
        damage_effect_id: "test-parry".into(),
        native_ga_name: None,
        native_damage_type_ex: "EExecutionDamageTypeEX::DAMAGE_TYPE_INCANTATION".into(),
        native_damage_source_category: None,
        damage_kind: DamageKind::Elemental,
        element: Some(DamageElement::Incantation),
        fixed_crit_rate: None,
        unbal_value: None,
        fixed_crit_rate_provenance: None,
        unbal_value_provenance: None,
        semantic_tags: tags.iter().map(|tag| (*tag).to_owned()).collect(),
        semantic_provenance: None,
        source_evidence: None,
        provenance: crate::simulation::metadata::MetadataProvenance::AcceptedModelHypothesis {
            id: "test.parry_packet".into(),
            description: "Test-owned synthetic parry packet metadata.".into(),
        },
        packet_kind: Default::default(),
    }
}

fn parry_action_packet(tagged: bool) -> ActionPacketMetadata {
    let provenance = MetadataProvenance::AcceptedModelHypothesis {
        id: "test.parry_parent_packet_v1".into(),
        description: "Synthetic connected parent packet for Crimson Hexblade runtime tests.".into(),
    };
    let gameplay_tags = if tagged {
        vec![crate::weapons::yaodao::PARRY_ATTACK_TAG.to_owned()]
    } else {
        Vec::new()
    };
    let mut packet = parry_packet(&[]);
    packet.damage_effect_id = "test-parry-parent".into();
    ActionPacketMetadata {
        action: ActionMetadata {
            categories: std::collections::BTreeSet::from([ActionCategory::Skill]),
            gameplay_tags,
            ability_id: "test-parry".into(),
            native_ability_index: None,
            input_role: None,
            input_role_provenance: None,
            primary_slot: None,
            source_evidence: None,
            identity_evidence: None,
            provenance: provenance.clone(),
        },
        packet,
        binding: ActionPacketBinding {
            energy_group_id: None,
            kind: ActionPacketBindingKind::ResearchDeclared,
            initiating_ability_id: "test-parry".into(),
            packet_ga_name: None,
            provenance: Some(provenance),
        },
    }
}

fn settle_test_parent(
    req: &GameScenarioRequest,
    metadata: ActionPacketMetadata,
) -> (
    GameSimulatorState,
    crate::simulation::timer_queue::EventQueue,
    Vec<crate::simulation::GameHitReceipt>,
    Result<(), String>,
) {
    let mut state = GameSimulatorState::new(req);
    crate::registry::initialize_actor(&req.actors[0], &mut state.actors[0], req).unwrap();
    state.advance_unpaused_test_clock(1_000_000);
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    let mut hits = Vec::new();
    let mut rng = None;
    let mut action_metadata_ledger = Vec::new();
    let result = {
        let mut ctx = crate::simulation::context::EventContext {
            request: req,
            prepared_packets: &[],
            state: &mut state,
            queue: &mut queue,
            sequence: &mut sequence,
            trace: &mut trace,
            rng: &mut rng,
            hits: &mut hits,
            action_metadata_ledger: &mut action_metadata_ledger,
        };
        crate::simulation::hit::settle(
            &mut ctx,
            crate::simulation::hit::HitIdentity {
                parent_hit_index: None,
                command_index: 0,
                actor_index: 0,
                target_index: 0,
            },
            "test-parry".into(),
            "test-parry-parent".into(),
            Ratio {
                numerator: 1,
                denominator: 1,
            },
            NativeScalingStat::Atk,
            metadata,
        )
    };
    (state, queue, hits, result)
}

fn settle_test_child(
    req: &GameScenarioRequest,
    state: &mut GameSimulatorState,
    queue: &mut crate::simulation::timer_queue::EventQueue,
    hits: &mut Vec<crate::simulation::GameHitReceipt>,
) {
    let event = queue.pop().expect("tagged parent should enqueue a child");
    let GameEventData::HitDelivery {
        command_index,
        actor_index,
        target_index,
        ability_id,
        damage_effect_id,
        atk_rate,
        scaling_stat,
        metadata,
    } = event.data
    else {
        panic!("expected queued Hexblade HitDelivery")
    };
    assert!(crate::weapons::yaodao::is_separate_packet(&metadata));
    let mut declared_lookalike = (*metadata).clone();
    declared_lookalike.binding.kind = ActionPacketBindingKind::ResearchDeclared;
    assert!(!crate::weapons::yaodao::is_separate_packet(
        &declared_lookalike
    ));
    assert_eq!(event.time_us, 1_000_000);
    state.advance_unpaused_test_clock(event.time_us);
    let mut trace = Vec::new();
    let mut rng = None;
    let mut action_metadata_ledger = Vec::new();
    let mut sequence = 1;
    {
        let mut ctx = crate::simulation::context::EventContext {
            request: req,
            prepared_packets: &[],
            state,
            queue,
            sequence: &mut sequence,
            trace: &mut trace,
            rng: &mut rng,
            hits,
            action_metadata_ledger: &mut action_metadata_ledger,
        };
        crate::simulation::hit::settle(
            &mut ctx,
            crate::simulation::hit::HitIdentity {
                parent_hit_index: None,
                command_index,
                actor_index,
                target_index,
            },
            ability_id,
            damage_effect_id,
            atk_rate,
            scaling_stat,
            *metadata,
        )
        .unwrap();
    }
}

fn yaodao_modifiers(
    req: &GameScenarioRequest,
    buff_stats: &BTreeMap<String, Ratio>,
    packet: &crate::simulation::metadata::PacketMetadata,
) -> Result<HitModifiers, String> {
    let state = GameSimulatorState::new(req);
    let action_metadata = super::support::declared_test_action(
        "caller-test",
        crate::simulation::metadata::ActionCategory::Skill,
    );
    let owned_status_counts = BTreeMap::new();
    let ctx = DamageContext {
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
        damage_effect_id: "test-parry",
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
    let base = HitModifiers::new(ExactFraction::new(0, 1).unwrap()).unwrap();
    crate::weapons::yaodao::damage_bonus(&ctx, packet, base)
}

fn parry_request() -> GameScenarioRequest {
    let mut req = research_request();
    req.stochastic_crit = None;
    let actor = &mut req.actors[0];
    actor.character_id = "character:1072".into();
    actor.weapon_id = "weapon:fork_yaodao".into();
    actor.weapon_tier = 1;
    actor.gear_procs = vec![t1_proc()];
    actor.static_stats.base_atk = 1000;
    actor.static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    actor.static_stats.crit_rate = Ratio::zero();
    actor.static_stats.damage_bonus = Ratio::zero();
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req
}

#[test]
fn yaodao_validate_config_rejects_zero_denominator_ratio() {
    let req = research_request();
    let mut good = req.actors[0].clone();
    good.weapon_id = "weapon:fork_yaodao".into();
    let limits = crate::execution::ExecutionLimits::default();
    let good_ctx = crate::gear::validation::ConfigContext {
        actor: &good,
        actor_index: 0,
        limits: &limits,
    };
    let bad = GearProcConfig::YaodaoParry {
        parry_damage_ratio: Ratio {
            numerator: 1,
            denominator: 0,
        },
    };
    // A zero denominator never reaches the positivity check: it fails inside
    // ExactFraction::from_ratio, so any Err (not a specific message) is the pass.
    let err = match crate::weapons::yaodao::validate_config(&bad, &good_ctx) {
        Ok(_) => panic!("zero denominator must be rejected"),
        Err(err) => err,
    };
    assert!(!err.is_empty(), "rejection must carry a message");
}

#[test]
fn yaodao_no_proc_no_bonus_even_with_tag() {
    let mut req = parry_request();
    req.actors[0].gear_procs.clear();
    let buff_stats = BTreeMap::new();
    let tagged = parry_packet(&[crate::weapons::yaodao::PARRY_ATTACK_TAG]);
    let modified = yaodao_modifiers(&req, &buff_stats, &tagged).unwrap();
    assert_eq!(modified.ratio_add, ExactFraction::new(0, 1).unwrap());
}

#[test]
fn yaodao_packet_level_trigger_tag_is_rejected_when_equipped() {
    let req = parry_request();
    let buff_stats = BTreeMap::new();
    let tagged = parry_packet(&[crate::weapons::yaodao::PARRY_ATTACK_TAG]);
    let error = yaodao_modifiers(&req, &buff_stats, &tagged).unwrap_err();
    assert!(
        error.contains("action metadata"),
        "unexpected error: {error}"
    );
}

#[test]
fn yaodao_tagged_connected_packet_emits_separate_ratio_child() {
    for (tier, numerator, denominator, expected_damage) in [
        (1, 2, 1, 2_000),
        (2, 23, 10, 2_300),
        (3, 13, 5, 2_600),
        (4, 29, 10, 2_900),
        (5, 16, 5, 3_200),
    ] {
        let mut req = parry_request();
        req.actors[0].weapon_tier = tier;
        req.actors[0].gear_procs = vec![GearProcConfig::YaodaoParry {
            parry_damage_ratio: Ratio {
                numerator,
                denominator,
            },
        }];
        let (mut state, mut queue, mut hits, result) =
            settle_test_parent(&req, parry_action_packet(true));
        result.unwrap();
        assert_eq!(hits.len(), 1, "the parent receipt is recorded first");
        assert_eq!(hits[0].damage_effect_id, "test-parry-parent");
        assert_eq!(hits[0].damage, ExactFraction::new(1_000, 1).unwrap());
        let parent_receipt = serde_json::to_value(&hits[0]).unwrap();
        let parent_time_us = hits[0].time_us;
        let parent_actor_index = hits[0].actor_index;
        let parent_target_index = hits[0].target_index;

        settle_test_child(&req, &mut state, &mut queue, &mut hits);

        assert!(queue.is_empty());
        assert_eq!(hits.len(), 2);
        assert_eq!(
            serde_json::to_value(&hits[0]).unwrap(),
            parent_receipt,
            "the triggering receipt stays intact"
        );
        let child = &hits[1];
        assert_eq!(child.time_us, parent_time_us);
        assert_eq!(child.actor_index, parent_actor_index);
        assert_eq!(child.target_index, parent_target_index);
        assert_eq!(
            child.damage_effect_id,
            "weapon:fork_yaodao:parry_incantation"
        );
        assert_eq!(child.scaling_stat, NativeScalingStat::Atk);
        assert_eq!(
            child.scaling_rate,
            Ratio {
                numerator,
                denominator
            }
        );
        let child_meta = &state.metadata_catalog[child.metadata_id];
        assert_eq!(child_meta.packet.element, Some(DamageElement::Incantation));
        assert_eq!(
            child.damage,
            ExactFraction::new(expected_damage.into(), 1).unwrap()
        );
        assert_eq!(child.rounded_damage, expected_damage);
        assert_eq!(child.crit_sample, None);
        assert!(
            child
                .execution_model_assumption_ids
                .iter()
                .any(|id| id == crate::weapons::yaodao::PACKET_HYPOTHESIS_ID.as_str())
        );
        assert_eq!(
            state
                .receipt_model_assumptions
                .get(crate::weapons::yaodao::PACKET_HYPOTHESIS_ID.as_str())
                .map(String::as_str),
            Some(crate::weapons::yaodao::PACKET_HYPOTHESIS_ID.description())
        );
        assert_eq!(
            child_meta.packet.provenance,
            crate::simulation::metadata::NormalizedMetadataProvenance::AcceptedModelHypothesis {
                hypothesis_id: crate::weapons::yaodao::PACKET_HYPOTHESIS_ID.into(),
            }
        );
    }
}

#[test]
fn yaodao_requires_configured_proc_and_explicit_action_tag() {
    for (has_proc, tagged) in [(false, true), (true, false)] {
        let mut req = parry_request();
        if !has_proc {
            req.actors[0].gear_procs.clear();
        }
        let (state, queue, hits, result) = settle_test_parent(&req, parry_action_packet(tagged));
        result.unwrap();
        assert!(queue.is_empty());
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].damage_effect_id, "test-parry-parent");
        assert_eq!(state.hits_settled, 1);
    }
}

#[test]
fn yaodao_conflicting_packet_hypothesis_fails_before_parent_damage() {
    let mut conflicting_req = parry_request();
    conflicting_req.model_assumptions.insert(
        crate::weapons::yaodao::PACKET_HYPOTHESIS_ID.into(),
        "conflicting description".into(),
    );
    let (state, queue, hits, result) =
        settle_test_parent(&conflicting_req, parry_action_packet(true));
    let error = result.unwrap_err();
    assert!(error.contains("conflicting reached model hypothesis"));
    assert!(queue.is_empty());
    assert!(hits.is_empty());
    assert_eq!(state.rounded_damage_total, 0);
}

#[test]
fn yaodao_input_role_parry_attack_emits_separate_ratio_child() {
    let mut req = parry_request();
    req.actors[0].weapon_tier = 1;
    req.actors[0].gear_procs = vec![GearProcConfig::YaodaoParry {
        parry_damage_ratio: Ratio {
            numerator: 2,
            denominator: 1,
        },
    }];
    let mut parent = parry_action_packet(false);
    parent.action.input_role = Some(InputActionRole::ParryAttack);
    let (mut state, mut queue, mut hits, result) = settle_test_parent(&req, parent);
    result.unwrap();
    assert_eq!(hits.len(), 1, "parent hit settles first");
    settle_test_child(&req, &mut state, &mut queue, &mut hits);
    assert!(queue.is_empty());
    assert_eq!(hits.len(), 2);
    let child = &hits[1];
    assert_eq!(
        child.damage_effect_id,
        "weapon:fork_yaodao:parry_incantation"
    );
    assert_eq!(
        state.metadata_catalog[child.metadata_id].action.input_role,
        Some(InputActionRole::Other)
    );
    assert_eq!(child.rounded_damage, 2_000);
}

#[test]
fn yaodao_refinement_scaling_and_continuing_cycles_t1_through_t5() {
    let ratios = [
        (1, 2, 1, 2_000),
        (2, 23, 10, 2_300),
        (3, 13, 5, 2_600),
        (4, 29, 10, 2_900),
        (5, 16, 5, 3_200),
    ];
    for (tier, numerator, denominator, expected_child_damage) in ratios {
        let mut req = parry_request();
        req.actors[0].weapon_tier = tier;
        req.actors[0].gear_procs = vec![GearProcConfig::YaodaoParry {
            parry_damage_ratio: Ratio {
                numerator,
                denominator,
            },
        }];
        req.relative_duration_us = 40_000_000;
        req.model_assumptions.insert(
            crate::weapons::yaodao::PACKET_HYPOTHESIS_ID.as_str().into(),
            crate::weapons::yaodao::PACKET_HYPOTHESIS_ID
                .description()
                .into(),
        );
        req.model_assumptions.insert(
            crate::weapons::yaodao::HYPOTHESIS_ID.as_str().into(),
            crate::weapons::yaodao::HYPOTHESIS_ID.description().into(),
        );

        req.actions.clear();
        req.action_metadata.clear();

        for (i, (time_us, is_parry)) in [
            (0, true),
            (10_000_000, false),
            (20_000_000, true),
            (30_000_000, false),
        ]
        .into_iter()
        .enumerate()
        {
            req.actions.push(GameActionCommand::InvokeAbility {
                timestamp_us: time_us,
                actor_index: 0,
                ability_id: "caller-test".into(),
                array_index: 0,
                damage_effect_id: "caller-effect".into(),
                atk_rate: Ratio::one(),
                scaling_stat: NativeScalingStat::Atk,
                energy_cost: 0,
                recovery_us: 500_000,
                cooldown_us: 0,
                hit_delay_us: 0,
            });
            let meta = crate::simulation::ResearchActionMetadata {
                hypothesis_id: "test.synthetic_action".into(),
                ability_id: "caller-test".into(),
                damage_effect_id: "caller-effect".into(),
                action_gameplay_tags: if is_parry {
                    Some(vec![crate::weapons::yaodao::PARRY_ATTACK_TAG.into()])
                } else {
                    Some(vec!["Ability.Melee".into()])
                },
                input_action_role: if is_parry {
                    Some(InputActionRole::ParryAttack)
                } else {
                    Some(InputActionRole::Normal)
                },
                damage_type_ex: Some("EExecutionDamageTypeEX::DAMAGE_TYPE_INCANTATION".into()),
                damage_source_category: None,
                fixed_crit_rate: Some(crate::simulation::ResearchFixedCritRate::NoOverride),
                unbal_value: None,
                packet_ga_name_binding: None,
            };
            req.action_metadata.insert(i, meta);
        }

        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );

        let hits = report["hit_receipts"].as_array().unwrap();
        assert_eq!(hits.len(), 6);

        let child_hits: Vec<_> = hits
            .iter()
            .filter(|h| h["damage_effect_id"] == "weapon:fork_yaodao:parry_incantation")
            .collect();
        assert_eq!(child_hits.len(), 2);

        for (hit, &expected_time) in child_hits.iter().zip(&[0u64, 20_000_000u64]) {
            assert_eq!(hit["time_us"], expected_time);
            assert_eq!(hit["rounded_damage"], expected_child_damage);
            assert_eq!(
                hit["non_crit_damage"],
                json!(format!("{}/{}", expected_child_damage, 1))
            );
            let meta = super::support::hit_metadata(&report, hit);
            assert_eq!(meta["packet"]["element"], "incantation");
            assert_eq!(meta["binding"]["kind"], "owner_child");
        }
    }
}
