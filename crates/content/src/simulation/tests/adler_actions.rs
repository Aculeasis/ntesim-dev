use super::*;
use crate::HypothesisId;
use crate::simulation::tests::support::formula_damage_total;
use serde_json::json;

pub(super) fn make_action(
    timestamp_us: u64,
    actor_index: usize,
    ability_id: &str,
    damage_effect_id: &str,
    energy_cost: u64,
    cooldown_us: u64,
) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index,
        ability_id: ability_id.into(),
        damage_effect_id: damage_effect_id.into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost,
        recovery_us: 0,
        cooldown_us,
        hit_delay_us: 100,
    }
}

pub(super) fn request() -> GameScenarioRequest {
    let mut req = sample_s1_request();
    req.model_assumptions.insert(
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_ID
            .description()
            .into(),
    );
    req.actors[0].character_id = "character:1033".into();
    req.actors[0].weapon_id.clear();
    req.actors[0].gear_procs.clear();
    req.actors[0].set_id = None;
    req.actors[0].talent_levels = Some([1; 3]);
    req.actors[0].character_max_level = Some(80);
    req.actors[0].character_level = 80;
    req.model_assumptions.insert(
        crate::characters::adler::skill::SKILL_TRAIN_HYPOTHESIS_ID.into(),
        crate::characters::adler::skill::SKILL_TRAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::adler::skill::SKILL_ALL_IN_RANGE_HYPOTHESIS_ID.into(),
        crate::characters::adler::skill::SKILL_ALL_IN_RANGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::adler::skill::KARMA_HYPOTHESIS_ID.into(),
        crate::characters::adler::skill::KARMA_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::packet_semantics::ADLER_SKILL_DOT_PACKET_HYPOTHESIS_ID.into(),
        crate::characters::packet_semantics::ADLER_SKILL_DOT_PACKET_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.targets[0].resistance = Ratio::zero();
    req.targets[0].level = 80;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Adler_Melee",
        "GE_Player_Adler_Melee1_Damage",
        0,
        0,
    )];
    req
}

pub(super) fn def_action(
    timestamp_us: u64,
    ability_id: &str,
    damage_effect_id: &str,
    rate: Ratio,
) -> GameActionCommand {
    let mut action = make_action(timestamp_us, 0, ability_id, damage_effect_id, 0, 0);
    if let GameActionCommand::InvokeAbility {
        atk_rate,
        scaling_stat,
        ..
    } = &mut action
    {
        *atk_rate = rate;
        *scaling_stat = crate::simulation::NativeScalingStat::Def;
    }
    action
}

#[test]
fn adler_direct_skill_and_burst_use_def_independently_of_atk() {
    for (base_def, def_percent, def_flat, atk, expected_skill, expected_burst) in [
        (1000, Ratio::zero(), Ratio::zero(), 700, 200, 1000),
        (
            0,
            Ratio {
                numerator: 1,
                denominator: 5,
            },
            Ratio {
                numerator: 40,
                denominator: 1,
            },
            700,
            8,
            40,
        ),
        (
            1000,
            Ratio {
                numerator: 1,
                denominator: 5,
            },
            Ratio {
                numerator: 40,
                denominator: 1,
            },
            700,
            248,
            1240,
        ),
        (
            1000,
            Ratio {
                numerator: 1,
                denominator: 5,
            },
            Ratio {
                numerator: 40,
                denominator: 1,
            },
            7000,
            248,
            1240,
        ),
        (
            2000,
            Ratio {
                numerator: 1,
                denominator: 5,
            },
            Ratio {
                numerator: 40,
                denominator: 1,
            },
            700,
            488,
            2440,
        ),
    ] {
        let mut req = request();
        req.actors[0].static_stats.base_def = Some(base_def);
        req.actors[0].static_stats.def_percent = def_percent;
        req.actors[0].static_stats.def_flat = def_flat;
        req.actors[0].static_stats.def =
            base_def * (def_percent.denominator + def_percent.numerator) / def_percent.denominator
                + def_flat.numerator / def_flat.denominator;
        req.actors[0].static_stats.atk = Ratio {
            numerator: atk,
            denominator: 1,
        };
        req.targets[0].defense = 0;
        req.targets[0].resistance = Ratio::zero();
        req.actions = vec![
            def_action(
                0,
                "GA_Adler_Skill",
                "GE_Player_Adler_Skill2_Damage",
                Ratio {
                    numerator: 1,
                    denominator: 5,
                },
            ),
            def_action(
                1_000_000,
                "GA_Adler_UltraSkill",
                "GE_Player_Adler_UltraSkill_Damage",
                Ratio::one(),
            ),
        ];
        req.model_assumptions.insert(
            crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID
                .as_str()
                .into(),
            crate::simulation::metadata::FIXED_CRIT_PROBABILITY_ID
                .description()
                .into(),
        );
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 23);
        assert_eq!(
            report["hit_receipts"][0]["damage_effect_id"],
            "GE_Player_Adler_Skill2_Damage"
        );
        let burst_packet = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|hit| hit["ability_id"] == "GA_Adler_UltraSkill")
            .unwrap();
        assert_eq!(
            burst_packet["damage_effect_id"],
            "GE_Player_Adler_UltraSkill_Damage"
        );
        assert_eq!(report["hit_receipts"][0]["scaling_stat"], "def");
        assert_eq!(report["hit_receipts"][1]["scaling_stat"], "def");
        assert_eq!(
            report["hit_receipts"][0]["rounded_damage"],
            expected_skill * 7 / 2
        );
        let burst_hit = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|hit| hit["ability_id"] == "GA_Adler_UltraSkill")
            .unwrap();
        assert_eq!(burst_hit["rounded_damage"], expected_burst);
    }
}

#[test]
fn adler_burst_counts_living_targets_and_uses_effective_def_per_strike() {
    for (level, awakening, coefficient, expected_per_hit) in [
        (1, 0, (1, 1), (1240, 1)),
        (8, 0, (857, 500), (53134, 25)),
        (8, 3, (1851, 1000), (57381, 25)),
    ] {
        for (initial_hp, expected_counts) in [
            (vec![100_000], vec![10]),
            (vec![100_000, 100_000], vec![5, 5]),
            (vec![1, 100_000], vec![1, 5]),
            (vec![0, 100_000, 0], vec![0, 10, 0]),
            (vec![100_000, 0, 100_000], vec![5, 0, 5]),
            (vec![0, 0], vec![0, 0]),
        ] {
            let mut req = request();
            req.actors[0].talent_levels = Some([1, 1, level]);
            req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
            req.actors[0].static_stats.base_def = Some(1000);
            req.actors[0].static_stats.def_percent = Ratio {
                numerator: 1,
                denominator: 5,
            };
            req.actors[0].static_stats.def_flat = Ratio {
                numerator: 40,
                denominator: 1,
            };
            req.actors[0].static_stats.def = 1240;
            req.actors[0].initial_energy = 100;
            let mut ally = req.actors[0].clone();
            ally.slot = 1;
            ally.character_id = "character:1019".into();
            req.actors.push(ally);
            let template = req.targets[0].clone();
            req.targets = initial_hp
                .iter()
                .enumerate()
                .map(|(index, hp)| {
                    let mut target = template.clone();
                    target.target_index = index;
                    target.hp_max = 100_000;
                    target.hp_current = *hp;
                    target.defense = 0;
                    target
                })
                .collect();
            req.actions = vec![make_action(
                0,
                0,
                "GA_Adler_UltraSkill",
                "GE_Player_Adler_UltraSkill_Damage",
                100,
                20_000_000,
            )];
            req.actions.push(GameActionCommand::SwitchActor {
                timestamp_us: 50,
                target_actor_index: 1,
                cooldown_us: 0,
            });
            let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
            assert_eq!(
                report["execution_completed"], true,
                "{} {}",
                report["halt_diagnostic"], report["rejected_action"]
            );
            let hits = report["hit_receipts"].as_array().unwrap();
            let mut counts = vec![0; req.targets.len()];
            for hit in hits {
                assert_eq!(hit["actor_index"], 0);
                assert_eq!(hit["time_us"], 100);
                counts[hit["target_index"].as_u64().unwrap() as usize] += 1;
                assert_eq!(hit["scaling_stat"], "def");
                assert_eq!(
                    hit["scaling_rate"],
                    json!(Ratio {
                        numerator: coefficient.0,
                        denominator: coefficient.1
                    })
                );
                assert_eq!(
                    hit["non_crit_damage"],
                    json!(format!("{}/{}", expected_per_hit.0, expected_per_hit.1))
                );
            }
            assert_eq!(counts, expected_counts);
            assert_eq!(report["final_buff_state"]["active_actor_index"], 1);
            let expected_total = ExactFraction::new(expected_per_hit.0, expected_per_hit.1)
                .unwrap()
                .mul(ExactFraction::new(expected_counts.iter().sum::<u128>(), 1).unwrap())
                .unwrap();
            assert_eq!(
                serde_json::json!(formula_damage_total(&report)),
                json!(expected_total)
            );
            assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
            assert_eq!(
                report["resolved_model_assumptions"]
                    [crate::characters::adler::skill::BURST_HYPOTHESIS_ID.as_str()],
                crate::characters::adler::skill::BURST_HYPOTHESIS_ID.description()
            );
        }
    }
}

#[test]
fn adler_burst_uses_population_after_earlier_hits_and_keeps_pending_train() {
    let mut req = request();
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def = 1000;
    req.targets[0].hp_current = 1;
    req.targets[0].defense = 0;
    let mut survivor = req.targets[0].clone();
    survivor.target_index = 1;
    survivor.hp_max = 100_000;
    survivor.hp_current = 100_000;
    req.targets.push(survivor);
    req.actions = vec![def_action(
        0,
        "GA_Adler_Skill",
        "GE_Player_Adler_Skill2_Damage",
        Ratio::one(),
    )];
    req.relative_duration_us = 200;
    let skill_report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let skill_hits = skill_report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["ability_id"] == "GA_Adler_Skill")
        .collect::<Vec<_>>();
    assert_eq!(skill_hits.len(), 4);
    assert_eq!(
        skill_hits
            .iter()
            .filter(|hit| hit["target_index"] == 0)
            .count(),
        1
    );
    assert_eq!(
        skill_hits
            .iter()
            .filter(|hit| hit["target_index"] == 1)
            .count(),
        3
    );
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Adler_Melee",
            "GE_Player_Adler_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            101,
            0,
            "GA_Adler_UltraSkill",
            "GE_Player_Adler_UltraSkill_Damage",
            100,
            20_000_000,
        ),
    ];
    req.relative_duration_us = 200;
    req.absolute_limit_us = 200;
    let pending = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(pending["execution_completed"], false);
    assert_eq!(pending["pending_events"], 12);
    assert_eq!(pending["hits_settled"], 1);
    req.relative_duration_us = 201;
    req.absolute_limit_us = 3_600_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 11);
    for hit in &report["hit_receipts"].as_array().unwrap()[1..] {
        assert_eq!(hit["target_index"], 1);
        assert_eq!(hit["rounded_damage"], 1000);
    }
}

#[test]
fn legacy_atk_ignores_live_def_percent_while_legacy_def_requires_base_provenance() {
    use crate::simulation::state::{GameSimulatorState, TimedBuffDeclaration};
    use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope};

    let mut req = request();
    req.actors[0].static_stats.base_def = None;
    req.actors[0].static_stats.def = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.model_assumptions.insert(
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_ID
            .description()
            .into(),
    );
    let tables = ntesim_data::skill_levels::frozen::tables();
    let atk_metadata = crate::simulation::metadata::resolve_requested_action_packet(
        &req.content_id,
        tables,
        "character:1033",
        "GA_Adler_Melee",
        "GE_Player_Adler_Melee1_Damage",
        None,
        &req.model_assumptions,
    )
    .unwrap();
    let def_metadata = crate::simulation::metadata::resolve_requested_action_packet(
        &req.content_id,
        tables,
        "character:1033",
        "GA_Adler_Skill",
        "GE_Player_Adler_Skill2_Damage",
        None,
        &req.model_assumptions,
    )
    .unwrap();
    let prepared_packets = vec![Some(atk_metadata.clone()), Some(def_metadata.clone())];
    let mut state = GameSimulatorState::new(&req);
    state
        .apply_buff(
            0,
            TimedBuffDeclaration::reset(
                "test.legacy_def_percent",
                "character:1033",
                BuffRecipient::TeamAll,
                100,
                BuffStackingScope::PerSource,
                BTreeMap::from([("def_percent".into(), ExactFraction::new(1, 5).unwrap())]),
            ),
        )
        .unwrap();
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    let mut rng = None;
    let mut hits = Vec::new();
    let mut ledger = Vec::new();
    let mut ctx = crate::simulation::context::EventContext {
        request: &req,
        prepared_packets: &prepared_packets,
        state: &mut state,
        queue: &mut queue,
        sequence: &mut sequence,
        trace: &mut trace,
        rng: &mut rng,
        hits: &mut hits,
        action_metadata_ledger: &mut ledger,
    };

    crate::simulation::hit::settle(
        &mut ctx,
        crate::simulation::hit::HitIdentity {
            parent_hit_index: None,
            command_index: 0,
            actor_index: 0,
            target_index: 0,
        },
        "GA_Adler_Melee".into(),
        "GE_Player_Adler_Melee1_Damage".into(),
        Ratio {
            numerator: 201,
            denominator: 1000,
        },
        crate::simulation::NativeScalingStat::Atk,
        atk_metadata,
    )
    .unwrap();
    assert_eq!(ctx.hits.last().unwrap().rounded_damage, 201);

    let error = crate::simulation::hit::settle(
        &mut ctx,
        crate::simulation::hit::HitIdentity {
            parent_hit_index: None,
            command_index: 1,
            actor_index: 0,
            target_index: 0,
        },
        "GA_Adler_Skill".into(),
        "GE_Player_Adler_Skill2_Damage".into(),
        Ratio {
            numerator: 1,
            denominator: 5,
        },
        crate::simulation::NativeScalingStat::Def,
        def_metadata,
    )
    .unwrap_err();
    assert_eq!(error, "DEF percent requires a native base-DEF projection");
    assert_eq!(ctx.hits.len(), 1);
}

#[test]
fn adler_delayed_def_packets_keep_off_field_owner_at_horizon_and_skip_defeated_target() {
    let mut req = request();
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def_percent = Ratio {
        numerator: 1,
        denominator: 5,
    };
    req.actors[0].static_stats.def_flat = Ratio {
        numerator: 40,
        denominator: 1,
    };
    req.actors[0].static_stats.def = 1240;
    req.targets[0].hp_max = 1500;
    req.targets[0].hp_current = 1500;
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    req.actors.push(ally);
    req.actions = vec![
        def_action(
            0,
            "GA_Adler_Skill",
            "GE_Player_Adler_Skill2_Damage",
            Ratio::one(),
        ),
        def_action(
            1_000_101,
            "GA_Adler_UltraSkill",
            "GE_Player_Adler_UltraSkill_Damage",
            Ratio::one(),
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 50,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    req.model_assumptions.insert(
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_ID
            .description()
            .into(),
    );

    req.actions.swap(1, 2);
    // Return for the later Burst, then leave again. Both casts are admitted
    // while their owner is on field; the delayed packets retain that owner.
    req.actions.insert(
        2,
        GameActionCommand::SwitchActor {
            timestamp_us: 1_000_101,
            target_actor_index: 0,
            cooldown_us: 0,
        },
    );
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 1_000_151,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    req.relative_duration_us = 99;
    let pending = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(pending["execution_completed"], true);
    assert_eq!(pending["pending_events"], 7); // Four Skill packets, the Burst and two switches.
    assert_eq!(pending["hits_settled"], 0);

    req.relative_duration_us = 10_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["pending_events"], 0);
    assert_eq!(report["hits_settled"], 2);
    assert_eq!(report["final_buff_state"]["active_actor_index"], 1);
    let receipts = report["hit_receipts"].as_array().unwrap();
    assert_eq!(receipts.len(), 2);
    for receipt in receipts {
        assert_eq!(receipt["actor_index"], 0);
        assert_eq!(receipt["scaling_stat"], "def");
    }
    assert_eq!(
        receipts[0]["damage_effect_id"],
        "GE_Player_Adler_Skill2_Damage"
    );
    assert_eq!(
        receipts[1]["damage_effect_id"],
        "GE_Player_Adler_Skill3_Damage"
    );
    assert_eq!(receipts[0]["target_index"], 0);
    assert_eq!(receipts[0]["scaling_rate"], json!("7/10"));
    assert_eq!(receipts[0]["rounded_damage"], 868);
    assert_eq!(receipts[0]["applied_hp_loss"], 868);
    assert_eq!(receipts[1]["target_index"], 0);
    assert_eq!(
        receipts[1]["damage_effect_id"],
        "GE_Player_Adler_Skill3_Damage"
    );
    assert_eq!(receipts[1]["rounded_damage"], 868);
    assert_eq!(receipts[1]["applied_hp_loss"], 632);

    req.targets[0].hp_current = 100;
    req.targets[0].hp_max = 100;
    let dead_target_report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(dead_target_report["hits_settled"], 1);
    assert_eq!(
        dead_target_report["hit_receipts"][0]["damage_effect_id"],
        "GE_Player_Adler_Skill2_Damage"
    );
    assert!(
        dead_target_report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| {
                event["event_kind"] == "hit_target_unavailable"
                    && event["data"]["damage_effect_id"] == "GE_Player_Adler_UltraSkill_Damage"
            })
    );
}

#[test]
fn adler_def_scaling_reads_live_base_percent_and_flat_buffs_until_expiry() {
    use crate::simulation::state::{GameSimulatorState, TimedBuffDeclaration};
    use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope};

    let mut req = request();
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def = 1000;
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    let mut second_target = req.targets[0].clone();
    second_target.target_index = 1;
    req.targets.push(second_target);
    req.actions = vec![def_action(
        0,
        "GA_Adler_Skill",
        "GE_Player_Adler_Skill_Damage",
        Ratio {
            numerator: 1,
            denominator: 5,
        },
    )];
    req.model_assumptions.insert(
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_ID
            .description()
            .into(),
    );

    let metadata = crate::simulation::metadata::resolve_requested_action_packet(
        &req.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        "character:1033",
        "GA_Adler_Skill",
        "GE_Player_Adler_Skill_Damage",
        None,
        &req.model_assumptions,
    )
    .unwrap();
    let prepared_packets = vec![Some(metadata.clone())];
    let mut state = GameSimulatorState::new(&req);
    state
        .apply_buff(
            0,
            TimedBuffDeclaration::reset(
                "test.adler_def",
                "character:1033",
                BuffRecipient::TeamAll,
                5,
                BuffStackingScope::PerSource,
                BTreeMap::from([
                    ("def_percent".into(), ExactFraction::new(1, 5).unwrap()),
                    ("def_add".into(), ExactFraction::new(50, 1).unwrap()),
                ]),
            ),
        )
        .unwrap();
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    let mut rng = None;
    let mut hits = Vec::new();
    let mut ledger = Vec::new();
    let mut ctx = crate::simulation::context::EventContext {
        request: &req,
        prepared_packets: &prepared_packets,
        state: &mut state,
        queue: &mut queue,
        sequence: &mut sequence,
        trace: &mut trace,
        rng: &mut rng,
        hits: &mut hits,
        action_metadata_ledger: &mut ledger,
    };

    for (time_us, target_index, expected_damage) in [(0, 0, 250), (5, 1, 200)] {
        ctx.state.advance_unpaused_test_clock(time_us);
        ctx.state.buffs.advance_time(time_us);
        crate::simulation::hit::settle(
            &mut ctx,
            crate::simulation::hit::HitIdentity {
                parent_hit_index: None,
                command_index: 0,
                actor_index: 0,
                target_index,
            },
            "GA_Adler_Skill".into(),
            "GE_Player_Adler_Skill_Damage".into(),
            Ratio {
                numerator: 1,
                denominator: 5,
            },
            crate::simulation::NativeScalingStat::Def,
            metadata.clone(),
        )
        .unwrap();
        assert_eq!(ctx.hits.last().unwrap().rounded_damage, expected_damage);
    }
}

#[test]
fn adler_normal_uses_native_talent_coefficients_and_resonance_bonus() {
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 201, 2),
        (8, 0, 172, 1),
        (8, 1, 172, 1),
        (8, 3, 186, 1),
        (8, 4, 186, 1),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([level, 1, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 1);
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            json!(format!("{}/{}", expected_num, expected_den))
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"
        );
    }
}

#[test]
fn adler_normal_chain_settles_twelve_hits_and_loops_at_levels_one_and_eight() {
    for (level, awakening, expected_5_num, expected_5_den, expected_6_num, expected_6_den) in [
        (1, 0, 3181, 2, 1691, 1),
        (8, 0, 2726, 1, 2898, 1),
        (8, 3, 2944, 1, 3130, 1),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([level, 1, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = (0..5)
            .map(|index| {
                make_action(
                    index * 200_000,
                    0,
                    "GA_Adler_Melee",
                    "GE_Player_Adler_Melee1_Damage",
                    0,
                    0,
                )
            })
            .collect();
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 12);
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            json!(format!("{}/{}", expected_5_num, expected_5_den))
        );

        let receipts = report["hit_receipts"].as_array().unwrap();
        assert_eq!(
            receipts[0]["damage_effect_id"],
            "GE_Player_Adler_Melee1_Damage"
        );
        assert_eq!(
            receipts[1]["damage_effect_id"],
            "GE_Player_Adler_Melee2_Damage"
        );
        for item in &receipts[2..9] {
            assert_eq!(item["damage_effect_id"], "GE_Player_Adler_Melee3_Damage");
        }
        assert_eq!(
            receipts[9]["damage_effect_id"],
            "GE_Player_Adler_Melee4_1_Damage"
        );
        assert_eq!(
            receipts[10]["damage_effect_id"],
            "GE_Player_Adler_Melee4_2_Damage"
        );
        assert_eq!(
            receipts[11]["damage_effect_id"],
            "GE_Player_Adler_Melee5_Damage"
        );

        req.actions.push(make_action(
            1_000_000,
            0,
            "GA_Adler_Melee",
            "GE_Player_Adler_Melee1_Damage",
            0,
            0,
        ));
        let loop_report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(loop_report["execution_completed"], true);
        assert_eq!(loop_report["hits_settled"], 13);
        assert_eq!(
            serde_json::json!(formula_damage_total(&loop_report)),
            json!(format!("{}/{}", expected_6_num, expected_6_den))
        );
        assert_eq!(
            loop_report["hit_receipts"][12]["damage_effect_id"],
            "GE_Player_Adler_Melee1_Damage"
        );
    }
}

#[test]
fn adler_normal_chain_resets_on_ability_gap_and_field_exit() {
    let mut req = request();
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Adler_Melee",
            "GE_Player_Adler_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            200_000,
            0,
            "GA_Adler_QTE",
            "GE_Player_Adler_QTE_Damage",
            0,
            0,
        ),
        make_action(
            400_000,
            0,
            "GA_Adler_Melee",
            "GE_Player_Adler_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 3);
    assert_eq!(
        report["hit_receipts"][0]["damage_effect_id"],
        "GE_Player_Adler_Melee1_Damage"
    );
    assert_eq!(
        report["hit_receipts"][1]["damage_effect_id"],
        "GE_Player_Adler_QTE_Damage"
    );
    assert_eq!(
        report["hit_receipts"][2]["damage_effect_id"],
        "GE_Player_Adler_Melee1_Damage"
    );

    let mut gap_req = request();
    gap_req.actions = vec![
        make_action(
            0,
            0,
            "GA_Adler_Melee",
            "GE_Player_Adler_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            2_100_000,
            0,
            "GA_Adler_Melee",
            "GE_Player_Adler_Melee1_Damage",
            0,
            0,
        ),
    ];
    let gap_report = run_game_scenario(&gap_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(gap_report["hits_settled"], 2);
    assert_eq!(
        gap_report["hit_receipts"][0]["damage_effect_id"],
        "GE_Player_Adler_Melee1_Damage"
    );
    assert_eq!(
        gap_report["hit_receipts"][1]["damage_effect_id"],
        "GE_Player_Adler_Melee1_Damage"
    );

    let mut switch_req = request();
    switch_req.actions = vec![
        make_action(
            0,
            0,
            "GA_Adler_Melee",
            "GE_Player_Adler_Melee1_Damage",
            0,
            0,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 100_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        make_action(
            200_000,
            0,
            "GA_Adler_Melee",
            "GE_Player_Adler_Melee1_Damage",
            0,
            0,
        ),
    ];
    let switch_report = run_game_scenario(&switch_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(switch_report["hits_settled"], 2);
    assert_eq!(
        switch_report["hit_receipts"][1]["damage_effect_id"],
        "GE_Player_Adler_Melee2_Damage"
    );
}

#[test]
fn adler_normal_chain_acceptance() {
    let mut strict_req = request();
    strict_req.actors[0].talent_levels = None;
    strict_req.actors[0].purchased_skill_level = 1;
    strict_req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    strict_req.model_assumptions.insert(
        crate::characters::adler::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::adler::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    strict_req.model_assumptions.insert(
        crate::simulation::energy::ADLER_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::ADLER_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&strict_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 1);
}

#[test]
fn adler_riposte_from_critical_dodge_consumed_once_and_grants_energy() {
    let mut req = request();
    req.combat_stamina = Some(crate::simulation::CombatStaminaProfile {
        capacity: Ratio {
            numerator: 60,
            denominator: 1,
        },
        initial: Ratio {
            numerator: 60,
            denominator: 1,
        },
        recovery_rate: Ratio::zero(),
        use_efficiency: Ratio::one(),
    });
    req.model_assumptions.insert(
        crate::characters::adler::riposte::HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::characters::adler::riposte::HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actors[0].talent_levels = Some([8, 1, 1]);
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].initial_energy = 0;
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.actions = vec![
        GameActionCommand::PlayerInput {
            timestamp_us: 100_000,
            actor_index: 0,
            input: PlayerInput::Dash,
            critical_success: true,
        },
        make_action(
            600_000,
            0,
            "GA_Adler_Melee",
            "GE_Player_Adler_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            2_000_000,
            0,
            "GA_Adler_Melee",
            "GE_Player_Adler_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 2);
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(
        hits[0]["damage_effect_id"],
        "GE_Player_Adler_PerfectEvadeAttack_Damage"
    );
    assert_eq!(
        super::support::hit_metadata(&report, &hits[0])["packet"]["native_damage_source_category"],
        "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Z"
    );
    assert_eq!(hits[0]["non_crit_damage"], json!("3016/1"));
    assert_eq!(hits[1]["damage_effect_id"], "GE_Player_Adler_Melee1_Damage");
    assert_eq!(report["final_actor_states"][0]["energy"], 5);
}

#[test]
fn adler_plunge_damage_category_energy_and_karma() {
    let mut req = request();
    req.model_assumptions.insert(
        crate::characters::adler::plunge::HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::characters::adler::plunge::HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actors[0].talent_levels = Some([8, 1, 1]);
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].initial_energy = 0;
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.actions = vec![make_action(
        0,
        0,
        "GA_Adler_Melee",
        "GE_Player_Adler_AirAttack_Damage",
        0,
        0,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 1);
    let hit = &report["hit_receipts"][0];
    assert_eq!(hit["damage_effect_id"], "GE_Player_Adler_AirAttack_Damage");
    assert_eq!(
        super::support::hit_metadata(&report, hit)["packet"]["native_damage_source_category"],
        "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_X"
    );
    assert_eq!(hit["non_crit_damage"], json!("617/1"));
    assert_eq!(report["final_actor_states"][0]["energy"], 2);
}

#[test]
fn adler_thirteen_packet_normal_chain_under_hypothesis() {
    let mut req = request();
    req.model_assumptions.insert(
        crate::characters::adler::skill::CHAIN_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::characters::adler::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        HypothesisId::AdlerNormalChain13V1.as_str().into(),
        HypothesisId::AdlerNormalChain13V1.description().into(),
    );
    req.actors[0].talent_levels = Some([8, 1, 1]);
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.actions = (0..5)
        .map(|index| {
            make_action(
                index * 200_000,
                0,
                "GA_Adler_Melee",
                "GE_Player_Adler_Melee1_Damage",
                0,
                0,
            )
        })
        .collect();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 13);
    let receipts = report["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts[0]["damage_effect_id"],
        "GE_Player_Adler_Melee1_Damage"
    );
    assert_eq!(
        receipts[1]["damage_effect_id"],
        "GE_Player_Adler_Melee2_Damage"
    );
    for item in &receipts[2..9] {
        assert_eq!(item["damage_effect_id"], "GE_Player_Adler_Melee3_Damage");
    }
    assert_eq!(
        receipts[9]["damage_effect_id"],
        "GE_Player_Adler_Melee4_1_Damage"
    );
    assert_eq!(
        receipts[10]["damage_effect_id"],
        "GE_Player_Adler_Melee4_2_Damage"
    );
    assert_eq!(
        receipts[11]["damage_effect_id"],
        "GE_Player_Adler_Melee4_3_Damage"
    );
    assert_eq!(
        receipts[12]["damage_effect_id"],
        "GE_Player_Adler_Melee5_Damage"
    );
    assert_eq!(
        serde_json::json!(formula_damage_total(&report)),
        json!("6731/1")
    );
}

#[test]
fn adler_burst_interval_volleys_and_callbacks() {
    let mut req = request();
    req.model_assumptions.insert(
        HypothesisId::AdlerBurstIntervalsV1.as_str().into(),
        HypothesisId::AdlerBurstIntervalsV1.description().into(),
    );
    req.actors[0].talent_levels = Some([1, 1, 8]);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([1, 5]).unwrap();
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def = 1000;
    req.actors[0].initial_energy = 120;
    req.actors[0].energy_capacity = 120;
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.relative_duration_us = 5_000_000;
    req.actions = vec![make_action(
        1_000_000,
        0,
        "GA_Adler_UltraSkill",
        "GE_Player_Adler_UltraSkill_Damage",
        120,
        20_000_000,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 10);
    let hits = report["hit_receipts"].as_array().unwrap();
    for hit in &hits[0..5] {
        assert_eq!(hit["time_us"], 3_185_103);
    }
    for hit in &hits[5..10] {
        assert_eq!(hit["time_us"], 3_343_074);
    }
    assert_eq!(report["final_actor_states"][0]["energy"], 20);
    let a5_event = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["event_kind"] == "adler_a5_energy_refund")
        .unwrap();
    assert_eq!(a5_event["time_us"], 3_343_074);
    let a1_event = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["event_kind"] == "adler_a1_karma_grant")
        .unwrap();
    assert_eq!(a1_event["time_us"], 3_343_074);
}

#[test]
fn adler_righteous_heart_b4_passive_def_bonus() {
    let mut req = request();
    req.model_assumptions.insert(
        HypothesisId::AdlerRighteousHeartV1.as_str().into(),
        HypothesisId::AdlerRighteousHeartV1.description().into(),
    );
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def = 1000;
    req.actors[0].talent_levels = Some([1, 1, 1]);
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.actions = vec![make_action(
        0,
        0,
        "GA_Adler_UltraSkill",
        "GE_Player_Adler_UltraSkill_Damage",
        0,
        0,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hit = &report["hit_receipts"][0];
    assert_eq!(hit["non_crit_damage"], json!("1200/1"));
}

#[test]
fn adler_blessing_party_shield_interception_on_switch() {
    let mut req = request();
    req.model_assumptions.insert(
        crate::characters::adler::skill::BLESSING_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::characters::adler::skill::BLESSING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        HypothesisId::AdlerBlessingPartyInterceptionV1
            .as_str()
            .into(),
        HypothesisId::AdlerBlessingPartyInterceptionV1
            .description()
            .into(),
    );
    req.actors[0].talent_levels = Some([1, 8, 1]);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([6]).unwrap();
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def = 1000;
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    req.actors.push(ally);
    req.relative_duration_us = 10_500_000;
    req.actions = vec![
        def_action(
            0,
            "GA_Adler_Skill",
            "GE_Player_Adler_Skill2_Damage",
            Ratio {
                numerator: 1,
                denominator: 1,
            },
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 1_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let actors = report["final_actor_states"].as_array().unwrap();
    assert!(actors[0].get("active_shields").is_none());
    let ally_shields = actors[1]["active_shields"].as_array().unwrap();
    assert_eq!(ally_shields.len(), 1);
    assert_eq!(ally_shields[0]["id"], "adler.blessing:0");
    assert_eq!(ally_shields[0]["remaining_hp"], 3084);
}

#[test]
fn adler_two_continuing_cycles_preserve_resources_and_state() {
    let mut req = request();
    req.model_assumptions.insert(
        crate::characters::adler::skill::CHAIN_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::characters::adler::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        HypothesisId::AdlerNormalChain13V1.as_str().into(),
        HypothesisId::AdlerNormalChain13V1.description().into(),
    );
    req.model_assumptions.insert(
        HypothesisId::AdlerBurstIntervalsV1.as_str().into(),
        HypothesisId::AdlerBurstIntervalsV1.description().into(),
    );
    req.model_assumptions.insert(
        HypothesisId::AdlerRighteousHeartV1.as_str().into(),
        HypothesisId::AdlerRighteousHeartV1.description().into(),
    );
    req.actors[0].talent_levels = Some([8, 8, 8]);
    req.actors[0].character_level = 80;
    req.actors[0].character_max_level = Some(80);
    req.actors[0].static_stats.base_atk = 832;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1252,
        denominator: 1,
    };
    req.actors[0].static_stats.base_def = Some(994);
    req.actors[0].static_stats.def = 3186;
    req.actors[0].static_stats.def_percent = Ratio {
        numerator: 15,
        denominator: 100,
    };
    req.actors[0].initial_energy = 0;
    req.actors[0].energy_capacity = 120;
    req.targets[0].defense = 1080;
    req.targets[0].resistance = Ratio {
        numerator: 20,
        denominator: 100,
    };
    req.targets[0].hp_max = 24_000_000;
    req.targets[0].hp_current = 24_000_000;
    req.relative_duration_us = 120_000_000;

    let mut actions = Vec::new();
    // 2 cycles, 60s each
    for cycle in 0..2 {
        let base_us = cycle * 60_000_000;
        // 5 Normal chains (5 * 6.0317295s = 30.1586475s)
        for chain in 0..5 {
            let chain_start = base_us + chain * 6_031_729;
            for stage in 0..5 {
                actions.push(make_action(
                    chain_start + stage * 200_000,
                    0,
                    "GA_Adler_Melee",
                    "GE_Player_Adler_Melee1_Damage",
                    0,
                    0,
                ));
            }
        }
        // At 32s: Skill
        actions.push(def_action(
            base_us + 32_000_000,
            "GA_Adler_Skill",
            "GE_Player_Adler_Skill2_Damage",
            Ratio {
                numerator: 12,
                denominator: 10,
            },
        ));
        // At 34s: Burst (120 cost, 20s CD)
        actions.push(make_action(
            base_us + 34_000_000,
            0,
            "GA_Adler_UltraSkill",
            "GE_Player_Adler_UltraSkill_Damage",
            120,
            20_000_000,
        ));
        // Sixth Normal chain overlapping ticks at 40s
        for stage in 0..5 {
            actions.push(make_action(
                base_us + 40_000_000 + stage * 200_000,
                0,
                "GA_Adler_Melee",
                "GE_Player_Adler_Melee1_Damage",
                0,
                0,
            ));
        }
    }
    req.actions = actions;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // 101 packets per cycle * 2 cycles = 202 hits!
    assert_eq!(report["hits_settled"], 202);
    // Total energy spent: 120 * 2 = 240
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 240);
    // Overflow cumulative = 37 (6 in period 1 + 31 in period 2)
    assert_eq!(report["final_actor_states"][0]["overflow_energy"], 37);
    // Retained energy = 25 (with 0.18 fractional carry)
    assert_eq!(report["final_actor_states"][0]["energy"], 25);
    // Final Karma = 13 (after consuming 20 at 92s, 6th chain adds 13)
    let state = &report["final_actor_states"][0];
    let pool_summary = state["pool_summary"].as_object();
    if let Some(karma) = pool_summary.and_then(|pool| pool.get("karma")) {
        assert_eq!(karma["current"], 13);
    }
}
