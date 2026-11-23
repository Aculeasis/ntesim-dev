use super::*;
use serde_json::{Value, json};

fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::blackbird_actions::request();
    for id in [
        crate::HypothesisId::BlackbirdOrdinaryClockV1,
        crate::HypothesisId::BlackbirdOrdinaryMaliceV1,
        crate::HypothesisId::BlackbirdCoordinatedBirdV1,
        crate::HypothesisId::BlackbirdOrdinarySecondaryV1,
        crate::HypothesisId::BlackbirdQualifiedSupportV1,
        crate::HypothesisId::BlackbirdPrimaryFormV1,
    ] {
        req.model_assumptions
            .insert(id.as_str().into(), id.description().into());
    }
    req.actors[0].initial_energy = 100;
    req.actors[0].energy_capacity = 100;
    req.actors[0].purchased_skill_level = 8;
    req.actors[0].talent_levels = Some([1, 8, 1]);
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.relative_duration_us = 35_000_000;
    for target_index in 1..targets {
        let mut target = req.targets[0].clone();
        target.target_index = target_index;
        req.targets.push(target);
    }
    for target in &mut req.targets {
        target.hp_current = 100_000_000;
        target.hp_max = 100_000_000;
    }
    req
}

fn burst(time: u64, energy_cost: u64) -> GameActionCommand {
    let mut action = super::blackbird_actions::request().actions.remove(0);
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        ability_id,
        damage_effect_id,
        energy_cost: cost,
        cooldown_us,
        ..
    } = &mut action
    {
        *timestamp_us = time;
        *ability_id = "GA_BlackBird_UltraSkill".into();
        *damage_effect_id = "GE_Player_BlackBird_UltraSkill_Damage".into();
        *cost = energy_cost;
        *cooldown_us = 0;
    }
    action
}

fn normal(time: u64) -> GameActionCommand {
    let mut action = super::blackbird_actions::request().actions.remove(0);
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut action {
        *timestamp_us = time;
    }
    action
}

fn run(req: &GameScenarioRequest) -> Value {
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}

fn times(report: &Value) -> Vec<u64> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_BlackBird_UltraSkill_Damage")
        .map(|h| h["time_us"].as_u64().unwrap())
        .collect()
}

#[test]
fn blackbird_primary_form_timing_cost_cooldown_and_hitpause() {
    let mut req = request(1);
    req.actions = vec![burst(0, 100)];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );

    // 9 primary contacts use native progress with .067s/.3 owner pauses (46,900us pause per hit):
    assert_eq!(
        times(&report),
        [
            3_217_557, 3_505_236, 3_800_861, 4_290_042, 4_438_203, 4_584_444, 4_731_482, 4_878_010,
            5_025_131
        ]
    );

    // Energy: 100 spent, 0 remaining
    let actor = &report["final_actor_states"][0];
    assert_eq!(actor["spent_energy"], 100);
    assert_eq!(actor["energy"], 0);
    assert_eq!(actor["energy_conserved"], true);

    // Team ATK buff active while form is active (horizon 10s)
    let mut req_buff = request(1);
    req_buff.relative_duration_us = 10_000_000;
    req_buff.actions = vec![burst(0, 100)];
    let report_buff = run(&req_buff);
    assert_eq!(report_buff["execution_completed"], true);
    let instances = crate::simulation::tests::support::final_buffs(&report_buff);
    let team_buff = instances
        .iter()
        .find(|b| b["id"] == "blackbird.requies_animarum_team");
    assert!(team_buff.is_some());
    assert_eq!(team_buff.unwrap()["stat_modifiers"]["atk_percent"], "1/5");

    // After form expires at 21_500_000us, normal is admitted at or after 21_500_000us
    req.actions = vec![burst(0, 100), normal(21_500_000 + 5_540_254)];
    let report2 = run(&req);
    assert_eq!(report2["execution_completed"], true);
    assert_eq!(report2["hits_settled"], 10);
}

#[test]
fn blackbird_primary_form_awakenings_and_resonance() {
    // A2: Form DEF buff active during form (horizon 10s), removed after form ends (horizon 25s)
    let mut req = request(1);
    req.relative_duration_us = 10_000_000;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(vec![2]).unwrap();
    req.actions = vec![burst(0, 100)];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );

    let instances = crate::simulation::tests::support::final_buffs(&report);
    let def_buff = instances.iter().find(|b| b["id"] == "blackbird.form_def");
    assert!(def_buff.is_some());
    assert_eq!(def_buff.unwrap()["stat_modifiers"]["def_percent"], "1/5");

    // After form expires at 21.5s, def buff is removed
    req.relative_duration_us = 25_000_000;
    let report_expired = run(&req);
    let instances_expired = crate::simulation::tests::support::final_buffs(&report_expired);
    assert!(
        instances_expired
            .iter()
            .all(|b| b["id"] != "blackbird.form_def")
    );

    let form_ended =
        report_expired["trace"].as_array().unwrap().iter().any(|e| {
            e["event_kind"] == "blackbird_form_end" && e["time_us"] == 21_500_000 + 5_540_254
        });
    assert!(form_ended);

    // A4: Team buff duration is 25s, team damage bonus +12% (still active at 20s)
    let mut req_a4 = request(1);
    req_a4.relative_duration_us = 20_000_000;
    req_a4.actors[0].awakening = ntesim_data::AwakeningSelection::selected(vec![4]).unwrap();
    req_a4.actions = vec![burst(0, 100)];
    let report_a4 = run(&req_a4);
    assert_eq!(report_a4["execution_completed"], true);
    let instances_a4 = crate::simulation::tests::support::final_buffs(&report_a4);
    let a4_buff = instances_a4
        .iter()
        .find(|b| b["id"] == "blackbird.requies_animarum_team");
    assert!(a4_buff.is_some());
    assert_eq!(a4_buff.unwrap()["stat_modifiers"]["damage_bonus"], "3/25");

    // A6: Own Psyche damage bonus +25% for 25s applied at UltraSkill.3
    let mut req_a6 = request(1);
    req_a6.relative_duration_us = 10_000_000;
    req_a6.actors[0].awakening = ntesim_data::AwakeningSelection::selected(vec![6]).unwrap();
    req_a6.actions = vec![burst(0, 100)];
    let report_a6 = run(&req_a6);
    assert_eq!(report_a6["execution_completed"], true);
    let instances_a6 = crate::simulation::tests::support::final_buffs(&report_a6);
    let a6_buff = instances_a6
        .iter()
        .find(|b| b["id"] == "blackbird.a6_psyche_damage");
    assert!(a6_buff.is_some());
    assert_eq!(
        a6_buff.unwrap()["stat_modifiers"]["damage_bonus:ECharacterElementType::CHARACTER_ELEMENT_TYPE_PSYCHE"],
        "1/4"
    );

    // R3 (Awakening count >= 3): grants 20 Malice on commit, no ordinary gains during form
    let mut req_r3 = request(1);
    req_r3.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(3);
    req_r3.actions = vec![burst(0, 100)];
    let report_r3 = run(&req_r3);
    assert_eq!(report_r3["execution_completed"], true);
    // Malice: 20 battle entry + 20 R3 grant = 40. Hits during form do not grant malice.
    assert_eq!(
        report_r3["final_actor_states"][0]["fractional_resources"]["blackbird.malice"]["current"],
        json!("40/1")
    );
}

#[test]
fn blackbird_primary_form_off_field_pause_and_resume() {
    let mut req =
        super::reactions::request(&[(0, "character:1042", 100, 0), (1, "character:1004", 0, 0)]);
    for id in [
        crate::HypothesisId::BlackbirdOrdinaryClockV1,
        crate::HypothesisId::BlackbirdOrdinaryMaliceV1,
        crate::HypothesisId::BlackbirdCoordinatedBirdV1,
        crate::HypothesisId::BlackbirdOrdinarySecondaryV1,
        crate::HypothesisId::BlackbirdQualifiedSupportV1,
        crate::HypothesisId::BlackbirdPrimaryFormV1,
    ] {
        req.model_assumptions
            .insert(id.as_str().into(), id.description().into());
    }
    req.actors[0].energy_capacity = 100;
    req.actors[0].purchased_skill_level = 8;
    req.actors[0].talent_levels = Some([1, 8, 1]);
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.relative_duration_us = 40_000_000;

    // Burst at 0 (takes 5.54s recovery).
    // Switch to actor 1 at 6_000_000us (on-field 6s, 15.5s remaining).
    // Wait on actor 1 until 16_000_000us.
    // Switch back to Blackbird at 16_000_000us.
    // Form should resume and expire at 16_000_000 + 15_500_000 = 31_500_000us.
    req.actions = vec![
        burst(0, 100),
        super::reactions::switch(6_000_000, 1),
        super::reactions::switch(16_000_000, 0),
    ];

    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );

    let pauses: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "blackbird_form_pause")
        .collect();
    assert_eq!(pauses.len(), 1);
    assert_eq!(pauses[0]["time_us"], 6_000_000);
    assert_eq!(pauses[0]["data"]["duration_us"], 21_040_254);

    let resumes: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "blackbird_form_resume")
        .collect();
    assert_eq!(resumes.len(), 1);
    assert_eq!(resumes[0]["time_us"], 16_000_000);

    let ends: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "blackbird_form_end")
        .collect();
    assert_eq!(ends.len(), 1);
    assert_eq!(ends[0]["time_us"], 37_040_254);
}

#[test]
fn blackbird_primary_form_rejects_unsupported_evolved_inputs() {
    let mut req = request(1);
    // Trying normal attack while form is active (after recovery at 6s)
    req.actions = vec![burst(0, 100), normal(6_000_000)];
    let err = run_game_scenario(&req, &ExecutionLimits::default()).unwrap_err();
    assert!(err.contains("UNSUPPORTED_BLACKBIRD_EVOLVE_INPUT"));

    // Trying player input (dash) while form is active
    let mut req_dash = request(1);
    req_dash.actions = vec![
        burst(0, 100),
        GameActionCommand::PlayerInput {
            timestamp_us: 6_000_000,
            actor_index: 0,
            input: crate::simulation::PlayerInput::Dash,
            critical_success: false,
        },
    ];
    let err_dash = run_game_scenario(&req_dash, &ExecutionLimits::default()).unwrap_err();
    assert!(err_dash.contains("UNSUPPORTED_BLACKBIRD_EVOLVE_INPUT"));
}

#[test]
fn blackbird_primary_form_validation_and_energy_cost() {
    // Missing required assumption
    let mut req_missing = request(1);
    req_missing
        .model_assumptions
        .remove("blackbird.ordinary_clock_v1");
    let err = run_game_scenario(&req_missing, &ExecutionLimits::default()).unwrap_err();
    assert!(err.contains("Blackbird primary form requires ordinary clock"));

    // Energy cost != 100
    let mut req_cost = request(1);
    req_cost.actions = vec![burst(0, 50)];
    let err_cost = run_game_scenario(&req_cost, &ExecutionLimits::default()).unwrap_err();
    assert!(
        err_cost
            .contains("Blackbird transformation Burst requires its native100 Ultimate energy cost")
    );
}

#[test]
fn blackbird_primary_form_death_cleans_up_form_but_buffs_persist() {
    let mut req =
        super::reactions::request(&[(0, "character:1042", 100, 0), (1, "character:1004", 0, 0)]);
    for id in [
        crate::HypothesisId::BlackbirdOrdinaryClockV1,
        crate::HypothesisId::BlackbirdOrdinaryMaliceV1,
        crate::HypothesisId::BlackbirdCoordinatedBirdV1,
        crate::HypothesisId::BlackbirdOrdinarySecondaryV1,
        crate::HypothesisId::BlackbirdQualifiedSupportV1,
        crate::HypothesisId::BlackbirdPrimaryFormV1,
        crate::HypothesisId::CombatIncomingDamageFixtureV1,
    ] {
        req.model_assumptions
            .insert(id.as_str().into(), id.description().into());
    }
    req.actors[0].energy_capacity = 100;
    req.actors[0].purchased_skill_level = 8;
    req.actors[0].talent_levels = Some([1, 8, 1]);
    req.relative_duration_us = 10_000_000;

    req.actions = vec![burst(0, 100)];
    // Incoming fatal damage at 5_000_000us (after team buff applied at 3_166_927us)
    req.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
        timestamp_us: 5_000_000,
        actor_index: 0,
        delivered_damage: 100_000_000,
    }];

    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["final_actor_states"][0]["hp"], 0);

    // Form ended on death
    let ends: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "blackbird_form_end")
        .collect();
    assert_eq!(ends.len(), 1);
    assert_eq!(ends[0]["time_us"], 10_540_254);

    // Team buff persists on living teammate
    let instances = crate::simulation::tests::support::final_buffs(&report);
    let team_buff = instances
        .iter()
        .find(|b| b["id"] == "blackbird.requies_animarum_team");
    assert!(team_buff.is_some());
}
