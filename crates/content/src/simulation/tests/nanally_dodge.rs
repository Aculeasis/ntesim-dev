use super::*;
use crate::simulation::tests::support::formula_damage_total;

fn run(rotation: &str, duration: u64) -> crate::simulation::SourceScenarioReport {
    let source = format!(
        "options duration={duration} crit=random stamina_capacity=60 initial_stamina=60
        stamina_use_efficiency=1 stamina_recovery_rate=0;
        nanally char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0;
        target hp=100000000 def=1080 resist=0; active nanally; {rotation}"
    );
    let result = PreparedSourceScenario::new(&source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    result
}

#[test]
fn nanally_dodge_preserves_next_normal_and_rebases_its_idle_window_without_refunding_stamina() {
    let result = run("nanally attack; nanally dash; nanally attack;", 5);
    assert_eq!(result.report.hits_settled, 2);
    assert_eq!(
        result.report.hit_receipts[1].damage_effect_id,
        "GE_Player_Nanally_Melee2_Damage"
    );
    assert_eq!(result.report.hit_receipts[1].time_us, 1_320_884);
    assert_eq!(result.report.final_actor_states[0].granted_energy, 2);
    assert_eq!(
        result.report.final_actor_states[0].charge_remainder,
        Some(ExactFraction::new(63, 100).unwrap())
    );
    let report = serde_json::to_value(&result.report).unwrap();
    assert_eq!(
        report["combat_stamina"]["total_spent"],
        serde_json::json!("15/1")
    );
    let result = run(
        "nanally attack:3; wait_until(2.9943); nanally dash; nanally attack;",
        6,
    );
    let fourth: Vec<_> = result
        .report
        .hit_receipts
        .iter()
        .filter(|hit| hit.damage_effect_id == "GE_Player_Nanally_Melee6_Damage")
        .collect();
    assert_eq!(fourth.len(), 7);
    assert_eq!(fourth[0].time_us, 3_843_778); // The old Normal3 start was over2s ago.
    super::assert_native_replay(&result);
}

#[test]
fn nanally_dodge_two_continuing_heavy_chains_preserve_packets_resources_real_time_and_replay() {
    let result = PreparedSourceScenario::new(
        include_str!("../../../../../presets/native/nanally-dodge-control.nte"),
        Default::default(),
        &Default::default(),
    )
    .unwrap()
    .run(42)
    .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    assert_eq!(result.report.hits_settled, 14);
    assert_eq!(
        result
            .report
            .hit_receipts
            .iter()
            .map(|hit| hit.time_us)
            .collect::<Vec<_>>(),
        [
            191054, 760520, 893587, 2445293, 2706842, 2936193, 3689517, 6191054, 6760520, 6893587,
            8445293, 8706842, 8936193, 9689517
        ]
    );
    assert_eq!(result.report.final_actor_states[0].granted_energy, 28);
    assert_eq!(
        result.report.final_actor_states[0].charge_remainder,
        Some(ExactFraction::new(239, 500).unwrap())
    );
    let cycle = result.report.final_actor_states[0].cycle.as_ref().unwrap();
    assert_eq!(cycle.current, 47);
    assert_eq!(cycle.carry, ExactFraction::new(201, 500).unwrap());
    let report = serde_json::to_value(&result.report).unwrap();
    assert_eq!(
        report["combat_stamina"]["total_spent"],
        serde_json::json!("30/1")
    );
    let mut request = result.request.clone();
    request.actors[0].static_stats.base_atk = 1000;
    request.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    request.actors[0].static_stats.elemental_bonus = Ratio::zero();
    request.actors[0].static_stats.crit_rate = Ratio::zero();
    request.stochastic_crit = None;
    let measured = run_game_scenario_report(&request, &Default::default()).unwrap();
    assert!(measured.success && measured.execution_completed);
    assert_eq!(
        formula_damage_total(&measured),
        ExactFraction::new(6329, 1).unwrap()
    );
    super::assert_native_replay(&result);
}

#[test]
fn nanally_dodge_heavy_resume_requires_new_real_hold_and_has_the_selected_idle_boundary() {
    for (at, count, effect) in [
        ("2.016", 4, "GE_Player_Nanally_Melee1_Damage"),
        ("3.516", 6, "GE_Player_Nanally_ExtraAtkIntro1_Damage"),
        ("3.516001", 4, "GE_Player_Nanally_Melee1_Damage"),
    ] {
        let hold = if at == "2.016" { "199999us" } else { "200ms" };
        let result = run(
            &format!(
                "nanally attack[hold=500ms]; wait_until(1.516); nanally dash;
            wait_until({at}); nanally attack[hold={hold}];"
            ),
            7,
        );
        assert_eq!(result.report.hits_settled, count);
        assert_eq!(result.report.hit_receipts[3].damage_effect_id, effect);
        if count == 6 {
            assert_eq!(result.report.hit_receipts[3].time_us, 3_945_293);
        }
    }
}

#[test]
fn nanally_dodge_resumes_third_heavy_without_replaying_previous_sections() {
    let result = run(
        "nanally attack[hold=1500ms]; nanally dash; nanally attack[hold=200ms];",
        6,
    );
    assert_eq!(result.report.hits_settled, 7);
    assert_eq!(
        result.report.hit_receipts[6].damage_effect_id,
        "GE_Player_Nanally_ExtraAtkOutro1_Damage"
    );
    assert_eq!(result.report.hit_receipts[6].time_us, 3_350_737);
    assert_eq!(result.report.final_actor_states[0].granted_energy, 14);
    assert_eq!(
        result.report.final_actor_states[0].charge_remainder,
        Some(ExactFraction::new(239, 1000).unwrap())
    );
    super::assert_native_replay(&result);
}

#[test]
fn nanally_dodge_noop_swap_preserves_resume_but_paid_critical_riposte_takes_priority() {
    let root = "nanally attack[hold=500ms]; wait_until(1.516);";
    let result = run(
        &format!("{root} nanally dash; nanally swap; nanally attack[hold=200ms];"),
        6,
    );
    assert_eq!(result.report.hits_settled, 6);
    assert_eq!(
        result.report.hit_receipts[3].damage_effect_id,
        "GE_Player_Nanally_ExtraAtkIntro1_Damage"
    );
    assert_eq!(result.report.hit_receipts[3].time_us, 2_445_293);
    let result = run(
        &format!("{root} nanally dash[critical=true]; nanally attack[hold=200ms];"),
        6,
    );
    assert_eq!(result.report.hits_settled, 4);
    assert_eq!(
        result.report.hit_receipts[3].damage_effect_id,
        "GE_Player_Nanally_PerfectEvadeAttack_Damage"
    );
    assert_eq!(result.report.hit_receipts[3].time_us, 2_283_562);
    super::assert_native_replay(&result);
    let result = run(
        &format!("{root} nanally dash; nanally attack; nanally attack[hold=200ms];"),
        6,
    );
    assert_eq!(result.report.hits_settled, 5);
    assert_eq!(
        result.report.hit_receipts[3].damage_effect_id,
        "GE_Player_Nanally_Melee1_Damage"
    );
    assert_eq!(
        result.report.hit_receipts[4].damage_effect_id,
        "GE_Player_Nanally_Melee2_Damage"
    );
}
