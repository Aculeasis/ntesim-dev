use super::*;
use serde_json::{Value, json};

fn actor(req: &mut GameScenarioRequest, slot: usize, character_id: &str, cycle: u64, ci: u64) {
    let mut base = req.actors[0].clone();
    base.slot = slot;
    base.character_id = character_id.into();
    base.initial_cycle = cycle;
    base.static_stats.cycle_intensity = Ratio {
        numerator: ci,
        denominator: 1,
    };
    base.static_stats.crit_damage_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };
    base.gear_procs.clear();
    if slot == 0 {
        req.actors[0] = base;
    } else {
        while (slot + 1) > req.actors.len() {
            let mut filler = req.actors[0].clone();
            filler.slot = req.actors.len();
            filler.gear_procs.clear();
            req.actors.push(filler);
        }
        req.actors[slot] = base;
    }
}

pub(super) fn request(pairs: &[(usize, &str, u64, u64)]) -> GameScenarioRequest {
    let mut req = crate::simulation::tests::research_request();
    req.stochastic_crit = None;
    req.actions.clear();
    req.action_metadata.clear();
    for (slot, character_id, cycle, ci) in pairs {
        actor(&mut req, *slot, character_id, *cycle, *ci);
    }
    req
}

pub(super) fn switch(timestamp_us: u64, target: usize) -> GameActionCommand {
    GameActionCommand::SwitchActor {
        timestamp_us,
        target_actor_index: target,
        cooldown_us: 0,
    }
}

fn run(req: &GameScenarioRequest) -> Value {
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}

fn reaction_hits<'a>(report: &'a Value, effect: &str) -> Vec<&'a Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == effect)
        .collect()
}

fn trace_count(report: &Value, kind: &str) -> usize {
    report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == kind)
        .count()
}

#[test]
fn temperance_draw_consumes_one_outcome_and_applies_its_exact_target_stat() {
    for (seed, outcome, attack, scorch_damage, break_efficiency) in [
        (2, "atk", "800/1", "1080/1", "1/1"),
        (0, "resistance", "1000/1", "1215/1", "1/1"),
        (7, "break", "1000/1", "1080/1", "11/10"),
    ] {
        let source = format!(
            "options duration=3 iteration=1 crit=fixed seed={seed};
            adler char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
            lacrimosa char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
            target hp=10000000 def=1080 resist=0.2 original_atk=1000
                break_gauge=100 max_break_gauge=100 broken_duration=3;
            active adler; lacrimosa swap; lacrimosa attack; wait(1);"
        );
        let result = crate::simulation::PreparedSourceScenario::new(
            &source,
            Default::default(),
            &Default::default(),
        )
        .unwrap()
        .run(seed)
        .unwrap();
        assert!(result.report.execution_completed, "{:?}", result.diagnostic);
        let report = serde_json::to_value(&result.report).unwrap();
        let target = &report["final_target_states"][0];
        assert_eq!(target["attack"], attack);
        assert!(
            target["statuses"]
                .as_array()
                .unwrap()
                .iter()
                .any(
                    |status| status["id"] == format!("adler.temperance_{outcome}")
                        && status["source_actor_index"] == 0
                        && status["stacks"] == 1
                )
        );
        assert_eq!(trace_count(&report, "adler_temperance_applied"), 1);
        assert_eq!(
            report["reaction_rng"]["streams"]["adler.temperance"]["draws"],
            1
        );
        assert_eq!(
            reaction_hits(&report, "ntesim.reaction.scorch")[0]["non_crit_damage"],
            scorch_damage
        );
        assert!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .any(|event| event["event_kind"] == "target_break_gauge_reduced"
                    && event["data"]["exact_efficiency"] == break_efficiency)
        );
        super::assert_native_replay(&result);
    }
}

#[test]
fn temperance_refresh_updates_only_the_drawn_status_and_uses_world_expiry() {
    let mut req = request(&[
        (0, "character:1003", 100, 0),
        (1, "character:1004", 100, 0),
        (2, "character:1033", 0, 0),
    ]);
    req.targets[0].original_atk = Some(1000);
    req.actions = vec![switch(0, 1), switch(1_200_000, 0)];
    req.relative_duration_us = 15_000_000;
    for (seed, expected_live, expected_attack) in [
        (2, "adler.temperance_atk", "800/1"),
        (3, "adler.temperance_break", "1000/1"),
    ] {
        req.seed = seed;
        let report = run(&req);
        assert_eq!(trace_count(&report, "adler_temperance_applied"), 2);
        let target = &report["final_target_states"][0];
        let live: Vec<_> = target["statuses"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|status| {
                status["id"]
                    .as_str()
                    .unwrap()
                    .starts_with("adler.temperance_")
            })
            .collect();
        assert_eq!(live.len(), 1);
        assert_eq!(live[0]["id"], expected_live);
        assert_eq!(live[0]["expires_at_us"], 16_200_000);
        assert_eq!(live[0]["source_actor_index"], 2);
        assert_eq!(live[0]["stacks"], 1);
        assert_eq!(target["attack"], expected_attack);
    }
    req.relative_duration_us = 16_200_000;
    assert!(
        run(&req)["final_target_states"][0]["statuses"]
            .as_array()
            .is_none_or(Vec::is_empty)
    );
    req.actors[2].character_level = 30;
    req.actors[2].purchased_qte_level = Some(1);
    req.relative_duration_us = 2_000_000;
    assert_eq!(trace_count(&run(&req), "adler_temperance_applied"), 0);
}

#[test]
fn blossom_pistils_explode_with_captured_owner_intensity() {
    // COSMOS + NATURE pair: actor 0 carries a full Cycle pool and pays it.
    let mut req = request(&[(0, "character:1046", 100, 0), (1, "character:1019", 0, 600)]);
    req.actions = vec![switch(0, 1)];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(trace_count(&report, "reaction_blossom_created"), 1);
    assert_eq!(report["reaction_counts"]["Blossom"], 1);
    let hits = reaction_hits(&report, "ntesim.reaction.blossom");
    // Five pistils at +2..+10s; the incoming NATURE actor owns the higher CI
    // (600 -> Q=2), so every explosion is 9000*2*1/2*4/5 = 7200.
    assert_eq!(hits.len(), 5);
    for (index, hit) in hits.iter().enumerate() {
        assert_eq!(hit["time_us"], json!(2_000_000 * (index as u64 + 1)));
        assert_eq!(hit["actor_index"], 1);
        assert_eq!(hit["non_crit_damage"], json!("7200/1"));
        assert_eq!(hit["critical"], false);
    }
    assert_eq!(
        report["resolved_model_assumptions"]["combat.esper_cycle_reactions_v1"],
        json!(crate::HypothesisId::CombatEsperCycleReactionsV1.description())
    );
}

#[test]
fn qualified_switches_spend_outgoing_cycle_once() {
    let mut req = request(&[(0, "character:1046", 100, 0), (1, "character:1019", 100, 0)]);
    req.actions = vec![switch(0, 1), switch(1_000_000, 0), switch(2_000_000, 1)];
    let report = run(&req);
    // Two qualified switches spend both pools; the third finds an empty
    // outgoing pool and stays ordinary.
    assert_eq!(trace_count(&report, "reaction_blossom_created"), 2);
    assert_eq!(report["reaction_counts"]["Blossom"], 2);
    assert_eq!(reaction_hits(&report, "ntesim.reaction.blossom").len(), 10);
}

#[test]
fn unqualified_switches_create_no_reactions() {
    // Same element (NATURE+NATURE), no Cycle pool and a dead locked target
    // each stay ordinary.
    let mut req = request(&[(0, "character:1019", 100, 0), (1, "character:1019", 100, 0)]);
    req.actions = vec![switch(0, 1)];
    let report = run(&req);
    assert_eq!(trace_count(&report, "reaction_blossom_created"), 0);
    assert_eq!(reaction_hits(&report, "ntesim.reaction.blossom").len(), 0);

    let mut req = request(&[(0, "character:1046", 99, 0), (1, "character:1019", 0, 0)]);
    req.actions = vec![switch(0, 1)];
    let report = run(&req);
    assert_eq!(trace_count(&report, "reaction_blossom_created"), 0);

    let mut req = request(&[(0, "character:1046", 100, 0), (1, "character:1019", 0, 0)]);
    req.targets[0].hp_current = 0;
    req.actions = vec![switch(0, 1)];
    let report = run(&req);
    assert_eq!(trace_count(&report, "reaction_blossom_created"), 0);
}

#[test]
fn scorch_ticks_sample_fixed_fifty_crit_and_keep_phase_on_refresh() {
    let mut req = request(&[(0, "character:1003", 100, 0), (1, "character:1004", 100, 0)]);
    // Equal INCANTATION/CHAOS resistances keep tick numbers element-neutral.
    req.targets[0].elemental_resistances.insert(
        "INCANTATION".into(),
        Ratio {
            numerator: 1,
            denominator: 5,
        },
    );
    req.targets[0].elemental_resistances.insert(
        "CHAOS".into(),
        Ratio {
            numerator: 1,
            denominator: 5,
        },
    );
    req.stochastic_crit = Some(crate::simulation::ResearchCritPolicy {
        roll_policy: crate::simulation::ResearchCritRollPolicy::IndependentPerTargetHitV1,
        hypothesis: "Test reaction crit policy".into(),
        eligible_effects: BTreeMap::from([
            ("character:1003".into(), BTreeSet::from(["*".into()])),
            ("character:1004".into(), BTreeSet::from(["*".into()])),
        ]),
        actor_sequences: BTreeMap::from([
            ("character:1003".into(), 54),
            ("character:1004".into(), 55),
        ]),
    });
    // First switch applies Scorch at t=0; the second at t=1.2s re-applies it
    // with the same tick phase and a refreshed expiry.
    req.actions = vec![switch(0, 1), switch(1_200_000, 0)];
    req.relative_duration_us = 17_000_000;
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(trace_count(&report, "reaction_scorch_applied"), 2);
    let hits = reaction_hits(&report, "ntesim.reaction.scorch");
    // Fifteen phase ticks (+0.99..+14.85) plus the single refresh-window tick
    // at 15.84; a phase reset would instead insert 2.19.
    assert_eq!(hits.len(), 16);
    let mut times: Vec<u64> = hits
        .iter()
        .map(|hit| hit["time_us"].as_u64().unwrap())
        .collect();
    times.sort_unstable();
    assert_eq!(times[0], 990_000);
    assert_eq!(times[1], 1_980_000);
    assert_eq!(times[14], 14_850_000);
    assert_eq!(times[15], 15_840_000);
    for hit in &hits {
        assert_eq!(hit["crit_rate"], json!("1/2"));
        assert_eq!(
            super::support::hit_metadata(&report, hit)["packet"]["fixed_crit_rate"],
            json!("1/2")
        );
        assert_eq!(hit["non_crit_damage"], json!("1350/1"));
        assert_eq!(hit["crit_damage"], json!("2025/1"));
        let expected = if hit["critical"].as_bool().unwrap() {
            // 2700*1/2*4/5 = 1080 base Scorch; Lv100 Sakiri BT>=2 Passive 1 (1 handle) -> * 5/4 = 1350 non-crit; CD 0.5 -> 2025.
            2025
        } else {
            1350
        };
        assert_eq!(
            hit["damage"],
            json!(format!("{}/{}", expected, 1)),
            "time {}",
            hit["time_us"]
        );
    }
}

#[test]
fn sakiri_passive1_scorch_dot_bonus_requires_bt2_and_counts_dot_handles() {
    let mut bt0_req = request(&[(0, "character:1004", 100, 0), (1, "character:1003", 0, 0)]);
    bt0_req.actors[1].character_level = 1;
    bt0_req.actors[1].character_max_level = Some(20);
    bt0_req.actors[1].talent_levels = Some([1, 1, 1]);
    bt0_req.actors[1].purchased_skill_level = 1;
    bt0_req.actors[1].purchased_qte_level = Some(1);
    bt0_req.actions = vec![switch(0, 1)];
    bt0_req.relative_duration_us = 2_000_000;
    let bt0_report = run(&bt0_req);
    let bt0_ticks = reaction_hits(&bt0_report, "ntesim.reaction.scorch");
    assert_eq!(bt0_ticks.len(), 2);
    // Lv100 Lacrimosa base Scorch: 2700 * 1/2 (DEF 1000) * 4/5 (resist 0.2) = 1080, no BT2 bonus.
    assert_eq!(bt0_ticks[0]["non_crit_damage"], json!("1080/1"));

    let mut bt2_req = request(&[(0, "character:1004", 100, 0), (1, "character:1003", 0, 0)]);
    bt2_req.actions = vec![switch(0, 1)];
    bt2_req.relative_duration_us = 2_000_000;
    let bt2_report = run(&bt2_req);
    let bt2_ticks = reaction_hits(&bt2_report, "ntesim.reaction.scorch");
    assert_eq!(bt2_ticks.len(), 2);
    // Lv100 Sakiri BT>=2 adds +25% FinalDamageUp for active Scorch handle -> 1080 * 5/4 = 1350.
    assert_eq!(bt2_ticks[0]["non_crit_damage"], json!("1350/1"));
}

#[test]
fn scorch_reapplication_after_expiry_anchors_a_fresh_phase() {
    let mut req = request(&[(0, "character:1003", 100, 0), (1, "character:1004", 100, 0)]);
    req.stochastic_crit = Some(crate::simulation::ResearchCritPolicy {
        roll_policy: crate::simulation::ResearchCritRollPolicy::IndependentPerTargetHitV1,
        hypothesis: "Test reaction crit policy".into(),
        eligible_effects: BTreeMap::from([
            ("character:1003".into(), BTreeSet::from(["*".into()])),
            ("character:1004".into(), BTreeSet::from(["*".into()])),
        ]),
        actor_sequences: BTreeMap::from([
            ("character:1003".into(), 54),
            ("character:1004".into(), 55),
        ]),
    });
    // The second switch lands after the 15s expiry: the expired generation
    // must not resurrect its phase, so a fresh 15-tick window starts at 16s.
    req.actions = vec![switch(0, 1), switch(16_000_000, 0)];
    req.relative_duration_us = 31_000_000;
    let report = run(&req);
    let hits = reaction_hits(&report, "ntesim.reaction.scorch");
    assert_eq!(hits.len(), 30);
    let mut times: Vec<u64> = hits
        .iter()
        .map(|hit| hit["time_us"].as_u64().unwrap())
        .collect();
    times.sort_unstable();
    assert_eq!(times[14], 14_850_000);
    assert_eq!(times[15], 16_990_000);
    assert_eq!(times[29], 30_850_000);
}

#[test]
fn nova_bypasses_defense_and_refresh_detonates_pending_once() {
    let mut req = request(&[(0, "character:1004", 100, 0), (1, "character:1020", 100, 0)]);
    req.targets[0].defense = 1080;
    req.actions = vec![switch(0, 1), switch(3_000_000, 0)];
    let report = run(&req);
    let hits = reaction_hits(&report, "ntesim.reaction.nova");
    // The refresh at 3s detonates generation one immediately; generation two
    // settles at 8s. No stale event fires at 5s and nothing duplicates.
    assert_eq!(hits.len(), 2);
    let mut times: Vec<u64> = hits
        .iter()
        .map(|hit| hit["time_us"].as_u64().unwrap())
        .collect();
    times.sort_unstable();
    assert_eq!(times, vec![3_000_000, 8_000_000]);
    for hit in &hits {
        // 45000 * 4/5 = 36000: Mental packet ignores DEF entirely.
        assert_eq!(hit["non_crit_damage"], json!("36000/1"));
        assert_eq!(hit["critical"], false);
    }
}

#[test]
fn nova_status_tag_feeds_devils_blood_conditional_bonus() {
    let build = |cycle: u64| {
        let mut req = request(&[
            (0, "character:1004", cycle, 0),
            (1, "character:1020", 0, 0),
            (2, "character:1019", 0, 0),
        ]);
        req.actors[2].set_id = Some("set:Suit3".into());
        let (proc, _) = crate::sets::devils_blood::projection::compile(
            ntesim_data::stat_projection::frozen_tables(),
        )
        .unwrap();
        req.actors[2].gear_procs = vec![proc];
        // NATURE+PSYCHE is nonadjacent, so the second switch is ordinary and
        // leaves the Devil's Blood wearer active before the Nova tag expires.
        req.actions = vec![switch(0, 1), switch(500_000, 2)];
        req.actions.push(GameActionCommand::InvokeAbility {
            timestamp_us: 1_000_000,
            actor_index: 2,
            ability_id: "caller-test".into(),
            array_index: 0,
            damage_effect_id: "caller-effect".into(),
            atk_rate: Ratio {
                numerator: 1,
                denominator: 1,
            },
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 0,
        });
        req.model_assumptions.insert(
            "test.synthetic_action".into(),
            "Test-only explicit metadata for a synthetic action.".into(),
        );
        req.action_metadata.insert(
            2,
            crate::simulation::ResearchActionMetadata {
                hypothesis_id: "test.synthetic_action".into(),
                ability_id: "caller-test".into(),
                damage_effect_id: "caller-effect".into(),
                action_gameplay_tags: Some(vec!["Ability.Skill".into()]),
                input_action_role: None,
                damage_type_ex: Some("EExecutionDamageTypeEX::DAMAGE_TYPE_NATURE".into()),
                damage_source_category: None,
                fixed_crit_rate: Some(crate::simulation::ResearchFixedCritRate::NoOverride),
                unbal_value: None,
                packet_ga_name_binding: None,
            },
        );
        run(&req)
    };
    let ordinary = build(0);
    let nova = build(100);
    // 1000 ATK * 1 * 1/2 DEF * 4/5 RES: Devil's Blood base +18% gives 472;
    // the live Nova tag (Reaction.AnHun) adds the conditional +18% -> 544.
    assert_eq!(
        reaction_hits(&ordinary, "caller-effect")[0]["non_crit_damage"],
        json!("472/1")
    );
    assert_eq!(
        reaction_hits(&nova, "caller-effect")[0]["non_crit_damage"],
        json!("544/1")
    );
    assert_eq!(reaction_hits(&nova, "ntesim.reaction.nova").len(), 1);
}

#[test]
fn dead_outgoing_actor_cannot_admit_a_new_reaction() {
    let mut req = request(&[(0, "character:1046", 100, 0), (1, "character:1019", 100, 0)]);
    req.actors[0].initial_hp = 0;
    req.actions = vec![switch(0, 1)];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(trace_count(&report, "switch_actor"), 1);
    assert_eq!(trace_count(&report, "reaction_blossom_created"), 0);
    assert!(report["hit_receipts"].as_array().unwrap().is_empty());
    assert!(
        report["resolved_model_assumptions"]
            .get("combat.esper_cycle_reactions_v1")
            .is_none()
    );
}

#[test]
fn scorch_rng_is_seeded_persistent_and_independent_of_ordinary_crit_policy() {
    let mut req = request(&[(0, "character:1003", 100, 0), (1, "character:1004", 100, 0)]);
    req.actions = vec![switch(0, 1), switch(1_200_000, 0)];
    let report = run(&req);
    assert_eq!(report, run(&req));
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = reaction_hits(&report, "ntesim.reaction.scorch");
    assert_eq!(hits.len(), 16);
    // Seed 42, selector 54 starts with a15c02b7: odd -> noncritical;
    // selector 55 starts an independent stream when refresh changes the owner.
    assert_eq!(hits[0]["crit_sample"]["draws"][0]["value"], 0xa15c02b7u64);
    assert_eq!(hits[0]["crit_sample"]["draws"][0]["stream"], "scorch:0");
    assert_eq!(hits[0]["critical"], false);
    assert_eq!(report["reaction_rng"]["streams"]["scorch:0"]["draws"], 1);
    assert_eq!(report["reaction_rng"]["streams"]["scorch:1"]["draws"], 15);
    assert!(report.get("rng").is_none());
    req.stochastic_crit = Some(ResearchCritPolicy {
        roll_policy: ResearchCritRollPolicy::IndependentPerTargetHitV1,
        hypothesis: "Only the incoming actor has ordinary crit eligibility".into(),
        eligible_effects: BTreeMap::from([("character:1004".into(), BTreeSet::from(["*".into()]))]),
        actor_sequences: BTreeMap::from([("character:1004".into(), 999)]),
    });
    let partial = run(&req);
    assert_eq!(reaction_hits(&partial, "ntesim.reaction.scorch"), hits);
    assert_eq!(partial["reaction_rng"], report["reaction_rng"]);
    assert_eq!(partial["rng"]["streams"]["character:1004"]["draws"], 1);
    req.seed += 1;
    let changed = run(&req);
    assert_ne!(changed["hit_receipts"], report["hit_receipts"]);
    assert_eq!(changed["reaction_rng"]["seed"], req.seed);
}

#[test]
fn reaction_refresh_keeps_one_shared_status_with_the_latest_real_owner() {
    for (first, second, id, refresh, horizon) in [
        (
            "character:1004",
            "character:1020",
            "ntesim.reaction.nova",
            3_000_000,
            8_050_000,
        ),
        (
            "character:1003",
            "character:1004",
            "ntesim.reaction.scorch",
            1_200_000,
            1_300_000,
        ),
    ] {
        let mut req = request(&[(0, first, 100, 0), (1, second, 100, 0)]);
        req.relative_duration_us = horizon;
        req.actions = vec![switch(0, 1), switch(refresh, 0)];
        let report = run(&req);
        let statuses = report["final_target_states"][0]["statuses"]
            .as_array()
            .unwrap();
        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0]["id"], id);
        assert_eq!(statuses[0]["source_actor_index"], 1);
        assert_eq!(statuses[0]["stacks"], 1);
        let buffs = crate::simulation::tests::support::final_buffs(&report);
        let status = buffs.iter().find(|buff| buff["id"] == id).unwrap();
        assert_eq!(status["stacking_scope"], "shared");
        assert_eq!(status["owner_actor"], 1);
        for hit in report["hit_receipts"].as_array().unwrap() {
            let origin = &hit["switch_origin"];
            if origin["kind"] == "support" {
                continue;
            }
            assert_eq!(origin["kind"], "esper_cycle");
            let command = hit["command_index"].as_u64().unwrap();
            assert_eq!(origin["outgoing_actor_index"], command);
            assert_eq!(origin["incoming_actor_index"], 1 - command);
        }
    }
}

#[test]
fn reaction_kill_runs_the_same_equipment_defeat_callback_as_an_ordinary_kill() {
    let mut req = request(&[(0, "character:1003", 100, 0), (1, "character:1004", 0, 0)]);
    req.actors[0].initial_hp = 50;
    req.actors[0].initial_hp_max = 100;
    req.actors[0].weapon_id = "weapon:fork_vine".into();
    req.actors[0].gear_procs = vec![GearProcConfig::BeHappy {
        heal_ratio: Ratio {
            numerator: 3,
            denominator: 25,
        },
        cooldown_us: 20_000_000,
    }];
    req.actions = vec![switch(0, 1)];
    let probe = run(&req);
    let support_loss =
        reaction_hits(&probe, "GE_Player_Lacrimosa_QTE1_Damage")[0]["applied_hp_loss"]
            .as_u64()
            .unwrap();
    req.targets[0].hp_current = support_loss + 1;
    req.targets[0].hp_max = support_loss + 1;
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(reaction_hits(&report, "ntesim.reaction.scorch").len(), 1);
    assert_eq!(trace_count(&report, "be_happy_heal"), 1);
    // 100 max HP * 3/25 = 12: exactly one kill restores 50 -> 62 HP.
    assert_eq!(report["final_actor_states"][0]["hp"], 62);
    assert_eq!(report["final_target_states"][0]["hp_current"], 0);
}

#[test]
fn hexed_switch_spends_cycle_and_applies_status() {
    // Nature (Nanally, 1010) + Incantation (Adler, 1033) = Hexed
    let mut req = request(&[(0, "character:1010", 100, 0), (1, "character:1033", 0, 0)]);
    req.actions = vec![switch(0, 1)];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["final_actor_states"][0]["cycle"]["current"], 0);
    assert_eq!(trace_count(&report, "reaction_hexed_applied"), 1);
    assert_eq!(report["reaction_counts"]["Hexed"], 1);
}

#[test]
fn hexed_support_followup_preserves_the_parent_switch_and_owner() {
    let mut compiled = crate::dsl::compiler::compile_scenario(
        "options duration=2 crit=fixed;
         mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
         sakiri char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
         target hp=100000000 def=1080 resist=0.2; active mint; wait(1);",
        Default::default(),
    )
    .unwrap();
    compiled.request.actions = vec![switch(0, 1)];
    let report = run(&compiled.request);
    let hits = report["hit_receipts"].as_array().unwrap();
    let child_index = hits
        .iter()
        .position(|hit| hit["damage_effect_id"] == "ntesim.reaction.hexed_followup")
        .unwrap();
    let child = &hits[child_index];
    let parent = &hits[child_index - 1];
    assert_eq!(child["parent_hit_index"], child_index - 1);
    assert_eq!(child["command_index"], 0);
    assert_eq!(child["actor_index"], parent["actor_index"]);
    assert_eq!(child["switch_origin"], parent["switch_origin"]);
    assert_eq!(child["switch_origin"]["kind"], "support");
    assert_eq!(child["switch_origin"]["incoming_actor_index"], 1);
    assert!(report["trace"].as_array().unwrap().iter().any(|event| {
        event["event_kind"] == "reaction_damage"
            && event["hit_index"] == child_index
            && event["data"]["target_index"] == 0
    }));
    for field in ["parent_hit_index", "actor_index", "switch_origin"] {
        let mut tampered = report.clone();
        tampered["hit_receipts"][child_index][field] = match field {
            "parent_hit_index" => json!(child_index),
            "actor_index" => json!(0),
            _ => serde_json::Value::Null,
        };
        let decoded: crate::simulation::GameScenarioReport =
            serde_json::from_value(tampered).unwrap();
        assert!(decoded.validate_damage_parentage().is_err(), "{field}");
    }
    assert_eq!(
        child["rounded_damage"].as_u64().unwrap(),
        (parent["applied_hp_loss"].as_u64().unwrap() + 2) / 5
    );
}

#[test]
fn hexed_followup_damage_matches_independent_ci_oracle() {
    let build = |ci: u64| {
        let mut req = request(&[(0, "character:1010", 100, 0), (1, "character:1033", 0, ci)]);
        // Base ATK 2500, DEF factor 1/2, RES factor 4/5 -> 2500 * 0.5 * 0.8 = 1000 applied damage
        req.actors[1].static_stats.base_atk = 2500;
        req.actors[1].static_stats.atk = Ratio {
            numerator: 2500,
            denominator: 1,
        };
        req.actions = vec![
            switch(0, 1),
            GameActionCommand::InvokeAbility {
                timestamp_us: 1_400_000,
                actor_index: 1,
                ability_id: "test-incantation-hit".into(),
                array_index: 0,
                damage_effect_id: "test-incantation-effect".into(),
                atk_rate: Ratio {
                    numerator: 1,
                    denominator: 1,
                },
                scaling_stat: crate::simulation::NativeScalingStat::Atk,
                energy_cost: 0,
                recovery_us: 0,
                cooldown_us: 0,
                hit_delay_us: 0,
            },
        ];
        req.model_assumptions.insert(
            "test.hexed_action".into(),
            "Explicit metadata for Hexed test action.".into(),
        );
        req.action_metadata.insert(
            1,
            crate::simulation::ResearchActionMetadata {
                hypothesis_id: "test.hexed_action".into(),
                ability_id: "test-incantation-hit".into(),
                damage_effect_id: "test-incantation-effect".into(),
                action_gameplay_tags: Some(vec!["Ability.Skill".into()]),
                input_action_role: None,
                damage_type_ex: Some("EExecutionDamageTypeEX::DAMAGE_TYPE_INCANTATION".into()),
                damage_source_category: None,
                fixed_crit_rate: Some(crate::simulation::ResearchFixedCritRate::NoOverride),
                unbal_value: None,
                packet_ga_name_binding: None,
            },
        );
        run(&req)
    };

    // At CI 0: applied 1000 -> Hexed follow-up 1000 * 0.20 = 200
    let rep0 = build(0);
    assert_eq!(rep0["execution_completed"], true);
    let hits0: Vec<_> = reaction_hits(&rep0, "ntesim.reaction.hexed_followup")
        .into_iter()
        .filter(|hit| hit["command_index"] == 1)
        .collect();
    assert_eq!(hits0.len(), 1);
    assert_eq!(hits0[0]["rounded_damage"], 200);
    let parent = hits0[0]["parent_hit_index"].as_u64().unwrap() as usize;
    assert_eq!(
        rep0["hit_receipts"][parent]["damage_effect_id"],
        "test-incantation-effect"
    );

    // At CI 108: h(108) = 1.2 * (1 + 0.2 * 108 / 288) - 1 = 0.29 -> 1000 * 0.29 = 290
    let rep108 = build(108);
    assert_eq!(rep108["execution_completed"], true);
    let hits108: Vec<_> = reaction_hits(&rep108, "ntesim.reaction.hexed_followup")
        .into_iter()
        .filter(|hit| hit["command_index"] == 1)
        .collect();
    assert_eq!(hits108.len(), 1);
    assert_eq!(hits108[0]["rounded_damage"], 290);
}

#[test]
fn hexed_followup_ignores_unrelated_elements_and_does_not_recurse() {
    let mut req = request(&[(0, "character:1010", 100, 0), (1, "character:1033", 0, 0)]);
    req.actions = vec![
        switch(0, 1),
        GameActionCommand::InvokeAbility {
            timestamp_us: 1_400_000,
            actor_index: 1,
            ability_id: "test-chaos-hit".into(),
            array_index: 0,
            damage_effect_id: "test-chaos-effect".into(),
            atk_rate: Ratio {
                numerator: 1,
                denominator: 1,
            },
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 0,
        },
    ];
    req.model_assumptions.insert(
        "test.chaos_action".into(),
        "Explicit metadata for test Chaos action.".into(),
    );
    req.action_metadata.insert(
        1,
        crate::simulation::ResearchActionMetadata {
            hypothesis_id: "test.chaos_action".into(),
            ability_id: "test-chaos-hit".into(),
            damage_effect_id: "test-chaos-effect".into(),
            action_gameplay_tags: Some(vec!["Ability.Skill".into()]),
            input_action_role: None,
            damage_type_ex: Some("EExecutionDamageTypeEX::DAMAGE_TYPE_CHAOS".into()),
            damage_source_category: None,
            fixed_crit_rate: Some(crate::simulation::ResearchFixedCritRate::NoOverride),
            unbal_value: None,
            packet_ga_name_binding: None,
        },
    );
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let followups = reaction_hits(&report, "ntesim.reaction.hexed_followup");
    assert_eq!(followups.len(), 1);
    assert_eq!(followups[0]["command_index"], 0);
}

#[test]
fn mint_a3_attendance_bonus_applied_on_reaction_participation() {
    // Cosmos (Zero, 1046) + Nature (Mint, 1019) = Blossom
    let mut req = request(&[(0, "character:1046", 100, 0), (1, "character:1019", 0, 0)]);
    req.actors[1].awakening = ntesim_data::AwakeningSelection::prefix(3);
    req.actions = vec![switch(0, 1)];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(trace_count(&report, "reaction_blossom_created"), 1);
    assert_eq!(trace_count(&report, "mint_a3_attendance_bonus_applied"), 1);
    assert_eq!(report["reaction_counts"]["Blossom"], 1);
}

#[test]
fn reaction_counts_track_switches_and_charge_trigger() {
    let mut req = request(&[
        (0, "character:1008", 100, 0), // Lakshana
        (1, "character:1046", 100, 0), // Cosmos
        (2, "character:1019", 0, 0),   // Nature
    ]);
    req.actions = vec![switch(0, 1), switch(1_000_000, 2)];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["reaction_counts"]["Remora"], 1);
    assert_eq!(report["reaction_counts"]["Blossom"], 1);
    assert!(report["reaction_counts"]["Charge"].as_u64().unwrap() >= 1);
}

#[test]
fn zero_charge_healing_passive_restores_hp_on_charge_grant() {
    let mut req = request(&[
        (0, "character:1008", 100, 0), // Lakshana
        (1, "character:1046", 100, 0), // Cosmos (Zero)
        (2, "character:1019", 0, 0),   // Nature (Mint)
    ]);
    req.actors[1].static_stats.base_atk = 1000;
    req.actors[2].initial_hp_max = 10_000;
    req.actors[2].initial_hp = 5000;
    req.actions = vec![switch(0, 1), switch(1_000_000, 2)];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(trace_count(&report, "zero_charge_heal"), 1);
    let heals = report["healing_receipts"].as_array().unwrap();
    let zero_heal = heals
        .iter()
        .find(|h| h["source_actor_index"] == 1 && h["recipient_actor_index"] == 2)
        .expect("expected healing receipt from Zero to Mint");
    assert_eq!(zero_heal["gross"], 500);
    assert_eq!(zero_heal["applied"], 500);
    assert_eq!(zero_heal["hp_before"], 5000);
    assert_eq!(zero_heal["hp_after"], 5500);
}

#[test]
fn character_reaction_passives_audit_and_dispatch() {
    let mut req = request(&[(0, "character:1055", 100, 0)]);
    let state = crate::simulation::state::GameSimulatorState::new(&req);
    let default_policy = crate::registry::blossom_bud_policy(&req, &state);
    assert_eq!(default_policy.bud_cap, 3);
    assert_eq!(default_policy.num_buds, 1);

    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([2]).unwrap();
    let b2_policy = crate::registry::blossom_bud_policy(&req, &state);
    assert_eq!(b2_policy.bud_cap, 6);
    assert_eq!(b2_policy.num_buds, 2);

    let mut sakiri_req = request(&[(0, "character:1003", 100, 0)]);
    let sakiri_state = crate::simulation::state::GameSimulatorState::new(&sakiri_req);
    assert!(crate::registry::sakiri_scorch_extension_eligible(
        &sakiri_req,
        &sakiri_state
    ));
    sakiri_req.actors[0].character_level = 1;
    sakiri_req.actors[0].character_max_level = Some(20);
    assert!(!crate::registry::sakiri_scorch_extension_eligible(
        &sakiri_req,
        &sakiri_state
    ));

    let shinku_req = request(&[(0, "character:1076", 100, 0)]);
    let mut shinku_state = crate::simulation::state::GameSimulatorState::new(&shinku_req);
    assert!(crate::registry::shinku_reaction_aoe_eligible(
        &shinku_req,
        &shinku_state
    ));
    shinku_state.actors[0].hp = 0;
    assert!(!crate::registry::shinku_reaction_aoe_eligible(
        &shinku_req,
        &shinku_state
    ));
}
