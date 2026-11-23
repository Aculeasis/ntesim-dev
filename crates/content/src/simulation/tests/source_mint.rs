use crate::simulation::{PreparedSourceScenario, SourceScenarioReport};
use serde_json::json;

fn run(source: &str) -> SourceScenarioReport {
    let prepared =
        PreparedSourceScenario::new(source, Default::default(), &Default::default()).unwrap();
    PreparedSourceScenario::from_export(
        &serde_json::from_value(serde_json::to_value(prepared.export()).unwrap()).unwrap(),
        &Default::default(),
    )
    .unwrap()
    .run(42)
    .unwrap()
}
fn completed(result: &SourceScenarioReport) {
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    super::assert_native_replay(result);
}
fn source(actions: &str) -> String {
    format!(
        "options duration=30 crit=random seed=42 stamina_capacity=200 initial_stamina=200
        stamina_use_efficiency=1 stamina_recovery_rate=0;
        mint char lvl=80/80 awakening=0 talent=8,8,8 qte=8;
        target hp=100000000 def=1080 resist=0;
        target hp=100000000 def=1080 resist=0; active mint; {actions}"
    )
}

#[test]
fn source_mint_equipped_ordinary_periods_keep_native_packets_and_energy_without_resets() {
    let script = include_str!("../../../../../examples/mint-native-cycles.nte");
    for (periods, dummies) in [(1, 1), (2, 2)] {
        let script = script
            .replace("repeat 2 {", &format!("repeat {periods} {{"))
            .replace("duration=120", &format!("duration={}", 60 * periods));
        let script = if dummies == 2 {
            script.replace(
                "active mint;",
                "target hp=24000000 def=1080 resist=0.2; active mint;",
            )
        } else {
            script
        };
        let result = run(&script);
        completed(&result);
        assert_eq!(
            result.report.quality,
            ntesim_contracts::Quality::BoundedHypothesis,
            "{:?}",
            result.report.model_assumptions
        );
        assert_eq!(result.report.hits_settled, 83 * periods * dummies);
        assert_eq!(result.report.hit_receipts[0].time_us, 154636);
        // (596 character +555 weapon)*1.75 +420 flat =2434.25 ATK;
        // level8 .178 * ATK *1.10 set bonus *.5 DEF *.8 RES.
        assert_eq!(result.request.actors[0].static_stats.base_atk, 1151);
        assert_eq!(
            result.report.hit_receipts[0].non_crit_damage,
            crate::simulation::ExactFraction::new(9532523, 50000).unwrap()
        );
        let actor = serde_json::to_value(&result.report.final_actor_states[0]).unwrap();
        assert_eq!(actor["spent_energy"], 100 * periods);
        assert_eq!(actor["energy"], 0);
        assert_eq!(actor["overflow_energy"], 31 * periods);
        assert_eq!(
            actor["charge_remainder"],
            serde_json::to_value(
                crate::simulation::ExactFraction::new(291 * periods as u128, 1000).unwrap()
            )
            .unwrap()
        );
        assert!(
            result
                .request
                .model_assumptions
                .contains_key("mint.ordinary_clock_v1")
        );
    }
}

#[test]
fn source_mint_two_held_inputs_keep_actual_end_completion_and_one_paid_account() {
    let result = run(&source(
        "repeat 2 {
        let start=.time; mint attack[hold=7s]; wait_until(start+8);
    }",
    ));
    completed(&result);
    assert_eq!(result.request.hold_inputs.len(), 2);
    assert_eq!(result.report.hits_settled, 144);
    assert_eq!(result.occurrences[0].completed_us, Some(6591672));
    let report = serde_json::to_value(&result.report).unwrap();
    assert_eq!(report["combat_stamina"]["total_spent"], json!("120/1"));
    assert_eq!(
        report["combat_stamina"]["spends"].as_array().unwrap().len(),
        60
    );
}

#[test]
fn source_mint_continuing_paid_controls_bind_real_command_indices_and_airborne_profiles() {
    let result = run(&source(
        "let approach=1; repeat 2 {
        mint dash[critical=true,window=skill], attack;
        mint jump;
        mint plunge[airtime=500ms,height_bonus=1/2,target=t1];
        mint attack[approach=approach];
        approach=1-approach; wait(1);
    }",
    ));
    completed(&result);
    assert_eq!(
        result
            .request
            .approach_inputs
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [4]
    );
    assert_eq!(
        result
            .request
            .plunge_inputs
            .keys()
            .copied()
            .collect::<Vec<_>>(),
        [3, 9]
    );
    for packet in [
        "GE_Player_Mint_PerfectEvadeAttack_Damage",
        "GE_Player_Mint_AirAttack1_Damage",
    ] {
        assert_eq!(
            result
                .report
                .hit_receipts
                .iter()
                .filter(|hit| hit.damage_effect_id == packet)
                .count(),
            4
        );
    }
    let report = serde_json::to_value(&result.report).unwrap();
    assert_eq!(report["combat_stamina"]["total_spent"], json!("30/1"));
    assert_eq!(result.report.hit_receipts[0].time_us, 567421);
}

#[test]
fn source_mint_parry_yields_at_delayed_back_skill_gate_without_a_free_riposte() {
    let result = run(&source(
        "repeat 2 {
        mint dash[parry=true,window=skill]; mint skill; wait(6);
    }",
    ));
    completed(&result);
    assert_eq!(result.request.parry_events.len(), 2);
    assert_eq!(result.occurrences[0].completed_us, Some(468540));
    assert_eq!(result.report.hit_receipts[0].time_us, 668718);
    assert!(
        result
            .report
            .hit_receipts
            .iter()
            .all(|hit| hit.damage_effect_id != "GE_Player_Mint_PerfectEvadeAttack_Damage")
    );
}

#[test]
fn source_mint_landing_hold_retains_flight_and_hold_bindings_for_each_repetition() {
    let result = run(&source(
        "repeat 2 {
        mint jump; mint plunge[airtime=500ms,height_bonus=0,hold=1700ms]; wait(1);
    }",
    ));
    completed(&result);
    assert_eq!(
        result
            .request
            .hold_inputs
            .keys()
            .copied()
            .collect::<Vec<_>>(),
        [1, 4]
    );
    assert_eq!(
        result
            .request
            .plunge_inputs
            .keys()
            .copied()
            .collect::<Vec<_>>(),
        [1, 4]
    );
    assert_eq!(result.report.hits_settled, 12);
    // Release during the second opener's connected pause retains its remaining
    // 31065us * .99 loss in End, then pays616639us recovery (ceil to us).
    assert_eq!(result.occurrences[1].completed_us, Some(2347394));
    let report = serde_json::to_value(&result.report).unwrap();
    assert!(
        report["combat_stamina"]["spends"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn source_mint_qualified_switches_retain_support_and_unfunded_entry_without_hidden_normals() {
    let result = run("options duration=15 crit=random;
        zero char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_cycle=100;
        mint char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_cycle=100;
        hotori char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_cycle=100;
        target hp=100000000 def=1080 resist=0; active zero;
        mint swap; wait_until(2); hotori swap;
        wait_until(5); mint swap; wait_until(7); zero swap;
        wait_until(10); mint swap; wait_until(10.5); mint attack;
    ");
    completed(&result);
    let support = result
        .report
        .hit_receipts
        .iter()
        .filter(|hit| hit.damage_effect_id == "GE_Player_Mint_QTE1_Damage")
        .collect::<Vec<_>>();
    assert_eq!(
        support.iter().map(|hit| hit.time_us).collect::<Vec<_>>(),
        [739424, 5739424]
    );
    let entry = result
        .report
        .hit_receipts
        .iter()
        .find(|hit| hit.damage_effect_id == "GE_Player_Mint_Melee1_Damage_New")
        .unwrap();
    assert_eq!(entry.time_us, 10699890);
}

#[test]
fn source_mint_flight_and_approach_failures_keep_typed_admission_and_utf16_ranges() {
    let result = run(&source("mint plunge[airtime=500ms,height_bonus=0];"));
    assert!(!result.report.success && !result.report.execution_completed);
    let report = serde_json::to_value(&result.report).unwrap();
    assert_eq!(report["rejected_action"]["reason"]["kind"], "other");
    assert!(
        result
            .diagnostic
            .as_ref()
            .unwrap()
            .message
            .contains("new admitted Jump")
    );
    super::assert_native_replay(&result);
    for (action, token) in [
        ("mint attack[approach=2]", "2"),
        ("mint skill[approach=false]", "false"),
    ] {
        let source = format!("# 🍃\n{}", source(&format!("if 0 {{{action};}}")));
        let diagnostics =
            PreparedSourceScenario::new(&source, Default::default(), &Default::default())
                .err()
                .unwrap();
        assert_eq!(
            diagnostics[0].code,
            crate::dsl::validation::DiagnosticCode::Configuration
        );
        let loc = &diagnostics[0].loc;
        assert_eq!(
            String::from_utf16(
                &source
                    .encode_utf16()
                    .skip(loc.offset)
                    .take(loc.length)
                    .collect::<Vec<_>>()
            )
            .unwrap(),
            token
        );
    }
}
