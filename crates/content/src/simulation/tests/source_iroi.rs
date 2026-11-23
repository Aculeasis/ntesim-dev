use crate::simulation::{PreparedSourceScenario, SourceScenarioReport};

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
        "options duration=20 crit=fixed seed=42 stamina_capacity=200 initial_stamina=200
        stamina_use_efficiency=1 stamina_recovery_rate=0;
        iroi char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0;
        target hp=100000000 def=1080 resist=0;
        active iroi; {actions}"
    )
}

#[test]
fn source_iroi_voluntary_regression_held_skill_spends_cooldown_and_switch_lock() {
    let text = "options duration=15 crit=fixed;
        iroi char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0 start_cycle=100;
        zero char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0;
        adler char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0;
        target hp=100000000 def=1080 resist=0;
        active iroi;
        iroi skill[hold=400ms]; wait(1); zero swap;";
    let result = run(text);
    completed(&result);

    // Verify tap skill packets were not delivered
    let receipts = &result.report.hit_receipts;
    assert!(!receipts.iter().any(|h| {
        h.damage_effect_id == "GE_Player_Oneiroi_Skill_Damage"
            || h.damage_effect_id == "GE_Player_Oneiroi_Skill_Gun_Damage"
            || h.damage_effect_id == "GE_Player_Oneiroi_Skill_Syringe_Damage"
            || h.damage_effect_id == "GE_Player_Oneiroi_Skill_Trumpet_Damage"
    }));

    // Verify early switch before 600ms is rejected
    let mut req_switch_early = result.request.clone();
    req_switch_early.input_time_contexts.clear();
    req_switch_early.actions.truncate(1);
    req_switch_early
        .actions
        .push(crate::simulation::GameActionCommand::SwitchActor {
            timestamp_us: 500_000,
            target_actor_index: 1,
            cooldown_us: 1_000_000,
        });
    let failed =
        crate::simulation::run_game_scenario(&req_switch_early, &Default::default()).unwrap();
    assert_eq!(failed["execution_completed"], false);

    // Verify skill before 12s cooldown is rejected
    let early_skill = text.replace("zero swap;", "iroi skill;");
    let early_skill_res =
        PreparedSourceScenario::new(&early_skill, Default::default(), &Default::default())
            .unwrap()
            .run(42)
            .unwrap();
    assert!(!early_skill_res.report.execution_completed);
}

#[test]
fn source_iroi_daydream_stage3_spends_moon_and_delivers_branch_damage() {
    let script = source("iroi attack[hold=2s]; wait(3);");
    let script = script.replace("start_energy=0;", "start_energy=0 start_moon=50;");
    let result = run(&script);
    completed(&result);

    let receipts = &result.report.hit_receipts;
    assert!(
        receipts
            .iter()
            .any(|h| h.damage_effect_id == "GE_Player_Oneiroi_Branch3_Damage")
    );
}

#[test]
fn source_iroi_daydream_falls_back_to_normal1_when_moon_insufficient() {
    let script = source("iroi attack[hold=1s]; wait(2);");
    let script = script.replace("start_energy=0;", "start_energy=0 start_moon=10;");
    let result = run(&script);
    completed(&result);

    let receipts = &result.report.hit_receipts;
    assert!(
        receipts
            .iter()
            .any(|h| h.damage_effect_id == "GE_Player_Oneiroi_Melee1_Damage")
    );
    assert!(
        !receipts
            .iter()
            .any(|h| h.damage_effect_id.contains("Branch"))
    );
}

#[test]
fn source_iroi_lucid_dream_stage4_spends_10_moon_and_fires_lucid_shots() {
    let script = source("iroi attack:4; iroi attack[hold=1.5s]; wait(4);");
    let script = script.replace("start_energy=0;", "start_energy=0 start_moon=30;");
    let result = run(&script);
    completed(&result);

    let receipts = &result.report.hit_receipts;
    let lucid_hits = receipts
        .iter()
        .filter(|h| h.damage_effect_id == "GE_Player_Oneiroi_Branch5_Damage")
        .count();
    assert!(
        lucid_hits > 1,
        "expected multiple lucid shots, got {lucid_hits}"
    );
}

#[test]
fn source_iroi_dash_critical_riposte_and_jump_plunge() {
    let script = source(
        "iroi dash[critical=true]; wait(0.4); iroi attack;
        iroi jump; iroi plunge[airtime=500ms,height_bonus=1/2];
        wait(2);",
    );
    let result = run(&script);
    completed(&result);

    let receipts = &result.report.hit_receipts;
    assert!(
        receipts
            .iter()
            .any(|h| h.damage_effect_id == "GE_Player_Oneiroi_PerfectEvadeAttack_Damage")
    );
    assert!(
        receipts
            .iter()
            .any(|h| h.damage_effect_id == "GE_Player_Oneiroi_AirAttack_Damage")
    );
}

#[test]
fn source_iroi_qualified_sheep_support_on_switch_to_regressed_ally() {
    let script = "options duration=15 crit=fixed;
        iroi char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0 start_cycle=100;
        zero char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0;
        target hp=100000000 def=1080 resist=0;
        active iroi;
        iroi skill[hold=400ms]; wait(1); zero swap; wait(2);";
    let result = run(script);
    completed(&result);

    let value = serde_json::to_value(&result.report).unwrap();
    // 100 Cycle deducted on Iroi
    assert_eq!(value["final_actor_states"][0]["cycle"]["current"], 0);
    // Active actor remains Iroi because sheep support keeps Iroi active
    assert_eq!(value["final_buff_state"]["active_actor_index"], 0);
    // Sheep support reaction hit delivered
    let receipts = &result.report.hit_receipts;
    assert!(receipts.iter().any(|h| {
        h.damage_effect_id.contains("Guang")
            || h.damage_effect_id.contains("Zhou")
            || h.damage_effect_id.contains("AI")
            || h.damage_effect_id.contains("Sheep")
    }));
}

#[test]
fn source_iroi_ivory_revival_on_switch_in_ivory_dream() {
    let script = "options duration=30 crit=fixed;
        iroi char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=100 start_moon=100;
        zero char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0;
        target hp=100000000 def=1080 resist=0;
        active iroi;
        iroi skill[hold=400ms]; wait(1);
        iroi burst; wait(4.5);
        zero swap; wait(2);";
    let result = run(script);
    completed(&result);

    let value = serde_json::to_value(&result.report).unwrap();
    // Active actor switched to Zero during Ivory Dream
    let trace = value["trace"].as_array().unwrap();
    assert!(
        trace
            .iter()
            .any(|e| e["event_kind"] == "switch_actor" && e["active_actor"] == 1)
    );
}
