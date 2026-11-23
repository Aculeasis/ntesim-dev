use super::*;
use crate::characters::blackbird::support::PACKET;
use serde_json::{Value, json};

pub(super) fn request(targets: usize) -> GameScenarioRequest {
    let mut req =
        super::reactions::request(&[(0, "character:1004", 100, 0), (1, "character:1042", 0, 0)]);
    req.relative_duration_us = 20_000_000;
    for id in [
        crate::HypothesisId::BlackbirdOrdinaryClockV1,
        crate::HypothesisId::BlackbirdOrdinaryMaliceV1,
        crate::HypothesisId::BlackbirdQualifiedSupportV1,
    ] {
        req.model_assumptions
            .insert(id.as_str().into(), id.description().into());
    }
    for actor in &mut req.actors {
        actor.character_level = 80;
        actor.character_max_level = Some(80);
        actor.purchased_skill_level = 8;
        actor.talent_levels = Some([1, 8, 1]);
        actor.weapon_id.clear();
        actor.gear_procs.clear();
        actor.set_id = None;
        actor.initial_energy = 0;
        actor.static_stats.base_atk = 1000;
        actor.static_stats.atk = Ratio {
            numerator: 1000,
            denominator: 1,
        };
        actor.static_stats.crit_rate = Ratio::zero();
    }
    for target_index in 1..targets {
        let mut target = req.targets[0].clone();
        target.target_index = target_index;
        req.targets.push(target);
    }
    for target in &mut req.targets {
        target.hp_current = 100_000_000;
        target.hp_max = 100_000_000;
        target.level = 80;
        target.defense = 1080;
        target.resistance = Ratio::zero();
    }
    req.actions = vec![super::reactions::switch(0, 1)];
    req
}
fn run(req: &GameScenarioRequest) -> Value {
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}
fn qte(report: &Value) -> Vec<&Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["damage_effect_id"] == PACKET)
        .collect()
}
fn marker_times(report: &Value) -> Vec<u64> {
    report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "blackbird_qte_malice")
        .map(|e| e["time_us"].as_u64().unwrap())
        .collect()
}
fn normal(time: u64) -> GameActionCommand {
    let mut action = super::blackbird_actions::request().actions.remove(0);
    if let GameActionCommand::InvokeAbility {
        actor_index,
        timestamp_us,
        ..
    } = &mut action
    {
        *actor_index = 1;
        *timestamp_us = time;
    }
    action
}

#[test]
fn blackbird_support_uses_real_qualification_talent_aoe_and_independent_b4_marker() {
    for targets in [1, 4] {
        let req = request(targets);
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(qte(&report).len(), targets);
        for hit in qte(&report) {
            assert_eq!(hit["time_us"], 1_000_222);
            assert_eq!(hit["command_index"], 0);
            // Support talent8, not Normal1:3.428*1000*.5 =1714.
            assert_eq!(hit["non_crit_damage"], json!("1714/1"));
        }
        assert_eq!(marker_times(&report), [1_131_188]);
        assert_eq!(report["final_actor_states"][0]["cycle"]["spent"], 100);
        assert_eq!(
            report["final_actor_states"][1]["fractional_resources"]["blackbird.malice"]["current"],
            json!("40/1")
        );
        assert_eq!(report["final_actor_states"][1]["granted_energy"], 7);
        assert_eq!(
            report["final_actor_states"][1]["charge_remainder"],
            json!("14/25")
        );
    }
}

#[test]
fn blackbird_support_malice_marker_survives_miss_and_obeys_b4_selection() {
    let mut req = request(1);
    req.targets[0].resistance = Ratio::one();
    let report = run(&req);
    assert_eq!(marker_times(&report), [1_002_488]);
    assert_eq!(report["final_actor_states"][1]["granted_energy"], 0);
    assert_eq!(
        report["final_actor_states"][1]["fractional_resources"]["blackbird.malice"]["current"],
        json!("40/1")
    );
    req.actors[1].character_level = 20;
    req.actors[1].purchased_qte_level = Some(1);
    req.actors[1].character_max_level = Some(20);
    req.actors[1].purchased_skill_level = 1;
    req.actors[1].talent_levels = Some([1; 3]);
    let report = run(&req);
    assert!(marker_times(&report).is_empty());
    assert_eq!(
        report["final_actor_states"][1]["fractional_resources"]["blackbird.malice"]["current"],
        json!("20/1")
    );
}

#[test]
fn blackbird_support_attack_recovery_and_switch_lock_have_distinct_causal_boundaries() {
    let mut req = request(1);
    req.actions.push(normal(1_461_881));
    assert_eq!(run(&req)["execution_completed"], true);
    req.actions[1] = normal(1_461_880);
    assert_eq!(run(&req)["execution_completed"], false);
    req.actions[1] = super::reactions::switch(1_473_208, 0);
    let report = run(&req);
    assert_eq!(report["execution_completed"], false);
    assert_eq!(report["rejected_action"]["reason"]["until_us"], 1_473_209);
    req.actions[1] = super::reactions::switch(1_473_209, 0);
    assert_eq!(run(&req)["execution_completed"], true);
}

#[test]
fn blackbird_support_cannot_be_forged_or_refilled_by_an_unqualified_entry() {
    let mut req = request(1);
    req.actors[0].initial_cycle = 99;
    assert!(qte(&run(&req)).is_empty());
    req = request(1);
    req.active_actor_index = 1;
    let mut action = normal(0);
    if let GameActionCommand::InvokeAbility {
        ability_id,
        damage_effect_id,
        ..
    } = &mut action
    {
        *ability_id = "GA_BlackBird_QTE".into();
        *damage_effect_id = PACKET.into();
    }
    req.actions = vec![action];
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("engine-owned qualified entry")
    );
}

#[test]
fn blackbird_support_death_before_the_b4_marker_preserves_prior_hp_and_charge_only() {
    let mut req = request(1);
    let id = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(id.as_str().into(), id.description().into());
    req.incoming_damage_events
        .push(crate::simulation::GameActorDamageEvent {
            timestamp_us: 1_050_000,
            actor_index: 1,
            delivered_damage: 1_000_000,
        });
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(qte(&report).len(), 1);
    assert!(marker_times(&report).is_empty());
    assert_eq!(report["final_actor_states"][1]["granted_energy"], 7);
    assert_eq!(
        report["final_actor_states"][1]["fractional_resources"]["blackbird.malice"]["current"],
        json!("20/1")
    );
}
