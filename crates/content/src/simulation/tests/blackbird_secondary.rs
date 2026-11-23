use super::*;
use crate::characters::blackbird::secondary::{PLUNGE, RIPOSTE};
use serde_json::{Value, json};

fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::blackbird_actions::request();
    req.actors[0].initial_energy = 0;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.relative_duration_us = 20_000_000;
    for id in [
        crate::HypothesisId::BlackbirdOrdinaryClockV1,
        crate::HypothesisId::BlackbirdOrdinaryMaliceV1,
        crate::HypothesisId::BlackbirdOrdinarySecondaryV1,
    ] {
        req.model_assumptions
            .insert(id.as_str().into(), id.description().into());
    }
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
fn normal(time: u64) -> GameActionCommand {
    let mut action = super::blackbird_actions::request().actions.remove(0);
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut action {
        *timestamp_us = time;
    }
    action
}
fn plunge(req: &mut GameScenarioRequest, time: u64) {
    let index = req.actions.len();
    let mut action = normal(time);
    if let GameActionCommand::InvokeAbility {
        damage_effect_id, ..
    } = &mut action
    {
        *damage_effect_id = PLUNGE.into();
    }
    req.actions.push(action);
    req.plunge_inputs.insert(
        index,
        crate::simulation::GamePlungeInput {
            landing_delay_us: 600_000,
            height_bonus: Ratio {
                numerator: 1,
                denominator: 2,
            },
        },
    );
}
pub(super) fn dash(time: u64, critical: bool) -> GameActionCommand {
    GameActionCommand::PlayerInput {
        timestamp_us: time,
        actor_index: 0,
        input: PlayerInput::Dash,
        critical_success: critical,
    }
}
pub(super) fn stamina(req: &mut GameScenarioRequest, initial: u64) {
    req.combat_stamina = Some(crate::simulation::CombatStaminaProfile {
        capacity: Ratio {
            numerator: 100,
            denominator: 1,
        },
        initial: Ratio {
            numerator: initial,
            denominator: 1,
        },
        recovery_rate: Ratio::zero(),
        use_efficiency: Ratio::one(),
    });
}
fn run(req: &GameScenarioRequest) -> Value {
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}

#[test]
fn blackbird_plunge_uses_explicit_airtime_height_and_one_resource_opportunity() {
    for targets in [1, 4] {
        let mut req = request(targets);
        req.actions.clear();
        plunge(&mut req, 0);
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let hits = report["hit_receipts"].as_array().unwrap();
        assert_eq!(hits.len(), targets);
        for hit in hits {
            assert_eq!(hit["time_us"], 653_784);
            // .476*1.5*1000*.5 =357, without multiplying a jump's velocity.
            assert_eq!(hit["non_crit_damage"], json!("357/1"));
            assert_eq!(
                support::hit_metadata(&report, hit)["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_X"
            );
        }
        let actor = &report["final_actor_states"][0];
        assert_eq!(actor["granted_energy"], 1);
        assert_eq!(actor["charge_remainder"], json!("17/25"));
        assert_eq!(actor["cycle"]["carry"], json!("3/10"));
        assert_eq!(
            actor["fractional_resources"]["blackbird.malice"]["current"],
            json!("21723077/1000000")
        );
        req.actions.push(normal(1_009_948));
        assert_eq!(run(&req)["execution_completed"], false);
        req.actions[1] = normal(1_009_949);
        assert_eq!(run(&req)["execution_completed"], true);
    }
}

#[test]
fn blackbird_plunge_requires_a_fresh_valid_profile_and_zero_hp_adds_no_pause_or_malice() {
    let mut req = request(1);
    req.actions.clear();
    plunge(&mut req, 0);
    req.plunge_inputs.clear();
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("explicit airborne")
    );
    req.actions.clear();
    plunge(&mut req, 0);
    req.plunge_inputs.get_mut(&0).unwrap().height_bonus = Ratio {
        numerator: 101,
        denominator: 100,
    };
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("height bonus0..1")
    );
    req.plunge_inputs.get_mut(&0).unwrap().height_bonus = Ratio::zero();
    req.targets[0].resistance = Ratio::one();
    req.actions.push(normal(881_249));
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(
        report["final_actor_states"][0]["fractional_resources"]["blackbird.malice"]["current"],
        json!("20/1")
    );
}

#[test]
fn blackbird_critical_dodge_pays_stamina_and_selects_exactly_one_native_riposte() {
    let mut req = request(3);
    stamina(&mut req, 100);
    req.actions = vec![dash(0, true), normal(1_004_760), normal(2_037_135)];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(
        report["combat_stamina"]["spends"].as_array().unwrap().len(),
        1
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    let ripostes: Vec<_> = hits
        .iter()
        .filter(|h| h["damage_effect_id"] == RIPOSTE)
        .collect();
    assert_eq!(ripostes.len(), 3);
    for hit in ripostes {
        assert_eq!(hit["time_us"], 1_287_038);
        assert_eq!(hit["non_crit_damage"], json!("1275/2"));
        let meta = support::hit_metadata(&report, hit);
        assert_eq!(meta["action"]["input_role"], "critical_riposte");
        assert_eq!(
            meta["binding"]["packet_ga_name"],
            "GA_BlackBird_PerfectEvade"
        );
    }
    assert_eq!(
        hits.iter()
            .filter(|h| h["damage_effect_id"] == "GE_Player_BlackBird_Melee1_Damage")
            .count(),
        3
    );
    // Entry20 +Riposte4.615385 +Normal1 .857919; no invented parry20 grant.
    assert_eq!(
        report["final_actor_states"][0]["fractional_resources"]["blackbird.malice"]["current"],
        json!("3184163/125000")
    );
    req.actions[2] = normal(2_037_134);
    assert_eq!(run(&req)["rejected_action"]["reason"]["kind"], "recovery");
}

#[test]
fn blackbird_ordinary_or_failed_dodge_and_intervening_actions_never_create_a_free_counter() {
    let mut req = request(1);
    stamina(&mut req, 100);
    req.actions = vec![dash(0, false), normal(1_004_760)];
    let report = run(&req);
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|h| h["damage_effect_id"] != RIPOSTE)
    );
    req.actions = vec![dash(0, true), normal(1_004_760)];
    stamina(&mut req, 14);
    let report = run(&req);
    assert_eq!(report["execution_completed"], false);
    assert_eq!(report["hits_settled"], 0);
    assert!(
        report["combat_stamina"]["spends"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    stamina(&mut req, 100);
    req.actions = vec![dash(0, true), dash(1_004_760, false), normal(2_009_520)];
    assert!(
        run(&req)["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|h| h["damage_effect_id"] != RIPOSTE)
    );
}

#[test]
fn blackbird_riposte_permission_clears_on_exit_and_pending_counter_cancels_on_death() {
    let mut req = request(1);
    stamina(&mut req, 100);
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1004".into();
    req.actors.push(ally);
    req.actions = vec![
        dash(0, true),
        super::reactions::switch(1_004_760, 1),
        super::reactions::switch(2_000_000, 0),
        normal(2_000_000),
    ];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|h| h["damage_effect_id"] != RIPOSTE)
    );
    req.actions = vec![dash(0, true), normal(1_004_760)];
    let id = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(id.as_str().into(), id.description().into());
    req.incoming_damage_events
        .push(crate::simulation::GameActorDamageEvent {
            timestamp_us: 1_100_000,
            actor_index: 0,
            delivered_damage: 1_000_000,
        });
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 0);
    assert_eq!(
        report["final_actor_states"][0]["fractional_resources"]["blackbird.malice"]["current"],
        json!("20/1")
    );
}
