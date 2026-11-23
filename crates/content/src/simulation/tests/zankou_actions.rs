use super::*;
use crate::simulation::tests::support::formula_damage_total;
use serde_json::json;

fn action(timestamp_us: u64) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index: 0,
        ability_id: "GA_Zankou_Melee".into(),
        damage_effect_id: "GE_Player_Zankou_Melee1_Damage".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 100,
    }
}

pub(super) fn request() -> GameScenarioRequest {
    let mut req = sample_s1_request();
    req.actors[0].character_id = "character:1036".into();
    req.actors[0].weapon_id.clear();
    req.actors[0].gear_procs.clear();
    req.actors[0].set_id = None;
    req.actors[0].talent_levels = Some([1; 3]);
    req.actors[0].character_max_level = Some(80);
    req.actors[0].character_level = 80;
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonuses.clear();
    req.actors[0].initial_hp = 10_000;
    req.actors[0].initial_hp_max = 10_000;
    req.targets[0].resistance = Ratio::zero();
    req.actions = vec![action(0)];
    req
}

#[test]
fn zankou_kindling_uses_packet_identity_and_magic_ultra_requires_a4() {
    for nodes in [
        vec![1, 2],
        vec![1, 2, 3],
        vec![1, 2, 3, 4],
        vec![1, 2, 4],
        vec![3],
        vec![6],
        vec![1, 3, 5],
    ] {
        for (packet, magic) in [
            ("GE_Player_Zankou_UltraSkill1_Damage", false),
            ("GE_Player_Zankou_UltraSkill2_Damage", false),
            ("GE_Player_Zankou_ForceUltraSkill1_Damage", false),
            ("GE_Player_Zankou_ForceUltraSkill2_Damage", false),
            ("GE_Player_Zankou_MagicUltraSkill1_Damage", true),
            ("GE_Player_Zankou_MagicUltraSkill2_Damage", true),
        ] {
            for kindling in [false, true] {
                let mut req = request();
                req.actors[0].awakening =
                    ntesim_data::AwakeningSelection::selected(nodes.clone()).unwrap();
                let mut state = crate::simulation::state::GameSimulatorState::new(&req);
                state.actors[0]
                    .character
                    .zankou_mut()
                    .unwrap()
                    .kindling_active_for_ultra = kindling;
                let ctx = crate::runtime_api::DamageContext {
                    foreign_dot_stacks: 0,
                    packet_group_id: None,
                    command_index: 0,
                    config: &req.actors[0],
                    start_fons: None,
                    ability_id: "GA_Zankou_UltraSkill",
                    action_metadata: &super::support::declared_test_action(
                        "GA_Zankou_UltraSkill",
                        crate::simulation::metadata::ActionCategory::UltraSkill,
                    ),
                    target: &state.targets[0],
                    actor: &state.actors[0],
                    world_time_us: 0,
                    time_us: 0,
                    debuff_count: 0,
                    damage_effect_id: packet,
                    actor_index: 0,
                    active_actor_index: 0,
                    target_count: 1,
                    non_boss_target_count: 1,
                    target_index: 0,
                    buffs: &state.buffs,
                    buff_stats: Default::default(),
                    owned_status_counts: &Default::default(),
                    team_has_three_elements: false,
                };
                let modifiers = crate::characters::zankou::awakenings::damage_bonus(
                    &ctx,
                    crate::runtime_api::HitModifiers::new(ExactFraction::new(0, 1).unwrap())
                        .unwrap(),
                )
                .unwrap();
                let expected_final =
                    if nodes.contains(&2) && kindling && (!magic || nodes.contains(&4)) {
                        ExactFraction::new(3, 2).unwrap()
                    } else {
                        ExactFraction::new(0, 1).unwrap()
                    };
                assert_eq!(
                    modifiers.final_bonus, expected_final,
                    "{packet} nodes={nodes:?} Kindling={kindling}"
                );
                assert_eq!(
                    modifiers.ratio_add,
                    ExactFraction::new(if nodes.len() >= 3 { 1 } else { 0 }, 5).unwrap()
                );
            }
        }
    }
}

#[test]
fn zankou_normal_chain_uses_native_published_coefficients_and_loops() {
    for (level, awakening, mut expected, mut chain_total, mut loop_total) in [
        (
            1,
            0,
            vec![
                (79, 1),
                (137, 1),
                (176, 1),
                (289, 1),
                (183, 2),
                (353, 2),
                (341, 1),
                (247, 2),
                (114, 1),
                (261, 1),
                (103, 2),
                (182, 1),
                (182, 1),
                (63, 1),
                (63, 1),
                (965, 2),
                (317, 1),
                (79, 1),
                (137, 1),
            ],
            (6259, 2),
            (6691, 2),
        ),
        (
            8,
            0,
            vec![
                (271, 2),
                (235, 1),
                (603, 2),
                (991, 2),
                (157, 1),
                (605, 2),
                (1169, 2),
                (423, 2),
                (391, 2),
                (895, 2),
                (177, 2),
                (312, 1),
                (312, 1),
                (108, 1),
                (108, 1),
                (827, 1),
                (1087, 2),
                (271, 2),
                (235, 1),
            ],
            (5365, 1),
            (11471, 2),
        ),
        (
            8,
            3,
            vec![
                (1022, 5),
                (3549, 10),
                (2282, 5),
                (749, 1),
                (2373, 10),
                (4571, 10),
                (4417, 5),
                (3199, 10),
                (1477, 5),
                (3381, 5),
                (1337, 10),
                (2359, 5),
                (2359, 5),
                (1631, 10),
                (1631, 10),
                (6251, 5),
                (8211, 10),
                (1022, 5),
                (3549, 10),
            ],
            (40544, 5),
            (86681, 10),
        ),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([level, 1, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = [0, 1_084_427, 2_241_797, 3_359_148, 5_337_383, 6_811_396]
            .into_iter()
            .map(action)
            .collect();
        expected.swap(12, 13);
        if awakening == 0 {
            for (n, d) in &mut expected {
                let f = ExactFraction::new(*n * 5, *d * 4).unwrap();
                *n = f.numerator;
                *d = f.denominator;
            }
            for total in [&mut chain_total, &mut loop_total] {
                let f = ExactFraction::new(total.0 * 5, total.1 * 4).unwrap();
                *total = (f.numerator, f.denominator);
            }
        }
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "halt: {:?}",
            report["halt_diagnostic"]
        );
        assert_eq!(report["hits_settled"], 19);
        let receipts = report["hit_receipts"].as_array().unwrap();
        let expected_effects = [
            "GE_Player_Zankou_Melee1_Damage",
            "GE_Player_Zankou_Melee1_1_Damage",
            "GE_Player_Zankou_Melee2_Damage",
            "GE_Player_Zankou_Melee3_Damage",
            "GE_Player_Zankou_Melee3_1_Damage",
            "GE_Player_Zankou_Melee3_2_Damage",
            "GE_Player_Zankou_Melee4_Damage",
            "GE_Player_Zankou_Melee4_1_Damage",
            "GE_Player_Zankou_Melee4_2_Damage",
            "GE_Player_Zankou_Melee4_3_Damage",
            "GE_Player_Zankou_Melee4_4_Damage",
            "GE_Player_Zankou_Melee5_Damage",
            "GE_Player_Zankou_Melee5_1_Damage",
            "GE_Player_Zankou_Melee5_Damage",
            "GE_Player_Zankou_Melee5_1_Damage",
            "GE_Player_Zankou_Melee5_2_Damage",
            "GE_Player_Zankou_Melee6_Damage",
            "GE_Player_Zankou_Melee1_Damage",
            "GE_Player_Zankou_Melee1_1_Damage",
        ];
        for ((receipt, (numerator, denominator)), effect) in
            receipts.iter().zip(expected).zip(expected_effects)
        {
            assert_eq!(
                receipt["non_crit_damage"],
                json!(format!("{}/{}", numerator, denominator))
            );
            assert_eq!(receipt["damage_effect_id"], effect);
            let meta = super::support::hit_metadata(&report, receipt);
            assert_eq!(
                meta["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"
            );
            assert_eq!(
                meta["packet"]["native_damage_type_ex"],
                "EExecutionDamageTypeEX::DAMAGE_TYPE_INCANTATION"
            );
        }
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            json!(format!("{}/{}", loop_total.0, loop_total.1))
        );
        let mut chain_req = req;
        chain_req.actions.truncate(5);
        let chain = run_game_scenario(&chain_req, &ExecutionLimits::default()).unwrap();
        assert_eq!(chain["hits_settled"], 17);
        assert_eq!(
            serde_json::json!(formula_damage_total(&chain)),
            json!(format!("{}/{}", chain_total.0, chain_total.1))
        );
    }
}

#[test]
fn zankou_normal_chain_damage_ignores_hp_current_hp_and_character_defense() {
    let baseline = run_game_scenario(&request(), &ExecutionLimits::default()).unwrap();
    for changed in [
        {
            let mut req = request();
            req.actors[0].static_stats.hp += 50_000;
            req.actors[0].initial_hp += 50_000;
            req.actors[0].initial_hp_max += 50_000;
            req
        },
        {
            let mut req = request();
            req.actors[0].initial_hp = 1;
            req
        },
        {
            let mut req = request();
            req.actors[0].static_stats.def += 20_000;
            req
        },
    ] {
        let report = run_game_scenario(&changed, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            baseline["hit_receipts"][0]["non_crit_damage"],
            report["hit_receipts"][0]["non_crit_damage"]
        );
        assert_eq!(
            baseline["hit_receipts"][0]["damage_effect_id"],
            report["hit_receipts"][0]["damage_effect_id"]
        );
    }
}

#[test]
fn zankou_normal_chain_strict_mode_and_off_field_packet_horizon() {
    let source = "options duration=4 crit=fixed;zankou char lvl=80/80 awakening=0 talent=1,1,1 qte=1;mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;target hp=100000000 def=0 resist=0;active zankou;zankou attack;mint swap;";
    let result = crate::simulation::PreparedSourceScenario::new(
        source,
        Default::default(),
        &Default::default(),
    )
    .unwrap()
    .run(42)
    .unwrap();
    assert!(result.report.execution_completed);
    super::assert_native_replay(&result);
    let mut req = result.request;
    req.model_assumptions
        .extend(result.report.model_assumptions);
    req.model_assumptions
        .extend(result.report.resolved_model_assumptions);
    req.model_assumptions
        .extend(result.report.callback_model_assumptions);
    for actor in &mut req.actors {
        actor.talent_levels = None;
        actor.purchased_skill_level = 1;
    }
    req.model_assumptions.retain(|id, description| {
        ntesim_contracts::get_hypothesis_description(id).is_some_and(|known| known == description)
    });
    let id = crate::characters::zankou::forms::MODEL;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    let report = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hit_receipts"][0]["time_us"], 124456);
    assert_eq!(report["hit_receipts"][1]["time_us"], 342267);
    assert_eq!(report["final_buff_state"]["active_actor_index"], 1);
    req.model_assumptions
        .insert(id.into(), "incorrect description".into());
    assert!(
        run_game_scenario(&req, &Default::default())
            .unwrap_err()
            .contains("conflicting reached model hypothesis")
    );
}

#[test]
fn zankou_normal_chain_drops_later_packets_after_target_defeat() {
    let mut req = request();
    req.targets[0].hp_current = 70;
    req.actions = vec![action(0)];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 1);
    assert_eq!(report["pending_events"], 0);
    assert_eq!(report["final_target_states"][0]["hp_current"], 0);
    let receipts = report["hit_receipts"].as_array().unwrap();
    assert_eq!(receipts.len(), 1);
    assert_eq!(
        receipts[0]["damage_effect_id"],
        "GE_Player_Zankou_Melee1_Damage"
    );
}

#[test]
fn zankou_normal_chain_gap_boundary_resets_only_after_two_seconds() {
    let mut req = request();
    req.actions = vec![action(0), action(3_084_427), action(6_241_798)];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let effects = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|receipt| receipt["damage_effect_id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        effects,
        [
            "GE_Player_Zankou_Melee1_Damage",
            "GE_Player_Zankou_Melee1_1_Damage",
            "GE_Player_Zankou_Melee2_Damage",
            "GE_Player_Zankou_Melee1_Damage",
            "GE_Player_Zankou_Melee1_1_Damage",
        ]
    );
}

#[test]
fn zankou_scorch_spread_on_multi_target() {
    let source = "options duration=60 crit=fixed;
        adler char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        lacrimosa char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        zankou char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=1080 resist=0;
        target hp=100000000 def=1080 resist=0;
        active adler; adler skill; lacrimosa swap; wait(1); zankou swap; zankou skill; zankou attack;";
    let req = crate::simulation::PreparedSourceScenario::new(
        source,
        Default::default(),
        &Default::default(),
    )
    .unwrap()
    .run(42)
    .unwrap()
    .request;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "halt: {:?}",
        report["halt_diagnostic"]
    );

    let trace = report["trace"].as_array().unwrap();
    let spread_events: Vec<_> = trace
        .iter()
        .filter(|e| e["event_kind"] == "zankou_scorch_spread")
        .collect();
    assert!(
        !spread_events.is_empty(),
        "expected zankou_scorch_spread event in trace"
    );

    let scorch_hits: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "ntesim.reaction.scorch")
        .collect();
    let targets_with_scorch: std::collections::BTreeSet<u64> = scorch_hits
        .iter()
        .map(|h| h["target_index"].as_u64().unwrap())
        .collect();
    assert!(
        targets_with_scorch.contains(&0) && targets_with_scorch.contains(&1),
        "expected Scorch hits on both targets after spread, found targets: {targets_with_scorch:?}"
    );
}

#[test]
fn spread_scorch_preserves_ownership_replacement_cause_across_recipients() {
    let source = "options iteration=1 duration=70 crit=fixed;
      zankou char lvl=80/80 awakening=0 talent=8,8,1 qte=8 start_energy=0;
      zankou add weapon=fork_NestBird refine=1;
      lacrimosa char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
      target hp=100000000 def=1080 resist=0; target hp=100000000 def=1080 resist=0;
      active zankou; zankou skill; wait(1); lacrimosa swap;
      wait(3); zankou swap; wait(9); zankou skill; zankou attack; lacrimosa swap;";
    let request = crate::simulation::PreparedSourceScenario::new(
        source,
        Default::default(),
        &Default::default(),
    )
    .unwrap()
    .run(42)
    .unwrap()
    .request;
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    let mut crossed_targets = false;
    for (index, hit) in hits.iter().enumerate() {
        if hit["switch_origin"]["kind"] != "esper_cycle_replacement" {
            continue;
        }
        let trigger_index = hit["switch_origin"]["trigger_hit_index"].as_u64().unwrap() as usize;
        assert!(trigger_index < index);
        let trigger = &hits[trigger_index];
        assert!(trigger["time_us"].as_u64().unwrap() <= hit["time_us"].as_u64().unwrap());
        assert_eq!(hit["actor_index"], 0);
        assert_eq!(hit["damage_effect_id"], "ntesim.reaction.scorch");
        crossed_targets |= trigger["target_index"] != hit["target_index"];
    }
    let recipients = hits
        .iter()
        .filter(|h| h["switch_origin"]["kind"] == "esper_cycle_replacement")
        .map(|h| h["target_index"].as_u64().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(recipients, std::collections::BTreeSet::from([0, 1]));
    let _ = crossed_targets; // Native all-target donors need not cross target identity.
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_kind"] == "zankou_scorch_spread")
    );
}

#[test]
fn zankou_r6_scorch_kill_grants_only_the_native_timed_enhancement() {
    let source = "options duration=70 crit=fixed;
        adler char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        lacrimosa char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        zankou char lvl=80/80 awakening=6 talent=1,1,1 qte=1;
        target hp=10000 def=0 resist=0;
        target hp=100000000 def=0 resist=0;
        active adler; adler skill; lacrimosa swap; wait(1); zankou swap; zankou skill;";
    let req = crate::simulation::PreparedSourceScenario::new(
        source,
        Default::default(),
        &Default::default(),
    )
    .unwrap()
    .run(42)
    .unwrap()
    .request;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "halt: {:?}",
        report["halt_diagnostic"]
    );

    let trace = report["trace"].as_array().unwrap();
    let r6_events: Vec<_> = trace
        .iter()
        .filter(|e| e["event_kind"] == "zankou_r6_scorch_kill_enhancement")
        .collect();
    assert!(
        !r6_events.is_empty(),
        "expected zankou_r6_scorch_kill_enhancement event in trace"
    );
    assert_eq!(report["final_target_states"][0]["hp_current"], 0);
}

#[test]
fn zankou_managed_hunt_uses_the_actual_action_and_selected_a1_without_double_counting() {
    for (nodes, expected) in [(vec![], "395/4"), (vec![1], "553/5")] {
        let mut req = request();
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(nodes).unwrap();
        let report = run_game_scenario(&req, &Default::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hit_receipts"][0]["non_crit_damage"], expected);
    }
}

#[test]
fn zankou_a2_a4_share_one_capture_across_two_actual_ultimate_pairs() {
    let run = |nodes: &str| {
        let source = format!(
            "options duration=120 crit=fixed;
            zankou char lvl=80/80 awakening={nodes} talent=1,1,1 qte=1 start_energy=120;
            zankou add stats ce=1;
            mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
            target hp=1000000000 def=0 resist=0; target hp=1000000000 def=0 resist=0; active zankou;
            repeat 2 {{mint swap; zankou burst; zankou burst; zankou attack:20;}}"
        );
        let result = crate::simulation::PreparedSourceScenario::new(
            &source,
            Default::default(),
            &Default::default(),
        )
        .unwrap()
        .run(42)
        .unwrap();
        assert!(
            result.report.execution_completed,
            "{:?}",
            result.report.halt_diagnostic
        );
        super::assert_native_replay(&result);
        result
    };
    let baseline = run("[4]");
    let boosted = run("[2,4]");
    for result in [&baseline, &boosted] {
        assert_eq!(result.report.final_actor_states[0].spent_energy, 240);
    }
    let packets = |result: &crate::simulation::SourceScenarioReport| {
        result
            .report
            .hit_receipts
            .iter()
            .filter(|h| {
                h.damage_effect_id.contains("UltraSkill") && !h.damage_effect_id.contains("Dot")
            })
            .map(|h| (h.damage_effect_id.clone(), h.non_crit_damage))
            .collect::<Vec<_>>()
    };
    let a = packets(&baseline);
    let b = packets(&boosted);
    assert_eq!(a.len(), 24);
    assert_eq!(a.len(), b.len());
    for ((a_name, a_damage), (b_name, b_damage)) in a.iter().zip(&b) {
        assert_eq!(a_name, b_name);
        assert_eq!(
            *b_damage,
            a_damage.mul(ExactFraction::new(5, 2).unwrap()).unwrap()
        );
    }
    let report = serde_json::to_value(&boosted.report).unwrap();
    let grants = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "zankou_a2_kindling_generated")
        .collect::<Vec<_>>();
    assert!(grants.len() >= 2);
    assert!(grants.iter().all(|e| e["data"]["stacks"] == 1));
}

#[test]
fn zankou_without_a4_preserves_kindling_for_the_free_inferno_only() {
    let run = |nodes: &str| {
        let source = format!(
            "options duration=45 crit=fixed;
            zankou char lvl=80/80 awakening={nodes} talent=1,1,1 qte=1 start_energy=100;
            mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
            target hp=1000000000 def=0 resist=0; active zankou;
            mint swap; zankou attack; zankou attack[hold=200ms]; zankou skill;
            zankou attack:4; zankou skill; zankou burst; zankou burst;"
        );
        let result = crate::simulation::PreparedSourceScenario::new(
            &source,
            Default::default(),
            &Default::default(),
        )
        .unwrap()
        .run(42)
        .unwrap();
        assert!(
            result.report.execution_completed,
            "{:?}",
            result.report.halt_diagnostic
        );
        super::assert_native_replay(&result);
        result
    };
    let baseline = run("[]");
    let boosted = run("[2]");
    let direct = |r: &crate::simulation::SourceScenarioReport| {
        r.report
            .hit_receipts
            .iter()
            .filter(|h| {
                h.damage_effect_id.contains("UltraSkill") && !h.damage_effect_id.contains("Dot")
            })
            .map(|h| (h.damage_effect_id.clone(), h.non_crit_damage))
            .collect::<Vec<_>>()
    };
    let a = direct(&baseline);
    let b = direct(&boosted);
    assert_eq!(a.len(), 6);
    for ((name, amount), (_, enhanced)) in a.iter().zip(&b) {
        let factor = if name.contains("Force") {
            ExactFraction::new(5, 2).unwrap()
        } else {
            ExactFraction::new(1, 1).unwrap()
        };
        assert_eq!(*enhanced, amount.mul(factor).unwrap());
    }
}

#[test]
fn zankou_enhanced_skill_spreading_requires_exact_selected_a1() {
    for nodes in ["[]", "[1]", "[4]"] {
        let source = format!(
            "options duration=35 crit=fixed;
            zankou char lvl=80/80 awakening={nodes} talent=1,1,1 qte=1;
            target hp=1000000000 def=0 resist=0; active zankou;
            zankou attack:4; zankou skill; wait(4); zankou skill;"
        );
        let result = crate::simulation::PreparedSourceScenario::new(
            &source,
            Default::default(),
            &Default::default(),
        )
        .unwrap()
        .run(42)
        .unwrap();
        assert!(
            result.report.execution_completed,
            "{:?}",
            result.report.halt_diagnostic
        );
        super::assert_native_replay(&result);
        assert_eq!(
            result
                .report
                .trace
                .iter()
                .filter(|e| e.event_kind == "zankou_dot_spread")
                .count(),
            usize::from(nodes == "[1]")
        );
    }
}
