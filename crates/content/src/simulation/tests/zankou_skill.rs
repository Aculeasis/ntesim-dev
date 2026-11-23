use super::*;
use serde_json::{Value, json};

#[path = "zankou_skill_aoe.rs"]
mod aoe;

fn request(normal: u32, skill: u32, duration: u64, rotation: &str) -> GameScenarioRequest {
    let energy = if rotation.contains("burst") { 120 } else { 0 };
    let source = format!(
        "options duration={duration} crit=fixed;
        zankou char lvl=80/80 awakening=0 talent={normal},{skill},1 qte={skill} start_energy={energy};
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; target hp=100000000 def=0 resist=0;
        active zankou; {rotation}"
    );
    let response = crate::simulation::run_source(
        &crate::dsl::compiler::SourceRequest {
            source,
            limits: Default::default(),
        },
        &Default::default(),
    )
    .unwrap();
    assert!(response.execution.is_some(), "{:?}", response.diagnostics);
    let execution = response.execution.unwrap();
    assert!(
        execution.report.execution_completed,
        "{:?}",
        execution.report.halt_diagnostic
    );
    let mut req = execution.request;
    // Independent energy ledger oracle uses an explicit 120-unit cap.
    req.actors[0].energy_capacity = 120;
    let stats = &mut req.actors[0].static_stats;
    stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    stats.base_atk = 1000;
    stats.crit_damage_bonus = Ratio::zero();
    stats.crit_rate = Ratio::zero();
    stats.damage_bonus = Ratio::zero();
    stats.elemental_bonus = Ratio::zero();
    req
}

fn run(req: &GameScenarioRequest) -> Value {
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}

#[test]
fn selected_a6_changes_only_the_actual_enhanced_soulcross_final_contact() {
    let run = |nodes: &str| {
        let source = format!(
            "options duration=40 crit=fixed;zankou char lvl=80/80 awakening={nodes} talent=1,1,1 qte=1;target hp=100000000 def=0 resist=0;target hp=100000000 def=0 resist=0;active zankou;zankou attack:4;zankou skill;zankou attack:4;zankou skill;"
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
    let a = run("[]");
    let b = run("[6]");
    assert_eq!(a.report.hit_receipts.len(), b.report.hit_receipts.len());
    let mut finals = 0;
    for (a, b) in a.report.hit_receipts.iter().zip(&b.report.hit_receipts) {
        let factor = if a.damage_effect_id == "GE_Player_Zankou_Skill3_2_Damage" {
            finals += 1;
            ExactFraction::new(49, 25).unwrap()
        } else {
            ExactFraction::new(1, 1).unwrap()
        };
        // Ten Heartwrench stacks add1.20 to the existing.25 Hunt zone.
        assert_eq!(
            b.non_crit_damage,
            a.non_crit_damage.mul(factor).unwrap(),
            "{} at{}",
            a.damage_effect_id,
            a.time_us
        );
    }
    assert_eq!(finals, 2);
}

fn packets<'a>(report: &'a Value, effect: &str) -> Vec<&'a Value> {
    all_packets(report, effect)
        .into_iter()
        .filter(|hit| hit["target_index"] == 0)
        .collect()
}

fn all_packets<'a>(report: &'a Value, effect: &str) -> Vec<&'a Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == effect)
        .collect()
}

#[test]
fn ordinary_skill_uses_current_reality_packets_without_invented_heartwrench() {
    for (normal, skill, lead, finish) in [(8, 1, 225, 600), (1, 8, 386, 1028)] {
        let report = run(&request(normal, skill, 35, "zankou skill;"));
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let hits = packets(&report, "GE_Player_Zankou_Skill2_Damage");
        assert_eq!(hits.len(), 4);
        for (hit, time) in hits.iter().zip([122_519, 226_651, 345_053, 453_684]) {
            assert_eq!(hit["time_us"], time);
            assert_eq!(hit["non_crit_damage"], format!("{lead}/1"));
        }
        assert_eq!(
            packets(&report, "GE_Player_Zankou_Skill2_1_Damage")[0]["time_us"],
            666_135
        );
        assert_eq!(
            packets(&report, "GE_Player_Zankou_Skill2_1_Damage")[0]["non_crit_damage"],
            format!("{finish}/1")
        );
        assert!(packets(&report, "GE_Player_Zankou_DotDamage").is_empty());
        // Four*.57+1.52=3.8 energy, independent of talent and recipient count.
        assert_eq!(report["final_actor_states"][0]["granted_energy"], 3);
        assert_eq!(report["final_actor_states"][0]["charge_remainder"], "4/5");
        assert_eq!(
            all_packets(&report, "GE_Player_Zankou_Skill2_Damage").len(),
            8
        );
    }
}

fn transitions(report: &Value) -> Vec<u64> {
    report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "zankou_form_transition")
        .map(|e| e["time_us"].as_u64().unwrap())
        .collect()
}

#[test]
fn current_forms_complete_enhanced_return_and_discard_illusion_on_swap() {
    let full = run(&request(1, 1, 20, "zankou skill; wait(4); zankou skill;"));
    assert_eq!(
        full["execution_completed"], true,
        "{} {}",
        full["halt_diagnostic"], full["rejected_action"]
    );
    assert_eq!(transitions(&full), [496_507, 7_065_304]);
    assert_eq!(
        packets(&full, "GE_Player_Zankou_Skill3_2_Damage")[0]["time_us"],
        6_885_950
    );
    assert!(packets(&full, "GE_Player_Zankou_Skill4_2_Damage").is_empty());
    let swap = run(&request(
        1,
        1,
        20,
        "zankou skill; mint swap; wait(5); zankou swap; zankou skill;",
    ));
    assert_eq!(swap["execution_completed"], true);
    assert_eq!(packets(&swap, "GE_Player_Zankou_Skill2_Damage").len(), 8);
    assert!(packets(&swap, "GE_Player_Zankou_Skill3_Damage").is_empty());
    assert!(packets(&swap, "GE_Player_Zankou_Skill4_Damage").is_empty());
}

#[test]
fn form_half_open_deadline_and_admitted_return_survive_natural_expiry() {
    for (at, entry) in [(8_496_506, false), (8_496_507, true)] {
        let mut req = request(1, 1, 20, "zankou skill; wait(8); zankou skill;");
        let last = req
            .actions
            .iter_mut()
            .rev()
            .find(|a| matches!(a, GameActionCommand::InvokeAbility { .. }))
            .unwrap();
        if let GameActionCommand::InvokeAbility { timestamp_us, .. } = last {
            *timestamp_us = at;
        }
        // Remove the authored wait; native fixtures retain explicit event times.
        req.actions
            .retain(|a| !matches!(a, GameActionCommand::Wait { .. }));
        req.input_time_contexts.clear();
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(
            packets(&report, "GE_Player_Zankou_Skill2_Damage").len(),
            if entry { 8 } else { 4 }
        );
        assert_eq!(
            packets(&report, "GE_Player_Zankou_Skill3_2_Damage").len(),
            usize::from(!entry)
        );
        if !entry {
            assert_eq!(transitions(&report), [496_507, at + 2_039_354]);
        }
    }
}

#[test]
fn interrupted_entry_cancels_unborn_transition_and_keeps_committed_cooldown() {
    let mut req = request(1, 1, 20, "zankou skill;");
    req.interrupt_events
        .push(crate::simulation::GameActorInterruptEvent {
            timestamp_us: 496_506,
            actor_index: 0,
            armor_damage: None,
        });
    let report = run(&req);
    assert!(transitions(&report).is_empty());
    assert_eq!(packets(&report, "GE_Player_Zankou_Skill2_Damage").len(), 4);
    assert!(packets(&report, "GE_Player_Zankou_Skill2_1_Damage").is_empty());
    assert_eq!(
        report["interrupt_receipts"][0]["skill_cooldown_before_us"],
        5_000_000
    );
    assert_eq!(
        report["interrupt_receipts"][0]["skill_cooldown_after_us"],
        5_000_000
    );
    req.interrupt_events[0].timestamp_us = 496_507;
    assert_eq!(transitions(&run(&req)), [496_507]);
}

#[test]
fn enhanced_entry_permission_belongs_to_the_current_reality_section() {
    for (gap, enhanced) in [("", true), ("wait(0.000001);", false)] {
        let report = run(&request(
            1,
            1,
            25,
            &format!(
                "zankou attack; zankou attack; zankou attack; zankou attack; {gap} zankou skill;"
            ),
        ));
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(
            packets(&report, "GE_Player_Zankou_Skill1_Damage").len(),
            if enhanced { 4 } else { 0 }
        );
        assert_eq!(
            packets(&report, "GE_Player_Zankou_Skill2_Damage").len(),
            if enhanced { 0 } else { 4 }
        );
    }
}

#[test]
fn current_normal_chains_preserve_independent_totals_and_causal_resources() {
    for (rotation, prefix, count, total, energy, remainder) in [
        (
            "zankou attack; zankou attack; zankou attack; zankou attack; zankou attack;",
            "GE_Player_Zankou_Melee",
            17,
            6259,
            0,
            "",
        ),
        (
            "zankou skill; zankou attack; zankou attack; zankou attack; zankou attack;",
            "GE_Player_Zankou_MagicMelee",
            11,
            4990,
            20,
            "399/1000",
        ),
    ] {
        let baseline = request(1, 1, 35, rotation);
        let mut resources = None;
        for targets in [1, 2] {
            let mut req = baseline.clone();
            req.targets.truncate(targets);
            let report = run(&req);
            assert_eq!(
                report["execution_completed"], true,
                "{}",
                report["halt_diagnostic"]
            );
            let hits: Vec<_> = report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|h| {
                    h["target_index"] == 0
                        && h["damage_effect_id"].as_str().unwrap().starts_with(prefix)
                })
                .collect();
            assert_eq!(hits.len(), count);
            let sum = hits
                .iter()
                .fold(ExactFraction::new(0, 1).unwrap(), |sum, h| {
                    sum.add(serde_json::from_value(h["non_crit_damage"].clone()).unwrap())
                        .unwrap()
                });
            assert_eq!(
                sum,
                if prefix == "GE_Player_Zankou_Melee" {
                    ExactFraction::new(31295, 4).unwrap()
                } else {
                    ExactFraction::new(total, 1).unwrap()
                }
            );
            for hit in hits {
                let packet = &super::support::hit_metadata(&report, hit)["packet"];
                assert_eq!(
                    packet["native_damage_source_category"],
                    "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"
                );
                if prefix.contains("Magic") {
                    assert!(
                        packet["semantic_tags"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|t| t == "Ability.Player.Nanally.XieTongDamage")
                    );
                }
            }
            let state = &report["final_actor_states"][0];
            let ledger = (
                state["granted_energy"].clone(),
                state["charge_remainder"].clone(),
                state["cycle"].clone(),
            );
            if let Some(expected) = &resources {
                assert_eq!(&ledger, expected);
            } else {
                resources = Some(ledger);
            }
            if energy > 0 {
                assert_eq!(state["granted_energy"], energy);
                assert_eq!(state["charge_remainder"], remainder);
                // Four Illusion stages:1.018+5.544+5.486+15.653, plus6.3 entry.
                assert_eq!(state["cycle"]["granted"], 34);
                assert_eq!(state["cycle"]["carry"], "1/1000");
            }
        }
    }
}

#[test]
fn form_specific_held_attacks_use_current_normal_prerequisites_and_native_a_talent() {
    for (rotation, effect, total, enhanced) in [
        (
            "zankou attack; zankou attack[hold=0.2]; zankou skill;",
            "GE_Player_Zankou_Branch",
            2178,
            true,
        ),
        (
            "zankou skill; zankou attack; zankou attack[hold=0.2];",
            "GE_Player_Zankou_MagicBranch",
            2138,
            false,
        ),
    ] {
        let report = run(&request(1, 1, 35, rotation));
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let sum = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| {
                h["target_index"] == 0
                    && h["damage_effect_id"].as_str().unwrap().starts_with(effect)
            })
            .fold(ExactFraction::new(0, 1).unwrap(), |sum, h| {
                sum.add(serde_json::from_value(h["non_crit_damage"].clone()).unwrap())
                    .unwrap()
            });
        assert_eq!(
            sum,
            if enhanced {
                ExactFraction::new(5445, 2).unwrap()
            } else {
                ExactFraction::new(total, 1).unwrap()
            }
        );
        assert_eq!(
            packets(&report, "GE_Player_Zankou_Skill1_Damage").len(),
            if enhanced { 4 } else { 0 }
        );
    }
    let source = "options duration=20 crit=fixed; zankou char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=100000000 def=0 resist=0; active zankou; zankou attack[hold=0.2];";
    let failed = crate::simulation::run_source(
        &crate::dsl::compiler::SourceRequest {
            source: source.into(),
            limits: Default::default(),
        },
        &Default::default(),
    )
    .unwrap();
    let failed = serde_json::to_value(failed.execution.unwrap().report).unwrap();
    assert_eq!(failed["execution_completed"], false);
    assert!(
        failed["halt_diagnostic"]
            .as_str()
            .unwrap()
            .contains("Normal1 or Normal2")
    );
}

#[test]
fn secondary_inputs_earn_riposte_plunge_and_support_through_shared_admission() {
    let source="options duration=20 crit=fixed stamina_capacity=200 initial_stamina=200 stamina_use_efficiency=1 stamina_recovery_rate=0;
        zankou char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; target hp=100000000 def=0 resist=0;
        active zankou; zankou dash[critical=true]; zankou attack;
        zankou jump; zankou plunge[airtime=500ms,height_bonus=1/2]; zankou attack;";
    let result = crate::simulation::PreparedSourceScenario::new(
        source,
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
    let report = serde_json::to_value(&result.report).unwrap();
    assert_eq!(
        all_packets(&report, "GE_Player_Zankou_PerfectEvadeAttack_Damage").len(),
        2
    );
    assert_eq!(
        packets(&report, "GE_Player_Zankou_PerfectEvadeAttack_Damage")[0]["time_us"],
        1_180_713
    );
    assert_eq!(
        packets(&report, "GE_Player_Zankou_AirAttack_Damage")[0]["time_us"],
        2_283_462
    );
    assert_eq!(report["combat_stamina"]["total_spent"], "15/1");
    let support = "options duration=20 crit=fixed;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        zankou char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active mint; zankou swap;";
    let result = crate::simulation::PreparedSourceScenario::new(
        support,
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
    let report = serde_json::to_value(&result.report).unwrap();
    assert_eq!(
        packets(&report, "GE_Player_Zankou_QTE_Damage")[0]["time_us"],
        791_367
    );
    let unearned = source.replace(
        "zankou dash[critical=true]; zankou attack;\n        zankou jump;",
        "",
    );
    let failed = crate::simulation::PreparedSourceScenario::new(
        &unearned,
        Default::default(),
        &Default::default(),
    )
    .unwrap()
    .run(42)
    .unwrap();
    assert!(!failed.report.execution_completed);
    assert!(failed.report.hit_receipts.is_empty());
}

#[test]
fn derived_bloodfeast_and_free_inferno_preserve_canonical_identity_and_real_paid_cost() {
    let rotation = "zankou skill; wait(4); zankou skill; zankou burst; zankou burst;";
    let req = request(1, 1, 35, rotation);
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(
        packets(&report, "GE_Player_Zankou_MagicUltraSkill1_Damage").len(),
        3
    );
    assert_eq!(
        packets(&report, "GE_Player_Zankou_MagicUltraSkill2_Damage").len(),
        1
    );
    assert_eq!(
        packets(&report, "GE_Player_Zankou_ForceUltraSkill1_Damage").len(),
        1
    );
    assert_eq!(
        packets(&report, "GE_Player_Zankou_ForceUltraSkill2_Damage").len(),
        1
    );
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 120);
    let bursts: Vec<_> = req
        .actions
        .iter()
        .filter_map(|a| match a {
            GameActionCommand::InvokeAbility {
                ability_id,
                energy_cost,
                cooldown_us,
                ..
            } if ability_id == "GA_Zankou_UltraSkill" => Some((*energy_cost, *cooldown_us)),
            _ => None,
        })
        .collect();
    assert_eq!(bursts, [(120, 20_000_000), (0, 0)]);
    let source = format!(
        "options duration=40 crit=fixed; zankou char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=120; target hp=100000000 def=0 resist=0; active zankou; {rotation} zankou burst;"
    );
    let result = crate::simulation::PreparedSourceScenario::new(
        &source,
        Default::default(),
        &Default::default(),
    )
    .unwrap()
    .run(42)
    .unwrap();
    assert!(!result.report.execution_completed);
    assert_eq!(
        result
            .report
            .hit_receipts
            .iter()
            .filter(|h| h.damage_effect_id == "GE_Player_Zankou_ForceUltraSkill1_Damage")
            .count(),
        1
    );
}

#[test]
fn free_inferno_half_open_expiry_interrupt_and_field_exit_keep_the_paid_deadline() {
    let prefix = "zankou skill; wait(4); zankou skill; zankou burst;";
    let base = request(1, 1, 40, prefix);
    let pair = request(1, 1, 40, &format!("{prefix} zankou burst;"));
    let free = pair.actions.last().unwrap().clone();
    for (at, accepted) in [(17_991_073, true), (17_991_074, false)] {
        let mut req = base.clone();
        req.input_time_contexts.clear();
        let mut command = free.clone();
        if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut command {
            *timestamp_us = at;
        }
        req.actions.push(command);
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], accepted,
            "{}",
            report["halt_diagnostic"]
        );
        if !accepted {
            assert_eq!(report["rejected_action"]["reason"]["until_us"], 32_991_074);
        }
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 120);
    }
    let mut interrupted = pair.clone();
    interrupted.input_time_contexts.clear();
    interrupted
        .interrupt_events
        .push(crate::simulation::GameActorInterruptEvent {
            timestamp_us: 12_969_833,
            actor_index: 0,
            armor_damage: None,
        });
    let report = run(&interrupted);
    assert_eq!(report["execution_completed"], false);
    assert!(packets(&report, "GE_Player_Zankou_ForceUltraSkill1_Damage").is_empty());
    assert_eq!(report["rejected_action"]["reason"]["until_us"], 32_969_833);
    let mut swapped = base;
    swapped.input_time_contexts.clear();
    swapped.actions.extend([
        GameActionCommand::SwitchActor {
            timestamp_us: 12_991_074,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 15_000_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
    ]);
    let mut command = free;
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut command {
        *timestamp_us = 17_000_000;
    }
    swapped.actions.push(command);
    let report = run(&swapped);
    assert_eq!(report["execution_completed"], false);
    assert!(packets(&report, "GE_Player_Zankou_ForceUltraSkill1_Damage").is_empty());
    assert_eq!(report["rejected_action"]["reason"]["until_us"], 32_991_074);
}

#[test]
fn qualifying_illusion_spread_caps_both_independent_trains_and_preserves_vile_ash_phase() {
    let rotation =
        "zankou attack:4; zankou skill; wait(4); zankou skill; zankou burst; zankou burst;
        zankou attack; zankou attack[hold=200ms]; while .zankou.skill.cd>0 {wait(.zankou.skill.cd);} zankou skill; zankou attack; mint swap;";
    let report = run(&request(8, 1, 65, rotation));
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    let ticks = packets(&report, "GE_Player_Zankou_DotUltraDamage");
    // Both cinematics retain the original phase. The short Hunt modifier
    // expires before the third tick; spreading doubles stacks without
    // restarting the next tick's remaining 99,105us.
    assert_eq!(ticks[0]["time_us"], 24_146_579);
    assert!(ticks[..2].iter().all(|h| h["non_crit_damage"] == "375/4"));
    assert_eq!(ticks[2]["time_us"], 26_146_579);
    assert_eq!(ticks[2]["non_crit_damage"], "75/1");
    assert_eq!(ticks[3]["time_us"], 27_146_579);
    assert!(ticks[3..].iter().all(|h| h["non_crit_damage"] == "150/1"));
    assert!(
        ticks
            .windows(2)
            .all(|pair| pair[1]["time_us"].as_u64().unwrap()
                - pair[0]["time_us"].as_u64().unwrap()
                == 1_000_000)
    );
    let heart = packets(&report, "GE_Player_Zankou_DotDamage");
    assert_eq!(heart.last().unwrap()["non_crit_damage"], "260/1");
    let spreads = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "zankou_dot_spread")
        .collect::<Vec<_>>();
    assert_eq!(spreads.len(), 1);
    assert_eq!(spreads[0]["data"]["count"], 2);
    assert_eq!(spreads[0]["time_us"], 27_047_474);
}

#[test]
fn spread_preserves_four_carriers_and_the_original_nightmare_owner_without_recursion() {
    let source = "options duration=55 crit=fixed;
        adler char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        lacrimosa char lvl=80/80 awakening=[4] talent=1,1,1 qte=1 start_cycle=100;
        zankou char lvl=80/80 awakening=[4] talent=1,1,1 qte=1 start_energy=120;
        target hp=100000000 def=0 resist=0; target hp=100000000 def=0 resist=0;
        active adler; adler skill; lacrimosa swap; lacrimosa skill; zankou swap;
        zankou burst; zankou skill; zankou attack;";
    let result = crate::simulation::PreparedSourceScenario::new(
        source,
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
    let report = serde_json::to_value(result.report).unwrap();
    let spreads = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "zankou_dot_spread")
        .collect::<Vec<_>>();
    assert_eq!(spreads.len(), 1);
    assert_eq!(spreads[0]["data"]["count"], 4);
    for tick in all_packets(&report, "GE_Player_Lacrimosa_Blood_Damage") {
        assert_eq!(tick["actor_index"], 1);
    }
    assert!(
        all_packets(&report, "GE_Player_Zankou_DotUltraDamage")
            .iter()
            .any(|h| h["time_us"].as_u64().unwrap() > 30_000_000)
    );
}

#[test]
fn illusion_spread_refreshes_adler_attached_dot_without_restarting_its_native_phase() {
    let source =
        "options duration=20 crit=fixed; adler char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        zankou char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; target hp=100000000 def=0 resist=0;
        active adler; adler skill; zankou swap; zankou skill; zankou attack;";
    let result = crate::simulation::PreparedSourceScenario::new(
        source,
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
    let report = serde_json::to_value(result.report).unwrap();
    let ticks = packets(&report, "GE_Player_Adler_Skill_Damage");
    assert_eq!(ticks.len(), 12);
    for (i, tick) in ticks.iter().enumerate() {
        assert_eq!(tick["time_us"], (i as u64 + 1) * 1_000_000);
        assert_eq!(tick["actor_index"], 0);
    }
    let spread = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["event_kind"] == "zankou_dot_spread")
        .unwrap();
    assert_eq!(spread["data"]["count"], 2);
}

#[test]
fn overlapping_hunt_and_delusion_contribute_once_to_live_dot_damage_and_critical_damage() {
    let report = run(&request(
        8,
        1,
        35,
        "zankou attack:4; zankou skill; zankou attack; wait(3); zankou skill; mint swap;",
    ));
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let ticks = packets(&report, "GE_Player_Zankou_DotDamage");
    // Ten stacks*.026*1000*1.25 Hunt, then1.25 critical multiplier from Delusion.
    assert_eq!(ticks[0]["time_us"], 6_999_438);
    assert_eq!(ticks[0]["non_crit_damage"], "325/1");
    assert_eq!(ticks[0]["crit_damage"], "1625/4");
}

#[test]
fn a4_uses_paid_readiness_and_bloodfeast_discards_the_current_illusion() {
    let source = "options duration=25 crit=fixed;
        zankou char lvl=80/80 awakening=[4] talent=1,1,1 qte=1 start_energy=120;
        target hp=100000000 def=0 resist=0; active zankou; zankou skill;
        if !.zankou.illusion.active {let assertion_failure=1/0;} zankou burst;
        if .zankou.illusion.active || !.zankou.reality.active {let assertion_failure=1/0;} zankou burst;";
    let result = crate::simulation::PreparedSourceScenario::new(
        source,
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
    assert_eq!(result.report.final_actor_states[0].spent_energy, 120);
    assert_eq!(
        result
            .report
            .hit_receipts
            .iter()
            .filter(|h| h.damage_effect_id == "GE_Player_Zankou_MagicUltraSkill2_Damage")
            .count(),
        1
    );
    assert_eq!(
        result
            .report
            .hit_receipts
            .iter()
            .filter(|h| h.damage_effect_id == "GE_Player_Zankou_ForceUltraSkill2_Damage")
            .count(),
        1
    );
}

#[test]
fn clock_cancellation_preserves_launched_dots_and_exact_contact_boundary() {
    let mut req = request(1, 1, 40, "zankou attack:4; zankou skill;");
    req.interrupt_events
        .push(crate::simulation::GameActorInterruptEvent {
            timestamp_us: 6_000_000,
            actor_index: 0,
            armor_damage: None,
        });
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let ticks = packets(&report, "GE_Player_Zankou_DotDamage");
    assert_eq!(ticks.len(), 29);
    assert_eq!(ticks[0]["time_us"], 6_999_438);
    assert_eq!(ticks[0]["non_crit_damage"], "375/4");
    assert!(packets(&report, "GE_Player_Zankou_Skill1_1_Damage").is_empty());
    for (at, contacts) in [(342_266, 1), (342_267, 2)] {
        let mut req = request(1, 1, 5, "zankou attack;");
        req.interrupt_events
            .push(crate::simulation::GameActorInterruptEvent {
                timestamp_us: at,
                actor_index: 0,
                armor_damage: None,
            });
        let report = run(&req);
        assert_eq!(
            report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|h| h["target_index"] == 0)
                .count(),
            contacts
        );
    }
    let mut early = request(1, 1, 15, "zankou skill;");
    early.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 496_506,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    let report = run(&early);
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    assert!(transitions(&report).is_empty());
    assert!(packets(&report, "GE_Player_Zankou_Skill2_1_Damage").is_empty());
}

#[test]
fn support_switch_gate_uses_contact_pause_and_its_exact_native_boundary() {
    let source="options duration=20 crit=fixed; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        zankou char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=100000000 def=0 resist=0; active mint; zankou swap;";
    let base = crate::simulation::PreparedSourceScenario::new(
        source,
        Default::default(),
        &Default::default(),
    )
    .unwrap()
    .run(42)
    .unwrap()
    .request;
    for (at, accepted) in [(1_335_889, false), (1_335_890, true)] {
        let mut req = base.clone();
        req.actions.push(GameActionCommand::SwitchActor {
            timestamp_us: at,
            target_actor_index: 0,
            cooldown_us: 0,
        });
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], accepted,
            "{}",
            report["halt_diagnostic"]
        );
        if !accepted {
            assert_eq!(report["rejected_action"]["reason"]["until_us"], 1_335_890);
        }
    }
}

#[test]
fn heartwrench_reapplication_preserves_phase_and_state_through_two_cycles() {
    let report = run(&request(
        8,
        8,
        60,
        "zankou attack:4; zankou skill; wait(12); zankou attack:4; zankou skill; mint swap;",
    ));
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let ticks = packets(&report, "GE_Player_Zankou_DotDamage");
    // Applications at5.999438 and24.362771:18 five-stack ticks, then30 ten-stack ticks.
    assert_eq!(ticks.len(), 48);
    let mut total = ExactFraction::new(0, 1).unwrap();
    for (i, hit) in ticks.iter().enumerate() {
        assert_eq!(hit["time_us"], 6_999_438 + i as u64 * 1_000_000);
        let damage = if i < 18 {
            if !(7..12).contains(&i) {
                ExactFraction::new(325, 2).unwrap()
            } else {
                ExactFraction::new(130, 1).unwrap()
            }
        } else {
            ExactFraction::new(260, 1).unwrap()
        };
        total = total.add(damage).unwrap();
        assert_eq!(
            serde_json::from_value::<ExactFraction>(hit["non_crit_damage"].clone()).unwrap(),
            damage
        );
    }
    assert_eq!(total, ExactFraction::new(21125, 2).unwrap());
    assert_eq!(report["final_buff_state"]["active_actor_index"], 1);
}

#[test]
fn heartwrench_and_burst_keep_distinct_native_classes_stacks_and_targets() {
    let report = run(&request(
        8,
        1,
        60,
        "zankou attack:4; zankou skill; wait(4); zankou skill; zankou burst; mint swap;",
    ));
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let normal = all_packets(&report, "GE_Player_Zankou_DotDamage");
    let burst = all_packets(&report, "GE_Player_Zankou_DotUltraDamage");
    assert_eq!(normal.len(), 72);
    assert_eq!(burst.len(), 58);
    assert_eq!(normal[0]["non_crit_damage"], "325/2");
    assert!(normal.iter().any(|h| h["non_crit_damage"] == "260/1"));
    assert!(burst.iter().all(|h| h["non_crit_damage"] == "75/1"));
    for hit in normal {
        assert_eq!(
            super::support::hit_metadata(&report, hit)["binding"]["packet_ga_name"],
            "GA_Zankou_Melee"
        );
    }
    for hit in burst {
        assert_eq!(
            super::support::hit_metadata(&report, hit)["binding"]["packet_ga_name"],
            "GA_Zankou_UltraSkill"
        );
    }
}

#[test]
fn skill_cooldown_is_five_real_seconds_with_exact_boundary() {
    for (at, accepted) in [(4_999_999, false), (5_000_000, true)] {
        let mut req = request(1, 1, 20, "zankou skill; wait(5); zankou skill;");
        req.actions
            .retain(|a| !matches!(a, GameActionCommand::Wait { .. }));
        req.input_time_contexts.clear();
        if let GameActionCommand::InvokeAbility { timestamp_us, .. } =
            req.actions.last_mut().unwrap()
        {
            *timestamp_us = at;
        }
        let report = run(&req);
        assert_eq!(report["execution_completed"], accepted);
        if !accepted {
            assert_eq!(
                report["rejected_action"]["reason"],
                json!({"kind":"cooldown","until_us":5_000_000})
            );
        }
    }
}

#[test]
fn zero_damage_periodics_continue_without_energy_or_equipment_procs_and_owner_death_cancels() {
    let mut req = request(8, 1, 40, "zankou attack:4; zankou skill;");
    req.input_time_contexts.clear();
    // Immune contacts add no pause: submit Skill at Normal4's unpaused end.
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = req.actions.last_mut().unwrap() {
        *timestamp_us = 5_317_583;
    }
    req.actors[0].weapon_id = "weapon:fork_NestBird".into();
    req.actors[0].gear_procs.push(GearProcConfig::NestBirdGaze {
        damage_reduction: Ratio {
            numerator: 9,
            denominator: 50,
        },
        duration_us: 20_000_000,
    });
    for target in &mut req.targets {
        target.resistance = Ratio::one();
    }
    let report = run(&req);
    assert_eq!(packets(&report, "GE_Player_Zankou_DotDamage").len(), 29);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 0);
    assert!(
        crate::simulation::tests::support::final_buffs(&report)
            .iter()
            .all(|buff| !buff["id"].as_str().unwrap().contains("nest_bird"))
    );
    assert!(report["trace"].as_array().unwrap().iter().all(|event| {
        !event["event_kind"]
            .as_str()
            .unwrap_or("")
            .starts_with("nest_bird")
    }));
    for target in &mut req.targets {
        target.resistance = Ratio::zero();
    }
    req.actors[0].weapon_id.clear();
    req.actors[0].gear_procs.clear();
    let id = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    req.incoming_damage_events
        .push(crate::simulation::GameActorDamageEvent {
            timestamp_us: 7_000_000,
            actor_index: 0,
            delivered_damage: req.actors[0].initial_hp,
        });
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(packets(&report, "GE_Player_Zankou_DotDamage").len(), 1);
    assert_eq!(report["final_actor_states"][0]["hp"], 0);
    assert!(
        packets(&report, "GE_Player_Zankou_DotDamage")
            .iter()
            .all(|hit| hit["time_us"].as_u64().unwrap() < 7_000_000)
    );
}

#[test]
fn heartwrench_refresh_after_last_old_tick_preserves_phase_and_expiry_drops_old_stacks() {
    for (wait, expected, first_new, damage) in [
        ("23.136667", 59, 35_999_438, 260),
        ("24.136667", 58, 37_499_438, 130),
    ] {
        let report = run(&request(
            8,
            1,
            75,
            &format!(
                "zankou attack:4; zankou skill; wait({wait}); zankou attack:4; zankou skill; mint swap;"
            ),
        ));
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let ticks = packets(&report, "GE_Player_Zankou_DotDamage");
        assert_eq!(ticks.len(), expected);
        assert_eq!(ticks[29]["time_us"], first_new);
        assert_eq!(ticks[29]["non_crit_damage"], format!("{damage}/1"));
    }
}

#[test]
fn continuing_heartwrench_caps_at_ten_with_live_energy_carry_and_overflow() {
    let rotation = "zankou attack:4; zankou skill; wait(12);".repeat(8);
    let report = run(&request(8, 1, 190, &rotation));
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let ticks = packets(&report, "GE_Player_Zankou_DotDamage");
    assert_eq!(ticks.last().unwrap()["time_us"], 163_999_438);
    assert_eq!(ticks.last().unwrap()["non_crit_damage"], "260/1");
    assert!(ticks.iter().all(|hit| {
        let d: ExactFraction = serde_json::from_value(hit["non_crit_damage"].clone()).unwrap();
        d.numerator <= d.denominator * 325
    }));
    // Eight(12.287 Normal+3.8 enhanced-entry) Charge, no DoT Charge.
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 128);
    assert_eq!(report["final_actor_states"][0]["overflow_energy"], 8);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        "87/125"
    );
}
