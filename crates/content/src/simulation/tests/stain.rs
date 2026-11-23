use super::*;
use crate::HypothesisId;
use serde_json::{Value, json};

pub(super) fn fixture(times: &[u64], element: &str) -> GameScenarioRequest {
    let mut req = research_request();
    req.relative_duration_us = 24_000_000;
    req.stochastic_crit = None;
    req.actors[0].character_id = "character:1042".into();
    // This shared baseline intentionally has no breakthrough2 owner extension.
    req.actors[0].character_level = 1;
    req.actors[0].purchased_qte_level = Some(1);
    req.actors[0].character_max_level = Some(20);
    req.actors[0].initial_cycle = 100;
    req.actors[0].static_stats.cycle_intensity = Ratio {
        numerator: 108,
        denominator: 1,
    };
    let mut other = req.actors[0].clone();
    other.slot = 1;
    other.character_id = "character:1025".into();
    other.initial_cycle = 0;
    other.static_stats.cycle_intensity = Ratio {
        numerator: 999,
        denominator: 1,
    };
    req.actors.push(other);
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].hp_max = 100_000_000;
    let hit = req.actions[0].clone();
    let mut metadata = req.action_metadata[&0].clone();
    metadata.damage_type_ex = Some(format!("EExecutionDamageTypeEX::DAMAGE_TYPE_{element}"));
    req.actions = [0, 1]
        .into_iter()
        .enumerate()
        .map(|(index, time)| GameActionCommand::SwitchActor {
            timestamp_us: time,
            target_actor_index: 1 - index,
            cooldown_us: 0,
        })
        .collect();
    req.action_metadata.clear();
    for time in times {
        let mut action = hit.clone();
        if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut action {
            *timestamp_us = *time;
        }
        req.action_metadata
            .insert(req.actions.len(), metadata.clone());
        req.actions.push(action);
    }
    req
}

fn run(req: &GameScenarioRequest) -> Value {
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}

fn damages(report: &Value) -> Vec<Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|hit| hit["non_crit_damage"].clone())
        .collect()
}

#[test]
fn stain_uses_damage_actor_live_intensity_in_a_separate_zone_and_expires_exactly() {
    for element in ["PSYCHE", "LAKSHANA", "NATURE", "PSYCHICALLY"] {
        let mut req = fixture(&[2, 11_999_999, 12_000_000], element);
        req.actors[0].static_stats.damage_bonus = Ratio::one();
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        // E(108)=1.075; 1.2*E=1.29. The applier's CI999 is irrelevant.
        // Ordinary +100% damage bonus multiplies separately: 2000*1.29=2580.
        let boosted = match element {
            "PSYCHE" | "LAKSHANA" => 2580,
            _ => 2000,
        };
        assert_eq!(
            damages(&report),
            [boosted, boosted, 2000].map(|n| json!(format!("{}/{}", n, 1)))
        );
        assert_eq!(
            report["hit_receipts"].as_array().unwrap().len(),
            3,
            "Stain creates no own packet"
        );
        assert_eq!(
            report["resolved_model_assumptions"][HypothesisId::CombatStainSwitchV1.as_str()],
            HypothesisId::CombatStainSwitchV1.description()
        );
    }
}

#[test]
fn stain_live_twinflight_ci_and_devils_blood_target_bonus_compose_once() {
    let mut req = fixture(&[2], "PSYCHE");
    req.actors[0].static_stats.cycle_intensity = Ratio::zero();
    let source = "options duration=2; blackbird char lvl=80/80 awakening=0 talent=1,1,1 qte=1; blackbird add weapon=fork_twinbirds refine=3; target hp=1000000 def=0 resist=0; active blackbird; blackbird attack;";
    let compiled = crate::dsl::compiler::compile_scenario(source, Default::default()).unwrap();
    req.actors[0].weapon_id = compiled.request.actors[0].weapon_id.clone();
    req.actors[0].gear_procs = compiled.request.actors[0].gear_procs.clone();
    assert_eq!(
        damages(&run(&req)),
        [json!("1677/1")],
        "1000*(1+.30)*1.29 from live CI108"
    );
    let (proc, _) = crate::sets::devils_blood::projection::compile(
        ntesim_data::stat_projection::frozen_tables(),
    )
    .unwrap();
    req.actors[0].set_id = Some("set:Suit3".into());
    req.actors[0].gear_procs.push(proc);
    // Both set terms join the .30 weapon zone; Stain remains separate.
    assert_eq!(damages(&run(&req)), [json!("10707/5")]);
}

#[test]
fn stain_refresh_replaces_expiry_without_stacking_or_resetting_cycle() {
    let mut req = fixture(&[11_000_000, 12_000_000, 21_999_999, 22_000_000], "PSYCHE");
    req.actors[1].initial_cycle = 100;
    if let GameActionCommand::SwitchActor { timestamp_us, .. } = &mut req.actions[1] {
        *timestamp_us = 10_000_000;
    }
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 23_000_000,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    let report = run(&req);
    assert_eq!(
        damages(&report),
        [1290, 1290, 1290, 1000].map(|n| json!(format!("{}/{}", n, 1)))
    );
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["event_kind"] == "reaction_stain_applied")
            .count(),
        2
    );
}

#[test]
fn stain_survives_applier_death_but_target_death_cleans_the_status() {
    let mut req = fixture(&[2_000_000], "PSYCHE");
    req.relative_duration_us = 3_000_000;
    req.model_assumptions.insert(
        HypothesisId::CombatIncomingDamageFixtureV1.as_str().into(),
        HypothesisId::CombatIncomingDamageFixtureV1
            .description()
            .into(),
    );
    req.incoming_damage_events
        .push(crate::simulation::GameActorDamageEvent {
            timestamp_us: 1_000_000,
            actor_index: 1,
            delivered_damage: req.actors[1].initial_hp,
        });
    let report = run(&req);
    assert_eq!(report["final_actor_states"][1]["hp"], 0);
    assert_eq!(damages(&report), [json!("1290/1")]);
    req.targets[0].hp_current = 1;
    let report = run(&req);
    assert_eq!(report["final_target_states"][0]["hp_current"], 0);
    assert!(
        crate::simulation::tests::support::final_buffs(&report)
            .iter()
            .all(|b| b["id"] != "ntesim.reaction.stain")
    );
}
