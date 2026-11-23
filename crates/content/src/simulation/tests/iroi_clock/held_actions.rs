use super::*;
use crate::simulation::GameHoldInput;

fn request_with_moon(moon: u64, dummies: usize) -> GameScenarioRequest {
    let mut req = request(dummies);
    req.actors[0].initial_resources.moon = Some(Ratio {
        numerator: moon,
        denominator: 1,
    });
    let model = crate::characters::iroi::dream::HYPOTHESIS_ID;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req
}

#[test]
fn held_transform_voluntarily_regresses_allies_commits_cooldown_and_switch_lock() {
    let mut req = crate::dsl::compiler::compile_scenario(
        "options duration=15 crit=fixed;
        iroi char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0;
        mint char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0;
        adler char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0;
        target hp=100000000 def=1080 resist=0;
        active iroi;",
        Default::default(),
    )
    .unwrap()
    .request;

    let model = crate::characters::iroi::clock::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    let dream_model = crate::characters::iroi::dream::HYPOTHESIS_ID;
    req.model_assumptions
        .insert(dream_model.into(), dream_model.description().into());
    req.action_admission_order = GameActionAdmissionOrder::AfterSettledEvents;

    req.actions = vec![action(0, "GA_Oneiroi_Skill")];
    req.hold_inputs.insert(
        0,
        GameHoldInput {
            duration_us: 400_000,
            reserved_stocks: None,
            moving: false,
        },
    );

    let report = run(&req);
    // Tap packets should NOT be scheduled
    let receipts = report["hit_receipts"].as_array().unwrap();
    assert!(!receipts.iter().any(
        |h| h["damage_effect_id"] == "GE_Player_Oneiroi_Skill_Damage"
            || h["damage_effect_id"] == "GE_Player_Oneiroi_Skill_Gun_Damage"
            || h["damage_effect_id"] == "GE_Player_Oneiroi_Skill_Syringe_Damage"
            || h["damage_effect_id"] == "GE_Player_Oneiroi_Skill_Trumpet_Damage"
    ));
    // Trace should not contain tap heal
    let trace = report["trace"].as_array().unwrap();
    assert!(!trace.iter().any(|e| e["event_kind"] == "iroi_skill_heal"));

    // Switch before 600ms is rejected
    let mut req_switch_early = req.clone();
    req_switch_early
        .actions
        .push(GameActionCommand::SwitchActor {
            timestamp_us: 500_000,
            target_actor_index: 1,
            cooldown_us: 1_000_000,
        });
    let failed = run_game_scenario(&req_switch_early, &Default::default()).unwrap();
    assert_eq!(failed["execution_completed"], false);
    assert_eq!(failed["rejected_action"]["command_index"], 1);

    // Skill before 12s is rejected
    let mut req_skill_early = req.clone();
    req_skill_early
        .actions
        .push(action(11_999_999, "GA_Oneiroi_Skill"));
    let cd_failed = run_game_scenario(&req_skill_early, &Default::default()).unwrap();
    assert_eq!(cd_failed["execution_completed"], false);
}

#[test]
fn held_daydream_stage3_spends_moon_and_delivers_damage() {
    let mut req = request_with_moon(50, 1);
    req.actions = vec![action(0, "GA_Oneiroi_Melee")];
    // Natural full charge duration >= 1_767_000
    req.hold_inputs.insert(
        0,
        GameHoldInput {
            duration_us: 2_000_000,
            reserved_stocks: None,
            moving: false,
        },
    );

    let report = run(&req);
    let receipts = report["hit_receipts"].as_array().unwrap();
    // Should deliver Daydream Stage 3 hits (Trumpet / Branch3)
    assert!(
        receipts
            .iter()
            .any(|h| h["damage_effect_id"] == "GE_Player_Oneiroi_Branch3_Damage")
    );
}

#[test]
fn held_daydream_falls_back_to_normal1_when_moon_insufficient() {
    let mut req = request_with_moon(10, 1);
    req.actions = vec![action(0, "GA_Oneiroi_Melee")];
    // Hold for Daydream
    req.hold_inputs.insert(
        0,
        GameHoldInput {
            duration_us: 1_000_000,
            reserved_stocks: None,
            moving: false,
        },
    );

    let report = run(&req);
    let receipts = report["hit_receipts"].as_array().unwrap();
    // Should fall back to Melee1
    assert!(
        receipts
            .iter()
            .any(|h| h["damage_effect_id"] == "GE_Player_Oneiroi_Melee1_Damage")
    );
    // Should NOT deliver any Branch damage
    assert!(
        !receipts
            .iter()
            .any(|h| h["damage_effect_id"].as_str().unwrap().contains("Branch"))
    );
}

#[test]
fn held_lucid_dream_stage4_spends_10_moon_and_fires_shots() {
    let mut req = request_with_moon(30, 1);
    // First run 4 normal attacks to reach stage 4
    chain(&mut req, 0);
    // Replace 5th action with held Melee
    req.actions[4] = action(3239721, "GA_Oneiroi_Melee");
    req.hold_inputs.insert(
        4,
        GameHoldInput {
            duration_us: 1_500_000,
            reserved_stocks: None,
            moving: false,
        },
    );

    let report = run(&req);
    let receipts = report["hit_receipts"].as_array().unwrap();
    let lucid_hits = receipts
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_Oneiroi_Branch5_Damage")
        .count();
    // Lucid Dream fires multiple Branch5 shots
    assert!(
        lucid_hits > 1,
        "expected multiple lucid shots, got {lucid_hits}"
    );
}

#[test]
fn held_lucid_dream_falls_back_to_normal1_when_moon_insufficient() {
    let mut req = request_with_moon(5, 1);
    let mut act = action(0, "GA_Oneiroi_Melee");
    if let GameActionCommand::InvokeAbility {
        damage_effect_id, ..
    } = &mut act
    {
        *damage_effect_id = crate::characters::iroi::branch::BRANCH5.into();
    }
    req.actions = vec![act];
    req.hold_inputs.insert(
        0,
        GameHoldInput {
            duration_us: 1_000_000,
            reserved_stocks: None,
            moving: false,
        },
    );

    let report = run(&req);
    let receipts = report["hit_receipts"].as_array().unwrap();
    // 1st action fell back to Normal1
    let melee1_hits = receipts
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_Oneiroi_Melee1_Damage")
        .count();
    assert_eq!(melee1_hits, 1);
    assert!(
        !receipts
            .iter()
            .any(|h| h["damage_effect_id"] == crate::characters::iroi::branch::BRANCH5)
    );
}

#[test]
fn a2_assist_uniform_candidates_heal_team_and_deliver_damage() {
    let mut req = crate::dsl::compiler::compile_scenario(
        "options duration=15 crit=fixed;
        iroi char lvl=80/80 awakening=2 talent=8,8,8 qte=8 start_energy=0;
        mint char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0;
        target hp=100000000 def=1080 resist=0;
        active iroi;",
        Default::default(),
    )
    .unwrap()
    .request;

    let model = crate::characters::iroi::clock::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    let dream_model = crate::characters::iroi::dream::HYPOTHESIS_ID;
    req.model_assumptions
        .insert(dream_model.into(), dream_model.description().into());
    req.action_admission_order = GameActionAdmissionOrder::AfterSettledEvents;

    req.actions = vec![action(0, "GA_Oneiroi_Skill")];

    let report = run(&req);
    let trace = report["trace"].as_array().unwrap();
    // Trace should contain A2 assist event and A2 team heal
    assert!(trace.iter().any(|e| e["event_kind"] == "iroi_a2_assist"));
    assert!(trace.iter().any(|e| e["event_kind"] == "iroi_a2_heal"));

    // And assist damage packet should be settled
    let receipts = report["hit_receipts"].as_array().unwrap();
    assert!(receipts.iter().any(|h| {
        let id = h["damage_effect_id"].as_str().unwrap();
        id == "GE_Damage_sheep_gun_awaken2"
            || id == "GE_Damage_sheep_syringe_awaken2"
            || id == "GE_Damage_sheep_trumpet_awaken2"
    }));
}

#[test]
fn qualified_sheep_support_requires_100_cycle_and_delivers_reaction() {
    let mut req = crate::dsl::compiler::compile_scenario(
        "options duration=15 crit=fixed;
        iroi char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0 start_cycle=100;
        zero char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0;
        target hp=100000000 def=1080 resist=0;
        active iroi;",
        Default::default(),
    )
    .unwrap()
    .request;

    let model = crate::characters::iroi::clock::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    let dream_model = crate::characters::iroi::dream::HYPOTHESIS_ID;
    req.model_assumptions
        .insert(dream_model.into(), dream_model.description().into());
    req.action_admission_order = GameActionAdmissionOrder::AfterSettledEvents;

    // 1. Voluntarily regress Zero via held Transform
    req.actions = vec![
        action(0, "GA_Oneiroi_Skill"),
        GameActionCommand::SwitchActor {
            timestamp_us: 1_000_000,
            target_actor_index: 1,
            cooldown_us: 1_000_000,
        },
    ];
    req.hold_inputs.insert(
        0,
        GameHoldInput {
            duration_us: 400_000,
            reserved_stocks: None,
            moving: false,
        },
    );

    let report = run(&req);
    // Cycle deducted from 100 to 0 on Iroi
    let final_states = report["final_actor_states"].as_array().unwrap();
    assert_eq!(final_states[0]["cycle"]["current"], 0);

    // Active actor remains Iroi (index 0) because sheep support leaves Iroi active
    assert_eq!(report["final_buff_state"]["active_actor_index"], 0);

    // Qualified sheep support packet was delivered
    let receipts = report["hit_receipts"].as_array().unwrap();
    assert!(receipts.iter().any(|h| {
        let id = h["damage_effect_id"].as_str().unwrap();
        id.contains("Guang") || id.contains("Zhou") || id.contains("AI") || id.contains("Sheep")
    }));
}

#[test]
fn ivory_revival_restores_10_percent_hp_and_re_regresses_on_dream_end() {
    let mut req = crate::dsl::compiler::compile_scenario(
        "options duration=30 crit=fixed;
        iroi char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=100 start_moon=100;
        zero char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0;
        target hp=100000000 def=1080 resist=0;
        active iroi;",
        Default::default(),
    )
    .unwrap()
    .request;

    let model = crate::characters::iroi::clock::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    let dream_model = crate::characters::iroi::dream::HYPOTHESIS_ID;
    req.model_assumptions
        .insert(dream_model.into(), dream_model.description().into());
    req.action_admission_order = GameActionAdmissionOrder::AfterSettledEvents;

    // Regress Zero first, then Burst to enter Ivory Dream, then switch to Zero during Ivory Dream
    req.actions = vec![
        action(0, "GA_Oneiroi_Skill"),
        action(1_000_000, "GA_Oneiroi_UltraSkill"),
        GameActionCommand::SwitchActor {
            timestamp_us: 5_500_000,
            target_actor_index: 1,
            cooldown_us: 1_000_000,
        },
    ];
    req.hold_inputs.insert(
        0,
        GameHoldInput {
            duration_us: 400_000,
            reserved_stocks: None,
            moving: false,
        },
    );

    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );

    // Zero was revived and became active at 5.5s
    let trace = report["trace"].as_array().unwrap();
    assert!(
        trace
            .iter()
            .any(|e| { e["event_kind"] == "switch_actor" && e["active_actor"] == 1 })
    );

    // Final actor states: dream has ended (duration 30s), so Zero is regressed again with retained HP (10% of MaxHP)
    let final_states = report["final_actor_states"].as_array().unwrap();
    assert_eq!(final_states[1]["hp"], 5367);
}
